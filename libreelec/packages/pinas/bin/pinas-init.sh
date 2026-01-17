#!/bin/sh
# PiNAS initialization script
# Creates necessary directories in /storage (persistent across reboots)

set -e

echo "PiNAS: Initializing directories..."

# Create main directories
mkdir -p /storage/.pinas/files
mkdir -p /storage/.pinas/data
mkdir -p /storage/.pinas/logs

# Create default folders for file manager
mkdir -p /storage/.pinas/files/Documents
mkdir -p /storage/.pinas/files/Photos
mkdir -p /storage/.pinas/files/Videos
mkdir -p /storage/.pinas/files/Music

echo "PiNAS: Directories initialized successfully"
