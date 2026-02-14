#!/bin/bash
# Launch PiNAS ARM64 VM with QEMU (direct kernel boot)
# Requires: qemu-system-aarch64
#
# Usage:
#   ./scripts/run-vm-qemu.sh                          # Auto-detect from target/
#   ./scripts/run-vm-qemu.sh <KERNEL> <DISK>          # Explicit paths
#
# Environment variables:
#   PINAS_VM_RAM=2048    # RAM in MB (default: 2048)
#   PINAS_VM_CPUS=2      # CPU cores (default: 2)
#
# Port forwards:
#   3000 → 3000  (PiNAS web UI)
#   2222 → 22    (SSH)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TARGET_DIR="${PROJECT_ROOT}/target"

KERNEL="${1:-$(ls -t ${TARGET_DIR}/*-KERNEL 2>/dev/null | head -1)}"
DISK="${2:-$(ls -t ${TARGET_DIR}/*.qcow2 2>/dev/null | head -1)}"

# Check qemu is installed
if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "Error: qemu-system-aarch64 not found"
    echo ""
    echo "Install QEMU:"
    echo "  macOS:  brew install qemu"
    echo "  Ubuntu: sudo apt-get install qemu-system-arm"
    exit 1
fi

# Validate kernel
if [ -z "$KERNEL" ] || [ ! -f "$KERNEL" ]; then
    echo "Error: Kernel file not found"
    echo "  Looked in: ${TARGET_DIR}/*-KERNEL"
    echo ""
    echo "Usage: $0 [KERNEL] [DISK]"
    echo "  Build first: ./scripts/build-arm64-vm.sh"
    exit 1
fi

# Validate disk — try qcow2, then raw .img
DISK_FORMAT="qcow2"
if [ -z "$DISK" ] || [ ! -f "$DISK" ]; then
    DISK=$(ls -t ${TARGET_DIR}/*.img 2>/dev/null | head -1)
    DISK_FORMAT="raw"
fi

if [ -z "$DISK" ] || [ ! -f "$DISK" ]; then
    echo "Error: Disk image not found"
    echo "  Looked in: ${TARGET_DIR}/*.qcow2 and ${TARGET_DIR}/*.img"
    echo ""
    echo "Usage: $0 [KERNEL] [DISK]"
    echo "  Build first: ./scripts/build-arm64-vm.sh"
    exit 1
fi

if [[ "$DISK" == *.img ]]; then
    DISK_FORMAT="raw"
fi

RAM="${PINAS_VM_RAM:-2048}"
CPUS="${PINAS_VM_CPUS:-2}"

echo "PiNAS ARM64 VM"
echo "  Kernel: $KERNEL"
echo "  Disk:   $DISK ($DISK_FORMAT)"
echo "  RAM:    ${RAM}MB"
echo "  CPUs:   $CPUS"
echo "  Ports:  3000->3000 (web), 2222->22 (ssh)"
echo ""
echo "Press Ctrl+A X to exit QEMU"
echo ""

qemu-system-aarch64 \
    -machine virt \
    -cpu cortex-a72 \
    -smp ${CPUS} -m ${RAM} \
    -kernel "${KERNEL}" \
    -append "boot=LABEL=LIBREELEC disk=LABEL=STORAGE quiet console=ttyAMA0,115200" \
    -drive file="${DISK}",format=${DISK_FORMAT},if=virtio \
    -netdev user,id=net0,hostfwd=tcp::3000-:3000,hostfwd=tcp::2222-:22 \
    -device virtio-net-pci,netdev=net0 \
    -nographic
