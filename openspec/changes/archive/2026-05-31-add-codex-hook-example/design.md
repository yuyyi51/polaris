## Context

Polaris already implements `polaris hook session-start`, which reads Codex hook JSON from stdin and emits `hookSpecificOutput.additionalContext` only for compact `SessionStart` events when workspace memory exists. The README includes an inline hook configuration, but there is no standalone example file users can copy into a Codex hooks configuration.

The Codex hooks documentation treats `SessionStart` matcher values as start sources, including `compact`, and adds returned `additionalContext` as developer context. Polaris should keep using that mechanism while preserving its current rule that hook output never inlines saved memory.

## Goals / Non-Goals

**Goals:**

- Provide a copyable Codex hooks example in the repository.
- Make the example discoverable from README guidance.
- Validate that the example remains parseable JSON and invokes the supported Polaris hook command.
- Preserve the existing compact-session behavior and memory safety contract.

**Non-Goals:**

- Change the `polaris hook session-start` CLI behavior.
- Add new dependencies or a new runtime hook script.
- Automatically install or modify a user's Codex configuration.
- Support non-compact session sources in the example.

## Decisions

- Store the example under `examples/codex-hooks/hooks.json`.
  - Rationale: `examples/` keeps copyable user-facing material separate from implementation code and avoids implying that the file is active repository configuration.
  - Alternative considered: `.codex/hooks.json`. That could look immediately usable, but it risks being interpreted as local project configuration rather than a reference example.

- Use a command hook that runs `polaris hook session-start` with `matcher: "compact"`.
  - Rationale: This mirrors the existing CLI command and Codex's `SessionStart` source matcher model.
  - Alternative considered: a wrapper shell script. That would add maintenance surface without improving the example.

- Keep README instructions short and point to the example file.
  - Rationale: The example should be the source of truth for the full JSON shape; README should explain when to use it and note that `polaris` must be available on `PATH`.
  - Alternative considered: duplicate the complete JSON in README and the example. Duplication is acceptable if helpful, but tests should still validate the standalone file.

- Validate the example with an integration-style test.
  - Rationale: The project already uses CLI integration tests, and the risk here is configuration drift rather than complex runtime behavior.
  - Alternative considered: no test because this is documentation. A small JSON validation test is cheap and protects the example users will copy.

## Risks / Trade-offs

- Example command assumes `polaris` is available on `PATH` -> README will tell users to adapt the command if their binary is installed elsewhere.
- Codex hook schema may evolve -> keep the example minimal and based only on stable fields already used by Polaris: `SessionStart`, `matcher`, `type`, `command`, and `statusMessage`.
- Users may expect the example to inline memory -> README and spec guidance will state that the hook only asks the agent to run `polaris recall`.
