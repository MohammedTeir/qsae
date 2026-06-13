use crate::codecs::codec_by_id;
use crate::error::{QsaeError, Result};
use crate::format::block_table::BlockTable;
use crate::format::footer::Footer;
use crate::format::header::QsaeHeader;
use crate::format::switch_map::SwitchMap;
use xxhash_rust::xxh64::xxh64;
use std::io::Read;

/// Decompressor for .qsae files.
pub struct Decompressor;

impl Decompressor {
    pub fn new() -> Self {
        Self
    }

    /// Decompress bytes in memory.
    pub fn decompress(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.len() < 32 + 16 {
            return Err(QsaeError::Format("Input too small for valid .qsae file".to_string()));
        }

        let mut cursor = std::io::Cursor::new(input);

        // 1. Read header
        let header = QsaeHeader::read(&mut cursor)?;

        // 2. Read block table
        let block_table = BlockTable::read(&mut cursor, header.block_count)?;

        // 3. Read switch map (Phase 2: arithmetic-coded)
        let switch_map = SwitchMap::read(&mut cursor)?;
        let flat_assignments = switch_map.to_flat();

        // 4. Read and decompress each block
        let mut output = Vec::with_capacity(header.original_size as usize);

        for (i, entry) in block_table.entries.iter().enumerate() {
            let codec_id = if i < flat_assignments.len() {
                flat_assignments[i]
            } else {
                return Err(QsaeError::Format(format!(
                    "Block {} has no codec assignment in switch map", i
                )))
            };

            let mut payload = vec![0u8; entry.compressed_len as usize];
            cursor.read_exact(&mut payload)?;

            let codec = codec_by_id(codec_id)?;
            let decompressed = codec.decompress(&payload, entry.original_len as usize)?;

            if decompressed.len() != entry.original_len as usize {
                return Err(QsaeError::Decompression(format!(
                    "Block {} decompressed to {} bytes, expected {}",
                    i, decompressed.len(), entry.original_len
                )));
            }

            output.extend_from_slice(&decompressed);
        }

        // 5. Read footer and verify checksum
        let footer = Footer::read(&mut cursor)?;
        let computed_hash = xxh64(&output, 0);

        if computed_hash != footer.xxhash64 {
            return Err(QsaeError::ChecksumMismatch {
                computed: computed_hash,
                expected: footer.xxhash64,
            });
        }

        output.truncate(header.original_size as usize);
        Ok(output)
    }

    /// Decompress a .qsae file to output path.
    pub fn decompress_file(&self, input_path: &str, output_path: &str) -> Result<usize> {
        let input = std::fs::read(input_path)?;
        let output = self.decompress(&input)?;
        std::fs::write(output_path, &output)?;
        Ok(output.len())
    }

    /// Inspect a .qsae file without decompressing (Phase 2: enhanced).
    pub fn inspect(&self, input: &[u8]) -> Result<FileInfo> {
        let mut cursor = std::io::Cursor::new(input);

        let header = QsaeHeader::read(&mut cursor)?;
        let block_table = BlockTable::read(&mut cursor, header.block_count)?;
        let switch_map = SwitchMap::read(&mut cursor)?;
        let flat_assignments = switch_map.to_flat();

        let mut codec_counts = std::collections::HashMap::new();
        for &id in &flat_assignments {
            *codec_counts.entry(id).or_insert(0) += 1;
        }

        let mut codec_breakdown: Vec<(String, usize, f64)> = codec_counts.iter()
            .map(|(&id, &count)| {
                let name = crate::dispatcher::Dispatcher::codec_name(id);
                let pct = (count as f64 / flat_assignments.len() as f64) * 100.0;
                (name.to_string(), count, pct)
            })
            .collect();
        codec_breakdown.sort_by(|a, b| b.1.cmp(&a.1));

        let total_compressed = block_table.total_compressed_size();
        let overhead = input.len() as u64 - total_compressed;

        // Phase 2: Calculate switch map overhead
        let switch_map_overhead = switch_map.overhead_ratio();

        // Per-block codec info
        let block_info: Vec<BlockInfo> = block_table.entries.iter().enumerate()
            .map(|(i, entry)| {
                let codec_id = if i < flat_assignments.len() {
                    flat_assignments[i]
                } else {
                    0xFF
                };
                BlockInfo {
                    index: i,
                    codec_id,
                    codec_name: crate::dispatcher::Dispatcher::codec_name(codec_id).to_string(),
                    original_len: entry.original_len,
                    compressed_len: entry.compressed_len,
                    ratio: entry.original_len as f64 / entry.compressed_len.max(1) as f64,
                }
            })
            .collect();

        Ok(FileInfo {
            version: header.version,
            block_count: header.block_count as usize,
            original_size: header.original_size,
            compressed_size: input.len() as u64,
            ratio: header.original_size as f64 / input.len() as u64 as f64,
            codec_breakdown,
            overhead_bytes: overhead,
            switch_map_overhead_ratio: switch_map_overhead,
            block_info,
        })
    }
}

