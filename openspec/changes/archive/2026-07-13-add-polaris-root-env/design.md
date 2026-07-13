## Context

`PolarisStore` currently stores a workspace path and derives its root by appending `.polaris`. Ordinary commands construct the store from the process current directory, while hooks construct it from the `cwd` supplied in hook JSON. Notes and maintenance drafts additionally build `.polaris/...` paths directly, so changing only `PolarisStore::root()` would leave part of the persistence surface bound to the old location.

The new environment variable must select the same store for every command path, preserve existing behavior when it is absent, and work before the target directory exists so that `polaris init` can create it. This change also drops support for the previous `.polaris/...` relative path convention in favor of absolute paths so that the override behavior is consistent in both default and override modes.

## Goals / Non-Goals

**Goals:**

- Resolve one Polaris data root consistently for ordinary CLI commands and hooks.
- Treat `POLARIS_ROOT` as the exact data directory rather than a workspace to which `.polaris` is appended.
- Require `POLARIS_ROOT` to be an absolute path so the override is unambiguous and hook-evaluable.
- Preserve `<effective-cwd>/.polaris` as the storage root when `POLARIS_ROOT` is unset.
- Make every root-contained file, directory, lock, draft, hook state file, and workspace configuration live under the selected root.
- Make every persisted or printed path absolute in both default and override modes so the override behavior is consistent with the default behavior.
- Make invalid environment configuration fail before a store-backed command mutates data.

**Non-Goals:**

- Searching parent directories for a Polaris store.
- Supporting relative `POLARIS_ROOT` values (including `~` expansion); users must set an absolute path.
- Merging, synchronizing, or automatically migrating data between the default and overridden roots.
- Preserving the historical `.polaris/...` relative path form in new memory records and command output.
- Adding a CLI flag or persisted global setting for the root.
- Restricting the selected directory to a path whose basename is `.polaris`.

## Decisions

### Centralize root resolution in store construction

Both current-directory and workspace-based construction will call one resolver with an effective workspace path:

- If `POLARIS_ROOT` is absent, the resolver selects `<effective-workspace>/.polaris`.
- If it is present and non-empty, the value must be an absolute path. The resolver returns that path as the data root without appending another `.polaris` component.
- If it is present and empty, the resolver fails before any store-backed command mutates data.
- If it is present and relative, the resolver fails with a clear message that absolute paths are required.

The resolver is the single place that reads `POLARIS_ROOT`. Both `from_current_dir()` and `from_workspace(workspace)` call it. The hook command factory closures also call it. This keeps environment handling out of individual commands and makes future store-backed commands inherit the behavior automatically.

Resolving via the env resolver rather than handling the variable separately in every command was chosen because it would be easy for new or nested subcommands to bypass the override. The empty-value and relative-value rejections are part of the same resolver to keep all POLARIS_ROOT validation in one place.

### Store the resolved data root explicitly

`PolarisStore` will retain the effective workspace for resolving caller-supplied relative draft paths and will also hold the resolved data root as an absolute path. All internal file helpers derive paths from that root.

Notes, maintenance drafts, and any other root-relative artifact will be written to an absolute path under the selected root, and the recorded `path` field (for note records) and printed draft paths will be that absolute path. The historical `.polaris/...` relative form is no longer emitted or stored in new records. Existing relative `path` fields from before this change are not migrated; commands that consume them treat them as legacy data and may fail to resolve them, which is acceptable because this change does not promise backward compatibility for the previous path convention.

The workspace field is preserved because `apply_replacement_draft` and `apply_merge_draft` accept a draft path that the caller may express relative to the process working directory. Those commands keep their existing behavior of joining a relative draft path against the workspace, and they also accept an absolute draft path, so absolute paths produced by the new record-and-print code work without further changes.

### Let the environment override hook payload cwd only for storage selection

Hook event matching and input validation remain unchanged. When a matching hook needs a store, its existing store factory calls the shared resolver. A non-empty absolute `POLARIS_ROOT` therefore takes precedence over the hook payload `cwd`; without the variable, hooks retain their current payload-cwd behavior. The empty or relative cases are rejected by the resolver, so they do not reach hook selection.

The direct `session-start` hook and the `post-compact`/`post-tool-use` fallback pair use the same selected root. This ensures fallback pending state and recalled memory cannot be split across directories.

### Treat selected-root configuration as workspace configuration

The first configuration candidate becomes `<selected-root>/config.toml` for both hook recall prompts and maintenance prompts. `~/.polaris/config.toml` remains the user-level fallback, and the two files remain unmerged. This keeps data, hook state, and store-specific policy together when an override is active. Because the root is now always absolute, the candidate path is unambiguous.

### Document environment propagation for hooks

Documentation will show ordinary shell usage and explain that hook commands and later agent-invoked commands such as `polaris recall` must inherit the same `POLARIS_ROOT`. Because `POLARIS_ROOT` is required to be absolute, the example values will use absolute paths, and the documentation will not need to discuss relative-path pitfalls. The recommendation to use an absolute path is enforced by the resolver, not just a documentation hint.

## Risks / Trade-offs

- [A custom root can point to an unwritable or malformed location] → Preserve contextual filesystem errors and reject an empty or relative value before mutation.
- [Existing note records can contain `.polaris/docs/...` paths from before this change] → Do not rewrite existing records automatically. Document that the variable selects a store and that legacy relative path values in memory records may no longer resolve.
- [Some command path may retain a hard-coded `.polaris` component] → Route internal persistence through root helpers and cover notes, drafts, citations, configuration, and all hook-state behavior in integration tests.
- [A user expects `POLARIS_ROOT=../shared` to work] → Reject relative values with an explicit error. Provide a documented `$(pwd)/../shared` pattern as the alternative.

## Migration Plan

No migration is required when `POLARIS_ROOT` is unset. Users can point the variable at an existing Polaris data directory or run `polaris init` to create a new isolated store. Unsetting the variable returns the process to `<cwd>/.polaris`; it does not delete or modify the overridden store. Memory records that contain legacy `.polaris/...` relative paths created before this change are not migrated, and commands that try to resolve them may fail; users who need the new absolute path form should re-create those records under the selected root.

## Open Questions

None.
