use std::collections::HashMap;

use belmarsh::{
    dependency::{
        cycle::CycleDetector,
        list::{DependencyList, DependencyListFromRepositoryError},
    },
    module::Module,
    repository::{Repository, RepositoryFromStringError},
};
use clap::{Args, command};

#[derive(Args, Debug)]
#[command(about = "Validate")]
pub struct ValidateCommand {
    repository_path: String,
}

#[derive(Debug)]
pub enum ValidateCommandError {
    CouldNotParseRepository(RepositoryFromStringError),
    CouldNotGetDependencies(DependencyListFromRepositoryError),
}

impl From<RepositoryFromStringError> for ValidateCommandError {
    fn from(err: RepositoryFromStringError) -> Self {
        ValidateCommandError::CouldNotParseRepository(err)
    }
}

impl From<DependencyListFromRepositoryError> for ValidateCommandError {
    fn from(value: DependencyListFromRepositoryError) -> Self {
        ValidateCommandError::CouldNotGetDependencies(value)
    }
}

impl ValidateCommand {
    pub fn run(self) -> Result<(), ValidateCommandError> {
        let repository: Repository = self.repository_path.try_into()?;
        let dependencies: DependencyList<Module, Module> = repository.try_into()?;

        let grouped_by_from = dependencies.group_by_from();
        let string_grouped_dependencies: HashMap<String, Vec<String>> = grouped_by_from
            .into_iter()
            .map(|(from, to_list)| {
                (
                    from.to_string(),
                    to_list.into_iter().map(|to| to.to_string()).collect(),
                )
            })
            .collect();

        let detector = CycleDetector::new(string_grouped_dependencies);
        let chains = detector.find_cycles();

        for chain in chains.iter() {
            println!("Circular dependency: {}", chain);
        }

        println!("\n\nTotal: {}", chains.len());

        Ok(())
    }
}
