pub mod statistics;

use clap::{Parser, Subcommand};
use statistics::StatisticsCommand;

#[derive(Parser, Debug)]
#[command(name = "belmarsh")]
#[command(about = "A tool for analyzing the inner module dependencies of a repository")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Statistics(StatisticsCommand),
}

