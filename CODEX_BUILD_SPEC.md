# Intent Layer — Codex Master Build Specification & Skills Playbook

**Working title:** Intent Layer  
**Target platforms:** Windows 11 and macOS (Apple Silicon first; Intel where CI/hardware permits)  
**Stack:** Tauri 2 · Rust · React 19 · TypeScript · Vite · native OS adapters  
**License:** MIT for original project code; respect the separate licenses of all bundled dependencies  
**Document version:** 1.0 — 25 October 2026  
**Audience:** OpenAI Codex (primary), human contributors, and compatible AI coding agents

> **Start here, but inspect the repository before making changes.** This is an implementation specification, not proof that future features exist. The starter has three hardcoded copy-only recipes, a React interface, Tauri IPC, native file selection, and in-window drop handling. The authoring environment did **not** successfully compile and launch the desktop binary on Windows or macOS. Do not claim either platform passes until it is tested on the actual OS.

## 0. Instruction hierarchy / first read

1. Read `AGENTS.md`, `README.md`, `START_WITH_CODEX.md`, `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, `docs/RECIPE_SPEC.md`, `docs/SKILLS.md`, `SECURITY.md`, and this file.
2. Read the current Rust implementation in `src-tauri/src/recipe.rs` and `src-tauri/src/lib.rs`, the React implementation in `src/App.tsx`, and the Tauri capability file.
3. Treat this specification as intended direction, not permission to discard existing working code. If instructions conflict, **source safety and restrictive permissions win**; document the conflict.
4. Before coding, produce an auditable baseline: installed toolchain versions, successful/failed commands, observed UX issues, and a short prioritized work plan.
5. Work in small, testable commits. Never fabricate tests, screenshots, successful builds, performance results, or supported file formats.

## 1. Product vision and differentiation

**One sentence:** A local-first action layer for Windows and macOS: select files, choose the *outcome* you want, preview exactly what will happen, and execute a reusable, inspectable Recipe.

This is **not** a Tangerine rebrand, another format-conversion menu, or an autonomous filesystem AI. We are designing a general-purpose, trustworthy desktop workflow experience.

**North-star user journey**

1. From Finder, Explorer, or the main app, select or drop one or more files.
2. See only relevant intent cards: `Organize`, `Rename`, `Collect`, eventually `Prepare to share`, `Web-ready images`, `Protect metadata`.
3. Pick a Recipe; inspect a human-readable, file-by-file execution plan and approximate disk impact.
4. Choose the output location and grant necessary permissions explicitly.
5. Execute with bounded concurrency, visible progress, cancellation, and clear results.
6. Find the output and review a local activity record; eventually safely undo *only tracked outputs*.

**Design principles**

- Outcome-first: do not start with a codec list.
- Offline by default; zero mandatory account, telemetry, online service, or AI.
- Preview before mutation; user files remain untouched in v0.1.
- Accessible via keyboard and pointer; predictable over clever.
- One portable recipe engine, narrow OS-specific integrations.
- Community-friendly: MIT source, repeatable builds, documented recipes, transparent permissions.

## 2. Actual baseline vs planned capabilities

| Capability | Existing starter | Status / required work |
| --- | --- | --- |
| Tauri 2 + React desktop shell | Present | **Unverified native builds** on both target OSes |
| Main-window drop and file picker | Implemented in UI | Manual Windows/macOS behavior verification required |
| Organize by extension | Hardcoded Rust recipe | Add tests, input safety and error UX |
| Sequential copy rename | Hardcoded Rust recipe | Add tests for case, Unicode, collisions |
| Handoff collection | Hardcoded Rust recipe | Add tests for duplicate filenames |
| Preview list | Present | **Execution currently replans:** must make approved plan binding explicit |
| Atomic job state / cancellation | Not present | Later milestone |
| Recipe JSON schema + editor | Draft doc only | Not implemented; version and validate before enabling |
| Native Finder / Explorer actions | Not present | Platform-specific milestone |
| Radial popup near cursor | Not present | Feasibility prototype later; not a v0.1 promise |
| Tray/menu bar, global shortcut | Not present | After stable core |
| Image/PDF conversion | Not present | Future carefully licensed adapters |
| Cross-platform installers | Config present | Unsigned build verification and CI first |

### Critical baseline correctness concern

The UI calls `preview_recipe(...)` to show a plan, but `execute_recipe(...)` currently calls `create_plan(...)` again. Therefore, a changed filesystem may yield outputs different from those approved in the preview. **Milestone 0 must replace this with a server-side approved-plan token or equivalent immutable validated plan.** It must revalidate file identity/metadata and output safety before copying, fail closed on changed plans, and never use an untrusted client-supplied destination list as authority. Do not mistake exclusive `create_new` output creation (already used) for complete preview-to-execution binding.

## 3. Functional specifications

### 3.1 Select and classify

- Accept ordinary, user-selected regular files, batch selections, and drag/drop into the main app.
- Show count, readable total size, simplified filename, full path on demand, and classification.
- Validate file state on the Rust side: missing files, directories, symlinks, unsupported OS path edge cases, inaccessible files, duplicates, and oversized batches.
- Disallow implicit recursive directory traversal in the first release.
- Never call network services to classify file types; use explicit deterministic rules. Extension-based grouping is an alpha approximation, **not** content verification.

### 3.2 Recipes in v0.1

- **Organize by type:** produce copies grouped into `Images`, `Documents`, `Media`, `Archives`, `Other`.
- **Rename cleanly:** produce prefixed, sequentially numbered copies without rewriting originals.
- **Collect for handoff:** copy into one dedicated output folder.
- For all three, preserve bytes; handle duplicate basenames with unique destinations; never overwrite a destination; preview all path mappings, not just a count.
- Use a dedicated subfolder; avoid writing directly beside the source unless the user explicitly approves a valid output location.

### 3.3 Workflow preview contract

The Rust engine returns a validated plan with:

- `plan_id` (opaque, unguessable identifier), recipe and schema versions;
- ordered source/destination pairs, sizes, expected source metadata or content signature strategy;
- reasoned warnings, required capabilities, estimated bytes, collision policy;
- creation time / validity window and explicit output root.

Execute **only** the plan identified by `plan_id`. A UI change to files, recipe, or output folder invalidates the plan. Before executing, re-check source identity/type and destination safety, and reject changed files/paths with actionable reasons. Never switch to a newly computed destination silently. If plan persistence is in-memory for v0.1, expire it on app exit; do not persist stale absolute paths automatically.

### 3.4 Execution contract

- Job state machine: `draft -> planned -> running -> completed | partial | failed | cancelled` (cancellation may be milestone 2).
- Each file has independent status and errors; track per-file successes and failures.
- `create_new`/exclusive writes, safe directory boundaries, no overwrite, bounded memory.
- Crash or copy failure must not result in an output being presented as successful when it is incomplete. Use temporary files in the same destination filesystem and safe finalization where feasible; document platform-specific atomicity limits.
- Recheck OS filesystem behavior for symlinks/junctions, hard links, TOCTOU races, case-insensitive names, path lengths, and output-root changes.
- Sources are never deleted, renamed, or modified by the first-release Recipes.
- Each run produces an understandable summary; avoid exposing sensitive full paths in optional diagnostics or public issue templates.

### 3.5 UX requirements

- **Workspace:** drag/drop or browse, selected-file list, intent cards, output destination, preview, run, results.
- **Preview:** show first rows + expandable full plan, input/output count, byte estimate, warnings, collision treatment, and what will *not* change.
- **Results:** complete/partial/failure clearly differentiated; show per-item failures and an `Open output folder` action when supported.
- **States:** empty, dragging, validation error, loading, successful preview, running, partial failure, completed.
- **Accessibility:** logical tab order, accessible names, visible focus, contrast meeting WCAG 2.2 AA where applicable, OS reduced-motion support, no icon-only meaning, keyboard alternatives to drag/drop.
- **Visual direction:** restrained, fast, native-feeling, no fake metrics, no ornamental dashboards. Radial overlay is future exploration, not a prerequisite to safely running Recipes.

## 4. Architecture and boundaries

### Current layout (keep until justified)

```text
intent-layer/
  AGENTS.md
  CODEX_BUILD_SPEC.md          # this implementation playbook
  START_WITH_CODEX.md
  README.md
  docs/
    ARCHITECTURE.md
    ROADMAP.md
    RECIPE_SPEC.md
    SKILLS.md
  src/
    App.tsx                    # frontend, presentation and IPC clients
    styles.css
  src-tauri/
    src/
      lib.rs                   # IPC commands / Tauri initialization
      recipe.rs                # alpha planner and copy-only runner
    capabilities/default.json  # restrictive permission grants
    tauri.conf.json
  .github/workflows/verify.yml
