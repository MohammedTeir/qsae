use clap::{Parser, Subcommand};
use qsae_core::BenchmarkSuite;

mod commands;

use commands::{compress, decompress, inspect};

#[derive(Parser)]
#[command(name = "qsae")]
#[command(about = "QSAE — Quorum Sensing Adaptive Encoder (Phase 3)")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compress a file or directory
    Compress {
        input: String,
        output: String,
        #[arg(long, default_value = "0.5")]
        lambda: f64,
        #[arg(long, default_value = "1.2")]
        delta: f64,
        #[arg(long, default_value = "65536")]
        block_size: usize,
        #[arg(long)]
        simple: bool,
    },
    /// Decompress a .qsae file
    Decompress {
        input: String,
        output: String,
    },
    /// Inspect a .qsae file
    Inspect {
        input: String,
    },
    /// Show compression statistics
    Stats {
        input: String,
    },
    /// Benchmark against different configurations
    Bench {
        input: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compress { input, output, lambda, delta, block_size, simple } => {
            compress::run(&input, &output, lambda, delta, block_size, simple)?;
        }
        Commands::Decompress { input, output } => {
            decompress::run(&input, &output)?;
        }
        Commands::Inspect { input } => {
            inspect::run(&input)?;
        }
        Commands::Stats { input } => {
            inspect::run_stats(&input)?;
        }
        Commands::Bench { input } => {
            run_bench(&input)?;
        }
    }

    Ok(())
}

fn run_bench(input: &str) -> anyhow::Result<()> {
    use console::style;
    use human_bytes::human_bytes;

    let data = std::fs::read(input)?;
    println!("{}", style("QSAE Benchmark Suite").bold().cyan());
    println!("  File: {} ({})", input, human_bytes(data.len() as f64));
    println!();

    let suite = BenchmarkSuite::new();
    let results = suite.benchmark_qsae_variants(&data);
    BenchmarkSuite::print_results(&results);

    Ok(())
}
