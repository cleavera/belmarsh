use belmarsh::{
    dependency::list::{DependencyList, DependencyListFromRepositoryError},
    module::Module,
    repository::{child::RepositoryChildPath, Repository, RepositoryFromStringError},
};
use clap::{Args, command};

#[derive(Args, Debug)]
#[command(about = "Lists all the files that draw in a non-internal dependency")]
pub struct InspectCommand {
    repository_path: String,
}

#[derive(Debug)]
pub enum InspectCommandError {
    CouldNotParseRepository(RepositoryFromStringError),
    CouldNotGetDependencies(DependencyListFromRepositoryError),
}

impl From<RepositoryFromStringError> for InspectCommandError {
    fn from(err: RepositoryFromStringError) -> Self {
        InspectCommandError::CouldNotParseRepository(err)
    }
}

impl From<DependencyListFromRepositoryError> for InspectCommandError {
    fn from(value: DependencyListFromRepositoryError) -> Self {
        InspectCommandError::CouldNotGetDependencies(value)
    }
}

impl InspectCommand {
    pub fn run(self) -> Result<(), InspectCommandError> {
        let repository: Repository = self.repository_path.try_into()?;
        let dependencies: DependencyList<RepositoryChildPath, Module> = repository.try_into()?;

        println!("{}", dependencies);

        Ok(())
    }
}
