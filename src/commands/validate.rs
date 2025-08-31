use belmarsh::{depenendency:: Dependency, file_path::FilePath, module::Module, repository::{child::{RepositoryChildPath, RepositoryChildPathFromImportPathError, RepositoryChildPathModuleError}, file::{RepositoryFileModuleError, RepositoryFileResolveImportsError}, Repository, RepositoryFilesError, RepositoryFromStringError}};
use clap::{Args, command};
use rayon::prelude::*;

#[derive(Args, Debug)]
#[command(about = "Validate")]
pub struct ValidateCommand {
    repository_path: String,
}

#[derive(Debug)]
pub enum ValidateCommandError {
    CouldNotParseRepository(RepositoryFromStringError),
    CouldNotAnalyzeFile(RepositoryFilesError),
    CannotGetModuleForRepositoryFile(RepositoryFileModuleError),
    InvalidModule(RepositoryChildPathModuleError),
    InvalidImport(RepositoryChildPathFromImportPathError, FilePath),
    CannotResolveImports(RepositoryFileResolveImportsError),
}

impl From<RepositoryFilesError> for ValidateCommandError {
    fn from(value: RepositoryFilesError) -> Self {
        ValidateCommandError::CouldNotAnalyzeFile(value)
    }
}

impl From<RepositoryFileModuleError> for ValidateCommandError {
    fn from(err: RepositoryFileModuleError) -> Self {
        ValidateCommandError::CannotGetModuleForRepositoryFile(err)
    }
}

impl From<RepositoryFromStringError> for ValidateCommandError {
    fn from(err: RepositoryFromStringError) -> Self {
        ValidateCommandError::CouldNotParseRepository(err)
    }
}

impl From<RepositoryChildPathModuleError> for ValidateCommandError {
    fn from(err: RepositoryChildPathModuleError) -> Self {
        ValidateCommandError::InvalidModule(err)
    }
}

impl From<RepositoryFileResolveImportsError> for ValidateCommandError {
    fn from(err: RepositoryFileResolveImportsError) -> Self {
        ValidateCommandError::CannotResolveImports(err)
    }
}

impl ValidateCommand {
    pub fn run(self) -> Result<(), ValidateCommandError> {
        let repository: Repository = self.repository_path.try_into()?;

        let dependencies = repository
            .files()
            .map(|analyzed_file_result| -> Result<Vec<Dependency<Module, Module>>, ValidateCommandError> {
                let analyzed_file = match analyzed_file_result {
                    Ok(file) => file,
                    Err(e) => match e {
                        RepositoryFilesError::CannotAnalyzeFile(_) => return Ok(vec![]),
                        _ => return Err(e.into()),
                    },
                };

                let current_module = analyzed_file.module()?;

                analyzed_file
                    .imports()?
                    .into_iter()
                    .map(|import_path| {
                        let imported_module = RepositoryChildPath::from_import_path(
                            import_path,
                            &analyzed_file,
                        )
                        .map_err(|e| {
                            ValidateCommandError::InvalidImport(
                                e,
                                analyzed_file.file_path().clone(),
                            )
                        })?
                        .module()?;

                        Ok(Dependency::create(
                            current_module.clone(),
                            imported_module.clone(),
                        ))
                    })
                    .filter(|dependency_result| {
                        dependency_result
                            .as_ref()
                            .map_or(true, |d| !d.is_internal())
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<Vec<_>>, _>>()
            .map(|vecs| vecs.into_iter().flatten().collect::<Vec<_>>())?;

        println!("{:?}", dependencies);

        Ok(())
    }
}
