use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tempfile::TempDir;

fn polaris() -> Command {
    Command::cargo_bin("polaris").expect("binary exists")
}

fn temp_workspace() -> TempDir {
    tempfile::tempdir().expect("temp workspace")
}

fn init_workspace(dir: &Path) {
    polaris().current_dir(dir).arg("init").assert().success();
}

fn run_parallel_remember_commands(dir: &Path, commands: Vec<Vec<String>>) {
    let binary = assert_cmd::cargo::cargo_bin("polaris");
    let mut children = commands
        .into_iter()
        .map(|args| {
            StdCommand::new(&binary)
                .current_dir(dir)
                .args(args)
                .spawn()
                .expect("spawn polaris")
        })
        .collect::<Vec<_>>();

    for child in children.drain(..) {
        let output = child.wait_with_output().expect("wait for polaris");
        assert!(
            output.status.success(),
            "polaris command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn assert_jsonl_records(path: &Path, expected_count: usize) -> Vec<Value> {
    let memories = fs::read_to_string(path).expect("read memories");
    let mut records = Vec::new();
    for (index, line) in memories.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let record: Value = serde_json::from_str(line)
            .unwrap_or_else(|error| panic!("line {} should be valid JSON: {error}", index + 1));
        records.push(record);
    }
    assert_eq!(records.len(), expected_count);
    records
}

fn hook_context(output: &[u8]) -> String {
    let hook: Value = serde_json::from_slice(output).expect("hook json");
    hook["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("additional context")
        .to_string()
}

#[test]
fn init_creates_workspace_memory_idempotently() {
    let dir = temp_workspace();

    polaris()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    assert!(dir.path().join(".polaris/state.json").is_file());
    assert!(dir.path().join(".polaris/memories.jsonl").is_file());
    assert!(dir.path().join(".polaris/docs").is_dir());

    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "goal",
            "--text",
            "survives reinit",
            "--title",
            "Goal",
        ])
        .assert()
        .success();

    polaris()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success();

    polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .stdout(predicate::str::contains("survives reinit"));
}

#[test]
fn status_reports_uninitialized_and_initialized_counts() {
    let dir = temp_workspace();

    let output = polaris()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status: Value = serde_json::from_slice(&output).expect("status json");
    assert_eq!(status["initialized"], false);

    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "goal",
            "--text",
            "keep the goal",
            "--title",
            "Goal",
        ])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["note", "create", "--title", "Architecture notes"])
        .assert()
        .success();

    let output = polaris()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status: Value = serde_json::from_slice(&output).expect("status json");
    assert_eq!(status["initialized"], true);
    assert_eq!(status["memory_count"], 2);
    assert_eq!(status["document_count"], 1);
    assert!(status["root"].as_str().unwrap().ends_with(".polaris"));
}

#[test]
fn remember_rejects_before_init_and_records_text_and_stdin() {
    let dir = temp_workspace();

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "nope"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("polaris init"));

    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "goal",
            "--text",
            "task goal",
            "--title",
            "Goal",
        ])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args([
            "remember", "--key", "decision", "--stdin", "--title", "Decision",
        ])
        .write_stdin("use SessionStart compact\n")
        .assert()
        .success();

    let memories = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();
    assert!(memories.contains("\"key\":\"goal\""));
    assert!(memories.contains("\"key\":\"decision\""));
    assert!(memories.contains("task goal"));
    assert!(memories.contains("use SessionStart compact"));
}

#[test]
fn remember_requires_key_and_rejects_duplicate_without_replace() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--text", "missing key"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--key"));

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "first goal"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "second goal"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"))
        .stderr(predicate::str::contains("--replace"));

    let recall = polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let recall = String::from_utf8(recall).unwrap();
    assert!(recall.contains("first goal"));
    assert!(!recall.contains("second goal"));
}

#[test]
fn remember_replace_overwrites_keyed_memory() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "first goal"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "goal",
            "--replace",
            "--text",
            "replacement goal",
        ])
        .assert()
        .success();

    let recall = polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let recall = String::from_utf8(recall).unwrap();
    assert!(recall.contains("Key: goal"));
    assert!(recall.contains("replacement goal"));
    assert!(!recall.contains("first goal"));

    let memories = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();
    assert_eq!(memories.matches("\"key\":\"goal\"").count(), 1);
}

