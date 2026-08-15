use crate::{domain::{AppSettings, OperationResult}, scanner};
use chrono::Utc;
use std::{fs, path::{Path, PathBuf}};
use walkdir::WalkDir;

fn safe_child(root: &Path, name: &str) -> Result<PathBuf, String> {
    if name.is_empty() || name == "." || name == ".." || name.chars().any(|character| character == '/' || character == '\\') { return Err("技能名称包含无效路径字符".into()); }
    Ok(root.join(name))
}

fn copy_tree(source: &Path, target: &Path) -> Result<(), String> {
    if !source.join("SKILL.md").is_file() { return Err("源目录不包含 SKILL.md".into()); }
    if target.exists() { return Err(format!("目标已存在: {}", target.display())); }
    for entry in WalkDir::new(source).follow_links(false) {
        let entry = entry.map_err(|e| e.to_string())?;
        let relative = entry.path().strip_prefix(source).map_err(|e| e.to_string())?;
        let destination = target.join(relative);
        if entry.file_type().is_dir() { fs::create_dir_all(&destination).map_err(|e| e.to_string())?; }
        else if entry.file_type().is_file() { if let Some(parent) = destination.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; } fs::copy(entry.path(), destination).map_err(|e| e.to_string())?; }
    }
    Ok(())
}

pub fn import(settings: &AppSettings, source: &Path, name: &str) -> Result<OperationResult, String> {
    fs::create_dir_all(&settings.library_path).map_err(|e| e.to_string())?;
    let target = safe_child(&settings.library_path, name)?;
    copy_tree(source, &target)?;
    if scanner::hash_directory(source)? != scanner::hash_directory(&target)? { let _ = fs::remove_dir_all(&target); return Err("导入后的内容校验失败".into()); }
    Ok(OperationResult { success: true, message: format!("已导入 {name}"), affected_paths: vec![target] })
}

pub fn distribute(settings: &AppSettings, name: &str, agent_ids: &[String]) -> Result<OperationResult, String> {
    let target = safe_child(&settings.library_path, name)?;
    if !target.join("SKILL.md").is_file() { return Err("中央仓库中不存在该技能".into()); }
    let mut affected = Vec::new();
    for id in agent_ids {
        let root = settings.agent_paths.get(id).ok_or_else(|| format!("未知 Agent: {id}"))?;
        fs::create_dir_all(root).map_err(|e| e.to_string())?;
        let link = safe_child(root, name)?;
        if link.exists() {
            if fs::canonicalize(&link).ok() == fs::canonicalize(&target).ok() { continue; }
            return Err(format!("{} 已存在同名目录，请先解决冲突", link.display()));
        }
        create_directory_link(&target, &link)?;
        affected.push(link);
    }
    Ok(OperationResult { success: true, message: format!("已分发到 {} 个 Agent", affected.len()), affected_paths: affected })
}

#[cfg(windows)]
fn create_directory_link(target: &Path, link: &Path) -> Result<(), String> { junction::create(target, link).map_err(|e| format!("创建 Junction 失败: {e}")) }

#[cfg(not(windows))]
fn create_directory_link(target: &Path, link: &Path) -> Result<(), String> { std::os::unix::fs::symlink(target, link).map_err(|e| e.to_string()) }

fn same_target(left: &Path, right: &Path) -> bool {
    fs::canonicalize(left).ok().zip(fs::canonicalize(right).ok()).is_some_and(|(left, right)| left == right)
}

#[cfg(windows)]
fn directly_links_to(link: &Path, target: &Path) -> bool {
    junction::get_target(link).ok().is_some_and(|value| normalize_windows_path(&value) == normalize_windows_path(target))
}

