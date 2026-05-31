use anyhow::{Context, Result, anyhow};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use uuid::Uuid;

const POLARIS_DIR: &str = ".polaris";
const STATE_FILE: &str = "state.json";
const MEMORIES_FILE: &str = "memories.jsonl";
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
        let record = MemoryRecord {
            id: new_id(),
            created_at: now(),
            kind: MemoryKind::Inline,
            key: Some(input.key.clone()),
            title: input.title,
            text: Some(input.text),
            path: None,
        };

        let mut records = self.load_records()?;
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

    pub fn forget(&self, key: &str) -> Result<()> {
        let mut records = self.load_records()?;
        let original_len = records.len();
        records.retain(|record| !record.is_inline_key(key));
        if records.len() == original_len {
            return Err(anyhow!("No memory exists for key `{key}`"));
        }
        self.write_records(&records)
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
        let records = self.load_records()?;
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

    pub fn clear(&self) -> Result<()> {
        fs::write(self.memories_file(), "")?;
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
        let file = File::open(self.memories_file())
            .with_context(|| format!("failed to open {}", self.memories_file().display()))?;
        let mut records = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            records.push(serde_json::from_str(&line)?);
        }
        Ok(records)
    }

    fn append_record(&self, record: &MemoryRecord) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.memories_file())?;
        writeln!(file, "{}", serde_json::to_string(record)?)?;
        Ok(())
    }

    fn state_file(&self) -> PathBuf {
        self.root().join(STATE_FILE)
    }

    fn memories_file(&self) -> PathBuf {
        self.root().join(MEMORIES_FILE)
    }

    fn docs_dir(&self) -> PathBuf {
        self.root().join(DOCS_DIR)
    }
}

impl MemoryRecord {
    fn is_inline_key(&self, key: &str) -> bool {
        matches!(self.kind, MemoryKind::Inline) && self.key.as_deref() == Some(key)
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
