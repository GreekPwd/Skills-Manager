export type AgentId = "claude" | "codex" | "gemini" | "cursor" | "agentbro";
export type SkillStatus = "healthy" | "update" | "conflict" | "local" | "invalid";

export interface Skill {
  id: string;
  name: string;
  description: string;
  path: string;
  status: SkillStatus;
  source: "git" | "local";
  sourceLabel: string;
  updatedAt: string;
  files: number;
  agents: AgentId[];
  version?: string;
  sourceUrl?: string;
  sourceSubdir?: string;
  sourceBranch?: string;
}

export interface SkillSource {
  url: string;
  subdir: string;
  branch?: string;
}

export interface SkillRepository {
  id: string;
  name: string;
  url: string;
  branch?: string;
  skillCount: number;
}

export interface RepositorySkill {
  name: string;
  description: string;
  subdir: string;
  installed: boolean;
}

export interface AgentConnection {
  id: AgentId;
  name: string;
  path: string;
  detected: boolean;
  linkedSkills: number;
  color?: string;
}

export interface AppSettings {
  libraryPath: string;
  recyclePath: string;
  repositoryCachePath: string;
  gitProxy?: string;
  agentPaths: Record<string, string>;
}