#[cfg(windows)]
fn normalize_windows_path(path: &Path) -> String {
    path.to_string_lossy().trim_start_matches(r"\\?\").replace('/', "\\").trim_end_matches('\\').to_lowercase()
}

#[cfg(not(windows))]
fn directly_links_to(link: &Path, target: &Path) -> bool { fs::read_link(link).ok().is_some_and(|value| value == target) }

#[cfg(windows)]
fn directory_link_target(link: &Path) -> Result<PathBuf, String> { junction::get_target(link).map_err(|e| e.to_string()) }

#[cfg(not(windows))]
fn directory_link_target(link: &Path) -> Result<PathBuf, String> { fs::read_link(link).map_err(|e| e.to_string()) }

fn path_entry_exists(path: &Path) -> bool { fs::symlink_metadata(path).is_ok() }

pub fn consolidate(settings: &AppSettings, agent_ids: &[String]) -> Result<OperationResult, String> {
    fs::create_dir_all(&settings.library_path).map_err(|e| e.to_string())?;
    let backup_root = settings.recycle_path.parent().ok_or("无法确定备份目录")?.join("backups");

    for id in agent_ids {
        let root = settings.agent_paths.get(id).ok_or_else(|| format!("未知 Agent: {id}"))?;
        if directly_links_to(root, &settings.library_path) { continue; }
        if root.starts_with(&settings.library_path) || settings.library_path.starts_with(root) {
            return Err(format!("{} 与中央仓库路径重叠，无法安全统一", root.display()));
        }
    }

    let mut affected = Vec::new();
    let mut consolidated = 0usize;
    for id in agent_ids {
        let root = settings.agent_paths.get(id).ok_or_else(|| format!("未知 Agent: {id}"))?;
        if directly_links_to(root, &settings.library_path) { continue; }
        if let Some(parent) = root.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }

        let backup = if path_entry_exists(root) {
            fs::create_dir_all(&backup_root).map_err(|e| e.to_string())?;
            let name = format!("{}--{}", id, Utc::now().format("%Y%m%d%H%M%S%3f"));
            let backup = safe_child(&backup_root, &name)?;
            fs::rename(root, &backup).map_err(|e| format!("备份 {} 失败: {e}", root.display()))?;
            Some(backup)
        } else { None };

        if let Err(error) = create_directory_link(&settings.library_path, root) {
            if let Some(backup) = &backup { let _ = fs::rename(backup, root); }
            return Err(error);
        }

        let mut imported = Vec::new();
        if let Some(backup) = &backup {
            let import_result = (|| -> Result<(), String> {
                for entry in fs::read_dir(backup).map_err(|e| e.to_string())? {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let source = entry.path();
                    if !source.join("SKILL.md").is_file() { continue; }
                    let name = entry.file_name().to_string_lossy().to_string();
                    let target = safe_child(&settings.library_path, &name)?;
                    if target.exists() { continue; }
                    copy_tree(&source, &target)?;
                    if scanner::hash_directory(&source)? != scanner::hash_directory(&target)? {
                        let _ = fs::remove_dir_all(&target);
                        return Err(format!("导入 {name} 后内容校验失败"));
                    }
                    imported.push(target);
                }
                Ok(())
            })();
            if let Err(error) = import_result {
                for path in imported.iter().rev() { let _ = fs::remove_dir_all(path); }
                let _ = remove_directory_link(root);
                let _ = fs::rename(backup, root);
                return Err(format!("统一 {id} 失败，已回滚: {error}"));
            }
            affected.push(backup.clone());
        }
        affected.extend(imported);
        affected.push(root.clone());
        consolidated += 1;
    }

    Ok(OperationResult { success: true, message: format!("已将 {consolidated} 个 Agent 统一到中央仓库"), affected_paths: affected })
}

