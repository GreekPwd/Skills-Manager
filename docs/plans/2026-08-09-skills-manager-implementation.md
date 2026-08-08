# Skills Manager Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Windows-first desktop application that manages one canonical Skills library shared by Claude Code, Codex, Gemini CLI, and Cursor.

**Architecture:** Tauri 2 exposes narrow Rust commands for discovery, parsing, settings, Git source inspection, and filesystem transactions. React consumes typed command results and presents a three-pane management workbench plus first-run, conflict, update, and deletion flows.

**Tech Stack:** Tauri 2, Rust, React 19, TypeScript, Vite, Vitest, Testing Library, Lucide React, Playwright

---

### Task 1: Project Foundation

**Files:**
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`
- Create: `src/main.tsx`, `src/App.tsx`, `src/styles.css`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

1. Scaffold the Vite React and Tauri application with test and packaging scripts.
2. Add a shell-render test and run it to verify the initial failure.
3. Implement the application shell and Tauri bootstrap.
4. Run frontend tests and `cargo check`.
5. Commit the foundation.

### Task 2: Domain Model And Metadata Parsing

**Files:**
- Create: `src-tauri/src/domain.rs`
- Create: `src-tauri/src/metadata.rs`

1. Add Rust tests for valid frontmatter, missing frontmatter, malformed metadata, and display-name fallback.
2. Run targeted tests and confirm failures.
3. Implement serializable Skill, source, Agent, diagnostic, and conflict models plus safe frontmatter parsing.
4. Run Rust tests.
5. Commit the domain layer.

### Task 3: Settings And Agent Discovery

**Files:**
- Create: `src-tauri/src/settings.rs`
- Create: `src-tauri/src/agents.rs`
- Modify: `src-tauri/src/lib.rs`

1. Add tests for default paths, environment expansion, explicit overrides, and settings round trips in temporary directories.
2. Run tests and confirm failures.
3. Implement platform application-data settings and built-in adapters for Claude Code, Codex, Gemini CLI, and Cursor.
4. Expose `get_settings`, `save_settings`, and `detect_agents` commands.
5. Run Rust tests and commit.

### Task 4: Library Scan And Conflict Classification

**Files:**
- Create: `src-tauri/src/scanner.rs`
- Modify: `src-tauri/src/lib.rs`

1. Add tests for content hashing, invalid Skills, identical duplicates, conflicting names, and linked directories.
2. Run tests and confirm failures.
3. Implement deterministic recursive hashing and scan/classification services.
4. Expose `scan_library` and `scan_candidates` commands.
5. Run tests and commit.

### Task 5: Safe Filesystem Transactions

**Files:**
- Create: `src-tauri/src/operations.rs`
- Modify: `src-tauri/src/lib.rs`

1. Add temporary-directory tests for import, backup, junction planning, delete-to-recycle, restore, path containment, and rollback journals.
2. Run tests and confirm failures.
3. Implement operation previews and execution with validated roots, backups, hash checks, Windows junction creation, and rollback.
4. Expose preview, adopt, delete, restore, distribute, and migrate commands.
5. Run tests and commit.

### Task 6: Git And Local Editing

**Files:**
- Create: `src-tauri/src/sources.rs`
- Modify: `src-tauri/src/lib.rs`

1. Add tests around Git URL parsing, clean/dirty status, update availability, subdirectory sources, path validation, and atomic text saves.
2. Run tests and confirm failures.
3. Implement Git-backed update checking through command execution, change previews, confirmed pulls, file reads, and atomic local writes.
4. Expose source and editor commands.
5. Run tests and commit.

### Task 7: Frontend Data Layer And Workbench

**Files:**
- Create: `src/lib/api.ts`, `src/lib/types.ts`, `src/lib/format.ts`
- Create: `src/components/AppShell.tsx`, `SkillTable.tsx`, `SkillInspector.tsx`, `StatusBadge.tsx`, `EmptyState.tsx`
- Modify: `src/App.tsx`, `src/styles.css`
- Test: `src/App.test.tsx`, `src/components/SkillTable.test.tsx`

1. Add tests for loading, filtering, selection, empty, diagnostic, and responsive inspector states.
2. Run tests and confirm failures.
3. Implement typed Tauri calls and the dense three-pane library interface using Lucide icons and accessible controls.
4. Run frontend tests and commit.

### Task 8: Operational Workflows

**Files:**
- Create: `src/components/SetupWizard.tsx`, `ConflictResolver.tsx`, `UpdateDialog.tsx`, `DeleteDialog.tsx`, `AgentConnections.tsx`, `SettingsView.tsx`, `RecycleBin.tsx`
- Modify: `src/App.tsx`, `src/styles.css`
- Test: corresponding `*.test.tsx` files

1. Add interaction tests for first-run adoption, explicit conflict selection, update preview, delete impact, restore, path migration, and adapter overrides.
2. Run tests and confirm failures.
3. Implement each workflow with progress, error, success, and disabled states.
4. Run frontend tests and commit.

### Task 9: Packaging And Verification

**Files:**
- Create: `.gitignore`, `README.md`, `playwright.config.ts`, `e2e/workbench.spec.ts`
- Modify: `src-tauri/tauri.conf.json`, `package.json`

1. Document prerequisites, development, test, and Windows packaging commands.
2. Configure NSIS and MSI bundle metadata and application icons.
3. Run `npm test`, `npm run build`, `cargo test`, and `cargo check`.
4. Start the Vite server and capture Playwright screenshots at desktop and compact widths; verify no overlap or overflow.
5. Run a Tauri packaging smoke check where the installed toolchain permits it.
6. Commit verified packaging and documentation.
