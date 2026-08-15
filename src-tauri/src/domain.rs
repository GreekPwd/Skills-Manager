use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub status: SkillStatus,
    pub source: SourceKind,
    pub source_label: String,
    pub updated_at: String,
    pub files: usize,
    pub agents: Vec<String>,
    pub version: Option<String>,
    pub content_hash: String,
    pub source_url: Option<String>,
    pub source_subdir: Option<String>,
    pub source_branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillSource {
    pub url: String,
    #[serde(default)]
    pub subdir: String,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryRecord {
    pub id: String,
    pub name: String,
    pub url: String,
    pub branch: Option<String>,
    #[serde(default)]
    pub skill_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySkill {
    pub name: String,
    pub description: String,
    pub subdir: String,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SkillStatus { Healthy, Update, Conflict, Local, Invalid }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SourceKind { Git, Local }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRecord {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub detected: bool,
    pub linked_skills: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub library_path: PathBuf,
    pub recycle_path: PathBuf,
    #[serde(default)]
    pub repository_cache_path: PathBuf,
    #[serde(default)]
    pub git_proxy: Option<String>,
    pub agent_paths: std::collections::BTreeMap<String, PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub affected_paths: Vec<PathBuf>,
}
