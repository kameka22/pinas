#!/bin/bash
# Generate VM kernel config from RPi5 base config
# One-shot script: generates libreelec/projects/Virtual/linux/linux.aarch64.conf
# Re-run when rebasing on new LibreELEC versions

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
LIBREELEC_DIR="${PROJECT_ROOT}/extra/LibreELEC.tv"
SOURCE_CONFIG="${LIBREELEC_DIR}/projects/RPi/devices/RPi5/linux/linux.aarch64.conf"
TARGET_DIR="${PROJECT_ROOT}/libreelec/projects/Virtual/linux"
TARGET_CONFIG="${TARGET_DIR}/linux.aarch64.conf"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check that LibreELEC source is cloned
if [ ! -d "$LIBREELEC_DIR" ]; then
    echo -e "${RED}Error: LibreELEC source not found at ${LIBREELEC_DIR}${NC}"
    echo "Clone it first:"
    echo "  mkdir -p ${PROJECT_ROOT}/extra"
    echo "  git clone https://github.com/LibreELEC/LibreELEC.tv.git ${LIBREELEC_DIR}"
    exit 1
fi

# Check that RPi5 config exists
if [ ! -f "$SOURCE_CONFIG" ]; then
    echo -e "${RED}Error: RPi5 kernel config not found at ${SOURCE_CONFIG}${NC}"
    echo "Make sure you're on the correct LibreELEC branch (libreelec-12.2)"
    exit 1
fi

# Use scripts/config from LibreELEC kernel source if available, otherwise use sed
SCRIPTS_CONFIG=""
KERNEL_SRC=$(find "${LIBREELEC_DIR}/build.LibreELEC-"*/linux-* -maxdepth 0 -type d 2>/dev/null | head -1)
if [ -n "$KERNEL_SRC" ] && [ -f "${KERNEL_SRC}/scripts/config" ]; then
    SCRIPTS_CONFIG="${KERNEL_SRC}/scripts/config"
    echo "Using kernel scripts/config from: ${SCRIPTS_CONFIG}"
fi

# Helper functions for config manipulation
enable_config() {
    local opt="$1"
    if [ -n "$SCRIPTS_CONFIG" ]; then
        "$SCRIPTS_CONFIG" --file "$TARGET_CONFIG" --enable "$opt"
    else
        # Remove any existing line for this option
        sed -i "/^CONFIG_${opt}[ =]/d" "$TARGET_CONFIG"
        sed -i "/^# CONFIG_${opt} is not set/d" "$TARGET_CONFIG"
        echo "CONFIG_${opt}=y" >> "$TARGET_CONFIG"
    fi
}

module_config() {
    local opt="$1"
    if [ -n "$SCRIPTS_CONFIG" ]; then
        "$SCRIPTS_CONFIG" --file "$TARGET_CONFIG" --module "$opt"
    else
        sed -i "/^CONFIG_${opt}[ =]/d" "$TARGET_CONFIG"
        sed -i "/^# CONFIG_${opt} is not set/d" "$TARGET_CONFIG"
        echo "CONFIG_${opt}=m" >> "$TARGET_CONFIG"
    fi
}

disable_config() {
    local opt="$1"
    if [ -n "$SCRIPTS_CONFIG" ]; then
        "$SCRIPTS_CONFIG" --file "$TARGET_CONFIG" --disable "$opt"
    else
        sed -i "/^CONFIG_${opt}[ =]/d" "$TARGET_CONFIG"
        sed -i "/^# CONFIG_${opt} is not set/d" "$TARGET_CONFIG"
        echo "# CONFIG_${opt} is not set" >> "$TARGET_CONFIG"
    fi
}

echo "=== Generating VM Kernel Config ==="
echo "Source: ${SOURCE_CONFIG}"
echo "Target: ${TARGET_CONFIG}"
echo ""

# 1. Copy RPi5 config as base
mkdir -p "$TARGET_DIR"
cp "$SOURCE_CONFIG" "$TARGET_CONFIG"
echo -e "${GREEN}✓${NC} Copied RPi5 config as base"

# 2. Enable QEMU virt platform
echo ">>> Enabling QEMU virt platform..."
enable_config ARCH_VEXPRESS
echo -e "${GREEN}✓${NC} ARCH_VEXPRESS enabled"

# 3. Enable virtio drivers
echo ">>> Enabling virtio drivers..."
for opt in VIRTIO VIRTIO_PCI VIRTIO_BLK VIRTIO_NET VIRTIO_CONSOLE \
           VIRTIO_MMIO VIRTIO_INPUT VIRTIO_BALLOON SCSI_VIRTIO \
           DRM_VIRTIO_GPU; do
    enable_config "$opt"
done
module_config HW_RANDOM_VIRTIO
module_config VIRTIO_FS
echo -e "${GREEN}✓${NC} Virtio drivers enabled"

# 4. Enable RAID support (disabled in RPi5 config)
echo ">>> Enabling RAID support..."
for opt in BLK_DEV_MD MD_AUTODETECT; do
    enable_config "$opt"
done
for opt in MD_RAID0 MD_RAID1 MD_RAID10 MD_RAID456; do
    module_config "$opt"
done
echo -e "${GREEN}✓${NC} RAID support enabled"

# 5. Disable RPi/SoC-specific drivers (not needed in VM)
echo ">>> Disabling RPi-specific drivers..."
for opt in ARCH_BCM ARCH_BCM2835 ARCH_BRCMSTB \
           BCM2835_WDT BCM2835_THERMAL BCM2711_THERMAL \
           SERIAL_8250_BCM2835AUX BCM2835_VCHIQ \
           SND_BCM2835_SOC_I2S DRM_VC4; do
    disable_config "$opt"
done
echo -e "${GREEN}✓${NC} RPi-specific drivers disabled"

echo ""
echo "=== Kernel Config Generated ==="
echo "Output: ${TARGET_CONFIG}"
echo ""
echo "Note: The config uses RPi5 as base with LINUX=\"default\" (mainline kernel)."
echo "Options not present in mainline will be resolved by 'make olddefconfig'"
echo "during the LibreELEC build process."
