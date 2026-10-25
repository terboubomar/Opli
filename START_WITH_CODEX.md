# First task for Codex or Claude Code

Read **`CODEX_BUILD_SPEC.md`**, `README.md`, `AGENTS.md`, and all files in `docs/`. The build spec includes Codex prompts, skill installation, architecture, milestones, and exact acceptance gates.

**Milestone 0:** get the included Tauri 2 + React + Rust starter building and running on Windows and macOS. The existing app has a native file picker, in-window file drop support, a three-recipe UI, Rust planning and safe copying, and Rust tests; none have been independently native-built in the authoring environment.

1. Install Node, Rust and Tauri platform prerequisites. Run `npm install`, `npm run build`, then `cargo test` under `src-tauri`.
2. Fix dependency/compiler errors without weakening permissions, source protection, or fallback behavior.
3. Run `npm run tauri dev` on Windows 11 and macOS; log all manual and automated results.
4. Verify selecting files, drag/drop, output directory selection, preview, execution, duplicate filenames, permissions, paths, symlink tests, source checksums, disk-full and cancellation behavior. Add reproducible tests where absent.
5. Build a CI matrix that passes on both platforms; generate and commit a lock file.
6. Only then prototype Finder Quick Actions and Explorer integration as platform adapters, without claiming global drag interception.

Follow the local project `AGENTS.md` and use applicable Superpowers + Tauri skills. Record deficiencies accurately in `docs/ROADMAP.md` and do not claim tested functionality without running it.
