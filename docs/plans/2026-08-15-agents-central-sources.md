# Central Agents Skills And Sources Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `%USERPROFILE%\.agents\skills` the only active local Skills repository, link every supported Agent directory to it, and let Skills Manager update each registered Skill from its official GitHub source.

**Architecture:** Rust owns a recoverable canonical-root migration, whole-root Windows Junction creation, source metadata, and GitHub clone/replace transactions. The source registry lives in the application configuration directory so third-party Skill content is not modified with bookkeeping files. React exposes source registration and update actions, while Skills without a registered source remain explicitly local/manual.

**Tech Stack:** Tauri 2, Rust, React 19, TypeScript, Vitest, Testing Library, Git CLI, Windows Junctions.

---

### Task 1: Canonical Root And Agent Adapter Model

**Files:**
- Modify: `src-tauri/src/settings.rs`
- Modify: `src-tauri/src/agents.rs`
- Modify: `src-tauri/src/domain.rs`
- Modify: `src/lib/types.ts`
- Test: `src-tauri/src/settings.rs`, `src-tauri/src/agents.rs`

1. Add failing tests asserting the default canonical root is `.agents\\skills`, `.codex\\skills` remains an Agent root, `.agentbro\\skills` is discovered, and legacy `.codex\\skills` is selected only as a migration fallback.
2. Run targeted Rust tests and verify the new assertions fail against the current `.codex` default.
3. Implement the new default and `agentbro` adapter without removing Claude, Codex, Gemini, or Cursor.
4. Extend serialized source fields with optional URL, subdirectory, and branch metadata.
5. Run Rust tests.

### Task 2: Recoverable Canonical Migration

**Files:**
- Modify: `src-tauri/src/operations.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`
- Test: `src-tauri/src/operations.rs`

1. Add a failing temporary-directory test that merges `.agents`, `.agentbro`, and `.codex`, preserves canonical-name conflicts in backups, imports unique valid Skills, and creates roots linked to the new central directory.
2. Run the targeted test and verify it fails before implementation.
3. Implement a validated `migrate_canonical` transaction with timestamped backups, hash verification, rollback, and whole-root Junctions for all configured Agent roots.
4. Expose a Tauri command and a CLI flag for the real-machine migration.
5. Run the full Rust test suite.

### Task 3: GitHub Source Registry And Update Transactions

**Files:**
- Modify: `src-tauri/src/domain.rs`
- Modify: `src-tauri/src/sources.rs`
- Modify: `src-tauri/src/scanner.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/api.ts`
- Test: `src-tauri/src/sources.rs`, `src-tauri/src/scanner.rs`

1. Add failing tests for GitHub URL validation, source metadata round trips, dirty/local replacement protection, subdirectory sources, and atomic update rollback.
2. Run targeted tests and verify failures.
3. Implement the app-level source registry and update flow: clone the registered GitHub repository into a temporary directory, locate the configured Skill subdirectory, verify `SKILL.md`, stage the replacement, and atomically swap it into the central root.
4. Keep the existing independent-repository update path, but return an actionable “未登记官方来源” result for local Skills.
5. Expose source read/write and update commands through typed frontend API functions.
6. Run Rust tests.

### Task 4: Desktop Source And Update Workflow

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/components/SkillInspector.tsx`
- Modify: `src/lib/types.ts`
- Modify: `src/styles.css`
- Test: `src/App.test.tsx`, `src/App.desktop.test.tsx`

1. Add failing interaction tests for registering a GitHub source, showing the source URL, updating a registered Skill, and disabling automatic update for unregistered local Skills.
2. Run frontend tests and verify failures.
3. Implement the source dialog and update confirmation/progress/error states.
4. Make the inspector display the registered official URL and link it through the safe backend command.
5. Run frontend tests and the production TypeScript build.

### Task 5: Real Machine Migration And Packaging Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/plans/2026-08-15-agents-central-sources.md`

1. Verify the exact real-machine roots and stop Skills Manager before migration.
2. Run the tested migration CLI, preserving timestamped backups under the Skills Manager backup directory.
3. Verify `.agents\\skills` is physical canonical storage and `.codex\\skills`, `.agentbro\\skills`, `.claude\\skills`, `.cursor\\skills`, and `.gemini\\skills` resolve to it.
4. Run `npm test`, `npm run build`, Rust tests, and the formal Tauri production build.
5. Reopen the packaged executable and verify the window loads without a development-server dependency.

