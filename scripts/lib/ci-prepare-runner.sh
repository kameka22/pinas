#!/usr/bin/env bash
# Prepare a GitHub-hosted Ubuntu runner for a LibreELEC build: install the
# build dependencies and free disk space (a full image build needs well over
# the ~20 GB a fresh runner has left).
#
# Usage: scripts/lib/ci-prepare-runner.sh

set -euo pipefail

sudo apt-get update
sudo apt-get install -y \
    xfonts-utils \
    rdfind \
    libparse-yapp-perl \
    gperf \
    xsltproc \
    libxml-parser-perl \
    patchutils \
    lzop \
    bc \
    gcc-multilib \
    g++-multilib \
    libc6-dev-i386 \
    lib32gcc-s1 \
    lib32stdc++6 \
    qemu-utils \
    mtools \
    dosfstools \
    zstd

sudo rm -rf /usr/share/dotnet
sudo rm -rf /usr/local/lib/android
sudo rm -rf /opt/ghc
sudo rm -rf /opt/hostedtoolcache/CodeQL
sudo docker image prune --all --force
df -h
