export type AgentId = "claude" | "codex" | "gemini" | "cursor";
export type SkillStatus = "healthy" | "update" | "conflict" | "local";

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
}

export interface AgentConnection {
  id: AgentId;
  name: string;
  path: string;
  detected: boolean;
  linkedSkills: number;
  color: string;
}

export interface AppSettings {
  libraryPath: string;
  recyclePath: string;
  agentPaths: Record<string, string>;
}
