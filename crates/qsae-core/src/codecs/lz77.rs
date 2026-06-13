use crate::error::Result;
use crate::codecs::Codec;

/// LZ77 codec with custom sliding window implementation.
/// 
/// Format: [window_size: u16][literal_count: u32][literals...][(offset: u16, length: u16)...]
/// 
/// Uses a hash-based search for matching strings in the sliding window.
/// Window size: up to 256KB (configurable, default 32KB for balance).
/// Match length: 3-258 bytes (LZ77 standard range).
/// 
/// Phase 3: Full custom implementation replacing DEFLATE fallback.
pub struct Lz77Codec {
    window_size: usize,
    min_match_len: usize,
    max_match_len: usize,
}

impl Default for Lz77Codec {
    fn default() -> Self {
        Self {
            window_size: 32768,    // 32KB default window
            min_match_len: 3,     // Minimum match length
            max_match_len: 258,   // Maximum match length (LZ77 standard)
        }
    }
}

impl Lz77Codec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size.clamp(1024, 262144); // 1KB to 256KB
        self
    }

    /// Find the longest match for data starting at `pos` within the sliding window.
    /// Returns (offset, length) or None if no match found.
    fn find_match(
        &self,
        data: &[u8],
        pos: usize,
        head: &[i32; 65536],
        prev: &[i32],
    ) -> Option<(u16, u16)> {
        let remaining = data.len() - pos;
        if remaining < self.min_match_len {
            return None;
        }

        // Hash of the 3-byte sequence at pos
        let h = (((data[pos] as usize) << 10) ^ ((data[pos + 1] as usize) << 5) ^ (data[pos + 2] as usize)) & 0xFFFF;

        let mut curr = head[h];
        let window_start = pos.saturating_sub(self.window_size);

        let mut best_offset = 0u16;
        let mut best_len = 0u16;

        let mut chain_len = 0;
        const MAX_CHAIN_LEN: usize = 128; // Limit traversal to prevent worst-case slowdowns

        while curr != -1 && chain_len < MAX_CHAIN_LEN {
            let match_pos = curr as usize;
            if match_pos < window_start {
                break; // Out of sliding window
            }

            // Quick check: only examine if it can improve on best_len
            let current_best = best_len as usize;
            if pos + current_best < data.len() && data[match_pos + current_best] == data[pos + current_best] {
                // Check prefix
                let mut match_len = 0;
                let available = remaining.min(self.max_match_len);
                while match_len < available && data[match_pos + match_len] == data[pos + match_len] {
                    match_len += 1;
                }

                if match_len >= self.min_match_len && match_len > current_best {
                    best_len = match_len as u16;
                    best_offset = (pos - match_pos) as u16;

                    if match_len >= self.max_match_len {
                        break; // Found max match
                    }
                }
            }

            curr = prev[match_pos];
            chain_len += 1;
        }

        if best_len >= self.min_match_len as u16 {
            Some((best_offset, best_len))
        } else {
            None
        }
    }

    /// Compress using LZ77 algorithm.
    /// Output format: [window_size: u16][data...]
    /// Where data is interleaved literals and (offset, length) pairs.
    /// A special marker distinguishes literals from references.
    fn lz77_compress(&self, data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }

        let mut output = Vec::with_capacity(data.len());

        // Write window size
        output.extend_from_slice(&(self.window_size as u16).to_le_bytes());

        let mut head = [-1i32; 65536];
        let mut prev = vec![-1i32; data.len()];

        let mut pos = 0;
        let mut literals = Vec::new();

        while pos < data.len() {
            let found_match = self.find_match(data, pos, &head, &prev);

            if let Some((offset, length)) = found_match {
                // Flush pending literals first
                if !literals.is_empty() {
                    output.push(0x00); // Literal marker
                    output.extend_from_slice(&(literals.len() as u16).to_le_bytes());
                    output.extend_from_slice(&literals);
                    literals.clear();
                }

                // Write back-reference
                output.push(0x01); // Reference marker
                output.extend_from_slice(&offset.to_le_bytes());
                output.extend_from_slice(&length.to_le_bytes());

                // Insert positions in match
                let len = length as usize;
                for i in 0..len {
                    let p = pos + i;
                    if p + 3 <= data.len() {
                        let h = (((data[p] as usize) << 10) ^ ((data[p + 1] as usize) << 5) ^ (data[p + 2] as usize)) & 0xFFFF;
                        prev[p] = head[h];
                        head[h] = p as i32;
                    }
                }
                pos += len;
            } else {
                // Accumulate literal
                literals.push(data[pos]);

                // Insert current position
                if pos + 3 <= data.len() {
                    let h = (((data[pos] as usize) << 10) ^ ((data[pos + 1] as usize) << 5) ^ (data[pos + 2] as usize)) & 0xFFFF;
                    prev[pos] = head[h];
                    head[h] = pos as i32;
                }
                pos += 1;

                // Flush if literals buffer is full
                if literals.len() >= 65535 {
                    output.push(0x00); // Literal marker
                    output.extend_from_slice(&(literals.len() as u16).to_le_bytes());
                    output.extend_from_slice(&literals);
                    literals.clear();
                }
            }
        }

        // Flush remaining literals
        if !literals.is_empty() {
            output.push(0x00); // Literal marker
            output.extend_from_slice(&(literals.len() as u16).to_le_bytes());
            output.extend_from_slice(&literals);
        }

        output
    }

    /// Decompress LZ77 data.
    fn lz77_decompress(&self, data: &[u8], original_len: usize) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut output = Vec::with_capacity(original_len);
        let mut pos = 2; // Skip window size header

        while pos < data.len() && output.len() < original_len {
            let marker = data[pos];
            pos += 1;

            if marker == 0x00 {
                // Literal block
                if pos + 2 > data.len() {
                    break;
                }
                let len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2;

                if pos + len > data.len() {
                    return Err(crate::error::QsaeError::Decompression(
                        "Literal block exceeds data bounds".to_string()
                    ));
                }

                output.extend_from_slice(&data[pos..pos + len]);
                pos += len;
            } else if marker == 0x01 {
                // Back-reference
                if pos + 4 > data.len() {
                    break;
                }
                let offset = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
                let length = u16::from_le_bytes([data[pos + 2], data[pos + 3]]) as usize;
                pos += 4;

                if offset > output.len() {
                    return Err(crate::error::QsaeError::Decompression(
                        format!("Invalid back-reference offset: {} > {}", offset, output.len())
                    ));
                }

                // Copy from output buffer (handles overlapping matches)
                let start = output.len() - offset;
                for i in 0..length {
                    if output.len() >= original_len {
                        break;
                    }
                    output.push(output[start + (i % offset)]);
                }
            } else {
                return Err(crate::error::QsaeError::Decompression(
                    format!("Unknown marker byte: 0x{:02X}", marker)
                ));
            }
        }

        output.truncate(original_len);
        Ok(output)
    }
}

