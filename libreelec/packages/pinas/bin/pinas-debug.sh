#!/bin/sh
# PiNAS Debug Tool
# Helps diagnose issues with PiNAS on LibreELEC

VERSION="1.0"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Counters
PASSED=0
WARNINGS=0
ERRORS=0

# Paths
PINAS_BIN="/usr/bin/pinas"
PINAS_DB="/storage/.pinas/data/pinas.db"
PINAS_WWW="/storage/.pinas/www"
PINAS_CONFIG="/storage/.pinas/config.toml"
PINAS_DATA_DIR="/storage/.pinas"
PINAS_PORT=3000

# Helper functions
print_header() {
    echo ""
    echo -e "${CYAN}>>> $1${NC}"
}

print_ok() {
    echo -e "    ${GREEN}✓${NC} $1"
    PASSED=$((PASSED + 1))
}

print_warn() {
    echo -e "    ${YELLOW}!${NC} $1"
    WARNINGS=$((WARNINGS + 1))
}

print_error() {
    echo -e "    ${RED}✗${NC} $1"
    ERRORS=$((ERRORS + 1))
}

print_info() {
    echo -e "    ${CYAN}ℹ${NC} $1"
}

show_usage() {
    echo "PiNAS Debug Tool v${VERSION}"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -s, --status      Check systemd service status"
    echo "  -l, --logs        Show last 50 log lines"
    echo "  -f, --follow      Follow logs in real-time (Ctrl+C to quit)"
    echo "  -t, --test        Test API endpoints"
    echo "  -c, --check       Check files (binary, DB, frontend, config)"
    echo "  -n, --network     Check network (port 3000, interfaces)"
    echo "  -m, --manual      Stop service and run manually in debug mode"
    echo "  -a, --all         Run all checks (except --manual and --follow)"
    echo "  -h, --help        Show this help"
    echo ""
    echo "Without options: equivalent to --all"
    exit 0
}

# Check functions
check_status() {
    print_header "Service Status"

    if systemctl is-active --quiet pinas; then
        PID=$(systemctl show pinas --property=MainPID --value)
        SINCE=$(systemctl show pinas --property=ActiveEnterTimestamp --value)
        print_ok "pinas.service is running (PID: $PID)"
        print_info "Active since: $SINCE"
    else
        STATE=$(systemctl is-active pinas)
        print_error "pinas.service is not running (state: $STATE)"

        # Check if service exists
        if ! systemctl list-unit-files | grep -q pinas.service; then
            print_error "pinas.service not found in systemd"
        fi
    fi

    # Check resize service
    if [ -f "/storage/.pinas/.storage-resized" ]; then
        print_ok "Storage resize completed"
    else
        STORAGE_SIZE=$(df /storage 2>/dev/null | tail -1 | awk '{print $2}')
        STORAGE_MB=$((STORAGE_SIZE / 1024))
        if [ "$STORAGE_MB" -lt 1000 ]; then
            print_warn "Storage not resized yet (${STORAGE_MB}MB) - will resize on next boot"
        else
            print_ok "Storage size OK (${STORAGE_MB}MB)"
        fi
    fi
}

check_files() {
    print_header "File Check"

    # Binary
    if [ -f "$PINAS_BIN" ]; then
        SIZE=$(ls -lh "$PINAS_BIN" | awk '{print $5}')
        print_ok "$PINAS_BIN ($SIZE)"

        # Check if executable
        if [ -x "$PINAS_BIN" ]; then
            print_ok "Binary is executable"
        else
            print_error "Binary is not executable"
        fi
    else
        print_error "$PINAS_BIN (missing)"
    fi

    # Data directory
    if [ -d "$PINAS_DATA_DIR" ]; then
        print_ok "$PINAS_DATA_DIR exists"
    else
        print_error "$PINAS_DATA_DIR (missing)"
    fi

    # Database
    if [ -f "$PINAS_DB" ]; then
        SIZE=$(ls -lh "$PINAS_DB" | awk '{print $5}')
        print_ok "$PINAS_DB ($SIZE)"
    else
        print_warn "$PINAS_DB (missing - will be created on first run)"
    fi

    # Frontend
    if [ -d "$PINAS_WWW" ] && [ -f "$PINAS_WWW/index.html" ]; then
        COUNT=$(find "$PINAS_WWW" -type f 2>/dev/null | wc -l)
        print_ok "$PINAS_WWW ($COUNT files)"
    else
        print_error "$PINAS_WWW/index.html (missing)"
    fi

    # Config (optional)
    if [ -f "$PINAS_CONFIG" ]; then
        print_ok "$PINAS_CONFIG"
    else
        print_info "$PINAS_CONFIG (not present - using defaults)"
    fi
}

check_network() {
    print_header "Network Check"

    # Check if port is listening
    if netstat -tlnp 2>/dev/null | grep -q ":${PINAS_PORT}"; then
        PROC=$(netstat -tlnp 2>/dev/null | grep ":${PINAS_PORT}" | awk '{print $7}')
        print_ok "Port $PINAS_PORT is listening ($PROC)"
    else
        print_error "Port $PINAS_PORT is not listening"
    fi

    # Check network interfaces
    for iface in eth0 wlan0 end0; do
        IP=$(ip -4 addr show $iface 2>/dev/null | grep inet | awk '{print $2}' | cut -d/ -f1)
        if [ -n "$IP" ]; then
            print_ok "Interface $iface: $IP"
        fi
    done

    # Show access URL
    MAIN_IP=$(ip route get 1 2>/dev/null | awk '{print $7; exit}')
    if [ -n "$MAIN_IP" ]; then
        print_info "Access PiNAS at: http://${MAIN_IP}:${PINAS_PORT}"
    fi
}

