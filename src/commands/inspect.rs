use std::collections::HashSet;

use belmarsh::{
    dependency::Dependency,
    dependency::list::{DependencyList, DependencyListFromRepositoryError},
    module::Module,
    repository::{Repository, RepositoryFromStringError, child::RepositoryChildPath},
};
use clap::{Args, command};

#[derive(Args, Debug)]
#[command(about = "Lists all the files that draw in a non-internal dependency")]
pub struct InspectCommand {
    repository_path: String,

    #[arg(long)]
    filter_from: Option<String>,
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

        if let Some(filter_module_name) = self.filter_from {
            let filtered_deps_set: HashSet<Dependency<RepositoryChildPath, Module>> = dependencies
                .as_ref()
                .iter()
                .filter(|dep| dep.is_from_module(&filter_module_name))
                .cloned()
                .collect();

            let filtered_deps_list = DependencyList::from(filtered_deps_set);
            println!("{}", filtered_deps_list);
        } else {
            println!("{}", dependencies);
        }

        Ok(())
    }
}
