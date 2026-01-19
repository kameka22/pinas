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
- [ ] Setup SQLx avec SQLite
- [ ] Migrations initiales :
  - [ ] Table `users` (id, username, password_hash, is_admin, created_at)
  - [ ] Table `sessions` (id, user_id, token, expires_at)
  - [ ] Table `settings` (key, value, updated_at)
  - [ ] Table `shares` (id, name, path, type, config, enabled)
  - [ ] Table `notifications` (id, type, message, read, created_at)
- [ ] Pool de connexions
- [ ] Requêtes préparées

### 1.3 Authentification
- [ ] Endpoint `POST /api/auth/login`
- [ ] Endpoint `POST /api/auth/logout`
- [ ] Endpoint `GET /api/auth/me`
- [ ] JWT tokens (jsonwebtoken)
- [ ] Password hashing (argon2)
- [ ] Middleware auth (extraction du token)
- [ ] Refresh tokens (optionnel)

### 1.4 WebSocket
- [ ] Setup WebSocket avec Axum
- [ ] Broadcast channel pour events
- [ ] Types d'events :
  - [ ] `system.stats` (CPU, RAM, réseau)
  - [ ] `storage.update` (changements disques)
  - [ ] `notification.new`
  - [ ] `task.progress`
- [ ] Reconnexion automatique côté client

---

## Phase 2 : Backend Rust - Services Système

### 2.1 System Service
- [ ] `GET /api/system/info` - Infos système
  - [ ] Hostname
  - [ ] Version OS
  - [ ] Uptime
  - [ ] CPU model, cores, usage
  - [ ] RAM total, used, available
  - [ ] Load average
- [ ] `GET /api/system/services` - État des services
- [ ] `POST /api/system/reboot` - Redémarrer
- [ ] `POST /api/system/shutdown` - Éteindre
- [ ] `POST /api/system/hostname` - Changer hostname
- [ ] Utiliser crate `sysinfo` pour les métriques
- [ ] Utiliser crate `nix` pour les appels système

### 2.2 Storage Service
- [ ] `GET /api/storage/disks` - Liste des disques
  - [ ] Device path
  - [ ] Model, serial
  - [ ] Size
  - [ ] Type (HDD/SSD/NVMe)
  - [ ] S.M.A.R.T. status
- [ ] `GET /api/storage/partitions` - Partitions
- [ ] `GET /api/storage/filesystems` - FS montés
- [ ] `POST /api/storage/mount` - Monter
- [ ] `POST /api/storage/unmount` - Démonter
- [ ] `POST /api/storage/format` - Formater (avec confirmation)
- [ ] Parser `/proc/partitions`, `/proc/mounts`
- [ ] Commandes : `lsblk`, `blkid`, `mount`, `umount`

### 2.3 Share Service
- [ ] `GET /api/shares` - Liste des partages
- [ ] `POST /api/shares` - Créer partage
- [ ] `GET /api/shares/:id` - Détails partage
- [ ] `PUT /api/shares/:id` - Modifier partage
- [ ] `DELETE /api/shares/:id` - Supprimer partage
- [ ] Support types : SMB, NFS
- [ ] Génération config Samba (`smb.conf`)
- [ ] Génération config NFS (`/etc/exports`)
- [ ] Restart services après modification

### 2.4 User Service
- [ ] `GET /api/users` - Liste utilisateurs
- [ ] `POST /api/users` - Créer utilisateur
- [ ] `GET /api/users/:id` - Détails utilisateur
- [ ] `PUT /api/users/:id` - Modifier utilisateur
- [ ] `DELETE /api/users/:id` - Supprimer utilisateur
- [ ] `GET /api/groups` - Liste groupes
- [ ] `POST /api/groups` - Créer groupe
- [ ] Gestion permissions sur partages
- [ ] Sync avec utilisateurs système Linux

### 2.5 Network Service
- [ ] `GET /api/network/interfaces` - Interfaces réseau
- [ ] `GET /api/network/status` - État connectivité
- [ ] `PUT /api/network/interface/:name` - Configurer interface
- [ ] Support DHCP et IP statique
- [ ] Configuration DNS

### 2.6 Smart Service
- [ ] `GET /api/smart/devices` - Devices S.M.A.R.T.
- [ ] `GET /api/smart/device/:id` - Détails device
- [ ] `POST /api/smart/test/:id` - Lancer test
- [ ] Parser sortie `smartctl`
- [ ] Alertes sur attributs critiques

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
- [ ] Tokens de design :
  - [ ] Couleurs (primary, secondary, accent, semantic)
  - [ ] Spacing (4px base unit)
  - [ ] Typography (fonts, sizes)
  - [ ] Shadows (3 niveaux)
  - [ ] Border radius
  - [ ] Transitions
