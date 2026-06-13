use crate::error::Result;
use crate::codecs::Codec;

/// Skip/Store codec — pass-through for incompressible data.
/// Prevents QSAE from bloating already-compressed or encrypted content.
pub struct SkipCodec;

impl Codec for SkipCodec {
    fn id(&self) -> u8 { 0x00 }
    fn name(&self) -> &'static str { "Skip" }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    fn decompress(&self, data: &[u8], _original_len: usize) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    fn suitable_for(&self, entropy: f64) -> bool {
        entropy >= 7.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_roundtrip() {
        let codec = SkipCodec;
        let data = vec![0xAB; 1000];
        let compressed = codec.compress(&data).unwrap();
        assert_eq!(data, compressed);
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(data, decompressed);
    }
}
