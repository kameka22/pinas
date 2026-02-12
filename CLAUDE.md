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
├── TODO.md / MVP.md / BUILD.md
├── backend/                  # API Rust (voir backend/CLAUDE.md)
│   ├── src/
│   │   ├── api/              # Handlers REST + WebSocket + middleware
│   │   ├── services/         # Logique métier
│   │   ├── models/           # Structs DB
│   │   ├── config/           # Configuration
│   │   └── db/               # Pool SQLite
│   └── migrations/           # Migrations SQLite (001-004)
├── frontend/                 # UI SvelteKit (voir frontend/CLAUDE.md)
│   └── src/lib/
│       ├── components/       # desktop/, apps/, ui/, auth/, modals/, onboarding/
│       ├── stores/           # api, desktop, windows, websocket, system, update, onboarding
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
PINAS_HOMES_ROOT=/storage/homes
PINAS_HOME_ON_DELETE=archive           # archive, delete, ou keep
PINAS_DEV_MODE=false                   # true pour simuler les opérations

# Frontend (build-time)
PUBLIC_API_URL=/api
```

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
- `npm run check` peut échouer (rollup ARM64 mismatch), utiliser Docker
- Docker n'est PAS une app built-in, il s'installe via App Center
