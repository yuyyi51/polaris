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

fn assert_jsonl_records_if_exists(path: &Path, expected_count: usize) -> Vec<Value> {
    if path.exists() {
        assert_jsonl_records(path, expected_count)
    } else {
        assert_eq!(expected_count, 0, "expected JSONL file {}", path.display());
        Vec::new()
    }
}

fn path_from_output(output: &[u8], prefix: &str) -> String {
    let output = String::from_utf8(output.to_vec()).expect("utf8 output");
    output
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .unwrap_or_else(|| panic!("missing `{prefix}` line in output:\n{output}"))
        .to_string()
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
fn remember_and_note_create_record_lifecycle_and_reject_invalid_lifecycle() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "branch",
            "--text",
            "working on filters",
            "--lifecycle",
            "state",
        ])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args([
            "note",
            "create",
            "--title",
            "Release archive",
            "--lifecycle",
            "archive",
        ])
        .assert()
        .success();

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
    assert!(
        summaries
            .iter()
            .any(|record| record["key"] == "branch" && record["lifecycle"] == "state")
    );
    assert!(summaries.iter().any(|record| {
        record["title"] == "Release archive" && record["lifecycle"] == "archive"
    }));

    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "goal",
            "--text",
            "task",
            "--lifecycle",
            "temporary",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("durable"))
        .stderr(predicate::str::contains("state"))
        .stderr(predicate::str::contains("log"))
        .stderr(predicate::str::contains("archive"));

    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 2);
    assert!(records.iter().all(|record| record["key"] != "goal"));
}

#[test]
fn legacy_records_without_lifecycle_load_as_durable() {
    let dir = temp_workspace();
    init_workspace(dir.path());
    fs::write(
        dir.path().join(".polaris/memories.jsonl"),
        r#"{"id":"legacy","created_at":"2026-05-31T00:00:00Z","kind":"inline","key":"legacy","text":"old memory"}"#,
    )
    .unwrap();

    let output = polaris()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let summaries: Value = serde_json::from_slice(&output).expect("list json");
    assert_eq!(summaries[0]["lifecycle"], "durable");

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--lifecycle", "durable"])
        .assert()
        .success()
        .stdout(predicate::str::contains("old memory"));

    let output = polaris()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status: Value = serde_json::from_slice(&output).expect("status json");
    assert_eq!(status["lifecycle_counts"]["durable"], 1);
}

#[test]
fn list_and_recall_filter_by_lifecycle_with_existing_filters() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    for (key, text, lifecycle) in [
        ("decision.current", "current decision", "durable"),
        ("decision.archived", "archived decision", "archive"),
        ("state.branch", "branch state", "state"),
        ("log.today", "today log", "log"),
    ] {
        polaris()
            .current_dir(dir.path())
            .args([
                "remember",
                "--key",
                key,
                "--text",
                text,
                "--lifecycle",
                lifecycle,
            ])
            .assert()
            .success();
    }

    let output = polaris()
        .current_dir(dir.path())
        .args(["list", "--json", "--lifecycle", "state"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let summaries: Value = serde_json::from_slice(&output).expect("list json");
    let summaries = summaries.as_array().expect("summary array");
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0]["key"], "state.branch");

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--lifecycle", "durable"])
        .assert()
        .success()
        .stdout(predicate::str::contains("current decision"))
        .stdout(predicate::str::contains("branch state").not())
        .stdout(predicate::str::contains("archived decision").not());

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--prefix", "decision.", "--lifecycle", "durable"])
        .assert()
        .success()
        .stdout(predicate::str::contains("current decision"))
        .stdout(predicate::str::contains("archived decision").not());

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--key", "state.branch", "--lifecycle", "state"])
        .assert()
        .success()
        .stdout(predicate::str::contains("branch state"));

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--lifecycle", "log", "--exclude", "log."])
        .assert()
        .success()
        .stdout(predicate::str::contains("No matching Polaris memory"));
}

#[test]
fn remember_replace_updates_metadata_and_preserves_creation_and_lifecycle() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "branch",
            "--text",
            "old branch",
            "--lifecycle",
            "state",
        ])
        .assert()
        .success();
    let initial_records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 1);
    let initial_id = initial_records[0]["id"]
        .as_str()
        .expect("initial id")
        .to_string();
    let created_at = initial_records[0]["created_at"]
        .as_str()
        .expect("created_at")
        .to_string();

    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "branch",
            "--replace",
            "--text",
            "new branch",
        ])
        .assert()
        .success();
    let output = polaris()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let summaries: Value = serde_json::from_slice(&output).expect("list json");
    let branch = summaries
        .as_array()
        .expect("summary array")
        .iter()
        .find(|record| record["key"] == "branch")
        .expect("branch summary");
    assert_eq!(branch["created_at"], created_at);
    assert_eq!(branch["lifecycle"], "state");
    assert_eq!(branch["replacement_count"], 1);
    assert_eq!(branch["replaced_from"], initial_id);
    assert!(branch["updated_at"].as_str().expect("updated_at") >= created_at.as_str());

    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "branch",
            "--replace",
            "--text",
            "archived branch",
            "--lifecycle",
            "archive",
        ])
        .assert()
        .success();
    let output = polaris()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let summaries: Value = serde_json::from_slice(&output).expect("list json");
    let branch = &summaries[0];
    assert_eq!(branch["created_at"], created_at);
    assert_eq!(branch["lifecycle"], "archive");
    assert_eq!(branch["replacement_count"], 2);
}

#[test]
fn lifecycle_move_updates_inline_memory_by_key_and_prefix() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "state.branch", "--text", "branch text"])
        .assert()
        .success();
    let before = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 1);
    let created_at = before[0]["created_at"].clone();
    let id = before[0]["id"].clone();

    polaris()
        .current_dir(dir.path())
        .args([
            "lifecycle",
            "move",
            "--key",
            "state.branch",
            "--to",
            "state",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Moved 1 memory"))
        .stdout(predicate::str::contains("state.branch"));

    let after = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 1);
    assert_eq!(after[0]["id"], id);
    assert_eq!(after[0]["created_at"], created_at);
    assert_eq!(after[0]["key"], "state.branch");
    assert_eq!(after[0]["text"], "branch text");
    assert_eq!(after[0]["lifecycle"], "state");
    assert!(after[0]["updated_at"].as_str().is_some());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "state.plan", "--text", "plan text"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "goal text"])
        .assert()
        .success();

    polaris()
        .current_dir(dir.path())
        .args([
            "lifecycle",
            "move",
            "--prefix",
            "state.",
            "--from",
            "durable",
            "--to",
            "state",
            "--yes",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Moved 1 memory"))
        .stdout(predicate::str::contains("Skipped 1 memory"));

    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 3);
    assert!(
        records
            .iter()
            .any(|record| { record["key"] == "state.branch" && record["lifecycle"] == "state" })
    );
    assert!(
        records
            .iter()
            .any(|record| { record["key"] == "state.plan" && record["lifecycle"] == "state" })
    );
    assert!(
        records
            .iter()
            .any(|record| record["key"] == "goal" && record.get("lifecycle").is_none())
    );
}

#[test]
fn lifecycle_move_updates_notes_by_id_and_kind_without_touching_note_file() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args([
            "note",
            "create",
            "--title",
            "Current note",
            "--lifecycle",
            "log",
        ])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["note", "create", "--title", "Durable note"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "log.inline",
            "--text",
            "inline log",
            "--lifecycle",
            "log",
        ])
        .assert()
        .success();

    let before = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 3);
    let current_note = before
        .iter()
        .find(|record| record["title"] == "Current note")
        .expect("current note");
    let current_id = current_note["id"].as_str().expect("note id").to_string();
    let note_path = dir
        .path()
        .join(current_note["path"].as_str().expect("note path"));
    let note_contents = fs::read_to_string(&note_path).expect("note file");
    let created_at = current_note["created_at"].clone();

    polaris()
        .current_dir(dir.path())
        .args(["lifecycle", "move", "--id", &current_id, "--to", "archive"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&current_id));

    assert_eq!(
        fs::read_to_string(&note_path).expect("note file"),
        note_contents
    );
    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 3);
    let current_note = records
        .iter()
        .find(|record| record["id"] == current_id)
        .expect("current note");
    assert_eq!(current_note["lifecycle"], "archive");
    assert_eq!(current_note["created_at"], created_at);
    assert!(current_note["updated_at"].as_str().is_some());

    polaris()
        .current_dir(dir.path())
        .args([
            "lifecycle",
            "move",
            "--kind",
            "note",
            "--from",
            "durable",
            "--to",
            "archive",
            "--yes",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Moved 1 memory"));

    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 3);
    assert!(
        records
            .iter()
            .filter(|record| record["kind"] == "note")
            .all(|record| record["lifecycle"] == "archive")
    );
    assert!(
        records
            .iter()
            .any(|record| { record["key"] == "log.inline" && record["lifecycle"] == "log" })
    );

    polaris()
        .current_dir(dir.path())
        .args([
            "lifecycle",
            "move",
            "--kind",
            "inline",
            "--from",
            "log",
            "--to",
            "state",
            "--yes",
        ])
        .assert()
        .success();

    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 3);
    assert!(
        records
            .iter()
            .any(|record| { record["key"] == "log.inline" && record["lifecycle"] == "state" })
    );
}

