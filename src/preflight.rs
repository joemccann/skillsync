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
    // Check PATH first, then common npm/nvm installation locations
    let gemini_found = if let Ok(path) = which::which("gemini") {
        out.gemini_cli_ok = true;
        info!(binary = %path.display(), "Gemini CLI detected");
        true
    } else {
        // Check common npm/node installation locations:
        // - nvm (Node Version Manager)
        // - fnm (Fast Node Manager)
        // - Volta (JavaScript toolchain manager)
        // - nodenv (rbenv-style version manager)
        // - asdf (multi-language version manager)
        // - npm global installs
        // - Homebrew (Apple Silicon and Intel)
        // - system paths
        let home = std::env::var("HOME").unwrap_or_default();
        let search_paths = vec![
            // nvm
            format!("{}/.nvm/versions/node", home),
            // fnm
            format!("{}/.fnm/node-versions", home),
            // Volta
            format!("{}/.volta/bin/gemini", home),
            // nodenv
            format!("{}/.nodenv/versions", home),
            // asdf
            format!("{}/.asdf/installs/nodejs", home),
            // npm global
            format!("{}/.npm-global/bin/gemini", home),
            // Homebrew (Apple Silicon - M1/M2/M3)
            "/opt/homebrew/bin/gemini".to_string(),
            // Homebrew (Intel Mac)
            "/usr/local/bin/gemini".to_string(),
        ];

        let mut found = false;
        for base_path in search_paths {
            let path = Path::new(&base_path);
            // For nvm/fnm/nodenv/asdf, we need to search recursively for the gemini binary
            if base_path.contains("nvm") || base_path.contains("fnm") || base_path.contains("nodenv") || base_path.contains("asdf") {
                if path.exists() {
                    if let Ok(entries) = std::fs::read_dir(path) {
                        for entry in entries.flatten() {
                            let gemini_bin = entry.path().join("bin/gemini");
                            if gemini_bin.exists() {
                                out.gemini_cli_ok = true;
                                info!(binary = %gemini_bin.display(), "Gemini CLI detected (version manager)");
                                found = true;
                                break;
                            }
                        }
                    }
                }
            } else if path.exists() {
                out.gemini_cli_ok = true;
                info!(binary = %path.display(), "Gemini CLI detected");
                found = true;
                break;
            }
            if found {
                break;
            }
        }

        if !found {
            error!(
                "Gemini CLI not found on PATH or in common installation locations.\nSkillSync requires the 'gemini' binary.\nRemediation: install the Gemini CLI:\n  • npm install -g @google/gemini-cli\nSupported Node.js installation methods:\n  • Homebrew (brew install node)\n  • nvm (Node Version Manager)\n  • fnm (Fast Node Manager)\n  • Volta (JavaScript toolchain manager)\n  • nodenv (rbenv-style version manager)\n  • asdf (multi-language version manager)\n  • Official installer from nodejs.org\nThe installer will automatically detect and configure your Node.js installation."
            );
        }
        found
    };

    if !gemini_found {
        out.gemini_cli_ok = false;
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
