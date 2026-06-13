# Polaris Memory Edit Review Examples

These examples show safe, explicit maintenance flows for lifecycle cleanup and replacement review.

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
