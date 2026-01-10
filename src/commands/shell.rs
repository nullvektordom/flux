use anyhow::{Context, Result};
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::path::PathBuf;

use crate::commands;
use crate::config::Config;

pub struct ReplSession {
    editor: DefaultEditor,
    active_profile: String,
    history_path: PathBuf,
}

impl ReplSession {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let active_profile = config.active_profile.clone();

        let history_path = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("flux")
            .join("history.txt");

        let mut editor = DefaultEditor::new()?;

        // Load history if it exists
        if history_path.exists() {
            let _ = editor.load_history(&history_path);
        }

        Ok(Self {
            editor,
            active_profile,
            history_path,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        println!("{}", "Flux Interactive Shell".bold().cyan());
        println!("Type 'help' for available commands, 'exit' or 'quit' to leave.\n");

        loop {
            let prompt = format!("flux({})> ", self.active_profile.green());

            match self.editor.readline(&prompt) {
                Ok(line) => {
                    let line = line.trim();

                    if line.is_empty() {
                        continue;
                    }

                    // Add to history
                    let _ = self.editor.add_history_entry(line);

                    // Handle command
                    if let Err(e) = self.handle_command(line) {
                        eprintln!("{} {}", "Error:".red(), e);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("exit");
                    break;
                }
                Err(err) => {
                    eprintln!("Error reading line: {}", err);
                    break;
                }
            }
        }

        // Save history before exiting
        self.save_history()?;

        Ok(())
    }

    fn handle_command(&mut self, input: &str) -> Result<()> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "exit" | "quit" => {
                self.save_history()?;
                std::process::exit(0);
            }
            "help" => {
                self.show_help();
            }
            "clear" => {
                print!("\x1B[2J\x1B[1;1H");
            }
            "init" => {
                commands::init::run()?;
                // Reload active profile after init
                let config = Config::load()?;
                self.active_profile = config.active_profile;
            }
            "profile" => {
                self.handle_profile_command(&parts[1..])?;
            }
            "commit" => {
                let mut stage_all = false;
                let mut dry_run = false;

                // Parse flags
                for arg in &parts[1..] {
                    match *arg {
                        "--all" | "-a" => stage_all = true,
                        "--dry-run" | "-n" => dry_run = true,
                        _ => eprintln!("{} Unknown flag: {}", "Warning:".yellow(), arg),
                    }
                }

                let options = commands::commit::CommitOptions { stage_all, dry_run };
                commands::commit::run(options)?;
            }
            "status" => {
                commands::status::run()?;
            }
            "flow" => {
                self.handle_flow_command(&parts[1..])?;
            }
            "pr" => {
                self.handle_pr_command(&parts[1..])?;
            }
            "shell" => {
                println!("{} Already in shell mode", "Note:".yellow());
            }
            cmd => {
                eprintln!("{} Unknown command: '{}'", "Error:".red(), cmd);
                eprintln!("Type 'help' for available commands.");
            }
        }

        Ok(())
    }

    fn handle_profile_command(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            eprintln!("Usage: profile <list|switch|create|edit>");
            return Ok(());
        }

        match args[0] {
            "list" => {
                commands::profile::list()?;
            }
            "switch" => {
                if args.len() < 2 {
                    eprintln!("Usage: profile switch <name>");
                    return Ok(());
                }
                commands::profile::switch(args[1].to_string())?;
                // Update active profile in session
                let config = Config::load()?;
                self.active_profile = config.active_profile;
            }
            "create" => {
                commands::profile::create()?;
            }
            "edit" => {
                if args.len() < 2 {
                    eprintln!("Usage: profile edit <name>");
                    return Ok(());
                }
                commands::profile::edit(args[1].to_string())?;
            }
            subcmd => {
                eprintln!("{} Unknown profile subcommand: '{}'", "Error:".red(), subcmd);
            }
        }

