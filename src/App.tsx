import { useEffect, useMemo, useState } from "react";
import { AlertTriangle, ArchiveRestore, Box, Boxes, ChevronDown, FolderCog, GitFork, Link2, Plus, Search, Settings, SlidersHorizontal, Sparkles } from "lucide-react";
import { SkillInspector } from "./components/SkillInspector";
import { SkillTable } from "./components/SkillTable";
import { SettingsView } from "./components/SettingsView";
import { RepositoryView } from "./components/RepositoryView";
import { demoAgents, demoSkills } from "./lib/demo";
import * as api from "./lib/api";
import type { AgentConnection, Skill, SkillSource, SkillStatus } from "./lib/types";

type View = "library" | "repositories" | "agents" | "conflicts" | "recycle" | "settings";

const navigation = [
  { id: "repositories", label: "技能仓库", icon: GitFork },
  { id: "library", label: "技能库", icon: Boxes },
  { id: "agents", label: "Agent 连接", icon: Link2 },
  { id: "conflicts", label: "冲突", icon: AlertTriangle },
  { id: "recycle", label: "回收站", icon: ArchiveRestore },
  { id: "settings", label: "设置", icon: Settings },
] as const;

const agentColors: Record<string, string> = { claude: "#d97748", codex: "#171b19", gemini: "#4285f4", cursor: "#7657d5", agentbro: "#2f855a" };

interface AgentsViewProps {
  agents: AgentConnection[];
  consolidating: boolean;
  onConfigure: () => void;
  onConsolidate: () => void;
}

function AgentsView({ agents, consolidating, onConfigure, onConsolidate }: AgentsViewProps) {
  return <section className="content-view"><header className="page-head"><div><span className="eyebrow">分发目标</span><h1>Agent 连接</h1><p>所有 Agent 读取同一份中央技能库。</p></div><div className="page-actions"><button className="btn" onClick={onConfigure}><FolderCog size={16} />配置 Agent</button><button className="btn btn--primary" disabled={consolidating} onClick={onConsolidate}><Link2 size={16} />{consolidating ? "正在统一…" : "统一到中央仓库"}</button></div></header><div className="agent-list">{agents.map(agent => <div className="agent-row" key={agent.id}><span className="agent-mark" style={{ background: agent.color ?? agentColors[agent.id] }}>{agent.name[0]}</span><div><strong>{agent.name}</strong><small>{agent.path}</small></div><span className={agent.detected ? "connection-ok" : "muted"}>{agent.detected ? "已连接" : "未检测到"}</span><span className="muted">{agent.linkedSkills} 个技能</span><button className="icon-btn" title={`配置 ${agent.name} 路径`} onClick={onConfigure}><FolderCog size={17} /></button></div>)}</div></section>;
}

function PlaceholderView({ view, conflicts }: { view: "conflicts" | "recycle"; conflicts: number }) {
  const content = {
    conflicts: ["冲突", conflicts ? `${conflicts} 个技能需要检查` : "当前没有待处理冲突"],
    recycle: ["回收站", "已删除的技能会保留在本机回收目录"],
  }[view];
  return <section className="content-view"><header className="page-head"><div><span className="eyebrow">Skills Manager</span><h1>{content[0]}</h1><p>{content[1]}</p></div></header><div className="placeholder-panel"><Box size={30} /><h2>{content[0]}</h2><p>{content[1]}</p></div></section>;
}

