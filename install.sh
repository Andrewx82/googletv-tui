#!/bin/sh
set -e

# Google TV Remote TUI Universal Linux Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/Andrewx82/googletv-tui/main/install.sh | sh

REPO="Andrewx82/googletv-tui"
BINARY_NAME="googletv-tui"

# ANSI colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

# Handle uninstall flag
if [ "$1" = "--uninstall" ] || [ "$1" = "uninstall" ]; then
    printf "${BLUE}${BOLD}==>${NC} ${BOLD}Uninstalling Google TV Remote TUI (${BINARY_NAME})...${NC}\n"
    REMOVED=0
    for LOC in "/usr/local/bin/${BINARY_NAME}" "$HOME/.local/bin/${BINARY_NAME}"; do
        if [ -f "$LOC" ]; then
            rm -f "$LOC"
            printf "${GREEN}✓ Removed %s${NC}\n" "$LOC"
            REMOVED=1
        fi
    done
    if [ "$REMOVED" -eq 0 ]; then
        printf "${YELLOW}Binary not found in standard paths (/usr/local/bin, ~/.local/bin).${NC}\n"
    fi
    printf "\nTo also remove saved devices, pairing certificates, and settings:\n"
    printf "  rm -rf ~/.config/googletv-tui\n\n"
    exit 0
fi

printf "${BLUE}${BOLD}==>${NC} ${BOLD}Installing / Updating Google TV Remote TUI (${BINARY_NAME})...${NC}\n"

# 1. Detect OS & Architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

if [ "$OS" != "linux" ]; then
    printf "${RED}Error: This installer currently supports Linux only. Detected: %s${NC}\n" "$OS" >&2
    exit 1
fi

case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="linux-x86_64"
        ;;
    *)
        printf "${RED}Error: Precompiled binaries are currently built for x86_64. Detected: %s${NC}\n" "$ARCH" >&2
        printf "You can build from source with: cargo install --git https://github.com/%s.git\n" "$REPO" >&2
        exit 1
        ;;
esac

# 2. Determine installation directory
if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
elif [ "$(id -u)" -eq 0 ]; then
    INSTALL_DIR="/usr/local/bin"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

# 3. Determine download URL (latest release)
LATEST_TAG="$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || echo "v0.2.2")"
if [ -z "$LATEST_TAG" ]; then
    LATEST_TAG="v0.2.2"
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${BINARY_NAME}-${TARGET_ARCH}"
TARGET_FILE="${INSTALL_DIR}/${BINARY_NAME}"

printf "${BLUE}==>${NC} Downloading ${BOLD}%s (%s)${NC} from release %s...\n" "$BINARY_NAME" "$TARGET_ARCH" "$LATEST_TAG"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${BINARY_NAME}"
elif command -v wget >/dev/null 2>&1; then
    wget -qO "${TMP_DIR}/${BINARY_NAME}" "$DOWNLOAD_URL"
else
    printf "${RED}Error: curl or wget is required to download the binary.${NC}\n" >&2
    exit 1
fi

chmod +x "${TMP_DIR}/${BINARY_NAME}"
mv "${TMP_DIR}/${BINARY_NAME}" "$TARGET_FILE"

printf "${GREEN}${BOLD}✓ Installed successfully to: %s${NC}\n\n" "$TARGET_FILE"

# 4. Check PATH
case ":$PATH:" in
    *":${INSTALL_DIR}:"*)
        ;;
    *)
        printf "${YELLOW}Note:${NC} %s is not in your PATH.\n" "$INSTALL_DIR"
        printf "Add it to your shell configuration (e.g. ~/.bashrc or ~/.zshrc):\n"
        printf "  export PATH=\"%s:\$PATH\"\n\n" "$INSTALL_DIR"
        ;;
esac

printf "${BOLD}To start using Google TV Remote TUI right now:${NC}\n"
printf "  ${GREEN}%s${NC}\n\n" "$BINARY_NAME"
