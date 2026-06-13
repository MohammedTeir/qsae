use crate::entropy::shannon_entropy;
use crate::quorum::QuorumParams;

/// Block partitioner with entropy-driven boundaries.
/// 
/// Algorithm:
/// 1. Start with default block size (64KB)
/// 2. Scan for entropy gradients within each block
/// 3. If gradient > delta, split at the boundary
/// 4. Enforce min/max block sizes to prevent fragmentation
/// 5. Merge tiny adjacent blocks if both below min_size
#[derive(Debug, Clone)]
pub struct BlockPartitioner {
    params: QuorumParams,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub offset: usize,
    pub len: usize,
    pub entropy: f64,
}

impl BlockPartitioner {
    pub fn new(params: QuorumParams) -> Self {
        Self { params }
    }

    /// Partition data into variable-length blocks with entropy-driven boundaries.
    pub fn partition(&self, data: &[u8]) -> Vec<Block> {
        if data.is_empty() {
            return Vec::new();
        }

        let default_size = self.params.default_block_size;
        let min_size = self.params.min_block_size;
        let max_size = self.params.max_block_size;
        let delta = self.params.delta;

        let mut blocks = Vec::new();
        let mut offset = 0;

        while offset < data.len() {
            let remaining = data.len() - offset;

            // Determine candidate block size
            let candidate_size = default_size.min(remaining).min(max_size);

            // Look for entropy gradient within candidate window
            let block_data = &data[offset..offset + candidate_size];
            let split_point = self.find_entropy_boundary(block_data, min_size, delta);

            let actual_size = if let Some(split) = split_point {
                // Ensure split respects min_size
                let split_pos = split.max(min_size).min(candidate_size - min_size);
                if split_pos >= min_size && split_pos <= candidate_size - min_size {
                    split_pos
                } else {
                    candidate_size
                }
            } else {
                candidate_size
            };

            // Handle final block (may be smaller)
            let actual_size = actual_size.min(remaining);

            let block_slice = &data[offset..offset + actual_size];
            let entropy = shannon_entropy(block_slice);

            blocks.push(Block {
                offset,
                len: actual_size,
                entropy,
            });

            offset += actual_size;
        }

        // Post-process: merge adjacent tiny blocks if beneficial
        self.merge_small_blocks(blocks, data, min_size)
    }

    /// Find the best split point within a block based on entropy gradient.
    /// Returns the offset (relative to block start) where the gradient is steepest.
    fn find_entropy_boundary(&self, data: &[u8], min_window: usize, threshold: f64) -> Option<usize> {
        if data.len() < min_window * 4 {
            return None; // Too small to split meaningfully
        }

        let window_size = min_window.max(1024);
        let scan_step = window_size / 8; // Fine-grained scanning

        let mut max_gradient = 0.0;
        let mut best_split = None;

        // Scan from min_window to len - min_window
        let start = min_window;
        let end = data.len() - min_window;

        for pos in (start..end).step_by(scan_step) {
            let left_window = window_size.min(pos);
            let right_window = window_size.min(data.len() - pos);

            if left_window < min_window / 2 || right_window < min_window / 2 {
                continue;
            }

            let left_entropy = shannon_entropy(&data[pos - left_window..pos]);
            let right_entropy = shannon_entropy(&data[pos..pos + right_window]);
            let gradient = (right_entropy - left_entropy).abs();

            if gradient > threshold && gradient > max_gradient {
                max_gradient = gradient;
                best_split = Some(pos);
            }
        }

        best_split
    }

