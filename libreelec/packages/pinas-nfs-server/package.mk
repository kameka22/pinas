# SPDX-License-Identifier: GPL-3.0
# PiNAS NFS server: LibreELEC builds nfs-utils but only installs the client
# (mount.nfs). This package ships the server daemons from the same build so
# PiNAS can export folders over NFSv3/v4 (REMEDIATION_PLAN P5).
#
# Requires CONFIG_NFSD in the kernel: the build scripts apply
# libreelec/kernel/nfsd.conf to the project's kernel config.

PKG_NAME="pinas-nfs-server"
PKG_VERSION="1.0"
PKG_LICENSE="GPL-2.0-or-later"
PKG_SITE="https://github.com/kameka22/pinas"
PKG_URL=""
PKG_DEPENDS_TARGET="toolchain nfs-utils rpcbind"
PKG_LONGDESC="NFS server daemons (rpc.nfsd, rpc.mountd, exportfs, rpc.statd) for PiNAS"
PKG_TOOLCHAIN="manual"

makeinstall_target() {
  local nfs_build="$(get_build_dir nfs-utils)"
  mkdir -p ${INSTALL}/usr/sbin

  cp "${nfs_build}/.${TARGET_NAME}/utils/nfsd/nfsd"          ${INSTALL}/usr/sbin/rpc.nfsd
  cp "${nfs_build}/.${TARGET_NAME}/utils/mountd/mountd"      ${INSTALL}/usr/sbin/rpc.mountd
  cp "${nfs_build}/.${TARGET_NAME}/utils/exportfs/exportfs"  ${INSTALL}/usr/sbin/exportfs
  cp "${nfs_build}/.${TARGET_NAME}/utils/statd/statd"        ${INSTALL}/usr/sbin/rpc.statd
  cp "${nfs_build}/.${TARGET_NAME}/utils/statd/sm-notify"    ${INSTALL}/usr/sbin/sm-notify
  chmod 755 ${INSTALL}/usr/sbin/rpc.nfsd ${INSTALL}/usr/sbin/rpc.mountd ${INSTALL}/usr/sbin/exportfs \
            ${INSTALL}/usr/sbin/rpc.statd ${INSTALL}/usr/sbin/sm-notify

  # exportfs insists on /etc/exports existing; the real table is pushed with `exportfs -o`
  mkdir -p ${INSTALL}/etc
  echo "# Managed by PiNAS: exports are applied at runtime with exportfs" > ${INSTALL}/etc/exports

  # NFS state lives on the tmpfs /var
  mkdir -p ${INSTALL}/usr/lib/tmpfiles.d
  cat > ${INSTALL}/usr/lib/tmpfiles.d/pinas-nfs.conf <<'TMPFILES'
d /var/lib/nfs 0755 root root -
d /var/lib/nfs/sm 0755 root root -
d /var/lib/nfs/sm.bak 0755 root root -
d /var/lib/nfs/v4recovery 0755 root root -
d /var/lib/nfs/rpc_pipefs 0755 root root -
f /var/lib/nfs/etab 0644 root root -
f /var/lib/nfs/rmtab 0644 root root -
f /var/lib/nfs/state 0644 root root -
TMPFILES
}
