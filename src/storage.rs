use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Duration, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use uuid::Uuid;

const POLARIS_DIR: &str = ".polaris";
const STATE_FILE: &str = "state.json";
const HOOK_STATE_FILE: &str = "hook-state.json";
const CONFIG_FILE: &str = "config.toml";
const MEMORIES_FILE: &str = "memories.jsonl";
const MEMORIES_LOCK_FILE: &str = "memories.lock";
const DOCS_DIR: &str = "docs";
const MAINTENANCE_DIR: &str = "maintenance";
const STALE_VOLATILE_DAYS: i64 = 7;
const SUGGESTIONS_PLACEHOLDER: &str = "{{suggestions}}";
const STATUS_PLACEHOLDER: &str = "{{status}}";
const KEYS_PLACEHOLDER: &str = "{{keys}}";
const DEFAULT_PRUNE_PROMPT: &str = r#"You are reviewing Polaris prune suggestions.

These suggestions are heuristic candidates, not deletion decisions.
Before removing any memory:
1. Inspect each candidate with `polaris recall --key <key>`.
2. Check whether the memory is still relevant to the current task, active branch, OpenSpec work, or recent user intent.
3. Preserve durable decisions, constraints, and verification results unless they are clearly superseded.
4. Treat old `state` and `log` records as review candidates, not automatic deletion targets.
5. Prefer `polaris rename`, `polaris remember --replace`, or lifecycle changes when the memory is useful but mislabeled.
6. Only run `polaris forget <key>` when the memory is clearly stale, redundant, or misleading.

Workspace status:
{{status}}

Known keys:
{{keys}}

Prune suggestions:
{{suggestions}}"#;
const DEFAULT_COMPACT_PROMPT: &str = r#"You are reviewing Polaris compact suggestions.

These suggestions are heuristic consolidation candidates, not merge decisions.
Before merging any memory:
1. Recall every source key with `polaris recall --key <key>` or use a narrow prefix recall.
2. Decide whether the source memories describe one coherent durable fact, decision, state summary, or handoff note.
3. Do not merge unrelated records just because they share a prefix or lifecycle.
4. If merging is appropriate, run `polaris merge --into <target> <source>...` to create a draft.
5. Edit the draft so the merged memory is concise, accurate, and preserves important constraints.
6. Apply with `polaris merge apply <draft> --yes`.
7. Use `--forget-sources` only when the merged target fully supersedes every source key.

Workspace status:
{{status}}

Known keys:
{{keys}}

Compact suggestions:
{{suggestions}}"#;

#[derive(Debug)]
pub struct PolarisStore {
    workspace: PathBuf,
}

#[derive(Serialize)]
struct State {
    schema_version: u8,
    created_at: String,
}

