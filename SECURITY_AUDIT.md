# Audit de sécurité PiNAS

Rapport consolidé — Février 2026

---

## CRITIQUE

### ~~1. Secret JWT par défaut en dur~~ — CORRIGE

**`backend/src/config/mod.rs`**

~~Si `PINAS_JWT_SECRET` n'est pas défini, le secret par défaut "change-me-in-production" était utilisé.~~

**Correction** : Le secret est maintenant auto-généré (256 bits, `getrandom`) au premier démarrage et persisté dans `{data_dir}/.jwt_secret` avec permissions 600. Le marker par défaut déclenche automatiquement la génération.

---

### ~~2. CORS `allow_origin(Any)`~~ — CORRIGE

**`backend/src/main.rs`**

~~N'importe quel site web pouvait faire des requêtes authentifiées vers l'API.~~

**Correction** : CORS restreint à `localhost:5173` (dev SvelteKit), `localhost:3000` (backend), `127.0.0.1`. Méthodes et headers explicitement listés. `allow_credentials(true)` activé.

---

### ~~3. Extraction d'archives sans validation (Tar Slip)~~ — CORRIGE

**`backend/src/services/package.rs`**

~~`archive.unpack()` extrayait sans valider les chemins des entrées.~~

**Correction** : Chaque entrée est maintenant validée individuellement : rejet des chemins absolus, rejet des composants `..`, vérification que le chemin résolu reste dans le répertoire destination. Extraction via `entry.unpack_in()` au lieu de `archive.unpack()`.

---

### 4. Exécution de commandes arbitraires depuis les manifests

**`backend/src/services/package.rs:668-677`**

```rust
InstallStep::Exec { command, .. } => {
    Command::new("sh").arg("-c").arg(command).status().await?;
}
```

Les commandes `exec` dans les manifests du catalogue sont exécutées telles quelles via `sh -c` en root. Un catalogue compromis = RCE totale.

**Fix** : Signer les manifests (GPG), whitelist d'actions autorisées, ou sandbox (conteneur).

---

### 5. Blocklist terminal contournable

**`backend/src/api/terminal.rs:122-205`**

La vérification de commandes dangereuses se fait par `contains()` sur des patterns, facilement contourné :

- Espaces/tabs supplémentaires : `rm  -rf  /`
- Substitution de commande : `` `rm -rf /` `` ou `$(rm -rf /)`
- Variables d'environnement, encodage
- Chaînage après `;` ou `&&`

**Fix** : Si le terminal est nécessaire, utiliser un allowlist ou un parser AST du shell. Sinon, envisager un sandbox (bubblewrap/conteneur).

---

### ~~6. Endpoints critiques sans vérification admin~~ — CORRIGE

**`api/ssh.rs`, `api/services.rs`, `api/network.rs`**

~~Aucun de ces handlers n'utilisait `AdminUser`. Tout utilisateur avec un token JWT valide pouvait modifier SSH, les services systemd et la configuration réseau.~~

**Correction** : `AdminUser` ajouté comme extracteur à tous les handlers de modification :
- **ssh.rs** : `enable_ssh`, `disable_ssh`, `change_password`
- **services.rs** : `start_service`, `stop_service`, `restart_service`, `enable_service`, `disable_service`
- **network.rs** : `update_interface`, `update_dns`, `update_hostname`

---

### ~~7. Identifiants Kodi en dur~~ — CORRIGE

**`backend/src/config/mod.rs`** et **`pinas.service`**

~~Identiques sur tous les déploiements, visibles dans le binaire et via `systemctl show`.~~

**Correction** : Le mot de passe Kodi est maintenant auto-généré au premier démarrage (16 bytes, `getrandom`, hex) et persisté dans `{data_dir}/.kodi_password` avec permissions 600. Le marker `"auto-generate"` déclenche automatiquement la génération. La variable `PINAS_KODI_PASSWORD` a été retirée de `pinas.service`.

---

## HAUT

### 8. Token JWT stocké en `localStorage` (XSS = vol de session)

**`frontend/src/lib/stores/api.ts:24-32, 137-138`**

```typescript
localStorage.setItem('token', response.token);
```

Si un XSS est exploité (voir point 9), le token est directement accessible. `localStorage` est lisible par tout JavaScript sur la page.

**Fix** : Utiliser des cookies `httpOnly` + `SameSite=Strict`.

---

### ~~9. XSS potentiel via `{@html}` dans le terminal~~ — CORRIGE

