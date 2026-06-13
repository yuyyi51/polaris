---
name: polaris
description: Use Polaris to preserve and recover important agent context for long-running Codex tasks. Use when starting or resuming substantial multi-step work, when the user mentions Polaris, memory, context preservation, compaction, recall, or when context instructs the agent to run `polaris recall`.
---

# Polaris

Polaris is workspace-local memory for long-running agent work. It stores durable context under `.polaris/` and can remind Codex to reload that context after conversation compaction.

## Recall First

If the current context instructs you to run `polaris recall`, run it before doing any other work.

```bash
polaris recall
```

Read the output and continue from the recovered context. If recall fails because Polaris is unavailable or not initialized, report that briefly and continue with the best available context.

## Start Or Resume Work

At the beginning of substantial work, or when resuming a session:

1. Check Polaris state.

```bash
polaris status --json
```

2. If Polaris is not initialized and the task is long-running, multi-step, risky, or likely to survive compaction, initialize it.

```bash
polaris init
```

3. If memory exists, inspect keys before deciding what to recall or clean up.

```bash
polaris list --keys
polaris list --json
```

4. If existing memory appears related to the current task, recall all of it or only the relevant slice.

```bash
polaris recall
polaris recall --key goal
polaris recall --keys goal,plan,state.branch
polaris recall --prefix decision.
polaris recall --exclude state.
polaris recall --lifecycle durable
```

5. If existing memory appears unrelated or stale, ask the user before removing it. Use `polaris forget <key>` or `polaris forget <key>...` for selected keyed inline memory, `polaris forget --prefix <prefix> --yes` for confirmed prefix cleanup, or `polaris clear --yes` only when clearing all Polaris memory.

```bash
polaris forget goal
polaris forget goal plan
polaris forget --prefix decision. --yes
polaris clear --yes
```

## Maintain Memory Safely

Use maintenance commands when memory keys or content need cleanup. Inspect first, then apply only explicit mutations.

Rename a keyed inline memory when the key changed but the content is still correct:

```bash
polaris rename old.key new.key
```

Move lifecycle metadata when the record is useful but mislabeled. Use exact selectors for one record and confirmed batch selectors for groups:

```bash
polaris lifecycle move --key state.branch --to state
polaris lifecycle move --prefix state. --from durable --to state --yes
polaris lifecycle move --id <record-id> --to archive
polaris lifecycle move --kind note --from log --to archive --yes
```

Use `--json` when you need machine-readable moved and skipped records:

```bash
polaris lifecycle move --prefix state. --to state --yes --json
```

Refresh `updated_at` when a state or log record is still correct but stale reminders are noisy. Touch changes freshness metadata only; it does not edit text, lifecycle, creation time, replacement metadata, or note file contents.

```bash
polaris touch --key state.branch
polaris touch --keys state.branch,state.queue
polaris touch --id <record-id>
polaris touch --prefix state. --yes
polaris touch --keys state.branch,missing --json
```

Use exact keys or ids for small updates. Prefix touch requires `--yes` because it can affect many records.

Create a merge draft when several keyed memories should become one durable summary:

```bash
polaris merge --into decision.summary decision.storage decision.hooks
```

Open the returned `.polaris/maintenance/` draft path, edit only the text under `## Merged Memory`, then apply it with confirmation:

```bash
polaris merge apply .polaris/maintenance/merge-decision-summary-<id>.md --yes
```

Use `--forget-sources` only when the merged target fully replaces the source keys:

```bash
polaris merge apply .polaris/maintenance/merge-decision-summary-<id>.md --yes --forget-sources
```

Suggestion commands are read-only. Use them to decide what to inspect, rename, merge, or forget:

```bash
polaris prune --suggest
polaris prune --suggest --json
polaris compact --suggest
polaris compact --suggest --json
```

