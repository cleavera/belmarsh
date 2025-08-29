use std::path::{Path, PathBuf};
use std::{
    fs::File,
    io::BufReader
};

#[derive(Clone, Debug)]
pub struct FilePath(PathBuf);

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
