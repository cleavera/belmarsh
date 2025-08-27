use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
use walkdir::WalkDir;

#[derive(Debug)]
pub enum BelmarshError {
    CouldNotParseRepository(RepositoryPathFromStringError),
    CouldNotGetFiles(RepositoryFilesError),
    CouldNotParseFilePath(FilePathFromEntryError),
    CouldNotReadFile(FilePathContentsError),
    InvalidFile(RepositoryChildPathFromFilePathError),
    InvalidModule(RepositoryChildPathModuleError),
    InvalidImport(RepositoryChildPathFromImportPathError, FilePath),
    Io(std::io::Error, PathBuf),
    Walkdir(walkdir::Error),
    PathError(String),
    Regex(regex::Error),
    ParseImportPath(ImportPathFromImportStringError, FilePath),
    AnalyzedFileError(AnalyzedFileFromEntryError),
}

impl From<RepositoryFilesError> for BelmarshError {
    fn from(value: RepositoryFilesError) -> Self {
        BelmarshError::CouldNotGetFiles(value)
    }
}

impl From<AnalyzedFileFromEntryError> for BelmarshError {
    fn from(value: AnalyzedFileFromEntryError) -> Self {
        BelmarshError::AnalyzedFileError(value)
    }
}

impl From<RepositoryChildPathModuleError> for BelmarshError {
    fn from(value: RepositoryChildPathModuleError) -> Self {
        BelmarshError::InvalidModule(value)
    }
}

impl From<RepositoryChildPathFromFilePathError> for BelmarshError {
    fn from(value: RepositoryChildPathFromFilePathError) -> Self {
        BelmarshError::InvalidFile(value)
    }
}

impl From<RepositoryPathFromStringError> for BelmarshError {
    fn from(value: RepositoryPathFromStringError) -> Self {
        BelmarshError::CouldNotParseRepository(value)
    }
}

impl From<FilePathFromEntryError> for BelmarshError {
    fn from(value: FilePathFromEntryError) -> Self {
        BelmarshError::CouldNotParseFilePath(value)
    }
}

impl From<FilePathContentsError> for BelmarshError {
    fn from(value: FilePathContentsError) -> Self {
        BelmarshError::CouldNotReadFile(value)
    }
}

impl From<walkdir::Error> for BelmarshError {
    fn from(err: walkdir::Error) -> Self {
        BelmarshError::Walkdir(err)
    }
}

impl From<regex::Error> for BelmarshError {
    fn from(err: regex::Error) -> Self {
        BelmarshError::Regex(err)
    }
}

#[derive(Debug)]
pub enum AnalyzedFileFromEntryError {
    WalkdirError(walkdir::Error),
    FilePathError(FilePathFromEntryError),
    ModuleError(RepositoryChildPathModuleError),
    IoError(std::io::Error, PathBuf),
    ParseImportPath(ImportPathFromImportStringError, FilePath),
    InvalidImport(RepositoryChildPathFromImportPathError, FilePath),
}

impl From<walkdir::Error> for AnalyzedFileFromEntryError {
    fn from(value: walkdir::Error) -> Self {
        AnalyzedFileFromEntryError::WalkdirError(value)
    }
}

impl From<FilePathFromEntryError> for AnalyzedFileFromEntryError {
    fn from(value: FilePathFromEntryError) -> Self {
        AnalyzedFileFromEntryError::FilePathError(value)
    }
}

impl From<RepositoryChildPathModuleError> for AnalyzedFileFromEntryError {
    fn from(value: RepositoryChildPathModuleError) -> Self {
        AnalyzedFileFromEntryError::ModuleError(value)
    }
}

#[derive(Clone, Debug)]
pub struct FilePath(PathBuf);

use once_cell::sync::OnceCell;

pub struct AnalyzedFile {
    file_path: FilePath,
    base_path: RepositoryPath,

    module: OnceCell<Module>,
    imports: OnceCell<Vec<ImportPath>>,
}

impl AnalyzedFile {
    pub fn try_from_entry(
        entry: walkdir::DirEntry,
        base_path: &RepositoryPath,
    ) -> Result<Self, AnalyzedFileFromEntryError> {
        let file_path: FilePath = entry
            .try_into()
            .map_err(AnalyzedFileFromEntryError::FilePathError)?;

        Ok(AnalyzedFile {
            file_path,
            base_path: base_path.clone(),
            module: OnceCell::new(),
            imports: OnceCell::new(),
        })
    }

    pub fn file_path(&self) -> &FilePath {
        &self.file_path
    }

    pub fn module(&self) -> Result<&Module, BelmarshError> {
        self.module.get_or_try_init(|| {
            RepositoryChildPath::from_file_path(&self.file_path, &self.base_path)?
                .module()
                .map_err(BelmarshError::from)
        })
    }

