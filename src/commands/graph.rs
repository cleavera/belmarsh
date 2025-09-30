use belmarsh::{
    dependency::list::{DependencyList, DependencyListFromRepositoryError},
    module::Module,
    module_mapping::{ModuleMappings, ModuleMappingsFromParamStringsError},
    repository::{Repository, RepositoryFromStringError, path::RepositoryPathFromStringError},
};
use clap::{Args, command};

#[derive(Args, Debug)]
#[command(about = "Output a dependency graph in the dot format")]
pub struct GraphCommand {
    repository_path: String,

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
pub enum GraphCommandError {
    CouldNotParseRepository(RepositoryFromStringError),
    CouldNotGetDependencies(DependencyListFromRepositoryError),
    CouldNotCreateRepositoryPath(RepositoryPathFromStringError),
    CouldNotParseModuleMapCollection(ModuleMappingsFromParamStringsError),
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

impl From<RepositoryPathFromStringError> for GraphCommandError {
    fn from(err: RepositoryPathFromStringError) -> Self {
        GraphCommandError::CouldNotCreateRepositoryPath(err)
    }
}

impl From<ModuleMappingsFromParamStringsError> for GraphCommandError {
    fn from(err: ModuleMappingsFromParamStringsError) -> Self {
        GraphCommandError::CouldNotParseModuleMapCollection(err)
    }
}

impl GraphCommand {
    pub fn run(self) -> Result<(), GraphCommandError> {
        let module_mappings: ModuleMappings =
            ModuleMappings::from_param_strings(self.module_mapping)?;

        let repository: Repository = Repository::new(
            self.repository_path.try_into()?,
            module_mappings,
            self.skip_folders,
        );
        let dependencies: DependencyList<Module, Module> = repository.try_into()?;

        println!("digraph G {{");

        for dependency in dependencies.as_ref().iter() {
            println!("{}", dependency.to_dot_format());
        }

        println!("}}");

        Ok(())
    }
}
