use crate::file_parent_path::FileParentPath;
use crate::file_path::{FilePath, FilePathContentsError, FilePathFromEntryError};
use crate::import_path::{ImportPath, ImportPathFromImportStringError};
use crate::module::Module;
use crate::module_mapping::ModuleMapping;
use crate::repository::{
    child::{
        RepositoryChildPath, RepositoryChildPathFromFilePathError, RepositoryChildPathModuleError,
    },
    path::RepositoryPath,
};
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use regex::Regex;
use std::collections::HashSet;
use std::{io::BufRead, path::PathBuf};

#[derive(Debug)]
pub enum RepositoryFileFromEntryError {
    FilePathError(FilePathFromEntryError),
}

impl From<FilePathFromEntryError> for RepositoryFileFromEntryError {
    fn from(value: FilePathFromEntryError) -> Self {
        RepositoryFileFromEntryError::FilePathError(value)
    }
}

#[derive(Debug)]
pub enum RepositoryFileModuleError {
    Module(RepositoryChildPathModuleError),
    FilePath(RepositoryChildPathFromFilePathError),
}

impl From<RepositoryChildPathModuleError> for RepositoryFileModuleError {
    fn from(value: RepositoryChildPathModuleError) -> Self {
        RepositoryFileModuleError::Module(value)
    }
}

impl From<RepositoryChildPathFromFilePathError> for RepositoryFileModuleError {
    fn from(value: RepositoryChildPathFromFilePathError) -> Self {
        RepositoryFileModuleError::FilePath(value)
    }
}

#[derive(Debug)]
pub enum RepositoryFileResolveImportsError {
    Io(std::io::Error, PathBuf),
    ParseImportPath(ImportPathFromImportStringError, FilePath),
    CannotGetContents(FilePathContentsError),
}

impl From<FilePathContentsError> for RepositoryFileResolveImportsError {
    fn from(value: FilePathContentsError) -> Self {
        RepositoryFileResolveImportsError::CannotGetContents(value)
    }
}

#[derive(Debug)]
pub struct RepositoryFile {
    file_path: FilePath,
    base_path: RepositoryPath,
    import_mappings: HashSet<ModuleMapping>,

    module: OnceCell<Module>,
    imports: OnceCell<Vec<ImportPath>>,
}

impl AsRef<RepositoryPath> for RepositoryFile {
    fn as_ref(&self) -> &RepositoryPath {
        &self.base_path
    }
}

impl RepositoryFile {
    pub fn try_from_entry(
        entry: walkdir::DirEntry,
        base_path: &RepositoryPath,
        import_mappings: HashSet<ModuleMapping>,
    ) -> Result<Self, RepositoryFileFromEntryError> {
        let file_path: FilePath = entry
            .try_into()
            .map_err(RepositoryFileFromEntryError::FilePathError)?;

        Ok(RepositoryFile {
            file_path,
            base_path: base_path.clone(),
            import_mappings,
            module: OnceCell::new(),
            imports: OnceCell::new(),
        })
    }

    pub fn file_path(&self) -> &FilePath {
        &self.file_path
    }

    pub fn module(&self) -> Result<&Module, RepositoryFileModuleError> {
        self.module.get_or_try_init(|| {
            RepositoryChildPath::from_file_path(&self.file_path, &self.base_path)?
                .module()
                .map_err(RepositoryFileModuleError::from)
        })
    }

    pub fn imports(&self) -> Result<&[ImportPath], RepositoryFileResolveImportsError> {
        lazy_static! {
            static ref IMPORT_REGEX: Regex =
                Regex::new(r"import\s*\{[^}]*\}\s*from\s*'(\.[^']+)';")
                    .expect("Failed to compile regex");
        }

        self.imports
            .get_or_try_init(|| {
                let mut imports = Vec::new();
                let reader = self.file_path.contents()?;
                let parent_dir: FileParentPath = FileParentPath::from_file_path(&self.file_path);

                for line in reader.lines() {
                    let line = line.map_err(|e| {
                        RepositoryFileResolveImportsError::Io(
                            e,
                            self.file_path.as_ref().to_path_buf(),
                        )
                    })?;

                    if let Some(captures) = IMPORT_REGEX.captures(&line) {
                        if let Some(path_capture) = captures.get(1) {
                            let import_path = match ImportPath::from_import_string(
                                path_capture.as_str(),
                                &parent_dir,
                            )
                            .map_err(|e| {
                                RepositoryFileResolveImportsError::ParseImportPath(
                                    e,
                                    self.file_path.clone(),
                                )
                            }) {
                                Ok(path) => path,
                                Err(e) => {
                                    eprintln!("{:?}", e);
                                    continue;
                                }
                            };
                            imports.push(import_path);
                        }
                    }
                }
                Ok(imports)
            })
            .map(|v| v.as_slice())
    }
}
