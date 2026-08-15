import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { AgentConnection, AppSettings, RepositorySkill, Skill, SkillRepository, SkillSource } from "./types";

export const isTauri = () => "__TAURI_INTERNALS__" in window;

export async function scanLibrary(): Promise<Skill[]> { return invoke("scan_library"); }
export async function getSettings(): Promise<AppSettings> { return invoke("get_settings"); }
export async function saveSettings(value: AppSettings): Promise<void> { await invoke("save_settings", { value }); }
export async function detectAgents(): Promise<AgentConnection[]> { return invoke("detect_agents"); }
export async function deleteSkill(name: string): Promise<void> { await invoke("delete_skill", { name }); }
export async function updateSkill(name: string): Promise<string> { return invoke("git_update", { name }); }
export async function getSkillSource(name: string): Promise<SkillSource | null> { return invoke("get_skill_source", { name }); }
export async function setSkillSource(name: string, source: SkillSource): Promise<void> { await invoke("set_skill_source", { name, source }); }
export async function listRepositories(): Promise<SkillRepository[]> { return invoke("list_repositories"); }
export async function addRepository(url: string, branch?: string): Promise<SkillRepository> { return invoke("add_repository", { url, branch }); }
export async function removeRepository(id: string): Promise<void> { await invoke("remove_repository", { id }); }
export async function scanRepository(id: string): Promise<RepositorySkill[]> { return invoke("scan_repository", { id }); }
export async function installRepositorySkills(id: string, subdirs: string[]): Promise<void> { await invoke("install_repository_skills", { id, subdirs }); }
export async function distributeSkill(name: string, agentIds: string[]): Promise<void> { await invoke("distribute_skill", { name, agentIds }); }
export async function consolidateAgents(agentIds: string[]): Promise<void> { await invoke("consolidate_agents", { agentIds }); }
export async function readSkillFile(name: string, relative: string): Promise<string> { return invoke("read_skill_file", { name, relative }); }
export async function writeSkillFile(name: string, relative: string, content: string): Promise<void> { await invoke("write_skill_file", { name, relative, content }); }
export async function importSkill(): Promise<void> {
  const source = await open({ directory: true, multiple: false, title: "选择包含 SKILL.md 的目录" });
  if (!source) return;
  const name = source.replace(/[\\/]+$/, "").split(/[\\/]/).pop();
  if (!name) throw new Error("无法从所选目录确定技能名称");
  await invoke("import_skill", { source, name });
}
