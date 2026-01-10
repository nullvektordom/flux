mod config;
mod commands;
mod context;
mod git;
mod llm;
mod validator;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "flux")]
#[command(author, version, about, long_about = None)]
#[command(about = "AI-guided Git workflow assistant with profile-aware guardrails")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Flux and create first profile
    Init,

    /// Manage profiles (list, switch, create, edit)
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

    /// Smart commit with AI-generated message
    Commit {
        /// Stage all changes before committing
        #[arg(short, long)]
        all: bool,

        /// Show what would be committed without actually committing
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Show enhanced git status with profile info
    Status,

    /// Interactive shell (REPL mode)
    Shell,
}

#[derive(Subcommand)]
enum ProfileAction {
    /// List all profiles
    List,

    /// Switch to a different profile
    Switch {
        /// Profile name to switch to
        name: String,
    },

    /// Create a new profile
    Create,

    /// Edit an existing profile
    Edit {
        /// Profile name to edit
        name: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            commands::init::run()?;
        }
        Some(Commands::Profile { action }) => {
            match action {
                ProfileAction::List => commands::profile::list()?,
                ProfileAction::Switch { name } => commands::profile::switch(name)?,
                ProfileAction::Create => commands::profile::create()?,
                ProfileAction::Edit { name } => commands::profile::edit(name)?,
            }
        }
        Some(Commands::Commit { all, dry_run }) => {
            let options = commands::commit::CommitOptions {
                stage_all: all,
                dry_run,
            };
            commands::commit::run(options)?;
        }
        Some(Commands::Status) => {
            commands::status::run()?;
        }
        Some(Commands::Shell) => {
            commands::shell::run()?;
        }
        None => {
            println!("flux - AI-guided Git workflow assistant");
            println!("Run 'flux --help' for usage information");
        }
    }

    Ok(())
}