#[test]
fn lifecycle_move_requires_confirmation_for_batches_and_rejects_conflicting_selectors() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "state.branch", "--text", "branch"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["note", "create", "--title", "A note"])
        .assert()
        .success();
    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 2);
    let note_id = records
        .iter()
        .find(|record| record["kind"] == "note")
        .expect("note")["id"]
        .as_str()
        .expect("note id")
        .to_string();

    polaris()
        .current_dir(dir.path())
        .args(["lifecycle", "move", "--prefix", "state.", "--to", "state"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--yes"));
    assert!(
        assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 2)
            .iter()
            .all(|record| record.get("lifecycle").is_none())
    );

    polaris()
        .current_dir(dir.path())
        .args([
            "lifecycle",
            "move",
            "--key",
            "state.branch",
            "--id",
            &note_id,
            "--to",
            "state",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("selectors conflict"));
    assert!(
        assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 2)
            .iter()
            .all(|record| record.get("lifecycle").is_none())
    );
}

#[test]
fn lifecycle_move_json_reports_moved_and_skipped_records() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "state.one", "--text", "one"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "state.two",
            "--text",
            "two",
            "--lifecycle",
            "state",
        ])
        .assert()
        .success();

    let output = polaris()
        .current_dir(dir.path())
        .args([
            "lifecycle",
            "move",
            "--prefix",
            "state.",
            "--to",
            "state",
            "--yes",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&output).expect("move json");
    let moved = report["moved"].as_array().expect("moved records");
    let skipped = report["skipped"].as_array().expect("skipped records");
    assert_eq!(moved.len(), 1);
    assert_eq!(skipped.len(), 1);
    assert_eq!(moved[0]["key"], "state.one");
    assert_eq!(moved[0]["previous_lifecycle"], "durable");
    assert_eq!(moved[0]["new_lifecycle"], "state");
    assert_eq!(skipped[0]["key"], "state.two");
    assert_eq!(skipped[0]["reason"], "already at target lifecycle");
}

#[test]
fn replace_saves_history_and_diff_reports_latest_replacement() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "line one\nold line"])
        .assert()
        .success();
    let initial = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 1);
    let initial_id = initial[0]["id"].as_str().expect("initial id").to_string();

    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "goal",
            "--replace",
            "--text",
            "line one\nnew line",
        ])
        .assert()
        .success();

    let history = assert_jsonl_records(&dir.path().join(".polaris/replacement-history.jsonl"), 1);
    assert_eq!(history[0]["id"], initial_id);
    assert_eq!(history[0]["text"], "line one\nold line");

    polaris()
        .current_dir(dir.path())
        .args(["diff", "--key", "goal"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Diff for memory goal"))
        .stdout(predicate::str::contains("--- previous"))
        .stdout(predicate::str::contains("+++ current"))
        .stdout(predicate::str::contains("-old line"))
        .stdout(predicate::str::contains("+new line"));

    let output = polaris()
        .current_dir(dir.path())
        .args(["diff", "--key", "goal", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let diff: Value = serde_json::from_slice(&output).expect("diff json");
    assert_eq!(diff["key"], "goal");
    assert_eq!(diff["previous_id"], initial_id);
    assert!(diff["current_id"].as_str().is_some());
    assert!(diff["diff"].as_str().expect("diff").contains("-old line"));
    assert!(diff["diff"].as_str().expect("diff").contains("+new line"));
}

#[test]
fn diff_reports_no_snapshot_and_rejects_missing_or_uninitialized_keys() {
    let dir = temp_workspace();

    polaris()
        .current_dir(dir.path())
        .args(["diff", "--key", "goal"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("polaris init"));

    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args(["diff", "--key", "missing"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No memory exists for key `missing`",
        ));

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "stable goal"])
        .assert()
        .success();

    polaris()
        .current_dir(dir.path())
        .args(["diff", "--key", "goal"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No replacement snapshot is available for goal",
        ));

    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 1);
    let updated = records[0].clone();
    let mut object = updated.as_object().expect("record object").clone();
    object.insert(
        "replaced_from".to_string(),
        Value::String("legacy-missing".to_string()),
    );
    fs::write(
        dir.path().join(".polaris/memories.jsonl"),
        format!("{}\n", Value::Object(object)),
    )
    .unwrap();

    polaris()
        .current_dir(dir.path())
        .args(["diff", "--key", "goal"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No replacement snapshot is available for goal",
        ));
}

#[test]
fn remember_replace_dry_run_creates_editable_draft_without_mutating_memory() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "old goal"])
        .assert()
        .success();
    let before = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();

    let output = polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "goal",
            "--replace",
            "--dry-run",
            "--text",
            "new goal",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Replacement draft preview for goal",
        ))
        .stdout(predicate::str::contains("-old goal"))
        .stdout(predicate::str::contains("+new goal"))
        .stdout(predicate::str::contains(
            format!(
                "Path: {}/.polaris/maintenance/replace-goal-",
                dir.path().display()
            )
            .as_str(),
        ))
        .get_output()
        .stdout
        .clone();
    let draft_path = path_from_output(&output, "Path: ");
    let draft = fs::read_to_string(&draft_path).expect("replace draft");
    assert!(draft.contains("polaris-replace-draft-v1"));
    assert!(draft.contains("key: goal"));
    assert!(draft.contains("## Current Memory"));
    assert!(draft.contains("old goal"));
    assert!(draft.contains("## Replacement Memory"));
    assert!(draft.contains("new goal"));

    assert_eq!(
        fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap(),
        before
    );
    assert_jsonl_records_if_exists(&dir.path().join(".polaris/replacement-history.jsonl"), 0);
}

#[test]
fn remember_replace_dry_run_accepts_stdin_and_requires_replace() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "old goal"])
        .assert()
        .success();
    let before = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();

    let output = polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "goal",
            "--replace",
            "--dry-run",
            "--stdin",
        ])
        .write_stdin("stdin replacement\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let draft_path = path_from_output(&output, "Path: ");
    let draft = fs::read_to_string(dir.path().join(&draft_path)).expect("replace draft");
    assert!(draft.contains("## Replacement Memory\n\nstdin replacement\n"));

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--dry-run", "--text", "bad"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--dry-run requires --replace"));
    assert_eq!(
        fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap(),
        before
    );
}

#[test]
fn replace_apply_uses_edited_draft_and_writes_replacement_history() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "goal",
            "--text",
            "old goal",
            "--lifecycle",
            "state",
        ])
        .assert()
        .success();
    let initial = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 1);
    let initial_id = initial[0]["id"].as_str().expect("initial id").to_string();
    let created_at = initial[0]["created_at"].clone();

    let output = polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "goal",
            "--replace",
            "--dry-run",
            "--text",
            "draft goal",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let draft_path = path_from_output(&output, "Path: ");
    let full_draft_path = dir.path().join(&draft_path);
    let draft = fs::read_to_string(&full_draft_path).expect("replace draft");
    fs::write(
        &full_draft_path,
        draft.replace("draft goal", "edited replacement goal"),
    )
    .unwrap();

    polaris()
        .current_dir(dir.path())
        .args(["replace", "apply", &draft_path, "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Applied replacement draft to goal",
        ));

    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 1);
    assert_eq!(records[0]["key"], "goal");
    assert_eq!(records[0]["text"], "edited replacement goal");
    assert_eq!(records[0]["created_at"], created_at);
    assert_eq!(records[0]["lifecycle"], "state");
    assert_eq!(records[0]["replacement_count"], 1);
    assert_eq!(records[0]["replaced_from"], initial_id);

    let history = assert_jsonl_records(&dir.path().join(".polaris/replacement-history.jsonl"), 1);
    assert_eq!(history[0]["id"], initial_id);
    assert_eq!(history[0]["text"], "old goal");

    polaris()
        .current_dir(dir.path())
        .args(["diff", "--key", "goal"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-old goal"))
        .stdout(predicate::str::contains("+edited replacement goal"));
}

#[test]
fn replace_apply_requires_confirmation_and_rejects_invalid_drafts() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "old goal"])
        .assert()
        .success();
    let output = polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "goal",
            "--replace",
            "--dry-run",
            "--text",
            "new goal",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let draft_path = path_from_output(&output, "Path: ");
    let before = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();

    polaris()
        .current_dir(dir.path())
        .args(["replace", "apply", &draft_path])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--yes"));

    let bad_path = dir.path().join(".polaris/maintenance/bad-replace.md");
    fs::write(&bad_path, "not a replacement draft").unwrap();
    polaris()
        .current_dir(dir.path())
        .args([
            "replace",
            "apply",
            ".polaris/maintenance/bad-replace.md",
            "--yes",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "replacement draft cannot be applied",
        ));

    assert_eq!(
        fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap(),
        before
    );
}

