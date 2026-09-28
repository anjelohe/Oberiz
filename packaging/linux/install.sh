#!/usr/bin/env bash
set -euo pipefail

if [[ ${EUID} -ne 0 ]]; then
  echo "Run this installer with sudo."
  exit 1
fi

PACKAGE_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
APP_DIR=/opt/oberiz
DATA_DIR=/var/lib/oberiz
SERVICE_USER=oberiz

if ! id -u "$SERVICE_USER" >/dev/null 2>&1; then
  useradd --system --home "$DATA_DIR" --shell /usr/sbin/nologin "$SERVICE_USER"
fi

install -d -m 0755 "$APP_DIR" "$DATA_DIR" "$DATA_DIR/config/indexers/custom" "$DATA_DIR/config/indexers/upstream"
install -m 0755 "$PACKAGE_DIR/bin/oberiz" "$APP_DIR/oberiz"
rm -rf "$APP_DIR/frontend"
cp -a "$PACKAGE_DIR/frontend" "$APP_DIR/frontend"
cp -an "$PACKAGE_DIR/config/." "$DATA_DIR/config/" || true
chown -R "$SERVICE_USER:$SERVICE_USER" "$DATA_DIR"

install -m 0644 "$PACKAGE_DIR/oberiz.service" /etc/systemd/system/oberiz.service
systemctl daemon-reload
systemctl enable --now oberiz

echo "Oberiz is running at http://localhost:2032"
echo "Persistent data: $DATA_DIR"
