import { useEffect, useState } from "react";
import { Check, FolderCog, HardDrive, Save } from "lucide-react";
import { getSettings, isTauri, saveSettings } from "../lib/api";
import type { AppSettings } from "../lib/types";

const demoSettings: AppSettings = {
  libraryPath: "C:\\Users\\admin\\.skills-manager\\skills",
  recyclePath: "C:\\Users\\admin\\.skills-manager\\recycle",
  agentPaths: { claude: "~/.claude/skills", codex: "~/.codex/skills", gemini: "~/.gemini/skills", cursor: "~/.cursor/skills" },
};

export function SettingsView() {
  const [settings, setSettings] = useState<AppSettings>(demoSettings);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string>();
  useEffect(() => { if (isTauri()) void getSettings().then(setSettings).catch((value) => setError(String(value))); }, []);
  const update = (key: keyof AppSettings, value: string) => setSettings(current => ({ ...current, [key]: value }));
  const submit = async () => {
    setError(undefined);
    try { if (isTauri()) await saveSettings(settings); setSaved(true); window.setTimeout(() => setSaved(false), 2200); } catch (value) { setError(String(value)); }
  };
  return <section className="content-view settings-view"><header className="page-head"><div><span className="eyebrow">Workspace</span><h1>设置</h1><p>配置中央仓库和 Agent 扫描路径。</p></div><button className="btn btn--primary" onClick={submit}>{saved ? <Check size={16} /> : <Save size={16} />}{saved ? "已保存" : "保存设置"}</button></header><div className="settings-form">
    <section className="settings-section"><div className="settings-title"><span className="settings-icon"><HardDrive size={17} /></span><div><h2>中央仓库</h2><p>所有 Agent 共享的唯一 Skill 副本。</p></div></div><label>Skill 存储路径<input value={settings.libraryPath} onChange={event => update("libraryPath", event.target.value)} /></label><label>回收站路径<input value={settings.recyclePath} onChange={event => update("recyclePath", event.target.value)} /></label></section>
    <section className="settings-section"><div className="settings-title"><span className="settings-icon"><FolderCog size={17} /></span><div><h2>Agent 路径</h2><p>每个 Agent 目录会指向中央仓库中的 Skill。</p></div></div>{[["claude", "Claude Code"], ["codex", "Codex"], ["gemini", "Gemini CLI"], ["cursor", "Cursor"]].map(([id, name]) => <label key={id}>{name}<input value={settings.agentPaths[id] ?? ""} onChange={event => setSettings(current => ({ ...current, agentPaths: { ...current.agentPaths, [id]: event.target.value } }))} /></label>)}</section>
    {error && <p className="settings-error">{error}</p>}
  </div></section>;
}
