use anyhow::{Context, Result, anyhow};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use uuid::Uuid;

const POLARIS_DIR: &str = ".polaris";
const STATE_FILE: &str = "state.json";
const HOOK_STATE_FILE: &str = "hook-state.json";
const MEMORIES_FILE: &str = "memories.jsonl";
const MEMORIES_LOCK_FILE: &str = "memories.lock";
const DOCS_DIR: &str = "docs";

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
    pub kind: MemoryKind,
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

pub struct MemoryInput {
    pub key: String,
    pub title: Option<String>,
    pub text: String,
}

#[derive(Clone, Debug, Default)]
pub struct MemoryFilter {
    pub key: Option<String>,
    pub prefix: Option<String>,
    pub exclude_prefixes: Vec<String>,
}

#[derive(Serialize)]
pub struct MemorySummary {
    id: String,
    created_at: String,
    kind: MemoryKind,
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

#[derive(Serialize)]
pub struct Status {
    initialized: bool,
    root: String,
    memory_count: usize,
    document_count: usize,
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
        Ok(Status {
            initialized,
            root: self.root().display().to_string(),
            memory_count: if initialized { self.memory_count()? } else { 0 },
            document_count: if initialized {
                self.document_count()?
            } else {
                0
            },
        })
    }

    pub fn remember(&self, input: MemoryInput, replace: bool) -> Result<MemoryRecord> {
        let _lock = self.lock_memories_exclusive()?;
        let record = MemoryRecord {
            id: new_id(),
            created_at: now(),
            kind: MemoryKind::Inline,
            key: Some(input.key.clone()),
            title: input.title,
            text: Some(input.text),
            path: None,
        };

        let mut records = self.load_records_unlocked()?;
        let existing = records
            .iter()
            .any(|record| record.is_inline_key(&input.key));
        if existing {
            if !replace {
                return Err(anyhow!(
                    "memory key `{}` already exists; use --replace to overwrite it",
                    input.key
                ));
            }
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

    pub fn create_note(&self, title: &str) -> Result<CreatedNote> {
        let _lock = self.lock_memories_exclusive()?;
        let id = new_id();
        let file_name = format!("{}-{}.md", id, slugify(title));
        let relative_path = PathBuf::from(POLARIS_DIR).join(DOCS_DIR).join(&file_name);
        let absolute_path = self.workspace.join(&relative_path);
        fs::write(&absolute_path, format!("# {title}\n\n"))?;

        let record = MemoryRecord {
            id: id.clone(),
            created_at: now(),
            kind: MemoryKind::Note,
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

    pub fn list_keys(&self) -> Result<Vec<String>> {
        Ok(self
            .load_records()?
            .into_iter()
            .filter_map(|record| match record.kind {
                MemoryKind::Inline => record.key,
                MemoryKind::Note => None,
            })
            .collect())
    }

    pub fn list_summaries(&self) -> Result<Vec<MemorySummary>> {
        Ok(self
            .load_records()?
            .into_iter()
            .map(MemorySummary::from)
            .collect())
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
}

impl MemoryRecord {
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

        let key = match self.key.as_deref() {
            Some(key) => key,
            None => return filter.key.is_none() && filter.prefix.is_none(),
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
        self.key.is_some() || self.prefix.is_some() || !self.exclude_prefixes.is_empty()
    }
}

impl From<MemoryRecord> for MemorySummary {
    fn from(record: MemoryRecord) -> Self {
        Self {
            id: record.id,
            created_at: record.created_at,
            kind: record.kind,
            key: record.key,
            title: record.title,
            path: record.path,
        }
    }
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
