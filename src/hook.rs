use crate::storage::PolarisStore;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const RECALL_INSTRUCTION: &str = "Polaris has saved workspace context for this project. Run `polaris recall` immediately before doing any more work.";

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

    Ok(Some(recall_output("SessionStart")))
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

    Ok(Some(recall_output("PostToolUse")))
}

impl HookInput {
    fn cwd(self) -> Result<PathBuf> {
        match self.cwd {
            Some(cwd) => Ok(cwd),
            None => Ok(std::env::current_dir()?),
        }
    }
}

fn recall_output(hook_event_name: &str) -> HookOutput {
    HookOutput {
        hook_specific_output: HookSpecificOutput {
            hook_event_name: hook_event_name.to_string(),
            additional_context: RECALL_INSTRUCTION.to_string(),
        },
    }
}
