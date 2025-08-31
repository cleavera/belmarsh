pub mod statistics;
pub mod validate;

use clap::{Parser, Subcommand};
use statistics::StatisticsCommand;

use crate::commands::validate::ValidateCommand;

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
    Validate(ValidateCommand),
}