#[derive(Serialize, Deserialize)]
struct HookState {
    schema_version: u8,
    compact_recall_pending: bool,
    recorded_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub kind: MemoryKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<MemoryLifecycle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryKind {
    Inline,
    Note,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryLifecycle {
    Durable,
    State,
    Log,
    Archive,
}

pub struct MemoryInput {
    pub key: String,
    pub title: Option<String>,
    pub text: String,
    pub lifecycle: Option<MemoryLifecycle>,
}

#[derive(Clone, Debug, Default)]
pub struct MemoryFilter {
    pub key: Option<String>,
    pub prefix: Option<String>,
    pub exclude_prefixes: Vec<String>,
    pub lifecycle: Option<MemoryLifecycle>,
}

#[derive(Serialize)]
pub struct MemorySummary {
    id: String,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated_at: Option<String>,
    kind: MemoryKind,
    lifecycle: MemoryLifecycle,
    #[serde(skip_serializing_if = "Option::is_none")]
    replacement_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replaced_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

pub struct CreatedNote {
    pub id: String,
    pub path: PathBuf,
}

pub struct CreatedMergeDraft {
    pub id: String,
    pub path: PathBuf,
}

struct MergeDraft {
    target: String,
    sources: Vec<String>,
    text: String,
}

#[derive(Deserialize)]
struct PolarisConfig {
    maintenance: Option<MaintenanceConfig>,
}

#[derive(Deserialize)]
struct MaintenanceConfig {
    prompts: Option<MaintenancePromptsConfig>,
}

#[derive(Default, Deserialize)]
struct MaintenancePromptsConfig {
    prune: Option<String>,
    compact: Option<String>,
}

#[derive(Serialize)]
pub struct Status {
    initialized: bool,
    root: String,
    memory_count: usize,
    document_count: usize,
    lifecycle_counts: BTreeMap<MemoryLifecycle, usize>,
    stale_volatile_memory: StaleVolatileMemory,
}

#[derive(Serialize)]
pub struct StaleVolatileMemory {
    threshold_days: i64,
    count: usize,
    records: Vec<StaleVolatileRecord>,
}

#[derive(Serialize)]
pub struct StaleVolatileRecord {
    id: String,
    lifecycle: MemoryLifecycle,
    #[serde(skip_serializing_if = "Option::is_none")]
    key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated_at: Option<String>,
}

#[derive(Serialize)]
pub struct PruneSuggestion {
    pub candidate_keys: Vec<String>,
    pub reasons: Vec<String>,
    pub suggested_commands: Vec<String>,
}

#[derive(Serialize)]
pub struct CompactSuggestion {
    pub source_keys: Vec<String>,
    pub proposed_target_key: String,
    pub reasons: Vec<String>,
    pub suggested_command: String,
}

#[derive(Serialize)]
pub struct SuggestionResponse<T> {
    pub suggestions: Vec<T>,
    pub prompt: String,
    pub prompt_context: SuggestionPromptContext,
}

#[derive(Serialize)]
pub struct SuggestionPromptContext {
    status: Status,
    keys: Vec<String>,
}

enum MaintenancePromptKind {
    Prune,
    Compact,
}

impl PolarisStore {
    pub fn from_current_dir() -> Result<Self> {
        Self::from_workspace(std::env::current_dir()?)
    }

    pub fn from_workspace(workspace: PathBuf) -> Result<Self> {
        Ok(Self { workspace })
    }

    pub fn root(&self) -> PathBuf {
        self.workspace.join(POLARIS_DIR)
    }

    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(self.docs_dir())?;
        if !self.memories_file().exists() {
            File::create(self.memories_file())?;
        }
        if !self.state_file().exists() {
            let state = State {
                schema_version: 1,
                created_at: now(),
            };
            fs::write(self.state_file(), serde_json::to_string_pretty(&state)?)?;
        }
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.root().is_dir()
            && self.state_file().is_file()
            && self.memories_file().is_file()
            && self.docs_dir().is_dir()
    }

    pub fn require_initialized(&self) -> Result<()> {
        if self.is_initialized() {
            Ok(())
        } else {
            Err(anyhow!(
                "Polaris is not initialized in this workspace; run `polaris init` first"
            ))
        }
    }

    pub fn status(&self) -> Result<Status> {
        let initialized = self.is_initialized();
        let records = if initialized {
            self.load_records()?
        } else {
            Vec::new()
        };
        let lifecycle_counts = lifecycle_counts(&records);
        let stale_volatile_memory = stale_volatile_memory(&records);
        Ok(Status {
            initialized,
            root: self.root().display().to_string(),
            memory_count: records.len(),
            document_count: if initialized {
                self.document_count()?
            } else {
                0
            },
            lifecycle_counts,
            stale_volatile_memory,
        })
    }

    pub fn remember(&self, input: MemoryInput, replace: bool) -> Result<MemoryRecord> {
        let _lock = self.lock_memories_exclusive()?;
        let now = now();
        let mut record = MemoryRecord {
            id: new_id(),
            created_at: now.clone(),
            updated_at: None,
            kind: MemoryKind::Inline,
            lifecycle: input.lifecycle,
            replacement_count: None,
            replaced_from: None,
            key: Some(input.key.clone()),
            title: input.title,
            text: Some(input.text),
            path: None,
        };

        let mut records = self.load_records_unlocked()?;
        let existing = records
            .iter()
            .find(|record| record.is_inline_key(&input.key))
            .cloned();
        if let Some(existing) = existing {
            if !replace {
                return Err(anyhow!(
                    "memory key `{}` already exists; use --replace to overwrite it",
                    input.key
                ));
            }
            record.created_at = existing.created_at;
            record.updated_at = Some(now);
            record.lifecycle = input.lifecycle.or(existing.lifecycle);
            record.replacement_count = Some(existing.replacement_count.unwrap_or(0) + 1);
            record.replaced_from = Some(existing.id);
            records.retain(|record| !record.is_inline_key(&input.key));
            records.push(record);
            self.write_records(&records)?;
            Ok(records.pop().expect("record was just pushed"))
        } else {
            self.append_record(&record)?;
            Ok(record)
        }
    }

    pub fn forget_keys(&self, keys: &[String]) -> Result<usize> {
        if keys.is_empty() {
            return Err(anyhow!("forget requires at least one key or --prefix"));
        }

        let _lock = self.lock_memories_exclusive()?;
        let mut records = self.load_records_unlocked()?;
        for key in keys {
            if !records.iter().any(|record| record.is_inline_key(key)) {
                return Err(anyhow!("No memory exists for key `{key}`"));
            }
        }

        let original_len = records.len();
        records.retain(|record| !keys.iter().any(|key| record.is_inline_key(key.as_str())));
        let removed = original_len - records.len();
        self.write_records(&records)?;
        Ok(removed)
    }

    pub fn forget_prefix(&self, prefix: &str) -> Result<usize> {
        if prefix.is_empty() {
            return Err(anyhow!("prefix must not be empty"));
        }

        let _lock = self.lock_memories_exclusive()?;
        let mut records = self.load_records_unlocked()?;
        let original_len = records.len();
        records.retain(|record| !record.is_inline_key_prefix(prefix));
        let removed = original_len - records.len();
        if removed == 0 {
            return Err(anyhow!("No memory exists for prefix `{prefix}`"));
        }
        self.write_records(&records)?;
        Ok(removed)
    }

    pub fn rename_key(&self, old_key: &str, new_key: &str) -> Result<()> {
        if old_key.trim().is_empty() {
            return Err(anyhow!("source key must not be empty"));
        }
        if new_key.trim().is_empty() {
            return Err(anyhow!("destination key must not be empty"));
        }

        let _lock = self.lock_memories_exclusive()?;
        let mut records = self.load_records_unlocked()?;
        let source_index = records
            .iter()
            .position(|record| record.is_inline_key(old_key))
            .ok_or_else(|| anyhow!("source key `{old_key}` does not exist"))?;
        if records.iter().any(|record| record.is_inline_key(new_key)) {
            return Err(anyhow!("destination key `{new_key}` already exists"));
        }

        records[source_index].key = Some(new_key.to_string());
        self.write_records(&records)?;
        Ok(())
    }

    fn write_records(&self, records: &[MemoryRecord]) -> Result<()> {
        let temp_path = self
            .root()
            .join(format!(".{MEMORIES_FILE}.{}.tmp", new_id()));
        {
            let mut file = File::create(&temp_path).with_context(|| {
                format!(
                    "failed to create temporary memory file {}",
                    temp_path.display()
                )
            })?;
            for record in records {
                writeln!(file, "{}", serde_json::to_string(record)?)?;
            }
        }
        fs::rename(&temp_path, self.memories_file()).with_context(|| {
            format!(
                "failed to replace {} with {}",
                self.memories_file().display(),
                temp_path.display()
            )
        })?;
        Ok(())
    }

    pub fn create_note(
        &self,
        title: &str,
        lifecycle: Option<MemoryLifecycle>,
    ) -> Result<CreatedNote> {
        let _lock = self.lock_memories_exclusive()?;
        let id = new_id();
        let file_name = format!("{}-{}.md", id, slugify(title));
        let relative_path = PathBuf::from(POLARIS_DIR).join(DOCS_DIR).join(&file_name);
        let absolute_path = self.workspace.join(&relative_path);
        fs::write(&absolute_path, format!("# {title}\n\n"))?;

        let record = MemoryRecord {
            id: id.clone(),
            created_at: now(),
            updated_at: None,
            kind: MemoryKind::Note,
            lifecycle,
            replacement_count: None,
            replaced_from: None,
            key: None,
            title: Some(title.to_string()),
            text: None,
            path: Some(relative_path.display().to_string()),
        };
        self.append_record(&record)?;

        Ok(CreatedNote {
            id,
            path: relative_path,
        })
    }

    pub fn create_merge_draft(
        &self,
        target: &str,
        sources: &[String],
    ) -> Result<CreatedMergeDraft> {
        let target = target.trim();
        if target.is_empty() {
            return Err(anyhow!("target key must not be empty"));
        }
        if sources.is_empty() {
            return Err(anyhow!("merge requires at least one source key"));
        }

        let records = self.load_records()?;
        let mut source_records = Vec::new();
        for source in sources {
            let record = records
                .iter()
                .find(|record| record.is_inline_key(source))
                .cloned()
                .ok_or_else(|| anyhow!("source key `{source}` does not exist"))?;
            source_records.push(record);
        }

        fs::create_dir_all(self.maintenance_dir())?;
        let id = new_id();
        let file_name = format!("merge-{}-{}.md", slugify(target), id);
        let relative_path = PathBuf::from(POLARIS_DIR)
            .join(MAINTENANCE_DIR)
            .join(file_name);
        let absolute_path = self.workspace.join(&relative_path);
        fs::write(
            &absolute_path,
            render_merge_draft(target, sources, &source_records)?,
        )?;

        Ok(CreatedMergeDraft {
            id,
            path: relative_path,
        })
    }

    pub fn apply_merge_draft(&self, draft_path: &Path, forget_sources: bool) -> Result<String> {
        let absolute_path = if draft_path.is_absolute() {
            draft_path.to_path_buf()
        } else {
            self.workspace.join(draft_path)
        };
        let draft =
            parse_merge_draft(&fs::read_to_string(&absolute_path).with_context(|| {
                format!("failed to read merge draft {}", absolute_path.display())
            })?)?;

        let _lock = self.lock_memories_exclusive()?;
        let mut records = self.load_records_unlocked()?;
        if forget_sources {
            for source in &draft.sources {
                if !records.iter().any(|record| record.is_inline_key(source)) {
                    return Err(anyhow!("source key `{source}` does not exist"));
                }
            }
        }

        let now = now();
        let existing = records
            .iter()
            .find(|record| record.is_inline_key(&draft.target))
            .cloned();
        let mut target_record = MemoryRecord {
            id: new_id(),
            created_at: now.clone(),
            updated_at: None,
            kind: MemoryKind::Inline,
            lifecycle: Some(MemoryLifecycle::Durable),
            replacement_count: None,
            replaced_from: None,
            key: Some(draft.target.clone()),
            title: None,
            text: Some(draft.text),
            path: None,
        };
        if let Some(existing) = existing {
            target_record.created_at = existing.created_at;
            target_record.updated_at = Some(now);
            target_record.lifecycle = existing.lifecycle;
            target_record.replacement_count = Some(existing.replacement_count.unwrap_or(0) + 1);
            target_record.replaced_from = Some(existing.id);
        }

        records.retain(|record| {
            !(record.is_inline_key(&draft.target)
                || forget_sources
                    && record
                        .key
                        .as_deref()
                        .is_some_and(|key| draft.sources.iter().any(|source| source == key)))
        });
        records.push(target_record);
        self.write_records(&records)?;
        Ok(draft.target)
    }

    pub fn recall(&self) -> Result<String> {
        self.recall_filtered(&MemoryFilter::default())
    }

    pub fn recall_filtered(&self, filter: &MemoryFilter) -> Result<String> {
        let records = self.load_records()?;
        let filtered = records
            .into_iter()
            .filter(|record| record.matches_filter(filter))
            .collect::<Vec<_>>();

        if filter.is_active() && filtered.is_empty() {
            return Ok("No matching Polaris memory is stored for this workspace.\n".to_string());
        }

        Self::render_recall(filtered)
    }

    fn render_recall(records: Vec<MemoryRecord>) -> Result<String> {
        if records.is_empty() {
            return Ok("No Polaris memory is stored for this workspace.\n".to_string());
        }

        let mut output = String::from("# Polaris Recall\n\n");
        for record in records {
            let title = record
                .title
                .clone()
                .or_else(|| record.key.clone())
                .unwrap_or_else(|| format!("Memory {}", record.id));
            match record.kind {
                MemoryKind::Inline => {
                    output.push_str(&format!("## {title}\n"));
                    if let Some(key) = record.key.as_deref() {
                        output.push_str(&format!("- Key: {key}\n"));
                    }
                    output.push_str(record.text.as_deref().unwrap_or(""));
                    output.push_str("\n\n");
                }
                MemoryKind::Note => {
                    output.push_str(&format!("## {title}\n"));
                    output.push_str(&format!(
                        "- Path: {}\n\n",
                        record.path.as_deref().unwrap_or("<missing path>")
                    ));
                }
            }
        }
        Ok(output)
    }

    pub fn list_keys(&self, filter: &MemoryFilter) -> Result<Vec<String>> {
        Ok(self
            .load_records()?
            .into_iter()
            .filter(|record| record.matches_filter(filter))
            .filter_map(|record| match record.kind {
                MemoryKind::Inline => record.key,
                MemoryKind::Note => None,
            })
            .collect())
    }

    pub fn list_summaries(&self, filter: &MemoryFilter) -> Result<Vec<MemorySummary>> {
        Ok(self
            .load_records()?
            .into_iter()
            .filter(|record| record.matches_filter(filter))
            .map(MemorySummary::from)
            .collect())
    }

    pub fn prune_suggestions(&self) -> Result<Vec<PruneSuggestion>> {
        let records = self.load_records()?;
        Ok(prune_suggestions(&records))
    }

    pub fn compact_suggestions(&self) -> Result<Vec<CompactSuggestion>> {
        let records = self.load_records()?;
        Ok(compact_suggestions(&records))
    }

    pub fn prune_suggestion_response(&self) -> Result<SuggestionResponse<PruneSuggestion>> {
        let suggestions = self.prune_suggestions()?;
        self.suggestion_response(suggestions, MaintenancePromptKind::Prune)
    }

    pub fn compact_suggestion_response(&self) -> Result<SuggestionResponse<CompactSuggestion>> {
        let suggestions = self.compact_suggestions()?;
        self.suggestion_response(suggestions, MaintenancePromptKind::Compact)
    }

    fn suggestion_response<T: Serialize>(
        &self,
        suggestions: Vec<T>,
        kind: MaintenancePromptKind,
    ) -> Result<SuggestionResponse<T>> {
        let status = self.status()?;
        let keys = self.list_keys(&MemoryFilter::default())?;
        let prompt = self.render_maintenance_prompt(&kind, &suggestions, &status, &keys)?;

        Ok(SuggestionResponse {
            suggestions,
            prompt,
            prompt_context: SuggestionPromptContext { status, keys },
        })
    }

    fn render_maintenance_prompt<T: Serialize>(
        &self,
        kind: &MaintenancePromptKind,
        suggestions: &[T],
        status: &Status,
        keys: &[String],
    ) -> Result<String> {
        let prompts = self.configured_maintenance_prompts()?;
        let template = match kind {
            MaintenancePromptKind::Prune => {
                prompts.prune.as_deref().unwrap_or(DEFAULT_PRUNE_PROMPT)
            }
            MaintenancePromptKind::Compact => {
                prompts.compact.as_deref().unwrap_or(DEFAULT_COMPACT_PROMPT)
            }
        };

        Ok(template
            .replace(
                SUGGESTIONS_PLACEHOLDER,
                &serde_json::to_string_pretty(suggestions)?,
            )
            .replace(STATUS_PLACEHOLDER, &serde_json::to_string_pretty(status)?)
            .replace(KEYS_PLACEHOLDER, &serde_json::to_string_pretty(keys)?))
    }

    fn configured_maintenance_prompts(&self) -> Result<MaintenancePromptsConfig> {
        let workspace_config = self.root().join(CONFIG_FILE);
        if config_exists(&workspace_config)? {
            return read_maintenance_prompts(&workspace_config);
        }

        let Some(home) = std::env::var_os("HOME") else {
            return Ok(MaintenancePromptsConfig::default());
        };
        let user_config = PathBuf::from(home).join(POLARIS_DIR).join(CONFIG_FILE);
        if config_exists(&user_config)? {
            return read_maintenance_prompts(&user_config);
        }

        Ok(MaintenancePromptsConfig::default())
    }

    pub fn clear(&self) -> Result<()> {
        let _lock = self.lock_memories_exclusive()?;
        fs::write(self.memories_file(), "")?;
        let hook_state_file = self.hook_state_file();
        if hook_state_file.exists() {
            fs::remove_file(hook_state_file)?;
        }
        if self.docs_dir().exists() {
            for entry in fs::read_dir(self.docs_dir())? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(path)?;
                } else if path.is_dir() {
                    fs::remove_dir_all(path)?;
                }
            }
        }
        Ok(())
    }

