# PiNAS

Modern NAS operating system for Raspberry Pi 5, built on LibreELEC.

![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)

## Overview

PiNAS transforms your Raspberry Pi into a full-featured NAS with a modern web interface inspired by Synology DSM. It runs as a native package on LibreELEC, allowing your Pi to serve as both a media center (Kodi) and a NAS.

## Features

- **Desktop-like web interface** — Windows, dock, topbar, and widgets
- **File management** — Browse, upload, download, rename, create folders across locations
- **Storage management** — Pools (RAID 0/1/5/10, JBOD, Btrfs), volumes, S.M.A.R.T. monitoring
- **Share management** — SMB/Samba shares configuration
- **User & group management** — Multi-user with RBAC permissions per folder
- **App Center** — Install 27 apps from catalog (Docker-based)
- **Docker Compose** — Multi-container apps (Nextcloud, PhotoPrism, etc.)
- **Network configuration** — Interfaces, DNS, hostname (connman)
- **SSH management** — Enable/disable, password change
- **Printer sharing** — CUPS integration (USB printers shared via IPP/AirPrint)
- **Real-time monitoring** — CPU, RAM, network, disk via WebSocket
- **Terminal** — Web terminal with command history
- **Process Manager** — System process monitoring and management
- **Onboarding wizard** — 7-step setup (language, device name, user, password, SSH, features)
- **i18n** — English and French
- **Lightweight** — ~50MB RAM, ~100MB disk footprint

## Tech Stack

| Component | Technology |
|-----------|------------|
| OS | LibreELEC 12.x |
| Backend | Rust (Axum + Tokio) |
| Frontend | SvelteKit + Svelte 5 (SSG) |
| Database | SQLite |
| Style | TailwindCSS v4 |
| Icons | Iconify (MDI) |
| Target | Raspberry Pi 5 (ARM64), ARM64 VM, x86_64 VM |

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
5. Follow the onboarding wizard to set up your admin account

## Development

### Prerequisites

- Docker (for building — no local Rust/cargo needed)
- Node.js 20+ / npm (for frontend dev)

### Frontend Dev

```bash
cd frontend
npm install
npm run dev
```

Access at `http://localhost:5173`

### Build Scripts

All build scripts run on a Linux VM (native ARM64 or x86_64). They share common options:

```bash
--backend-only      # Only build the Rust backend
--frontend-only     # Only build the SvelteKit frontend
--skip-libreelec    # Skip LibreELEC image generation
--clean             # Clean build directories first
```

| Script | Target | Output |
|--------|--------|--------|
| `build-arm64.sh` | Raspberry Pi 5 | `.img.gz` (flash to SD) |
| `build-arm64-vm.sh` | ARM64 VM (QEMU, UTM, Proxmox) | `.qcow2` + `KERNEL` |
| `build-x86.sh` | x86_64 VM/PC | `.img.gz` (+ `--vmdk` option) |

```bash
# ARM64 RPi5
./scripts/build-arm64.sh

# ARM64 VM (QCOW2 for QEMU/UTM)
./scripts/build-arm64-vm.sh
./scripts/build-arm64-vm.sh --raw    # Raw .img instead of QCOW2

# x86_64
./scripts/build-x86.sh
./scripts/build-x86.sh --vmdk       # With VMDK conversion
```

### Remote Build (recommended)

Build on a remote Linux VM via SSH — the script handles sync, build, and copies artifacts back:

```bash
./scripts/remote-build.sh                       # ARM64 RPi5 (default)
./scripts/remote-build.sh --arch arm64-vm       # ARM64 VM (QCOW2)
./scripts/remote-build.sh --arch x86            # x86_64
./scripts/remote-build.sh --arch x86 --vmdk     # x86_64 + VMDK
./scripts/remote-build.sh --new                 # Reconfigure VM connection
```

On first run, it asks for the VM's IP and username, then saves the config in `.vm-config`. All build options (`--skip-libreelec`, `--backend-only`, etc.) are forwarded to the underlying build script.

### Run ARM64 VM

```bash
# With QEMU (direct kernel boot)
./scripts/run-vm-qemu.sh

# Or manually
qemu-system-aarch64 -machine virt -cpu cortex-a72 -smp 2 -m 2048 \
  -kernel target/pinas-arm64-vm-*-KERNEL \
  -append "boot=LABEL=LIBREELEC disk=LABEL=STORAGE quiet console=ttyAMA0,115200" \
  -drive file=target/pinas-arm64-vm-*.qcow2,format=qcow2,if=virtio \
  -netdev user,id=net0,hostfwd=tcp::3000-:3000,hostfwd=tcp::2222-:22 \
  -device virtio-net-pci,netdev=net0 -nographic
```

For **UTM (macOS Apple Silicon)**: New VM > Virtualize > Linux, set Kernel to `KERNEL` file, boot args `boot=LABEL=LIBREELEC disk=LABEL=STORAGE quiet console=ttyAMA0,115200`, import `.qcow2` as VirtIO drive, port forward 3000.

### Deploy to Pi

```bash
./scripts/deploy-pi.sh          # Hot deploy backend/frontend via SSH
./scripts/connect-pi.sh         # Discover Pi on LAN and SSH into it
```

## Project Structure

```
├── backend/              # Rust API server (Axum)
├── frontend/             # SvelteKit web interface
├── app-catalog/          # App catalog (27 apps)
│   ├── catalog.json
│   └── apps/
├── libreelec/            # LibreELEC integration
│   ├── packages/
│   │   ├── pinas/        # Main PiNAS package
│   │   └── cups/         # Printer sharing (CUPS)
│   └── projects/
│       └── Virtual/      # ARM64 VM project (kernel config, options)
├── scripts/              # Build & deploy scripts
│   ├── build-arm64.sh         # RPi5 build
│   ├── build-arm64-vm.sh      # ARM64 VM build (QCOW2)
│   ├── build-x86.sh           # x86_64 build
│   ├── remote-build.sh        # Remote build via SSH
│   ├── run-vm-qemu.sh         # Launch ARM64 VM
│   ├── deploy-pi.sh           # Hot deploy to Pi
│   ├── connect-pi.sh          # SSH to Pi (auto-discover)
│   └── convert-umbrel.py      # Umbrel app converter
└── docker/               # Dev environment
```

