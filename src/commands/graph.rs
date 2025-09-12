use belmarsh::{
    dependency::list::{DependencyList, DependencyListFromRepositoryError},
    module::Module,
    repository::{Repository, RepositoryFromStringError},
};
use clap::{Args, command};

#[derive(Args, Debug)]
#[command(about = "Output a dependency graph in the dot format")]
pub struct GraphCommand {
    repository_path: String,
}

#[derive(Debug)]
pub enum GraphCommandError {
    CouldNotParseRepository(RepositoryFromStringError),
    CouldNotGetDependencies(DependencyListFromRepositoryError),
}

impl From<RepositoryFromStringError> for GraphCommandError {
    fn from(err: RepositoryFromStringError) -> Self {
        GraphCommandError::CouldNotParseRepository(err)
    }
}

impl From<DependencyListFromRepositoryError> for GraphCommandError {
    fn from(value: DependencyListFromRepositoryError) -> Self {
        GraphCommandError::CouldNotGetDependencies(value)
    }
}

impl GraphCommand {
    pub fn run(self) -> Result<(), GraphCommandError> {
        let repository: Repository = self.repository_path.try_into()?;
        let dependencies: DependencyList<Module, Module> = repository.try_into()?;

        println!("digraph G {{");

        for dependency in dependencies.as_ref().iter() {
            println!("{}", dependency.to_dot_format());
        }

        println!("}}");

        Ok(())
    }
}
