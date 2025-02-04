use std::path::Path;
use std::fs;
use std::io;

pub fn copy_recursively(src: &Path, dst: &Path) -> io::Result<()> {
    if !src.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source path {:?} does not exist", src),
        ));
    }

    if src.is_file() {
        // If src is a file, directly copy it to the destination.
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?; // Ensure parent directory exists.
        }
        fs::copy(src, dst)?;
    } else if src.is_dir() {
        // If src is a directory, create the corresponding directory at the destination.
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let entry_path = entry.path();
            let dest_path = dst.join(entry.file_name());

            // Recursively copy each entry.
            copy_recursively(&entry_path, &dest_path)?;
        }
    }

    Ok(())
}