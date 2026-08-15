use crate::{domain::{AppSettings, OperationResult, RepositoryRecord, RepositorySkill, SkillSource}, git_support, metadata, sources};
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, BTreeSet}, fs, path::{Path, PathBuf}};
use walkdir::WalkDir;

#[derive(Debug, Default, Serialize, Deserialize)]
struct RepositoryRegistry {
    #[serde(default)]
    repositories: BTreeMap<String, RepositoryRecord>,
}

fn config_root() -> Result<PathBuf, String> {
    dirs::config_dir().map(|root| root.join("skills-manager")).ok_or_else(|| "Cannot resolve app config directory".into())
}

fn registry_path() -> Result<PathBuf, String> { Ok(config_root()?.join("repositories.json")) }

fn load_registry() -> Result<RepositoryRegistry, String> {
    let path = registry_path()?;
    if !path.is_file() {
        let mut registry = RepositoryRegistry::default();
        seed_from_skill_lock(&mut registry)?;
        save_registry(&registry)?;
        return Ok(registry);
    }
    serde_json::from_str(&fs::read_to_string(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn seed_from_skill_lock(registry: &mut RepositoryRegistry) -> Result<(), String> {
    let path = dirs::home_dir().ok_or("Cannot resolve home directory")?.join(".agents").join(".skill-lock.json");
    if !path.is_file() { return Ok(()); }
    let value: serde_json::Value = serde_json::from_str(&fs::read_to_string(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let Some(skills) = value.get("skills").and_then(|value| value.as_object()) else { return Ok(()); };
    for skill in skills.values() {
        let Some(url) = skill.get("sourceUrl").and_then(|value| value.as_str()) else { continue; };
        let Ok((id, name, normalized_url)) = normalize_repository(url) else { continue; };
        let repository = registry.repositories.entry(id.clone()).or_insert(RepositoryRecord { id, name, url: normalized_url, branch: None, skill_count: 0 });
        repository.skill_count += 1;
    }
    Ok(())
}

fn save_registry(registry: &RepositoryRegistry) -> Result<(), String> {
    let path = registry_path()?;
    fs::create_dir_all(path.parent().ok_or("Invalid repository registry path")?).map_err(|e| e.to_string())?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(registry).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    if path.exists() { fs::remove_file(&path).map_err(|e| e.to_string())?; }
    fs::rename(temporary, path).map_err(|e| e.to_string())
}

fn normalize_repository(input: &str) -> Result<(String, String, String), String> {
    let trimmed = input.trim().trim_end_matches('/').trim_end_matches(".git");
    let slug = if let Some(value) = trimmed.strip_prefix("https://github.com/") { value } else if !trimmed.contains("://") { trimmed } else { return Err("Only HTTPS GitHub repositories are supported".into()); };
    let parts = slug.split('/').collect::<Vec<_>>();
    if parts.len() != 2 || parts.iter().any(|part| part.is_empty() || part.starts_with('.')) { return Err("Repository must be owner/name or https://github.com/owner/name".into()); }
    let name = format!("{}/{}", parts[0], parts[1]);
    let id = format!("{}--{}", parts[0], parts[1]).to_lowercase();
    Ok((id, name.clone(), format!("https://github.com/{name}.git")))
}

fn sync_repository(settings: &AppSettings, repository: &RepositoryRecord) -> Result<PathBuf, String> {
    let root = settings.repository_cache_path.clone();
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let cache = root.join(&repository.id);
    if cache.join(".git").is_dir() {
        git_support::run(settings, &["fetch", "--prune", "origin"], Some(&cache))?;
        let reference = repository.branch.as_deref().unwrap_or("HEAD");
        git_support::run(settings, &["reset", "--hard", &format!("origin/{reference}")], Some(&cache)).or_else(|_| git_support::run(settings, &["reset", "--hard", "FETCH_HEAD"], Some(&cache)))?;
        return Ok(cache);
    }
    if cache.exists() { return Err(format!("Repository cache is invalid: {}", cache.display())); }
    let cache_text = cache.to_string_lossy().to_string();
    let mut args = vec!["clone", "--depth", "1"];
    if let Some(branch) = repository.branch.as_deref() { args.extend(["--branch", branch]); }
    args.extend([repository.url.as_str(), cache_text.as_str()]);
    git_support::run(settings, &args, None)?;
    Ok(cache)
}

fn discover_skills(root: &Path, settings: &AppSettings) -> Result<Vec<RepositorySkill>, String> {
    let mut skills = Vec::new();
    let mut names = BTreeSet::new();
    for entry in WalkDir::new(root).follow_links(false).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() || entry.file_name() != "SKILL.md" { continue; }
        let relative = entry.path().strip_prefix(root).map_err(|e| e.to_string())?;
        if relative.components().any(|part| part.as_os_str() == ".git") { continue; }
        let subdir_path = relative.parent().unwrap_or(Path::new(""));
        let folder_name = subdir_path.file_name().and_then(|value| value.to_str()).unwrap_or("skill");
        let parsed = metadata::parse_skill_file(entry.path()).unwrap_or_default();
        let name = parsed.name.unwrap_or_else(|| folder_name.to_string());
        if name.is_empty() || name.contains('/') || name.contains('\\') || !names.insert(name.clone()) { continue; }
        skills.push(RepositorySkill {
            installed: settings.library_path.join(&name).join("SKILL.md").is_file(),
            name,
            description: parsed.description.unwrap_or_else(|| "No description".into()),
            subdir: subdir_path.to_string_lossy().replace('\\', "/"),
        });
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

pub fn list() -> Result<Vec<RepositoryRecord>, String> {
    Ok(load_registry()?.repositories.into_values().collect())
}

pub fn add(settings: &AppSettings, input: &str, branch: Option<String>) -> Result<RepositoryRecord, String> {
    let (id, name, url) = normalize_repository(input)?;
    let branch = branch.map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
    if branch.as_deref().is_some_and(|value| value.starts_with('-')) { return Err("Invalid branch".into()); }
    let mut repository = RepositoryRecord { id: id.clone(), name, url, branch, skill_count: 0 };
    let cache = sync_repository(settings, &repository)?;
    repository.skill_count = discover_skills(&cache, settings)?.len();
    let mut registry = load_registry()?;
    registry.repositories.insert(id, repository.clone());
    save_registry(&registry)?;
    Ok(repository)
}

pub fn remove(id: &str) -> Result<(), String> {
    let mut registry = load_registry()?;
    if registry.repositories.remove(id).is_none() { return Err("Repository is not registered".into()); }
    save_registry(&registry)
}

pub fn scan(settings: &AppSettings, id: &str) -> Result<Vec<RepositorySkill>, String> {
    let mut registry = load_registry()?;
    let repository = registry.repositories.get(id).cloned().ok_or("Repository is not registered")?;
    let cache = sync_repository(settings, &repository)?;
    let skills = discover_skills(&cache, settings)?;
    if let Some(saved) = registry.repositories.get_mut(id) { saved.skill_count = skills.len(); }
    save_registry(&registry)?;
    Ok(skills)
}

pub fn install(settings: &AppSettings, id: &str, subdirs: &[String]) -> Result<OperationResult, String> {
    if subdirs.is_empty() { return Err("Select at least one Skill".into()); }
    let registry = load_registry()?;
    let repository = registry.repositories.get(id).cloned().ok_or("Repository is not registered")?;
    let cache = sync_repository(settings, &repository)?;
    let available = discover_skills(&cache, settings)?.into_iter().map(|skill| (skill.subdir.clone(), skill)).collect::<BTreeMap<_, _>>();
    let mut affected = Vec::new();
    for subdir in subdirs {
        let skill = available.get(subdir).ok_or_else(|| format!("Skill path is not available: {subdir}"))?;
        let source_root = if subdir.is_empty() { cache.clone() } else { cache.join(subdir) };
        let source = SkillSource { url: repository.url.clone(), subdir: subdir.clone(), branch: repository.branch.clone() };
        affected.push(sources::install_from_path(settings, &skill.name, &source_root, source)?);
    }
    Ok(OperationResult { success: true, message: format!("Installed {} Skills into {}", affected.len(), settings.library_path.display()), affected_paths: affected })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn normalizes_repository_shorthand() {
        let (id, name, url) = normalize_repository("anthropics/skills").unwrap();
        assert_eq!(id, "anthropics--skills");
        assert_eq!(name, "anthropics/skills");
        assert_eq!(url, "https://github.com/anthropics/skills.git");
    }

    #[test]
    fn discovers_nested_skills_and_marks_installed() {
        let root = tempfile::tempdir().unwrap();
        let repository = root.path().join("repo");
        let library = root.path().join(".agents").join("skills");
        fs::create_dir_all(repository.join("skills").join("one")).unwrap();
        fs::write(repository.join("skills").join("one").join("SKILL.md"), "---\nname: one\ndescription: First\n---\n").unwrap();
        fs::create_dir_all(library.join("one")).unwrap();
        fs::write(library.join("one").join("SKILL.md"), "installed").unwrap();
        let settings = AppSettings { library_path: library, recycle_path: root.path().join("recycle"), repository_cache_path: root.path().join("cache"), git_proxy: None, agent_paths: BTreeMap::new() };
        let skills = discover_skills(&repository, &settings).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].subdir, "skills/one");
        assert!(skills[0].installed);
    }
}
