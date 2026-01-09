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
            println!("flux init - not yet implemented");
        }
        Some(Commands::Profile { action }) => {
            match action {
                ProfileAction::List => println!("flux profile list - not yet implemented"),
                ProfileAction::Switch { name } => println!("flux profile switch {} - not yet implemented", name),
                ProfileAction::Create => println!("flux profile create - not yet implemented"),
                ProfileAction::Edit { name } => println!("flux profile edit {} - not yet implemented", name),
            }
        }
        Some(Commands::Commit { all, dry_run }) => {
            println!("flux commit (all: {}, dry_run: {}) - not yet implemented", all, dry_run);
        }
        Some(Commands::Status) => {
            println!("flux status - not yet implemented");
        }
        Some(Commands::Shell) => {
            println!("flux shell - not yet implemented");
        }
        None => {
            println!("Flux - AI-guided Git workflow assistant");
            println!("Run 'flux --help' for usage information");
        }
    }

    Ok(())
}
