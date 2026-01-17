#!/bin/bash
set -euo pipefail

# SkillSync - Installation Script
# Builds and installs the daemon as a launchd service

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BINARY_NAME="skillsync"
PLIST_NAME="com.skillsync.plist"

echo "=== SkillSync Installer ==="
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
mkdir -p "$HOME/skillsync/logs"
mkdir -p "$HOME/.gemini/skills"
mkdir -p "$HOME/.gemini/antigravity/skills"
mkdir -p "$HOME/.gemini/commands"
mkdir -p "$HOME/Library/LaunchAgents"

# Step 2: Build release binary
info "Building release binary..."
cd "$PROJECT_DIR"
cargo build --release

if [[ ! -f "target/release/$BINARY_NAME" ]]; then
    error "Build failed - binary not found"
fi

# Step 3: Stop existing service if running
if launchctl list | grep -q "com.skillsync" 2>/dev/null; then
    info "Stopping existing service..."
    launchctl stop "com.skillsync" 2>/dev/null || true
    launchctl unload "$HOME/Library/LaunchAgents/$PLIST_NAME" 2>/dev/null || true
fi

# Step 4: Install binary
info "Installing binary to /usr/local/bin..."
sudo mkdir -p /usr/local/bin
sudo cp "target/release/$BINARY_NAME" "/usr/local/bin/$BINARY_NAME"
sudo chmod +x "/usr/local/bin/$BINARY_NAME"

# Step 5: Install launchd plist with dynamic PATH
info "Installing launchd configuration..."

# Detect node/npm bin paths from multiple installation methods
# Supports: nvm, fnm, Volta, nodenv, asdf, Homebrew, npm global, system
NODE_BIN_PATHS=()

# 1. Check current shell's node (could be nvm, fnm, asdf, etc.)
if command -v node &> /dev/null; then
    CURRENT_NODE_BIN=$(node -p "process.execPath" 2>/dev/null | xargs dirname 2>/dev/null || true)
    if [[ -n "$CURRENT_NODE_BIN" ]] && [[ -d "$CURRENT_NODE_BIN" ]]; then
        NODE_BIN_PATHS+=("$CURRENT_NODE_BIN")
        info "Detected Node.js in shell: $CURRENT_NODE_BIN"
    fi
fi

# 2. Check Volta installation
if [[ -d "$HOME/.volta/bin" ]]; then
    if [[ ! " ${NODE_BIN_PATHS[@]} " =~ " $HOME/.volta/bin " ]]; then
        NODE_BIN_PATHS+=("$HOME/.volta/bin")
        info "Detected Volta: $HOME/.volta/bin"
    fi
fi

# 3. Check Homebrew (Apple Silicon)
if [[ -d "/opt/homebrew/bin" ]]; then
    if [[ ! " ${NODE_BIN_PATHS[@]} " =~ " /opt/homebrew/bin " ]]; then
        NODE_BIN_PATHS+=("/opt/homebrew/bin")
        info "Detected Homebrew (Apple Silicon): /opt/homebrew/bin"
    fi
fi

# 4. Check Homebrew (Intel)
if [[ -d "/usr/local/bin" ]]; then
    if [[ ! " ${NODE_BIN_PATHS[@]} " =~ " /usr/local/bin " ]]; then
        NODE_BIN_PATHS+=("/usr/local/bin")
    fi
fi

# Build PATH with all detected node locations
if [[ ${#NODE_BIN_PATHS[@]} -gt 0 ]]; then
    # Join paths with colons and append system paths
    LAUNCH_PATH=$(IFS=:; echo "${NODE_BIN_PATHS[*]}")
    LAUNCH_PATH="$LAUNCH_PATH:/usr/bin:/bin:/usr/sbin:/sbin"
else
    LAUNCH_PATH="/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
    warn "Node.js not detected - launchd will use default PATH"
fi

# Copy template plist and inject PATH using Python
cp "$PROJECT_DIR/resources/$PLIST_NAME" "/tmp/$PLIST_NAME.tmp"

# Insert EnvironmentVariables section before closing </dict>
python3 -c "
import sys
import re

with open('/tmp/$PLIST_NAME.tmp', 'r') as f:
    content = f.read()

if '<key>EnvironmentVariables</key>' in content:
    # Update existing PATH
    content = re.sub(
        r'(<key>PATH</key>\s*<string>)[^<]*(</string>)',
        r'\1$LAUNCH_PATH\2',
        content
    )
else:
    # Insert new EnvironmentVariables before </dict></plist>
    env_vars = '''    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>$LAUNCH_PATH</string>
    </dict>
'''
    content = content.replace('</dict>\n</plist>', env_vars + '</dict>\n</plist>')

with open('/tmp/$PLIST_NAME.tmp', 'w') as f:
    f.write(content)
"

sudo mv "/tmp/$PLIST_NAME.tmp" "$HOME/Library/LaunchAgents/$PLIST_NAME"
sudo chown "$USER:staff" "$HOME/Library/LaunchAgents/$PLIST_NAME"

# Step 6: Load and start service
info "Loading and starting service..."
launchctl load "$HOME/Library/LaunchAgents/$PLIST_NAME"
launchctl start "com.skillsync"

# Verify
sleep 1
if launchctl list | grep -q "com.skillsync"; then
    echo
    info "Installation complete!"
    echo
    echo "Service status:"
    launchctl list | grep "skillsync" || true
    echo
    echo "Logs: $HOME/skillsync/logs/skillsync.log"
    echo
    echo "Commands:"
    echo "  View logs:     tail -f ~/skillsync/logs/skillsync.log"
    echo "  Stop service:  launchctl stop com.skillsync"
    echo "  Start service: launchctl start com.skillsync"
    echo "  Uninstall:     ./scripts/uninstall.sh"
else
    warn "Service may not have started correctly. Check logs:"
    echo "  tail -f ~/skillsync/logs/skillsync.log"
fi
