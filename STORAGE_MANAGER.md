# Plan : Parité Stockage OMV — Pools & Volumes

## Contexte

PiNAS dispose d'un Storage Manager fonctionnel (pools, volumes, disques, SMART) mais il manque plusieurs fonctionnalités que OpenMediaVault propose pour un NAS fiable. L'objectif est de combler les gaps les plus impactants pour un utilisateur home NAS, en restant réaliste pour un Raspberry Pi 5 sous LibreELEC.

## Comparatif rapide PiNAS vs OMV

| Feature | PiNAS | OMV | Gap |
|---------|-------|-----|-----|
| RAID 0/1/5/10, Btrfs | OK | OK | - |
| RAID 6 | - | OK | Manquant |
| RAID Health Monitor (/proc/mdstat) | - | OK | **Critique** |
| RAID Grow/Expand (ajouter disques) | - | OK | Manquant |
| Pool Scrub/Check | 501 stub | OK | **À implémenter** |
| Volume Resize | - | OK | Manquant |
| Mount Options (noatime, discard) | Champ DB existe, inutilisé | OK | **Facile** |
| Filesystem Check (fsck) | - | OK | Manquant |
| SMART Scheduled Tests | - | OK | Manquant |
| Disk Power Management (APM, spindown) | - | OK | Manquant |
| Secure Wipe (shred) | Quick seulement | Quick + Secure | Partiel |
| Btrfs Snapshots | - | OK | Manquant |
| Storage Notifications (WS alerts) | - | OK (email) | Manquant |
| Quota Management | - | OK (via mount opts) | LOW - skip |

## Features à implémenter (priorisées)

| # | Feature | Priorité | Effort | Migration DB ? | Statut |
|---|---------|----------|--------|----------------|--------|
| 1 | **RAID Health Monitoring** — /proc/mdstat + btrfs device stats | HIGH | Moyen | Non (metadata JSON) | ✅ DONE |
| 2 | **Pool Scrub/Check** — btrfs scrub + mdadm --check | HIGH | Moyen | Non (metadata JSON) | ✅ DONE |
| 3 | **Volume Resize** — grow filesystem (ext4/btrfs/xfs) | HIGH | Moyen | Non | ✅ DONE |
| 4 | **Mount Options** — noatime, discard, ACL, presets | MEDIUM | Faible | Non (champ existe) | ✅ DONE |
| 5 | **Filesystem Check (fsck)** — e2fsck, btrfs check, xfs_repair | MEDIUM | Faible | Non | ✅ DONE |
| 6 | **Secure Wipe** — mode quick/zeros/secure | MEDIUM | Faible | Non | ✅ DONE |
| 7 | **SMART Scheduled Tests** — short/long tests planifiés | MEDIUM | Moyen | Oui (nouvelle table) | ✅ DONE |
| 8 | **RAID Grow/Expand** — ajouter disques à un array existant | MEDIUM | Élevé | Non (devices JSON) | ✅ DONE |
| 9 | **Disk Power Management** — APM, spindown, write cache | MEDIUM | Moyen | Oui (nouvelle table) | ✅ DONE |
| 10 | **Btrfs Snapshots** — create/delete/list snapshots | MEDIUM | Élevé | Oui (nouvelle table) | ✅ DONE |
| 11 | **RAID 6** — double parité mdadm + btrfs-raid5/6 | LOW | Faible | Non (validation Rust) | ✅ DONE |
| 12 | **Storage Notifications** — alertes WS temps réel | HIGH | Faible | Non (WS existant) | ✅ DONE |

**Hors scope** : Quota management (complexité élevée, peu utilisé en home NAS).

---

## Phase 1 : Fiabilité (features 1, 2, 12)

### 1.1 RAID Health Monitoring + Storage Notifications

**Fichiers backend :**

`backend/src/models/storage.rs` — Ajouter :
```rust
pub struct PoolHealthInfo {
    pub pool_id: String,
    pub is_healthy: bool,
    pub degraded_devices: Vec<String>,
    pub missing_devices: Vec<String>,
    pub rebuild_progress: Option<u8>,  // 0-100%
    pub device_errors: Vec<DeviceErrorInfo>,
    pub last_checked: String,
}
pub struct DeviceErrorInfo {
    pub device: String,
    pub read_errors: u64,
    pub write_errors: u64,
    pub corruption_errors: u64,
}
```

