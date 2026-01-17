//! File system watching and event handling

use anyhow::{Context, Result};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::sync::SkillSync;

const DEBOUNCE_MS: u64 = 100;

pub fn watch_and_sync(sync: SkillSync, source: &Path) -> Result<()> {
    // Set up file watcher with debouncing
    let (tx, rx) = channel();

    let mut debouncer =
        new_debouncer(Duration::from_millis(DEBOUNCE_MS), tx).context("Failed to create debouncer")?;

    // Watch source directory
    debouncer
        .watcher()
        .watch(source, RecursiveMode::Recursive)
        .with_context(|| format!("Failed to watch {}", source.display()))?;

    info!(path = %source.display(), "watching for changes");

    // Set up signal handling for graceful shutdown
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        info!("shutdown signal received");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })
    .context("Failed to set signal handler")?;

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

    info!("skillsync daemon shutting down");
    Ok(())
}