test_api() {
    print_header "API Test"

    BASE_URL="http://localhost:${PINAS_PORT}"

    # Health endpoint
    RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/health" 2>/dev/null)
    if [ "$RESPONSE" = "200" ]; then
        print_ok "GET /api/health (200 OK)"
    elif [ "$RESPONSE" = "000" ]; then
        print_error "GET /api/health (connection refused)"
    else
        print_error "GET /api/health ($RESPONSE)"
    fi

    # System info endpoint
    RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/system/info" 2>/dev/null)
    if [ "$RESPONSE" = "200" ]; then
        print_ok "GET /api/system/info (200 OK)"
    elif [ "$RESPONSE" = "000" ]; then
        print_error "GET /api/system/info (connection refused)"
    else
        print_warn "GET /api/system/info ($RESPONSE)"
    fi

    # Auth me endpoint (expected to fail if not authenticated)
    RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/auth/me" 2>/dev/null)
    if [ "$RESPONSE" = "200" ]; then
        print_ok "GET /api/auth/me (200 OK)"
    elif [ "$RESPONSE" = "401" ]; then
        print_info "GET /api/auth/me (401 - normal if not logged in)"
    elif [ "$RESPONSE" = "000" ]; then
        print_error "GET /api/auth/me (connection refused)"
    else
        print_warn "GET /api/auth/me ($RESPONSE)"
    fi

    # Setup status
    RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/setup/status" 2>/dev/null)
    if [ "$RESPONSE" = "200" ]; then
        SETUP_DONE=$(curl -s "$BASE_URL/api/setup/status" 2>/dev/null | grep -o '"is_complete":[^,}]*' | cut -d: -f2)
        print_ok "GET /api/setup/status (setup complete: $SETUP_DONE)"
    elif [ "$RESPONSE" = "000" ]; then
        print_error "GET /api/setup/status (connection refused)"
    else
        print_warn "GET /api/setup/status ($RESPONSE)"
    fi
}

show_logs() {
    print_header "Recent Logs (last 50 lines)"
    echo ""
    journalctl -u pinas -n 50 --no-pager
}

follow_logs() {
    print_header "Following logs (Ctrl+C to quit)"
    echo ""
    journalctl -u pinas -f
}

run_manual() {
    print_header "Manual Debug Mode"
    echo ""
    echo -e "${YELLOW}Stopping pinas service...${NC}"
    systemctl stop pinas

    echo ""
    echo -e "${YELLOW}Starting pinas manually with debug logging...${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo ""

    RUST_LOG=debug \
    PINAS_DATABASE_URL="sqlite:${PINAS_DB}?mode=rwc" \
    PINAS_BIND_ADDRESS="0.0.0.0:${PINAS_PORT}" \
    PINAS_STATIC_DIR="${PINAS_WWW}" \
    PINAS_FILES_ROOT="/storage/.pinas/files" \
    $PINAS_BIN

    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    echo -e "${YELLOW}Manual mode ended. Restarting service...${NC}"
    systemctl start pinas
    echo -e "${GREEN}Service restarted${NC}"
}

print_summary() {
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    if [ $ERRORS -gt 0 ]; then
        echo -e "Summary: ${GREEN}$PASSED passed${NC}, ${YELLOW}$WARNINGS warnings${NC}, ${RED}$ERRORS errors${NC}"
    elif [ $WARNINGS -gt 0 ]; then
        echo -e "Summary: ${GREEN}$PASSED passed${NC}, ${YELLOW}$WARNINGS warnings${NC}"
    else
        echo -e "Summary: ${GREEN}$PASSED passed${NC} - All checks OK!"
    fi
    echo "═══════════════════════════════════════════════════════════"
}

print_banner() {
    echo ""
    echo -e "${CYAN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║              PiNAS Debug Tool v${VERSION}                        ║${NC}"
    echo -e "${CYAN}╚═══════════════════════════════════════════════════════════╝${NC}"
}

# Parse arguments
DO_STATUS=false
DO_LOGS=false
DO_FOLLOW=false
DO_TEST=false
DO_CHECK=false
DO_NETWORK=false
DO_MANUAL=false
DO_ALL=false

if [ $# -eq 0 ]; then
    DO_ALL=true
fi

while [ $# -gt 0 ]; do
    case "$1" in
        -s|--status)
            DO_STATUS=true
            shift
            ;;
        -l|--logs)
            DO_LOGS=true
            shift
            ;;
        -f|--follow)
            DO_FOLLOW=true
            shift
            ;;
        -t|--test)
            DO_TEST=true
            shift
            ;;
        -c|--check)
            DO_CHECK=true
            shift
            ;;
        -n|--network)
            DO_NETWORK=true
            shift
            ;;
        -m|--manual)
            DO_MANUAL=true
            shift
            ;;
        -a|--all)
            DO_ALL=true
            shift
            ;;
        -h|--help)
            show_usage
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            ;;
    esac
done

# Run selected checks
print_banner

if [ "$DO_ALL" = true ]; then
    check_status
    check_files
    check_network
    test_api
    print_summary
elif [ "$DO_FOLLOW" = true ]; then
    follow_logs
elif [ "$DO_MANUAL" = true ]; then
    run_manual
else
    [ "$DO_STATUS" = true ] && check_status
    [ "$DO_CHECK" = true ] && check_files
    [ "$DO_NETWORK" = true ] && check_network
    [ "$DO_TEST" = true ] && test_api
    [ "$DO_LOGS" = true ] && show_logs

    # Print summary if any checks were run
    if [ "$DO_STATUS" = true ] || [ "$DO_CHECK" = true ] || [ "$DO_NETWORK" = true ] || [ "$DO_TEST" = true ]; then
        print_summary
    fi
fi