    pub fn memory_count(&self) -> Result<usize> {
        Ok(self.load_records()?.len())
    }

    pub fn mark_compact_recall_pending(&self) -> Result<()> {
        let state = HookState {
            schema_version: 1,
            compact_recall_pending: true,
            recorded_at: now(),
        };
        fs::write(
            self.hook_state_file(),
            serde_json::to_string_pretty(&state)?,
        )?;
        Ok(())
    }

    pub fn consume_compact_recall_pending(&self) -> Result<bool> {
        let path = self.hook_state_file();
        if !path.exists() {
            return Ok(false);
        }

        let state: HookState = serde_json::from_str(
            &fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?,
        )?;
        fs::remove_file(&path).with_context(|| format!("failed to remove {}", path.display()))?;
        Ok(state.compact_recall_pending)
    }

    fn document_count(&self) -> Result<usize> {
        if !self.docs_dir().exists() {
            return Ok(0);
        }
        let mut count = 0;
        for entry in fs::read_dir(self.docs_dir())? {
            if entry?.path().is_file() {
                count += 1;
            }
        }
        Ok(count)
    }

    fn load_records(&self) -> Result<Vec<MemoryRecord>> {
        let _lock = self.lock_memories_shared()?;
        self.load_records_unlocked()
    }

    fn load_records_unlocked(&self) -> Result<Vec<MemoryRecord>> {
        let memories_file = self.memories_file();
        let file = File::open(&memories_file)
            .with_context(|| format!("failed to open {}", memories_file.display()))?;
        let mut records = Vec::new();
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            records.push(serde_json::from_str(&line).map_err(|error| {
                anyhow!(
                    "failed to parse {}:{}: {}",
                    memories_file.display(),
                    index + 1,
                    error
                )
            })?);
        }
        Ok(records)
    }

    fn append_record(&self, record: &MemoryRecord) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(self.memories_file())
            .with_context(|| format!("failed to open {}", self.memories_file().display()))?;

        let len = file.metadata()?.len();
        if len > 0 {
            file.seek(SeekFrom::End(-1))?;
            let mut last_byte = [0; 1];
            file.read_exact(&mut last_byte)?;
            if last_byte[0] != b'\n' {
                file.write_all(b"\n")?;
            }
        }

        let serialized = serde_json::to_string(record)?;
        file.write_all(serialized.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    fn lock_memories_exclusive(&self) -> Result<File> {
        let file = self.open_memories_lock_file()?;
        file.lock()
            .with_context(|| format!("failed to lock {}", self.memories_lock_file().display()))?;
        Ok(file)
    }

    fn lock_memories_shared(&self) -> Result<File> {
        let file = self.open_memories_lock_file()?;
        file.lock_shared()
            .with_context(|| format!("failed to lock {}", self.memories_lock_file().display()))?;
        Ok(file)
    }

    fn open_memories_lock_file(&self) -> Result<File> {
        OpenOptions::new()
            .create(true)
            .read(true)
            .truncate(false)
            .write(true)
            .open(self.memories_lock_file())
            .with_context(|| format!("failed to open {}", self.memories_lock_file().display()))
    }

    fn state_file(&self) -> PathBuf {
        self.root().join(STATE_FILE)
    }

    fn hook_state_file(&self) -> PathBuf {
        self.root().join(HOOK_STATE_FILE)
    }

    fn memories_file(&self) -> PathBuf {
        self.root().join(MEMORIES_FILE)
    }

    fn memories_lock_file(&self) -> PathBuf {
        self.root().join(MEMORIES_LOCK_FILE)
    }

    fn docs_dir(&self) -> PathBuf {
        self.root().join(DOCS_DIR)
    }

    fn maintenance_dir(&self) -> PathBuf {
        self.root().join(MAINTENANCE_DIR)
    }
}

