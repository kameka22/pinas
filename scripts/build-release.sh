#!/bin/bash
#
# Build a PiNAS update release archive via remote VM
# Connects to a Linux VM via SSH to build backend (native ARM64) + frontend
# then generates the update archive and copies it back
#
# Uses the same VM config as remote-build.sh (.vm-config)
#
# Usage:
#   ./scripts/build-release.sh                  # minor (backend + frontend)
#   ./scripts/build-release.sh --full           # major (+ scripts, services, system)
#   ./scripts/build-release.sh --frontend-only  # patch (frontend only)
#   ./scripts/build-release.sh --changelog "Fixed bugs"
#   ./scripts/build-release.sh --new            # reconfigure VM

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CONFIG_FILE="${PROJECT_ROOT}/.vm-config"

# SSH ControlMaster settings
SSH_CONTROL_DIR="/tmp/pinas-release-$$"
SSH_CONTROL_SOCKET="${SSH_CONTROL_DIR}/control"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Parse arguments
MODE="minor"
CHANGELOG_EN=""
CHANGELOG_FR=""
MIN_VERSION=""
RESET_CONFIG=false

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --full               Major update (backend + frontend + scripts + services + system)"
    echo "  --frontend-only      Patch update (frontend only)"
    echo "  --changelog \"text\"    Changelog in English"
    echo "  --changelog-fr \"text\" Changelog in French"
    echo "  --min-version \"0.01\" Minimum version required"
    echo "  --new                Reset VM configuration"
    echo "  -h, --help           Show this help"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Minor update (backend + frontend)"
    echo "  $0 --frontend-only --changelog \"UI fix\" # Quick patch"
    echo "  $0 --full --changelog \"Major release\"   # Full update with system files"
    exit 0
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --full)
            MODE="major"
            shift
            ;;
        --frontend-only)
            MODE="patch"
            shift
            ;;
        --changelog)
            CHANGELOG_EN="$2"
            shift 2
            ;;
        --changelog-fr)
            CHANGELOG_FR="$2"
            shift 2
            ;;
        --min-version)
            MIN_VERSION="$2"
            shift 2
            ;;
        --new)
            RESET_CONFIG=true
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

# Cleanup function
cleanup() {
    if [ -S "$SSH_CONTROL_SOCKET" ]; then
        ssh -o ControlPath="$SSH_CONTROL_SOCKET" -O exit "$VM_USER@$VM_IP" 2>/dev/null || true
    fi
    rm -rf "$SSH_CONTROL_DIR" 2>/dev/null || true
}
trap cleanup EXIT

# Load/save VM config (shared with remote-build.sh)
load_config() {
    if [ -f "$CONFIG_FILE" ]; then
        source "$CONFIG_FILE"
        return 0
    fi
    return 1
}

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

configure_vm() {
    echo -e "${CYAN}=== VM Configuration ===${NC}"
    echo ""
    local default_ip=""
    local default_user=""

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

establish_ssh() {
    echo -e "${CYAN}>>> Establishing SSH connection...${NC}"
    mkdir -p "$SSH_CONTROL_DIR"
    chmod 700 "$SSH_CONTROL_DIR"

    ssh -o ControlMaster=yes \
        -o ControlPath="$SSH_CONTROL_SOCKET" \
        -o ControlPersist=600 \
        -o StrictHostKeyChecking=no \
        -o UserKnownHostsFile=/dev/null \
        "$VM_USER@$VM_IP" "echo 'Connection established'"

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}    SSH connection established${NC}"
        return 0
    else
        echo -e "${RED}    SSH connection failed${NC}"
        return 1
    fi
}

run_remote() {
    ssh -o ControlPath="$SSH_CONTROL_SOCKET" "$VM_USER@$VM_IP" "$1"
}

run_remote_tty() {
    ssh -t -o ControlPath="$SSH_CONTROL_SOCKET" "$VM_USER@$VM_IP" "$1"
}

