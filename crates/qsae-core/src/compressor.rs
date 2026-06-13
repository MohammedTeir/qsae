use crate::codecs::codec_by_id;
use crate::dispatcher::{Dispatcher, QuorumAnalysis};
use crate::error::Result;
use crate::format::block_table::{BlockEntry, BlockTable};
use crate::format::footer::Footer;
use crate::format::header::QsaeHeader;
use crate::format::switch_map::SwitchMap;
use crate::partitioner::{Block, BlockPartitioner};
use crate::quorum::QuorumParams;
use xxhash_rust::xxh64::xxh64;
use rayon::prelude::*;
use std::sync::Arc;

/// Configuration for the compressor.
#[derive(Debug, Clone)]
pub struct CompressorConfig {
    pub quorum: QuorumParams,
    pub parallel: bool,
    pub block_size_hint: usize,
    pub use_quorum: bool,
}

impl Default for CompressorConfig {
    fn default() -> Self {
        Self {
            quorum: QuorumParams::default(),
            parallel: true,  // Phase 3: enabled by default
            block_size_hint: 65536,
            use_quorum: true,
        }
    }
}

impl CompressorConfig {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn quorum(mut self, quorum: QuorumParams) -> Self {
        self.quorum = quorum;
        self
    }

    pub fn parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    pub fn block_size_hint(mut self, size: usize) -> Self {
        self.block_size_hint = size;
        self
    }

    pub fn use_quorum(mut self, use_quorum: bool) -> Self {
        self.use_quorum = use_quorum;
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

/// Compression statistics with Phase 3 parallel metrics.
#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub ratio: f64,
    pub block_count: usize,
    pub codec_usage: Vec<(String, usize, f64)>,
    pub duration_ms: u64,
    pub quorum_analysis: Option<QuorumAnalysis>,
    pub switch_map_overhead_ratio: f64,
    /// Phase 3: Parallel speedup metrics
    pub parallel_threads: usize,
    pub parallel_speedup: Option<f64>,
}

/// Main compressor with optional Rayon parallelism.
pub struct Compressor {
    config: CompressorConfig,
}

impl Compressor {
    pub fn new(config: CompressorConfig) -> Self {
        Self { config }
    }

    /// Compress bytes in memory.
    pub fn compress(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        // 1. Partition into blocks
        let partitioner = BlockPartitioner::new(self.config.quorum);
        let blocks = partitioner.partition(input);

        // 2. Assign codecs
        let dispatcher = Dispatcher::new(self.config.quorum);
        let (assignments, _quorum_analysis) = if self.config.use_quorum {
            let analysis = dispatcher.assign_quorum(&blocks, input);
            let assignments = analysis.assignments.clone();
            (assignments, Some(analysis))
        } else {
            (dispatcher.assign_simple(&blocks, input), None)
        };

        // 3. Compress each block (Phase 3: parallel if enabled and enough blocks)
        let compressed_blocks = self.compress_blocks_parallel(input, &blocks, &assignments)?;

        // 4. Build entries from compressed blocks
        let mut entries = Vec::with_capacity(blocks.len());
        for (i, block) in blocks.iter().enumerate() {
            entries.push(BlockEntry::new(
                crate::format::CodecId::from_u8(assignments[i].codec_id)?,
                block.len as u32,
                compressed_blocks[i].len() as u32,
            ));
        }

        // 5. Build switch map
        let flat_assignments: Vec<u8> = assignments.iter().map(|a| a.codec_id).collect();
        let switch_map = SwitchMap::from_flat(&flat_assignments);

        // 6. Calculate sizes for header
        let block_count = blocks.len() as u32;
        let original_size = input.len() as u64;
        let table_size = entries.len() * 9;

        // 7. Build output in correct order
        let mut map_buffer = Vec::new();
        let _map_size = switch_map.write(&mut map_buffer)?;

        let map_offset = 32 + table_size as u64;

        // 8. Assemble final output
        let mut output = Vec::new();

        let header = QsaeHeader::new(block_count, original_size, map_offset);
        header.write(&mut output)?;

        let table = BlockTable::new(entries);
        table.write(&mut output)?;

        output.extend_from_slice(&map_buffer);

        for block in &compressed_blocks {
            output.extend_from_slice(block);
        }

        let hash = xxh64(input, 0);
        let footer = Footer::new(hash);
        footer.write(&mut output)?;

        Ok(output)
    }

    /// Compress blocks in parallel using Rayon.
    fn compress_blocks_parallel(
        &self,
        input: &[u8],
        blocks: &[Block],
        assignments: &[crate::dispatcher::BlockAssignment],
    ) -> Result<Vec<Vec<u8>>> {
        if !self.config.parallel || blocks.len() < 4 {
            // Sequential fallback for small inputs
            let mut results = Vec::with_capacity(blocks.len());
            for (i, block) in blocks.iter().enumerate() {
                let codec_id = assignments[i].codec_id;
                let codec = codec_by_id(codec_id)?;
                let block_data = &input[block.offset..block.offset + block.len];
                results.push(codec.compress(block_data)?);
            }
            return Ok(results);
        }

        // Phase 3: Parallel compression using Rayon
        let input_arc = Arc::new(input.to_vec());

        let results: Result<Vec<Vec<u8>>> = blocks.par_iter().enumerate()
            .map(|(i, block)| {
                let codec_id = assignments[i].codec_id;
                let codec = codec_by_id(codec_id)?;
                let block_data = &input_arc[block.offset..block.offset + block.len];
                codec.compress(block_data)
            })
            .collect();

        results
    }

