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
PACKAGE_DIR="${PROJECT_ROOT}/libreelec/packages/pinas"

# Build options (can be overridden via arguments)
BUILD_BACKEND=true
BUILD_FRONTEND=true
BUILD_LIBREELEC=true
CLEAN_BUILD=false

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse command line arguments
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --backend-only     Only build the Rust backend"
    echo "  --frontend-only    Only build the SvelteKit frontend"
    echo "  --skip-libreelec   Skip LibreELEC image build"
    echo "  --clean            Clean build directories before building"
    echo "  -h, --help         Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                      # Full build"
    echo "  $0 --frontend-only      # Only rebuild frontend"
    echo "  $0 --skip-libreelec     # Build PiNAS but skip LibreELEC image"
    exit 0
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --backend-only)
            BUILD_FRONTEND=false
            BUILD_LIBREELEC=false
            shift
            ;;
        --frontend-only)
            BUILD_BACKEND=false
            BUILD_LIBREELEC=false
            shift
            ;;
        --skip-libreelec)
            BUILD_LIBREELEC=false
            shift
            ;;
        --clean)
            CLEAN_BUILD=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            ;;
    esac
done

echo "=== Building PiNAS for LibreELEC (ARM64 Native) ==="
echo "Project root: $PROJECT_ROOT"
echo "Build backend: $BUILD_BACKEND"
echo "Build frontend: $BUILD_FRONTEND"
echo "Build LibreELEC: $BUILD_LIBREELEC"
echo ""

# Function to extract version from Cargo.toml
get_version() {
    grep '^version' "${PROJECT_ROOT}/backend/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/'
}

# Function to verify required package files exist
verify_package_files() {
    echo ">>> Verifying package structure..."
    local missing=0

    local required_files=(
        "package.mk"
        "bin/pinas-init.sh"
        "system.d/pinas.service"
        "tmpfiles.d/pinas.conf"
    )

    for file in "${required_files[@]}"; do
        if [ ! -f "${PACKAGE_DIR}/${file}" ]; then
            echo -e "    ${RED}Missing: ${file}${NC}"
            missing=$((missing + 1))
        else
            echo -e "    ${GREEN}✓${NC} ${file}"
        fi
    done

    if [ $missing -gt 0 ]; then
        echo -e "${RED}Error: $missing required package file(s) missing${NC}"
        exit 1
    fi
    echo "    All package files present"
}

# Function to sync version in package.mk
sync_version() {
    local version=$(get_version)
    echo ">>> Synchronizing version: $version"

    if [ -f "${PACKAGE_DIR}/package.mk" ]; then
        sed -i "s/^PKG_VERSION=.*/PKG_VERSION=\"${version}\"/" "${PACKAGE_DIR}/package.mk"
        echo "    Updated package.mk version to $version"
    fi
}

# Function to clean build directories
clean_build_dirs() {
    echo ">>> Cleaning build directories..."

    if [ "$BUILD_BACKEND" = true ]; then
        rm -rf "${PROJECT_ROOT}/backend/target"
        echo "    Cleaned backend/target/"
    fi

    if [ "$BUILD_FRONTEND" = true ]; then
        rm -rf "${PROJECT_ROOT}/frontend/build"
        rm -rf "${PROJECT_ROOT}/frontend/.svelte-kit"
        echo "    Cleaned frontend/build/ and .svelte-kit/"
    fi

    # Always clean the package staging directories
    rm -rf "${PACKAGE_DIR}/bin/pinas"
    rm -rf "${PACKAGE_DIR}/www"
    echo "    Cleaned package staging directories"
}

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

# Clean if requested
if [ "$CLEAN_BUILD" = true ]; then
    clean_build_dirs
    echo ""
fi

# Verify package structure
verify_package_files
echo ""

# Sync version
sync_version
VERSION=$(get_version)
echo ""

# Calculate total steps
TOTAL_STEPS=3
[ "$BUILD_BACKEND" = true ] && TOTAL_STEPS=$((TOTAL_STEPS + 1))
[ "$BUILD_FRONTEND" = true ] && TOTAL_STEPS=$((TOTAL_STEPS + 1))
[ "$BUILD_LIBREELEC" = true ] && TOTAL_STEPS=$((TOTAL_STEPS + 1))
CURRENT_STEP=0

