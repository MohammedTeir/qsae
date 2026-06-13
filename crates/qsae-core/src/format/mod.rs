pub mod block_table;
pub mod footer;
pub mod header;
pub mod switch_map;

use crate::error::{QsaeError, Result};

/// Codec IDs as per specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CodecId {
    Skip = 0x00,
    Rle = 0x01,
    Lz4 = 0x02,
    Lz77 = 0x03,
    Huffman = 0x04,
    Ans = 0x05,
    Bwt = 0x06,
    Delta = 0x07,
    Deflate = 0x08,
}

impl CodecId {
    pub fn from_u8(id: u8) -> Result<Self> {
        match id {
            0x00 => Ok(CodecId::Skip),
            0x01 => Ok(CodecId::Rle),
            0x02 => Ok(CodecId::Lz4),
            0x03 => Ok(CodecId::Lz77),
            0x04 => Ok(CodecId::Huffman),
            0x05 => Ok(CodecId::Ans),
            0x06 => Ok(CodecId::Bwt),
            0x07 => Ok(CodecId::Delta),
            0x08 => Ok(CodecId::Deflate),
            _ => Err(QsaeError::CodecUnavailable(id)),
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}