`backend/src/services/storage.rs` — Ajouter :
- `get_pool_health(pool_id)` : pour mdadm → parser `/proc/mdstat` ; pour btrfs → `btrfs device stats <mount>`
- `start_health_monitor(db, storage_tx)` : boucle `tokio::spawn` toutes les 60s, check tous les pools, si transition Normal→Degraded → update DB + broadcast WS

`backend/src/api/storage.rs` — Ajouter :
- `GET /pools/:id/health` → retourne `PoolHealthInfo`

`backend/src/api/ws.rs` — Ajouter variant :
```rust
#[serde(rename = "storage.alert")]
StorageAlert(StorageAlertEvent),  // { pool_id, severity, message }
```

`backend/src/main.rs` — Ajouter :
- Nouveau broadcast channel `storage_tx` dans `AppState`
- Appeler `start_health_monitor()` après initialisation DB
- Passer `storage_tx` au WS handler (dans le `select!` loop)

**Frontend :**

`StorageManager.svelte` :
- Pool cards : ajouter indicateur santé (vert/jaune/rouge) à côté du status badge
- Si `Rebuilding` : progress bar avec % dans la carte pool
- Nouveau bouton "Health" dans context menu → modal détails par device

`frontend/src/lib/stores/websocket.ts` :
- Handler pour `storage.alert` → toast notification

`frontend/src/lib/stores/api.ts` :
- Type `PoolHealthInfo`, méthode `getPoolHealth(id)`

**Dev mode** : retourner des fausses données santé (tout healthy, 0 erreurs).

---

### 1.2 Pool Scrub/Check (remplacer le 501)

**Backend :**

`backend/src/services/storage.rs` — Ajouter :
- `scrub_pool(pool_id) -> Result<String>` (retourne task_id) :
  - btrfs : `btrfs scrub start <mount>`, monitoring via `btrfs scrub status`
  - mdadm : write `check` to `/sys/block/mdX/md/sync_action`, progress via `sync_completed`
- Pattern `_start` / `_execute` avec `task_tx` broadcast pour progress WS
- Stocker dans metadata JSON : `last_scrub_date`, `last_scrub_result`

`backend/src/api/storage.rs` — Remplacer le handler 501 par l'implémentation réelle.

**Frontend :**

`StorageManager.svelte` :
- Le bouton scrub existe déjà dans le context menu + `scrubbingPools` set
- Ajouter progress bar pendant le scrub (réutiliser les events `task.progress`)
- Afficher date du dernier scrub dans les détails du pool

**Dev mode** : simuler progression scrub sur 10s via `task.progress` WS events.

---

## Phase 2 : Usabilité (features 3, 4, 5, 6)

### 2.1 Volume Resize

**Backend :**

`backend/src/models/storage.rs` — Ajouter :
```rust
pub struct ResizeVolumeRequest { pub size: Option<u64> }  // None = max available
```

`backend/src/services/storage.rs` — Ajouter :
- `resize_volume(volume_id, new_size)` :
  - ext4 : `resize2fs <device> [size]` (grow online, shrink = unmount requis)
  - btrfs : `btrfs filesystem resize <size> <mount>` (grow + shrink online)
  - xfs : `xfs_growfs <mount>` (grow seulement, online)
  - f2fs : `resize.f2fs <device>` (offline seulement)
- Validation : pas shrink sous used, pas grow au-delà du pool

`backend/src/api/storage.rs` — Ajouter :
- `POST /volumes/:id/resize` → body `ResizeVolumeRequest`

**Frontend :**

`StorageManager.svelte` :
- "Resize" dans le context menu volume → modal avec :
  - Taille actuelle, espace utilisé, espace pool disponible
  - Input taille ou checkbox "Use all available space"
  - Warning si shrink

### 2.2 Mount Options

Le champ `mount_options` existe déjà dans `storage_volumes` mais n'est pas utilisé.

**Backend :**

`backend/src/models/storage.rs` — Modifier `CreateVolumeRequest` :
```rust
pub struct CreateVolumeRequest {
    pub name: String,
    pub fs_type: String,
    pub size: Option<u64>,
    pub mount_options: Option<String>,  // NOUVEAU
}
```
Ajouter `UpdateVolumeRequest { pub mount_options: Option<String> }`