# 1. Build backend (static binary with musl)
if [ "$BUILD_BACKEND" = true ]; then
    CURRENT_STEP=$((CURRENT_STEP + 1))
    echo ">>> [${CURRENT_STEP}/${TOTAL_STEPS}] Building backend for aarch64 (static with musl)..."
    cd "${PROJECT_ROOT}/backend"
    cargo build --release --target aarch64-unknown-linux-musl

    # Verify binary is statically linked
    BINARY_PATH="target/aarch64-unknown-linux-musl/release/pinas"
    if [ ! -f "$BINARY_PATH" ]; then
        echo -e "    ${RED}Error: Binary not found at $BINARY_PATH${NC}"
        exit 1
    fi

    BINARY_SIZE=$(ls -lh "$BINARY_PATH" | awk '{print $5}')
    echo -e "    ${GREEN}✓${NC} Binary built: $BINARY_PATH ($BINARY_SIZE)"

    # Check if binary is statically linked (on Linux)
    if command -v file >/dev/null 2>&1; then
        if file "$BINARY_PATH" | grep -q "statically linked"; then
            echo -e "    ${GREEN}✓${NC} Binary is statically linked"
        else
            echo -e "    ${YELLOW}Warning: Binary may not be statically linked${NC}"
        fi
    fi
fi

# 2. Build frontend
if [ "$BUILD_FRONTEND" = true ]; then
    CURRENT_STEP=$((CURRENT_STEP + 1))
    echo ""
    echo ">>> [${CURRENT_STEP}/${TOTAL_STEPS}] Building frontend (SSG)..."
    cd "${PROJECT_ROOT}/frontend"
    npm install --silent
    npm run build

    # Verify build
    if [ -f "build/index.html" ]; then
        FILE_COUNT=$(find build -type f | wc -l)
        BUILD_SIZE=$(du -sh build | awk '{print $1}')
        echo -e "    ${GREEN}✓${NC} Frontend build successful ($FILE_COUNT files, $BUILD_SIZE)"
    else
        echo -e "    ${RED}Error: Frontend build failed (no index.html)${NC}"
        exit 1
    fi
fi

# 3. Copy binary and frontend to LibreELEC package
CURRENT_STEP=$((CURRENT_STEP + 1))
echo ""
echo ">>> [${CURRENT_STEP}/${TOTAL_STEPS}] Preparing LibreELEC package..."

# Ensure directories exist and are clean
mkdir -p "${PACKAGE_DIR}/bin"
rm -rf "${PACKAGE_DIR}/www"
mkdir -p "${PACKAGE_DIR}/www"

# Copy binary (if backend was built)
if [ "$BUILD_BACKEND" = true ]; then
    cp "${PROJECT_ROOT}/backend/target/aarch64-unknown-linux-musl/release/pinas" \
       "${PACKAGE_DIR}/bin/"
    chmod +x "${PACKAGE_DIR}/bin/pinas"
    echo -e "    ${GREEN}✓${NC} Binary copied to package"
elif [ -f "${PACKAGE_DIR}/bin/pinas" ]; then
    echo "    Using existing binary in package"
else
    echo -e "    ${YELLOW}Warning: No binary in package (use --backend-only or full build)${NC}"
fi

# Copy frontend (if frontend was built)
if [ "$BUILD_FRONTEND" = true ]; then
    cp -r "${PROJECT_ROOT}/frontend/build/"* "${PACKAGE_DIR}/www/"
    WWW_COUNT=$(find "${PACKAGE_DIR}/www" -type f | wc -l)
    echo -e "    ${GREEN}✓${NC} Frontend copied to package ($WWW_COUNT files)"
else
    echo -e "    ${YELLOW}Warning: Frontend not rebuilt (www/ directory may be empty)${NC}"
fi

# Skip LibreELEC steps if not building
if [ "$BUILD_LIBREELEC" = false ]; then
    echo ""
    echo "=== PiNAS Package Build Complete ==="
    echo "Package location: ${PACKAGE_DIR}"
    echo ""
    echo "Package contents:"
    find "${PACKAGE_DIR}" -type f | sort | while read f; do
        SIZE=$(ls -lh "$f" | awk '{print $5}')
        echo "  $f ($SIZE)"
    done
    echo ""
    echo "To build the full LibreELEC image, run without --skip-libreelec"
    exit 0