```

### Target modules (introduce incrementally, not a premature rewrite)

```text
crates/
  recipe-schema/               # typed, versioned serde recipe definitions
  recipe-engine/               # validation, planning, execution, journal
  operation-adapters/          # opt-in, audited format-specific operations
platforms/
  windows/                     # optional Explorer integration adapter
  macos/                       # Finder Quick Action/extension adapter
apps/
  desktop/                     # shared Tauri shell + React UI
```

**Rules:**

- Frontend never gets general filesystem or shell execution rights. All data-changing operations go through typed Rust commands and an explicit scope check.
- The core engine has **no** Explorer, Finder, window, keyboard, or Tauri dependencies where separation is practical.
- Native adapters translate OS activation events into validated file-selection intents. They never bypass preview or call the recipe engine directly to modify files.
- Introduce SQLite only when persistent jobs, history, migrations or safe undo require it. Do not add a database merely for three simple recipes.

### Suggested IPC surface, to implement in stages

```ts
// Illustrative contract, not current code.
type PlanRequest = {
  paths: string[];
  recipeId: string;
  outputDirectory: string;
  options?: Record<string, unknown>;
};
type PlanResponse = {
  planId: string;
  recipeId: string;
  outputs: Array<{ input: string; output: string; bytes: number }>;
  warnings: string[];
  estimatedBytes: number;
};
type ExecuteRequest = { planId: string };
type JobProgress = { jobId: string; done: number; total: number; state: string };
```

Generate or share types from the schema where feasible; validate unknown input and avoid unchecked `unwrap()` on user-controlled data.

## 5. Cross-platform integration strategy

### Stage A — dependable and universal

- Main application accepts dragged paths and native dialog choices on Windows and macOS.
- All three alpha recipes and the preview safety contract work without OS-specific plugins.
- A global shortcut and tray/menu bar are **not** required to claim this stage.

### Stage B — native entry points

- **Windows:** assess `IExplorerCommand` and Windows 11 packaging constraints; use a documented Explorer context-menu or `Send to` fallback if necessary. Prototype on a real Windows machine; support many selected paths safely via an authenticated local activation mechanism or bounded argument transport.
- **macOS:** prototype a Finder Quick Action / Action extension using Swift. Pass file URLs securely into the running app or a validated launch handoff; test file access/consent, sandbox, and app lifecycle. Do not assume an extension can use the app's current process directly.
- Add single-instance routing so later launches focus an existing app and transfer selected input safely.
- Never request Accessibility, Screen Recording, or Input Monitoring permissions for ordinary file selection or Quick Actions. Permission requests must be narrowly justified by an actual feature.

### Stage C — optional pointer-local / radial interaction

- Prototype on both OSes independently: transparent always-on-top window, pointer anchoring, multi-monitor scaling, focus behavior, app switching, keyboard navigation, screen-reader behavior, Escape-to-dismiss.
- Treat a system-wide intercept of arbitrary third-party drags as **research**, not a v1 dependency or marketing promise. Keep dropzone, Finder/Explorer actions, and shortcuts as stable fallbacks.
- Preserve a non-radial accessible list mode.

## 6. Recipe ecosystem, after core is stable

- Start with **signed, reviewed built-in operations** and a versioned declarative JSON schema, not arbitrary code.
- Each Recipe declares: unique identifier, schema version, supported input kinds, steps, estimated output behavior, permissions, resource limits, provenance, and optional minimum app version.
- Validation enforces an allowlisted step type, typed arguments, normalized paths confined to an approved destination, and explicit access scopes.
- No `eval`, inline shell scripts, PowerShell, Bash, arbitrary process execution, hidden network requests, or unsafe dynamic plugins from community recipes.
- Include inspectable preview, local execution history, repeatability, and eventually permission-aware community review.
- If optional AI generation is added, AI only drafts a declarative recipe for **human review**, never directly triggers filesystem writes.

## 7. Milestones, tasks, and definition of done

### M0 — baseline verification and preview safety (**must finish first**)

**Work items**

1. Install actual platform prerequisites and capture versions. Lock dependencies (`package-lock.json`, `Cargo.lock`) intentionally; use lockfile-aware CI installs.
2. Run `npm install`, `npm run build`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `npm run tauri dev` on Windows and macOS. Fix actual errors.
3. Reproduce and fix preview/re-plan mismatch with an approved-plan ID; add tests for collisions introduced after preview and changed input metadata.
4. Test 1, 10, and 100 files, filename case/Unicode, duplicate basenames, symlinks, folder inputs, missing paths, long paths, and destination access failures.
5. Verify source bytes/checksums unchanged; failures do not corrupt preexisting outputs; report partial success truthfully.
6. Add accessible full-plan inspection and clear per-file error reporting.
7. Extend CI for both OSes and publish a **manual smoke-test matrix** in `docs/ROADMAP.md` with actual run dates and hardware/OS versions.

**Gate:** real Windows 11 and macOS UI smoke tests plus relevant unit tests pass, sources remain unchanged, approved plan never changes silently. A CI build alone is **not** the entire gate.

### M1 — native convenience surface

1. Build tray/menu bar access and configurable shortcut, plus shortcut conflict / permissions UX.
2. Add safe single-instance activation and input handoff.
3. Prototype Finder Quick Action and Windows Explorer context action independently.
4. If one OS is blocked, ship fallback for that OS and mark the native integration as beta; never misstate parity.

**Gate:** select a file in Finder/Explorer, invoke Intent Layer, preview, run, and open outputs on each supported OS without unrequested filesystem mutation.

### M2 — durable jobs and first rich intent

1. Typed Recipe schema and migrations; reject unknown or untrusted step types.
2. Background job queue with progress, cancellation, recovery manifest, disk-space handling and safe partial-file cleanup.
3. First differentiated workflow: **Web-ready images** (explicit resize mode, image format, quality, metadata policy), with image-quality tests and dependency license review.
4. Safe undo for **tracked app-created outputs only**, with user confirmation and identity checks; never delete or revert untracked user files.

**Gate:** deterministic sample fixtures, no source mutation, no untracked deletion, clear error/progress, repeatability across OSes.

### M3 — contextual/radial UX and community

1. Two OS-specific overlay feasibility prototypes with focus/monitor/accessibility test reports.
2. A reusable Recipe editor, import/export validation, schema versioning, contribution policy.
3. Unsigned preview builds for testers, then code-signing/notarization infrastructure before stable distribution.

**Gate:** interaction is not blocking, launch and packaging are reproducible, permission requests are scoped and explained, regression suites remain green.

### M4 — release readiness

- Security review of path handling, IPC, native activation, update mechanism and dependency supply chain.
- Signed Windows release artifacts, signed/notarized macOS app, versioned release notes, source tarball, checksums, SBOM / third-party notices where practical.
- Contributors can build and run from documented prerequisites with no private credentials.

## 8. Test strategy and release quality bar

**Rust tests:** planning determinism, identical names from different folders, extension case, Unicode, symlink refusal, nonexistent paths, case-insensitive collisions, destination modification after preview, permission denied, zero-byte files, 10k file limit, source changes, safe partial cleanup.

**Frontend tests:** state transitions, preview invalidated on selection/destination/recipe change, busy/disabled controls, accessibility semantics, unexpected IPC errors, large list performance. Add a small frontend test runner only when adding tests (e.g. Vitest + Testing Library).

**Platform tests:** Windows 11 Explorer and multiple monitors (100%, 150%, 200% DPI); macOS Finder and multiple displays; Intel vs Apple Silicon when available; spaces/fullscreen when overlay exists; restricted folders, removable drives, concurrent write interference.

**CI:** both OSes run frontend build, Rust format/lints/tests. Production bundling tested separately; CI machine success does not imply a Finder extension, Explorer menu, or graphics overlay functions.

**For every pull request** record:

```text
Summary:
Risks and migrations:
Relevant skill(s) used:
Test command / OS / result (PASS, FAIL, NOT RUN):
File safety and permission impacts:
Screenshots or short video (for visual work, if available):
Known limitations:
```

## 9. Recommended Codex skills — curated, with installation

These are **agent-development guidance**, not runtime application dependencies. Do not commit copied third-party skill code into this MIT repository without checking its license. Install into the project or user skills directory and review the contents before executing installer scripts.

### Essential by task

| Skill / repository | Apply when | Priority |
| --- | --- | --- |
| [Superpowers](https://github.com/obra/superpowers) | Requirements, task breakdown, TDD, code reviews, bug isolation | Every milestone |
| [Tauri Skills](https://github.com/full-stack-skills/tauri-skills): `tauri-app-develop`, `tauri-ipc`, `tauri-app-dialog` | Initial Tauri build, React ↔ Rust bridge, file picker | M0 |
| Tauri Skills: `tauri-security`, `tauri-framework-security`, `tauri-app-plugin-permissions` | Scoped permissions, IPC and safe native integrations | M0 onwards |
| Tauri Skills: `tauri-app-planning`, `tauri-build` | Architecture decisions, dual-platform CI, packaging | M0 / M4 |
| [Impeccable](https://github.com/pbakaus/impeccable) | Accessible UI audits, interaction cleanup, visual consistency | M0 UX, M3 |
| Tauri Skills: `tauri-app-global-shortcut`, `tauri-app-system-tray`, `tauri-app-single-instance` | Native launch and activation | M1 |
| Tauri Skills: `tauri-window`, `tauri-app-positioner` | Overlay/window feasibility | M3 |
| [Taste Skill](https://github.com/Leonxlnx/taste-skill) `design-taste-frontend` | Visual hierarchy, typography, motion experiments (adapt web guidance to desktop) | Optional M3 |

**Scope control:** install just the skill(s) relevant to the current milestone rather than installing all 52 Tauri skills. Skills never supersede official Tauri docs or repository safety invariants.

### Tauri Skills installation — Windows PowerShell **or** macOS Terminal

From the repo root, after installing Node.js:

```sh
npx skills add full-stack-skills/tauri-skills --skill tauri-app-develop -a codex
npx skills add full-stack-skills/tauri-skills --skill tauri-ipc -a codex
npx skills add full-stack-skills/tauri-skills --skill tauri-app-dialog -a codex
npx skills add full-stack-skills/tauri-skills --skill tauri-security -a codex
npx skills add full-stack-skills/tauri-skills --skill tauri-build -a codex
```

If your version of `skills` asks about destination/installation mode interactively, choose Codex and a project-local installation. Verify the resulting skills are discoverable; CLI switches can evolve.

**When M1 starts:**

```sh
npx skills add full-stack-skills/tauri-skills --skill tauri-app-global-shortcut -a codex
npx skills add full-stack-skills/tauri-skills --skill tauri-app-system-tray -a codex
npx skills add full-stack-skills/tauri-skills --skill tauri-app-single-instance -a codex
```

### Impeccable (Codex)

```sh
npx impeccable install --providers=codex --scope=project
```

This installs Codex-ready design skills; if Codex requests approval for project hooks, inspect them first. See the project's current installer documentation: https://github.com/pbakaus/impeccable .

### Taste Skill (optional, in the design phase)

```sh
npx skills add https://github.com/Leonxlnx/taste-skill --skill design-taste-frontend -a codex
```

### Superpowers for Codex

Superpowers uses native Codex skill discovery. Read its official setup instructions before running commands:

- https://github.com/obra/superpowers/blob/main/.codex/INSTALL.md
- https://github.com/obra/superpowers/blob/main/docs/README.codex.md

A non-executable instruction to give Codex: `Read and follow the installation instructions in https://raw.githubusercontent.com/obra/superpowers/refs/heads/main/.codex/INSTALL.md, and explain any commands that change global directories before executing them.`