`backend/src/services/storage.rs` :
- `create_volume()` : passer mount_options au `mount()` et sauver en DB
- `mount_volume()` : lire mount_options depuis DB et passer à `mount -o`
- Presets : `Default` = `defaults`, `SSD` = `noatime,discard`, `NAS` = `noatime,nofail`

`backend/src/api/storage.rs` — Ajouter :
- `PUT /volumes/:id` → body `UpdateVolumeRequest` (modifier mount options, nécessite remount)

**Frontend :**

`StorageManager.svelte` :
- Create Volume modal : ajouter section "Mount Options" avec presets (boutons radio) + input custom
- Volume context menu : "Edit" pour modifier mount options

### 2.3 Filesystem Check (fsck)

**Backend :**

`backend/src/services/storage.rs` — Ajouter :
- `check_filesystem(volume_id, repair: bool)` :
  - Volume doit être unmounted (erreur sinon)
  - ext4 : `e2fsck -f [-y] <device>`
  - btrfs : `btrfs check [--repair] <device>` (offline)
  - xfs : `xfs_repair [-n] <device>`
- Long-running → pattern `_start`/`_execute` avec progress WS

`backend/src/api/storage.rs` — Ajouter :
- `POST /volumes/:id/check` → body `{ repair: bool }`

**Frontend :**

`StorageManager.svelte` :
- "Check Filesystem" dans context menu volume
- Warning "volume must be unmounted"
- Modal résultat avec output fsck

### 2.4 Secure Wipe

**Backend :**

`backend/src/models/storage.rs` — Ajouter :
```rust
pub enum WipeMode { Quick, Zeros, Secure }
pub struct WipeDiskRequest { pub mode: WipeMode }
```

`backend/src/services/storage.rs` :
- Modifier `wipe_disk()` pour accepter un mode :
  - `Quick` : `sgdisk --zap-all` (existant)
  - `Zeros` : `dd if=/dev/zero of=<device> bs=1M status=progress`
  - `Secure` : `shred -vfz -n 3 <device>`
- Long-running pour Zeros/Secure → pattern `_start`/`_execute`

`backend/src/api/storage.rs` :
- Modifier `POST /disks/:name/wipe` pour accepter body optionnel `{ mode }`, default "quick"

**Frontend :**

`StorageManager.svelte` :
- Modifier le modal Wipe pour ajouter sélection du mode (radio buttons)
- Progress bar pour Zeros/Secure (longues opérations)

---

## Phase 3 : Avancé (features 7, 8, 9, 10, 11)

### 3.1 SMART Scheduled Tests

**Migration DB** — `backend/migrations/011_storage_features.sql` :
```sql
CREATE TABLE IF NOT EXISTS smart_test_schedules (
    id TEXT PRIMARY KEY NOT NULL,
    device_path TEXT NOT NULL,
    device_name TEXT NOT NULL,
    test_type TEXT NOT NULL CHECK(test_type IN ('short', 'long', 'conveyance', 'offline')),
    interval_hours INTEGER NOT NULL DEFAULT 168,
    last_run TEXT, next_run TEXT NOT NULL,
    last_result TEXT, enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS disk_power_settings (
    device_path TEXT PRIMARY KEY NOT NULL,
    device_name TEXT NOT NULL,
    apm_level INTEGER, spindown_minutes INTEGER,
    write_cache INTEGER, updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS btrfs_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    volume_id TEXT NOT NULL REFERENCES storage_volumes(id) ON DELETE CASCADE,
    name TEXT NOT NULL, path TEXT NOT NULL,
    snapshot_type TEXT NOT NULL DEFAULT 'manual' CHECK(snapshot_type IN ('manual', 'scheduled')),
    created_at TEXT NOT NULL
);
```

**Backend** : CRUD schedules + `start_smart_scheduler()` boucle tokio (check toutes les 60s si un test est dû). `smartctl -t short|long <device>` pour lancer, `smartctl -l selftest -j` pour résultats.

**API** : `POST /disks/:name/smart/test`, `GET /disks/:name/smart/tests`, `GET/POST/DELETE /smart/schedules`

**Frontend** : Enrichir le modal SMART avec onglet "Self-Tests" (historique + bouton Run) et section Schedules dans l'onglet "Advanced Settings" (actuellement placeholder).