- [ ] Composants de base :
  - [ ] `Button` (variants: primary, secondary, ghost, danger)
  - [ ] `Input` (text, password, number)
  - [ ] `Select`
  - [ ] `Checkbox` / `Switch`
  - [ ] `Card`
  - [ ] `Badge`
  - [ ] `Avatar`
  - [ ] `Spinner` / `Skeleton`
  - [ ] `Modal`
  - [ ] `Dropdown`
  - [ ] `Tabs`
  - [ ] `Table`
  - [ ] `Toast` / `Notification`

### 3.3 Client API
- [ ] Créer client API typé (`$lib/api/client.ts`)
- [ ] Intercepteur pour JWT
- [ ] Gestion erreurs centralisée
- [ ] Types TypeScript pour toutes les réponses API
- [ ] Store pour état authentification

### 3.4 WebSocket Client
- [x] Store WebSocket (`$lib/stores/websocket.ts`)
- [x] Reconnexion automatique
- [ ] Dispatcher d'événements
- [x] Store système pour stats (`$lib/stores/system.ts`)

---

## Phase 4 : Interface Desktop

### 4.1 Shell Desktop
- [x] Layout principal (`+layout.svelte`)
- [x] TopBar :
  - [x] Logo / Bouton App Launcher
  - [ ] Barre de recherche globale
  - [x] Mini-widgets système (CPU, RAM, réseau)
  - [x] Centre de notifications (icône + badge)
  - [x] Menu utilisateur (avatar)
  - [ ] Toggle thème dark/light
- [x] Dock (barre inférieure - style macOS) :
  - [x] Icônes applications avec gradients
  - [x] Indicateurs apps ouvertes
  - [x] Animation hover (scale + translate)
  - [x] Section fenêtres minimisées
- [x] Zone de travail avec fond d'écran
- [x] App Launcher (menu applications) :
  - [x] Grille d'apps par catégories
  - [x] Barre de recherche
  - [x] Fermeture Escape / clic extérieur
  - [x] Menu contextuel (clic droit) pour ajouter/retirer du bureau
  - [x] Indicateur apps épinglées sur le bureau
- [x] Desktop Icons :
  - [x] Store de gestion des icônes épinglées (`$lib/stores/desktop.ts`)
  - [x] Persistance localStorage
  - [x] Grille compacte et alignée (80px)
  - [x] Menu contextuel (clic droit) : Open, Remove from Desktop
- [x] Composant ContextMenu réutilisable (`$lib/components/ui/ContextMenu.svelte`)

### 4.2 Window Manager
- [x] Composant `Window` :
  - [x] Header avec titre, icône, boutons (help, minimize, maximize, close)
  - [x] Contenu (slot)
  - [x] Draggable (avec contraintes)
  - [ ] Resizable (optionnel)
  - [x] Focus management (z-index)
  - [x] Design light avec glass morphism
- [x] Store `windows` :
  - [x] Liste des fenêtres ouvertes
  - [x] Fenêtre active
  - [x] Positions et tailles
  - [x] État (normal, minimized, maximized)
- [x] Actions :
  - [x] Ouvrir nouvelle fenêtre
  - [x] Fermer fenêtre
  - [x] Minimiser / Restaurer
  - [x] Maximiser / Restaurer
  - [x] Mettre au premier plan

