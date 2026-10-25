# Intent Layer — cross-platform starter

**Working title.** A local-first, open-source *action layer* for desktop files: choose an outcome, preview every output, and run repeatable recipes on **Windows and macOS**.

## Implemented in source (native behavior awaits platform smoke tests)

- A React + TypeScript desktop UI hosted in Tauri 2, with native file picker and window file-drop handling.
- Choose files, a recipe, and an output folder. Preview all output paths before execution. Execution consumes a server-held, single-use approved plan ID and rejects changed inputs or occupied destinations.
- Three **real, copy-only** Rust workflows: **Organize by type**, **Rename cleanly**, **Collect for handoff**.
- Duplicate-name avoidance, exclusive temporary output creation and hard-link publication, source-file symlink rejection, and Rust regression tests. Twelve Rust tests pass on this Windows host outside its hard-link-restricted sandbox.
- No account, remote AI, upload, network connection for processing, or arbitrary recipe scripts.

**Not yet implemented:** image conversion / optimization, PDF tools, radial mouse menu, global file-drag interception, Finder / Explorer integration, system tray, undo log, plugins, AI assistance, workflow marketplace, automatic updates, signing/notarization. Do not advertise these as built.

## Requirements

- Node.js LTS and npm
- Rust stable toolchain (recent enough for Tauri 2's dialog plugin; currently Rust 1.90+)
- Tauri OS prerequisites: Windows C++ Build Tools + WebView2, or macOS Xcode Command Line Tools. See https://v2.tauri.app/start/prerequisites/

## Run

```bash
npm ci
npm run tauri dev
```

## Test

```bash
npm run build
npm run test
cd src-tauri && cargo test
```

Requires platform development prerequisites. On 25 October 2026 the frontend build, 12 Rust tests, Clippy, native Windows binary build, and a limited Windows desktop smoke test passed. The macOS build/UI and hosted CI matrix have not run. See `docs/M0_BASELINE.md`.

## Main behavior

1. Drag files onto the app window, or choose **browse files**.
2. Select one of the three recipes.
3. Pick an existing output folder.
4. Click **Preview workflow** to inspect the planned destination files.
5. Click **Run recipe** to create copies in a dedicated `Intent - ...` subdirectory.

## Cross-platform roadmap

- **Phase 0 — prove core:** Run GUI on macOS and Windows, test actual drag events + file picker, run Rust test suite and manual E2E.
- **Phase 1 — dependable native entry:** macOS Finder Quick Action in Swift; Windows Explorer entry via vetted menu mechanism; keyboard shortcuts, menubar/tray, installers.
- **Phase 2 — richer recipes:** media and documents with isolated, licensed conversion tooling; cancellable jobs; progress; verified undo manifest.
- **Phase 3 — radial UI:** pointer-anchored overlay prototype on both OSes, only after native feasibility testing. Don't rely on undocumented OS interception.
- **Phase 4 — community:** versioned declarative recipe schema, explicit privileges, signed releases, extension model, contribution governance.

Read **`CODEX_BUILD_SPEC.md`** for the complete Codex build specification, phase gates, skills and setup. Also read `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, `docs/RECIPE_SPEC.md`, and `AGENTS.md` before implementing extensions.

## Legal & safety

This project is independent of Tangerine for Mac, avoids its trademarks/assets/source, and reimplements only publicly observable interaction ideas. Dependency licenses must be reviewed before bundling codecs or other engines. Code is MIT licensed. See `SECURITY.md`.

## Development caveats

The starter was authored without a native build. A Windows native binary now builds and a limited two-file workflow passes; macOS remains unverified. Production releases require code-signing and macOS notarization.

