use structopt::StructOpt;

use crate::config::CliConfig;

/// A cross-platform TUI database management tool written in Rust
#[derive(StructOpt, Debug)]
#[structopt(name = "gobang")]
pub struct Cli {
    #[structopt(flatten)]
    pub config: CliConfig,
}

pub fn parse() -> Cli {
    Cli::from_args()
}
