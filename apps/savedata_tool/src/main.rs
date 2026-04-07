use std::fs::{self, create_dir_all, remove_dir};
use std::path::{Path, PathBuf};
use std::io;
use std::collections::HashSet;
use clap::Parser;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::FileOptions;

fn zip_folder(src_dir: &Path, dst_file: &Path) -> io::Result<()> {
    let file = File::create(dst_file)?;
    let mut zip = zip::ZipWriter::new(file);
    
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
    
    let walkdir = WalkDir::new(src_dir);
    let prefix = src_dir.parent().unwrap_or(src_dir);
    
    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(prefix).unwrap();
        
        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(path)?;
            io::copy(&mut f, &mut zip)?;
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }
    
    zip.finish()?;
    Ok(())
}

pub fn copy_savedata<P: AsRef<Path>>(
    src: P,
    dst: P,
    blacklist: Vec<P>
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
    let blacklist = blacklist.iter().map(|v| v.as_ref()).collect();
    let mut stats = CopyStats::default();

    // Create the destination directory if it doesn't exist
    fs::create_dir_all(dst)?;

    copy_recursive(src, dst, src, &extensions, 42 * 1024, &mut stats, &blacklist)?;

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
    blacklist: &Vec<&Path>
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
            copy_recursive(base_src, base_dst, &path, allowed_extensions, max_size_bytes, stats, blacklist)?;
        } else if path.is_file() {
            if blacklist.iter().any(|v| *v == path.parent().unwrap()) {
                continue;
            }

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

#[derive(Parser)]
#[command(name = "copy-savedata")]
#[command(about = "Copy savedata with blacklist support", long_about = None)]
struct Args {
    /// Source directory
    #[arg(short, long)]
    src: PathBuf,

    /// Destination archive
    #[arg(short, long)]
    dst: PathBuf,

    /// Blacklisted paths (can be specified multiple times)
    #[arg(short, long)]
    blacklist: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let dst = PathBuf::from("/tmp/temp_savedatatool");
    let archive = args.dst.join("/data.zip");

    if dst.exists() {
        remove_dir(&dst);
    }
    create_dir_all(&dst);
    let _ = copy_savedata(args.src, dst.clone(), args.blacklist);
    zip_folder(&dst, &archive);
}

