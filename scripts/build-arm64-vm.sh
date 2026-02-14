#!/bin/bash
# Build LibreELEC image with PiNAS pre-integrated
# For ARM64 VMs (QEMU, UTM on macOS Apple Silicon, Proxmox ARM64)
# Target: Virtual aarch64 (mainline kernel + virtio)

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LIBREELEC_BRANCH="libreelec-12.2"
PROJECT="Virtual"
ARCH="aarch64"
RUST_TARGET="aarch64-unknown-linux-musl"
LIBREELEC_DIR="${PROJECT_ROOT}/extra/LibreELEC.tv"
PACKAGE_DIR="${PROJECT_ROOT}/libreelec/packages/pinas"

# Build options (can be overridden via arguments)
BUILD_BACKEND=true
BUILD_FRONTEND=true
BUILD_LIBREELEC=true
CLEAN_BUILD=false
OUTPUT_QCOW2=true
OUTPUT_RAW=false

# Disk image sizes
DISK_SIZE_MB=2048
BOOT_SIZE_MB=1024
STORAGE_SIZE_MB=960

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse command line arguments
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Build PiNAS for ARM64 VMs (QEMU, UTM, Proxmox)"
    echo ""
    echo "Options:"
    echo "  --backend-only     Only build the Rust backend"
    echo "  --frontend-only    Only build the SvelteKit frontend"
    echo "  --skip-libreelec   Skip LibreELEC image build"
    echo "  --clean            Clean build directories before building"
    echo "  --qcow2            Output QCOW2 image (default)"
    echo "  --raw              Output raw .img only (no QCOW2 conversion)"
    echo "  -h, --help         Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                      # Full build (QCOW2 output)"
    echo "  $0 --raw                # Full build (raw .img output)"
    echo "  $0 --frontend-only      # Only rebuild frontend"
    echo "  $0 --skip-libreelec     # Build PiNAS but skip LibreELEC image"
    echo ""
    echo "Running the VM:"
    echo "  ./scripts/run-vm-qemu.sh                              # Auto-detect files"
    echo "  qemu-system-aarch64 -machine virt -kernel KERNEL ...  # Manual"
    echo ""
    echo "UTM (macOS Apple Silicon):"
    echo "  1. New VM → Virtualize → Linux"
    echo "  2. Kernel: pinas-arm64-vm-KERNEL"
    echo "  3. Boot args: boot=LABEL=LIBREELEC disk=LABEL=STORAGE quiet console=ttyAMA0,115200"
    echo "  4. Drive: import .qcow2, interface VirtIO"
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
        --qcow2)
            OUTPUT_QCOW2=true
            OUTPUT_RAW=false
            shift
            ;;
        --raw)
            OUTPUT_RAW=true
            OUTPUT_QCOW2=false
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

echo "=== Building PiNAS for ARM64 VM (Virtual/aarch64) ==="
echo "Project root: $PROJECT_ROOT"
echo "Target: $PROJECT ($ARCH)"
echo "Rust target: $RUST_TARGET"
echo "Build backend: $BUILD_BACKEND"
echo "Build frontend: $BUILD_FRONTEND"
echo "Build LibreELEC: $BUILD_LIBREELEC"
echo "Output QCOW2: $OUTPUT_QCOW2"
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
        musl-tools qemu-utils \
        parted mtools e2fsprogs dosfstools
    echo "    System dependencies installed"
}

# Function to install Rust
install_rust() {
    echo ">>> Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    rustup target add $RUST_TARGET
    echo "    Rust installed"
}

# Function to install Node.js
install_nodejs() {
    echo ">>> Installing Node.js 20..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt-get install -y nodejs
    echo "    Node.js installed"
}

