use qsae_core::{Compressor, CompressorConfig, QuorumParams};
use crate::{QuorumData, BlockAssignment};

#[tauri::command]
pub async fn analyze_file(
    path: String,
    lambda: f64,
    delta: f64,
) -> Result<QuorumData, String> {
    let data = std::fs::read(&path).map_err(|e| e.to_string())?;

    let config = CompressorConfig::builder()
        .quorum(QuorumParams::new().with_lambda(lambda).with_delta(delta))
        .build();

    let compressor = Compressor::new(config);
    let analysis = compressor.analyze(&data).map_err(|e| e.to_string())?;

    let block_assignments = analysis.assignments.into_iter().map(|a| BlockAssignment {
        index: a.block_index,
        codec_name: qsae_core::dispatcher::Dispatcher::codec_name(a.codec_id).to_string(),
        entropy: a.entropy,
        quorum_signal: a.quorum_signal,
        is_switch_point: a.is_switch_point,
    }).collect();

    Ok(QuorumData {
        entropy_profile: analysis.entropy_profile,
        quorum_curve: analysis.quorum_curve,
        switch_points: analysis.switch_points,
        block_assignments,
    })
}

#[tauri::command]
pub async fn get_quorum_data(
    path: String,
    lambda: f64,
    delta: f64,
) -> Result<QuorumData, String> {
    analyze_file(path, lambda, delta).await
}
