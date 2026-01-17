# Build PiNAS pour LibreELEC

## Prérequis

- **VM ARM64 Ubuntu 22.04+** (recommandé : UTM sur Mac ou VM cloud)
- ~50 GB d'espace disque
- Connexion internet stable

## Build rapide (une commande)

```bash
git clone https://github.com/ton-user/pinas.git
cd pinas
./scripts/build-arm64.sh
```

Le script installe automatiquement toutes les dépendances si nécessaire.

---

## Build manuel (étape par étape)

### 1. Installer les dépendances système

```bash
sudo apt-get update
sudo apt-get install -y \
    git build-essential gcc g++ make \
    xfonts-utils rdfind gperf xsltproc lzop patchutils bc \
    libparse-yapp-perl libxml-parser-perl \
    wget curl unzip zip \
    python3 python3-pip \
    default-jre-headless \
    texinfo flex bison \
    libncurses5-dev libssl-dev \
    musl-tools
```

### 2. Installer Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
rustup target add aarch64-unknown-linux-musl
```

### 3. Installer Node.js

```bash
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```

### 4. Cloner et builder

```bash
git clone https://github.com/ton-user/pinas.git
cd pinas

# Backend (binaire statique avec musl)
cd backend
cargo build --release --target aarch64-unknown-linux-musl
cd ..

# Frontend
cd frontend
npm install
npm run build
cd ..

# Copier le binaire et le frontend dans le package
mkdir -p libreelec/packages/pinas/bin
mkdir -p libreelec/packages/pinas/www
cp backend/target/aarch64-unknown-linux-musl/release/pinas libreelec/packages/pinas/bin/
cp -r frontend/build/* libreelec/packages/pinas/www/

# Cloner LibreELEC
mkdir -p extra
cd extra
git clone --branch libreelec-12.2 https://github.com/LibreELEC/LibreELEC.tv.git
cd LibreELEC.tv

# Installer le package PiNAS
cp -r ../../libreelec/packages/pinas packages/
echo 'PKG_DEPENDS_TARGET="$PKG_DEPENDS_TARGET pinas"' >> packages/virtual/mediacenter/package.mk

# Builder l'image (2-4 heures)
PROJECT=RPi DEVICE=RPi5 ARCH=aarch64 make image
```

### 5. Récupérer l'image

```bash
ls -la extra/LibreELEC.tv/target/*.img.gz
```

---

## Flasher l'image

### Avec dd (Linux/Mac)

```bash
gunzip -c extra/LibreELEC.tv/target/LibreELEC-RPi5.aarch64-*.img.gz | \
    sudo dd of=/dev/sdX bs=4M status=progress conv=fsync
```

### Avec balenaEtcher (GUI)

1. Ouvrir balenaEtcher
2. Sélectionner l'image `.img.gz`
3. Sélectionner la carte SD
4. Flasher

---

## Premier démarrage

1. Insérer la SD dans le Raspberry Pi 5
2. Brancher écran, clavier, réseau
3. Attendre le boot complet (~1-2 min)
4. Kodi se lance automatiquement

### Activer SSH (si nécessaire)

Dans Kodi : Settings → LibreELEC → Services → SSH → Enable

### Vérifier PiNAS

```bash
# Connexion SSH (mot de passe: libreelec)
ssh root@<IP_DU_PI>

# Activer le service (si pas actif)
systemctl enable pinas
systemctl start pinas

# Vérifier le service
systemctl status pinas

# Voir les logs
journalctl -u pinas -f

# Tester l'API
curl http://localhost:3000/api/health
```

### Accéder à l'interface web

Ouvrir dans un navigateur : `http://<IP_DU_PI>:3000/`

---

## Structure du package LibreELEC

```
libreelec/packages/pinas/
├── package.mk              # Définition du package
├── bin/
│   ├── pinas               # Binaire backend (compilé)
│   └── pinas-init.sh       # Script d'initialisation
├── www/                    # Frontend (build SvelteKit)
│   ├── index.html
│   └── ...
├── system.d/
│   └── pinas.service       # Service systemd
└── tmpfiles.d/
    └── pinas.conf          # Création des dossiers
```

---

## Dépannage

| Problème | Cause | Solution |
|----------|-------|----------|
| Service ne démarre pas | Binaire non exécutable | `chmod +x /usr/bin/pinas` |
| "not found" au lancement | Binaire non statique | Recompiler avec `--target aarch64-unknown-linux-musl` |
| 404 sur le frontend | Frontend pas copié | Vérifier `/storage/.pinas/www/` |
| Port 3000 inaccessible | Service pas démarré | `systemctl start pinas` |

### Commandes utiles sur le Pi

```bash
# Logs système
journalctl -u pinas -n 50

# Processus
ps aux | grep pinas

# Ports ouverts
ss -tlnp | grep 3000

# Espace disque
df -h

# Contenu PiNAS
ls -la /storage/.pinas/
```

---

## Variables d'environnement

| Variable | Défaut | Description |
|----------|--------|-------------|
| `PINAS_BIND_ADDRESS` | `0.0.0.0:3000` | Adresse d'écoute |
| `PINAS_DATABASE_URL` | `sqlite:./data/pinas.db` | Chemin base de données |
| `PINAS_FILES_ROOT` | `./data/files` | Racine du gestionnaire de fichiers |
| `PINAS_STATIC_DIR` | _(none)_ | Dossier des fichiers frontend |
| `PINAS_LOG_LEVEL` | `info` | Niveau de log |

Sur LibreELEC, les valeurs sont configurées dans `/usr/lib/systemd/system/pinas.service` :
- `PINAS_FILES_ROOT=/storage/.pinas/files`
- `PINAS_DATABASE_URL=sqlite:/storage/.pinas/data/pinas.db?mode=rwc`
- `PINAS_STATIC_DIR=/storage/.pinas/www`