    pub fn imports(&self) -> Result<&[ImportPath], BelmarshError> {
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
                    let line = line
                        .map_err(|e| BelmarshError::Io(e, self.file_path.as_ref().to_path_buf()))?;

                    if let Some(captures) = IMPORT_REGEX.captures(&line) {
                        if let Some(path_capture) = captures.get(1) {
                            let import_path = match ImportPath::from_import_string(
                                path_capture.as_str(),
                                &parent_dir,
                            )
                            .map_err(|e| BelmarshError::ParseImportPath(e, self.file_path.clone()))
                            {
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

#[derive(Debug)]
pub enum FilePathContentsError {
    Io(std::io::Error, PathBuf),
}

impl FilePath {
    pub fn contents(&self) -> Result<BufReader<File>, FilePathContentsError> {
        let file = File::open(self)
            .map_err(|e| FilePathContentsError::Io(e, self.as_ref().to_path_buf()))?;

        Ok(BufReader::new(file))
    }
}

#[derive(Debug)]
pub enum FilePathFromEntryError {
    IncorrectPath(FilePathFromPathBufError),
}

impl From<FilePathFromPathBufError> for FilePathFromEntryError {
    fn from(value: FilePathFromPathBufError) -> Self {
        FilePathFromEntryError::IncorrectPath(value)
    }
}

impl TryFrom<walkdir::DirEntry> for FilePath {
    type Error = FilePathFromEntryError;

    fn try_from(entry: walkdir::DirEntry) -> Result<Self, Self::Error> {
        let path: PathBuf = entry.path().to_path_buf();

        Ok(FilePath::try_from(path).map_err(FilePathFromEntryError::from)?)
    }
}

#[derive(Debug)]
pub enum FilePathFromPathBufError {
    NotAValidFile(PathBuf),
    Io(std::io::Error, PathBuf),
}

impl TryFrom<PathBuf> for FilePath {
    type Error = FilePathFromPathBufError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        if !path.is_file() || path.extension().map_or(true, |ext| ext != "ts") {
            return Err(FilePathFromPathBufError::NotAValidFile(path));
        }

        Ok(FilePath(path.canonicalize().map_err(|e| {
            FilePathFromPathBufError::Io(e, path.to_path_buf())
        })?))
    }
}

impl AsRef<Path> for FilePath {
    fn as_ref(&self) -> &Path {
        &self.0.as_ref()
    }
}

pub struct FileParentPath(PathBuf);

impl FileParentPath {
    fn from_file_path(value: &FilePath) -> Self {
        let path: PathBuf = value.as_ref().to_path_buf();

        FileParentPath(path.parent().unwrap_or(Path::new("")).to_path_buf())
    }
}

impl AsRef<Path> for FileParentPath {
    fn as_ref(&self) -> &Path {
        &self.0.as_ref()
    }
}

#[derive(Debug)]
pub enum RepositoryPathFromStringError {
    IoError(std::io::Error, String),
}

#[derive(Clone, Debug)]
pub struct RepositoryPath(PathBuf);

impl TryFrom<String> for RepositoryPath {
    type Error = RepositoryPathFromStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let base_path = Path::new(&value)
            .canonicalize()
            .map_err(|e| RepositoryPathFromStringError::IoError(e, value.into()))?;

        Ok(RepositoryPath(base_path))
    }
}

impl TryFrom<&str> for RepositoryPath {
    type Error = RepositoryPathFromStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        RepositoryPath::try_from(value.to_owned())
    }
}

impl Into<PathBuf> for RepositoryPath {
    fn into(self) -> PathBuf {
        self.0
    }
}

impl AsRef<Path> for RepositoryPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug)]
pub enum ImportPathFromImportStringError {
    CannotFindFile(String),
}

pub struct ImportPath(PathBuf);

impl Into<PathBuf> for ImportPath {
    fn into(self) -> PathBuf {
        self.0
    }
}

impl AsRef<Path> for ImportPath {
    fn as_ref(&self) -> &Path {
        &self.0.as_ref()
    }
}

impl ImportPath {
    fn from_import_string(
        import_path: &str,
        cwd: &FileParentPath,
    ) -> Result<Self, ImportPathFromImportStringError> {
        let base_path: &Path = cwd.as_ref();
        let resolved_path = if import_path.ends_with(".ts") {
            base_path.join(import_path)
        } else {
            let ts_path = base_path.join(format!("{}.ts", import_path));

            if ts_path.exists() {
                ts_path
            } else {
                let d_ts_path = base_path.join(format!("{}.d.ts", import_path));

                if d_ts_path.exists() {
                    d_ts_path
                } else {
                    let index_ts_path = base_path.join(format!("{}/index.ts", import_path));

                    if index_ts_path.exists() {
                        index_ts_path
                    } else {
                        base_path.join(import_path)
                    }
                }
            }
        };

        if let Ok(canonicalized_path) = resolved_path.canonicalize() {
            Ok(ImportPath(canonicalized_path))
        } else {
            Err(ImportPathFromImportStringError::CannotFindFile(
                resolved_path.display().to_string(),
            ))
        }
    }
}

