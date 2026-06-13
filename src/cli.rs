use crate::hook;
use crate::storage::{MemoryFilter, MemoryInput, MemoryLifecycle, PolarisStore};
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
    Rename(RenameArgs),
    Merge(MergeArgs),
    Prune(SuggestArgs),
    Compact(SuggestArgs),
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
    #[arg(long)]
    lifecycle: Option<MemoryLifecycle>,
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
struct RenameArgs {
    old_key: String,
    new_key: String,
}

#[derive(Args)]
struct MergeArgs {
    #[command(subcommand)]
    command: Option<MergeCommand>,
    #[arg(long)]
    into: Option<String>,
    #[arg(value_name = "SOURCE")]
    sources: Vec<String>,
}

#[derive(Subcommand)]
enum MergeCommand {
    Apply(MergeApplyArgs),
}

#[derive(Args)]
struct MergeApplyArgs {
    draft: PathBuf,
    #[arg(long)]
    yes: bool,
    #[arg(long)]
    forget_sources: bool,
}

#[derive(Args)]
struct SuggestArgs {
    #[arg(long)]
    suggest: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ListArgs {
    #[arg(long)]
    keys: bool,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    lifecycle: Option<MemoryLifecycle>,
}

#[derive(Args)]
struct RecallArgs {
    #[arg(long)]
    key: Option<String>,
    #[arg(long)]
    prefix: Option<String>,
    #[arg(long = "exclude")]
    exclude_prefixes: Vec<String>,
    #[arg(long)]
    lifecycle: Option<MemoryLifecycle>,
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
    #[arg(long)]
    lifecycle: Option<MemoryLifecycle>,
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
                    lifecycle: args.lifecycle,
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
        Command::Rename(args) => {
            let store = PolarisStore::from_current_dir()?;
            store.require_initialized()?;
            store.rename_key(&args.old_key, &args.new_key)?;
            println!("Renamed memory {} to {}", args.old_key, args.new_key);
        }
        Command::Merge(args) => {
            let store = PolarisStore::from_current_dir()?;
            store.require_initialized()?;
            match args.command {
                Some(MergeCommand::Apply(apply_args)) => {
                    if !apply_args.yes {
                        return Err(anyhow!("refusing to apply merge draft without --yes"));
                    }
                    let target =
                        store.apply_merge_draft(&apply_args.draft, apply_args.forget_sources)?;
                    println!("Applied merge draft to {target}");
                }
                None => {
                    let target = args
                        .into
                        .ok_or_else(|| anyhow!("merge requires --into <target>"))?;
                    let draft = store.create_merge_draft(&target, &args.sources)?;
                    println!("Created merge draft {}", draft.id);
                    println!("Path: {}", draft.path.display());
                }
            }
        }
        Command::Prune(args) => {
            if !args.suggest {
                return Err(anyhow!("prune currently requires --suggest"));
            }
            let store = PolarisStore::from_current_dir()?;
            store.require_initialized()?;
            let suggestions = store.prune_suggestions()?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&suggestions)?);
            } else if suggestions.is_empty() {
                println!("No prune suggestions are available.");
            } else {
                println!("Prune suggestions:");
                for suggestion in suggestions {
                    println!("- {}", suggestion.candidate_keys.join(", "));
                    for reason in suggestion.reasons {
                        println!("  Reason: {reason}");
                    }
                    for command in suggestion.suggested_commands {
                        println!("  Command: {command}");
                    }
                }
            }
        }
        Command::Compact(args) => {
            if !args.suggest {
                return Err(anyhow!("compact currently requires --suggest"));
            }
            let store = PolarisStore::from_current_dir()?;
            store.require_initialized()?;
            let suggestions = store.compact_suggestions()?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&suggestions)?);
            } else if suggestions.is_empty() {
                println!("No compact suggestions are available.");
            } else {
                println!("Compact suggestions:");
                for suggestion in suggestions {
                    println!(
                        "- {} -> {}",
                        suggestion.source_keys.join(", "),
                        suggestion.proposed_target_key
                    );
                    for reason in suggestion.reasons {
                        println!("  Reason: {reason}");
                    }
                    println!("  Command: {}", suggestion.suggested_command);
                }
            }
        }
        Command::List(args) => {
            if args.keys == args.json {
                return Err(anyhow!("list requires exactly one of --keys or --json"));
            }
            let store = PolarisStore::from_current_dir()?;
            store.require_initialized()?;
            let filter = MemoryFilter {
                lifecycle: args.lifecycle,
                ..MemoryFilter::default()
            };
            if args.keys {
                for key in store.list_keys(&filter)? {
                    println!("{key}");
                }
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&store.list_summaries(&filter)?)?
                );
            }
        }
        Command::Note(args) => match args.command {
            NoteCommand::Create(args) => {
                let store = PolarisStore::from_current_dir()?;
                store.require_initialized()?;
                let note = store.create_note(&args.title, args.lifecycle)?;
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
                    lifecycle: args.lifecycle,
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
