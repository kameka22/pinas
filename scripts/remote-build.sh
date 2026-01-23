#!/bin/bash
# Remote build script for PiNAS
# Connects to a Linux VM via SSH to build the LibreELEC image
# and copies the result back to the host
#
# Supports both ARM64 (Raspberry Pi) and x86_64 (VM/Synology) builds
# Uses SSH ControlMaster to maintain connection (password asked once)

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CONFIG_FILE="${PROJECT_ROOT}/.vm-config"
TARGET_DIR="${PROJECT_ROOT}/target"

# SSH ControlMaster settings
SSH_CONTROL_DIR="/tmp/pinas-ssh-$$"
SSH_CONTROL_SOCKET="${SSH_CONTROL_DIR}/control"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Parse command line arguments
RESET_CONFIG=false
BUILD_ARGS=""
BUILD_ARCH="arm64"  # Default architecture

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --arch <arch>        Target architecture: arm64 (default) or x86"
    echo "  --new                Reset VM configuration (re-enter IP, user)"
    echo "  --backend-only       Only build the Rust backend"
    echo "  --frontend-only      Only build the SvelteKit frontend"
    echo "  --skip-libreelec     Skip LibreELEC image build"
    echo "  --clean              Clean build directories before building"
    echo "  --vmdk               Convert to VMDK format (x86 only, for VMware/Synology)"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Architectures:"
    echo "  arm64    Raspberry Pi 5 (aarch64) - default"
    echo "  x86      Generic x86_64 (VM, Synology NAS, PC)"
    echo ""
    echo "Examples:"
    echo "  $0                       # Full ARM64 build (default)"
    echo "  $0 --arch x86            # Full x86_64 build"
    echo "  $0 --arch x86 --vmdk     # x86 build with VMDK conversion"
    echo "  $0 --arch x86 --new      # x86 build with VM reconfiguration"
    echo "  $0 --skip-libreelec      # Build PiNAS only (no LibreELEC image)"
    exit 0
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --arch)
            if [ -z "$2" ] || [[ "$2" == --* ]]; then
                echo -e "${RED}Error: --arch requires an argument (arm64 or x86)${NC}"
                exit 1
            fi
            case "$2" in
                arm64|aarch64)
                    BUILD_ARCH="arm64"
                    ;;
                x86|x86_64|x64)
                    BUILD_ARCH="x86"
                    ;;
                *)
                    echo -e "${RED}Error: Unknown architecture '$2'. Use 'arm64' or 'x86'${NC}"
                    exit 1
                    ;;
            esac
            shift 2
            ;;
        --new)
            RESET_CONFIG=true
            shift
            ;;
        --backend-only|--frontend-only|--skip-libreelec|--clean|--vmdk)
            BUILD_ARGS="$BUILD_ARGS $1"
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

# Set architecture-specific variables
if [ "$BUILD_ARCH" = "arm64" ]; then
    BUILD_SCRIPT="build-arm64.sh"
    IMAGE_PATTERN="LibreELEC-RPi*.aarch64-*.img.gz"
    ARCH_DISPLAY="ARM64 (Raspberry Pi 5)"
else
    BUILD_SCRIPT="build-x86.sh"
    IMAGE_PATTERN="LibreELEC-Generic.x86_64-*.img.gz"
    ARCH_DISPLAY="x86_64 (VM/Synology/PC)"
fi

# Cleanup function
cleanup() {
    if [ -S "$SSH_CONTROL_SOCKET" ]; then
        ssh -o ControlPath="$SSH_CONTROL_SOCKET" -O exit "$VM_USER@$VM_IP" 2>/dev/null || true
    fi
    rm -rf "$SSH_CONTROL_DIR" 2>/dev/null || true
}
trap cleanup EXIT

# Function to load configuration
load_config() {
    if [ -f "$CONFIG_FILE" ]; then
        source "$CONFIG_FILE"
        return 0
    fi
    return 1
}

# Function to save configuration
save_config() {
    cat > "$CONFIG_FILE" << EOF
# VM Configuration for PiNAS remote build
# Generated on $(date)
VM_IP="$VM_IP"
VM_USER="$VM_USER"
EOF
    chmod 600 "$CONFIG_FILE"
    echo -e "${GREEN}Configuration saved to $CONFIG_FILE${NC}"
}

