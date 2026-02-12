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
  - [x] `task.progress` (progression installation packages)
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
- [x] `GET /api/system/services` - État des services (Samba réel, NFS/SSH stubs)
- [ ] `POST /api/system/reboot` - Redémarrer
- [ ] `POST /api/system/shutdown` - Éteindre
- [ ] `POST /api/system/hostname` - Changer hostname
- [x] Utiliser crate `sysinfo` pour les métriques
- [ ] Utiliser crate `nix` pour les appels système

### 2.2 Storage Service (Storage Manager complet) ✅

#### 2.2.1 Disques physiques
- [x] `GET /api/storage/disks` - Liste disques + partitions (lsblk)
- [x] `GET /api/storage/disks/:name/smart` - Données S.M.A.R.T. (smartctl)
- [x] `POST /api/storage/disks/:name/wipe` - Effacer disque (protégé si système)
- [x] `GET /api/storage/candidates` - Disques disponibles pour pools
- [x] Protection automatique disques système (/flash, /storage, mmcblk0)

#### 2.2.2 Pools de stockage
- [x] Migration DB `005_storage.sql` (storage_pools, storage_volumes)
- [x] `GET /api/storage/pools` - Liste des pools
- [x] `POST /api/storage/pools` - Créer pool (Basic, JBOD, RAID)
- [x] `GET /api/storage/pools/:id` - Détails pool
- [x] `PUT /api/storage/pools/:id` - Modifier pool (nom, description)
- [x] `DELETE /api/storage/pools/:id` - Supprimer pool
- [x] `POST /api/storage/pools/:id/scrub` - Vérification RAID

#### 2.2.3 Volumes
- [x] `GET /api/storage/volumes` - Liste volumes
- [x] `POST /api/storage/pools/:id/volumes` - Créer volume (mkfs)
- [x] `DELETE /api/storage/volumes/:id` - Supprimer volume
- [x] `POST /api/storage/volumes/:id/mount` - Monter
- [x] `POST /api/storage/volumes/:id/unmount` - Démonter
- [ ] `POST /api/storage/volumes/:id/resize` - Redimensionner

#### 2.2.4 Types RAID supportés
- [x] Basic (partition simple)
- [x] JBOD (Just a Bunch of Disks)
- [x] RAID 0, 1, 5, 10
- [x] Btrfs RAID (single, raid0, raid1, raid10 natif)

### 2.3 Share Service ✅ NOUVEAU
- [x] `GET /api/shares` - Liste des partages (enrichie avec config + permissions)
- [x] `POST /api/shares` - Créer partage (+ création dossier physique + smb.conf)
- [x] `GET /api/shares/:id` - Détails partage
- [x] `PUT /api/shares/:id` - Modifier partage (+ régénération smb.conf)
- [x] `DELETE /api/shares/:id` - Supprimer partage (+ nettoyage permissions + smb.conf)
- [x] `POST /api/shares/:id/toggle` - Activer/désactiver partage
- [x] `GET /api/shares/samba/status` - Statut service Samba (enabled, running, version, users connectés)
- [x] `POST /api/shares/samba/enable` - Activer Samba (systemctl enable --now smbd nmbd)
- [x] `POST /api/shares/samba/disable` - Désactiver Samba
- [x] `GET /api/shares/samba/config` - Config globale SMB (workgroup, protocols)
- [x] `PUT /api/shares/samba/config` - Modifier config globale SMB
- [x] Génération automatique `smb.conf` avec résolution permissions utilisateurs/groupes
- [x] Sync utilisateurs Samba (smbpasswd) lors création/suppression/changement mot de passe
- [x] Mode dev avec AtomicBool (pattern CupsService)
- [x] Types : `SmbShareConfig`, `SmbGlobalConfig`, `SambaStatus`, `ShareInfo`, `SharePermissionEntry`

### 2.4 User Service ✅
- [x] `GET /api/users` - Liste utilisateurs
- [x] `POST /api/users` - Créer utilisateur
- [x] `GET /api/users/:id` - Détails utilisateur
- [x] `PUT /api/users/:id` - Modifier utilisateur
- [x] `DELETE /api/users/:id` - Supprimer utilisateur
- [x] Password hashing avec Argon2
- [x] Validation et contraintes

