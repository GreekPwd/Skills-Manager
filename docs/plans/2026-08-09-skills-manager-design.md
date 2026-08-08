# Skills Manager Design

## Goal

Build a Windows-first desktop application that keeps one canonical local Skills library and exposes it to Claude Code, Codex, Gemini CLI, and Cursor. Users can inspect, import, edit, update, delete, restore, and redistribute Skills without maintaining duplicate copies.

## Architecture

The application uses Tauri 2 with a React and TypeScript frontend. Rust owns filesystem access, Git operations, path discovery, content hashing, backups, and directory-link transactions. The frontend renders state returned by typed Tauri commands and never mutates managed directories directly.

The canonical library defaults to `%USERPROFILE%\.skills-manager\skills` and can be changed. Application settings and operation history live in the platform application-data directory rather than inside the library.

Each Skill is a directory containing `SKILL.md`. YAML frontmatter supplies the display name and description. The application stores source metadata separately so it does not rewrite third-party Skill content solely for bookkeeping.

## Agent Adapters

Built-in adapters cover Claude Code, Codex, Gemini CLI, and Cursor. Each adapter defines candidate user-level Skills directories and supports an explicit path override. Custom adapters can be added later through the same data model.

Managed Agent entries are directory junctions on Windows. Junctions are preferred because they normally do not require Developer Mode or administrator privileges. The application can report copy-only fallback as an explicit user choice but never silently creates duplicate managed state.

## Discovery And Adoption

The first-run wizard selects the library, scans known Agent directories, groups identical Skills by normalized name and content hash, and creates a conflict report. Identical copies merge automatically. Conflicting copies require a selected winner or a rename so both can be retained.

Before adoption, the application shows an operation preview. Execution first creates a recoverable backup, copies the selected canonical data, verifies hashes, then replaces Agent copies with links. A failed step rolls back changes made by that operation.

Changing the canonical path follows copy, verify, relink, and retain-old-directory ordering. The old directory is not deleted automatically.

## Skill Operations

The library view supports search, source and status filters, multi-select, file inspection, and Agent distribution status.

Local editing works through a text editor with a file diff before save. Git-backed Skills retain repository URL, branch, and subdirectory metadata. Update checks fetch remote state, show changed files, and require confirmation before replacing canonical content.

Delete moves a Skill into an application-managed recycle area and removes its Agent links. Restore recreates the Skill and its selected links. Permanent deletion is a separate confirmed action.

## Interface

The main window is a dense desktop workbench. A dark navigation rail contains Library, Agents, Conflicts, Recycle Bin, and Settings. A central table lists Skills. A right inspector shows metadata, files, source state, and Agent connections. On narrow windows the inspector becomes a drawer.

The visual system uses a neutral light workspace, near-black navigation, and restrained teal status accents. It opens directly into the usable library rather than a landing page. First-run, conflict resolution, update review, and library migration use focused multi-step dialogs.

## Failure Handling

All destructive filesystem commands operate on resolved, validated paths under configured roots. Backups and operation journals make adoption and relinking recoverable. Broken links, missing targets, malformed frontmatter, dirty Git sources, permission failures, and hash mismatches are surfaced as actionable diagnostics.

## Testing

Rust unit tests cover metadata parsing, hashing, path validation, discovery grouping, conflict classification, adapter paths, and operation planning. Integration tests use temporary directories for import, link, delete, restore, and rollback behavior. Frontend tests cover filtering, selection, inspector state, conflict decisions, and first-run flow. A production build plus Playwright screenshots at desktop and compact widths verifies layout, interaction, and overflow.

## Scope

The first release is local-only. It does not include a public marketplace, cloud sync, Agent session monitoring, MCP management, plugins, or automatic publication. Windows packaging targets NSIS first, with MSI left available through Tauri configuration.
