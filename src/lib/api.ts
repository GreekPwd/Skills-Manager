import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { AgentConnection, AppSettings, Skill } from "./types";

export const isTauri = () => "__TAURI_INTERNALS__" in window;

export async function scanLibrary(): Promise<Skill[]> { return invoke("scan_library"); }
export async function getSettings(): Promise<AppSettings> { return invoke("get_settings"); }
export async function saveSettings(value: AppSettings): Promise<void> { await invoke("save_settings", { value }); }
export async function detectAgents(): Promise<AgentConnection[]> { return invoke("detect_agents"); }
export async function deleteSkill(name: string): Promise<void> { await invoke("delete_skill", { name }); }
export async function updateSkill(name: string): Promise<string> { return invoke("git_update", { name }); }
export async function distributeSkill(name: string, agentIds: string[]): Promise<void> { await invoke("distribute_skill", { name, agentIds }); }
export async function importSkill(): Promise<void> {
  const source = await open({ directory: true, multiple: false, title: "选择包含 SKILL.md 的目录" });
  if (!source) return;
  const name = source.replace(/[\\/]+$/, "").split(/[\\/]/).pop();
  if (!name) throw new Error("无法从所选目录确定技能名称");
  await invoke("import_skill", { source, name });
}
