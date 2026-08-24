use std::process::Command;

pub fn is_mounted(source: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("mount | grep -q '{}'", source))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
