use crate::{domain::{AppSettings, SkillRecord, SkillStatus, SourceKind}, metadata};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
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
    for entry in fs::read_dir(&settings.library_path).map_err(|e| e.to_string())?.flatten().filter(|entry| entry.path().is_dir()) {
        let path = entry.path();
        let skill_file = path.join("SKILL.md");
        if !skill_file.is_file() { continue; }
        let metadata = metadata::parse_skill_file(&skill_file)?;
        let id = entry.file_name().to_string_lossy().to_string();
        let git = path.join(".git").exists();
        let agents = settings.agent_paths.iter().filter(|(_, root)| same_target(&root.join(&id), &path)).map(|(id, _)| id.clone()).collect();
        let file_count = WalkDir::new(&path).into_iter().filter_map(Result::ok).filter(|e| e.file_type().is_file()).count();
        skills.push(SkillRecord {
            id: id.clone(), name: metadata.name.unwrap_or(id), description: metadata.description.unwrap_or_else(|| "未提供描述".into()), path: path.clone(),
            status: if git { SkillStatus::Healthy } else { SkillStatus::Local }, source: if git { SourceKind::Git } else { SourceKind::Local },
            source_label: if git { "Git 仓库".into() } else { "本地创建".into() }, updated_at: "本机".into(), files: file_count,
            agents, version: metadata.version, content_hash: hash_directory(&path)?,
        });
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

fn same_target(link: &Path, target: &Path) -> bool {
    fs::canonicalize(link).ok().zip(fs::canonicalize(target).ok()).is_some_and(|(a, b)| a == b)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn directory_hash_changes_with_content() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("SKILL.md"), "one").unwrap();
        let first = hash_directory(dir.path()).unwrap();
        fs::write(dir.path().join("SKILL.md"), "two").unwrap();
        assert_ne!(first, hash_directory(dir.path()).unwrap());
    }
}
