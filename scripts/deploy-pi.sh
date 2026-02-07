#!/bin/bash
# PiNAS - Hot Deploy to Raspberry Pi
# Builds on remote VM and deploys directly to Pi via SSH
#
# Prerequisites:
#   - Code committed and pushed to git
#   - VM configured (reuses .vm-config from remote-build.sh)
#   - Pi on the local network with SSH enabled
#
# Usage:
#   ./deploy-pi.sh              # Build all + deploy all
#   ./deploy-pi.sh --backend    # Build + deploy backend only
#   ./deploy-pi.sh --frontend   # Build + deploy frontend only
#   ./deploy-pi.sh --scripts    # Deploy scripts + services only (no build)
#   ./deploy-pi.sh --restart    # Just restart pinas service on Pi
#   ./deploy-pi.sh --logs       # Show live logs from Pi
#   ./deploy-pi.sh --revert     # Remove overrides, revert to image version
#   ./deploy-pi.sh --new        # Reconfigure VM/Pi connection

set -e

# ── Configuration ────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VM_CONFIG_FILE="${PROJECT_ROOT}/.vm-config"
PI_CONFIG_FILE="${PROJECT_ROOT}/.pi-deploy.conf"

# SSH ControlMaster settings
SSH_CONTROL_DIR="/tmp/pinas-deploy-$$"
VM_SSH_SOCKET="${SSH_CONTROL_DIR}/vm-control"
PI_SSH_SOCKET="${SSH_CONTROL_DIR}/pi-control"

# Pi defaults
PI_USER="root"
PI_PASS="libreelec"
PI_PORT=22

# Remote paths
PI_BIN_DIR="/storage/.pinas/bin"
PI_WWW_DIR="/storage/.pinas/www"
PI_SYSTEMD_DIR="/storage/.config/system.d"

# Pi discovery
MDNS_NAMES=("libreelec.local" "pinas.local")
PI_MAC_PREFIXES=("b8:27:eb" "dc:a6:32" "e4:5f:01" "d8:3a:dd" "2c:cf:67")
SCAN_TIMEOUT=3

# ── Colors & Symbols ────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

CHECK="${GREEN}✔${NC}"
CROSS="${RED}✘${NC}"
ARROW="${CYAN}➜${NC}"
PI_ICON="${MAGENTA}🍓${NC}"

# ── Parse arguments ──────────────────────────────────────────────
MODE="all"  # all, backend, frontend, scripts, restart, logs, revert
RESET_CONFIG=false

usage() {
    echo ""
    echo -e "  ${BOLD}PiNAS Deploy${NC} - Hot deploy to Raspberry Pi"
    echo ""
    echo "  Usage: $0 [OPTIONS]"
    echo ""
    echo "  Build & Deploy:"
    echo "    (no flags)       Build all + deploy all"
    echo "    --backend        Build + deploy backend only"
    echo "    --frontend       Build + deploy frontend only"
    echo "    --scripts        Deploy scripts + services only (no build)"
    echo ""
    echo "  Pi Control:"
    echo "    --restart        Restart pinas service on Pi"
    echo "    --logs           Show live logs from Pi"
    echo "    --revert         Remove overrides, revert to image version"
    echo ""
    echo "  Config:"
    echo "    --new            Reconfigure VM and/or Pi connection"
    echo "    -h, --help       Show this help"
    echo ""
    exit 0
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --backend)   MODE="backend"; shift ;;
        --frontend)  MODE="frontend"; shift ;;
        --scripts)   MODE="scripts"; shift ;;
        --restart)   MODE="restart"; shift ;;
        --logs)      MODE="logs"; shift ;;
        --revert)    MODE="revert"; shift ;;
        --new)       RESET_CONFIG=true; shift ;;
        -h|--help)   usage ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            ;;
    esac
done

# ── Cleanup ──────────────────────────────────────────────────────
cleanup() {
    if [ -S "$VM_SSH_SOCKET" ]; then
        ssh -o ControlPath="$VM_SSH_SOCKET" -O exit "$VM_USER@$VM_IP" 2>/dev/null || true
    fi
    if [ -S "$PI_SSH_SOCKET" ]; then
        ssh -o ControlPath="$PI_SSH_SOCKET" -O exit "${PI_USER}@${PI_HOST}" 2>/dev/null || true
    fi
    rm -rf "$SSH_CONTROL_DIR" 2>/dev/null || true
}
trap cleanup EXIT