**Do not** assume Superpowers is installed just because this Markdown lists it. Confirm skill discovery in Codex.

### How Codex should choose skills

- **Architecture or significant feature:** Superpowers planning → Tauri planning → security skill.
- **Rust IPC and file handling:** Superpowers TDD → `tauri-ipc` → `tauri-security`.
- **Window, shortcut, tray, Finder/Explorer activation:** relevant Tauri skill + OS API docs + real machine test.
- **UI/UX work:** Impeccable first; Taste for optional visual exploration; always validate React accessibility.
- **Build, CI and distribution:** `tauri-build` + official Tauri release/signing docs.

## 10. Codex execution instructions — use these prompts

### Master kickoff prompt (paste into Codex at repo root)

```text
You are the lead cross-platform desktop engineer for Intent Layer.
Read AGENTS.md, CODEX_BUILD_SPEC.md, README.md, START_WITH_CODEX.md,
SECURITY.md and all docs before touching code. Inspect the codebase.

Goal for the FIRST iteration: complete M0 only. Do not rewrite the app,
add AI, add an overlay, or install broad filesystem/shell permissions.

1. Identify the baseline: what actually compiles/tests here, and what is unverified.
2. Prioritize the preview-to-execution mismatch and input/output safety.
3. Propose small, reviewable tasks with failing tests first.
4. Implement the minimum safe fixes, run real commands, report results.
5. Preserve original-file contents, zero-overwrite behavior, and offline operation.
6. Document Windows/macOS verification gaps honestly.

Use installed Superpowers, relevant Tauri skills and Impeccable where applicable.
Deliver a concise summary, changed files, test matrix and remaining risks.
```

