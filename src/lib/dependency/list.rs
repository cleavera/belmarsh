use rayon::{iter::Either, prelude::*};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;

use crate::{
    dependency::{Dependency, chain::DependencyChain},
    module::Module,
    repository::{
        Repository, RepositoryFilesError,
        child::{
            RepositoryChildPath, RepositoryChildPathFromImportPathError,
            RepositoryChildPathFromPathError, RepositoryChildPathFromRepositoryFileError,
            RepositoryChildPathModuleError,
        },
        file::{RepositoryFile, RepositoryFileModuleError, RepositoryFileResolveImportsError},
    },
};

use super::chain::DependencyChainListBuilder;

#[derive(Debug)]
pub struct DependencyList<TFrom: Display, TTo: Display>(HashSet<Dependency<TFrom, TTo>>);

impl<TFrom: Display, TTo: Display> From<HashSet<Dependency<TFrom, TTo>>>
    for DependencyList<TFrom, TTo>
{
    fn from(value: HashSet<Dependency<TFrom, TTo>>) -> Self {
        DependencyList(value)
    }
}

impl<TFrom: Display, TTo: Display> AsRef<HashSet<Dependency<TFrom, TTo>>>
    for DependencyList<TFrom, TTo>
{
    fn as_ref(&self) -> &HashSet<Dependency<TFrom, TTo>> {
        &self.0
    }
}

impl<TFrom: Display + Clone + Eq + Hash, TTo: Display + Clone + Eq + Hash> DependencyList<TFrom, TTo> {
    pub fn group_by_from(&self) -> HashMap<TFrom, Vec<TTo>>
    where
        TFrom: Eq + Hash,
    {
        let mut map: HashMap<TFrom, Vec<TTo>> = HashMap::new();
        for dep in self.0.iter() {
            map.entry(dep.from.clone()).or_default().push(dep.to.clone());
        }
        map
    }
}

impl<TDependencyType: Clone + Display + Eq + Hash + Send + Sync>
    DependencyList<TDependencyType, TDependencyType>
{
    pub fn to_dependency_chain_list(&self) -> HashSet<DependencyChain> {
        let grouped_by_from = self.group_by_from();
        let string_grouped_dependencies: HashMap<String, Vec<String>> = grouped_by_from
            .into_iter()
            .map(|(from, to_list)| {
                (
                    from.to_string(),
                    to_list.into_iter().map(|to| to.to_string()).collect(),
                )
            })
            .collect();

        DependencyChainListBuilder::build(string_grouped_dependencies)
    }
}

#[derive(Debug)]
pub enum DependencyListFromRepositoryFileError {
    InvalidImports(Vec<RepositoryChildPathFromImportPathError>),
    CouldNotScanFile(RepositoryFilesError),
    CouldNotReadImports(RepositoryFileResolveImportsError),
    CouldNotLocateFileWithinRepository(RepositoryChildPathFromRepositoryFileError),
}

impl From<RepositoryFileResolveImportsError> for DependencyListFromRepositoryFileError {
    fn from(value: RepositoryFileResolveImportsError) -> Self {
        DependencyListFromRepositoryFileError::CouldNotReadImports(value)
    }
}

impl From<RepositoryChildPathFromRepositoryFileError> for DependencyListFromRepositoryFileError {
    fn from(value: RepositoryChildPathFromRepositoryFileError) -> Self {
        DependencyListFromRepositoryFileError::CouldNotLocateFileWithinRepository(value)
    }
}

impl TryFrom<RepositoryFile> for DependencyList<RepositoryChildPath, RepositoryChildPath> {
    type Error = DependencyListFromRepositoryFileError;

    fn try_from(analyzed_file: RepositoryFile) -> Result<Self, Self::Error> {
        let repository_child_path = RepositoryChildPath::from_repository_file(&analyzed_file)?;
        let (dependencies, errors): (
            Vec<
                Result<
                    Dependency<RepositoryChildPath, RepositoryChildPath>,
                    RepositoryChildPathFromImportPathError,
                >,
            >,
            Vec<
                Result<
                    Dependency<RepositoryChildPath, RepositoryChildPath>,
                    RepositoryChildPathFromImportPathError,
                >,
            >,
        ) = analyzed_file
            .imports()?
            .into_iter()
            .map(
                |import_path| -> Result<
                    Dependency<RepositoryChildPath, RepositoryChildPath>,
                    RepositoryChildPathFromImportPathError,
                > {
                    RepositoryChildPath::from_import_path(import_path, &analyzed_file).map(
                        |imported_file| {
                            Dependency::create(repository_child_path.clone(), imported_file)
                        },
                    )
                },
            )
            .filter(|dependency_result| match dependency_result {
                Ok(_) => true,
                Err(RepositoryChildPathFromImportPathError::Path(e)) => match e {
                    RepositoryChildPathFromPathError::ImportOutsideRoot(_) => false,
                },
            })
            .partition(|result| result.is_ok());

        if !errors.is_empty() {
            return Err(DependencyListFromRepositoryFileError::InvalidImports(
                errors.into_iter().map(|r| r.unwrap_err()).collect(),
            ));
        }

        Ok(dependencies
            .into_iter()
            .map(|r| r.unwrap())
            .collect::<HashSet<Dependency<RepositoryChildPath, RepositoryChildPath>>>()
            .into())
    }
}

