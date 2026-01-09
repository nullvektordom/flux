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
                println!("commit - not yet implemented");
            }
            "status" => {
                commands::status::run()?;
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
