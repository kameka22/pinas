#!/bin/bash
# PiNAS - Connect to Raspberry Pi
# Discovers the Pi on the local network and opens an SSH session
# Default credentials: root:libreelec

set -e

# ── Colors & Symbols ──────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

CHECK="${GREEN}✔${NC}"
CROSS="${RED}✘${NC}"
ARROW="${CYAN}➜${NC}"
PI="${MAGENTA}🍓${NC}"

# ── Config ────────────────────────────────────────────────────────
DEFAULT_USER="root"
DEFAULT_PASS="libreelec"
DEFAULT_PORT=22
MDNS_NAMES=("libreelec.local" "pinas.local")
PI_MAC_PREFIXES=("b8:27:eb" "dc:a6:32" "e4:5f:01" "d8:3a:dd" "2c:cf:67")
SCAN_TIMEOUT=3

# ── Helpers ───────────────────────────────────────────────────────
spinner() {
    local pid=$1
    local msg=$2
    local frames=("⠋" "⠙" "⠹" "⠸" "⠼" "⠴" "⠦" "⠧" "⠇" "⠏")
    local i=0
    while kill -0 "$pid" 2>/dev/null; do
        printf "\r  ${CYAN}${frames[$i]}${NC} %s" "$msg"
        i=$(( (i + 1) % ${#frames[@]} ))
        sleep 0.1
    done
    printf "\r"
}

banner() {
    echo ""
    echo -e "  ${MAGENTA}${BOLD}┌─────────────────────────────────┐${NC}"
    echo -e "  ${MAGENTA}${BOLD}│${NC}  ${PI} ${BOLD}PiNAS Connect${NC}               ${MAGENTA}${BOLD}│${NC}"
    echo -e "  ${MAGENTA}${BOLD}│${NC}  ${DIM}Find & connect to your Pi${NC}      ${MAGENTA}${BOLD}│${NC}"
    echo -e "  ${MAGENTA}${BOLD}└─────────────────────────────────┘${NC}"
    echo ""
}

found_hosts=()
found_ips=()

# ── Discovery Methods ─────────────────────────────────────────────

discover_mdns() {
    local found=false
    local tmp_dir
    tmp_dir=$(mktemp -d)

    # Launch all lookups in parallel
    for name in "${MDNS_NAMES[@]}"; do
        (
            local ip=""
            # macOS: dns-sd is the fastest native mDNS resolver
            if [[ -z "$ip" ]] && command -v dns-sd &>/dev/null; then
                ip=$(dns-sd -timeout 1 -G v4 "$name" 2>/dev/null | tail -1 | awk '{print $NF}' || true)
                # dns-sd can return "..." on timeout
                [[ "$ip" =~ ^[0-9]+\. ]] || ip=""
            fi
            # Linux: avahi-resolve is instant
            if [[ -z "$ip" ]] && command -v avahi-resolve &>/dev/null; then
                ip=$(avahi-resolve -4 -n "$name" 2>/dev/null | awk '{print $2}' || true)
            fi
            # Fallback: getent (uses system resolver, may support mDNS via nsswitch)
            if [[ -z "$ip" ]]; then
                ip=$(getent hosts "$name" 2>/dev/null | awk '{print $1}' | head -1 || true)
            fi
            # Last resort: quick ping with 1s timeout
            if [[ -z "$ip" ]]; then
                ip=$(ping -c 1 -W 1 "$name" 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+' | head -1 || true)
            fi
            if [[ -n "$ip" ]]; then
                echo "${ip}|${name}" > "${tmp_dir}/${name}"
            fi
        ) &
    done
    wait 2>/dev/null

    # Collect results
    for f in "$tmp_dir"/*; do
        [[ -f "$f" ]] || continue
        local entry ip name
        entry=$(cat "$f")
        ip="${entry%%|*}"
        name="${entry##*|}"
        add_host "$ip" "$name (mDNS)"
        found=true
    done
    rm -rf "$tmp_dir"
    $found
}

discover_arp() {
    local subnet
    # Detect local subnet
    if command -v ip &>/dev/null; then
        subnet=$(ip route 2>/dev/null | grep "default" | head -1 | awk '{print $3}' | sed 's/\.[0-9]*$/.0\/24/')
    fi
    if [[ -z "$subnet" ]]; then
        # macOS fallback
        local gateway
        gateway=$(route -n get default 2>/dev/null | grep gateway | awk '{print $2}')
        if [[ -n "$gateway" ]]; then
            subnet=$(echo "$gateway" | sed 's/\.[0-9]*$/.0\/24/')
        fi
    fi
    if [[ -z "$subnet" ]]; then
        subnet="192.168.1.0/24"
    fi

    # Check ARP table for known Pi MAC prefixes
    local arp_output
    arp_output=$(arp -a 2>/dev/null)

    local found=false
    for prefix in "${PI_MAC_PREFIXES[@]}"; do
        while IFS= read -r line; do
            local ip
            ip=$(echo "$line" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+' | head -1)
            if [[ -n "$ip" ]]; then
                add_host "$ip" "ARP table, MAC ${prefix}:..."
                found=true
            fi
        done <<< "$(echo "$arp_output" | grep -i "$prefix")"
    done
    $found
}

discover_nmap() {
    if ! command -v nmap &>/dev/null; then
        return 1
    fi

    local subnet
    if command -v ip &>/dev/null; then
        subnet=$(ip -4 route 2>/dev/null | grep "default" | head -1 | awk '{print $3}' | sed 's/\.[0-9]*$/.0\/24/')
    fi
    if [[ -z "$subnet" ]]; then
        local gateway
        gateway=$(route -n get default 2>/dev/null | grep gateway | awk '{print $2}')
        if [[ -n "$gateway" ]]; then
            subnet=$(echo "$gateway" | sed 's/\.[0-9]*$/.0\/24/')
        fi
    fi
    if [[ -z "$subnet" ]]; then
        subnet="192.168.1.0/24"
    fi

    echo -e "  ${DIM}Scanning ${subnet}...${NC}"

    local nmap_output
    nmap_output=$(nmap -sn "$subnet" 2>/dev/null)

    local found=false
    # Look for Raspberry Pi by vendor
    local ip=""
    while IFS= read -r line; do
        if echo "$line" | grep -q "Nmap scan report for"; then
            ip=$(echo "$line" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+')
        fi
        if echo "$line" | grep -qi "raspberry\|raspberrypi"; then
            if [[ -n "$ip" ]]; then
                add_host "$ip" "nmap scan"
                found=true
            fi
        fi
    done <<< "$nmap_output"
    $found
}

discover_ssh_probe() {
    local subnet
    if command -v ip &>/dev/null; then
        subnet=$(ip -4 route 2>/dev/null | grep "default" | head -1 | awk '{print $3}' | sed 's/\.[0-9]*$//')
    fi
    if [[ -z "$subnet" ]]; then
        local gateway
        gateway=$(route -n get default 2>/dev/null | grep gateway | awk '{print $2}')
        if [[ -n "$gateway" ]]; then
            subnet=$(echo "$gateway" | sed 's/\.[0-9]*$//')
        fi
    fi
    if [[ -z "$subnet" ]]; then
        subnet="192.168.1"
    fi

    echo -e "  ${DIM}Probing ${subnet}.0/24 for SSH on port 22...${NC}"

    local found=false
    local pids=()
    local tmp_dir
    tmp_dir=$(mktemp -d)

    # Parallel SSH banner grab on common Pi IPs
    for i in $(seq 1 254); do
        (
            local ip="${subnet}.${i}"
            local banner
            banner=$(timeout "$SCAN_TIMEOUT" bash -c "echo '' | nc -w 2 '$ip' 22 2>/dev/null" || true)
            if echo "$banner" | grep -qi "ssh"; then
                echo "$ip" > "${tmp_dir}/${i}"
            fi
        ) &
        pids+=($!)
        # Limit parallel connections
        if (( ${#pids[@]} >= 50 )); then
            wait "${pids[0]}" 2>/dev/null || true
            pids=("${pids[@]:1}")
        fi
    done
    wait 2>/dev/null

    for f in "$tmp_dir"/*; do
        [[ -f "$f" ]] || continue
        local ip
        ip=$(cat "$f")
        add_host "$ip" "SSH probe"
        found=true
    done
    rm -rf "$tmp_dir"
    $found
}

add_host() {
    local ip=$1
    local source=$2
    # Deduplicate
    for existing in "${found_ips[@]}"; do
        if [[ "$existing" == "$ip" ]]; then
            return
        fi
    done
    found_ips+=("$ip")
    found_hosts+=("$ip|$source")
}

# ── Verify PiNAS is running ──────────────────────────────────────

check_pinas() {
    local ip=$1
    local status
    status=$(curl -s --connect-timeout 3 "http://${ip}:3000/api/health" 2>/dev/null || true)
    if [[ -n "$status" ]]; then
        return 0
    fi
    return 1
}

# ── Connect ───────────────────────────────────────────────────────

do_connect() {
    local ip=$1
    echo ""
    echo -e "  ${ARROW} Connecting to ${BOLD}${ip}${NC} as ${CYAN}${DEFAULT_USER}${NC}"
    echo -e "  ${DIM}Password: ${DEFAULT_PASS}${NC}"
    echo ""

    if command -v sshpass &>/dev/null; then
        sshpass -p "$DEFAULT_PASS" ssh \
            -o StrictHostKeyChecking=no \
            -o UserKnownHostsFile=/dev/null \
            -o LogLevel=ERROR \
            -p "$DEFAULT_PORT" \
            "${DEFAULT_USER}@${ip}"
    else
        echo -e "  ${DIM}Tip: install ${CYAN}sshpass${DIM} for auto-login (brew install sshpass / apt install sshpass)${NC}"
        echo -e "  ${DIM}Password is: ${BOLD}${DEFAULT_PASS}${NC}"
        echo ""
        ssh \
            -o StrictHostKeyChecking=no \
            -o UserKnownHostsFile=/dev/null \
            -o LogLevel=ERROR \
            -p "$DEFAULT_PORT" \
            "${DEFAULT_USER}@${ip}"
    fi
}

# ── Main ──────────────────────────────────────────────────────────

main() {
    banner

    # Allow direct IP argument
    if [[ -n "$1" ]]; then
        echo -e "  ${ARROW} Using provided IP: ${BOLD}$1${NC}"
        do_connect "$1"
        exit 0
    fi

    # ── Step 1: mDNS ──
    echo -e "  ${BOLD}Step 1/3${NC} ${DIM}— mDNS discovery${NC}"
    if discover_mdns; then
        echo -e "  ${CHECK} Found via mDNS"
    else
        echo -e "  ${CROSS} No mDNS response"
    fi

    # ── Step 2: ARP table ──
    echo -e "  ${BOLD}Step 2/3${NC} ${DIM}— ARP table (known Pi MACs)${NC}"
    if discover_arp; then
        echo -e "  ${CHECK} Found in ARP table"
    else
        echo -e "  ${CROSS} No Pi found in ARP cache"
    fi

    # ── Step 3: Deeper scan (only if nothing found yet) ──
    if [[ ${#found_ips[@]} -eq 0 ]]; then
        echo -e "  ${BOLD}Step 3/3${NC} ${DIM}— Network scan${NC}"
        if command -v nmap &>/dev/null; then
            if discover_nmap; then
                echo -e "  ${CHECK} Found via nmap"
            fi
        else
            echo -e "  ${DIM}nmap not available, probing SSH ports...${NC}"
            if discover_ssh_probe; then
                echo -e "  ${CHECK} Found via SSH probe"
            fi
        fi
    else
        echo -e "  ${BOLD}Step 3/3${NC} ${DIM}— Network scan ${GREEN}(skipped, already found)${NC}"
    fi

    echo ""

    # ── No results ──
    if [[ ${#found_ips[@]} -eq 0 ]]; then
        echo -e "  ${CROSS} ${RED}No Raspberry Pi found on the network${NC}"
        echo ""
        echo -e "  ${DIM}Troubleshooting:${NC}"
        echo -e "  ${DIM}  1. Is the Pi powered on and connected?${NC}"
        echo -e "  ${DIM}  2. Are you on the same network/VLAN?${NC}"
        echo -e "  ${DIM}  3. Try with a direct IP: ${CYAN}$0 192.168.1.x${NC}"
        echo ""
        exit 1
    fi

    # ── Display results ──
    echo -e "  ${PI} ${BOLD}Found ${#found_ips[@]} device(s):${NC}"
    echo ""

    for i in "${!found_hosts[@]}"; do
        local entry="${found_hosts[$i]}"
        local ip="${entry%%|*}"
        local source="${entry##*|}"
        local pinas_status=""

        if check_pinas "$ip"; then
            pinas_status=" ${GREEN}● PiNAS running${NC}"
        fi

        local num=$((i + 1))
        echo -e "    ${BOLD}${num})${NC}  ${CYAN}${ip}${NC}  ${DIM}(${source})${NC}${pinas_status}"
    done

    echo ""

    # ── Single result → ask directly ──
    if [[ ${#found_ips[@]} -eq 1 ]]; then
        local ip="${found_ips[0]}"
        echo -ne "  ${ARROW} Connect to ${BOLD}${ip}${NC}? ${DIM}[Y/n]${NC} "
        read -r answer
        answer=${answer:-y}
        if [[ "$answer" =~ ^[Yy]$ ]]; then
            do_connect "$ip"
        else
            echo -e "  ${DIM}Bye!${NC}"
        fi
        exit 0
    fi

    # ── Multiple results → pick one ──
    echo -ne "  ${ARROW} Which one? ${DIM}[1-${#found_ips[@]}, q to quit]${NC} "
    read -r choice

    if [[ "$choice" == "q" || "$choice" == "Q" ]]; then
        echo -e "  ${DIM}Bye!${NC}"
        exit 0
    fi

    if [[ "$choice" =~ ^[0-9]+$ ]] && (( choice >= 1 && choice <= ${#found_ips[@]} )); then
        local idx=$((choice - 1))
        do_connect "${found_ips[$idx]}"
    else
        echo -e "  ${CROSS} Invalid choice"
        exit 1
    fi
}

main "$@"
