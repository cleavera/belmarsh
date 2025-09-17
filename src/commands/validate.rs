use std::{collections::HashMap, fmt::Display};

use belmarsh::{
    dependency::{
        chain::DependencyChain,
        cycle::CycleDetector,
        list::{DependencyList, DependencyListFromRepositoryError},
    },
    module::Module,
    repository::{Repository, RepositoryFromStringError, child::RepositoryChildPath},
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
    CircularModuleError(ValidateCircularModuleError),
    CircularFileError(ValidateCircularFilesError),
    CouldNotParseRepository(RepositoryFromStringError),
}

impl From<ValidateCircularFilesError> for ValidateCommandError {
    fn from(value: ValidateCircularFilesError) -> Self {
        ValidateCommandError::CircularFileError(value)
    }
}

impl From<ValidateCircularModuleError> for ValidateCommandError {
    fn from(value: ValidateCircularModuleError) -> Self {
        ValidateCommandError::CircularModuleError(value)
    }
}

impl From<RepositoryFromStringError> for ValidateCommandError {
    fn from(err: RepositoryFromStringError) -> Self {
        ValidateCommandError::CouldNotParseRepository(err)
    }
}

#[derive(Debug)]
pub enum ValidateCircularModuleError {
    CouldNotGetDependencies(DependencyListFromRepositoryError),
}

impl From<DependencyListFromRepositoryError> for ValidateCircularModuleError {
    fn from(value: DependencyListFromRepositoryError) -> Self {
        ValidateCircularModuleError::CouldNotGetDependencies(value)
    }
}

#[derive(Debug)]
pub enum ValidateCircularFilesError {
    CouldNotGetDependencies(DependencyListFromRepositoryError),
}

impl From<DependencyListFromRepositoryError> for ValidateCircularFilesError {
    fn from(value: DependencyListFromRepositoryError) -> Self {
        ValidateCircularFilesError::CouldNotGetDependencies(value)
    }
}

enum ValidationFailure {
    CircularDependency(DependencyChain),
}

impl Display for ValidationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationFailure::CircularDependency(chain) => {
                write!(f, "Circular dependency: {}", chain)
            }
        }
    }
}

impl ValidateCommand {
    pub fn run(self) -> Result<(), ValidateCommandError> {
        let run_all = !self.circular_modules && !self.circular_files;

        if run_all || self.circular_modules || self.circular_files {
            let repository: Repository = self.repository_path.try_into()?;

            if run_all || self.circular_modules {
                println!("Running circular module validation");
                let failures = ValidateCommand::validate_circular_modules(repository.clone())?;

                for failure in failures.iter() {
                    println!("{}", failure);
                }

                println!("\n\nTotal: {}", failures.len());
            }

            if run_all || self.circular_files {
                println!("\nRunning circular file validation");
                let failures = ValidateCommand::validate_circular_files(repository.clone())?;

                for failure in failures.iter() {
                    println!("{}", failure);
                }

                println!("\n\nTotal: {}", failures.len());
            }
        }

        Ok(())
    }

    fn validate_circular_modules(
        repository: Repository,
    ) -> Result<Vec<ValidationFailure>, ValidateCircularModuleError> {
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

        Ok(detector
            .find_cycles()
            .into_iter()
            .map(|chain| ValidationFailure::CircularDependency(chain))
            .collect())
    }

    fn validate_circular_files(
        repository: Repository,
    ) -> Result<Vec<ValidationFailure>, ValidateCircularFilesError> {
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

        Ok(detector
            .find_cycles()
            .into_iter()
            .map(|chain| ValidationFailure::CircularDependency(chain))
            .collect())
    }
}

