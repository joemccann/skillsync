use skillsync::{Destination, DestinationType, SkillSync};
use std::fs;
use tempfile::TempDir;

mod common;

/// Helper to create a test SkillSync instance with temporary directories
fn create_test_sync() -> (SkillSync, TempDir, TempDir, TempDir, TempDir) {
    let source_dir = TempDir::new().unwrap();
    let dest1_dir = TempDir::new().unwrap();
    let dest2_dir = TempDir::new().unwrap();
    let dest3_dir = TempDir::new().unwrap();

    let sync = SkillSync::new(
        source_dir.path().to_path_buf(),
        vec![
            Destination::new(dest1_dir.path().to_path_buf(), DestinationType::ClaudeStyle),
            Destination::new(dest2_dir.path().to_path_buf(), DestinationType::ClaudeStyle),
            Destination::new(dest3_dir.path().to_path_buf(), DestinationType::GeminiToml),
        ],
    );

    (sync, source_dir, dest1_dir, dest2_dir, dest3_dir)
}

#[test]
fn test_sync_file_claude_style() {
    let (sync, source_dir, dest1_dir, dest2_dir, _dest3_dir) = create_test_sync();

    // Create a test skill
    let skill_dir = source_dir.path().join("test-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    let skill_file = skill_dir.join("SKILL.md");
    fs::write(&skill_file, "---\ndescription: Test\n---\nContent").unwrap();

    // Sync the directory
    sync.ensure_directories().unwrap();
    sync.initial_sync().unwrap();

    // Check ClaudeStyle destinations
    let dest1_file = dest1_dir.path().join("test-skill").join("SKILL.md");
    let dest2_file = dest2_dir.path().join("test-skill").join("SKILL.md");

    assert!(dest1_file.exists());
    assert!(dest2_file.exists());

    let content = fs::read_to_string(&dest1_file).unwrap();
    assert!(content.contains("---\ndescription: Test\n---"));
}

#[test]
fn test_sync_file_gemini_toml() {
    let (sync, source_dir, _dest1_dir, _dest2_dir, dest3_dir) = create_test_sync();

    // Create a test skill
    let skill_dir = source_dir.path().join("my-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    let skill_file = skill_dir.join("SKILL.md");
    fs::write(
        &skill_file,
        "---\ndescription: My test skill\n---\n\n# Test\nContent here",
    )
    .unwrap();

    // Sync the directory
    sync.ensure_directories().unwrap();
    sync.initial_sync().unwrap();

    // Check GeminiToml destination
    let toml_file = dest3_dir.path().join("my-skill.toml");
    assert!(toml_file.exists());

    let content = fs::read_to_string(&toml_file).unwrap();
    assert!(content.contains("description = \"My test skill\""));
    assert!(content.contains("prompt = '''"));
    assert!(content.contains("# Test"));
    assert!(content.contains("Content here"));
    // YAML frontmatter should be stripped
    assert!(!content.contains("---"));
}

#[test]
fn test_cleanup_orphans_claude() {
    let (sync, _source_dir, dest1_dir, _dest2_dir, _dest3_dir) = create_test_sync();

    // Create an orphaned file in destination (no corresponding source)
    let orphan_dir = dest1_dir.path().join("orphan-skill");
    fs::create_dir_all(&orphan_dir).unwrap();
    fs::write(orphan_dir.join("SKILL.md"), "Orphan").unwrap();

    assert!(orphan_dir.exists());

    // Run initial sync which includes cleanup
    sync.ensure_directories().unwrap();
    sync.initial_sync().unwrap();

    // Orphan should be removed
    assert!(!orphan_dir.exists());
}

#[test]
fn test_cleanup_orphans_toml() {
    let (sync, _source_dir, _dest1_dir, _dest2_dir, dest3_dir) = create_test_sync();

    // Create an orphaned TOML file (no corresponding source)
    let orphan_toml = dest3_dir.path().join("orphan-skill.toml");
    fs::write(
        &orphan_toml,
        "description = \"Test\"\nprompt = \"\"\"Test\"\"\"",
    )
    .unwrap();

    assert!(orphan_toml.exists());

    // Run initial sync which includes cleanup
    sync.ensure_directories().unwrap();
    sync.initial_sync().unwrap();

    // Orphan TOML should be removed
    assert!(!orphan_toml.exists());
}

#[test]
fn test_sync_directory_recursive() {
    let (sync, source_dir, dest1_dir, _dest2_dir, _dest3_dir) = create_test_sync();

    // Create nested structure
    let skill1_dir = source_dir.path().join("skill1");
    let skill2_dir = source_dir.path().join("skill2");
    fs::create_dir_all(&skill1_dir).unwrap();
    fs::create_dir_all(&skill2_dir).unwrap();

    fs::write(skill1_dir.join("SKILL.md"), "Skill 1").unwrap();
    fs::write(skill2_dir.join("SKILL.md"), "Skill 2").unwrap();

    // Sync entire directory
    sync.ensure_directories().unwrap();
    sync.initial_sync().unwrap();

    // Check both skills synced
    assert!(dest1_dir.path().join("skill1").join("SKILL.md").exists());
    assert!(dest1_dir.path().join("skill2").join("SKILL.md").exists());
}

#[test]
fn test_non_skill_files_ignored_in_toml_dest() {
    let (sync, source_dir, _dest1_dir, _dest2_dir, dest3_dir) = create_test_sync();

    // Create a non-SKILL.md file
    let skill_dir = source_dir.path().join("test-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    let other_file = skill_dir.join("README.md");
    fs::write(&other_file, "Not a skill").unwrap();

    // Sync the directory
    sync.ensure_directories().unwrap();
    sync.initial_sync().unwrap();

    // TOML destination should not have anything
    let toml_file = dest3_dir.path().join("test-skill.toml");
    assert!(!toml_file.exists());
}
