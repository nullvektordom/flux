use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Nexus project metadata for enhanced commit context
#[derive(Debug, Clone)]
pub struct NexusContext {
    /// Current sprint number/name
    pub active_sprint: String,

    /// Sprint status
    pub sprint_status: String,

    /// Project name
    pub project_name: String,

    /// Whether in adhoc mode (outside sprint)
    pub is_adhoc_mode: bool,
}

/// Nexus configuration structure
#[derive(Debug, Deserialize)]
struct NexusConfig {
    project: ProjectConfig,
    #[serde(default)]
    state: Option<StateConfig>,
}

#[derive(Debug, Deserialize)]
struct ProjectConfig {
    name: String,
}

#[derive(Debug, Deserialize)]
struct StateConfig {
    active_sprint: Option<ActiveSprintConfig>,
}

#[derive(Debug, Deserialize)]
struct ActiveSprintConfig {
    current: String,
    #[serde(default)]
    status: String,
}

impl NexusContext {
    /// Detect and parse Nexus project if present
    pub fn detect() -> Option<Self> {
        // Look for nexus.toml in current directory
        let nexus_path = Path::new("nexus.toml");

        if !nexus_path.exists() {
            return None;
        }

        Self::from_config(nexus_path).ok()
    }

    /// Parse Nexus config from file
    fn from_config(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .context("Failed to read nexus.toml")?;

        let config: NexusConfig = toml::from_str(&contents)
            .context("Failed to parse nexus.toml")?;

        let (active_sprint, sprint_status, is_adhoc_mode) = if let Some(state) = config.state {
            if let Some(sprint) = state.active_sprint {
                (sprint.current, sprint.status, false)
            } else {
                ("adhoc".to_string(), "none".to_string(), true)
            }
        } else {
            ("adhoc".to_string(), "none".to_string(), true)
        };

        Ok(NexusContext {
            active_sprint,
            sprint_status,
            project_name: config.project.name,
            is_adhoc_mode,
        })
    }

    /// Format context for display
    pub fn format_summary(&self) -> String {
        if self.is_adhoc_mode {
            format!("Project: {} (adhoc work)", self.project_name)
        } else {
            format!(
                "Project: {} | Sprint: {} ({})",
                self.project_name, self.active_sprint, self.sprint_status
            )
        }
    }

    /// Get scope prefix for commit messages
    pub fn scope_prefix(&self) -> String {
        if self.is_adhoc_mode {
            "adhoc".to_string()
        } else {
            self.active_sprint.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_nexus_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
[project]
name = "test-project"

[state.active_sprint]
current = "sprint-3"
status = "in_progress"
        "#
        )
        .unwrap();

        let context = NexusContext::from_config(temp_file.path()).unwrap();

        assert_eq!(context.project_name, "test-project");
        assert_eq!(context.active_sprint, "sprint-3");
        assert_eq!(context.sprint_status, "in_progress");
        assert!(!context.is_adhoc_mode);
    }

    #[test]
    fn test_parse_nexus_config_adhoc() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            r#"
[project]
name = "test-project"
        "#
        )
        .unwrap();

        let context = NexusContext::from_config(temp_file.path()).unwrap();

        assert_eq!(context.project_name, "test-project");
        assert_eq!(context.active_sprint, "adhoc");
        assert!(context.is_adhoc_mode);
    }

    #[test]
    fn test_scope_prefix() {
        let context = NexusContext {
            active_sprint: "sprint-4".to_string(),
            sprint_status: "in_progress".to_string(),
            project_name: "test".to_string(),
            is_adhoc_mode: false,
        };

        assert_eq!(context.scope_prefix(), "sprint-4");

        let adhoc_context = NexusContext {
            active_sprint: "adhoc".to_string(),
            sprint_status: "none".to_string(),
            project_name: "test".to_string(),
            is_adhoc_mode: true,
        };

        assert_eq!(adhoc_context.scope_prefix(), "adhoc");
    }

    #[test]
    fn test_format_summary() {
        let context = NexusContext {
            active_sprint: "sprint-4".to_string(),
            sprint_status: "in_progress".to_string(),
            project_name: "flux".to_string(),
            is_adhoc_mode: false,
        };

        let summary = context.format_summary();
        assert!(summary.contains("flux"));
        assert!(summary.contains("sprint-4"));
        assert!(summary.contains("in_progress"));
    }
}
