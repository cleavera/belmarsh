use rayon::{iter::Either, prelude::*};
use std::fmt::Display;

use crate::{
    dependency::Dependency,
    module::Module,
    repository::{
        child::{
            RepositoryChildPath, RepositoryChildPathFromImportPathError, RepositoryChildPathFromPathError, RepositoryChildPathFromRepositoryFileError, RepositoryChildPathModuleError
        }, file::{RepositoryFile, RepositoryFileModuleError, RepositoryFileResolveImportsError}, Repository, RepositoryFilesError
    },
};

#[derive(Debug)]
pub struct DependencyList<TFrom: Display, TTo: Display>(Vec<Dependency<TFrom, TTo>>);

impl<TFrom: Display, TTo: Display> From<Vec<Dependency<TFrom, TTo>>>
    for DependencyList<TFrom, TTo>
{
    fn from(value: Vec<Dependency<TFrom, TTo>>) -> Self {
        DependencyList(value)
    }
}

impl<TFrom: Display, TTo: Display> AsRef<Vec<Dependency<TFrom, TTo>>>
    for DependencyList<TFrom, TTo>
{
    fn as_ref(&self) -> &Vec<Dependency<TFrom, TTo>> {
        &self.0
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
                        |imported_file| Dependency::create(repository_child_path.clone(), imported_file),
                    )
                },
            )
            .filter(|dependency_result| match dependency_result {
                Ok(dependency) => !dependency.is_internal(),
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
            .collect::<Vec<Dependency<RepositoryChildPath, RepositoryChildPath>>>()
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
                        .map(|d| -> Result<Dependency<Module, Module>, DependencyListFromRepositoryAnalyzeFileError> {
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
                        })
                        .collect::<Result<Vec<Dependency<Module, Module>>, DependencyListFromRepositoryAnalyzeFileError>>()
                },
            )
            .partition_map(|result| match result {
                Ok(deps) => Either::Left(deps),
                Err(e) => Either::Right(e),
            });

        if !errors.is_empty() {
            return Err(DependencyListFromRepositoryError::InvalidFiles(
                errors
            ));
        }

        Ok(dependencies
            .into_iter()
            .flatten()
            .collect::<Vec<Dependency<Module, Module>>>()
            .into())
    }
}
