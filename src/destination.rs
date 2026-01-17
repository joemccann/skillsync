//! Destination types and configuration

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum DestinationType {
    /// Direct copy preserving directory structure and YAML frontmatter
    ClaudeStyle,
    /// Transform SKILL.md to TOML format at base path (flat structure)
    GeminiToml,
}

#[derive(Debug, Clone)]
pub struct Destination {
    pub base_path: PathBuf,
    pub dest_type: DestinationType,
}

impl Destination {
    pub fn new(base_path: PathBuf, dest_type: DestinationType) -> Self {
        Self {
            base_path,
            dest_type,
        }
    }
}
