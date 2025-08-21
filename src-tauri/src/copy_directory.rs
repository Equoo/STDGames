use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use walkdir::{WalkDir, DirEntry};
use std::os::unix;
use anyhow::anyhow;

pub struct CopyData {
	pub num_files: usize,
	pub files_copied: usize,
}

fn collect_files(src: &Path) -> io::Result<Vec<DirEntry>> {
	let filter_entries = |entry: &DirEntry| {
		let file_type = entry.file_type();
		file_type.is_file() || file_type.is_symlink()
	};

	Ok(WalkDir::new(src).follow_links(false).follow_root_links(false).into_iter().filter_map(|e| e.ok())
		.filter(filter_entries).collect())
}

/// Copy a single file, creating directories as needed
fn copy_file(src_file: &Path, src_root: &Path, dest_root: &Path) -> io::Result<()> {
    // Get relative path from source root
    let relative_path = src_file.strip_prefix(src_root).unwrap_or(src_root);
    let dest_path = dest_root.join(relative_path);

	if dest_path.is_file() {
		return Ok(()); // Skip if the file already exists
	}

    // Create parent directories if they don't exist
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Copy the file
    fs::copy(src_file, dest_path)?;
    Ok(())
}

fn copy_symlink(src_file: &Path, src_root: &Path, dest_root: &Path) -> io::Result<()> {
	// Get relative path from source root
    let relative_path = src_file.strip_prefix(src_root).unwrap_or(src_root);
    let dest_path = dest_root.join(relative_path);

	if dest_path.is_symlink() {
		return Ok(()); // Skip if the file already exists
	}

	// Create parent directories if they don't exist
	if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

	let original_target = fs::read_link(src_file)?;
	//let link = dest_root.join(original_target);
	
	unix::fs::symlink(original_target, dest_path)?;
	Ok(())
}

/// Recursively copy files from one folder to another with progress
pub fn copy_directory(src: &Path, dest: &Path, handle: impl Fn(CopyData)) -> Result<(), Box<dyn Error>> {
	let files = collect_files(src)?;
	let total = files.len();
	println!("Found {} files to copy.", total);

	for (i, entry) in files.iter().enumerate() {

		let file_type = entry.file_type();
		let file = entry.path();

		println!("Copying file {} of {}: {:?}", i + 1, total, file);
		if file_type.is_symlink() {
			let _ = copy_symlink(file, src, dest)?;
		}
		else if file_type.is_file() {
			let _ = copy_file(file, src, dest);
		}
		handle(CopyData {
			num_files: total,
			files_copied: i + 1,
		});
	}

	Ok(())
}

