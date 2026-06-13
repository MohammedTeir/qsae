use crate::error::Result;
use crate::codecs::Codec;

/// Run-Length Encoding codec.
/// Encodes consecutive identical bytes as (count, value) pairs.
/// Format: [count: u16][value: u8]...
/// For runs > 65535, splits into multiple entries.
pub struct RleCodec;

impl Codec for RleCodec {
    fn id(&self) -> u8 { 0x01 }
    fn name(&self) -> &'static str { "RLE" }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut output = Vec::with_capacity(data.len() / 2);
        let mut current = data[0];
        let mut count = 1u16;

        for &byte in &data[1..] {
            if byte == current && count < u16::MAX {
                count += 1;
            } else {
                output.extend_from_slice(&count.to_le_bytes());
                output.push(current);
                current = byte;
                count = 1;
            }
        }

        // Flush final run
        output.extend_from_slice(&count.to_le_bytes());
        output.push(current);

        Ok(output)
    }

    fn decompress(&self, data: &[u8], original_len: usize) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut output = Vec::with_capacity(original_len);
        let mut i = 0;

        while i < data.len() {
            if i + 2 > data.len() {
                break;
            }
            let count = u16::from_le_bytes([data[i], data[i + 1]]) as usize;
            let value = data[i + 2];
            output.extend(std::iter::repeat(value).take(count));
            i += 3;
        }

        output.truncate(original_len);
        Ok(output)
    }

    fn suitable_for(&self, entropy: f64) -> bool {
        entropy < 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_roundtrip() {
        let codec = RleCodec;
        let data = vec![0xAA; 1000];
        let compressed = codec.compress(&data).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_rle_mixed() {
        let codec = RleCodec;
        let mut data = Vec::new();
        data.extend(vec![0x00; 100]);
        data.extend(vec![0xFF; 50]);
        data.extend(vec![0x00; 200]);

        let compressed = codec.compress(&data).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_rle_long_run() {
        let codec = RleCodec;
        let data = vec![0x42; 100000];
        let compressed = codec.compress(&data).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);
        assert!(compressed.len() < 1000); // Should be tiny
    }

    #[test]
    fn test_rle_empty() {
        let codec = RleCodec;
        let data: Vec<u8> = Vec::new();
        let compressed = codec.compress(&data).unwrap();
        assert!(compressed.is_empty());
    }
}
