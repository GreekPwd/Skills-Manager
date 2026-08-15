import { useEffect, useState } from "react";
import { Check, FolderCog, FolderOpen, GitBranch, HardDrive, Save } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { getSettings, isTauri, saveSettings } from "../lib/api";
import type { AppSettings } from "../lib/types";

const demoSettings: AppSettings = {
  libraryPath: "C:\\Users\\admin\\.agents\\skills",
  recyclePath: "C:\\Users\\admin\\.skills-manager\\recycle",
  repositoryCachePath: "C:\\Users\\admin\\.skills-manager\\repositories",
  gitProxy: "",
  agentPaths: { claude: "~/.claude/skills", codex: "~/.codex/skills", gemini: "~/.gemini/skills", cursor: "~/.cursor/skills" },
};

export function SettingsView() {
  const [settings, setSettings] = useState<AppSettings>(demoSettings);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string>();
  useEffect(() => { if (isTauri()) void getSettings().then(setSettings).catch((value) => setError(String(value))); }, []);
  const update = (key: keyof AppSettings, value: string) => setSettings(current => ({ ...current, [key]: value }));
  const chooseCachePath = async () => {
    try {
      const selected = await open({ directory: true, multiple: false, title: "选择 GitHub 仓库缓存目录", defaultPath: settings.repositoryCachePath });
      if (selected) update("repositoryCachePath", selected);
    } catch (value) { setError(String(value)); }
  };
  const submit = async () => {
    setError(undefined);
    try { if (isTauri()) await saveSettings(settings); setSaved(true); window.setTimeout(() => setSaved(false), 2200); } catch (value) { setError(String(value)); }
  };
  return <section className="content-view settings-view"><header className="page-head"><div><span className="eyebrow">Workspace</span><h1>设置</h1><p>配置中央仓库和 Agent 扫描路径。</p></div><button className="btn btn--primary" onClick={submit}>{saved ? <Check size={16} /> : <Save size={16} />}{saved ? "已保存" : "保存设置"}</button></header><div className="settings-form">
    <section className="settings-section"><div className="settings-title"><span className="settings-icon"><HardDrive size={17} /></span><div><h2>中央仓库</h2><p>所有 Agent 共享的唯一 Skill 副本。</p></div></div><label>Skill 存储路径（固定）<input value={settings.libraryPath} readOnly disabled /></label><label>回收站路径<input value={settings.recyclePath} onChange={event => update("recyclePath", event.target.value)} /></label></section>
    <section className="settings-section"><div className="settings-title"><span className="settings-icon"><GitBranch size={17} /></span><div><h2>GitHub 仓库</h2><p>配置仓库克隆缓存以及本程序 Git 命令使用的代理。</p></div></div><label>仓库缓存路径<div className="settings-path-row"><input value={settings.repositoryCachePath} onChange={event => update("repositoryCachePath", event.target.value)} /><button type="button" className="btn" onClick={chooseCachePath}><FolderOpen size={15} />选择目录</button></div></label><label>Git 代理（可选）<input aria-label="Git 代理" placeholder="http://127.0.0.1:7890 或 socks5h://127.0.0.1:1080" value={settings.gitProxy ?? ""} onChange={event => update("gitProxy", event.target.value)} /><small>留空时自动读取 Windows 系统代理；手工填写可覆盖系统代理，不会修改全局 Git 配置。</small></label></section>
    <section className="settings-section"><div className="settings-title"><span className="settings-icon"><FolderCog size={17} /></span><div><h2>Agent 路径</h2><p>每个 Agent 的 skills 目录均链接到中央仓库。</p></div></div>{[["claude", "Claude Code"], ["codex", "Codex"], ["gemini", "Gemini CLI"], ["cursor", "Cursor"], ["agentbro", "AgentBro"]].map(([id, name]) => <label key={id}>{name}<input value={settings.agentPaths[id] ?? ""} onChange={event => setSettings(current => ({ ...current, agentPaths: { ...current.agentPaths, [id]: event.target.value } }))} /></label>)}</section>
    {error && <p className="settings-error">{error}</p>}
  </div></section>;
}
