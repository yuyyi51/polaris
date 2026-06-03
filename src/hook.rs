use crate::storage::PolarisStore;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const RECALL_INSTRUCTION: &str = "Polaris has saved workspace context for this project. Run `polaris recall` immediately before doing any more work.";
const RECALL_PLACEHOLDER: &str = "{{recall}}";
const CONFIG_FILE: &str = "config.toml";

/// Selects which host adapter the hook commands should target.
///
/// `Codex` keeps the historical Polaris hook behavior (stdout JSON wrapped in
/// `hookSpecificOutput.hookEventName`, Codex `SessionStart`+`source=compact`
/// semantics, post-compact pending state fallback). `Traecli` switches to the
/// TraeCLI / Coco host adapter, which emits a `hookSpecificOutput` payload
/// without the `hookEventName` wrapper and only triggers recall on the
/// dedicated `PostCompact` event.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, clap::ValueEnum)]
#[clap(rename_all = "lower")]
pub enum HookTarget {
    #[default]
    Codex,
    Traecli,
}

#[derive(Deserialize)]
struct HookInput {
    hook_event_name: Option<String>,
    source: Option<String>,
    cwd: Option<PathBuf>,
}

/// Host-shaped hook stdout.
///
/// Codex requires `hookSpecificOutput.hookEventName`; TraeCLI does not include
/// it and only consumes `hookSpecificOutput.additionalContext`.
#[derive(Debug)]
pub enum HookOutput {
    Codex {
        hook_event_name: String,
        additional_context: String,
    },
    Traecli {
        additional_context: String,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CodexHookOutput<'a> {
    hook_specific_output: CodexHookSpecificOutput<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CodexHookSpecificOutput<'a> {
    hook_event_name: &'a str,
    additional_context: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TraecliHookOutput<'a> {
    hook_specific_output: TraecliHookSpecificOutput<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TraecliHookSpecificOutput<'a> {
    additional_context: &'a str,
}

impl HookOutput {
    pub fn to_json_string(&self) -> Result<String> {
        match self {
            HookOutput::Codex {
                hook_event_name,
                additional_context,
            } => Ok(serde_json::to_string(&CodexHookOutput {
                hook_specific_output: CodexHookSpecificOutput {
                    hook_event_name,
                    additional_context,
                },
            })?),
            HookOutput::Traecli { additional_context } => {
                Ok(serde_json::to_string(&TraecliHookOutput {
                    hook_specific_output: TraecliHookSpecificOutput { additional_context },
                })?)
            }
        }
    }
}

#[derive(Deserialize)]
struct PolarisConfig {
    hooks: Option<HookConfig>,
}

#[derive(Deserialize)]
struct HookConfig {
    recall_prompt: Option<String>,
}

pub fn session_start_output<F>(
    target: HookTarget,
    input: &str,
    store_for: F,
) -> Result<Option<HookOutput>>
where
    F: FnOnce(PathBuf) -> Result<PolarisStore>,
{
    match target {
        HookTarget::Codex => codex_session_start_output(input, store_for),
        HookTarget::Traecli => Ok(None),
    }
}

pub fn post_compact<F>(target: HookTarget, input: &str, store_for: F) -> Result<Option<HookOutput>>
where
    F: FnOnce(PathBuf) -> Result<PolarisStore>,
{
    match target {
        HookTarget::Codex => {
            codex_post_compact(input, store_for)?;
            Ok(None)
        }
        HookTarget::Traecli => traecli_post_compact_output(input, store_for),
    }
}

pub fn post_tool_use_output<F>(
    target: HookTarget,
    input: &str,
    store_for: F,
) -> Result<Option<HookOutput>>
where
    F: FnOnce(PathBuf) -> Result<PolarisStore>,
{
    match target {
        HookTarget::Codex => codex_post_tool_use_output(input, store_for),
        HookTarget::Traecli => Ok(None),
    }
}

fn codex_session_start_output<F>(input: &str, store_for: F) -> Result<Option<HookOutput>>
where
    F: FnOnce(PathBuf) -> Result<PolarisStore>,
{
    let input: HookInput = serde_json::from_str(input)?;
    if input.hook_event_name.as_deref() != Some("SessionStart")
        || input.source.as_deref() != Some("compact")
    {
        return Ok(None);
    }

    let store = store_for(input.cwd()?)?;
    if !store.is_initialized() || store.memory_count()? == 0 {
        return Ok(None);
    }

    Ok(Some(codex_recall_output("SessionStart", &store)?))
}

fn codex_post_compact<F>(input: &str, store_for: F) -> Result<()>
where
    F: FnOnce(PathBuf) -> Result<PolarisStore>,
{
    let input: HookInput = serde_json::from_str(input)?;
    if input.hook_event_name.as_deref() != Some("PostCompact") {
        return Ok(());
    }

    let store = store_for(input.cwd()?)?;
    if store.is_initialized() {
        store.mark_compact_recall_pending()?;
    }
    Ok(())
}

fn codex_post_tool_use_output<F>(input: &str, store_for: F) -> Result<Option<HookOutput>>
where
    F: FnOnce(PathBuf) -> Result<PolarisStore>,
{
    let input: HookInput = serde_json::from_str(input)?;
    if input.hook_event_name.as_deref() != Some("PostToolUse") {
        return Ok(None);
    }

    let store = store_for(input.cwd()?)?;
    if !store.is_initialized()
        || !store.consume_compact_recall_pending()?
        || store.memory_count()? == 0
    {
        return Ok(None);
    }

    Ok(Some(codex_recall_output("PostToolUse", &store)?))
}

fn traecli_post_compact_output<F>(input: &str, store_for: F) -> Result<Option<HookOutput>>
where
    F: FnOnce(PathBuf) -> Result<PolarisStore>,
{
    let input: HookInput = serde_json::from_str(input)?;
    if input.hook_event_name.as_deref() != Some("PostCompact") {
        return Ok(None);
    }

    let store = store_for(input.cwd()?)?;
    if !store.is_initialized() || store.memory_count()? == 0 {
        return Ok(None);
    }

    Ok(Some(HookOutput::Traecli {
        additional_context: recall_context(&store)?,
    }))
}

impl HookInput {
    fn cwd(self) -> Result<PathBuf> {
        match self.cwd {
            Some(cwd) => Ok(cwd),
            None => Ok(std::env::current_dir()?),
        }
    }
}

fn codex_recall_output(hook_event_name: &str, store: &PolarisStore) -> Result<HookOutput> {
    Ok(HookOutput::Codex {
        hook_event_name: hook_event_name.to_string(),
        additional_context: recall_context(store)?,
    })
}

fn recall_context(store: &PolarisStore) -> Result<String> {
    let prompt = configured_recall_prompt(store)?.unwrap_or_else(|| RECALL_INSTRUCTION.to_string());
    if prompt.contains(RECALL_PLACEHOLDER) {
        Ok(prompt.replace(RECALL_PLACEHOLDER, &store.recall()?))
    } else {
        Ok(prompt)
    }
}

fn configured_recall_prompt(store: &PolarisStore) -> Result<Option<String>> {
    let workspace_config = store.root().join(CONFIG_FILE);
    if config_exists(&workspace_config)? {
        return read_recall_prompt(&workspace_config);
    }

    let Some(home) = std::env::var_os("HOME") else {
        return Ok(None);
    };
    let user_config = PathBuf::from(home).join(".polaris").join(CONFIG_FILE);
    if config_exists(&user_config)? {
        return read_recall_prompt(&user_config);
    }

    Ok(None)
}

fn config_exists(path: &Path) -> Result<bool> {
    path.try_exists()
        .with_context(|| format!("failed to inspect Polaris config {}", path.display()))
}

fn read_recall_prompt(path: &Path) -> Result<Option<String>> {
    let config = fs::read_to_string(path)
        .with_context(|| format!("failed to read Polaris config {}", path.display()))?;
    let config: PolarisConfig = toml::from_str(&config)
        .with_context(|| format!("failed to parse Polaris config {}", path.display()))?;
    Ok(config.hooks.and_then(|hooks| hooks.recall_prompt))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{MemoryInput, PolarisStore};
    use serde_json::Value;
    use tempfile::TempDir;

    fn fresh_store(workspace: &TempDir) -> PolarisStore {
        PolarisStore::from_workspace(workspace.path().to_path_buf()).expect("store")
    }

    fn init_with_memory(workspace: &TempDir) -> PolarisStore {
        let store = fresh_store(workspace);
        store.init().expect("init");
        store
            .remember(
                MemoryInput {
                    key: "goal".to_string(),
                    title: Some("Goal".to_string()),
                    text: "task goal".to_string(),
                },
                false,
            )
            .expect("remember");
        store
    }

    fn store_for(workspace: &TempDir) -> impl FnOnce(PathBuf) -> Result<PolarisStore> + '_ {
        |_cwd: PathBuf| PolarisStore::from_workspace(workspace.path().to_path_buf())
    }