# ── Banner ───────────────────────────────────────────────────────
banner() {
    echo ""
    echo -e "  ${MAGENTA}${BOLD}┌───────────────────────────────────────┐${NC}"
    echo -e "  ${MAGENTA}${BOLD}│${NC}  ${PI_ICON} ${BOLD}PiNAS Deploy${NC}                       ${MAGENTA}${BOLD}│${NC}"
    echo -e "  ${MAGENTA}${BOLD}│${NC}  ${DIM}Build on VM → Deploy to Pi${NC}            ${MAGENTA}${BOLD}│${NC}"
    echo -e "  ${MAGENTA}${BOLD}└───────────────────────────────────────┘${NC}"
    echo ""
    echo -e "  Mode: ${CYAN}${BOLD}${MODE}${NC}"
    echo ""
}

# ── VM Config (reuse .vm-config from remote-build.sh) ───────────
load_vm_config() {
    if [ -f "$VM_CONFIG_FILE" ]; then
        source "$VM_CONFIG_FILE"
        return 0
    fi
    return 1
}

configure_vm() {
    echo -e "  ${BOLD}VM Configuration${NC}"

    local default_ip="${VM_IP:-}"
    local default_user="${VM_USER:-}"

    if [ -n "$default_ip" ]; then
        read -p "  VM IP [$default_ip]: " VM_IP
        VM_IP="${VM_IP:-$default_ip}"
    else
        read -p "  VM IP: " VM_IP
    fi

    if [ -n "$default_user" ]; then
        read -p "  VM user [$default_user]: " VM_USER
        VM_USER="${VM_USER:-$default_user}"
    else
        read -p "  VM user: " VM_USER
    fi

    # Save to .vm-config (shared with remote-build.sh)
    cat > "$VM_CONFIG_FILE" << EOF
# VM Configuration for PiNAS
# Generated on $(date)
VM_IP="$VM_IP"
VM_USER="$VM_USER"
EOF
    chmod 600 "$VM_CONFIG_FILE"
    echo -e "  ${CHECK} VM config saved"
    echo ""
}

# ── Pi Config ────────────────────────────────────────────────────
load_pi_config() {
    if [ -f "$PI_CONFIG_FILE" ]; then
        source "$PI_CONFIG_FILE"
        return 0
    fi
    return 1
}

save_pi_config() {
    cat > "$PI_CONFIG_FILE" << EOF
# Pi Configuration for PiNAS deploy
# Generated on $(date)
PI_HOST="$PI_HOST"
PI_USER="$PI_USER"
PI_PASS="$PI_PASS"
EOF
    chmod 600 "$PI_CONFIG_FILE"
}

# ── Pi Discovery (from connect-pi.sh) ───────────────────────────
found_ips=()

