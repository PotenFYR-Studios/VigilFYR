#!/bin/sh
set -eu

REPO="PotenFYR-Studios/VigilFYR"
INSTALL_DIR="${VIGIL_INSTALL_DIR:-$HOME/.local/bin}"
DRY_RUN=0

case "${1:-}" in
  --dry-run) DRY_RUN=1 ;;
  "") ;;
  *) echo "usage: install.sh [--dry-run]" >&2; exit 2 ;;
esac

case "$(uname -s)" in
  Linux) OS=linux ;;
  Darwin) OS=macos ;;
  *) echo "unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac

case "$(uname -m)" in
  x86_64|amd64) ARCH=x86_64 ;;
  aarch64|arm64) ARCH=arm64 ;;
  armv7l) ARCH=armv7 ;;
  riscv64) ARCH=riscv64 ;;
  *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

echo "install plan: vigil-$OS-$ARCH.tar.gz -> $INSTALL_DIR"
if [ "$DRY_RUN" -eq 1 ]; then exit 0; fi

API="https://api.github.com/repos/$REPO/releases/latest"
# Artifact names come from release.yml: vigil-<platform>-<rust-target>.tar.gz
case "$OS-$ARCH" in
  linux-x86_64)  TGT="x86_64-unknown-linux-gnu" ;;
  linux-arm64)   TGT="aarch64-unknown-linux-gnu" ;;
  linux-armv7)   TGT="armv7-unknown-linux-gnueabihf" ;;
  linux-riscv64) TGT="riscv64gc-unknown-linux-gnu" ;;
  macos-x86_64)  TGT="x86_64-apple-darwin" ;;
  macos-arm64)   TGT="aarch64-apple-darwin" ;;
  *) echo "unsupported platform: $OS-$ARCH (no release artifact)" >&2; exit 1 ;;
esac
ASSET="https://github.com/$REPO/releases/latest/download/vigil-$OS-$TGT.tar.gz"
CHECKSUM="$ASSET.sha256"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "$API" > "$TMP/release.json"
curl -fL "$ASSET" -o "$TMP/vigil.tar.gz"
curl -fL "$CHECKSUM" -o "$TMP/vigil.tar.gz.sha256"

cd "$TMP"
# Checksum file contains "vigil-<platform>-<target>.tar.gz"; verify by value.
EXPECTED="$(cut -d' ' -f1 vigil.tar.gz.sha256)"
ACTUAL="$(sha256sum vigil.tar.gz 2>/dev/null | cut -d' ' -f1 || shasum -a 256 vigil.tar.gz | cut -d' ' -f1)"
if [ "$EXPECTED" != "$ACTUAL" ]; then
  echo "checksum mismatch: expected $EXPECTED, got $ACTUAL" >&2
  exit 1
fi
echo "checksum OK"
mkdir -p "$INSTALL_DIR"
tar -xzf vigil.tar.gz
find "$TMP" -type f -name vigil -perm -u+x -exec mv {} "$INSTALL_DIR/vigil" \;
chmod +x "$INSTALL_DIR/vigil"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "note: add $INSTALL_DIR to PATH" ;;
esac
echo "installed. run: vigil setup"
