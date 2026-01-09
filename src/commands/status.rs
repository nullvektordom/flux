use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::context::GitContext;

pub fn run() -> Result<()> {
    let config = Config::load()?;
    let profile = config.get_active_profile()?;

    println!("{} {}", "Profile:".bold(), profile.name.cyan());

    // Try to gather git context
    match GitContext::gather(profile) {
        Ok(context) => {
            // Branch info
            let branch_status = if context.is_protected_branch {
                format!("{} ({})", context.current_branch, "protected".red())
            } else {
                format!("{} ({})", context.current_branch, "safe".green())
            };
            println!("{} {}", "Branch:".bold(), branch_status);

            // Protected branches
            if !profile.protected_branches.is_empty() {
                println!(
                    "{} {}",
                    "Protected:".bold(),
                    profile.protected_branches.join(", ")
                );
            }

            // Repository path
            println!("{} {}", "Repository:".bold(), context.repo_path.display());

            // Staged changes
            if !context.staged_files.is_empty() {
                println!("\n{}", "Staged changes:".bold().green());
                for file in &context.staged_files {
                    println!("  {} {}", "●".green(), file);
                }
            }

            // Unstaged changes
            if !context.unstaged_files.is_empty() {
                println!("\n{}", "Unstaged changes:".bold().yellow());
                for file in &context.unstaged_files {
                    println!("  {} {}", "●".yellow(), file);
                }
            }

            // Untracked files
            if !context.untracked_files.is_empty() {
                println!("\n{}", "Untracked files:".bold().red());
                for file in &context.untracked_files {
                    println!("  {} {}", "●".red(), file);
                }
            }

            // Clean status
            if context.total_changes() == 0 {
                println!("\n{}", "Working tree clean ✓".green());
            }
        }
        Err(e) => {
            println!("{} {}", "Branch:".bold(), "N/A (not in git repository)".red());
            println!("\n{} {}", "Error:".red(), e);
        }
    }

    Ok(())
}
