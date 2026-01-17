# Déploiement MVP PiNAS sur LibreELEC (Raspberry Pi 5)

> **Objectif** : Créer une image LibreELEC personnalisée avec PiNAS pré-intégré
> **Cible** : Raspberry Pi 5 (aarch64)
> **LibreELEC** : Version 12.x ou 13.x

---

## Vue d'ensemble

```
┌─────────────────────────────────────────────────────────────────┐
│                    Image LibreELEC Custom                       │
├─────────────────────────────────────────────────────────────────┤
│  Kodi (media center)              PiNAS (NAS management)        │
│  Port 8080                        Port 3000                     │
├─────────────────────────────────────────────────────────────────┤
│  /flash (read-only)         │     /storage (read-write)        │
│  └── SYSTEM (squashfs)      │     ├── .pinas/                  │
│      └── pinas binary       │     │   ├── pinas.db             │
│                             │     │   └── files/               │
│                             │     └── .kodi/                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Prérequis

### Sur votre machine de développement (Mac/Linux)

- [ ] Docker installé (pour cross-compilation avec `cross`)
- [ ] Rust installé (`rustup`)
- [ ] Node.js 18+ installé
- [ ] ~50 GB d'espace disque (pour le build LibreELEC)
- [ ] Connexion internet stable

### Installation des outils

```bash
# Installer cross (cross-compilation Rust)
cargo install cross

# Vérifier Docker
docker --version
```

---

## Phase 1 : Préparer le Backend (Cross-compilation)

### 1.1 Installer la cible aarch64

```bash
rustup target add aarch64-unknown-linux-musl
```

### 1.2 Cross-compiler avec `cross`

```bash
cd backend

# Compiler en release pour aarch64 (statique avec musl)
cross build --release --target aarch64-unknown-linux-musl

# Vérifier le binaire
file target/aarch64-unknown-linux-musl/release/pinas
# Doit afficher: "statically linked"

# Taille du binaire
ls -lh target/aarch64-unknown-linux-musl/release/pinas
```

### 1.3 Vérifier que le binaire est statique

```bash
# Ne doit rien afficher (pas de dépendances dynamiques)
readelf -d target/aarch64-unknown-linux-musl/release/pinas 2>/dev/null || echo "OK: static binary"
```

---

## Phase 2 : Préparer le Frontend (SSG Build)

### 2.1 Builder le frontend

```bash
cd frontend

# Installer les dépendances
npm install

# Builder en mode SSG (Static Site Generation)
npm run build

# Vérifier le build
ls -la build/
```

### 2.2 Le frontend sera embarqué

Option A : **Embarqué dans le binaire Rust** (via `rust-embed`) - Recommandé
Option B : **Fichiers statiques séparés** servis par le backend

Pour le MVP, le backend servira les fichiers statiques depuis un dossier.

---

## Phase 3 : Créer le package LibreELEC

### 3.1 Structure du package

```
libreelec/
└── packages/
    └── pinas/
        ├── package.mk                 # Définition du package
        ├── system.d/
        │   └── pinas.service          # Service systemd
        ├── tmpfiles.d/
        │   └── pinas.conf             # Création des dossiers au boot
        └── bin/
            ├── pinas                  # Binaire compilé (copié ici)
            └── pinas-init.sh          # Script d'initialisation
```

### 3.2 Fichier package.mk

```makefile
# SPDX-License-Identifier: GPL-3.0
# PiNAS - NAS Management for LibreELEC

PKG_NAME="pinas"
PKG_VERSION="0.1.0"
PKG_LICENSE="GPL-3.0"
PKG_SITE="https://github.com/your-repo/pinas"
PKG_URL=""
PKG_DEPENDS_TARGET="toolchain"
PKG_LONGDESC="PiNAS - Modern NAS management interface for Raspberry Pi"
PKG_TOOLCHAIN="manual"

makeinstall_target() {
  # Installer le binaire
  mkdir -p ${INSTALL}/usr/bin
  cp ${PKG_DIR}/bin/pinas ${INSTALL}/usr/bin/
  chmod 755 ${INSTALL}/usr/bin/pinas

  # Installer le script d'init
  cp ${PKG_DIR}/bin/pinas-init.sh ${INSTALL}/usr/bin/
  chmod 755 ${INSTALL}/usr/bin/pinas-init.sh

  # Installer le service systemd
  mkdir -p ${INSTALL}/usr/lib/systemd/system
  cp ${PKG_DIR}/system.d/pinas.service ${INSTALL}/usr/lib/systemd/system/

  # Installer la config tmpfiles (création dossiers)
  mkdir -p ${INSTALL}/usr/lib/tmpfiles.d
  cp ${PKG_DIR}/tmpfiles.d/pinas.conf ${INSTALL}/usr/lib/tmpfiles.d/
}

