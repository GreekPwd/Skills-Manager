import { useEffect, useMemo, useState } from "react";
import { ArrowLeft, Download, GitBranch, Plus, RefreshCw, Trash2 } from "lucide-react";
import * as api from "../lib/api";
import type { RepositorySkill, SkillRepository } from "../lib/types";

interface Props {
  onBack: () => void;
  onInstalled: () => Promise<void>;
  onNotice: (message: string) => void;
}

export function RepositoryView({ onBack, onInstalled, onNotice }: Props) {
  const [repositories, setRepositories] = useState<SkillRepository[]>([]);
  const [url, setUrl] = useState("");
  const [branch, setBranch] = useState("main");
  const [busy, setBusy] = useState(false);
  const [selectedRepository, setSelectedRepository] = useState<SkillRepository>();
  const [repositorySkills, setRepositorySkills] = useState<RepositorySkill[]>([]);
  const [selectedSubdirs, setSelectedSubdirs] = useState<Set<string>>(new Set());
  const selectableSubdirs = useMemo(() => repositorySkills.map(skill => skill.subdir), [repositorySkills]);
  const allSelected = selectableSubdirs.length > 0 && selectableSubdirs.every(value => selectedSubdirs.has(value));

  const loadRepositories = async () => {
    try { setRepositories(await api.listRepositories()); }
    catch (error) { onNotice(`读取仓库失败：${String(error)}`); }
  };
  useEffect(() => { void loadRepositories(); }, []);

  const addRepository = async () => {
    if (!url.trim()) return;
    setBusy(true);
    try {
      const repository = await api.addRepository(url.trim(), branch.trim() || undefined);
      setUrl("");
      await loadRepositories();
      onNotice(`已添加 ${repository.name}，识别到 ${repository.skillCount} 个 Skill`);
    } catch (error) { onNotice(`添加仓库失败：${String(error)}`); }
    finally { setBusy(false); }
  };

  const openRepository = async (repository: SkillRepository) => {
    setSelectedRepository(repository);
    setBusy(true);
    try {
      const skills = await api.scanRepository(repository.id);
      setRepositorySkills(skills);
      setSelectedSubdirs(new Set());
      await loadRepositories();
    } catch (error) { onNotice(`扫描仓库失败：${String(error)}`); }
    finally { setBusy(false); }
  };

  const removeRepository = async (repository: SkillRepository) => {
    setBusy(true);
    try {
      await api.removeRepository(repository.id);
      if (selectedRepository?.id === repository.id) { setSelectedRepository(undefined); setRepositorySkills([]); }
      await loadRepositories();
      onNotice(`已移除仓库 ${repository.name}`);
    } catch (error) { onNotice(`移除仓库失败：${String(error)}`); }
    finally { setBusy(false); }
  };

  const toggleSkill = (subdir: string) => setSelectedSubdirs(current => {
    const next = new Set(current);
    if (next.has(subdir)) next.delete(subdir); else next.add(subdir);
    return next;
  });

  const toggleAll = () => setSelectedSubdirs(allSelected ? new Set() : new Set(selectableSubdirs));

  const installSelected = async () => {
    if (!selectedRepository || selectedSubdirs.size === 0) return;
    const installCount = selectedSubdirs.size;
    setBusy(true);
    try {
      await api.installRepositorySkills(selectedRepository.id, [...selectedSubdirs]);
      await onInstalled();
      await openRepository(selectedRepository);
      onNotice(`已将 ${installCount} 个 Skill 安装到 .agents\\skills`);
    } catch (error) { onNotice(`安装失败：${String(error)}`); }
    finally { setBusy(false); }
  };

  return <section className="content-view repository-view">
    <header className="repository-title"><button className="back-button" title="返回技能库" onClick={onBack}><ArrowLeft size={18} /></button><div><span className="eyebrow">GitHub Sources</span><h1>管理技能仓库</h1><p>从官方仓库选择 Skills，统一安装到 C:\Users\admin\.agents\skills。</p></div></header>
    <div className="repository-scroll">
      <section className="repository-add-card">
        <h2>添加技能仓库</h2>
        <label>仓库 URL<input aria-label="仓库 URL" placeholder="owner/name 或 https://github.com/owner/name" value={url} onChange={event => setUrl(event.target.value)} /></label>
        <label>分支<input aria-label="仓库分支" placeholder="main" value={branch} onChange={event => setBranch(event.target.value)} /></label>
        <button className="repo-add-button" disabled={busy || !url.trim()} onClick={addRepository}><Plus size={17} />{busy ? "正在同步…" : "添加仓库"}</button>
      </section>

      <section className="repository-list-section">
        <h2>已添加的仓库</h2>
        <div className="repository-cards">
          {repositories.map(repository => <button key={repository.id} className={`repository-card ${selectedRepository?.id === repository.id ? "is-selected" : ""}`} onClick={() => void openRepository(repository)}><GitBranch size={18} /><div><strong>{repository.name}</strong><span>分支: {repository.branch || "默认"}<b>识别到 {repository.skillCount} 个技能</b></span></div><span className="repo-chevron">›</span></button>)}
          {!repositories.length && <div className="repository-empty">尚未添加仓库</div>}
        </div>
      </section>

      {selectedRepository && <section className="repository-skills-section">
        <header><div><h2>{selectedRepository.name}</h2><p>{repositorySkills.length} 个可安装 Skill</p></div><div><button className="btn" disabled={busy} onClick={() => void openRepository(selectedRepository)}><RefreshCw size={15} />刷新</button><button className="btn repo-remove" disabled={busy} onClick={() => void removeRepository(selectedRepository)}><Trash2 size={15} />移除仓库</button></div></header>
        <div className="repo-selection-bar"><label><input type="checkbox" aria-label="全选仓库 Skills" checked={allSelected} onChange={toggleAll} />全选</label><span>已选择 {selectedSubdirs.size} / {repositorySkills.length}</span><button className="btn btn--primary" disabled={busy || selectedSubdirs.size === 0} onClick={installSelected}><Download size={15} />安装所选</button></div>
        <div className="repository-skill-list">{repositorySkills.map(skill => <label className="repository-skill-row" key={skill.subdir}><input type="checkbox" checked={selectedSubdirs.has(skill.subdir)} onChange={() => toggleSkill(skill.subdir)} /><div><strong>{skill.name}</strong><small>{skill.description}</small><code>{skill.subdir || "."}</code></div><span className={skill.installed ? "repo-installed" : "repo-available"}>{skill.installed ? "已安装，将更新" : "可安装"}</span></label>)}</div>
      </section>}
    </div>
  </section>;
}
