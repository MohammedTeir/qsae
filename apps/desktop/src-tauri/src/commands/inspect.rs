use qsae_core::Decompressor;
use crate::{FileInfoResponse, BlockInfo};

#[tauri::command]
pub async fn inspect_file(path: String) -> Result<FileInfoResponse, String> {
    let data = std::fs::read(&path).map_err(|e| e.to_string())?;
    let decompressor = Decompressor::new();
    let info = decompressor.inspect(&data).map_err(|e| e.to_string())?;

    let block_info = info.block_info.into_iter().map(|b| BlockInfo {
        index: b.index,
        codec_name: b.codec_name,
        original_len: b.original_len,
        compressed_len: b.compressed_len,
        ratio: b.ratio,
    }).collect();

    Ok(FileInfoResponse {
        version: info.version,
        block_count: info.block_count,
        original_size: info.original_size,
        compressed_size: info.compressed_size,
        ratio: info.ratio,
        codec_breakdown: info.codec_breakdown,
        block_info,
    })
}
