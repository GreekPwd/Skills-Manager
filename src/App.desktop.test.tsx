import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, it, vi } from "vitest";
import App from "./App";
import type { Skill } from "./lib/types";

const apiMocks = vi.hoisted(() => ({
  scanLibrary: vi.fn(),
  detectAgents: vi.fn(),
  consolidateAgents: vi.fn(),
  readSkillFile: vi.fn(),
  writeSkillFile: vi.fn(),
  distributeSkill: vi.fn(),
  getSkillSource: vi.fn(),
  setSkillSource: vi.fn(),
  listRepositories: vi.fn(),
  addRepository: vi.fn(),
  removeRepository: vi.fn(),
  scanRepository: vi.fn(),
  installRepositorySkills: vi.fn(),
}));

vi.mock("./lib/api", () => ({
  isTauri: () => true,
  scanLibrary: apiMocks.scanLibrary,
  detectAgents: apiMocks.detectAgents,
  consolidateAgents: apiMocks.consolidateAgents,
  readSkillFile: apiMocks.readSkillFile,
  writeSkillFile: apiMocks.writeSkillFile,
  distributeSkill: apiMocks.distributeSkill,
  getSkillSource: apiMocks.getSkillSource,
  setSkillSource: apiMocks.setSkillSource,
  listRepositories: apiMocks.listRepositories,
  addRepository: apiMocks.addRepository,
  removeRepository: apiMocks.removeRepository,
  scanRepository: apiMocks.scanRepository,
  installRepositorySkills: apiMocks.installRepositorySkills,
  updateSkill: vi.fn(),
  deleteSkill: vi.fn(),
  importSkill: vi.fn(),
}));

const loadedSkill: Skill = {
  id: "loaded-skill",
  name: "loaded-skill",
  description: "来自真实目录",
  path: "C:\\Users\\admin\\.codex\\skills\\loaded-skill",
  status: "local",
  source: "local",
  sourceLabel: "本地创建",
  updatedAt: "本机",
  files: 1,
  agents: ["codex"],
};

beforeEach(() => {
  apiMocks.scanLibrary.mockReset();
  apiMocks.detectAgents.mockReset();
  apiMocks.consolidateAgents.mockReset();
  apiMocks.readSkillFile.mockReset();
  apiMocks.writeSkillFile.mockReset();
  apiMocks.distributeSkill.mockReset();
  apiMocks.getSkillSource.mockReset();
  apiMocks.setSkillSource.mockReset();
  apiMocks.listRepositories.mockReset();
  apiMocks.addRepository.mockReset();
  apiMocks.removeRepository.mockReset();
  apiMocks.scanRepository.mockReset();
  apiMocks.installRepositorySkills.mockReset();
  apiMocks.detectAgents.mockResolvedValue([]);
  apiMocks.consolidateAgents.mockResolvedValue(undefined);
  apiMocks.writeSkillFile.mockResolvedValue(undefined);
  apiMocks.distributeSkill.mockResolvedValue(undefined);
  apiMocks.getSkillSource.mockResolvedValue(null);
  apiMocks.setSkillSource.mockResolvedValue(undefined);
  apiMocks.listRepositories.mockResolvedValue([]);
  apiMocks.removeRepository.mockResolvedValue(undefined);
  apiMocks.installRepositorySkills.mockResolvedValue(undefined);
});

it("shows a real loading state instead of demo skills in the desktop app", async () => {
  let finishScan: (skills: Skill[]) => void = () => undefined;
  apiMocks.scanLibrary.mockReturnValue(new Promise<Skill[]>((resolve) => { finishScan = resolve; }));

  render(<App />);

  expect(screen.getByText("正在加载技能库…")).toBeInTheDocument();
  expect(screen.queryByText("frontend-design")).not.toBeInTheDocument();

  finishScan([loadedSkill]);
  expect(await screen.findAllByText("loaded-skill")).toHaveLength(2);
});

it("consolidates Claude and Cursor after confirmation", async () => {
  const user = userEvent.setup();
  apiMocks.scanLibrary.mockResolvedValue([loadedSkill]);
  apiMocks.detectAgents.mockResolvedValue([
    { id: "claude", name: "Claude Code", path: "C:\\Users\\admin\\.claude\\skills", detected: true, linkedSkills: 1 },
    { id: "cursor", name: "Cursor", path: "C:\\Users\\admin\\.cursor\\skills", detected: false, linkedSkills: 0 },
  ]);
  render(<App />);

  await user.click(screen.getByRole("button", { name: "Agent 连接" }));
  await user.click(await screen.findByRole("button", { name: "统一到中央仓库" }));
  await user.click(screen.getByRole("button", { name: "确认统一" }));

  expect(apiMocks.consolidateAgents).toHaveBeenCalledWith(["claude", "cursor"]);
});

it("edits and saves SKILL.md", async () => {
  const user = userEvent.setup();
  apiMocks.scanLibrary.mockResolvedValue([loadedSkill]);
  apiMocks.readSkillFile.mockResolvedValue("# Original");
  render(<App />);
  await screen.findAllByText("loaded-skill");

  await user.click(screen.getByRole("button", { name: "编辑技能" }));
  const editor = await screen.findByLabelText("SKILL.md 内容");
  await user.clear(editor);
  await user.type(editor, "# Updated");
  await user.click(screen.getByRole("button", { name: "保存修改" }));

  expect(apiMocks.writeSkillFile).toHaveBeenCalledWith("loaded-skill", "SKILL.md", "# Updated");
});

it("registers an official GitHub source for a skill", async () => {
  const user = userEvent.setup();
  apiMocks.scanLibrary.mockResolvedValue([loadedSkill]);
  render(<App />);
  await screen.findAllByText("loaded-skill");

  await user.click(document.querySelector(".source-line") as HTMLButtonElement);
  await user.type(screen.getByLabelText(/GitHub/), "https://github.com/openai/skills");
  await user.type(screen.getByLabelText(/Skill/), "skills/example");
  const dialog = screen.getByRole("dialog");
  await user.click(dialog.querySelector(".btn--primary") as HTMLButtonElement);

  expect(apiMocks.setSkillSource).toHaveBeenCalledWith("loaded-skill", {
    url: "https://github.com/openai/skills",
    subdir: "skills/example",
    branch: undefined,
  });
});

it("selects all skills from a repository and installs them", async () => {
  const user = userEvent.setup();
  apiMocks.scanLibrary.mockResolvedValue([loadedSkill]);
  apiMocks.listRepositories.mockResolvedValue([{ id: "anthropics--skills", name: "anthropics/skills", url: "https://github.com/anthropics/skills.git", branch: "main", skillCount: 2 }]);
  apiMocks.scanRepository.mockResolvedValue([
    { name: "one", description: "One", subdir: "skills/one", installed: false },
    { name: "two", description: "Two", subdir: "skills/two", installed: true },
  ]);
  render(<App />);

  await user.click(screen.getByRole("button", { name: "技能仓库" }));
  await user.click(await screen.findByRole("button", { name: /anthropics\/skills/ }));
  await user.click(await screen.findByLabelText("全选仓库 Skills"));
  await user.click(screen.getByRole("button", { name: "安装所选" }));

  expect(apiMocks.installRepositorySkills).toHaveBeenCalledWith("anthropics--skills", ["skills/one", "skills/two"]);
});
