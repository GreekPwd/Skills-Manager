use crate::domain::AppSettings;
use std::{fs, path::{Path, PathBuf}, process::Command};

fn skill_path(settings: &AppSettings, name: &str) -> Result<PathBuf, String> {
    if name.is_empty() || name.chars().any(|character| character == '/' || character == '\\') || name == ".." { return Err("技能名称无效".into()); }
    Ok(settings.library_path.join(name))
}

pub fn read_text(settings: &AppSettings, name: &str, relative: &str) -> Result<String, String> {
    let root = skill_path(settings, name)?;
    let path = contained_path(&root, relative)?;
    fs::read_to_string(path).map_err(|e| e.to_string())
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
    if relative_path.is_absolute() || relative_path.components().any(|part| matches!(part, std::path::Component::ParentDir)) { return Err("文件路径越过技能目录".into()); }
    Ok(root.join(relative_path))
}

pub fn git_status(settings: &AppSettings, name: &str) -> Result<String, String> {
    git(settings, name, &["status", "--short", "--branch"])
}

pub fn git_update(settings: &AppSettings, name: &str) -> Result<String, String> {
    let status = git(settings, name, &["status", "--porcelain"])?;
    if !status.trim().is_empty() { return Err("Skill 有未提交的本地修改，更新已取消".into()); }
    git(settings, name, &["pull", "--ff-only"])
}

fn git(settings: &AppSettings, name: &str, args: &[&str]) -> Result<String, String> {
    let root = skill_path(settings, name)?;
    if !root.join(".git").exists() { return Err("该 Skill 不是独立 Git 仓库".into()); }
    let output = Command::new("git").args(args).current_dir(root).output().map_err(|e| format!("无法运行 Git: {e}"))?;
    if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).trim().to_string()); }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contained_path_rejects_parent_segments() {
        assert!(contained_path(Path::new("C:/skills/api"), "../secret").is_err());
        assert!(contained_path(Path::new("C:/skills/api"), "references/doc.md").is_ok());
    }
}