    fn input_json(event: &str, workspace: &TempDir) -> String {
        serde_json::json!({
            "hook_event_name": event,
            "cwd": workspace.path(),
        })
        .to_string()
    }

    #[test]
    fn traecli_post_compact_emits_additional_context_without_hook_event_name() {
        let workspace = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        // SAFETY: setting HOME for the duration of the test is acceptable in
        // single-process unit tests; tests of hook prompts go through env vars.
        let prev_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", home.path());
        }
        init_with_memory(&workspace);

        let output = post_compact(
            HookTarget::Traecli,
            &input_json("PostCompact", &workspace),
            store_for(&workspace),
        )
        .expect("post compact")
        .expect("output");
        let serialized = output.to_json_string().expect("serialize");
        let parsed: Value = serde_json::from_str(&serialized).unwrap();
        assert!(parsed["hookSpecificOutput"]["additionalContext"].is_string());
        assert!(parsed["hookSpecificOutput"]["hookEventName"].is_null());

        // pending state file MUST NOT be written.
        assert!(!workspace.path().join(".polaris/hook-state.json").exists());

        match prev_home {
            Some(value) => unsafe { std::env::set_var("HOME", value) },
            None => unsafe { std::env::remove_var("HOME") },
        }
    }

    #[test]
    fn traecli_post_compact_quiet_when_workspace_uninitialized() {
        let workspace = tempfile::tempdir().unwrap();
        let result = post_compact(
            HookTarget::Traecli,
            &input_json("PostCompact", &workspace),
            store_for(&workspace),
        )
        .expect("post compact");
        assert!(result.is_none());
        assert!(!workspace.path().join(".polaris/hook-state.json").exists());
    }

    #[test]
    fn traecli_post_compact_quiet_when_no_memory() {
        let workspace = tempfile::tempdir().unwrap();
        let store = fresh_store(&workspace);
        store.init().expect("init");
        let result = post_compact(
            HookTarget::Traecli,
            &input_json("PostCompact", &workspace),
            store_for(&workspace),
        )
        .expect("post compact");
        assert!(result.is_none());
        assert!(!workspace.path().join(".polaris/hook-state.json").exists());
    }

    #[test]
    fn traecli_post_compact_quiet_on_mismatched_event() {
        let workspace = tempfile::tempdir().unwrap();
        init_with_memory(&workspace);
        let result = post_compact(
            HookTarget::Traecli,
            &input_json("PreToolUse", &workspace),
            store_for(&workspace),
        )
        .expect("post compact");
        assert!(result.is_none());
        assert!(!workspace.path().join(".polaris/hook-state.json").exists());
    }

    #[test]
    fn traecli_session_start_is_noop() {
        let workspace = tempfile::tempdir().unwrap();
        init_with_memory(&workspace);
        let payload = serde_json::json!({
            "hook_event_name": "SessionStart",
            "source": "compact",
            "cwd": workspace.path(),
        })
        .to_string();
        let result = session_start_output(HookTarget::Traecli, &payload, store_for(&workspace))
            .expect("session start");
        assert!(result.is_none());
        assert!(!workspace.path().join(".polaris/hook-state.json").exists());
    }

    #[test]
    fn traecli_post_tool_use_is_noop() {
        let workspace = tempfile::tempdir().unwrap();
        init_with_memory(&workspace);
        let result = post_tool_use_output(
            HookTarget::Traecli,
            &input_json("PostToolUse", &workspace),
            store_for(&workspace),
        )
        .expect("post tool use");
        assert!(result.is_none());
        assert!(!workspace.path().join(".polaris/hook-state.json").exists());
    }

    #[test]
    fn codex_default_session_start_preserves_hook_event_name_byte_layout() {
        let workspace = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        let prev_home = std::env::var_os("HOME");
        unsafe {
            std::env::set_var("HOME", home.path());
        }
        init_with_memory(&workspace);

        let payload = serde_json::json!({
            "hook_event_name": "SessionStart",
            "source": "compact",
            "cwd": workspace.path(),
        })
        .to_string();
        let output = session_start_output(HookTarget::Codex, &payload, store_for(&workspace))
            .expect("session start")
            .expect("hook output");
        let serialized = output.to_json_string().expect("serialize");
        // Byte-level structure: same field order as historical Codex output.
        assert_eq!(
            serialized,
            format!(
                r#"{{"hookSpecificOutput":{{"hookEventName":"SessionStart","additionalContext":{}}}}}"#,
                serde_json::to_string(RECALL_INSTRUCTION).unwrap()
            )
        );

        match prev_home {
            Some(value) => unsafe { std::env::set_var("HOME", value) },
            None => unsafe { std::env::remove_var("HOME") },
        }
    }

    #[test]
    fn codex_post_compact_still_writes_pending_state() {
        let workspace = tempfile::tempdir().unwrap();
        init_with_memory(&workspace);
        let payload = input_json("PostCompact", &workspace);
        let result =
            post_compact(HookTarget::Codex, &payload, store_for(&workspace)).expect("post compact");
        assert!(result.is_none());
        assert!(workspace.path().join(".polaris/hook-state.json").exists());
    }
}
