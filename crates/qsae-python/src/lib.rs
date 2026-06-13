use pyo3::prelude::*;
use qsae_core::{Compressor, CompressorConfig, Decompressor, QuorumParams};

/// QSAE Python bindings.
/// Exposes the full compression engine to Python via PyO3.

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyCompressor {
    config: CompressorConfig,
}

#[pymethods]
impl PyCompressor {
    #[new]
    #[pyo3(signature = (lambda=0.5, delta=1.2, block_size=65536, use_quorum=true))]
    fn new(lambda: f64, delta: f64, block_size: usize, use_quorum: bool) -> Self {
        let config = CompressorConfig::builder()
            .quorum(QuorumParams::new().with_lambda(lambda).with_delta(delta))
            .block_size_hint(block_size)
            .use_quorum(use_quorum)
            .build();

        Self { config }
    }

    /// Compress bytes and return compressed data.
    fn compress(&self, data: &[u8]) -> PyResult<Vec<u8>> {
        let compressor = Compressor::new(self.config.clone());
        compressor.compress(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Compression failed: {}", e)))
    }

    /// Compress a file to a .qsae file.
    #[pyo3(signature = (input_path, output_path))]
    fn compress_file(&self, input_path: &str, output_path: &str) -> PyResult<PyCompressionStats> {
        let compressor = Compressor::new(self.config.clone());
        let stats = compressor.compress_file(input_path, output_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Compression failed: {}", e)))?;

        Ok(PyCompressionStats {
            original_size: stats.original_size,
            compressed_size: stats.compressed_size,
            ratio: stats.ratio,
            block_count: stats.block_count,
            duration_ms: stats.duration_ms,
            codec_usage: stats.codec_usage,
        })
    }

    /// Analyze data without compressing (for visualization).
    fn analyze(&self, data: &[u8]) -> PyResult<PyQuorumAnalysis> {
        let compressor = Compressor::new(self.config.clone());
        let analysis = compressor.analyze(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Analysis failed: {}", e)))?;

        Ok(PyQuorumAnalysis {
            entropy_profile: analysis.entropy_profile,
            quorum_curve: analysis.quorum_curve,
            switch_points: analysis.switch_points,
            codec_assignments: analysis.assignments.iter().map(|a| a.codec_id).collect(),
        })
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyDecompressor;

#[pymethods]
impl PyDecompressor {
    #[new]
    fn new() -> Self {
        Self
    }

    /// Decompress bytes.
    fn decompress(&self, data: &[u8]) -> PyResult<Vec<u8>> {
        let decompressor = Decompressor::new();
        decompressor.decompress(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Decompression failed: {}", e)))
    }

    /// Decompress a .qsae file.
    #[pyo3(signature = (input_path, output_path))]
    fn decompress_file(&self, input_path: &str, output_path: &str) -> PyResult<usize> {
        let decompressor = Decompressor::new();
        decompressor.decompress_file(input_path, output_path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Decompression failed: {}", e)))
    }

    /// Inspect a .qsae file without decompressing.
    fn inspect(&self, data: &[u8]) -> PyResult<PyFileInfo> {
        let decompressor = Decompressor::new();
        let info = decompressor.inspect(data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Inspect failed: {}", e)))?;

        Ok(PyFileInfo {
            version: info.version,
            block_count: info.block_count,
            original_size: info.original_size,
            compressed_size: info.compressed_size,
            ratio: info.ratio,
            codec_breakdown: info.codec_breakdown,
        })
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyCompressionStats {
    #[pyo3(get)]
    pub original_size: usize,
    #[pyo3(get)]
    pub compressed_size: usize,
    #[pyo3(get)]
    pub ratio: f64,
    #[pyo3(get)]
    pub block_count: usize,
    #[pyo3(get)]
    pub duration_ms: u64,
    #[pyo3(get)]
    pub codec_usage: Vec<(String, usize, f64)>,
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyQuorumAnalysis {
    #[pyo3(get)]
    pub entropy_profile: Vec<f64>,
    #[pyo3(get)]
    pub quorum_curve: Vec<f64>,
    #[pyo3(get)]
    pub switch_points: Vec<usize>,
    #[pyo3(get)]
    pub codec_assignments: Vec<u8>,
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct PyFileInfo {
    #[pyo3(get)]
    pub version: u8,
    #[pyo3(get)]
    pub block_count: usize,
    #[pyo3(get)]
    pub original_size: u64,
    #[pyo3(get)]
    pub compressed_size: u64,
    #[pyo3(get)]
    pub ratio: f64,
    #[pyo3(get)]
    pub codec_breakdown: Vec<(String, usize, f64)>,
}

/// Convenience functions at module level.
#[pyfunction]
#[pyo3(signature = (data, lambda=0.5, delta=1.2, block_size=65536))]
fn compress(data: &[u8], lambda: f64, delta: f64, block_size: usize) -> PyResult<Vec<u8>> {
    let config = CompressorConfig::builder()
        .quorum(QuorumParams::new().with_lambda(lambda).with_delta(delta))
        .block_size_hint(block_size)
        .build();

    let compressor = Compressor::new(config);
    compressor.compress(data)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Compression failed: {}", e)))
}

#[pyfunction]
fn decompress(data: &[u8]) -> PyResult<Vec<u8>> {
    let decompressor = Decompressor::new();
    decompressor.decompress(data)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Decompression failed: {}", e)))
}

#[pyfunction]
#[pyo3(signature = (data, lambda=0.5, delta=1.2, block_size=65536))]
fn analyze(data: &[u8], lambda: f64, delta: f64, block_size: usize) -> PyResult<PyQuorumAnalysis> {
    let config = CompressorConfig::builder()
        .quorum(QuorumParams::new().with_lambda(lambda).with_delta(delta))
        .block_size_hint(block_size)
        .build();

    let compressor = Compressor::new(config);
    let analysis = compressor.analyze(data)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Analysis failed: {}", e)))?;

    Ok(PyQuorumAnalysis {
        entropy_profile: analysis.entropy_profile,
        quorum_curve: analysis.quorum_curve,
        switch_points: analysis.switch_points,
        codec_assignments: analysis.assignments.iter().map(|a| a.codec_id).collect(),
    })
}

#[pymodule]
fn qsae(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCompressor>()?;
    m.add_class::<PyDecompressor>()?;
    m.add_class::<PyCompressionStats>()?;
    m.add_class::<PyQuorumAnalysis>()?;
    m.add_class::<PyFileInfo>()?;
    m.add_function(wrap_pyfunction!(compress, m)?)?;
    m.add_function(wrap_pyfunction!(decompress, m)?)?;
    m.add_function(wrap_pyfunction!(analyze, m)?)?;
    Ok(())
}
