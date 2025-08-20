use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};
use walkdir::WalkDir;

#[derive(Debug)]
pub enum BelmarshError {
    Io(std::io::Error, PathBuf),
    Walkdir(walkdir::Error),
    StripPrefix(std::path::StripPrefixError),
    PathError(String),
    Regex(regex::Error),
}

impl From<walkdir::Error> for BelmarshError {
    fn from(err: walkdir::Error) -> Self {
        BelmarshError::Walkdir(err)
    }
}

impl From<std::path::StripPrefixError> for BelmarshError {
    fn from(err: std::path::StripPrefixError) -> Self {
        BelmarshError::StripPrefix(err)
    }
}

impl From<regex::Error> for BelmarshError {
    fn from(err: regex::Error) -> Self {
        BelmarshError::Regex(err)
    }
}

fn main() -> Result<(), BelmarshError> {
    let start = Instant::now();
    let base_path_str = "../MHR.Web.Apps.PeopleFirst/src/app";
    let base_path = Path::new(base_path_str).canonicalize().map_err(|e| BelmarshError::Io(e, base_path_str.into()))?;

    lazy_static! {
        static ref IMPORT_REGEX: Regex =
            Regex::new(r"import\s*\{[^}]*\}\s*from\s*'(\.[^']+)';").expect("Failed to compile regex");
    }

    let file_check_count = AtomicUsize::new(0);
    let counts: Result<Vec<usize>, BelmarshError> = WalkDir::new(&base_path)
        .into_iter()
        .par_bridge()
        .map(|entry| -> Result<usize, BelmarshError> {
            file_check_count.fetch_add(1, Ordering::SeqCst);
            let entry = entry?;
            if !entry.file_type().is_file() || entry.path().extension().map_or(true, |ext| ext != "ts") {
                return Ok(0);
            }

            let mut count = 0;
            let parent_dir = entry.path().parent().unwrap_or(Path::new(""));
            let path = entry.path();
            let cannonicalized_path = path.canonicalize().map_err(|e| BelmarshError::Io(e, path.to_path_buf()))?;
            let adjusted_file_path = cannonicalized_path
                .strip_prefix(&base_path)?;

            let module = adjusted_file_path.components().next().ok_or_else(|| BelmarshError::PathError(format!("Could not get module from path: {}", adjusted_file_path.display())))?;

            let file = File::open(path).map_err(|e| BelmarshError::Io(e, path.to_path_buf()))?;

            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line.map_err(|e| BelmarshError::Io(e, path.to_path_buf()))?;
                if let Some(captures) = IMPORT_REGEX.captures(&line) {
                    if let Some(path_capture) = captures.get(1) {
                        let import_path_raw = path_capture.as_str();
                        let resolved_path = if import_path_raw.ends_with(".ts") {
                            parent_dir.join(import_path_raw)
                        } else {
                            let ts_path = parent_dir.join(format!("{}.ts", import_path_raw));
                            if ts_path.exists() {
                                ts_path
                            } else {
                                parent_dir.join(format!("{}/index.ts", import_path_raw))
                            }
                        };

                        if let Ok(canonicalized_path) = resolved_path.canonicalize() {
                            if let Ok(relative_path) =
                                canonicalized_path.strip_prefix(&base_path)
                            {
                                let imported_module =
                                    relative_path.components().next().ok_or_else(|| BelmarshError::PathError(format!("Could not get imported module from path: {}", relative_path.display())))?;

                                if module != imported_module {
                                    count = count + 1;
                                }
                            } else {
                                // Handle imports that point outside the root directory
                                eprintln!(
                                    "Warning: Path is outside root: {} {}",
                                    canonicalized_path.display(),
                                    base_path.display()
                                );
                            }
                        } else {
                            eprintln!(
                                "Warning: Could not canonicalize path: {}",
                                resolved_path.display(),
                            );
                        }
                    }
                }
            }
            Ok(count)
        }).collect();

    let total_count = counts?.into_iter().sum::<usize>();
    let total_files_checked = file_check_count.load(Ordering::SeqCst);

    println!("Total imports from outside own modules: {}", total_count);
    println!("Total files checked: {}", total_files_checked);

    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);

    Ok(())
}
