use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn scan_md_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(err) => {
                eprintln!("[scanner] walkdir error (possible permission issue): {}", err);
                None
            }
        })
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .map_or(false, |ext| ext == "md")
        })
        .map(|e| e.into_path())
        .collect()
}