        Ok(())
    }

    fn handle_flow_command(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            eprintln!("Usage: flow <start|finish|abort|status>");
            return Ok(());
        }

        match args[0] {
            "start" => {
                let mut name = None;
                let mut dry_run = false;

                // Parse arguments
                let mut i = 1;
                while i < args.len() {
                    match args[i] {
                        "--dry-run" | "-n" => dry_run = true,
                        arg => {
                            if name.is_none() {
                                name = Some(arg.to_string());
                            } else {
                                eprintln!("{} Unknown argument: {}", "Warning:".yellow(), arg);
                            }
                        }
                    }
                    i += 1;
                }

                commands::flow::start(name, dry_run)?;
            }
            "finish" => {
                let mut dry_run = false;

                // Parse flags
                for arg in &args[1..] {
                    match *arg {
                        "--dry-run" | "-n" => dry_run = true,
                        _ => eprintln!("{} Unknown flag: {}", "Warning:".yellow(), arg),
                    }
                }

                commands::flow::finish(dry_run)?;
            }
            "abort" => {
                commands::flow::abort()?;
            }
            "status" => {
                commands::flow::status()?;
            }
            subcmd => {
                eprintln!("{} Unknown flow subcommand: '{}'", "Error:".red(), subcmd);
            }
        }

        Ok(())
    }

    fn handle_pr_command(&mut self, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            eprintln!("Usage: pr <create|preview>");
            return Ok(());
        }

        match args[0] {
            "create" => {
                let mut draft = false;
                let mut base = None;
                let mut preview = false;

                // Parse arguments
                let mut i = 1;
                while i < args.len() {
                    match args[i] {
                        "--draft" | "-d" => draft = true,
                        "--preview" | "-n" => preview = true,
                        "--base" | "-b" => {
                            if i + 1 < args.len() {
                                base = Some(args[i + 1].to_string());
                                i += 1;
                            } else {
                                eprintln!("{} --base requires a branch name", "Error:".red());
                                return Ok(());
                            }
                        }
                        arg => eprintln!("{} Unknown argument: {}", "Warning:".yellow(), arg),
                    }
                    i += 1;
                }

                let options = commands::pr::PrOptions {
                    draft,
                    base,
                    preview_only: preview,
                };
                commands::pr::create(options)?;
            }
            "preview" => {
                let mut base = None;

                // Parse arguments
                let mut i = 1;
                while i < args.len() {
                    match args[i] {
                        "--base" | "-b" => {
                            if i + 1 < args.len() {
                                base = Some(args[i + 1].to_string());
                                i += 1;
                            } else {
                                eprintln!("{} --base requires a branch name", "Error:".red());
                                return Ok(());
                            }
                        }
                        arg => eprintln!("{} Unknown argument: {}", "Warning:".yellow(), arg),
                    }
                    i += 1;
                }

                let options = commands::pr::PrOptions {
                    draft: false,
                    base,
                    preview_only: true,
                };
                commands::pr::create(options)?;
            }
            subcmd => {
                eprintln!("{} Unknown pr subcommand: '{}'", "Error:".red(), subcmd);
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("{}", "Available commands:".bold());
        println!("  {}          Initialize Flux and create first profile", "init".cyan());
        println!("  {}       Manage profiles", "profile".cyan());
        println!("    {} {}       List all profiles", "profile".cyan(), "list".cyan());
        println!("    {} {} <name> Switch to profile", "profile".cyan(), "switch".cyan());
        println!("    {} {}     Create new profile", "profile".cyan(), "create".cyan());
        println!("    {} {} <name>   View profile details", "profile".cyan(), "edit".cyan());
        println!("  {}        Smart commit with AI", "commit".cyan());
        println!("  {}        Show enhanced git status", "status".cyan());
        println!("  {}          Workflow orchestration", "flow".cyan());
        println!("    {} {}       Start new workflow", "flow".cyan(), "start".cyan());
        println!("    {} {}      Finish workflow", "flow".cyan(), "finish".cyan());
        println!("    {} {}       Abort workflow", "flow".cyan(), "abort".cyan());
        println!("    {} {}      Show workflow status", "flow".cyan(), "status".cyan());
        println!("  {}            Pull request automation", "pr".cyan());
        println!("    {} {}      Create PR", "pr".cyan(), "create".cyan());
        println!("    {} {}     Preview PR", "pr".cyan(), "preview".cyan());
        println!();
        println!("{}", "Special commands:".bold());
        println!("  {}          Show this help message", "help".cyan());
        println!("  {}         Clear screen", "clear".cyan());
        println!("  {}, {}       Exit shell", "exit".cyan(), "quit".cyan());
    }

    fn save_history(&mut self) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.history_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create history directory")?;
        }

        self.editor
            .save_history(&self.history_path)
            .context("Failed to save command history")?;

        Ok(())
    }
}

pub fn run() -> Result<()> {
    let mut session = ReplSession::new()?;
    session.run()
}