discover_pi() {
    echo -e "  ${BOLD}Discovering Pi on network...${NC}"

    # Method 1: mDNS
    echo -ne "    mDNS... "
    for name in "${MDNS_NAMES[@]}"; do
        local ip=""
        if command -v dns-sd &>/dev/null; then
            ip=$(dns-sd -timeout 1 -G v4 "$name" 2>/dev/null | tail -1 | awk '{print $NF}' || true)
            [[ "$ip" =~ ^[0-9]+\. ]] || ip=""
        fi
        if [[ -z "$ip" ]] && command -v avahi-resolve &>/dev/null; then
            ip=$(avahi-resolve -4 -n "$name" 2>/dev/null | awk '{print $2}' || true)
        fi
        if [[ -z "$ip" ]]; then
            ip=$(getent hosts "$name" 2>/dev/null | awk '{print $1}' | head -1 || true)
        fi
        if [[ -z "$ip" ]]; then
            ip=$(ping -c 1 -W 1 "$name" 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+' | head -1 || true)
        fi
        if [[ -n "$ip" ]]; then
            add_pi_ip "$ip"
        fi
    done
    if [[ ${#found_ips[@]} -gt 0 ]]; then
        echo -e "${CHECK}"
    else
        echo -e "${DIM}not found${NC}"
    fi

    # Method 2: ARP
    if [[ ${#found_ips[@]} -eq 0 ]]; then
        echo -ne "    ARP table... "
        local arp_output
        arp_output=$(arp -a 2>/dev/null || true)
        for prefix in "${PI_MAC_PREFIXES[@]}"; do
            while IFS= read -r line; do
                local ip
                ip=$(echo "$line" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+' | head -1)
                if [[ -n "$ip" ]] && ping -c 1 -W 1 "$ip" &>/dev/null; then
                    add_pi_ip "$ip"
                fi
            done <<< "$(echo "$arp_output" | grep -i "$prefix" || true)"
        done
        if [[ ${#found_ips[@]} -gt 0 ]]; then
            echo -e "${CHECK}"
        else
            echo -e "${DIM}not found${NC}"
        fi
    fi

    # Method 3: nmap (if still not found)
    if [[ ${#found_ips[@]} -eq 0 ]] && command -v nmap &>/dev/null; then
        echo -ne "    nmap scan... "
        local subnet
        if command -v ip &>/dev/null; then
            subnet=$(ip -4 route 2>/dev/null | grep "default" | head -1 | awk '{print $3}' | sed 's/\.[0-9]*$/.0\/24/')
        fi
        if [[ -z "$subnet" ]]; then
            local gateway
            gateway=$(route -n get default 2>/dev/null | grep gateway | awk '{print $2}' || true)
            [[ -n "$gateway" ]] && subnet=$(echo "$gateway" | sed 's/\.[0-9]*$/.0\/24/')
        fi
        [[ -z "$subnet" ]] && subnet="192.168.1.0/24"

        local nmap_output ip=""
        nmap_output=$(nmap -sn "$subnet" 2>/dev/null || true)
        while IFS= read -r line; do
            if echo "$line" | grep -q "Nmap scan report for"; then
                ip=$(echo "$line" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+')
            fi
            if echo "$line" | grep -qi "raspberry\|raspberrypi"; then
                [[ -n "$ip" ]] && add_pi_ip "$ip"
            fi
        done <<< "$nmap_output"
        if [[ ${#found_ips[@]} -gt 0 ]]; then
            echo -e "${CHECK}"
        else
            echo -e "${DIM}not found${NC}"
        fi
    fi
}

add_pi_ip() {
    local ip=$1
    for existing in "${found_ips[@]}"; do
        [[ "$existing" == "$ip" ]] && return
    done
    found_ips+=("$ip")
}

verify_pinas() {
    local ip=$1
    curl -s --connect-timeout 3 "http://${ip}:3000/api/health" &>/dev/null
}

resolve_pi_host() {
    # Try saved config first
    if [[ "$RESET_CONFIG" != true ]] && load_pi_config && [[ -n "$PI_HOST" ]]; then
        echo -ne "  Saved Pi: ${BOLD}${PI_HOST}${NC} ... "
        if ping -c 1 -W 2 "$PI_HOST" &>/dev/null; then
            echo -e "${CHECK} reachable"
            return 0
        else
            echo -e "${CROSS} unreachable, rediscovering..."
        fi
    fi

    # Discover
    discover_pi

    if [[ ${#found_ips[@]} -eq 0 ]]; then
        echo ""
        echo -e "  ${CROSS} No Pi found on the network"
        echo -ne "  Enter Pi IP manually: "
        read -r PI_HOST
        if [[ -z "$PI_HOST" ]]; then
            echo -e "  ${RED}No IP provided, aborting${NC}"
            exit 1
        fi
    elif [[ ${#found_ips[@]} -eq 1 ]]; then
        PI_HOST="${found_ips[0]}"
        echo -e "  ${CHECK} Found Pi at ${BOLD}${PI_HOST}${NC}"
    else
        echo ""
        echo -e "  Found ${#found_ips[@]} devices:"
        for i in "${!found_ips[@]}"; do
            local pinas_status=""
            verify_pinas "${found_ips[$i]}" && pinas_status=" ${GREEN}● PiNAS${NC}"
            echo -e "    $((i+1))) ${CYAN}${found_ips[$i]}${NC}${pinas_status}"
        done
        echo -ne "  ${ARROW} Which one? [1-${#found_ips[@]}]: "
        read -r choice
        if [[ "$choice" =~ ^[0-9]+$ ]] && (( choice >= 1 && choice <= ${#found_ips[@]} )); then
            PI_HOST="${found_ips[$((choice-1))]}"
        else
            echo -e "  ${RED}Invalid choice${NC}"
            exit 1
        fi
    fi

    save_pi_config
}

# ── SSH Helpers ──────────────────────────────────────────────────
establish_vm_ssh() {
    echo -e "  ${ARROW} Connecting to VM ${BOLD}${VM_USER}@${VM_IP}${NC}..."
    mkdir -p "$SSH_CONTROL_DIR"
    chmod 700 "$SSH_CONTROL_DIR"

    ssh -o ControlMaster=yes \
        -o ControlPath="$VM_SSH_SOCKET" \
        -o ControlPersist=600 \
        -o StrictHostKeyChecking=no \
        -o UserKnownHostsFile=/dev/null \
        "$VM_USER@$VM_IP" "echo ok" >/dev/null

    echo -e "  ${CHECK} VM connected"
}

establish_pi_ssh() {
    echo -e "  ${ARROW} Connecting to Pi ${BOLD}${PI_USER}@${PI_HOST}${NC}..."
    mkdir -p "$SSH_CONTROL_DIR"
    chmod 700 "$SSH_CONTROL_DIR"

    local ssh_opts=(
        -o ControlMaster=yes
        -o "ControlPath=$PI_SSH_SOCKET"
        -o ControlPersist=600
        -o StrictHostKeyChecking=no
        -o UserKnownHostsFile=/dev/null
        -o LogLevel=ERROR
        -p "$PI_PORT"
    )

    if command -v sshpass &>/dev/null; then
        sshpass -p "$PI_PASS" ssh "${ssh_opts[@]}" "${PI_USER}@${PI_HOST}" "echo ok" >/dev/null
    else
        echo -e "  ${DIM}Password: ${PI_PASS}${NC}"
        ssh "${ssh_opts[@]}" "${PI_USER}@${PI_HOST}" "echo ok" >/dev/null
    fi

    echo -e "  ${CHECK} Pi connected"
}

vm_run() {
    ssh -o ControlPath="$VM_SSH_SOCKET" "$VM_USER@$VM_IP" "$1"
}

vm_run_tty() {
    ssh -t -o ControlPath="$VM_SSH_SOCKET" "$VM_USER@$VM_IP" "$1"
}

pi_run() {
    ssh -o ControlPath="$PI_SSH_SOCKET" -p "$PI_PORT" "${PI_USER}@${PI_HOST}" "$1"
}

pi_run_tty() {
    ssh -t -o ControlPath="$PI_SSH_SOCKET" -p "$PI_PORT" "${PI_USER}@${PI_HOST}" "$1"
}

# SCP from VM to Pi (via VM's ssh to Pi)
vm_to_pi() {
    local remote_path="$1"
    local pi_path="$2"
    # VM sends files directly to Pi
    vm_run "sshpass -p '${PI_PASS}' scp -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -P ${PI_PORT} ${remote_path} ${PI_USER}@${PI_HOST}:${pi_path}"
}

vm_to_pi_dir() {
    local remote_path="$1"
    local pi_path="$2"
    # VM sends directory directly to Pi
    vm_run "sshpass -p '${PI_PASS}' scp -r -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -P ${PI_PORT} ${remote_path} ${PI_USER}@${PI_HOST}:${pi_path}"
}

# ── Build on VM ──────────────────────────────────────────────────
REMOTE_PROJECT_DIR=""

build_backend() {
    echo ""
    echo -e "  ${BOLD}Building backend (aarch64-musl)...${NC}"

    vm_run_tty "source \$HOME/.cargo/env 2>/dev/null; cd ${REMOTE_PROJECT_DIR}/backend && cargo build --release --target aarch64-unknown-linux-musl"

    # Verify static binary
    local file_info
    file_info=$(vm_run "file ${REMOTE_PROJECT_DIR}/backend/target/aarch64-unknown-linux-musl/release/pinas")
    if echo "$file_info" | grep -q "statically linked"; then
        echo -e "  ${CHECK} Backend built (static binary)"
    else
        echo -e "  ${YELLOW}⚠ Binary may not be statically linked${NC}"
        echo -e "  ${DIM}${file_info}${NC}"
    fi
}

build_frontend() {
    echo ""
    echo -e "  ${BOLD}Building frontend (SSG)...${NC}"

    vm_run_tty "export PATH=\$HOME/.cargo/bin:/usr/local/bin:\$PATH; cd ${REMOTE_PROJECT_DIR}/frontend && npm install --no-audit --no-fund && npm run build"

    echo -e "  ${CHECK} Frontend built"
}

# ── Deploy to Pi ─────────────────────────────────────────────────
deploy_backend() {
    echo -ne "    Binary → ${PI_BIN_DIR}/pinas ... "
    pi_run "mkdir -p ${PI_BIN_DIR}"
    vm_to_pi "${REMOTE_PROJECT_DIR}/backend/target/aarch64-unknown-linux-musl/release/pinas" "${PI_BIN_DIR}/pinas"
    pi_run "chmod 755 ${PI_BIN_DIR}/pinas"
    echo -e "${CHECK}"
}

deploy_frontend() {
    echo -ne "    Frontend → ${PI_WWW_DIR}/ ... "
    pi_run "mkdir -p ${PI_WWW_DIR}"
    # Clean old frontend first, then copy new
    pi_run "rm -rf ${PI_WWW_DIR}/*"
    vm_to_pi_dir "${REMOTE_PROJECT_DIR}/frontend/build/." "${PI_WWW_DIR}/"
    echo -e "${CHECK}"
}

deploy_scripts() {
    local src="${REMOTE_PROJECT_DIR}/libreelec/packages/pinas"

    echo -ne "    pinas-init.sh → ${PI_BIN_DIR}/ ... "
    vm_to_pi "${src}/bin/pinas-init.sh" "${PI_BIN_DIR}/pinas-init.sh"
    pi_run "chmod 755 ${PI_BIN_DIR}/pinas-init.sh"
    echo -e "${CHECK}"

    echo -ne "    pinas-resize-storage.sh → ${PI_BIN_DIR}/ ... "
    vm_to_pi "${src}/bin/pinas-resize-storage.sh" "${PI_BIN_DIR}/pinas-resize-storage.sh"
    pi_run "chmod 755 ${PI_BIN_DIR}/pinas-resize-storage.sh"
    echo -e "${CHECK}"

    echo -ne "    pinas-kodi-config.sh → ${PI_BIN_DIR}/ ... "
    vm_to_pi "${src}/bin/pinas-kodi-config.sh" "${PI_BIN_DIR}/pinas-kodi-config.sh"
    pi_run "chmod 755 ${PI_BIN_DIR}/pinas-kodi-config.sh"
    echo -e "${CHECK}"

    # Deploy service files with paths pointing to /storage/.pinas/bin/
    echo -ne "    pinas.service → ${PI_SYSTEMD_DIR}/ ... "
    pi_run "mkdir -p ${PI_SYSTEMD_DIR}"

    # Create modified service that uses /storage/.pinas/bin/ paths
    pi_run "cat > ${PI_SYSTEMD_DIR}/pinas.service << 'SVCEOF'
[Unit]
Description=PiNAS - NAS Management Service
After=network-online.target kodi.service
Wants=network-online.target

[Service]
Type=simple
Environment=PINAS_FILES_ROOT=/storage/.pinas/files
Environment=PINAS_HOMES_ROOT=/storage/.pinas/homes
Environment=PINAS_DATABASE_URL=sqlite:/storage/.pinas/data/pinas.db?mode=rwc
Environment=PINAS_BIND_ADDRESS=0.0.0.0:3000
Environment=PINAS_LOG_LEVEL=info
Environment=PINAS_STATIC_DIR=/storage/.pinas/www
Environment=PINAS_KODI_USERNAME=kodi
Environment=PINAS_KODI_PASSWORD=pinas
ExecStartPre=/storage/.pinas/bin/pinas-init.sh
ExecStart=/storage/.pinas/bin/pinas
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
SVCEOF"
    echo -e "${CHECK}"

    echo -ne "    pinas-resize-storage.service → ${PI_SYSTEMD_DIR}/ ... "
    pi_run "cat > ${PI_SYSTEMD_DIR}/pinas-resize-storage.service << 'SVCEOF'
[Unit]
Description=PiNAS - Auto-resize storage partition
DefaultDependencies=no
Before=pinas.service local-fs-pre.target
After=systemd-remount-fs.service
ConditionPathExists=!/storage/.pinas/.storage-resized

[Service]
Type=oneshot
ExecStart=/storage/.pinas/bin/pinas-resize-storage.sh
RemainAfterExit=yes
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=local-fs-pre.target
SVCEOF"
    echo -e "${CHECK}"

    echo -ne "    pinas-kodi-config.service → ${PI_SYSTEMD_DIR}/ ... "
    pi_run "cat > ${PI_SYSTEMD_DIR}/pinas-kodi-config.service << 'SVCEOF'
[Unit]
Description=PiNAS - Configure Kodi webserver
DefaultDependencies=no
Before=kodi.service
After=systemd-remount-fs.service

[Service]
Type=oneshot
ExecStart=/storage/.pinas/bin/pinas-kodi-config.sh
RemainAfterExit=yes
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
SVCEOF"
    pi_run "systemctl enable ${PI_SYSTEMD_DIR}/pinas-kodi-config.service 2>/dev/null || true"
    echo -e "${CHECK}"
}

restart_service() {
    echo -ne "    systemctl daemon-reload ... "
    pi_run "systemctl daemon-reload"
    echo -e "${CHECK}"

    echo -ne "    systemctl restart pinas ... "
    pi_run "systemctl restart pinas"
    echo -e "${CHECK}"
}

health_check() {
    echo -ne "    Health check ... "
    local retries=5
    local i=0
    while (( i < retries )); do
        sleep 1
        local status
        status=$(curl -s --connect-timeout 3 "http://${PI_HOST}:3000/api/health" 2>/dev/null || true)
        if [[ -n "$status" ]]; then
            echo -e "${CHECK} ${GREEN}${status}${NC}"
            return 0
        fi
        i=$((i + 1))
    done
    echo -e "${CROSS} ${RED}Service not responding after ${retries}s${NC}"
    echo -e "    ${DIM}Check logs with: $0 --logs${NC}"
    return 1
}

# ── Revert ───────────────────────────────────────────────────────
do_revert() {
    echo -e "  ${BOLD}Reverting to image version...${NC}"
    echo ""

    echo -ne "    Remove service overrides... "
    pi_run "rm -f ${PI_SYSTEMD_DIR}/pinas.service ${PI_SYSTEMD_DIR}/pinas-resize-storage.service ${PI_SYSTEMD_DIR}/pinas-kodi-config.service"
    echo -e "${CHECK}"

    echo -ne "    Remove deployed binary... "
    pi_run "rm -f ${PI_BIN_DIR}/pinas"
    echo -e "${CHECK}"

    echo -ne "    Remove deployed scripts... "
    pi_run "rm -f ${PI_BIN_DIR}/pinas-init.sh ${PI_BIN_DIR}/pinas-resize-storage.sh ${PI_BIN_DIR}/pinas-kodi-config.sh"
    echo -e "${CHECK}"

    echo -ne "    Restore frontend from image... "
    pi_run "rm -rf ${PI_WWW_DIR}/* && cp -r /usr/share/pinas/www/. ${PI_WWW_DIR}/ 2>/dev/null || true"
    echo -e "${CHECK}"

    restart_service
    echo ""
    health_check
    echo ""
    echo -e "  ${CHECK} ${GREEN}Reverted to image version${NC}"
}

# ── Main ─────────────────────────────────────────────────────────
main() {
    banner

    # ── Resolve Pi ──
    resolve_pi_host
    echo ""

    # ── Quick modes (Pi only, no VM) ──
    case "$MODE" in
        restart)
            establish_pi_ssh
            echo ""
            echo -e "  ${BOLD}Restarting service...${NC}"
            restart_service
            health_check
            echo ""
            exit 0
            ;;
        logs)
            establish_pi_ssh
            echo ""
            echo -e "  ${BOLD}Live logs (Ctrl+C to stop)${NC}"
            echo ""
            pi_run_tty "journalctl -u pinas -f --no-hostname -o short-iso"
            exit 0
            ;;
        revert)
            establish_pi_ssh
            echo ""
            do_revert
            exit 0
            ;;
    esac

    # ── Modes that need the VM ──
    if [[ "$MODE" != "scripts" ]]; then
        # Load VM config
        if [[ "$RESET_CONFIG" == true ]] || ! load_vm_config; then
            configure_vm
        else
            echo -e "  VM: ${BOLD}${VM_USER}@${VM_IP}${NC}"
        fi

        establish_vm_ssh

        REMOTE_PROJECT_DIR="/home/$VM_USER/pinas"

        # Git pull on VM
        echo ""
        echo -e "  ${BOLD}Updating sources on VM...${NC}"
        echo -ne "    git pull ... "
        vm_run "cd ${REMOTE_PROJECT_DIR} && git pull --ff-only" >/dev/null
        echo -e "${CHECK}"

        # Build
        case "$MODE" in
            all)
                build_backend
                build_frontend
                ;;
            backend)
                build_backend
                ;;
            frontend)
                build_frontend
                ;;
        esac

        # Ensure sshpass is available on VM for VM→Pi transfers
        echo ""
        echo -ne "  Checking VM has sshpass... "
        if vm_run "command -v sshpass" &>/dev/null; then
            echo -e "${CHECK}"
        else
            echo -e "${YELLOW}installing...${NC}"
            vm_run "sudo apt-get install -y sshpass >/dev/null 2>&1"
            echo -e "  ${CHECK} sshpass installed"
        fi
    fi

    # Connect to Pi
    establish_pi_ssh

    # Deploy
    echo ""
    echo -e "  ${BOLD}Deploying to Pi (${PI_HOST})...${NC}"

    case "$MODE" in
        all)
            deploy_backend
            deploy_frontend
            deploy_scripts
            ;;
        backend)
            deploy_backend
            deploy_scripts
            ;;
        frontend)
            deploy_frontend
            ;;
        scripts)
            # scripts-only: copy from local machine instead of VM
            echo -ne "    pinas-init.sh → ${PI_BIN_DIR}/ ... "
            pi_run "mkdir -p ${PI_BIN_DIR}"
            scp -o ControlPath="$PI_SSH_SOCKET" -P "$PI_PORT" \
                "${PROJECT_ROOT}/libreelec/packages/pinas/bin/pinas-init.sh" \
                "${PI_USER}@${PI_HOST}:${PI_BIN_DIR}/pinas-init.sh"
            pi_run "chmod 755 ${PI_BIN_DIR}/pinas-init.sh"
            echo -e "${CHECK}"

            echo -ne "    pinas-resize-storage.sh → ${PI_BIN_DIR}/ ... "
            scp -o ControlPath="$PI_SSH_SOCKET" -P "$PI_PORT" \
                "${PROJECT_ROOT}/libreelec/packages/pinas/bin/pinas-resize-storage.sh" \
                "${PI_USER}@${PI_HOST}:${PI_BIN_DIR}/pinas-resize-storage.sh"
            pi_run "chmod 755 ${PI_BIN_DIR}/pinas-resize-storage.sh"
            echo -e "${CHECK}"

            deploy_scripts
            ;;
    esac

    # Restart & verify
    echo ""
    echo -e "  ${BOLD}Restarting service...${NC}"
    restart_service
    echo ""
    health_check

    # Summary
    echo ""
    echo -e "  ${GREEN}${BOLD}Deploy complete!${NC} ${PI_ICON}"
    echo ""
    echo -e "  ${DIM}Web UI:${NC}  http://${PI_HOST}:3000"
    echo -e "  ${DIM}Logs:${NC}    $0 --logs"
    echo -e "  ${DIM}Revert:${NC}  $0 --revert"
    echo ""
}

main "$@"