#[test]
fn concurrent_remember_records_distinct_keys_as_valid_jsonl() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    let commands = (0..64)
        .map(|index| {
            vec![
                "remember".to_string(),
                "--key".to_string(),
                format!("k{index}"),
                "--text".to_string(),
                format!("memory {index} {}", "x".repeat(512)),
            ]
        })
        .collect();
    run_parallel_remember_commands(dir.path(), commands);

    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 64);
    for index in 0..64 {
        assert!(
            records
                .iter()
                .any(|record| record["key"] == format!("k{index}")),
            "missing key k{index}"
        );
    }

    polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .stdout(predicate::str::contains("memory 63"));
    polaris()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"memory_count\": 64"));
}

#[test]
fn concurrent_replace_preserves_unrelated_replacements() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    for index in 0..80 {
        polaris()
            .current_dir(dir.path())
            .args([
                "remember",
                "--key",
                &format!("k{index}"),
                "--text",
                &format!("initial {index}"),
            ])
            .assert()
            .success();
    }

    let commands = (0..80)
        .map(|index| {
            vec![
                "remember".to_string(),
                "--key".to_string(),
                format!("k{index}"),
                "--replace".to_string(),
                "--text".to_string(),
                format!("replacement {index} {}", "y".repeat(512)),
            ]
        })
        .collect();
    run_parallel_remember_commands(dir.path(), commands);

    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 80);
    for index in 0..80 {
        let record = records
            .iter()
            .find(|record| record["key"] == format!("k{index}"))
            .unwrap_or_else(|| panic!("missing key k{index}"));
        assert!(
            record["text"]
                .as_str()
                .expect("record text")
                .starts_with(&format!("replacement {index}")),
            "key k{index} was not replaced: {record:?}"
        );
    }
}

#[test]
fn remember_separates_append_after_missing_final_newline() {
    let dir = temp_workspace();
    init_workspace(dir.path());
    fs::write(
        dir.path().join(".polaris/memories.jsonl"),
        r#"{"id":"first","created_at":"2026-05-31T00:00:00Z","kind":"inline","key":"first","text":"first memory"}"#,
    )
    .unwrap();

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "second", "--text", "second memory"])
        .assert()
        .success();

    assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 2);
    polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .stdout(predicate::str::contains("first memory"))
        .stdout(predicate::str::contains("second memory"));
}

#[test]
fn recall_and_status_ignore_blank_memory_lines() {
    let dir = temp_workspace();
    init_workspace(dir.path());
    fs::write(
        dir.path().join(".polaris/memories.jsonl"),
        "\n  \n{\"id\":\"first\",\"created_at\":\"2026-05-31T00:00:00Z\",\"kind\":\"inline\",\"key\":\"first\",\"text\":\"first memory\"}\n\n",
    )
    .unwrap();

    polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .stdout(predicate::str::contains("first memory"));
    polaris()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"memory_count\": 1"));
}

#[test]
fn malformed_memory_jsonl_errors_include_path_and_line() {
    let dir = temp_workspace();
    init_workspace(dir.path());
    fs::write(
        dir.path().join(".polaris/memories.jsonl"),
        "{\"id\":\"first\",\"created_at\":\"2026-05-31T00:00:00Z\",\"kind\":\"inline\",\"key\":\"first\",\"text\":\"first memory\"}\n{\"id\":\"bad1\",\"created_at\":\"2026-05-31T00:00:00Z\",\"kind\":\"inline\",\"key\":\"bad1\",\"text\":\"bad memory\"}{\"id\":\"bad2\",\"created_at\":\"2026-05-31T00:00:00Z\",\"kind\":\"inline\",\"key\":\"bad2\",\"text\":\"bad memory\"}\n",
    )
    .unwrap();

    polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .failure()
        .stderr(predicate::str::contains(".polaris/memories.jsonl:2"))
        .stderr(predicate::str::contains("trailing characters"));

    polaris()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(".polaris/memories.jsonl:2"))
        .stderr(predicate::str::contains("trailing characters"));
}