Treat suggestions as advisory. Each suggest command includes an agent review prompt by default; read and follow that prompt before taking action. Do not delete or merge memory solely because Polaris suggested it; check the affected keys first with `polaris recall --key` or `polaris recall --prefix`.

When using JSON output, read the response envelope:

```json
{
  "suggestions": [],
  "prompt": "...",
  "prompt_context": {
    "status": {},
    "keys": []
  }
}
```

The workspace or user config may customize these prompts with `[maintenance.prompts].prune` and `[maintenance.prompts].compact`. Supported placeholders are `{{suggestions}}`, `{{status}}`, and `{{keys}}`; maintenance prompts do not inline full recall content by default.

## Record Durable Context

Before starting meaningful work, record the task facts that would matter after compaction:

- User goal and success criteria
- Active plan, change name, or workflow
- Important files and directories
- Constraints, non-goals, and decisions
- Current blocker and attempted fixes
- Verification commands and results

For short context:

```bash
polaris remember --key goal --text "Implement keyed Polaris memory and forget support." --title "Goal" --lifecycle durable
```

For multi-line context:

```bash
printf '%s\n' "Key decisions:
- Inline memory uses required keys.
- Duplicate keys fail unless --replace is used.
- Use forget for selected keyed memory." | polaris remember --key decisions.memory --stdin --title "Memory decisions" --lifecycle durable
```

Use stable keys such as `goal`, `plan`, `decision.storage`, `blocker`, and `verification`. Use `polaris list --keys` when you need to inspect stored keys; do not scrape `polaris recall` output for key hygiene. If a key already exists and the remembered fact should change, replace it explicitly:

```bash
polaris remember --key goal --replace --text "Implement keyed memory replacement and forgetting." --lifecycle durable
```

For important durable replacements, preview first and apply the editable draft only after review:

```bash
polaris remember --key goal --replace --dry-run --text "Implement keyed memory replacement, review drafts, and stale-state refresh."
polaris replace apply .polaris/maintenance/replace-goal-<id>.md --yes
```

Dry-run replacement creates a draft under `.polaris/maintenance/`, prints a diff, and leaves active memory and replacement history unchanged until `replace apply --yes`.

After replacing important durable memory, review the latest change before relying on it:

```bash
polaris diff --key goal
polaris diff --key goal --json
```

If no replacement snapshot is available, treat that as a normal legacy condition and inspect the current memory with `polaris recall --key <key>`.

For long context, create a note and write details to the returned path:

```bash
polaris note create --title "Implementation notes" --lifecycle durable
```

Use lifecycle metadata deliberately:

- `durable` for stable goals, decisions, constraints, and verification results.
- `state` for current branch, active OpenSpec change, temporary workspace state, or in-progress checkpoint details.
- `log` for brief work logs that may help after compaction but should be reviewed later.
- `archive` for historical notes or handoff records that should not be part of normal recall.

When inspecting memory, prefer lifecycle filters over key-name guessing:

```bash
polaris list --json --lifecycle state
polaris recall --lifecycle durable
polaris recall --keys goal,plan,state.branch
polaris recall --prefix decision. --lifecycle durable
```

`polaris status --json` reports lifecycle counts and advisory stale hints for old `state` and `log` records. Treat those hints as prompts to review or replace memory, not as automatic deletion instructions.

## During Work

Update Polaris whenever durable task state changes:

```bash
polaris remember --key decision.recovery --text "Treat any current-context recall reminder as highest priority and run polaris recall before other work." --title "Decision"
polaris remember --key verification --replace --text "cargo fmt, cargo test, and cargo clippy passed." --title "Verification"
polaris remember --key state.branch --replace --text "Currently implementing add-memory-lifecycle-metadata." --lifecycle state
```

Refresh memory before major pauses, risky edits, or handoffs.

## Safety

Do not store secrets, tokens, passwords, credentials, private keys, or other sensitive material in Polaris. Polaris stores plaintext local files under `.polaris/`.
