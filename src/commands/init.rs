use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Input};

use crate::config::{Config, Profile};

pub fn run() -> Result<()> {
    println!("{}", "Welcome to Flux!".bold().cyan());
    println!("Let's set up your first profile.\n");

    // Check if already initialized
    if Config::exists() {
        let overwrite = Confirm::new()
            .with_prompt("Flux is already initialized. Do you want to create a new profile instead?")
            .default(false)
            .interact()?;

        if overwrite {
            // Load existing config and add new profile
            let mut config = Config::load()?;
            let profile = create_profile_interactive()?;
            config.add_profile(profile)?;
            config.save()?;
            println!("\n{} Profile '{}' created!", "✓".green(), config.profiles.last().unwrap().name);
            return Ok(());
        } else {
            println!("Use 'flux profile create' to add a new profile.");
            return Ok(());
        }
    }

    // Create first profile
    let profile = create_profile_interactive()?;
    let profile_name = profile.name.clone();

    // Create new config
    let config = Config {
        active_profile: profile_name.clone(),
        profiles: vec![profile],
    };

    // Save config
    config.save()
        .context("Failed to save configuration")?;

    println!("\n{} Profile '{}' created and activated!", "✓".green(), profile_name);
    println!("Config saved to: {}", Config::config_path()?.display());
    println!("\nYou can now use flux in any git repository.");
    println!("Try: {}", "flux status".cyan());

    Ok(())
}

pub fn create_profile_interactive() -> Result<Profile> {
    let name: String = Input::new()
        .with_prompt("Profile name")
        .default("personal".to_string())
        .interact_text()?;

    let user_name: String = Input::new()
        .with_prompt("Git name")
        .interact_text()?;

    let user_email: String = Input::new()
        .with_prompt("Git email")
        .interact_text()?;

    let gpg_signing = Confirm::new()
        .with_prompt("Enable GPG signing?")
        .default(false)
        .interact()?;

    let gpg_key = if gpg_signing {
        Some(Input::new()
            .with_prompt("GPG key ID")
            .interact_text()?)
    } else {
        None
    };

    let protected_branches_input: String = Input::new()
        .with_prompt("Protected branches (comma-separated)")
        .default("main,develop".to_string())
        .interact_text()?;

    let protected_branches: Vec<String> = protected_branches_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let default_branch: String = Input::new()
        .with_prompt("Default branch")
        .default("main".to_string())
        .interact_text()?;

    println!("\nDefine workflow rules (one per line, empty line to finish):");
    let mut rules = Vec::new();
    loop {
        let rule: String = Input::new()
            .with_prompt(">")
            .allow_empty(true)
            .interact_text()?;

        if rule.is_empty() {
            break;
        }
        rules.push(rule);
    }

    let conventional_commits = Confirm::new()
        .with_prompt("Enforce conventional commits?")
        .default(true)
        .interact()?;

    let auto_push = Confirm::new()
        .with_prompt("Auto-push after commit?")
        .default(false)
        .interact()?;

    Ok(Profile {
        name,
        user_name,
        user_email,
        gpg_signing,
        gpg_key,
        protected_branches,
        default_branch,
        rules,
        conventional_commits,
        auto_push,
    })
}
