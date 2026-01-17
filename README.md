# SkillSync

Turn skills into everywhere‑available commands: a lightning‑fast macOS daemon mirroring Claude Code to Gemini and Antigravity.

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
- Node.js (for Gemini CLI via `npm install -g @google/gemini-cli`)
  - Supported installation methods: Homebrew, nvm, fnm, Volta, nodenv, asdf, or official installer

### Quick Install

```bash
git clone https://github.com/joemccann/skillsync.git
cd skillsync
./scripts/install.sh
```

This will:
1. Build the release binary
2. Install to `/usr/local/bin/skillsync`
3. Detect your Node.js installation and configure PATH for launchd
4. Install `resources/com.skillsync.plist` to `~/Library/LaunchAgents/`
5. Configure launchd for auto-start and start the service

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

### Preflight Checks

On startup the daemon runs environment checks before syncing:
- Claude Code skills directory exists at `~/.claude/skills/` (required)
- Gemini CLI (`gemini`) is available on PATH or in common installation locations (required)
- Antigravity destination directory presence (warns if missing; will be created)

The preflight check searches for `gemini` in multiple locations:
- System PATH
- Homebrew: `/opt/homebrew/bin` (Apple Silicon), `/usr/local/bin` (Intel)
- nvm: `~/.nvm/versions/node/*/bin`
- fnm: `~/.fnm/node-versions/*/bin`
- Volta: `~/.volta/bin`
- nodenv: `~/.nodenv/versions/*/bin`
- asdf: `~/.asdf/installs/nodejs/*/bin`
- npm global: `~/.npm-global/bin`

The `install.sh` script automatically detects your Node.js installation and configures the launchd PATH to include the necessary directories.

If the Claude skills directory or Gemini CLI is missing, the daemon logs an error message and exits gracefully.

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

## Contributing

Contributions are welcome! Here's how you can help:

### Reporting Bugs

If you find a bug, please [open an issue](https://github.com/joemccann/skillsync/issues/new) with:
- A clear, descriptive title
- Steps to reproduce the issue
- Expected vs actual behavior
- Your environment:
  - macOS version
  - Node.js installation method (Homebrew, nvm, etc.)
  - Gemini CLI version (`gemini --version`)
  - Relevant log snippets from `~/skillsync/logs/skillsync.log`

### Requesting Features

For feature requests, [open an issue](https://github.com/joemccann/skillsync/issues/new) with:
- A clear description of the feature
- Use case and motivation
- Any implementation ideas (optional)

### Submitting Pull Requests

1. **Fork the repository** and create your branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the existing code style:
   - Run `cargo fmt` to format code
   - Run `cargo clippy` to catch common issues
   - Add tests for new functionality

3. **Test your changes**:
   ```bash
   cargo test
   ./scripts/test_sync.sh
   ```

4. **Commit your changes** with a descriptive message:
   ```bash
   git commit -m "Add feature: your feature description"
   ```

5. **Push to your fork** and [open a pull request](https://github.com/joemccann/skillsync/compare):
   - Describe what changed and why
   - Reference any related issues
   - Include test results

### Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR-USERNAME/skillsync.git
cd skillsync

# Build and test
cargo build
cargo test

# Install locally for testing
./scripts/install.sh
```

### Code of Conduct

Be respectful and constructive. This project follows standard open source community guidelines.

## License

MIT
