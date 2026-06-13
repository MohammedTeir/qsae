use crate::partitioner::Block;
use crate::quorum::{QuorumEngine, QuorumParams};

/// Dispatcher routes blocks to optimal codecs using quorum sensing.
/// 
/// Phase 1: Simple entropy-based routing (direct mapping).
/// Phase 2: Quorum-aware contextual routing with entropy inflection detection.
#[derive(Debug, Clone)]
pub struct Dispatcher {
    params: QuorumParams,
    quorum: QuorumEngine,
}

#[derive(Debug, Clone)]
pub struct BlockAssignment {
    pub block_index: usize,
    pub codec_id: u8,
    pub entropy: f64,
    pub quorum_signal: f64,
    pub is_switch_point: bool,
}

/// Result of quorum analysis including visualization data.
#[derive(Debug, Clone)]
pub struct QuorumAnalysis {
    pub assignments: Vec<BlockAssignment>,
    pub quorum_curve: Vec<f64>,
    pub switch_points: Vec<usize>,
    pub entropy_profile: Vec<f64>,
}

impl Dispatcher {
    pub fn new(params: QuorumParams) -> Self {
        let quorum = QuorumEngine::new(params);
        Self { params, quorum }
    }

    /// Phase 1: Simple entropy-based routing (no quorum context).
    pub fn assign_simple(&self, blocks: &[Block], data: &[u8]) -> Vec<BlockAssignment> {
        blocks.iter().enumerate().map(|(i, block)| {
            let block_data = &data[block.offset..block.offset + block.len];
            let codec = crate::codecs::select_codec_for_data(block.entropy, block_data);
            BlockAssignment {
                block_index: i,
                codec_id: codec.id(),
                entropy: block.entropy,
                quorum_signal: block.entropy, // No quorum in simple mode
                is_switch_point: false,
            }
        }).collect()
    }

    /// Phase 2: Quorum-aware routing with contextual codec assignment.
    /// 
    /// Algorithm:
    /// 1. Compute quorum signal Q(i) for each block
    /// 2. Detect inflection points where Q(i) changes significantly
    /// 3. At switch points, re-evaluate optimal codec based on local context
    /// 4. Between switches, maintain codec stability (hysteresis)
    pub fn assign_quorum(&self, blocks: &[Block], data: &[u8]) -> QuorumAnalysis {
        if blocks.is_empty() {
            return QuorumAnalysis {
                assignments: Vec::new(),
                quorum_curve: Vec::new(),
                switch_points: Vec::new(),
                entropy_profile: Vec::new(),
            };
        }

        let entropies: Vec<f64> = blocks.iter().map(|b| b.entropy).collect();
        let quorum_curve = self.quorum.quorum_curve(&entropies);
        let switches = self.quorum.compute_switches(&entropies);
        let switch_points: Vec<usize> = switches.iter().map(|(idx, _)| *idx).collect();

        let mut assignments = Vec::with_capacity(blocks.len());
        let first_data = &data[blocks[0].offset..blocks[0].offset + blocks[0].len];
        let mut current_codec = self.select_codec_for_quorum(quorum_curve[0], entropies[0], first_data).id();
        let mut prev_quorum = quorum_curve[0];

        for (i, block) in blocks.iter().enumerate() {
            let q = quorum_curve[i];
            let is_switch = switch_points.contains(&i);

            let block_data = &data[block.offset..block.offset + block.len];

            // At switch points, re-evaluate codec based on local trend
            if is_switch {
                let trend = if i > 0 { q - prev_quorum } else { 0.0 };
                let predicted_entropy = if trend > 0.0 {
                    // Quorum increasing → entropy regime shifting up
                    block.entropy.min(7.5)
                } else {
                    // Quorum decreasing → entropy regime shifting down
                    block.entropy.max(0.0)
                };
                current_codec = self.select_codec_for_quorum(q, predicted_entropy, block_data).id();
            }

            // Hysteresis: don't switch too frequently
            // If not a switch point but codec mismatch is severe, force switch
            let optimal_codec = self.select_codec_for_quorum(q, block.entropy, block_data).id();
            if !is_switch && optimal_codec != current_codec {
                let mismatch_severity = (q - prev_quorum).abs();
                if mismatch_severity > self.params.delta * 1.5 {
                    current_codec = optimal_codec;
                }
            }

            assignments.push(BlockAssignment {
                block_index: i,
                codec_id: current_codec,
                entropy: block.entropy,
                quorum_signal: q,
                is_switch_point: is_switch,
            });

            prev_quorum = q;
        }

        QuorumAnalysis {
            assignments,
            quorum_curve,
            switch_points,
            entropy_profile: entropies,
        }
    }

    /// Select codec based on both quorum signal, raw entropy, and block data.
    /// Quorum signal provides context about neighborhood entropy trend.
    fn select_codec_for_quorum(&self, quorum_signal: f64, entropy: f64, data: &[u8]) -> Box<dyn crate::codecs::Codec> {
        // Use quorum signal to adjust entropy zone boundaries
        // High quorum signal in neighborhood suggests sustained high entropy → be conservative
        let adjusted_entropy = if quorum_signal > 6.0 {
            // High neighborhood entropy: bias toward higher-entropy codecs
            entropy.min(7.5)
        } else if quorum_signal < 2.0 {
            // Low neighborhood entropy: bias toward lower-entropy codecs
            entropy.max(0.0)
        } else {
            entropy
        };

        crate::codecs::select_codec_for_data(adjusted_entropy, data)
    }