## App Catalog

27 apps available across 4 categories:

| Category | Apps |
|----------|------|
| **Containers** | Docker, Portainer |
| **Media** | Plex, Jellyfin, Emby, Sonarr, Radarr, Lidarr, qBittorrent, Transmission, SABnzbd, PhotoPrism |
| **Network** | Pi-hole, AdGuard Home, WireGuard, Nginx Proxy Manager |
| **Utilities** | Nextcloud, Home Assistant, Syncthing, Vaultwarden, Grafana, Uptime Kuma, File Browser, Code Server, Node-RED, Paperless-ngx, Duplicati |

Apps are installed via the App Center UI. Single-container apps use Docker pull/create/start steps. Multi-container apps (Nextcloud, PhotoPrism, etc.) use Docker Compose.

See [app-catalog/README.md](app-catalog/README.md) for details.

## API Overview

| Area | Endpoints |
|------|-----------|
| Auth | `/api/auth/login`, `/api/auth/logout`, `/api/auth/me` |
| Setup | `/api/setup/status`, `/api/setup/complete` |
| Users | `/api/users` (CRUD) |
| Groups | `/api/groups` (CRUD + members) |
| Permissions | `/api/permissions` (CRUD per folder) |
| System | `/api/system/info` |
| Storage | `/api/storage/disks`, `/pools`, `/volumes` |
| Files | `/api/files` (browse, upload, download) |
| Locations | `/api/locations` (home, shares, volumes) |
| Shares | `/api/shares` (CRUD) |
| Docker | `/api/docker/status`, `/containers`, `/images` |
| Packages | `/api/packages/catalog`, `/install`, `/task/:id` |
| Apps | `/api/apps/registry`, `/api/apps/:id/i18n/:locale` |
| Services | `/api/services/:name` (start/stop/restart/logs) |
| Network | `/api/network/status`, `/interface`, `/dns`, `/hostname` |
| SSH | `/api/ssh/status`, `/enable`, `/disable`, `/password` |
| CUPS | `/api/cups/status`, `/printers`, `/detect`, `/jobs` |
| Terminal | `/api/terminal/exec` |
| Display | `/api/display/settings` |
| Kodi | `/api/kodi/settings`, `/restart` |
| Update | `/api/update/check`, `/install`, `/progress/:id` |
| Preferences | `/api/preferences` (CRUD) |
| WebSocket | `/api/ws` (real-time events) |

## Roadmap

### Done

- [x] Desktop-like web interface (TopBar, Dock, Window Manager)
- [x] Authentication (JWT + Argon2 + sessions)
- [x] User & group management with RBAC permissions
- [x] Storage Manager (pools, volumes, RAID, S.M.A.R.T.)
- [x] File Manager with dynamic locations (home, shares, volumes)
- [x] App Center with 27 Docker apps
- [x] Docker Compose support (multi-container apps)
- [x] Network configuration (interfaces, DNS, hostname)
- [x] SSH management (enable/disable, password)
- [x] CUPS printer sharing (USB printers via IPP/AirPrint)
- [x] Terminal app with command history
- [x] Process Manager
- [x] Onboarding wizard (7 steps)
- [x] i18n (English + French)
- [x] LibreELEC package + ARM64 build pipeline
- [x] Umbrel app conversion script (Python)
- [x] SMB/Samba share configuration (backend API + frontend UI)
- [x] Display/Kodi configuration app
- [x] Security audit (all critical/high issues fixed)

### In Progress

- [ ] NFS/FTP share configuration UI (placeholders exist)
- [ ] Real-time Storage Manager updates via WebSocket
- [ ] Upload files in File Manager
- [ ] Drag & drop in File Manager

### Planned

**Services & Sharing:**
- [ ] NFS shares (export configuration, client restrictions)
- [ ] FTP/SFTP server (ProFTPD, TLS, user restrictions)

**System Management:**
- [ ] Email notifications (SMTP relay, S.M.A.R.T. alerts, disk space alerts)
- [ ] Scheduled tasks / cron jobs (reboot, shutdown, scripts)
- [ ] Power management (CPU frequency scaling, Wake-on-LAN, scheduled shutdown)
- [ ] SSL/TLS certificates (Let's Encrypt, CSR generation)
- [ ] Firewall (iptables/nftables, IPv4/IPv6, port forwarding)
- [ ] Disk quotas (per user/group, soft/hard limits)

**Backup & Sync:**
- [ ] Rsync backup (scheduled jobs, push/pull, SSH transport)
- [ ] USB backup (auto-sync on plug, udev detection)
- [ ] Cloud sync (rclone — OneDrive, Google Drive, S3)

**Monitoring:**
- [ ] RRD graphs (CPU/RAM/disk/network history over 1h/1d/1w/1m)
- [ ] UPS support (Network UPS Tools, auto-shutdown on low battery)
- [ ] SNMP agent (external monitoring)
- [ ] Remote syslog (forward logs to external server)

**Advanced Storage:**
- [ ] Volume resize
- [ ] LVM (Logical Volume Management)
- [ ] Bcache SSD caching
- [ ] S3/MinIO (object storage)

**UI & Infra:**
- [ ] Dark theme
- [ ] CI/CD (GitHub Actions)

## License

GPL-3.0
