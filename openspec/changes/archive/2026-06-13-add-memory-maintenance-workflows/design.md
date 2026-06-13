## Context

Phase 1 gives agents stable ways to inspect, filter, and delete memory. Phase 2 adds lifecycle and update metadata. This phase builds on those primitives to reduce manual memory maintenance: key renames, consolidation of several records into one durable record, and explainable cleanup suggestions.

Polaris should remain a local deterministic CLI. It should not call an LLM or attempt opaque semantic compression. When summarization is needed, Polaris should prepare an editable draft that an agent or user can revise before applying.

## Goals / Non-Goals

**Goals:**

- Rename keyed inline memories without changing text content.
- Generate merge drafts from multiple keyed inline memories.
- Apply merge drafts explicitly to a target key and optionally remove sources.
- Suggest prune and compact candidates using deterministic, explainable rules.
- Keep destructive actions confirmed and all-or-nothing.

**Non-Goals:**

- No automatic semantic summarization.
- No automatic deletion from suggest commands.
- No remote service, daemon, or external database.
- No editing note file contents as part of merge.

## Decisions

1. Make rename a direct keyed inline operation.

   `polaris rename old-key new-key` should preserve text and metadata while changing the key. It should reject missing source keys and reject destination keys that already exist unless an explicit replacement flag is later designed. This avoids accidental overwrites.

2. Use draft files for merge.

   `polaris merge --into target keyA keyB` should create a markdown draft under `.polaris/maintenance/` containing source keys, metadata, and text in a deterministic format. Draft creation should not modify active memory.

3. Separate merge draft creation from apply.

   `polaris merge apply <draft> --yes` should read the edited draft, write or replace the target memory, and optionally remove source memories when explicitly requested. Keeping apply separate gives agents a safe point to edit the merged content.

4. Keep suggestions deterministic.

   `prune --suggest` can flag old `state` or `log` records, archived records that are no longer normally recalled, duplicate exact text, and records superseded by replacement metadata. `compact --suggest` can flag clusters with shared key prefixes and related lifecycle values. Each suggestion should include reasons and candidate commands.

5. Output suggestions as human text by default and JSON on request.

   A future agent can consume `--json`, while humans can scan the default output. Suggest commands must not mutate `.polaris/memories.jsonl`.

## Risks / Trade-offs

- Merge drafts might be mistaken for applied memory -> print clear draft paths and require explicit apply.
- Suggestion quality may be limited without semantic analysis -> explain rules and keep commands conservative.
- Rename can affect scripts that expect old keys -> reject overwrites and document that key changes are intentional maintenance operations.
- Draft files add another directory under `.polaris/` -> keep them under `.polaris/maintenance/` so they are easy to inspect and clear.