#[test]
fn status_json_reports_lifecycle_counts_and_stale_volatile_hints() {
    let dir = temp_workspace();
    init_workspace(dir.path());
    fs::write(
        dir.path().join(".polaris/memories.jsonl"),
        concat!(
            "{\"id\":\"durable\",\"created_at\":\"2026-06-11T00:00:00Z\",\"kind\":\"inline\",\"key\":\"goal\",\"text\":\"goal\",\"lifecycle\":\"durable\"}\n",
            "{\"id\":\"state\",\"created_at\":\"2026-05-01T00:00:00Z\",\"kind\":\"inline\",\"key\":\"state.branch\",\"text\":\"branch\",\"lifecycle\":\"state\"}\n",
            "{\"id\":\"log\",\"created_at\":\"2026-05-02T00:00:00Z\",\"kind\":\"inline\",\"key\":\"log.today\",\"text\":\"log\",\"lifecycle\":\"log\"}\n",
            "{\"id\":\"archive\",\"created_at\":\"2026-06-11T00:00:00Z\",\"kind\":\"inline\",\"key\":\"archive.release\",\"text\":\"archive\",\"lifecycle\":\"archive\"}\n",
        ),
    )
    .unwrap();

    let output = polaris()
        .current_dir(dir.path())
        .args(["status", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status: Value = serde_json::from_slice(&output).expect("status json");
    assert_eq!(status["lifecycle_counts"]["durable"], 1);
    assert_eq!(status["lifecycle_counts"]["state"], 1);
    assert_eq!(status["lifecycle_counts"]["log"], 1);
    assert_eq!(status["lifecycle_counts"]["archive"], 1);
    assert_eq!(status["stale_volatile_memory"]["count"], 2);
    let stale = status["stale_volatile_memory"]["records"]
        .as_array()
        .expect("stale records");
    assert!(
        stale
            .iter()
            .any(|record| record["key"] == "state.branch" && record["lifecycle"] == "state")
    );
    assert!(
        stale
            .iter()
            .any(|record| record["key"] == "log.today" && record["lifecycle"] == "log")
    );
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
fn recall_keys_renders_in_requested_order_and_reports_missing_or_filtered_keys() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    for (key, text, lifecycle) in [
        ("goal", "goal memory", "durable"),
        ("plan", "plan memory", "durable"),
        ("state.branch", "state memory", "state"),
        ("other", "other memory", "durable"),
    ] {
        polaris()
            .current_dir(dir.path())
            .args([
                "remember",
                "--key",
                key,
                "--text",
                text,
                "--lifecycle",
                lifecycle,
            ])
            .assert()
            .success();
    }

    let output = polaris()
        .current_dir(dir.path())
        .args(["recall", "--keys", "plan,goal,state.branch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("other memory").not())
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    let plan = output.find("plan memory").expect("plan");
    let goal = output.find("goal memory").expect("goal");
    let state = output.find("state memory").expect("state");
    assert!(plan < goal);
    assert!(goal < state);

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--keys", "goal,missing"])
        .assert()
        .success()
        .stdout(predicate::str::contains("goal memory"))
        .stdout(predicate::str::contains("Missing keys: missing"));

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--keys", "missing,absent"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No matching Polaris memory"))
        .stdout(predicate::str::contains("Missing keys: missing, absent"));

    polaris()
        .current_dir(dir.path())
        .args([
            "recall",
            "--keys",
            "goal,state.branch",
            "--lifecycle",
            "durable",
            "--exclude",
            "state.",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("goal memory"))
        .stdout(predicate::str::contains("state memory").not())
        .stdout(predicate::str::contains("Missing keys: state.branch"));
}

#[test]
fn recall_keys_rejects_selector_conflicts() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--keys", "goal,plan", "--key", "goal"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--keys"))
        .stderr(predicate::str::contains("conflict"));

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--keys", "goal,plan", "--prefix", "goal"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--keys"))
        .stderr(predicate::str::contains("conflict"));
}

#[test]
fn touch_updates_timestamps_without_changing_memory_content_or_metadata() {
    let dir = temp_workspace();
    init_workspace(dir.path());
    fs::write(
        dir.path().join(".polaris/memories.jsonl"),
        concat!(
            "{\"id\":\"state\",\"created_at\":\"2026-05-01T00:00:00Z\",\"updated_at\":\"2026-05-02T00:00:00Z\",\"kind\":\"inline\",\"key\":\"state.branch\",\"text\":\"branch\",\"lifecycle\":\"state\",\"replacement_count\":2,\"replaced_from\":\"old-state\"}\n",
            "{\"id\":\"queue\",\"created_at\":\"2026-05-01T00:00:00Z\",\"kind\":\"inline\",\"key\":\"state.queue\",\"text\":\"queue\",\"lifecycle\":\"state\"}\n",
            "{\"id\":\"goal\",\"created_at\":\"2026-05-01T00:00:00Z\",\"kind\":\"inline\",\"key\":\"goal\",\"text\":\"goal\",\"lifecycle\":\"durable\"}\n",
        ),
    )
    .unwrap();

    polaris()
        .current_dir(dir.path())
        .args(["touch", "--key", "state.branch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Touched 1 memory"))
        .stdout(predicate::str::contains("state.branch"));

    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 3);
    let state = records
        .iter()
        .find(|record| record["key"] == "state.branch")
        .expect("state");
    assert_eq!(state["id"], "state");
    assert_eq!(state["created_at"], "2026-05-01T00:00:00Z");
    assert_ne!(state["updated_at"], "2026-05-02T00:00:00Z");
    assert_eq!(state["text"], "branch");
    assert_eq!(state["lifecycle"], "state");
    assert_eq!(state["replacement_count"], 2);
    assert_eq!(state["replaced_from"], "old-state");

    polaris()
        .current_dir(dir.path())
        .args(["touch", "--keys", "state.branch,state.queue"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Touched 2 memories"));
    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 3);
    assert!(
        records
            .iter()
            .filter(|record| record["key"]
                .as_str()
                .is_some_and(|key| key.starts_with("state.")))
            .all(|record| record["updated_at"].as_str().is_some())
    );
    let goal = records
        .iter()
        .find(|record| record["key"] == "goal")
        .expect("goal");
    assert!(goal.get("updated_at").is_none());
}

#[test]
fn touch_updates_note_by_id_and_prefix_with_confirmation() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args([
            "note",
            "create",
            "--title",
            "State note",
            "--lifecycle",
            "state",
        ])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "state.branch", "--text", "branch"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "goal"])
        .assert()
        .success();

    let before = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 3);
    let note = before
        .iter()
        .find(|record| record["kind"] == "note")
        .expect("note");
    let note_id = note["id"].as_str().expect("note id").to_string();
    let note_path = dir.path().join(note["path"].as_str().expect("note path"));
    let note_contents = fs::read_to_string(&note_path).expect("note contents");

    polaris()
        .current_dir(dir.path())
        .args(["touch", "--id", &note_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(&note_id));
    assert_eq!(fs::read_to_string(&note_path).unwrap(), note_contents);

    polaris()
        .current_dir(dir.path())
        .args(["touch", "--prefix", "state."])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--yes"));

    polaris()
        .current_dir(dir.path())
        .args(["touch", "--prefix", "state.", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Touched 1 memory"));

    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 3);
    assert!(
        records
            .iter()
            .find(|record| record["key"] == "state.branch")
            .expect("state branch")["updated_at"]
            .as_str()
            .is_some()
    );
    assert!(
        records
            .iter()
            .find(|record| record["key"] == "goal")
            .expect("goal")
            .get("updated_at")
            .is_none()
    );
}

#[test]
fn touch_rejects_conflicting_selectors_and_reports_json() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "state.branch", "--text", "branch"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "state.queue", "--text", "queue"])
        .assert()
        .success();
    let before = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 2);
    let id = before[0]["id"].as_str().expect("id").to_string();

    polaris()
        .current_dir(dir.path())
        .args(["touch", "--key", "state.branch", "--id", &id])
        .assert()
        .failure()
        .stderr(predicate::str::contains("selectors conflict"));
    assert!(
        assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 2)
            .iter()
            .all(|record| record.get("updated_at").is_none())
    );

    let output = polaris()
        .current_dir(dir.path())
        .args(["touch", "--keys", "state.branch,missing", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: Value = serde_json::from_slice(&output).expect("touch json");
    assert_eq!(report["touched"].as_array().expect("touched").len(), 1);
    assert_eq!(report["touched"][0]["key"], "state.branch");
    assert!(report["touched"][0]["previous_updated_at"].is_null());
    assert!(report["touched"][0]["new_updated_at"].as_str().is_some());
    assert_eq!(report["missing_keys"][0], "missing");
    assert_eq!(report["skipped"].as_array().expect("skipped").len(), 0);
}

#[test]
fn cite_records_by_key_id_and_multi_selectors_without_mutating_memory() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "goal",
            "--text",
            "goal text",
            "--title",
            "Goal",
            "--lifecycle",
            "state",
        ])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "plan", "--text", "plan text"])
        .assert()
        .success();
    let before = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();
    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 2);
    let goal_id = records
        .iter()
        .find(|record| record["key"] == "goal")
        .expect("goal")["id"]
        .as_str()
        .expect("goal id")
        .to_string();
    let plan_id = records
        .iter()
        .find(|record| record["key"] == "plan")
        .expect("plan")["id"]
        .as_str()
        .expect("plan id")
        .to_string();

    polaris()
        .current_dir(dir.path())
        .args(["cite", "--key", "goal"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Cited 1 memory"))
        .stdout(predicate::str::contains("goal"));
    polaris()
        .current_dir(dir.path())
        .args(["cite", "--id", &plan_id])
        .assert()
        .success()
        .stdout(predicate::str::contains(&plan_id));
    polaris()
        .current_dir(dir.path())
        .args(["cite", "--keys", "goal,missing", "--quiet"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("Missing keys: missing"));

    assert_eq!(
        fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap(),
        before
    );

    let citations = assert_jsonl_records(&dir.path().join(".polaris/citations.jsonl"), 3);
    assert_eq!(citations[0]["record_id"], goal_id);
    assert_eq!(citations[0]["key"], "goal");
    assert_eq!(citations[0]["title"], "Goal");
    assert_eq!(citations[0]["kind"], "inline");
    assert_eq!(citations[0]["record_created_at"], records[0]["created_at"]);
    assert!(citations[0]["cited_at"].as_str().is_some());
}

#[test]
fn cite_rejects_invalid_selectors_and_no_match_commands() {
    let dir = temp_workspace();

    polaris()
        .current_dir(dir.path())
        .args(["cite", "--key", "goal"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("polaris init"));

    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "goal text"])
        .assert()
        .success();
    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 1);
    let goal_id = records[0]["id"].as_str().expect("goal id").to_string();

    polaris()
        .current_dir(dir.path())
        .args(["cite", "--key", "goal", "--id", &goal_id])
        .assert()
        .failure()
        .stderr(predicate::str::contains("selectors conflict"));
    polaris()
        .current_dir(dir.path())
        .args(["cite", "--key", "missing"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No memory exists for key `missing`",
        ));
    polaris()
        .current_dir(dir.path())
        .args(["cite", "--keys", "missing,absent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No requested memory records were cited",
        ));
    polaris()
        .current_dir(dir.path())
        .args(["cite", "--prefix", "decision."])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--prefix"));

    assert_jsonl_records_if_exists(&dir.path().join(".polaris/citations.jsonl"), 0);
}