#[derive(Debug)]
pub enum RepositoryChildPathModuleError {
    CouldNotGetModule(String),
    ModuleConversionError(ModuleFromComponentError),
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
    pub fn from_import_path(
        import_path: &ImportPath,
        repository_path: &RepositoryPath,
    ) -> Result<RepositoryChildPath, RepositoryChildPathFromImportPathError> {
        Ok(RepositoryChildPath::from_path(
            import_path.as_ref(),
            repository_path,
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

#[derive(Debug)]
pub enum ModuleFromComponentError {
    InvalidComponent(String),
}

impl From<ModuleFromComponentError> for RepositoryChildPathModuleError {
    fn from(err: ModuleFromComponentError) -> Self {
        RepositoryChildPathModuleError::ModuleConversionError(err)
    }
}

impl From<RepositoryChildPathFromFilePathError> for RepositoryChildPathModuleError {
    fn from(value: RepositoryChildPathFromFilePathError) -> Self {
        RepositoryChildPathModuleError::CouldNotGetModule(format!(
            "Could not get module from file path error: {:?}",
            value
        ))
    }
}

fn component_to_string(component: Component) -> String {
    match component {
        Component::Normal(os_str) => os_str.to_string_lossy().into_owned(),
        Component::RootDir => "/".to_string(),
        Component::CurDir => ".".to_string(),
        Component::ParentDir => "..".to_string(),
        Component::Prefix(prefix) => prefix.as_os_str().to_string_lossy().into_owned(),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Module(String);

impl<'a> TryFrom<Component<'a>> for Module {
    type Error = ModuleFromComponentError;

    fn try_from(value: Component<'a>) -> Result<Self, Self::Error> {
        match value {
            Component::Normal(os_str) => Ok(Module(os_str.to_string_lossy().into_owned())),
            other => Err(ModuleFromComponentError::InvalidComponent(
                component_to_string(other),
            )),
        }
    }
}

impl From<String> for Module {
    fn from(value: String) -> Self {
        Module(value)
    }
}

pub struct Repository(RepositoryPath);

pub enum RepositoryFromStringError {
    CouldNotCreateRepositoryPath(RepositoryPathFromStringError),
}

impl From<RepositoryPathFromStringError> for RepositoryFromStringError {
    fn from(value: RepositoryPathFromStringError) -> Self {
        RepositoryFromStringError::CouldNotCreateRepositoryPath(value)
    }
}

impl TryFrom<String> for Repository {
    type Error = RepositoryPathFromStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Repository(value.try_into()?))
    }
}

impl TryFrom<&str> for Repository {
    type Error = RepositoryPathFromStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Repository(value.try_into()?))
    }
}

#[derive(Debug)]
pub enum RepositoryFilesError {
    CannotScanFiles(walkdir::Error),
    CannotAnalyzeFile(AnalyzedFileFromEntryError),
}

impl From<AnalyzedFileFromEntryError> for RepositoryFilesError {
    fn from(value: AnalyzedFileFromEntryError) -> Self {
        RepositoryFilesError::CannotAnalyzeFile(value)
    }
}

impl From<walkdir::Error> for RepositoryFilesError {
    fn from(value: walkdir::Error) -> Self {
        RepositoryFilesError::CannotScanFiles(value)
    }
}

impl Repository {
    pub fn files(&self) -> rayon::iter::Map<rayon::iter::IterBridge<walkdir::IntoIter>, impl Fn(Result<walkdir::DirEntry, walkdir::Error>) -> Result<AnalyzedFile, RepositoryFilesError>> {
        WalkDir::new(self.0.as_ref())
            .into_iter()
            .par_bridge()
            .map(|entry| -> Result<AnalyzedFile, RepositoryFilesError> {
                Ok(AnalyzedFile::try_from_entry(entry?, &self.0)?)
            })
    }
}

fn main() -> Result<(), BelmarshError> {
    let start = Instant::now();

    lazy_static! {
        static ref IMPORT_REGEX: Regex = Regex::new(r"import\s*\{[^}]*\}\s*from\s*'(\.[^']+)';")
            .expect("Failed to compile regex");
    }

    let repository: Repository = "../MHR.Web.Apps.PeopleFirst/src/app".try_into()?;

    let file_check_count = AtomicUsize::new(0);
    let counts: Result<Vec<usize>, BelmarshError> = repository.files()
        .map(|analyzed_file_result| -> Result<usize, BelmarshError> {
            file_check_count.fetch_add(1, Ordering::SeqCst);

            let analyzed_file = match analyzed_file_result {
                Ok(file) => file,
                Err(e) => match e {
                    RepositoryFilesError::CannotAnalyzeFile(inner_e) => match inner_e {
                        AnalyzedFileFromEntryError::FilePathError(_) => return Ok(0),
                        _ => return Err(RepositoryFilesError::CannotAnalyzeFile(inner_e).into()),
                    }
                    _ => return Err(e.into()),
                },
            };

            let mut count = 0;
            let current_module = analyzed_file.module()?;

            for import_path in analyzed_file.imports()? {
                let imported_module = match RepositoryChildPath::from_import_path(
                    import_path,
                    &analyzed_file.base_path,
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
