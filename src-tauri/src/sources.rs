use crate::{domain::{AppSettings, SkillSource}, git_support};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};
use walkdir::WalkDir;

#[derive(Debug, Default, Serialize, Deserialize)]
struct SourceRegistry {
    #[serde(default)]
    skills: BTreeMap<String, SkillSource>,
}

#[derive(Debug, Default, Deserialize)]
struct LegacySkillLock {
    #[serde(default)]
    skills: BTreeMap<String, LegacyLockEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyLockEntry {
    source_url: String,
    skill_path: String,
}

fn skill_path(settings: &AppSettings, name: &str) -> Result<PathBuf, String> {
    validate_skill_name(name)?;
    Ok(settings.library_path.join(name))
}

fn validate_skill_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name == "." || name == ".." || name.chars().any(|c| c == '/' || c == '\\') {
        return Err("Invalid skill name".into());
    }
    Ok(())
}

fn registry_path() -> Result<PathBuf, String> {
    dirs::config_dir().map(|root| root.join("skills-manager").join("sources.json")).ok_or_else(|| "Cannot resolve app config directory".into())
}

fn load_registry_from(path: &Path) -> Result<SourceRegistry, String> {
    if !path.exists() { return Ok(SourceRegistry::default()); }
    serde_json::from_str(&fs::read_to_string(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn save_registry_to(path: &Path, registry: &SourceRegistry) -> Result<(), String> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(registry).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    if path.exists() { fs::remove_file(path).map_err(|e| e.to_string())?; }
    fs::rename(temporary, path).map_err(|e| e.to_string())
}

pub fn get_source(name: &str) -> Result<Option<SkillSource>, String> {
    validate_skill_name(name)?;
    if let Some(source) = load_registry_from(&registry_path()?)?.skills.get(name).cloned() { return Ok(Some(source)); }
    Ok(legacy_lock_sources()?.remove(name))
}

fn legacy_lock_sources() -> Result<BTreeMap<String, SkillSource>, String> {
    let path = dirs::home_dir().ok_or_else(|| "Cannot resolve home directory".to_string())?.join(".agents").join(".skill-lock.json");
    if !path.is_file() { return Ok(BTreeMap::new()); }
    let lock: LegacySkillLock = serde_json::from_str(&fs::read_to_string(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    Ok(lock.skills.into_iter().map(|(name, entry)| {
        let skill_path = Path::new(&entry.skill_path);
        let subdir = skill_path.parent().filter(|value| !value.as_os_str().is_empty()).map(|value| value.to_string_lossy().replace('\\', "/")).unwrap_or_default();
        (name, SkillSource { url: entry.source_url, subdir, branch: None })
    }).collect())
}

pub(crate) fn all_sources() -> Result<BTreeMap<String, SkillSource>, String> {
    let mut sources = legacy_lock_sources()?;
    sources.extend(load_registry_from(&registry_path()?)?.skills);
    Ok(sources)
}

pub fn set_source(name: &str, source: SkillSource) -> Result<(), String> {
    validate_skill_name(name)?;
    validate_source(&source)?;
    let path = registry_path()?;
    let mut registry = load_registry_from(&path)?;
    registry.skills.insert(name.to_string(), source);
    save_registry_to(&path, &registry)
}

pub(crate) fn validate_source(source: &SkillSource) -> Result<(), String> {
    let url = source.url.trim().trim_end_matches('/').trim_end_matches(".git");
    let parts = url.strip_prefix("https://github.com/").ok_or_else(|| "Only official HTTPS GitHub repository URLs are supported".to_string())?.split('/').collect::<Vec<_>>();
    if parts.len() != 2 || parts.iter().any(|part| part.is_empty()) { return Err("GitHub URL must be https://github.com/<owner>/<repository>".into()); }
    let subdir = Path::new(&source.subdir);
    if subdir.is_absolute() || subdir.components().any(|part| matches!(part, std::path::Component::ParentDir)) { return Err("Source subdirectory must stay inside the repository".into()); }
    if source.branch.as_deref().is_some_and(|value| value.trim().is_empty() || value.starts_with('-')) { return Err("Invalid branch name".into()); }
    Ok(())
}

pub fn read_text(settings: &AppSettings, name: &str, relative: &str) -> Result<String, String> {
    let root = skill_path(settings, name)?;
    fs::read_to_string(contained_path(&root, relative)?).map_err(|e| e.to_string())
}

pub fn write_text(settings: &AppSettings, name: &str, relative: &str, content: &str) -> Result<(), String> {
    let root = skill_path(settings, name)?;
    let path = contained_path(&root, relative)?;
    let temporary = path.with_extension("skills-manager.tmp");
    fs::write(&temporary, content).map_err(|e| e.to_string())?;
    fs::rename(temporary, path).map_err(|e| e.to_string())
}

fn contained_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute() || relative_path.components().any(|part| matches!(part, std::path::Component::ParentDir)) { return Err("File path escapes the skill directory".into()); }
    Ok(root.join(relative_path))
}

pub fn git_status(settings: &AppSettings, name: &str) -> Result<String, String> {
    if get_source(name)?.is_some() { return Ok("Registered GitHub source".into()); }
    git(settings, name, &["status", "--short", "--branch"])
}

pub fn git_update(settings: &AppSettings, name: &str) -> Result<String, String> {
    if let Some(source) = get_source(name)? { return update_registered(settings, name, &source); }
    let root = skill_path(settings, name)?;
    if !root.join(".git").exists() { return Err("No official GitHub source is registered for this skill".into()); }
    let status = git(settings, name, &["status", "--porcelain"])?;
    if !status.trim().is_empty() { return Err("Skill has local uncommitted changes; update cancelled".into()); }
    git(settings, name, &["pull", "--ff-only"])
}

fn update_registered(settings: &AppSettings, name: &str, source: &SkillSource) -> Result<String, String> {
    validate_source(source)?;
    let target = skill_path(settings, name)?;
    if !target.is_dir() { return Err("Skill directory does not exist".into()); }
    let stamp = Utc::now().format("%Y%m%d%H%M%S%3f");
    let work = std::env::temp_dir().join(format!("skills-manager-update-{name}-{stamp}"));
    let clone = work.join("repo");
    fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    let mut command = git_support::command(settings)?;
    command.args(["clone", "--depth", "1"]);
    if let Some(branch) = source.branch.as_deref() { command.args(["--branch", branch]); }
    let output = command.arg(&source.url).arg(&clone).output().map_err(|e| format!("Cannot run Git: {e}"))?;
    if !output.status.success() { let _ = fs::remove_dir_all(&work); return Err(String::from_utf8_lossy(&output.stderr).trim().to_string()); }
    let source_root = if source.subdir.trim().is_empty() { clone.clone() } else { clone.join(&source.subdir) };
    if !source_root.join("SKILL.md").is_file() { let _ = fs::remove_dir_all(&work); return Err("The configured repository path does not contain SKILL.md".into()); }
    let parent = target.parent().ok_or_else(|| "Invalid canonical skill path".to_string())?;
    let staging = parent.join(format!(".{name}.update-{stamp}"));
    copy_tree_without_git(&source_root, &staging)?;
    let backup_root = settings.recycle_path.parent().unwrap_or(&settings.recycle_path).join("backups");
    fs::create_dir_all(&backup_root).map_err(|e| e.to_string())?;
    let backup = backup_root.join(format!("{name}--update-{stamp}"));
    if let Err(error) = fs::rename(&target, &backup).and_then(|_| fs::rename(&staging, &target)) {
        let _ = fs::remove_dir_all(&staging);
        if backup.exists() && !target.exists() { let _ = fs::rename(&backup, &target); }
        let _ = fs::remove_dir_all(&work);
        return Err(format!("Update replacement failed and was rolled back: {error}"));
    }
    let _ = fs::remove_dir_all(&work);
    Ok(format!("Updated {name} from {}. Previous version: {}", source.url, backup.display()))
}

pub(crate) fn install_from_path(settings: &AppSettings, name: &str, source_root: &Path, source: SkillSource) -> Result<PathBuf, String> {
    validate_skill_name(name)?;
    validate_source(&source)?;
    if !source_root.join("SKILL.md").is_file() { return Err(format!("{name} source does not contain SKILL.md")); }
    fs::create_dir_all(&settings.library_path).map_err(|e| e.to_string())?;
    let target = skill_path(settings, name)?;
    let stamp = Utc::now().format("%Y%m%d%H%M%S%3f");
    let staging = settings.library_path.join(format!(".{name}.install-{stamp}"));
    copy_tree_without_git(source_root, &staging)?;
    let backup_root = settings.recycle_path.parent().unwrap_or(&settings.recycle_path).join("backups");
    fs::create_dir_all(&backup_root).map_err(|e| e.to_string())?;
    let backup = backup_root.join(format!("{name}--repo-install-{stamp}"));
    let had_target = target.exists();
    if had_target { fs::rename(&target, &backup).map_err(|e| format!("Cannot back up {name}: {e}"))?; }
    if let Err(error) = fs::rename(&staging, &target) {
        let _ = fs::remove_dir_all(&staging);
        if had_target { let _ = fs::rename(&backup, &target); }
        return Err(format!("Cannot install {name}: {error}"));
    }
    if let Err(error) = set_source(name, source) {
        let _ = fs::remove_dir_all(&target);
        if had_target { let _ = fs::rename(&backup, &target); }
        return Err(format!("Cannot record source for {name}; installation rolled back: {error}"));
    }
    Ok(target)
}

pub(crate) fn copy_tree_without_git(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target).map_err(|e| e.to_string())?;
    for entry in WalkDir::new(source).follow_links(false).into_iter().filter_map(Result::ok) {
        let relative = entry.path().strip_prefix(source).map_err(|e| e.to_string())?;
        if relative.components().next().is_some_and(|part| part.as_os_str() == ".git") { continue; }
        let destination = target.join(relative);
        if entry.file_type().is_dir() { fs::create_dir_all(destination).map_err(|e| e.to_string())?; }
        else if entry.file_type().is_file() { if let Some(parent) = destination.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; } fs::copy(entry.path(), destination).map_err(|e| e.to_string())?; }
    }
    Ok(())
}

fn git(settings: &AppSettings, name: &str, args: &[&str]) -> Result<String, String> {
    let root = skill_path(settings, name)?;
    if !root.join(".git").exists() { return Err("This skill is not an independent Git repository".into()); }
    git_support::run(settings, args, Some(&root))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contained_path_rejects_parent_segments() {
        assert!(contained_path(Path::new("C:/skills/api"), "../secret").is_err());
        assert!(contained_path(Path::new("C:/skills/api"), "references/doc.md").is_ok());
    }
    #[test]
    fn accepts_github_repository_and_safe_subdirectory() {
        assert!(validate_source(&SkillSource { url: "https://github.com/openai/skills".into(), subdir: "skills/docs".into(), branch: Some("main".into()) }).is_ok());
    }
    #[test]
    fn rejects_non_github_and_parent_subdirectory() {
        assert!(validate_source(&SkillSource { url: "https://example.com/a/b".into(), subdir: "".into(), branch: None }).is_err());
        assert!(validate_source(&SkillSource { url: "https://github.com/a/b".into(), subdir: "../secret".into(), branch: None }).is_err());
    }
    #[test]
    fn registry_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sources.json");
        let mut registry = SourceRegistry::default();
        registry.skills.insert("one".into(), SkillSource { url: "https://github.com/a/b".into(), subdir: "skills/one".into(), branch: None });
        save_registry_to(&path, &registry).unwrap();
        assert_eq!(load_registry_from(&path).unwrap().skills["one"].subdir, "skills/one");
    }

    #[test]
    fn converts_lock_file_skill_path_to_repository_subdirectory() {
        let entry = LegacyLockEntry { source_url: "https://github.com/anthropics/skills.git".into(), skill_path: "skills/frontend-design/SKILL.md".into() };
        let subdir = Path::new(&entry.skill_path).parent().unwrap().to_string_lossy().replace('\\', "/");
        assert_eq!(subdir, "skills/frontend-design");
    }
}