### Follow-up task: typed execution plan

```text
Implement an opaque approved-plan ID for Rust IPC. `preview_recipe` must
register an immutable validated plan; `execute_recipe` must use that exact
plan, not regenerate destinations. Fail closed when inputs, output directory,
permissions, or intended output state changed after preview. Add Rust unit
and integration tests for the preview-vs-execute race, symlinks, duplicate
names, and source integrity. Keep the frontend contract typed and invalidate
plans on any relevant UI change. Do not add broad shell or filesystem grants.
```

### Follow-up task: Windows/macOS parity

```text
Run the current app on a real Windows 11 machine and a real macOS machine.
Verify file picker, in-window drop, preview, all 3 recipes and partial errors.
Provide reproducible test steps, OS/build info, PASS/FAIL/NOT RUN status,
and screenshots only if actually captured. Fix OS-specific bugs in the
smallest adapter or UI surface; do not claim cross-platform parity if one
platform is not tested.
```

### Follow-up task: UI design

```text
Use Impeccable to audit the workspace and preview flow for accessible,
keyboard-first, minimalist desktop UX. If available, use Taste Skill only
for motion/typography direction. Implement coherent empty/loading/error/
partial-success states, full preview inspection and reduced-motion support.
Preserve existing working recipe functionality and verify with tests.
```

