## Context

Polaris currently stores workspace memory as JSONL records under `.polaris/memories.jsonl`. Inline memories can be addressed by key, but the CLI only supports full recall and single-key deletion. Agents that need to clean up or selectively restore context must scrape `polaris recall` output, which is brittle and wastes context as memory count grows.

The current storage model already has the fields needed for basic hygiene: `kind`, `key`, `title`, `text`, `path`, and `created_at`. This change should reuse that model and the existing shared/exclusive memory locks.

## Goals / Non-Goals

**Goals:**

- Provide a stable `polaris list` command that agents can use instead of parsing recall markdown.
- Let agents filter recall output by exact key, key prefix, and excluded key prefix.
- Let agents remove multiple keyed inline memories in one command.
- Let agents remove keyed inline memories by prefix only when they explicitly confirm the broad deletion.
- Keep the current storage format and existing command behavior compatible.

**Non-Goals:**

- No lifecycle categories such as `durable`, `state`, `log`, or `archive`.
- No rename, merge, prune, or compact suggestion workflow.
- No semantic similarity, OpenSpec/git-aware stale detection, or LLM-generated summaries.
- No deletion of note records by key prefix, because note records do not currently have keys.

## Decisions

1. Use one shared memory filtering helper for list, recall, and forget.

   The helper should load all records once, then apply optional exact-key, prefix, and exclude-prefix predicates to keyed inline memories. Note records should remain visible for unfiltered list/recall, but key-based filters should match only records with keys. This keeps filtering predictable and avoids inventing note addressing in this change.

2. Add `polaris list --keys` and `polaris list --json`.

   `--keys` should print one key per line for keyed inline memories. `--json` should print a stable array of memory summary objects containing existing metadata such as `id`, `kind`, `key`, `title`, `path`, and `created_at`, without adding a storage migration. Text bodies should be omitted from list summaries so the command stays useful for inspection without dumping all context.

3. Extend `polaris recall` with additive filter flags.

   Supported filters should be `--key <key>`, `--prefix <prefix>`, and repeatable `--exclude <prefix>`. `--key` and `--prefix` should conflict with each other because one selects a single key while the other selects a key family. `--exclude` can combine with no selector or with `--prefix`. If filters match no records, recall should exit successfully with an explicit no-matching-memory message.

4. Extend `polaris forget` conservatively.

   Positional keys should become variadic: `polaris forget key1 key2 key3`. Prefix deletion should use `polaris forget --prefix <prefix> --yes`, with `--prefix` conflicting with positional keys. Requiring `--yes` for prefix deletion mirrors the existing `clear --yes` safety pattern and prevents accidental broad deletion.

5. Preserve concurrency guarantees by keeping exclusive locks around mutations.

   Batch and prefix forget should perform a single locked load/filter/write cycle, rather than calling single-key forget repeatedly. This prevents partial cleanup if one of several keys is missing and reduces whole-file rewrites.

## Risks / Trade-offs

- Prefix deletion can remove more memory than intended -> require `--yes`, print how many records were removed, and reject empty prefixes.
- `list --json` could become an accidental full-memory dump -> return summaries without inline `text`.
- Filtering notes is ambiguous because notes have no keys -> leave notes out of key-filtered recall/list results and document that filters target keyed inline memory.
- Multi-key forget error semantics can surprise users -> reject the whole command if any requested key is missing, leaving memory unchanged, so cleanup is all-or-nothing.
