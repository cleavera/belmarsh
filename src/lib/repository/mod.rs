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
        })
    }
}

impl TryFrom<&str> for Repository {
    type Error = RepositoryFromStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Repository {
            path: value.try_into()?,
            mappings: HashSet::new().into(),
        })
    }
}

impl Repository {
    pub fn new(path: RepositoryPath, mappings: ModuleMappings) -> Self {
        Repository { path, mappings }
    }

    pub fn files(
        &self,
    ) -> rayon::iter::Map<
        rayon::iter::IterBridge<walkdir::IntoIter>,
        impl Fn(
            Result<walkdir::DirEntry, walkdir::Error>,
        ) -> Result<RepositoryFile, RepositoryFilesError>,
    > {
        let mappings = self.mappings.clone();
        WalkDir::new(self.path.as_ref())
            .into_iter()
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