copy_from_remote() {
    scp -o ControlPath="$SSH_CONTROL_SOCKET" "$VM_USER@$VM_IP:$1" "$2"
}

# ──────────────────────────────────────────────────────────────
# Start
# ──────────────────────────────────────────────────────────────

echo ""
echo -e "${CYAN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║            PiNAS Release Builder (Remote VM)             ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "Mode: ${GREEN}$MODE${NC}"

NEED_SAVE_CONFIG=false

if [ "$RESET_CONFIG" = true ] || ! load_config; then
    configure_vm
    NEED_SAVE_CONFIG=true
else
    echo -e "VM: ${GREEN}$VM_USER@$VM_IP${NC}  (use --new to reconfigure)"
fi
echo ""

# Connect
if ! establish_ssh; then
    echo "Would you like to reconfigure? (y/n)"
    read -r answer
    if [ "$answer" = "y" ]; then
        configure_vm
        NEED_SAVE_CONFIG=true
        if ! establish_ssh; then
            echo -e "${RED}Connection failed. Check your VM settings.${NC}"
            exit 1
        fi
    else
        exit 1
    fi
fi

if [ "$NEED_SAVE_CONFIG" = true ]; then
    save_config
fi

REMOTE_PROJECT="/home/$VM_USER/pinas"

# ──────────────────────────────────────────────────────────────
# Step 1: Prepare remote environment
# ──────────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}>>> [1/4] Preparing remote environment...${NC}"

echo -n "    Resetting git... "
run_remote "cd $REMOTE_PROJECT && git reset --hard HEAD >/dev/null 2>&1"
echo -e "${GREEN}done${NC}"

echo -n "    Removing frontend/package-lock.json... "
run_remote "rm -f $REMOTE_PROJECT/frontend/package-lock.json"
echo -e "${GREEN}done${NC}"

echo "    Pulling latest changes..."
run_remote "cd $REMOTE_PROJECT && git pull"

echo -e "${GREEN}    Remote environment ready${NC}"

# Read VERSION from remote
VERSION=$(run_remote "cat $REMOTE_PROJECT/VERSION | tr -d '[:space:]'")
if [ -z "$VERSION" ]; then
    echo -e "${RED}Error: VERSION file empty or missing on remote${NC}"
    exit 1
fi
echo -e "    Version: ${GREEN}$VERSION${NC}"

# ──────────────────────────────────────────────────────────────
# Step 2: Build on remote VM
# ──────────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}>>> [2/4] Building on remote VM...${NC}"

# Determine what to build
INCLUDE_BACKEND=false
INCLUDE_FRONTEND=false
INCLUDE_MIGRATIONS=false
INCLUDE_SCRIPTS=false
INCLUDE_SERVICES=false
INCLUDE_SYSTEM=false
REBOOT_REQUIRED=false

case $MODE in
    patch)
        INCLUDE_FRONTEND=true
        ;;
    minor)
        INCLUDE_BACKEND=true
        INCLUDE_FRONTEND=true
        INCLUDE_MIGRATIONS=true
        ;;
    major)
        INCLUDE_BACKEND=true
        INCLUDE_FRONTEND=true
        INCLUDE_MIGRATIONS=true
        INCLUDE_SCRIPTS=true
        INCLUDE_SERVICES=true
        INCLUDE_SYSTEM=true
        REBOOT_REQUIRED=true
        ;;
esac

# Build the release on the VM
REMOTE_BUILD_DIR="$REMOTE_PROJECT/build/release"

# Clean remote build dir
run_remote "rm -rf $REMOTE_BUILD_DIR && mkdir -p $REMOTE_BUILD_DIR"

