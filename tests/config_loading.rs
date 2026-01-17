use skillsync::Config;

#[test]
fn test_config_has_codex_destination() {
    let config = Config::new().expect("Failed to create config");

    // We expect 5 destinations now:
    // 1. ~/.gemini/skills (ClaudeStyle)
    // 2. ~/.gemini/antigravity/skills (ClaudeStyle)
    // 3. ~/.codex/skills (ClaudeStyle)
    // 4. ~/.cursor/skills (ClaudeStyle)
    // 5. ~/.gemini/commands (GeminiToml)
    assert_eq!(config.destinations.len(), 5, "Expected 5 destinations");

    let has_codex = config
        .destinations
        .iter()
        .any(|d| d.base_path.to_string_lossy().contains(".codex/skills"));

    let has_cursor = config
        .destinations
        .iter()
        .any(|d| d.base_path.to_string_lossy().contains(".cursor/skills"));

    assert!(
        has_codex,
        "Config should include ~/.codex/skills destination"
    );
    assert!(
        has_cursor,
        "Config should include ~/.cursor/skills destination"
    );
}
