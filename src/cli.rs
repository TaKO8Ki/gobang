use crate::config::CliConfig;
use structopt::StructOpt;

/// A cross-platform terminal database tool written in Rust
#[derive(StructOpt, Debug)]
#[structopt(name = "gobang")]
pub struct Cli {
    #[structopt(flatten)]
    pub config: CliConfig,
}

pub fn parse() -> Cli {
    Cli::from_args()
}