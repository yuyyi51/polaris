## Context

Polaris stores active memory records in `.polaris/memories.jsonl`. Keyed inline records have stable keys and text bodies; note records have ids, titles, and paths but no keys. Lifecycle metadata already exists for both record kinds, and replacement metadata already tracks `updated_at`, `replacement_count`, and `replaced_from` for keyed inline replacements.

The remaining gap is edit review. Agents can see that a record was replaced, but cannot inspect what changed. Agents can also create lifecycle metadata on new records, but cannot reclassify existing records without rewriting the whole memory through `remember --replace` or recreating notes.

## Goals / Non-Goals

**Goals:**
- Allow explicit lifecycle reclassification for inline and note records without rewriting memory text or note files.
- Keep batch lifecycle moves confirmed and inspectable.
- Preserve previous inline record snapshots when `remember --replace` supersedes a key.
- Add a diff command that compares the current active keyed inline memory with the latest replacement snapshot.
- Keep outputs stable enough for agents by supporting JSON for lifecycle moves and diffs.
- Update user-facing and agent-facing documentation and examples where the new workflow should be discoverable.

**Non-Goals:**
- No automatic pruning, compaction, merging, or lifecycle classification.
- No note body diffing.
- No note title matching for lifecycle moves.
- No full replacement history browser beyond the latest replacement diff.
- No migration that rewrites existing memories to synthesize missing historical snapshots.

## Decisions

### Lifecycle move uses stable selectors by record type

Inline records can be selected by `--key` or `--prefix`. Note records can be selected by `--id`, or included in broad moves with `--kind note`. The command does not support note title matching because titles are human-facing labels, not stable identifiers.

Alternatives considered:
- Use `--title-prefix` for notes. This is convenient but risks changing unrelated notes with similar titles.
- Require note keys. This would force a larger data model change and is not needed for lifecycle maintenance.

### Batch moves require confirmation

Single-record moves selected by `--key` or `--id` do not require `--yes`. Batch moves selected by `--prefix` or `--kind` require `--yes`. This matches the existing destructive-command style while recognizing that lifecycle moves are metadata edits rather than deletion.

### Lifecycle move changes only lifecycle metadata

The command updates the matched record's lifecycle and `updated_at`. It preserves `created_at`, text, note path, replacement metadata, and note file contents. Records that already have the target lifecycle are reported as skipped rather than rewritten.

### Replacement snapshots live outside active memory

When `remember --replace` supersedes a keyed inline record, Polaris writes the previous active record to `.polaris/replacement-history.jsonl` before writing the replacement to `.polaris/memories.jsonl`. The active replacement continues to store `replaced_from`, making the latest snapshot discoverable without bloating the active memory record.

Alternatives considered:
- Embed previous text in the active memory record. This keeps lookup simple but makes active recall/list data heavier over time.
- Store only a generated diff. This loses the ability to render different diff formats later and makes audit data harder to trust.

### Diff is latest-replacement only

`polaris diff --key <key>` compares the active keyed inline record with the snapshot referenced by `replaced_from`. If no snapshot exists, the command reports that no replacement snapshot is available. This keeps the first version focused while leaving room for a later `history` command.

## Risks / Trade-offs

- Existing replaced records have `replaced_from` but no saved snapshot -> `diff --key` must report no snapshot rather than failing.
- A separate history JSONL file can grow over time -> acceptable for audit data; future pruning can be proposed separately.
- Lifecycle moves are non-destructive but can still harm recall quality -> batch selectors require `--yes`, and JSON output lists moved and skipped records.
- Replacement and lifecycle move both rewrite active memory data -> reuse existing lock discipline so readers see consistent JSONL state.

## Migration Plan

No active migration is required. Existing memory records remain valid. Existing replacement metadata remains visible, but `polaris diff --key` only works after a replacement snapshot has been captured by a newer `remember --replace` operation.

Rollback is straightforward: active memories continue to live in `.polaris/memories.jsonl`; if the new command surface is removed later, `.polaris/replacement-history.jsonl` can be ignored without affecting recall.

## Open Questions

- None.
