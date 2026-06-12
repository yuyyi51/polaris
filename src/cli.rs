use crate::hook;
use crate::storage::{MemoryFilter, MemoryInput, PolarisStore};
use anyhow::{Result, anyhow};
use clap::{Args, Parser, Subcommand};
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "polaris",
    about = "Workspace-local memory for long-running agent tasks"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init,
    Status(StatusArgs),
    Remember(RememberArgs),
    Forget(ForgetArgs),
    List(ListArgs),
    Note(NoteArgs),
    Recall(RecallArgs),
    Clear(ClearArgs),
    Hook(HookArgs),
}

#[derive(Args)]
struct StatusArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct RememberArgs {
    #[arg(long)]
    key: Option<String>,
    #[arg(long, conflicts_with = "stdin")]
    text: Option<String>,
    #[arg(long)]
    stdin: bool,
    #[arg(long)]
    title: Option<String>,
    #[arg(long)]
    replace: bool,
}

#[derive(Args)]
struct ForgetArgs {
    #[arg(value_name = "KEY")]
    keys: Vec<String>,
    #[arg(long)]
    prefix: Option<String>,
    #[arg(long)]
    yes: bool,
}

#[derive(Args)]
struct ListArgs {
    #[arg(long)]
    keys: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct RecallArgs {
    #[arg(long)]
    key: Option<String>,
    #[arg(long)]
    prefix: Option<String>,
    #[arg(long = "exclude")]
    exclude_prefixes: Vec<String>,
}

#[derive(Args)]
struct NoteArgs {
    #[command(subcommand)]
    command: NoteCommand,
}

#[derive(Subcommand)]
enum NoteCommand {
    Create(NoteCreateArgs),
}

#[derive(Args)]
struct NoteCreateArgs {
    #[arg(long)]
    title: String,
}

#[derive(Args)]
struct ClearArgs {
    #[arg(long)]
    yes: bool,
}

#[derive(Args)]
struct HookArgs {
    #[command(subcommand)]
    command: HookCommand,
}

#[derive(Subcommand)]
enum HookCommand {
    #[command(name = "session-start")]
    SessionStart,
    #[command(name = "post-compact")]
    PostCompact,
    #[command(name = "post-tool-use")]
    PostToolUse,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => {
            let store = PolarisStore::from_current_dir()?;
            store.init()?;
            println!("Initialized Polaris at {}", store.root().display());
        }
        Command::Status(args) => {
            if !args.json {
                return Err(anyhow!("status currently requires --json"));
            }
            let store = PolarisStore::from_current_dir()?;
            let status = store.status()?;
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        Command::Remember(args) => {
            let store = PolarisStore::from_current_dir()?;
            store.require_initialized()?;
            let key = args
                .key
                .filter(|key| !key.trim().is_empty())
                .ok_or_else(|| anyhow!("remember requires --key <key>"))?;
            let text = if args.stdin {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                input.trim_end_matches(['\r', '\n']).to_string()
            } else {
                args.text
                    .ok_or_else(|| anyhow!("remember requires --text <text> or --stdin"))?
            };
            let record = store.remember(
                MemoryInput {
                    key,
                    title: args.title,
                    text,
                },
                args.replace,
            )?;
            println!("Recorded memory {}", record.id);
        }
        Command::Forget(args) => {
            let store = PolarisStore::from_current_dir()?;
            store.require_initialized()?;
            if args.prefix.is_some() && !args.keys.is_empty() {
                return Err(anyhow!("positional keys cannot be used with --prefix"));
            }
            if let Some(prefix) = args.prefix {
                if !args.yes {
                    return Err(anyhow!("refusing to forget memory by prefix without --yes"));
                }
                let removed = store.forget_prefix(&prefix)?;
                println!("Forgot {removed} memories with prefix {prefix}");
            } else {
                let removed = store.forget_keys(&args.keys)?;
                if removed == 1 {
                    println!("Forgot memory {}", args.keys[0]);
                } else {
                    println!("Forgot {removed} memories");
                }
            }
        }
        Command::List(args) => {
            if args.keys == args.json {
                return Err(anyhow!("list requires exactly one of --keys or --json"));
            }
            let store = PolarisStore::from_current_dir()?;
            store.require_initialized()?;
            if args.keys {
                for key in store.list_keys()? {
                    println!("{key}");
                }
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&store.list_summaries()?)?
                );
            }
        }
        Command::Note(args) => match args.command {
            NoteCommand::Create(args) => {
                let store = PolarisStore::from_current_dir()?;
                store.require_initialized()?;
                let note = store.create_note(&args.title)?;
                println!("Created note {}", note.id);
                println!("Path: {}", note.path.display());
            }
        },
        Command::Recall(args) => {
            if args.key.is_some() && args.prefix.is_some() {
                return Err(anyhow!("recall --key cannot be used with --prefix"));
            }
            let store = PolarisStore::from_current_dir()?;
            store.require_initialized()?;
            print!(
                "{}",
                store.recall_filtered(&MemoryFilter {
                    key: args.key,
                    prefix: args.prefix,
                    exclude_prefixes: args.exclude_prefixes,
                })?
            );
        }
        Command::Clear(args) => {
            if !args.yes {
                return Err(anyhow!("refusing to clear Polaris memory without --yes"));
            }
            let store = PolarisStore::from_current_dir()?;
            store.require_initialized()?;
            store.clear()?;
            println!("Cleared Polaris memory");
        }
        Command::Hook(args) => match args.command {
            HookCommand::SessionStart => {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                if let Some(output) = hook::session_start_output(&input, |cwd: PathBuf| {
                    PolarisStore::from_workspace(cwd)
                })? {
                    println!("{}", serde_json::to_string(&output)?);
                }
            }
            HookCommand::PostCompact => {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                hook::post_compact(&input, |cwd: PathBuf| PolarisStore::from_workspace(cwd))?;
            }
            HookCommand::PostToolUse => {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                if let Some(output) = hook::post_tool_use_output(&input, |cwd: PathBuf| {
                    PolarisStore::from_workspace(cwd)
                })? {
                    println!("{}", serde_json::to_string(&output)?);
                }
            }
        },
    }
    Ok(())
}
