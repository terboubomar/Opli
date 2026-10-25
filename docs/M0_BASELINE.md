# Milestone 0 baseline — 25 October 2026

**Completion status: OPEN.** Windows x64 compilation, the automated safety suite, and a limited real desktop workflow pass. The required macOS build/UI verification and hosted Windows/macOS CI matrix have not run. Windows drag/drop, larger GUI batches, low-disk and removable/network destinations, and adversarial directory swaps remain unverified. Do not claim cross-platform compatibility or start M1 yet.

## Scope and environment

The supplied ZIP was extracted into this repository. The source contains a Tauri 2 shell, a React/Vite frontend, three copy-only Rust recipes, a native file picker, and window drop handling. These are implementation facts, not native behavior claims.

Local host: Windows build 26100 / 24H2, x64 (the legacy registry `ProductName` reports Windows 10 Pro; CIM access was unavailable). Node 25.4.0; npm 11.7.0; project-local stable Rust/Cargo 1.99.1 targeting `x86_64-pc-windows-msvc`, Clippy 0.1.99 and rustfmt 1.10.0. Visual Studio Community 2022 17.14.36518.9 now includes MSVC 14.44.35207 C++ tools and Windows SDK 10.0.26100.0. `link.exe`, `kernel32.lib`, `rc.exe`, and `mt.exe` exist. WebView2 Runtime registry version is 155.0.4283.62. No macOS host is available in this workspace.

### Installer error 5007 diagnosis and resolution

