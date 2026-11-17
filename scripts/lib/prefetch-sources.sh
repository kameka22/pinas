#!/usr/bin/env bash
# Pre-seed LibreELEC's source cache for packages whose upstream tarball can no
# longer be fetched (download blocked or file removed).
#
# Usage: scripts/lib/prefetch-sources.sh <LibreELEC.tv dir>
#
# Some upstreams answer scripted clients with an HTML challenge page instead of
# the tarball (linuxtv.org sits behind an anti-bot wall since 2026-11), others
# have removed the release (releases.pagure.org: newt 0.52.24 is a 404), and the
# LibreELEC mirror does not always carry the file. `make image` then aborts.
# For each package listed below, this script downloads the very same tarball
# from a fallback mirror, checks it against PKG_SHA256 from the package's
# package.mk, and stores it where scripts/get_archive expects it
# (sources/<pkg>/<file> + .url and .sha256 stamps). A tarball whose checksum
# does not match is discarded, so LibreELEC's own integrity check is never
# bypassed. Failures are warnings only: LibreELEC still tries upstream itself.

set -euo pipefail

LE_DIR="${1:?usage: $0 <LibreELEC.tv dir>}"

# package.mk path (relative to LE_DIR) | fallback URL; {VERSION} is replaced
# with PKG_VERSION. The mirrors carry byte-identical copies of the upstream
# tarballs (Debian .orig, BLFS conglomeration); the sha256 check enforces it.
FALLBACKS=(
    "packages/sysutils/v4l-utils/package.mk|https://deb.debian.org/debian/pool/main/v/v4l-utils/v4l-utils_{VERSION}.orig.tar.xz"
    "packages/tools/newt/package.mk|https://ftp.osuosl.org/pub/blfs/conglomeration/newt/newt-{VERSION}.tar.gz"
)

pkg_var() {
    # Value of a plain PKG_X="..." assignment in a package.mk.
    sed -n "s/^$2=\"\(.*\)\"$/\1/p" "$1" | head -n1
}

for entry in "${FALLBACKS[@]}"; do
    mk="$LE_DIR/${entry%%|*}"
    fallback="${entry#*|}"
    if [ ! -f "$mk" ]; then
        echo "::warning::prefetch: $mk not found, skipped"
        continue
    fi

    name=$(pkg_var "$mk" PKG_NAME)
    version=$(pkg_var "$mk" PKG_VERSION)
    sha=$(pkg_var "$mk" PKG_SHA256)
    url=$(pkg_var "$mk" PKG_URL)
    if [ -z "$name" ] || [ -z "$version" ] || [ -z "$sha" ] || [ -z "$url" ]; then
        echo "::warning::prefetch: cannot read PKG_NAME/VERSION/SHA256/URL from $mk, skipped"
        continue
    fi
    url=${url//\$\{PKG_NAME\}/$name}
    url=${url//\$\{PKG_VERSION\}/$version}
    fallback=${fallback//\{VERSION\}/$version}

    dest_dir="$LE_DIR/sources/$name"
    dest="$dest_dir/$(basename "$url")"
    if [ -f "$dest" ] && [ -f "$dest.url" ] && [ "$(cat "$dest.sha256" 2>/dev/null)" = "$sha" ]; then
        echo "prefetch: $name $version already cached"
        continue
    fi

    mkdir -p "$dest_dir"
    tmp=$(mktemp "$dest_dir/.prefetch.XXXXXX")
    if ! curl -fsSL --retry 3 -o "$tmp" "$fallback"; then
        rm -f "$tmp"
        echo "::warning::prefetch: download of $fallback failed, LibreELEC will try upstream"
        continue
    fi
    got=$(sha256sum "$tmp" | cut -d' ' -f1)
    if [ "$got" != "$sha" ]; then
        rm -f "$tmp"
        echo "::warning::prefetch: $fallback has sha256 $got, expected $sha; discarded"
        continue
    fi
    chmod 644 "$tmp"
    mv "$tmp" "$dest"
    echo "$url" > "$dest.url"
    echo "$sha" > "$dest.sha256"
    echo "prefetch: $name $version from $fallback (sha256 verified)"
done
