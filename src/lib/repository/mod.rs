pub mod child;
pub mod file;
pub mod path;

use crate::module_mapping::ModuleMappings;
use file::{RepositoryFile, RepositoryFileFromEntryError};
use path::{RepositoryPath, RepositoryPathFromStringError};
use rayon::prelude::*;
use std::collections::HashSet;
use walkdir::WalkDir;

#[derive(Debug)]
pub enum RepositoryFilesError {
    CannotScanFiles(walkdir::Error),
    CannotAnalyzeFile(RepositoryFileFromEntryError),
}

impl From<RepositoryFileFromEntryError> for RepositoryFilesError {
    fn from(value: RepositoryFileFromEntryError) -> Self {
        RepositoryFilesError::CannotAnalyzeFile(value)
    }
}

impl From<walkdir::Error> for RepositoryFilesError {
    fn from(value: walkdir::Error) -> Self {
        RepositoryFilesError::CannotScanFiles(value)
    }
}

#[derive(Debug, Clone)]
pub struct Repository {
    path: RepositoryPath,
    mappings: ModuleMappings,
    skip_folders: Vec<String>,
}

#[derive(Debug)]
pub enum RepositoryFromStringError {
    CouldNotCreateRepositoryPath(RepositoryPathFromStringError),
}

impl From<RepositoryPathFromStringError> for RepositoryFromStringError {
    fn from(value: RepositoryPathFromStringError) -> Self {
        RepositoryFromStringError::CouldNotCreateRepositoryPath(value)
    }
}

impl TryFrom<String> for Repository {
    type Error = RepositoryFromStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Repository {
            path: value.try_into()?,
            mappings: HashSet::new().into(),
            skip_folders: Vec::new(),
        })
    }
}

impl TryFrom<&str> for Repository {
    type Error = RepositoryFromStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Repository {
            path: value.try_into()?,
            mappings: HashSet::new().into(),
            skip_folders: Vec::new(),
        })
    }
}

impl Repository {
    pub fn new(path: RepositoryPath, mappings: ModuleMappings, skip_folders: Vec<String>) -> Self {
        Repository {
            path,
            mappings,
            skip_folders,
        }
    }

    pub fn files(
        &self,
    ) -> impl ParallelIterator<Item = Result<RepositoryFile, RepositoryFilesError>> {
        let mappings = self.mappings.clone();
        let skip_folders = self.skip_folders.clone();
        WalkDir::new(self.path.as_ref())
            .into_iter()
            .filter_entry(move |entry| {
                !entry
                    .file_name()
                    .to_str()
                    .map(|s| skip_folders.contains(&s.to_string()))
                    .unwrap_or(false)
            })
            .par_bridge()
            .map(
                move |entry| -> Result<RepositoryFile, RepositoryFilesError> {
                    Ok(RepositoryFile::try_from_entry(
                        entry?,
                        &self.path,
                        mappings.clone(),
                    )?)
                },
            )
    }
}
