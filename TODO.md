# PiNAS - TODO List Complète

> **Projet** : NAS OS moderne pour Raspberry Pi 5
> **Stack** : LibreELEC + Rust + SvelteKit
> **Interface** : Desktop-like (inspiré UGOS / Synology DSM)
> **Distribution** : Addon LibreELEC (cohabitation avec Kodi)

---

## Phase 0 : Setup Projet

### 0.1 Structure initiale
- [x] Créer structure dossiers (`backend/`, `frontend/`, `docker/`)
- [x] Initialiser projet Rust (`cargo init backend`)
- [x] Initialiser projet SvelteKit (`npm create svelte@latest frontend`)
- [x] Configurer Git (`.gitignore`, hooks)
- [ ] Setup CI/CD basique (GitHub Actions)

### 0.2 Configuration développement
- [x] Docker Compose pour dev local (`docker/docker-compose/docker-compose.yml`)
- [ ] Script de dev unifié (`./dev.sh`)
- [ ] Hot reload backend (cargo-watch)
- [x] Proxy Vite vers backend Rust

---

## Phase 1 : Backend Rust - Fondations

### 1.1 Setup Axum
- [x] Créer structure API (`main.rs`, `api/`, `services/`)
- [x] Configurer Axum avec Tokio
- [x] Router de base avec healthcheck (`GET /api/health`)
- [x] Middleware logging (tower-http + tracing)
- [x] Middleware CORS
- [ ] Gestion d'erreurs centralisée
- [x] Configuration via variables d'environnement

### 1.2 Base de données SQLite
- [x] Setup SQLx avec SQLite
- [x] Migrations initiales :
  - [x] Table `users` (id, username, password_hash, is_admin, created_at)
  - [x] Table `sessions` (id, user_id, token, expires_at)
  - [x] Table `settings` (key, value, updated_at)
  - [x] Table `shares` (id, name, path, type, config, enabled)
  - [x] Table `notifications` (id, type, message, read, created_at)
  - [x] Table `installed_packages` (package manager)
  - [x] Table `package_tasks` (installation progress)
  - [x] Table `app_translations` (dynamic i18n)
- [x] Pool de connexions
- [x] Requêtes préparées

### 1.3 Authentification
- [x] Endpoint `POST /api/auth/login`
- [x] Endpoint `POST /api/auth/logout`
- [x] Endpoint `GET /api/auth/me`
- [x] JWT tokens (jsonwebtoken)
- [x] Password hashing (argon2)
- [x] Middleware auth (extraction du token)
- [ ] Refresh tokens (optionnel)

### 1.4 WebSocket
- [x] Setup WebSocket avec Axum
- [x] Broadcast channel pour events
- [x] Types d'events :
  - [x] `system.stats` (CPU, RAM, réseau)
  - [ ] `storage.update` (changements disques)
  - [ ] `notification.new`
  - [ ] `task.progress`
- [x] Reconnexion automatique côté client

---

## Phase 2 : Backend Rust - Services Système

### 2.1 System Service
- [x] `GET /api/system/info` - Infos système
  - [x] Hostname
  - [x] Version OS
  - [x] Uptime
  - [x] CPU model, cores, usage
  - [x] RAM total, used, available
  - [x] Load average
- [ ] `GET /api/system/services` - État des services
- [ ] `POST /api/system/reboot` - Redémarrer
- [ ] `POST /api/system/shutdown` - Éteindre
- [ ] `POST /api/system/hostname` - Changer hostname
- [x] Utiliser crate `sysinfo` pour les métriques
- [ ] Utiliser crate `nix` pour les appels système

### 2.2 Storage Service
- [ ] `GET /api/storage/disks` - Liste des disques
- [ ] `GET /api/storage/partitions` - Partitions
- [ ] `GET /api/storage/filesystems` - FS montés
- [ ] `POST /api/storage/mount` - Monter
- [ ] `POST /api/storage/unmount` - Démonter
- [ ] `POST /api/storage/format` - Formater (avec confirmation)

### 2.3 Share Service
- [ ] `GET /api/shares` - Liste des partages
- [ ] `POST /api/shares` - Créer partage
- [ ] `GET /api/shares/:id` - Détails partage
- [ ] `PUT /api/shares/:id` - Modifier partage
- [ ] `DELETE /api/shares/:id` - Supprimer partage

### 2.4 User Service
- [ ] `GET /api/users` - Liste utilisateurs
- [ ] `POST /api/users` - Créer utilisateur
- [ ] `GET /api/users/:id` - Détails utilisateur
- [ ] `PUT /api/users/:id` - Modifier utilisateur
- [ ] `DELETE /api/users/:id` - Supprimer utilisateur

