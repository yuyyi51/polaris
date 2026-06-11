## Context

Polaris stores workspace memory in `.polaris/memories.jsonl`, with one JSON object per line. Multiple CLI processes can operate on the same workspace at the same time when agents issue parallel `polaris remember` commands.

The current storage code appends records directly and rewrites the entire JSONL file for replacement and deletion without any cross-process coordination. That creates two classes of failure:

- concurrent appends can leave adjacent JSON objects on the same physical line if write boundaries are not preserved;
- concurrent read-modify-write operations can lose updates when one process renames a stale snapshot over another process's completed write.

## Goals / Non-Goals

**Goals:**

- Make `.polaris/memories.jsonl` reads and writes safe across concurrent Polaris processes in the same workspace.
- Preserve JSONL line boundaries for every memory record, even when appending after a malformed or manually edited file that lacks a final newline.
- Avoid readers observing a file while a writer is appending or replacing it.
- Report malformed memory records with the file path and physical line number.
- Keep blank lines harmless while reading memories.

**Non-Goals:**

- Repair already-corrupted JSONL automatically.
- Change the public CLI shape or JSON memory record schema.
- Introduce a daemon, database, or workspace-global service.
- Coordinate with non-Polaris processes that edit `.polaris/memories.jsonl` without taking the Polaris lock.

## Decisions

### Use a separate lock file

Use a stable lock path under `.polaris/`, such as `.polaris/memories.lock`, for all operations that read or write `memories.jsonl`.

Rationale: locking `memories.jsonl` itself is fragile because replace operations write a temp file and `rename` it over the original. A lock held on the old file inode would not protect the new file. A dedicated lock file remains stable across replacements.

Alternative considered: rely on `OpenOptions::append(true)` and write one serialized line at a time. This does not protect read-modify-write replacement from lost updates and still leaves the record/newline boundary dependent on lower-level write behavior.

### Lock the full logical operation

Take an exclusive lock for mutating operations: `remember`, `forget`, `note create`, and `clear`. The lock must cover the full read/check/write sequence, not just the final file write.

Take a shared lock for read-only operations that load memories, including `recall`, `status --json` memory counts, and hook paths that inline recall.

Rationale: replace and forget are read-modify-write operations. Locking only the write step still allows stale snapshots to overwrite newer data.

### Normalize append boundaries inside the lock

Before appending a serialized record, inspect `memories.jsonl` while holding the exclusive lock. If the file is non-empty and does not end with `\n`, write a separator newline before writing the new `record\n`.

Rationale: this preserves JSONL boundaries for new writes and makes append robust after manual edits or older corrupted final-line states. It does not attempt to split two already-concatenated JSON objects.

### Add parse context at the line level

Keep skipping whitespace-only lines, but wrap JSON parse failures with `.polaris/memories.jsonl:<line>` context.

Rationale: the current serde error reports only the position within the line, which is not enough to locate the bad record in the file.

## Risks / Trade-offs

- Cross-platform lock behavior may require a dependency or careful use of standard-library APIs. Mitigation: prefer the available Rust standard library lock APIs on the supported toolchain; use a small focused dependency only if needed.
- Serializing writes reduces parallel write throughput. Mitigation: Polaris memory writes are small and workspace-local, so correctness is more important than concurrent write performance.
- Shared reader locks can block briefly during writes. Mitigation: write operations are short, and preventing readers from seeing partial state avoids cascading failures.
- Already-corrupted files will still need manual repair. Mitigation: improved diagnostics identify the exact path and line to edit.
