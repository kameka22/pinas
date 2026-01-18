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
  if [ ! -f "${PKG_DIR}/bin/pinas" ]; then
    echo "ERROR: Backend binary not found at ${PKG_DIR}/bin/pinas"
    echo "Run scripts/build-arm64.sh first to build the backend"
    exit 1
  fi
  cp ${PKG_DIR}/bin/pinas ${INSTALL}/usr/bin/
  chmod 755 ${INSTALL}/usr/bin/pinas

  # Installer le script d'init
  cp ${PKG_DIR}/bin/pinas-init.sh ${INSTALL}/usr/bin/
  chmod 755 ${INSTALL}/usr/bin/pinas-init.sh

  # Installer les fichiers frontend (staging, sera copié vers /storage au premier boot)
  # IMPORTANT: Vérifier que le frontend existe avant de copier
  if [ ! -d "${PKG_DIR}/www" ] || [ ! -f "${PKG_DIR}/www/index.html" ]; then
    echo "ERROR: Frontend files not found at ${PKG_DIR}/www/"
    echo "Run scripts/build-arm64.sh first to build the frontend"
    echo "Expected: ${PKG_DIR}/www/index.html"
    ls -la "${PKG_DIR}/" || true
    exit 1
  fi
  mkdir -p ${INSTALL}/usr/share/pinas/www
  # Utiliser cp -r dir/. au lieu de dir/* pour éviter les problèmes de glob
  cp -r ${PKG_DIR}/www/. ${INSTALL}/usr/share/pinas/www/
  echo "Frontend installed: $(find ${INSTALL}/usr/share/pinas/www -type f | wc -l) files"

  # Installer le service systemd
  mkdir -p ${INSTALL}/usr/lib/systemd/system
  cp ${PKG_DIR}/system.d/pinas.service ${INSTALL}/usr/lib/systemd/system/

  # Activer le service au démarrage (créer le symlink directement)
  mkdir -p ${INSTALL}/usr/lib/systemd/system/default.target.wants
  ln -sf ../pinas.service ${INSTALL}/usr/lib/systemd/system/default.target.wants/pinas.service
  echo "Service enabled: pinas.service -> default.target.wants"

  # Installer la config tmpfiles (création dossiers)
  mkdir -p ${INSTALL}/usr/lib/tmpfiles.d
  cp ${PKG_DIR}/tmpfiles.d/pinas.conf ${INSTALL}/usr/lib/tmpfiles.d/
}
