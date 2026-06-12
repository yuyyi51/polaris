## Context

Phase 1 adds stable listing, filtered recall, and safer deletion primitives. The next problem is that memory records still have no first-class distinction between long-lived knowledge, volatile workspace state, short logs, and archived history. Agents can infer intent from key names like `state.branch` or `decision.storage`, but that convention is not machine-readable enough for status summaries, lifecycle-specific recall, or later maintenance suggestions.

Current JSONL records already include `created_at`; replacements create a new record and discard old metadata. This change adds optional metadata fields while keeping legacy records readable.

## Goals / Non-Goals

**Goals:**

- Add lifecycle metadata for inline memories and note records.
- Provide lifecycle-aware list and recall filtering.
- Track simple replacement metadata for keyed inline memory updates.
- Extend status JSON with lifecycle counts and stale volatile-memory hints.
- Keep existing records compatible without requiring a migration command.

**Non-Goals:**

- No rename, merge, prune, or compact suggestion workflow.
- No full historical audit log or old text retention.
- No automatic lifecycle inference from content.
- No semantic freshness checks against git or OpenSpec state.

## Decisions

1. Use `lifecycle` as the field name.

   `kind` already means storage representation (`inline` or `note`), so overloading it for durable/state/log/archive would be confusing. `lifecycle` describes intended retention and freshness behavior. Supported values should be `durable`, `state`, `log`, and `archive`.

2. Default missing lifecycle to `durable`.

   Existing records should load and behave as durable memory unless replaced with another lifecycle. This keeps recall behavior stable and avoids a migration step.

3. Allow lifecycle on both inline memories and notes.

   Notes may hold durable architecture context or archived handoff details. The storage model should accept lifecycle metadata for note records even though note text is stored separately.

4. Track replacement metadata on the active record only.

   Replacing a keyed inline memory should preserve the original `created_at` when possible, set `updated_at` to the replacement time, increment `replacement_count`, and optionally record `replaced_from` as the prior active record id. It should not keep old text content or create an audit log in this phase.

5. Make lifecycle filtering additive to Phase 1 filters.

   `polaris list --json --lifecycle state` and `polaris recall --lifecycle state` should compose with existing key/prefix/exclude filters where relevant. This keeps one filter model rather than separate durable/state commands.

6. Surface volatile-memory hints in status JSON.

   `polaris status --json` should include lifecycle counts and a simple list/count of `state` or `log` records that may need review after a conservative age threshold. The hint should not change exit status.

## Risks / Trade-offs

- Users may disagree with lifecycle defaults -> defaulting legacy records to `durable` preserves current behavior and lets users opt into other lifecycles later.
- Replacement metadata could imply a full audit trail -> document that only active-record metadata is retained.
- Stale hints could be noisy -> keep them machine-readable and advisory, with conservative thresholds.
- Adding fields to JSONL could break external ad hoc parsers -> serde-compatible optional fields preserve Polaris compatibility; documented `list --json` should be the stable integration point.
