#!/bin/sh
# PiNAS initialization script
# Creates necessary directories in /storage (persistent across reboots)

set -e

LOG_FILE="/storage/.pinas/logs/init.log"

log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') $1"
    # Also log to file if possible
    if [ -d /storage/.pinas/logs ]; then
        echo "$(date '+%Y-%m-%d %H:%M:%S') $1" >> "$LOG_FILE"
    fi
}

log "PiNAS: Initializing directories..."

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

log "PiNAS: Base directories created"

# Copy frontend files from read-only system to writable storage (if newer or missing)
log "PiNAS: Checking frontend source at /usr/share/pinas/www..."

if [ -d /usr/share/pinas/www ]; then
    SOURCE_FILES=$(find /usr/share/pinas/www -type f 2>/dev/null | wc -l)
    log "PiNAS: Found $SOURCE_FILES files in /usr/share/pinas/www"

    if [ -f /usr/share/pinas/www/index.html ]; then
        # Check if we need to update (compare index.html modification time)
        if [ ! -f /storage/.pinas/www/index.html ] || \
           [ /usr/share/pinas/www/index.html -nt /storage/.pinas/www/index.html ]; then
            log "PiNAS: Copying frontend files to /storage/.pinas/www..."
            # Use cp -r dir/. to copy contents reliably
            cp -r /usr/share/pinas/www/. /storage/.pinas/www/
            DEST_FILES=$(find /storage/.pinas/www -type f 2>/dev/null | wc -l)
            log "PiNAS: Frontend files copied ($DEST_FILES files)"
        else
            log "PiNAS: Frontend files already up to date"
        fi
    else
        log "PiNAS: WARNING - /usr/share/pinas/www/index.html not found!"
        log "PiNAS: Frontend was not included in the image build."
        ls -la /usr/share/pinas/www/ 2>&1 | while read line; do log "  $line"; done
    fi
else
    log "PiNAS: WARNING - /usr/share/pinas/www directory does not exist!"
    log "PiNAS: Frontend was not included in the image build."
    log "PiNAS: Contents of /usr/share/pinas/:"
    ls -la /usr/share/pinas/ 2>&1 | while read line; do log "  $line"; done || log "  Directory does not exist"
fi

# Final verification
if [ -f /storage/.pinas/www/index.html ]; then
    log "PiNAS: Frontend ready at /storage/.pinas/www"
else
    log "PiNAS: ERROR - Frontend not available! Web interface will not work."
fi

log "PiNAS: Initialization complete"
