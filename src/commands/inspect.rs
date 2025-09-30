use std::collections::HashSet;

use belmarsh::{
    dependency::Dependency,
    dependency::list::{DependencyList, DependencyListFromRepositoryError},
    module::Module,
    module_mapping::{ModuleMappings, ModuleMappingsFromParamStringsError},
    repository::{
        Repository, RepositoryFromStringError, child::RepositoryChildPath,
        path::RepositoryPathFromStringError,
    },
};
use clap::{Args, command};

#[derive(Args, Debug)]
#[command(about = "Lists all the files that draw in a non-internal dependency")]
pub struct InspectCommand {
    repository_path: String,

    #[arg(long)]
    filter_from: Option<String>,

    #[arg(
        long,
        help = "Folders to skip when walking the repository (e.g. node_modules)",
        value_name = "FOLDER_NAME",
        default_value = "node_modules"
    )]
    skip_folders: Vec<String>,

    #[arg(
        long,
        help = "Add a module mapping e.g. --module-mapping @prefix:./path/to/modules",
        value_name = "ALIAS:PATH"
    )]
    module_mapping: Vec<String>,
}

#[derive(Debug)]
pub enum InspectCommandError {
    CouldNotParseRepository(RepositoryFromStringError),
    CouldNotGetDependencies(DependencyListFromRepositoryError),
    CouldNotCreateRepositoryPath(RepositoryPathFromStringError),
    CouldNotParseModuleMapCollection(ModuleMappingsFromParamStringsError),
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

impl From<RepositoryPathFromStringError> for InspectCommandError {
    fn from(err: RepositoryPathFromStringError) -> Self {
        InspectCommandError::CouldNotCreateRepositoryPath(err)
    }
}

impl From<ModuleMappingsFromParamStringsError> for InspectCommandError {
    fn from(err: ModuleMappingsFromParamStringsError) -> Self {
        InspectCommandError::CouldNotParseModuleMapCollection(err)
    }
}

impl InspectCommand {
    pub fn run(self) -> Result<(), InspectCommandError> {
        let module_mappings: ModuleMappings =
            ModuleMappings::from_param_strings(self.module_mapping)?;

        let repository: Repository = Repository::new(
            self.repository_path.try_into()?,
            module_mappings,
            self.skip_folders,
        );
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
