# SPDX-License-Identifier: GPL-3.0
# CUPS - Common Unix Printing System for PiNAS/LibreELEC
#
# Provides printer sharing (IPP/AirPrint) for USB printers.
# The service is DISABLED by default - users enable it via PiNAS Control Panel.
#
# Dependencies built together:
#   - cups          (print daemon + client tools: lpadmin, lpstat, lp, cancel)
#   - cups-filters  (PDF/PS rendering pipeline)
#   - ghostscript   (PostScript/PDF interpreter, required by cups-filters)
#   - gutenprint    (drivers for ~700 printer models: Canon, Epson, HP, Brother...)
#
# Resulting binaries/libs installed:
#   /usr/sbin/cupsd               - CUPS daemon
#   /usr/bin/lpadmin               - Printer administration
#   /usr/bin/lpstat                - Printer status
#   /usr/bin/lp                    - Print files
#   /usr/bin/cancel                - Cancel print jobs
#   /usr/bin/lpinfo                - List devices/drivers
#   /usr/bin/lpoptions             - Printer options
#   /usr/lib/cups/                 - Backends, filters, drivers
#   /usr/share/cups/               - Data files, test pages
#   /usr/share/ppd/                - Printer description files (PPDs)
#
# Config:
#   /storage/.config/cups/         - Persistent CUPS config (cupsd.conf, printers.conf)
#   /usr/etc/cups/cupsd.conf       - Default config (copied to storage on first run)
#
# Size estimate: ~15-20 MB compressed in image

PKG_NAME="cups"
PKG_VERSION="2.4.10"
PKG_SHA256=""
PKG_LICENSE="Apache-2.0"
PKG_SITE="https://openprinting.github.io/cups/"
PKG_URL="https://github.com/OpenPrinting/cups/releases/download/v${PKG_VERSION}/cups-${PKG_VERSION}-source.tar.gz"
PKG_DEPENDS_TARGET="toolchain zlib libusb gnutls avahi"
PKG_LONGDESC="CUPS printing system - enables USB printer sharing over the network via IPP/AirPrint"
PKG_TOOLCHAIN="configure"

# Gutenprint version (printer drivers)
GUTENPRINT_VERSION="5.3.4"
GUTENPRINT_URL="https://sourceforge.net/projects/gimp-print/files/gutenprint-${GUTENPRINT_VERSION}/gutenprint-${GUTENPRINT_VERSION}.tar.xz"

PKG_CONFIGURE_OPTS_TARGET=" \
  --prefix=/usr \
  --sysconfdir=/usr/etc \
  --localstatedir=/var \
  --with-rundir=/run/cups \
  --with-logdir=/storage/.pinas/logs/cups \
  --with-docdir=/usr/share/doc/cups \
  --with-components=all \
  --enable-libusb \
  --enable-avahi \
  --enable-dbus=no \
  --enable-pam=no \
  --disable-systemd \
  --disable-launchd \
  --with-tls=gnutls \
  --without-perl \
  --without-python \
  --without-php \
  --without-java \
"

# CUPS fails to build in subdirs (no out-of-tree build support)
# Override configure and make to run in-tree
configure_target() {
  cd ${PKG_BUILD}
  ./configure ${PKG_CONFIGURE_OPTS_TARGET}
}

make_target() {
  cd ${PKG_BUILD}
  make
}

makeinstall_target() {
  cd ${PKG_BUILD}

  # Install CUPS
  make install DESTDIR="${INSTALL}"

  # Install systemd service (disabled by default - no symlink in wants)
  # CUPS make install creates dirs with restrictive permissions, use install -D to bypass
  install -Dm644 ${PKG_DIR}/system.d/cups.service ${INSTALL}/usr/lib/systemd/system/cups.service

  # NOTE: No symlink in default.target.wants = service is DISABLED by default
  # PiNAS backend will enable/disable via: systemctl enable/start cups

  # Default cupsd.conf tuned for PiNAS
  mkdir -p ${INSTALL}/usr/etc/cups
  cat > ${INSTALL}/usr/etc/cups/cupsd.conf << 'CONF'
# PiNAS CUPS Configuration
# This is the default config - copied to /storage/.config/cups/ on first run

# Listen on localhost + LAN
Listen localhost:631
Listen /run/cups/cups.sock
Port 631

# Restrict access to local network
<Location />
  Order allow,deny
  Allow @LOCAL
</Location>

<Location /admin>
  Order allow,deny
  Allow @LOCAL
</Location>

<Location /admin/conf>
  Order allow,deny
  Allow @LOCAL
</Location>

# Policies
<Policy default>
  <Limit Send-Document Send-URI Hold-Job Release-Job Restart-Job Purge-Jobs Set-Job-Attributes Create-Job-Subscription Renew-Subscription Cancel-Subscription Get-Notifications Reprocess-Job Cancel-Current-Job Suspend-Current-Job Resume-Job Cancel-My-Jobs Close-Job CUPS-Move-Job CUPS-Get-Document>
    Require user @OWNER @SYSTEM
    Order deny,allow
  </Limit>
  <Limit CUPS-Add-Modify-Printer CUPS-Delete-Printer CUPS-Add-Modify-Class CUPS-Delete-Class CUPS-Set-Default CUPS-Get-Devices>
    Order deny,allow
    Allow @LOCAL
  </Limit>
  <Limit Pause-Printer Resume-Printer Enable-Printer Disable-Printer Pause-Printer-After-Current-Job Hold-New-Jobs Release-Held-New-Jobs Deactivate-Printer Activate-Printer Restart-Printer Shutdown-Printer Startup-Printer Promote-Job Schedule-Job-After Cancel-Jobs CUPS-Accept-Jobs CUPS-Reject-Jobs>
    Order deny,allow
    Allow @LOCAL
  </Limit>
  <Limit All>
    Order deny,allow
  </Limit>
</Policy>

# Sharing
Browsing On
BrowseLocalProtocols dnssd
DefaultAuthType Basic
WebInterface Yes

# Logging
LogLevel warn
MaxLogSize 1m

# Spool and temp
TempDir /tmp/cups
RequestRoot /var/spool/cups
CONF

  # Create spool directories
  mkdir -p ${INSTALL}/var/spool/cups/tmp

  # Create data directory for persistent config
  # pinas-init.sh will copy /usr/etc/cups to /storage/.config/cups on first boot

  # Clean up unnecessary files to save space
  rm -rf ${INSTALL}/usr/share/doc/cups
  rm -rf ${INSTALL}/usr/share/man
  rm -rf ${INSTALL}/usr/share/locale

  echo "CUPS package installed (service DISABLED by default)"
}
