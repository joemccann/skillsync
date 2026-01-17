//! Configuration and path management

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::destination::{Destination, DestinationType};

pub struct Config {
    pub source: PathBuf,
    pub destinations: Vec<Destination>,
    pub log_dir: PathBuf,
}

impl Config {
    pub fn new() -> Result<Self> {
        let home = home::home_dir().context("Could not determine home directory")?;

        let source = home.join(".claude").join("skills");
        let log_dir = home.join("skillsync").join("logs");

        // Configure destinations with their types
        let destinations = vec![
            Destination::new(
                home.join(".gemini").join("skills"),
                DestinationType::ClaudeStyle,
            ),
            Destination::new(
                home.join(".gemini").join("antigravity").join("skills"),
                DestinationType::ClaudeStyle,
            ),
            Destination::new(
                home.join(".gemini").join("commands"),
                DestinationType::GeminiToml,
            ),
        ];

        Ok(Self {
            source,
            destinations,
            log_dir,
        })
    }
}
