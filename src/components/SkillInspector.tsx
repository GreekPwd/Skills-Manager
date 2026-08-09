import { ExternalLink, FileText, FolderOpen, GitBranch, MoreHorizontal, Pencil, RefreshCw, Trash2, X } from "lucide-react";
import type { Skill } from "../lib/types";
import { StatusBadge } from "./StatusBadge";

interface Props { skill?: Skill; onClose?: () => void; onUpdate?: (skill: Skill) => void; onDelete?: (skill: Skill) => void; }

export function SkillInspector({ skill, onClose, onUpdate, onDelete }: Props) {
  if (!skill) return <aside className="inspector inspector--empty"><FileText size={26} /><p>选择一个技能查看详情</p></aside>;
  return (
    <aside className="inspector">
      <header className="inspector-head">
        <div><span className="eyebrow">技能详情</span><h2>{skill.name}</h2></div>
        <div className="inspector-actions"><button title="更多操作"><MoreHorizontal size={18} /></button>{onClose && <button title="关闭详情" onClick={onClose}><X size={18} /></button>}</div>
      </header>
      <p className="inspector-description">{skill.description}</p>
      <StatusBadge status={skill.status} />
      <div className="primary-actions">
        <button className="btn btn--primary" onClick={() => onUpdate?.(skill)}><RefreshCw size={15} />检查更新</button>
        <button className="icon-btn" title="编辑技能"><Pencil size={16} /></button>
        <button className="icon-btn danger" title="删除技能" onClick={() => onDelete?.(skill)}><Trash2 size={16} /></button>
      </div>
      <section className="detail-section">
        <h3>来源</h3>
        <div className="source-line"><span className="source-icon"><GitBranch size={16} /></span><div><strong>{skill.source === "git" ? "GitHub 来源" : "本地技能"}</strong><small>{skill.sourceLabel}</small></div><ExternalLink size={14} /></div>
        {skill.version && <dl className="meta-grid"><div><dt>版本</dt><dd>v{skill.version}</dd></div><div><dt>文件</dt><dd>{skill.files} 个</dd></div></dl>}
      </section>
      <section className="detail-section">
        <h3>Agent 分发</h3>
        <div className="distribution-list">
          {[["C", "Claude Code", "claude"], ["X", "Codex", "codex"], ["G", "Gemini CLI", "gemini"], ["R", "Cursor", "cursor"]].map(([letter, name, id]) => (
            <div key={id}><i className={skill.agents.includes(id as never) ? "active" : ""}>{letter}</i><span>{name}</span><b>{skill.agents.includes(id as never) ? "已连接" : "未连接"}</b></div>
          ))}
        </div>
      </section>
      <button className="path-line" title={skill.path}><FolderOpen size={15} /><span>{skill.path}</span></button>
    </aside>
  );
}