#[test]
fn forget_removes_keyed_memory_and_preserves_unrelated_records() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "forget me"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "plan", "--text", "keep me"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["note", "create", "--title", "Architecture notes"])
        .assert()
        .success();

    polaris()
        .current_dir(dir.path())
        .args(["forget", "goal"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Forgot memory goal"));

    let recall = polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let recall = String::from_utf8(recall).unwrap();
    assert!(!recall.contains("forget me"));
    assert!(recall.contains("keep me"));
    assert!(recall.contains("Architecture notes"));
}

#[test]
fn forget_rejects_unknown_key_and_missing_workspace() {
    let dir = temp_workspace();

    polaris()
        .current_dir(dir.path())
        .args(["forget", "goal"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("polaris init"));

    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "keep me"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["forget", "missing"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No memory exists for key `missing`",
        ));

    polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .stdout(predicate::str::contains("keep me"));
}

#[test]
fn list_outputs_keys_and_json_summaries_without_inline_text() {
    let dir = temp_workspace();

    polaris()
        .current_dir(dir.path())
        .args(["list", "--keys"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("polaris init"));

    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "goal",
            "--text",
            "private goal text",
            "--title",
            "Goal",
        ])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "plan", "--text", "private plan text"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["note", "create", "--title", "Architecture notes"])
        .assert()
        .success();

    polaris()
        .current_dir(dir.path())
        .args(["list", "--keys"])
        .assert()
        .success()
        .stdout(predicate::eq("goal\nplan\n"));

    let output = polaris()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let summaries: Value = serde_json::from_slice(&output).expect("list json");
    let summaries = summaries.as_array().expect("summary array");
    assert_eq!(summaries.len(), 3);
    assert!(summaries.iter().any(|record| record["key"] == "goal"));
    assert!(summaries.iter().any(|record| record["key"] == "plan"));
    assert!(
        summaries
            .iter()
            .any(|record| record["kind"] == "note" && record["title"] == "Architecture notes")
    );
    assert!(summaries.iter().all(|record| record.get("text").is_none()));
}

#[test]
fn recall_filters_by_key_prefix_and_excluded_prefix() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    for (key, text) in [
        ("goal", "goal memory"),
        ("decision.current", "current decision"),
        ("decision.old.storage", "old decision"),
        ("state.branch", "state memory"),
    ] {
        polaris()
            .current_dir(dir.path())
            .args(["remember", "--key", key, "--text", text])
            .assert()
            .success();
    }

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--key", "goal"])
        .assert()
        .success()
        .stdout(predicate::str::contains("goal memory"))
        .stdout(predicate::str::contains("current decision").not());

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--prefix", "decision."])
        .assert()
        .success()
        .stdout(predicate::str::contains("current decision"))
        .stdout(predicate::str::contains("old decision"))
        .stdout(predicate::str::contains("goal memory").not());

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--exclude", "state."])
        .assert()
        .success()
        .stdout(predicate::str::contains("goal memory"))
        .stdout(predicate::str::contains("state memory").not());

    polaris()
        .current_dir(dir.path())
        .args([
            "recall",
            "--prefix",
            "decision.",
            "--exclude",
            "decision.old.",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("current decision"))
        .stdout(predicate::str::contains("old decision").not());

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--key", "goal", "--prefix", "decision."])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--key", "missing"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No matching Polaris memory"));
}