# Function to create disk image from KERNEL and SYSTEM files
create_disk_image() {
    local kernel_file="$1"
    local system_file="$2"
    local output_dir="$3"
    local version="$4"

    local raw_image="${output_dir}/pinas-arm64-vm-${version}.img"
    local qcow2_image="${output_dir}/pinas-arm64-vm-${version}.qcow2"

    echo ">>> Creating disk image..."
    echo "    KERNEL: ${kernel_file}"
    echo "    SYSTEM: ${system_file}"
    echo "    Size: ${DISK_SIZE_MB}MB (boot: ${BOOT_SIZE_MB}MB, storage: ${STORAGE_SIZE_MB}MB)"

    # Create sparse raw image
    dd if=/dev/zero of="${raw_image}" bs=1M count=0 seek=${DISK_SIZE_MB} 2>/dev/null
    echo -e "    ${GREEN}✓${NC} Created sparse image (${DISK_SIZE_MB}MB)"

    # Create partition table
    parted -s "${raw_image}" mklabel msdos
    parted -s "${raw_image}" mkpart primary fat32 1MiB $((BOOT_SIZE_MB + 1))MiB
    parted -s "${raw_image}" mkpart primary ext4 $((BOOT_SIZE_MB + 1))MiB 100%
    parted -s "${raw_image}" set 1 boot on
    echo -e "    ${GREEN}✓${NC} Partition table created"

    # Set up loop device
    LOOP_DEV=$(sudo losetup --find --show --partscan "${raw_image}")
    echo "    Loop device: ${LOOP_DEV}"

    # Wait for partition devices to appear
    sleep 1
    sudo partprobe "${LOOP_DEV}" 2>/dev/null || true
    sleep 1

    BOOT_PART="${LOOP_DEV}p1"
    STORAGE_PART="${LOOP_DEV}p2"

    # Verify partitions exist
    if [ ! -b "$BOOT_PART" ] || [ ! -b "$STORAGE_PART" ]; then
        echo -e "    ${RED}Error: Partition devices not found${NC}"
        sudo losetup -d "${LOOP_DEV}"
        exit 1
    fi

    # Format partitions
    sudo mkfs.vfat -n "LIBREELEC" "${BOOT_PART}"
    sudo mkfs.ext4 -L "STORAGE" -m 0 "${STORAGE_PART}"
    echo -e "    ${GREEN}✓${NC} Partitions formatted"

    # Mount and populate boot partition
    local boot_mount=$(mktemp -d)
    sudo mount "${BOOT_PART}" "${boot_mount}"
    sudo cp "${kernel_file}" "${boot_mount}/KERNEL"
    sudo cp "${system_file}" "${boot_mount}/SYSTEM"
    sudo umount "${boot_mount}"
    rmdir "${boot_mount}"
    echo -e "    ${GREEN}✓${NC} KERNEL and SYSTEM copied to boot partition"

    # Mount and populate storage partition (create resize marker)
    local storage_mount=$(mktemp -d)
    sudo mount "${STORAGE_PART}" "${storage_mount}"
    sudo touch "${storage_mount}/.please_resize_me"
    sudo umount "${storage_mount}"
    rmdir "${storage_mount}"
    echo -e "    ${GREEN}✓${NC} Storage partition initialized"

    # Detach loop device
    sudo losetup -d "${LOOP_DEV}"

    echo -e "    ${GREEN}✓${NC} Raw image: ${raw_image}"

    # Convert to QCOW2 if requested
    if [ "$OUTPUT_QCOW2" = true ]; then
        echo ">>> Converting to QCOW2..."
        qemu-img convert -f raw -O qcow2 "${raw_image}" "${qcow2_image}"
        QCOW2_SIZE=$(ls -lh "${qcow2_image}" | awk '{print $5}')
        echo -e "    ${GREEN}✓${NC} QCOW2 image: ${qcow2_image} (${QCOW2_SIZE})"

        # Generate checksum
        sha256sum "${qcow2_image}" > "${qcow2_image}.sha256"
        echo -e "    ${GREEN}✓${NC} Checksum: ${qcow2_image}.sha256"

        # Remove raw image unless --raw was also specified
        if [ "$OUTPUT_RAW" = false ]; then
            rm -f "${raw_image}"
        fi
    else
        # Generate checksum for raw image
        sha256sum "${raw_image}" > "${raw_image}.sha256"
        echo -e "    ${GREEN}✓${NC} Checksum: ${raw_image}.sha256"
    fi
}

# Check and install system dependencies if missing
if ! command -v gperf >/dev/null 2>&1 || ! command -v musl-gcc >/dev/null 2>&1 \
   || ! command -v qemu-img >/dev/null 2>&1 || ! command -v parted >/dev/null 2>&1; then
    install_system_deps
fi

# Check and install Rust if missing
if ! command -v cargo >/dev/null 2>&1; then
    install_rust
else
    # Ensure musl target is installed
    if ! rustup target list --installed | grep -q "$RUST_TARGET"; then
        echo ">>> Adding Rust musl target..."
        rustup target add $RUST_TARGET
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
[ "$BUILD_LIBREELEC" = true ] && TOTAL_STEPS=$((TOTAL_STEPS + 2))  # +2 for LE build + image creation
CURRENT_STEP=0

