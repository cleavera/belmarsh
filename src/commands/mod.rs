pub mod graph;
pub mod inspect;
pub mod statistics;
pub mod validate;

use clap::{Parser, Subcommand};

use crate::commands::graph::GraphCommand;
use crate::commands::inspect::InspectCommand;
use crate::commands::statistics::StatisticsCommand;
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
    Graph(GraphCommand),
    Inspect(InspectCommand),
}
