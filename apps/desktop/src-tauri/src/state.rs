use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub current_file: Option<String>,
    pub compression_in_progress: bool,
    pub last_compression_stats: Option<CompressionStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub ratio: f64,
    pub duration_ms: u64,
    pub block_count: usize,
}
