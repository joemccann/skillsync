#!/bin/bash
set -euo pipefail

# SkillSync - Uninstallation Script

PLIST_NAME="com.skillsync.plist"
BINARY_NAME="skillsync"

echo "=== SkillSync Uninstaller ==="
echo

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $1"; }

# Stop and unload service
if launchctl list | grep -q "com.skillsync" 2>/dev/null; then
    info "Stopping service..."
    launchctl stop "com.skillsync" 2>/dev/null || true
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
echo "Note: Log directory ~/skillsync/logs/ was preserved."
echo "Note: Synced files in ~/.gemini/skillsync/skills/ were preserved."
echo
echo "To remove all data:"
echo "  rm -rf ~/skillsync"
echo "  rm -rf ~/.gemini/skillsync"