impl MemoryRecord {
    fn effective_lifecycle(&self) -> MemoryLifecycle {
        self.lifecycle.unwrap_or(MemoryLifecycle::Durable)
    }

    fn is_inline_key(&self, key: &str) -> bool {
        matches!(self.kind, MemoryKind::Inline) && self.key.as_deref() == Some(key)
    }

    fn is_inline_key_prefix(&self, prefix: &str) -> bool {
        matches!(self.kind, MemoryKind::Inline)
            && self
                .key
                .as_deref()
                .is_some_and(|key| key.starts_with(prefix))
    }

    fn matches_filter(&self, filter: &MemoryFilter) -> bool {
        if !filter.is_active() {
            return true;
        }

        if let Some(lifecycle) = filter.lifecycle
            && self.effective_lifecycle() != lifecycle
        {
            return false;
        }

        let Some(key) = self.key.as_deref() else {
            return filter.key.is_none() && filter.prefix.is_none();
        };

        if filter
            .exclude_prefixes
            .iter()
            .any(|prefix| key.starts_with(prefix))
        {
            return false;
        }

        if let Some(exact) = filter.key.as_deref() {
            return key == exact;
        }

        if let Some(prefix) = filter.prefix.as_deref() {
            return key.starts_with(prefix);
        }

        true
    }
}

