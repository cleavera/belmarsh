use std::collections::HashMap;

use belmarsh::{
    dependency::{
        cycle::CycleDetector, list::{DependencyList, DependencyListFromRepositoryError}
    },
    module::Module,
    repository::{child::RepositoryChildPath, Repository, RepositoryFromStringError},
};
use clap::{Args, command};

#[derive(Args, Debug)]
#[command(about = "Validate")]
pub struct ValidateCommand {
    repository_path: String,

    #[arg(long, help = "Run circular module validation")]
    circular_modules: bool,

    #[arg(long, help = "Run circular file validation")]
    circular_files: bool,
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
        let run_all = !self.circular_modules && !self.circular_files;

        if run_all || self.circular_modules || self.circular_files {
            let repository: Repository = self.repository_path.try_into()?;

            if run_all || self.circular_modules {
                println!("Running circular module validation");
                let dependencies: DependencyList<Module, Module> = repository.clone().try_into()?;
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
            }

            if run_all || self.circular_files {
                println!("\nRunning circular file validation");
                let dependencies: DependencyList<RepositoryChildPath, RepositoryChildPath> =
                    repository.try_into()?;
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
            }
        }

        Ok(())
    }
}

