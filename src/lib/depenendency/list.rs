use std::fmt::Display;
use rayon::prelude::*;

use crate::{depenendency::Dependency, file_path::FilePath, module::Module, repository::{child::{RepositoryChildPath, RepositoryChildPathFromImportPathError, RepositoryChildPathFromPathError, RepositoryChildPathModuleError}, file::{RepositoryFile, RepositoryFileModuleError}, Repository, RepositoryFilesError}};

pub struct DependencyList<TFrom: Display, TTo: Display>(Vec<Dependency<TFrom, TTo>>);

impl<TFrom: Display, TTo: Display> From<Vec<Dependency<TFrom, TTo>>> for DependencyList<TFrom, TTo> {
    fn from(value: Vec<Dependency<TFrom, TTo>>) -> Self {
        DependencyList(value)
    }
}

#[derive(Debug)]
pub enum DependencyListFromRepositoryFileError {
    InvalidImports(Vec<RepositoryChildPathFromImportPathError>),
    CouldNotScanFile(RepositoryFilesError),
}


impl TryFrom<RepositoryFile> for DependencyList<FilePath, FilePath> {
    type Error = DependencyListFromRepositoryFileError;

    fn try_from(analyzed_file: RepositoryFile) -> Result<Self, Self::Error> {
                let (dependencies, errors): (Vec<Result<Dependency<FilePath, FilePath>, DependencyListFromRepositoryFileError>>, Vec<Result<Dependency<FilePath, FilePath>, DependencyListFromRepositoryFileError>>) = analyzed_file
                    .imports()?
                    .into_iter()
                    .map(|import_path| {
                        RepositoryChildPath::from_import_path(
                            import_path,
                            &analyzed_file,
                        )
                        .map(|imported_file| Dependency::create(analyzed_file.file_path().clone(), imported_file))
                    })
                    .filter(|dependency_result| {
                        match dependency_result {
                            Ok(depenendency) => !depenendency.is_internal(),
                            Err(RepositoryChildPathFromImportPathError::Path(e)) => match e {
                                RepositoryChildPathFromPathError::ImportOutsideRoot(_) => false,
                            },
                        }
                    })
                    .partition(|result| result.is_ok());

                if !errors.is_empty() {
                    return Err(DependencyListFromRepositoryFileError::InvalidImports(errors.into_iter().map(|r| r.unwrap_err()).collect()));
                }

                Ok(dependencies.into_iter().map(|r| r.unwrap()).collect::<DependencyList<FilePath, FilePath>>())
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

impl From<DependencyListFromRepositoryFileError> for DependencyListFromRepositoryAnalyzeFileError {
    fn from(value: DependencyListFromRepositoryFileError) -> Self {
        DependencyListFromRepositoryAnalyzeFileError::CouldNotGetDependencyList(value)
    }
}

impl TryFrom<Repository> for DependencyList<Module, Module> {
    type Error = DependencyListFromRepositoryError;

    fn try_from(repository: Repository) -> Result<Self, Self::Error> {
                let (dependencies, errors): (Vec<Result<Dependency<Module, Module>, DependencyListFromRepositoryAnalyzeFileError>>, Vec<Result<Dependency<Module, Module>, DependencyListFromRepositoryAnalyzeFileError>>) = repository
            .files()
            .map(|analyzed_file_result| -> Result<Vec<Dependency<Module, Module>>, DependencyListFromRepositoryAnalyzeFileError> {
                let analyzed_file = match analyzed_file_result {
                    Ok(file) => file,
                    Err(e) => match e {
                        RepositoryFilesError::CannotAnalyzeFile(_) => return Ok(vec![]),
                        _ => return Err(e.into()),
                    },
                };

                let current_module = analyzed_file.module()?;
                let dependencies: DependencyList<FilePath, FilePath> =  match analyzed_file.try_into() {
                    Ok(d) => d,
                    Err(e) => return Err(e.into()),
                };

                todo!()
            }).collect::<Vec<Result<Vec<Dependency<Module, Module>>, DependencyListFromRepositoryAnalyzeFileError>>>().into_iter()
                    .partition(|result| result.is_ok());

                if !errors.is_empty() {
                    return Err(DependencyListFromRepositoryError::InvalidFiles(errors.into_iter().map(|r| r.unwrap_err()).collect()));
                }

                Ok(dependencies.into_iter().map(|r| r.unwrap()).collect::<Vec<Dependency<Module, Module>>>().into())
    }
}
