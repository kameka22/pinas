#!/usr/bin/env bash
#
# Batch convert Umbrel apps to PiNAS manifests.
#
# Usage:
#   ./scripts/convert-umbrel-batch.sh [--clone] [--umbrel-dir <path>]
#
# Options:
#   --clone         Clone the umbrel-apps repository first
#   --umbrel-dir    Path to existing umbrel-apps clone (default: /tmp/umbrel-apps)
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CATALOG_DIR="$PROJECT_DIR/app-catalog"
CATALOG_JSON="$CATALOG_DIR/catalog.json"
CONVERTER="$SCRIPT_DIR/convert-umbrel.py"
UMBREL_DIR="/tmp/umbrel-apps"

DO_CLONE=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --clone)
            DO_CLONE=true
            shift
            ;;
        --umbrel-dir)
            UMBREL_DIR="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Curated list of NAS-relevant apps to convert
APPS=(
    # Tier 1: Core NAS
    "jellyfin"
    "nextcloud"
    "syncthing"
    "pi-hole"
    "transmission"
    "vaultwarden"
    "file-browser"
    "photoprism"

    # Tier 2: Media & Downloads
    "plex"
    "qbittorrent"
    "emby"
    "sonarr"
    "radarr"
    "lidarr"
    "sabnzbd"

    # Tier 3: Network & Monitoring
    "adguard-home"
    "wireguard"
    "uptime-kuma"
    "grafana"
    "nginx-proxy-manager"

    # Tier 4: Utilities
    "home-assistant"
    "node-red"
    "paperless"
    "duplicati"
    "code-server"
)

# Clone Umbrel apps repo if needed
if [ "$DO_CLONE" = true ] || [ ! -d "$UMBREL_DIR" ]; then
    echo "==> Cloning umbrel-apps repository..."
    rm -rf "$UMBREL_DIR"
    git clone --depth 1 https://github.com/getumbrel/umbrel-apps.git "$UMBREL_DIR"
fi

echo "==> Using Umbrel apps from: $UMBREL_DIR"
echo "==> Output to: $CATALOG_DIR"
echo ""

# Counters
converted=0
skipped=0
failed=0

for app in "${APPS[@]}"; do
    app_dir="$UMBREL_DIR/$app"

    if [ ! -d "$app_dir" ]; then
        echo "[SKIP] $app - not found in Umbrel repo"
        skipped=$((skipped + 1))
        continue
    fi

    if [ ! -f "$app_dir/umbrel-app.yml" ]; then
        echo "[SKIP] $app - no umbrel-app.yml"
        skipped=$((skipped + 1))
        continue
    fi

    output_dir="$CATALOG_DIR/apps/$app"
    echo "[CONVERTING] $app..."

    if python3 "$CONVERTER" "$app_dir" "$output_dir" --catalog "$CATALOG_JSON" 2>&1; then
        converted=$((converted + 1))
    else
        echo "[FAIL] $app - conversion error"
        failed=$((failed + 1))
    fi

    echo ""
done

echo "================================="
echo "Done! Converted: $converted, Skipped: $skipped, Failed: $failed"
echo ""
echo "Next steps:"
echo "  1. Review generated manifests in $CATALOG_DIR/apps/"
echo "  2. Adjust icons, gradients, and FR descriptions as needed"
echo "  3. Commit changes to app-catalog"