### 2.5 Docker Service ✅
- [x] `GET /api/docker/status` - État Docker
- [x] `GET /api/docker/containers` - Liste containers
- [x] `POST /api/docker/containers/:id/start` - Démarrer
- [x] `POST /api/docker/containers/:id/stop` - Arrêter
- [x] `POST /api/docker/containers/:id/restart` - Redémarrer
- [x] `DELETE /api/docker/containers/:id` - Supprimer
- [x] `GET /api/docker/containers/:id/logs` - Logs
- [x] `GET /api/docker/images` - Liste images
- [x] Pull image via bollard
- [x] Create container via bollard

---

## Phase 3 : Frontend SvelteKit - Fondations

### 3.1 Setup projet
- [x] Créer projet SvelteKit avec TypeScript
- [x] Configurer TailwindCSS v4
- [x] Installer dépendances :
  - [x] `@iconify/svelte` (icônes)
  - [ ] `bits-ui` ou composants custom
- [x] Structure dossiers (`lib/`, `routes/`, `components/`)
- [ ] Thème dark/light avec CSS variables

### 3.2 Design System
- [ ] Tokens de design (couleurs, spacing, typography)
- [ ] Composants de base (Button, Input, Select, etc.)

### 3.3 Client API
- [x] Créer client API typé (`$lib/stores/api.ts`)
- [ ] Intercepteur pour JWT
- [ ] Types TypeScript pour toutes les réponses API
- [ ] Store pour état authentification

### 3.4 WebSocket Client
- [x] Store WebSocket (`$lib/stores/websocket.ts`)
- [x] Reconnexion automatique
- [x] Store système pour stats (`$lib/stores/system.ts`)

### 3.5 Internationalisation (i18n) ✅
- [x] Store i18n avec locale switching
- [x] Traductions EN/FR complètes
- [x] Support traductions dynamiques par app
- [x] Endpoint `GET /api/apps/:id/i18n/:locale`
- [x] Fonction `loadAppTranslations()` pour apps installées

---

## Phase 4 : Interface Desktop

### 4.1 Shell Desktop
- [x] Layout principal (`+layout.svelte`)
- [x] TopBar avec widgets système
- [x] Dock (barre inférieure - style macOS)
- [x] Zone de travail avec fond d'écran
- [x] App Launcher (menu applications)
- [x] Desktop Icons avec menu contextuel
- [x] Composant ContextMenu réutilisable

### 4.2 Window Manager ✅
- [x] Composant `Window` (drag, focus, minimize, maximize)
- [x] Store `windows` (positions, tailles, z-index)
- [x] Support `appConfig` pour composants dynamiques
- [x] Support `gradient` pour styling dynamique
- [x] Registre de composants (`apps/index.ts`)
- [x] Chargement dynamique des composants

### 4.3 Dashboard
- [x] Layout avec grid de widgets (UI mockup)
- [x] Widgets (CPU, RAM, Network, Storage)
- [ ] Connexion avec données réelles backend

### 4.4 Notifications
- [x] Centre de notifications
- [x] Types : info, success, warning, error

---

## Phase 5 : Applications (Fenêtres)

### 5.1-5.6 Applications existantes
- [x] Storage Manager (UI mockup)
- [x] File Manager (UI + connexion API)
- [x] Share Manager (UI mockup)
- [x] User Manager (complet style UGOS)
- [x] Control Panel (complet style UGOS)
- [x] System Settings (UI mockup)

### 5.9 Package Manager / App Center ✅ NOUVEAU
- [x] App Center (UI complète avec catégories, recherche, détails)
- [x] API backend complète :
  - [x] `GET /api/packages` - Liste packages installés
  - [x] `GET /api/packages/catalog` - Catalogue distant
  - [x] `POST /api/packages/install` - Installer package
  - [x] `DELETE /api/packages/:id` - Désinstaller
  - [x] `GET /api/packages/task/:id` - Statut installation
  - [x] `GET /api/apps/registry` - Apps avec fenêtre
- [x] Modèle Manifest complet (`models/manifest.rs`)
- [x] Service Package avec steps d'installation :
  - [x] Download avec SHA256
  - [x] Extract tar.gz
  - [x] Copy, Symlink, Chmod, Mkdir
  - [x] Template et WriteFile (base64)
  - [x] Exec commandes shell
  - [x] Docker steps (pull, create, start, stop, rm)
- [x] Substitution de variables (`${DATA_DIR}`, `${PACKAGES_DIR}`, etc.)
- [x] Catalogue distant GitHub (`kameka22/pinas-app-catalog`)
- [x] Fallback catalogue intégré