post_install() {
  # Activer le service au démarrage
  enable_service pinas.service
}
```

### 3.3 Service systemd (pinas.service)

```ini
[Unit]
Description=PiNAS - NAS Management Service
After=network-online.target kodi.service
Wants=network-online.target

[Service]
Type=simple
Environment=PINAS_FILES_ROOT=/storage/.pinas/files
Environment=PINAS_DATABASE_URL=sqlite:/storage/.pinas/data/pinas.db?mode=rwc
Environment=PINAS_BIND_ADDRESS=0.0.0.0:3000
Environment=PINAS_LOG_LEVEL=info
ExecStartPre=/usr/bin/pinas-init.sh
ExecStart=/usr/bin/pinas
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

### 3.4 Script d'initialisation (pinas-init.sh)

```bash
#!/bin/sh
# Créer les dossiers PiNAS dans /storage (persistant)

mkdir -p /storage/.pinas/files
mkdir -p /storage/.pinas/data
mkdir -p /storage/.pinas/logs

# Créer quelques dossiers par défaut
mkdir -p /storage/.pinas/files/Documents
mkdir -p /storage/.pinas/files/Photos
mkdir -p /storage/.pinas/files/Videos
mkdir -p /storage/.pinas/files/Music

echo "PiNAS directories initialized"
```

### 3.5 Config tmpfiles (pinas.conf)

```
# Créer les dossiers PiNAS au démarrage
d /storage/.pinas 0755 root root -
d /storage/.pinas/files 0755 root root -
d /storage/.pinas/data 0755 root root -
d /storage/.pinas/logs 0755 root root -
```

---

## Phase 4 : Intégrer dans le build LibreELEC

### 4.1 Cloner LibreELEC

```bash
# Cloner le repo officiel
git clone https://github.com/LibreELEC/LibreELEC.tv.git
cd LibreELEC.tv

# Checkout la branche stable (12.0 ou 13.0)
git checkout libreelec-12.0
```

### 4.2 Copier le package PiNAS

```bash
# Créer le dossier du package
mkdir -p packages/pinas/pinas

# Copier les fichiers du package
cp -r /chemin/vers/openmediavault/libreelec/packages/pinas/* packages/pinas/pinas/

# Copier le binaire compilé
cp /chemin/vers/openmediavault/backend/target/aarch64-unknown-linux-musl/release/pinas \
   packages/pinas/pinas/bin/
```

### 4.3 Ajouter PiNAS comme dépendance

Modifier `packages/virtual/mediacenter/package.mk` :

```bash
# Ajouter pinas aux dépendances
echo 'PKG_DEPENDS_TARGET="$PKG_DEPENDS_TARGET pinas"' >> packages/virtual/mediacenter/package.mk
```

Ou créer un package virtuel custom dans `packages/virtual/pinas-bundle/package.mk`.

### 4.4 (Optionnel) Personnaliser le branding

Modifier `distributions/LibreELEC/version` pour ajouter "-PiNAS" au nom.

---

## Phase 5 : Builder l'image

### 5.1 Configuration du build

```bash
# Variables d'environnement
export PROJECT=RPi
export DEVICE=RPi5
export ARCH=aarch64
```

### 5.2 Lancer le build

```bash
# Build complet (première fois : ~2-4 heures)
make image

# Ou avec plus de détails
PROJECT=RPi DEVICE=RPi5 ARCH=aarch64 make image 2>&1 | tee build.log
```

### 5.3 Récupérer l'image

```bash
# L'image sera dans target/
ls -la target/LibreELEC-RPi5.aarch64-*.img.gz
```

---

## Phase 6 : Flasher et tester

### 6.1 Flasher la carte SD

```bash
# Avec balenaEtcher (recommandé) ou dd
gunzip -c target/LibreELEC-RPi5.aarch64-12.0-pinas.img.gz | \
  sudo dd of=/dev/sdX bs=4M status=progress conv=fsync
```

