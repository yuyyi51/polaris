# Polaris Memory Edit Review Examples

These examples show safe, explicit maintenance flows for lifecycle cleanup, replacement review, and freshness updates.

## Reclassify Existing Inline Memory

Use an exact key for one record:

```bash
polaris lifecycle move --key state.branch --to state
```

Use a confirmed prefix migration when old state-like keys were recorded as durable:

```bash
polaris lifecycle move --prefix state. --from durable --to state --yes
```

Use JSON output when an agent needs to inspect moved and skipped records:

```bash
polaris lifecycle move --prefix state. --to state --yes --json
```

## Reclassify Existing Notes

Notes do not have keys. Use the note record id from `polaris list --json` for one note:

```bash
polaris list --json --lifecycle log
polaris lifecycle move --id 0123456789abcdef0123456789abcdef --to archive
```

Use `--kind note` for confirmed batch note migration:

```bash
polaris lifecycle move --kind note --from log --to archive --yes
```

Polaris does not support note title matching for lifecycle moves. Titles are human-facing labels, not stable selectors.

## Review Replacement Diffs

After replacing important keyed inline memory, inspect the latest replacement diff:

```bash
polaris remember --key goal --replace --text "Updated task goal." --lifecycle durable
polaris diff --key goal
```

Use JSON when an agent needs a stable payload:

```bash
polaris diff --key goal --json
```

If Polaris reports that no replacement snapshot is available, inspect the current record directly:

```bash
polaris recall --key goal
```

That can happen for memory replaced before replacement history was introduced or for records that have never been replaced.

## Preview Replacement Before Writing

Use a dry-run replacement when the new text should be reviewed or edited before it touches active memory:

```bash
polaris remember --key goal --replace --dry-run --text "Updated task goal."
```

The command prints a preview diff and returns a draft path under `.polaris/maintenance/`. Edit only the text under `## Replacement Memory`, then apply the reviewed draft explicitly:

```bash
polaris replace apply .polaris/maintenance/replace-goal-<id>.md --yes
polaris diff --key goal
```

Dry-run replacement does not mutate active memory and does not write replacement history. Applying the draft uses normal replacement behavior, so `diff --key` can compare the previous memory with the applied text.

## Recall Several Exact Keys

Use multi-key recall when the next step needs a precise set of records in a stable order:

```bash
polaris recall --keys goal,plan,state.branch
polaris recall --keys goal,plan,state.branch --exclude state.
polaris recall --keys goal,decision.storage --lifecycle durable
```

Missing or filtered keys are reported in the output while matching keys are still recalled.

## Refresh Still-Valid State

Use touch when a state or log record is still accurate but stale reminders are no longer useful:

```bash
polaris touch --key state.branch
polaris touch --keys state.branch,state.queue
polaris touch --id 0123456789abcdef0123456789abcdef
```

Use confirmed prefix touch for a batch of related keyed inline records:

```bash
polaris touch --prefix state. --yes
```

Use JSON output when an agent needs to inspect touched records and missing keys:

```bash
polaris touch --keys state.branch,missing --json
```

Touch updates only `updated_at`. It preserves memory text, lifecycle, creation time, replacement metadata, and note file contents.