pub fn migrate_canonical(settings: &AppSettings, source_roots: &[PathBuf]) -> Result<OperationResult, String> {
    fs::create_dir_all(&settings.library_path).map_err(|e| e.to_string())?;
    let backup_root = settings.recycle_path.parent().ok_or("无法确定备份目录")?.join("backups");
    let mut moved = Vec::<(PathBuf, PathBuf)>::new();
    let mut moved_aliases = Vec::<(PathBuf, PathBuf)>::new();
    let mut imported = Vec::<PathBuf>::new();
    let mut linked = Vec::<PathBuf>::new();
    let mut relinked = Vec::<(PathBuf, PathBuf)>::new();
    let mut affected = Vec::new();

    fs::create_dir_all(&backup_root).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(&settings.library_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let central_child = entry.path();
        let name = entry.file_name();
        let aliases_source = source_roots.iter().filter(|root| !same_target(root, &settings.library_path)).any(|root| same_target(&central_child, &root.join(&name)));
        if !aliases_source { continue; }
        let backup = backup_root.join(format!("central-alias-{}--{}", name.to_string_lossy(), Utc::now().format("%Y%m%d%H%M%S%3f")));
        fs::rename(&central_child, &backup).map_err(|e| format!("备份中央仓库别名 {} 失败: {e}", central_child.display()))?;
        moved_aliases.push((central_child, backup.clone()));
        affected.push(backup);
    }

    for root in source_roots {
        if same_target(root, &settings.library_path) || !path_entry_exists(root) { continue; }
        if root.starts_with(&settings.library_path) || settings.library_path.starts_with(root) {
            return Err(format!("{} 与中央仓库路径重叠，无法安全迁移", root.display()));
        }
        let id = settings.agent_paths.iter().find(|(_, path)| *path == root).map(|(id, _)| id.as_str()).unwrap_or("legacy");
        let backup = backup_root.join(format!("{}--{}", id, Utc::now().format("%Y%m%d%H%M%S%3f")));
        if let Err(error) = fs::rename(root, &backup) {
            for (original, saved) in moved.iter().rev() { if path_entry_exists(saved) { let _ = fs::rename(saved, original); } }
            for (original, saved) in moved_aliases.iter().rev() { if path_entry_exists(saved) { let _ = fs::rename(saved, original); } }
            return Err(format!("备份 {} 失败，已回滚: {error}", root.display()));
        }
        moved.push((root.clone(), backup.clone()));
        affected.push(backup);
    }

    let operation = (|| -> Result<(), String> {
        for (_, backup) in &moved {
            for entry in fs::read_dir(backup).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let source = entry.path();
                if !source.join("SKILL.md").is_file() { continue; }
                let name = entry.file_name().to_string_lossy().to_string();
                let target = safe_child(&settings.library_path, &name)?;
                if target.exists() { continue; }
                copy_tree(&source, &target)?;
                if scanner::hash_directory(&source)? != scanner::hash_directory(&target)? {
                    let _ = fs::remove_dir_all(&target);
                    return Err(format!("导入 {name} 后内容校验失败"));
                }
                imported.push(target);
            }
        }

        for root in settings.agent_paths.values() {
            if directly_links_to(root, &settings.library_path) { continue; }
            if let Some(parent) = root.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
            if same_target(root, &settings.library_path) {
                let previous_target = directory_link_target(root)?;
                remove_directory_link(root)?;
                if let Err(error) = create_directory_link(&settings.library_path, root) {
                    let _ = create_directory_link(&previous_target, root);
                    return Err(format!("重建 {} 的直接链接失败: {error}", root.display()));
                }
                relinked.push((root.clone(), previous_target));
                affected.push(root.clone());
                continue;
            }
            if path_entry_exists(root) {
                let id = settings.agent_paths.iter().find(|(_, path)| *path == root).map(|(id, _)| id.as_str()).unwrap_or("agent");
                fs::create_dir_all(&backup_root).map_err(|e| e.to_string())?;
                let backup = backup_root.join(format!("{}--{}", id, Utc::now().format("%Y%m%d%H%M%S%3f")));
                fs::rename(root, &backup).map_err(|e| format!("备份 {} 失败: {e}", root.display()))?;
                moved.push((root.clone(), backup.clone()));
                affected.push(backup);
            }
            create_directory_link(&settings.library_path, root)?;
            linked.push(root.clone());
            affected.push(root.clone());
        }
        Ok(())
    })();

    if let Err(error) = operation {
        for path in linked.iter().rev() { let _ = remove_directory_link(path); }
        for (path, previous_target) in relinked.iter().rev() { let _ = remove_directory_link(path); let _ = create_directory_link(previous_target, path); }
        for path in imported.iter().rev() { let _ = fs::remove_dir_all(path); }
        for (original, backup) in moved.iter().rev() {
            if path_entry_exists(backup) { let _ = fs::rename(backup, original); }
        }
        for (original, backup) in moved_aliases.iter().rev() {
            if path_entry_exists(backup) { let _ = fs::rename(backup, original); }
        }
        return Err(format!("中央仓库迁移失败，已回滚: {error}"));
    }

    Ok(OperationResult { success: true, message: format!("已将 {} 个来源合并到中央仓库", moved.len()), affected_paths: affected })
}

