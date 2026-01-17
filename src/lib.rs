//! SkillSync - Sync Claude skills to Gemini in real-time
//!
//! A macOS daemon that watches ~/.claude/skills/ and mirrors changes
//! to multiple destinations with tool-specific transformations.

pub mod config;
pub mod destination;
pub mod sync;
pub mod transform;
pub mod watcher;

pub use config::Config;
pub use destination::{Destination, DestinationType};
pub use sync::SkillSync;

use anyhow::Result;
use std::fs;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::format::FmtSpan;

/// Set up logging to file
pub fn setup_logging(log_dir: &std::path::Path) -> Result<tracing_appender::non_blocking::WorkerGuard> {
    // Ensure log directory exists
    fs::create_dir_all(log_dir)?;

    // Create a file appender that writes to skillsync.log
    let file_appender = RollingFileAppender::new(Rotation::NEVER, log_dir, "skillsync.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Set up subscriber with file output
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

/// Initialize and run the SkillSync daemon
pub fn run() -> Result<()> {
    // Load configuration
    let config = Config::new()?;

    // Set up logging
    let _guard = setup_logging(&config.log_dir)?;

    tracing::info!("skillsync daemon starting");

    // Initialize sync manager
    let sync = SkillSync::new(config.source.clone(), config.destinations);
    sync.ensure_directories()?;

    // Perform initial sync
    if let Err(e) = sync.initial_sync() {
        tracing::error!(error = %e, "initial sync failed");
    }

    // Start watching and syncing
    watcher::watch_and_sync(sync, &config.source)?;

    Ok(())
}
