#!/usr/bin/env bash
# Build a .deb package for the openhuman-core CLI binary.
# Usage: build.sh <binary_path> <version> <arch>
#   arch: amd64 | arm64
set -euo pipefail

BINARY="$1"
VERSION="$2"
ARCH="$3"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT

PKG_NAME="openhuman_${VERSION}_${ARCH}"
PKG_DIR="$WORK_DIR/$PKG_NAME"

mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/DEBIAN"

install -m 755 "$BINARY" "$PKG_DIR/usr/bin/openhuman"

sed \
  -e "s/@VERSION@/${VERSION}/g" \
  -e "s/@ARCH@/${ARCH}/g" \
  "$SCRIPT_DIR/control.in" > "$PKG_DIR/DEBIAN/control"

OUTPUT="${PKG_NAME}.deb"
dpkg-deb --build --root-owner-group "$PKG_DIR" "$OUTPUT"
echo "[deb] Built: $OUTPUT"
