export interface CompressionResult {
  success: boolean;
  original_size: number;
  compressed_size: number;
  ratio: number;
  block_count: number;
  duration_ms: number;
  codec_usage: [string, number, number][];
  error?: string;
  is_decompression?: boolean;
  input_path?: string;
  output_path?: string;
}

export interface QuorumData {
  entropy_profile: number[];
  quorum_curve: number[];
  switch_points: number[];
  block_assignments: BlockAssignment[];
}

export interface BlockAssignment {
  index: number;
  codec_name: string;
  entropy: number;
  quorum_signal: number;
  is_switch_point: boolean;
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

export interface CompressionParams {
  lambda: number;
  delta: number;
  block_size: number;
  use_quorum: boolean;
}