#[test]
fn list_json_reports_direct_citation_metadata_and_empty_history() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "goal text"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "plan", "--text", "plan text"])
        .assert()
        .success();

    let output = polaris()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let summaries: Value = serde_json::from_slice(&output).expect("list json");
    let summaries = summaries.as_array().expect("summaries");
    assert!(summaries.iter().all(|summary| {
        summary["direct_cite_count"] == 0 && summary["inherited_cite_count"] == 0
    }));

    polaris()
        .current_dir(dir.path())
        .args(["cite", "--key", "goal"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["cite", "--key", "goal"])
        .assert()
        .success();

    let output = polaris()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let summaries: Value = serde_json::from_slice(&output).expect("list json");
    let summaries = summaries.as_array().expect("summaries");
    let goal = summaries
        .iter()
        .find(|summary| summary["key"] == "goal")
        .expect("goal summary");
    let plan = summaries
        .iter()
        .find(|summary| summary["key"] == "plan")
        .expect("plan summary");
    assert_eq!(goal["direct_cite_count"], 2);
    assert_eq!(goal["inherited_cite_count"], 0);
    assert!(goal["last_cited_at"].as_str().is_some());
    assert!(goal["created_at"].as_str().is_some());
    assert_eq!(plan["direct_cite_count"], 0);
    assert_eq!(plan["inherited_cite_count"], 0);
}

#[test]
fn citation_lineage_separates_inherited_and_direct_counts() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "old goal"])
        .assert()
        .success();
    let before_replace = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 1);
    let old_goal_id = before_replace[0]["id"]
        .as_str()
        .expect("old id")
        .to_string();
    polaris()
        .current_dir(dir.path())
        .args(["cite", "--key", "goal"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["cite", "--key", "goal"])
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
            "new goal",
        ])
        .assert()
        .success();

    let output = polaris()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let summaries: Value = serde_json::from_slice(&output).expect("list json");
    let goal = summaries
        .as_array()
        .expect("summaries")
        .iter()
        .find(|summary| summary["key"] == "goal")
        .expect("goal summary");
    assert_eq!(goal["direct_cite_count"], 0);
    assert_eq!(goal["inherited_cite_count"], 2);
    assert!(
        goal["citation_source_ids"]
            .as_array()
            .expect("source ids")
            .iter()
            .any(|source| source == &old_goal_id)
    );

    polaris()
        .current_dir(dir.path())
        .args(["cite", "--key", "goal"])
        .assert()
        .success();
    let output = polaris()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let summaries: Value = serde_json::from_slice(&output).expect("list json");
    let goal = summaries
        .as_array()
        .expect("summaries")
        .iter()
        .find(|summary| summary["key"] == "goal")
        .expect("goal summary");
    assert_eq!(goal["direct_cite_count"], 1);
    assert_eq!(goal["inherited_cite_count"], 2);
}

#[test]
fn merge_citation_lineage_inherits_source_counts_without_double_counting() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "keyA", "--text", "old alpha"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["cite", "--key", "keyA"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["cite", "--key", "keyA"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "keyA",
            "--replace",
            "--text",
            "new alpha",
        ])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["cite", "--key", "keyA"])
        .assert()
        .success();

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "keyB", "--text", "beta"])
        .assert()
        .success();
    let output = polaris()
        .current_dir(dir.path())
        .args(["merge", "--into", "keyB", "keyA"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    let key_b_draft = output
        .lines()
        .find_map(|line| line.strip_prefix("Path: "))
        .expect("keyB draft");
    fs::write(
        dir.path().join(key_b_draft),
        "<!-- polaris-merge-draft-v1\ntarget: keyB\nsources: keyA\n-->\n\n# Polaris Merge Draft\n\n## Merged Memory\n\nbeta summary\n",
    )
    .expect("edit keyB draft");
    polaris()
        .current_dir(dir.path())
        .args(["merge", "apply", key_b_draft, "--yes"])
        .assert()
        .success();

    let output = polaris()
        .current_dir(dir.path())
        .args(["merge", "--into", "model.summary", "keyA", "keyB"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    let summary_draft = output
        .lines()
        .find_map(|line| line.strip_prefix("Path: "))
        .expect("summary draft");
    fs::write(
        dir.path().join(summary_draft),
        "<!-- polaris-merge-draft-v1\ntarget: model.summary\nsources: keyA,keyB\n-->\n\n# Polaris Merge Draft\n\n## Merged Memory\n\nmerged summary\n",
    )
    .expect("edit summary draft");
    polaris()
        .current_dir(dir.path())
        .args(["merge", "apply", summary_draft, "--yes", "--forget-sources"])
        .assert()
        .success();

    let output = polaris()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let summaries: Value = serde_json::from_slice(&output).expect("list json");
    let summary = summaries
        .as_array()
        .expect("summaries")
        .iter()
        .find(|summary| summary["key"] == "model.summary")
        .expect("model summary");
    assert_eq!(summary["direct_cite_count"], 0);
    assert_eq!(summary["inherited_cite_count"], 3);
    let source_ids = summary["citation_source_ids"]
        .as_array()
        .expect("source ids");
    let unique = source_ids
        .iter()
        .map(|source| source.as_str().expect("source id"))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(source_ids.len(), unique.len());
}

#[test]
fn citation_jsonl_diagnostics_and_concurrent_recording_are_stable() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    for key in ["one", "two", "three"] {
        polaris()
            .current_dir(dir.path())
            .args(["remember", "--key", key, "--text", key])
            .assert()
            .success();
    }

    run_parallel_remember_commands(
        dir.path(),
        vec![
            vec![
                "cite".to_string(),
                "--key".to_string(),
                "one".to_string(),
                "--quiet".to_string(),
            ],
            vec![
                "cite".to_string(),
                "--key".to_string(),
                "two".to_string(),
                "--quiet".to_string(),
            ],
            vec![
                "cite".to_string(),
                "--key".to_string(),
                "three".to_string(),
                "--quiet".to_string(),
            ],
        ],
    );
    assert_jsonl_records(&dir.path().join(".polaris/citations.jsonl"), 3);
    polaris()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .success();

    let mut citations = fs::read_to_string(dir.path().join(".polaris/citations.jsonl")).unwrap();
    citations.push_str("\n   \n");
    fs::write(dir.path().join(".polaris/citations.jsonl"), citations).unwrap();
    polaris()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .success();

    fs::write(
        dir.path().join(".polaris/citations.jsonl"),
        "{\"id\":\"good\",\"record_id\":\"one\",\"cited_at\":\"2026-06-16T00:00:00Z\",\"kind\":\"inline\",\"record_created_at\":\"2026-06-16T00:00:00Z\"}\nnot json\n",
    )
    .unwrap();
    polaris()
        .current_dir(dir.path())
        .args(["list", "--json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(".polaris/citations.jsonl"))
        .stderr(predicate::str::contains(":2:"))
        .stderr(predicate::str::contains("expected"));
}

#[test]
fn rename_changes_key_and_preserves_memory_text_and_metadata() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "old.key",
            "--text",
            "keep this text",
            "--lifecycle",
            "state",
        ])
        .assert()
        .success();
    let before = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 1);
    let created_at = before[0]["created_at"].clone();
    let id = before[0]["id"].clone();

    polaris()
        .current_dir(dir.path())
        .args(["rename", "old.key", "new.key"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Renamed memory old.key to new.key",
        ));

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--key", "new.key"])
        .assert()
        .success()
        .stdout(predicate::str::contains("keep this text"));
    polaris()
        .current_dir(dir.path())
        .args(["recall", "--key", "old.key"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No matching Polaris memory"));

    let after = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 1);
    assert_eq!(after[0]["id"], id);
    assert_eq!(after[0]["created_at"], created_at);
    assert_eq!(after[0]["key"], "new.key");
    assert_eq!(after[0]["lifecycle"], "state");
    assert_eq!(after[0]["text"], "keep this text");
}

#[test]
fn rename_rejects_missing_source_and_existing_destination_without_mutating() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "old.key", "--text", "old text"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "existing.key",
            "--text",
            "existing text",
        ])
        .assert()
        .success();
    let before = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();

    polaris()
        .current_dir(dir.path())
        .args(["rename", "missing", "new.key"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "source key `missing` does not exist",
        ));
    let after_missing = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();
    assert_eq!(after_missing, before);

    polaris()
        .current_dir(dir.path())
        .args(["rename", "old.key", "existing.key"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "destination key `existing.key` already exists",
        ));
    let after_existing = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();
    assert_eq!(after_existing, before);
}

