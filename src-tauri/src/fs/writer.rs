use std::fs;
use std::io;
use std::path::Path;

pub fn write_file_atomic(path: &Path, content: &str) -> io::Result<()> {
    let tmp_path = path.with_extension("md.tmp");
    fs::write(&tmp_path, content)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}