# Function to configure VM connection (does NOT save - call save_config after successful connection)
configure_vm() {
    echo -e "${CYAN}=== VM Configuration ===${NC}"
    echo ""

    local default_ip=""
    local default_user=""

    # Load existing values as defaults if not resetting
    if [ "$RESET_CONFIG" = false ] && load_config; then
        default_ip="$VM_IP"
        default_user="$VM_USER"
    fi

    if [ -n "$default_ip" ]; then
        read -p "VM IP address [$default_ip]: " VM_IP
        VM_IP="${VM_IP:-$default_ip}"
    else
        read -p "VM IP address: " VM_IP
    fi

    if [ -n "$default_user" ]; then
        read -p "VM username [$default_user]: " VM_USER
        VM_USER="${VM_USER:-$default_user}"
    else
        read -p "VM username: " VM_USER
    fi

    echo ""
}

# Function to establish SSH connection with ControlMaster
establish_ssh() {
    echo -e "${CYAN}>>> Establishing SSH connection...${NC}"
    echo "    Connecting to $VM_USER@$VM_IP"
    echo "    (Enter password when prompted)"
    echo ""

    # Create control directory
    mkdir -p "$SSH_CONTROL_DIR"
    chmod 700 "$SSH_CONTROL_DIR"

    # Establish master connection (will prompt for password)
    ssh -o ControlMaster=yes \
        -o ControlPath="$SSH_CONTROL_SOCKET" \
        -o ControlPersist=600 \
        -o StrictHostKeyChecking=no \
        -o UserKnownHostsFile=/dev/null \
        "$VM_USER@$VM_IP" "echo 'Connection established successfully'"

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ SSH connection established${NC}"
        return 0
    else
        echo -e "${RED}✗ SSH connection failed${NC}"
        echo ""
        echo -e "${YELLOW}Tip: Make sure you run this script from your host machine,${NC}"
        echo -e "${YELLOW}not from inside a container or workspace environment.${NC}"
        return 1
    fi
}

# Function to run command on VM (uses existing connection)
run_remote() {
    local cmd="$1"
    ssh -o ControlPath="$SSH_CONTROL_SOCKET" "$VM_USER@$VM_IP" "$cmd"
}

# Function to run command on VM with TTY (for interactive output)
run_remote_tty() {
    local cmd="$1"
    ssh -t -o ControlPath="$SSH_CONTROL_SOCKET" "$VM_USER@$VM_IP" "$cmd"
}

# Function to copy file from VM
copy_from_remote() {
    local remote_path="$1"
    local local_path="$2"
    scp -o ControlPath="$SSH_CONTROL_SOCKET" "$VM_USER@$VM_IP:$remote_path" "$local_path"
}

echo ""
echo -e "${CYAN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║              PiNAS Remote Build for LibreELEC             ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "Target architecture: ${GREEN}$ARCH_DISPLAY${NC}"
echo -e "Build script: ${GREEN}$BUILD_SCRIPT${NC}"
echo ""

# Track if we need to save config after successful connection
NEED_SAVE_CONFIG=false

# Load or configure VM connection
if [ "$RESET_CONFIG" = true ] || ! load_config; then
    configure_vm
    NEED_SAVE_CONFIG=true
else
    echo -e "Using saved configuration: ${GREEN}$VM_USER@$VM_IP${NC}"
    echo -e "Use ${YELLOW}--new${NC} to reconfigure"
    echo ""
fi

# Establish SSH connection (will prompt for password)
if ! establish_ssh; then
    echo ""
    echo "Would you like to reconfigure? (y/n)"
    read -r answer
    if [ "$answer" = "y" ]; then
        configure_vm
        NEED_SAVE_CONFIG=true
        if ! establish_ssh; then
            echo -e "${RED}Connection still failing. Please check your VM settings.${NC}"
            exit 1
        fi
    else
        exit 1
    fi
fi

# Save config only after successful connection
if [ "$NEED_SAVE_CONFIG" = true ]; then
    save_config
fi

