#!/usr/bin/env bash
# Apply the CONFIG_* fragment(s) in libreelec/kernel/*.conf to a LibreELEC kernel config.
# Usage: apply-kernel-config.sh <path/to/linux.<arch>.conf>
# Existing lines (set or "# CONFIG_X is not set") are replaced; missing ones are appended.
set -euo pipefail

target="${1:?kernel config path required}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
fragments_dir="${here}/../../libreelec/kernel"

[ -f "$target" ] || { echo "Kernel config not found: $target" >&2; exit 1; }

for fragment in "$fragments_dir"/*.conf; do
  [ -f "$fragment" ] || continue
  while IFS= read -r line; do
    case "$line" in ''|'#'*) continue ;; esac
    key="${line%%=*}"
    value="${line#*=}"
    if [ "$value" = "n" ]; then
      replacement="# ${key} is not set"
    else
      replacement="$line"
    fi
    if grep -qE "^(${key}=|# ${key} is not set)" "$target"; then
      sed -i -E "s~^(${key}=.*|# ${key} is not set)\$~${replacement}~" "$target"
    else
      echo "$replacement" >> "$target"
    fi
  done < "$fragment"
  echo "    Applied kernel fragment $(basename "$fragment") to $(basename "$target")"
done
