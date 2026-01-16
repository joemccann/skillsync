# SkillSync

A macOS daemon that mirrors Claude skills to Gemini in real-time.

Claude is the source of truth. Any change in `~/.claude/skills/` is automatically synced to `~/.gemini/skillsync/skills/`.

## Features

- **Real-time sync** using macOS FSEvents
- **Initial sync** on startup copies all existing files
- **Deletion sync** removes files when deleted from source
- **Orphan cleanup** removes destination files not in source
- **Debouncing** batches rapid changes (100ms window)
- **Structured logging** to `~/skillsync/logs/`
- **launchd integration** for auto-start on login

## Installation

### Prerequisites

- macOS
- [Rust](https://rustup.rs)

### Quick Install

```bash
git clone https://github.com/joemccann/skillsync.git
cd skillsync
./scripts/install.sh
```

This will:
1. Build the release binary
2. Install to `/usr/local/bin/skillsync`
3. Configure launchd for auto-start
4. Start the service

### Manual Install

```bash
# Build
cargo build --release

# Install binary
sudo cp target/release/skillsync /usr/local/bin/
sudo chmod +x /usr/local/bin/skillsync

# Create directories
mkdir -p ~/skillsync/logs
mkdir -p ~/.gemini/skillsync/skills

# Install and start launchd service
cp com.skillsync.plist ~/Library/LaunchAgents/
launchctl load ~/Library/LaunchAgents/com.skillsync.plist
launchctl start com.skillsync
```

## Usage

Once installed, SkillSync runs automatically in the background.

### View Logs

```bash
tail -f ~/skillsync/logs/skillsync.log
```

### Service Management

```bash
# Stop
launchctl stop com.skillsync

# Start
launchctl start com.skillsync

# Restart (reload config)
launchctl unload ~/Library/LaunchAgents/com.skillsync.plist
launchctl load ~/Library/LaunchAgents/com.skillsync.plist

# Check status
launchctl list | grep skillsync
```

### Uninstall

```bash
./scripts/uninstall.sh
```

Or manually:

```bash
launchctl stop com.skillsync
launchctl unload ~/Library/LaunchAgents/com.skillsync.plist
rm ~/Library/LaunchAgents/com.skillsync.plist
sudo rm /usr/local/bin/skillsync
```

## Paths

| Path | Purpose |
|------|---------|
| `~/.claude/skills/` | Source (watched) |
| `~/.gemini/skillsync/skills/` | Destination (synced) |
| `~/skillsync/logs/skillsync.log` | Application logs |
| `/usr/local/bin/skillsync` | Installed binary |
| `~/Library/LaunchAgents/com.skillsync.plist` | launchd config |

## License

MIT