pub fn delete(settings: &AppSettings, name: &str) -> Result<OperationResult, String> {
    let source = safe_child(&settings.library_path, name)?;
    if !source.exists() { return Err("技能不存在".into()); }
    fs::create_dir_all(&settings.recycle_path).map_err(|e| e.to_string())?;
    for root in settings.agent_paths.values() {
        if fs::canonicalize(root).ok() == fs::canonicalize(&settings.library_path).ok() { continue; }
        let link = safe_child(root, name)?;
        if link.exists() && fs::canonicalize(&link).ok() == fs::canonicalize(&source).ok() { remove_directory_link(&link)?; }
    }
    let recycle_name = format!("{}--{}", name, Utc::now().format("%Y%m%d%H%M%S"));
    let destination = safe_child(&settings.recycle_path, &recycle_name)?;
    fs::rename(&source, &destination).map_err(|e| format!("移动到回收站失败: {e}"))?;
    Ok(OperationResult { success: true, message: format!("{name} 已移入回收站"), affected_paths: vec![source, destination] })
}

pub fn restore(settings: &AppSettings, recycle_name: &str) -> Result<OperationResult, String> {
    let source = safe_child(&settings.recycle_path, recycle_name)?;
    let name = recycle_name.split("--").next().ok_or("回收站名称无效")?;
    let destination = safe_child(&settings.library_path, name)?;
    if destination.exists() { return Err("中央仓库已有同名技能".into()); }
    fs::rename(&source, &destination).map_err(|e| format!("恢复失败: {e}"))?;
    Ok(OperationResult { success: true, message: format!("已恢复 {name}"), affected_paths: vec![destination] })
}