# 1. Build backend (static binary with musl)
if [ "$BUILD_BACKEND" = true ]; then
    CURRENT_STEP=$((CURRENT_STEP + 1))
    echo ">>> [${CURRENT_STEP}/${TOTAL_STEPS}] Building backend for aarch64 (static with musl)..."
    cd "${PROJECT_ROOT}/backend"
    cargo build --release --target $RUST_TARGET

    # Verify binary
    BINARY_PATH="target/${RUST_TARGET}/release/pinas"
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

    # Clean node_modules and lock file to avoid cross-platform incompatibilities
    if [ -d "node_modules" ]; then
        echo "    Cleaning node_modules (cross-platform compatibility)..."
        rm -rf node_modules
    fi
    if [ -f "package-lock.json" ]; then
        rm -f package-lock.json
    fi

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
    cp "${PROJECT_ROOT}/backend/target/${RUST_TARGET}/release/pinas" \
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
    echo "=== PiNAS Package Build Complete (ARM64 VM) ==="
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

# 5. Install PiNAS packages and inject Virtual project
CURRENT_STEP=$((CURRENT_STEP + 1))
echo ""
echo ">>> [${CURRENT_STEP}/${TOTAL_STEPS}] Installing PiNAS packages into LibreELEC..."

# Inject Virtual project
rm -rf "${LIBREELEC_DIR}/projects/Virtual"
cp -r "${PROJECT_ROOT}/libreelec/projects/Virtual" "${LIBREELEC_DIR}/projects/"
echo -e "    ${GREEN}✓${NC} Virtual project injected"

# Install PiNAS package
rm -rf "${LIBREELEC_DIR}/packages/pinas"
cp -r "${PACKAGE_DIR}" "${LIBREELEC_DIR}/packages/"

# Install CUPS package (printer sharing)
rm -rf "${LIBREELEC_DIR}/packages/cups"
rm -rf "${LIBREELEC_DIR}/packages/addons/addon-depends/chrome-depends/cups"
cp -r "${PROJECT_ROOT}/libreelec/packages/cups" "${LIBREELEC_DIR}/packages/"
echo -e "    ${GREEN}✓${NC} CUPS package installed"

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

# Always regenerate kernel config to ensure virtio options are correct
echo ""
echo ">>> Generating VM kernel config from RPi5 base..."
"${SCRIPT_DIR}/generate-vm-kernel-config.sh"
# Re-inject the updated Virtual project
cp -r "${PROJECT_ROOT}/libreelec/projects/Virtual" "${LIBREELEC_DIR}/projects/"

# Force kernel rebuild to pick up new config
echo ">>> Cleaning previous kernel build to pick up new config..."
BUILD_PREFIX="${LIBREELEC_DIR}/build.LibreELEC-Virtual.aarch64-"
# Delete kernel build + install
rm -rf ${BUILD_PREFIX}*/build/linux-*
rm -rf ${BUILD_PREFIX}*/install_pkg/linux-*
# Delete kernel-related stamps only (not all stamps to avoid parallel rebuild issues)
find ${BUILD_PREFIX}*/.stamps/ -name "linux-*" -exec rm -rf {} + 2>/dev/null || true
find ${BUILD_PREFIX}*/.stamps/ -name "linux:*" -exec rm -rf {} + 2>/dev/null || true
# Delete image staging (must be rebuilt with new kernel modules)
rm -rf ${BUILD_PREFIX}*/image/
# Delete image stamp to force image recreation
find ${BUILD_PREFIX}*/.stamps/ -name "image" -exec rm -rf {} + 2>/dev/null || true
# Clean target artifacts
rm -f "${LIBREELEC_DIR}/target/LibreELEC-Virtual.aarch64-"*

# 6. Build LibreELEC (without make image — we create the disk image ourselves)
CURRENT_STEP=$((CURRENT_STEP + 1))
echo ""
echo ">>> [${CURRENT_STEP}/${TOTAL_STEPS}] Building LibreELEC + creating VM image..."
echo "    This will take 2-4 hours on first build..."
echo "    Building: PROJECT=$PROJECT ARCH=$ARCH"
echo "    PiNAS version: $VERSION"
echo ""

cd "$LIBREELEC_DIR"

# Build LibreELEC — use 'make image' and handle the result
# If mkimage fails for ARM64 Virtual (no proper bootloader), we'll fall back to manual image creation
PROJECT=$PROJECT ARCH=$ARCH make image 2>&1 | tee "${PROJECT_ROOT}/build-arm64-vm.log" || {
    echo -e "    ${YELLOW}Note: 'make image' had errors (expected for Virtual project)${NC}"
    echo "    Falling back to manual image creation from build artifacts..."
}

# Locate build artifacts
echo ""
echo ">>> Locating build artifacts..."