#[test]
fn forget_removes_multiple_keys_atomically() {
    let dir = temp_workspace();

    polaris()
        .current_dir(dir.path())
        .args(["forget", "goal", "plan"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("polaris init"));

    init_workspace(dir.path());
    for (key, text) in [
        ("goal", "goal memory"),
        ("plan", "plan memory"),
        ("decision", "decision memory"),
    ] {
        polaris()
            .current_dir(dir.path())
            .args(["remember", "--key", key, "--text", text])
            .assert()
            .success();
    }

    polaris()
        .current_dir(dir.path())
        .args(["forget", "goal", "missing"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No memory exists for key `missing`",
        ));

    polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .stdout(predicate::str::contains("goal memory"))
        .stdout(predicate::str::contains("plan memory"))
        .stdout(predicate::str::contains("decision memory"));

    polaris()
        .current_dir(dir.path())
        .args(["forget", "goal", "plan"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Forgot 2 memories"));

    polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .stdout(predicate::str::contains("goal memory").not())
        .stdout(predicate::str::contains("plan memory").not())
        .stdout(predicate::str::contains("decision memory"));
}

#[test]
fn forget_prefix_requires_confirmation_and_preserves_unmatched_records() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    for (key, text) in [
        ("decision.storage", "storage decision"),
        ("decision.hooks", "hook decision"),
        ("goal", "goal memory"),
    ] {
        polaris()
            .current_dir(dir.path())
            .args(["remember", "--key", key, "--text", text])
            .assert()
            .success();
    }

    polaris()
        .current_dir(dir.path())
        .args(["forget", "--prefix", "decision."])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--yes"));

    polaris()
        .current_dir(dir.path())
        .args(["forget", "--prefix", "", "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("prefix must not be empty"));

    polaris()
        .current_dir(dir.path())
        .args(["forget", "--prefix", "missing.", "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No memory exists for prefix `missing.`",
        ));

    polaris()
        .current_dir(dir.path())
        .args(["forget", "goal", "--prefix", "decision.", "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));

    polaris()
        .current_dir(dir.path())
        .args(["forget", "--prefix", "decision.", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Forgot 2 memories"));

    polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .stdout(predicate::str::contains("storage decision").not())
        .stdout(predicate::str::contains("hook decision").not())
        .stdout(predicate::str::contains("goal memory"));
}

#[test]
fn recall_preserves_legacy_unkeyed_inline_memory() {
    let dir = temp_workspace();
    init_workspace(dir.path());
    fs::write(
        dir.path().join(".polaris/memories.jsonl"),
        r#"{"id":"legacy","created_at":"2026-05-31T00:00:00Z","kind":"inline","title":"Legacy","text":"old memory"}"#,
    )
    .unwrap();

    polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .stdout(predicate::str::contains("old memory"));
}

#[test]
fn note_create_records_markdown_file_and_recall_lists_context() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "scope",
            "--text",
            "keep scope small",
            "--title",
            "Scope",
        ])
        .assert()
        .success();
    let output = polaris()
        .current_dir(dir.path())
        .args(["note", "create", "--title", "Architecture notes"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let created = String::from_utf8(output).unwrap();
    assert!(created.contains(".polaris/docs/"));

    let recall = polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let recall = String::from_utf8(recall).unwrap();
    assert!(recall.contains("keep scope small"));
    assert!(recall.contains("Architecture notes"));
    assert!(recall.contains(".polaris/docs/"));
}

#[test]
fn recall_empty_and_missing_workspace_behaviors_are_explicit() {
    let dir = temp_workspace();

    polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .failure()
        .stderr(predicate::str::contains("polaris init"));

    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .arg("recall")
        .assert()
        .success()
        .stdout(predicate::str::contains("No Polaris memory"));
}

#[test]
fn clear_requires_confirmation_and_preserves_initialization() {
    let dir = temp_workspace();
    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "stale", "--text", "stale"])
        .assert()
        .success();
    let output = polaris()
        .current_dir(dir.path())
        .args(["note", "create", "--title", "Stale note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    let note_path = output
        .lines()
        .find_map(|line| line.strip_prefix("Path: "))
        .expect("note path in output");
    let note_path = dir.path().join(note_path);
    assert!(note_path.is_file());

    polaris()
        .current_dir(dir.path())
        .arg("clear")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--yes"));

    polaris()
        .current_dir(dir.path())
        .args(["clear", "--yes"])
        .assert()
        .success();

    assert!(dir.path().join(".polaris/state.json").is_file());
    assert!(dir.path().join(".polaris/docs").is_dir());
    assert!(!note_path.exists());
    let memories = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();
    assert!(memories.is_empty());
}

#[test]
fn session_start_hook_is_quiet_unless_compact_initialized_and_has_memory() {
    let dir = temp_workspace();
    let home = temp_workspace();
    let compact_input = serde_json::json!({
        "hook_event_name": "SessionStart",
        "source": "compact",
        "cwd": dir.path(),
    })
    .to_string();

    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "session-start"])
        .write_stdin(compact_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "session-start"])
        .write_stdin(compact_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "secret", "--text", "secret memory"])
        .assert()
        .success();

    let startup_input = serde_json::json!({
        "hook_event_name": "SessionStart",
        "source": "startup",
        "cwd": dir.path(),
    })
    .to_string();
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "session-start"])
        .write_stdin(startup_input)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let output = polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "session-start"])
        .write_stdin(compact_input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let hook: Value = serde_json::from_slice(&output).expect("hook json");
    let context = hook_context(&output);
    assert_eq!(hook["hookSpecificOutput"]["hookEventName"], "SessionStart");
    assert!(context.contains("polaris recall"));
    assert!(!context.contains("secret memory"));
}

