mod commands;

use std::time::Instant;
use clap::Parser;
use commands::{Cli, Commands, statistics::StatisticsCommandError};

#[derive(Debug)]
pub enum BelmarshCliError {
    Statistics(StatisticsCommandError),
}

fn main() -> Result<(), BelmarshCliError> {
    let start = Instant::now();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Statistics(statistics) => {
            statistics.run().map_err(|e| BelmarshCliError::Statistics(e))?;
        }
    }

    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);

    Ok(())
}