The previous `dd_installer_20261025164057.log` shows an unquoted `--installPath` parsed as `C:\Program`, `vs.willow.iselevated : False`, and the explicit message “Commands with --quiet or --passive should be run elevated from the beginning.” An earlier attempt also used unsupported `--wait`. A corrected quoted and initially elevated `modify` command installed `Microsoft.VisualStudio.Component.VC.Tools.x86.x64` and `Microsoft.VisualStudio.Component.Windows11SDK.26100`, exiting 0. `vswhere -requires` finds both components and `dd_setup_20261025165621_099_Win11SDK_10.0.26100.log` reports child exit 0. The failed installer command was not repeated. See [Tauri's prerequisites](https://v2.tauri.app/start/prerequisites/) and [Microsoft's WebView2 runtime detection guidance](https://learn.microsoft.com/microsoft-edge/webview2/concepts/distribution).

## Commands run

| Command | Result |
| --- | --- |
| `npm install --no-audit --no-fund` | PASS; 74 packages installed; `package-lock.json` created. |
| `npm ci --no-audit --no-fund` | PASS; lockfile-aware reinstall of 151 packages after adding Vitest. |
| `npm run build` | PASS; TypeScript and Vite production build after the final error-message fix. |
| `npm run test` | PASS; 5 Vitest UI contract tests. `TEMP` and `TMP` were set to `.test-temp` because the sandbox system temp rejected a rename. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS after formatting the final Rust test. |
| `cargo metadata --manifest-path src-tauri/Cargo.toml --no-deps --format-version 1` | PASS; manifest parses. |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked` | PASS outside the filesystem sandbox: 12 Rust tests, 0 failures. A sandboxed run had 6 failures because it blocks hard-link creation; a one-file probe and the full suite passed outside it. |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings` | PASS outside the filesystem sandbox after the 12-test suite. |
| `npm run tauri -- build --debug --no-bundle --ci` | PASS; native debug executable. |
| `npm run tauri -- build --no-bundle --ci` | PASS; optimized Windows executable (no installer bundle or signing). |
| `npm run tauri -- build --bundles nsis --ci` | PASS; unsigned Windows NSIS installer rebuilt after the final UI fix. The installer has not been installed or smoke tested. |
| Windows desktop UI smoke test | PASS for native picker, output picker, preview, two-file runs of all three recipes, output change/collision rejection, and source/output SHA-256 equality. See below for exact scope. |
| macOS build and UI smoke test | NOT RUN: no macOS environment. |
| Windows/macOS GitHub Actions matrix | Prepared, **NOT RUN**: this Git repository has no remote and the connected GitHub account has no matching Intent Layer repository. |

## Defects found and changes

- **Critical:** `execute_recipe` recomputed the plan after preview. The selected destination could silently change. Execution now takes a server-held, opaque, single-use `plan_id`; no client path list is accepted at execution.
- Source bytes and metadata are checked before execution. Occupied destinations and unsafe output parents reject the plan before copying. Every copy uses an exclusive temporary file and publishes a complete file with an exclusive hard link. Originals are never written by these recipes.
- The UI originally showed only five preview rows and one failure. It now provides the complete mapping in 100-row pages and all failures, and invalidates plans when selection, recipe, or output folder changes.
- The initial CI lacked format/lint checks and lockfile-aware installs. The workflow now uses `npm ci`, Vitest, Rust fmt, Clippy, and Rust tests on Windows and macOS runners.
- The CI workflow also performs a Tauri native binary build on both runners. No hosted job has run because the local repository has no remote.
- A Windows UI smoke run exposed duplicate “Preview again” text on backend errors. A frontend regression test failed before the fix; it and the frontend build pass after the UI appends that instruction only when absent. The final native release binary was rebuilt afterward, but that exact error text has not been rechecked in the rebuilt GUI.
- The Windows preview exposed an internal `\\?\` path prefix. The UI now strips this prefix for display while retaining the unchanged plan path for execution; a frontend regression test and final native build pass. The post-fix path rendering has not been rechecked in the GUI.
- A zero-copy run with per-file failures previously used the same green result style as a successful run. It now shows a failure alert, “No files copied,” and the failed-file list; a regression test failed before the fix and passes afterward. The final native executable and installer were rebuilt after this change, but this exact state has not been rechecked in the GUI.

## Safety regression and Windows desktop evidence

The 12 Rust tests cover all three recipes, unchanged originals, duplicate basenames, case-variant names, changed source after preview, collision after preview, expired/unknown plan IDs, invalid/malformed paths, concurrent execution of one plan, concurrent plans targeting one destination, a 100-file Unicode-name batch with source/output SHA-256 equality, and Windows output-junction rejection. Five Vitest tests cover approved ID use, preview invalidation when the recipe changes, single retry messaging, hiding the internal Windows path prefix, and reporting a zero-copy run with failures as failed.

The optimized Windows executable launched. The native picker selected `notes.txt` (14 bytes) and `photo.PNG` (23 bytes); the folder picker selected a temporary output. All three recipe previews showed the planned destinations. Organize, Rename, and Collect each reported two completed copies. Independent SHA-256 checks of all six outputs matched the two originals. Two externally occupied `notes` destinations remained unchanged and Collect chose `notes (2).txt` after a fresh preview.

Creating the output subfolder after preview made execution reject the plan with `Output folder changed`. With the subfolder already present, occupying the planned `notes (1).txt` after preview made execution reject with `Destination is now occupied`; it created no other output from that plan. A new preview selected the next free name and completed. The initial sandboxed debug UI run failed copies with access denied. The same hard-link operation failed in a sandboxed PowerShell probe, succeeded outside the sandbox, and the production UI run outside the sandbox passed. No app-created `.partial` files remained; one `manual.partial` was created by the diagnostic probe.

The native builds used `vcvars64.bat` and project-local Rust directories. Rust warned that hard links in its *incremental compilation cache* were unavailable and copied cache files instead; that cache warning did not prevent the successful build or unsandboxed tests.

The unsigned NSIS package is `src-tauri/target/release/bundle/nsis/Intent Layer_0.1.0_x64-setup.exe` (2,090,880 bytes; SHA-256 `B40653B912180765655913AD7922CBD6389035710AE94BBA8CACDBAA49C7D0F5`). The package was built from the final frontend source. Installation, uninstall, and signature checks have not been performed.

## Remaining risks and priorities

1. Attach a repository remote with workflow permission, execute `.github/workflows/verify.yml` on hosted `windows-latest` and `macos-latest`, and fix any failures. It currently includes npm CI/build/test, Rust fmt/Clippy/tests, and a Tauri binary build.
2. Run the native acceptance matrix on a real macOS host. CI is necessary but does not establish Finder or GUI behavior.
3. Complete Windows and macOS GUI drop tests for 1/10/100 files, source changes, invalid paths, symlinks/junctions, restricted and low-space outputs, and app restart. Only the Windows picker and two-file cases described above were completed; Windows junction rejection has a passing Rust test.
4. Hard-link publication can fail on filesystems without support; this yields a per-file failure and leaves originals intact. Validate removable/network destinations on both OSes.
5. Pathname checks and writes still face races from another process swapping an output directory or reparse point between validation and publication. The [filesystem race audit](FILESYSTEM_RACES.md) specifies platform protections. Same-process plan concurrency tests do not prove hostile same-user filesystem safety.
6. Add progress, cancellation, partial-job recovery, and an output manifest only after the M0 safety gate passes on both platforms.

## Manual smoke matrix to execute

Record date, OS build, hardware, app commit, and PASS/FAIL for each run. On both Windows 11 and macOS: picker; 1/10/100 file drops; full preview; each of the three recipes; duplicate names; occupied output after preview; changed source after preview; symlink and directory rejection; restricted output; source checksums before/after; app restart invalidating a preview. Include screenshots only if captured. The Windows picker, two-file recipes, occupied output, and checksum cells are **PASS** as scoped above; other manual cells are **NOT RUN**. No source commit exists yet, so the build is identified by the current untracked workspace rather than a commit hash.
