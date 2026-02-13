use std::process::Command;

#[derive(Clone, Debug)]
pub struct DiskInfo {
    pub identifier: String,
    pub name: String,
    pub size: u64,
    pub size_display: String,
}

pub fn list_external_disks() -> Result<Vec<DiskInfo>, String> {
    let output = Command::new("diskutil")
        .args(["list", "-plist", "external", "physical"])
        .output()
        .map_err(|e| format!("Failed to run diskutil: {e}"))?;

    if !output.status.success() {
        return Err("diskutil returned an error".to_string());
    }

    let plist: plist::Value = plist::from_bytes(&output.stdout)
        .map_err(|e| format!("Failed to parse plist: {e}"))?;

    let dict = plist.as_dictionary().ok_or("Expected dictionary")?;

    let disk_ids = match dict.get("WholeDisks") {
        Some(plist::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_string().map(|s| s.to_string()))
            .collect::<Vec<_>>(),
        _ => return Ok(Vec::new()),
    };

    let mut disks = Vec::new();

    for disk_id in &disk_ids {
        if let Ok(info) = get_disk_info(disk_id) {
            if info.is_removable {
                disks.push(DiskInfo {
                    identifier: disk_id.clone(),
                    name: info.name,
                    size: info.size,
                    size_display: info.size_display,
                });
            }
        }
    }

    Ok(disks)
}

struct DiskDetail {
    name: String,
    size: u64,
    size_display: String,
    is_removable: bool,
}

fn get_disk_info(disk_id: &str) -> Result<DiskDetail, String> {
    let output = Command::new("diskutil")
        .args(["info", "-plist", disk_id])
        .output()
        .map_err(|e| format!("Failed to run diskutil info: {e}"))?;

    if !output.status.success() {
        return Err("diskutil info failed".to_string());
    }

    let plist: plist::Value = plist::from_bytes(&output.stdout)
        .map_err(|e| format!("Failed to parse plist: {e}"))?;

    let dict = plist.as_dictionary().ok_or("Expected dictionary")?;

    let name = dict
        .get("MediaName")
        .and_then(|v| v.as_string())
        .unwrap_or("Unknown Disk")
        .to_string();

    let size = dict
        .get("TotalSize")
        .and_then(|v| v.as_unsigned_integer())
        .unwrap_or(0);

    let removable = dict
        .get("RemovableMediaOrExternalDevice")
        .and_then(|v| v.as_boolean())
        .unwrap_or(false);

    // Also check Ejectable as a secondary signal
    let ejectable = dict
        .get("Ejectable")
        .and_then(|v| v.as_boolean())
        .unwrap_or(false);

    let size_display = bytesize::ByteSize(size).to_string();

    Ok(DiskDetail {
        name,
        size,
        size_display,
        is_removable: removable || ejectable,
    })
}

pub fn unmount_disk(disk_id: &str) -> Result<(), String> {
    let output = Command::new("diskutil")
        .args(["unmountDisk", &format!("/dev/{disk_id}")])
        .output()
        .map_err(|e| format!("Failed to unmount: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Unmount failed: {stderr}"));
    }

    Ok(())
}

pub fn eject_disk(disk_id: &str) -> Result<(), String> {
    let output = Command::new("diskutil")
        .args(["eject", &format!("/dev/{disk_id}")])
        .output()
        .map_err(|e| format!("Failed to eject: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Eject failed: {stderr}"));
    }

    Ok(())
}
