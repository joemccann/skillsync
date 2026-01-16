//! Antigravity - Sync Claude skills to Gemini in real-time
//!
//! A macOS daemon that watches ~/.claude/skills/ and mirrors changes
//! to ~/.gemini/antigravity/skills/

use anyhow::{Context, Result};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use tracing::{error, info, warn, Level};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::format::FmtSpan;

const DEBOUNCE_MS: u64 = 100;

struct AntigravitySync {
    source: PathBuf,
    destination: PathBuf,
}

impl AntigravitySync {
    fn new() -> Result<Self> {
        let home = home::home_dir().context("Could not determine home directory")?;

        let source = home.join(".claude").join("skills");
        let destination = home.join(".gemini").join("antigravity").join("skills");

        Ok(Self { source, destination })
    }

    /// Ensure all required directories exist
    fn ensure_directories(&self) -> Result<()> {
        let home = home::home_dir().context("Could not determine home directory")?;
        let log_dir = home.join("antigravity").join("logs");

        // Create log directory
        fs::create_dir_all(&log_dir)
            .with_context(|| format!("Failed to create log directory: {}", log_dir.display()))?;

        // Create destination directory
        fs::create_dir_all(&self.destination)
            .with_context(|| format!("Failed to create destination: {}", self.destination.display()))?;

        info!("directories initialized");
        Ok(())
    }

    /// Perform initial full sync from source to destination
    fn initial_sync(&self) -> Result<()> {
        info!("starting initial sync");

        if !self.source.exists() {
            warn!(path = %self.source.display(), "source directory does not exist, waiting for creation");
            return Ok(());
        }

        self.sync_directory(&self.source)?;

        // Clean up orphaned files in destination
        self.cleanup_orphans()?;

        info!("initial sync completed");
        Ok(())
    }

    /// Recursively sync a directory
    fn sync_directory(&self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.sync_directory(&path)?;
            } else {
                self.sync_file(&path)?;
            }
        }

        Ok(())
    }

    /// Sync a single file from source to destination
    fn sync_file(&self, source_path: &Path) -> Result<()> {
        let relative = source_path
            .strip_prefix(&self.source)
            .with_context(|| format!("Path {} is not under source", source_path.display()))?;

        let dest_path = self.destination.join(relative);

        // Create parent directories if needed
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy file
        fs::copy(source_path, &dest_path)
            .with_context(|| format!("Failed to copy {} to {}", source_path.display(), dest_path.display()))?;

        info!(file = %relative.display(), "synced");
        Ok(())
    }

    /// Remove a file from destination
    fn remove_file(&self, source_path: &Path) -> Result<()> {
        let relative = source_path
            .strip_prefix(&self.source)
            .with_context(|| format!("Path {} is not under source", source_path.display()))?;

        let dest_path = self.destination.join(relative);

        if dest_path.exists() {
            if dest_path.is_dir() {
                fs::remove_dir_all(&dest_path)?;
            } else {
                fs::remove_file(&dest_path)?;
            }
            info!(file = %relative.display(), "removed");
        }

        Ok(())
    }

    /// Remove orphaned files/directories in destination that don't exist in source
    fn cleanup_orphans(&self) -> Result<()> {
        self.cleanup_orphans_recursive(&self.destination)
    }

    fn cleanup_orphans_recursive(&self, dest_dir: &Path) -> Result<()> {
        if !dest_dir.exists() {
            return Ok(());
        }

        let mut entries_to_remove = Vec::new();

        for entry in fs::read_dir(dest_dir)? {
            let entry = entry?;
            let dest_path = entry.path();

            let relative = dest_path
                .strip_prefix(&self.destination)
                .context("Invalid destination path")?;

            let source_path = self.source.join(relative);

            if !source_path.exists() {
                entries_to_remove.push(dest_path.clone());
            } else if dest_path.is_dir() {
                self.cleanup_orphans_recursive(&dest_path)?;
            }
        }

        for path in entries_to_remove {
            let relative = path.strip_prefix(&self.destination).unwrap_or(&path);
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
            info!(file = %relative.display(), "removed orphan");
        }

        Ok(())
    }

    /// Handle a file system event
    fn handle_event(&self, path: &Path) -> Result<()> {
        // Only process events under our source directory
        if !path.starts_with(&self.source) {
            return Ok(());
        }

        if path.exists() {
            if path.is_dir() {
                self.sync_directory(path)?;
            } else {
                self.sync_file(path)?;
            }
        } else {
            // File was deleted
            self.remove_file(path)?;
        }

        Ok(())
    }
}

fn setup_logging() -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let home = home::home_dir().context("Could not determine home directory")?;
    let log_dir = home.join("antigravity").join("logs");

    // Ensure log directory exists
    fs::create_dir_all(&log_dir)?;

    // Create a file appender that writes to antigravity.log
    let file_appender = RollingFileAppender::new(Rotation::NEVER, &log_dir, "antigravity.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Set up subscriber with both stdout and file output
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_span_events(FmtSpan::NONE)
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    Ok(guard)
}

fn main() -> Result<()> {
    // Set up logging first
    let _guard = setup_logging()?;

    info!("antigravity daemon starting");

    // Initialize sync manager
    let sync = AntigravitySync::new()?;
    sync.ensure_directories()?;

    // Perform initial sync
    if let Err(e) = sync.initial_sync() {
        error!(error = %e, "initial sync failed");
    }

    // Set up file watcher with debouncing
    let (tx, rx) = channel();

    let mut debouncer = new_debouncer(Duration::from_millis(DEBOUNCE_MS), tx)
        .context("Failed to create debouncer")?;

    // Watch source directory
    debouncer
        .watcher()
        .watch(&sync.source, RecursiveMode::Recursive)
        .with_context(|| format!("Failed to watch {}", sync.source.display()))?;

    info!(path = %sync.source.display(), "watching for changes");

    // Set up signal handling for graceful shutdown
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        info!("shutdown signal received");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    }).context("Failed to set signal handler")?;

    // Main event loop
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Ok(events)) => {
                for event in events {
                    if let Err(e) = sync.handle_event(&event.path) {
                        warn!(
                            path = %event.path.display(),
                            error = %e,
                            "failed to handle event"
                        );
                    }
                }
            }
            Ok(Err(err)) => {
                error!(error = %err, "watch error");
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Normal timeout, continue loop
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                error!("watch channel disconnected");
                break;
            }
        }
    }

    info!("antigravity daemon shutting down");
    Ok(())
}
