# PiNAS

Modern NAS operating system for Raspberry Pi 5, built on LibreELEC.

![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)

## Overview

PiNAS transforms your Raspberry Pi into a full-featured NAS with a modern web interface inspired by Synology DSM. It runs as an addon on LibreELEC, allowing your Pi to serve as both a media center (Kodi) and a NAS.

## Features

- **Desktop-like web interface** - Windows, dock, and widgets
- **File management** - Browse, upload, download files
- **Storage management** - Disk monitoring and mounting
- **Share management** - SMB/Samba shares configuration
- **User management** - Multi-user support with permissions
- **Real-time monitoring** - CPU, RAM, network, disk usage
- **Lightweight** - ~50MB RAM, ~100MB disk footprint

## Tech Stack

| Component | Technology |
|-----------|------------|
| OS | LibreELEC 12.x |
| Backend | Rust (Axum + Tokio) |
| Frontend | SvelteKit + Svelte 5 |
| Database | SQLite |
| Target | Raspberry Pi 5 (ARM64) |

## Requirements

- Raspberry Pi 5 (4GB+ RAM recommended)
- microSD card (16GB+ for system)
- USB/NVMe storage for data

## Installation

### Pre-built Image

1. Download the latest PiNAS image from [Releases](https://github.com/your-repo/pinas/releases)
2. Flash to SD card:
   ```bash
   gunzip -c LibreELEC-RPi5.aarch64-*.img.gz | sudo dd of=/dev/sdX bs=4M status=progress conv=fsync
   ```
3. Insert SD card and boot your Pi
4. Access PiNAS at `http://<pi-ip>:3000`

### Default Credentials

- Username: `admin`
- Password: `admin`

## Development

### Prerequisites

- Rust 1.70+
- Node.js 20+
- npm

### Backend

```bash
cd backend
cargo run
```

### Frontend

```bash
cd frontend
npm install
npm run dev
```

Access the dev server at `http://localhost:5173`

## Building for Raspberry Pi

### Requirements

- Ubuntu ARM64 VM or native ARM64 machine
- 20GB+ free disk space
- 4GB+ RAM

### Build Commands

```bash
# Full build (backend + frontend + LibreELEC image)
./scripts/build-arm64.sh

# Build only PiNAS package (skip LibreELEC image)
./scripts/build-arm64.sh --skip-libreelec

# Rebuild frontend only
./scripts/build-arm64.sh --frontend-only

# Rebuild backend only
./scripts/build-arm64.sh --backend-only

# Clean build
./scripts/build-arm64.sh --clean
```

The build creates a LibreELEC image with PiNAS pre-integrated at:
```
extra/LibreELEC.tv/target/LibreELEC-RPi5.aarch64-*.img.gz
```

## Project Structure

```
├── backend/           # Rust API server
├── frontend/          # SvelteKit web interface
├── libreelec/         # LibreELEC package definition
│   └── packages/pinas/
├── scripts/           # Build scripts
└── addon/             # Kodi addon (alternative install)
```

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /api/health` | Health check |
| `POST /api/auth/login` | Authentication |
| `GET /api/system/info` | System information |
| `GET /api/storage/disks` | List disks |
| `GET /api/shares` | List shares |
| `GET /api/files/*` | File browser |
| `WS /api/ws` | Real-time events |

## Configuration

Environment variables (set in `/usr/lib/systemd/system/pinas.service`):

| Variable | Default | Description |
|----------|---------|-------------|
| `PINAS_BIND_ADDRESS` | `0.0.0.0:3000` | Server address |
| `PINAS_DATABASE_URL` | `sqlite:/storage/.pinas/data/pinas.db` | Database path |
| `PINAS_FILES_ROOT` | `/storage/.pinas/files` | Files root directory |
| `PINAS_STATIC_DIR` | `/storage/.pinas/www` | Frontend files |

## Troubleshooting

### Service not starting

```bash
# Check service status
systemctl status pinas

# View logs
journalctl -u pinas -f

# Check init logs
cat /storage/.pinas/logs/init.log
```

### Frontend not loading

```bash
# Verify frontend files exist
ls -la /usr/share/pinas/www/
ls -la /storage/.pinas/www/

# Manually copy frontend
cp -r /usr/share/pinas/www/. /storage/.pinas/www/
```

## License

GPL-3.0
