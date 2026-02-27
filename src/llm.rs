use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::config::Profile;
use crate::context::GitContext;

const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_MODEL: &str = "claude-3-haiku-20240307";
const TIMEOUT_SECS: u64 = 10;

/// Request to LLM for generating content
#[derive(Debug, Clone)]
pub struct LLMRequest {
    pub profile: Profile,
    pub git_context: GitContext,
    pub operation_type: OperationType,
    pub user_prompt: Option<String>,
}

#[derive(Debug, Clone)]
pub enum OperationType {
    Commit,
    BranchName,
    Message,
}

impl OperationType {
    fn as_str(&self) -> &str {
        match self {
            OperationType::Commit => "commit",
            OperationType::BranchName => "branch",
            OperationType::Message => "message",
        }
    }
}

/// Anthropic Claude Haiku API request structure
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// LLM client for Anthropic Claude
pub struct AnthropicClient {
    base_url: String,
    model: String,
    timeout: Duration,
    api_key: String,
}

impl AnthropicClient {
    pub fn new() -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY environment variable not set");
        Self {
            base_url: ANTHROPIC_URL.to_string(),
            model: ANTHROPIC_MODEL.to_string(),
            timeout: Duration::from_secs(TIMEOUT_SECS),
            api_key,
        }
    }

    /// Generate commit message from context
    pub fn generate_commit_message(&self, request: &LLMRequest) -> Result<String> {
        let prompt = self.build_commit_prompt(request);
        self.generate(&prompt)
            .context("Failed to generate commit message from LLM")
    }

    /// Build prompt for commit message generation
    fn build_commit_prompt(&self, request: &LLMRequest) -> String {
        let mut prompt = String::new();
        prompt.push_str("You are a git commit message generator. Generate a conventional commit message.\n\n");
        if !request.profile.rules.is_empty() {
            prompt.push_str("Workflow rules:\n");
            for rule in &request.profile.rules {
                prompt.push_str(&format!("- {}\n", rule));
            }
            prompt.push_str("\n");
        }
        if let Some(ref nexus) = request.git_context.nexus_context {
            prompt.push_str(&format!("Project context: {}\n", nexus.format_summary()));
            if !nexus.is_adhoc_mode {
                prompt.push_str(&format!("Suggested scope: {}\n", nexus.scope_prefix()));
            }
            prompt.push_str("\n");
        }
        prompt.push_str("Staged files:\n");
        for file in &request.git_context.staged_files {
            prompt.push_str(&format!("- {}\n", file));
        }
        prompt.push_str("\n");
        if request.profile.conventional_commits {
            prompt.push_str("Use conventional commit format: <type>(<scope>): <description>\n");
            prompt.push_str("Types: feat, fix, docs, style, refactor, test, chore\n\n");
        }
        if let Some(ref user_prompt) = request.user_prompt {
            prompt.push_str(&format!("Additional context: {}\n\n", user_prompt));
        }
        prompt.push_str("Generate ONLY the commit message, without quotes or extra explanation. ");
        prompt.push_str("Use a concise subject line (50 chars max), then blank line, then bullet points for details if needed.");
        prompt
    }

    /// Send request to Anthropic Claude and get response
    pub fn generate(&self, prompt: &str) -> Result<String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .build()
            .context("Failed to build HTTP client")?;

        let request_body = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 128,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let response = client
            .post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .send()
            .context("Failed to connect to Anthropic API")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Anthropic API returned error status: {}",
                response.status()
            );
        }

        let json: serde_json::Value = response.json().context("Failed to parse Anthropic response")?;
        let content = json["content"][0]["text"].as_str().unwrap_or("").trim().to_string();
        if content.is_empty() {
            anyhow::bail!("Anthropic API returned empty response");
        }
        Ok(content)
    }
}

impl Default for AnthropicClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Profile;
    use crate::context::GitContext;
    use std::path::PathBuf;

    fn create_test_profile() -> Profile {
        Profile {
            name: "test".to_string(),
            user_name: "Test User".to_string(),
            user_email: "test@example.com".to_string(),
            gpg_signing: false,
            gpg_key: None,
            protected_branches: vec!["main".to_string()],
            default_branch: "main".to_string(),
            rules: vec!["Use conventional commits".to_string()],
            conventional_commits: true,
            auto_push: false,
        }
    }

    fn create_test_context() -> GitContext {
        GitContext {
            repo_path: PathBuf::from("/test"),
            current_branch: "feature".to_string(),
            staged_files: vec!["src/main.rs".to_string(), "README.md".to_string()],
            unstaged_files: vec![],
            untracked_files: vec![],
            is_protected_branch: false,
            has_uncommitted_changes: true,
            nexus_context: None,
        }
    }

    #[test]
    fn test_build_commit_prompt() {
        let client = AnthropicClient::new();
        let profile = create_test_profile();
        let context = create_test_context();

        let request = LLMRequest {
            profile,
            git_context: context,
            operation_type: OperationType::Commit,
            user_prompt: Some("Add new feature".to_string()),
        };

        let prompt = client.build_commit_prompt(&request);

        assert!(prompt.contains("conventional commit"));
        assert!(prompt.contains("src/main.rs"));
        assert!(prompt.contains("README.md"));
        assert!(prompt.contains("Use conventional commits"));
        assert!(prompt.contains("Add new feature"));
    }

    #[test]
    fn test_anthropic_client_creation() {
        std::env::set_var("ANTHROPIC_API_KEY", "dummy");
        let client = AnthropicClient::new();
        assert_eq!(client.model, ANTHROPIC_MODEL);
        assert_eq!(client.base_url, ANTHROPIC_URL);
    }

    #[test]
    fn test_operation_type_as_str() {
        assert_eq!(OperationType::Commit.as_str(), "commit");
        assert_eq!(OperationType::BranchName.as_str(), "branch");
        assert_eq!(OperationType::Message.as_str(), "message");
    }
}
