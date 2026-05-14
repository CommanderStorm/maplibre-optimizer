mod cmd;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "maplibre-style-optimize", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Optimize a `MapLibre` style JSON document.
    Optimize(cmd::optimize::OptimizeArgs),

    /// Collect tile statistics from an `MBTiles` file.
    Stats(cmd::stats::StatsArgs),

    /// Apply a tile pruning advisory: prune + MLT-encode tiles, rewrite style.
    Advisory(cmd::advisory::AdvisoryArgs),

    /// Compute static complexity metrics for a style JSON document.
    Complexity(cmd::complexity::ComplexityArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Optimize(args) => cmd::optimize::run(args),
        Command::Stats(ref args) => cmd::stats::run(args),
        Command::Advisory(ref args) => cmd::advisory::run(args),
        Command::Complexity(ref args) => cmd::complexity::run(args),
    }
}
