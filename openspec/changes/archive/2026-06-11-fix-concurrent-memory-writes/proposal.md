## Why

Concurrent `polaris remember --replace` and other memory writes can corrupt `.polaris/memories.jsonl` or lose updates because memory operations currently run without cross-process coordination. Once the JSONL file contains two records on one line, later `polaris recall`, `polaris status --json`, and new writes fail while parsing the shared memory file.

## What Changes

- Serialize memory-file mutations across Polaris processes so concurrent appends, replacements, note records, forgets, and clears do not interleave or overwrite each other.
- Preserve JSONL record boundaries by ensuring every stored memory record is separated by a newline, including when appending after a file that does not already end with `\n`.
- Keep memory readers from observing partially written records while another process is mutating the memory file.
- Improve JSONL parse errors with the memory file path and physical line number.
- Keep blank lines non-fatal when reading `.polaris/memories.jsonl`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `polaris-recall`: memory recording, replacement, status, and recall must remain reliable under concurrent access and report malformed JSONL with actionable location context.

## Impact

- Affects `src/storage.rs` memory persistence and JSONL parsing.
- Adds regression coverage in `tests/cli.rs` for concurrent memory writes and malformed JSONL diagnostics.
- May add a small cross-platform file-locking dependency if the Rust standard library lock APIs are not sufficient for the supported toolchain and platforms.
