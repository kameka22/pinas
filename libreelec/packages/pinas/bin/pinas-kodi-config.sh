#!/bin/sh
# PiNAS - Configure Kodi webserver for JSON-RPC access
# Runs BEFORE kodi.service to ensure the webserver is enabled on first boot
# and credentials match PiNAS configuration.
#
# The password is shared with the PiNAS backend via a file:
#   /storage/.pinas/data/.kodi_password
# This script generates it if missing; the backend reads the same file.

set -e

GUISETTINGS="/storage/.kodi/userdata/guisettings.xml"
MARKER="/storage/.pinas/.kodi-configured"
PASSWORD_FILE="/storage/.pinas/data/.kodi_password"
KODI_USER="kodi"
KODI_PORT="8080"

log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') [kodi-config] $1"
}

# Load or generate the shared Kodi password
load_or_generate_password() {
    mkdir -p "$(dirname "$PASSWORD_FILE")"

    if [ -f "$PASSWORD_FILE" ] && [ -s "$PASSWORD_FILE" ]; then
        KODI_PASS="$(cat "$PASSWORD_FILE")"
        log "Kodi password loaded from $PASSWORD_FILE"
    else
        # Generate a random password (32 hex chars)
        KODI_PASS="$(head -c 16 /dev/urandom | od -An -tx1 | tr -d ' \n')"
        printf '%s' "$KODI_PASS" > "$PASSWORD_FILE"
        chmod 600 "$PASSWORD_FILE"
        log "Generated new Kodi password at $PASSWORD_FILE"
    fi
}

load_or_generate_password

# If Kodi settings don't exist yet (first boot), create userdata dir
if [ ! -d "/storage/.kodi/userdata" ]; then
    log "First boot: creating Kodi userdata directory"
    mkdir -p /storage/.kodi/userdata
fi

# Case 1: guisettings.xml doesn't exist (first boot)
# Create a minimal one with webserver enabled
if [ ! -f "$GUISETTINGS" ]; then
    log "Creating guisettings.xml with webserver enabled"
    cat > "$GUISETTINGS" << EOF
<settings version="2">
    <setting id="services.webserver">true</setting>
    <setting id="services.webserverport">${KODI_PORT}</setting>
    <setting id="services.webserverusername">${KODI_USER}</setting>
    <setting id="services.webserverpassword">${KODI_PASS}</setting>
    <setting id="services.esallinterfaces">true</setting>
    <setting id="services.esenabled">true</setting>
</settings>
EOF
    log "Done - Kodi webserver will be enabled on first start"
    touch "$MARKER"
    exit 0
fi

# Case 2: guisettings.xml exists - ensure webserver is enabled and password is in sync.
# We always re-apply settings because Kodi overwrites guisettings.xml on exit,
# and the password must stay synchronized with the backend.
log "Updating guisettings.xml - ensuring webserver is enabled..."

# Enable webserver settings using sed
# Each setting can be: missing, set to true/false, or have default="true" attribute

enable_setting() {
    local setting_id="$1"
    local value="$2"

    if grep -q "id=\"${setting_id}\"" "$GUISETTINGS"; then
        # Setting exists - update its value
        sed -i "s|<setting id=\"${setting_id}\"[^>]*>[^<]*</setting>|<setting id=\"${setting_id}\">${value}</setting>|" "$GUISETTINGS"
        log "  Updated ${setting_id} = ${value}"
    else
        # Setting doesn't exist - add it before </settings>
        sed -i "s|</settings>|    <setting id=\"${setting_id}\">${value}</setting>\n</settings>|" "$GUISETTINGS"
        log "  Added ${setting_id} = ${value}"
    fi
}

enable_setting "services.webserver" "true"
enable_setting "services.webserverport" "$KODI_PORT"
enable_setting "services.webserverusername" "$KODI_USER"
enable_setting "services.webserverpassword" "$KODI_PASS"
enable_setting "services.esallinterfaces" "true"
enable_setting "services.esenabled" "true"

touch "$MARKER"
log "Kodi webserver configured successfully"
