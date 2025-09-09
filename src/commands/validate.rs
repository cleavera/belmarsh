use std::collections::HashSet;

use belmarsh::{
    dependency::{chain::DependencyChain, list::{DependencyList, DependencyListFromRepositoryError}},
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
        let chains = dependencies.to_dependency_chain_list().into_iter().filter(|chain| chain.is_circular()).collect::<HashSet<DependencyChain<Module>>>();

        println!("{:?} \n \n Count: {:?}", chains, chains.len());

        Ok(())
    }
}