### 2.5 Groups Service ✅ NOUVEAU
- [x] `GET /api/groups` - Liste groupes
- [x] `POST /api/groups` - Créer groupe
- [x] `GET /api/groups/:id` - Détails groupe
- [x] `PUT /api/groups/:id` - Modifier groupe
- [x] `DELETE /api/groups/:id` - Supprimer groupe
- [x] `GET /api/groups/:id/members` - Membres du groupe
- [x] `POST /api/groups/:id/members` - Ajouter membre
- [x] `DELETE /api/groups/:id/members/:user_id` - Retirer membre
- [x] Groupes système (administrators, users)

### 2.6 Setup Service ✅ NOUVEAU
- [x] `GET /api/setup/status` - Vérifie si setup effectué
- [x] `POST /api/setup/complete` - Crée admin initial + retourne JWT
- [x] Auto-login après onboarding

### 2.7 Terminal Service ✅ NOUVEAU
- [x] `POST /api/terminal/exec` - Exécute commande shell
- [x] Blocage commandes dangereuses (rm -rf /, mkfs, etc.)
- [x] Mode dev (simulation sans exécution)
- [x] Timeout 30 secondes

### 2.8 Docker Service ✅
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

### 2.9 Home Service ✅ NOUVEAU
- [x] Configuration `homes_root` et `home_on_delete` dans config
- [x] Service `HomeService` (`services/home.rs`)
  - [x] `create_home(username)` - Crée dossier home avec sous-dossiers
  - [x] `handle_user_deletion(username)` - Archive/supprime/conserve selon config
  - [x] `get_home_path(username)` - Retourne chemin du home
- [x] Sous-dossiers par défaut : Documents, Downloads, Photos, Music, Videos
- [x] Intégration avec User Service (création/suppression utilisateur)
- [x] Création home admin lors du setup initial

### 2.10 Locations Service ✅ NOUVEAU
- [x] `GET /api/locations` - Emplacements navigables pour File Manager
  - [x] Home directories utilisateurs
  - [x] Partages (shares)
  - [x] Volumes montés (admin uniquement)
- [x] Support `location_id` dans API Files
  - [x] `GET /api/files?location_id=...`
  - [x] `POST /api/files/folder` avec location_id
  - [x] `DELETE /api/files` avec location_id
  - [x] `PATCH /api/files/rename` avec location_id

### 2.11 SSH Service ✅ NOUVEAU
- [x] `GET /api/ssh/status` - État du service SSH
- [x] `POST /api/ssh/enable` - Activer SSH
- [x] `POST /api/ssh/disable` - Désactiver SSH
- [x] `POST /api/ssh/password` - Changer mot de passe root
- [x] Support mode dev (simulation sans systemctl)
- [x] Compatibilité LibreELEC (OpenSSH/sshd)
- [x] Création automatique répertoire config `/storage/.cache/services`

### 2.12 Permissions Service ✅ NOUVEAU
- [x] `GET /api/permissions` - Liste permissions par dossier
- [x] `GET /api/permissions/folders` - Liste dossiers avec permissions
- [x] `GET /api/permissions/folder?path=...` - Permissions d'un dossier
- [x] `GET /api/permissions/user/:id` - Permissions d'un utilisateur
- [x] `POST /api/permissions` - Créer permission
- [x] `PUT /api/permissions/:id` - Modifier permission
- [x] `DELETE /api/permissions/:id` - Supprimer permission
- [x] Niveaux: none, read, write
- [x] Création automatique permission write sur home à la création utilisateur

### 2.13 Network Service ✅ NOUVEAU
- [x] `GET /api/network/status` - Interfaces, DNS, gateway, hostname
- [x] `PUT /api/network/interface` - Configure une interface (DHCP/static)
- [x] `PUT /api/network/dns` - Configure les DNS
- [x] `PUT /api/network/hostname` - Change le hostname
- [x] Support connman (LibreELEC network manager)
- [x] Mode dev (simulation)

