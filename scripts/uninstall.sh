#!/bin/bash
set -euo pipefail

# Antigravity Sync - Uninstallation Script

PLIST_NAME="com.antigravity.sync.plist"
BINARY_NAME="antigravity"

echo "=== Antigravity Sync Uninstaller ==="
echo

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $1"; }

# Stop and unload service
if launchctl list | grep -q "com.antigravity.sync" 2>/dev/null; then
    info "Stopping service..."
    launchctl stop "com.antigravity.sync" 2>/dev/null || true
    launchctl unload "$HOME/Library/LaunchAgents/$PLIST_NAME" 2>/dev/null || true
fi

# Remove plist
if [[ -f "$HOME/Library/LaunchAgents/$PLIST_NAME" ]]; then
    info "Removing launchd configuration..."
    rm "$HOME/Library/LaunchAgents/$PLIST_NAME"
fi

# Remove binary
if [[ -f "/usr/local/bin/$BINARY_NAME" ]]; then
    info "Removing binary..."
    sudo rm "/usr/local/bin/$BINARY_NAME"
fi

echo
info "Uninstallation complete!"
echo
echo "Note: Log directory ~/antigravity/logs/ was preserved."
echo "Note: Synced files in ~/.gemini/antigravity/skills/ were preserved."
echo
echo "To remove all data:"
echo "  rm -rf ~/antigravity"
echo "  rm -rf ~/.gemini/antigravity"
