# Agent instructions for Codex and Claude Code

## Mission

Build **Intent Layer**, a safe, fast and delightful open-source local-first file-outcome tool for both Windows and macOS. No copied Tangerine assets. Zero unapproved destructive operations. Prioritize platform reliability over feature count.

## Must-read

- `CODEX_BUILD_SPEC.md` — master Codex build specification and skills playbook
- `README.md` — actual shipped scope vs aspirations
- `docs/ARCHITECTURE.md` — platform split, safety invariants
- `docs/ROADMAP.md` — staged acceptance gates
- `docs/RECIPE_SPEC.md` — future spec (do not treat as implemented)

## Practices

1. Before adding a capability, inspect existing implementation and write tests first.
2. Never advertise features as working unless verified on both platforms or labeled platform-specific.
3. Favor explicit native APIs. Do not silently request Accessibility/Screen Recording/Input Monitoring entitlements.
4. Keep the main deterministic Rust engine platform-agnostic; OS hooks in their own adapter.
5. Do not use general-purpose command execution for untrusted recipes.
6. Protect source files: exclusive output creation and non-destructive defaults.
7. Every output-sensitive feature must include preview, error reporting, and collision tests.
8. Keep app completely useful offline. No mandatory AI, login, or cloud.
9. Before merge: frontend typecheck, Rust fmt/clippy/tests, Windows and macOS smoke tests.
10. When a test cannot run, explicitly list missing environment and never claim it passed.

## Suggested skills

- https://github.com/obra/superpowers — spec discipline, TDD, debugging and reviews
- https://github.com/full-stack-skills/tauri-skills — Tauri v2 platform tooling
- https://github.com/pbakaus/impeccable — interface critique/accessibility/polish
- https://github.com/Leonxlnx/taste-skill — visual/motion inspiration; adapt carefully to desktop

## First Codex task

"Read AGENTS.md, README.md and all docs. Install dependencies. Run TypeScript build and Rust tests, fix platform setup errors without weakening security. Verify the three copy-only recipes with real temporary files. Do not begin native Finder/Explorer integration until the universal app dropzone works and a Windows/macOS test matrix exists."
