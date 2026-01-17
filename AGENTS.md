# AGENTS.md

This file provides guidance to AI coding assistants when working with code in this repository.

## Project Overview

SkillSync is a macOS daemon written in Rust that mirrors Claude skills to Gemini, Codex, and Cursor in real-time using FSEvents. It watches `~/.claude/skills/` as the source of truth and syncs changes to five destinations with tool-specific transformations:
- `~/.gemini/skills/` - Claude-style (preserves YAML frontmatter)
- `~/.gemini/antigravity/skills/` - Claude-style (preserves YAML frontmatter)
- `~/.codex/skills/` - Claude-style (preserves YAML frontmatter)
- `~/.cursor/skills/` - Claude-style (preserves YAML frontmatter)
- `~/.gemini/commands/` - Gemini CLI TOML format (strips YAML, wraps in TOML)

## Development Commands

### Build and Test
```bash
# Build debug version
cargo build

# Build optimized release binary
cargo build --release

# Check code without building
cargo check

# Run tests
cargo test

# Run the daemon locally (for testing)
cargo run

# Test one-time sync (3 second timeout)
./scripts/test_sync.sh
```

### Installation and Service Management
```bash
# Install the daemon (builds, installs binary, configures launchd)
./scripts/install.sh

# Uninstall the daemon
./scripts/uninstall.sh

# Check if service is running
launchctl list | grep skillsync

# View live logs
tail -f ~/skillsync/logs/skillsync.log

# Start service
launchctl start com.skillsync

# Stop service
launchctl stop com.skillsync

# Restart service (after code changes)
launchctl unload ~/Library/LaunchAgents/com.skillsync.plist
launchctl load ~/Library/LaunchAgents/com.skillsync.plist
```

## Architecture

### Modular Design
The codebase is organized into focused modules:
- `src/main.rs` - Minimal binary entrypoint (~7 lines)
- `src/lib.rs` - Public library interface
- `src/config.rs` - Configuration and path management
- `src/destination.rs` - Destination types and configuration
- `src/transform.rs` - Content transformations (YAML/TOML)
- `src/sync.rs` - Core sync logic and SkillSync struct
- `src/watcher.rs` - File watching and event handling
- `src/preflight.rs` - Environment checks (Claude, Gemini CLI, Antigravity)
- `tests/` - Integration and validation tests (TOML parsing, YAML preservation)

### Core Components

**SkillSync struct** - Main sync manager that:
- Initializes source (`~/.claude/skills/`) and destination paths with types
- Performs initial full sync on startup with transformations
- Handles file system events (create, modify, delete)
- Maintains sync to multiple destinations with per-destination transformation logic
- Cleans up orphaned files that don't exist in source (including reverse-mapped TOML files)

**Destination Types**:
- `ClaudeStyle`: Direct copy preserving directory structure and YAML frontmatter
- `GeminiToml`: Transforms `SKILL.md` files to TOML format at base path (flat structure)

**Content Transformations**:
- YAML frontmatter parsing: Extracts `description` field from frontmatter
- YAML stripping: Removes content between `---` markers for TOML destinations
- TOML generation: Wraps content in `description` (escaped) and `prompt` fields; uses TOML literal multiline strings (`'''`) for prompt to avoid escaping content
- Path mapping: `ui-skills/SKILL.md` → `ui-skills.toml` for Gemini CLI

**Event Loop** - Uses `notify-debouncer-mini` to:
- Watch source directory recursively via FSEvents
- Debounce rapid changes (100ms window)
- Process batched events efficiently
- Handle graceful shutdown via SIGINT/SIGTERM

**Logging** - Uses `tracing` crate with:
- Structured logging to `~/skillsync/logs/skillsync.log`
- Non-blocking file appender
- INFO level by default

### Preflight

