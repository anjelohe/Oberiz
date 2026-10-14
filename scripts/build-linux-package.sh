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
# Statically linked against musl, not the build host's glibc, when
# CARGO_BUILD_TARGET names a musl target: a binary dynamically linked
# against glibc only runs on a host whose glibc is at least as new as the
# one it was built against, and a CI runner's glibc tends to be newer than
# what many still-supported distributions ship — the actual minimum was
# never verified for this package before. A musl build has no such minimum
# to track; it carries its own libc. Falls back to an ordinary native build
# (unchanged from before) when the variable isn't set, so local/manual use
# of this script doesn't require the musl toolchain to be installed.
if [[ -n "${CARGO_BUILD_TARGET:-}" ]]; then
  rustup target add "$CARGO_BUILD_TARGET" >/dev/null 2>&1 || true
  cargo build --release --locked --target "$CARGO_BUILD_TARGET"
  BINARY="target/$CARGO_BUILD_TARGET/release/oberiz"
else
  cargo build --release --locked
  BINARY="target/release/oberiz"
fi

rm -rf "$STAGING"
mkdir -p "$STAGING/bin" "$STAGING/config/indexers/custom" "$STAGING/config/indexers/upstream"
cp "$ROOT/backend/$BINARY" "$STAGING/bin/oberiz"
cp -a "$ROOT/frontend/dist" "$STAGING/frontend"
cp -a "$ROOT/config/." "$STAGING/config/"
cp "$ROOT/packaging/linux/install.sh" "$ROOT/packaging/linux/uninstall.sh" "$ROOT/packaging/linux/oberiz.service" "$STAGING/"
chmod +x "$STAGING/install.sh" "$STAGING/uninstall.sh" "$STAGING/bin/oberiz"

tar -C "$ROOT/dist" -czf "$ROOT/dist/$PACKAGE.tar.gz" "$PACKAGE"
echo "Created dist/$PACKAGE.tar.gz"
