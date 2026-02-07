#!/bin/sh
# PiNAS - Configure Kodi webserver for JSON-RPC access
# Runs BEFORE kodi.service to ensure the webserver is enabled on first boot
# and credentials match PiNAS configuration.

set -e

GUISETTINGS="/storage/.kodi/userdata/guisettings.xml"
MARKER="/storage/.pinas/.kodi-configured"
KODI_USER="kodi"
KODI_PASS="pinas"
KODI_PORT="8080"

log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') [kodi-config] $1"
}

# If Kodi settings don't exist yet (first boot), create userdata dir
if [ ! -d "/storage/.kodi/userdata" ]; then
    log "First boot: creating Kodi userdata directory"
    mkdir -p /storage/.kodi/userdata
fi

# Case 1: guisettings.xml doesn't exist (first boot)
# Create a minimal one with webserver enabled
if [ ! -f "$GUISETTINGS" ]; then
    log "Creating guisettings.xml with webserver enabled"
    cat > "$GUISETTINGS" << 'EOF'
<settings version="2">
    <setting id="services.webserver">true</setting>
    <setting id="services.webserverport">8080</setting>
    <setting id="services.webserverusername">kodi</setting>
    <setting id="services.webserverpassword">pinas</setting>
    <setting id="services.esallinterfaces">true</setting>
    <setting id="services.esenabled">true</setting>
</settings>
EOF
    log "Done - Kodi webserver will be enabled on first start"
    touch "$MARKER"
    exit 0
fi

# Case 2: guisettings.xml exists but webserver not configured by PiNAS yet
if [ -f "$MARKER" ]; then
    log "Kodi webserver already configured by PiNAS"
    exit 0
fi

log "Existing guisettings.xml found, enabling webserver..."

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
