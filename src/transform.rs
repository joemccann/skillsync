//! Content transformations for YAML frontmatter and TOML generation

#[derive(Debug, Default)]
pub struct FrontmatterData {
    pub description: Option<String>,
}

/// Parse YAML frontmatter and return extracted data + content without frontmatter
pub fn parse_frontmatter(content: &str) -> (FrontmatterData, String) {
    let lines: Vec<&str> = content.lines().collect();

    // Check if content starts with ---
    if lines.is_empty() || lines[0] != "---" {
        return (FrontmatterData::default(), content.to_string());
    }

    // Find the closing ---
    let mut end_idx = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if *line == "---" {
            end_idx = Some(i);
            break;
        }
    }

    let Some(end) = end_idx else {
        return (FrontmatterData::default(), content.to_string());
    };

    // Parse frontmatter for description field
    let mut description = None;
    for line in &lines[1..end] {
        if let Some(desc) = line.strip_prefix("description:") {
            description = Some(desc.trim().trim_matches('"').trim_matches('\'').to_string());
        }
    }

    // Content after frontmatter (skip the closing --- and any following empty line)
    let content_start = if end + 1 < lines.len() && lines[end + 1].is_empty() {
        end + 2
    } else {
        end + 1
    };

    let stripped_content = lines[content_start..].join("\n");

    (FrontmatterData { description }, stripped_content)
}

/// Generate TOML format for Gemini CLI
/// - Escapes description for TOML basic strings
/// - Uses TOML literal multiline string (''') for prompt to avoid escaping
pub fn generate_toml(description: Option<String>, content: &str) -> String {
    let desc = description.unwrap_or_else(|| "Custom skill".to_string());
    let desc_escaped = desc.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        "description = \"{}\"\nprompt = '''\n{}\n'''\n",
        desc_escaped, content
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_with_description() {
        let content = "---\nname: test\ndescription: Test skill\n---\n\n# Content\nTest content";
        let (frontmatter, stripped) = parse_frontmatter(content);

        assert_eq!(frontmatter.description, Some("Test skill".to_string()));
        assert_eq!(stripped, "# Content\nTest content");
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let content = "# Content\nNo frontmatter here";
        let (frontmatter, stripped) = parse_frontmatter(content);

        assert_eq!(frontmatter.description, None);
        assert_eq!(stripped, content);
    }

    #[test]
    fn test_parse_frontmatter_with_quotes() {
        let content = "---\ndescription: \"Quoted description\"\n---\nContent";
        let (frontmatter, stripped) = parse_frontmatter(content);

        assert_eq!(
            frontmatter.description,
            Some("Quoted description".to_string())
        );
        assert_eq!(stripped, "Content");
    }

    #[test]
fn test_generate_toml_with_description() {
        let toml = generate_toml(Some("My skill".to_string()), "Test content");

        assert!(toml.contains("description = \"My skill\""));
        assert!(toml.contains("prompt = '''"));
        assert!(toml.contains("Test content"));
    }

    #[test]
fn test_generate_toml_without_description() {
        let toml = generate_toml(None, "Test content");

        assert!(toml.contains("description = \"Custom skill\""));
        assert!(toml.contains("prompt = '''"));
        assert!(toml.contains("Test content"));
    }
}
