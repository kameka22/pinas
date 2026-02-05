#!/bin/bash
#
# Reset PiNAS backend data
# This script removes all backend data: database, home directories, files, etc.
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BACKEND_DIR="$PROJECT_ROOT/backend"

# Default data directory (relative to backend)
DATA_DIR="$BACKEND_DIR/data"

echo -e "${YELLOW}=== PiNAS Backend Data Reset ===${NC}"
echo ""
echo "This will delete ALL backend data:"
echo "  - Database (pinas.db)"
echo "  - User home directories (data/homes/)"
echo "  - Files directory (data/files/)"
echo "  - SQLite WAL/SHM files"
echo ""
echo -e "${RED}WARNING: This action is irreversible!${NC}"
echo ""

# Check if running interactively
if [ -t 0 ]; then
    read -p "Are you sure you want to continue? (y/N) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 0
    fi
fi

# Check if data directory exists
if [ ! -d "$DATA_DIR" ]; then
    echo -e "${YELLOW}Data directory does not exist: $DATA_DIR${NC}"
    echo "Nothing to reset."
    exit 0
fi

echo ""
echo "Resetting data in: $DATA_DIR"
echo ""

# Remove database files
if [ -f "$DATA_DIR/pinas.db" ]; then
    echo -e "  ${RED}✗${NC} Removing database: pinas.db"
    rm -f "$DATA_DIR/pinas.db"
fi

if [ -f "$DATA_DIR/pinas.db-wal" ]; then
    echo -e "  ${RED}✗${NC} Removing WAL file: pinas.db-wal"
    rm -f "$DATA_DIR/pinas.db-wal"
fi

if [ -f "$DATA_DIR/pinas.db-shm" ]; then
    echo -e "  ${RED}✗${NC} Removing SHM file: pinas.db-shm"
    rm -f "$DATA_DIR/pinas.db-shm"
fi

# Remove homes directory
if [ -d "$DATA_DIR/homes" ]; then
    echo -e "  ${RED}✗${NC} Removing homes directory"
    rm -rf "$DATA_DIR/homes"
fi

# Remove files directory
if [ -d "$DATA_DIR/files" ]; then
    echo -e "  ${RED}✗${NC} Removing files directory"
    rm -rf "$DATA_DIR/files"
fi

# Remove packages directory (installed apps)
if [ -d "$DATA_DIR/packages" ]; then
    echo -e "  ${RED}✗${NC} Removing packages directory"
    rm -rf "$DATA_DIR/packages"
fi

# Remove downloads directory
if [ -d "$DATA_DIR/downloads" ]; then
    echo -e "  ${RED}✗${NC} Removing downloads directory"
    rm -rf "$DATA_DIR/downloads"
fi

# Remove logs directory
if [ -d "$DATA_DIR/logs" ]; then
    echo -e "  ${RED}✗${NC} Removing logs directory"
    rm -rf "$DATA_DIR/logs"
fi

# Recreate empty directories
echo ""
echo "Recreating empty directories..."
mkdir -p "$DATA_DIR/homes"
mkdir -p "$DATA_DIR/files"
echo -e "  ${GREEN}✓${NC} Created data/homes/"
echo -e "  ${GREEN}✓${NC} Created data/files/"

echo ""
echo -e "${GREEN}=== Reset complete! ===${NC}"
echo ""
echo "The database will be recreated with migrations when you restart the backend."
echo "Run: cd backend && cargo run"
