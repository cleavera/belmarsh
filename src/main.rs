use belmarsh::{
    file_path::FilePath,
    repository::{
        Repository,
        RepositoryFilesError,
        RepositoryFromStringError,
        child::{RepositoryChildPath, RepositoryChildPathFromImportPathError, RepositoryChildPathModuleError},
        file::{RepositoryFileModuleError, RepositoryFileResolveImportsError},
    },
};
use rayon::prelude::*;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

#[derive(Debug)]
pub enum BelmarshError {
    CouldNotParseRepository(RepositoryFromStringError),
    CouldNotGetFiles(RepositoryFilesError),
    InvalidImport(RepositoryChildPathFromImportPathError, FilePath),
    InvalidModule(RepositoryChildPathModuleError),
    CannotGetModuleForRepositoryFile(RepositoryFileModuleError),
    CannotResolveImports(RepositoryFileResolveImportsError),
}

impl From<RepositoryFilesError> for BelmarshError {
    fn from(value: RepositoryFilesError) -> Self {
        BelmarshError::CouldNotGetFiles(value)
    }
}

impl From<RepositoryFromStringError> for BelmarshError {
    fn from(value: RepositoryFromStringError) -> Self {
        BelmarshError::CouldNotParseRepository(value)
    }
}

impl From<RepositoryFileModuleError> for BelmarshError {
    fn from(err: RepositoryFileModuleError) -> Self {
        BelmarshError::CannotGetModuleForRepositoryFile(err)
    }
}

impl From<RepositoryFileResolveImportsError> for BelmarshError {
    fn from(value: RepositoryFileResolveImportsError) -> Self {
        BelmarshError::CannotResolveImports(value)
    }
}

impl From<RepositoryChildPathModuleError> for BelmarshError {
    fn from(value: RepositoryChildPathModuleError) -> Self {
        BelmarshError::InvalidModule(value)
    }
}

fn main() -> Result<(), BelmarshError> {
    let start = Instant::now();

    let repository: Repository = "../MHR.Web.Apps.PeopleFirst/src/app".try_into()?;

    let file_check_count = AtomicUsize::new(0);
    let counts: Result<Vec<usize>, BelmarshError> = repository
        .files()
        .map(|analyzed_file_result| -> Result<usize, BelmarshError> {
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
                .map_err(|e| BelmarshError::InvalidImport(e, analyzed_file.file_path().clone()))
                {
                    Ok(repository_child_path) => repository_child_path.module()?,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        continue;
                    }
                };

                if current_module != &imported_module {
                    count = count + 1;
                }
            }
            Ok(count)
        })
        .collect();

    let total_count = counts?.into_iter().sum::<usize>();
    let total_files_checked = file_check_count.load(Ordering::SeqCst);

    println!("Total imports from outside own modules: {}", total_count);
    println!("Total files checked: {}", total_files_checked);

    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);

    Ok(())
}
