# Contributing

We welcome platform testing, bug reports, translations, UI improvements and new safe recipe operations.

- Open an issue describing your OS version and a reproducible scenario.
- Run `npm run build` and `cargo test` before PRs.
- For OS-native changes, add a test matrix covering Windows 11 and macOS with screenshots / logs.
- Preserve the non-destructive defaults and avoid arbitrary execution of recipe-defined code.
- Respect licenses and disclose externally bundled binaries/codecs in each PR.
- Do not commit real user files, private paths, credentials, signing keys or personally identifiable test data.
