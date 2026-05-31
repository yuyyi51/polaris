## Context

Polaris currently stores records in `.polaris/memories.jsonl` with generated ids, timestamps, a kind, optional title, and either inline text or a note path. `polaris remember` always appends a new inline record, so repeated updates to facts like "Goal" or "Current blocker" accumulate stale copies. The only cleanup command is `polaris clear --yes`, which is too broad when one remembered fact should be removed or replaced.

The change introduces user-meaningful keys for inline memory records. Keys become the stable identity agents use for current-state facts, while generated ids remain internal record metadata.

## Goals / Non-Goals

**Goals:**

- Require new inline memories to have a non-empty key.
- Prevent accidental duplicate active records for the same key.
- Support explicit replacement with `polaris remember --key <key> ... --replace`.
- Support individual deletion with `polaris forget <key>`.
- Keep existing unkeyed records readable after upgrade.
- Update bundled agent guidance so examples use the actual required CLI surface.

**Non-Goals:**

- Do not add a general query language, tags, or arbitrary key-value storage API.
- Do not implement single-note deletion or note keys in this change.
- Do not add encryption, sync, or multi-user coordination.
- Do not silently migrate or delete existing unkeyed records.

## Decisions

### Add `key` to inline memory records

`MemoryRecord` will gain an optional `key` field. New records created by `polaris remember` MUST set it. Legacy records without `key` remain valid for recall and status so existing `.polaris/` data does not break after upgrade.

Keys are exact, case-sensitive strings. The CLI rejects missing or empty keys, but it does not impose a broader naming grammar. The bundled skill should still recommend short stable keys such as `goal`, `plan`, `decision.storage`, `blocker`, or `verification`.

Alternative considered: repurpose `title` as the key. That would avoid a new field, but title is display-oriented and already optional; overloading it would make existing records ambiguous and make future human-readable titles harder to preserve.

### Duplicate keys fail unless replacement is explicit

`polaris remember --key <key> ...` will check active inline records for the same key. If one exists, the command fails with an error that tells the caller to use `--replace` when overwriting is intended.

`polaris remember --key <key> ... --replace` rewrites the active record set by removing the previous record for that key and writing the new inline record. The replacement receives a fresh generated id and timestamp because it is a new remembered fact, while the key is the durable identity users and agents rely on.

Alternative considered: make duplicate `remember` calls overwrite by default. That is convenient for state-like facts, but too easy to trigger accidentally during agent work. An explicit `--replace` flag makes the lossy operation visible.

### Implement forget as a keyed active-record rewrite

Add top-level `polaris forget <key>`. The command loads all records, removes the active inline record with the matching key, and rewrites `memories.jsonl`. If no keyed inline record matches, it exits with an error and leaves records unchanged.

The rewrite should use a temporary file in `.polaris/` followed by rename so an interrupted write is less likely to leave a truncated memory file. This same helper can serve `remember --replace`.

Alternative considered: append tombstone records to preserve an audit history. That keeps deletion history, but complicates recall, status counts, and legacy file inspection. Polaris is currently a small active-memory tool rather than an audit log.

### Recall should expose keys

Recall output should include the key for keyed inline records, so agents can see the identifier they would use with `forget` or `--replace`. Legacy unkeyed records should still render without a key marker.

### Notes stay separate

This change does not add keys to `note create` records. Note records remain created and recalled as they are today, and `clear --yes` still removes all note files. A later change can add `polaris note list` and `polaris note delete` if single-note lifecycle management becomes important.

## Risks / Trade-offs

- Existing scripts using `polaris remember --text ...` will break -> Mark the required `--key` as a breaking CLI change and update README, tests, and bundled skill examples together.
- Rewriting JSONL is more complex than append-only writes -> Keep the rewrite helper focused, test replacement and forget paths, and use temp-file-plus-rename.
- Legacy unkeyed records cannot be individually forgotten by key -> Preserve recall compatibility and rely on `clear --yes` for old stale data; avoid inventing id-based deletion in the same change.
- Fresh ids on replacement may surprise users who inspect JSONL directly -> Treat key as the public identity and document/output key rather than asking agents to depend on ids.
