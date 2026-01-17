//! File synchronization logic

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::destination::{Destination, DestinationType};
use crate::transform::{generate_toml, parse_frontmatter};

pub struct SkillSync {
    source: PathBuf,
    destinations: Vec<Destination>,
}

impl SkillSync {
    pub fn new(source: PathBuf, destinations: Vec<Destination>) -> Self {
        Self {
            source,
            destinations,
        }
    }

    /// Ensure all required directories exist
    pub fn ensure_directories(&self) -> Result<()> {
        // Create all destination base directories
        for dest in &self.destinations {
            fs::create_dir_all(&dest.base_path).with_context(|| {
                format!("Failed to create destination: {}", dest.base_path.display())
            })?;
        }

        info!(
            destinations = self.destinations.len(),
            "directories initialized"
        );
        Ok(())
    }

    /// Perform initial full sync from source to all destinations
    pub fn initial_sync(&self) -> Result<()> {
        info!("starting initial sync");

        if !self.source.exists() {
            warn!(
                path = %self.source.display(),
                "source directory does not exist, waiting for creation"
            );
            return Ok(());
        }

        self.sync_directory(&self.source)?;

        // Clean up orphaned files in all destinations
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

    /// Sync a single file from source to all destinations
    fn sync_file(&self, source_path: &Path) -> Result<()> {
        let relative = source_path
            .strip_prefix(&self.source)
            .with_context(|| format!("Path {} is not under source", source_path.display()))?;

        // Read source file content once
        let source_content = fs::read_to_string(source_path)
            .with_context(|| format!("Failed to read {}", source_path.display()))?;

        for dest in &self.destinations {
            match dest.dest_type {
                DestinationType::ClaudeStyle => {
                    // Direct copy with same structure
                    let dest_path = dest.base_path.join(relative);

                    // Create parent directories if needed
                    if let Some(parent) = dest_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    // Write original content
                    fs::write(&dest_path, &source_content)
                        .with_context(|| format!("Failed to write to {}", dest_path.display()))?;
                }
                DestinationType::GeminiToml => {
                    // Transform to TOML format
                    // Only process SKILL.md files
                    if source_path.file_name() != Some(std::ffi::OsStr::new("SKILL.md")) {
                        continue;
                    }

                    // Get the parent directory name (e.g., "ui-skills" from "ui-skills/SKILL.md")
                    let skill_name = relative
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("skill");

                    // Parse frontmatter and generate TOML
                    let (frontmatter, stripped_content) = parse_frontmatter(&source_content);
                    let toml_content = generate_toml(frontmatter.description, &stripped_content);

                    // Write as {skill_name}.toml in commands directory
                    let dest_path = dest.base_path.join(format!("{}.toml", skill_name));
                    fs::write(&dest_path, toml_content)
                        .with_context(|| format!("Failed to write TOML to {}", dest_path.display()))?;
                }
            }
        }

        info!(file = %relative.display(), "synced");
        Ok(())
    }

    /// Remove a file from all destinations
    fn remove_file(&self, source_path: &Path) -> Result<()> {
        let relative = source_path
            .strip_prefix(&self.source)
            .with_context(|| format!("Path {} is not under source", source_path.display()))?;

        for dest in &self.destinations {
            match dest.dest_type {
                DestinationType::ClaudeStyle => {
                    let dest_path = dest.base_path.join(relative);

                    if dest_path.exists() {
                        if dest_path.is_dir() {
                            fs::remove_dir_all(&dest_path)?;
                        } else {
                            fs::remove_file(&dest_path)?;
                        }
                    }
                }
                DestinationType::GeminiToml => {
                    // Only handle SKILL.md files
                    if source_path.file_name() != Some(std::ffi::OsStr::new("SKILL.md")) {
                        continue;
                    }

                    // Get skill name from parent directory
                    let skill_name = relative
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("skill");

                    // Remove the corresponding .toml file
                    let dest_path = dest.base_path.join(format!("{}.toml", skill_name));
                    if dest_path.exists() {
                        fs::remove_file(&dest_path)?;
                    }
                }
            }
        }

        info!(file = %relative.display(), "removed");
        Ok(())
    }

    /// Remove orphaned files/directories in all destinations that don't exist in source
    fn cleanup_orphans(&self) -> Result<()> {
        for dest in &self.destinations {
            self.cleanup_orphans_for_dest(dest)?;
        }
        Ok(())
    }

    fn cleanup_orphans_for_dest(&self, dest: &Destination) -> Result<()> {
        match dest.dest_type {
            DestinationType::ClaudeStyle => {
                self.cleanup_orphans_recursive_claude(&dest.base_path, &dest.base_path)
            }
            DestinationType::GeminiToml => self.cleanup_orphans_toml(&dest.base_path),
        }
    }

    fn cleanup_orphans_recursive_claude(&self, dest_root: &Path, dest_dir: &Path) -> Result<()> {
        if !dest_dir.exists() {
            return Ok(());
        }

        let mut entries_to_remove = Vec::new();

        for entry in fs::read_dir(dest_dir)? {
            let entry = entry?;
            let dest_path = entry.path();

            let relative = dest_path
                .strip_prefix(dest_root)
                .context("Invalid destination path")?;

            let source_path = self.source.join(relative);

            if !source_path.exists() {
                entries_to_remove.push(dest_path.clone());
            } else if dest_path.is_dir() {
                self.cleanup_orphans_recursive_claude(dest_root, &dest_path)?;
            }
        }

        for path in entries_to_remove {
            let relative = path.strip_prefix(dest_root).unwrap_or(&path);
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
            info!(file = %relative.display(), "removed orphan");
        }

        Ok(())
    }

    fn cleanup_orphans_toml(&self, dest_dir: &Path) -> Result<()> {
        if !dest_dir.exists() {
            return Ok(());
        }

        // Get all .toml files in commands directory
        for entry in fs::read_dir(dest_dir)? {
            let entry = entry?;
            let dest_path = entry.path();

            // Only check .toml files
            if dest_path.extension() != Some(std::ffi::OsStr::new("toml")) {
                continue;
            }

            // Get the skill name from filename (e.g., "ui-skills.toml" -> "ui-skills")
            if let Some(skill_name) = dest_path.file_stem().and_then(|s| s.to_str()) {
                // Check if corresponding SKILL.md exists in source
                let source_skill_path = self.source.join(skill_name).join("SKILL.md");

                if !source_skill_path.exists() {
                    fs::remove_file(&dest_path)?;
                    info!(
                        file = %dest_path.file_name().unwrap().to_string_lossy(),
                        "removed orphan TOML"
                    );
                }
            }
        }

        Ok(())
    }

    /// Handle a file system event
    pub fn handle_event(&self, path: &Path) -> Result<()> {
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
