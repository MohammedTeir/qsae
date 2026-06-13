pub mod ans;
pub mod bwt;
pub mod delta;
pub mod deflate;
pub mod huffman;
pub mod lz4;
pub mod lz77;
pub mod rle;
pub mod skip;

use crate::error::Result;

/// Trait for all compression codecs.
pub trait Codec: Send + Sync {
    /// Unique identifier for this codec.
    fn id(&self) -> u8;

    /// Compress data.
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Decompress data.
    fn decompress(&self, data: &[u8], original_len: usize) -> Result<Vec<u8>>;

    /// Estimate if this codec is suitable for given entropy level.
    fn suitable_for(&self, entropy: f64) -> bool;

    /// Human-readable name.
    fn name(&self) -> &'static str;
}

/// Factory to create codec instances by ID.
pub fn codec_by_id(id: u8) -> Result<Box<dyn Codec>> {
    match id {
        0x00 => Ok(Box::new(skip::SkipCodec)),
        0x01 => Ok(Box::new(rle::RleCodec)),
        0x02 => Ok(Box::new(lz4::Lz4Codec)),
        0x03 => Ok(Box::new(lz77::Lz77Codec::new())),      // Phase 3: Custom implementation
        0x04 => Ok(Box::new(huffman::HuffmanCodec)),
        0x05 => Ok(Box::new(ans::AnsCodec)),
        0x06 => Ok(Box::new(bwt::BwtCodec)),
        0x07 => Ok(Box::new(delta::DeltaCodec)),
        0x08 => Ok(Box::new(deflate::DeflateCodec)),
        _ => Err(crate::error::QsaeError::CodecUnavailable(id)),
    }
}

/// Select optimal codec based on entropy value.
/// Phase 3: Full codec pool with proper routing.
pub fn select_codec(entropy: f64) -> Box<dyn Codec> {
    select_codec_for_data(entropy, &[])
}

/// Select optimal codec based on entropy and raw block data.
///
/// When `data` is available, probes for numeric patterns first:
/// if the block looks like sequential integers, timestamps, or sensor
/// readings, DeltaCodec is returned regardless of entropy (it handles
/// the fallback internally for non-numeric data).
///
/// Phase 3: Full codec pool with data-aware routing.
pub fn select_codec_for_data(entropy: f64, data: &[u8]) -> Box<dyn Codec> {
    if !data.is_empty() && delta::DeltaCodec::is_numeric(data) {
        return Box::new(delta::DeltaCodec);
    }

    if entropy < 1.0 {
        Box::new(rle::RleCodec)
    } else if entropy < 3.5 {
        Box::new(lz4::Lz4Codec)
    } else if entropy < 5.0 {
        Box::new(lz77::Lz77Codec::new())
    } else if entropy < 6.0 {
        Box::new(huffman::HuffmanCodec)
    } else if entropy < 6.5 {
        Box::new(bwt::BwtCodec)
    } else if entropy < 7.5 {
        Box::new(ans::AnsCodec)
    } else {
        Box::new(skip::SkipCodec)  // ← must reach here for exe blocks
    }
}
