use crate::error::{QsaeError, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub const MAGIC: [u8; 4] = *b"QSAE";
pub const MAGIC_END: [u8; 4] = *b"EASQ";
pub const HEADER_SIZE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QsaeHeader {
    pub version: u8,
    pub flags: u8,
    pub block_count: u32,
    pub original_size: u64,
    pub map_offset: u64,
}

impl QsaeHeader {
    pub fn new(block_count: u32, original_size: u64, map_offset: u64) -> Self {
        Self {
            version: 1,
            flags: 0,
            block_count,
            original_size,
            map_offset,
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&MAGIC)?;
        writer.write_u8(self.version)?;
        writer.write_u8(self.flags)?;
        writer.write_u32::<LittleEndian>(self.block_count)?;
        writer.write_u64::<LittleEndian>(self.original_size)?;
        writer.write_u64::<LittleEndian>(self.map_offset)?;
        writer.write_all(&[0u8; 6])?; // reserved
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if magic != MAGIC {
            return Err(QsaeError::InvalidMagic(magic));
        }

        let version = reader.read_u8()?;
        if version != 1 {
            return Err(QsaeError::UnsupportedVersion(version));
        }

        let flags = reader.read_u8()?;
        let block_count = reader.read_u32::<LittleEndian>()?;
        let original_size = reader.read_u64::<LittleEndian>()?;
        let map_offset = reader.read_u64::<LittleEndian>()?;

        let mut reserved = [0u8; 6];
        reader.read_exact(&mut reserved)?;

        Ok(Self {
            version,
            flags,
            block_count,
            original_size,
            map_offset,
        })
    }
}
