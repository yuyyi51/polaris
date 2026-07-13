## 1. Store Root Resolution

- [x] 1.1 Add CLI integration tests for unset, absolute, empty, and isolated `POLARIS_ROOT` selection, including the resolved `status --json` root, and an explicit test that rejects a relative `POLARIS_ROOT` with a clear error.
- [x] 1.2 Implement one environment-aware root resolver shared by current-directory and workspace-based `PolarisStore` construction, treating an absolute override as the exact root and rejecting empty or relative values before any mutation.
- [x] 1.3 Store the resolved data root explicitly as an absolute path and route state, memories, history, citations, locks, docs, maintenance, hook state, and store-specific configuration through root-based helpers.

## 2. CLI and Hook Integration

- [x] 2.1 Audit every store-backed ordinary command to use the shared resolver and add regression coverage showing reads and writes never fall back to the cwd store when an override is active.
- [x] 2.2 Add direct and fallback hook tests for override precedence, payload-cwd defaults, missing-cwd fallback, and shared overridden pending state, then connect all three hook commands to the shared resolver.
- [x] 2.3 Add hook and maintenance prompt tests proving `<selected-root>/config.toml` precedes `~/.polaris/config.toml` under an override while default-mode precedence remains unchanged.

## 3. Root-Contained Artifacts

- [x] 3.1 Make note and maintenance-draft paths absolute in both default and override modes; record the absolute path on new memory records, print absolute paths from create commands, and verify the new apply commands accept those absolute paths.
- [x] 3.2 Add end-to-end coverage for note records, replacement and merge draft creation/application, citations, replacement history, locks, and hook state under a custom root, including assertions that cwd `.polaris` receives no corresponding files and that recorded `path` values are absolute.

## 4. Documentation

- [x] 4.1 Document `POLARIS_ROOT` semantics, exact-directory behavior, the absolute-path requirement (including rejection of empty and relative values), store isolation, and shell examples in `README.md`.
- [x] 4.2 Update `skills/codex/polaris/SKILL.md` with custom-root and hook environment guidance, including the requirement to use an absolute path and propagate the same value to later `polaris recall` commands.

## 5. Verification

- [x] 5.1 Run `cargo fmt --check`, `cargo test`, and `cargo clippy`, and resolve any failures introduced by the change.
- [x] 5.2 Run strict OpenSpec validation for `add-polaris-root-env` and confirm the implementation matches every added and modified scenario.
