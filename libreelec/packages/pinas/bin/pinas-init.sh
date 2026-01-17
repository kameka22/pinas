#!/bin/sh
# PiNAS initialization script
# Creates necessary directories in /storage (persistent across reboots)

set -e

echo "PiNAS: Initializing directories..."

# Create main directories
mkdir -p /storage/.pinas/files
mkdir -p /storage/.pinas/data
mkdir -p /storage/.pinas/logs
mkdir -p /storage/.pinas/www

# Create default folders for file manager
mkdir -p /storage/.pinas/files/Documents
mkdir -p /storage/.pinas/files/Photos
mkdir -p /storage/.pinas/files/Videos
mkdir -p /storage/.pinas/files/Music

# Copy frontend files from read-only system to writable storage (if newer or missing)
if [ -d /usr/share/pinas/www ] && [ -f /usr/share/pinas/www/index.html ]; then
    # Check if we need to update (compare index.html modification time)
    if [ ! -f /storage/.pinas/www/index.html ] || \
       [ /usr/share/pinas/www/index.html -nt /storage/.pinas/www/index.html ]; then
        echo "PiNAS: Copying frontend files to /storage/.pinas/www..."
        cp -r /usr/share/pinas/www/* /storage/.pinas/www/
        echo "PiNAS: Frontend files updated"
    fi
fi

echo "PiNAS: Directories initialized successfully"
