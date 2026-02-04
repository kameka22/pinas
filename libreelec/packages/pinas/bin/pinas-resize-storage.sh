#!/bin/sh
# PiNAS - Auto-resize storage partition on first boot
# This script runs once to expand /storage to use all available disk space

MARKER_FILE="/storage/.pinas/.storage-resized"
LOG_FILE="/storage/.pinas/resize.log"

# Logging function
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" >> "$LOG_FILE" 2>/dev/null
    echo "PiNAS-resize: $1"
}

# Ensure marker directory exists
mkdir -p /storage/.pinas

# Check if already resized
if [ -f "$MARKER_FILE" ]; then
    log "Storage already resized, skipping."
    exit 0
fi

log "=== PiNAS Storage Resize Starting ==="

# Detect storage device from /proc/mounts
STORAGE_PART=$(awk '/\/storage/ {print $1; exit}' /proc/mounts)

if [ -z "$STORAGE_PART" ]; then
    log "ERROR: Could not detect storage partition"
    exit 1
fi

log "Storage partition: $STORAGE_PART"

# Derive disk device and partition number
case "$STORAGE_PART" in
    /dev/sd[a-z][0-9]*)
        # SATA/USB: /dev/sda2 -> disk=/dev/sda partnum=2
        DISK=$(echo "$STORAGE_PART" | sed 's/[0-9]*$//')
        PARTNUM=$(echo "$STORAGE_PART" | grep -o '[0-9]*$')
        ;;
    /dev/mmcblk[0-9]*p[0-9]*|/dev/nvme[0-9]*n[0-9]*p[0-9]*)
        # SD/NVMe: /dev/mmcblk0p2 -> disk=/dev/mmcblk0 partnum=2
        DISK=$(echo "$STORAGE_PART" | sed 's/p[0-9]*$//')
        PARTNUM=$(echo "$STORAGE_PART" | grep -o '[0-9]*$')
        ;;
    *)
        log "ERROR: Unknown device type: $STORAGE_PART"
        exit 1
        ;;
esac

log "Disk: $DISK, Partition number: $PARTNUM"

# Get current partition size (in KB)
CURRENT_SIZE_KB=$(df -k /storage 2>/dev/null | awk 'NR==2 {print $2}')
CURRENT_SIZE_MB=$((CURRENT_SIZE_KB / 1024))
log "Current /storage size: ${CURRENT_SIZE_MB}MB (${CURRENT_SIZE_KB}KB)"

# Get disk total size
DISK_SIZE=$(blockdev --getsize64 "$DISK" 2>/dev/null)
DISK_SIZE_GB=$((DISK_SIZE / 1024 / 1024 / 1024))
log "Total disk size: ${DISK_SIZE_GB}GB"

# If storage is less than 500MB, it definitely needs resizing
if [ "$CURRENT_SIZE_MB" -lt 500 ]; then
    log "Storage partition is small (${CURRENT_SIZE_MB}MB), resize needed"

    # Check for required tools
    MISSING_TOOLS=""
    command -v parted >/dev/null 2>&1 || MISSING_TOOLS="$MISSING_TOOLS parted"
    command -v resize2fs >/dev/null 2>&1 || MISSING_TOOLS="$MISSING_TOOLS resize2fs"

    if [ -n "$MISSING_TOOLS" ]; then
        log "ERROR: Missing required tools:$MISSING_TOOLS"
        log "Manual resize: parted $DISK resizepart $PARTNUM 100%"
        exit 1
    fi

    # Show partition table before
    log "Current partition table:"
    parted -s "$DISK" print >> "$LOG_FILE" 2>&1

    # Resize partition to use all remaining space
    log "Resizing partition $PARTNUM to 100%..."
    if parted -s "$DISK" resizepart "$PARTNUM" 100% >> "$LOG_FILE" 2>&1; then
        log "Partition resize command successful"
    else
        log "ERROR: parted resizepart failed (exit code: $?)"
        # Try alternative: parted with fix flag
        log "Trying with --fix flag..."
        if parted -s --fix "$DISK" resizepart "$PARTNUM" 100% >> "$LOG_FILE" 2>&1; then
            log "Partition resize with --fix successful"
        else
            log "ERROR: Partition resize failed completely"
            exit 1
        fi
    fi

    # Force kernel to re-read partition table
    log "Reloading partition table..."
    sync

    # Try multiple methods to reload partition table
    if command -v partprobe >/dev/null 2>&1; then
        partprobe "$DISK" >> "$LOG_FILE" 2>&1 || true
        log "partprobe executed"
    fi

    if command -v partx >/dev/null 2>&1; then
        partx -u "$DISK" >> "$LOG_FILE" 2>&1 || true
        log "partx executed"
    fi

    # Also try blockdev
    blockdev --rereadpt "$DISK" >> "$LOG_FILE" 2>&1 || true

    # Wait for kernel to process
    sleep 3

    # Show new partition size from kernel's perspective
    NEW_PART_SIZE=$(blockdev --getsize64 "$STORAGE_PART" 2>/dev/null)
    NEW_PART_SIZE_MB=$((NEW_PART_SIZE / 1024 / 1024))
    log "New partition size (kernel): ${NEW_PART_SIZE_MB}MB"

    # Resize filesystem (online resize for ext4)
    log "Resizing ext4 filesystem..."
    if resize2fs "$STORAGE_PART" >> "$LOG_FILE" 2>&1; then
        log "Filesystem resize successful"

        # Verify new size
        sync
        NEW_FS_SIZE_KB=$(df -k /storage 2>/dev/null | awk 'NR==2 {print $2}')
        NEW_FS_SIZE_MB=$((NEW_FS_SIZE_KB / 1024))

        log "New /storage size: ${NEW_FS_SIZE_MB}MB"
        log "Resize complete: ${CURRENT_SIZE_MB}MB -> ${NEW_FS_SIZE_MB}MB"

        # Create marker file
        cat > "$MARKER_FILE" << EOF
Resized successfully on $(date)
Before: ${CURRENT_SIZE_MB}MB
After: ${NEW_FS_SIZE_MB}MB
Disk: $DISK
Partition: $STORAGE_PART
EOF

        log "=== Storage Resize Complete ==="
    else
        log "ERROR: resize2fs failed"
        log "The partition was resized but filesystem resize failed."
        log "Try running manually after reboot: resize2fs $STORAGE_PART"

        # Create partial marker to avoid re-running parted
        echo "Partition resized, filesystem pending - $(date)" > "${MARKER_FILE}.pending"
        exit 1
    fi
else
    log "Storage already at ${CURRENT_SIZE_MB}MB (>500MB), no resize needed"
    echo "No resize needed (${CURRENT_SIZE_MB}MB) - $(date)" > "$MARKER_FILE"
fi

exit 0
