use belmarsh::{depenendency:: Dependency, file_path::FilePath, module::Module, repository::{child::{RepositoryChildPath, RepositoryChildPathFromImportPathError, RepositoryChildPathFromPathError, RepositoryChildPathModuleError}, file::{RepositoryFileModuleError, RepositoryFileResolveImportsError}, Repository, RepositoryFilesError, RepositoryFromStringError}};
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
    InvalidImport(RepositoryChildPathFromImportPathError),
    InvalidImports(Vec<ValidateCommandError>, FilePath),
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

                let (dependencies, errors): (Vec<Result<Dependency<Module, Module>, ValidateCommandError>>, Vec<Result<Dependency<Module, Module>, ValidateCommandError>>) = analyzed_file
                    .imports()?
                    .into_iter()
                    .map(|import_path| {
                        RepositoryChildPath::from_import_path(
                            import_path,
                            &analyzed_file,
                        )
                        .map_err(|e| ValidateCommandError::InvalidImport(e))
                        .and_then(|repository_child| repository_child.module().map_err(|e| ValidateCommandError::InvalidModule(e)))
                        .map(|imported_module| Dependency::create(current_module.clone(), imported_module))
                    })
                    .filter(|dependency_result| {
                        match dependency_result {
                            Ok(depenendency) => !depenendency.is_internal(),
                            Err(ValidateCommandError::InvalidImport(RepositoryChildPathFromImportPathError::Path(e))) => match e {
                                RepositoryChildPathFromPathError::ImportOutsideRoot(_) => false,
                            },
                            Err(_) => true
                        }
                    })
                    .partition(|result| result.is_ok());

                if !errors.is_empty() {
                    return Err(ValidateCommandError::InvalidImports(errors.into_iter().map(|r| r.unwrap_err()).collect(), analyzed_file.file_path().clone()));
                }

                Ok(dependencies.into_iter().map(|r| r.unwrap()).collect())
            })
            .collect::<Result<Vec<Vec<_>>, _>>()
            .map(|vecs| vecs.into_iter().flatten().collect::<Vec<_>>())?;

        println!("{:?}", dependencies);

        Ok(())
    }
}
