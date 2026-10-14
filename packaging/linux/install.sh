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

# Stop before replacing anything, not just before restarting at the end: an
# already-running oberiz keeps executing the old binary in memory regardless
# of what the file at that path now contains, and `enable --now` does not
# restart a unit that's already active — so an upgrade over a live install
# used to report success while the old process (and old frontend contract)
# kept serving traffic until something else happened to restart it.
systemctl stop oberiz 2>/dev/null || true

install -d -m 0755 "$APP_DIR" "$DATA_DIR" "$DATA_DIR/config/indexers/custom" "$DATA_DIR/config/indexers/upstream"
install -m 0755 "$PACKAGE_DIR/bin/oberiz" "$APP_DIR/oberiz"
rm -rf "$APP_DIR/frontend"
cp -a "$PACKAGE_DIR/frontend" "$APP_DIR/frontend"
cp -an "$PACKAGE_DIR/config/." "$DATA_DIR/config/" || true
chown -R "$SERVICE_USER:$SERVICE_USER" "$DATA_DIR"

install -m 0644 "$PACKAGE_DIR/oberiz.service" /etc/systemd/system/oberiz.service
systemctl daemon-reload
systemctl enable oberiz
systemctl start oberiz

# Confirm the new process actually comes up before declaring success, rather
# than trusting that `start` returning means the service stayed healthy.
for _ in $(seq 1 10); do
  if systemctl is-active --quiet oberiz; then
    echo "Oberiz is running at http://localhost:2032"
    echo "Persistent data: $DATA_DIR"
    exit 0
  fi
  sleep 1
done
echo "Oberiz did not report as active after install/upgrade. Check: journalctl -u oberiz -e" >&2
exit 1
