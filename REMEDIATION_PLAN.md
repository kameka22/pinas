# PiNAS — Plan de remédiation et de finalisation

> Établi le 2026-09-08 à partir de l'audit complet du dépôt (obsolescence, sécurité, architecture, fonctionnalités inachevées).
> Il couvre **tout ce qui a été identifié** : ce qui doit être corrigé, ce qui doit être terminé, et ce qui doit être retiré parce que ça ne sera pas fait.
> Chaque item a une taille (S ≈ ½ journée, M ≈ 1–2 j, L ≈ 3–5 j, XL > 1 semaine), ses dépendances et un critère de fin vérifiable.

---

## 0. Règles du jeu

| Sujet | Décision |
|---|---|
| Branches | **Pas de branches** : tout le travail se fait directement sur `master` en local, un commit par item ; rien n'est pushé sans demande explicite. (`fix/p0-security` et `chore/backend-deps-bump` ont été rapatriées sur `master` le 2026-09-08.) |
| Definition of Done | `cargo test` vert, `svelte-check` 0 erreur, `npm run build` OK, testé sur Pi (ou VM QEMU) pour tout ce qui touche au système, i18n en+fr à jour, README/TODO.md synchronisés. |
| Ce qu'on **retire** plutôt que de finir | Domain/LDAP, Indexing Service, onglet *Device Analysis*, `Settings.svelte`, alias `NetdiskTools`. Un NAS Pi mono-site n'en a pas besoin ; afficher une entrée "under development" est pire que ne rien afficher. |
| Hors périmètre de ce plan | La roadmap long terme de `TODO.md` §9 (rsync, quotas, UPS, SNMP, syslog distant, S3, LVM, bcache, cloud sync, RRD). Voir §9 de ce document pour la manière de la réintégrer une fois les phases 0–7 terminées. |

