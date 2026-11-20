#!/usr/bin/env bash
# Run LibreELEC's `make image` in CI, with retries and an optional time budget.
#
# Usage: scripts/lib/ci-make-image.sh <LibreELEC.tv dir> [deadline]
#   deadline  Unix timestamp after which the build is stopped so the job can
#             still hand its build state over to the next job before GitHub's
#             6-hour job limit. Without it, the build runs until it ends.
#
# Exit status: 0 image built, 3 deadline reached (build state is resumable),
# anything else a real failure.
#
# LibreELEC fetches ~300 tarballs from upstream mirrors and a single failed
# download aborts `make image`. The build is incremental (per-package stamps),
# so a re-run resumes where it stopped: retry a few times. PROJECT, DEVICE and
# ARCH come from the environment, as for a manual build.

set -uo pipefail

LE_DIR="${1:?usage: $0 <LibreELEC.tv dir> [deadline]}"
DEADLINE="${2:-}"
TIME_UP=3

cd "$LE_DIR" || exit 1

for attempt in 1 2 3; do
    if [ -n "$DEADLINE" ]; then
        remaining=$(( DEADLINE - $(date +%s) ))
        if [ "$remaining" -le 60 ]; then
            echo "::notice::time budget used up before attempt ${attempt}: build state will be resumed by the next job"
            exit "$TIME_UP"
        fi
        echo "make image (attempt ${attempt}/3), $(( remaining / 60 )) min left in this job"
        # timeout signals the whole process group (make, pkgbuilder, compilers).
        timeout --kill-after=5m "${remaining}s" make image
        status=$?
        if [ "$status" -eq 124 ] || [ "$status" -eq 137 ]; then
            echo "::notice::time budget reached: build state will be resumed by the next job"
            exit "$TIME_UP"
        fi
    else
        echo "make image (attempt ${attempt}/3)"
        make image
        status=$?
    fi
    [ "$status" -eq 0 ] && exit 0
    if [ "$attempt" -lt 3 ]; then
        echo "::warning::make image failed (attempt ${attempt}/3), retrying in 60 s"
        sleep 60
    fi
done

echo "::error::make image failed after 3 attempts"
exit 1
