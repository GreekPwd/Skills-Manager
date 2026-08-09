use crate::domain::AppSettings;
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};

pub fn default_settings() -> Result<AppSettings, String> {
    let home = dirs::home_dir().ok_or("无法确定用户主目录")?;
    let root = home.join(".skills-manager");
    let agent_paths = BTreeMap::from([
        ("claude".into(), home.join(".claude").join("skills")),
        ("codex".into(), home.join(".codex").join("skills")),
        ("gemini".into(), home.join(".gemini").join("skills")),
        ("cursor".into(), home.join(".cursor").join("skills")),
    ]);
    Ok(AppSettings { library_path: root.join("skills"), recycle_path: root.join("recycle"), agent_paths })
}

fn settings_file() -> Result<PathBuf, String> {
    dirs::config_dir().map(|path| path.join("skills-manager").join("settings.json")).ok_or_else(|| "无法确定应用配置目录".into())
}

pub fn load() -> Result<AppSettings, String> {
    let path = settings_file()?;
    if !path.exists() { return default_settings(); }
    serde_json::from_str(&fs::read_to_string(&path).map_err(|e| e.to_string())?).map_err(|e| format!("设置文件无效: {e}"))
}

pub fn save(settings: &AppSettings) -> Result<(), String> {
    validate(settings)?;
    let path = settings_file()?;
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::write(path, serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn validate(settings: &AppSettings) -> Result<(), String> {
    if settings.library_path.as_os_str().is_empty() || settings.recycle_path.as_os_str().is_empty() { return Err("仓库路径不能为空".into()); }
    if paths_overlap(&settings.library_path, &settings.recycle_path) { return Err("中央仓库与回收站不能互相包含".into()); }
    Ok(())
}

fn paths_overlap(a: &Path, b: &Path) -> bool { a.starts_with(b) || b.starts_with(a) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_nested_recycle_path() {
        let settings = AppSettings { library_path: "C:/skills".into(), recycle_path: "C:/skills/recycle".into(), agent_paths: BTreeMap::new() };
        assert!(validate(&settings).is_err());
    }
}
