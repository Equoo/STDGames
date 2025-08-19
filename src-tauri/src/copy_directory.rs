use std::any::Any;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::{WalkDir, DirEntry};
use std::os::unix;
use anyhow::anyhow;

fn collect_files(src: &Path) -> io::Result<Vec<DirEntry>> {
	// let mut files = Vec::new();
	// for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
	// 	let file_type = entry.file_type();
	// 	if file_type.is_file() || file_type.is_symlink() {
	// 		files.push(entry);
	// 	}
	// }
	// Ok(files)
	
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
    let relative_path = src_file.strip_prefix(src_root).unwrap();
    let dest_path = dest_root.join(relative_path);

    // Create parent directories if they don't exist
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Copy the file
    fs::copy(src_file, dest_path)?;
    Ok(())
}

fn copy_symlink(src_file: &Path, src_root: &Path, dest_root: &Path) -> Result<(), Box<dyn Error>> {

	prefix = src_root;

	let original_target = fs::read_link(src_file)?;

	if original_target.is_absolute() {
		if original_target.starts_with(prefix) {
			original_target = original_target.strip_prefix(prefix)
		}
	}

	link_target = original_target;
	// let link_target = original_target.strip_prefix(src_root.parent().ok_or(anyhow!("no parent directory"))?)?;

	println!("create symlink from {:#?} to {:#?}", link_target, dest_root.join(link_target));
	unix::fs::symlink(link_target, dest_root.join(link_target))?;

	Ok(())
}



/// Recursively copy files from one folder to another with progress
pub fn copy_directory(src: &Path, dest: &Path) -> Result<(), Box<dyn Error>> {
	let files = collect_files(src)?;
	let total = files.len();
	println!("Found {} files to copy.", total);

	for (i, entry) in files.iter().enumerate() {

		let file_type = entry.file_type();
		let file = entry.path();

		if file_type.is_symlink() {
			let _ = copy_symlink(file, src, dest)?;
		}
		else if file_type.is_file() {
			let _ = copy_file(file, src, dest);
		}


		// if std::fs::read_link(file).is_ok() {
		//if std::fs::symlink_metadata(file).unwrap().is_symlink() {
		// if !file.is_symlink() {
			//println!("Copied {} of {} files; name {:#?}", i + 1, total, file.as_path());
		//}
		// let _ = copy_file(file, src, dest);
	}

	Ok(())
}

