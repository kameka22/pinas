# Check-list de release PiNAS (validation sur Raspberry Pi 5)

À exécuter **à chaque release** sur un Pi 5 réel (les opérations système sont simulées en `PINAS_DEV_MODE`, la CI ne couvre que la compilation et les tests unitaires). Copier le tableau dans la PR/release et cocher chaque ligne ; une ligne ✗ bloque la publication.

## 0. Préparation

- [ ] `VERSION` incrémenté, `scripts/sync-version.sh --check` vert (CI `Version consistency`).
- [ ] `ci.yml` vert sur `master` (cargo test / `-D warnings`, svelte-check 0 erreur, Vitest, build).
- [ ] Image construite depuis le commit taggé (`build-libreelec.yml` ou `scripts/remote-build.sh`) ; archive de mise à jour + `.sha256` (`scripts/build-release.sh --tag`).
- [ ] Un Pi « neuf » (image flashée) **et** un Pi en version N-1 (pour la mise à jour) ; 2 clés USB pour le RAID ; un client SMB/NFS sur le LAN.

## 1. Boot et services (image neuve)

| # | Étape | Attendu | OK |
|---|---|---|---|
| 1.1 | Boot, Kodi démarre | Écran Kodi, `ssh root@<ip>` possible après activation | ☐ |
| 1.2 | `systemctl status pinas` | `active (running)`, pas de redémarrage en boucle | ☐ |
| 1.3 | `journalctl -u pinas -b` | Aucun `ERROR` ; migrations appliquées ; JWT secret généré dans `/storage/.pinas/data/.jwt_secret` | ☐ |
| 1.4 | `ProtectSystem=strict` + `ReadWritePaths` | Le service écrit bien `/storage/.pinas`, `/etc/samba`, `/var/lib/nfs` (squashfs LibreELEC) | ☐ |
| 1.5 | `systemctl status pinas-rpcbind pinas-nfs-server` | Inactifs tant que NFS n'est pas activé dans l'UI ; pas d'échec | ☐ |
| 1.6 | `curl -k https://<ip>:3000/api/health` si `PINAS_TLS_ENABLED`/réglage Sécurité | 200 ; certificat auto-signé dans `/storage/.pinas/data/.tls/` | ☐ |

## 2. Onboarding et authentification

| # | Étape | Attendu | OK |
|---|---|---|---|
| 2.1 | Onboarding 7 étapes | Admin créé, langue/hostname appliqués (`hostnamectl`), redirection vers le bureau | ☐ |
| 2.2 | Politique de mot de passe | Un mot de passe faible est refusé à l'onboarding **et** à la création d'utilisateur | ☐ |
| 2.3 | Login / logout | Après logout, `GET /api/auth/me` avec l'ancien cookie → 401 `SESSION_REVOKED` | ☐ |
| 2.4 | Rate-limit | 6 mauvais mots de passe en 1 min → 429 (par utilisateur et par IP) | ☐ |
| 2.5 | Non-admin | Utilisateur standard : `/api/storage/disks`, `/api/docker/status`, `/api/terminal/exec` → 403 ; apps admin absentes du lanceur | ☐ |
| 2.6 | Reset par l'admin | Changer le mot de passe d'un utilisateur connecté → sa session est révoquée | ☐ |
| 2.7 | Journal des connexions | Control Panel › Sécurité liste les tentatives (succès/échec, IP) | ☐ |

## 3. Utilisateurs, partages, accès réseau

| # | Étape | Attendu | OK |
|---|---|---|---|
| 3.1 | Créer un utilisateur | Home créé sous `/storage/.pinas/homes/<user>` ; `pdbedit -L` le liste (smbpasswd) | ☐ |
| 3.2 | Partage SMB | Création → visible depuis un client (`smbclient -L //<ip> -U <user>`), lecture/écriture selon permissions, invité refusé | ☐ |
| 3.3 | Activer NFS | `pinas-rpcbind` + `pinas-nfs-server` actifs ; `exportfs -v` liste l'export ; `showmount -e <ip>` depuis un client | ☐ |
| 3.4 | Export NFS | `mount -t nfs <ip>:/storage/… /mnt` depuis un client ; `ro`/`rw`, `root_squash` respectés | ☐ |
| 3.5 | Redémarrage | Après reboot, exports NFS réappliqués (`ExecStartPost` → `/api/internal/nfs/reexport`) | ☐ |
| 3.6 | Suppression utilisateur | Home archivé (`PINAS_HOME_ON_DELETE=archive`), compte Samba retiré | ☐ |

