mod agents;
mod domain;
mod metadata;
mod operations;
mod scanner;
mod settings;
mod sources;

use domain::{AgentRecord, AppSettings, OperationResult, SkillRecord};

#[tauri::command]
fn get_settings() -> Result<AppSettings, String> { settings::load() }

#[tauri::command]
fn save_settings(value: AppSettings) -> Result<(), String> { settings::save(&value) }

#[tauri::command]
fn detect_agents() -> Result<Vec<AgentRecord>, String> { Ok(agents::detect(&settings::load()?)) }

#[tauri::command]
fn scan_library() -> Result<Vec<SkillRecord>, String> { scanner::scan(&settings::load()?) }

#[tauri::command]
fn import_skill(source: String, name: String) -> Result<OperationResult, String> { operations::import(&settings::load()?, std::path::Path::new(&source), &name) }

#[tauri::command]
fn distribute_skill(name: String, agent_ids: Vec<String>) -> Result<OperationResult, String> { operations::distribute(&settings::load()?, &name, &agent_ids) }

#[tauri::command]
fn delete_skill(name: String) -> Result<OperationResult, String> { operations::delete(&settings::load()?, &name) }

#[tauri::command]
fn restore_skill(recycle_name: String) -> Result<OperationResult, String> { operations::restore(&settings::load()?, &recycle_name) }

#[tauri::command]
fn read_skill_file(name: String, relative: String) -> Result<String, String> { sources::read_text(&settings::load()?, &name, &relative) }

#[tauri::command]
fn write_skill_file(name: String, relative: String, content: String) -> Result<(), String> { sources::write_text(&settings::load()?, &name, &relative, &content) }

#[tauri::command]
fn git_status(name: String) -> Result<String, String> { sources::git_status(&settings::load()?, &name) }

#[tauri::command]
fn git_update(name: String) -> Result<String, String> { sources::git_update(&settings::load()?, &name) }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![get_settings, save_settings, detect_agents, scan_library, import_skill, distribute_skill, delete_skill, restore_skill, read_skill_file, write_skill_file, git_status, git_update])
        .run(tauri::generate_context!())
        .expect("failed to run Skills Manager");
}
