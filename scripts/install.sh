#!/bin/bash
set -euo pipefail

# Antigravity Sync - Installation Script
# Builds and installs the daemon as a launchd service

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BINARY_NAME="antigravity"
PLIST_NAME="com.antigravity.sync.plist"

echo "=== Antigravity Sync Installer ==="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Check for Rust
if ! command -v cargo &> /dev/null; then
    error "Rust/Cargo not found. Install from https://rustup.rs"
fi

# Step 1: Create required directories
info "Creating directories..."
mkdir -p "$HOME/antigravity/logs"
mkdir -p "$HOME/.gemini/antigravity/skills"
mkdir -p "$HOME/Library/LaunchAgents"

# Step 2: Build release binary
info "Building release binary..."
cd "$PROJECT_DIR"
cargo build --release

if [[ ! -f "target/release/$BINARY_NAME" ]]; then
    error "Build failed - binary not found"
fi

# Step 3: Stop existing service if running
if launchctl list | grep -q "$PLIST_NAME" 2>/dev/null; then
    info "Stopping existing service..."
    launchctl stop "com.antigravity.sync" 2>/dev/null || true
    launchctl unload "$HOME/Library/LaunchAgents/$PLIST_NAME" 2>/dev/null || true
fi

# Step 4: Install binary
info "Installing binary to /usr/local/bin..."
sudo mkdir -p /usr/local/bin
sudo cp "target/release/$BINARY_NAME" "/usr/local/bin/$BINARY_NAME"
sudo chmod +x "/usr/local/bin/$BINARY_NAME"

# Step 5: Install launchd plist
info "Installing launchd configuration..."
cp "$PROJECT_DIR/$PLIST_NAME" "$HOME/Library/LaunchAgents/$PLIST_NAME"

# Step 6: Load and start service
info "Loading and starting service..."
launchctl load "$HOME/Library/LaunchAgents/$PLIST_NAME"
launchctl start "com.antigravity.sync"

# Verify
sleep 1
if launchctl list | grep -q "com.antigravity.sync"; then
    echo
    info "Installation complete!"
    echo
    echo "Service status:"
    launchctl list | grep "antigravity" || true
    echo
    echo "Logs: $HOME/antigravity/logs/antigravity.log"
    echo
    echo "Commands:"
    echo "  View logs:     tail -f ~/antigravity/logs/antigravity.log"
    echo "  Stop service:  launchctl stop com.antigravity.sync"
    echo "  Start service: launchctl start com.antigravity.sync"
    echo "  Uninstall:     ./scripts/uninstall.sh"
else
    warn "Service may not have started correctly. Check logs:"
    echo "  tail -f ~/antigravity/logs/antigravity.log"
fi
