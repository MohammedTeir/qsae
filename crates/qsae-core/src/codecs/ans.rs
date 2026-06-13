use crate::error::Result;
use crate::codecs::Codec;
use ans::{encode, decode, FrequencyTable};

/// ANS (Asymmetric Numeral Systems) codec.
/// 
/// Uses rANS (range variant) for near-theoretical compression.
/// rANS combines compression ratio of arithmetic coding with speed of Huffman.
/// 
/// Phase 3: Full rANS implementation using the `ans` crate.
/// Encodes symbols in reverse order (rANS requirement), decodes forward.
pub struct AnsCodec;

impl AnsCodec {
    /// Map byte value to symbol index (skipping zero-count symbols).
    fn byte_to_symbol(byte: u8, counts: &[u32; 256]) -> u32 {
        let mut symbol = 0u32;
        for i in 0..byte as usize {
            if counts[i] > 0 {
                symbol += 1;
            }
        }
        symbol
    }

    /// Map symbol index back to byte value.
    fn symbol_to_byte(symbol: u32, counts: &[u32; 256]) -> u8 {
        let mut current = 0u32;
        for i in 0..256 {
            if counts[i] > 0 {
                if current == symbol {
                    return i as u8;
                }
                current += 1;
            }
        }
        0 // Fallback (shouldn't happen with valid data)
    }
}

impl Codec for AnsCodec {
    fn id(&self) -> u8 { 0x05 }
    fn name(&self) -> &'static str { "ANS" }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        if data.len() < 50 {
            // Too small for ANS overhead to be worth it
            let mut output = vec![0x00]; // Marker: raw data
            output.extend_from_slice(&(data.len() as u32).to_le_bytes());
            output.extend_from_slice(data);
            return Ok(output);
        }

        // Build frequency table
        let mut counts = [0u32; 256];
        for &byte in data {
            counts[byte as usize] += 1;
        }

        let non_zero: Vec<u32> = counts.iter().copied().filter(|&c| c > 0).collect();
        let table = FrequencyTable::from_counts(&non_zero, 12)
            .map_err(|e| crate::error::QsaeError::Compression(format!("ANS table: {:?}", e)))?;

        // Convert bytes to symbol indices
        let symbols: Vec<u32> = data.iter()
            .map(|&byte| Self::byte_to_symbol(byte, &counts))
            .collect();

        // Encode with rANS
        let encoded = encode(&symbols, &table)
            .map_err(|e| crate::error::QsaeError::Compression(format!("ANS encode: {:?}", e)))?;

        // Serialize: [marker: 1][symbol_count: u16][counts...][encoded_len: u32][encoded...]
        let mut output = Vec::new();
        output.push(0x01); // Marker: ANS compressed

        // Write symbol count and which bytes are present
        let present_symbols: Vec<u8> = (0..256).filter(|&i| counts[i] > 0).map(|i| i as u8).collect();
        output.extend_from_slice(&(present_symbols.len() as u16).to_le_bytes());
        for &sym in &present_symbols {
            output.push(sym);
            output.extend_from_slice(&counts[sym as usize].to_le_bytes());
        }

        output.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        output.extend_from_slice(&encoded);

        Ok(output)
    }

    fn decompress(&self, data: &[u8], original_len: usize) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let marker = data[0];

        if marker == 0x00 {
            // Raw data
            let len = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
            return Ok(data[5..5 + len].to_vec());
        }

        // ANS compressed
        let mut pos = 1;
        let symbol_count = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;

        // Rebuild counts table
        let mut counts = [0u32; 256];
        let mut present_symbols = Vec::with_capacity(symbol_count);
        for _ in 0..symbol_count {
            let sym = data[pos];
            pos += 1;
            let count = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
            pos += 4;
            counts[sym as usize] = count;
            present_symbols.push(sym);
        }

        let encoded_len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        let encoded = &data[pos..pos + encoded_len];

        // Rebuild frequency table
        let non_zero: Vec<u32> = counts.iter().copied().filter(|&c| c > 0).collect();
        let table = FrequencyTable::from_counts(&non_zero, 12)
            .map_err(|e| crate::error::QsaeError::Decompression(format!("ANS table: {:?}", e)))?;

        // Decode symbols
        let symbols = decode(encoded, &table, original_len)
            .map_err(|e| crate::error::QsaeError::Decompression(format!("ANS decode: {:?}", e)))?;

        // Convert symbols back to bytes
        let output: Vec<u8> = symbols.iter()
            .map(|&sym| Self::symbol_to_byte(sym, &counts))
            .collect();

        Ok(output)
    }

    fn suitable_for(&self, entropy: f64) -> bool {
        entropy >= 6.0 && entropy < 7.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ans_roundtrip() {
        let codec = AnsCodec;
        let data = b"Hello, world! ANS test data with various bytes. ".repeat(100);
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);
    }

    #[test]
    fn test_ans_high_entropy() {
        let codec = AnsCodec;
        // High entropy data (near-random but with some structure)
        let mut data = Vec::new();
        for i in 0..10000 {
            data.push(((i * 7 + 13) % 256) as u8);
        }

        let compressed = codec.compress(&data).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_ans_small_data() {
        let codec = AnsCodec;
        // Small data should pass through
        let data = b"tiny";
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);
    }

    #[test]
    fn test_ans_skewed_distribution() {
        let codec = AnsCodec;
        // Highly skewed (ideal for ANS)
        let mut data = Vec::new();
        for _ in 0..5000 { data.push(0x00); }
        for _ in 0..3000 { data.push(0xFF); }
        for _ in 0..1500 { data.push(0x42); }
        for _ in 0..500 { data.push(0xAB); }

        let compressed = codec.compress(&data).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);

        // Should compress well due to skewed distribution
        assert!(compressed.len() < data.len() / 2,
            "ANS should compress skewed data, got {} -> {}", data.len(), compressed.len());
    }

    #[test]
    fn test_ans_empty() {
        let codec = AnsCodec;
        let data: Vec<u8> = Vec::new();
        let compressed = codec.compress(&data).unwrap();
        assert!(compressed.is_empty());
    }
}