# Try to find KERNEL and SYSTEM in target directory
KERNEL_FILE=$(ls "${LIBREELEC_DIR}/target/LibreELEC-Virtual.aarch64-"*.kernel 2>/dev/null | head -1)
SYSTEM_FILE=$(ls "${LIBREELEC_DIR}/target/LibreELEC-Virtual.aarch64-"*.system 2>/dev/null | head -1)

# If not in target, try to find them in the build directory
if [ -z "$KERNEL_FILE" ] || [ -z "$SYSTEM_FILE" ]; then
    BUILD_ROOT=$(ls -d "${LIBREELEC_DIR}/build.LibreELEC-Virtual.aarch64-"* 2>/dev/null | head -1)
    if [ -n "$BUILD_ROOT" ]; then
        # Look for kernel Image
        KERNEL_FILE=$(find "$BUILD_ROOT" -name "Image" -path "*/arch/arm64/boot/*" 2>/dev/null | head -1)
        # Look for SYSTEM squashfs
        SYSTEM_FILE=$(find "${LIBREELEC_DIR}/target" -name "*.system" 2>/dev/null | head -1)
    fi
fi

if [ -z "$KERNEL_FILE" ] || [ ! -f "$KERNEL_FILE" ]; then
    echo -e "    ${RED}Error: KERNEL file not found in build output${NC}"
    echo "    Expected: ${LIBREELEC_DIR}/target/LibreELEC-Virtual.aarch64-*.kernel"
    echo "    Check build-arm64-vm.log for errors"
    exit 1
fi

if [ -z "$SYSTEM_FILE" ] || [ ! -f "$SYSTEM_FILE" ]; then
    echo -e "    ${RED}Error: SYSTEM file not found in build output${NC}"
    echo "    Expected: ${LIBREELEC_DIR}/target/LibreELEC-Virtual.aarch64-*.system"
    echo "    Check build-arm64-vm.log for errors"
    exit 1
fi

KERNEL_SIZE=$(ls -lh "$KERNEL_FILE" | awk '{print $5}')
SYSTEM_SIZE_VAL=$(ls -lh "$SYSTEM_FILE" | awk '{print $5}')
echo -e "    ${GREEN}✓${NC} KERNEL: ${KERNEL_FILE} (${KERNEL_SIZE})"
echo -e "    ${GREEN}✓${NC} SYSTEM: ${SYSTEM_FILE} (${SYSTEM_SIZE_VAL})"

# Check if a pre-built image already exists (make image succeeded)
PREBUILT_IMAGE=$(ls "${LIBREELEC_DIR}/target/LibreELEC-Virtual.aarch64-"*.img.gz 2>/dev/null | head -1)

if [ -n "$PREBUILT_IMAGE" ] && [ -f "$PREBUILT_IMAGE" ]; then
    echo -e "    ${GREEN}✓${NC} LibreELEC created image: ${PREBUILT_IMAGE}"
    echo "    Using pre-built image and converting..."

    OUTPUT_DIR="${PROJECT_ROOT}/target"
    mkdir -p "$OUTPUT_DIR"

    # Extract
    RAW_IMAGE="${OUTPUT_DIR}/pinas-arm64-vm-${VERSION}.img"
    gunzip -c "$PREBUILT_IMAGE" > "$RAW_IMAGE"

    if [ "$OUTPUT_QCOW2" = true ]; then
        QCOW2_IMAGE="${OUTPUT_DIR}/pinas-arm64-vm-${VERSION}.qcow2"
        qemu-img convert -f raw -O qcow2 "$RAW_IMAGE" "$QCOW2_IMAGE"
        QCOW2_SIZE=$(ls -lh "${QCOW2_IMAGE}" | awk '{print $5}')
        echo -e "    ${GREEN}✓${NC} QCOW2 image: ${QCOW2_IMAGE} (${QCOW2_SIZE})"
        sha256sum "${QCOW2_IMAGE}" > "${QCOW2_IMAGE}.sha256"
        if [ "$OUTPUT_RAW" = false ]; then
            rm -f "$RAW_IMAGE"
        fi
    else
        sha256sum "${RAW_IMAGE}" > "${RAW_IMAGE}.sha256"
    fi
else
    # Create disk image manually from KERNEL and SYSTEM
    echo ""
    echo ">>> Creating disk image manually..."

    # Check required tools
    for tool in parted mkfs.vfat mkfs.ext4 qemu-img; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            echo -e "    ${RED}Error: Required tool '$tool' not found${NC}"
            echo "    Install with: sudo apt-get install parted mtools e2fsprogs dosfstools qemu-utils"
            exit 1
        fi
    done

    OUTPUT_DIR="${PROJECT_ROOT}/target"
    mkdir -p "$OUTPUT_DIR"

    create_disk_image "$KERNEL_FILE" "$SYSTEM_FILE" "$OUTPUT_DIR" "$VERSION"