impl MemoryFilter {
    pub fn is_active(&self) -> bool {
        self.key.is_some()
            || self.prefix.is_some()
            || !self.exclude_prefixes.is_empty()
            || self.lifecycle.is_some()
    }
}

impl From<MemoryRecord> for MemorySummary {
    fn from(record: MemoryRecord) -> Self {
        let lifecycle = record.effective_lifecycle();
        Self {
            id: record.id,
            created_at: record.created_at,
            updated_at: record.updated_at,
            kind: record.kind,
            lifecycle,
            replacement_count: record.replacement_count,
            replaced_from: record.replaced_from,
            key: record.key,
            title: record.title,
            path: record.path,
        }
    }
}

impl MemoryLifecycle {
    fn as_str(self) -> &'static str {
        match self {
            Self::Durable => "durable",
            Self::State => "state",
            Self::Log => "log",
            Self::Archive => "archive",
        }
    }

    fn all() -> [Self; 4] {
        [Self::Durable, Self::State, Self::Log, Self::Archive]
    }
}

impl std::fmt::Display for MemoryLifecycle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for MemoryLifecycle {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "durable" => Ok(Self::Durable),
            "state" => Ok(Self::State),
            "log" => Ok(Self::Log),
            "archive" => Ok(Self::Archive),
            _ => Err("supported lifecycle values are durable, state, log, and archive".to_string()),
        }
    }
}