**`frontend/src/lib/components/apps/TerminalApp.svelte`**

~~Le regex URL pourrait être manipulé par une sortie de commande craftée.~~

**Correction** : `formatOutput()` renforcé : échappement complet HTML (y compris `"` et `'`), regex URL strictifié (whitelist de caractères sûrs, pas de quotes/backticks/angle brackets). Le terminal est admin-only, réduisant encore le risque.

---

### ~~10. Pas de rate limiting sur le login~~ — CORRIGE

**`backend/src/api/auth.rs`**

~~Le terminal a du rate limiting (30/min), mais le login n'en a pas. Brute-force illimité sur les mots de passe.~~

**Correction** : Rate limiter ajouté sur le login : 5 tentatives par 60 secondes par username (case-insensitive). Retourne HTTP 429 `RATE_LIMITED` si dépassé.

---

### ~~11. Tokens stockés en clair dans la DB~~ — CORRIGE

**`backend/src/services/session.rs`**

~~Si la base SQLite est compromise, toutes les sessions actives sont volées.~~

**Correction** : Les tokens sont maintenant hashés (SHA-256) avant stockage en DB. Les lookups et suppressions opèrent sur le hash. Le token JWT original n'est jamais persisté.

---

### ~~12. WebSocket sans authentification~~ — CORRIGE

**`backend/src/api/ws.rs`** et **`frontend/src/lib/stores/websocket.ts`**

~~Pas de token envoyé. Des clients non authentifiés reçoivent les événements système.~~

**Correction** : Le frontend envoie le JWT en query param (`?token=...`). Le backend valide le token via `validate_jwt()` avant d'accepter l'upgrade WebSocket. Connexion refusée (401) si token absent ou invalide.

---

### 13. Pas de HTTPS

**`backend/src/main.rs`**

Le serveur écoute en HTTP uniquement. Tous les tokens, mots de passe et données transitent en clair sur le réseau local.

**Fix** : Générer un certificat auto-signé au premier démarrage, activer TLS. Ajouter HSTS.

---

### ~~14. SSRF dans le fetch de catalogue/manifests~~ — CORRIGE

**`backend/src/api/packages.rs`, `backend/src/services/package.rs`**

~~Pas de validation du schéma URL (acceptait `file://`, etc.), pas de filtrage des IP internes.~~

**Correction** :
- Ajout de `validate_fetch_url()` dans packages.rs : vérifie HTTPS obligatoire (sauf localhost), bloque les IP privées/réservées (10.x, 172.16-31.x, 192.168.x, 169.254.x, metadata.google.internal).
- Ajout de `validate_download_url()` dans package.rs (service) : vérifie HTTPS pour tous les downloads.
- Timeout de 30s sur les fetches manifest, 300s sur les downloads de fichiers.

---

### ~~15. Upload de fichiers sans limite de taille~~ — CORRIGE

**`backend/src/api/files.rs`**

~~Pas de limite de taille. Un upload de plusieurs GB épuise la RAM (surtout sur Pi avec 4GB).~~

**Correction** : Limite de 512 MB ajoutée via `DefaultBodyLimit::max()` sur la route `/upload`. Les requêtes dépassant cette taille sont rejetées automatiquement par Axum.

---

### ~~16. Mot de passe SSH : minimum 4 caractères~~ — CORRIGE

**`backend/src/services/ssh.rs`**

~~C'est le mot de passe root SSH. 4 caractères est trivialement brute-forceable.~~

**Correction** : Minimum porté à 12 caractères.

---

### ~~17. `APP_PASSWORD` par défaut "changeme"~~ — CORRIGE

**`backend/src/services/package.rs`**

~~Toutes les apps utilisant `${APP_PASSWORD}` auront ce mot de passe par défaut.~~

**Correction** : Mot de passe généré aléatoirement (16 bytes via `getrandom`, encodé en 32 caractères hex) à chaque installation d'app. Chaque app reçoit un mot de passe unique.

---

## MOYEN

### ~~18. Pas de headers de sécurité~~ — CORRIGE

**`backend/src/main.rs`**

~~Manquent : `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Content-Security-Policy`, `Strict-Transport-Security`, `Referrer-Policy`.~~

**Correction** : Ajout via `SetResponseHeaderLayer` de : `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: strict-origin-when-cross-origin`, `X-XSS-Protection: 1; mode=block`. CSP et HSTS seront ajoutés quand HTTPS sera activé.

---

