# PiNAS MVP - File Manager

## Statut : Fonctionnel

Ce document décrit le MVP (Minimum Viable Product) implémenté pour valider le concept PiNAS : un File Manager connecté au backend.

---

## Fonctionnalités implémentées

| Feature | Backend | Frontend | Status |
|---------|---------|----------|--------|
| Lister fichiers | `GET /api/files?path=` | FileManager.svelte | OK |
| Créer dossier | `POST /api/files/folder` | Bouton + modal | OK |
| Supprimer fichier/dossier | `DELETE /api/files?path=` | Menu contextuel | OK |
| Renommer | `PATCH /api/files/rename` | Modal rename | OK |
| Navigation dossiers | Query param `path` | Double-click + history | OK |

---

## Architecture

```
Frontend (SvelteKit)              Backend (Rust/Axum)
     │                                   │
     │  GET /api/files?path=             │
     │ ─────────────────────────────────►│
     │                                   │  fs::read_dir()
     │◄─────────────────────────────────│
     │  JSON: [{name, path, type, ...}]  │
     │                                   │
     │  POST /api/files/folder           │
     │  {path, name}                     │
     │ ─────────────────────────────────►│
     │                                   │  fs::create_dir()
     │◄─────────────────────────────────│
     │  201 Created + FileItem           │
     │                                   │
     │  DELETE /api/files?path=x         │
     │ ─────────────────────────────────►│
     │                                   │  fs::remove_*()
     │◄─────────────────────────────────│
     │  204 No Content                   │
     │                                   │
     │  PATCH /api/files/rename          │
     │  {path, new_name}                 │
     │ ─────────────────────────────────►│
     │                                   │  fs::rename()
     │◄─────────────────────────────────│
     │  200 OK + FileItem                │
```

---

## Fichiers modifiés/créés

### Backend

| Fichier | Description |
|---------|-------------|
| `backend/src/config/mod.rs` | Ajout `files_root` configuration |
| `backend/src/api/files.rs` | **Nouveau** - API complète files |
| `backend/src/api/mod.rs` | Export du module files |
| `backend/src/main.rs` | Route `/api/files` |
| `backend/data/files/` | Dossier racine (dev) |

### Frontend

| Fichier | Description |
|---------|-------------|
| `frontend/src/lib/stores/api.ts` | Méthodes getFiles, createFolder, deleteFile, renameFile |
| `frontend/src/lib/components/apps/FileManager.svelte` | Connexion API, états loading/error, actions |

---

## Sécurité

### Path Traversal Protection

Le backend valide tous les chemins pour empêcher les attaques de type path traversal :

```rust
fn validate_path(base: &Path, requested: &str) -> Result<PathBuf, String> {
    let full_path = base.join(requested);
    let canonical = full_path.canonicalize()?;

    if !canonical.starts_with(base.canonicalize()?) {
        return Err("Access denied: path outside allowed directory");
    }

    Ok(full_path)
}
```

Protections :
- Normalisation des chemins
- Vérification que le chemin reste dans `files_root`
- Validation des noms de fichiers (pas de `/`, `\`, ou `.` en début)
- Interdiction de supprimer la racine

---

## Lancer le MVP

### 1. Backend

```bash
cd backend
cargo run
# Server starts on http://localhost:3000
```

### 2. Frontend

```bash
cd frontend
npm run dev
# Server starts on http://localhost:5173
# API calls proxied to backend
```

### 3. Tester

1. Ouvrir http://localhost:5173
2. Cliquer sur l'icône "Files" dans le dock
3. Voir les fichiers du dossier `backend/data/files/`
4. Créer un dossier avec le bouton "+"
5. Supprimer via le menu contextuel (...)
6. Renommer via le menu contextuel

---

## Variables d'environnement

```bash
# Backend
PINAS_FILES_ROOT=./data/files        # Dev (défaut)
PINAS_FILES_ROOT=/storage/.pinas/files  # Production (LibreELEC)

PINAS_BIND_ADDRESS=0.0.0.0:3000
PINAS_DATABASE_URL=sqlite:./data/pinas.db?mode=rwc
```

---

## Tests API (curl)

```bash
# Lister fichiers racine
curl http://localhost:3000/api/files

# Lister sous-dossier
curl "http://localhost:3000/api/files?path=Documents"

# Créer dossier
curl -X POST http://localhost:3000/api/files/folder \
  -H "Content-Type: application/json" \
  -d '{"path":"","name":"NewFolder"}'

# Supprimer
curl -X DELETE "http://localhost:3000/api/files?path=NewFolder"

# Renommer
curl -X PATCH http://localhost:3000/api/files/rename \
  -H "Content-Type: application/json" \
  -d '{"path":"test.txt","new_name":"renamed.txt"}'
```

---

## Prochaines étapes (V2)

- [ ] Upload de fichiers
- [ ] Download de fichiers
- [ ] Copier/Coller/Déplacer
- [ ] Prévisualisation (images, texte, PDF)
- [ ] Corbeille
- [ ] Recherche dans les fichiers
- [ ] Permissions utilisateurs

---

## Validation LibreELEC

Pour tester sur LibreELEC :

1. Cross-compiler le backend :
   ```bash
   cross build --release --target aarch64-unknown-linux-musl
   ```

2. Copier le binaire sur le Pi :
   ```bash
   scp target/aarch64-unknown-linux-musl/release/pinas root@libreelec:/storage/.pinas/
   ```

3. Créer le dossier files :
   ```bash
   ssh root@libreelec "mkdir -p /storage/.pinas/files"
   ```

4. Lancer :
   ```bash
   ssh root@libreelec "PINAS_FILES_ROOT=/storage/.pinas/files /storage/.pinas/pinas"
   ```

---

*MVP validé le 16 janvier 2026*