fn lifecycle_counts(records: &[MemoryRecord]) -> BTreeMap<MemoryLifecycle, usize> {
    let mut counts = BTreeMap::new();
    for lifecycle in MemoryLifecycle::all() {
        counts.insert(lifecycle, 0);
    }
    for record in records {
        *counts.entry(record.effective_lifecycle()).or_insert(0) += 1;
    }
    counts
}

fn stale_volatile_memory(records: &[MemoryRecord]) -> StaleVolatileMemory {
    let threshold = Utc::now() - Duration::days(STALE_VOLATILE_DAYS);
    let records = records
        .iter()
        .filter(|record| {
            matches!(
                record.effective_lifecycle(),
                MemoryLifecycle::State | MemoryLifecycle::Log
            )
        })
        .filter(|record| {
            is_before_threshold(
                record.updated_at.as_deref().unwrap_or(&record.created_at),
                threshold,
            )
        })
        .map(|record| StaleVolatileRecord {
            id: record.id.clone(),
            lifecycle: record.effective_lifecycle(),
            key: record.key.clone(),
            title: record.title.clone(),
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        })
        .collect::<Vec<_>>();

    StaleVolatileMemory {
        threshold_days: STALE_VOLATILE_DAYS,
        count: records.len(),
        records,
    }
}