### 2.14 CUPS Service ✅ NOUVEAU
- [x] `GET /api/cups/status` - État du service CUPS
- [x] `POST /api/cups/enable` - Activer CUPS
- [x] `POST /api/cups/disable` - Désactiver CUPS
- [x] `GET /api/cups/printers` - Liste imprimantes configurées
- [x] `POST /api/cups/printers` - Ajouter imprimante
- [x] `DELETE /api/cups/printers/:name` - Supprimer imprimante
- [x] `PUT /api/cups/printers/:name` - Modifier imprimante
- [x] `POST /api/cups/printers/:name/test` - Page de test
- [x] `GET /api/cups/detect` - Détecter imprimantes USB
- [x] `GET /api/cups/drivers?uri=...` - Drivers disponibles
- [x] `GET /api/cups/jobs` - File d'attente impression
- [x] `DELETE /api/cups/jobs/:id` - Annuler job
- [x] Guard `require_enabled` (retourne 503 si désactivé)
- [x] Mode dev avec OnceLock/AtomicBool pour état simulé
- [x] Package LibreELEC CUPS (`libreelec/packages/cups/`)

### 2.15 Docker Compose Support ✅ NOUVEAU
- [x] `ComposeUp` step dans InstallStep (écrit docker-compose.yml + `docker compose up -d`)
- [x] `ComposeDown` step dans InstallStep (`docker compose down`)
- [x] Substitution variables dans contenu YAML
- [x] Support `project_name` et `remove_volumes`
- [x] Champs étendus dans ContainerConfig :
  - [x] `cap_add`, `cap_drop`, `user`, `command`, `entrypoint`
  - [x] `dns`, `extra_hosts`, `tmpfs`
- [x] Fix `network_mode` dans `create_container()`
- [x] Variables supplémentaires : `${APP_DATA_DIR}`, `${DEVICE_HOSTNAME}`

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
- [x] Storage Manager (complet - pools, volumes, disks, partitions, S.M.A.R.T., wipe)
- [x] File Manager (complet - sidebar dynamique avec home/shares/volumes)
- [x] Share Manager (complet - CRUD API, create/edit/delete modals, toggle, permissions)
- [x] User Manager (complet style UGOS)
- [x] Control Panel (complet style UGOS)
- [x] System Settings (UI mockup)

### 5.14 File Manager amélioré ✅ NOUVEAU
- [x] Sidebar dynamique avec sections collapsibles
  - [x] Section "Personnel" (dossier home utilisateur)
  - [x] Section "Dossiers partagés" (shares actifs)
  - [x] Section "Volumes" (volumes montés, admin uniquement)
- [x] Navigation entre locations (home, shares, volumes)
- [x] Barre d'utilisation pour volumes
- [x] Indicateurs de statut (monté/démonté, activé/désactivé)
- [x] i18n complet (EN/FR) pour les nouvelles sections

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
- [x] Installation en background (tokio::spawn) avec suivi progression WebSocket
  - [x] `install_start()` : crée enregistrements DB, retourne task_id immédiatement
  - [x] `install_execute()` : exécute les steps en tâche de fond
  - [x] Progression en temps réel via WebSocket (`task.progress` events)
- [x] Substitution de variables (`${DATA_DIR}`, `${PACKAGES_DIR}`, etc.)
- [x] Catalogue distant GitHub (`kameka22/pinas-app-catalog`)
- [x] Fallback catalogue intégré

### 5.10 Docker App ✅
- [x] Interface complète (overview, containers, images)
- [x] Gauges CPU/RAM
- [x] Actions containers (start, stop, restart, remove)
- [x] Liste images avec taille et date
- [x] Connexion API Docker via bollard

### 5.11 Terminal App ✅ NOUVEAU
- [x] Interface terminal complète (style console)
- [x] Prompt personnalisé `pinas@host:~$`
- [x] Historique commandes (flèches haut/bas)
- [x] Commandes built-in (help, clear, history)
- [x] Raccourcis clavier (Ctrl+C, Ctrl+L)
- [x] Connexion API backend `/api/terminal/exec`
- [x] Auto-scroll et focus management

### 5.12 Process Manager ✅ NOUVEAU
- [x] Interface gestionnaire de processus (style Task Manager)
- [x] Stats système (CPU, mémoire)
- [x] Liste processus triable (PID, nom, CPU%, RAM)
- [x] Barre de recherche/filtre
- [x] Connexion API backend avec données réelles
- [x] Kill process avec confirmation
- [x] Auto-refresh toutes les 3 secondes
- [x] i18n complet (EN/FR)

### 5.13 Composants Apps Génériques ✅
- [x] `IframeApp.svelte` - Affiche app web dans iframe avec toolbar
- [x] `WebviewApp.svelte` - Placeholder + bouton ouvrir dans nouvel onglet
- [x] `ServiceApp.svelte` - UI gestion service (start/stop/restart/logs)
- [x] Registre composants (`apps/index.ts`)
- [x] Traductions i18n pour les 3 composants (EN/FR)

