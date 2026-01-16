# Antigravity Sync Design

A macOS background daemon that mirrors Claude skills to Gemini.

## Overview

**Purpose:** Sync `~/.claude/skills/` to `~/.gemini/antigravity/skills/` in real-time. Claude is the source of truth — any change overwrites the corresponding Gemini file.

**Scope (v1):** Direct copy only. No transformation. Mirrored directory structure.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Antigravity Daemon                    │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌──────────────┐    ┌────────────┐ │
│  │ File Watcher│───▶│ Sync Engine  │───▶│   Logger   │ │
│  │  (notify)   │    │ (copy/delete)│    │(~/antigrav)│ │
│  └─────────────┘    └──────────────┘    └────────────┘ │
└─────────────────────────────────────────────────────────┘
         │                    │
         ▼                    ▼
   ~/.claude/skills/    ~/.gemini/antigravity/skills/
      (source)               (destination)
```

## Behavior

- **On startup:** Full sync — copy all files from source to destination
- **On file create/modify:** Copy the changed file to destination
- **On file delete:** Delete the corresponding file in destination
- **On directory create:** Create directory in destination
- **On directory delete:** Remove directory from destination (recursive)
- **Debouncing:** 100ms delay to batch rapid changes

## Project Structure

```
claude-antigravity-sync/
├── Cargo.toml
├── src/
│   └── main.rs
├── scripts/
│   └── install.sh
└── com.antigravity.sync.plist
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `notify` | File system watcher (FSEvents on macOS) |
| `notify-debouncer-mini` | Debounce rapid file changes |
| `home` | Cross-platform `~` expansion |
| `tracing` + `tracing-subscriber` | Structured logging |
| `chrono` | Timestamps for log files |
| `anyhow` | Error handling |

## Paths

| Path | Purpose |
|------|---------|
| `~/.claude/skills/` | Source (watched) |
| `~/.gemini/antigravity/skills/` | Destination (synced) |
| `~/antigravity/logs/` | Log files |
| `/usr/local/bin/antigravity` | Installed binary |
| `~/Library/LaunchAgents/com.antigravity.sync.plist` | launchd config |

## Event Handling

| Event | Action |
|-------|--------|
| `Create(file)` | Copy to destination |
| `Modify(file)` | Overwrite in destination |
| `Remove(file)` | Delete from destination |
| `Create(dir)` | Create dir in destination |
| `Remove(dir)` | Remove dir from destination (recursive) |
| `Rename(from, to)` | Delete old path, copy new path |

## Edge Cases

- **Rapid saves:** Debouncer batches events within 100ms window
- **Partial writes:** Wait for debounce before copying
- **Permission errors:** Log and continue (don't crash)
- **Missing source:** Log error, skip sync, keep running
- **Destination exists:** Overwrite without prompt
- **Symlinks:** Copy as regular files
- **Hidden files:** Sync them too

## Logging

- Location: `~/antigravity/logs/antigravity.log`
- Format: `2026-01-16T14:30:00 INFO synced: vercel-deploy/SKILL.md`

```
INFO  startup: initial sync started
INFO  synced: path/to/file.md (created|modified)
INFO  removed: path/to/file.md
WARN  permission denied: path/to/file.md (skipped)
ERROR source directory missing: ~/.claude/skills/
INFO  shutdown: received SIGTERM
```

## launchd Configuration

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.antigravity.sync</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/antigravity</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>~/antigravity/logs/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>~/antigravity/logs/stderr.log</string>
</dict>
</plist>
```

## Installation

```bash
# Build
cargo build --release

# Install binary
sudo cp target/release/antigravity /usr/local/bin/
sudo chmod +x /usr/local/bin/antigravity

# Install launchd plist
cp com.antigravity.sync.plist ~/Library/LaunchAgents/

# Load and start
launchctl load ~/Library/LaunchAgents/com.antigravity.sync.plist
launchctl start com.antigravity.sync
```

## Management Commands

```bash
# Stop
launchctl stop com.antigravity.sync

# Unload
launchctl unload ~/Library/LaunchAgents/com.antigravity.sync.plist

# Reload
launchctl unload ~/Library/LaunchAgents/com.antigravity.sync.plist
launchctl load ~/Library/LaunchAgents/com.antigravity.sync.plist

# Status
launchctl list | grep antigravity

# Logs
tail -f ~/antigravity/logs/antigravity.log
```

## Future Enhancements

- Content transformation (field mapping when formats diverge)
- Watch additional directories (`~/.claude/settings.json`, etc.)
- Log rotation
- Config file for customizable paths