# Build backend
if [ "$INCLUDE_BACKEND" = true ]; then
    echo ""
    echo -e "    ${CYAN}--- Building backend (aarch64-musl) ---${NC}"
    run_remote_tty "cd $REMOTE_PROJECT/backend && \
        if ! rustup target list --installed | grep -q aarch64-unknown-linux-musl; then \
            rustup target add aarch64-unknown-linux-musl; \
        fi && \
        cargo build --release --target aarch64-unknown-linux-musl"

    run_remote "cp $REMOTE_PROJECT/backend/target/aarch64-unknown-linux-musl/release/pinas $REMOTE_BUILD_DIR/pinas && \
        chmod 755 $REMOTE_BUILD_DIR/pinas"

    BINARY_SIZE=$(run_remote "ls -lh $REMOTE_BUILD_DIR/pinas | awk '{print \$5}'")
    echo -e "    ${GREEN}Backend binary: $BINARY_SIZE${NC}"
fi

# Build frontend
if [ "$INCLUDE_FRONTEND" = true ]; then
    echo ""
    echo -e "    ${CYAN}--- Building frontend (SSG) ---${NC}"
    run_remote_tty "cd $REMOTE_PROJECT/frontend && \
        rm -rf node_modules package-lock.json && \
        npm install --silent && \
        npm run build"

    run_remote "mkdir -p $REMOTE_BUILD_DIR/www && \
        cp -r $REMOTE_PROJECT/frontend/build/. $REMOTE_BUILD_DIR/www/"

    WWW_SIZE=$(run_remote "du -sh $REMOTE_BUILD_DIR/www | awk '{print \$1}'")
    echo -e "    ${GREEN}Frontend: $WWW_SIZE${NC}"
fi

# Copy migrations
if [ "$INCLUDE_MIGRATIONS" = true ]; then
    echo ""
    echo -e "    ${CYAN}--- Copying migrations ---${NC}"
    run_remote "mkdir -p $REMOTE_BUILD_DIR/migrations && \
        cp $REMOTE_PROJECT/backend/migrations/*.sql $REMOTE_BUILD_DIR/migrations/"
    MIG_COUNT=$(run_remote "ls $REMOTE_BUILD_DIR/migrations/ | wc -l | tr -d ' '")
    echo -e "    ${GREEN}Migrations: $MIG_COUNT files${NC}"
fi

# Copy scripts
if [ "$INCLUDE_SCRIPTS" = true ]; then
    echo ""
    echo -e "    ${CYAN}--- Copying scripts ---${NC}"
    run_remote "mkdir -p $REMOTE_BUILD_DIR/scripts && \
        cp $REMOTE_PROJECT/libreelec/packages/pinas/bin/*.sh $REMOTE_BUILD_DIR/scripts/ 2>/dev/null || true && \
        chmod +x $REMOTE_BUILD_DIR/scripts/*.sh 2>/dev/null || true"
fi

# Copy services
if [ "$INCLUDE_SERVICES" = true ]; then
    echo ""
    echo -e "    ${CYAN}--- Copying services ---${NC}"
    run_remote "mkdir -p $REMOTE_BUILD_DIR/services && \
        cp $REMOTE_PROJECT/libreelec/packages/pinas/system.d/*.service $REMOTE_BUILD_DIR/services/ 2>/dev/null || true"
fi

# Copy system files
if [ "$INCLUDE_SYSTEM" = true ]; then
    echo ""
    echo -e "    ${CYAN}--- Copying system files ---${NC}"
    run_remote "mkdir -p $REMOTE_BUILD_DIR/system && \
        LIBREELEC_BUILD=$REMOTE_PROJECT/extra/LibreELEC.tv/target && \
        if [ -f \$LIBREELEC_BUILD/SYSTEM ]; then cp \$LIBREELEC_BUILD/SYSTEM $REMOTE_BUILD_DIR/system/; fi && \
        if [ -f \$LIBREELEC_BUILD/KERNEL ]; then cp \$LIBREELEC_BUILD/KERNEL $REMOTE_BUILD_DIR/system/; fi"
fi

# ──────────────────────────────────────────────────────────────
# Step 3: Generate update.json + archive on remote
# ──────────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}>>> [3/4] Generating update archive on remote...${NC}"