#[test]
fn post_compact_fallback_hook_records_pending_and_post_tool_use_prompts_once() {
    let dir = temp_workspace();
    let home = temp_workspace();
    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "secret", "--text", "secret memory"])
        .assert()
        .success();

    let post_compact_input = serde_json::json!({
        "hook_event_name": "PostCompact",
        "cwd": dir.path(),
    })
    .to_string();
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "post-compact"])
        .write_stdin(post_compact_input)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let hook_state =
        fs::read_to_string(dir.path().join(".polaris/hook-state.json")).expect("hook state exists");
    assert!(hook_state.contains("\"compact_recall_pending\": true"));

    let post_tool_input = serde_json::json!({
        "hook_event_name": "PostToolUse",
        "cwd": dir.path(),
    })
    .to_string();
    let output = polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "post-tool-use"])
        .write_stdin(post_tool_input.as_str())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let hook: Value = serde_json::from_slice(&output).expect("hook json");
    let context = hook_context(&output);
    assert_eq!(hook["hookSpecificOutput"]["hookEventName"], "PostToolUse");
    assert!(context.contains("polaris recall"));
    assert!(!context.contains("secret memory"));

    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "post-tool-use"])
        .write_stdin(post_tool_input)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn hook_uses_workspace_prompt_without_inlining_recall() {
    let dir = temp_workspace();
    let home = temp_workspace();
    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "secret", "--text", "secret memory"])
        .assert()
        .success();
    fs::write(
        dir.path().join(".polaris/config.toml"),
        "[hooks]\nrecall_prompt = \"Custom workspace prompt\"\n",
    )
    .expect("write workspace config");

    let compact_input = serde_json::json!({
        "hook_event_name": "SessionStart",
        "source": "compact",
        "cwd": dir.path(),
    })
    .to_string();
    let output = polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "session-start"])
        .write_stdin(compact_input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let context = hook_context(&output);

    assert_eq!(context, "Custom workspace prompt");
    assert!(!context.contains("polaris recall"));
    assert!(!context.contains("secret memory"));
}

#[test]
fn hook_inlines_recall_when_workspace_prompt_contains_placeholder() {
    let dir = temp_workspace();
    let home = temp_workspace();
    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "inline memory"])
        .assert()
        .success();
    fs::write(
        dir.path().join(".polaris/config.toml"),
        "[hooks]\nrecall_prompt = \"Before\\n{{recall}}\\nAfter\"\n",
    )
    .expect("write workspace config");

    let compact_input = serde_json::json!({
        "hook_event_name": "SessionStart",
        "source": "compact",
        "cwd": dir.path(),
    })
    .to_string();
    let output = polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "session-start"])
        .write_stdin(compact_input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let context = hook_context(&output);
    assert!(context.starts_with("Before\n# Polaris Recall"));
    assert!(context.contains("- Key: goal"));
    assert!(context.contains("inline memory"));
    assert!(context.ends_with("\nAfter"));

    let post_compact_input = serde_json::json!({
        "hook_event_name": "PostCompact",
        "cwd": dir.path(),
    })
    .to_string();
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "post-compact"])
        .write_stdin(post_compact_input)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
    let post_tool_input = serde_json::json!({
        "hook_event_name": "PostToolUse",
        "cwd": dir.path(),
    })
    .to_string();
    let output = polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "post-tool-use"])
        .write_stdin(post_tool_input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let context = hook_context(&output);
    assert!(context.contains("# Polaris Recall"));
    assert!(context.contains("inline memory"));
}

#[test]
fn hook_prompt_prefers_workspace_config_and_falls_back_to_user_config() {
    let dir = temp_workspace();
    let home = temp_workspace();
    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "memory"])
        .assert()
        .success();
    fs::create_dir_all(home.path().join(".polaris")).expect("create user polaris config dir");
    fs::write(
        home.path().join(".polaris/config.toml"),
        "[hooks]\nrecall_prompt = \"User prompt\"\n",
    )
    .expect("write user config");

    let compact_input = serde_json::json!({
        "hook_event_name": "SessionStart",
        "source": "compact",
        "cwd": dir.path(),
    })
    .to_string();
    let output = polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "session-start"])
        .write_stdin(compact_input.as_str())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(hook_context(&output), "User prompt");

    fs::write(
        dir.path().join(".polaris/config.toml"),
        "[hooks]\nrecall_prompt = \"Workspace prompt\"\n",
    )
    .expect("write workspace config");
    let output = polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "session-start"])
        .write_stdin(compact_input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(hook_context(&output), "Workspace prompt");
}

#[test]
fn hook_reports_config_parse_errors_with_path() {
    let dir = temp_workspace();
    let home = temp_workspace();
    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "memory"])
        .assert()
        .success();
    fs::write(dir.path().join(".polaris/config.toml"), "[hooks\n").expect("write invalid config");

    let compact_input = serde_json::json!({
        "hook_event_name": "SessionStart",
        "source": "compact",
        "cwd": dir.path(),
    })
    .to_string();
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["hook", "session-start"])
        .write_stdin(compact_input)
        .assert()
        .failure()
        .stderr(predicate::str::contains(".polaris/config.toml"));
}

