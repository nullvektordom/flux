mod config;
mod commands;
mod context;
mod git;
mod llm;
mod nexus;
mod pr;
mod validator;
mod workflow;

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

    /// Workflow orchestration (branch creation, merge, etc.)
    Flow {
        #[command(subcommand)]
        action: FlowAction,
    },

    /// Pull request automation
    Pr {
        #[command(subcommand)]
        action: PrAction,
    },

    /// Interactive shell (REPL mode)
    Shell,
}

#[derive(Subcommand)]
enum FlowAction {
    /// Start a new workflow and create branch
    Start {
        /// Branch name (e.g., 'auth' or 'feature/auth')
        name: Option<String>,

        /// Preview without creating branch
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Finish workflow (commit, merge, cleanup)
    Finish {
        /// Preview without executing
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Abort workflow and return to original branch
    Abort,

    /// Show current workflow status
    Status,
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

#[derive(Subcommand)]
enum PrAction {
    /// Create PR from current branch
    Create {
        /// Create as draft PR
        #[arg(short, long)]
        draft: bool,

        /// Target branch (defaults to profile default_branch)
        #[arg(short, long)]
        base: Option<String>,

        /// Preview only, don't create PR
        #[arg(short = 'n', long)]
        preview: bool,
    },

    /// Preview PR without creating
    Preview {
        /// Target branch (defaults to profile default_branch)
        #[arg(short, long)]
        base: Option<String>,
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
        Some(Commands::Flow { action }) => {
            match action {
                FlowAction::Start { name, dry_run } => commands::flow::start(name, dry_run)?,
                FlowAction::Finish { dry_run } => commands::flow::finish(dry_run)?,
                FlowAction::Abort => commands::flow::abort()?,
                FlowAction::Status => commands::flow::status()?,
            }
        }
        Some(Commands::Pr { action }) => {
            match action {
                PrAction::Create { draft, base, preview } => {
                    let options = commands::pr::PrOptions {
                        draft,
                        base,
                        preview_only: preview,
                    };
                    commands::pr::create(options)?;
                }
                PrAction::Preview { base } => {
                    let options = commands::pr::PrOptions {
                        draft: false,
                        base,
                        preview_only: true,
                    };
                    commands::pr::create(options)?;
                }
            }
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
