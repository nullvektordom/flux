use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::commands::init::create_profile_interactive;

pub fn list() -> Result<()> {
    let config = Config::load()?;

    println!("{}", "Available profiles:".bold());
    for profile in &config.profiles {
        if profile.name == config.active_profile {
            println!("  {} {} (active)", "*".green(), profile.name.cyan());
        } else {
            println!("    {}", profile.name);
        }
    }

    Ok(())
}

pub fn switch(name: String) -> Result<()> {
    let mut config = Config::load()?;

    config.switch_profile(&name)?;
    config.save()?;

    println!("{} Switched to profile '{}'", "✓".green(), name.cyan());

    Ok(())
}

pub fn create() -> Result<()> {
    let mut config = Config::load()?;

    println!("{}", "Create a new profile".bold().cyan());
    let profile = create_profile_interactive()?;
    let name = profile.name.clone();

    config.add_profile(profile)?;
    config.save()?;

    println!("\n{} Profile '{}' created!", "✓".green(), name.cyan());
    println!("Use 'flux profile switch {}' to activate it.", name);

    Ok(())
}

pub fn edit(name: String) -> Result<()> {
    let config = Config::load()?;

    let profile = config.get_profile(&name)
        .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", name))?;

    println!("{} Profile '{}':", "Edit".bold().cyan(), name.cyan());
    println!("\nCurrent settings:");
    println!("  Name: {}", profile.name);
    println!("  Git name: {}", profile.user_name);
    println!("  Git email: {}", profile.user_email);
    println!("  GPG signing: {}", profile.gpg_signing);
    if let Some(ref key) = profile.gpg_key {
        println!("  GPG key: {}", key);
    }
    println!("  Protected branches: {}", profile.protected_branches.join(", "));
    println!("  Default branch: {}", profile.default_branch);
    println!("  Conventional commits: {}", profile.conventional_commits);
    println!("  Auto-push: {}", profile.auto_push);
    if !profile.rules.is_empty() {
        println!("  Rules:");
        for rule in &profile.rules {
            println!("    - {}", rule);
        }
    }

    println!("\n{} Direct config editing not yet implemented.", "Note:".yellow());
    println!("Edit manually at: {}", Config::config_path()?.display());

    Ok(())
}