### 5.15 File Service (Control Panel) ✅ NOUVEAU
- [x] Interface avec onglets (SMB, NFS, FTP)
- [x] Intégration dans Control Panel (section "File Service")
- [x] i18n complet (EN/FR)
- [x] Onglet SMB (complet - enable/disable toggle, statut, config globale, liste partages actifs, bouton "Gérer les partages")
- [ ] Onglet NFS (placeholder)
- [ ] Onglet FTP (placeholder)

### 5.17 Terminal Settings (Control Panel) ✅ NOUVEAU
- [x] Composant `TerminalSettings.svelte` dédié
- [x] SSH déplacé de File Service vers catégorie Terminal
- [x] Toggle enable/disable SSH
- [x] Affichage statut (running/stopped)
- [x] Affichage port
- [x] Changement mot de passe (modal)
- [x] Informations connexion (`ssh root@IP -p port`)
- [x] Intégration dans Control Panel (section "Terminal")
- [x] i18n complet (EN/FR)

### 5.16 FolderPicker Component ✅ NOUVEAU
- [x] Composant réutilisable pour sélection de dossiers
- [x] Navigation dans les locations (home, shares, volumes)
- [x] Breadcrumbs et navigation parent
- [x] Intégré dans :
  - [x] UserManager (permissions modal)
  - [x] ShareManager (create share modal)
  - [x] KodiApp (add media source modal)
- [x] i18n complet (EN/FR)

### 5.18 Network Settings (Control Panel) ✅ NOUVEAU
- [x] Composant `NetworkSettings.svelte` dédié
- [x] Configuration interfaces réseau (DHCP/Static)
- [x] Configuration DNS
- [x] Configuration hostname
- [x] Intégration dans Control Panel (section "Network")
- [x] i18n complet (EN/FR)

### 5.19 Printer Settings (Control Panel) ✅ NOUVEAU
- [x] Composant `PrinterSettings.svelte` dédié
- [x] Toggle enable/disable CUPS
- [x] Détection imprimantes USB
- [x] Ajout/suppression/modification imprimantes
- [x] File d'attente d'impression (jobs)
- [x] Page de test
- [x] Informations protocoles (IPP, AirPrint, raw queue)
- [x] Intégration dans Control Panel (section "Printer")
- [x] i18n complet (EN/FR)

---

## Phase 6 : Onboarding ✅

### 6.1 First Setup Wizard (7 étapes)
- [x] Page onboarding (`/onboarding`)
- [x] Step 1 : Bienvenue (welcome screen avec logo et description)
- [x] Step 2 : Choix de la langue (FR/EN)
- [x] Step 3 : Nom du device
- [x] Step 4 : Nom d'utilisateur admin
- [x] Step 5 : Mot de passe admin
- [x] Step 6 : Activation SSH (toggle + mot de passe optionnel)
- [x] Step 7 : Sélection des fonctionnalités (Docker, Samba, etc.)
- [x] Store onboarding avec validation par étape
- [x] Redirection vers desktop après completion
- [x] Connexion API backend `/api/setup/complete`
- [x] Auto-login avec JWT après création admin

---

## Phase 7 : Addon LibreELEC

### 7.1-7.6 Package LibreELEC (Fonctionnel ✅)
- [x] Package `libreelec/packages/pinas/` complet
- [x] Cross-compilation aarch64-musl fonctionnelle
- [x] Script `scripts/build-arm64.sh`
- [x] Service systemd auto-activé au boot
- [x] Testé sur Raspberry Pi 5 avec LibreELEC 12

---

## Phase 7.5 : Authentification & Gestion Utilisateurs ✅ NOUVEAU

### 7.5.1 Backend Auth Services
- [x] Service Auth (`services/auth.rs`) - Hash Argon2id + JWT
- [x] Service User (`services/user.rs`) - CRUD utilisateurs
- [x] Service Session (`services/session.rs`) - Gestion sessions
- [x] Service Group (`services/group.rs`) - CRUD groupes
- [x] Middleware Auth (`api/middleware.rs`) - Extraction JWT
- [x] Migration groupes et permissions

