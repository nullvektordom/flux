use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

use crate::llm::{LLMRequest, OllamaClient, OperationType};

/// Pull request data
#[derive(Debug, Clone)]
pub struct PullRequest {
    pub title: String,
    pub body: String,
    pub base_branch: String,
    pub head_branch: String,
    pub draft: bool,
    pub labels: Vec<String>,
}

/// Analysis of commits for PR generation
#[derive(Debug, Clone)]
pub struct CommitAnalysis {
    pub commits: Vec<CommitInfo>,
    pub commits_by_type: HashMap<String, Vec<CommitInfo>>,
    pub breaking_changes: Vec<String>,
    pub linked_issues: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub commit_type: Option<String>,
    pub scope: Option<String>,
    pub description: String,
}

impl CommitAnalysis {
    /// Analyze commits from git log
    pub fn from_git_log(base_branch: &str, head_branch: &str) -> Result<Self> {
        let output = Command::new("git")
            .args(["log", &format!("{}..{}", base_branch, head_branch), "--pretty=format:%H|||%s"])
            .output()
            .context("Failed to run git log")?;

        if !output.status.success() {
            anyhow::bail!("Git log failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let log = String::from_utf8(output.stdout)?;
        let mut commits = Vec::new();
        let mut commits_by_type: HashMap<String, Vec<CommitInfo>> = HashMap::new();
        let mut breaking_changes = Vec::new();
        let mut linked_issues = Vec::new();

        for line in log.lines() {
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split("|||").collect();
            if parts.len() < 2 {
                continue;
            }

            let hash = parts[0].to_string();
            let message = parts[1].to_string();

            // Parse conventional commit
            let (commit_type, scope, description) = parse_conventional_commit(&message);

            let commit_info = CommitInfo {
                hash: hash[..7].to_string(), // Short hash
                message: message.clone(),
                commit_type: commit_type.clone(),
                scope,
                description: description.clone(),
            };

            // Check for breaking changes
            if message.contains("BREAKING CHANGE") || message.contains('!') {
                breaking_changes.push(description.clone());
            }

            // Extract issue references (#123, JIRA-456, etc.)
            for word in message.split_whitespace() {
                if word.starts_with('#') || word.contains('-') && word.chars().any(|c| c.is_numeric()) {
                    if !linked_issues.contains(&word.to_string()) {
                        linked_issues.push(word.to_string());
                    }
                }
            }

            // Group by type
            if let Some(ref t) = commit_type {
                commits_by_type.entry(t.clone()).or_default().push(commit_info.clone());
            }

            commits.push(commit_info);
        }

        Ok(CommitAnalysis {
            commits,
            commits_by_type,
            breaking_changes,
            linked_issues,
        })
    }

    /// Generate PR body from analysis
    pub fn generate_pr_body(&self, nexus_context: Option<&crate::nexus::NexusContext>) -> String {
        let mut body = String::new();

        // Summary
        body.push_str("## Summary\n\n");
        body.push_str(&format!("This PR includes {} commits.\n\n", self.commits.len()));

        // Nexus context if available
        if let Some(nexus) = nexus_context {
            body.push_str(&format!("**Project:** {}\n\n", nexus.format_summary()));
        }

        // Changes by type
        for (commit_type, commits) in &self.commits_by_type {
            let section_title = match commit_type.as_str() {
                "feat" => "✨ Features",
                "fix" => "🐛 Bug Fixes",
                "docs" => "📚 Documentation",
                "style" => "💄 Style",
                "refactor" => "♻️ Refactoring",
                "test" => "✅ Tests",
                "chore" => "🔧 Chores",
                _ => commit_type.as_str(),
            };

            body.push_str(&format!("### {}\n\n", section_title));
            for commit in commits {
                body.push_str(&format!("- {} ({})\n", commit.description, commit.hash));
            }
            body.push_str("\n");
        }

        // Breaking changes
        if !self.breaking_changes.is_empty() {
            body.push_str("### ⚠️ BREAKING CHANGES\n\n");
            for change in &self.breaking_changes {
                body.push_str(&format!("- {}\n", change));
            }
            body.push_str("\n");
        }

        // Linked issues
        if !self.linked_issues.is_empty() {
            body.push_str("### 🔗 Linked Issues\n\n");
            for issue in &self.linked_issues {
                body.push_str(&format!("- {}\n", issue));
            }
            body.push_str("\n");
        }

        body.push_str("---\n");
        body.push_str("*Generated by [Flux](https://github.com/nullvektor/flux)*\n");

        body
    }
}

/// Parse conventional commit message
fn parse_conventional_commit(message: &str) -> (Option<String>, Option<String>, String) {
    // Try to parse: type(scope): description
    if let Some(colon_pos) = message.find(':') {
        let prefix = &message[..colon_pos];
        let description = message[colon_pos + 1..].trim().to_string();

        if let Some(paren_pos) = prefix.find('(') {
            let commit_type = prefix[..paren_pos].trim().to_string();
            let scope = prefix[paren_pos + 1..].trim_end_matches(')').to_string();
            (Some(commit_type), Some(scope), description)
        } else {
            let commit_type = prefix.trim().to_string();
            (Some(commit_type), None, description)
        }
    } else {
        (None, None, message.to_string())
    }
}

/// Create PR using GitHub CLI
pub fn create_pr_with_gh(pr: &PullRequest) -> Result<String> {
    let mut args = vec![
        "pr",
        "create",
        "--title",
        &pr.title,
        "--body",
        &pr.body,
        "--base",
        &pr.base_branch,
        "--head",
        &pr.head_branch,
    ];

    if pr.draft {
        args.push("--draft");
    }

    let output = Command::new("gh")
        .args(&args)
        .output()
        .context("Failed to run gh CLI. Is it installed and authenticated?")?;

    if !output.status.success() {
        anyhow::bail!("gh pr create failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let url = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_conventional_commit() {
        let (t, s, d) = parse_conventional_commit("feat(auth): add login");
        assert_eq!(t, Some("feat".to_string()));
        assert_eq!(s, Some("auth".to_string()));
        assert_eq!(d, "add login");

        let (t, s, d) = parse_conventional_commit("fix: correct bug");
        assert_eq!(t, Some("fix".to_string()));
        assert_eq!(s, None);
        assert_eq!(d, "correct bug");
    }
}
