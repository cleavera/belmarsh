use std::fmt::Display;

use belmarsh::{
    dependency::{Dependency, chain::DependencyChain},
    module_mapping::{ModuleMapping, ModuleMappingFromParamStringError},
    repository::{Repository, child::RepositoryChildPath, path::RepositoryPathFromStringError},
};
use clap::{Args, command};
use std::collections::HashSet;

pub mod barrel_imports_barrel;
pub mod circular_files;
pub mod circular_modules;
pub mod external_barrel_imports;

use barrel_imports_barrel::{ValidateBarrelImportsBarrelError, validate_barrel_imports_barrel};
use circular_files::{ValidateCircularFilesError, validate_circular_files};
use circular_modules::{ValidateCircularModuleError, validate_circular_modules};
use external_barrel_imports::{
    ValidateExternalBarrelImportsError, validate_external_barrel_imports,
};

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

    #[arg(long, help = "Run barrel imports barrel validation")]
    barrel_imports_barrel: bool,

    #[arg(
        long,
        help = "Add a module mapping e.g. --module-mapping @prefix:./path/to/modules",
        value_name = "ALIAS:PATH"
    )]
    module_mapping_params: Vec<String>,
}

#[derive(Debug)]
pub enum ValidateCommandError {
    CircularModuleError(ValidateCircularModuleError),
    CircularFileError(ValidateCircularFilesError),
    ExternalBarrelImportsError(ValidateExternalBarrelImportsError),
    BarrelImportsBarrelError(ValidateBarrelImportsBarrelError),
    CouldNotCreateRepositoryPath(RepositoryPathFromStringError),
    CouldNotParseModuleMapCollection(ModuleMappingFromParamStringError),
}

impl From<ValidateBarrelImportsBarrelError> for ValidateCommandError {
    fn from(value: ValidateBarrelImportsBarrelError) -> Self {
        ValidateCommandError::BarrelImportsBarrelError(value)
    }
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

impl From<RepositoryPathFromStringError> for ValidateCommandError {
    fn from(err: RepositoryPathFromStringError) -> Self {
        ValidateCommandError::CouldNotCreateRepositoryPath(err)
    }
}

impl From<ModuleMappingFromParamStringError> for ValidateCommandError {
    fn from(err: ModuleMappingFromParamStringError) -> Self {
        ValidateCommandError::CouldNotParseModuleMapCollection(err)
    }
}

pub enum ValidationFailure {
    CircularDependency(DependencyChain),
    ExternalBarrelImport(Dependency<RepositoryChildPath, RepositoryChildPath>),
    BarrelImportsBarrel(Dependency<RepositoryChildPath, RepositoryChildPath>),
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
            ValidationFailure::BarrelImportsBarrel(dependency) => {
                write!(f, "Barrel file imports another barrel file: {}", dependency)
            }
        }
    }
}

impl ValidateCommand {
    pub fn run(self) -> Result<(), ValidateCommandError> {
        let run_all = !self.circular_modules
            && !self.circular_files
            && !self.external_barrel_imports
            && !self.barrel_imports_barrel;

        if run_all
            || self.circular_modules
            || self.circular_files
            || self.external_barrel_imports
            || self.barrel_imports_barrel
        {
            let module_mappings: HashSet<ModuleMapping> =
                self.module_mapping_params
                    .iter()
                    .map(|param_string| {
                        Ok(ModuleMapping::from_param_string(param_string)?)
                    })
                    .collect::<Result<
                        HashSet<ModuleMapping>,
                        ModuleMappingFromParamStringError,
                    >>()?;

            let repository: Repository =
                Repository::new(self.repository_path.try_into()?, module_mappings);

            if run_all || self.circular_modules {
                println!("Running circular module validation");
                let failures = validate_circular_modules(repository.clone())?;

                for failure in failures.iter() {
                    println!("{}", failure);
                }

                println!("\n\nTotal: {}", failures.len());
            }

            if run_all || self.circular_files {
                println!("\nRunning circular file validation");
                let failures = validate_circular_files(repository.clone())?;

                for failure in failures.iter() {
                    println!("{}", failure);
                }

                println!("\n\nTotal: {}", failures.len());
            }

            if run_all || self.external_barrel_imports {
                println!("\nRunning external barrel import validation");
                let failures = validate_external_barrel_imports(repository.clone())?;

                for failure in failures.iter() {
                    println!("{}", failure);
                }

                println!("\n\nTotal: {}", failures.len());
            }

            if run_all || self.barrel_imports_barrel {
                println!("\nRunning barrel imports barrel validation");
                let failures = validate_barrel_imports_barrel(repository.clone())?;

                for failure in failures.iter() {
                    println!("{}", failure);
                }

                println!("\n\nTotal: {}", failures.len());
            }
        }

        Ok(())
    }
}