fn prune_suggestions(records: &[MemoryRecord]) -> Vec<PruneSuggestion> {
    let threshold = Utc::now() - Duration::days(STALE_VOLATILE_DAYS);
    let mut suggestions = Vec::new();

    for record in keyed_inline_records(records) {
        let lifecycle = record.effective_lifecycle();
        if matches!(lifecycle, MemoryLifecycle::State | MemoryLifecycle::Log)
            && is_before_threshold(
                record.updated_at.as_deref().unwrap_or(&record.created_at),
                threshold,
            )
            && let Some(key) = record.key.clone()
        {
            suggestions.push(PruneSuggestion {
                candidate_keys: vec![key.clone()],
                reasons: vec![format!("old volatile memory ({lifecycle})")],
                suggested_commands: vec![format!("polaris forget {key}")],
            });
        }
    }

    let mut text_keys: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for record in keyed_inline_records(records) {
        if let (Some(key), Some(text)) = (record.key.as_ref(), record.text.as_ref())
            && !text.trim().is_empty()
        {
            text_keys.entry(text.clone()).or_default().push(key.clone());
        }
    }
    for keys in text_keys.values().filter(|keys| keys.len() > 1) {
        suggestions.push(PruneSuggestion {
            candidate_keys: keys.clone(),
            reasons: vec!["duplicate exact text".to_string()],
            suggested_commands: keys
                .iter()
                .skip(1)
                .map(|key| format!("polaris forget {key}"))
                .collect(),
        });
    }

    for record in keyed_inline_records(records) {
        if record.replacement_count.unwrap_or(0) > 0
            && let Some(key) = record.key.clone()
        {
            suggestions.push(PruneSuggestion {
                candidate_keys: vec![key.clone()],
                reasons: vec![
                    "replacement metadata indicates prior superseded content".to_string(),
                ],
                suggested_commands: vec![format!("polaris recall --key {key}")],
            });
        }
    }

    suggestions
}

fn compact_suggestions(records: &[MemoryRecord]) -> Vec<CompactSuggestion> {
    let mut suggestions = Vec::new();
    let mut prefix_keys: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for record in keyed_inline_records(records) {
        if let Some(key) = record.key.as_ref()
            && let Some((prefix, _)) = key.split_once('.')
        {
            prefix_keys
                .entry(prefix.to_string())
                .or_default()
                .push(key.clone());
        }
    }
    for (prefix, keys) in prefix_keys {
        if keys.len() > 1 {
            let target = format!("{prefix}.summary");
            suggestions.push(CompactSuggestion {
                source_keys: keys.clone(),
                proposed_target_key: target.clone(),
                reasons: vec![format!("shared key prefix `{prefix}.`")],
                suggested_command: format!("polaris merge --into {target} {}", keys.join(" ")),
            });
        }
    }

    let mut lifecycle_keys: BTreeMap<MemoryLifecycle, Vec<String>> = BTreeMap::new();
    for record in keyed_inline_records(records) {
        let lifecycle = record.effective_lifecycle();
        if matches!(lifecycle, MemoryLifecycle::State | MemoryLifecycle::Log)
            && let Some(key) = record.key.clone()
        {
            lifecycle_keys.entry(lifecycle).or_default().push(key);
        }
    }
    for (lifecycle, keys) in lifecycle_keys {
        if keys.len() > 2 {
            let target = format!("{lifecycle}.summary");
            suggestions.push(CompactSuggestion {
                source_keys: keys.clone(),
                proposed_target_key: target.clone(),
                reasons: vec![format!("related `{lifecycle}` lifecycle records")],
                suggested_command: format!("polaris merge --into {target} {}", keys.join(" ")),
            });
        }
    }

    suggestions
}

