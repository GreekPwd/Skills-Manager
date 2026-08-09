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

pub fn delete(settings: &AppSettings, name: &str) -> Result<OperationResult, String> {
    let source = safe_child(&settings.library_path, name)?;
    if !source.exists() { return Err("技能不存在".into()); }
    fs::create_dir_all(&settings.recycle_path).map_err(|e| e.to_string())?;
    for root in settings.agent_paths.values() {
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
    #[test]
    fn rejects_path_traversal_names() {
        assert!(safe_child(Path::new("C:/skills"), "../outside").is_err());
        assert!(safe_child(Path::new("C:/skills"), "nested/name").is_err());
    }
}
