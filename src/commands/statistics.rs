use belmarsh::{
    dependency::Dependency,
    file_path::FilePath,
    repository::{
        Repository, RepositoryFilesError, RepositoryFromStringError,
        child::{
            RepositoryChildPath, RepositoryChildPathFromImportPathError,
            RepositoryChildPathModuleError,
        },
        file::{RepositoryFileModuleError, RepositoryFileResolveImportsError},
    },
};
use clap::{Args, command};
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Args, Debug)]
#[command(about = "Generate statistics")]
pub struct StatisticsCommand {
    repository_path: String,
}

#[derive(Debug)]
pub enum StatisticsCommandError {
    CouldNotParseRepository(RepositoryFromStringError),
    CouldNotGetFiles(RepositoryFilesError),
    InvalidImport(RepositoryChildPathFromImportPathError, FilePath),
    InvalidModule(RepositoryChildPathModuleError),
    CannotGetModuleForRepositoryFile(RepositoryFileModuleError),
    CannotResolveImports(RepositoryFileResolveImportsError),
}

impl From<RepositoryFilesError> for StatisticsCommandError {
    fn from(value: RepositoryFilesError) -> Self {
        StatisticsCommandError::CouldNotGetFiles(value)
    }
}

impl From<RepositoryFromStringError> for StatisticsCommandError {
    fn from(value: RepositoryFromStringError) -> Self {
        StatisticsCommandError::CouldNotParseRepository(value)
    }
}

impl From<RepositoryFileModuleError> for StatisticsCommandError {
    fn from(err: RepositoryFileModuleError) -> Self {
        StatisticsCommandError::CannotGetModuleForRepositoryFile(err)
    }
}

impl From<RepositoryFileResolveImportsError> for StatisticsCommandError {
    fn from(value: RepositoryFileResolveImportsError) -> Self {
        StatisticsCommandError::CannotResolveImports(value)
    }
}

impl From<RepositoryChildPathModuleError> for StatisticsCommandError {
    fn from(value: RepositoryChildPathModuleError) -> Self {
        StatisticsCommandError::InvalidModule(value)
    }
}

impl StatisticsCommand {
    pub fn run(self) -> Result<(), StatisticsCommandError> {
        let repository: Repository = self.repository_path.try_into()?;

        let file_check_count = AtomicUsize::new(0);
        let counts: Result<Vec<usize>, StatisticsCommandError> = repository
            .files()
            .map(
                |analyzed_file_result| -> Result<usize, StatisticsCommandError> {
                    file_check_count.fetch_add(1, Ordering::SeqCst);

                    let analyzed_file = match analyzed_file_result {
                        Ok(file) => file,
                        Err(e) => match e {
                            RepositoryFilesError::CannotAnalyzeFile(_) => return Ok(0),
                            _ => return Err(e.into()),
                        },
                    };

                    let mut count = 0;
                    let current_module = analyzed_file.module()?;

                    for import_path in analyzed_file.imports()? {
                        let imported_module = match RepositoryChildPath::from_import_path(
                            import_path,
                            &analyzed_file,
                        )
                        .map_err(|e| {
                            StatisticsCommandError::InvalidImport(
                                e,
                                analyzed_file.file_path().clone(),
                            )
                        }) {
                            Ok(repository_child_path) => repository_child_path.module()?,
                            Err(e) => {
                                eprintln!("{:?}", e);
                                continue;
                            }
                        };

                        let dependency = Dependency::create(current_module, &imported_module);

                        if !dependency.is_internal() {
                            count = count + 1;
                        }
                    }
                    Ok(count)
                },
            )
            .collect();

        let total_count = counts?.into_iter().sum::<usize>();
        let total_files_checked = file_check_count.load(Ordering::SeqCst);

        println!("Total imports from outside own modules: {}", total_count);
        println!("Total files checked: {}", total_files_checked);

        Ok(())
    }
}
