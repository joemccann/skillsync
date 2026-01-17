//! Preflight checks to validate environment before syncing

use anyhow::Result;
use std::path::Path;
use tracing::{error, info, warn};

use crate::config::Config;

pub struct PreflightOutcome {
    pub claude_ok: bool,
    pub gemini_cli_ok: bool,
    pub antigravity_ok: bool,
}

impl PreflightOutcome {
    pub fn all_good(&self) -> bool {
        self.claude_ok && self.gemini_cli_ok && self.antigravity_ok
    }
}

pub fn check_all(cfg: &Config) -> Result<PreflightOutcome> {
    let mut out = PreflightOutcome {
        claude_ok: false,
        gemini_cli_ok: false,
        antigravity_ok: false,
    };

    // 1) Claude Code skills source must exist
    if cfg.source.exists() {
        out.claude_ok = true;
        info!(path = %cfg.source.display(), "Claude skills source detected");
    } else {
        warn!(path = %cfg.source.display(), "Claude skills directory not found");
        error!(
            "Claude Code appears missing. Install or create ~/.claude/skills before running SkillSync."
        );
    }

    // 2) Gemini CLI presence (best-effort)
    match which::which("gemini") {
        Ok(path) => {
            out.gemini_cli_ok = true;
            info!(binary = %path.display(), "Gemini CLI detected");
        }
        Err(_) => {
            warn!(
                "Gemini CLI not found on PATH. If installed via nvm, ensure your shell exports it before launchd starts, or provide an absolute path."
            );
        }
    }

    // 3) Antigravity destination presence (directory)
    let antigravity_dir = cfg
        .destinations
        .iter()
        .find(|d| d.base_path.to_string_lossy().contains("antigravity/skills"))
        .map(|d| d.base_path.as_path())
        .unwrap_or(Path::new("/nonexistent"));

    if antigravity_dir.exists() {
        out.antigravity_ok = true;
        info!(path = %antigravity_dir.display(), "Antigravity directory detected");
    } else {
        // Not fatal — we'll create it later — but surface as a warning so users can verify install
        warn!(path = %antigravity_dir.display(), "Antigravity directory not found (will be created)");
        out.antigravity_ok = true; // treat as OK since we can create it
    }

    Ok(out)
}
