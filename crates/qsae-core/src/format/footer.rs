use crate::error::{QsaeError, Result};
use crate::format::header::MAGIC_END;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub const FOOTER_SIZE: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Footer {
    pub xxhash64: u64,
}

impl Footer {
    pub fn new(xxhash64: u64) -> Self {
        Self { xxhash64 }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u64::<LittleEndian>(self.xxhash64)?;
        writer.write_all(&MAGIC_END)?;
        writer.write_all(&[0u8; 4])?;
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let xxhash64 = reader.read_u64::<LittleEndian>()?;
        let mut magic_end = [0u8; 4];
        reader.read_exact(&mut magic_end)?;
        if magic_end != MAGIC_END {
            return Err(QsaeError::Format("Invalid end magic".to_string()));
        }
        let mut padding = [0u8; 4];
        reader.read_exact(&mut padding)?;
        Ok(Self { xxhash64 })
    }
}
