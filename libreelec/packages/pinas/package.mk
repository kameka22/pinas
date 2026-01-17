# SPDX-License-Identifier: GPL-3.0
# PiNAS - NAS Management for LibreELEC

PKG_NAME="pinas"
PKG_VERSION="0.1.0"
PKG_LICENSE="GPL-3.0"
PKG_SITE="https://github.com/your-repo/pinas"
PKG_URL=""
PKG_DEPENDS_TARGET="toolchain"
PKG_LONGDESC="PiNAS - Modern NAS management interface for Raspberry Pi"
PKG_TOOLCHAIN="manual"

makeinstall_target() {
  # Installer le binaire
  mkdir -p ${INSTALL}/usr/bin
  cp ${PKG_DIR}/bin/pinas ${INSTALL}/usr/bin/
  chmod 755 ${INSTALL}/usr/bin/pinas

  # Installer le script d'init
  cp ${PKG_DIR}/bin/pinas-init.sh ${INSTALL}/usr/bin/
  chmod 755 ${INSTALL}/usr/bin/pinas-init.sh

  # Installer le service systemd
  mkdir -p ${INSTALL}/usr/lib/systemd/system
  cp ${PKG_DIR}/system.d/pinas.service ${INSTALL}/usr/lib/systemd/system/

  # Installer la config tmpfiles (création dossiers)
  mkdir -p ${INSTALL}/usr/lib/tmpfiles.d
  cp ${PKG_DIR}/tmpfiles.d/pinas.conf ${INSTALL}/usr/lib/tmpfiles.d/
}

post_install() {
  # Activer le service au démarrage
  enable_service pinas.service
}
