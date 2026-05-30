use crate::storage::PolarisStore;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const RECALL_INSTRUCTION: &str = "Polaris has saved workspace context for this project. Run `polaris recall` immediately before doing any more work.";

#[derive(Deserialize)]
struct SessionStartInput {
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
    let input: SessionStartInput = serde_json::from_str(input)?;
    if input.hook_event_name.as_deref() != Some("SessionStart")
        || input.source.as_deref() != Some("compact")
    {
        return Ok(None);
    }

    let cwd = match input.cwd {
        Some(cwd) => cwd,
        None => std::env::current_dir()?,
    };
    let store = store_for(cwd)?;
    if !store.is_initialized() || store.memory_count()? == 0 {
        return Ok(None);
    }

    Ok(Some(HookOutput {
        hook_specific_output: HookSpecificOutput {
            hook_event_name: "SessionStart".to_string(),
            additional_context: RECALL_INSTRUCTION.to_string(),
        },
    }))
}