## 4. Stockage

| # | Étape | Attendu | OK |
|---|---|---|---|
| 4.1 | Disques | Les 2 clés USB apparaissent avec modèle/taille ; la carte SD/NVMe système est marquée protégée (wipe → 403) | ☐ |
| 4.2 | Pool RAID1 | Création sur les 2 clés (`cat /proc/mdstat`), statut *normal* | ☐ |
| 4.3 | Volume | Création ext4/btrfs, monté sous `/storage/pools/<pool>/<vol>`, visible dans Files et comme cible de partage | ☐ |
| 4.4 | Dégradation | Débrancher une clé → pool *degraded* + notification ; rebrancher → scrub/resync | ☐ |
| 4.5 | S.M.A.R.T. | Test court sur un disque, historique enregistré | ☐ |
| 4.6 | Reboot | Pool et volume remontés automatiquement | ☐ |

## 5. Fichiers

| # | Étape | Attendu | OK |
|---|---|---|---|
| 5.1 | Upload / download | Fichier de 500 Mo dans les deux sens, checksum identique | ☐ |
| 5.2 | Dossier → zip | Téléchargement d'un dossier produit une archive lisible | ☐ |
| 5.3 | Aperçu | Image, texte, PDF, vidéo (lecture avec seek → `Range`) | ☐ |
| 5.4 | Drag & drop, renommer, supprimer, propriétés | Opérations reflétées sur le disque | ☐ |
| 5.5 | Recherche `Ctrl+K` | Un fichier du home ou d'un partage est trouvé ; les fichiers d'un partage non autorisé n'apparaissent pas | ☐ |

## 6. Apps, Docker, système

| # | Étape | Attendu | OK |
|---|---|---|---|
| 6.1 | App Center › Docker | Installation, `docker info` OK, reconnexion du backend sans redémarrage | ☐ |
| 6.2 | Installer Jellyfin (mono-conteneur) puis Nextcloud (compose) | Progression en temps réel, apps accessibles sur leurs ports, redémarrage → `unless-stopped` | ☐ |
| 6.3 | Désinstaller | Conteneurs/volumes/fichiers suivis supprimés | ☐ |
| 6.4 | Docker app | Logs en mode suivi, stats, compteur de conteneurs par réseau, prune images/volumes | ☐ |
| 6.5 | Terminal & Process Manager | Commande exécutée, processus listés, kill d'un processus test | ☐ |
| 6.6 | Hardware & Power | Changement de gouverneur visible dans `/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor` ; reboot planifié dans 2 min s'exécute | ☐ |
| 6.7 | Time & Language | `timedatectl` reflète fuseau/NTP ; langue UI persistée | ☐ |
| 6.8 | Display | Bascule Kodi ⇄ PiNAS sur HDMI, Kodi redémarre proprement | ☐ |
| 6.9 | Imprimante | CUPS activé, imprimante USB détectée, page de test | ☐ |
| 6.10 | Widgets, notifications, fond d'écran | Panneau Widgets, centre de notifications (pool dégradé, update dispo), fond d'écran persistant après reconnexion | ☐ |

## 7. Mise à jour depuis N-1 (second Pi)

| # | Étape | Attendu | OK |
|---|---|---|---|
| 7.1 | Vérification | Control Panel › Mise à jour voit la nouvelle version et le changelog | ☐ |
| 7.2 | Intégrité | Modifier un octet de l'archive sur un miroir de test → installation refusée (`sha256` invalide) | ☐ |
| 7.3 | Installation | Écran plein, redémarrage du service, `GET /api/system/info` renvoie la nouvelle version, modal « mis à jour » | ☐ |
| 7.4 | Données | Utilisateurs, partages, pools, apps, préférences intacts ; migrations SQL appliquées sans erreur | ☐ |
| 7.5 | Cohabitation Kodi | Kodi toujours fonctionnel après la mise à jour | ☐ |

## 8. Publication

- [ ] Résultats consignés ci-dessous (date, version, image, testeur) ; toute ligne ✗ a un ticket.
- [ ] Draft GitHub Release `vX.Y.Z` complété : image `.img.gz`, `pinas-update-vX.Y.Z.tar.gz` + `.sha256`, notes générées par `build-release.sh`.
- [ ] Tag `vX.Y.Z` poussé (`git push origin vX.Y.Z`), release publiée.

## Historique des validations

| Date | Version | Image / commit | Testeur | Résultat |
|---|---|---|---|---|
| | | | | |
