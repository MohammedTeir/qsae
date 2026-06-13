use qsae_core::{Compressor, CompressorConfig, Decompressor, QuorumParams};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_roundtrip_text() {
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("input.txt");
    let output_path = dir.path().join("output.qsae");
    let restored_path = dir.path().join("restored.txt");

    let data = b"Hello, world! This is QSAE roundtrip test data. ".repeat(1000);
    fs::write(&input_path, &data[..]).unwrap();

    let compressor = Compressor::new(CompressorConfig::default());
    compressor.compress_file(input_path.to_str().unwrap(), output_path.to_str().unwrap()).unwrap();

    let decompressor = Decompressor::new();
    decompressor.decompress_file(output_path.to_str().unwrap(), restored_path.to_str().unwrap()).unwrap();

    let restored = fs::read(&restored_path).unwrap();
    assert_eq!(&data[..], &restored[..]);
}

#[test]
fn test_roundtrip_rle_data() {
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("zeros.bin");
    let output_path = dir.path().join("zeros.qsae");
    let restored_path = dir.path().join("restored.bin");

    let data = vec![0x42; 100000];
    fs::write(&input_path, &data).unwrap();

    let compressor = Compressor::new(CompressorConfig::default());
    let stats = compressor.compress_file(input_path.to_str().unwrap(), output_path.to_str().unwrap()).unwrap();

    assert!(stats.ratio > 5.0, "RLE data should compress >5x, got {:.2}x", stats.ratio);
    assert!(stats.quorum_analysis.is_some(), "Should have quorum analysis");

    let decompressor = Decompressor::new();
    decompressor.decompress_file(output_path.to_str().unwrap(), restored_path.to_str().unwrap()).unwrap();

    let restored = fs::read(&restored_path).unwrap();
    assert_eq!(data, restored);
}

#[test]
fn test_roundtrip_mixed_entropy() {
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("mixed.bin");
    let output_path = dir.path().join("mixed.qsae");
    let restored_path = dir.path().join("restored.bin");

    let mut data = vec![0x00; 50000];
    for i in 0..50000 {
        data.push(((i * 17 + 31) % 256) as u8);
    }
    fs::write(&input_path, &data).unwrap();

    let compressor = Compressor::new(CompressorConfig::default());
    let stats = compressor.compress_file(input_path.to_str().unwrap(), output_path.to_str().unwrap()).unwrap();

    // Should use multiple codecs for mixed data
    assert!(stats.codec_usage.len() > 1, "Mixed data should use multiple codecs, got {:?}", stats.codec_usage);

    // Should detect switch points
    if let Some(ref analysis) = stats.quorum_analysis {
        assert!(!analysis.switch_points.is_empty(), "Should detect entropy switches");
    }

    let decompressor = Decompressor::new();
    decompressor.decompress_file(output_path.to_str().unwrap(), restored_path.to_str().unwrap()).unwrap();

    let restored = fs::read(&restored_path).unwrap();
    assert_eq!(data, restored);
}

#[test]
fn test_simple_vs_quorum_mode() {
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("test.txt");
    let quorum_path = dir.path().join("quorum.qsae");
    let simple_path = dir.path().join("simple.qsae");

    let data = b"Test data for mode comparison. ".repeat(500);
    fs::write(&input_path, &data[..]).unwrap();

    let quorum_config = CompressorConfig::builder().use_quorum(true).build();
    let simple_config = CompressorConfig::builder().use_quorum(false).build();

    let quorum_compressor = Compressor::new(quorum_config);
    let simple_compressor = Compressor::new(simple_config);

    let quorum_stats = quorum_compressor.compress_file(
        input_path.to_str().unwrap(), 
        quorum_path.to_str().unwrap()
    ).unwrap();

    let simple_stats = simple_compressor.compress_file(
        input_path.to_str().unwrap(), 
        simple_path.to_str().unwrap()
    ).unwrap();

    // Both should produce valid files
    assert!(quorum_path.exists());
    assert!(simple_path.exists());

    // Quorum mode should have analysis data
    assert!(quorum_stats.quorum_analysis.is_some());

    // Both should decompress correctly
    let decompressor = Decompressor::new();
    let restored1 = dir.path().join("restored1.txt");
    let restored2 = dir.path().join("restored2.txt");

    decompressor.decompress_file(quorum_path.to_str().unwrap(), restored1.to_str().unwrap()).unwrap();
    decompressor.decompress_file(simple_path.to_str().unwrap(), restored2.to_str().unwrap()).unwrap();

    assert_eq!(fs::read(&restored1).unwrap(), &data[..]);
    assert_eq!(fs::read(&restored2).unwrap(), &data[..]);
}