impl Codec for Lz77Codec {
    fn id(&self) -> u8 { 0x03 }
    fn name(&self) -> &'static str { "LZ77" }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(self.lz77_compress(data))
    }

    fn decompress(&self, data: &[u8], original_len: usize) -> Result<Vec<u8>> {
        self.lz77_decompress(data, original_len)
    }

    fn suitable_for(&self, entropy: f64) -> bool {
        entropy >= 1.0 && entropy < 5.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz77_roundtrip() {
        let codec = Lz77Codec::new();
        let data = b"Hello, world! This is a test of the LZ77 codec in QSAE. ".repeat(100);
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);
    }

    #[test]
    fn test_lz77_repetitive() {
        let codec = Lz77Codec::new();
        let data = b"AAAAAAAAAABBBBBBBBBBCCCCCCCCCC".repeat(1000);
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);

        // Should compress well due to repetition
        assert!(compressed.len() < data.len() / 2, 
            "Expected compression, got {} -> {}", data.len(), compressed.len());
    }

    #[test]
    fn test_lz77_window_size() {
        let codec = Lz77Codec::new().with_window_size(4096);
        let data = b"Repeat this pattern. ".repeat(500);
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);
    }

    #[test]
    fn test_lz77_empty() {
        let codec = Lz77Codec::new();
        let data: Vec<u8> = Vec::new();
        let compressed = codec.compress(&data).unwrap();
        assert!(compressed.is_empty());
        let decompressed = codec.decompress(&compressed, 0).unwrap();
        assert!(decompressed.is_empty());
    }

    #[test]
    fn test_lz77_large_window() {
        let codec = Lz77Codec::new().with_window_size(65536);
        let mut data = vec![0u8; 10000];
        data.extend(vec![0x42; 50000]); // Large repeated section

        let compressed = codec.compress(&data).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_lz77_overlapping_matches() {
        let codec = Lz77Codec::new();
        // Data that creates overlapping back-references (like "aaaaaa...")
        let data = vec![0x61; 10000]; // 'a' repeated

        let compressed = codec.compress(&data).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);

        // Should achieve excellent compression
        assert!(compressed.len() < 250, 
            "Highly repetitive data should compress to <250 bytes, got {}", compressed.len());
    }
}
