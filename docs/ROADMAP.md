# Roadmap and acceptance gates

## Milestone 0 — proof on BOTH platforms

Implementation progress and exact command results: [M0 baseline](M0_BASELINE.md). The M0 gate is still open.

- [ ] `npm install`, `npm run build`, `npm run tauri dev` work on Windows 11 x64.
- [ ] Same commands work on a supported Intel or Apple Silicon macOS machine.
- [ ] Native file chooser adds valid paths from Explorer and Finder.
- [ ] Drop of 1/10/100 normal files works on both OSes; dragging out of the window doesn't trigger a run.
- [ ] Invalid/unsupported input yields a usable message, no crash.
- [ ] 3 Rust recipes produce valid copies with planned destination structure.
- [ ] Duplicate basenames; file collision; source mutation mid-run; symlinks; low-disk-space behavior tested.
- [ ] Source checksums unchanged after each workflow.
- [ ] CI runs frontend build and Rust tests on Windows and macOS. Matrix configured; hosted run not yet verified.

- [x] Approved preview ID replaces silent execution replanning; 12 native Rust tests pass on Windows outside the hard-link-restricted sandbox.
- [x] Frontend production build passes on this Windows host.
- [x] Windows native debug and optimized binary builds pass; native picker and two-file runs of all three recipes pass in the release app.
- [x] Both lockfiles exist and CI uses lockfile-aware installs.

**Gate: do not proceed until M0 passes on BOTH operating systems.**

## Milestone 1 — native launch surfaces

- [ ] Tray (Windows) and menu bar (macOS) entry.
- [ ] Finder Quick Action prototype that passes selected paths through native channel/URL to app.
- [ ] Explorer action prototype via native `IExplorerCommand` or a documented fallback.
- [ ] Clear onboarding for OS permissions; no blanket Accessibility request.
- [ ] User-selectable keyboard shortcuts with conflict handling.

## Milestone 2 — durable workflows

- [ ] Typed versioned recipe schema and migrations.
- [ ] Execution journal, cancellation, recovery from interrupted batches.
- [ ] Image resize/convert with benchmark, metadata handling, and objective tests.
- [ ] More advanced file analysis and format support, licensing review.
- [ ] Reproducible signed builds; notarization; upgrade and rollback.

## Milestone 3 — intentional delight

- [ ] Floating command menu and radial overlay (separate OS feasibility POCs).
- [ ] Natural language recipe designer (optional/offline-capable, never arbitrary code execution).
- [ ] Community recipe sharing with secure review and trust indicators.

## Non-goals for v0.1

- System-wide interception of every drag operation.
- Autonomous AI changing the disk without preview.
- Claims of 275 supported conversions.
- Adopting or copying Tangerine branding, icons, or proprietary assets.
