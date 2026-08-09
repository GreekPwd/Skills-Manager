# Skills Manager

Windows-first desktop manager for keeping one canonical local Skills library shared by Claude Code, Codex, Gemini CLI, and Cursor.

## Development

```powershell
npm install
npm run dev
```

Run the browser UI at `http://127.0.0.1:1423`. In a Tauri-enabled environment:

```powershell
npm run tauri dev
```

The app stores one real copy of each Skill in the configured central directory. Agent directories are connected with Windows Junctions where possible. Imports and deletes are handled by Rust commands; deletes move Skills to the configured recycle directory.

## Verification

```powershell
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

The current development machine has Node.js but does not have Rust/Cargo installed, so the frontend checks can run locally while Tauri compilation and NSIS/MSI packaging require the Rust toolchain and WebView2 prerequisites.