DATE=$(date +%Y-%m-%d)

if [ -z "$CHANGELOG_EN" ]; then
    CHANGELOG_EN="PiNAS update to version $VERSION"
fi
if [ -z "$CHANGELOG_FR" ]; then
    CHANGELOG_FR="Mise à jour de PiNAS vers la version $VERSION"
fi

MIN_VERSION_LINE=""
if [ -n "$MIN_VERSION" ]; then
    MIN_VERSION_LINE="\"min_version\": \"$MIN_VERSION\","
fi

# Generate update.json on remote
run_remote "cat > $REMOTE_BUILD_DIR/update.json << 'ENDJSON'
{
  \"version\": \"$VERSION\",
  $MIN_VERSION_LINE
  \"date\": \"$DATE\",
  \"type\": \"$MODE\",
  \"reboot_required\": $REBOOT_REQUIRED,
  \"changelog\": {
    \"en\": $(printf '%s' "$CHANGELOG_EN" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read().strip()))"),
    \"fr\": $(printf '%s' "$CHANGELOG_FR" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read().strip()))")
  },
  \"contents\": {
    \"backend\": $INCLUDE_BACKEND,
    \"frontend\": $INCLUDE_FRONTEND,
    \"migrations\": $INCLUDE_MIGRATIONS,
    \"scripts\": $INCLUDE_SCRIPTS,
    \"services\": $INCLUDE_SERVICES,
    \"system\": $INCLUDE_SYSTEM
  }
}
ENDJSON"

# Create archive on remote
ARCHIVE_NAME="pinas-update-v${VERSION}.tar.gz"
run_remote "cd $REMOTE_BUILD_DIR && tar -czf $REMOTE_PROJECT/build/$ARCHIVE_NAME ."

ARCHIVE_SIZE=$(run_remote "ls -lh $REMOTE_PROJECT/build/$ARCHIVE_NAME | awk '{print \$5}'")
ARCHIVE_SHA=$(run_remote "sha256sum $REMOTE_PROJECT/build/$ARCHIVE_NAME | awk '{print \$1}'")

echo -e "    ${GREEN}Archive: $ARCHIVE_NAME ($ARCHIVE_SIZE)${NC}"
echo -e "    SHA256: $ARCHIVE_SHA"

# ──────────────────────────────────────────────────────────────
# Step 4: Copy archive back to local machine
# ──────────────────────────────────────────────────────────────
echo ""
echo -e "${CYAN}>>> [4/4] Copying archive to local machine...${NC}"

mkdir -p "$PROJECT_ROOT/build"
copy_from_remote "$REMOTE_PROJECT/build/$ARCHIVE_NAME" "$PROJECT_ROOT/build/$ARCHIVE_NAME"

if [ ! -f "$PROJECT_ROOT/build/$ARCHIVE_NAME" ]; then
    echo -e "${RED}Error: Failed to copy archive${NC}"
    exit 1
fi

# Clean up remote build
run_remote "rm -rf $REMOTE_BUILD_DIR $REMOTE_PROJECT/build/$ARCHIVE_NAME"

LOCAL_SIZE=$(ls -lh "$PROJECT_ROOT/build/$ARCHIVE_NAME" | awk '{print $5}')

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                  Release Build Complete!                  ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Version:  ${CYAN}$VERSION${NC}"
echo -e "  Mode:     ${CYAN}$MODE${NC}"
echo -e "  Archive:  ${CYAN}build/$ARCHIVE_NAME${NC}"
echo -e "  Size:     ${CYAN}$LOCAL_SIZE${NC}"
echo -e "  SHA256:   ${CYAN}$ARCHIVE_SHA${NC}"
echo ""
echo "To create a GitHub release:"
echo -e "  ${YELLOW}gh release create v$VERSION build/$ARCHIVE_NAME --title \"PiNAS v$VERSION\" --notes \"$CHANGELOG_EN\"${NC}"
echo ""
