use std::path::{Path, PathBuf};

use crate::file_path::FilePath;

pub struct FileParentPath(PathBuf);

impl FileParentPath {
    pub fn from_file_path(value: &FilePath) -> Self {
        let path: PathBuf = value.as_ref().to_path_buf();

        FileParentPath(path.parent().unwrap_or(Path::new("")).to_path_buf())
    }
}

impl AsRef<Path> for FileParentPath {
    fn as_ref(&self) -> &Path {
        &self.0.as_ref()
    }
}
