use crate::{
    file_path::FilePath,
    import_path::ImportPath,
    module::{Module, ModuleFromComponentError},
    repository::{path::{RepositoryPath}}
};

use std::{
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum RepositoryChildPathModuleError {
    CouldNotGetModule(String),
    ModuleConversionError(ModuleFromComponentError),
}

impl From<RepositoryChildPathFromFilePathError> for RepositoryChildPathModuleError {
    fn from(value: RepositoryChildPathFromFilePathError) -> Self {
        RepositoryChildPathModuleError::CouldNotGetModule(format!(
            "Could not get module from file path error: {:?}",
            value
        ))
    }
}

impl From<ModuleFromComponentError> for RepositoryChildPathModuleError {
    fn from(err: ModuleFromComponentError) -> Self {
        RepositoryChildPathModuleError::ModuleConversionError(err)
    }
}

#[derive(Debug)]
pub enum RepositoryChildPathFromImportPathError {
    Path(RepositoryChildPathFromPathError),
}

impl From<RepositoryChildPathFromPathError> for RepositoryChildPathFromImportPathError {
    fn from(value: RepositoryChildPathFromPathError) -> Self {
        RepositoryChildPathFromImportPathError::Path(value)
    }
}

#[derive(Debug)]
pub enum RepositoryChildPathFromFilePathError {
    Path(RepositoryChildPathFromPathError),
}

impl From<RepositoryChildPathFromPathError> for RepositoryChildPathFromFilePathError {
    fn from(value: RepositoryChildPathFromPathError) -> Self {
        RepositoryChildPathFromFilePathError::Path(value)
    }
}

#[derive(Debug)]
pub enum RepositoryChildPathFromPathError {
    ImportOutsideRoot(String),
}

pub struct RepositoryChildPath(PathBuf);

impl RepositoryChildPath {
    pub fn from_import_path<TRepositoryPathRef: AsRef<RepositoryPath>>(
        import_path: &ImportPath,
        repository_path: TRepositoryPathRef,
    ) -> Result<RepositoryChildPath, RepositoryChildPathFromImportPathError> {
        Ok(RepositoryChildPath::from_path(
            import_path.as_ref(),
            repository_path.as_ref(),
        )?)
    }

    pub fn from_file_path(
        file_path: &FilePath,
        repository_path: &RepositoryPath,
    ) -> Result<RepositoryChildPath, RepositoryChildPathFromFilePathError> {
        Ok(RepositoryChildPath::from_path(
            file_path.as_ref(),
            repository_path,
        )?)
    }

    fn from_path(
        path: &Path,
        repository_path: &RepositoryPath,
    ) -> Result<RepositoryChildPath, RepositoryChildPathFromPathError> {
        if let Ok(relative_path) = path.strip_prefix(repository_path) {
            Ok(RepositoryChildPath(relative_path.into()))
        } else {
            Err(RepositoryChildPathFromPathError::ImportOutsideRoot(
                path.display().to_string(),
            ))
        }
    }

    pub fn module(&self) -> Result<Module, RepositoryChildPathModuleError> {
        let component = self.0.components().next().ok_or_else(|| {
            RepositoryChildPathModuleError::CouldNotGetModule(self.0.display().to_string())
        })?;

        Ok(Module::try_from(component)?)
    }
}
