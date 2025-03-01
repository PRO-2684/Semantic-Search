//! Utility functions for the semantic search CLI.

use log::info;
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    fs::File,
    io::{Read, Result as IOResult},
    path::{Path, PathBuf},
};

/// Create a directory if it doesn't exist.
fn create_dir(dir: &PathBuf) -> IOResult<()> {
    if !dir.exists() {
        info!("Creating `{:?}` directory...", dir);
        std::fs::create_dir(dir)?;
    }
    Ok(())
}

/// Initialize the config directory.
pub fn init() -> IOResult<()> {
    // Create `data` directory if it doesn't exist
    let data_dir = PathBuf::from(".sense");
    create_dir(&data_dir)?;

    Ok(())
}

/// Calculate SHA-256 hash of a file.
pub fn hash_file<T: AsRef<Path>>(file: T) -> IOResult<String> {
    let mut file = File::open(file)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    let result = base16ct::lower::encode_string(&result);

    Ok(result)
}

/// Walk a directory and call a function on each file path and its relative path to `ref_path`, skipping entries starting with `.`.
///
/// Note that it assumes `dir` and `ref_path` are canonicalized directories.
pub fn walk_dir<T1: AsRef<Path>, T2: AsRef<Path>>(
    dir: T1,
    ref_path: T2,
    func: &mut impl FnMut(&PathBuf, Cow<'_, str>) -> IOResult<()>,
) -> IOResult<()> {
    let ref_path = ref_path.as_ref();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.file_name().unwrap().to_string_lossy().starts_with('.') {
            continue;
        }
        let path = path.canonicalize()?;
        if path.is_dir() {
            walk_dir(&path, ref_path, func)?;
        } else {
            // Get relative path (relative to `ref_path`)
            let relative = path.strip_prefix(ref_path).unwrap().to_string_lossy();

            func(&path, relative)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_license() {
        // Hash `LICENSE` file, which should be stable enough
        let hash = hash_file(&Path::new("LICENSE")).unwrap();

        assert_eq!(
            hash,
            "3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986"
        );
    }
}
