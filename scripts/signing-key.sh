#!/usr/bin/env bash
# Manage the Ed25519 key that signs PiNAS update archives.
#
# The private key never enters the repository. Devices trust the public keys
# listed in UPDATE_SIGNING_KEYS (backend/src/services/update.rs) and refuse any
# update whose .sha256 is not signed by one of them.
#
# Usage:
#   scripts/signing-key.sh status            Key present? public key? trusted by the code?
#   scripts/signing-key.sh restore <backup>  Put a backed-up key in place (mode 600) and check it is trusted
#   scripts/signing-key.sh remove            Shred the local key (after a release, once it is backed up)
#   scripts/signing-key.sh new [--out FILE]  Generate a new key and add its public key to UPDATE_SIGNING_KEYS
#
# Key path: $PINAS_UPDATE_SIGNING_KEY, default ~/.config/pinas/update-signing.pem.
#
# Rotation: installed devices only trust the keys compiled into their version.
# A release that introduces a new key must still be signed with an old trusted
# key; only the following releases can be signed with the new one. Losing every
# trusted key means installed devices can no longer be updated over the air.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
UPDATE_RS="$PROJECT_ROOT/backend/src/services/update.rs"
KEY="${PINAS_UPDATE_SIGNING_KEY:-$HOME/.config/pinas/update-signing.pem}"

die() { echo "Error: $*" >&2; exit 1; }

pubkey_of() {
    openssl pkey -in "$1" -pubout -outform DER | tail -c 32 | base64 -w0
}

is_trusted() {
    grep -qF "\"$1\"" "$UPDATE_RS"
}

check_ed25519() {
    openssl pkey -in "$1" -noout -text 2>/dev/null | grep -q "ED25519 Private-Key" \
        || die "$1 is not an Ed25519 private key"
}

cmd_status() {
    echo "Key path: $KEY"
    if [ ! -e "$KEY" ]; then
        echo "  absent (restore it from the backup before running build-release.sh)"
        return 0
    fi
    check_ed25519 "$KEY"
    local pub
    pub=$(pubkey_of "$KEY")
    echo "  public key: $pub"
    if is_trusted "$pub"; then
        echo "  trusted: yes (listed in UPDATE_SIGNING_KEYS)"
    else
        echo "  trusted: NO (devices would reject releases signed with it)"
    fi
    echo "  mode: $(stat -c %a "$KEY")"
}

cmd_restore() {
    local src="${1:-}"
    [ -n "$src" ] || die "usage: $0 restore <backup-file>"
    [ -r "$src" ] || die "cannot read $src"
    [ -e "$KEY" ] && die "$KEY already exists; remove it first"
    check_ed25519 "$src"
    local pub
    pub=$(pubkey_of "$src")
    is_trusted "$pub" || die "public key $pub of $src is not in UPDATE_SIGNING_KEYS: wrong backup?"
    install -d -m 700 "$(dirname "$KEY")"
    install -m 600 "$src" "$KEY"
    echo "Restored $KEY (public key $pub, trusted)."
}

cmd_remove() {
    [ -e "$KEY" ] || { echo "$KEY is already absent."; return 0; }
    shred -u "$KEY"
    echo "Shredded $KEY. Restore it with: $0 restore <backup>"
}

cmd_new() {
    local out="$KEY"
    if [ "${1:-}" = "--out" ]; then
        out="${2:-}"
        [ -n "$out" ] || die "usage: $0 new [--out FILE]"
    fi
    [ -e "$out" ] && die "$out already exists; refusing to overwrite a key"
    install -d -m 700 "$(dirname "$out")"
    (umask 077 && openssl genpkey -algorithm ed25519 -out "$out")
    local pub
    pub=$(pubkey_of "$out")
    python3 - "$UPDATE_RS" "$pub" <<'PY'
import re, sys
path, pub = sys.argv[1], sys.argv[2]
src = open(path).read()
pattern = re.compile(r'(const UPDATE_SIGNING_KEYS: &\[&str\] = &\[)([^\]]*)(\];)')
match = pattern.search(src)
if not match:
    sys.exit("UPDATE_SIGNING_KEYS not found in " + path)
keys = [k.strip() for k in match.group(2).split(",") if k.strip()]
keys.append(f'"{pub}"')
src = src[:match.start()] + match.group(1) + ", ".join(keys) + match.group(3) + src[match.end():]
open(path, "w").write(src)
PY
    echo "Generated $out (mode 600)."
    echo "Public key $pub added to UPDATE_SIGNING_KEYS in $UPDATE_RS."
    echo
    echo "Next steps:"
    echo "  1. Back the private key up outside this machine NOW."
    echo "  2. Commit update.rs. Devices only learn the new key from a release signed"
    echo "     with a key they already trust; drop the old key from the list only after that."
}

case "${1:-status}" in
    status)  cmd_status ;;
    restore) shift; cmd_restore "$@" ;;
    remove)  cmd_remove ;;
    new)     shift; cmd_new "$@" ;;
    -h|--help|help) sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//' ;;
    *) die "unknown command '$1' (status, restore, remove, new)" ;;
esac
