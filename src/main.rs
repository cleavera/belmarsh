use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    time::Instant,
};
use walkdir::WalkDir;

#[derive(Debug)]
pub enum BelmarshError {}

fn main() -> Result<(), BelmarshError> {
    let start = Instant::now();
    let base_path_str = "../avaritia/src";
    let base_path = Path::new(base_path_str).canonicalize().unwrap();

    lazy_static! {
        static ref IMPORT_REGEX: Regex =
            Regex::new(r"import\s*\{[^}]*\}\s*from\s*'(\.[^']+)';").unwrap();
    }

    let total_count = WalkDir::new(&base_path)
        .into_iter()
        .par_bridge()
        .map(|entry| {
            let mut count = 0;
            let entry = entry.unwrap();
            let parent_dir = entry.path().parent().unwrap_or(Path::new(""));
            if entry.file_type().is_file() {
                let path = entry.path();
                let cannonicalized_path = path.canonicalize().unwrap();
                let adjusted_file_path = cannonicalized_path
                    .strip_prefix(&base_path)
                    .unwrap();

                let module = adjusted_file_path.components().next().unwrap();

                let file = File::open(path).unwrap();

                let reader = BufReader::new(file);

                for line in reader.lines() {
                    if let Some(captures) = IMPORT_REGEX.captures(&line.unwrap()) {
                        if let Some(path) = captures.get(1) {
                            let import_path_raw = path.as_str();
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
                                        relative_path.components().next().unwrap();

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
            }

            count
        }).sum::<usize>();

    println!("Total imports from outside own files: {}", total_count);

    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);

    Ok(())
}
