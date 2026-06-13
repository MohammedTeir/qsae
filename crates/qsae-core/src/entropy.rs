/// Shannon entropy calculator — the foundation of QSAE's quorum sensing.
/// 
/// H(block) = -Σ p(x) × log₂(p(x))
/// Output range: 0.0 (all bytes identical) → 8.0 (perfectly random)

pub fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut counts = [0u64; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &count in &counts {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }

    entropy
}

/// Entropy profile for a byte slice split into blocks
pub fn entropy_profile(data: &[u8], block_size: usize) -> Vec<f64> {
    data.chunks(block_size)
        .map(shannon_entropy)
        .collect()
}

/// Detect if data appears to be already compressed/encrypted (high entropy)
pub fn is_incompressible(data: &[u8]) -> bool {
    shannon_entropy(data) >= 7.5
}

/// Detect if data is run-dominated (very low entropy)
pub fn is_run_dominated(data: &[u8]) -> bool {
    shannon_entropy(data) < 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_entropy() {
        let data = vec![0xAA; 1000];
        assert_eq!(shannon_entropy(&data), 0.0);
    }

    #[test]
    fn test_random_entropy() {
        // Pseudo-random data should have high entropy
        let data: Vec<u8> = (0..10000).map(|i| ((i * 7 + 13) % 256) as u8).collect();
        let h = shannon_entropy(&data);
        assert!(h > 7.0, "Expected high entropy, got {}", h);
    }

    #[test]
    fn test_two_symbol_entropy() {
        let mut data = Vec::with_capacity(1000);
        for i in 0..1000 {
            data.push(if i % 2 == 0 { 0x00 } else { 0xFF });
        }
        let h = shannon_entropy(&data);
        assert!((h - 1.0).abs() < 0.01, "Expected ~1.0, got {}", h);
    }

    #[test]
    fn test_empty_entropy() {
        assert_eq!(shannon_entropy(&[]), 0.0);
    }

    #[test]
    fn test_entropy_profile() {
        let mut data = vec![0x00; 1000];
        data.extend(vec![0xFF; 1000]);
        let profile = entropy_profile(&data, 500);
        assert_eq!(profile.len(), 4);
        assert_eq!(profile[0], 0.0);
        assert_eq!(profile[1], 0.0);
        assert_eq!(profile[2], 0.0);
        assert_eq!(profile[3], 0.0);
    }
}
