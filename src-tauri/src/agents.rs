use crate::domain::{AgentRecord, AppSettings};
use std::fs;

pub fn detect(settings: &AppSettings) -> Vec<AgentRecord> {
    let names = [("claude", "Claude Code"), ("codex", "Codex"), ("gemini", "Gemini CLI"), ("cursor", "Cursor")];
    names.into_iter().filter_map(|(id, name)| {
        settings.agent_paths.get(id).map(|path| AgentRecord {
            id: id.into(), name: name.into(), path: path.clone(), detected: path.parent().is_some_and(|p| p.exists()),
            linked_skills: fs::read_dir(path).map(|entries| entries.flatten().count()).unwrap_or_default(),
        })
    }).collect()
}