# Remote paths
REMOTE_PROJECT_DIR="/home/$VM_USER/pinas"
REMOTE_SCRIPTS_DIR="$REMOTE_PROJECT_DIR/scripts"
REMOTE_TARGET_DIR="$REMOTE_PROJECT_DIR/extra/LibreELEC.tv/target"

echo ""
echo -e "${CYAN}>>> Cleaning local target directory...${NC}"
if [ -d "$TARGET_DIR" ]; then
    rm -rf "$TARGET_DIR"/*.img.gz "$TARGET_DIR"/*.img "$TARGET_DIR"/*.tar 2>/dev/null || true
    echo -e "    ${GREEN}✓${NC} Local target cleaned"
else
    mkdir -p "$TARGET_DIR"
    echo -e "    ${GREEN}✓${NC} Local target directory created"
fi

echo ""
echo -e "${CYAN}>>> Preparing remote environment...${NC}"

# Step 1: Reset any local changes
echo -n "    [1/4] Resetting git repository... "
run_remote "cd $REMOTE_PROJECT_DIR && git reset --hard HEAD >/dev/null 2>&1"
echo -e "${GREEN}done${NC}"

# Step 2: Remove package-lock.json
echo -n "    [2/4] Removing frontend/package-lock.json... "
run_remote "rm -f $REMOTE_PROJECT_DIR/frontend/package-lock.json"
echo -e "${GREEN}done${NC}"

# Step 3: Pull latest changes
echo "    [3/4] Pulling latest changes..."
run_remote "cd $REMOTE_PROJECT_DIR && git pull"

# Step 4: Clean previous builds
echo -n "    [4/4] Cleaning previous builds in target/... "
run_remote "rm -rf $REMOTE_TARGET_DIR/*.img.gz $REMOTE_TARGET_DIR/*.img $REMOTE_TARGET_DIR/*.tar 2>/dev/null || true"
echo -e "${GREEN}done${NC}"

echo ""
echo -e "${GREEN}✓ Remote environment ready${NC}"
echo ""

# Step 4: Run the build script
echo -e "${CYAN}>>> Starting remote build...${NC}"
echo "    This may take several hours for a full LibreELEC build."
echo "    Build arguments: ${BUILD_ARGS:-'(full build)'}"
echo ""

# Run build with live output (TTY for colors and progress)
run_remote_tty "cd $REMOTE_SCRIPTS_DIR && bash $BUILD_SCRIPT $BUILD_ARGS"

BUILD_EXIT_CODE=$?

if [ $BUILD_EXIT_CODE -ne 0 ]; then
    echo -e "${RED}Build failed with exit code $BUILD_EXIT_CODE${NC}"
    exit $BUILD_EXIT_CODE
fi

echo ""
echo -e "${CYAN}>>> Checking for built images...${NC}"

# Step 5: Find and copy the image (using architecture-specific pattern)
IMAGE_NAME=$(run_remote "ls -1 $REMOTE_TARGET_DIR/$IMAGE_PATTERN 2>/dev/null | head -1 | xargs -r basename" 2>/dev/null || echo "")

# Also check for VMDK file if --vmdk was used
VMDK_NAME=$(run_remote "ls -1 $REMOTE_TARGET_DIR/*.vmdk 2>/dev/null | head -1 | xargs -r basename" 2>/dev/null || echo "")

if [ -z "$IMAGE_NAME" ] && [ -z "$VMDK_NAME" ]; then
    echo -e "${YELLOW}No image file found in target directory.${NC}"
    echo "This is normal if you used --skip-libreelec"
    exit 0
fi

# Create local target directory
mkdir -p "$TARGET_DIR"

# Copy VMDK if exists (preferred for x86)
if [ -n "$VMDK_NAME" ]; then
    echo -e "${GREEN}Found VMDK: $VMDK_NAME${NC}"
    echo ""
    echo -e "${CYAN}>>> Copying VMDK to local machine...${NC}"
    echo "    Source: $VM_USER@$VM_IP:$REMOTE_TARGET_DIR/$VMDK_NAME"
    echo "    Destination: $TARGET_DIR/$VMDK_NAME"
    echo ""

    copy_from_remote "$REMOTE_TARGET_DIR/$VMDK_NAME" "$TARGET_DIR/$VMDK_NAME"

    if [ -f "$TARGET_DIR/$VMDK_NAME" ]; then
        VMDK_SIZE=$(ls -lh "$TARGET_DIR/$VMDK_NAME" | awk '{print $5}')
        echo -n "    Cleaning up remote VMDK... "
        run_remote "rm -f $REMOTE_TARGET_DIR/$VMDK_NAME"
        echo -e "${GREEN}done${NC}"
    fi
fi

# Copy raw image if exists
if [ -n "$IMAGE_NAME" ]; then
    echo -e "${GREEN}Found image: $IMAGE_NAME${NC}"
    echo ""
    echo -e "${CYAN}>>> Copying image to local machine...${NC}"
    echo "    Source: $VM_USER@$VM_IP:$REMOTE_TARGET_DIR/$IMAGE_NAME"
    echo "    Destination: $TARGET_DIR/$IMAGE_NAME"
    echo ""

    copy_from_remote "$REMOTE_TARGET_DIR/$IMAGE_NAME" "$TARGET_DIR/$IMAGE_NAME"
fi

# Verify copy and show results
if [ -n "$VMDK_NAME" ] && [ -f "$TARGET_DIR/$VMDK_NAME" ]; then
    IMAGE_SIZE=$(ls -lh "$TARGET_DIR/$VMDK_NAME" | awk '{print $5}')
    FINAL_IMAGE="$TARGET_DIR/$VMDK_NAME"
    IS_VMDK=true

    # Delete raw image on VM if VMDK was created
    if [ -n "$IMAGE_NAME" ]; then
        echo -n "    Cleaning up remote image... "
        run_remote "rm -f $REMOTE_TARGET_DIR/$IMAGE_NAME"
        echo -e "${GREEN}done${NC}"
    fi
elif [ -n "$IMAGE_NAME" ] && [ -f "$TARGET_DIR/$IMAGE_NAME" ]; then
    IMAGE_SIZE=$(ls -lh "$TARGET_DIR/$IMAGE_NAME" | awk '{print $5}')
    FINAL_IMAGE="$TARGET_DIR/$IMAGE_NAME"
    IS_VMDK=false

    # Delete image on VM after successful copy
    echo -n "    Cleaning up remote image... "
    run_remote "rm -f $REMOTE_TARGET_DIR/$IMAGE_NAME"
    echo -e "${GREEN}done${NC}"
else
    echo -e "${RED}Error: Failed to copy image${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                    Build Complete!                        ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "Architecture: ${CYAN}$ARCH_DISPLAY${NC}"
echo -e "Image: ${CYAN}$FINAL_IMAGE${NC}"
echo -e "Size:  ${CYAN}$IMAGE_SIZE${NC}"
echo ""

if [ "$BUILD_ARCH" = "arm64" ]; then
    echo "To flash to SD card (Raspberry Pi):"
    echo -e "  ${YELLOW}gunzip -c $FINAL_IMAGE | sudo dd of=/dev/sdX bs=4M status=progress conv=fsync${NC}"
elif [ "$IS_VMDK" = true ]; then
    echo -e "${GREEN}VMDK file ready to use with VMware or Synology VMM${NC}"
    echo ""
    echo "To import in Synology VMM:"
    echo "  1. Copy $FINAL_IMAGE to your NAS"
    echo "  2. VMM > Image > Create > Import from hard disk"
else
    echo "To use with VMs, first extract and convert:"
    echo ""
    echo "  # Extract image"
    echo -e "  ${YELLOW}gunzip -k $FINAL_IMAGE${NC}"
    echo ""
    echo "  # Convert to VMDK (VMware, Synology VMM):"
    echo -e "  ${YELLOW}qemu-img convert -f raw -O vmdk ${FINAL_IMAGE%.gz} pinas-x86.vmdk${NC}"
    echo ""
    echo "  # Convert to VDI (VirtualBox):"
    echo -e "  ${YELLOW}qemu-img convert -f raw -O vdi ${FINAL_IMAGE%.gz} pinas-x86.vdi${NC}"
    echo ""
    echo "  # Convert to QCOW2 (Proxmox, KVM):"
    echo -e "  ${YELLOW}qemu-img convert -f raw -O qcow2 ${FINAL_IMAGE%.gz} pinas-x86.qcow2${NC}"
fi
echo ""
