#!/bin/sh
# PiNAS - Auto-resize storage partition on first boot
# This script runs once to expand /storage to use all available disk space
# Uses sfdisk for partition resize (works better with mounted partitions)

MARKER_FILE="/storage/.pinas/.storage-resized"
LOG_FILE="/storage/.pinas/resize.log"

# Logging function
log() {
    # Ensure log directory exists
    mkdir -p /storage/.pinas 2>/dev/null
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" >> "$LOG_FILE" 2>/dev/null
    echo "PiNAS-resize: $1"
}

# Check if already resized
if [ -f "$MARKER_FILE" ]; then
    echo "PiNAS-resize: Storage already resized, skipping."
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
        DISK=$(echo "$STORAGE_PART" | sed 's/[0-9]*$//')
        PARTNUM=$(echo "$STORAGE_PART" | grep -o '[0-9]*$')
        ;;
    /dev/mmcblk[0-9]*p[0-9]*|/dev/nvme[0-9]*n[0-9]*p[0-9]*)
        DISK=$(echo "$STORAGE_PART" | sed 's/p[0-9]*$//')
        PARTNUM=$(echo "$STORAGE_PART" | grep -o '[0-9]*$')
        ;;
    *)
        log "ERROR: Unknown device type: $STORAGE_PART"
        exit 1
        ;;
esac

log "Disk: $DISK, Partition number: $PARTNUM"

# Get current filesystem size (in KB)
CURRENT_SIZE_KB=$(df -k /storage 2>/dev/null | awk 'NR==2 {print $2}')
CURRENT_SIZE_MB=$((CURRENT_SIZE_KB / 1024))
log "Current /storage filesystem size: ${CURRENT_SIZE_MB}MB"

# Get disk total size
DISK_SIZE_BYTES=$(blockdev --getsize64 "$DISK" 2>/dev/null)
DISK_SIZE_GB=$((DISK_SIZE_BYTES / 1024 / 1024 / 1024))
log "Total disk size: ${DISK_SIZE_GB}GB"

# If storage is less than 500MB but disk is larger, it needs resizing
if [ "$CURRENT_SIZE_MB" -lt 500 ] && [ "$DISK_SIZE_GB" -gt 1 ]; then
    log "Storage partition is small (${CURRENT_SIZE_MB}MB on ${DISK_SIZE_GB}GB disk), resize needed"

    # Check for required tools
    if ! command -v resize2fs >/dev/null 2>&1; then
        log "ERROR: resize2fs not found"
        exit 1
    fi

    # Resize partition using parted
    # Note: parted needs confirmation when partition is in use, so we pipe "Yes"
    RESIZE_OK=0
    if command -v parted >/dev/null 2>&1; then
        log "Attempting resize with parted..."

        # Method 1: Try with Yes confirmation (for mounted partitions)
        if echo "Yes" | parted "$DISK" ---pretend-input-tty resizepart "$PARTNUM" 100% >> "$LOG_FILE" 2>&1; then
            log "parted partition resize successful (with confirmation)"
            RESIZE_OK=1
        # Method 2: Try script mode
        elif parted -s "$DISK" resizepart "$PARTNUM" 100% >> "$LOG_FILE" 2>&1; then
            log "parted partition resize successful (script mode)"
            RESIZE_OK=1
        # Method 3: Try with fix flag
        elif parted -s --fix "$DISK" resizepart "$PARTNUM" 100% >> "$LOG_FILE" 2>&1; then
            log "parted partition resize successful (with --fix)"
            RESIZE_OK=1
        else
            log "All parted methods failed"
        fi
    else
        log "ERROR: parted not found"
    fi

    if [ "$RESIZE_OK" -eq 0 ]; then
        log "ERROR: Could not resize partition"
        log "Manual resize required. Run interactively:"
        log "  parted $DISK"
        log "  resizepart $PARTNUM 100%"
        log "  quit"
        log "Then run: partprobe $DISK && resize2fs $STORAGE_PART"
        exit 1
    fi

    # Force kernel to re-read partition table
    log "Reloading partition table..."
    sync

    # Try partprobe first
    if command -v partprobe >/dev/null 2>&1; then
        partprobe "$DISK" >> "$LOG_FILE" 2>&1 || true
    fi

    # Also try partx
    if command -v partx >/dev/null 2>&1; then
        partx -u "$STORAGE_PART" >> "$LOG_FILE" 2>&1 || true
    fi

    # And blockdev
    blockdev --rereadpt "$DISK" >> "$LOG_FILE" 2>&1 || true

    # Wait for kernel to process
    sleep 2

    # Get new partition size
    NEW_PART_SIZE=$(blockdev --getsize64 "$STORAGE_PART" 2>/dev/null)
    NEW_PART_SIZE_MB=$((NEW_PART_SIZE / 1024 / 1024))
    log "New partition size: ${NEW_PART_SIZE_MB}MB"

    # Resize filesystem (online resize for ext4)
    log "Resizing ext4 filesystem..."
    if resize2fs "$STORAGE_PART" >> "$LOG_FILE" 2>&1; then
        log "Filesystem resize successful"

        # Verify new size
        sync
        sleep 1
        NEW_FS_SIZE_KB=$(df -k /storage 2>/dev/null | awk 'NR==2 {print $2}')
        NEW_FS_SIZE_MB=$((NEW_FS_SIZE_KB / 1024))
        NEW_FS_SIZE_GB=$((NEW_FS_SIZE_MB / 1024))

        log "New /storage size: ${NEW_FS_SIZE_MB}MB (~${NEW_FS_SIZE_GB}GB)"
        log "Resize complete: ${CURRENT_SIZE_MB}MB -> ${NEW_FS_SIZE_MB}MB"

        # Create marker file
        mkdir -p /storage/.pinas
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
        log "Partition was resized but filesystem resize failed."
        log "Try running manually: resize2fs $STORAGE_PART"

        # Create partial marker to avoid re-running partition resize
        mkdir -p /storage/.pinas
        echo "Partition resized, filesystem pending - $(date)" > "${MARKER_FILE}.pending"
        exit 1
    fi
else
    log "Storage already adequate (${CURRENT_SIZE_MB}MB) or disk too small (${DISK_SIZE_GB}GB)"
    mkdir -p /storage/.pinas
    echo "No resize needed (${CURRENT_SIZE_MB}MB) - $(date)" > "$MARKER_FILE"
fi

exit 0