### 6.2 Premier démarrage

1. Insérer la SD dans le Pi 5
2. Brancher écran, clavier, réseau
3. Attendre le boot complet (~1-2 minutes)
4. Kodi se lance automatiquement

### 6.3 Activer SSH (si pas déjà fait)

Dans Kodi :
- Settings → LibreELEC → Services → SSH → Enable

### 6.4 Vérifier PiNAS

```bash
# Se connecter en SSH
ssh root@<IP_DU_PI>
# Mot de passe par défaut : libreelec

# Vérifier le service
systemctl status pinas

# Voir les logs
journalctl -u pinas -f

# Tester l'API
curl http://localhost:3000/api/health
curl http://localhost:3000/api/files
```

### 6.5 Accéder à l'interface web

Ouvrir dans un navigateur : `http://<IP_DU_PI>:3000`

---

## Phase 7 : Débogage

### Problèmes courants

| Problème | Cause probable | Solution |
|----------|----------------|----------|
| Service ne démarre pas | Binaire non exécutable | `chmod +x /usr/bin/pinas` |
| "not found" au lancement | Binaire non statique | Recompiler avec musl |
| Erreur SQLite | Dossier /storage/.pinas manquant | Vérifier pinas-init.sh |
| Port 3000 inaccessible | Firewall | Vérifier avec `ss -tlnp` |

### Commandes utiles

```bash
# Logs système
journalctl -xe

# Processus en cours
ps aux | grep pinas

# Ports ouverts
ss -tlnp

# Espace disque
df -h

# Contenu de /storage
ls -la /storage/.pinas/
```

---

## Phase 8 : Automatisation (CI/CD)

> **IMPORTANT** : Le build LibreELEC nécessite **Linux x86_64**. Il ne fonctionne pas sur macOS (ni Intel, ni Apple Silicon).

### Option 1 : GitHub Actions (Recommandé)

Un workflow GitHub Actions est disponible dans `.github/workflows/build-libreelec.yml`.

**Pour lancer un build :**

1. Push le code sur GitHub
2. Aller dans Actions → "Build LibreELEC Image with PiNAS"
3. Cliquer "Run workflow"
4. Attendre ~4-6 heures
5. Télécharger l'image depuis les artifacts

Le workflow :
- Cross-compile le backend Rust
- Build le frontend
- Clone LibreELEC
- Intègre le package PiNAS
- Build l'image complète
- Upload l'image en artifact (et en release draft)

### Option 2 : VM Linux

Si tu préfères builder localement, utilise une VM Ubuntu 22.04 :

```bash
# Dans la VM Ubuntu
git clone https://github.com/your-repo/pinas.git
cd pinas
./scripts/build-libreelec-image.sh
```

### Script de build (pour Linux uniquement)

Le script `scripts/build-libreelec-image.sh` automatise tout le processus :

```bash
./scripts/build-libreelec-image.sh
```

Ce script :
1. Cross-compile le backend pour aarch64
2. Build le frontend
3. Clone LibreELEC (si nécessaire)
4. Copie le package PiNAS
5. Build l'image via Docker (sur Linux)

---

## Checksums et vérification

| Composant | Fichier | Vérification |
|-----------|---------|--------------|
| Backend | `pinas` | `file pinas` → "statically linked" |
| Frontend | `build/` | Contient index.html |
| Image | `*.img.gz` | `sha256sum *.img.gz` |

---

## Ressources

- [LibreELEC GitHub](https://github.com/LibreELEC/LibreELEC.tv)
- [LibreELEC Wiki - Build](https://wiki.libreelec.tv/development/build-commands)
- [Forum LibreELEC - Development](https://forum.libreelec.tv/board/24-development/)
- [Cross (Rust cross-compilation)](https://github.com/cross-rs/cross)

---

## Checklist finale

- [ ] Backend cross-compilé (aarch64-musl, statique)
- [ ] Frontend buildé (SSG)
- [ ] Package LibreELEC créé
- [ ] LibreELEC cloné et configuré
- [ ] Image buildée
- [ ] Image flashée sur SD
- [ ] Pi 5 démarre correctement
- [ ] Service PiNAS actif
- [ ] API accessible sur :3000
- [ ] Interface web fonctionnelle

---

*Dernière mise à jour : Janvier 2025*
