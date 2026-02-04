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
│   │   │   ├── ws.rs         # WebSocket handler
│   │   │   └── middleware.rs # Auth middleware, CORS
│   │   ├── services/         # Logique métier
│   │   │   ├── auth.rs       # JWT, password hashing
│   │   │   ├── user.rs       # Gestion utilisateurs
│   │   │   ├── group.rs      # Gestion groupes
│   │   │   ├── session.rs    # Sessions
│   │   │   ├── package.rs    # Installation packages
│   │   │   ├── docker.rs     # Client Docker (bollard)
│   │   │   ├── service.rs    # Contrôle systemd
│   │   │   ├── storage.rs    # Opérations stockage
│   │   │   ├── share.rs      # Gestion partages
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
│   │       │   ├── apps/         # Applications (16 composants)
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
│   └── packages/pinas/
│       ├── package.mk            # Définition package (version, deps, install)
│       ├── bin/
│       │   ├── pinas             # Binaire backend compilé
│       │   ├── pinas-init.sh     # Script d'initialisation
│       │   ├── pinas-debug.sh    # Script debug
│       │   └── pinas-resize-storage.sh
│       ├── system.d/
│       │   ├── pinas.service     # Service systemd principal
│       │   └── pinas-resize-storage.service
│       └── tmpfiles.d/
│           └── pinas.conf        # Création répertoires au boot
├── scripts/                  # Scripts de build
│   ├── build-arm64.sh            # Build complet ARM64
│   ├── build-x86.sh              # Build x86 (dev)
│   ├── build-libreelec-image.sh  # Build image LibreELEC complète
│   └── remote-build.sh           # Build sur VM distante
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

# Fichiers
GET    /api/files                 # Liste fichiers (query: path)
POST   /api/files/mkdir           # Créer dossier
POST   /api/files/delete          # Supprimer fichier/dossier
POST   /api/files/rename          # Renommer
POST   /api/files/upload          # Upload fichier (multipart)
GET    /api/files/download        # Télécharger fichier

# Stockage (partiellement implémenté)
GET    /api/storage/disks         # Liste des disques
GET    /api/storage/filesystems   # Systèmes de fichiers montés

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
| Control Panel | `ControlPanel.svelte` | Hub de paramètres |
| File Manager | `FileManager.svelte` | Gestionnaire fichiers |
| Storage Manager | `StorageManager.svelte` | Gestion disques/FS |
| Share Manager | `ShareManager.svelte` | Partages SMB/NFS |
| User Manager | `UserManager.svelte` | Utilisateurs et groupes |
| Docker | `DockerApp.svelte` | Gestion containers |
| App Center | `AppCenter.svelte` | Installation d'apps |
| Terminal | `TerminalApp.svelte` | Terminal web |
| Process Manager | `ProcessManager.svelte` | Gestionnaire processus |
| Dashboard | `Dashboard.svelte` | Vue d'ensemble système |
| Settings | `Settings.svelte` | Paramètres système |

### Composants génériques pour apps installées

| Composant | Usage |
|-----------|-------|
| `IframeApp.svelte` | App web dans iframe (ex: Portainer) |
| `WebviewApp.svelte` | Placeholder + bouton nouvel onglet (ex: Plex) |
| `ServiceApp.svelte` | Contrôle service systemd (start/stop/logs) |

---

## Contraintes LibreELEC

### Filesystem

| Chemin | Type | Usage PiNAS |
|--------|------|-------------|
| `/flash` | Read-only | Aucun (système) |
| `/storage` | Read-write | Config, DB, logs, shares |
| `/storage/.pinas/` | Read-write | Données PiNAS |
| `/storage/.pinas/www/` | Read-write | Frontend (copié au 1er boot) |
| `/tmp` | tmpfs | Cache temporaire |

### Services disponibles

| Service | Status | Notes |
|---------|--------|-------|
| Samba | Disponible | Configurable via PiNAS |
| SSH | Intégré | Activable dans Kodi |
| Docker | Optionnel | Via package ou binaires statiques |
| Avahi/mDNS | Intégré | Actif par défaut |

### Contraintes techniques

1. **Pas de gestionnaire de paquets** : Tout doit être embarqué ou installé dans `/storage`
2. **Binaire statique requis** : Compiler avec `musl` (pas de glibc dynamique)
3. **Python 3 disponible** : Pour scripts auxiliaires
4. **Systemd disponible** : Services dans `/storage/.config/system.d/` ou intégrés à l'image
5. **Root par défaut** : LibreELEC n'a qu'un seul utilisateur système (root)
6. **Docker possible** : Binaires statiques disponibles sur download.docker.com

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
├── catalog.json              # Index avec metadata de toutes les apps
└── apps/
    ├── docker/manifest.json  # Manifest d'installation Docker
    ├── portainer/manifest.json
    ├── plex/manifest.json
    ├── pihole/manifest.json
    └── samba/manifest.json
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
    "steps": [
      { "type": "docker_pull", "image": "..." },
      { "type": "docker_create", "config": {...} },
      { "type": "docker_start" }
    ]
  },
  "uninstall": {
    "steps": [...]
  },
  "frontend": {
    "component": "IframeApp",
    "icon": "mdi:application",
    "gradient": "from-blue-500 to-purple-500",
    "window": { "width": 1200, "height": 800 },
    "config": { "url": "http://localhost:9000" },
    "i18n": { "en": {...}, "fr": {...} }
  }
}
```

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
| DiskMgmt | ⏳ UI seule | Gestion des disques |
| FileSystemMgmt | ⏳ UI seule | Montage/démontage FS |
| ShareMgmt | ⏳ UI seule | Dossiers partagés |
| Smb | ⏳ Partiel | Partages Samba |
| Nfs | ❌ | Partages NFS |
| Smart | ❌ | Monitoring S.M.A.R.T. |

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