fn keyed_inline_records(records: &[MemoryRecord]) -> impl Iterator<Item = &MemoryRecord> {
    records
        .iter()
        .filter(|record| matches!(record.kind, MemoryKind::Inline) && record.key.is_some())
}

fn config_exists(path: &Path) -> Result<bool> {
    path.try_exists()
        .with_context(|| format!("failed to inspect Polaris config {}", path.display()))
}

fn read_maintenance_prompts(path: &Path) -> Result<MaintenancePromptsConfig> {
    let config = fs::read_to_string(path)
        .with_context(|| format!("failed to read Polaris config {}", path.display()))?;
    let config: PolarisConfig = toml::from_str(&config)
        .with_context(|| format!("failed to parse Polaris config {}", path.display()))?;
    Ok(config
        .maintenance
        .and_then(|maintenance| maintenance.prompts)
        .unwrap_or_default())
}

fn is_before_threshold(timestamp: &str, threshold: DateTime<Utc>) -> bool {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|timestamp| timestamp.with_timezone(&Utc) < threshold)
        .unwrap_or(false)
}

fn render_merge_draft(
    target: &str,
    sources: &[String],
    records: &[MemoryRecord],
) -> Result<String> {
    let mut output = String::new();
    output.push_str("<!-- polaris-merge-draft-v1\n");
    output.push_str(&format!("target: {target}\n"));
    output.push_str(&format!("sources: {}\n", sources.join(",")));
    output.push_str("-->\n\n");
    output.push_str("# Polaris Merge Draft\n\n");
    output.push_str(
        "Edit the text under `## Merged Memory`, then run `polaris merge apply <path> --yes`.\n\n",
    );
    output.push_str("## Sources\n\n");
    for record in records {
        let key = record
            .key
            .as_deref()
            .ok_or_else(|| anyhow!("merge source record is missing key"))?;
        output.push_str(&format!("## Source {key}\n"));
        output.push_str(&format!("- id: {}\n", record.id));
        output.push_str(&format!("- created_at: {}\n", record.created_at));
        if let Some(updated_at) = record.updated_at.as_deref() {
            output.push_str(&format!("- updated_at: {updated_at}\n"));
        }
        output.push_str(&format!("- lifecycle: {}\n", record.effective_lifecycle()));
        output.push_str("\n```text\n");
        output.push_str(record.text.as_deref().unwrap_or(""));
        output.push_str("\n```\n\n");
    }
    output.push_str("## Merged Memory\n\n");
    for record in records {
        let key = record
            .key
            .as_deref()
            .ok_or_else(|| anyhow!("merge source record is missing key"))?;
        output.push_str(&format!("From {key}:\n"));
        output.push_str(record.text.as_deref().unwrap_or(""));
        output.push_str("\n\n");
    }
    Ok(output)
}

fn parse_merge_draft(contents: &str) -> Result<MergeDraft> {
    let Some(header) = contents.strip_prefix("<!-- polaris-merge-draft-v1\n") else {
        return Err(anyhow!(
            "merge draft cannot be applied: missing Polaris merge draft header"
        ));
    };
    let Some((metadata, body)) = header.split_once("-->") else {
        return Err(anyhow!(
            "merge draft cannot be applied: missing Polaris merge draft metadata terminator"
        ));
    };

    let mut target = None;
    let mut sources = None;
    for line in metadata.lines() {
        if let Some(value) = line.strip_prefix("target: ") {
            target = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("sources: ") {
            sources = Some(
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|source| !source.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            );
        }
    }

    let target = target
        .filter(|target| !target.is_empty())
        .ok_or_else(|| anyhow!("merge draft cannot be applied: target key is missing"))?;
    let sources = sources
        .filter(|sources| !sources.is_empty())
        .ok_or_else(|| anyhow!("merge draft cannot be applied: source keys are missing"))?;
    let Some((_, text)) = body.split_once("## Merged Memory\n") else {
        return Err(anyhow!(
            "merge draft cannot be applied: missing `## Merged Memory` section"
        ));
    };
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err(anyhow!(
            "merge draft cannot be applied: merged memory text is empty"
        ));
    }

    Ok(MergeDraft {
        target,
        sources,
        text,
    })
}

fn new_id() -> String {
    Uuid::new_v4().simple().to_string()
}

fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn slugify(title: &str) -> String {
    let slug: String = title
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if slug.is_empty() {
        "note".to_string()
    } else {
        slug
    }
}
