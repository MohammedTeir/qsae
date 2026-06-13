#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

mod commands;
mod state;

use commands::{compress, decompress, inspect, analyze};
use state::AppState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompressionRequest {
    pub input_path: String,
    pub output_path: String,
    pub lambda: f64,
    pub delta: f64,
    pub block_size: usize,
    pub use_quorum: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompressionResponse {
    pub success: bool,
    pub original_size: usize,
    pub compressed_size: usize,
    pub ratio: f64,
    pub block_count: usize,
    pub duration_ms: u64,
    pub codec_usage: Vec<(String, usize, f64)>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuorumData {
    pub entropy_profile: Vec<f64>,
    pub quorum_curve: Vec<f64>,
    pub switch_points: Vec<usize>,
    pub block_assignments: Vec<BlockAssignment>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockAssignment {
    pub index: usize,
    pub codec_name: String,
    pub entropy: f64,
    pub quorum_signal: f64,
    pub is_switch_point: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileInfoResponse {
    pub version: u8,
    pub block_count: usize,
    pub original_size: u64,
    pub compressed_size: u64,
    pub ratio: f64,
    pub codec_breakdown: Vec<(String, usize, f64)>,
    pub block_info: Vec<BlockInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockInfo {
    pub index: usize,
    pub codec_name: String,
    pub original_len: u32,
    pub compressed_len: u32,
    pub ratio: f64,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            compress::compress_file,
            decompress::decompress_file,
            inspect::inspect_file,
            analyze::analyze_file,
            analyze::get_quorum_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
