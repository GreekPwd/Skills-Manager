# Skills Manager

Windows-first desktop manager for keeping one canonical local Skills library shared by Claude Code, Codex, Gemini CLI, Cursor, and AgentBro.

## Development

```powershell
npm install
npm run dev
```

Run the browser UI at `http://127.0.0.1:1423`. In a Tauri-enabled environment:

```powershell
npm run tauri dev
```

The default canonical directory is `%USERPROFILE%\.agents\skills`. Product-specific directories such as `.codex\skills`, `.claude\skills`, `.cursor\skills`, `.gemini\skills`, and `.agentbro\skills` are connected to it with Windows Junctions. Migration first backs up physical legacy directories and imports unique Skills; same-name conflicts keep the canonical copy.

Each Skill can register an official HTTPS GitHub repository URL, optional branch, and optional repository subdirectory. Updates clone into a temporary directory, require a valid `SKILL.md`, back up the installed version, and then replace it. Source metadata is stored in `%APPDATA%\skills-manager\sources.json` rather than inside the shared Skills tree.

To run the recoverable one-time canonical migration from a release binary:

```powershell
.\skills-manager.exe --migrate-canonical
```

## Verification

```powershell
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Desktop development additionally requires the Rust toolchain, Microsoft C++ build tools, and WebView2. Install Rust with the official Rustup installer before running the Tauri commands.
