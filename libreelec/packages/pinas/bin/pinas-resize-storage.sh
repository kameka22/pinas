#!/bin/sh
# PiNAS - Auto-resize storage partition on first boot
# This script runs once to expand /storage to use all available disk space

MARKER_FILE="/storage/.pinas/.storage-resized"
STORAGE_DEV="/dev/mmcblk0"
STORAGE_PART="${STORAGE_DEV}p2"

# Check if already resized
if [ -f "$MARKER_FILE" ]; then
    echo "PiNAS: Storage already resized, skipping."
    exit 0
fi

# Ensure marker directory exists
mkdir -p /storage/.pinas

# Get current storage size in MB
CURRENT_SIZE_KB=$(df /storage 2>/dev/null | tail -1 | awk '{print $2}')
CURRENT_SIZE_MB=$((CURRENT_SIZE_KB / 1024))

echo "PiNAS: Checking storage partition..."
echo "  Current /storage size: ${CURRENT_SIZE_MB}MB"

# If storage is less than 1GB, it needs resizing
if [ "$CURRENT_SIZE_MB" -lt 1000 ]; then
    echo "PiNAS: Storage partition is small (${CURRENT_SIZE_MB}MB), resizing to use full disk..."

    # Resize partition to use all remaining space
    if parted -s $STORAGE_DEV resizepart 2 100%; then
        echo "PiNAS: Partition resized successfully"

        # Resize filesystem
        if resize2fs $STORAGE_PART; then
            # Get new size
            NEW_SIZE=$(df -h /storage | tail -1 | awk '{print $2}')
            echo "PiNAS: Filesystem resized successfully"
            echo "PiNAS: New storage size: $NEW_SIZE"

            # Create marker file
            echo "Resized on $(date)" > "$MARKER_FILE"
            echo "PiNAS: Storage resize complete!"
        else
            echo "PiNAS: ERROR - Failed to resize filesystem"
            exit 1
        fi
    else
        echo "PiNAS: ERROR - Failed to resize partition"
        exit 1
    fi
else
    echo "PiNAS: Storage partition already at ${CURRENT_SIZE_MB}MB, no resize needed"
    # Create marker to avoid checking every boot
    echo "No resize needed (${CURRENT_SIZE_MB}MB) - $(date)" > "$MARKER_FILE"
fi

exit 0
