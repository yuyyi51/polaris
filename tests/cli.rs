use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::Path;
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
    let compact_input = serde_json::json!({
        "hook_event_name": "SessionStart",
        "source": "compact",
        "cwd": dir.path(),
    })
    .to_string();

    polaris()
        .current_dir(dir.path())
        .args(["hook", "session-start"])
        .write_stdin(compact_input.as_str())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    init_workspace(dir.path());
    polaris()
        .current_dir(dir.path())
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
        .args(["hook", "session-start"])
        .write_stdin(startup_input)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let output = polaris()
        .current_dir(dir.path())
        .args(["hook", "session-start"])
        .write_stdin(compact_input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let hook: Value = serde_json::from_slice(&output).expect("hook json");
    let context = hook["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("additional context");
    assert_eq!(hook["hookSpecificOutput"]["hookEventName"], "SessionStart");
    assert!(context.contains("polaris recall"));
    assert!(!context.contains("secret memory"));
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
