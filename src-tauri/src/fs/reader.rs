use std::fs;
use std::io;
use std::path::Path;

pub fn read_file(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}
