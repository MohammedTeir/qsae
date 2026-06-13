use napi::bindgen_prelude::*;
use napi_derive::napi;
use qsae_core::{Compressor, CompressorConfig, Decompressor, QuorumParams};

/// QSAE Node.js bindings using napi-rs.
/// Exposes async compression API for Node.js applications.

#[napi(object)]
#[derive(Clone, Debug)]
pub struct CompressionOptions {
    pub lambda: Option<f64>,
    pub delta: Option<f64>,
    pub block_size: Option<u32>,
    pub use_quorum: Option<bool>,
}

impl Default for CompressionOptions {
    fn default() -> Self {
        Self {
            lambda: Some(0.5),
            delta: Some(1.2),
            block_size: Some(65536),
            use_quorum: Some(true),
        }
    }
}

#[napi(object)]
pub struct CompressionResult {
    pub data: Buffer,
    pub original_size: u32,
    pub compressed_size: u32,
    pub ratio: f64,
    pub duration_ms: u32,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct AnalysisResult {
    pub entropy_profile: Vec<f64>,
    pub quorum_curve: Vec<f64>,
    pub switch_points: Vec<u32>,
    pub codec_assignments: Vec<u8>,
    pub block_count: u32,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct FileInfo {
    pub version: u8,
    pub block_count: u32,
    pub original_size: i64,
    pub compressed_size: i64,
    pub ratio: f64,
    pub codec_breakdown: Vec<CodecInfo>,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct CodecInfo {
    pub name: String,
    pub count: u32,
    pub percentage: f64,
}

fn build_config(options: Option<CompressionOptions>) -> CompressorConfig {
    let opts = options.unwrap_or_default();
    CompressorConfig::builder()
        .quorum(QuorumParams::new()
            .with_lambda(opts.lambda.unwrap_or(0.5))
            .with_delta(opts.delta.unwrap_or(1.2)))
        .block_size_hint(opts.block_size.unwrap_or(65536) as usize)
        .use_quorum(opts.use_quorum.unwrap_or(true))
        .build()
}

/// Compress data asynchronously.
#[napi]
pub async fn compress(data: Buffer, options: Option<CompressionOptions>) -> Result<CompressionResult> {
    let start = std::time::Instant::now();
    let config = build_config(options);
    let compressor = Compressor::new(config);

    let compressed = compressor.compress(&data)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Compression failed: {}", e)))?;

    let duration = start.elapsed().as_millis() as u32;

    Ok(CompressionResult {
        original_size: data.len() as u32,
        compressed_size: compressed.len() as u32,
        ratio: data.len() as f64 / compressed.len() as f64,
        duration_ms: duration,
        data: compressed.into(),
    })
}

/// Decompress data asynchronously.
#[napi]
pub async fn decompress(data: Buffer) -> Result<Buffer> {
    let decompressor = Decompressor::new();
    let original = decompressor.decompress(&data)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Decompression failed: {}", e)))?;

    Ok(original.into())
}

/// Analyze data without compressing.
#[napi]
pub async fn analyze(data: Buffer, options: Option<CompressionOptions>) -> Result<AnalysisResult> {
    let config = build_config(options);
    let compressor = Compressor::new(config);

    let analysis = compressor.analyze(&data)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Analysis failed: {}", e)))?;

    Ok(AnalysisResult {
        entropy_profile: analysis.entropy_profile,
        quorum_curve: analysis.quorum_curve,
        switch_points: analysis.switch_points.into_iter().map(|i| i as u32).collect(),
        codec_assignments: analysis.assignments.iter().map(|a| a.codec_id).collect(),
        block_count: analysis.assignments.len() as u32,
    })
}

/// Inspect a .qsae file.
#[napi]
pub async fn inspect(data: Buffer) -> Result<FileInfo> {
    let decompressor = Decompressor::new();
    let info = decompressor.inspect(&data)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Inspect failed: {}", e)))?;

    let codec_breakdown = info.codec_breakdown.into_iter()
        .map(|(name, count, percentage)| CodecInfo {
            name,
            count: count as u32,
            percentage,
        })
        .collect();

    Ok(FileInfo {
        version: info.version,
        block_count: info.block_count as u32,
        original_size: info.original_size as i64,
        compressed_size: info.compressed_size as i64,
        ratio: info.ratio,
        codec_breakdown,
    })
}

/// Compress a file asynchronously.
#[napi]
pub async fn compress_file(input_path: String, output_path: String, options: Option<CompressionOptions>) -> Result<CompressionResult> {
    let start = std::time::Instant::now();
    let config = build_config(options);
    let compressor = Compressor::new(config);

    let stats = compressor.compress_file(&input_path, &output_path)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Compression failed: {}", e)))?;

    let duration = start.elapsed().as_millis() as u32;

    let compressed = std::fs::read(&output_path)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Read failed: {}", e)))?;

    Ok(CompressionResult {
        data: compressed.into(),
        original_size: stats.original_size as u32,
        compressed_size: stats.compressed_size as u32,
        ratio: stats.ratio,
        duration_ms: duration,
    })
}

/// Decompress a file asynchronously.
#[napi]
pub async fn decompress_file(input_path: String, output_path: String) -> Result<u32> {
    let decompressor = Decompressor::new();
    let size = decompressor.decompress_file(&input_path, &output_path)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Decompression failed: {}", e)))?;

    Ok(size as u32)
}

/// Get library version.
#[napi]
pub fn version() -> String {
    qsae_core::VERSION.to_string()
}
