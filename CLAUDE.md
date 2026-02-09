# PiNAS - NAS OS Moderne pour Raspberry Pi

## Vue d'ensemble du projet

PiNAS est un système d'exploitation NAS moderne et performant, inspiré des interfaces commerciales comme Synology DSM, conçu principalement pour Raspberry Pi 5. Il s'installe en tant que **package natif LibreELEC** (intégré à l'image), permettant de transformer un media center en NAS complet.

### Stack technique

| Composant | Technologie |
|-----------|-------------|
| **OS Hôte** | LibreELEC (Just enough OS for Kodi) |
| **Backend** | Rust (Axum + Tokio) - binaire statique |
| **Frontend** | SvelteKit + Svelte 5 (SSG) |
| **Base de données** | SQLite (dans /storage) |
| **UI Style** | Desktop-like (type Synology DSM / UGOS) |
| **Packaging** | Package LibreELEC natif (intégré à l'image) |

### Pourquoi LibreELEC ?

[LibreELEC](https://github.com/LibreELEC/LibreELEC.tv) est un OS minimaliste ("Just enough OS") conçu pour Kodi :

- **Ultra-léger** : ~100MB d'empreinte système
- **Optimisé ARM** : Support natif Raspberry Pi (ARMv8/aarch64)
- **Stable** : Système read-only, impossible à corrompre
- **Extensible** : Système de packages pour ajouter des fonctionnalités
- **Dual-usage** : Media center (Kodi) + NAS (PiNAS)

### Architecture LibreELEC

```
┌─────────────────────────────────────────────────────────────┐
│                    LibreELEC System                         │
├─────────────────────────────────────────────────────────────┤
│  /flash (read-only)          │  /storage (read-write)      │
│  ├── SYSTEM (squashfs)       │  ├── .kodi/                 │
│  ├── KERNEL                  │  ├── .config/               │
│  ├── config.txt              │  ├── .pinas/                │
│  └── cmdline.txt             │  │   ├── pinas.db           │
│                              │  │   ├── config.toml        │
│                              │  │   ├── www/               │
│                              │  │   └── logs/              │
│                              │  └── shares/                │
├─────────────────────────────────────────────────────────────┤
│                   Packages intégrés                         │
│  ├── pinas                   (Backend Rust + Frontend)     │
│  ├── samba                   (Partages SMB)                │
│  └── docker                  (Containers - optionnel)      │
└─────────────────────────────────────────────────────────────┘
```

### Objectifs

- Interface desktop moderne avec fenêtres, dock et widgets
- Performance optimisée pour Raspberry Pi 5 (ARM64, 4-8GB RAM)
- Empreinte minimale (~50MB RAM pour PiNAS, ~100MB disque)
- Cohabitation parfaite avec Kodi
- Installation simple via image LibreELEC custom

---

## Structure du projet

```
/
├── CLAUDE.md                 # Ce fichier (documentation projet)
├── TODO.md                   # Liste des tâches et état d'avancement
├── README.md                 # Présentation rapide
├── BUILD.md                  # Instructions de build
├── MVP.md                    # Spécifications MVP
├── backend/                  # API Rust
│   ├── src/
│   │   ├── main.rs           # Point d'entrée, setup Axum
│   │   ├── api/              # Handlers API REST
│   │   │   ├── auth.rs       # Login, logout, JWT
│   │   │   ├── users.rs      # CRUD utilisateurs
│   │   │   ├── groups.rs     # CRUD groupes et permissions
│   │   │   ├── setup.rs      # Onboarding initial
│   │   │   ├── system.rs     # Infos système
│   │   │   ├── storage.rs    # Gestion disques
│   │   │   ├── shares.rs     # Partages SMB/NFS
│   │   │   ├── files.rs      # Gestionnaire fichiers
│   │   │   ├── docker.rs     # API Docker
│   │   │   ├── packages.rs   # Gestionnaire packages
│   │   │   ├── apps.rs       # Registre applications
│   │   │   ├── services.rs   # Services systemd
│   │   │   ├── terminal.rs   # Exécution commandes shell
│   │   │   ├── locations.rs  # Emplacements navigables (home, shares, volumes)
│   │   │   ├── network.rs   # Configuration réseau
│   │   │   ├── ssh.rs        # SSH enable/disable/password
│   │   │   ├── permissions.rs # Permissions par dossier
│   │   │   ├── cups.rs       # Imprimantes CUPS (detect, add, jobs)
│   │   │   ├── ws.rs         # WebSocket handler
│   │   │   └── middleware.rs # Auth middleware, CORS
│   │   ├── services/         # Logique métier
│   │   │   ├── auth.rs       # JWT, password hashing
│   │   │   ├── user.rs       # Gestion utilisateurs
│   │   │   ├── group.rs      # Gestion groupes
│   │   │   ├── home.rs       # Gestion répertoires home utilisateurs
│   │   │   ├── session.rs    # Sessions
│   │   │   ├── network.rs    # Configuration réseau (connman)
│   │   │   ├── package.rs    # Installation packages
│   │   │   ├── docker.rs     # Client Docker (bollard)
│   │   │   ├── service.rs    # Contrôle systemd
│   │   │   ├── storage.rs    # Opérations stockage
│   │   │   ├── share.rs      # Gestion partages
│   │   │   ├── ssh.rs        # Service SSH (enable/disable/password)
│   │   │   ├── cups.rs       # Service CUPS (imprimantes USB, IPP/AirPrint)
│   │   │   └── system.rs     # Métriques système
│   │   ├── models/           # Structs DB
│   │   │   ├── user.rs       # Modèle utilisateur
│   │   │   ├── group.rs      # Modèle groupe/permissions
│   │   │   ├── session.rs    # Modèle session
│   │   │   ├── share.rs      # Modèle partage
│   │   │   ├── package.rs    # Modèle package installé
│   │   │   └── manifest.rs   # Structure manifest app
│   │   ├── config/           # Configuration
│   │   └── db/               # Pool SQLite
│   ├── Cargo.toml
│   └── migrations/           # Migrations SQLite
│       ├── 001_initial.sql           # Users, sessions, settings, shares
│       ├── 002_packages.sql          # Packages, docker_containers
│       ├── 003_app_registry.sql      # Frontend config, translations
│       └── 004_groups_permissions.sql # Groupes et permissions RBAC
├── frontend/                 # UI SvelteKit
│   ├── src/
│   │   ├── routes/           # Pages
│   │   │   ├── +layout.svelte    # Shell desktop principal
│   │   │   └── +page.svelte      # Desktop avec icônes
│   │   └── lib/
│   │       ├── components/   # Composants Svelte
│   │       │   ├── desktop/      # TopBar, Dock, WindowManager, Window
│   │       │   ├── apps/         # Applications + sous-composants Control Panel
│   │       │   ├── ui/           # Composants UI (ContextMenu)
│   │       │   ├── auth/         # Login
│   │       │   ├── modals/       # ProfileModal, ChangePasswordModal
│   │       │   └── onboarding/   # Wizard setup initial
│   │       ├── stores/       # État global
│   │       │   ├── api.ts        # Client API typé + auth store
│   │       │   ├── desktop.ts    # Apps et registre
│   │       │   ├── windows.ts    # État fenêtres
│   │       │   ├── websocket.ts  # Connexion WS
│   │       │   ├── system.ts     # Stats système
│   │       │   └── onboarding.ts # État onboarding
│   │       └── i18n/         # Internationalisation
│   │           ├── en.ts         # Traductions anglais
│   │           ├── fr.ts         # Traductions français
│   │           └── index.ts      # Store i18n
│   ├── static/               # Assets statiques
│   ├── package.json
│   └── svelte.config.js
├── libreelec/                # Package LibreELEC natif
│   └── packages/
│       ├── pinas/
│       │   ├── package.mk            # Définition package (version, deps, install)
│       │   ├── bin/
│       │   │   ├── pinas             # Binaire backend compilé
│       │   │   ├── pinas-init.sh     # Script d'initialisation
│       │   │   ├── pinas-debug.sh    # Script debug
│       │   │   └── pinas-resize-storage.sh
│       │   ├── system.d/
│       │   │   ├── pinas.service     # Service systemd principal
│       │   │   └── pinas-resize-storage.service
│       │   └── tmpfiles.d/
│       │       └── pinas.conf        # Création répertoires au boot
│       └── cups/
│           ├── package.mk            # CUPS 2.4.10 (autotools build)
│           └── system.d/
│               └── cups.service      # Service CUPS (désactivé par défaut)
├── scripts/                  # Scripts de build
│   ├── build-arm64.sh            # Build complet ARM64
│   ├── build-x86.sh              # Build x86 (dev)
│   ├── build-libreelec-image.sh  # Build image LibreELEC complète
│   ├── remote-build.sh           # Build sur VM distante
│   ├── deploy-pi.sh              # Déploiement sur Pi via SSH
│   ├── convert-umbrel.py         # Conversion apps Umbrel → PiNAS
│   └── convert-umbrel-batch.sh   # Conversion batch
├── docker/                   # Environnement dev Docker
│   ├── docker-compose/
│   │   └── docker-compose.yml
│   └── dockerfiles/
│       ├── backend.Dockerfile
│       └── frontend.Dockerfile
├── app-catalog/              # Catalogue d'apps (git submodule)
│   ├── catalog.json              # Index du catalogue
│   └── apps/                     # Manifests par app
│       ├── docker/manifest.json
│       ├── portainer/manifest.json
│       ├── plex/manifest.json
│       ├── pihole/manifest.json
│       └── samba/manifest.json
├── exemple/                  # Mockups UI (référence design)
│   ├── app center/
│   ├── docker app/
│   └── welcome screen/
└── extra/                    # Sources externes (gitignored)
    ├── LibreELEC.tv/             # Clone LibreELEC pour build
    └── openmediavault/           # Sources OMV (référence)
```

---

## Commandes de développement

### Backend (Rust)

```bash
# Développement local
cd backend
cargo run                     # Lancer le serveur dev
cargo watch -x run            # Hot reload
cargo test                    # Tests
cargo build --release         # Build production

# Cross-compilation pour LibreELEC (aarch64)
# Option 1: Via cross (recommandé)
cross build --release --target aarch64-unknown-linux-musl

# Option 2: Via Docker buildx
docker buildx build --platform linux/arm64 -t pinas-backend .

# Le binaire doit être statique (musl) pour LibreELEC
```

### Frontend (SvelteKit)

```bash
# Développement
cd frontend
npm install                   # Installer dépendances
npm run dev                   # Serveur dev (http://localhost:5173)
npm run build                 # Build SSG (Static Site Generation)
npm run preview               # Preview build

# Le build génère des fichiers statiques dans build/
# Ces fichiers seront servis par le backend Rust
```

### Package LibreELEC

```bash
# Build complet (backend + frontend + image LibreELEC)
./scripts/build-arm64.sh

# Options disponibles
./scripts/build-arm64.sh --backend-only    # Backend seul
./scripts/build-arm64.sh --frontend-only   # Frontend seul
./scripts/build-arm64.sh --skip-libreelec  # Sans image complète
./scripts/build-arm64.sh --clean           # Clean avant build

# Build image LibreELEC complète avec PiNAS intégré
./scripts/build-libreelec-image.sh
```

### Docker (développement)

```bash
# Environnement complet
cd docker/docker-compose
docker-compose up -d
```

---

## Architecture détaillée

### Interface Desktop

```
┌─────────────────────────────────────────────────────────────────────┐
│ [Logo]        Recherche...              [CPU] [RAM] [Net]  [User]  │ ← TopBar
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│   ┌─────────────────┐    ┌─────────────────┐                       │
│   │   Storage       │    │   Settings      │    Fenêtres           │
│   │   Manager    ✕  │    │              ✕  │    flottantes         │
│   │                 │    │                 │                        │
│   │  [Contenu]      │    │  [Contenu]      │                        │
│   │                 │    │                 │                        │
│   └─────────────────┘    └─────────────────┘                        │
│                                                                     │
│   ┌─────────────────────────────────────────┐                      │
│   │            Dashboard Widgets            │                       │
│   └─────────────────────────────────────────┘                      │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│  Dashboard  Storage  Files  Network  Settings                      │ ← Dock
└─────────────────────────────────────────────────────────────────────┘
```

### API Backend

```
# Authentification
POST   /api/auth/login            # Login avec username/password → JWT
POST   /api/auth/logout           # Déconnexion (invalide session)
GET    /api/auth/me               # User courant
POST   /api/auth/change-password  # Changer mot de passe

# Setup (onboarding)
GET    /api/setup/status          # Vérifie si setup effectué
POST   /api/setup/complete        # Crée admin initial → JWT

# Utilisateurs
GET    /api/users                 # Liste utilisateurs
POST   /api/users                 # Créer utilisateur
GET    /api/users/:id             # Détails utilisateur
PUT    /api/users/:id             # Modifier utilisateur
DELETE /api/users/:id             # Supprimer utilisateur

# Groupes et permissions
GET    /api/groups                # Liste groupes
POST   /api/groups                # Créer groupe
GET    /api/groups/:id            # Détails groupe
PUT    /api/groups/:id            # Modifier groupe
DELETE /api/groups/:id            # Supprimer groupe
GET    /api/groups/:id/members    # Membres du groupe
POST   /api/groups/:id/members    # Ajouter membre
DELETE /api/groups/:id/members/:user_id  # Retirer membre

# Système
GET    /api/system/info           # Infos système (CPU, RAM, uptime, etc.)

# Emplacements (File Manager sidebar)
GET    /api/locations             # Liste emplacements navigables (home, shares, volumes)

# Fichiers
GET    /api/files                 # Liste fichiers (query: path, location_id)
POST   /api/files/folder          # Créer dossier (avec location_id)
DELETE /api/files                 # Supprimer fichier/dossier (query: path, location_id)
PATCH  /api/files/rename          # Renommer (avec location_id)
POST   /api/files/upload          # Upload fichier (multipart)
GET    /api/files/download        # Télécharger fichier

# Stockage (Storage Manager)
# Disques physiques
GET    /api/storage/disks              # Liste disques + partitions
GET    /api/storage/disks/:name/smart  # Données S.M.A.R.T. détaillées
POST   /api/storage/disks/:name/wipe   # Effacer disque (protégé si système)
GET    /api/storage/candidates         # Disques disponibles pour pools

# Pools de stockage
GET    /api/storage/pools              # Liste des pools
POST   /api/storage/pools              # Créer pool (RAID type, disques)
GET    /api/storage/pools/:id          # Détails pool
PUT    /api/storage/pools/:id          # Modifier pool (nom, description)
DELETE /api/storage/pools/:id          # Supprimer pool
POST   /api/storage/pools/:id/scrub    # Lancer vérification RAID

# Volumes
GET    /api/storage/volumes            # Liste tous les volumes
POST   /api/storage/pools/:id/volumes  # Créer volume dans pool
GET    /api/storage/volumes/:id        # Détails volume
DELETE /api/storage/volumes/:id        # Supprimer volume
POST   /api/storage/volumes/:id/mount  # Monter volume
POST   /api/storage/volumes/:id/unmount # Démonter volume
POST   /api/storage/volumes/:id/resize # Redimensionner volume

# Legacy (compatibilité)
GET    /api/storage/filesystems        # Alias pour volumes montés

# Partages (UI existe, backend partiel)
GET    /api/shares                # Liste des partages
POST   /api/shares                # Créer partage
PUT    /api/shares/:id            # Modifier partage
DELETE /api/shares/:id            # Supprimer partage

# Docker
GET    /api/docker/status         # État Docker daemon
GET    /api/docker/containers     # Liste containers
POST   /api/docker/containers/:id/start    # Démarrer
POST   /api/docker/containers/:id/stop     # Arrêter
POST   /api/docker/containers/:id/restart  # Redémarrer
DELETE /api/docker/containers/:id          # Supprimer
GET    /api/docker/containers/:id/logs     # Logs container
GET    /api/docker/images         # Liste images
POST   /api/docker/images/pull    # Pull image

# Packages (App Center)
GET    /api/packages              # Liste packages installés
GET    /api/packages/catalog      # Catalogue distant
POST   /api/packages/install      # Installer package
DELETE /api/packages/:id          # Désinstaller
GET    /api/packages/task/:id     # Statut installation

# Applications
GET    /api/apps/registry         # Apps avec fenêtre (pour desktop)
GET    /api/apps/:id/i18n/:locale # Traductions d'une app

# Services systemd
GET    /api/services              # Liste services
GET    /api/services/:name        # État d'un service
POST   /api/services/:name/start  # Démarrer
POST   /api/services/:name/stop   # Arrêter
POST   /api/services/:name/restart # Redémarrer
GET    /api/services/:name/logs   # Logs service

# Terminal
POST   /api/terminal/exec         # Exécuter commande shell

# Network
GET    /api/network/status         # Interfaces, DNS, gateway, hostname
PUT    /api/network/interface      # Configure une interface (DHCP/static)
PUT    /api/network/dns            # Configure les DNS
PUT    /api/network/hostname       # Change le hostname

# SSH
GET    /api/ssh/status             # État du service SSH
POST   /api/ssh/enable             # Activer SSH
POST   /api/ssh/disable            # Désactiver SSH
POST   /api/ssh/password           # Changer mot de passe root

# Permissions
GET    /api/permissions            # Liste permissions par dossier
GET    /api/permissions/folders    # Liste dossiers avec permissions
GET    /api/permissions/folder     # Permissions d'un dossier (query: path)
GET    /api/permissions/user/:id   # Permissions d'un utilisateur
POST   /api/permissions            # Créer permission
PUT    /api/permissions/:id        # Modifier permission
DELETE /api/permissions/:id        # Supprimer permission

# CUPS (Imprimantes)
GET    /api/cups/status            # État du service CUPS
POST   /api/cups/enable            # Activer CUPS
POST   /api/cups/disable           # Désactiver CUPS
GET    /api/cups/printers          # Liste imprimantes configurées
POST   /api/cups/printers          # Ajouter imprimante
DELETE /api/cups/printers/:name    # Supprimer imprimante
PUT    /api/cups/printers/:name    # Modifier imprimante
POST   /api/cups/printers/:name/test # Page de test
GET    /api/cups/detect            # Détecter imprimantes USB
GET    /api/cups/drivers           # Drivers disponibles (query: uri)
GET    /api/cups/jobs              # File d'attente impression
DELETE /api/cups/jobs/:id          # Annuler job

# WebSocket
WS     /api/ws                    # Events temps réel
```

### Intégration LibreELEC

```
┌─────────────────────────────────────────────────────────────────┐
│                        LibreELEC                                │
│  ┌─────────────┐     ┌──────────────────────────────────────┐  │
│  │    Kodi     │     │            PiNAS                     │  │
│  │   :8080     │     │  ┌─────────────────────────────────┐ │  │
│  │             │     │  │     Backend Rust :3000          │ │  │
│  │  (media     │     │  │  ┌────────────────────────────┐ │ │  │
│  │   center)   │     │  │  │  Frontend (static files)   │ │ │  │
│  │             │     │  │  │  /api/* → handlers         │ │ │  │
│  └─────────────┘     │  │  │  /ws   → websocket         │ │ │  │
│        │             │  │  └────────────────────────────┘ │ │  │
│        │             │  └─────────────────────────────────┘ │  │
│        │             └──────────────────────────────────────┘  │
│        │                          │                            │
│        └──────────────────────────┼────────────────────────────┤
│                                   │                            │
│  ┌────────────────────────────────┴──────────────────────────┐ │
│  │                    /storage                               │ │
│  │   ├── .pinas/          (config, db, www, logs)           │ │
│  │   ├── .kodi/           (config Kodi)                     │ │
│  │   └── shares/          (données partagées)               │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Communication temps réel

```
Frontend (Svelte)              Backend (Rust)
     │                              │
     │◄────── WebSocket ───────────►│
     │                              │
     │  Events:                     │
     │  - system.stats.update       │
     │  - storage.disk.added        │
     │  - storage.disk.removed      │
     │  - share.created             │
     │  - task.progress             │
     │  - notification.new          │
```

---

## Applications Frontend

### Apps built-in

| App | Composant | Description |
|-----|-----------|-------------|
| Control Panel | `ControlPanel.svelte` | Hub de paramètres (FileService, TerminalSettings, etc.) |
| File Manager | `FileManager.svelte` | Gestionnaire fichiers |
| Storage Manager | `StorageManager.svelte` | Gestion disques/FS |
| Share Manager | `ShareManager.svelte` | Partages SMB/NFS |
| User Manager | `UserManager.svelte` | Utilisateurs et groupes |
| App Center | `AppCenter.svelte` | Installation d'apps |
| Terminal | `TerminalApp.svelte` | Terminal web |
| Process Manager | `ProcessManager.svelte` | Gestionnaire processus |
| Dashboard | `Dashboard.svelte` | Vue d'ensemble système |
| Settings | `Settings.svelte` | Paramètres système |
| Kodi | `KodiApp.svelte` | Configuration Kodi (sources media) |

### Composants Control Panel

| Composant | Catégorie | Description |
|-----------|-----------|-------------|
| `FileService.svelte` | File Service | Onglets SMB/NFS/FTP (placeholders) |
| `NetworkSettings.svelte` | Network | Configuration réseau (interfaces, DNS, hostname) |
| `TerminalSettings.svelte` | Terminal | Configuration SSH (enable/disable, port, password) |
| `PrinterSettings.svelte` | Printer | Imprimantes CUPS (detect, add, jobs, test page) |

### Composants génériques pour apps installées

| Composant | Usage |
|-----------|-------|
| `IframeApp.svelte` | App web dans iframe (ex: Portainer) |
| `WebviewApp.svelte` | Placeholder + bouton nouvel onglet (ex: Plex) |
| `ServiceApp.svelte` | Contrôle service systemd (start/stop/logs) |

---

## Storage Manager

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        DISQUES PHYSIQUES                        │
├─────────────────────────────────────────────────────────────────┤
│  /dev/mmcblk0 (SD System)     │  /dev/sda (USB)  │  /dev/nvme0  │
│  ├─ mmcblk0p1 (/flash) 🔒     │  └─ sda1         │  └─ nvme0n1p1│
│  └─ mmcblk0p2 (/storage) 🔒   │     (disponible) │    (dispo)   │
├─────────────────────────────────────────────────────────────────┤
│                        STORAGE POOLS                            │
├─────────────────────────────────────────────────────────────────┤
│  Pool "Media" (RAID1)         │  Pool "Backup" (Basic)         │
│  ├─ /dev/sda + /dev/sdb       │  └─ /dev/nvme0n1p1             │
│  └─ Volume 1 (ext4, 2TB)      │     └─ Volume 1 (btrfs, 500GB) │
│     └─ /storage/pools/media   │        └─ /storage/pools/backup│
└─────────────────────────────────────────────────────────────────┘
```

### Structures de données

```rust
// Disque physique
struct Disk {
    device_name: String,          // "sda", "nvme0n1"
    device_path: String,          // "/dev/sda"
    device_by_id: Option<String>, // "/dev/disk/by-id/..."
    model: String,
    serial: String,
    size: u64,                    // bytes
    disk_type: DiskType,          // SSD, HDD, NVMe, SD, USB
    temperature: Option<i32>,     // S.M.A.R.T.
    health_status: Option<String>,
    is_system: bool,              // 🔒 Protégé (ne peut pas être modifié)
    is_removable: bool,
    partitions: Vec<Partition>,
}

// Pool de stockage
struct StoragePool {
    id: String,                   // UUID
    name: String,
    description: Option<String>,
    raid_type: RaidType,          // Basic, JBOD, RAID0, RAID1, RAID5
    status: PoolStatus,           // Normal, Degraded, Error
    devices: Vec<String>,         // device paths
    total_size: u64,
    created_at: DateTime,
}

// Volume dans un pool
struct Volume {
    id: String,
    pool_id: String,
    name: String,
    fs_type: String,              // ext4, btrfs, xfs
    size: u64,
    used: u64,
    mount_point: String,          // /storage/pools/{pool}/{volume}
    status: VolumeStatus,
}

enum RaidType { Basic, JBOD, RAID0, RAID1, RAID5, RAID10 }
```

### Protection disque système

Les disques contenant `/flash` ou `/storage` sont automatiquement protégés :
- Affichés avec indicateur 🔒 dans l'UI
- Endpoints de modification retournent erreur 403
- Détection via `/proc/mounts` et analyse des partitions

### Tables base de données

```sql
-- storage_pools: Pools de stockage configurés
CREATE TABLE storage_pools (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    raid_type TEXT NOT NULL,      -- basic, jbod, raid0, raid1, raid5
    status TEXT DEFAULT 'normal',
    devices TEXT NOT NULL,        -- JSON array
    created_at TEXT NOT NULL
);

-- storage_volumes: Volumes dans les pools
CREATE TABLE storage_volumes (
    id TEXT PRIMARY KEY,
    pool_id TEXT REFERENCES storage_pools(id),
    name TEXT NOT NULL,
    fs_type TEXT NOT NULL,
    mount_point TEXT UNIQUE,
    created_at TEXT NOT NULL
);
```

---

## Contraintes LibreELEC

### Filesystem

| Chemin | Type | Usage PiNAS |
|--------|------|-------------|
| `/flash` | Read-only | Aucun (système) |
| `/storage` | Read-write | Config, DB, logs, shares |
| `/storage/.pinas/` | Read-write | Données PiNAS |
| `/storage/.pinas/www/` | Read-write | Frontend (copié au 1er boot) |
| `/storage/homes/` | Read-write | Répertoires home utilisateurs |
| `/storage/homes/{username}/` | Read-write | Home avec Documents, Downloads, Photos, Music, Videos |
| `/tmp` | tmpfs | Cache temporaire |

### Services disponibles

| Service | Status | Notes |
|---------|--------|-------|
| Samba | Disponible | Configurable via PiNAS |
| SSH | Intégré | Configurable via PiNAS (Control Panel > Terminal) |
| Docker | Optionnel | Via package App Center (binaires statiques) |
| CUPS | Intégré | Désactivé par défaut, configurable via Control Panel > Printer |
| Avahi/mDNS | Intégré | Actif par défaut |

### Contraintes techniques

1. **Pas de gestionnaire de paquets** : Tout doit être embarqué ou installé dans `/storage`
2. **Binaire statique requis** : Compiler avec `musl` (pas de glibc dynamique)
3. **Python 3 disponible** : Pour scripts auxiliaires
4. **Systemd disponible** : Services dans `/storage/.config/system.d/` ou intégrés à l'image
5. **Root par défaut** : LibreELEC n'a qu'un seul utilisateur système (root)
6. **Docker possible** : Binaires statiques disponibles sur download.docker.com

### Redimensionnement du stockage

Au premier démarrage, le script `pinas-resize-storage.sh` étend automatiquement la partition `/storage` pour utiliser tout l'espace disponible sur la carte SD/USB :

```
Service: pinas-resize-storage.service (oneshot, Before=pinas.service)
Script: /storage/.pinas/bin/pinas-resize-storage.sh
Marker: /storage/.pinas/.storage-resized
Logs: /storage/.pinas/resize.log
```

Le script :
1. Détecte le type de périphérique (SD/USB/NVMe)
2. Utilise `parted` pour étendre la partition
3. Utilise `resize2fs` pour étendre le système de fichiers
4. Crée un fichier marker pour éviter les exécutions répétées

---

## Dépendances principales

### Backend (Cargo.toml)

```toml
[dependencies]
axum = "0.7"                  # Web framework
tokio = { version = "1", features = ["full"] }
tower-http = "0.6"            # Middlewares (CORS, static files, compression)
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jsonwebtoken = "9"            # Auth JWT
argon2 = "0.5"                # Password hashing
sysinfo = "0.32"              # Infos système
tracing = "0.1"               # Logging
tracing-subscriber = "0.3"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
bollard = "0.18"              # Client Docker API
reqwest = "0.12"              # HTTP client (catalogue)
flate2 = "1"                  # Décompression gzip
tar = "0.4"                   # Archives tar
sha2 = "0.10"                 # Hash SHA256
hex = "0.4"                   # Encoding hex
base64 = "0.22"               # Encoding base64
```

### Frontend (package.json)

```json
{
  "devDependencies": {
    "@sveltejs/adapter-static": "^3.0.0",
    "@sveltejs/kit": "^2.0.0",
    "svelte": "^5.0.0",
    "tailwindcss": "^4.0.0",
    "typescript": "^5.0.0",
    "vite": "^6.0.0",
    "@iconify/svelte": "^4.0.0"
  }
}
```

---

## App Catalog

Le catalogue d'applications est hébergé sur GitHub : `kameka22/pinas-app-catalog`

### Structure

```
app-catalog/
├── catalog.json              # Index avec metadata de toutes les apps (28 apps)
└── apps/
    ├── docker/manifest.json  # Manifest d'installation Docker (binaire)
    ├── portainer/manifest.json
    ├── samba/manifest.json   # Manifest Samba (binaire)
    ├── jellyfin/manifest.json
    ├── nextcloud/manifest.json  # Multi-container (Compose)
    └── ...                   # 28 apps total
```

### Format Manifest

```json
{
  "id": "app-id",
  "name": "App Name",
  "version": "1.0.0",
  "description": { "en": "...", "fr": "..." },
  "author": "...",
  "license": "MIT",
  "requirements": {
    "min_ram": 512,
    "min_disk": 100,
    "arch": ["aarch64", "x86_64"],
    "dependencies": ["docker"]
  },
  "install": {
    "type": "binary|docker",
    "steps": [
      { "action": "download", "url": "...", "dest": "...", "sha256": "..." },
      { "action": "extract", "src": "...", "dest": "..." },
      { "action": "mkdir", "path": "..." },
      { "action": "symlink", "src": "...", "dest": "..." },
      { "action": "chmod", "path": "...", "mode": "755" },
      { "action": "template", "src": "...", "dest": "..." },
      { "action": "exec", "command": "..." }
    ]
  },
  "uninstall": {
    "steps": [
      { "action": "exec", "command": "...", "ignore_error": true },
      { "action": "delete", "path": "..." }
    ]
  },
  "files": {
    "template-name": "base64-encoded-content"
  },
  "frontend": {
    "component": "IframeApp|WebviewApp|ServiceApp|DockerApp",
    "icon": "mdi:application",
    "gradient": "from-blue-500 to-purple-500",
    "window": { "width": 1200, "height": 800 },
    "config": { "url": "http://localhost:9000" },
    "i18n": { "en": {...}, "fr": {...} }
  }
}
```

### Types d'installation

| Type | Usage |
|------|-------|
| `binary` | Télécharge et extrait des binaires statiques (ex: Docker) |
| `docker` | Utilise Docker pour pull/create/start des containers |
| `compose` | Docker Compose pour apps multi-container (ex: Nextcloud) |

### Variables substituées

| Variable | Valeur |
|----------|--------|
| `${PACKAGES_DIR}` | `/storage/.pinas/packages` |
| `${DATA_DIR}` | `/storage/.pinas/data` |
| `${APP_DATA_DIR}` | `/storage/.pinas/data/apps/{app_id}` |
| `${BIN_DIR}` | `/storage/.pinas/bin` |
| `${DOWNLOADS_DIR}` | `/storage/.pinas/downloads` |
| `${ARCH}` | `aarch64` ou `x86_64` |
| `${DEVICE_HOSTNAME}` | Hostname système |

### Vérification SHA256

La vérification SHA256 est optionnelle. Pour les archives dont le checksum varie par build (comme Docker), omettre le champ `sha256`.

Pour les checksums différents par architecture :
```json
{
  "action": "download",
  "url": "https://example.com/${ARCH}/app.tgz",
  "dest": "${DOWNLOADS_DIR}/app.tgz",
  "sha256_aarch64": "abc123...",
  "sha256_x86_64": "def456..."
}
```

Le backend sélectionne automatiquement le bon checksum selon l'architecture cible.

### Gestion des dépendances

- Les dépendances sont définies dans `requirements.dependencies[]`
- Le frontend bloque l'installation si les dépendances ne sont pas installées
- Exemple : Pi-hole requiert Docker → `"dependencies": ["docker"]`

Fonctions frontend (AppCenter.svelte) :
- `isPackageInstalled(id)` : vérifie si un package est installé
- `getMissingDependencies(deps)` : retourne les dépendances manquantes
- `canInstall(app)` : true si toutes les dépendances sont satisfaites

UI : Le bouton "Install" est désactivé avec un message d'avertissement si des dépendances manquent.

### Configuration des apps installées

Les apps installées peuvent avoir une configuration spécifique passée au composant frontend :

```json
"frontend": {
  "component": "IframeApp",
  "config": {
    "url": "http://localhost:9000"
  }
}
```

Cette `config` est transmise via `appConfig` dans le store `desktop.ts` et passée au composant lors de l'ouverture de la fenêtre.

---

## Référence OpenMediaVault

Le dossier `extra/openmediavault/` contient les sources du projet OpenMediaVault, utilisé comme référence pour :

- **Services RPC** : `deb/openmediavault/usr/share/openmediavault/engined/rpc/`
- **Modules système** : `deb/openmediavault/usr/share/openmediavault/engined/module/`
- **DataModels** : `deb/openmediavault/usr/share/openmediavault/datamodels/`
- **Config Salt** : `deb/openmediavault/srv/salt/`

### Services OMV à implémenter

| Service OMV | Status | Description |
|-------------|--------|-------------|
| System | ✅ Partiel | Infos système (manque reboot/shutdown) |
| UserMgmt | ✅ Complet | Utilisateurs et groupes |
| DiskMgmt | ✅ Complet | Gestion des disques (pools, volumes, RAID) |
| FileSystemMgmt | ✅ Complet | Montage/démontage volumes |
| ShareMgmt | ⏳ UI seule | Dossiers partagés |
| Smb | ⏳ Partiel | Partages Samba |
| Nfs | ❌ | Partages NFS |
| Smart | ✅ Complet | Monitoring S.M.A.R.T. (intégré au Storage Manager) |

---

## Système de packages

### Suivi des fichiers installés

Chaque fichier créé lors de l'installation d'un package est enregistré dans la table `package_files`. Cela permet :
- Une désinstallation propre (suppression de tous les fichiers)
- La détection des fichiers orphelins
- L'inventaire des fichiers par package

### Suivi des containers Docker

Les containers créés par un package sont liés dans la table `docker_containers` :
- Création automatique du lien lors de `docker_create`
- Mise à jour du statut (running, stopped, etc.)
- Nettoyage automatique à la désinstallation

### Reconnexion Docker

Après l'installation de Docker, le service backend se reconnecte automatiquement au daemon Docker pour permettre l'installation immédiate de packages Docker sans redémarrage.

### Installation en background (pattern important)

L'installation de packages utilise un pattern de tâche asynchrone pour permettre le suivi en temps réel :

```rust
// 1. install_start() - Crée les enregistrements DB, retourne task_id immédiatement
let task_id = service.install_start(&manifest, manifest_url).await?;

// 2. install_execute() - Exécute les steps en tâche de fond
tokio::spawn(async move {
    service.install_execute(&manifest, &task_id).await;
});

// 3. Retourne task_id au frontend AVANT le début de l'installation
// Le frontend peut ainsi s'abonner aux events WebSocket task.progress
```

**Pourquoi** : Si l'API attend la fin de l'installation avant de répondre, le frontend ne reçoit jamais les events de progression car il n'a pas encore le `task_id` pour les filtrer.

---

## Cibles de compilation

### LibreELEC / Raspberry Pi 5

```bash
# Target: aarch64-unknown-linux-musl (binaire statique)
# OS: LibreELEC 12.x+ (ARM64)
# RAM minimum: 2GB (recommandé 4GB)
# Stockage: microSD (système) + USB/NVMe (données)

# Compilation
cross build --release --target aarch64-unknown-linux-musl

# Vérifier que le binaire est statique
file target/aarch64-unknown-linux-musl/release/pinas
# Doit afficher: "statically linked"
```

### Développement local

```bash
# macOS/Linux x86_64
cargo run                     # Backend natif
npm run dev                   # Frontend avec proxy vers backend
```

---

## Variables d'environnement

```bash
# Backend (production sur LibreELEC)
PINAS_DB_PATH=/storage/.pinas/pinas.db
PINAS_CONFIG_PATH=/storage/.pinas/config.toml
PINAS_LOG_PATH=/storage/.pinas/logs
PINAS_JWT_SECRET=<generated-on-first-run>
PINAS_BIND_ADDRESS=0.0.0.0:3000
PINAS_LOG_LEVEL=info
PINAS_SHARES_ROOT=/storage/shares
PINAS_WWW_PATH=/storage/.pinas/www
PINAS_PACKAGES_DIR=/storage/.pinas/packages
PINAS_DATA_DIR=/storage/.pinas/data
PINAS_HOMES_ROOT=/storage/homes        # Répertoires home utilisateurs
PINAS_HOME_ON_DELETE=archive           # archive, delete, ou keep

PINAS_DEV_MODE=false                  # true pour simuler les opérations (dev local)

# Frontend (build-time)
PUBLIC_API_URL=/api
```

---

## Conventions de code

### Rust

- Utiliser `thiserror` pour les erreurs custom
- Async/await partout (Tokio runtime)
- Structs avec `#[derive(Debug, Serialize, Deserialize)]`
- Tests dans le même fichier avec `#[cfg(test)]`
- Frontend servi depuis `/storage/.pinas/www/` via tower-http

### Svelte

- Composants en PascalCase : `DiskManager.svelte`
- Stores dans `$lib/stores/`
- Composants apps dans `$lib/components/apps/`
- Types partagés dans les stores
- Adapter static pour génération SSG
- i18n via `$lib/i18n/`

### Package LibreELEC

- Définition dans `libreelec/packages/pinas/package.mk`
- Services systemd dans `system.d/`
- Scripts d'init dans `bin/`
- Logs vers `/storage/.pinas/logs/`

### Git

- Commits conventionnels : `feat:`, `fix:`, `docs:`, `refactor:`
- Branches : `feature/`, `fix/`, `release/`

---

## Ressources LibreELEC

- [GitHub LibreELEC](https://github.com/LibreELEC/LibreELEC.tv)
- [Wiki - Build Packages](https://wiki.libreelec.tv/development/build-commands)
- [Forum Development](https://forum.libreelec.tv/) (catégorie Development)
- [Documentation officielle](https://github.com/LibreELEC/documentation)