fn remove_directory_link(path: &Path) -> Result<(), String> {
    #[cfg(windows)] { fs::remove_dir(path).map_err(|e| e.to_string()) }
    #[cfg(not(windows))] { fs::remove_file(path).map_err(|e| e.to_string()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn test_settings(root: &Path) -> AppSettings {
        let library = root.join("library");
        AppSettings {
            library_path: library.clone(),
            recycle_path: root.join("recycle"),
            repository_cache_path: root.join("repositories"),
            git_proxy: None,
            agent_paths: BTreeMap::from([("codex".into(), library)]),
        }
    }

    #[test]
    fn deleting_from_a_library_that_is_also_an_agent_root_is_safe() {
        let root = tempfile::tempdir().unwrap();
        let settings = test_settings(root.path());
        let skill = settings.library_path.join("shared");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "shared").unwrap();

        delete(&settings, "shared").unwrap();

        assert!(!skill.exists());
        assert_eq!(fs::read_dir(&settings.recycle_path).unwrap().count(), 1);
    }

    #[test]
    fn consolidates_agent_roots_into_the_library_with_backups() {
        let root = tempfile::tempdir().unwrap();
        let mut settings = test_settings(root.path());
        let claude = root.path().join("claude").join("skills");
        let cursor = root.path().join("cursor").join("skills");
        settings.agent_paths.insert("claude".into(), claude.clone());
        settings.agent_paths.insert("cursor".into(), cursor.clone());
        fs::create_dir_all(settings.library_path.join("shared")).unwrap();
        fs::write(settings.library_path.join("shared").join("SKILL.md"), "central").unwrap();
        fs::create_dir_all(claude.join("shared")).unwrap();
        fs::write(claude.join("shared").join("SKILL.md"), "claude variant").unwrap();
        fs::create_dir_all(claude.join("claude-only")).unwrap();
        fs::write(claude.join("claude-only").join("SKILL.md"), "unique").unwrap();

        let result = consolidate(&settings, &["claude".into(), "cursor".into()]).unwrap();

        assert_eq!(fs::canonicalize(&claude).unwrap(), fs::canonicalize(&settings.library_path).unwrap());
        assert_eq!(fs::canonicalize(&cursor).unwrap(), fs::canonicalize(&settings.library_path).unwrap());
        assert_eq!(fs::read_to_string(settings.library_path.join("shared").join("SKILL.md")).unwrap(), "central");
        assert!(settings.library_path.join("claude-only").join("SKILL.md").is_file());
        let backup = result.affected_paths.iter().find(|path| path.file_name().is_some_and(|name| name.to_string_lossy().starts_with("claude--"))).unwrap();
        assert_eq!(fs::read_to_string(backup.join("shared").join("SKILL.md")).unwrap(), "claude variant");
    }

    #[test]
    fn migrates_multiple_legacy_roots_into_agents_canonical_root() {
        let root = tempfile::tempdir().unwrap();
        let central = root.path().join(".agents").join("skills");
        let codex = root.path().join(".codex").join("skills");
        let agentbro = root.path().join(".agentbro").join("skills");
        let claude = root.path().join(".claude").join("skills");
        let cursor = root.path().join(".cursor").join("skills");
        let settings = AppSettings {
            library_path: central.clone(),
            recycle_path: root.path().join("recycle"),
            repository_cache_path: root.path().join("repositories"),
            git_proxy: None,
            agent_paths: BTreeMap::from([
                ("codex".into(), codex.clone()),
                ("agentbro".into(), agentbro.clone()),
                ("claude".into(), claude.clone()),
                ("cursor".into(), cursor.clone()),
            ]),
        };
        fs::create_dir_all(central.join("shared")).unwrap();
        fs::write(central.join("shared").join("SKILL.md"), "central").unwrap();
        fs::create_dir_all(codex.join("shared")).unwrap();
        fs::write(codex.join("shared").join("SKILL.md"), "codex").unwrap();
        fs::create_dir_all(codex.join("codex-only")).unwrap();
        fs::write(codex.join("codex-only").join("SKILL.md"), "codex-only").unwrap();
        fs::create_dir_all(agentbro.join("agentbro-only")).unwrap();
        fs::write(agentbro.join("agentbro-only").join("SKILL.md"), "agentbro-only").unwrap();

        let result = migrate_canonical(&settings, &[codex.clone(), agentbro.clone()]).unwrap();

        assert_eq!(fs::canonicalize(&codex).unwrap(), fs::canonicalize(&central).unwrap());
        assert_eq!(fs::canonicalize(&agentbro).unwrap(), fs::canonicalize(&central).unwrap());
        assert_eq!(fs::canonicalize(&claude).unwrap(), fs::canonicalize(&central).unwrap());
        assert_eq!(fs::canonicalize(&cursor).unwrap(), fs::canonicalize(&central).unwrap());
        assert_eq!(fs::read_to_string(central.join("shared").join("SKILL.md")).unwrap(), "central");
        assert!(central.join("codex-only").join("SKILL.md").is_file());
        assert!(central.join("agentbro-only").join("SKILL.md").is_file());
        assert!(result.affected_paths.iter().any(|path| path.to_string_lossy().contains("codex--")));
        assert!(result.affected_paths.iter().any(|path| path.to_string_lossy().contains("agentbro--")));

        migrate_canonical(&settings, &[codex.clone(), agentbro.clone()]).unwrap();
        assert_eq!(fs::read_to_string(central.join("shared").join("SKILL.md")).unwrap(), "central");
        assert!(central.join("codex-only").join("SKILL.md").is_file());
        assert!(central.join("agentbro-only").join("SKILL.md").is_file());
    }

    #[test]
    fn rejects_path_traversal_names() {
        assert!(safe_child(Path::new("C:/skills"), "../outside").is_err());
        assert!(safe_child(Path::new("C:/skills"), "nested/name").is_err());
    }
}
