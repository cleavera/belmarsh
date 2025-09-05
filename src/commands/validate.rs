use belmarsh::{depenendency::{list::{DependencyList, DependencyListFromRepositoryError}, Dependency}, file_path::FilePath, module::Module, repository::{child::{RepositoryChildPath, RepositoryChildPathFromImportPathError, RepositoryChildPathFromPathError, RepositoryChildPathModuleError}, file::{RepositoryFileModuleError, RepositoryFileResolveImportsError}, Repository, RepositoryFilesError, RepositoryFromStringError}};
use clap::{Args, command};
use rayon::prelude::*;

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

        println!("{:?}", dependencies);

        Ok(())
    }
}
