#!/bin/bash
# Kaido AI Shell - Installation Script
# https://github.com/RikaiDev/kaido

set -e

REPO="RikaiDev/kaido"
INSTALL_DIR="${KAIDO_INSTALL_DIR:-/usr/local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Detect OS and architecture
detect_platform() {
    OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    ARCH=$(uname -m)

    case "$OS" in
        linux)  OS="linux" ;;
        darwin) OS="macos" ;;
        *)      error "Unsupported OS: $OS" ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH="x64" ;;
        aarch64|arm64)  ARCH="arm64" ;;
        *)              error "Unsupported architecture: $ARCH" ;;
    esac

    PLATFORM="${OS}-${ARCH}"
    info "Detected platform: $PLATFORM"
}

# Get latest release version
get_latest_version() {
    VERSION=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    if [ -z "$VERSION" ]; then
        error "Failed to get latest version"
    fi
    info "Latest version: $VERSION"
}

# Download and install
install_kaido() {
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/kaido-${PLATFORM}.tar.gz"

    info "Downloading from: $DOWNLOAD_URL"

    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT

    curl -sL "$DOWNLOAD_URL" | tar xz -C "$TMP_DIR"

    if [ ! -f "$TMP_DIR/kaido" ]; then
        error "Download failed or archive is corrupted"
    fi

    # Check if we can write to INSTALL_DIR
    if [ -w "$INSTALL_DIR" ]; then
        mv "$TMP_DIR/kaido" "$INSTALL_DIR/"
        mv "$TMP_DIR/kaido-mcp" "$INSTALL_DIR/" 2>/dev/null || true
        info "Installed to $INSTALL_DIR"
    else
        info "Need sudo to install to $INSTALL_DIR"
        sudo mv "$TMP_DIR/kaido" "$INSTALL_DIR/"
        sudo mv "$TMP_DIR/kaido-mcp" "$INSTALL_DIR/" 2>/dev/null || true
        info "Installed to $INSTALL_DIR (with sudo)"
    fi

    chmod +x "$INSTALL_DIR/kaido"
    chmod +x "$INSTALL_DIR/kaido-mcp" 2>/dev/null || true
}

# Verify installation
verify_install() {
    if command -v kaido &> /dev/null; then
        info "Kaido installed successfully!"
        echo ""
        kaido --version
        echo ""
        info "Run 'kaido init' to configure your AI backend"
    else
        warn "Kaido installed but not in PATH"
        warn "Add $INSTALL_DIR to your PATH:"
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

main() {
    echo ""
    echo "  ╭─────────────────────────────────────╮"
    echo "  │   Kaido AI Shell - Installation     │"
    echo "  │   Your AI Ops Coach                 │"
    echo "  ╰─────────────────────────────────────╯"
    echo ""

    detect_platform
    get_latest_version
    install_kaido
    verify_install

    echo ""
    info "Documentation: https://github.com/${REPO}"
    echo ""
}

main
