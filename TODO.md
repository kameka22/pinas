# PiNAS - TODO List Complète

> **Projet** : NAS OS moderne pour Raspberry Pi 5
> **Stack** : LibreELEC + Rust + SvelteKit
> **Interface** : Desktop-like (inspiré UGOS / Synology DSM)
> **Distribution** : Addon LibreELEC (cohabitation avec Kodi)

---
## Historique (phases 0 à 8) — archivé

Les phases 0 à 8 (setup, backend, frontend, desktop, apps, onboarding, package LibreELEC, auth, catalogue) sont livrées. Le détail des reprises effectuées ensuite (sécurité, fondations, Control Panel, NFS, fichiers, desktop, ops) est tracé phase par phase dans **[REMEDIATION_PLAN.md](REMEDIATION_PLAN.md)** ; la validation matérielle de chaque release suit **[docs/RELEASE_CHECKLIST.md](docs/RELEASE_CHECKLIST.md)**.

Restes connus hors phase 9 : historique des métriques (widgets), thème sombre, redimensionnement de volume, `CreateVolumeRequest.size` ignoré, mises à jour temps réel du Storage Manager via WebSocket.

---

## Phase 9 : Features inspirées OpenMediaVault

> Référence : analyse du code source OMV (`extra/openmediavault/`)
> Objectif : atteindre la parité fonctionnelle avec OMV

### 9.1 Notifications & Alertes
> Notifications in-app livrées (P3 : table `notifications`, centre de notifications, WebSocket). Reste l'e-mail.
- [ ] Service email notifications (Postfix relay SMTP)
  - [ ] `GET /api/notifications/settings` - Config SMTP
  - [ ] `PUT /api/notifications/settings` - Modifier config
  - [ ] `POST /api/notifications/test` - Envoyer email test
  - [ ] Alertes : S.M.A.R.T., espace disque, mises à jour, UPS
- [ ] Frontend : NotificationSettings.svelte dans Control Panel

### 9.2 Scheduled Tasks (Cron)
- [ ] Service cron jobs
  - [ ] `GET /api/cron/jobs` - Liste jobs planifiés
  - [ ] `POST /api/cron/jobs` - Créer job
  - [ ] `PUT /api/cron/jobs/:id` - Modifier job
  - [ ] `DELETE /api/cron/jobs/:id` - Supprimer job
  - [ ] `POST /api/cron/jobs/:id/run` - Exécuter maintenant
  - [ ] Types : commande shell, reboot, shutdown, script custom
  - [ ] Planification : cron expression ou presets (quotidien, hebdo, mensuel)
- [ ] Frontend : ScheduledTasks.svelte dans Control Panel

### 9.3 Power Management ✅ (partiel)
> Livré en P4 : gouverneur CPU, reboot/shutdown planifiés (Control Panel › Hardware & Power). Reste Wake-on-LAN.
- [ ] Service power management
  - [x] `POST /api/system/reboot` - Redémarrer
  - [x] `POST /api/system/shutdown` - Éteindre
  - [ ] `GET /api/power/settings` - Config power
  - [ ] `PUT /api/power/settings` - Modifier config
  - [ ] CPU frequency scaling (performance, powersave, ondemand)
  - [ ] Wake-on-LAN (WoL) enable/disable
  - [ ] Shutdown/reboot planifié (via cron)
- [ ] Frontend : PowerSettings.svelte dans Control Panel

### 9.4 NFS Shares ✅
> Livré en P5 : `pinas-nfs-server` (rpc.nfsd/mountd/exportfs de LibreELEC), `/api/shares/nfs/*`, exports par partage avec clients/ro-rw/squash, onglet NFS de File Service.
- [ ] Service NFS
  - [ ] `GET /api/nfs/status` - État service NFS
  - [ ] `POST /api/nfs/enable` - Activer NFS
  - [ ] `POST /api/nfs/disable` - Désactiver NFS
  - [ ] `GET /api/nfs/shares` - Liste exports NFS
  - [ ] `POST /api/nfs/shares` - Créer export
  - [ ] `PUT /api/nfs/shares/:id` - Modifier export
  - [ ] `DELETE /api/nfs/shares/:id` - Supprimer export
  - [ ] Options : client restrictions, read/write, root squash
  - [ ] Génération `/etc/exports` depuis DB