## 11. Exact developer commands

```sh
# From repository root:
node --version
npm --version
rustc --version
cargo --version

npm install
npm run build

cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml

npm run tauri dev
# After environment and tests are green, try a local unsigned bundle:
npm run tauri build
```

Do not expect one machine to create and test fully native binaries for both OSes. Use a Windows host and a macOS host (or OS-specific hosted CI runners). For a portable release use the official Tauri Windows installer, macOS code-signing, and notarization documentation; do not publish unsigned artifacts as a trusted stable release.

## 12. Stop conditions / non-goals

**Stop and raise an issue rather than silently working around these:**

- An operation would overwrite, rename, delete, or corrupt an original input.
- A required OS permission or dangerous Tauri capability is broader than the feature needs.
- A proposed recipe executes arbitrary untrusted scripts or performs hidden network actions.
- A native integration depends on undocumented interception with no viable accessibility fallback.
- A dependency has an incompatible/unclear redistribution license.
- Tests fail or cannot run but release notes would claim success.

**Not in the initial MVP:** system-wide interception of all drags, an autonomous AI agent, cloud recipe sync, a public workflow marketplace, unreviewed executable plugins, background surveillance, 275 claimed formats, or replication of Tangerine trademarks/assets.

## 13. Release checklist (Codex must keep updated)

