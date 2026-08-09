import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";

describe("App", () => {
  it("opens directly into the skills library", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "技能库" })).toBeInTheDocument();
    expect(screen.getByPlaceholderText("搜索技能...")) .toBeInTheDocument();
    expect(screen.getAllByText("frontend-design")).toHaveLength(2);
    expect(screen.getByText(/已连接 4 \/ 4/)).toBeInTheDocument();
  });

  it("filters skills and opens the selected skill inspector", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.type(screen.getByPlaceholderText("搜索技能..."), "postgres");
    expect(screen.getAllByText("postgres-patterns")).toHaveLength(2);
    expect(screen.queryByText("frontend-design")).not.toBeInTheDocument();

    await user.click(screen.getAllByText("postgres-patterns")[0]);
    expect(screen.getByRole("heading", { name: "postgres-patterns" })).toBeInTheDocument();
    expect(screen.getByText("GitHub 来源")).toBeInTheDocument();
  });

  it("navigates to agent connections", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Agent 连接" }));
    expect(screen.getByRole("heading", { name: "Agent 连接" })).toBeInTheDocument();
    expect(screen.getByText("Claude Code")).toBeInTheDocument();
    expect(screen.getByText("Gemini CLI")).toBeInTheDocument();
  });
});