On startup the daemon runs environment checks:
- Requires Claude Code skills directory at `~/.claude/skills/` (exits if missing)
- Requires Gemini CLI binary `gemini` on PATH or in common installation locations (exits if missing)
  - Searches: PATH, Homebrew (Apple Silicon + Intel), nvm, fnm, Volta, nodenv, asdf, npm global
- Checks for Codex CLI binary `codex` on PATH or in common installation locations (warns if missing, continues)
- Warns if Antigravity, Codex, or Cursor destination directories are missing (they will be created)

**Node.js Version Manager Support:**
The preflight check and install script support all major Node.js installation methods on macOS:
- **Homebrew**: `/opt/homebrew/bin` (Apple Silicon), `/usr/local/bin` (Intel)
- **nvm**: `~/.nvm/versions/node/*/bin`
- **fnm** (Fast Node Manager): `~/.fnm/node-versions/*/bin`
- **Volta**: `~/.volta/bin`
- **nodenv**: `~/.nodenv/versions/*/bin`
- **asdf**: `~/.asdf/installs/nodejs/*/bin`
- **npm global**: `~/.npm-global/bin`
- **Official installer**: `/usr/local/bin`

The `install.sh` script automatically detects active Node.js installations and dynamically configures the launchd plist with appropriate PATH environment variables.

### Key Behaviors

- **Initial Sync**: On startup, recursively copies all existing files from source to destinations with appropriate transformations
- **Tool-Specific Sync**: ClaudeStyle destinations get direct copies, GeminiToml destinations get transformed TOML files
- **Orphan Cleanup**: Removes files in destinations that don't exist in source, including reverse-mapped TOML files
- **Debouncing**: Batches rapid file changes within 100ms to avoid excessive sync operations
- **Error Handling**: Individual event failures are logged but don't crash the daemon
- **Signal Handling**: Responds to Ctrl+C or kill signals for graceful shutdown

### launchd Integration

The daemon runs as a user-level LaunchAgent (`com.skillsync.plist`) with:
- Auto-start on login (`RunAtLoad`)
- Automatic restart if crashed (`KeepAlive`)
- Low priority I/O to minimize system impact
- Logs written to application-specific directory, not system logs

## Important Paths

- **Binary**: `/usr/local/bin/skillsync`
- **Source (watched)**: `~/.claude/skills/`
- **Destinations (synced)**:
  - `~/.gemini/skills/` (ClaudeStyle)
  - `~/.gemini/antigravity/skills/` (ClaudeStyle)
  - `~/.codex/skills/` (ClaudeStyle)
  - `~/.cursor/skills/` (ClaudeStyle)
  - `~/.gemini/commands/` (GeminiToml)
- **Logs**: `~/skillsync/logs/skillsync.log`
- **launchd config**: `~/Library/LaunchAgents/com.skillsync.plist`

## Dependencies

Key dependencies and their purpose:
- `notify` - macOS FSEvents file system watching
- `notify-debouncer-mini` - Event debouncing
- `tracing` / `tracing-subscriber` / `tracing-appender` - Structured logging
- `home` - Cross-platform home directory detection
- `anyhow` - Error handling with context
- `ctrlc` - Signal handling for graceful shutdown
- `which` - Locate external binaries (Gemini CLI) on PATH
- `tempfile` (dev) - Temporary directories for testing
- `toml` (dev) - Parse generated TOML in tests

## Development Notes

- This is a macOS-only daemon (uses FSEvents)
- The project uses aggressive release optimizations (`opt-level = 3`, `lto = true`, `strip = true`)
- Comprehensive test suite with 14 unit and integration tests covering:
  - YAML frontmatter parsing and stripping
  - TOML generation
  - ClaudeStyle and GeminiToml sync
  - File removal with path transformations
  - Orphan cleanup for both destination types
- When making changes, use `./scripts/install.sh` to rebuild and restart the service
- The daemon must run continuously as a background service, not as a one-time CLI tool
- Use `./scripts/test_sync.sh` for quick validation of sync behavior without installing