- [ ] M0 tests/builds verified on a real Windows 11 environment.
- [ ] M0 tests/builds verified on a real macOS environment.
- [ ] Execution uses a validated user-approved plan, not a silent replan.
- [ ] Safety tests for overwritten destinations, changed sources, duplicates and symlinks pass.
- [ ] README feature claims match tested behavior.
- [ ] Only least-privilege Tauri permissions are enabled.
- [ ] Dependency lockfiles and license attribution are captured.
- [ ] CI matrix passes for targeted runners.
- [ ] Reproducible install/run instructions and platform limitations are documented.
- [ ] Product branding is original and independent.

### Primary external references

- Tauri 2 official docs: https://v2.tauri.app/
- Tauri permissions / capabilities: https://v2.tauri.app/security/capabilities/
- Tauri global shortcuts: https://v2.tauri.app/plugin/global-shortcut/
- Tauri packaging/signing: https://v2.tauri.app/distribute/
- Superpowers: https://github.com/obra/superpowers
- Tauri Skills: https://github.com/full-stack-skills/tauri-skills
- Impeccable: https://github.com/pbakaus/impeccable
- Taste Skill: https://github.com/Leonxlnx/taste-skill

---

**Completion means demonstrable correctness, not a longer feature list.** Ship a trustworthy outcome-first workflow on both operating systems; expand only after the core works.
