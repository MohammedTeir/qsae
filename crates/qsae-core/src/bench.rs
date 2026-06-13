use crate::{Compressor, CompressorConfig, QuorumParams};
use std::time::Instant;

/// Benchmark result for a single compressor.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub original_size: usize,
    pub compressed_size: usize,
    pub ratio: f64,
    pub compress_time_ms: u64,
    pub decompress_time_ms: u64,
    pub compress_speed_mbps: f64,
    pub decompress_speed_mbps: f64,
}

/// Benchmark suite comparing QSAE against other compressors.
pub struct BenchmarkSuite;

impl BenchmarkSuite {
    pub fn new() -> Self {
        Self
    }

    /// Run QSAE benchmark with given configuration.
    pub fn benchmark_qsae(&self, data: &[u8], config: CompressorConfig) -> BenchmarkResult {
        let original_size = data.len();

        // Compression
        let compress_start = Instant::now();
        let compressor = Compressor::new(config);
        let compressed = compressor.compress(data).unwrap();
        let compress_time = compress_start.elapsed().as_millis() as u64;

        // Decompression
        let decompress_start = Instant::now();
        let decompressor = crate::Decompressor::new();
        let _decompressed = decompressor.decompress(&compressed).unwrap();
        let decompress_time = decompress_start.elapsed().as_millis() as u64;

        let compressed_size = compressed.len();
        let ratio = original_size as f64 / compressed_size as f64;

        BenchmarkResult {
            name: "QSAE".to_string(),
            original_size,
            compressed_size,
            ratio,
            compress_time_ms: compress_time,
            decompress_time_ms: decompress_time,
            compress_speed_mbps: (original_size as f64 / 1024.0 / 1024.0) / (compress_time as f64 / 1000.0).max(0.001),
            decompress_speed_mbps: (original_size as f64 / 1024.0 / 1024.0) / (decompress_time as f64 / 1000.0).max(0.001),
        }
    }

    /// Run QSAE with different quorum parameters.
    pub fn benchmark_qsae_variants(&self, data: &[u8]) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();

        // Default parameters
        results.push(self.benchmark_qsae(data, CompressorConfig::default()));

        // Aggressive switching
        let aggressive = CompressorConfig::builder()
            .quorum(QuorumParams::new().with_lambda(0.3).with_delta(0.8))
            .build();
        let mut aggressive_result = self.benchmark_qsae(data, aggressive);
        aggressive_result.name = "QSAE (aggressive)".to_string();
        results.push(aggressive_result);

        // Conservative switching
        let conservative = CompressorConfig::builder()
            .quorum(QuorumParams::new().with_lambda(0.8).with_delta(2.0))
            .build();
        let mut conservative_result = self.benchmark_qsae(data, conservative);
        conservative_result.name = "QSAE (conservative)".to_string();
        results.push(conservative_result);

        // Simple mode (no quorum)
        let simple = CompressorConfig::builder().use_quorum(false).build();
        let mut simple_result = self.benchmark_qsae(data, simple);
        simple_result.name = "QSAE (simple)".to_string();
        results.push(simple_result);

        results
    }

    /// Print formatted benchmark results.
    pub fn print_results(results: &[BenchmarkResult]) {
        println!("
{:=^70}", " Benchmark Results ");
        println!("{:<20} {:>10} {:>10} {:>12} {:>12}", 
            "Compressor", "Ratio", "Size", "Comp MB/s", "Decomp MB/s");
        println!("{:-<70}", "");

        for result in results {
            println!("{:<20} {:>10.2} {:>10} {:>12.1} {:>12.1}",
                result.name,
                result.ratio,
                format!("{:.1} KB", result.compressed_size as f64 / 1024.0),
                result.compress_speed_mbps,
                result.decompress_speed_mbps,
            );
        }

        println!("{:=<70}", "");

        // Find best ratio
        if let Some(best) = results.iter().max_by(|a, b| a.ratio.partial_cmp(&b.ratio).unwrap()) {
            println!("Best ratio: {} ({:.2}:1)", best.name, best.ratio);
        }

        // Find fastest compression
        if let Some(fastest) = results.iter().max_by(|a, b| a.compress_speed_mbps.partial_cmp(&b.compress_speed_mbps).unwrap()) {
            println!("Fastest compression: {} ({:.1} MB/s)", fastest.name, fastest.compress_speed_mbps);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_suite() {
        let suite = BenchmarkSuite::new();
        let data = b"Benchmark test data. ".repeat(10000);
        let results = suite.benchmark_qsae_variants(&data[..]);

        assert!(!results.is_empty());
        for result in &results {
            assert!(result.ratio > 0.0);
            assert!(result.compress_time_ms > 0);
        }
    }
}
