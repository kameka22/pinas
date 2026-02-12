#!/bin/sh
# PiNAS - Splash screen for HDMI display
# Displays system information on tty1 when no display service (e.g. Kodi) is active.
# Refreshes periodically to update IP address (DHCP).
# Text is centered on a black background.

set -e

TTY="/dev/tty1"

# Get the first active IP address
get_ip() {
    ip -4 addr show scope global 2>/dev/null | \
        grep -oP 'inet \K[\d.]+' | \
        head -1
}

# Print a string centered horizontally
# Args: $1=text, $2=cols
print_centered() {
    text="$1"
    cols="$2"
    len=${#text}
    if [ "$len" -ge "$cols" ]; then
        printf '%s' "$text"
    else
        pad=$(( (cols - len) / 2 ))
        printf '%*s%s' "$pad" '' "$text"
    fi
}

while true; do
    IP=$(get_ip)
    if [ -z "$IP" ]; then
        IP="..."
    fi

    PORT="3000"
    URL="http://${IP}:${PORT}"

    # Get terminal dimensions (fallback: 25x80)
    if TERM_SIZE=$(stty size < "$TTY" 2>/dev/null); then
        ROWS=$(echo "$TERM_SIZE" | cut -d' ' -f1)
        COLS=$(echo "$TERM_SIZE" | cut -d' ' -f2)
    else
        ROWS=25
        COLS=80
    fi

    # Content lines (14 lines)
    L1="____  _ _   _    _    ____"
    L2="|  _ \\(_) \\ | |  / \\  / ___|"
    L3="| |_) | |  \\| | / _ \\ \\___ \\"
    L4="|  __/| | |\\  |/ ___ \\ ___) |"
    L5="|_|   |_|_| \\_/_/   \\_\\____/"
    L6=""
    L7=""
    L8="Your NAS is ready."
    L9=""
    L10="Access PiNAS at:"
    L11="$URL"
    L12=""
    L13="IP address: $IP"
    L14=""

    CONTENT_LINES=14
    TOP_PAD=$(( (ROWS - CONTENT_LINES) / 2 ))
    if [ "$TOP_PAD" -lt 0 ]; then
        TOP_PAD=0
    fi
    BOTTOM_PAD=$(( ROWS - TOP_PAD - CONTENT_LINES ))
    if [ "$BOTTOM_PAD" -lt 0 ]; then
        BOTTOM_PAD=0
    fi

    # Build full screen output
    {
        # Reset terminal: clear screen, hide cursor, set white on black
        printf '\033[2J\033[H\033[?25l\033[0;37;40m'

        # Top padding (blank lines with black background)
        i=0
        while [ "$i" -lt "$TOP_PAD" ]; do
            printf '%*s\n' "$COLS" ''
            i=$((i + 1))
        done

        # Content lines
        for line in "$L1" "$L2" "$L3" "$L4" "$L5" "$L6" "$L7" "$L8" "$L9" "$L10" "$L11" "$L12" "$L13" "$L14"; do
            print_centered "$line" "$COLS"
            printf '\n'
        done

        # Bottom padding
        i=0
        while [ "$i" -lt "$BOTTOM_PAD" ]; do
            printf '%*s\n' "$COLS" ''
            i=$((i + 1))
        done
    } > "$TTY"

    # Refresh every 30 seconds
    sleep 30
done
