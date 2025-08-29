use std::path::{Path, PathBuf};

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
