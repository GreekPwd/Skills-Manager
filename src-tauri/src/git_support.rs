use crate::{domain::AppSettings, settings};
use std::{path::Path, process::Command};

pub fn command(settings: &AppSettings) -> Result<Command, String> {
    let mut command = Command::new("git");
    let configured = settings::normalize_proxy(settings.git_proxy.as_deref())?;
    if let Some(proxy) = configured.or_else(windows_system_proxy) {
        command.arg("-c").arg(format!("http.proxy={proxy}"));
        command.arg("-c").arg(format!("https.proxy={proxy}"));
    }
    Ok(command)
}

#[cfg(windows)]
fn windows_system_proxy() -> Option<String> {
    const KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings";
    let enabled = Command::new("reg.exe").args(["query", KEY, "/v", "ProxyEnable"]).output().ok()?;
    let server = Command::new("reg.exe").args(["query", KEY, "/v", "ProxyServer"]).output().ok()?;
    if !enabled.status.success() || !server.status.success() { return None; }
    parse_windows_proxy(&String::from_utf8_lossy(&enabled.stdout), &String::from_utf8_lossy(&server.stdout))
}

#[cfg(not(windows))]
fn windows_system_proxy() -> Option<String> { None }

#[cfg(windows)]
fn parse_windows_proxy(enabled: &str, server: &str) -> Option<String> {
    let enabled = enabled.lines().find(|line| line.contains("ProxyEnable"))?.split_whitespace().last()?;
    if enabled != "0x1" && enabled != "1" { return None; }
    let value = server.lines().find(|line| line.contains("ProxyServer"))?.split_whitespace().last()?.trim();
    let value = if value.contains('=') {
        value.split(';').find_map(|part| part.strip_prefix("https=")).or_else(|| value.split(';').find_map(|part| part.strip_prefix("http="))).or_else(|| value.split(';').find_map(|part| part.strip_prefix("socks=")))?
    } else { value };
    if value.contains("://") { Some(value.to_string()) } else { Some(format!("http://{value}")) }
}

pub fn run(settings: &AppSettings, args: &[&str], current_dir: Option<&Path>) -> Result<String, String> {
    let mut command = command(settings)?;
    command.args(args);
    if let Some(path) = current_dir { command.current_dir(path); }
    let output = command.output().map_err(|e| format!("Cannot run Git: {e}"))?;
    if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).trim().to_string()); }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, path::PathBuf};

    fn settings_with_proxy(proxy: Option<&str>) -> AppSettings {
        AppSettings {
            library_path: PathBuf::from("C:/skills"),
            recycle_path: PathBuf::from("C:/recycle"),
            repository_cache_path: PathBuf::from("C:/cache"),
            git_proxy: proxy.map(str::to_string),
            agent_paths: BTreeMap::new(),
        }
    }

    #[test]
    fn applies_proxy_only_to_the_spawned_git_command() {
        let command = command(&settings_with_proxy(Some("http://127.0.0.1:7890"))).unwrap();
        let args = command.get_args().map(|arg| arg.to_string_lossy().to_string()).collect::<Vec<_>>();
        assert_eq!(args, ["-c", "http.proxy=http://127.0.0.1:7890", "-c", "https.proxy=http://127.0.0.1:7890"]);
    }

    #[test]
    fn direct_mode_does_not_add_proxy_arguments() {
        let settings = settings_with_proxy(None);
        assert!(command(&settings).is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn parses_enabled_windows_proxy() {
        let enabled = "ProxyEnable    REG_DWORD    0x1";
        let server = "ProxyServer    REG_SZ    127.0.0.1:7897";
        assert_eq!(parse_windows_proxy(enabled, server).as_deref(), Some("http://127.0.0.1:7897"));
        assert_eq!(parse_windows_proxy("ProxyEnable REG_DWORD 0x0", server), None);
    }
}
