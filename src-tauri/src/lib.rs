mod agents;
mod domain;
mod git_support;
mod metadata;
mod operations;
mod repositories;
mod scanner;
mod settings;
mod sources;

use domain::{AgentRecord, AppSettings, OperationResult, RepositoryRecord, RepositorySkill, SkillRecord, SkillSource};

async fn run_blocking<T, F>(operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation).await.map_err(|error| format!("后台任务失败: {error}"))?
}

#[tauri::command]
fn get_settings() -> Result<AppSettings, String> { settings::load() }

#[tauri::command]
fn save_settings(value: AppSettings) -> Result<(), String> { settings::save(&value) }

#[tauri::command]
async fn detect_agents() -> Result<Vec<AgentRecord>, String> { run_blocking(|| Ok(agents::detect(&settings::load()?))).await }

#[tauri::command]
async fn scan_library() -> Result<Vec<SkillRecord>, String> { run_blocking(|| scanner::scan(&settings::load()?)).await }

#[tauri::command]
async fn import_skill(source: String, name: String) -> Result<OperationResult, String> { run_blocking(move || operations::import(&settings::load()?, std::path::Path::new(&source), &name)).await }

#[tauri::command]
async fn distribute_skill(name: String, agent_ids: Vec<String>) -> Result<OperationResult, String> { run_blocking(move || operations::distribute(&settings::load()?, &name, &agent_ids)).await }

#[tauri::command]
async fn consolidate_agents(agent_ids: Vec<String>) -> Result<OperationResult, String> { run_blocking(move || operations::consolidate(&settings::load()?, &agent_ids)).await }

#[tauri::command]
async fn migrate_canonical() -> Result<OperationResult, String> {
    run_blocking(|| {
        let settings = settings::load()?;
        let source_roots = ["codex", "agentbro", "gemini"].into_iter().filter_map(|id| settings.agent_paths.get(id).cloned()).collect::<Vec<_>>();
        operations::migrate_canonical(&settings, &source_roots)
    }).await
}

#[tauri::command]
async fn delete_skill(name: String) -> Result<OperationResult, String> { run_blocking(move || operations::delete(&settings::load()?, &name)).await }

#[tauri::command]
async fn restore_skill(recycle_name: String) -> Result<OperationResult, String> { run_blocking(move || operations::restore(&settings::load()?, &recycle_name)).await }

#[tauri::command]
async fn read_skill_file(name: String, relative: String) -> Result<String, String> { run_blocking(move || sources::read_text(&settings::load()?, &name, &relative)).await }

#[tauri::command]
async fn write_skill_file(name: String, relative: String, content: String) -> Result<(), String> { run_blocking(move || sources::write_text(&settings::load()?, &name, &relative, &content)).await }

#[tauri::command]
async fn git_status(name: String) -> Result<String, String> { run_blocking(move || sources::git_status(&settings::load()?, &name)).await }

#[tauri::command]
async fn git_update(name: String) -> Result<String, String> { run_blocking(move || sources::git_update(&settings::load()?, &name)).await }

#[tauri::command]
async fn get_skill_source(name: String) -> Result<Option<SkillSource>, String> { run_blocking(move || sources::get_source(&name)).await }

#[tauri::command]
async fn set_skill_source(name: String, source: SkillSource) -> Result<(), String> { run_blocking(move || sources::set_source(&name, source)).await }

#[tauri::command]
async fn list_repositories() -> Result<Vec<RepositoryRecord>, String> { run_blocking(repositories::list).await }

#[tauri::command]
async fn add_repository(url: String, branch: Option<String>) -> Result<RepositoryRecord, String> { run_blocking(move || repositories::add(&settings::load()?, &url, branch)).await }

#[tauri::command]
async fn remove_repository(id: String) -> Result<(), String> { run_blocking(move || repositories::remove(&id)).await }

#[tauri::command]
async fn scan_repository(id: String) -> Result<Vec<RepositorySkill>, String> { run_blocking(move || repositories::scan(&settings::load()?, &id)).await }

#[tauri::command]
async fn install_repository_skills(id: String, subdirs: Vec<String>) -> Result<OperationResult, String> { run_blocking(move || repositories::install(&settings::load()?, &id, &subdirs)).await }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![get_settings, save_settings, detect_agents, scan_library, import_skill, distribute_skill, consolidate_agents, migrate_canonical, delete_skill, restore_skill, read_skill_file, write_skill_file, git_status, git_update, get_skill_source, set_skill_source, list_repositories, add_repository, remove_repository, scan_repository, install_repository_skills])
        .run(tauri::generate_context!())
        .expect("failed to run Skills Manager");
}

pub fn consolidate_configured_agents(agent_ids: &[String]) -> Result<String, String> {
    Ok(operations::consolidate(&settings::load()?, agent_ids)?.message)
}

pub fn migrate_configured_canonical() -> Result<String, String> {
    let settings = settings::load()?;
    let source_roots = ["codex", "agentbro", "gemini"].into_iter().filter_map(|id| settings.agent_paths.get(id).cloned()).collect::<Vec<_>>();
    Ok(operations::migrate_canonical(&settings, &source_roots)?.message)
}

#[cfg(test)]
mod command_responsiveness_tests {
    #[test]
    fn io_heavy_tauri_commands_use_async_entry_points_and_blocking_workers() {
        let source = include_str!("lib.rs");
        let production = source.split("mod command_responsiveness_tests").next().unwrap_or(source);
        let worker_marker = ["tauri::async_runtime::", "spawn_blocking(operation).await"].concat();
        assert!(production.contains(&worker_marker), "blocking filesystem and Git work must leave the IPC event path");
        for command in ["detect_agents", "scan_library", "import_skill", "consolidate_agents", "git_update", "add_repository", "scan_repository", "install_repository_skills"] {
            assert!(production.contains(&format!("async fn {command}(")), "{command} must be an async Tauri command");
        }
    }
}
