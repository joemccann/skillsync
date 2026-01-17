use skillsync::{Destination, DestinationType, SkillSync};
use std::fs;
use tempfile::TempDir;

#[test]
fn gemini_toml_parses_with_prompt_and_description() {
    let source_dir = TempDir::new().unwrap();
    let gemini_dir = TempDir::new().unwrap();

    // Prepare a skill with quotes in description to test escaping
    let skill_dir = source_dir.path().join("web-design-guidelines");
    fs::create_dir_all(&skill_dir).unwrap();
    let md = r#"---
name: Web Design Guidelines
description: "Review my UI for accessibility & \"design\" sanity."
---

# Body
Hello
"#;
    fs::write(skill_dir.join("SKILL.md"), md).unwrap();

    // Run sync into GeminiToml destination only
    let sync = SkillSync::new(
        source_dir.path().to_path_buf(),
        vec![Destination::new(
            gemini_dir.path().to_path_buf(),
            DestinationType::GeminiToml,
        )],
    );
    sync.ensure_directories().unwrap();
    sync.initial_sync().unwrap();

    let toml_path = gemini_dir.path().join("web-design-guidelines.toml");
    assert!(toml_path.exists(), "expected TOML at {:?}", toml_path);

    let txt = fs::read_to_string(&toml_path).unwrap();

    // Parse with toml crate to validate syntax
    let value: toml::Value = toml::from_str(&txt).expect("valid TOML");

    // Required key per CLI docs
    assert!(value.get("prompt").is_some(), "prompt required");
    assert!(value.get("prompt").unwrap().is_str(), "prompt is string");

    // Description should exist and be a string (escaped properly)
    assert!(value.get("description").is_some());
    assert!(value.get("description").unwrap().is_str());

    // Prompt must not contain YAML markers (frontmatter stripped)
    let prompt = value.get("prompt").unwrap().as_str().unwrap();
    assert!(
        !prompt.contains("---"),
        "YAML frontmatter leaked into prompt"
    );
}

#[test]
fn antigravity_destination_preserves_yaml_frontmatter() {
    let source_dir = TempDir::new().unwrap();
    let antigravity_dir = TempDir::new().unwrap();

    // Create a Claude-style skill with frontmatter
    let skill_dir = source_dir.path().join("vercel-deploy");
    fs::create_dir_all(&skill_dir).unwrap();
    let md = r#"---
name: Vercel Deploy
description: Deploy apps to Vercel
---

Run deployment.
"#;
    fs::write(skill_dir.join("SKILL.md"), md).unwrap();

    let sync = SkillSync::new(
        source_dir.path().to_path_buf(),
        vec![Destination::new(
            antigravity_dir.path().to_path_buf(),
            DestinationType::ClaudeStyle,
        )],
    );
    sync.ensure_directories().unwrap();
    sync.initial_sync().unwrap();

    // Claude-style file should be present with the same frontmatter content
    let copied = antigravity_dir
        .path()
        .join("vercel-deploy")
        .join("SKILL.md");
    assert!(copied.exists());
    let out = fs::read_to_string(copied).unwrap();
    assert!(out.starts_with("---\n"));
    assert!(out.contains("description: Deploy apps to Vercel"));
}
