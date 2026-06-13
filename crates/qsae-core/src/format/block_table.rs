use crate::error::Result;
use crate::format::CodecId;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub const BLOCK_ENTRY_SIZE: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockEntry {
    pub codec_id: u8,
    pub original_len: u32,
    pub compressed_len: u32,
}

impl BlockEntry {
    pub fn new(codec_id: CodecId, original_len: u32, compressed_len: u32) -> Self {
        Self {
            codec_id: codec_id.as_u8(),
            original_len,
            compressed_len,
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u8(self.codec_id)?;
        writer.write_u32::<LittleEndian>(self.original_len)?;
        writer.write_u32::<LittleEndian>(self.compressed_len)?;
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let codec_id = reader.read_u8()?;
        let original_len = reader.read_u32::<LittleEndian>()?;
        let compressed_len = reader.read_u32::<LittleEndian>()?;
        Ok(Self {
            codec_id,
            original_len,
            compressed_len,
        })
    }
}

pub struct BlockTable {
    pub entries: Vec<BlockEntry>,
}

impl BlockTable {
    pub fn new(entries: Vec<BlockEntry>) -> Self {
        Self { entries }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        for entry in &self.entries {
            entry.write(writer)?;
        }
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R, count: u32) -> Result<Self> {
        let mut entries = Vec::with_capacity(count as usize);
        for _ in 0..count {
            entries.push(BlockEntry::read(reader)?);
        }
        Ok(Self { entries })
    }

    pub fn total_compressed_size(&self) -> u64 {
        self.entries.iter().map(|e| e.compressed_len as u64).sum()
    }
}
