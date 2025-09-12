mod commands;

use clap::Parser;
use commands::{Cli, Commands, statistics::StatisticsCommandError};
use std::time::Instant;

use crate::commands::graph::GraphCommandError;
use crate::commands::validate::ValidateCommandError;

#[derive(Debug)]
pub enum BelmarshCliError {
    Statistics(StatisticsCommandError),
    Validate(ValidateCommandError),
    Graph(GraphCommandError),
}

fn main() -> Result<(), BelmarshCliError> {
    let start = Instant::now();

    let cli = Cli::parse();

    match cli.command {
        Commands::Statistics(statistics) => {
            statistics
                .run()
                .map_err(|e| BelmarshCliError::Statistics(e))?;
        }
        Commands::Validate(validate) => {
            validate.run().map_err(|e| BelmarshCliError::Validate(e))?
        }
        Commands::Graph(graph) => graph.run().map_err(|e| BelmarshCliError::Graph(e))?,
    }

    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);

    Ok(())
}
