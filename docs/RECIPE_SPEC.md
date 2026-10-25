# Recipe contract — proposal (v0.2, NOT YET IMPLEMENTED)

A *recipe* describes an outcome using declarative steps, not arbitrary code.

```json
{
  "schemaVersion": "0.2-draft",
  "id": "org.intentlayer.organize-images",
  "displayName": "Sort photos into folders",
  "input": { "kinds": ["file"], "extensions": ["jpg", "jpeg", "png"] },
  "steps": [
    { "type": "ensure-directory", "path": "outputs/images" },
    { "type": "copy", "destination": "outputs/images/{filename}" }
  ],
  "permissions": ["read-selected-files", "write-user-approved-output"],
  "network": false
}
```

## Planned enforcement

- Validate schema with explicit allowlists; reject unknown step types and dangerous paths.
- Resolve tokens to normalized paths and check target folder boundaries.
- Show user a complete plan and required capabilities before execution.
- Publish an immutable execution plan with source metadata stamps, byte estimates, and collision strategy.
- Create outputs safely with exclusive file creation, and record per-operation manifests for rollback.
- Process conversions in sandboxed worker processes; distinguish true format conversions from file renames.
- Never expose a `shell`, `powershell`, `bash`, `eval`, or unreviewed plugin command as a default recipe operation.

The desktop alpha currently uses three hardcoded Rust recipes, not the draft JSON schema. Keep this distinction clear in README and UI.
