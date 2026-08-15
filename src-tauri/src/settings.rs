use crate::domain::AppSettings;
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};

pub fn default_settings() -> Result<AppSettings, String> {
    let home = dirs::home_dir().ok_or("无法确定用户主目录")?;
    Ok(default_settings_for_home(&home))
}

fn default_settings_for_home(home: &Path) -> AppSettings {
    let root = home.join(".skills-manager");
    let agents_skills = home.join(".agents").join("skills");
    let codex_skills = home.join(".codex").join("skills");
    let agent_paths = BTreeMap::from([
        ("claude".into(), home.join(".claude").join("skills")),
        ("codex".into(), codex_skills.clone()),
        ("gemini".into(), home.join(".gemini").join("skills")),
        ("cursor".into(), home.join(".cursor").join("skills")),
        ("agentbro".into(), home.join(".agentbro").join("skills")),
    ]);
    AppSettings {
        library_path: agents_skills,
        recycle_path: root.join("recycle"),
        repository_cache_path: root.join("repositories"),
        git_proxy: None,
        agent_paths,
    }
}

fn settings_file() -> Result<PathBuf, String> {
    dirs::config_dir().map(|path| path.join("skills-manager").join("settings.json")).ok_or_else(|| "无法确定应用配置目录".into())
}

pub fn load() -> Result<AppSettings, String> {
    let path = settings_file()?;
    if !path.exists() { return default_settings(); }
    let mut settings: AppSettings = serde_json::from_str(&fs::read_to_string(&path).map_err(|e| e.to_string())?).map_err(|e| format!("设置文件无效: {e}"))?;
    let defaults = default_settings()?;
    settings.library_path = defaults.library_path;
    if settings.repository_cache_path.as_os_str().is_empty() { settings.repository_cache_path = defaults.repository_cache_path; }
    settings.git_proxy = normalize_proxy(settings.git_proxy.as_deref())?;
    validate(&settings)?;
    Ok(settings)
}

pub fn save(settings: &AppSettings) -> Result<(), String> {
    let mut settings = settings.clone();
    settings.library_path = dirs::home_dir().ok_or("Cannot resolve home directory")?.join(".agents").join("skills");
    settings.git_proxy = normalize_proxy(settings.git_proxy.as_deref())?;
    validate(&settings)?;
    let path = settings_file()?;
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::write(path, serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

fn validate(settings: &AppSettings) -> Result<(), String> {
    if settings.library_path.as_os_str().is_empty() || settings.recycle_path.as_os_str().is_empty() || settings.repository_cache_path.as_os_str().is_empty() { return Err("仓库路径和缓存路径不能为空".into()); }
    if paths_overlap(&settings.library_path, &settings.recycle_path) { return Err("中央仓库与回收站不能互相包含".into()); }
    if paths_overlap(&settings.library_path, &settings.repository_cache_path) { return Err("中央仓库与仓库缓存不能互相包含".into()); }
    normalize_proxy(settings.git_proxy.as_deref())?;
    Ok(())
}

pub(crate) fn normalize_proxy(value: Option<&str>) -> Result<Option<String>, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else { return Ok(None); };
    if value.chars().any(char::is_whitespace) { return Err("代理地址不能包含空白字符".into()); }
    let supported = ["http://", "https://", "socks5://", "socks5h://"];
    let remainder = supported.iter().find_map(|prefix| value.strip_prefix(prefix)).ok_or("代理地址必须以 http://、https://、socks5:// 或 socks5h:// 开头")?;
    if remainder.is_empty() || !remainder.contains(':') { return Err("代理地址必须包含主机和端口，例如 http://127.0.0.1:7890".into()); }
    Ok(Some(value.to_string()))
}

fn paths_overlap(a: &Path, b: &Path) -> bool { a.starts_with(b) || b.starts_with(a) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_agents_skills_as_default_library() {
        let home = tempfile::tempdir().unwrap();
        let agents_skills = home.path().join(".agents").join("skills");
        fs::create_dir_all(&agents_skills).unwrap();

        let settings = default_settings_for_home(home.path());

        assert_eq!(settings.library_path, agents_skills);
        assert_eq!(settings.repository_cache_path, home.path().join(".skills-manager").join("repositories"));
    }

    #[test]
    fn does_not_fall_back_to_legacy_codex_library() {
        let home = tempfile::tempdir().unwrap();
        let codex_skills = home.path().join(".codex").join("skills");
        fs::create_dir_all(&codex_skills).unwrap();

        let settings = default_settings_for_home(home.path());

        assert_eq!(settings.library_path, home.path().join(".agents").join("skills"));
    }

    #[test]
    fn rejects_nested_recycle_path() {
        let settings = AppSettings { library_path: "C:/skills".into(), recycle_path: "C:/skills/recycle".into(), repository_cache_path: "C:/cache".into(), git_proxy: None, agent_paths: BTreeMap::new() };
        assert!(validate(&settings).is_err());
    }

    #[test]
    fn accepts_http_and_socks_proxies() {
        assert_eq!(normalize_proxy(Some(" http://127.0.0.1:7890 ")).unwrap().as_deref(), Some("http://127.0.0.1:7890"));
        assert!(normalize_proxy(Some("socks5h://localhost:1080")).is_ok());
        assert!(normalize_proxy(Some("127.0.0.1:7890")).is_err());
    }
}
