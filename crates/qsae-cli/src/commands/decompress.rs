use anyhow::Result;
use console::style;
use human_bytes::human_bytes;
use indicatif::{ProgressBar, ProgressStyle};
use qsae_core::Decompressor;
use std::fs::metadata;
use std::time::Instant;

pub fn run(input: &str, output: &str) -> Result<()> {
    let input_path = std::path::Path::new(input);
    if !input_path.exists() {
        anyhow::bail!("Input file not found: {}", input);
    }

    let compressed_size = metadata(input)?.len();

    println!("{}", style("QSAE Decompressor").bold().cyan());
    println!("  Input:  {} ({})", style(input).dim(), human_bytes(compressed_size as f64));
    println!("  Output: {}", style(output).dim());
    println!();

    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg} [{bar:40.cyan/blue}] {pos}%")
            .unwrap()
            .progress_chars("#>-"),
    );

    let start = Instant::now();

    pb.set_message("Decompressing...");
    pb.set_position(50);

    let decompressor = Decompressor::new();
    let original_size = decompressor.decompress_file(input, output)?;

    pb.set_message("Verifying...");
    pb.set_position(100);
    pb.finish_and_clear();

    let duration = start.elapsed();
    let speed = (original_size as f64 / 1024.0 / 1024.0) / duration.as_secs_f64();

    println!("{}", style("✓ Decompression complete").bold().green());
    println!();
    println!("  {} → {}", 
        style(human_bytes(compressed_size as f64)).dim(),
        style(human_bytes(original_size as f64)).bold()
    );
    println!("  Time:   {:.1}s  |  Speed: {:.0} MB/s", 
        duration.as_secs_f64(), 
        speed
    );

    Ok(())
}
