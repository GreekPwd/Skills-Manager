use crate::domain::{AgentRecord, AppSettings};
use std::fs;

pub fn detect(settings: &AppSettings) -> Vec<AgentRecord> {
    let names = [("claude", "Claude Code"), ("codex", "Codex"), ("gemini", "Gemini CLI"), ("cursor", "Cursor"), ("agentbro", "AgentBro")];
    names.into_iter().filter_map(|(id, name)| {
        settings.agent_paths.get(id).map(|path| AgentRecord {
            id: id.into(), name: name.into(), path: path.clone(), detected: path.parent().is_some_and(|p| p.exists()),
            linked_skills: fs::read_dir(path).map(|entries| entries.flatten().count()).unwrap_or_default(),
        })
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, fs};

    #[test]
    fn detects_agentbro_as_a_supported_agent() {
        let root = tempfile::tempdir().unwrap();
        let agentbro = root.path().join(".agentbro").join("skills");
        fs::create_dir_all(&agentbro).unwrap();
        let settings = AppSettings {
            library_path: root.path().join(".agents").join("skills"),
            recycle_path: root.path().join("recycle"),
            repository_cache_path: root.path().join("repositories"),
            git_proxy: None,
            agent_paths: BTreeMap::from([("agentbro".into(), agentbro.clone())]),
        };

        let agent = detect(&settings).into_iter().find(|item| item.id == "agentbro").unwrap();

        assert_eq!(agent.name, "AgentBro");
        assert!(agent.detected);
    }
}
