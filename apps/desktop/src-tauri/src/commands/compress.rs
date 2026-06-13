use tauri::State;
use std::sync::Mutex;
use qsae_core::{Compressor, CompressorConfig, QuorumParams};
use crate::{CompressionRequest, CompressionResponse, state::AppState};

#[tauri::command]
pub async fn compress_file(
    state: State<'_, Mutex<AppState>>,
    request: CompressionRequest,
) -> Result<CompressionResponse, String> {
    let start = std::time::Instant::now();

    {
        let mut app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.compression_in_progress = true;
    }

    let config = CompressorConfig::builder()
        .quorum(QuorumParams::new()
            .with_lambda(request.lambda)
            .with_delta(request.delta))
        .block_size_hint(request.block_size)
        .use_quorum(request.use_quorum)
        .parallel(true)
        .build();

    let compressor = Compressor::new(config);

    let result = compressor.compress_file(&request.input_path, &request.output_path);

    {
        let mut app_state = state.lock().map_err(|e| e.to_string())?;
        app_state.compression_in_progress = false;
    }

    match result {
        Ok(stats) => {
            let duration = start.elapsed().as_millis() as u64;
            Ok(CompressionResponse {
                success: true,
                original_size: stats.original_size,
                compressed_size: stats.compressed_size,
                ratio: stats.ratio,
                block_count: stats.block_count,
                duration_ms: duration,
                codec_usage: stats.codec_usage,
                error: None,
            })
        }
        Err(e) => Ok(CompressionResponse {
            success: false,
            original_size: 0,
            compressed_size: 0,
            ratio: 0.0,
            block_count: 0,
            duration_ms: 0,
            codec_usage: Vec::new(),
            error: Some(e.to_string()),
        }),
    }
}