    /// Get codec name for display.
    pub fn codec_name(id: u8) -> &'static str {
        match id {
            0x00 => "Skip",
            0x01 => "RLE",
            0x02 => "LZ4",
            0x03 => "LZ77",
            0x04 => "Huffman",
            0x05 => "ANS",
            0x06 => "BWT",
            0x07 => "Delta",
            0x08 => "DEFLATE",
            _ => "Unknown",
        }
    }

    /// Get codec description for display.
    pub fn codec_description(id: u8) -> &'static str {
        match id {
            0x00 => "Pass-through (incompressible)",
            0x01 => "Run-Length Encoding (uniform data)",
            0x02 => "LZ4 fast (structured patterns)",
            0x03 => "LZ77 sliding window (repetition)",
            0x04 => "Huffman coding (skewed distribution)",
            0x05 => "ANS arithmetic (dense data)",
            0x06 => "BWT+MTF (text-heavy)",
            0x07 => "Delta encoding (numeric sequences)",
            0x08 => "DEFLATE fallback",
            _ => "Unknown codec",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::partitioner::Block;

    #[test]
    fn test_simple_dispatch() {
        let params = QuorumParams::default();
        let dispatcher = Dispatcher::new(params);

        // Create dummy data (4000 bytes) so block offsets are valid
        let data = b"Hello, world! Non-numeric dummy data. ".repeat(110);

        let blocks = vec![
            Block { offset: 0, len: 1000, entropy: 0.5 },   // RLE
            Block { offset: 1000, len: 1000, entropy: 3.0 }, // LZ4
            Block { offset: 2000, len: 1000, entropy: 5.5 }, // Huffman
            Block { offset: 3000, len: 1000, entropy: 7.8 }, // Skip
        ];

        let assignments = dispatcher.assign_simple(&blocks, &data);
        assert_eq!(assignments[0].codec_id, 0x01); // RLE
        assert_eq!(assignments[1].codec_id, 0x02); // LZ4
        assert_eq!(assignments[2].codec_id, 0x04); // Huffman
        assert_eq!(assignments[3].codec_id, 0x00); // Skip
    }

    #[test]
    fn test_quorum_dispatch() {
        let params = QuorumParams::default();
        let dispatcher = Dispatcher::new(params);

        // Create dummy data (16000 bytes)
        let data = b"Hello, world! Non-numeric dummy data. ".repeat(430);

        // Step function: low entropy then high entropy
        let mut blocks = Vec::new();
        for i in 0..8 {
            blocks.push(Block { offset: i * 1000, len: 1000, entropy: 1.0 });
        }
        for i in 8..16 {
            blocks.push(Block { offset: i * 1000, len: 1000, entropy: 7.0 });
        }

        let analysis = dispatcher.assign_quorum(&blocks, &data);

        // Should have detected switch points
        assert!(!analysis.switch_points.is_empty(), "Should detect entropy transition");

        // First half should be low-entropy codec
        let first_codec = analysis.assignments[0].codec_id;
        assert!(first_codec == 0x01 || first_codec == 0x02, "First half should use low-entropy codec");

        // Second half should be high-entropy codec
        let last_codec = analysis.assignments[15].codec_id;
        assert!(last_codec == 0x00 || last_codec == 0x08 || last_codec == 0x05, "Second half should use high-entropy codec");

        // Quorum curve should show elevation in second half
        let mid_q = analysis.quorum_curve[8];
        let start_q = analysis.quorum_curve[0];
        assert!(mid_q > start_q, "Quorum should increase at transition");
    }

    #[test]
    fn test_quorum_analysis_structure() {
        let params = QuorumParams::default().with_delta(0.5);
        let dispatcher = Dispatcher::new(params);

        // Create dummy data (4000 bytes)
        let data = b"Hello, world! Non-numeric dummy data. ".repeat(110);

        let blocks = vec![
            Block { offset: 0, len: 1000, entropy: 2.0 },
            Block { offset: 1000, len: 1000, entropy: 2.5 },
            Block { offset: 2000, len: 1000, entropy: 6.0 },
            Block { offset: 3000, len: 1000, entropy: 6.5 },
        ];

        let analysis = dispatcher.assign_quorum(&blocks, &data);

        assert_eq!(analysis.assignments.len(), 4);
        assert_eq!(analysis.entropy_profile.len(), 4);
        assert_eq!(analysis.quorum_curve.len(), 4);

        // Should have switch point between low and high entropy
        let has_switch = analysis.switch_points.iter().any(|&idx| idx >= 1 && idx <= 3);
        assert!(has_switch, "Should detect transition between entropy regimes");
    }

    #[test]
    fn test_codec_names() {
        assert_eq!(Dispatcher::codec_name(0x01), "RLE");
        assert_eq!(Dispatcher::codec_name(0x02), "LZ4");
        assert_eq!(Dispatcher::codec_name(0xFF), "Unknown");
    }

    #[test]
    fn test_empty_quorum_dispatch() {
        let params = QuorumParams::default();
        let dispatcher = Dispatcher::new(params);
        let blocks: Vec<Block> = Vec::new();
        let data: Vec<u8> = Vec::new();

        let analysis = dispatcher.assign_quorum(&blocks, &data);
        assert!(analysis.assignments.is_empty());
        assert!(analysis.quorum_curve.is_empty());
    }
}
