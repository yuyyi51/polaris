## Why

Agents need a lightweight way to mark that a specific Polaris memory was actually used as evidence for work. Recall only shows memories, and `touch` only confirms freshness; neither signal should be treated as proof that a memory helped an agent make a decision.

## What Changes

- Add an explicit `polaris cite` command for recording that selected memories were used as cited context.
- Keep citation separate from recall and freshness: `recall` will not mutate citation data, and `touch` will not imply usefulness.
- Expose citation metadata such as created time, direct citation count, inherited citation count, and last citation time in machine-readable memory summaries.
- Preserve citation lineage through replacements and merges without treating inherited citations as direct citations on new content.
- Update maintenance outputs and agent guidance so citation data is advisory context, not an automatic deletion or retention rule.
- Update the bundled Codex skill to teach agents when to cite, when not to cite, and how to batch citation with nearby Polaris or shell work to avoid extra tool calls.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `polaris-recall`: Add explicit citation recording, citation metadata exposure, and citation-aware lineage for replaced or merged memory records.
- `polaris-codex-skill`: Teach agents to use citation signals deliberately and batch citation work when practical.

## Impact

- CLI: adds a `cite` command with exact key/id selectors, multi-selector support, and quiet output.
- Storage: adds durable local citation tracking under `.polaris/` and summary projection for active memory records.
- Maintenance: exposes citation metadata to prune/compact review prompts and JSON summaries without changing mutation behavior.
- Documentation and bundled skill: clarifies the distinction between recall, touch, and cite, and adds guidance for low-noise citation workflows.
