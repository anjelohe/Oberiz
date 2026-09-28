#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/backend/Cargo.toml" | head -n 1)
ARCH=$(uname -m)
PACKAGE="oberiz-${VERSION}-linux-${ARCH}"
STAGING="$ROOT/dist/$PACKAGE"

cd "$ROOT/frontend"
npm ci
npm run build

cd "$ROOT/backend"
cargo build --release --locked

rm -rf "$STAGING"
mkdir -p "$STAGING/bin" "$STAGING/config/indexers/custom" "$STAGING/config/indexers/upstream"
cp "$ROOT/backend/target/release/oberiz" "$STAGING/bin/oberiz"
cp -a "$ROOT/frontend/dist" "$STAGING/frontend"
cp -a "$ROOT/config/." "$STAGING/config/"
cp "$ROOT/packaging/linux/install.sh" "$ROOT/packaging/linux/uninstall.sh" "$ROOT/packaging/linux/oberiz.service" "$STAGING/"
chmod +x "$STAGING/install.sh" "$STAGING/uninstall.sh" "$STAGING/bin/oberiz"

tar -C "$ROOT/dist" -czf "$ROOT/dist/$PACKAGE.tar.gz" "$PACKAGE"
echo "Created dist/$PACKAGE.tar.gz"