    /// Merge adjacent blocks that are both below min_size or have similar entropy.
    fn merge_small_blocks(&self, blocks: Vec<Block>, data: &[u8], min_size: usize) -> Vec<Block> {
        if blocks.len() < 2 {
            return blocks;
        }

        let mut merged = Vec::with_capacity(blocks.len());
        let mut current = blocks[0].clone();

        for next in blocks.into_iter().skip(1) {
            let combined_len = current.len + next.len;

            // Merge if:
            // 1. Both blocks are tiny (< min_size/2)
            // 2. Combined still under max_size AND entropies are similar (< delta/2)
            let should_merge = if current.len < min_size / 2 && next.len < min_size / 2 {
                true
            } else if combined_len <= self.params.max_block_size {
                let entropy_diff = (current.entropy - next.entropy).abs();
                entropy_diff < self.params.delta / 2.0
            } else {
                false
            };

            if should_merge {
                current.len = combined_len;
                let combined_data = &data[current.offset..current.offset + current.len];
                current.entropy = shannon_entropy(combined_data);
            } else {
                merged.push(current);
                current = next;
            }
        }
        merged.push(current);

        merged
    }

    /// Get entropy profile for all blocks (useful for visualization).
    pub fn entropy_profile(&self, data: &[u8]) -> Vec<f64> {
        self.partition(data).into_iter().map(|b| b.entropy).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_partition() {
        let p = BlockPartitioner::new(QuorumParams::default());
        let blocks = p.partition(&[]);
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_uniform_partition() {
        let p = BlockPartitioner::new(QuorumParams::default());
        let data = vec![0u8; 100000];
        let blocks = p.partition(&data);

        assert!(!blocks.is_empty());
        let total: usize = blocks.iter().map(|b| b.len).sum();
        assert_eq!(total, data.len());

        for block in &blocks {
            assert!(block.entropy < 0.1, "Zero data should have low entropy, got {}", block.entropy);
        }
    }

    #[test]
    fn test_entropy_boundary_detection() {
        let params = QuorumParams::default();
        let p = BlockPartitioner::new(params);

        // Create data with clear entropy transition at 50K
        let mut data = vec![0x00; 50000]; // Low entropy
        for i in 0..50000 {
            data.push(((i * 17 + 31) % 256) as u8); // High entropy
        }

        let blocks = p.partition(&data);
        let total: usize = blocks.iter().map(|b| b.len).sum();
        assert_eq!(total, data.len());

        // Should have detected the transition and split near 50K
        let mut found_transition = false;
        for (i, block) in blocks.iter().enumerate() {
            if i == 0 { continue; }
            let prev = &blocks[i - 1];
            if prev.entropy < 1.0 && block.entropy > 6.0 {
                found_transition = true;
                // The boundary should be near 50000
                let boundary = prev.offset + prev.len;
                assert!(
                    boundary >= 45000 && boundary <= 55000,
                    "Boundary at {}, expected near 50000", boundary
                );
            }
        }
        assert!(found_transition, "Should detect entropy transition");
    }

    #[test]
    fn test_small_block_merging() {
        let mut params = QuorumParams::default();
        params.min_block_size = 4096;
        let p = BlockPartitioner::new(params);

        // Many tiny blocks that should be merged
        let mut data = Vec::new();
        for i in 0..100 {
            data.extend(vec![(i % 16) as u8; 100]);
        }

        let blocks = p.partition(&data);
        // Should have merged many tiny blocks
        assert!(blocks.len() < 50, "Should merge small blocks, got {} blocks", blocks.len());

        let total: usize = blocks.iter().map(|b| b.len).sum();
        assert_eq!(total, data.len());
    }

    #[test]
    fn test_max_block_size_respected() {
        let mut params = QuorumParams::default();
        params.max_block_size = 16384; // Small max for testing
        let p = BlockPartitioner::new(params);

        let data = vec![0x42; 100000];
        let blocks = p.partition(&data);

        for block in &blocks {
            assert!(block.len <= params.max_block_size, 
                "Block size {} exceeds max {}", block.len, params.max_block_size);
        }
    }

    #[test]
    fn test_entropy_profile() {
        let p = BlockPartitioner::new(QuorumParams::default());
        let mut data = vec![0x00; 10000];
        data.extend(vec![0xFF; 10000]);

        let profile = p.entropy_profile(&data);
        assert!(!profile.is_empty());
        assert!(profile.iter().all(|&e| e >= 0.0 && e <= 8.0));
    }
}
