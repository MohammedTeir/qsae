export interface WasmAnalysisResult {
  original_size: number;
  compressed_size: number;
  ratio: number;
  block_count: number;
  duration_ms: number;
  entropy_profile: number[];
  quorum_curve: number[];
  switch_points: number[];
  codec_assignments: number[];
  codec_names: string[];
  codec_counts: number[];
}

export interface CompressionParams {
  lambda: number;
  delta: number;
  block_size: number;
}

export interface FileInfo {
  version: number;
  block_count: number;
  original_size: number;
  compressed_size: number;
  ratio: number;
  codec_breakdown: [string, number, number][];
  block_info: BlockInfo[];
}

export interface BlockInfo {
  index: number;
  codec_name: string;
  original_len: number;
  compressed_len: number;
  ratio: number;
}
