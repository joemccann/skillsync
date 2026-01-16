# Antigravity

A macOS daemon that mirrors Claude skills to Gemini in real-time.

Claude is the source of truth. Any change in `~/.claude/skills/` is automatically synced to `~/.gemini/antigravity/skills/`.

## Features

- **Real-time sync** using macOS FSEvents
- **Initial sync** on startup copies all existing files
- **Deletion sync** removes files when deleted from source
- **Orphan cleanup** removes destination files not in source
- **Debouncing** batches rapid changes (100ms window)
- **Structured logging** to `~/antigravity/logs/`
- **launchd integration** for auto-start on login

## Installation

### Prerequisites

- macOS
- [Rust](https://rustup.rs)

### Quick Install

```bash
git clone https://github.com/joemccann/claude-antigravity-sync.git
cd claude-antigravity-sync
./scripts/install.sh
```

This will:
1. Build the release binary
2. Install to `/usr/local/bin/antigravity`
3. Configure launchd for auto-start
4. Start the service

### Manual Install

```bash
# Build
cargo build --release

# Install binary
sudo cp target/release/antigravity /usr/local/bin/
sudo chmod +x /usr/local/bin/antigravity

# Create directories
mkdir -p ~/antigravity/logs
mkdir -p ~/.gemini/antigravity/skills

# Install and start launchd service
cp com.antigravity.sync.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.antigravity.sync.plist
launchctl start com.antigravity.sync
```

## Usage

Once installed, Antigravity runs automatically in the background.

### View Logs

```bash
tail -f ~/antigravity/logs/antigravity.log
```

### Service Management

```bash
# Stop
launchctl stop com.antigravity.sync

# Start
launchctl start com.antigravity.sync

# Restart (reload config)
launchctl unload ~/Library/LaunchAgents/com.antigravity.sync.plist
launchctl load ~/Library/LaunchAgents/com.antigravity.sync.plist

# Check status
launchctl list | grep antigravity
```

### Uninstall

```bash
./scripts/uninstall.sh
```

Or manually:

```bash
launchctl stop com.antigravity.sync
launchctl unload ~/Library/LaunchAgents/com.antigravity.sync.plist
rm ~/Library/LaunchAgents/com.antigravity.sync.plist
sudo rm /usr/local/bin/antigravity
```

## Paths

| Path | Purpose |
|------|---------|
| `~/.claude/skills/` | Source (watched) |
| `~/.gemini/antigravity/skills/` | Destination (synced) |
| `~/antigravity/logs/antigravity.log` | Application logs |
| `/usr/local/bin/antigravity` | Installed binary |
| `~/Library/LaunchAgents/com.antigravity.sync.plist` | launchd config |

## License

MIT
