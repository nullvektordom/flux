use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::config::Profile;
use crate::context::GitContext;

const OLLAMA_URL: &str = "http://localhost:11434/api/generate";
const OLLAMA_MODEL: &str = "qwen2.5-coder:14b";
const TIMEOUT_SECS: u64 = 5;

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

/// Ollama API request structure
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
}

/// Ollama API response structure
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
}

/// LLM client for Ollama
pub struct OllamaClient {
    base_url: String,
    model: String,
    timeout: Duration,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            base_url: OLLAMA_URL.to_string(),
            model: OLLAMA_MODEL.to_string(),
            timeout: Duration::from_secs(TIMEOUT_SECS),
        }
    }

    /// Check if Ollama is available
    pub fn is_available(&self) -> bool {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        let version_url = self.base_url.replace("/api/generate", "/api/version");
        client.get(&version_url).send().is_ok()
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

        // Add profile rules if any
        if !request.profile.rules.is_empty() {
            prompt.push_str("Workflow rules:\n");
            for rule in &request.profile.rules {
                prompt.push_str(&format!("- {}\n", rule));
            }
            prompt.push_str("\n");
        }

        // Add Nexus context if available
        if let Some(ref nexus) = request.git_context.nexus_context {
            prompt.push_str(&format!("Project context: {}\n", nexus.format_summary()));
            if !nexus.is_adhoc_mode {
                prompt.push_str(&format!("Suggested scope: {}\n", nexus.scope_prefix()));
            }
            prompt.push_str("\n");
        }

        // Add git context
        prompt.push_str("Staged files:\n");
        for file in &request.git_context.staged_files {
            prompt.push_str(&format!("- {}\n", file));
        }
        prompt.push_str("\n");

        // Add conventional commit format requirement
        if request.profile.conventional_commits {
            prompt.push_str("Use conventional commit format: <type>(<scope>): <description>\n");
            prompt.push_str("Types: feat, fix, docs, style, refactor, test, chore\n\n");
        }

        // Add user prompt if provided
        if let Some(ref user_prompt) = request.user_prompt {
            prompt.push_str(&format!("Additional context: {}\n\n", user_prompt));
        }

        prompt.push_str("Generate ONLY the commit message, without quotes or extra explanation. ");
        prompt.push_str("Use a concise subject line (50 chars max), then blank line, then bullet points for details if needed.");

        prompt
    }

    /// Send request to Ollama and get response
    pub fn generate(&self, prompt: &str) -> Result<String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .build()
            .context("Failed to build HTTP client")?;

        let request_body = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: Some(OllamaOptions { temperature: 0.0 }),
        };

        let response = client
            .post(&self.base_url)
            .json(&request_body)
            .send()
            .context("Failed to connect to Ollama. Is it running?")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Ollama returned error status: {}",
                response.status()
            );
        }

        let ollama_response: OllamaResponse = response
            .json()
            .context("Failed to parse Ollama response")?;

        let message = ollama_response.response.trim().to_string();

        if message.is_empty() {
            anyhow::bail!("Ollama returned empty response");
        }

        Ok(message)
    }
}

impl Default for OllamaClient {
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
        let client = OllamaClient::new();
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
    fn test_ollama_client_creation() {
        let client = OllamaClient::new();
        assert_eq!(client.model, OLLAMA_MODEL);
        assert_eq!(client.base_url, OLLAMA_URL);
    }

    #[test]
    fn test_operation_type_as_str() {
        assert_eq!(OperationType::Commit.as_str(), "commit");
        assert_eq!(OperationType::BranchName.as_str(), "branch");
        assert_eq!(OperationType::Message.as_str(), "message");
    }
}
