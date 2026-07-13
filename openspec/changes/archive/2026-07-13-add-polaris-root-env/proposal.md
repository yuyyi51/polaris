## Why

Polaris currently binds every memory store to `<cwd>/.polaris`, which prevents callers from keeping the same durable context when commands or hooks execute from a different working directory. A `POLARIS_ROOT` override provides one explicit storage location while preserving the existing workspace-local default.

## What Changes

- Add a `POLARIS_ROOT` environment variable that selects the exact Polaris data directory for ordinary CLI commands and all hook commands.
- Require `POLARIS_ROOT` to be an absolute path; reject empty or relative values before any store-backed command mutates data.
- Keep `<effective-cwd>/.polaris` as the storage root when `POLARIS_ROOT` is unset.
- Apply the selected root consistently to initialization, memory and note persistence, maintenance drafts, citations, hook state, and workspace-level configuration.
- Make every persisted or printed Polaris path absolute in both default and override modes; drop the previous `.polaris/...` relative path convention for new memory records and command output.
- Add end-to-end coverage and user documentation for default and overridden storage roots, including absolute-path output and the absolute-path requirement on the override.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `polaris-recall`: Allow all store-backed CLI and hook behavior to select an explicit Polaris data root through an absolute `POLARIS_ROOT`, while retaining the current cwd-based default and emitting absolute paths for new records and command output.

## Impact

- Store construction and path handling in `src/storage.rs`.
- Ordinary command and hook store selection in `src/cli.rs` and `src/hook.rs`.
- CLI integration coverage in `tests/cli.rs`; assertions that depended on the `.polaris/...` relative path form must use absolute paths.
- Storage-root and hook guidance in `README.md` and `skills/codex/polaris/SKILL.md`.
- No new runtime dependency, no relative-path resolution of the override, and no automatic migration of legacy `.polaris/...` paths in existing memory records.