### 7.5.2 Frontend User Management
- [x] TopBar dropdown utilisateur (avatar, menu)
- [x] ProfileModal - Affichage/édition profil
- [x] ChangePasswordModal - Changement mot de passe
- [x] UserManager connecté aux vraies APIs
- [x] Gestion groupes dans UserManager
- [x] Traductions complètes (EN/FR)

### 7.5.3 Auth Flow
- [x] Login avec JWT storage
- [x] Logout avec suppression session
- [x] `POST /api/auth/change-password`
- [x] Protection routes admin

---

## Phase 8 : App Catalog ✅

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
- [x] 25 apps converties depuis Umbrel (Nextcloud, Jellyfin, Home Assistant, etc.)

### 8.4 Script Conversion Umbrel ✅ NOUVEAU
- [x] `scripts/convert-umbrel.py` - Convertit apps Umbrel → manifests PiNAS
- [x] `scripts/convert-umbrel-batch.sh` - Conversion batch
- [x] Support single-service → docker_pull/create/start
- [x] Support multi-service → compose_up avec YAML inline
- [x] Filtrage service app_proxy (Umbrel-spécifique)
- [x] Mapping variables Umbrel → PiNAS

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
- **App Center** : Installation d'apps depuis catalogue distant avec progression temps réel
- **Composants génériques** : IframeApp, WebviewApp, ServiceApp, DockerApp
- **i18n** : Traductions EN/FR + support dynamique par app + labelKey pour noms d'apps
- **Window Manager** : Support appConfig pour composants dynamiques
- **Terminal** : Application terminal web avec historique et commandes built-in
- **Process Manager** : Gestionnaire de processus avec données système réelles
- **User Dropdown** : Menu utilisateur dans TopBar (profil, mot de passe, logout)
- **Modales** : ProfileModal, ChangePasswordModal
- **Storage Manager** : Complet avec pools, volumes, disks, partitions, S.M.A.R.T., wipe, scrub
- **Share Manager** : Complet avec CRUD API, modals create/edit/delete, toggle enable/disable, permissions
- **File Service (SMB)** : Onglet SMB complet (enable/disable, statut, config globale workgroup/protocols, liste partages actifs)
- **File Manager** : Sidebar dynamique avec home/shares/volumes et navigation multi-locations
- **Control Panel** : SSH déplacé dans catégorie Terminal (`TerminalSettings.svelte`)
- **Mise à jour système** : Écran fullscreen (UpdateScreen) avec modal confirmation, progression WebSocket, mode dev test
- **Onboarding** : 7 étapes (welcome, langue, device, username, password, SSH, features)

### Backend (Fonctionnel)
- **Package Manager** : Installation en background (tokio::spawn) avec progression WebSocket
- **Docker Service** : API complète via bollard
- **Services Manager** : API `/api/services` pour gestion systemd (start/stop/restart/logs)
- **Apps Registry** : Endpoint pour apps avec fenêtre
- **Substitution variables** : Chemins dynamiques dans manifests
- **Auth complet** : JWT + Argon2 + sessions + middleware
- **Users/Groups** : CRUD complet avec API REST
- **Setup API** : Endpoint onboarding avec création admin
- **Terminal API** : Exécution commandes avec sécurité
- **Home Service** : Gestion automatique des dossiers home utilisateurs
- **Locations API** : Emplacements navigables (home, shares, volumes)
- **SSH Service** : Enable/disable SSH + changement mot de passe (compatible LibreELEC)
- **Permissions API** : CRUD permissions par dossier/utilisateur/groupe
- **Network Service** : Configuration interfaces, DNS, hostname (connman)
- **CUPS Service** : Partage imprimantes USB via IPP/AirPrint
- **Share Service (SMB)** : CRUD partages, génération smb.conf, sync smbpasswd, contrôle smbd/nmbd, config globale
- **Docker Compose** : Support apps multi-container (ComposeUp/ComposeDown)
- **Variables étendues** : `${APP_DATA_DIR}`, `${DEVICE_HOSTNAME}`

### App Catalog (GitHub)
- **Repository** : `kameka22/pinas-app-catalog`
- **Apps disponibles** : 28 apps (Docker, Portainer, Samba + 25 apps converties depuis Umbrel)
- **Catégories** : containers, media, network, utilities
- **Format** : Manifest JSON avec frontend config + i18n
- **Compose** : Support multi-container (Nextcloud, PhotoPrism, Paperless-ngx)

