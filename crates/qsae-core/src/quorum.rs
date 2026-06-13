
/// Parameters for the quorum sensing engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuorumParams {
    /// Decay constant λ — controls neighborhood sensitivity.
    /// Higher = more local sensitivity. Default: 0.5
    pub lambda: f64,
    /// Switching threshold δ — higher = fewer switches. Default: 1.2
    pub delta: f64,
    /// Neighborhood window size in blocks. Default: 8
    pub window: usize,
    /// Minimum block size in bytes. Default: 4096
    pub min_block_size: usize,
    /// Default block size in bytes. Default: 65536
    pub default_block_size: usize,
    /// Maximum block size in bytes. Default: 524288
    pub max_block_size: usize,
}

impl Default for QuorumParams {
    fn default() -> Self {
        Self {
            lambda: 0.5,
            delta: 1.2,
            window: 8,
            min_block_size: 4096,
            default_block_size: 65536,
            max_block_size: 524288,
        }
    }
}

impl QuorumParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_lambda(mut self, lambda: f64) -> Self {
        self.lambda = lambda;
        self
    }

    pub fn with_delta(mut self, delta: f64) -> Self {
        self.delta = delta;
        self
    }

    pub fn with_window(mut self, window: usize) -> Self {
        self.window = window;
        self
    }
}

/// Quorum signal Q(i) = Σ H(block_j) × e^{-λ × |i-j|}
/// Detects threshold crossings for codec switching.
#[derive(Debug, Clone)]  // Added Debug and Clone
pub struct QuorumEngine {
    params: QuorumParams,
}

impl QuorumEngine {
    pub fn new(params: QuorumParams) -> Self {
        Self { params }
    }

    /// Compute quorum signals and detect switch points.
    /// Returns a vector of (block_index, cumulative_quorum_value) at switch points.
    pub fn compute_switches(&self, entropies: &[f64]) -> Vec<(usize, f64)> {
        if entropies.is_empty() {
            return Vec::new();
        }

        let n = entropies.len();
        let mut switches = Vec::new();
        let mut prev_q = 0.0;

        for i in 0..n {
            let q = self.quorum_signal(entropies, i);

            // Detect inflection: significant change in quorum signal
            if i > 0 && (q - prev_q).abs() > self.params.delta {
                switches.push((i, q));
            }

            prev_q = q;
        }

        switches
    }

    /// Compute Q(i) for a single block index.
    fn quorum_signal(&self, entropies: &[f64], index: usize) -> f64 {
        let window = self.params.window;
        let lambda = self.params.lambda;
        let n = entropies.len();

        let start = index.saturating_sub(window);
        let end = (index + window + 1).min(n);

        let mut q = 0.0;
        let mut total_weight = 0.0;
        for j in start..end {
            let distance = if j > index { j - index } else { index - j };
            let weight = (-lambda * distance as f64).exp();
            q += entropies[j] * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            q / total_weight
        } else {
            0.0
        }
    }

    /// Compute full quorum signal curve for all blocks.
    pub fn quorum_curve(&self, entropies: &[f64]) -> Vec<f64> {
        entropies.iter().enumerate()
            .map(|(i, _)| self.quorum_signal(entropies, i))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quorum_signal_uniform() {
        let params = QuorumParams::default();
        let engine = QuorumEngine::new(params);
        let entropies = vec![3.0; 10];
        let switches = engine.compute_switches(&entropies);
        // Uniform entropy should produce no switches
        assert!(switches.is_empty(), "Expected no switches for uniform entropy");
    }

    #[test]
    fn test_quorum_signal_step() {
        let params = QuorumParams::default();
        let engine = QuorumEngine::new(params);
        // Step function: low entropy then high entropy
        let mut entropies = vec![1.0; 8];
        entropies.extend(vec![7.0; 8]);

        let switches = engine.compute_switches(&entropies);
        // Should detect switch around the boundary
        assert!(!switches.is_empty(), "Expected switches at step boundary");

        // Switch should be near index 8
        let first_switch = switches[0].0;
        assert!(first_switch >= 6 && first_switch <= 10, 
                "Switch at {}, expected near 8", first_switch);
    }

    #[test]
    fn test_quorum_curve() {
        let params = QuorumParams::default();
        let engine = QuorumEngine::new(params);
        let entropies = vec![2.0, 4.0, 6.0, 4.0, 2.0];
        let curve = engine.quorum_curve(&entropies);

        assert_eq!(curve.len(), 5);
        // Middle should have highest quorum due to neighborhood accumulation
        assert!(curve[2] > curve[0], "Middle should have higher quorum");
        assert!(curve[2] > curve[4], "Middle should have higher quorum");
    }
}
