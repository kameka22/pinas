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

# Config manipulation: replace in-place if line exists, otherwise append after
# a related option. This avoids appending at EOF where make olddefconfig ignores them.
set_config_y() {
    local opt="$1"
    local config_line="CONFIG_${opt}=y"
    local not_set_line="# CONFIG_${opt} is not set"

    # If "# CONFIG_X is not set" exists, replace it in-place
    if grep -q "^${not_set_line}$" "$TARGET_CONFIG"; then
        sed -i "s|^${not_set_line}$|${config_line}|" "$TARGET_CONFIG"
    # If "CONFIG_X=" already exists, replace the value
    elif grep -q "^CONFIG_${opt}=" "$TARGET_CONFIG"; then
        sed -i "s|^CONFIG_${opt}=.*|${config_line}|" "$TARGET_CONFIG"
    else
        # Option doesn't exist at all — append at end
        echo "${config_line}" >> "$TARGET_CONFIG"
    fi
}

set_config_m() {
    local opt="$1"
    local config_line="CONFIG_${opt}=m"
    local not_set_line="# CONFIG_${opt} is not set"

    if grep -q "^${not_set_line}$" "$TARGET_CONFIG"; then
        sed -i "s|^${not_set_line}$|${config_line}|" "$TARGET_CONFIG"
    elif grep -q "^CONFIG_${opt}=" "$TARGET_CONFIG"; then
        sed -i "s|^CONFIG_${opt}=.*|${config_line}|" "$TARGET_CONFIG"
    else
        echo "${config_line}" >> "$TARGET_CONFIG"
    fi
}

set_config_off() {
    local opt="$1"
    local not_set_line="# CONFIG_${opt} is not set"

    if grep -q "^CONFIG_${opt}=" "$TARGET_CONFIG"; then
        sed -i "s|^CONFIG_${opt}=.*|${not_set_line}|" "$TARGET_CONFIG"
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
set_config_y ARCH_VEXPRESS
echo -e "${GREEN}✓${NC} ARCH_VEXPRESS enabled"

# 3. Enable PCI host controller for QEMU virt machine
# RPi5 uses Broadcom PCIE_BRCMSTB, QEMU virt uses generic ECAM PCI host
echo ">>> Enabling generic PCI host controller..."
set_config_y PCI_HOST_COMMON
set_config_y PCI_HOST_GENERIC
echo -e "${GREEN}✓${NC} PCI_HOST_GENERIC enabled"

# 4. Enable virtio drivers (transport + devices + GPU)
echo ">>> Enabling virtio drivers..."
# Parent menuconfig — gates ALL virtio options below. Without this, make olddefconfig drops them.
set_config_y VIRTIO_MENU
# Transport layer (CRITICAL — without these, QEMU virt devices are invisible)
set_config_y VIRTIO
set_config_y VIRTIO_PCI
set_config_y VIRTIO_PCI_LIB
set_config_y VIRTIO_PCI_LEGACY
set_config_y VIRTIO_MMIO
# Device drivers
set_config_y VIRTIO_BLK
set_config_y VIRTIO_NET
set_config_y VIRTIO_CONSOLE
set_config_y VIRTIO_BALLOON
set_config_y VIRTIO_INPUT
set_config_y SCSI_VIRTIO
set_config_m HW_RANDOM_VIRTIO
set_config_m VIRTIO_FS
# GPU
set_config_y DRM_VIRTIO_GPU
echo -e "${GREEN}✓${NC} Virtio drivers enabled"

# 5. Enable RAID support (disabled in RPi5 config)
echo ">>> Enabling RAID support..."
set_config_y BLK_DEV_MD
set_config_y MD_AUTODETECT
set_config_m MD_RAID0
set_config_m MD_RAID1
set_config_m MD_RAID10
set_config_m MD_RAID456
echo -e "${GREEN}✓${NC} RAID support enabled"

# 6. Disable RPi/SoC-specific drivers (not needed in VM)
echo ">>> Disabling RPi-specific drivers..."
for opt in ARCH_BCM ARCH_BCM2835 ARCH_BRCMSTB \
           BCM2835_WDT BCM2835_THERMAL BCM2711_THERMAL \
           SERIAL_8250_BCM2835AUX BCM2835_VCHIQ \
           SND_BCM2835_SOC_I2S DRM_VC4; do
    set_config_off "$opt"
done
echo -e "${GREEN}✓${NC} RPi-specific drivers disabled"

# 7. Verify critical options are set
echo ""
echo ">>> Verifying critical options..."
MISSING=0
for opt in PCI_HOST_GENERIC VIRTIO VIRTIO_PCI VIRTIO_BLK VIRTIO_NET VIRTIO_MMIO DRM_VIRTIO_GPU PCI SERIAL_AMBA_PL011 ARM_GIC; do
    if grep -q "^CONFIG_${opt}=y" "$TARGET_CONFIG"; then
        echo -e "    ${GREEN}✓${NC} CONFIG_${opt}=y"
    elif grep -q "^CONFIG_${opt}=m" "$TARGET_CONFIG"; then
        echo -e "    ${GREEN}✓${NC} CONFIG_${opt}=m"
    else
        echo -e "    ${RED}✗${NC} CONFIG_${opt} MISSING"
        MISSING=$((MISSING + 1))
    fi
done

if [ $MISSING -gt 0 ]; then
    echo -e "${RED}Warning: $MISSING critical option(s) missing!${NC}"
else
    echo -e "${GREEN}All critical options verified${NC}"
fi

echo ""
echo "=== Kernel Config Generated ==="
echo "Output: ${TARGET_CONFIG}"
echo ""
echo "Note: The config uses RPi5 as base with LINUX=\"default\" (mainline kernel)."
echo "Options not present in mainline will be resolved by 'make olddefconfig'"
echo "during the LibreELEC build process."
