use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Confirm;
use git2::Repository;

use crate::config::Config;
use crate::git::GitOps;
use crate::workflow::{WorkflowState, WorkflowStep, WorkflowTemplate, StepType, StepStatus};

pub fn start(name: Option<String>, dry_run: bool) -> Result<()> {
    // Check if workflow already active
    if WorkflowState::exists() {
        anyhow::bail!("Workflow already active. Use 'flux flow abort' to cancel or 'flux flow finish' to complete.");
    }

    let config = Config::load()?;
    let profile = config.get_active_profile()?;

    // Get current branch
    let repo = GitOps::open_repo(None)?;
    let original_branch = GitOps::current_branch(&repo)?;

    // Determine branch name
    let branch_name = if let Some(n) = name {
        // Check if it looks like a template (feature/bugfix/hotfix)
        if let Some((template_name, task_name)) = n.split_once('/') {
            let templates = WorkflowTemplate::defaults();
            if let Some(template) = templates.iter().find(|t| t.name == template_name) {
                template.format_branch_name(task_name)
            } else {
                n
            }
        } else {
            // Default to feature template
            format!("feature/{}", n)
        }
    } else {
        anyhow::bail!("Branch name required. Usage: flux flow start <name>");
    };

    println!("{} Starting workflow", "→".cyan());
    println!("  Branch: {}", branch_name.cyan());
    println!("  From: {}", original_branch);

    if dry_run {
        println!("\n{} Dry run - no changes made", "ℹ".blue());
        return Ok(());
    }

    // Create and checkout branch
    create_branch(&repo, &branch_name, &original_branch)?;
    println!("{} Created and checked out branch '{}'", "✓".green(), branch_name);

    // Create workflow state
    let mut state = WorkflowState::new(
        branch_name.clone(),
        original_branch,
        "feature".to_string(),
    );

    state.add_step(WorkflowStep {
        step_type: StepType::CreateBranch,
        status: StepStatus::Complete,
        output: Some(format!("Created branch {}", branch_name)),
        timestamp: chrono::Local::now().to_rfc3339(),
    });

    state.save()?;

    println!("\n{} Workflow started. Make your changes and run 'flux flow finish' when done.", "✓".green().bold());

    Ok(())
}

pub fn finish(dry_run: bool) -> Result<()> {
    let mut state = WorkflowState::load()?;
    let config = Config::load()?;
    let profile = config.get_active_profile()?;

    println!("{} Finishing workflow for branch '{}'", "→".cyan(), state.branch_name);

    // Check for uncommitted changes
    let repo = GitOps::open_repo(None)?;
    if GitOps::has_uncommitted_changes(&repo)? {
        let should_commit = Confirm::new()
            .with_prompt("You have uncommitted changes. Commit them now?")
            .default(true)
            .interact()?;

        if should_commit {
            println!("{} Committing changes...", "→".cyan());
            if !dry_run {
                // Use our commit command
                crate::commands::commit::run(crate::commands::commit::CommitOptions {
                    stage_all: true,
                    dry_run: false,
                })?;

                state.add_step(WorkflowStep {
                    step_type: StepType::Commit,
                    status: StepStatus::Complete,
                    output: Some("Committed changes".to_string()),
                    timestamp: chrono::Local::now().to_rfc3339(),
                });
                state.save()?;
            }
        } else {
            anyhow::bail!("Cannot finish workflow with uncommitted changes");
        }
    }

    // Merge back to original branch
    let target_branch = state.original_branch.clone();
    println!("{} Merging into '{}'...", "→".cyan(), target_branch);

    if dry_run {
        println!("{} Dry run - would merge {} → {}", "ℹ".blue(), state.branch_name, target_branch);
    } else {
        merge_branch(&repo, &state.branch_name, &target_branch)?;
        println!("{} Merged successfully", "✓".green());

        state.add_step(WorkflowStep {
            step_type: StepType::Merge,
            status: StepStatus::Complete,
            output: Some(format!("Merged {} into {}", state.branch_name, target_branch)),
            timestamp: chrono::Local::now().to_rfc3339(),
        });
        state.save()?;
    }

    // Clean up
    if !dry_run {
        WorkflowState::clear()?;
    }

    println!("\n{} Workflow completed!", "✓".green().bold());
    Ok(())
}