### 3.2 RAID Grow/Expand

**Backend** : `grow_pool(pool_id, new_devices, wipe_devices)` :
- mdadm RAID1 : `mdadm --add /dev/mdX /dev/sdY`
- mdadm RAID5/6 : `mdadm --grow --raid-devices=N --add`
- btrfs : `btrfs device add` + `btrfs balance start`
- Update `devices` JSON et `total_size` en DB

**API** : `POST /pools/:id/grow` → body `{ devices: [], wipe_devices: bool }`

**Frontend** : "Add Disk" dans context menu pool → modal sélection disques (comme Create Pool step 1)

### 3.3 Disk Power Management

**Backend** : `get/set_disk_power_settings(device_path)` via `hdparm -B/-S/-W`. Re-appliquer au boot depuis DB (`apply_power_settings_on_boot()` dans main.rs).

**API** : `GET/PUT /disks/:name/power`

**Frontend** : "Power Settings" dans le disk card → modal APM slider, spindown dropdown, write cache toggle. Masqué pour SSD/NVMe.

### 3.4 Btrfs Snapshots

**Backend** : `list/create/delete_snapshot(volume_id)` via `btrfs subvolume snapshot/delete`. Stockage dans table `btrfs_snapshots`.

**API** : `GET/POST /volumes/:id/snapshots`, `DELETE /volumes/:id/snapshots/:snap_id`

**Frontend** : "Snapshots" dans context menu volume (btrfs seulement) → modal liste + Create/Delete.

### 3.5 RAID 6

Trivial : ajouter `Raid6` à l'enum `RaidType`, handler dans `create_mdadm_pool()` level 6, min 4 disks. Ajouter dans le frontend `raidTypes` array + i18n.

---

## Fichiers impactés (résumé)

| Fichier | Phases |
|---------|--------|
| `backend/src/models/storage.rs` | 1, 2, 3 — nouveaux structs/enums |
| `backend/src/services/storage.rs` | 1, 2, 3 — nouvelles méthodes service |
| `backend/src/api/storage.rs` | 1, 2, 3 — ~15 nouveaux endpoints |
| `backend/src/api/ws.rs` | 1 — `StorageAlert` event variant |
| `backend/src/main.rs` | 1, 3 — `storage_tx` channel, health monitor, smart scheduler |
| `backend/migrations/011_storage_features.sql` | 3 — 3 nouvelles tables |
| `frontend/.../StorageManager.svelte` | 1, 2, 3 — modals, indicateurs, context menu |
| `frontend/.../stores/api.ts` | 1, 2, 3 — types + méthodes API |
| `frontend/.../stores/websocket.ts` | 1 — handler `storage.alert` |
| `frontend/.../i18n/en.ts` + `fr.ts` | 1, 2, 3 — clés traduction |

## Architecture existante réutilisée

- **Pattern `_start`/`_execute`** (de `PackageService`) pour scrub, fsck, secure wipe
- **`task_tx` broadcast** (existant) pour progress WS des tâches longues
- **`WsEvent` enum** (ws.rs:28) pour ajouter `StorageAlert` variant
- **`AppState`** (main.rs:32) pour le nouveau `storage_tx` channel
- **`handle_socket` select! loop** (ws.rs:122) pour écouter le nouveau channel
- **`metadata` JSON** dans `storage_pools` pour stocker health/scrub data sans migration

## Vérification

1. **Phase 1** : Créer un pool mdadm RAID1 → vérifier que le health monitor détecte l'état normal → simuler degraded (retirer un disque en dev mode) → vérifier l'alerte WS
2. **Phase 1** : Lancer un scrub → vérifier les events `task.progress` → vérifier `last_scrub_date` dans metadata
3. **Phase 2** : Créer un volume ext4 → resize → vérifier la nouvelle taille
4. **Phase 2** : Créer un volume avec mount options `noatime,discard` → vérifier `mount` output
5. **Phase 2** : Unmount volume → fsck → vérifier résultat
6. **Phase 2** : Wipe disk mode secure → vérifier progression WS
7. **Phase 3** : Planifier un test SMART short → vérifier exécution automatique
8. **Phase 3** : Pool RAID5 → grow avec un disque → vérifier rebuild + nouveau total_size
9. **Phase 3** : Volume btrfs → créer snapshot → lister → supprimer
