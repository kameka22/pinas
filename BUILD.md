 1. Installer les dépendances
  sudo apt-get update
  sudo apt-get install -y \
      git build-essential gcc g++ make \
      xfonts-utils rdfind gperf xsltproc lzop patchutils bc \
      libparse-yapp-perl libxml-parser-perl \
      wget curl unzip zip \
      python3 python3-pip \
      default-jre-headless \
      texinfo flex bison \
      libncurses5-dev libssl-dev

  2. Installer Rust
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source ~/.cargo/env
  rustup target add aarch64-unknown-linux-musl

  3. Installer Node.js
  curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
  sudo apt-get install -y nodejs

  4. Cloner et builder
  git clone https://github.com/ton-user/pinas.git
  cd pinas

  # Backend (pas besoin de cross sur ARM64 natif)
  cd backend
  cargo build --release --target aarch64-unknown-linux-musl
  cd ..

  # Frontend
  cd frontend
  npm install
  npm run build
  cd ..

  # Copier le binaire
  mkdir -p libreelec/packages/pinas/bin
  cp backend/target/aarch64-unknown-linux-musl/release/pinas libreelec/packages/pinas/bin/

  # Cloner LibreELEC
  mkdir -p extra
  cd extra
  git clone --branch libreelec-12.2 https://github.com/LibreELEC/LibreELEC.tv.git
  cd LibreELEC.tv

  # Installer le package PiNAS
  cp -r ../../libreelec/packages/pinas packages/
  echo 'PKG_DEPENDS_TARGET="$PKG_DEPENDS_TARGET pinas"' >> packages/virtual/mediacenter/package.mk

  # Builder l'image
  PROJECT=RPi DEVICE=RPi5 ARCH=aarch64 make image

  5. Récupérer l'image
  ls -la target/*.img.gz

  L'image sera dans extra/LibreELEC.tv/target/