#[test]
fn test_quorum_params() {
    let params = QuorumParams::new()
        .with_lambda(0.3)
        .with_delta(0.8)
        .with_window(16);

    assert_eq!(params.lambda, 0.3);
    assert_eq!(params.delta, 0.8);
    assert_eq!(params.window, 16);
}

#[test]
fn test_inspect_phase2() {
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("test.txt");
    let output_path = dir.path().join("test.qsae");

    let data = b"Test data for Phase 2 inspection. ".repeat(100);
    fs::write(&input_path, &data[..]).unwrap();

    let compressor = Compressor::new(CompressorConfig::default());
    compressor.compress_file(input_path.to_str().unwrap(), output_path.to_str().unwrap()).unwrap();

    let compressed = fs::read(&output_path).unwrap();
    let decompressor = Decompressor::new();
    let info = decompressor.inspect(&compressed).unwrap();

    assert_eq!(info.version, 1);
    assert!(info.block_count > 0);
    assert_eq!(info.original_size, data.len() as u64);
    assert!(info.ratio > 0.0);
    assert!(!info.codec_breakdown.is_empty());

    // Phase 2 specific
    assert!(info.switch_map_overhead_ratio >= 0.0);
    assert!(!info.block_info.is_empty());
    assert_eq!(info.block_info.len(), info.block_count);
}

#[test]
fn test_entropy_boundary_detection() {
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("boundary.bin");
    let output_path = dir.path().join("boundary.qsae");
    let restored_path = dir.path().join("restored.bin");

    // Data with sharp entropy boundary at 50K
    let mut data = vec![0x00; 50000];
    for i in 0..50000 {
        data.push(((i * 17 + 31) % 256) as u8);
    }
    fs::write(&input_path, &data).unwrap();

    let compressor = Compressor::new(CompressorConfig::default());
    let stats = compressor.compress_file(input_path.to_str().unwrap(), output_path.to_str().unwrap()).unwrap();

    // Should detect the boundary and use different codecs
    if let Some(ref analysis) = stats.quorum_analysis {
        let low_entropy_blocks = analysis.entropy_profile.iter().filter(|&&e| e < 1.0).count();
        let high_entropy_blocks = analysis.entropy_profile.iter().filter(|&&e| e > 6.0).count();

        assert!(low_entropy_blocks > 0, "Should have low entropy blocks");
        assert!(high_entropy_blocks > 0, "Should have high entropy blocks");

        // Verify switch points detected the transition
        if !analysis.switch_points.is_empty() {
            let first_switch = analysis.switch_points[0];
            // Switch should be in the first half (near the boundary)
            assert!(first_switch < analysis.assignments.len() / 2 + 2,
                "Switch at {}, expected near boundary", first_switch);
        }
    }

    let decompressor = Decompressor::new();
    decompressor.decompress_file(output_path.to_str().unwrap(), restored_path.to_str().unwrap()).unwrap();

    let restored = fs::read(&restored_path).unwrap();
    assert_eq!(data, restored);
}

#[test]
fn test_switch_map_efficiency() {
    let compressor = Compressor::new(CompressorConfig::default());

    // Uniform data (all same codec)
    let uniform_data = vec![0x00; 100000];
    let uniform_stats = {
        let compressed = compressor.compress(&uniform_data).unwrap();
        let decompressor = Decompressor::new();
        decompressor.inspect(&compressed).unwrap()
    };

    // Switch map overhead should be very low for uniform data
    assert!(uniform_stats.switch_map_overhead_ratio < 0.3,
        "Uniform data switch map overhead should be <30%, got {:.1}%",
        uniform_stats.switch_map_overhead_ratio * 100.0);
}
