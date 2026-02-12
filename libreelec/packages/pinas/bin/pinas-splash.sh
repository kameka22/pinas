#!/bin/sh
# PiNAS - Splash screen for HDMI display
# Displays system information on tty1 when no display service (e.g. Kodi) is active.
# Refreshes periodically to update IP address (DHCP).

set -e

TTY="/dev/tty1"

# Get the first active IP address
get_ip() {
    ip -4 addr show scope global 2>/dev/null | \
        grep -oP 'inet \K[\d.]+' | \
        head -1
}

# Clear screen and hide cursor
setterm --clear all --cursor off > "$TTY" 2>/dev/null || true

while true; do
    IP=$(get_ip)
    if [ -z "$IP" ]; then
        IP="..."
    fi

    # Read port from pinas.service environment (default 3000)
    PORT="3000"
    URL="http://${IP}:${PORT}"

    cat > "$TTY" << SPLASH

    ____  _ _   _    _    ____
   |  _ \\(_) \\ | |  / \\  / ___|
   | |_) | |  \\| | / _ \\ \\___ \\
   |  __/| | |\\  |/ ___ \\ ___) |
   |_|   |_|_| \\_/_/   \\_\\____/


   Your NAS is ready.

   Access PiNAS at:
   ${URL}

   IP address: ${IP}

SPLASH

    # Refresh every 30 seconds
    sleep 30
done
