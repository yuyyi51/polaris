use crate::storage::PolarisStore;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const RECALL_INSTRUCTION: &str = "Polaris has saved workspace context for this project. Run `polaris recall` immediately before doing any more work.";
const RECALL_PLACEHOLDER: &str = "{{recall}}";
const CONFIG_FILE: &str = "config.toml";

#[derive(Deserialize)]
struct HookInput {
    hook_event_name: Option<String>,
    source: Option<String>,
    cwd: Option<PathBuf>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookOutput {
    hook_specific_output: HookSpecificOutput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HookSpecificOutput {
    hook_event_name: String,
    additional_context: String,
}

#[derive(Deserialize)]
struct PolarisConfig {
    hooks: Option<HookConfig>,
}

#[derive(Deserialize)]
struct HookConfig {
    recall_prompt: Option<String>,
}

pub fn session_start_output<F>(input: &str, store_for: F) -> Result<Option<HookOutput>>
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

    Ok(Some(recall_output("SessionStart", &store)?))
}

pub fn post_compact<F>(input: &str, store_for: F) -> Result<()>
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

pub fn post_tool_use_output<F>(input: &str, store_for: F) -> Result<Option<HookOutput>>
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

    Ok(Some(recall_output("PostToolUse", &store)?))
}

impl HookInput {
    fn cwd(self) -> Result<PathBuf> {
        match self.cwd {
            Some(cwd) => Ok(cwd),
            None => Ok(std::env::current_dir()?),
        }
    }
}

fn recall_output(hook_event_name: &str, store: &PolarisStore) -> Result<HookOutput> {
    Ok(HookOutput {
        hook_specific_output: HookSpecificOutput {
            hook_event_name: hook_event_name.to_string(),
            additional_context: recall_context(store)?,
        },
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
