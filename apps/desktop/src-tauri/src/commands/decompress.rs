use tauri::State;
use std::sync::Mutex;
use qsae_core::Decompressor;
use crate::state::AppState;

#[tauri::command]
pub async fn decompress_file(
    _state: State<'_, Mutex<AppState>>,
    input_path: String,
    output_path: String,
) -> Result<bool, String> {
    let decompressor = Decompressor::new();

    match decompressor.decompress_file(&input_path, &output_path) {
        Ok(_) => Ok(true),
        Err(e) => Err(e.to_string()),
    }
}
