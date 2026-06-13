use wasm_bindgen::prelude::*;
use qsae_core::{Compressor, CompressorConfig, Decompressor, QuorumParams};
use serde::{Deserialize, Serialize};

/// WASM entry point for QSAE web demo.
/// Exposes compression, analysis, and format inspection to JavaScript.

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Result of WASM analysis for visualization.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WasmAnalysisResult {
    pub original_size: usize,
    pub compressed_size: usize,
    pub ratio: f64,
    pub block_count: usize,
    pub duration_ms: u64,
    pub entropy_profile: Vec<f64>,
    pub quorum_curve: Vec<f64>,
    pub switch_points: Vec<usize>,
    pub codec_assignments: Vec<u8>,
    pub codec_names: Vec<String>,
    pub codec_counts: Vec<usize>,
}

/// Compress data and return analysis.
#[wasm_bindgen]
pub fn compress_and_analyze(data: &[u8], lambda: f64, delta: f64, block_size: usize) -> Result<JsValue, JsValue> {
    let start = std::time::Instant::now();

    let config = CompressorConfig::builder()
        .quorum(QuorumParams::new().with_lambda(lambda).with_delta(delta))
        .block_size_hint(block_size)
        .use_quorum(true)
        .parallel(false) // WASM is single-threaded
        .build();

    let compressor = Compressor::new(config);

    // Compress
    let compressed = compressor.compress(data)
        .map_err(|e| JsValue::from_str(&format!("Compression error: {}", e)))?;

    // Analyze
    let analysis = compressor.analyze(data)
        .map_err(|e| JsValue::from_str(&format!("Analysis error: {}", e)))?;

    let duration = start.elapsed().as_millis() as u64;

    // Build codec counts
    let mut counts = std::collections::HashMap::new();
    for a in &analysis.assignments {
        *counts.entry(a.codec_id).or_insert(0) += 1;
    }

    let mut codec_names = Vec::new();
    let mut codec_counts = Vec::new();
    for (&id, &count) in &counts {
        codec_names.push(qsae_core::dispatcher::Dispatcher::codec_name(id).to_string());
        codec_counts.push(count);
    }

    let result = WasmAnalysisResult {
        original_size: data.len(),
        compressed_size: compressed.len(),
        ratio: data.len() as f64 / compressed.len() as f64,
        block_count: analysis.assignments.len(),
        duration_ms: duration,
        entropy_profile: analysis.entropy_profile,
        quorum_curve: analysis.quorum_curve,
        switch_points: analysis.switch_points.clone(),
        codec_assignments: analysis.assignments.iter().map(|a| a.codec_id).collect(),
        codec_names,
        codec_counts,
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Analyze without compressing (for live parameter adjustment).
#[wasm_bindgen]
pub fn analyze_only(data: &[u8], lambda: f64, delta: f64, block_size: usize) -> Result<JsValue, JsValue> {
    let config = CompressorConfig::builder()
        .quorum(QuorumParams::new().with_lambda(lambda).with_delta(delta))
        .block_size_hint(block_size)
        .build();

    let compressor = Compressor::new(config);
    let analysis = compressor.analyze(data)
        .map_err(|e| JsValue::from_str(&format!("Analysis error: {}", e)))?;

    let mut counts = std::collections::HashMap::new();
    for a in &analysis.assignments {
        *counts.entry(a.codec_id).or_insert(0) += 1;
    }

    let mut codec_names = Vec::new();
    let mut codec_counts = Vec::new();
    for (&id, &count) in &counts {
        codec_names.push(qsae_core::dispatcher::Dispatcher::codec_name(id).to_string());
        codec_counts.push(count);
    }

    let result = WasmAnalysisResult {
        original_size: data.len(),
        compressed_size: 0,
        ratio: 0.0,
        block_count: analysis.assignments.len(),
        duration_ms: 0,
        entropy_profile: analysis.entropy_profile,
        quorum_curve: analysis.quorum_curve,
        switch_points: analysis.switch_points.clone(),
        codec_assignments: analysis.assignments.iter().map(|a| a.codec_id).collect(),
        codec_names,
        codec_counts,
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Decompress a .qsae file.
#[wasm_bindgen]
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    let decompressor = Decompressor::new();
    decompressor.decompress(data)
        .map_err(|e| JsValue::from_str(&format!("Decompression error: {}", e)))
}

/// Inspect a .qsae file.
#[wasm_bindgen]
pub fn inspect(data: &[u8]) -> Result<JsValue, JsValue> {
    let decompressor = Decompressor::new();
    let info = decompressor.inspect(data)
        .map_err(|e| JsValue::from_str(&format!("Inspect error: {}", e)))?;

    let result = serde_json::json!({
        "version": info.version,
        "block_count": info.block_count,
        "original_size": info.original_size,
        "compressed_size": info.compressed_size,
        "ratio": info.ratio,
        "codec_breakdown": info.codec_breakdown,
        "block_info": info.block_info,
    });

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Get library version.
#[wasm_bindgen]
pub fn version() -> String {
    qsae_core::VERSION.to_string()
}