fi

# Copy KERNEL separately for direct kernel boot
echo ""
echo ">>> Copying kernel for direct boot..."
KERNEL_OUTPUT="${OUTPUT_DIR}/pinas-arm64-vm-${VERSION}-KERNEL"
cp "$KERNEL_FILE" "$KERNEL_OUTPUT"
echo -e "    ${GREEN}✓${NC} Kernel: ${KERNEL_OUTPUT}"

# Generate QEMU launch script in output directory
LAUNCH_SCRIPT="${OUTPUT_DIR}/run-pinas-vm.sh"
cat > "$LAUNCH_SCRIPT" << 'LAUNCH_EOF'
#!/bin/bash
# Launch PiNAS ARM64 VM with QEMU (direct kernel boot)
# Generated by build-arm64-vm.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KERNEL="${1:-$(ls -t ${SCRIPT_DIR}/*-KERNEL 2>/dev/null | head -1)}"
DISK="${2:-$(ls -t ${SCRIPT_DIR}/*.qcow2 2>/dev/null | head -1)}"

if [ -z "$KERNEL" ] || [ ! -f "$KERNEL" ]; then
    echo "Error: Kernel file not found. Usage: $0 [KERNEL] [DISK]"
    exit 1
fi

if [ -z "$DISK" ] || [ ! -f "$DISK" ]; then
    # Try raw image
    DISK=$(ls -t ${SCRIPT_DIR}/*.img 2>/dev/null | head -1)
    DISK_FORMAT="raw"
fi

if [ -z "$DISK" ] || [ ! -f "$DISK" ]; then
    echo "Error: Disk image not found. Usage: $0 [KERNEL] [DISK]"
    exit 1
fi

# Detect disk format
DISK_FORMAT="${DISK_FORMAT:-qcow2}"
if [[ "$DISK" == *.img ]]; then
    DISK_FORMAT="raw"
fi

RAM="${PINAS_VM_RAM:-2048}"
CPUS="${PINAS_VM_CPUS:-2}"

echo "PiNAS ARM64 VM"
echo "  Kernel: $KERNEL"
echo "  Disk:   $DISK ($DISK_FORMAT)"
echo "  RAM:    ${RAM}MB"
echo "  CPUs:   $CPUS"
echo "  Ports:  3000->3000 (web), 2222->22 (ssh)"
echo ""
echo "Press Ctrl+A X to exit QEMU"
echo ""

qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a72 \
    -smp ${CPUS} -m ${RAM} \
    -kernel "${KERNEL}" \
    -append "boot=LABEL=LIBREELEC disk=LABEL=STORAGE quiet console=ttyAMA0,115200" \
    -drive file="${DISK}",format=${DISK_FORMAT},if=virtio \
    -netdev user,id=net0,hostfwd=tcp::3000-:3000,hostfwd=tcp::2222-:22 \
    -device virtio-net-pci,netdev=net0 \
    -nographic
LAUNCH_EOF
chmod +x "$LAUNCH_SCRIPT"
echo -e "    ${GREEN}✓${NC} Launch script: ${LAUNCH_SCRIPT}"

# Summary
echo ""
echo "=== Build Complete ==="
echo ""
echo -e "${GREEN}Output files:${NC}"
ls -lh "${OUTPUT_DIR}/pinas-arm64-vm-${VERSION}"* 2>/dev/null | while read line; do
    echo "  $line"
done
echo ""
echo "PiNAS version: $VERSION"
echo ""
echo "=== Running the VM ==="
echo ""
echo "# With QEMU (direct kernel boot):"
echo "  ${LAUNCH_SCRIPT}"
echo "  # or: ./scripts/run-vm-qemu.sh"
echo ""
echo "# Access:"
echo "  Web UI:  http://localhost:3000"
echo "  SSH:     ssh -p 2222 root@localhost (password: libreelec)"
echo ""
echo "# UTM (macOS Apple Silicon):"
echo "  1. New VM → Virtualize → Linux"
echo "  2. Kernel: ${KERNEL_OUTPUT}"
echo "  3. Boot args: boot=LABEL=LIBREELEC disk=LABEL=STORAGE quiet console=ttyAMA0,115200"
echo "  4. Drive: import ${OUTPUT_DIR}/pinas-arm64-vm-${VERSION}.qcow2, interface VirtIO"
echo "  5. Network: Shared Network, port forward 3000→3000"
