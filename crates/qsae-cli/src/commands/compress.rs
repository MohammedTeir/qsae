use anyhow::Result;
use console::style;
use human_bytes::human_bytes;
use indicatif::{ProgressBar, ProgressStyle};
use qsae_core::{Compressor, CompressorConfig, QuorumParams};
use std::fs::metadata;
use std::time::Instant;

pub fn run(input: &str, output: &str, lambda: f64, delta: f64, block_size: usize, simple: bool) -> Result<()> {
    let input_path = std::path::Path::new(input);
    if !input_path.exists() {
        anyhow::bail!("Input file not found: {}", input);
    }

    let original_size = metadata(input)?.len();

    println!("{}", style("QSAE Compressor").bold().cyan());
    if simple {
        println!("  Mode: Simple entropy routing (Phase 1)");
    } else {
        println!("  Mode: Quorum sensing adaptive (Phase 3)");
    }
    println!("  Input:  {} ({})", style(input).dim(), human_bytes(original_size as f64));
    println!("  Output: {}", style(output).dim());
    println!("  λ: {} | δ: {} | block: {}", lambda, delta, human_bytes(block_size as f64));
    println!();

    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg} [{bar:40.cyan/blue}] {pos}%")
            .unwrap()
            .progress_chars("#>-"),
    );

    let start = Instant::now();

    pb.set_message("Partitioning...");
    pb.set_position(10);

    let config = CompressorConfig::builder()
        .quorum(QuorumParams::new().with_lambda(lambda).with_delta(delta).with_window(8))
        .block_size_hint(block_size)
        .use_quorum(!simple)
        .parallel(true)
        .build();

    let compressor = Compressor::new(config);

    pb.set_message("Analyzing entropy...");
    pb.set_position(30);

    if !simple {
        let input_data = std::fs::read(input)?;
        let analysis = compressor.analyze(&input_data)?;

        pb.set_message("Quorum sensing...");
        pb.set_position(50);

        if !analysis.switch_points.is_empty() {
            println!("  {} entropy regime switches detected", analysis.switch_points.len());
        }
        if !analysis.quorum_curve.is_empty() {
            let min_q = analysis.quorum_curve.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_q = analysis.quorum_curve.iter().fold(0.0f64, |a, &b| a.max(b));
            println!("  Quorum range: {:.2} → {:.2}", min_q, max_q);
        }
    }

    pb.set_message("Compressing...");
    pb.set_position(60);

    let stats = compressor.compress_file(input, output)?;

    pb.set_message("Finalizing...");
    pb.set_position(100);
    pb.finish_and_clear();

    let duration = start.elapsed();
    let speed = (original_size as f64 / 1024.0 / 1024.0) / duration.as_secs_f64();

    println!("{}", style("✓ Compression complete").bold().green());
    println!();
    println!("  {} → {}", 
        style(human_bytes(original_size as f64)).bold(),
        style(human_bytes(stats.compressed_size as f64)).bold()
    );
    println!("  Ratio:  {:.2}:1  ({:.1}% smaller)", 
        stats.ratio, 
        (1.0 - 1.0/stats.ratio) * 100.0
    );
    println!("  Time:   {:.1}s  |  Speed: {:.0} MB/s", 
        duration.as_secs_f64(), 
        speed
    );
    println!("  Blocks: {} | Switch map overhead: {:.1}%", 
        stats.block_count,
        stats.switch_map_overhead_ratio * 100.0
    );

    // Phase 3: Show parallel metrics
    if stats.parallel_threads > 1 {
        println!("  Threads: {} (parallel)", stats.parallel_threads);
        if let Some(speedup) = stats.parallel_speedup {
            println!("  Parallel speedup factor: {:.1}x", speedup);
        }
    }
    println!();

    if !stats.codec_usage.is_empty() {
        println!("{}", style("Codec usage:").bold());
        for (name, count, pct) in &stats.codec_usage {
            let bar_len = (pct / 2.0) as usize;
            let bar = "█".repeat(bar_len);
            println!("  {:10} {:6} {:>5.1}% {}", name, count, pct, bar);
        }
    }

    if let Some(ref analysis) = stats.quorum_analysis {
        println!();
        println!("{}", style("Quorum analysis:").bold());
        println!("  Switch points: {}", analysis.switch_points.len());
        if !analysis.switch_points.is_empty() {
            println!("  Locations: {:?}", analysis.switch_points);
        }

        if !analysis.quorum_curve.is_empty() {
            let min_q = analysis.quorum_curve.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_q = analysis.quorum_curve.iter().fold(0.0f64, |a, &b| a.max(b));
            let avg_q = analysis.quorum_curve.iter().sum::<f64>() / analysis.quorum_curve.len() as f64;
            println!("  Q(i) range: {:.2} → {:.2} (avg {:.2})", min_q, max_q, avg_q);
        }
    }

    Ok(())
}
