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
ASSET="https://github.com/$REPO/releases/latest/download/vigil-$OS-$ARCH.tar.gz"
CHECKSUM="$ASSET.sha256"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

curl -fsSL "$API" > "$TMP/release.json"
curl -fL "$ASSET" -o "$TMP/vigil.tar.gz"
curl -fL "$CHECKSUM" -o "$TMP/vigil.tar.gz.sha256"

cd "$TMP"
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum -c vigil.tar.gz.sha256
elif command -v shasum >/dev/null 2>&1; then
  shasum -a 256 -c vigil.tar.gz.sha256
else
  echo "no SHA-256 verifier available" >&2
  exit 1
fi
mkdir -p "$INSTALL_DIR"
tar -xzf vigil.tar.gz
find "$TMP" -type f -name vigil -perm -u+x -exec mv {} "$INSTALL_DIR/vigil" \;
chmod +x "$INSTALL_DIR/vigil"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "note: add $INSTALL_DIR to PATH" ;;
esac
echo "installed. run: vigil setup"
