use crate::error::Result;
use crate::codecs::Codec;
use flate2::{Compression, write::{DeflateEncoder, DeflateDecoder}};
use std::io::Write;

/// DEFLATE codec — fallback for moderate-to-high entropy data.
/// Uses flate2's rust backend for pure-Rust implementation.
pub struct DeflateCodec;

impl Codec for DeflateCodec {
    fn id(&self) -> u8 { 0x08 }
    fn name(&self) -> &'static str { "DEFLATE" }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress(&self, data: &[u8], _original_len: usize) -> Result<Vec<u8>> {
        let mut decoder = DeflateDecoder::new(Vec::new());
        decoder.write_all(data)?;
        Ok(decoder.finish()?)
    }

    fn suitable_for(&self, entropy: f64) -> bool {
        entropy >= 6.0 && entropy < 7.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deflate_roundtrip() {
        let codec = DeflateCodec;
        let data = b"Hello, world! This is a test of DEFLATE in QSAE. ".repeat(100);
        let compressed = codec.compress(&data[..]).unwrap();
        let decompressed = codec.decompress(&compressed, data.len()).unwrap();
        assert_eq!(&data[..], &decompressed[..]);
    }
}
