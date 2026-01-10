use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Editor, Select};
use git2::{IndexAddOption, Repository, Signature};

use crate::config::Config;
use crate::context::GitContext;
use crate::git::GitOps;
use crate::llm::{LLMRequest, OllamaClient, OperationType};
use crate::validator::Validator;

pub struct CommitOptions {
    pub stage_all: bool,
    pub dry_run: bool,
}

pub fn run(options: CommitOptions) -> Result<()> {
    let config = Config::load()?;
    let profile = config.get_active_profile()?;

    println!("{} Gathering context...", "→".cyan());

    // Stage all if requested
    if options.stage_all {
        stage_all_changes()?;
        println!("{} Staged all changes", "✓".green());
    }

    // Gather git context
    let context = GitContext::gather(profile)?;

    // Validate commit operation
    println!("{} Validating...", "→".cyan());
    let validation = Validator::validate_commit(profile, &context)?;

    if !validation.is_safe {
        eprintln!("\n{} Cannot proceed with commit:", "✗".red().bold());
        for violation in &validation.violations {
            eprintln!("  {} {}", "•".red(), violation);
        }
        return Ok(());
    }

    // Show warnings
    if !validation.warnings.is_empty() {
        println!("\n{}", "Warnings:".yellow().bold());
        for warning in &validation.warnings {
            println!("  {} {}", "⚠".yellow(), warning);
        }
        println!();
    }

    // Show context summary
    println!("\n{}", "Context:".bold());
    println!("  Branch: {}", context.current_branch.cyan());
    println!("  Profile: {}", profile.name.cyan());
    println!("  Staged files: {}", context.staged_files.len());

    if options.dry_run {
        println!("\n{} Dry run mode - no commit will be made", "ℹ".blue());
        list_staged_files(&context);
        return Ok(());
    }

    // Try to generate commit message with LLM
    let commit_message = generate_or_prompt_commit_message(profile, &context)?;

    // Show preview and confirm
    println!("\n{}", "Generated commit message:".bold().green());
    println!("{}", "─".repeat(60));
    println!("{}", commit_message);
    println!("{}", "─".repeat(60));

    list_staged_files(&context);

    // Interactive confirmation
    let choice = Select::new()
        .with_prompt("What would you like to do?")
        .items(&[
            "✓ Commit with this message",
            "✎ Edit message",
            "↻ Regenerate message",
            "✗ Cancel",
        ])
        .default(0)
        .interact()?;

    match choice {
        0 => {
            // Commit
            execute_commit(&commit_message)?;
            println!("\n{} Commit successful!", "✓".green().bold());
        }
        1 => {
            // Edit
            if let Some(edited) = Editor::new().edit(&commit_message)? {
                execute_commit(&edited)?;
                println!("\n{} Commit successful!", "✓".green().bold());
            } else {
                println!("\n{} Edit cancelled", "✗".yellow());
            }
        }
        2 => {
            // Regenerate
            println!("\n{} Regenerating...", "↻".cyan());
            let new_message = generate_or_prompt_commit_message(profile, &context)?;
            println!("\n{}", "New commit message:".bold().green());
            println!("{}", "─".repeat(60));
            println!("{}", new_message);
            println!("{}", "─".repeat(60));

            if Confirm::new()
                .with_prompt("Commit with this message?")
                .default(true)
                .interact()?
            {
                execute_commit(&new_message)?;
                println!("\n{} Commit successful!", "✓".green().bold());
            } else {
                println!("\n{} Cancelled", "✗".yellow());
            }
        }
        _ => {
            println!("\n{} Cancelled", "✗".yellow());
        }
    }

    Ok(())
}

fn stage_all_changes() -> Result<()> {
    let repo = GitOps::open_repo(None)?;
    let mut index = repo.index()?;

    // Stage all changes (modified, new, deleted)
    index
        .add_all(["."].iter(), IndexAddOption::DEFAULT, None)
        .context("Failed to stage changes")?;

    index.write().context("Failed to write index")?;

    Ok(())
}

fn list_staged_files(context: &GitContext) {
    println!("\n{}", "Staged files:".bold());
    for file in &context.staged_files {
        println!("  {} {}", "●".green(), file);
    }
}

fn generate_or_prompt_commit_message(
    profile: &crate::config::Profile,
    context: &GitContext,
) -> Result<String> {
    let client = OllamaClient::new();

    // Check if Ollama is available
    if !client.is_available() {
        println!(
            "\n{} Ollama is not available. Falling back to manual mode.",
            "⚠".yellow()
        );
        return manual_commit_message();
    }

    println!("{} Generating commit message with AI...", "→".cyan());

    let request = LLMRequest {
        profile: profile.clone(),
        git_context: context.clone(),
        operation_type: OperationType::Commit,
        user_prompt: None,
    };

    match client.generate_commit_message(&request) {
        Ok(message) => Ok(message),
        Err(e) => {
            eprintln!(
                "\n{} Failed to generate with LLM: {}",
                "⚠".yellow(),
                e
            );
            println!("Falling back to manual mode.");
            manual_commit_message()
        }
    }
}

fn manual_commit_message() -> Result<String> {
    println!("\n{}", "Enter commit message:".bold());

    if let Some(message) = Editor::new().edit("")? {
        if message.trim().is_empty() {
            anyhow::bail!("Commit message cannot be empty");
        }
        Ok(message)
    } else {
        anyhow::bail!("Commit cancelled")
    }
}

fn execute_commit(message: &str) -> Result<()> {
    let repo = GitOps::open_repo(None)?;

    // Get current config for signature
    let config = repo.config()?;
    let name = config
        .get_string("user.name")
        .context("Git user.name not configured")?;
    let email = config
        .get_string("user.email")
        .context("Git user.email not configured")?;

    let signature = Signature::now(&name, &email)?;

    // Get the current index (staged changes)
    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Get parent commit (HEAD)
    let parent_commit = match repo.head() {
        Ok(head) => {
            let oid = head.target().context("HEAD has no target")?;
            Some(repo.find_commit(oid)?)
        }
        Err(_) => None, // Initial commit
    };

    // Create commit
    let parents: Vec<_> = parent_commit.iter().collect();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    )
    .context("Failed to create commit")?;

    Ok(())
}