- [ ] Frontend : compléter onglet NFS dans FileService.svelte

### 9.5 FTP/SFTP Server ✗ (abandonné)
> LibreELEC n'a qu'un compte Unix (root) : ni chroot SFTP par utilisateur ni FTP multi-utilisateur possibles sans démon dédié. SFTP reste disponible pour root via SSH.
- [ ] Service FTP (ProFTPD ou vsftpd)
  - [ ] `GET /api/ftp/status` - État service FTP
  - [ ] `POST /api/ftp/enable` - Activer FTP
  - [ ] `POST /api/ftp/disable` - Désactiver FTP
  - [ ] `GET /api/ftp/settings` - Config FTP
  - [ ] `PUT /api/ftp/settings` - Modifier config
  - [ ] `GET /api/ftp/shares` - Liste répertoires FTP
  - [ ] `POST /api/ftp/shares` - Ajouter répertoire
  - [ ] Options : TLS/SSL, anonymous access, bandwidth limit, user restrictions
- [ ] Frontend : compléter onglet FTP dans FileService.svelte

### 9.6 Rsync Backup
- [ ] Service rsync
  - [ ] `GET /api/rsync/jobs` - Liste jobs rsync
  - [ ] `POST /api/rsync/jobs` - Créer job (push/pull)
  - [ ] `PUT /api/rsync/jobs/:id` - Modifier job
  - [ ] `DELETE /api/rsync/jobs/:id` - Supprimer job
  - [ ] `POST /api/rsync/jobs/:id/run` - Exécuter maintenant
  - [ ] Rsync server (rsyncd) pour backup distant
  - [ ] Planification via cron
  - [ ] Options : compression, delete, exclude patterns, SSH transport
- [ ] Frontend : BackupSettings.svelte dans Control Panel

### 9.7 USB Backup
- [ ] Service USB backup
  - [ ] `GET /api/usb-backup/jobs` - Liste jobs USB
  - [ ] `POST /api/usb-backup/jobs` - Créer job
  - [ ] Détection auto branchement USB (udev rules)
  - [ ] Sync automatique ou manuelle
  - [ ] Direction : USB→NAS ou NAS→USB
- [ ] Frontend : USBBackup.svelte dans Control Panel

### 9.8 Firewall
- [ ] Service firewall (iptables/nftables)
  - [ ] `GET /api/firewall/rules` - Liste règles
  - [ ] `POST /api/firewall/rules` - Ajouter règle
  - [ ] `PUT /api/firewall/rules/:id` - Modifier règle
  - [ ] `DELETE /api/firewall/rules/:id` - Supprimer règle
  - [ ] Support IPv4 et IPv6
  - [ ] Port forwarding / NAT
  - [ ] Presets (allow SSH, block all, etc.)
- [ ] Frontend : FirewallSettings.svelte dans Control Panel

### 9.9 SSL/TLS Certificates
- [ ] Service certificats
  - [ ] `GET /api/certificates` - Liste certificats
  - [ ] `POST /api/certificates` - Créer/importer certificat
  - [ ] `DELETE /api/certificates/:id` - Supprimer certificat
  - [ ] `POST /api/certificates/csr` - Générer CSR
  - [ ] Let's Encrypt (ACME) auto-renouvellement
  - [ ] Appliquer à : web UI (HTTPS), FTP (FTPS), etc.
- [ ] Frontend : CertificateSettings.svelte dans Control Panel