#[test]
fn post_compact_fallback_hook_is_quiet_without_memory_or_initialization_or_matching_events() {
    let dir = temp_workspace();
    let post_compact_input = serde_json::json!({
        "hook_event_name": "PostCompact",
        "cwd": dir.path(),
    })
    .to_string();

    polaris()
        .current_dir(dir.path())
        .args(["hook", "post-compact"])
        .write_stdin(post_compact_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
    assert!(!dir.path().join(".polaris/hook-state.json").exists());

    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args(["hook", "post-compact"])
        .write_stdin(post_compact_input)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let post_tool_input = serde_json::json!({
        "hook_event_name": "PostToolUse",
        "cwd": dir.path(),
    })
    .to_string();
    polaris()
        .current_dir(dir.path())
        .args(["hook", "post-tool-use"])
        .write_stdin(post_tool_input)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let mismatched_input = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "cwd": dir.path(),
    })
    .to_string();
    polaris()
        .current_dir(dir.path())
        .args(["hook", "post-compact"])
        .write_stdin(mismatched_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
    polaris()
        .current_dir(dir.path())
        .args(["hook", "post-tool-use"])
        .write_stdin(mismatched_input)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn clear_removes_pending_post_compact_hook_state() {
    let dir = temp_workspace();
    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "task goal"])
        .assert()
        .success();
    let post_compact_input = serde_json::json!({
        "hook_event_name": "PostCompact",
        "cwd": dir.path(),
    })
    .to_string();
    polaris()
        .current_dir(dir.path())
        .args(["hook", "post-compact"])
        .write_stdin(post_compact_input)
        .assert()
        .success();
    assert!(dir.path().join(".polaris/hook-state.json").is_file());

    polaris()
        .current_dir(dir.path())
        .args(["clear", "--yes"])
        .assert()
        .success();

    assert!(!dir.path().join(".polaris/hook-state.json").exists());
}

#[test]
fn codex_hook_example_configures_compact_session_recall() {
    let example_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("codex-hooks")
        .join("hooks.json");
    let example = fs::read_to_string(&example_path).expect("example hooks.json exists");
    let hooks: Value = serde_json::from_str(&example).expect("example hooks.json is valid JSON");

    let session_start_hooks = hooks["hooks"]["SessionStart"]
        .as_array()
        .expect("SessionStart hook list exists");
    let compact_hook = session_start_hooks
        .iter()
        .find(|hook| hook["matcher"] == "compact")
        .expect("compact SessionStart hook exists");
    let command_hooks = compact_hook["hooks"]
        .as_array()
        .expect("compact SessionStart command hooks exist");
    let command_hook = command_hooks
        .iter()
        .find(|hook| hook["command"] == "polaris hook session-start")
        .expect("Polaris session-start hook command exists");

    assert_eq!(command_hook["type"], "command");
    assert!(command_hook.get("statusMessage").is_some());
    assert_eq!(command_hooks.len(), 1);
}

#[test]
fn codex_fallback_hook_example_configures_post_compact_recall() {
    let example_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("codex-hooks")
        .join("post-compact-hooks.json");
    let example = fs::read_to_string(&example_path).expect("fallback hooks.json exists");
    let hooks: Value = serde_json::from_str(&example).expect("fallback hooks.json is valid JSON");

    let post_compact_hooks = hooks["hooks"]["PostCompact"]
        .as_array()
        .expect("PostCompact hook list exists");
    let post_compact_command_hooks = post_compact_hooks
        .first()
        .and_then(|hook| hook["hooks"].as_array())
        .expect("PostCompact command hooks exist");
    let post_compact_command = post_compact_command_hooks
        .iter()
        .find(|hook| hook["command"] == "polaris hook post-compact")
        .expect("Polaris post-compact hook command exists");
    assert_eq!(post_compact_command["type"], "command");

    let post_tool_use_hooks = hooks["hooks"]["PostToolUse"]
        .as_array()
        .expect("PostToolUse hook list exists");
    let post_tool_use_command_hooks = post_tool_use_hooks
        .first()
        .and_then(|hook| hook["hooks"].as_array())
        .expect("PostToolUse command hooks exist");
    let post_tool_use_command = post_tool_use_command_hooks
        .iter()
        .find(|hook| hook["command"] == "polaris hook post-tool-use")
        .expect("Polaris post-tool-use hook command exists");
    assert_eq!(post_tool_use_command["type"], "command");
}
