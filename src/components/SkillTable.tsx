import { ChevronRight, Github } from "lucide-react";
import type { Skill } from "../lib/types";
import { StatusBadge } from "./StatusBadge";

interface Props {
  skills: Skill[];
  loading?: boolean;
  selectedId?: string;
  onSelect: (skill: Skill) => void;
}

export function SkillTable({ skills, loading = false, selectedId, onSelect }: Props) {
  if (loading) {
    return <div className="empty-list" role="status"><strong>正在加载技能库…</strong><span>正在扫描本机 Skill 目录</span></div>;
  }
  if (!skills.length) {
    return <div className="empty-list"><strong>没有匹配的技能</strong><span>调整搜索或筛选条件</span></div>;
  }
  return (
    <div className="skill-table" role="list">
      <div className="table-header" aria-hidden="true">
        <span>技能</span><span>状态</span><span>连接</span><span>更新于</span><span />
      </div>
      {skills.map((skill) => (
        <button
          type="button"
          role="listitem"
          className={`skill-row ${selectedId === skill.id ? "is-selected" : ""}`}
          key={skill.id}
          onClick={() => onSelect(skill)}
        >
          <span className="skill-name-cell">
            <span className="skill-icon"><Github size={16} /></span>
            <span><strong>{skill.name}</strong><small>{skill.description}</small></span>
          </span>
          <StatusBadge status={skill.status} />
          <span className="agent-dots" aria-label={`已连接 ${skill.agents.length} / 4`}>
            {["C", "X", "G", "R"].map((letter, index) => <i className={index < skill.agents.length ? "active" : ""} key={letter}>{letter}</i>)}
          </span>
          <span className="muted">{skill.updatedAt}</span>
          <ChevronRight className="row-chevron" size={16} />
        </button>
      ))}
    </div>
  );
}
