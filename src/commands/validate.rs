use std::{collections::HashMap, fmt::Display};

use belmarsh::{
    dependency::{
        Dependency,
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

    #[arg(long, help = "Run external barrel validation")]
    external_barrel_imports: bool,
}

#[derive(Debug)]
pub enum ValidateCommandError {
    CircularModuleError(ValidateCircularModuleError),
    CircularFileError(ValidateCircularFilesError),
    ExternalBarrelImportsError(ValidateExternalBarrelImportsError),
    CouldNotParseRepository(RepositoryFromStringError),
}

impl From<ValidateExternalBarrelImportsError> for ValidateCommandError {
    fn from(value: ValidateExternalBarrelImportsError) -> Self {
        ValidateCommandError::ExternalBarrelImportsError(value)
    }
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

#[derive(Debug)]
pub enum ValidateExternalBarrelImportsError {
    CouldNotGetDependencies(DependencyListFromRepositoryError),
}

impl From<DependencyListFromRepositoryError> for ValidateExternalBarrelImportsError {
    fn from(value: DependencyListFromRepositoryError) -> Self {
        ValidateExternalBarrelImportsError::CouldNotGetDependencies(value)
    }
}

enum ValidationFailure {
    CircularDependency(DependencyChain),
    ExternalBarrelImport(Dependency<RepositoryChildPath, RepositoryChildPath>),
}

impl Display for ValidationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationFailure::CircularDependency(chain) => {
                write!(f, "Circular dependency: {}", chain)
            }
            ValidationFailure::ExternalBarrelImport(dependency) => {
                write!(
                    f,
                    "External import does not use a barrel file: {}",
                    dependency
                )
            }
        }
    }
}

impl ValidateCommand {
    pub fn run(self) -> Result<(), ValidateCommandError> {
        let run_all =
            !self.circular_modules && !self.circular_files && !self.external_barrel_imports;

        if run_all || self.circular_modules || self.circular_files || self.external_barrel_imports {
            let repository: Repository = self.repository_path.clone().try_into()?;

            if run_all || self.circular_modules {
                println!("Running circular module validation");
                let failures = self.validate_circular_modules(repository.clone())?;

                for failure in failures.iter() {
                    println!("{}", failure);
                }

                println!("\n\nTotal: {}", failures.len());
            }

            if run_all || self.circular_files {
                println!("\nRunning circular file validation");
                let failures = self.validate_circular_files(repository.clone())?;

                for failure in failures.iter() {
                    println!("{}", failure);
                }

                println!("\n\nTotal: {}", failures.len());
            }

            if run_all || self.external_barrel_imports {
                println!("\nRunning external barrel import validation");
                let failures = self.validate_external_barrel_imports(repository.clone())?;

                for failure in failures.iter() {
                    println!("{}", failure);
                }

                println!("\n\nTotal: {}", failures.len());
            }
        }

        Ok(())
    }

    fn validate_circular_modules(
        &self,
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
        &self,
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

    fn validate_external_barrel_imports(
        &self,
        repository: Repository,
    ) -> Result<Vec<ValidationFailure>, ValidateExternalBarrelImportsError> {
        let dependencies: DependencyList<RepositoryChildPath, RepositoryChildPath> =
            repository.try_into()?;

        Ok(dependencies
            .as_ref()
            .iter()
            .filter(|dependency| !dependency.is_internal())
            .filter_map(|dependency| {
                if dependency.to.is_barrel() {
                    None
                } else {
                    Some(ValidationFailure::ExternalBarrelImport(dependency.clone()))
                }
            })
            .collect())
    }
}