/// Information about a .qsae file (Phase 2: enhanced).
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileInfo {
    pub version: u8,
    pub block_count: usize,
    pub original_size: u64,
    pub compressed_size: u64,
    pub ratio: f64,
    pub codec_breakdown: Vec<(String, usize, f64)>,
    pub overhead_bytes: u64,
    /// Phase 2: Switch map compression efficiency
    pub switch_map_overhead_ratio: f64,
    /// Phase 2: Per-block details
    pub block_info: Vec<BlockInfo>,
}

/// Per-block information for detailed inspection.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BlockInfo {
    pub index: usize,
    pub codec_id: u8,
    pub codec_name: String,
    pub original_len: u32,
    pub compressed_len: u32,
    pub ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compressor::{Compressor, CompressorConfig};

    #[test]
    fn test_decompress_roundtrip() {
        let compressor = Compressor::new(CompressorConfig::default());
        let decompressor = Decompressor::new();

        let data = b"Hello, world! This is QSAE roundtrip test. ".repeat(500);
        let compressed = compressor.compress(&data[..]).unwrap();
        let decompressed = decompressor.decompress(&compressed).unwrap();

        assert_eq!(&data[..], &decompressed[..]);
    }

    #[test]
    fn test_decompress_rle_data() {
        let compressor = Compressor::new(CompressorConfig::default());
        let decompressor = Decompressor::new();

        let data = vec![0x42; 100000];
        let compressed = compressor.compress(&data).unwrap();
        let decompressed = decompressor.decompress(&compressed).unwrap();

        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_inspect_phase2() {
        let compressor = Compressor::new(CompressorConfig::default());
        let decompressor = Decompressor::new();

        let data = b"Test data for Phase 2 inspection. ".repeat(100);
        let compressed = compressor.compress(&data[..]).unwrap();
        let info = decompressor.inspect(&compressed).unwrap();

        assert_eq!(info.version, 1);
        assert!(info.block_count > 0);
        assert_eq!(info.original_size, data.len() as u64);
        assert!(info.ratio > 0.0);
        assert!(!info.codec_breakdown.is_empty());

        // Phase 2 fields
        assert!(info.switch_map_overhead_ratio >= 0.0);
        assert!(!info.block_info.is_empty());
        assert_eq!(info.block_info.len(), info.block_count);

        // Verify block info structure
        let first_block = &info.block_info[0];
        assert_eq!(first_block.index, 0);
        assert!(!first_block.codec_name.is_empty());
        assert!(first_block.original_len > 0);
    }

    #[test]
    fn test_checksum_mismatch() {
        let compressor = Compressor::new(CompressorConfig::default());
        let decompressor = Decompressor::new();

        let data = b"Checksum test. ".repeat(100);
        let mut compressed = compressor.compress(&data[..]).unwrap();

        // Corrupt the last few bytes (specifically the xxhash64 checksum in the footer)
        let len = compressed.len();
        compressed[len - 12] ^= 0xFF;

        let result = decompressor.decompress(&compressed);
        assert!(matches!(result, Err(QsaeError::ChecksumMismatch { .. })));
    }

    #[test]
    fn test_mixed_data_roundtrip() {
        let compressor = Compressor::new(CompressorConfig::default());
        let decompressor = Decompressor::new();

        let mut data = vec![0x00; 30000];
        data.extend(vec![0xFF; 30000]);
        for i in 0..30000 {
            data.push(((i * 17 + 31) % 256) as u8);
        }

        let compressed = compressor.compress(&data).unwrap();
        let decompressed = decompressor.decompress(&compressed).unwrap();

        assert_eq!(data, decompressed);

        // Inspect should show multiple codecs used
        let info = decompressor.inspect(&compressed).unwrap();
        assert!(info.codec_breakdown.len() > 1, "Mixed data should use multiple codecs");
    }
}
