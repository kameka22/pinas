use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;

const BUFFER_SIZE: usize = 1024 * 1024; // 1 MB

pub enum FlashMessage {
    Progress { bytes_written: u64, total_bytes: u64 },
    Complete,
    Error(String),
}

/// Estimate the decompressed size of a gzip file.
/// gzip stores original size (mod 2^32) in the last 4 bytes.
fn estimate_decompressed_size(path: &Path, compressed_size: u64) -> u64 {
    if let Ok(mut f) = File::open(path) {
        if f.seek(std::io::SeekFrom::End(-4)).is_ok() {
            let mut buf = [0u8; 4];
            if f.read_exact(&mut buf).is_ok() {
                let stored = u32::from_le_bytes(buf) as u64;
                if stored > compressed_size {
                    return stored;
                }
            }
        }
    }
    compressed_size * 4
}

pub fn start_flash(
    image_path: &Path,
    disk_id: &str,
    compressed_size: u64,
    tx: mpsc::Sender<FlashMessage>,
) {
    let image_path = image_path.to_path_buf();
    let disk_id = disk_id.to_string();

    std::thread::spawn(move || {
        if let Err(e) = do_flash(&image_path, &disk_id, compressed_size, &tx) {
            let _ = tx.send(FlashMessage::Error(e));
        }
    });
}

fn do_flash(
    image_path: &Path,
    disk_id: &str,
    compressed_size: u64,
    tx: &mpsc::Sender<FlashMessage>,
) -> Result<(), String> {
    let total_bytes = estimate_decompressed_size(image_path, compressed_size);

    // Unmount the disk (doesn't require root)
    crate::disk::unmount_disk(disk_id)?;

    // Use authopen to get authorized write access to the raw device.
    // authopen shows the macOS authorization dialog, then copies stdin to the device.
    // This works on modern macOS where dd via osascript gets "Operation not permitted".
    let raw_device = format!("/dev/r{disk_id}");

    let mut child = Command::new("/usr/libexec/authopen")
        .args(["-w", &raw_device])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start authopen: {e}"))?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or("Failed to open stdin pipe to authopen")?;

    // Open gzip image and decompress, streaming to authopen
    let gz_file =
        File::open(image_path).map_err(|e| format!("Failed to open image: {e}"))?;
    let mut decoder = GzDecoder::new(gz_file);
    let mut buffer = vec![0u8; BUFFER_SIZE];
    let mut bytes_written: u64 = 0;

    loop {
        let n = decoder
            .read(&mut buffer)
            .map_err(|e| format!("Decompression error: {e}"))?;
        if n == 0 {
            break;
        }

        match stdin.write_all(&buffer[..n]) {
            Ok(()) => {}
            Err(e) => {
                // Broken pipe likely means authopen failed or user cancelled
                drop(stdin);
                let mut stderr_str = String::new();
                if let Some(mut stderr) = child.stderr.take() {
                    let _ = stderr.read_to_string(&mut stderr_str);
                }
                let _ = child.wait();

                if stderr_str.contains("canceled") || stderr_str.contains("cancelled") {
                    return Err("Operation cancelled by user.".to_string());
                }
                return Err(format!(
                    "Write error: {e}{}",
                    if stderr_str.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", stderr_str.trim())
                    }
                ));
            }
        }

        bytes_written += n as u64;
        let _ = tx.send(FlashMessage::Progress {
            bytes_written,
            total_bytes,
        });
    }

    // Close stdin to signal EOF
    drop(stdin);

    // Wait for authopen to finish writing
    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for authopen: {e}"))?;

    if !status.success() {
        let mut stderr_str = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            let _ = stderr.read_to_string(&mut stderr_str);
        }
        return Err(format!(
            "Flash failed: {}",
            if stderr_str.is_empty() {
                "authopen returned an error".to_string()
            } else {
                stderr_str.trim().to_string()
            }
        ));
    }

    // Sync and eject
    let _ = Command::new("sync").status();
    crate::disk::eject_disk(disk_id)?;

    let _ = tx.send(FlashMessage::Complete);
    Ok(())
}
