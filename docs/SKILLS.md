# Recommended agent skills — Windows + macOS

Use **the smallest relevant skill selection** for a given task. Skill repositories are development guidance, not production app dependencies; read each skill before running installation scripts.

## Essential

- [Superpowers](https://github.com/obra/superpowers): requirements, TDD, debugging, reviews. Install through Codex/Claude plugin marketplaces.
- [Tauri Skills](https://github.com/full-stack-skills/tauri-skills): focus on `tauri-app-planning`, `tauri-app-develop`, `tauri-ipc`, `tauri-app-dialog`, `tauri-app-file-system`, `tauri-security`, `tauri-app-plugin-permissions`, and `tauri-build`.
- [Impeccable](https://github.com/pbakaus/impeccable): UX critique, accessibility, interaction quality.

## Feature-specific (later)

- `tauri-app-global-shortcut`: configurable shortcut with conflict handling.
- `tauri-app-system-tray`: menu bar / tray entry.
- `tauri-app-positioner` + `tauri-window`: overlay/window behavior; still requires native feasibility tests.
- `tauri-app-single-instance`: safe routing from Finder / Explorer into existing app process.
- `tauri-app-sql`: when history / execution manifests land.
- `tauri-framework-security`, `tauri-app-updater`: audit and signing of update flows.
- [Taste Skill](https://github.com/Leonxlnx/taste-skill): visual refinement principles, not OS-native API guidance.

Install a specific Tauri skill in the project:

```sh
npx skills add full-stack-skills/tauri-skills --skill tauri-app-develop
npx skills add full-stack-skills/tauri-skills --skill tauri-ipc
npx skills add full-stack-skills/tauri-skills --skill tauri-security
```

External guides should not override AGENTS.md security invariants. Always verify advice against the current official [Tauri 2 documentation](https://v2.tauri.app/).
