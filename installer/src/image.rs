use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct ImageInfo {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub modified: SystemTime,
}

pub fn scan_images(target_dir: &Path) -> Vec<ImageInfo> {
    let mut images = Vec::new();

    let entries = match fs::read_dir(target_dir) {
        Ok(e) => e,
        Err(_) => return images,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        if !name.ends_with(".img.gz") {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        images.push(ImageInfo {
            path,
            name,
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        });
    }

    images.sort_by(|a, b| b.modified.cmp(&a.modified));
    images
}

pub fn format_date(time: SystemTime) -> String {
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    let days = secs / 86400;
    let years = (days as f64 / 365.25) as u64;
    let year = 1970 + years;

    let rem = secs - (years as f64 * 365.25 * 86400.0) as u64;
    let month = (rem / 86400 / 30).min(11) + 1;
    let day = ((rem / 86400) % 30) + 1;

    format!("{year:04}-{month:02}-{day:02}")
}