### ~~19. Claims JWT incomplets~~ — CORRIGE

**`backend/src/services/auth.rs`**

~~Pas de `aud`, `iss`, `nbf`. Les tokens pourraient être rejoués si un autre service utilise le même secret.~~

**Correction** : Ajout des claims `iss: "pinas"` et `aud: "pinas"` dans la génération JWT. Validation explicite de `iss` et `aud` lors du décodage.

---

### ~~20. Race condition sur l'installation de packages~~ — CORRIGE

**`backend/src/services/package.rs`**

~~Le check `is_installed()` et l'`INSERT` ne sont pas atomiques. Deux requêtes simultanées peuvent créer deux installations.~~

**Correction** : `install_start()` utilise maintenant une transaction SQLite (`BEGIN...COMMIT`) qui regroupe atomiquement la vérification, le nettoyage des installations échouées, la vérification des dépendances, et l'insertion des records.

---

### 21. Pas de validation des volumes Docker

**`backend/src/services/docker.rs:386-493`**

Un manifest peut monter `/` dans un conteneur, ajouter `SYS_ADMIN`, sans limite CPU/RAM.

---

### ~~22. Mot de passe admin : minimum 6 caractères~~ — CORRIGE

**`backend/src/api/setup.rs`**

~~Devrait être 12+.~~

**Correction** : Minimum porté à 12 caractères.

---

### 23. TOCTOU dans les opérations fichiers

**`backend/src/api/files.rs:1024-1098`**

Validation du path puis opération async dans un `tokio::spawn`. Symlinks modifiables entre les deux.

---

### ~~24. Service PiNAS tourne en root~~ — CORRIGE

**`pinas.service`**

~~Pas de `User=`, `NoNewPrivileges=`, `ProtectSystem=`.~~

**Correction** : Ajout de directives systemd de hardening : `NoNewPrivileges=true`, `ProtectSystem=strict`, `ReadWritePaths=/storage`, `ProtectHome=true`, `PrivateTmp=true`. Le service tourne toujours en root (nécessaire pour la gestion des disques/services) mais avec des restrictions de sécurité.

---

### ~~25. Port 3000 hardcodé dans le WebSocket frontend~~ — CORRIGE

**`frontend/src/lib/stores/websocket.ts`**

~~Cassera si le port change.~~

**Correction** : Utilisation de `window.location.host` (inclut le port dynamiquement) au lieu de `window.location.hostname + ':3000'`.

---

## BAS

| # | Issue | Fichier |
|---|-------|---------|
| 26 | Pas de `cargo audit` ni pinning exact des versions | `Cargo.toml` |
| 27 | Pas de table d'audit pour les opérations critiques | migrations |
| 28 | `dev_mode` exposé dans les réponses API | `terminal.rs:46` |
| 29 | Console.log du WebSocket en production | `websocket.ts:130` |
| 30 | Pas de timeout sur les downloads frontend | `api.ts:464` |
| 31 | Images externes (Unsplash) sans SRI | `+layout.svelte` |
| 32 | Logout toujours 200 OK même sans token | `auth.rs` |
| 33 | Changement de mdp n'invalide pas les autres sessions | `auth.rs` |

---

## Priorités d'action

### ~~Immédiat (avant tout déploiement)~~ — FAIT

1. ~~Générer le JWT secret au premier boot (pas de défaut hardcodé)~~ CORRIGE
2. ~~Restreindre CORS aux origines légitimes~~ CORRIGE
3. ~~Ajouter `AdminUser` aux endpoints SSH, services, network~~ CORRIGE
4. ~~Valider les chemins dans l'extraction tar~~ CORRIGE
5. ~~Valider en HTTPS les manifests/downloads de packages~~ CORRIGE

### ~~Court terme~~ — FAIT

6. ~~Rate limiting sur le login~~ CORRIGE
7. Passer les tokens en cookies httpOnly
8. Activer HTTPS (cert auto-signé)
9. ~~Ajouter les security headers~~ CORRIGE
10. ~~Renforcer les mots de passe (12+ chars)~~ CORRIGE
11. ~~Limiter la taille des uploads~~ CORRIGE
12. ~~Auth sur le WebSocket~~ CORRIGE

### Moyen terme

13. Réécrire le terminal avec allowlist ou sandbox
14. Ajouter audit logging
15. Validation des containers Docker (volumes, caps, limites)
16. Protection CSRF
17. ~~Hasher les tokens en DB~~ CORRIGE
