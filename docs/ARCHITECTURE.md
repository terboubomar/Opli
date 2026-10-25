# Architecture — one engine, two OS adapters

## Principle

**One user experience, not one brittle system hook.** Use Tauri 2 for the frontend, Rust for deterministic execution, and platform-specific thin adapters for Explorer and Finder. The main dropzone is the universal entry point and must work on both platforms before any background interception work.

```
Explorer (Windows)                Finder (macOS)
IExplorerCommand [later]          Quick Action extension [later]
         \\                      /
           → IPC / file input ←
                  ↓
       Tauri 2 + React shell
                  ↓
        Rust plan + validation
                  ↓
        User verifies preview
                  ↓
          Safe recipe runner
                  ↓
   Outputs + job manifest [later]
```

## Current package topology

- `src/`: React UI. Native file picker, native file-drop event, recipe cards, preview and execution results.
- `src-tauri/src/lib.rs`: small Tauri command adapters. No file logic in frontend.
- `src-tauri/src/recipe.rs`: current deterministic copy-only recipes with planning and safe file creation.
- `src-tauri/capabilities/default.json`: minimal dialog + core permissions; do not enable broad FS or shell permissions.
- `docs/`: contracts, milestone acceptance criteria, and security policy.

## Future structure (after first native build)

```
crates/recipe-schema/   # serde JSON recipe format + validation
crates/recipe-engine/   # DAG planner + byte-limited safe executor
crates/file-adapters/  # image / audio / document adapters
platforms/windows/     # Explorer adapter (native IExplorerCommand)
platforms/macos/       # Finder action extension written in Swift
apps/desktop/          # Tauri shell + React UI
```

## Platform-specific feasibility

| Capability | Windows | macOS |
|---|---|---|
| Receive files in app | Tauri webview file drop | Tauri webview file drop |
| Select arbitrary files | Native file picker | Native file picker |
| OS context action | Explorer IExplorerCommand (native COM component) | Finder Quick Action / Action extension (Swift) |
| Persistent entry | Tray icon | Menu bar extra |
| Pointer-local radial UI | Test transparent topmost Windows overlay | Test NSPanel / window level + Spaces permissions |
| Global intercept every drag | Not a v1 promise: diverse OLE data objects | Not a v1 promise: Finder/app variations and permissions |

Windows modern context-menu extension needs a packaged app identity and native COM implementation. macOS Finder Quick Actions can be integrated without requiring Accessibility permissions for ordinary file-action extension input, but input monitoring and cross-app interception have separate consent implications. Neither requires a global drag hook in milestone 1.

## Safety invariants

1. The initial recipes **never modify or delete the source file**.
2. Destination outputs are created with exclusive creation, **never overwritten**.
3. Preview must show each planned output, and changed file state at execution must fail closed rather than overwrite.
4. No execution of external shell scripts from third-party recipes (future plugins are capability limited, signature/permission gated).
5. Malformed paths and symlinks are explicitly rejected; detect path traversal if community recipes are introduced.
6. Hidden network calls, unrequested uploads, and broad disk scanning are prohibited.
7. Larger files must eventually use streaming processing, bounded concurrency, cancellation, and an operation journal.

## Current caveats

A preview is registered in memory under a random, single-use plan ID. Execution consumes that exact plan. Changed sources, output-root metadata, occupied paths, and unsafe existing output parents reject execution before copying. Each item is written to an exclusive temporary file, verified, and published with an exclusive same-filesystem hard link. Windows native compilation and a limited desktop workflow pass; macOS remains unverified. Hard-link support and concurrent directory replacement still need platform validation; see [filesystem race audit](FILESYSTEM_RACES.md). A future executor should add a journal and stronger platform-specific path controls. The GUI lacks automated E2E coverage.
