use crate::error::Result;
use crate::codecs::Codec;
use lz4_flex::block::{compress_prepend_size, decompress_size_prepended};

/// LZ4 codec — fast compression for structured data.
/// Uses lz4_flex block format with size prepended.
pub struct Lz4Codec;

impl Codec for Lz4Codec {
    fn id(&self) -> u8 { 0x02 }
    fn name(&self) -> &'static str { "LZ4" }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let compressed = compress_prepend_size(data);
        Ok(compressed)
    }

    fn decompress(&self, data: &[u8], _original_len: usize) -> Result<Vec<u8>> {
        let decompressed = decompress_size_prepended(data)
            .map_err(|e| crate::error::QsaeError::Decompression(format!("LZ4: {:?}", e)))?;
        Ok(decompressed)
    }

    fn suitable_for(&self, entropy: f64) -> bool {
        entropy >= 1.0 && entropy < 5.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz4_roundtrip() {
        let codec = Lz4Codec;
        let data = b"Hello, world! This is a test of the LZ4 codec in QSAE. ".repeat(100);
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);
    }

    #[test]
    fn test_lz4_repetitive() {
        let codec = Lz4Codec;
        let data = b"AAAAAAAAAABBBBBBBBBBCCCCCCCCCC".repeat(1000);
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);
        assert!(compressed.len() < data.len() / 2);
    }
}