### 4.3 Dashboard (fenêtre principale)
- [x] Layout avec grid de widgets (UI mockup)
- [x] Widgets (UI mockup) :
  - [x] System Info (hostname, uptime, version)
  - [x] CPU Gauge
  - [x] RAM Gauge
  - [x] Network Traffic (graphique placeholder)
  - [x] Storage Overview (barres d'usage)
  - [ ] Services Status
  - [ ] Recent Activity
  - [ ] Quick Actions
- [ ] Personnalisation layout (sauvegarde prefs)
- [ ] Connexion avec données réelles backend

### 4.4 Notifications
- [x] Centre de notifications (panel slide - `NotificationCenter.svelte`)
- [x] Types : info, success, warning, error
- [ ] Actions sur notifications
- [x] Marquer comme lu
- [x] Clear all
- [ ] Toast pour nouvelles notifications

---

## Phase 5 : Applications (Fenêtres)

### 5.1 Storage Manager
- [x] Vue liste des disques (UI mockup)
  - [x] Icône type (HDD/SSD/USB)
  - [x] Nom, taille, usage
  - [x] Status S.M.A.R.T.
- [x] Détails disque (sidebar click)
  - [x] Infos complètes
  - [x] Partitions
  - [ ] Historique S.M.A.R.T.
- [x] Actions UI :
  - [x] Monter / Démonter (boutons)
  - [ ] Formater (avec wizard)
  - [ ] Lancer test S.M.A.R.T.
- [ ] Connexion avec backend API

### 5.2 File Manager
- [x] Navigation dossiers (UI mockup)
- [x] Vue liste / grille toggle
- [x] Sidebar avec favoris
- [ ] Upload / Download
- [ ] Opérations (copy, move, delete, rename)
- [ ] Prévisualisation fichiers
- [ ] Connexion avec backend API

### 5.3 Share Manager
- [x] Liste des partages (UI mockup)
  - [x] Nom, chemin, type (SMB/NFS/FTP)
  - [x] Status (enabled/disabled)
  - [x] Nombre d'utilisateurs
- [x] Modal créer partage :
  - [x] Nom du partage
  - [x] Chemin source avec browse
  - [x] Type (SMB, NFS, FTP)
  - [ ] Permissions
- [x] Boutons modifier/supprimer
- [ ] Connexion avec backend API

### 5.4 User Manager (Design UGOS)
- [x] Onglet "User" :
  - [x] Tableau triable (Username, Description, Role, Status, Edit)
  - [x] Avatar circulaire + badge "Me" pour utilisateur courant
  - [x] Menu contextuel "..." (Edit, Delete)
  - [x] Bouton "Add" avec dropdown
  - [x] Filtre de recherche
  - [x] Modal "Permission viewer" avec matrice permissions/dossiers
- [x] Onglet "User Group" :
  - [x] Tableau (Group Name, Description, Members, Edit)
  - [x] Protection groupes système
- [x] Onglet "Advanced Settings" :
  - [x] Password strength rules (checkboxes)
  - [x] Password expiry rules (toggles, inputs)
  - [x] Permanent password users (table + Add/Remove)
- [x] Modals :
  - [x] Add User (username, description, password, role)
  - [x] Edit User (description, role, status, change password)
  - [x] Delete User (confirmation)
  - [x] Add/Edit Group
- [ ] Connexion avec backend API

### 5.5 Control Panel (Design UGOS)
- [x] Deux modes d'affichage :
  - [x] Vue grille avec icônes par catégories
  - [x] Vue détail avec sidebar
- [x] Bouton toggle (icône grille) pour basculer entre les vues
- [x] Sidebar avec catégories :
  - [x] Connection & Access (User Management, File Service, Device Connection, Domain/LDAP, Terminal)
  - [x] General (Hardware & Power, Time & Language, Network, Security, Indexing Service)
  - [x] Service (About)
- [x] Recherche dans les deux modes
- [x] Page About avec onglets (General, Storage, Service, Device analysis) :
  - [x] Device Info (Device Name, System Version, Device Owner)
  - [x] Device (Model, SN, Last startup, Power Time)
  - [x] Hardware (CPU, Memory)
  - [x] Network (IP, Speed, MAC)
- [x] User Management intégré dans Control Panel
- [ ] Connexion avec backend API

### 5.6 System Settings
- [x] Sidebar navigation (UI mockup)
- [x] Général :
  - [x] Hostname
  - [x] Timezone
  - [x] Langue
- [x] Network :
  - [x] Interface selection
  - [x] DHCP / Statique
  - [x] IP, Gateway, DNS
- [x] Services :
  - [x] Liste services (SMB, NFS, SSH, FTP)
  - [x] Toggle enable/disable
  - [x] Status badges
- [x] Power :
  - [x] Reboot / Shutdown buttons
  - [x] Scheduled tasks UI
- [ ] Connexion avec backend API

### 5.7 Task Manager
- [ ] Liste des processus
- [ ] CPU/RAM par processus
- [ ] Kill process
- [ ] Graphiques temps réel

### 5.8 Terminal
- [ ] Émulateur de terminal xterm.js
- [ ] Connexion SSH au backend
- [ ] Historique commandes

### 5.9 Package Manager / Docker
- [ ] Installation Docker (binaires statiques depuis download.docker.com)
- [ ] Service systemd pour Docker dans /storage/.config/system.d/
- [ ] API backend via crate `bollard` (client Docker)
- [ ] Liste des containers (docker ps)
- [ ] Start/Stop/Restart containers
- [ ] Logs containers
- [ ] Catalogue d'apps pré-configurées (Plex, Nextcloud, Transmission, etc.)
- [ ] Installation apps en un clic

### 5.10 Security
- [ ] Firewall rules
- [ ] Certificats SSL
- [ ] Logs de sécurité

### 5.11 Logs Viewer
- [ ] Liste types de logs
- [ ] Vue temps réel (tail)
- [ ] Filtres (niveau, date)
- [ ] Recherche
- [ ] Export

### 5.12 Netdisk Tools
- [ ] Cloud sync (S3, Google Drive, etc.)
- [ ] Remote mount
- [ ] Scheduled sync tasks

---

## Phase 6 : Pages d'authentification

### 6.1 Login
- [ ] Page `/login`
- [ ] Formulaire (username, password)
- [ ] Remember me
- [ ] Messages d'erreur
- [ ] Redirect après login
- [ ] Design moderne (background, logo)

### 6.2 First Setup (optionnel)
- [ ] Wizard premier démarrage
- [ ] Création compte admin
- [ ] Configuration réseau basique
- [ ] Nom du NAS

---

## Phase 7 : Addon LibreELEC

### 7.1 Structure Package LibreELEC
- [x] Créer dossier `libreelec/packages/pinas/`
- [x] Fichier `package.mk` (définition package)
- [x] Service systemd `pinas.service`
- [x] Script init `pinas-init.sh`
- [x] Config tmpfiles `pinas.conf`
- [ ] Icône et fanart pour l'addon (optionnel)

### 7.2 Cross-compilation Backend
- [x] Target aarch64-unknown-linux-musl configuré
- [x] Script `scripts/build-arm64.sh` pour build complet
- [x] Binaire statique vérifié sur LibreELEC
- [x] Frontend servi depuis filesystem (tower-http)
- [x] Testé sur Raspberry Pi 5 avec LibreELEC 12

### 7.3 Intégration LibreELEC
- [x] Service systemd avec ExecStartPre pour init
- [x] Gestion des chemins `/storage/.pinas/`
- [x] Création automatique des dossiers au premier lancement
- [x] Configuration réseau (port 3000)
- [x] Logs vers `/storage/.pinas/logs/`
- [x] Frontend copié de `/usr/share/pinas/www/` vers `/storage/.pinas/www/`

### 7.4 Intégration Samba (addon existant)
- [ ] Analyser `service.samba` de LibreELEC
- [ ] API pour configurer Samba via PiNAS
- [ ] Génération dynamique de `smb.conf`
- [ ] Restart service Samba après modifications

### 7.5 Build & Distribution
- [x] Script `scripts/build-arm64.sh` (build image complète)
- [ ] Créer `repository.pinas` (repo d'addons Kodi - optionnel)
- [ ] Script de release automatisé
- [x] Documentation installation (README.md)

### 7.6 Tests sur LibreELEC
- [x] Tests d'intégration sur Raspberry Pi 5 réel
- [x] Vérification cohabitation avec Kodi
- [ ] VM LibreELEC pour tests CI (optionnel)

---

## Phase 8 : Tests & Qualité

### 8.1 Tests Backend
- [ ] Tests unitaires services
- [ ] Tests intégration API
- [ ] Tests authentification
- [ ] Mocking système de fichiers

### 8.2 Tests Frontend
- [ ] Tests composants (Vitest)
- [ ] Tests E2E (Playwright)
- [ ] Tests accessibilité

### 8.3 Performance
- [ ] Benchmarks API (latence, throughput)
- [ ] Profiling mémoire backend
- [ ] Optimisation bundle frontend
- [ ] Tests sur Raspberry Pi réel

---

## Phase 9 : Documentation & Finalisation

### 9.1 Documentation
- [x] README.md complet
- [x] Guide installation (dans README)
- [ ] Guide utilisateur détaillé
- [ ] API documentation (OpenAPI)
- [ ] Guide développeur

### 9.2 Release
- [ ] Versioning (semver)
- [ ] Changelog
- [ ] Release notes
- [ ] Images pré-buildées
- [ ] Script mise à jour

---

## Écrans à implémenter (Référence OMV)

### Dashboard
- [ ] Dashboard principal avec widgets
- [ ] Paramètres dashboard

### System
- [ ] Paramètres généraux
- [ ] Date & Heure
- [ ] Notifications
- [ ] Monitoring
- [ ] Certificats SSL/SSH
- [ ] Tâches planifiées (cron)
- [ ] Gestion énergie
- [ ] Mises à jour
- [ ] Plugins (futur)

### Storage
- [ ] Disques
- [ ] S.M.A.R.T.
  - [ ] Paramètres
  - [ ] Devices
  - [ ] Tâches planifiées
- [ ] Systèmes de fichiers
- [ ] Dossiers partagés
  - [ ] Liste
  - [ ] Permissions
  - [ ] ACL
  - [ ] Snapshots (si btrfs)
- [ ] Quotas

### Services
- [ ] SSH
- [ ] SMB/CIFS
  - [ ] Paramètres
  - [ ] Partages
- [ ] NFS
  - [ ] Paramètres
  - [ ] Partages
- [ ] Rsync
  - [ ] Tâches
  - [ ] Serveur

### Utilisateurs
- [ ] Utilisateurs
- [ ] Groupes
- [ ] Paramètres

### Réseau
- [ ] Paramètres généraux
- [ ] Interfaces
- [ ] Firewall
- [ ] Proxy

### Diagnostics
- [ ] Informations système
- [ ] Processus
- [ ] Logs système
- [ ] Statistiques performance
- [ ] Rapport système

---

## Priorités

### MVP (Minimum Viable Product)
1. Backend : Auth + System + Storage API (fonctionnel)
2. Frontend : Login + Desktop shell + Control Panel (connecté au backend)
3. Addon LibreELEC : Structure de base + binaire statique
4. Intégration Samba via addon existant
5. Test sur Raspberry Pi 5 avec LibreELEC

### V1.0
- Toutes les applications principales connectées au backend
- Partages SMB fonctionnels via interface PiNAS
- Gestion utilisateurs PiNAS
- Interface complète et responsive
- Repository d'addon pour installation facile

### V1.x+
- Partages NFS
- File Manager complet avec upload/download
- S.M.A.R.T. monitoring
- Intégration Kodi (widget dashboard, notifications)
- Support multi-disques (RAID logiciel si possible)
- Synchronisation cloud (rclone)

---

## État actuel du projet

### Frontend (Avancé - UI complète)
- **Interface desktop** : Shell complet avec TopBar, Dock (en bas), Window Manager
- **Design** : Style UGOS (light theme, glass morphism, gradients)
- **Dock** : Affiche apps épinglées + toutes les fenêtres ouvertes
- **Control Panel** : Menu de configuration principal style UGOS
  - Vue grille et vue détail avec sidebar
  - Catégories : Connection & Access, General, Service
  - Page About avec infos système (Device, Hardware, Network)
  - User Management intégré
- **User Manager** : Implémentation complète style UGOS (3 onglets, modals, tables)
- **Desktop** :
  - Store `desktop.ts` pour gérer les icônes épinglées
  - Menu contextuel sur icônes (Add/Remove from Desktop)
  - Grille compacte et alignée
- **Composants UI** : ContextMenu réutilisable
- **Applications mockup** : Storage, Files, Shares, Settings
- **Prochaine étape** : Connexion des UI avec le backend API

### Backend (Structure en place)
- Structure Axum de base en place
- Modules créés : auth, system, storage, shares, users, files, ws
- Endpoints API basiques implémentés
- WebSocket pour temps réel
- **Prochaine étape** : Implémenter les endpoints API fonctionnels complets

### Package LibreELEC (Fonctionnel ✅)
- Package `libreelec/packages/pinas/` complet
- Cross-compilation aarch64-musl fonctionnelle
- Script `scripts/build-arm64.sh` avec options (--frontend-only, --skip-libreelec, etc.)
- Service systemd auto-activé au boot
- Frontend copié automatiquement vers /storage/.pinas/www/
- **Testé et fonctionnel sur Raspberry Pi 5 avec LibreELEC 12**

### Infrastructure
- Docker Compose configuré pour dev local
- Structure projet organisée
- Build ARM64 natif sur VM Ubuntu ARM64
- **Cible** : LibreELEC 12.x sur Raspberry Pi 5

### Documentation
- `README.md` : Documentation utilisateur
- `CLAUDE.md` : Documentation technique complète
- `TODO.md` : Liste des tâches (ce fichier)

---

## Fonctionnalités futures (planifiées)

### Package Manager (style NAS commercial)
- Installation d'apps via interface web PiNAS
- Support Docker (binaires statiques depuis download.docker.com)
- Catalogue d'apps pré-configurées (Plex, Nextcloud, etc.)
- API backend pour gérer Docker via crate `bollard`

---

*Dernière mise à jour : Janvier 2025*
*Cible OS : LibreELEC 12.x (package intégré à l'image)*
