# Security policy (alpha)

Do not test on irreplaceable files or shared production folders. Alpha output paths are copy-only, but this starter has not been independently audited or native-built on both platforms.

- Report vulnerabilities privately through the maintainer's future GitHub security advisory workflow (configure it before public release).
- Scope file access to explicit user-selected paths and destinations.
- Treat file formats as untrusted input. Future codecs must run in bounded isolated workers.
- Recipes must be declarative, validated, and permission-scoped.
- Never execute shell commands from downloaded recipes.
- Do not add telemetry, uploads, or automatic network access without explicit opt-in.