### Package LibreELEC (Fonctionnel ✅)
- Cross-compilation aarch64-musl fonctionnelle
- Testé sur Raspberry Pi 5 avec LibreELEC 12

---

## Prochaines étapes

### Terminé récemment ✅
- [x] **Service SMB / Partages de fichiers** (workflow Synology-like complet)
  - [x] Backend: ShareService complet (~900 lignes) avec CRUD, smb.conf, smbpasswd, smbd/nmbd
  - [x] Backend: API 11 endpoints (CRUD shares + samba status/enable/disable/config)
  - [x] Backend: Model enrichi (SmbShareConfig, SmbGlobalConfig, SambaStatus, ShareInfo)
  - [x] Backend: Sync utilisateurs Samba (create/delete/change password → smbpasswd)
  - [x] Backend: Fix get_services avec statut Samba réel (plus de stub hardcodé)
  - [x] Frontend: ShareManager complet (CRUD, modals create/edit/delete, toggle, loading states)
  - [x] Frontend: FileService onglet SMB (enable/disable, statut, config globale, partages actifs)
  - [x] Frontend: API client TypeScript (interfaces + méthodes pour tous les endpoints)
  - [x] i18n EN/FR complet pour SMB et partages
- [x] **Écran fullscreen de mise à jour** (style Synology DSM)
  - [x] Backend: `dev_mode` exposé dans SystemInfo (`/api/system/info`)
  - [x] Store `update.ts` pour état écran fullscreen
  - [x] `UpdateScreen.svelte` : overlay fullscreen 4 phases (starting, progress, completed, error)
  - [x] `UpdateSettings.svelte` : modal de confirmation + dispatch via store
  - [x] Bouton "Test update screen" en dev mode (simulation complète sans API)
  - [x] Intégration dans `+layout.svelte`
  - [x] i18n EN/FR (`systemUpdate.screen.*`)
- [x] **CUPS Printer Sharing**
  - [x] Backend: Service CUPS complet (detect, add, remove, jobs, test page)
  - [x] Frontend: PrinterSettings.svelte dans Control Panel
  - [x] Package LibreELEC CUPS (autotools build, cupsd.conf)
  - [x] Intégration build scripts (arm64, x86, libreelec-image)
- [x] **Network Settings**
  - [x] Backend: Service Network (connman, interfaces, DNS, hostname)
  - [x] Frontend: NetworkSettings.svelte dans Control Panel
- [x] **Docker Compose & Umbrel Apps**
  - [x] ComposeUp/ComposeDown steps dans package.rs
  - [x] ContainerConfig étendu (cap_add, user, command, tmpfs, etc.)
  - [x] Fix network_mode dans create_container()
  - [x] Script convert-umbrel.py + batch conversion
  - [x] 25 apps converties depuis Umbrel (28 total dans le catalogue)
- [x] **Onboarding étendu à 7 étapes**
- [x] **Installation packages en background** (tokio::spawn + WebSocket progress)
- [x] **Storage Manager complet** (pools, RAID, volumes, S.M.A.R.T.)
- [x] **Home directories & File Manager dynamique**

### Court terme
- [ ] Tester installation réelle d'apps sur Pi (Syncthing, Jellyfin, Nextcloud)
- [x] Implémenter endpoint `/api/services` pour ServiceApp
- [x] Connecter Process Manager aux vraies données système
- [ ] Améliorer Terminal avec auto-completion

### Moyen terme
- [ ] Connexion complète UI ↔ Backend pour toutes les apps
- [x] Gestion utilisateurs fonctionnelle
- [x] Partages SMB via interface (complet avec génération smb.conf + permissions)
- [x] Storage Manager connecté au backend
- [x] File Manager avec locations dynamiques
- [ ] Volume resize
- [ ] Drag & drop dans File Manager
- [ ] Upload fichiers dans File Manager

### Long terme
- [ ] Support NFS
- [x] S.M.A.R.T. monitoring (intégré au Storage Manager)
- [ ] Synchronisation cloud (rclone)
- [ ] Backup/Restore système
- [ ] Real-time updates via WebSocket pour Storage Manager
- [x] Permissions par dossier/volume (API + UI Permission viewer)

---

*Dernière mise à jour : 12 Février 2026*
*Cible OS : LibreELEC 12.x (package intégré à l'image)*
