// Steel Horizons — deterministic simulation engine CLI
// Copyright (c) 2026 Steel Horizons contributors
// UNLICENSED — private development; no public grant

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![forbid(noop_method_call)]

//! CLI entry point for `steel-horizons-engine`.

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "steel-horizons-engine",
    version,
    about = "Steel Horizons deterministic simulation engine"
)]
struct Cli {
    /// Preferred HTTP port; scans upward through +10
    #[arg(long, default_value_t = 4880)]
    preferred_port: u16,

    /// Development/test data directory override
    #[arg(long)]
    data_dir: Option<String>,

    /// Starting scenario ID (default: starting_system)
    #[arg(long, default_value = "starting_system")]
    scenario: String,

    /// Start lifecycle Unloaded instead of auto-loading
    #[arg(long)]
    no_autoload: bool,

    /// Disable persistence (test mode)
    #[arg(long)]
    no_save: bool,

    /// Disable authentication (development only; loud warning)
    #[arg(long)]
    insecure_no_auth: bool,

    /// Log format
    #[arg(long, default_value = "pretty")]
    log_format: LogFormat,
}

#[derive(clap::ValueEnum, Clone)]
enum LogFormat {
    Pretty,
    Json,
}

fn main() {
    let _cli = Cli::parse();
    println!("steel-horizons-engine {}", steel_horizons_engine::VERSION);
}
