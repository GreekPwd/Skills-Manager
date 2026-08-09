import { useEffect, useMemo, useState } from "react";
import { AlertTriangle, ArchiveRestore, Box, Boxes, ChevronDown, FolderCog, Link2, Plus, Search, Settings, SlidersHorizontal, Sparkles } from "lucide-react";
import { SkillInspector } from "./components/SkillInspector";
import { SkillTable } from "./components/SkillTable";
import { SettingsView } from "./components/SettingsView";
import { demoAgents, demoSkills } from "./lib/demo";
import * as api from "./lib/api";
import type { Skill } from "./lib/types";

type View = "library" | "agents" | "conflicts" | "recycle" | "settings";

const navigation = [
  { id: "library", label: "技能库", icon: Boxes },
  { id: "agents", label: "Agent 连接", icon: Link2 },
  { id: "conflicts", label: "冲突", icon: AlertTriangle, count: 1 },
  { id: "recycle", label: "回收站", icon: ArchiveRestore },
  { id: "settings", label: "设置", icon: Settings },
] as const;

function AgentsView() {
  return <section className="content-view"><header className="page-head"><div><span className="eyebrow">分发目标</span><h1>Agent 连接</h1><p>所有 Agent 读取同一份中央技能库。</p></div><button className="btn"><Plus size={16} />添加自定义 Agent</button></header><div className="agent-list">{demoAgents.map(agent => <div className="agent-row" key={agent.id}><span className="agent-mark" style={{ background: agent.color }}>{agent.name[0]}</span><div><strong>{agent.name}</strong><small>{agent.path}</small></div><span className="connection-ok">已连接</span><span className="muted">{agent.linkedSkills} 个技能</span><button className="icon-btn" title="配置路径"><FolderCog size={17} /></button></div>)}</div></section>;
}

function PlaceholderView({ view }: { view: Exclude<View, "library" | "agents"> }) {
  const content = {
    conflicts: ["冲突", "1 个同名技能等待处理", "对比版本"],
    recycle: ["回收站", "已删除的技能会保留 30 天", "打开回收站"],
    settings: ["设置", "中央仓库与扫描路径", "管理路径"],
  }[view];
  return <section className="content-view"><header className="page-head"><div><span className="eyebrow">Skills Manager</span><h1>{content[0]}</h1><p>{content[1]}</p></div></header><div className="placeholder-panel"><Box size={30} /><h2>{content[0]}</h2><p>{content[1]}</p><button className="btn btn--primary">{content[2]}</button></div></section>;
}

export default function App() {
  const [view, setView] = useState<View>("library");
  const [query, setQuery] = useState("");
  const [skills, setSkills] = useState(demoSkills);
  const [selected, setSelected] = useState<Skill | undefined>(demoSkills[0]);
  const [pendingDelete, setPendingDelete] = useState<Skill>();
  const [notice, setNotice] = useState<string>();
  const filtered = useMemo(() => skills.filter(skill => `${skill.name} ${skill.description}`.toLowerCase().includes(query.toLowerCase())), [query, skills]);
  const displayedSkill = selected && filtered.some(skill => skill.id === selected.id) ? selected : filtered[0];

  const refresh = async () => {
    if (!api.isTauri()) return;
    try { setSkills(await api.scanLibrary()); } catch (error) { setNotice(String(error)); }
  };
  useEffect(() => { void refresh(); }, []);

  const runUpdate = async (skill: Skill) => {
    if (!api.isTauri()) { setNotice("浏览器预览不会修改本机文件。请在桌面应用中执行更新。"); return; }
    try { const result = await api.updateSkill(skill.id); setNotice(result || "已更新到最新版本"); await refresh(); } catch (error) { setNotice(String(error)); }
  };
  const confirmDelete = async () => {
    if (!pendingDelete) return;
    if (!api.isTauri()) { setNotice("浏览器预览不会删除本机文件。"); setPendingDelete(undefined); return; }
    try { await api.deleteSkill(pendingDelete.id); setSelected(undefined); setPendingDelete(undefined); await refresh(); setNotice("技能已移入回收站"); } catch (error) { setNotice(String(error)); }
  };
  const runImport = async () => {
    if (!api.isTauri()) { setNotice("浏览器预览不会导入本机目录。"); return; }
    try { await api.importSkill(); await refresh(); } catch (error) { setNotice(String(error)); }
  };

  return <div className="app-shell">
    <aside className="sidebar">
      <div className="brand"><span><Sparkles size={17} /></span><strong>Skills Manager</strong></div>
      <nav aria-label="主导航">{navigation.map(item => { const Icon = item.icon; return <button type="button" key={item.id} className={view === item.id ? "active" : ""} onClick={() => setView(item.id)}><Icon size={17} /><span>{item.label}</span>{"count" in item && <b>{item.count}</b>}</button>; })}</nav>
      <div className="repo-health"><span className="health-dot" /><div><strong>中央仓库正常</strong><small>4 个 Agent 已连接</small></div><ChevronDown size={14} /></div>
    </aside>
    <main className="workspace">
      {view === "library" && <>
        <section className="library-pane">
          <header className="page-head"><div><span className="eyebrow">中央仓库</span><h1>技能库</h1><p>{skills.length} 个技能 · 已连接 4 / 4</p></div><button className="btn btn--primary" onClick={runImport}><Plus size={16} />导入技能</button></header>
          <div className="toolbar"><label className="search-box"><Search size={16} /><input aria-label="搜索技能" placeholder="搜索技能..." value={query} onChange={event => setQuery(event.target.value)} /></label><button className="btn btn--quiet"><SlidersHorizontal size={15} />筛选</button></div>
          <SkillTable skills={filtered} selectedId={displayedSkill?.id} onSelect={setSelected} />
          <footer className="library-footer"><span>上次扫描：刚刚</span><button onClick={refresh}>重新扫描</button></footer>
        </section>
        <SkillInspector skill={displayedSkill} onUpdate={runUpdate} onDelete={setPendingDelete} />
      </>}
      {view === "agents" && <AgentsView />}
      {view === "settings" && <SettingsView />}
      {view !== "library" && view !== "agents" && view !== "settings" && <PlaceholderView view={view} />}
    </main>
    {notice && <div className="toast" role="status"><span>{notice}</span><button aria-label="关闭通知" onClick={() => setNotice(undefined)}>×</button></div>}
    {pendingDelete && <div className="dialog-backdrop" role="presentation"><section className="dialog" role="dialog" aria-modal="true" aria-labelledby="delete-title"><span className="dialog-icon"><AlertTriangle size={20} /></span><h2 id="delete-title">删除 {pendingDelete.name}？</h2><p>该技能将从 {pendingDelete.agents.length} 个 Agent 断开并移入回收站，可以稍后恢复。</p><div><button className="btn" onClick={() => setPendingDelete(undefined)}>取消</button><button className="btn btn--danger" onClick={confirmDelete}>移入回收站</button></div></section></div>}
  </div>;
}