### 5.10 Docker App ✅
- [x] Interface complète (overview, containers, images)
- [x] Gauges CPU/RAM
- [x] Actions containers (start, stop, restart, remove)
- [x] Liste images avec taille et date
- [x] Connexion API Docker via bollard

### 5.11 Composants Apps Génériques ✅ NOUVEAU
- [x] `IframeApp.svelte` - Affiche app web dans iframe avec toolbar
- [x] `WebviewApp.svelte` - Placeholder + bouton ouvrir dans nouvel onglet
- [x] `ServiceApp.svelte` - UI gestion service (start/stop/restart/logs)
- [x] Registre composants (`apps/index.ts`)
- [x] Traductions i18n pour les 3 composants (EN/FR)

---

## Phase 6 : Onboarding ✅

### 6.1 First Setup Wizard
- [x] Page onboarding (`/onboarding`)
- [x] Step 1 : Choix de la langue
- [x] Step 2 : Nom du device
- [x] Step 3 : Nom d'utilisateur admin
- [x] Step 4 : Mot de passe
- [x] Store onboarding avec validation
- [x] Redirection vers desktop après completion

---

## Phase 7 : Addon LibreELEC

### 7.1-7.6 Package LibreELEC (Fonctionnel ✅)
- [x] Package `libreelec/packages/pinas/` complet
- [x] Cross-compilation aarch64-musl fonctionnelle
- [x] Script `scripts/build-arm64.sh`
- [x] Service systemd auto-activé au boot
- [x] Testé sur Raspberry Pi 5 avec LibreELEC 12

---

## Phase 8 : App Catalog ✅ NOUVEAU

### 8.1 Structure Catalog
- [x] Repository GitHub `kameka22/pinas-app-catalog`
- [x] Format `catalog.json` avec metadata apps
- [x] Support catégories (containers, media, utilities, network)

### 8.2 Manifests Apps
- [x] Docker manifest complet (installation binaires + systemd)
- [x] Portainer manifest (Docker + IframeApp)
- [x] Plex manifest (Docker + WebviewApp)
- [x] Pi-hole manifest (Docker + IframeApp)
- [x] Samba manifest (Binaire + ServiceApp)
- [ ] Nextcloud manifest
- [ ] Transmission manifest
- [ ] Home Assistant manifest

### 8.3 Format Manifest
- [x] Metadata (id, name, version, description, author, license)
- [x] Requirements (min_ram, min_disk, arch, dependencies)
- [x] Install config avec steps
- [x] Uninstall config avec steps
- [x] Files embarqués (base64)
- [x] Frontend config (component, icon, gradient, window, i18n)

---

## État actuel du projet

### Frontend (Avancé - UI complète + Apps dynamiques)
- **Interface desktop** : Shell complet avec TopBar, Dock, Window Manager
- **Design** : Style UGOS (light theme, glass morphism, gradients)
- **App Center** : Installation d'apps depuis catalogue distant
- **Composants génériques** : IframeApp, WebviewApp, ServiceApp
- **i18n** : Traductions EN/FR + support dynamique par app
- **Window Manager** : Support appConfig pour composants dynamiques

### Backend (Fonctionnel)
- **Package Manager** : Installation complète avec Docker support
- **Docker Service** : API complète via bollard
- **Services Manager** : API `/api/services` pour gestion systemd (start/stop/restart/logs)
- **Apps Registry** : Endpoint pour apps avec fenêtre
- **Substitution variables** : Chemins dynamiques dans manifests

### App Catalog (GitHub)
- **Repository** : `kameka22/pinas-app-catalog`
- **Apps disponibles** : Docker, Portainer, Plex, Pi-hole, Samba
- **Format** : Manifest JSON avec frontend config + i18n

### Package LibreELEC (Fonctionnel ✅)
- Cross-compilation aarch64-musl fonctionnelle
- Testé sur Raspberry Pi 5 avec LibreELEC 12

---

## Prochaines étapes

### Court terme
- [ ] Tester installation réelle d'apps sur Pi
- [ ] Ajouter plus d'apps au catalogue (Nextcloud, Transmission, Home Assistant)
- [x] Implémenter endpoint `/api/services` pour ServiceApp

### Moyen terme
- [ ] Connexion complète UI ↔ Backend pour toutes les apps
- [ ] Gestion utilisateurs fonctionnelle
- [ ] Partages SMB via interface

### Long terme
- [ ] Support NFS
- [ ] S.M.A.R.T. monitoring
- [ ] Synchronisation cloud (rclone)

---

*Dernière mise à jour : Janvier 2025*
*Cible OS : LibreELEC 12.x (package intégré à l'image)*