#[test]
fn merge_creates_editable_draft_without_mutating_memory() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "keyA",
            "--text",
            "alpha text",
            "--lifecycle",
            "state",
        ])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "keyB", "--text", "beta text"])
        .assert()
        .success();
    let before = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();

    let output = polaris()
        .current_dir(dir.path())
        .args(["merge", "--into", "model.summary", "keyA", "keyB"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created merge draft"))
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    let draft_path = output
        .lines()
        .find_map(|line| line.strip_prefix("Path: "))
        .expect("draft path in output");
    let draft = fs::read_to_string(dir.path().join(draft_path)).expect("read draft");
    assert!(draft.contains("polaris-merge-draft-v1"));
    assert!(draft.contains("target: model.summary"));
    assert!(draft.contains("sources: keyA,keyB"));
    assert!(draft.contains("## Source keyA"));
    assert!(draft.contains("- lifecycle: state"));
    assert!(draft.contains("alpha text"));
    assert!(draft.contains("## Source keyB"));
    assert!(draft.contains("beta text"));
    assert!(draft.contains("## Merged Memory"));

    let after = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();
    assert_eq!(after, before);
}

#[test]
fn merge_draft_rejects_missing_sources_and_empty_target_without_mutating() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "keyA", "--text", "alpha text"])
        .assert()
        .success();
    let before = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();

    polaris()
        .current_dir(dir.path())
        .args(["merge", "--into", "model.summary", "keyA", "missing"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "source key `missing` does not exist",
        ));
    assert!(!dir.path().join(".polaris/maintenance").exists());
    let after_missing = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();
    assert_eq!(after_missing, before);

    polaris()
        .current_dir(dir.path())
        .args(["merge", "--into", "", "keyA"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("target key must not be empty"));
    let after_empty = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();
    assert_eq!(after_empty, before);
}