fi

# 4. Clone/update LibreELEC
CURRENT_STEP=$((CURRENT_STEP + 1))
echo ""
echo ">>> [${CURRENT_STEP}/${TOTAL_STEPS}] Setting up LibreELEC source..."
mkdir -p "${PROJECT_ROOT}/extra"

if [ ! -d "$LIBREELEC_DIR" ]; then
    echo "    Cloning LibreELEC repository (this may take a while)..."
    git clone https://github.com/LibreELEC/LibreELEC.tv.git "$LIBREELEC_DIR"
fi

cd "$LIBREELEC_DIR"
git fetch origin
git checkout "$LIBREELEC_BRANCH"
git pull origin "$LIBREELEC_BRANCH" || true

echo -e "    ${GREEN}✓${NC} LibreELEC version: $LIBREELEC_BRANCH"

# 5. Install PiNAS package and add dependency
CURRENT_STEP=$((CURRENT_STEP + 1))
echo ""
echo ">>> [${CURRENT_STEP}/${TOTAL_STEPS}] Installing PiNAS package into LibreELEC..."
rm -rf "${LIBREELEC_DIR}/packages/pinas"
cp -r "${PACKAGE_DIR}" "${LIBREELEC_DIR}/packages/"

# Verify www directory was copied
if [ -d "${LIBREELEC_DIR}/packages/pinas/www" ] && [ -f "${LIBREELEC_DIR}/packages/pinas/www/index.html" ]; then
    echo -e "    ${GREEN}✓${NC} Frontend files included in package"
else
    echo -e "    ${RED}Error: Frontend files missing in package (no www/index.html)${NC}"
    echo "    Run with --frontend-only first or do a full build"
    exit 1
fi

# Verify binary was copied
if [ -f "${LIBREELEC_DIR}/packages/pinas/bin/pinas" ]; then
    echo -e "    ${GREEN}✓${NC} Backend binary included in package"
else
    echo -e "    ${RED}Error: Backend binary missing in package${NC}"
    echo "    Run with --backend-only first or do a full build"
    exit 1
fi

# Check if already added to mediacenter dependencies
if grep -q "pinas" "${LIBREELEC_DIR}/packages/virtual/mediacenter/package.mk"; then
    echo "    PiNAS already in mediacenter dependencies"
else
    echo 'PKG_DEPENDS_TARGET="$PKG_DEPENDS_TARGET pinas"' >> "${LIBREELEC_DIR}/packages/virtual/mediacenter/package.mk"
    echo -e "    ${GREEN}✓${NC} Added PiNAS to mediacenter dependencies"
fi

# 6. Build LibreELEC image (native, no Docker)
CURRENT_STEP=$((CURRENT_STEP + 1))
echo ""
echo ">>> [${CURRENT_STEP}/${TOTAL_STEPS}] Building LibreELEC image..."
echo "    This will take 2-4 hours on first build..."
echo "    Building: PROJECT=$PROJECT DEVICE=$DEVICE ARCH=$ARCH"
echo "    PiNAS version: $VERSION"
echo ""

cd "$LIBREELEC_DIR"
PROJECT=$PROJECT DEVICE=$DEVICE ARCH=$ARCH make image 2>&1 | tee "${PROJECT_ROOT}/build.log"

# Check result
echo ""
echo "=== Build Complete ==="
if ls "${LIBREELEC_DIR}/target/"*.img.gz 1>/dev/null 2>&1; then
    echo -e "${GREEN}Image(s) created:${NC}"
    ls -lh "${LIBREELEC_DIR}/target/"*.img.gz
    echo ""
    echo "PiNAS version: $VERSION"
    echo ""
    echo "To flash to SD card:"
    echo "  gunzip -c ${LIBREELEC_DIR}/target/LibreELEC-RPi5.aarch64-*.img.gz | sudo dd of=/dev/sdX bs=4M status=progress conv=fsync"
else
    echo -e "${RED}Error: No image file found. Check build.log for errors.${NC}"
    exit 1
fi