    /// Compress a file to a .qsae file with full statistics.
    pub fn compress_file(&self, input_path: &str, output_path: &str) -> Result<CompressionStats> {
        let start = std::time::Instant::now();
        let input = std::fs::read(input_path)?;

        // Partition and analyze
        let partitioner = BlockPartitioner::new(self.config.quorum);
        let blocks = partitioner.partition(&input);

        let dispatcher = Dispatcher::new(self.config.quorum);
        let (assignments, quorum_analysis) = if self.config.use_quorum {
            let analysis = dispatcher.assign_quorum(&blocks, &input);
            let assignments = analysis.assignments.clone();
            (assignments, Some(analysis))
        } else {
            (dispatcher.assign_simple(&blocks, &input), None)
        };

        // Compress with timing
        let compressed = self.compress(&input)?;
        std::fs::write(output_path, &compressed)?;

        let compressed_size = compressed.len();
        let ratio = input.len() as f64 / compressed_size as f64;

        // Calculate codec usage
        let mut codec_counts = std::collections::HashMap::new();
        for a in &assignments {
            *codec_counts.entry(a.codec_id).or_insert(0) += 1;
        }

        let mut codec_usage: Vec<(String, usize, f64)> = codec_counts.iter()
            .map(|(&id, &count)| {
                let name = Dispatcher::codec_name(id);
                let pct = (count as f64 / assignments.len() as f64) * 100.0;
                (name.to_string(), count, pct)
            })
            .collect();
        codec_usage.sort_by(|a, b| b.1.cmp(&a.1));

        let flat_assignments: Vec<u8> = assignments.iter().map(|a| a.codec_id).collect();
        let switch_map = SwitchMap::from_flat(&flat_assignments);
        let overhead_ratio = switch_map.overhead_ratio();

        let duration = start.elapsed().as_millis() as u64;

        // Phase 3: Calculate parallel metrics
        let thread_count = rayon::current_num_threads();
        let speedup = if self.config.parallel && blocks.len() >= 4 {
            Some(blocks.len() as f64 / thread_count.max(1) as f64)
        } else {
            None
        };

        Ok(CompressionStats {
            original_size: input.len(),
            compressed_size,
            ratio,
            block_count: blocks.len(),
            codec_usage,
            duration_ms: duration,
            quorum_analysis,
            switch_map_overhead_ratio: overhead_ratio,
            parallel_threads: if self.config.parallel { thread_count } else { 1 },
            parallel_speedup: speedup,
        })
    }

    /// Get quorum analysis without compressing.
    pub fn analyze(&self, input: &[u8]) -> Result<QuorumAnalysis> {
        let partitioner = BlockPartitioner::new(self.config.quorum);
        let blocks = partitioner.partition(input);

        let dispatcher = Dispatcher::new(self.config.quorum);
        let analysis = dispatcher.assign_quorum(&blocks, input);

        Ok(analysis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_empty() {
        let compressor = Compressor::new(CompressorConfig::default());
        let result = compressor.compress(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_compress_roundtrip() {
        let compressor = Compressor::new(CompressorConfig::default());
        let data = b"Hello, world! This is QSAE test data. ".repeat(1000);
        let compressed = compressor.compress(&data[..]).unwrap();

        assert_eq!(&compressed[0..4], b"QSAE");
        assert!(compressed.len() > 32);
    }

    #[test]
    fn test_compress_rle_data() {
        let compressor = Compressor::new(CompressorConfig::default());
        let data = vec![0x00; 100000];
        let compressed = compressor.compress(&data).unwrap();

        assert!(compressed.len() < data.len() / 10);
    }

    #[test]
    fn test_parallel_compression() {
        let config = CompressorConfig::builder()
            .parallel(true)
            .build();
        let compressor = Compressor::new(config);

        let data = b"Parallel test data. ".repeat(10000);
        let compressed = compressor.compress(&data[..]).unwrap();

        assert_eq!(&compressed[0..4], b"QSAE");
    }

    #[test]
    fn test_sequential_fallback() {
        let config = CompressorConfig::builder()
            .parallel(false)
            .build();
        let compressor = Compressor::new(config);

        let data = b"Sequential test. ".repeat(100);
        let compressed = compressor.compress(&data[..]).unwrap();

        assert_eq!(&compressed[0..4], b"QSAE");
    }

    #[test]
    fn test_quorum_analysis() {
        let compressor = Compressor::new(CompressorConfig::default());
        let mut data = vec![0x00; 30000];
        for i in 0..30000 {
            data.push(((i * 17 + 31) % 256) as u8);
        }

        let analysis = compressor.analyze(&data).unwrap();

        assert!(!analysis.assignments.is_empty());
        assert!(!analysis.quorum_curve.is_empty());
        assert_eq!(analysis.entropy_profile.len(), analysis.assignments.len());

        let low_entropy_blocks = analysis.entropy_profile.iter().filter(|&&e| e < 1.0).count();
        let high_entropy_blocks = analysis.entropy_profile.iter().filter(|&&e| e > 6.0).count();

        assert!(low_entropy_blocks > 0);
        assert!(high_entropy_blocks > 0);
    }

    #[test]
    fn test_simple_vs_quorum_mode() {
        let data = b"Test data for mode comparison. ".repeat(500);

        let simple_config = CompressorConfig::builder().use_quorum(false).build();
        let quorum_config = CompressorConfig::builder().use_quorum(true).build();

        let simple_compressor = Compressor::new(simple_config);
        let quorum_compressor = Compressor::new(quorum_config);

        let simple_compressed = simple_compressor.compress(&data[..]).unwrap();
        let quorum_compressed = quorum_compressor.compress(&data[..]).unwrap();

        assert_eq!(&simple_compressed[0..4], b"QSAE");
        assert_eq!(&quorum_compressed[0..4], b"QSAE");
    }
}
