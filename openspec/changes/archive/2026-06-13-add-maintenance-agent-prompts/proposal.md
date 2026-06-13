## Why

Polaris maintenance suggestions are currently shallow deterministic candidates, but the output does not guide agents through the judgment needed before deleting or merging memory. Adding configurable agent prompts keeps Polaris conservative while making suggestions more actionable for agent-driven workflows.

## What Changes

- Add default agent review prompts to `polaris prune --suggest` and `polaris compact --suggest`.
- Render prompt templates with structured placeholders for suggestions, workspace status, and known keys.
- Add workspace and user configuration for separate prune and compact prompt templates.
- Include prompts in both human and JSON suggest output by default.
- **BREAKING**: Change `prune --suggest --json` and `compact --suggest --json` from a bare suggestions array to a JSON envelope containing `suggestions`, `prompt`, and `prompt_context`.
- Keep prompts advisory only; suggest commands still must not mutate `.polaris/memories.jsonl` or inline full memory recall content by default.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `polaris-recall`: Maintenance suggestion output gains configurable agent prompts and a new JSON envelope shape.

## Impact

- Affected CLI surface: `polaris prune --suggest`, `polaris prune --suggest --json`, `polaris compact --suggest`, and `polaris compact --suggest --json`.
- Affected configuration: `.polaris/config.toml` and `~/.polaris/config.toml` gain `[maintenance.prompts].prune` and `[maintenance.prompts].compact`.
- Affected implementation: config parsing, suggestion output rendering, JSON response structs, tests, README, and the bundled Codex Polaris skill.
- No new runtime dependencies or external services are required.