Ordre : **P0 → P1 → P2** sont séquentielles (sécurité, puis fondations sur lesquelles tout le reste s'appuie). **P3 à P7** sont indépendantes entre elles et peuvent se paralléliser. **P8** court en continu.

---

## Phase 0 — Sécurité (bloquant, avant toute exposition réseau)

Taille totale : **M–L (≈ 3 j)**. Aucune dépendance.

| # | Item | Taille | Détail / critère de fin |
|---|---|---|---|
| 0.1 ✅ | **Middleware d'authentification global** | M | `main.rs` : `middleware::from_fn_with_state(state, require_auth)` appliqué via `route_layer` à tous les nests sauf `/api/auth/login`, `/api/setup/*`, `/api/health`. Le middleware réutilise l'extraction cookie → Bearer de `middleware.rs`. Test : `curl` anonyme sur `/api/storage/disks`, `/api/docker/containers`, `/api/packages`, `/api/system/update/check`, `/api/cups/status`, `/api/system/processes` → 401. |
| 0.2 ✅ | **`AdminUser` sur toutes les opérations destructives** | S | storage (wipe, pools, volumes, snapshots, SMART test, power), docker (create/remove/stop/prune/pull), packages (install/uninstall), update (install/dismiss), cups (enable/disable/add/remove), system (kill_process, reboot, shutdown), services (start/stop/enable/disable). Lecture seule reste `AuthUser`. Critère : un utilisateur non-admin reçoit 403 sur chacune. |
| 0.3 ✅ | **Révocation de session effective** | S | `middleware.rs` : après `validate_jwt`, vérifier l'existence du token dans `sessions` (requête indexée sur `token`, déjà en place). Logout et `delete_user`/désactivation appellent `delete_user_sessions`. Tâche Tokio périodique `cleanup_expired_sessions` (toutes les heures). Critère : après logout, le même cookie renvoie 401. |
| 0.4 ✅ | **Intégrité des mises à jour** | S | `scripts/build-release.sh` publie `pinas-update-vX.tar.gz.sha256` à côté de l'archive ; `update.rs` télécharge les deux, refuse l'installation si le hash diffère, loggue l'échec. Étape suivante optionnelle : signature minisign (clé publique embarquée dans le binaire). Critère : archive altérée → `install_update` échoue sans rien extraire. |
| 0.5 ✅ | **Politique Samba `force user = root`** | S | Décision à prendre et documenter dans `SECURITY_AUDIT.md` : (a) conserver (LibreELEC mono-utilisateur, ACL émulées par `valid users/read list/write list`) en forçant `guest ok = no` quand des permissions existent, ou (b) créer des utilisateurs Unix dans `/storage/.pinas/etc/passwd` et pointer Samba dessus (`passdb backend = tdbsam`, `username map`). Recommandation : (a) + garde-fou. Critère : un partage avec `folder_permissions` refuse l'accès guest. |
| 0.6 ⏳ | **Validation systemd sur le Pi** (point 11 de l'audit) | S | Vérifier que `pinas.service` démarre avec `ProtectSystem=strict` + `ReadWritePaths=/etc /var` sur le squashfs LibreELEC ; que `smbpasswd -a` fonctionne (`/etc/passwd` inscriptible ?) ; sinon ajuster l'unité et `share.rs::sync_user`. Critère : `journalctl -u pinas` propre au boot, création d'utilisateur → accès SMB OK. |
| 0.7 ✅ | **Rate limiting login** | S | 5 tentatives / minute / IP sur `/api/auth/login` (même mécanisme que `terminal.rs`). Critère : 6ᵉ tentative → 429. |
| 0.8 ✅ | Mettre à jour `SECURITY_AUDIT.md` | S | Ajouter les items 0.1–0.7 avec leur statut, retirer les `~~` obsolètes. |

---

## Phase 1 — Fondations backend ✅ (terminée le 2026-09-08)

Taille : **L (≈ 5 j)**. Dépend de P0. Le bump de dépendances (axum 0.8, sqlx 0.9, bollard 0.21, édition 2024) est déjà sur `master` ; à valider sur le Pi avec le reste.

| # | Item | Taille | Détail / critère de fin |
|---|---|---|---|
| 1.1 ✅ | **Type d'erreur central `AppError`** | M | `api/error.rs` : enum `AppError { NotFound, BadRequest, Unauthorized, Forbidden, Conflict, Internal(anyhow::Error), Db(sqlx::Error), … }` + `impl IntoResponse` (JSON `{ error, code }`) + `From<…>` pour `ShareError`, `StorageError`, `AuthError`, `sqlx::Error`, `anyhow::Error`. Migrer les **146** constructions manuelles de `(StatusCode, Json(…))` vers `Result<Json<T>, AppError>`. Critère : `grep -c 'StatusCode::INTERNAL_SERVER_ERROR' api/` < 10. |
| 1.2 ✅ | **Plus d'I/O bloquantes dans l'async** | M | `storage.rs` (46), `cups.rs` (21), `share.rs` (17), `network.rs` (17), `service.rs` (10), `ssh.rs` (8) : remplacer `std::process::Command` par `tokio::process::Command`, `std::fs` par `tokio::fs`, et envelopper ce qui reste synchrone (parsing lourd, `System::new_all`) dans `spawn_blocking`. Supprimer le `std::thread::sleep(200 ms)` de `get_processes`. Critère : `grep -rn 'std::process::Command\|std::thread::sleep' services api` = 0 (hors tests). |
| 1.3 ✅ | **`sysinfo::System` partagé** | S | Un `Arc<RwLock<System>>` dans `AppState`, rafraîchi par la tâche WS toutes les 2 s ; `get_info`, `get_processes`, `kill_process` lisent cette instance au lieu de `System::new_all()` par requête. |
| 1.4 ✅ | **Nettoyage schéma** | S | Migration `012_cleanup.sql` : `DROP TABLE permissions` (remplacée par `folder_permissions` en 006). **Garder** `notifications` (utilisée en P3). Supprimer `services/system.rs` (fichier d'une ligne). Critère : chaque table de `migrations/` est référencée au moins une fois dans `src/`. |
| 1.5 ✅ | **Hygiène compilation** | S | Éliminer les 61 warnings (imports inutilisés, variables mortes) ; `#![deny(warnings)]` en CI via `RUSTFLAGS=-D warnings`. Passer les 58 `unwrap()/expect()` de code de prod en revue : garder ceux sur des invariants (`parse` de constantes), remplacer les autres par `?`/`AppError`. |
| 1.6 ✅ | **Tests backend** | M | Cible : un module de test par service critique — `auth`, `permission`, `share` (génération `smb.conf`), `package` (parsing manifest + validation des steps), `update` (`is_newer_version`, vérification sha256), `files` (`validate_path`). Base SQLite en mémoire via `sqlx::SqlitePool::connect(":memory:")` + migrations. Critère : ≥ 40 tests, `cargo test` en CI. |
| 1.7 ✅ | **Variables d'environnement** | S | Un seul chemin de lecture : tout dans `AppConfig` (les 11 `std::env::var` disséminés — `PINAS_DATA_DIR`, `PINAS_CATALOG_URL`, `PINAS_GITHUB_*`, `PINAS_POOLS_PATH`, `DOCKER_HOST`… — deviennent des champs). `CLAUDE.md` mis à jour en conséquence. |
| 1.8 ✅ | **Docker : compteur de conteneurs par réseau** | S | `list_networks` appelle `inspect_network` (ou `list_containers` filtré par `network`) pour recalculer `containers` (perdu avec l'API Engine 1.53). |

---

## Phase 2 — Fondations frontend ✅ (terminée le 2026-09-08)

Taille : **M–L (≈ 4 j)**. Dépend de P1 (format d'erreur JSON unifié).

| # | Item | Taille | Détail / critère de fin |
|---|---|---|---|
| 2.1 ✅ | **Un seul client HTTP** | M | Migrer les 27 `fetch('/api/…')` bruts (`DockerApp` 17, `AppCenter` 5, `ServiceApp` 3, `TerminalApp` 2) vers `api.ts` (méthodes typées `getContainers()`, `installPackage()`, …). Le client gère 401 → écran de login, 403 → toast, erreurs → `AppError` typé. Critère : `grep -rn "fetch(\`/api\|fetch('/api" components` = 0 (hors upload). |
| 2.2 ✅ | **Système de toasts** | S | Store `toasts.ts` + composant `ToastHost.svelte` dans le desktop. Remplacer les **12 `alert()`** (Kodi, DisplayApp, UserManager). |
| 2.3 ✅ | **i18n des 44 chaînes en dur** | S | `'Failed to update permission'`, `'Failed to add folder'`, `'Configuration options coming soon'`, `'Widgets'`, `'Dashboard'`… → clés `en.ts`/`fr.ts`. Critère : script CI qui échoue sur `alert('` et sur `: 'Failed` dans les `.svelte`. |
| 2.4 ✅ | **Supprimer le mock aléatoire de `ServiceApp`** | S | En cas d'erreur : état `error` + message, jamais `Math.random()`. Le mode simulé ne s'active que si `$systemInfo.devMode`. |
| 2.5 ✅ | **Suppressions** | S | `Settings.svelte` (orphelin, 6 boutons morts), alias `NetdiskTools → Dashboard` dans `index.ts`, `frontend/yarn.lock` (le projet est npm), dossiers `test/build_python_direct`, `build/`, `build.log`. |
| 2.6 ✅ | **Erreur `StorageManager.svelte:1036`** | S | `poolHealth[pool.id].last_scrub` possiblement null → chaînage optionnel. `svelte-check` = 0 erreur, et **ajouter `svelte-check` à la CI**. |
| 2.7 ✅ | **Tests frontend** | M | Vitest + `@testing-library/svelte`. Cibles : `api.ts` (gestion 401/403, parsing d'erreurs), stores `windows`/`desktop` (ouverture/focus/fermeture), `formatBytes`/`formatUptime`, un test de rendu par app critique (FileManager, StorageManager, UserManager) avec API mockée. Critère : ≥ 30 tests, en CI. |
| 2.8 ✅ | **Boutons morts** | S | Retirer ou brancher : TopBar *Widgets* et *Search* (→ P7), FileManager *Sort* (→ P6), TimeLanguage *Apply/Sync* (→ P4), UserManager *Apply* (→ P4), Dashboard *Refresh*, NotificationCenter *Settings* (→ P3). Aucun `<button>` sans handler ne doit rester (lint custom ou revue). |

---

## Phase 3 — Notifications réelles et Dashboard ✅ (terminée le 2026-09-08)

Taille : **M (≈ 2 j)**. Dépend de P1, P2. Indépendante de P4–P7.

| # | Item | Taille | Détail / critère de fin |
|---|---|---|---|
| 3.1 ✅ | **`NotificationService`** | S | CRUD sur la table `notifications` existante (id, type, title, message, read, source, created_at — ajouter `title`/`source` par migration). Endpoints `GET /api/notifications`, `POST /:id/read`, `POST /read-all`, `DELETE /:id`, `DELETE /`. |
| 3.2 ✅ | **Émission** | S | Brancher les producteurs existants : `StorageAlertEvent` (santé pool, SMART), `TaskProgressEvent` (fin/échec installation), `update.rs` (mise à jour disponible/appliquée), login échoué répété (P0.7), disque branché/débranché (udev via `locations`). Diffusion WS `notification` (le client la gère déjà). |
| 3.3 ✅ | **NotificationCenter live** | S | Remplacer les 4 notifications statiques par le store alimenté par API + WS ; *Dismiss*, *Clear all*, *Mark read* fonctionnels ; badge non-lus dans la TopBar ; bouton *Settings* → préférences (P4.5). |
| 3.4 ✅ | **Dashboard réel** | S | *Recent activity* = 10 dernières notifications ; *Services* = `GET /api/services` (smbd, sshd, docker, cupsd, kodi) avec état réel ; bouton *Refresh* fonctionnel ; widgets CPU/RAM déjà branchés sur le WS. Supprimer les tableaux codés en dur. |

---

## Phase 4 — Control Panel : terminer ou retirer ✅ (terminée le 2026-09-08)

Taille : **L (≈ 6 j)**. Dépend de P1, P2.

| # | Item | Taille | Détail / critère de fin |
|---|---|---|---|
| 4.1 ✅ | **Time & Language** (actuellement 0 appel API) | M | Backend `api/time.rs` : `GET /api/system/time` (timezone, NTP actif, heure, serveurs), `PUT` (timezone via `timedatectl set-timezone`, NTP on/off via `timedatectl set-ntp`, serveurs dans `/storage/.config/timesyncd.conf.d/`), `POST /sync` (`systemctl restart systemd-timesyncd`). Liste des fuseaux via `timedatectl list-timezones`. Langue : persister dans `user_preferences` (endpoint `/api/preferences` existe) et appliquer au login. Critère : changer le fuseau dans l'UI → `date` sur le Pi change. |
| 4.2 ✅ | **Hardware & Power** | M | Regrouper l'existant : reboot/shutdown (endpoints existants), gestion d'alimentation disques (`disk_power_settings` + `hdparm -S`, déjà en base), gouverneur CPU (`/sys/devices/system/cpu/cpufreq/…`), température/ventilateur (lecture `thermal_zone0`, déjà exposée), **arrêt/redémarrage planifié** (table `scheduled_tasks` + tâche Tokio cron minimaliste — pose la base pour les tâches planifiées du TODO §9.2). |
| 4.3 ✅ | **Security** | M | Vue et réglages : durée de session JWT (`jwt_expiration_hours` → `settings`), HTTPS on/off (`PINAS_TLS_ENABLED` déplacé dans `settings`, redémarrage du listener), régénération du certificat auto-signé, journal des connexions (table `login_attempts` alimentée par P0.7), blocage auto d'IP après N échecs, accès SSH (déjà existant → déplacer ici depuis `TerminalSettings`). |
| 4.4 ✅ | **Device Connection** | S | Hostname (endpoint `network` existant), mDNS/Avahi (`avahi-daemon` est dans LibreELEC : activer/désactiver, nom `pinas.local`), QR code d'accès LAN, et rappel des ports exposés. Pas de "QuickConnect" cloud. |
| 4.5 ✅ | **Password policy** (onglet Settings de UserManager) | S | Persister dans `settings` (`password.min_length`, `password.require_mixed`, `password.expiry_days`) ; enforcer dans `create_user`/`change_user_password` ; bouton *Apply* branché. |
| 4.6 ✅ | **Onglet About** | S | *Storage* = résumé pools/volumes (API storage), *Service* = liste services (API services). **Supprimer** *Device Analysis* (doublon de Process Manager). Retirer le placeholder `contentFor`. |
| 4.7 ✅ | **Retraits** | S | Entrées *Domain/LDAP* et *Indexing Service* supprimées de `ControlPanel.svelte` + clés i18n + `README.md`. |

---

## Phase 5 — File Service : NFS et FTP/SFTP

Taille : **L–XL (≈ 8 j)**, dont une part de build d'image LibreELEC. Dépend de P1. Décision préalable : **vaut-il la peine d'embarquer NFS/FTP dans l'image ?** Si non, supprimer les onglets et le type de share associé (S) et clore la phase.

| # | Item | Taille | Détail / critère de fin |
|---|---|---|---|
| 5.1 | **NFS serveur — image** | M | Activer `CONFIG_NFSD`, `CONFIG_NFSD_V4` dans `libreelec/projects/…/linux.aarch64.conf` (absent aujourd'hui, seul le client `CONFIG_NFS_FS` est actif) ; package `nfs-utils` LibreELEC compilé avec le serveur (`rpc.nfsd`, `exportfs`, `rpcbind`) ; unités `pinas-nfsd.service`, `pinas-rpcbind.service` dans `libreelec/packages/pinas/system.d/`. Critère : `exportfs -v` sur le Pi. |
| 5.2 | **NFS — backend** | M | `share.rs` : `share_type == "nfs"` → génération de `/storage/.pinas/data/nfs/exports` (`/path client(rw,sync,no_subtree_check,…)`), `exportfs -ra`, options : clients autorisés (CIDR), `rw/ro`, squash. Modèle `NfsConfig` dans `config` JSON du share. Mode dev simulé. |
| 5.3 | **NFS — UI** | S | Onglet NFS de `FileService.svelte` : liste des exports, création (dossier via `FolderPicker`, clients, options), activation/désactivation. |
| 5.4 | **SFTP** (sans nouveau démon) | S | Réutiliser `sshd` : sous-système SFTP déjà présent ; restreindre par utilisateur avec `Match User` + `ChrootDirectory` dans un `sshd_config.d` géré par PiNAS ; UI : toggle "accès SFTP" par utilisateur dans UserManager (rejoint `service_access`). |
| 5.5 | **FTP/FTPS** (optionnel) | L | Package `vsftpd` LibreELEC + `pinas-vsftpd.service` ; backend : config générée (utilisateurs virtuels depuis la base, TLS via le certificat auto-signé, passive ports) ; UI onglet FTP. **Recommandation : ne pas faire** — SFTP couvre le besoin, FTP en clair est une régression sécurité. Si non fait : supprimer l'onglet. |
| 5.6 | Dashboard/Services | S | Les services NFS/FTP apparaissent dans `GET /api/services` avec leur vrai état (aujourd'hui "NFS running" est codé en dur). |

---

## Phase 6 — File Manager complet ✅ (terminée le 2026-09-08)

Taille : **M (≈ 3 j)**. Dépend de P1, P2. Indépendante de P3–P5.

| # | Item | Taille | Détail / critère de fin |
|---|---|---|---|
| 6.1 ✅ | **Téléchargement** | S | `GET /api/files/download?path=&location_id=` (stream `tokio::fs::File` + `Content-Disposition`, `validate_path`, permission `can_read`) ; dossier → archive `zip` à la volée. Menu contextuel *Download*. **Aujourd'hui aucun moyen de récupérer un fichier depuis l'UI.** |
| 6.2 ✅ | **Ouvrir / aperçu** | M | Double-clic (`console.log` aujourd'hui) → `FilePreview.svelte` : images, texte/code (< 1 Mo), PDF (iframe), audio/vidéo via `Range` sur l'endpoint download ; autres types → téléchargement. |
| 6.3 ✅ | **Tri** | S | Bouton *Sort* → nom / taille / date / type, asc/desc, persisté dans `user_preferences`. |
| 6.4 ✅ | **Drag & drop** | M | Déposer des fichiers du bureau → upload (endpoint existant, `fileTasks` déjà en place pour la progression) ; glisser un élément sur un dossier ou une location de la sidebar → `move` ; `Ctrl` → `copy`. |
| 6.5 ✅ | **Renommer / propriétés** | S | Renommer inline (utilise `move`) ; panneau propriétés : taille, dates, permissions effectives (`folder_permissions`), partage SMB associé. |
| 6.6 ✅ | Nettoyage | S | Retirer les `console.log` de `FolderPicker.svelte` et `FileManager.svelte`. |

---

## Phase 7 — Desktop : widgets, recherche, finitions

Taille : **M (≈ 3 j)**. Dépend de P2, P3.

| # | Item | Taille | Détail / critère de fin |
|---|---|---|---|
| 7.1 | **Panneau Widgets** | M | Bouton TopBar *Widgets* (mort aujourd'hui) → panneau latéral : CPU/RAM/réseau (WS), stockage (pools, % utilisé), notifications récentes, services, conteneurs Docker. Disposition persistée dans `user_preferences`. |
| 7.2 | **Recherche globale** | M | Bouton *Search* → palette (`Ctrl+K`) : apps, entrées du Control Panel, utilisateurs, partages, et fichiers (endpoint `GET /api/files/search?q=` limité aux locations autorisées, `walkdir` borné + timeout). |
| 7.3 | **Sessions & préférences** | S | Fond d'écran, thème clair/sombre (le CSS a `darkMode: 'class'` sans toggle), langue, disposition des icônes — tout via `/api/preferences` (existe, utilisé 1 fois). |
| 7.4 | **Docker UI** | S | Afficher le compteur de conteneurs par réseau rétabli en 1.8 ; logs en flux (`follow=true`) plutôt que snapshot. |

---

## Phase 8 — Ops, CI, documentation (continu)

| # | Item | Taille | Détail / critère de fin |
|---|---|---|---|
| 8.1 ✅ | **CI complète** | S | Workflow `ci.yml` séparé de `build-libreelec.yml`, sur PR et push : `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `npm ci && npm run check && npm run build`, test de génération `smb.conf`. Le build d'image reste manuel/tag. |
| 8.2 | **Release** | S | `build-release.sh` : génère `.sha256` (P0.4), tag `vX.Y.Z` depuis `VERSION`, notes de release depuis les commits conventionnels. Une seule source de version : `VERSION` → injecté dans `Cargo.toml`/`package.json`/`package.mk` par le script (plus de dérive). |
| 8.3 | **Validation Pi** | M | Check-list exécutée à chaque release : boot + `journalctl -u pinas`, onboarding, login/logout (révocation), SMB depuis un client, Storage (pool RAID1 sur 2 clés USB), App Center (installer Jellyfin), Docker, Terminal, Process Manager, mise à jour depuis la version N-1, cohabitation Kodi (bascule `display`). Résultats consignés dans `docs/RELEASE_CHECKLIST.md`. |
| 8.4 | **App-catalog** | S | Tester l'upgrade Paperless 2 → 3 et Nextcloud 32 → 33 sur une instance existante avant push ; migrer `photoprism` de `mariadb:10.5.12` (EOL) vers `mariadb:11` avec procédure dump/restore documentée ; automatiser le script de vérification des tags (déjà écrit) en action GitHub hebdomadaire ouvrant une PR. |
| 8.5 | **Documentation** | S | `README.md` : retirer les fonctionnalités non livrées (NFS/FTP tant que P5 n'est pas faite), roadmap = ce plan. `TODO.md` : archiver les phases 0–8 terminées, ne garder que §9. `SECURITY_AUDIT.md` : P0. `CLAUDE.md` : `AppError`, client API unique, conventions de tests. Supprimer `MVP.md` des mentions, `exemple/` déplacé hors du repo ou documenté comme maquettes. |
| 8.6 | **Dépendances** | S | Dependabot/Renovate (npm + cargo, hebdo, groupé). `frontend` : passer à TypeScript 6 quand `svelte-check` le supporte ; Tailwind 3 → 4 en dédié (réécriture `app.css`/config, S–M). |

---

## 9. Après ce plan : roadmap TODO.md §9

Une fois P0–P8 terminées, les briques manquantes du TODO deviennent des incréments sur des fondations saines. Ordre suggéré, en s'appuyant sur ce qui aura été posé :

1. **Tâches planifiées** (généralisation du cron minimal de P4.2) → rsync backup (§9.6), USB backup (§9.7), tests SMART planifiés (existent déjà en base).
2. **Notifications e-mail** (P3 fournit les événements ; ajouter un relais SMTP dans `settings`).
3. **Historique métriques** (P1.3 centralise `System` ; persister 1 point/min en SQLite, purge glissante) → graphes 1h/1j/1s/1m dans les widgets P7.1.
4. **Firewall nftables**, **Let's Encrypt** (nécessite un nom public), **quotas** (btrfs qgroups), **UPS/NUT**, **SNMP**, **syslog distant**, **cloud sync rclone** — chacun = package LibreELEC + service + onglet Control Panel, sur le modèle de P5.

---

## Récapitulatif

| Phase | Contenu | Taille | Dépend de |
|---|---|---|---|
| **P0** | Auth globale, admin, sessions, intégrité update, Samba, validation Pi, rate-limit | ≈ 3 j | — |
| **P1** | `AppError`, async propre, `System` partagé, schéma, warnings, tests, config, Docker réseaux | ≈ 5 j | P0 |
| **P2** | Client API unique, toasts, i18n, mocks, suppressions, svelte-check, tests | ≈ 4 j | P1 |
| **P3** | Notifications réelles, Dashboard réel | ≈ 2 j | P1, P2 |
| **P4** | Time & Language, Hardware & Power, Security, Device Connection, password policy, About | ≈ 6 j | P1, P2 |
| **P5** | NFS (image + backend + UI), SFTP ; FTP à ne pas faire | ≈ 8 j | P1 |
| **P6** | Download, aperçu, tri, drag & drop, renommer | ≈ 3 j | P1, P2 |
| **P7** | Widgets, recherche globale, préférences, Docker UI | ≈ 3 j | P2, P3 |
| **P8** | CI, release, validation Pi, catalogue, docs, dépendances | continu | — |
| **Total** | | **≈ 34 jours** (≈ 7 semaines à temps plein, P3–P7 parallélisables) | |

Livrable minimal "sûr à exposer sur un LAN" : **P0 seule (3 jours)**. Livrable "produit cohérent, sans placeholder" : **P0 → P4 + P6 (≈ 23 jours)**.
