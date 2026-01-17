#!/bin/bash
# Build LibreELEC image with PiNAS pre-integrated
# For native ARM64 Ubuntu VM (no Docker needed)
# Target: Raspberry Pi 5 (aarch64)

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LIBREELEC_BRANCH="libreelec-12.2"
PROJECT="RPi"
DEVICE="RPi5"
ARCH="aarch64"
LIBREELEC_DIR="${PROJECT_ROOT}/extra/LibreELEC.tv"

echo "=== Building PiNAS for LibreELEC (ARM64 Native) ==="
echo "Project root: $PROJECT_ROOT"
echo ""

# Function to install system dependencies
install_system_deps() {
    echo ">>> Installing system dependencies..."
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
    echo "    System dependencies installed"
}

# Function to install Rust
install_rust() {
    echo ">>> Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    rustup target add aarch64-unknown-linux-musl
    echo "    Rust installed"
}

# Function to install Node.js
install_nodejs() {
    echo ">>> Installing Node.js 20..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt-get install -y nodejs
    echo "    Node.js installed"
}

# Check and install system dependencies if missing
if ! command -v gperf >/dev/null 2>&1 || ! command -v musl-gcc >/dev/null 2>&1; then
    install_system_deps
fi

# Check and install Rust if missing
if ! command -v cargo >/dev/null 2>&1; then
    install_rust
else
    # Ensure musl target is installed
    if ! rustup target list --installed | grep -q "aarch64-unknown-linux-musl"; then
        echo ">>> Adding Rust musl target..."
        rustup target add aarch64-unknown-linux-musl
    fi
fi

# Check and install Node.js if missing
if ! command -v npm >/dev/null 2>&1; then
    install_nodejs
fi

echo ">>> All prerequisites installed"
echo ""

# 1. Build backend (static binary with musl)
echo ">>> [1/6] Building backend for aarch64 (static with musl)..."
cd "${PROJECT_ROOT}/backend"
cargo build --release --target aarch64-unknown-linux-musl

BINARY_SIZE=$(ls -lh target/aarch64-unknown-linux-musl/release/pinas | awk '{print $5}')
echo "    Binary built: target/release/pinas ($BINARY_SIZE)"

# 2. Build frontend
echo ""
echo ">>> [2/6] Building frontend (SSG)..."
cd "${PROJECT_ROOT}/frontend"
npm install --silent
npm run build

# Verify build
if [ -f "build/index.html" ]; then
    echo "    Frontend build successful"
else
    echo "    Error: Frontend build failed (no index.html)"
    exit 1
fi

# 3. Copy binary and frontend to LibreELEC package
echo ""
echo ">>> [3/6] Preparing LibreELEC package..."
mkdir -p "${PROJECT_ROOT}/libreelec/packages/pinas/bin"
mkdir -p "${PROJECT_ROOT}/libreelec/packages/pinas/www"

# Copy binary
cp "${PROJECT_ROOT}/backend/target/aarch64-unknown-linux-musl/release/pinas" \
   "${PROJECT_ROOT}/libreelec/packages/pinas/bin/"
chmod +x "${PROJECT_ROOT}/libreelec/packages/pinas/bin/pinas"
echo "    Binary copied to libreelec/packages/pinas/bin/"

# Copy frontend
cp -r "${PROJECT_ROOT}/frontend/build/"* "${PROJECT_ROOT}/libreelec/packages/pinas/www/"
echo "    Frontend copied to libreelec/packages/pinas/www/"

# 4. Clone/update LibreELEC
echo ""
echo ">>> [4/6] Setting up LibreELEC source..."
mkdir -p "${PROJECT_ROOT}/extra"

if [ ! -d "$LIBREELEC_DIR" ]; then
    echo "    Cloning LibreELEC repository (this may take a while)..."
    git clone https://github.com/LibreELEC/LibreELEC.tv.git "$LIBREELEC_DIR"
fi

cd "$LIBREELEC_DIR"
git fetch origin
git checkout "$LIBREELEC_BRANCH"
git pull origin "$LIBREELEC_BRANCH" || true

echo "    LibreELEC version: $LIBREELEC_BRANCH"

# 5. Install PiNAS package and add dependency
echo ""
echo ">>> [5/6] Installing PiNAS package..."
rm -rf "${LIBREELEC_DIR}/packages/pinas"
cp -r "${PROJECT_ROOT}/libreelec/packages/pinas" "${LIBREELEC_DIR}/packages/"

# Check if already added
if grep -q "pinas" "${LIBREELEC_DIR}/packages/virtual/mediacenter/package.mk"; then
    echo "    PiNAS already in mediacenter dependencies"
else
    echo 'PKG_DEPENDS_TARGET="$PKG_DEPENDS_TARGET pinas"' >> "${LIBREELEC_DIR}/packages/virtual/mediacenter/package.mk"
    echo "    Added PiNAS to mediacenter dependencies"
fi

# 6. Build LibreELEC image (native, no Docker)
echo ""
echo ">>> [6/6] Building LibreELEC image..."
echo "    This will take 2-4 hours on first build..."
echo "    Building: PROJECT=$PROJECT DEVICE=$DEVICE ARCH=$ARCH"
echo ""

cd "$LIBREELEC_DIR"
PROJECT=$PROJECT DEVICE=$DEVICE ARCH=$ARCH make image 2>&1 | tee "${PROJECT_ROOT}/build.log"

# Check result
echo ""
echo "=== Build Complete ==="
if ls "${LIBREELEC_DIR}/target/"*.img.gz 1>/dev/null 2>&1; then
    echo "Image(s) created:"
    ls -lh "${LIBREELEC_DIR}/target/"*.img.gz
    echo ""
    echo "To flash to SD card:"
    echo "  gunzip -c ${LIBREELEC_DIR}/target/LibreELEC-RPi.aarch64-*.img.gz | sudo dd of=/dev/sdX bs=4M status=progress conv=fsync"
else
    echo "Error: No image file found. Check build.log for errors."
    exit 1
fi
