use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::collections::HashSet;


pub fn copy_savedata<P: AsRef<Path>>(
    src: P,
    dst: P,
) -> io::Result<CopyStats> {
    let extensions: HashSet<&str> = [
        // Configuration files
        "ini", "cfg", "conf", "config", "settings", "options", "prefs", "preferences",
        
        // Save files
        "sav", "save", "savegame", "bak", "backup",
        
        // Text and logs
        "txt", "log", "rtf",
        
        // Data formats
        "json", "xml", "yaml", "yml", "toml",
        
        // Database files
        "db", "sqlite", "sql", "mdb",
        
        // Profile and user data
        "profile", "user", "usr", "player", "character",
        
        // Cache and temporary
        "cache", "tmp", "temp",
        
        // Replay and demo files
        "dem", "demo", "replay", "rec", "recording",
        
        // Keybindings and controls
        "bind", "binds", "keys", "controls",
        
        // Metadata
        "meta", "manifest", "index",
        
        // CSV for stats/data
        "csv",
        
        // Other common game formats
        "properties", "pref", "opt", "set",
    ].iter().copied().collect();
    let src = src.as_ref();
    let dst = dst.as_ref();
    let mut stats = CopyStats::default();

    // Create the destination directory if it doesn't exist
    fs::create_dir_all(dst)?;

    copy_recursive(src, dst, src, &extensions, 42 * 1024, &mut stats)?;
    
    Ok(stats)
}

#[derive(Default, Debug)]
pub struct CopyStats {
    copied: usize,
    skipped_exists: usize,
    skipped_extension: usize,
    skipped_too_large: usize,
    skipped_symlinks: usize,
}

fn copy_recursive(
    base_src: &Path,
    base_dst: &Path,
    current_src: &Path,
    allowed_extensions: &HashSet<&str>,
    max_size_bytes: u64,
    stats: &mut CopyStats,
) -> io::Result<()> {
    for entry in fs::read_dir(current_src)? {
        let entry = entry?;
        let path = entry.path();
        
        // Calculate relative path from base source
        let relative = path.strip_prefix(base_src)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let target = base_dst.join(relative);

        // Check if it's a symlink
        let metadata = fs::symlink_metadata(&path)?;
        
        if metadata.is_symlink() {
            // Skip all symlinks (both files and directories)
            stats.skipped_symlinks += 1;
            continue;
        }

        if path.is_dir() {
            // Create directory in destination if it doesn't exist
            if !target.exists() {
                fs::create_dir_all(&target)?;
            }
            // Recurse into subdirectory
            copy_recursive(base_src, base_dst, &path, allowed_extensions, max_size_bytes, stats)?;
        } else if path.is_file() {
            // Check file extension
            let extension = path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            
            if !allowed_extensions.contains(extension) {
                stats.skipped_extension += 1;
                continue;
            }

            // Check file size
            let file_metadata = fs::metadata(&path)?;
            if file_metadata.len() > max_size_bytes {
                stats.skipped_too_large += 1;
                continue;
            }

            // Check if file already exists in destination
            if target.exists() {
                stats.skipped_exists += 1;
                println!("Skipped (exists): {}", relative.display());
                continue;
            }

            // Copy file
            fs::copy(&path, &target)?;
            stats.copied += 1;
            println!("Copied: {} ({} bytes)", relative.display(), file_metadata.len());
        }
    }
    
    Ok(())
}

