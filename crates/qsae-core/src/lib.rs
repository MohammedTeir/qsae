//! QSAE — Quorum Sensing Adaptive Encoder
//! 
//! A bio-inspired compression engine that adaptively routes data regions
//! to optimal codecs based on local entropy signals.
//! 
//! Phase 3 features:
//! - Full codec pool: RLE, LZ4, LZ77, Huffman, ANS, BWT, Delta, Skip, DEFLATE
//! - Parallel block compression via Rayon
//! - Benchmark suite vs other compressors
//! - Performance metrics and speedup tracking

pub mod bench;
pub mod codecs;
pub mod compressor;
pub mod decompressor;
pub mod dispatcher;
pub mod entropy;
pub mod error;
pub mod format;
pub mod partitioner;
pub mod quorum;

pub use bench::{BenchmarkResult, BenchmarkSuite};
pub use compressor::{Compressor, CompressorConfig, CompressionStats};
pub use decompressor::{Decompressor, FileInfo, BlockInfo};
pub use dispatcher::QuorumAnalysis;
pub use error::{QsaeError, Result};
pub use format::header::QsaeHeader;
pub use quorum::QuorumParams;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
