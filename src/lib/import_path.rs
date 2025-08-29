use std::path::{Path, PathBuf};
use crate::file_parent_path::FileParentPath;

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
    pub fn from_import_string(
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
