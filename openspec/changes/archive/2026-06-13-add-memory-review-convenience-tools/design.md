## Context

Polaris already supports keyed recall, lifecycle moves, replacement history, and post-replace diffs. In practice, agents still hit three friction points: replacement review happens after writing, recalling several exact keys requires multiple commands or a broader recall, and still-valid `state` records can keep appearing as stale unless their timestamp is refreshed through a content rewrite.

The existing merge workflow already uses editable drafts under `.polaris/maintenance/`. This change extends that pattern to replacements while keeping recall and touch as small explicit CLI additions.

## Goals / Non-Goals

**Goals:**
- Let agents preview and edit a replacement draft before mutating active memory.
- Keep replacement draft apply explicit and confirmed.
- Allow precise recall of several exact keys in one command.
- Allow agents to refresh `updated_at` for still-valid records without changing content or lifecycle.
- Keep machine-readable output available where agents need it.
- Update documentation and examples for safe review workflows.

**Non-Goals:**
- No automatic replacement approval.
- No new interactive editor integration.
- No strict mode for `recall --keys`; missing keys are reported without failing.
- No note title matching for touch.
- No automatic suppression or deletion of stale state records.

## Decisions

### Replacement preview uses editable drafts

`polaris remember --replace --dry-run` creates a replace draft under `.polaris/maintenance/` and prints a preview diff. It does not modify `.polaris/memories.jsonl` or `.polaris/replacement-history.jsonl`. `polaris replace apply <draft> --yes` parses the final text under `## Replacement Memory` and performs a normal replacement.

Alternatives considered:
- Print only a diff. This is safe but still requires rerunning a full replace command after review.
- Print a suggested replace command. This helps copy/paste but does not support editing or durable review artifacts.

### Replace draft apply follows normal replacement semantics

Applying a replace draft updates `updated_at`, increments `replacement_count`, preserves `created_at`, preserves lifecycle, and writes replacement history. This keeps `polaris diff --key` useful after apply.

### Multi-key recall is a selector, not a new output format

`polaris recall --keys goal,plan,state.branch` reuses the existing recall rendering. It is mutually exclusive with `--key` and `--prefix`, but still composes with `--exclude` and `--lifecycle`. Matching records are rendered in the input key order. Missing or filtered keys are reported in the output without causing a failure.

### Touch updates freshness metadata only

`polaris touch` updates `updated_at` for selected records and preserves text, lifecycle, `created_at`, `replacement_count`, and `replaced_from`. Exact `--key`, comma-separated `--keys`, and `--id` selectors do not require confirmation. Prefix selection requires `--yes`.

Alternatives considered:
- Put touch under `lifecycle touch`. The shorter top-level command better matches user intent and can apply to any lifecycle.
- Limit touch to state records. That would match the first use case but unnecessarily prevents refreshing valid log or archive records.

## Risks / Trade-offs

- Editable replace drafts can differ from the dry-run diff shown at creation time -> Apply remains explicit, and users can run `polaris diff --key <key>` after apply to inspect the final replacement.
- `recall --keys` missing-key success could hide typos -> The command reports missing keys in the output so agents can notice without losing useful recalled context.
- Touch can make stale records look fresh without content review -> The command is explicit and preserves content; prefix touch requires `--yes`.
- Draft parsing creates another maintenance file format -> Reuse the merge draft style and validate required metadata before apply.

## Migration Plan

No migration is required. Existing memories, replacement history, and maintenance drafts remain valid. New replace drafts are opt-in and live under `.polaris/maintenance/`.

## Open Questions

- None.
