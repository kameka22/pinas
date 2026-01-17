#!/bin/bash
# Build LibreELEC image with PiNAS pre-integrated
# Target: Raspberry Pi 5 (aarch64)
# Note: Uses Docker for LibreELEC build (required on macOS)

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LIBREELEC_BRANCH="libreelec-12.0"
PROJECT="RPi"
DEVICE="RPi5"
ARCH="aarch64"
LIBREELEC_DIR="${PROJECT_ROOT}/extra/LibreELEC.tv"
DOCKER_IMAGE="libreelec-builder"

echo "=== Building PiNAS for LibreELEC ==="
echo "Project root: $PROJECT_ROOT"
echo ""

# Check prerequisites
command -v cross >/dev/null 2>&1 || { echo "Error: 'cross' not installed. Run: cargo install cross"; exit 1; }
command -v docker >/dev/null 2>&1 || { echo "Error: Docker not installed or not running"; exit 1; }
docker info >/dev/null 2>&1 || { echo "Error: Docker daemon not running"; exit 1; }

# 1. Cross-compile backend
echo ">>> [1/8] Cross-compiling backend for aarch64..."
cd "${PROJECT_ROOT}/backend"
cross build --release --target aarch64-unknown-linux-musl

# Verify binary is static
if file target/aarch64-unknown-linux-musl/release/pinas | grep -q "statically linked"; then
    echo "    Binary is statically linked"
else
    echo "    Warning: Binary may not be statically linked"
fi

BINARY_SIZE=$(ls -lh target/aarch64-unknown-linux-musl/release/pinas | awk '{print $5}')
echo "    Binary size: $BINARY_SIZE"

# 2. Build frontend
echo ""
echo ">>> [2/8] Building frontend (SSG)..."
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

# 3. Copy binary to LibreELEC package
echo ""
echo ">>> [3/8] Preparing LibreELEC package..."
mkdir -p "${PROJECT_ROOT}/libreelec/packages/pinas/bin"
cp "${PROJECT_ROOT}/backend/target/aarch64-unknown-linux-musl/release/pinas" \
   "${PROJECT_ROOT}/libreelec/packages/pinas/bin/"

echo "    Binary copied to libreelec/packages/pinas/bin/"

# 4. Clone/update LibreELEC
echo ""
echo ">>> [4/8] Setting up LibreELEC source..."
if [ ! -d "$LIBREELEC_DIR" ]; then
    echo "    Cloning LibreELEC repository (this may take a while)..."
    git clone https://github.com/LibreELEC/LibreELEC.tv.git "$LIBREELEC_DIR"
fi

cd "$LIBREELEC_DIR"
git fetch origin
git checkout "$LIBREELEC_BRANCH"
git pull origin "$LIBREELEC_BRANCH" || true

echo "    LibreELEC version: $LIBREELEC_BRANCH"

# 5. Copy PiNAS package
echo ""
echo ">>> [5/8] Installing PiNAS package..."
rm -rf "${LIBREELEC_DIR}/packages/pinas"
cp -r "${PROJECT_ROOT}/libreelec/packages/pinas" "${LIBREELEC_DIR}/packages/"

echo "    Package installed to packages/pinas/"

# 6. Add PiNAS as dependency
echo ""
echo ">>> [6/8] Adding PiNAS to build..."

# Check if already added
if grep -q "pinas" "${LIBREELEC_DIR}/packages/virtual/mediacenter/package.mk"; then
    echo "    PiNAS already in mediacenter dependencies"
else
    echo 'PKG_DEPENDS_TARGET="$PKG_DEPENDS_TARGET pinas"' >> "${LIBREELEC_DIR}/packages/virtual/mediacenter/package.mk"
    echo "    Added PiNAS to mediacenter dependencies"
fi

# 7. Build Docker image for LibreELEC build
echo ""
echo ">>> [7/8] Building Docker image for LibreELEC build..."
cd "$LIBREELEC_DIR"

if docker image inspect "$DOCKER_IMAGE" >/dev/null 2>&1; then
    echo "    Docker image '$DOCKER_IMAGE' already exists"
else
    echo "    Creating Docker image (first time only)..."
    docker build --pull -t "$DOCKER_IMAGE" tools/docker/jammy
fi

# 8. Build LibreELEC image inside Docker
echo ""
echo ">>> [8/8] Building LibreELEC image inside Docker..."
echo "    This will take 2-4 hours on first build..."
echo "    Building: PROJECT=$PROJECT DEVICE=$DEVICE ARCH=$ARCH"
echo ""

docker run --rm \
    --log-driver none \
    -v "${LIBREELEC_DIR}:/build" \
    -w /build \
    -e PROJECT=$PROJECT \
    -e DEVICE=$DEVICE \
    -e ARCH=$ARCH \
    "$DOCKER_IMAGE" \
    make image 2>&1 | tee "${PROJECT_ROOT}/build.log"

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
