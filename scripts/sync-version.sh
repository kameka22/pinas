#!/bin/bash
# Single source of truth for the PiNAS version: the VERSION file at the repo root.
# Propagates it to every file that carries a copy, or verifies they all agree.
#
# Usage:
#   scripts/sync-version.sh            # rewrite Cargo.toml, Cargo.lock, package.json, package-lock.json, package.mk
#   scripts/sync-version.sh --check    # exit 1 (listing the drift) if any copy differs — used by CI
#   scripts/sync-version.sh --set 0.11.0   # write VERSION, then propagate
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
MODE="sync"

case "${1:-}" in
    --check) MODE="check" ;;
    --set)
        [ -n "${2:-}" ] || { echo "usage: $0 --set X.Y.Z" >&2; exit 2; }
        [[ "$2" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "invalid version '$2' (expected X.Y.Z)" >&2; exit 2; }
        printf '%s\n' "$2" > "$ROOT/VERSION"
        ;;
    "") ;;
    *) echo "unknown option $1" >&2; exit 2 ;;
esac

VERSION="$(tr -d '[:space:]' < "$ROOT/VERSION")"
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "VERSION must be X.Y.Z, got '$VERSION'" >&2; exit 2; }

MODE="$MODE" VERSION="$VERSION" ROOT="$ROOT" python3 - <<'PY'
import os, re, sys
root, version, check = os.environ["ROOT"], os.environ["VERSION"], os.environ["MODE"] == "check"

# (file, regex with one capture group around the version, max replacements)
targets = [
    ("backend/Cargo.toml", r'(?m)^version = "([^"]+)"', 1),
    ("backend/Cargo.lock", r'(?ms)^\[\[package\]\]\nname = "pinas"\nversion = "([^"]+)"', 1),
    ("frontend/package.json", r'(?m)^  "version": "([^"]+)"', 1),
    # top-level "version" and packages[""].version, both within the first 20 lines of the lock file
    ("frontend/package-lock.json", r'(?m)^(?:  |      )"version": "([^"]+)"', 2),
    ("libreelec/packages/pinas/package.mk", r'(?m)^PKG_VERSION="([^"]+)"', 1),
]

drift = []
for rel, pattern, count in targets:
    path = os.path.join(root, rel)
    if not os.path.exists(path):
        continue
    text = open(path, encoding="utf-8").read()
    matches = list(re.finditer(pattern, text))[:count]
    if not matches:
        print(f"!! no version field found in {rel}", file=sys.stderr)
        sys.exit(1)
    current = [m.group(1) for m in matches]
    if all(c == version for c in current):
        continue
    drift.append((rel, current))
    if not check:
        # replace from the end so offsets stay valid
        for m in reversed(matches):
            text = text[: m.start(1)] + version + text[m.end(1):]
        open(path, "w", encoding="utf-8").write(text)
        print(f"   {rel}: {', '.join(current)} -> {version}")

if check:
    if drift:
        print(f"Version drift (VERSION = {version}):", file=sys.stderr)
        for rel, current in drift:
            print(f"  {rel}: {', '.join(current)}", file=sys.stderr)
        print("Run scripts/sync-version.sh to fix.", file=sys.stderr)
        sys.exit(1)
    print(f"All version fields match VERSION = {version}")
elif not drift:
    print(f"Already in sync: {version}")
PY
