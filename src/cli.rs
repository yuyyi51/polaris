use crate::hook::{self, HookTarget};
use crate::storage::{MemoryInput, PolarisStore};
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
    Note(NoteArgs),
    Recall,
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
    key: String,
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
    SessionStart(HookTargetArgs),
    #[command(name = "post-compact")]
    PostCompact(HookTargetArgs),
    #[command(name = "post-tool-use")]
    PostToolUse(HookTargetArgs),
}

#[derive(Args)]
struct HookTargetArgs {
    /// Host adapter that shapes hook stdin/stdout. Defaults to `codex`.
    #[arg(long, value_enum, default_value_t = HookTarget::default())]
    target: HookTarget,
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
            store.forget(&args.key)?;
            println!("Forgot memory {}", args.key);
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
        Command::Recall => {
            let store = PolarisStore::from_current_dir()?;
            store.require_initialized()?;
            print!("{}", store.recall()?);
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
            HookCommand::SessionStart(args) => {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                if let Some(output) =
                    hook::session_start_output(args.target, &input, |cwd: PathBuf| {
                        PolarisStore::from_workspace(cwd)
                    })?
                {
                    println!("{}", output.to_json_string()?);
                }
            }
            HookCommand::PostCompact(args) => {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                if let Some(output) = hook::post_compact(args.target, &input, |cwd: PathBuf| {
                    PolarisStore::from_workspace(cwd)
                })? {
                    println!("{}", output.to_json_string()?);
                }
            }
            HookCommand::PostToolUse(args) => {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                if let Some(output) =
                    hook::post_tool_use_output(args.target, &input, |cwd: PathBuf| {
                        PolarisStore::from_workspace(cwd)
                    })?
                {
                    println!("{}", output.to_json_string()?);
                }
            }
        },
    }
    Ok(())
}
