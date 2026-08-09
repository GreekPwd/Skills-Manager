use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct SkillMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
}

pub fn parse_skill_file(path: &Path) -> Result<SkillMetadata, String> {
    let content = fs::read_to_string(path).map_err(|error| format!("无法读取 {}: {error}", path.display()))?;
    parse_frontmatter(&content)
}

pub fn parse_frontmatter(content: &str) -> Result<SkillMetadata, String> {
    let normalized = content.replace("\r\n", "\n");
    let Some(rest) = normalized.strip_prefix("---\n") else { return Ok(SkillMetadata::default()); };
    let Some(end) = rest.find("\n---") else { return Err("SKILL.md frontmatter 缺少结束标记".into()); };
    serde_yaml::from_str(&rest[..end]).map_err(|error| format!("SKILL.md frontmatter 无效: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_yaml_frontmatter() {
        let metadata = parse_frontmatter("---\nname: api-design\ndescription: Design APIs\nversion: 1.2.0\n---\n# Body").unwrap();
        assert_eq!(metadata.name.as_deref(), Some("api-design"));
        assert_eq!(metadata.version.as_deref(), Some("1.2.0"));
    }

    #[test]
    fn accepts_files_without_frontmatter() {
        assert_eq!(parse_frontmatter("# Skill").unwrap(), SkillMetadata::default());
    }

    #[test]
    fn rejects_unclosed_frontmatter() {
        assert!(parse_frontmatter("---\nname: broken").is_err());
    }
}
