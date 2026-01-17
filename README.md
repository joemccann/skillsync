# SkillSync

A macOS daemon that mirrors Claude skills to Gemini in real-time.

Claude is the source of truth. Any change in `~/.claude/skills/` is automatically synced to three destinations with tool-specific transformations:
- `~/.gemini/skills/` (Claude-style, preserves YAML frontmatter)
- `~/.gemini/antigravity/skills/` (Claude-style, preserves YAML frontmatter)
- `~/.gemini/commands/` (Gemini CLI TOML; strips YAML, wraps prompt)

## Features

- **Real-time sync** using macOS FSEvents
- **Initial sync** on startup copies all existing files
- **Deletion sync** removes files when deleted from source
- **Tool-specific transforms** for Gemini CLI (YAML frontmatter parsing/stripping, TOML generation)
- **Orphan cleanup** removes destination files not in source (including reverse-mapped TOML)
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
3. Install `resources/com.skillsync.plist` to `~/Library/LaunchAgents/`
4. Configure launchd for auto-start and start the service

### Manual Install

```bash
# Build
cargo build --release

# Install binary
sudo cp target/release/skillsync /usr/local/bin/
sudo chmod +x /usr/local/bin/skillsync

# Create directories (optional; daemon will create if missing)
mkdir -p ~/skillsync/logs
mkdir -p ~/.gemini/skills
mkdir -p ~/.gemini/antigravity/skills
mkdir -p ~/.gemini/commands

# Install and start launchd service
cp resources/com.skillsync.plist ~/Library/LaunchAgents/
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

## Testing

- Run all tests (unit + integration):

```bash
cargo test
```

- Run only unit tests (module-level tests inside src):

```bash
cargo test --lib
```

- Run only integration tests (in tests/):

```bash
cargo test --test integration_tests
```

- Run a specific test by name (substring match):

```bash
cargo test test_generate_toml_with_description
```

- Quick manual validation (one-time sync; 3s timeout):

```bash
./scripts/test_sync.sh
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
| `~/.gemini/skills/` | Destination (Claude-style) |
| `~/.gemini/antigravity/skills/` | Destination (Claude-style) |
| `~/.gemini/commands/` | Destination (Gemini CLI TOML) |
| `~/skillsync/logs/skillsync.log` | Application logs |
| `/usr/local/bin/skillsync` | Installed binary |
| `~/Library/LaunchAgents/com.skillsync.plist` | launchd config |

## License

MIT
