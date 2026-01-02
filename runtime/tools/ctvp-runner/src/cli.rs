use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ctvp-runner")]
#[command(about = "MYTHOS Conformance Test Vector Pack Runner", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Verify implementation against CTVP
    Verify {
        /// Path to CTVP directory
        #[arg(long)]
        pack: PathBuf,

        /// Test suite to run (can, receipts, ledger, merkle, all)
        #[arg(long, default_value = "can")]
        suite: String,

        /// Run specific vector only
        #[arg(long)]
        vector: Option<String>,

        /// Stop on first failure
        #[arg(long)]
        fail_fast: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List available test vectors
    List {
        /// Path to CTVP directory
        #[arg(long)]
        pack: PathBuf,

        /// Filter by suite
        #[arg(long)]
        suite: Option<String>,
    },

    /// Show detailed info about a vector
    Info {
        /// Path to CTVP directory
        #[arg(long)]
        pack: PathBuf,

        /// Vector ID
        #[arg(long)]
        vector: String,
    },
}
