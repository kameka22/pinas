# PiNAS - NAS OS Moderne pour Raspberry Pi

## Vue d'ensemble du projet

PiNAS est un système d'exploitation NAS moderne et performant, inspiré des interfaces commerciales comme Synology DSM, conçu principalement pour Raspberry Pi 5. Il s'installe en tant qu'addon sur **LibreELEC**, permettant de transformer un media center en NAS complet.

### Stack technique

| Composant | Technologie |
|-----------|-------------|
| **OS Hôte** | LibreELEC (Just enough OS for Kodi) |
| **Backend** | Rust (Axum + Tokio) - binaire statique |
| **Frontend** | SvelteKit + Svelte 5 (SSG) |
| **Base de données** | SQLite (dans /storage) |
| **UI Style** | Desktop-like (type Synology DSM / UGOS) |
| **Packaging** | Addon LibreELEC |

### Pourquoi LibreELEC ?

[LibreELEC](https://github.com/LibreELEC/LibreELEC.tv) est un OS minimaliste ("Just enough OS") conçu pour Kodi :

- **Ultra-léger** : ~100MB d'empreinte système
- **Optimisé ARM** : Support natif Raspberry Pi (ARMv8/aarch64)
- **Stable** : Système read-only, impossible à corrompre
- **Extensible** : Système d'addons pour ajouter des fonctionnalités
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
│                              │  │   └── logs/              │
│                              │  └── shares/                │
├─────────────────────────────────────────────────────────────┤
│                     Addons                                  │
│  ├── service.pinas           (Backend Rust + Frontend)     │
│  ├── service.samba           (Partages SMB - existant)     │
│  └── service.nfs             (Partages NFS - à créer)      │
└─────────────────────────────────────────────────────────────┘
```

### Objectifs

- Interface desktop moderne avec fenêtres, dock et widgets
- Performance optimisée pour Raspberry Pi 5 (ARM64, 4-8GB RAM)
- Empreinte minimale (~50MB RAM pour PiNAS, ~100MB disque)
- Cohabitation parfaite avec Kodi
- Installation simple via addon repository

---

## Structure du projet

```
/
├── CLAUDE.md                 # Ce fichier
├── TODO.md                   # Liste des tâches
├── backend/                  # API Rust
│   ├── src/
│   │   ├── main.rs
│   │   ├── api/              # Handlers API REST
│   │   ├── services/         # Logique métier
│   │   ├── models/           # Structs DB
│   │   └── config/           # Configuration
│   ├── Cargo.toml
│   └── migrations/           # Migrations SQLite
├── frontend/                 # UI SvelteKit
│   ├── src/
│   │   ├── lib/
│   │   │   ├── components/   # Composants Svelte
│   │   │   ├── stores/       # État global
│   │   │   └── api/          # Client API typé
│   │   └── routes/           # Pages
│   ├── static/               # Assets statiques
│   ├── package.json
│   └── svelte.config.js
├── addon/                    # Addon LibreELEC
│   ├── service.pinas/
│   │   ├── addon.xml         # Métadonnées addon
│   │   ├── resources/
│   │   │   ├── bin/          # Binaire Rust compilé
│   │   │   ├── web/          # Frontend build (statique)
│   │   │   └── settings.xml  # Config Kodi
│   │   ├── default.py        # Script démarrage Python
│   │   └── service.py        # Service daemon
│   └── repository.pinas/     # Repo pour distribution
├── libreelec/                # Package LibreELEC
│   └── packages/pinas/       # Définition package PiNAS
├── scripts/                  # Scripts de build
│   └── build-arm64.sh        # Build PiNAS + image LibreELEC
└── extra/                    # Sources externes (gitignored)
    ├── LibreELEC.tv/         # Clone LibreELEC pour build local
    └── openmediavault/       # Sources OMV (référence)
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

### Addon LibreELEC

```bash
# Créer l'addon ZIP
cd addon
./build-addon.sh              # Génère service.pinas-x.x.x.zip

# Structure de l'addon généré
service.pinas-1.0.0.zip
├── addon.xml
├── resources/
│   ├── bin/pinas             # Binaire ARM64 statique
│   └── web/                  # Frontend statique
├── default.py
└── service.py
```

### Docker (développement)

```bash
# Environnement complet
cd docker/docker-compose
docker-compose up -d

# Simule l'environnement LibreELEC
docker-compose -f docker-compose.libreelec.yml up
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
POST /api/auth/login          # Authentification
POST /api/auth/logout         # Déconnexion
GET  /api/auth/me             # User courant

GET  /api/system/info         # Infos système (CPU, RAM, etc.)
GET  /api/system/services     # État des services
POST /api/system/reboot       # Redémarrer (via LibreELEC API)
POST /api/system/shutdown     # Éteindre

GET  /api/storage/disks       # Liste des disques
GET  /api/storage/filesystems # Systèmes de fichiers montés
POST /api/storage/mount       # Monter un FS
POST /api/storage/unmount     # Démonter

GET  /api/shares              # Liste des partages
POST /api/shares              # Créer partage
PUT  /api/shares/:id          # Modifier partage
DELETE /api/shares/:id        # Supprimer partage

GET  /api/users               # Liste utilisateurs (PiNAS users)
POST /api/users               # Créer utilisateur

GET  /api/kodi/status         # État de Kodi (optionnel)

WS   /api/ws                  # WebSocket (events temps réel)
```

### Intégration LibreELEC

```
┌─────────────────────────────────────────────────────────────────┐
│                        LibreELEC                                │
│  ┌─────────────┐     ┌──────────────────────────────────────┐  │
│  │    Kodi     │     │          service.pinas               │  │
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
│  │   ├── .pinas/          (config, db, logs)                │ │
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

## Contraintes LibreELEC

### Filesystem

| Chemin | Type | Usage PiNAS |
|--------|------|-------------|
| `/flash` | Read-only | Aucun (système) |
| `/storage` | Read-write | Config, DB, logs, shares |
| `/storage/.pinas/` | Read-write | Données PiNAS |
| `/tmp` | tmpfs | Cache temporaire |

### Services disponibles

LibreELEC fournit certains services via addons existants :

| Service | Addon LibreELEC | Status |
|---------|-----------------|--------|
| Samba | `service.samba` | Disponible, configurable |
| SSH | Intégré | Activable dans Kodi |
| NFS | À créer ou configurer | Custom addon |
| Avahi/mDNS | Intégré | Actif par défaut |

### Contraintes techniques

1. **Pas de gestionnaire de paquets** : Tout doit être embarqué ou installé dans `/storage`
2. **Binaire statique requis** : Compiler avec `musl` (pas de glibc dynamique)
3. **Python 3 disponible** : Pour les scripts d'addon
4. **Systemd disponible** : Services dans `/storage/.config/system.d/` ou intégrés à l'image
5. **Root par défaut** : LibreELEC n'a qu'un seul utilisateur (root)
6. **Docker possible** : Binaires statiques disponibles sur download.docker.com

---

## Dépendances principales

### Backend (Cargo.toml)

```toml
[dependencies]
axum = "0.7"                  # Web framework
tokio = { version = "1", features = ["full"] }
tower-http = "0.5"            # Middlewares (CORS, static files)
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jsonwebtoken = "9"            # Auth JWT
argon2 = "0.5"                # Password hashing
nix = "0.27"                  # Appels système Linux
sysinfo = "0.30"              # Infos système
tracing = "0.1"               # Logging
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
# Frontend servi via tower-http ServeDir (fichiers statiques)
```

### Frontend (package.json)

```json
{
  "dependencies": {
    "@sveltejs/kit": "^2.0.0",
    "svelte": "^5.0.0"
  },
  "devDependencies": {
    "tailwindcss": "^3.4.0",
    "typescript": "^5.0.0",
    "vite": "^5.0.0",
    "@iconify/svelte": "^4.0.0",
    "@sveltejs/adapter-static": "^3.0.0"
  }
}
```

---

## Référence OpenMediaVault

Le dossier `openmediavault/` contient les sources du projet OpenMediaVault original, utilisé comme référence pour :

- **Services RPC** : `openmediavault/deb/openmediavault/usr/share/openmediavault/engined/rpc/`
- **Modules système** : `openmediavault/deb/openmediavault/usr/share/openmediavault/engined/module/`
- **DataModels** : `openmediavault/deb/openmediavault/usr/share/openmediavault/datamodels/`
- **Config Salt** : `openmediavault/deb/openmediavault/srv/salt/`
- **Frontend Angular** : `openmediavault/deb/openmediavault/workbench/`

### Services OMV à implémenter

| Service OMV | Priorité | Description |
|-------------|----------|-------------|
| System | Haute | Infos système, reboot, shutdown |
| DiskMgmt | Haute | Gestion des disques |
| FileSystemMgmt | Haute | Montage/démontage FS |
| ShareMgmt | Haute | Dossiers partagés |
| UserMgmt | Haute | Utilisateurs et groupes |
| Smb | Haute | Partages Samba (via addon existant) |
| Nfs | Moyenne | Partages NFS |
| Smart | Moyenne | Monitoring S.M.A.R.T. |
| Network | Basse | Configuration réseau (limité sur LibreELEC) |

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
- API client dans `$lib/api/`
- Types partagés dans `$lib/types/`
- Adapter static pour génération SSG

### Addon LibreELEC

- Suivre les conventions de nommage LibreELEC (`service.pinas`)
- Script Python pour démarrage/arrêt du service
- Settings via `resources/settings.xml` (UI Kodi)
- Logs vers `/storage/.pinas/logs/`

### Git

- Commits conventionnels : `feat:`, `fix:`, `docs:`, `refactor:`
- Branches : `feature/`, `fix/`, `release/`

---

## Ressources LibreELEC

- [GitHub LibreELEC](https://github.com/LibreELEC/LibreELEC.tv)
- [Wiki - Build Add-ons](https://wiki.libreelec.tv/development/build-commands/build-addons)
- [Forum Development](https://forum.libreelec.tv/) (catégorie Development)
- [Documentation officielle](https://github.com/LibreELEC/documentation)
