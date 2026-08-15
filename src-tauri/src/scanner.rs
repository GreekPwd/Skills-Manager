use crate::{domain::{AppSettings, SkillRecord, SkillStatus, SourceKind}, metadata, sources};
use sha2::{Digest, Sha256};
use std::{fs, path::Path, time::UNIX_EPOCH};
use walkdir::WalkDir;

pub fn hash_directory(root: &Path) -> Result<String, String> {
    let mut files = WalkDir::new(root).follow_links(false).into_iter().filter_map(Result::ok).filter(|entry| entry.file_type().is_file()).collect::<Vec<_>>();
    files.sort_by_key(|entry| entry.path().to_path_buf());
    let mut hasher = Sha256::new();
    for entry in files {
        let relative = entry.path().strip_prefix(root).map_err(|e| e.to_string())?;
        hasher.update(relative.to_string_lossy().replace('\\', "/").as_bytes());
        hasher.update(fs::read(entry.path()).map_err(|e| e.to_string())?);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn scan(settings: &AppSettings) -> Result<Vec<SkillRecord>, String> {
    fs::create_dir_all(&settings.library_path).map_err(|e| e.to_string())?;
    let mut skills = Vec::new();
    let registered_sources = sources::all_sources().unwrap_or_default();
    for entry in fs::read_dir(&settings.library_path).map_err(|e| e.to_string())?.flatten().filter(|entry| entry.path().is_dir()) {
        let path = entry.path();
        let skill_file = path.join("SKILL.md");
        if !skill_file.is_file() { continue; }
        let (metadata, metadata_error) = match metadata::parse_skill_file(&skill_file) {
            Ok(metadata) => (metadata, None),
            Err(error) => (metadata::SkillMetadata::default(), Some(error)),
        };
        let id = entry.file_name().to_string_lossy().to_string();
        let git = path.join(".git").exists();
        let registered = registered_sources.get(&id).cloned();
        let agents = settings.agent_paths.iter().filter(|(_, root)| same_target(&root.join(&id), &path)).map(|(id, _)| id.clone()).collect();
        let (file_count, content_hash) = directory_fingerprint(&path)?;
        skills.push(SkillRecord {
            id: id.clone(), name: metadata.name.unwrap_or(id), description: metadata.description.unwrap_or_else(|| metadata_error.clone().unwrap_or_else(|| "未提供描述".into())), path: path.clone(),
            status: if metadata_error.is_some() { SkillStatus::Invalid } else if git || registered.is_some() { SkillStatus::Healthy } else { SkillStatus::Local }, source: if git || registered.is_some() { SourceKind::Git } else { SourceKind::Local },
            source_label: registered.as_ref().map(|source| source.url.clone()).unwrap_or_else(|| if git { "Git 仓库".into() } else { "本地创建".into() }), updated_at: "本机".into(), files: file_count,
            agents, version: metadata.version, content_hash,
            source_url: registered.as_ref().map(|source| source.url.clone()), source_subdir: registered.as_ref().map(|source| source.subdir.clone()), source_branch: registered.and_then(|source| source.branch),
        });
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

fn directory_fingerprint(root: &Path) -> Result<(usize, String), String> {
    let mut files = WalkDir::new(root).follow_links(false).into_iter().filter_map(Result::ok).filter(|entry| entry.file_type().is_file()).map(|entry| {
        let relative = entry.path().strip_prefix(root).map_err(|error| error.to_string())?.to_string_lossy().replace('\\', "/");
        let metadata = entry.metadata().map_err(|error| error.to_string())?;
        let modified = metadata.modified().ok().and_then(|value| value.duration_since(UNIX_EPOCH).ok()).map(|value| value.as_nanos()).unwrap_or_default();
        Ok::<_, String>((relative, metadata.len(), modified))
    }).collect::<Result<Vec<_>, _>>()?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut hasher = Sha256::new();
    for (relative, length, modified) in &files {
        hasher.update(relative.as_bytes());
        hasher.update(length.to_le_bytes());
        hasher.update(modified.to_le_bytes());
    }
    Ok((files.len(), hex::encode(hasher.finalize())))
}

fn same_target(link: &Path, target: &Path) -> bool {
    fs::canonicalize(link).ok().zip(fs::canonicalize(target).ok()).is_some_and(|(a, b)| a == b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn malformed_skill_does_not_abort_library_scan() {
        let root = tempfile::tempdir().unwrap();
        let valid = root.path().join("valid");
        let malformed = root.path().join("malformed");
        fs::create_dir_all(&valid).unwrap();
        fs::create_dir_all(&malformed).unwrap();
        fs::write(valid.join("SKILL.md"), "---\nname: valid\n---\n").unwrap();
        fs::write(malformed.join("SKILL.md"), "---\nname: broken").unwrap();
        let settings = AppSettings {
            library_path: root.path().to_path_buf(),
            recycle_path: root.path().join("recycle"),
            repository_cache_path: root.path().join("repositories"),
            git_proxy: None,
            agent_paths: BTreeMap::new(),
        };

        let skills = scan(&settings).unwrap();

        assert_eq!(skills.len(), 2);
        assert_eq!(skills.iter().find(|skill| skill.id == "malformed").unwrap().status, SkillStatus::Invalid);
    }

    #[test]
    fn directory_hash_changes_with_content() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("SKILL.md"), "one").unwrap();
        let first = hash_directory(dir.path()).unwrap();
        fs::write(dir.path().join("SKILL.md"), "two").unwrap();
        assert_ne!(first, hash_directory(dir.path()).unwrap());
    }
}
