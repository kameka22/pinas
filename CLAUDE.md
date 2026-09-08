# PiNAS - NAS OS Moderne pour Raspberry Pi

## Vue d'ensemble

PiNAS est un système d'exploitation NAS moderne, inspiré de Synology DSM, conçu pour Raspberry Pi 5. Il s'installe en tant que **package natif LibreELEC**, transformant un media center en NAS complet.

### Stack technique

| Composant | Technologie |
|-----------|-------------|
| **OS Hôte** | LibreELEC (Just enough OS for Kodi) |
| **Backend** | Rust (Axum + Tokio) - binaire statique musl |
| **Frontend** | SvelteKit + Svelte 5 (SSG) |
| **Base de données** | SQLite (dans /storage) |
| **UI Style** | Desktop-like (fenêtres, dock, widgets) |
| **Packaging** | Package LibreELEC natif (intégré à l'image) |

### Objectifs

- Interface desktop moderne avec fenêtres, dock et widgets
- Performance optimisée pour Raspberry Pi 5 (ARM64, 4-8GB RAM)
- Empreinte minimale (~50MB RAM, ~100MB disque)
- Cohabitation parfaite avec Kodi
- Installation simple via image LibreELEC custom

---

## Structure du projet (vue simplifiée)

```
/
├── CLAUDE.md                 # Ce fichier (instructions globales)
├── TODO.md / BUILD.md / STORAGE_MANAGER.md / SECURITY_AUDIT.md
├── backend/                  # API Rust (voir backend/CLAUDE.md)
│   ├── src/
│   │   ├── api/              # Handlers REST + WebSocket + middleware
│   │   ├── services/         # Logique métier
│   │   ├── models/           # Structs DB
│   │   ├── config/           # Configuration
│   │   └── db/               # Pool SQLite
│   └── migrations/           # Migrations SQLite (001-011)
├── frontend/                 # UI SvelteKit (voir frontend/CLAUDE.md)
│   └── src/lib/
│       ├── components/       # desktop/, apps/, ui/, auth/, modals/, onboarding/
│       ├── stores/           # api, desktop, windows, websocket, system, update, onboarding, power, taskManager
│       └── i18n/             # en, fr
├── libreelec/                # Package LibreELEC (voir libreelec/CLAUDE.md)
│   └── packages/pinas/      # package.mk, bin/, system.d/, tmpfiles.d/
├── app-catalog/              # Catalogue d'apps (voir app-catalog/CLAUDE.md)
├── scripts/                  # build-arm64.sh, deploy-pi.sh, etc.
└── docker/                   # Environnement dev Docker
```

**Documentation détaillée par domaine :**
- `backend/CLAUDE.md` : API endpoints, Storage Manager, patterns, compilation
- `frontend/CLAUDE.md` : Apps, composants, WebSocket events, commandes dev
- `libreelec/CLAUDE.md` : Architecture, contraintes, filesystem, services, build
- `app-catalog/CLAUDE.md` : Format manifest, types d'installation, variables

---

## Dev Mode

`PINAS_DEV_MODE=true` simule toutes les opérations système (stockage, packages, services) pour le développement local sans Raspberry Pi. Exposé via `/api/system/info` → `dev_mode: bool` → store `devMode`.

---

## Variables d'environnement

Le backend lit ses variables via `config::Environment::with_prefix("PINAS")` (`backend/src/config/mod.rs`) : chaque champ `snake_case` de `AppConfig` correspond à `PINAS_<CHAMP>`. Les valeurs ci-dessous sont celles de `libreelec/packages/pinas/system.d/pinas.service` (production) ; `backend/.env.dev` fournit l'équivalent dev (port 3388, `PINAS_DEV_MODE=true`).

```bash
# --- AppConfig (production sur LibreELEC) ---
PINAS_BIND_ADDRESS=0.0.0.0:3000
PINAS_DATABASE_URL=sqlite:/storage/.pinas/data/pinas.db?mode=rwc
PINAS_JWT_SECRET=<auto-généré au 1er démarrage dans ${PINAS_DATA_DIR}/.jwt_secret>
PINAS_JWT_EXPIRATION_HOURS=24
PINAS_FILES_ROOT=/storage/.pinas/files
PINAS_HOMES_ROOT=/storage/.pinas/homes
PINAS_HOME_ON_DELETE=archive           # archive, delete, ou keep
PINAS_STATIC_DIR=/storage/.pinas/www   # frontend servi par tower-http
PINAS_DEV_MODE=false                   # true pour simuler les opérations système
PINAS_KODI_USERNAME=kodi
PINAS_KODI_PASSWORD=<auto-généré dans ${PINAS_DATA_DIR}/.kodi_password>
PINAS_TLS_ENABLED=false                # true = HTTPS auto-signé (rcgen) dans ${PINAS_DATA_DIR}/.tls/

# --- Lus directement via std::env::var (services) ---
PINAS_DATA_DIR=/storage/.pinas/data    # secrets, TLS, samba/, etc.
PINAS_PACKAGES_DIR=/storage/.pinas/packages
PINAS_DOWNLOADS_DIR=/storage/.pinas/downloads
PINAS_BIN_DIR=/storage/.pinas/bin
PINAS_POOLS_PATH=<override du chemin de montage des pools>
PINAS_CATALOG_URL=https://raw.githubusercontent.com/kameka22/pinas-app-catalog/master/catalog.json
PINAS_GITHUB_OWNER=kameka22            # source des releases pour l'auto-update
PINAS_GITHUB_REPO=pinas
DOCKER_HOST=<socket Docker, optionnel>

# --- Logs : tracing EnvFilter (défaut si absent : pinas=debug,tower_http=debug) ---
RUST_LOG=pinas=info,tower_http=info

# Frontend : API_BASE est codé en dur à '/api' dans src/lib/stores/api.ts
# (proxy Vite → http://localhost:3388 en dev, même origine en prod).
```

> Seul `RUST_LOG` est lu pour le niveau de log (`PINAS_LOG_LEVEL` n'existe pas côté code).

---

## Conventions de code

### Rust
- `thiserror` pour les erreurs custom
- Async/await partout (Tokio runtime)
- Structs avec `#[derive(Debug, Serialize, Deserialize)]`
- Tests dans le même fichier avec `#[cfg(test)]`
- Frontend servi depuis `/storage/.pinas/www/` via tower-http

### Svelte
- Composants en PascalCase : `DiskManager.svelte`
- Stores dans `$lib/stores/`, composants apps dans `$lib/components/apps/`
- Types partagés dans les stores
- Adapter static pour SSG
- i18n via `$lib/i18n/`

### Git
- Commits conventionnels : `feat:`, `fix:`, `docs:`, `refactor:`
- Branches : `feature/`, `fix/`, `release/`

### Dev local
- Pas de cargo installé localement, builds via Docker
- `npm run check` / `npm run build` échouent en local si le binaire natif rollup manque (bug npm optionalDependencies) : `cd frontend && npm install --no-save @rollup/rollup-linux-arm64-gnu@$(node -p "require('./node_modules/rollup/package.json').version")` (adapter la plateforme), sans toucher au lock
- Pas de lint configuré (ESLint retiré) ; `npm run check` = svelte-check
- Docker n'est PAS une app built-in, il s'installe via App Center
