# Build PiNAS pour LibreELEC

## Prérequis

- **VM ARM64 Ubuntu 22.04+** (recommandé : UTM sur Mac ou VM cloud)
- ~50 GB d'espace disque
- Connexion internet stable

## Build rapide (une commande)

```bash
git clone https://github.com/kameka22/pinas.git
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
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt-get install -y nodejs
```

### 4. Cloner et builder

```bash
git clone https://github.com/kameka22/pinas.git
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
git clone --branch "$(cat ../libreelec/BRANCH)" https://github.com/LibreELEC/LibreELEC.tv.git   # branche définie dans libreelec/BRANCH
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

## Build d'image en CI (GitHub Actions)

`.github/workflows/build-libreelec.yml` construit les images à chaque push sur `master` (ou à la main via *Run workflow*) : Raspberry Pi 5 (`.img.gz`) et x86_64 (`.img.gz` + `.ova`), puis un brouillon de release.

Un job hébergé par GitHub est limité à 6 h. L'image Pi 5 tient dans un seul job (~2 h 30), mais l'image x86_64 en demande davantage en partant de zéro (LLVM, Mesa…). Elle est donc construite en deux jobs :

1. `build-libreelec` (x86_64, `handoff: true`) lance `make image` avec `AUTOREMOVE=yes` (LibreELEC supprime le dossier de travail d'un paquet dès que plus rien n'en dépend) et l'arrête 5 h après le début du job (`scripts/lib/ci-make-image.sh`, code de sortie 3). Il archive alors `LibreELEC.tv` (checkout, paquets PiNAS, toolchain, paquets construits et leurs stamps ; sans `sources/` ni ccache) en zstd et le publie en artefact `le-build-state-x86_64` (conservé 1 jour).
2. `build-libreelec-resume` récupère cet artefact au même chemin absolu, restaure le cache des archives sources et relance `make image` : les paquets déjà construits sont sautés grâce à leurs stamps. Il cherche d'abord l'artefact dans les artefacts du run (`gh api`, permission `actions: read`) : si le premier job a terminé l'image, il n'y en a pas et toutes les étapes sont sautées, sans erreur dans le journal.

Les archives sources passent d'un job et d'un run à l'autre par `actions/cache` (`le-sources-<branche>-<cible>-…`). Préparation du runner (dépendances, espace disque) : `scripts/lib/ci-prepare-runner.sh`.

---

## Dépannage

| Problème | Cause | Solution |
|----------|-------|----------|
| Service ne démarre pas | Binaire non exécutable | `chmod +x /usr/bin/pinas` |
| "not found" au lancement | Binaire non statique | Recompiler avec `--target aarch64-unknown-linux-musl` |
| 404 sur le frontend | Frontend pas copié | Vérifier `/storage/.pinas/www/` |
| Port 3000 inaccessible | Service pas démarré | `systemctl start pinas` |
| `make image` : « Incorrect checksum » puis « Cannot get … sources » | Le site amont renvoie une page anti-robots au lieu de l'archive (ex. linuxtv.org pour v4l-utils) ou l'archive a disparu (404, ex. releases.pagure.org pour newt) | Ajouter le paquet et un miroir de secours dans `scripts/lib/prefetch-sources.sh` (la somme `PKG_SHA256` de LibreELEC reste vérifiée) ; les scripts de build et la CI l'appellent avant `make image` |

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
| `PINAS_DB_PATH` | `/storage/.pinas/pinas.db` | Chemin base de données SQLite |
| `PINAS_CONFIG_PATH` | `/storage/.pinas/config.toml` | Fichier de configuration |
| `PINAS_LOG_PATH` | `/storage/.pinas/logs` | Dossier des logs |
| `PINAS_LOG_LEVEL` | `info` | Niveau de log |
| `PINAS_JWT_SECRET` | _(auto-généré)_ | Secret JWT (généré au premier boot) |
| `PINAS_SHARES_ROOT` | `/storage/shares` | Racine des partages |
| `PINAS_WWW_PATH` | `/storage/.pinas/www` | Dossier des fichiers frontend |
| `PINAS_PACKAGES_DIR` | `/storage/.pinas/packages` | Dossier des packages |
| `PINAS_DATA_DIR` | `/storage/.pinas/data` | Dossier des données |
| `PINAS_HOMES_ROOT` | `/storage/homes` | Dossier des homes utilisateurs |
| `PINAS_HOME_ON_DELETE` | `archive` | Action à la suppression user (archive/delete/keep) |
| `PINAS_DEV_MODE` | `false` | Mode dev (simule les opérations) |

Sur LibreELEC, les valeurs sont configurées dans `/usr/lib/systemd/system/pinas.service`.