#[test]
fn merge_apply_writes_or_replaces_target_from_edited_draft() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    for (key, text) in [
        ("model.summary", "old summary"),
        ("keyA", "alpha text"),
        ("keyB", "beta text"),
    ] {
        polaris()
            .current_dir(dir.path())
            .args(["remember", "--key", key, "--text", text])
            .assert()
            .success();
    }

    let output = polaris()
        .current_dir(dir.path())
        .args(["merge", "--into", "model.summary", "keyA", "keyB"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    let draft_path = output
        .lines()
        .find_map(|line| line.strip_prefix("Path: "))
        .expect("draft path in output");
    fs::write(
        dir.path().join(draft_path),
        "<!-- polaris-merge-draft-v1\ntarget: model.summary\nsources: keyA,keyB\n-->\n\n# Polaris Merge Draft\n\n## Merged Memory\n\nmerged summary\n",
    )
    .expect("edit draft");

    polaris()
        .current_dir(dir.path())
        .args(["merge", "apply", draft_path, "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Applied merge draft to model.summary",
        ));

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--key", "model.summary"])
        .assert()
        .success()
        .stdout(predicate::str::contains("merged summary"))
        .stdout(predicate::str::contains("old summary").not());
    polaris()
        .current_dir(dir.path())
        .args(["recall", "--key", "keyA"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha text"));

    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 3);
    let target = records
        .iter()
        .find(|record| record["key"] == "model.summary")
        .expect("target record");
    assert_eq!(target["replacement_count"], 1);
}

#[test]
fn merge_apply_requires_confirmation_can_forget_sources_and_rejects_invalid_drafts() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    for (key, text) in [("keyA", "alpha text"), ("keyB", "beta text")] {
        polaris()
            .current_dir(dir.path())
            .args(["remember", "--key", key, "--text", text])
            .assert()
            .success();
    }
    let before = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();
    let bad_draft = dir.path().join(".polaris/maintenance/bad.md");
    fs::create_dir_all(bad_draft.parent().expect("bad draft parent")).unwrap();
    fs::write(&bad_draft, "not a merge draft").unwrap();

    polaris()
        .current_dir(dir.path())
        .args(["merge", "apply", ".polaris/maintenance/bad.md"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--yes"));
    assert_eq!(
        fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap(),
        before
    );

    polaris()
        .current_dir(dir.path())
        .args(["merge", "apply", ".polaris/maintenance/bad.md", "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be applied"));
    assert_eq!(
        fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap(),
        before
    );

    let output = polaris()
        .current_dir(dir.path())
        .args(["merge", "--into", "model.summary", "keyA", "keyB"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    let draft_path = output
        .lines()
        .find_map(|line| line.strip_prefix("Path: "))
        .expect("draft path in output");
    fs::write(
        dir.path().join(draft_path),
        "<!-- polaris-merge-draft-v1\ntarget: model.summary\nsources: keyA,keyB\n-->\n\n# Polaris Merge Draft\n\n## Merged Memory\n\nmerged summary\n",
    )
    .expect("edit draft");

    polaris()
        .current_dir(dir.path())
        .args(["merge", "apply", draft_path, "--yes", "--forget-sources"])
        .assert()
        .success();

    polaris()
        .current_dir(dir.path())
        .args(["recall", "--key", "model.summary"])
        .assert()
        .success()
        .stdout(predicate::str::contains("merged summary"));
    polaris()
        .current_dir(dir.path())
        .args(["recall", "--key", "keyA"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No matching Polaris memory"));
    polaris()
        .current_dir(dir.path())
        .args(["recall", "--key", "keyB"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No matching Polaris memory"));
}

#[test]
fn prune_suggest_reports_human_json_and_no_suggestions_without_mutating() {
    let dir = temp_workspace();
    init_workspace(dir.path());
    fs::write(
        dir.path().join(".polaris/memories.jsonl"),
        concat!(
            "{\"id\":\"old-state\",\"created_at\":\"2026-05-01T00:00:00Z\",\"kind\":\"inline\",\"key\":\"state.branch\",\"text\":\"branch\",\"lifecycle\":\"state\"}\n",
            "{\"id\":\"dup-a\",\"created_at\":\"2026-06-11T00:00:00Z\",\"kind\":\"inline\",\"key\":\"decision.a\",\"text\":\"same\",\"lifecycle\":\"durable\"}\n",
            "{\"id\":\"dup-b\",\"created_at\":\"2026-06-11T00:00:00Z\",\"kind\":\"inline\",\"key\":\"decision.b\",\"text\":\"same\",\"lifecycle\":\"durable\"}\n",
            "{\"id\":\"replaced\",\"created_at\":\"2026-06-01T00:00:00Z\",\"updated_at\":\"2026-06-11T00:00:00Z\",\"kind\":\"inline\",\"key\":\"goal\",\"text\":\"new\",\"lifecycle\":\"durable\",\"replacement_count\":2,\"replaced_from\":\"old-goal\"}\n",
        ),
    )
    .unwrap();
    let before = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();

    polaris()
        .current_dir(dir.path())
        .args(["prune", "--suggest"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Prune suggestions"))
        .stdout(predicate::str::contains("state.branch"))
        .stdout(predicate::str::contains("old volatile memory"))
        .stdout(predicate::str::contains("decision.a"))
        .stdout(predicate::str::contains("duplicate exact text"))
        .stdout(predicate::str::contains("goal"))
        .stdout(predicate::str::contains("replacement metadata"))
        .stdout(predicate::str::contains("Agent prompt:"))
        .stdout(predicate::str::contains(
            "You are reviewing Polaris prune suggestions",
        ))
        .stdout(predicate::str::contains("Known keys:"))
        .stdout(predicate::str::contains("state.branch"))
        .stdout(predicate::str::contains("same").not());
    assert_eq!(
        fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap(),
        before
    );

    let output = polaris()
        .current_dir(dir.path())
        .args(["prune", "--suggest", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let response: Value = serde_json::from_slice(&output).expect("prune json");
    let suggestions = response["suggestions"]
        .as_array()
        .expect("suggestion array");
    assert!(suggestions.iter().any(|suggestion| {
        suggestion["candidate_keys"]
            .as_array()
            .expect("candidate keys")
            .iter()
            .any(|key| key == "state.branch")
    }));
    assert!(
        suggestions
            .iter()
            .any(|suggestion| suggestion["reasons"][0] == "duplicate exact text")
    );
    assert!(
        response["prompt"]
            .as_str()
            .expect("prompt")
            .contains("polaris recall --key")
    );
    assert_eq!(response["prompt_context"]["status"]["memory_count"], 4);
    assert!(
        response["prompt_context"]["keys"]
            .as_array()
            .expect("keys")
            .iter()
            .any(|key| key == "state.branch")
    );
    assert!(
        response["prompt_context"]["citation_summaries"]
            .as_array()
            .expect("citation summaries")
            .iter()
            .any(|summary| summary["key"] == "state.branch"
                && summary["created_at"] == "2026-05-01T00:00:00Z"
                && summary["direct_cite_count"] == 0
                && summary["inherited_cite_count"] == 0)
    );

    let clean = temp_workspace();
    init_workspace(clean.path());
    polaris()
        .current_dir(clean.path())
        .args(["remember", "--key", "goal", "--text", "goal"])
        .assert()
        .success();
    polaris()
        .current_dir(clean.path())
        .args(["prune", "--suggest"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No prune suggestions are available",
        ))
        .stdout(predicate::str::contains("Agent prompt:"));
}

#[test]
fn compact_suggest_reports_human_json_and_no_suggestions_without_mutating() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    for (key, text) in [
        ("decision.storage", "private storage body"),
        ("decision.hooks", "hook decision"),
        ("state.branch", "branch state"),
    ] {
        polaris()
            .current_dir(dir.path())
            .args(["remember", "--key", key, "--text", text])
            .assert()
            .success();
    }
    let before = fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap();

    polaris()
        .current_dir(dir.path())
        .args(["compact", "--suggest"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Compact suggestions"))
        .stdout(predicate::str::contains("decision.storage"))
        .stdout(predicate::str::contains("decision.hooks"))
        .stdout(predicate::str::contains(
            "polaris merge --into decision.summary",
        ))
        .stdout(predicate::str::contains("Agent prompt:"))
        .stdout(predicate::str::contains(
            "You are reviewing Polaris compact suggestions",
        ))
        .stdout(predicate::str::contains("Known keys:"))
        .stdout(predicate::str::contains("private storage body").not());
    assert_eq!(
        fs::read_to_string(dir.path().join(".polaris/memories.jsonl")).unwrap(),
        before
    );

    let output = polaris()
        .current_dir(dir.path())
        .args(["compact", "--suggest", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let response: Value = serde_json::from_slice(&output).expect("compact json");
    let suggestions = response["suggestions"]
        .as_array()
        .expect("suggestion array");
    assert!(suggestions.iter().any(|suggestion| {
        suggestion["source_keys"]
            .as_array()
            .expect("source keys")
            .iter()
            .any(|key| key == "decision.storage")
            && suggestion["proposed_target_key"] == "decision.summary"
    }));
    assert!(
        response["prompt"]
            .as_str()
            .expect("prompt")
            .contains("polaris merge --into")
    );
    assert_eq!(response["prompt_context"]["status"]["memory_count"], 3);
    assert!(
        response["prompt_context"]["citation_summaries"]
            .as_array()
            .expect("citation summaries")
            .iter()
            .any(|summary| summary["key"] == "decision.storage"
                && summary["direct_cite_count"] == 0
                && summary["inherited_cite_count"] == 0)
    );
    assert!(
        response["prompt_context"]["keys"]
            .as_array()
            .expect("keys")
            .iter()
            .any(|key| key == "decision.storage")
    );

    let clean = temp_workspace();
    init_workspace(clean.path());
    polaris()
        .current_dir(clean.path())
        .args(["remember", "--key", "goal", "--text", "goal"])
        .assert()
        .success();
    polaris()
        .current_dir(clean.path())
        .args(["compact", "--suggest"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "No compact suggestions are available",
        ))
        .stdout(predicate::str::contains("Agent prompt:"));
}

#[test]
fn documentation_and_skill_explain_citation_semantics_and_batching() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(manifest.join("README.md")).expect("read README");
    let skill = fs::read_to_string(manifest.join("skills/codex/polaris/SKILL.md"))
        .expect("read bundled skill");

    for content in [&readme, &skill] {
        assert!(content.contains("polaris cite --key"));
        assert!(content.contains("polaris cite --keys"));
        assert!(content.contains("recall shows"));
        assert!(content.contains("touch marks"));
        assert!(content.contains("cite marks"));
        assert!(content.contains("Do not cite every recalled"));
        assert!(content.contains("--quiet"));
        assert!(content.contains("advisory"));
    }
}

#[test]
fn maintenance_prompts_use_workspace_config_with_supported_placeholders() {
    let dir = temp_workspace();
    init_workspace(dir.path());
    fs::write(
        dir.path().join(".polaris/config.toml"),
        "[maintenance.prompts]\nprune = \"Workspace prune {{suggestions}} {{status}} {{keys}} {{unknown}}\"\ncompact = \"Workspace compact {{suggestions}} {{status}} {{keys}}\"\n",
    )
    .expect("write workspace config");
    polaris()
        .current_dir(dir.path())
        .args([
            "remember",
            "--key",
            "state.branch",
            "--text",
            "private branch",
            "--lifecycle",
            "state",
        ])
        .assert()
        .success();

    polaris()
        .current_dir(dir.path())
        .args(["prune", "--suggest"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Workspace prune"))
        .stdout(predicate::str::contains("state.branch"))
        .stdout(predicate::str::contains("\"memory_count\": 1"))
        .stdout(predicate::str::contains("{{unknown}}"))
        .stdout(predicate::str::contains("private branch").not());

    polaris()
        .current_dir(dir.path())
        .args(["compact", "--suggest"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Workspace compact"))
        .stdout(predicate::str::contains("\"state.branch\""))
        .stdout(predicate::str::contains("private branch").not());
}

#[test]
fn maintenance_prompts_use_user_config_and_missing_keys_fall_back_to_defaults() {
    let dir = temp_workspace();
    let home = temp_workspace();
    init_workspace(dir.path());
    fs::create_dir_all(home.path().join(".polaris")).expect("create home polaris");
    fs::write(
        home.path().join(".polaris/config.toml"),
        "[maintenance.prompts]\nprune = \"User prune {{keys}}\"\n",
    )
    .expect("write user config");
    polaris()
        .current_dir(dir.path())
        .args(["remember", "--key", "goal", "--text", "private goal"])
        .assert()
        .success();

    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["prune", "--suggest"])
        .assert()
        .success()
        .stdout(predicate::str::contains("User prune"))
        .stdout(predicate::str::contains("goal"))
        .stdout(predicate::str::contains("private goal").not());

    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["compact", "--suggest"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "You are reviewing Polaris compact suggestions",
        ));
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

#[test]
fn install_codex_skill_script_uses_codex_home_by_default() {
    let codex_home = temp_workspace();

    run_install_skill_script([("CODEX_HOME", codex_home.path().to_str().unwrap())]);

    let installed_skill = codex_home.path().join("skills/polaris/SKILL.md");
    let bundled_skill = Path::new(env!("CARGO_MANIFEST_DIR")).join("skills/codex/polaris/SKILL.md");
    assert_eq!(
        fs::read_to_string(installed_skill).expect("installed SKILL.md"),
        fs::read_to_string(bundled_skill).expect("bundled SKILL.md")
    );
}

#[test]
fn install_codex_skill_script_prefers_existing_codex_app_home() {
    let home = temp_workspace();
    fs::create_dir(home.path().join(".codex-app")).expect("create Codex app home");

    run_install_skill_script([("HOME", home.path().to_str().unwrap())]);

    let installed_skill = home.path().join(".codex-app/skills/polaris/SKILL.md");
    let bundled_skill = Path::new(env!("CARGO_MANIFEST_DIR")).join("skills/codex/polaris/SKILL.md");
    assert_eq!(
        fs::read_to_string(installed_skill).expect("installed SKILL.md"),
        fs::read_to_string(bundled_skill).expect("bundled SKILL.md")
    );
    assert!(!home.path().join("skills/polaris/SKILL.md").exists());
}

#[test]
fn install_codex_skill_script_respects_override_and_replaces_existing_skill() {
    let skills_dir = temp_workspace();
    let stale_skill = skills_dir.path().join("polaris");
    fs::create_dir_all(&stale_skill).expect("create stale skill dir");
    fs::write(stale_skill.join("SKILL.md"), "stale skill").expect("write stale skill");
    fs::write(stale_skill.join("local-only.txt"), "remove me").expect("write stale extra file");

    run_install_skill_script([(
        "POLARIS_CODEX_SKILLS_DIR",
        skills_dir.path().to_str().unwrap(),
    )]);

    let installed_skill = skills_dir.path().join("polaris/SKILL.md");
    let bundled_skill = Path::new(env!("CARGO_MANIFEST_DIR")).join("skills/codex/polaris/SKILL.md");
    assert_eq!(
        fs::read_to_string(installed_skill).expect("installed SKILL.md"),
        fs::read_to_string(bundled_skill).expect("bundled SKILL.md")
    );
    assert!(!stale_skill.join("local-only.txt").exists());
}

#[test]
fn polaris_root_unset_uses_cwd_dot_polaris() {
    let dir = temp_workspace();

    polaris()
        .current_dir(dir.path())
        .env_remove("POLARIS_ROOT")
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            format!("Initialized Polaris at {}/.polaris", dir.path().display()).as_str(),
        ));

    assert!(dir.path().join(".polaris/state.json").is_file());
    assert!(dir.path().join(".polaris/memories.jsonl").is_file());
    assert!(dir.path().join(".polaris/docs").is_dir());

    let output = polaris()
        .current_dir(dir.path())
        .env_remove("POLARIS_ROOT")
        .args(["status", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status: Value = serde_json::from_slice(&output).expect("status json");
    assert_eq!(
        status["root"].as_str().unwrap(),
        format!("{}/.polaris", dir.path().display())
    );
}

#[test]
fn polaris_root_absolute_selects_custom_root_and_keeps_cwd_store_untouched() {
    let dir = temp_workspace();
    let custom_root = dir.path().join("custom-store");
    let custom_root_string = custom_root.display().to_string();

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            format!("Initialized Polaris at {custom_root_string}").as_str(),
        ));

    assert!(custom_root.join("state.json").is_file());
    assert!(custom_root.join("memories.jsonl").is_file());
    assert!(custom_root.join("docs").is_dir());
    assert!(!dir.path().join(".polaris").exists());

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["remember", "--key", "goal", "--text", "isolated goal"])
        .assert()
        .success();
    assert!(!dir.path().join(".polaris/memories.jsonl").exists());

    let output = polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["status", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let status: Value = serde_json::from_slice(&output).expect("status json");
    assert_eq!(status["root"].as_str().unwrap(), custom_root_string);
    assert_eq!(status["memory_count"], 1);
    assert_eq!(status["initialized"], true);
}

#[test]
fn polaris_root_rejects_empty_value() {
    let dir = temp_workspace();

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", "")
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("POLARIS_ROOT"))
        .stderr(predicate::str::contains("empty"));

    assert!(!dir.path().join(".polaris").exists());
}

#[test]
fn polaris_root_rejects_relative_value() {
    let dir = temp_workspace();

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", "relative/store")
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("POLARIS_ROOT"))
        .stderr(predicate::str::contains("absolute"));

    assert!(!dir.path().join(".polaris").exists());
    assert!(!dir.path().join("relative").exists());
}

#[test]
fn polaris_root_absolute_init_creates_missing_directory() {
    let dir = temp_workspace();
    let fresh_root = dir.path().join("nested/fresh-store");
    let fresh_root_string = fresh_root.display().to_string();

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &fresh_root_string)
        .arg("init")
        .assert()
        .success();

    assert!(fresh_root.join("state.json").is_file());
    assert!(fresh_root.join("memories.jsonl").is_file());
    assert!(fresh_root.join("docs").is_dir());
}

#[test]
fn polaris_root_absolute_init_preserves_existing_records() {
    let dir = temp_workspace();
    let custom_root = dir.path().join("shared-store");
    let custom_root_string = custom_root.display().to_string();

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["remember", "--key", "goal", "--text", "carry me"])
        .assert()
        .success();

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success();

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("recall")
        .assert()
        .success()
        .stdout(predicate::str::contains("carry me"));
}

#[test]
fn note_path_is_absolute_in_default_and_override_modes() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    let output = polaris()
        .current_dir(dir.path())
        .env_remove("POLARIS_ROOT")
        .args(["note", "create", "--title", "Default note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let default_path = path_from_output(&output, "Path: ");
    assert!(default_path.starts_with(dir.path().to_str().unwrap()));
    assert!(Path::new(&default_path).is_absolute());
    assert!(Path::new(&default_path).is_file());

    let custom_root = dir.path().join("override-store");
    let custom_root_string = custom_root.display().to_string();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success();

    let output = polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["note", "create", "--title", "Override note"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let override_path = path_from_output(&output, "Path: ");
    assert!(Path::new(&override_path).is_absolute());
    assert!(Path::new(&override_path).starts_with(&custom_root));
    assert!(Path::new(&override_path).is_file());
}

#[test]
fn hook_session_start_uses_payload_cwd_when_root_unset() {
    let dir = temp_workspace();
    let home = temp_workspace();
    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["remember", "--key", "secret", "--text", "payload memory"])
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
        .env_remove("POLARIS_ROOT")
        .args(["hook", "session-start"])
        .write_stdin(compact_input.as_str())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let context = hook_context(&output);
    assert!(context.contains("payload memory"));
}

#[test]
fn hook_session_start_uses_absolute_root_override_over_payload_cwd() {
    let dir = temp_workspace();
    let home = temp_workspace();
    let custom_root = dir.path().join("hook-store");
    let custom_root_string = custom_root.display().to_string();

    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["remember", "--key", "secret", "--text", "override memory"])
        .assert()
        .success();
    fs::write(
        custom_root.join("config.toml"),
        "[hooks]\nrecall_prompt = \"Before\\n{{recall}}\\nAfter\"\n",
    )
    .expect("write root config");

    let compact_input = serde_json::json!({
        "hook_event_name": "SessionStart",
        "source": "compact",
        "cwd": dir.path(),
    })
    .to_string();
    let output = polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["hook", "session-start"])
        .write_stdin(compact_input.as_str())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let context = hook_context(&output);
    assert!(context.contains("override memory"));
    assert!(!context.contains("payload memory"));
    assert!(!dir.path().join(".polaris/memories.jsonl").exists());
}

#[test]
fn fallback_hooks_share_pending_state_under_absolute_root_override() {
    let dir = temp_workspace();
    let home = temp_workspace();
    let custom_root = dir.path().join("fallback-store");
    let custom_root_string = custom_root.display().to_string();

    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["remember", "--key", "secret", "--text", "fallback memory"])
        .assert()
        .success();
    fs::write(
        custom_root.join("config.toml"),
        "[hooks]\nrecall_prompt = \"Before\\n{{recall}}\\nAfter\"\n",
    )
    .expect("write root config");

    let post_compact_input = serde_json::json!({
        "hook_event_name": "PostCompact",
        "cwd": dir.path(),
    })
    .to_string();
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["hook", "post-compact"])
        .write_stdin(post_compact_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let post_tool_use_input = serde_json::json!({
        "hook_event_name": "PostToolUse",
        "cwd": dir.path(),
    })
    .to_string();
    let output = polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["hook", "post-tool-use"])
        .write_stdin(post_tool_use_input.as_str())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let context = hook_context(&output);
    assert!(context.contains("fallback memory"));

    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["hook", "post-tool-use"])
        .write_stdin(post_tool_use_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    assert!(!dir.path().join(".polaris/hook-state.json").exists());
}

#[test]
fn root_config_takes_precedence_over_user_config_under_absolute_root_override() {
    let dir = temp_workspace();
    let home = temp_workspace();
    let custom_root = dir.path().join("config-store");
    let custom_root_string = custom_root.display().to_string();

    fs::create_dir_all(home.path().join(".polaris")).expect("user polaris home");
    fs::write(
        home.path().join(".polaris/config.toml"),
        "[hooks]\nrecall_prompt = \"USER_PROMPT\"\n",
    )
    .unwrap();
    fs::create_dir_all(&custom_root).expect("custom root");
    fs::write(
        custom_root.join("config.toml"),
        "[hooks]\nrecall_prompt = \"ROOT_PROMPT\"\n",
    )
    .unwrap();

    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["remember", "--key", "secret", "--text", "config memory"])
        .assert()
        .success();

    let compact_input = serde_json::json!({
        "hook_event_name": "SessionStart",
        "source": "compact",
        "cwd": dir.path(),
    })
    .to_string();
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["hook", "session-start"])
        .write_stdin(compact_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("ROOT_PROMPT"))
        .stdout(predicate::str::contains("USER_PROMPT").not());
}

#[test]
fn store_backed_commands_write_no_cwd_files_under_absolute_root_override() {
    let dir = temp_workspace();
    let custom_root = dir.path().join("isolated-store");
    let custom_root_string = custom_root.display().to_string();

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["remember", "--key", "goal", "--text", "private goal"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args([
            "remember",
            "--key",
            "goal",
            "--replace",
            "--text",
            "private goal v2",
        ])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args([
            "remember",
            "--key",
            "goal",
            "--replace",
            "--dry-run",
            "--text",
            "v3 draft",
        ])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["touch", "--key", "goal"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["cite", "--key", "goal", "--quiet"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["note", "create", "--title", "Override note"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["merge", "--into", "merged", "goal"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["prune", "--suggest"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["compact", "--suggest"])
        .assert()
        .success();

    assert!(!dir.path().join(".polaris").exists());
    assert!(custom_root.join("state.json").is_file());
    assert!(custom_root.join("memories.jsonl").is_file());
    assert!(custom_root.join("replacement-history.jsonl").is_file());
    assert!(custom_root.join("citations.jsonl").is_file());
    assert!(custom_root.join("memories.lock").is_file());
    assert!(custom_root.join("docs").is_dir());
    assert!(custom_root.join("maintenance").is_dir());
}

#[test]
fn hook_falls_back_to_process_cwd_when_payload_cwd_is_missing() {
    let dir = temp_workspace();
    let home = temp_workspace();
    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .args(["remember", "--key", "secret", "--text", "fallback memory"])
        .assert()
        .success();
    fs::write(
        dir.path().join(".polaris/config.toml"),
        "[hooks]\nrecall_prompt = \"Before\\n{{recall}}\\nAfter\"\n",
    )
    .expect("write workspace config");

    let input_without_cwd = serde_json::json!({
        "hook_event_name": "SessionStart",
        "source": "compact",
    })
    .to_string();
    let output = polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env_remove("POLARIS_ROOT")
        .args(["hook", "session-start"])
        .write_stdin(input_without_cwd.as_str())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let context = hook_context(&output);
    assert!(context.contains("fallback memory"));
}

#[test]
fn hook_uses_absolute_root_override_when_payload_cwd_is_missing() {
    let dir = temp_workspace();
    let home = temp_workspace();
    let other_cwd = temp_workspace();
    let custom_root = dir.path().join("override-store");
    let custom_root_string = custom_root.display().to_string();

    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["remember", "--key", "secret", "--text", "override memory"])
        .assert()
        .success();
    fs::write(
        custom_root.join("config.toml"),
        "[hooks]\nrecall_prompt = \"Before\\n{{recall}}\\nAfter\"\n",
    )
    .expect("write root config");

    let input_without_cwd = serde_json::json!({
        "hook_event_name": "SessionStart",
        "source": "compact",
    })
    .to_string();
    let output = polaris()
        .current_dir(other_cwd.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["hook", "session-start"])
        .write_stdin(input_without_cwd.as_str())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let context = hook_context(&output);
    assert!(context.contains("override memory"));
}

#[test]
fn root_maintenance_prompts_take_precedence_under_absolute_root_override() {
    let dir = temp_workspace();
    let home = temp_workspace();
    let custom_root = dir.path().join("maint-store");
    let custom_root_string = custom_root.display().to_string();

    fs::create_dir_all(home.path().join(".polaris")).expect("user polaris home");
    fs::write(
        home.path().join(".polaris/config.toml"),
        r#"[maintenance.prompts]
prune = "USER_PRUNE_PROMPT"
compact = "USER_COMPACT_PROMPT"
"#,
    )
    .unwrap();
    fs::create_dir_all(&custom_root).expect("custom root");
    fs::write(
        custom_root.join("config.toml"),
        r#"[maintenance.prompts]
prune = "ROOT_PRUNE_PROMPT"
compact = "ROOT_COMPACT_PROMPT"
"#,
    )
    .unwrap();

    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["remember", "--key", "stale.state", "--text", "stale"])
        .assert()
        .success();

    let output = polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["prune", "--suggest"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("ROOT_PRUNE_PROMPT"));
    assert!(!output.contains("USER_PRUNE_PROMPT"));

    let output = polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["compact", "--suggest"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("ROOT_COMPACT_PROMPT"));
    assert!(!output.contains("USER_COMPACT_PROMPT"));
}

#[test]
fn user_maintenance_prompts_apply_when_root_config_is_absent_under_override() {
    let dir = temp_workspace();
    let home = temp_workspace();
    let custom_root = dir.path().join("maint-store");
    let custom_root_string = custom_root.display().to_string();

    fs::create_dir_all(home.path().join(".polaris")).expect("user polaris home");
    fs::write(
        home.path().join(".polaris/config.toml"),
        r#"[maintenance.prompts]
prune = "USER_PRUNE_PROMPT"
compact = "USER_COMPACT_PROMPT"
"#,
    )
    .unwrap();

    polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success();

    let output = polaris()
        .current_dir(dir.path())
        .env("HOME", home.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["prune", "--suggest"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).unwrap();
    assert!(output.contains("USER_PRUNE_PROMPT"));
}

#[test]
fn note_path_recorded_in_jsonl_is_absolute() {
    let dir = temp_workspace();
    init_workspace(dir.path());

    let output = polaris()
        .current_dir(dir.path())
        .env_remove("POLARIS_ROOT")
        .args(["note", "create", "--title", "Default path test"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let printed = path_from_output(&output, "Path: ");
    let records = assert_jsonl_records(&dir.path().join(".polaris/memories.jsonl"), 1);
    let recorded = records[0]["path"].as_str().expect("note path");
    assert!(Path::new(recorded).is_absolute());
    assert_eq!(recorded, printed);

    let custom_root = dir.path().join("absolute-store");
    let custom_root_string = custom_root.display().to_string();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success();

    let output = polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["note", "create", "--title", "Override path test"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let printed_override = path_from_output(&output, "Path: ");
    let records = assert_jsonl_records(&custom_root.join("memories.jsonl"), 1);
    let recorded_override = records[0]["path"].as_str().expect("note path");
    assert!(Path::new(recorded_override).is_absolute());
    assert_eq!(recorded_override, printed_override);
    assert!(Path::new(recorded_override).starts_with(&custom_root));
}

#[test]
fn replacement_draft_can_be_edited_and_applied_under_absolute_root_override() {
    let dir = temp_workspace();
    let custom_root = dir.path().join("replace-store");
    let custom_root_string = custom_root.display().to_string();

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args([
            "remember",
            "--key",
            "goal",
            "--text",
            "old goal",
            "--lifecycle",
            "state",
        ])
        .assert()
        .success();
    let initial = assert_jsonl_records(&custom_root.join("memories.jsonl"), 1);
    let initial_id = initial[0]["id"].as_str().expect("initial id").to_string();
    let created_at = initial[0]["created_at"].clone();

    let output = polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args([
            "remember",
            "--key",
            "goal",
            "--replace",
            "--dry-run",
            "--text",
            "draft goal",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let draft_path = path_from_output(&output, "Path: ");
    assert!(Path::new(&draft_path).is_absolute());
    let draft = fs::read_to_string(&draft_path).expect("read replace draft");
    assert!(draft.contains("polaris-replace-draft-v1"));
    let edited = draft.replace("draft goal", "edited override goal");
    fs::write(&draft_path, edited).expect("write edited replace draft");

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["replace", "apply", &draft_path, "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Applied replacement draft to goal",
        ));

    let records = assert_jsonl_records(&custom_root.join("memories.jsonl"), 1);
    assert_eq!(records[0]["key"], "goal");
    assert_eq!(records[0]["text"], "edited override goal");
    assert_eq!(records[0]["created_at"], created_at);
    assert_eq!(records[0]["lifecycle"], "state");
    assert_eq!(records[0]["replacement_count"], 1);
    assert_eq!(records[0]["replaced_from"], initial_id);

    let history = assert_jsonl_records(&custom_root.join("replacement-history.jsonl"), 1);
    assert_eq!(history[0]["id"], initial_id);
    assert_eq!(history[0]["text"], "old goal");

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["diff", "--key", "goal"])
        .assert()
        .success()
        .stdout(predicate::str::contains("-old goal"))
        .stdout(predicate::str::contains("+edited override goal"));

    assert!(!dir.path().join(".polaris").exists());
}

#[test]
fn merge_draft_can_be_edited_and_applied_under_absolute_root_override() {
    let dir = temp_workspace();
    let custom_root = dir.path().join("merge-store");
    let custom_root_string = custom_root.display().to_string();

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .arg("init")
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["remember", "--key", "a", "--text", "alpha body"])
        .assert()
        .success();
    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["remember", "--key", "b", "--text", "beta body"])
        .assert()
        .success();

    let output = polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["merge", "--into", "ab.summary", "a", "b"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let draft_path = path_from_output(&output, "Path: ");
    assert!(Path::new(&draft_path).is_absolute());
    let draft = fs::read_to_string(&draft_path).expect("read merge draft");
    assert!(draft.contains("polaris-merge-draft-v1"));
    let edited = draft.replace(
        "From a:\nalpha body\n\nFrom b:\nbeta body\n\n",
        "Merged override: alpha + beta\n",
    );
    fs::write(&draft_path, edited).expect("write edited merge draft");

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["merge", "apply", &draft_path, "--yes", "--forget-sources"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Applied merge draft to ab.summary",
        ));

    let records = assert_jsonl_records(&custom_root.join("memories.jsonl"), 1);
    assert_eq!(records[0]["key"], "ab.summary");
    assert_eq!(records[0]["text"], "Merged override: alpha + beta");

    polaris()
        .current_dir(dir.path())
        .env("POLARIS_ROOT", &custom_root_string)
        .args(["list", "--keys"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ab.summary"))
        .stdout(predicate::str::contains("a\n").not())
        .stdout(predicate::str::contains("b\n").not());

    assert!(!dir.path().join(".polaris").exists());
}

fn run_install_skill_script<const N: usize>(envs: [(&str, &str); N]) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = manifest_dir.join("scripts/install-codex-skill.sh");
    let mut command = StdCommand::new(&script);
    command
        .current_dir(manifest_dir)
        .env_remove("CODEX_HOME")
        .env_remove("POLARIS_CODEX_SKILLS_DIR");
    for (key, value) in envs {
        command.env(key, value);
    }

    let output = command.output().expect("run install skill script");
    assert!(
        output.status.success(),
        "install script failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
