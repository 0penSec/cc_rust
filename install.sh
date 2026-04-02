#!/bin/bash
set -e

# Claude Code RS Installer
# Usage: curl -sSL https://raw.githubusercontent.com/0penSec/cc_rust/main/install.sh | bash

REPO="0penSec/cc_rust"
BINARY_NAME="claude"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Detect OS and architecture
detect_platform() {
    local os arch
    
    case "$(uname -s)" in
        Linux*)     os=unknown-linux-gnu;;
        Darwin*)    os=apple-darwin;;
        CYGWIN*|MINGW*|MSYS*) os=pc-windows-msvc;;
        *)          echo "${RED}Unsupported operating system: $(uname -s)${NC}"; exit 1;;
    esac
    
    case "$(uname -m)" in
        x86_64)     arch=x86_64;;
        arm64|aarch64) arch=aarch64;;
        *)          echo "${RED}Unsupported architecture: $(uname -m)${NC}"; exit 1;;
    esac
    
    echo "${arch}-${os}"
}

# Get latest release version
get_latest_version() {
    curl -s "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"([^"]+)".*/\1/'
}

# Download and install
download_and_install() {
    local platform=$1
    local version=$2
    local asset_name
    
    if [[ "$platform" == *"windows"* ]]; then
        asset_name="claude-code-${platform}.zip"
    else
        asset_name="claude-code-${platform}.tar.gz"
    fi
    
    local download_url="https://github.com/${REPO}/releases/download/${version}/${asset_name}"
    local tmp_dir=$(mktemp -d)
    
    echo "${BLUE}Downloading ${asset_name}...${NC}"
    
    if ! curl -sL "$download_url" -o "${tmp_dir}/${asset_name}"; then
        echo "${RED}Failed to download ${asset_name}${NC}"
        rm -rf "$tmp_dir"
        exit 1
    fi
    
    echo "${BLUE}Extracting...${NC}"
    cd "$tmp_dir"
    
    if [[ "$asset_name" == *.zip ]]; then
        unzip -q "$asset_name"
    else
        tar xzf "$asset_name"
    fi
    
    # Install binary
    echo "${BLUE}Installing to ${INSTALL_DIR}...${NC}"
    if [[ -w "$INSTALL_DIR" ]]; then
        mv "$BINARY_NAME" "$INSTALL_DIR/"
    else
        sudo mv "$BINARY_NAME" "$INSTALL_DIR/"
    fi
    
    # Cleanup
    cd -
    rm -rf "$tmp_dir"
    
    echo "${GREEN}Successfully installed ${BINARY_NAME} ${version}${NC}"
}

# Main
main() {
    echo "${BLUE}=== Claude Code RS Installer ===${NC}"
    echo ""
    
    # Detect platform
    local platform
    platform=$(detect_platform)
    echo "Detected platform: ${YELLOW}${platform}${NC}"
    
    # Get version
    local version
    version=$(get_latest_version)
    if [[ -z "$version" ]]; then
        echo "${RED}Failed to get latest version${NC}"
        exit 1
    fi
    echo "Latest version: ${YELLOW}${version}${NC}"
    
    # Download and install
    download_and_install "$platform" "$version"
    
    echo ""
    echo "${GREEN}Installation complete!${NC}"
    echo ""
    echo "Run '${YELLOW}claude --help${NC}' to get started"
    echo "Set your API key: ${YELLOW}export ANTHROPIC_API_KEY='your-key'${NC}"
}

main "$@"