#[derive(Debug)]
pub enum DependencyListFromRepositoryError {
    InvalidFiles(Vec<DependencyListFromRepositoryAnalyzeFileError>),
}

#[derive(Debug)]
pub enum DependencyListFromRepositoryAnalyzeFileError {
    CouldNotScanFile(RepositoryFilesError),
    CouldNotGetModule(RepositoryFileModuleError),
    CouldNotConvertFilePathToModule(RepositoryChildPathModuleError),
    CouldNotGetDependencyList(DependencyListFromRepositoryFileError),
}

impl From<RepositoryFilesError> for DependencyListFromRepositoryAnalyzeFileError {
    fn from(value: RepositoryFilesError) -> Self {
        DependencyListFromRepositoryAnalyzeFileError::CouldNotScanFile(value)
    }
}

impl From<RepositoryFileModuleError> for DependencyListFromRepositoryAnalyzeFileError {
    fn from(value: RepositoryFileModuleError) -> Self {
        DependencyListFromRepositoryAnalyzeFileError::CouldNotGetModule(value)
    }
}

impl From<RepositoryChildPathModuleError> for DependencyListFromRepositoryAnalyzeFileError {
    fn from(value: RepositoryChildPathModuleError) -> Self {
        DependencyListFromRepositoryAnalyzeFileError::CouldNotConvertFilePathToModule(value)
    }
}

impl From<DependencyListFromRepositoryFileError> for DependencyListFromRepositoryAnalyzeFileError {
    fn from(value: DependencyListFromRepositoryFileError) -> Self {
        DependencyListFromRepositoryAnalyzeFileError::CouldNotGetDependencyList(value)
    }
}

impl TryFrom<Repository> for DependencyList<Module, Module> {
    type Error = DependencyListFromRepositoryError;

    fn try_from(repository: Repository) -> Result<Self, Self::Error> {
        let (dependencies, errors): (
            Vec<Vec<Dependency<Module, Module>>>,
            Vec<DependencyListFromRepositoryAnalyzeFileError>,
        ) = repository
            .files()
            .map(
                |analyzed_file_result| -> Result<
                    Vec<Dependency<Module, Module>>,
                    DependencyListFromRepositoryAnalyzeFileError,
                > {
                    let analyzed_file = match analyzed_file_result {
                        Ok(file) => file,
                        Err(e) => match e {
                            RepositoryFilesError::CannotAnalyzeFile(_) => return Ok(vec![]),
                            _ => return Err(e.into()),
                        },
                    };

                    let dependencies: DependencyList<RepositoryChildPath, RepositoryChildPath> =
                        match analyzed_file.try_into() {
                            Ok(d) => d,
                            Err(e) => return Err(e.into()),
                        };

                    dependencies
                        .as_ref()
                        .iter()
                        .map(
                            |d| -> Result<
                                Dependency<Module, Module>,
                                DependencyListFromRepositoryAnalyzeFileError,
                            > {
                                let (from_result, to_result) = (d.from.module(), d.to.module());

                                let from = match from_result {
                                    Ok(from_module) => from_module,
                                    Err(e) => return Err(e.into()),
                                };

                                let to = match to_result {
                                    Ok(to_module) => to_module,
                                    Err(e) => return Err(e.into()),
                                };

                                Ok(Dependency::create(from, to))
                            },
                        )
                        .filter(|dependency_result| match dependency_result {
                            Ok(dependency) => !dependency.is_internal(),
                            Err(_) => true,
                        })
                        .collect::<Result<
                            Vec<Dependency<Module, Module>>,
                            DependencyListFromRepositoryAnalyzeFileError,
                        >>()
                },
            )
            .partition_map(|result| match result {
                Ok(deps) => Either::Left(deps),
                Err(e) => Either::Right(e),
            });

        if !errors.is_empty() {
            return Err(DependencyListFromRepositoryError::InvalidFiles(errors));
        }

        Ok(dependencies
            .into_iter()
            .flatten()
            .collect::<HashSet<Dependency<Module, Module>>>()
            .into())
    }
}
