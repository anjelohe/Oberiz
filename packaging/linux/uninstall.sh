#!/usr/bin/env bash
set -euo pipefail

if [[ ${EUID} -ne 0 ]]; then
  echo "Run this uninstaller with sudo."
  exit 1
fi

systemctl disable --now oberiz 2>/dev/null || true
rm -f /etc/systemd/system/oberiz.service
rm -rf /opt/oberiz
systemctl daemon-reload
echo "Oberiz application files were removed. Your data remains in /var/lib/oberiz."
