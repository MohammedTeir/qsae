use anyhow::Result;
use console::style;
use human_bytes::human_bytes;
use qsae_core::Decompressor;

pub fn run(input: &str) -> Result<()> {
    let data = std::fs::read(input)?;
    let decompressor = Decompressor::new();
    let info = decompressor.inspect(&data)?;

    println!("{}", style("QSAE File Inspection").bold().cyan());
    println!("  (Phase 2 — Quorum Sensing Analysis)");
    println!();
    println!("  Format version: {}", info.version);
    println!("  Blocks:         {}", info.block_count);
    println!();
    println!("  Original size:  {}", style(human_bytes(info.original_size as f64)).bold());
    println!("  Compressed:     {}", style(human_bytes(info.compressed_size as f64)).bold());
    println!("  Ratio:          {:.2}:1", info.ratio);
    println!("  Overhead:       {} (metadata)", human_bytes(info.overhead_bytes as f64));
    println!("  Switch map:     {:.1}% of raw size", info.switch_map_overhead_ratio * 100.0);
    println!();

    if !info.codec_breakdown.is_empty() {
        println!("{}", style("Codec breakdown:").bold());
        for (name, count, pct) in &info.codec_breakdown {
            let bar_len = (pct / 2.0).max(1.0) as usize;
            let bar = "█".repeat(bar_len);
            println!("  {:10} {:6} {:>5.1}% {}", name, count, pct, bar);
        }
        println!();
    }

    // Phase 2: Show per-block details if not too many
    if info.block_count <= 20 {
        println!("{}", style("Per-block details:").bold());
        for block in &info.block_info {
            let ratio_str = if block.ratio > 1.0 {
                format!("{:.1}x", block.ratio)
            } else {
                format!("{:.0}%", block.ratio * 100.0)
            };
            println!("  [{:3}] {:10} {:>8} → {:>8}  {}", 
                block.index,
                block.codec_name,
                human_bytes(block.original_len as f64),
                human_bytes(block.compressed_len as f64),
                ratio_str
            );
        }
    } else {
        println!("  ({} blocks — use --verbose for full listing)", info.block_count);
    }

    Ok(())
}

pub fn run_stats(input: &str) -> Result<()> {
    // Same as inspect for now
    run(input)
}
