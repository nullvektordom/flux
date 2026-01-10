use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Confirm;

use crate::config::Config;
use crate::context::GitContext;
use crate::git::GitOps;
use crate::llm::{LLMRequest, OllamaClient, OperationType};
use crate::pr::{CommitAnalysis, PullRequest, create_pr_with_gh};

pub struct PrOptions {
    pub draft: bool,
    pub base: Option<String>,
    pub preview_only: bool,
}

pub fn create(options: PrOptions) -> Result<()> {
    let config = Config::load()?;
    let profile = config.get_active_profile()?;

    println!("{} Analyzing commits...", "→".cyan());

    // Get current branch
    let repo = GitOps::open_repo(None)?;
    let current_branch = GitOps::current_branch(&repo)?;

    // Determine base branch
    let base_branch = options.base.unwrap_or_else(|| profile.default_branch.clone());

    // Analyze commits
    let analysis = CommitAnalysis::from_git_log(&base_branch, &current_branch)?;

    if analysis.commits.is_empty() {
        anyhow::bail!("No commits found between {} and {}", base_branch, current_branch);
    }

    println!("{} Found {} commits", "✓".green(), analysis.commits.len());

    // Gather context for LLM
    let git_context = GitContext::gather(profile)?;

    // Generate PR title with LLM
    println!("{} Generating PR title...", "→".cyan());
    let pr_title = generate_pr_title(&analysis, &git_context)?;

    // Generate PR body
    let pr_body = analysis.generate_pr_body(git_context.nexus_context.as_ref());

    let pr = PullRequest {
        title: pr_title.clone(),
        body: pr_body.clone(),
        base_branch: base_branch.clone(),
        head_branch: current_branch.clone(),
        draft: options.draft,
        labels: Vec::new(),
    };

    // Display preview
    println!("\n{}", "Pull Request Preview:".bold().cyan());
    println!("{}", "─".repeat(70));
    println!("{} {}", "Title:".bold(), pr.title);
    println!("{} {} → {}", "Branch:".bold(), pr.head_branch, pr.base_branch);
    if pr.draft {
        println!("{} {}", "Status:".bold(), "Draft".yellow());
    }
    println!("\n{}", "Body:".bold());
    println!("{}", pr.body);
    println!("{}", "─".repeat(70));

    if options.preview_only {
        println!("\n{} Preview mode - no PR created", "ℹ".blue());
        return Ok(());
    }

    // Confirm creation
    let confirm = Confirm::new()
        .with_prompt("Create this pull request?")
        .default(true)
        .interact()?;

    if !confirm {
        println!("{} Cancelled", "✗".yellow());
        return Ok(());
    }

    // Create PR via gh CLI
    println!("{} Creating pull request...", "→".cyan());
    let pr_url = create_pr_with_gh(&pr)?;

    println!("\n{} Pull request created!", "✓".green().bold());
    println!("{} {}", "URL:".bold(), pr_url.cyan());

    Ok(())
}

fn generate_pr_title(analysis: &CommitAnalysis, git_context: &GitContext) -> Result<String> {
    let client = OllamaClient::new();

    // Check if Ollama available
    if !client.is_available() {
        // Fallback to first commit message or generic title
        return Ok(analysis.commits.first()
            .map(|c| c.message.clone())
            .unwrap_or_else(|| "Update".to_string()));
    }

    // Build prompt for PR title
    let mut prompt = String::new();
    prompt.push_str("Generate a concise pull request title from these commits:\n\n");

    for commit in &analysis.commits {
        prompt.push_str(&format!("- {}\n", commit.message));
    }

    prompt.push_str("\nGenerate ONLY the title in conventional commit format (type: description). Keep it under 60 characters.");

    // Use simplified LLM request
    let response = client.generate(&prompt)?;

    // Clean up response
    let title = response.trim().trim_matches('"').to_string();

    Ok(title)
}