### 9.10 Disk Quotas
- [ ] Service quotas
  - [ ] `GET /api/quotas` - Liste quotas
  - [ ] `POST /api/quotas` - Définir quota utilisateur/groupe
  - [ ] `PUT /api/quotas/:id` - Modifier quota
  - [ ] `DELETE /api/quotas/:id` - Supprimer quota
  - [ ] Quota par volume (ext4/XFS)
  - [ ] Soft limit + hard limit
- [ ] Frontend : QuotaSettings.svelte dans Control Panel

### 9.11 RRD Graphs (Historique métriques)
- [ ] Service collecte métriques
  - [ ] `GET /api/stats/cpu` - Historique CPU (1h, 1j, 1sem, 1mois)
  - [ ] `GET /api/stats/memory` - Historique RAM
  - [ ] `GET /api/stats/disk` - Historique I/O disque
  - [ ] `GET /api/stats/network` - Historique réseau
  - [ ] Stockage : SQLite ou RRD (round-robin database)
  - [ ] Collecte périodique (toutes les 30s)
- [ ] Frontend : StatsApp.svelte (graphiques interactifs)

### 9.12 UPS Support (Network UPS Tools)
- [ ] Service NUT
  - [ ] `GET /api/ups/status` - État UPS (charge, runtime, input voltage)
  - [ ] `GET /api/ups/settings` - Config NUT
  - [ ] `PUT /api/ups/settings` - Modifier config
  - [ ] Détection UPS USB
  - [ ] Shutdown automatique sur batterie faible
  - [ ] Notifications batterie
- [ ] Frontend : UPSSettings.svelte dans Control Panel

### 9.13 SNMP Agent
- [ ] Service SNMP
  - [ ] `GET /api/snmp/settings` - Config SNMP
  - [ ] `PUT /api/snmp/settings` - Modifier config
  - [ ] Community strings, SNMPv3, trap destinations
- [ ] Frontend : SNMPSettings.svelte dans Control Panel

### 9.14 Syslog distant
- [ ] Service rsyslog
  - [ ] `GET /api/syslog/settings` - Config syslog
  - [ ] `PUT /api/syslog/settings` - Modifier config
  - [ ] Envoi vers serveur syslog distant (UDP/TCP)
- [ ] Frontend : SyslogSettings.svelte dans Control Panel

### 9.15 Cloud Sync
- [ ] Service cloud sync (rclone)
  - [ ] `GET /api/cloud/providers` - Providers configurés
  - [ ] `POST /api/cloud/providers` - Ajouter provider (OneDrive, Google Drive, S3, etc.)
  - [ ] `GET /api/cloud/jobs` - Jobs de sync
  - [ ] `POST /api/cloud/jobs` - Créer job
  - [ ] `POST /api/cloud/jobs/:id/run` - Sync maintenant
  - [ ] Direction : upload, download, bidirectional
  - [ ] Planification via cron
- [ ] Frontend : CloudSync.svelte dans Control Panel

### 9.16 LVM (Logical Volume Management)
- [ ] Service LVM
  - [ ] Physical Volumes (PV) : create, delete
  - [ ] Volume Groups (VG) : create, extend, reduce, delete
  - [ ] Logical Volumes (LV) : create, resize, delete, snapshot
  - [ ] Intégration avec Storage Manager existant
- [ ] Frontend : section LVM dans Storage Manager

### 9.17 Bcache SSD Caching
- [ ] Service bcache
  - [ ] Créer backing device (HDD) + cache device (SSD)
  - [ ] Modes : writethrough, writeback, writearound
  - [ ] Stats cache (hit rate, dirty data)
- [ ] Frontend : section Bcache dans Storage Manager

### 9.18 S3/MinIO (Stockage objet)
- [ ] Service MinIO
  - [ ] `GET /api/s3/status` - État MinIO
  - [ ] `POST /api/s3/enable` - Activer MinIO
  - [ ] Config buckets, access keys
  - [ ] Compatible API S3
- [ ] Frontend : S3Settings.svelte dans Control Panel

---

*Dernière mise à jour : 8 septembre 2026*
*Cible OS : LibreELEC 12.x (package intégré à l'image)*
