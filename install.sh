#!/usr/bin/env bash
set -e

REPO="vixcho/LoveFormat"
BIN_NAME="loveformat"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64) TARGET="linux-x86_64" ;;
            aarch64|arm64) TARGET="linux-arm64" ;;
            *) echo "[!] Arquitectura no soportada: $ARCH" >&2; exit 1 ;;
        esac
        EXT="tar.gz"
        ;;
    Darwin)
        case "$ARCH" in
            x86_64) TARGET="macos-intel" ;;
            arm64) TARGET="macos-apple-silicon" ;;
            *) echo "[!] Arquitectura no soportada: $ARCH" >&2; exit 1 ;;
        esac
        EXT="tar.gz"
        ;;
    *)
        echo "[!] Sistema operativo no soportado por este instalador: $OS" >&2
        exit 1
        ;;
esac

if [ -f "Cargo.toml" ] && command -v cargo >/dev/null 2>&1; then
    cargo install --path .
    echo "[OK] LoveFormat instalado exitosamente con cargo."
    exit 0
fi

DEST_DIR="/usr/local/bin"
if [ ! -w "$DEST_DIR" ]; then
    DEST_DIR="$HOME/.local/bin"
    mkdir -p "$DEST_DIR"
fi

URL="https://github.com/${REPO}/releases/latest/download/loveformat-${TARGET}.${EXT}"
TMP_DIR="$(mktemp -d)"

cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo ":: Descargando LoveFormat para ${TARGET}..."
curl -fsSL "$URL" -o "$TMP_DIR/loveformat.${EXT}"
tar -xzf "$TMP_DIR/loveformat.${EXT}" -C "$TMP_DIR"

if [ -w "$DEST_DIR" ]; then
    install -m 755 "$TMP_DIR/$BIN_NAME" "$DEST_DIR/$BIN_NAME"
else
    sudo install -m 755 "$TMP_DIR/$BIN_NAME" "$DEST_DIR/$BIN_NAME"
fi

echo "[OK] LoveFormat instalado correctamente en $DEST_DIR/$BIN_NAME"
if [[ ":$PATH:" != *":$DEST_DIR:"* ]]; then
    echo "[!] Asegurate de que $DEST_DIR este en tu variable PATH."
fi