export default function App() {
  const desktop = api.isTauri();
  const [view, setView] = useState<View>("library");
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState<"all" | SkillStatus>("all");
  const [showFilters, setShowFilters] = useState(false);
  const [skills, setSkills] = useState<Skill[]>(desktop ? [] : demoSkills);
  const [agents, setAgents] = useState<AgentConnection[]>(desktop ? [] : demoAgents);
  const [loading, setLoading] = useState(desktop);
  const [selected, setSelected] = useState<Skill | undefined>(desktop ? undefined : demoSkills[0]);
  const [pendingDelete, setPendingDelete] = useState<Skill>();
  const [confirmConsolidation, setConfirmConsolidation] = useState(false);
  const [consolidating, setConsolidating] = useState(false);
  const [editing, setEditing] = useState<{ skill: Skill; content: string }>();
  const [savingEdit, setSavingEdit] = useState(false);
  const [sourceEditor, setSourceEditor] = useState<{ skill: Skill; source: SkillSource }>();
  const [savingSource, setSavingSource] = useState(false);
  const [notice, setNotice] = useState<string>();
  const filtered = useMemo(() => skills.filter(skill => `${skill.name} ${skill.description}`.toLowerCase().includes(query.toLowerCase()) && (statusFilter === "all" || skill.status === statusFilter)), [query, skills, statusFilter]);
  const displayedSkill = selected && filtered.some(skill => skill.id === selected.id) ? selected : filtered[0];
  const detectedAgents = agents.filter(agent => agent.detected).length;
  const conflictCount = skills.filter(skill => skill.status === "conflict" || skill.status === "invalid").length;

  const refresh = async () => {
    if (!desktop) return;
    setLoading(true);
    try {
      const [nextSkills, nextAgents] = await Promise.all([api.scanLibrary(), api.detectAgents()]);
      setSkills(nextSkills);
      setAgents(nextAgents);
    } catch (error) {
      setNotice(`加载失败：${String(error)}`);
    } finally {
      setLoading(false);
    }
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
  const confirmAgentConsolidation = async () => {
    if (!desktop) { setNotice("浏览器预览不会修改 Agent 目录。"); setConfirmConsolidation(false); return; }
    setConsolidating(true);
    try {
      await api.consolidateAgents(agents.map(agent => agent.id));
      setConfirmConsolidation(false);
      await refresh();
      setNotice("Claude Code 与 Cursor 已统一到 Codex 中央仓库");
    } catch (error) {
      setNotice(`统一失败：${String(error)}`);
    } finally {
      setConsolidating(false);
    }
  };
  const openEditor = async (skill: Skill) => {
    if (!desktop) { setNotice("浏览器预览不会编辑本机文件。"); return; }
    try { setEditing({ skill, content: await api.readSkillFile(skill.id, "SKILL.md") }); }
    catch (error) { setNotice(`读取失败：${String(error)}`); }
  };
  const saveEditor = async () => {
    if (!editing) return;
    setSavingEdit(true);
    try {
      await api.writeSkillFile(editing.skill.id, "SKILL.md", editing.content);
      setEditing(undefined);
      await refresh();
      setNotice("SKILL.md 已保存");
    } catch (error) { setNotice(`保存失败：${String(error)}`); }
    finally { setSavingEdit(false); }
  };
  const openSourceEditor = async (skill: Skill) => {
    if (!desktop) { setNotice("浏览器预览不会修改来源配置。"); return; }
    try {
      const source = await api.getSkillSource(skill.id);
      setSourceEditor({ skill, source: source ?? { url: "", subdir: "", branch: undefined } });
    } catch (error) { setNotice(`读取来源失败：${String(error)}`); }
  };
  const saveSource = async () => {
    if (!sourceEditor) return;
    setSavingSource(true);
    try {
      await api.setSkillSource(sourceEditor.skill.id, { ...sourceEditor.source, branch: sourceEditor.source.branch?.trim() || undefined });
      setSourceEditor(undefined);
      await refresh();
      setNotice("官方 GitHub 来源已登记，现在可以更新该 Skill。");
    } catch (error) { setNotice(`保存来源失败：${String(error)}`); }
    finally { setSavingSource(false); }
  };
  const runDistribute = async (skill: Skill) => {
    if (!desktop) { setNotice("浏览器预览不会修改 Agent 目录。"); return; }
    try {
      await api.distributeSkill(skill.id, agents.filter(agent => agent.id !== "codex").map(agent => agent.id));
      await refresh();
      setNotice(`${skill.name} 已分发到全部 Agent`);
    } catch (error) { setNotice(`分发失败：${String(error)}`); }
  };
  const copyPath = async (skill: Skill) => {
    try {
      if (!navigator.clipboard) throw new Error("clipboard unavailable");
      await navigator.clipboard.writeText(skill.path);
      setNotice("技能路径已复制");
    } catch { setNotice(`技能路径：${skill.path}`); }
  };

  return <div className="app-shell">
    <aside className="sidebar">
      <div className="brand"><span><Sparkles size={17} /></span><strong>Skills Manager</strong></div>
      <nav aria-label="主导航">{navigation.map(item => { const Icon = item.icon; return <button type="button" key={item.id} className={view === item.id ? "active" : ""} onClick={() => setView(item.id)}><Icon size={17} /><span>{item.label}</span>{item.id === "conflicts" && conflictCount > 0 && <b>{conflictCount}</b>}</button>; })}</nav>
      <div className="repo-health"><span className="health-dot" /><div><strong>{loading ? "正在扫描中央仓库" : "中央仓库正常"}</strong><small>{detectedAgents} 个 Agent 已连接</small></div><ChevronDown size={14} /></div>
    </aside>
    <main className="workspace">
      {view === "library" && <>
        <section className="library-pane">
          <header className="page-head"><div><span className="eyebrow">中央仓库</span><h1>技能库</h1><p>{skills.length} 个技能 · 已连接 {detectedAgents} / {agents.length}</p></div><div className="page-actions"><button className="btn" onClick={() => setView("repositories")}><GitFork size={16} />管理仓库</button><button className="btn btn--primary" onClick={runImport}><Plus size={16} />导入技能</button></div></header>
          <div className="toolbar"><label className="search-box"><Search size={16} /><input aria-label="搜索技能" placeholder="搜索技能..." value={query} onChange={event => setQuery(event.target.value)} /></label><button className="btn btn--quiet" aria-pressed={showFilters} onClick={() => setShowFilters(value => !value)}><SlidersHorizontal size={15} />筛选</button>{showFilters && <select aria-label="按状态筛选" value={statusFilter} onChange={event => setStatusFilter(event.target.value as "all" | SkillStatus)}><option value="all">全部状态</option><option value="healthy">正常</option><option value="update">可更新</option><option value="conflict">冲突</option><option value="local">本地</option><option value="invalid">无效</option></select>}</div>
          <SkillTable skills={filtered} loading={loading} selectedId={displayedSkill?.id} onSelect={setSelected} />
          <footer className="library-footer"><span>上次扫描：刚刚</span><button onClick={refresh}>重新扫描</button></footer>
        </section>
        <SkillInspector skill={displayedSkill} onUpdate={runUpdate} onDelete={setPendingDelete} onEdit={openEditor} onDistribute={runDistribute} onCopyPath={copyPath} onConfigureSource={openSourceEditor} />
      </>}
      {view === "agents" && <AgentsView agents={agents} consolidating={consolidating} onConfigure={() => setView("settings")} onConsolidate={() => setConfirmConsolidation(true)} />}
      {view === "repositories" && <RepositoryView onBack={() => setView("library")} onInstalled={refresh} onNotice={setNotice} />}
      {view === "settings" && <SettingsView />}
      {(view === "conflicts" || view === "recycle") && <PlaceholderView view={view} conflicts={conflictCount} />}
    </main>
    {notice && <div className="toast" role="status"><span>{notice}</span><button aria-label="关闭通知" onClick={() => setNotice(undefined)}>×</button></div>}
    {pendingDelete && <div className="dialog-backdrop" role="presentation"><section className="dialog" role="dialog" aria-modal="true" aria-labelledby="delete-title"><span className="dialog-icon"><AlertTriangle size={20} /></span><h2 id="delete-title">删除 {pendingDelete.name}？</h2><p>该技能将从 {pendingDelete.agents.length} 个 Agent 断开并移入回收站，可以稍后恢复。</p><div><button className="btn" onClick={() => setPendingDelete(undefined)}>取消</button><button className="btn btn--danger" onClick={confirmDelete}>移入回收站</button></div></section></div>}
    {confirmConsolidation && <div className="dialog-backdrop" role="presentation"><section className="dialog" role="dialog" aria-modal="true" aria-labelledby="consolidate-title"><span className="dialog-icon dialog-icon--safe"><Link2 size={20} /></span><h2 id="consolidate-title">统一 Claude 与 Cursor？</h2><p>现有 skills 目录会先备份，再链接到 Codex 中央仓库。独有技能会校验后导入；同名版本以中央仓库为准，原版本仍保留在备份中。</p><div><button className="btn" disabled={consolidating} onClick={() => setConfirmConsolidation(false)}>取消</button><button className="btn btn--primary" disabled={consolidating} onClick={confirmAgentConsolidation}>{consolidating ? "正在统一…" : "确认统一"}</button></div></section></div>}
    {editing && <div className="dialog-backdrop" role="presentation"><section className="dialog editor-dialog" role="dialog" aria-modal="true" aria-labelledby="editor-title"><h2 id="editor-title">编辑 {editing.skill.name}</h2><p>直接编辑中央仓库中的 SKILL.md。</p><textarea aria-label="SKILL.md 内容" spellCheck={false} value={editing.content} onChange={event => setEditing({ ...editing, content: event.target.value })} /><div><button className="btn" disabled={savingEdit} onClick={() => setEditing(undefined)}>取消</button><button className="btn btn--primary" disabled={savingEdit} onClick={saveEditor}>{savingEdit ? "正在保存…" : "保存修改"}</button></div></section></div>}
    {sourceEditor && <div className="dialog-backdrop" role="presentation"><section className="dialog source-dialog" role="dialog" aria-modal="true" aria-labelledby="source-title"><h2 id="source-title">配置 {sourceEditor.skill.name} 的官方来源</h2><p>登记对应的官方 GitHub 开源仓库。单仓库包含多个 Skills 时填写仓库内子目录。</p><label>GitHub 仓库 URL<input aria-label="GitHub 仓库 URL" placeholder="https://github.com/owner/repository" value={sourceEditor.source.url} onChange={event => setSourceEditor({ ...sourceEditor, source: { ...sourceEditor.source, url: event.target.value } })} /></label><label>Skill 子目录（可选）<input aria-label="Skill 子目录" placeholder="skills/example" value={sourceEditor.source.subdir} onChange={event => setSourceEditor({ ...sourceEditor, source: { ...sourceEditor.source, subdir: event.target.value } })} /></label><label>分支（可选）<input aria-label="分支" placeholder="main" value={sourceEditor.source.branch ?? ""} onChange={event => setSourceEditor({ ...sourceEditor, source: { ...sourceEditor.source, branch: event.target.value } })} /></label><div><button className="btn" disabled={savingSource} onClick={() => setSourceEditor(undefined)}>取消</button><button className="btn btn--primary" disabled={savingSource || !sourceEditor.source.url.trim()} onClick={saveSource}>{savingSource ? "正在保存…" : "保存来源"}</button></div></section></div>}
  </div>;
}