pub fn abort() -> Result<()> {
    let state = WorkflowState::load()?;

    println!("{} Aborting workflow", "→".yellow());
    println!("  Current branch: {}", state.branch_name);
    println!("  Will return to: {}", state.original_branch);

    let confirm = Confirm::new()
        .with_prompt("Are you sure? This will discard the workflow branch.")
        .default(false)
        .interact()?;

    if !confirm {
        println!("{} Cancelled", "✗".yellow());
        return Ok(());
    }

    let repo = GitOps::open_repo(None)?;

    // Checkout original branch
    checkout_branch(&repo, &state.original_branch)?;
    println!("{} Returned to '{}'", "✓".green(), state.original_branch);

    // Delete workflow branch
    delete_branch(&repo, &state.branch_name)?;
    println!("{} Deleted branch '{}'", "✓".green(), state.branch_name);

    // Clear state
    WorkflowState::clear()?;

    println!("\n{} Workflow aborted", "✓".green().bold());
    Ok(())
}

pub fn status() -> Result<()> {
    if !WorkflowState::exists() {
        println!("{} No active workflow", "ℹ".blue());
        println!("Use 'flux flow start <name>' to begin a workflow");
        return Ok(());
    }

    let state = WorkflowState::load()?;

    println!("{}", "Active Workflow:".bold());
    println!("  Branch: {}", state.branch_name.cyan());
    println!("  Template: {}", state.template_name);
    println!("  Original branch: {}", state.original_branch);
    println!("  Started: {}", state.started_at);

    if !state.steps.is_empty() {
        println!("\n{}", "Steps:".bold());
        for (i, step) in state.steps.iter().enumerate() {
            let icon = match step.status {
                StepStatus::Complete => "✓".green(),
                StepStatus::Failed => "✗".red(),
                StepStatus::Running => "→".cyan(),
                StepStatus::Pending => "○".normal(),
                StepStatus::Skipped => "⊘".yellow(),
            };
            println!("  {} {} - {}", icon, step.step_type.as_str(), step.status.as_str());
        }
    }

    println!("\n{} Next: Run 'flux flow finish' to complete or 'flux flow abort' to cancel", "→".cyan());

    Ok(())
}

fn create_branch(repo: &Repository, branch_name: &str, from_branch: &str) -> Result<()> {
    // Get current commit
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;

    // Create branch
    repo.branch(branch_name, &commit, false)
        .context("Failed to create branch")?;

    // Checkout branch
    checkout_branch(repo, branch_name)?;

    Ok(())
}

fn checkout_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    let obj = repo.revparse_single(&format!("refs/heads/{}", branch_name))?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&format!("refs/heads/{}", branch_name))?;
    Ok(())
}

fn delete_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    let mut branch = repo.find_branch(branch_name, git2::BranchType::Local)?;
    branch.delete()?;
    Ok(())
}

fn merge_branch(repo: &Repository, from_branch: &str, to_branch: &str) -> Result<()> {
    // Checkout target branch
    checkout_branch(repo, to_branch)?;

    // Find the branch to merge
    let branch = repo.find_branch(from_branch, git2::BranchType::Local)?;
    let branch_ref = branch.get();
    let annotated_commit = repo.find_annotated_commit(branch_ref.target().unwrap())?;

    // Perform merge
    repo.merge(&[&annotated_commit], None, None)?;

    // Create merge commit if needed
    if repo.index()?.has_conflicts() {
        anyhow::bail!("Merge conflicts detected. Please resolve manually.");
    }

    // Commit merge
    let signature = repo.signature()?;
    let tree_id = repo.index()?.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let target_commit = repo.head()?.peel_to_commit()?;
    let merge_commit = branch_ref.peel_to_commit()?;

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &format!("Merge branch '{}' into '{}'", from_branch, to_branch),
        &tree,
        &[&target_commit, &merge_commit],
    )?;

    Ok(())
}

impl StepStatus {
    pub fn as_str(&self) -> &str {
        match self {
            StepStatus::Pending => "pending",
            StepStatus::Running => "running",
            StepStatus::Complete => "complete",
            StepStatus::Failed => "failed",
            StepStatus::Skipped => "skipped",
        }
    }
}
