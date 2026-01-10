use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Workflow template defining branch patterns and merge strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    /// Template name
    pub name: String,

    /// Branch naming pattern (e.g., "feature/{name}", "hotfix/{name}")
    pub branch_pattern: String,

    /// Merge strategy: merge, squash, rebase
    pub merge_strategy: MergeStrategy,

    /// Tag pattern (e.g., "v{version}", "release-{version}")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_pattern: Option<String>,

    /// Auto-push after completion
    #[serde(default)]
    pub auto_push: bool,

    /// Target branch for merge (defaults to profile default_branch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_branch: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MergeStrategy {
    Merge,
    Squash,
    Rebase,
}

impl MergeStrategy {
    pub fn as_str(&self) -> &str {
        match self {
            MergeStrategy::Merge => "merge",
            MergeStrategy::Squash => "squash",
            MergeStrategy::Rebase => "rebase",
        }
    }
}

/// Current workflow state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Branch created for this workflow
    pub branch_name: String,

    /// Original branch before workflow started
    pub original_branch: String,

    /// Template used
    pub template_name: String,

    /// Steps completed
    pub steps: Vec<WorkflowStep>,

    /// Current step index
    pub current_step: usize,

    /// Can rollback to previous state
    pub can_rollback: bool,

    /// Timestamp when workflow started
    pub started_at: String,
}

/// Individual workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Type of step
    pub step_type: StepType,

    /// Status of step
    pub status: StepStatus,

    /// Output message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,

    /// Timestamp
    pub timestamp: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StepType {
    CreateBranch,
    Commit,
    Merge,
    Tag,
    Push,
}

impl StepType {
    pub fn as_str(&self) -> &str {
        match self {
            StepType::CreateBranch => "create_branch",
            StepType::Commit => "commit",
            StepType::Merge => "merge",
            StepType::Tag => "tag",
            StepType::Push => "push",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Pending,
    Running,
    Complete,
    Failed,
    Skipped,
}

impl WorkflowState {
    /// Create new workflow state
    pub fn new(branch_name: String, original_branch: String, template_name: String) -> Self {
        Self {
            branch_name,
            original_branch,
            template_name,
            steps: Vec::new(),
            current_step: 0,
            can_rollback: true,
            started_at: chrono::Local::now().to_rfc3339(),
        }
    }

    /// Get state file path
    pub fn state_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("flux");

        Ok(config_dir.join("workflow-state.json"))
    }

    /// Save state to file
    pub fn save(&self) -> Result<()> {
        let path = Self::state_file_path()?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&path, contents)?;

        Ok(())
    }

    /// Load state from file
    pub fn load() -> Result<Self> {
        let path = Self::state_file_path()?;

        if !path.exists() {
            anyhow::bail!("No active workflow. Use 'flux flow start' to begin.");
        }

        let contents = fs::read_to_string(&path)?;
        let state: WorkflowState = serde_json::from_str(&contents)?;

        Ok(state)
    }

    /// Check if workflow state exists
    pub fn exists() -> bool {
        Self::state_file_path()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Delete state file
    pub fn clear() -> Result<()> {
        let path = Self::state_file_path()?;
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Add a step to the workflow
    pub fn add_step(&mut self, step: WorkflowStep) {
        let is_complete = step.status == StepStatus::Complete;
        self.steps.push(step);
        if is_complete {
            self.current_step += 1;
        }
    }

    /// Mark current step as complete
    pub fn complete_current_step(&mut self, output: Option<String>) {
        if let Some(step) = self.steps.last_mut() {
            step.status = StepStatus::Complete;
            step.output = output;
            step.timestamp = chrono::Local::now().to_rfc3339();
        }
    }

    /// Mark current step as failed
    pub fn fail_current_step(&mut self, error: String) {
        if let Some(step) = self.steps.last_mut() {
            step.status = StepStatus::Failed;
            step.output = Some(error);
            step.timestamp = chrono::Local::now().to_rfc3339();
        }
        self.can_rollback = false;
    }
}

impl WorkflowTemplate {
    /// Get default templates
    pub fn defaults() -> Vec<Self> {
        vec![
            WorkflowTemplate {
                name: "feature".to_string(),
                branch_pattern: "feature/{name}".to_string(),
                merge_strategy: MergeStrategy::Squash,
                tag_pattern: None,
                auto_push: false,
                target_branch: None,
            },
            WorkflowTemplate {
                name: "bugfix".to_string(),
                branch_pattern: "bugfix/{name}".to_string(),
                merge_strategy: MergeStrategy::Squash,
                tag_pattern: None,
                auto_push: false,
                target_branch: None,
            },
            WorkflowTemplate {
                name: "hotfix".to_string(),
                branch_pattern: "hotfix/{name}".to_string(),
                merge_strategy: MergeStrategy::Merge,
                tag_pattern: Some("v{version}".to_string()),
                auto_push: true,
                target_branch: None,
            },
            WorkflowTemplate {
                name: "release".to_string(),
                branch_pattern: "release/{version}".to_string(),
                merge_strategy: MergeStrategy::Merge,
                tag_pattern: Some("v{version}".to_string()),
                auto_push: true,
                target_branch: None,
            },
        ]
    }

    /// Apply template pattern with name
    pub fn format_branch_name(&self, name: &str) -> String {
        self.branch_pattern.replace("{name}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_template_format_branch() {
        let template = WorkflowTemplate {
            name: "feature".to_string(),
            branch_pattern: "feature/{name}".to_string(),
            merge_strategy: MergeStrategy::Squash,
            tag_pattern: None,
            auto_push: false,
            target_branch: None,
        };

        assert_eq!(template.format_branch_name("auth"), "feature/auth");
    }

    #[test]
    fn test_workflow_state_creation() {
        let state = WorkflowState::new(
            "feature/test".to_string(),
            "main".to_string(),
            "feature".to_string(),
        );

        assert_eq!(state.branch_name, "feature/test");
        assert_eq!(state.original_branch, "main");
        assert_eq!(state.current_step, 0);
        assert!(state.can_rollback);
    }

    #[test]
    fn test_merge_strategy_serialization() {
        assert_eq!(MergeStrategy::Merge.as_str(), "merge");
        assert_eq!(MergeStrategy::Squash.as_str(), "squash");
        assert_eq!(MergeStrategy::Rebase.as_str(), "rebase");
    }

    #[test]
    fn test_default_templates() {
        let templates = WorkflowTemplate::defaults();
        assert_eq!(templates.len(), 4);

        let feature = templates.iter().find(|t| t.name == "feature").unwrap();
        assert_eq!(feature.merge_strategy, MergeStrategy::Squash);
    }
}
