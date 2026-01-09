use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// User's git identity and workflow rules for a specific context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Profile identifier
    pub name: String,

    /// Git user.name
    pub user_name: String,

    /// Git user.email
    pub user_email: String,

    /// Enable GPG signing
    #[serde(default)]
    pub gpg_signing: bool,

    /// GPG key ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpg_key: Option<String>,

    /// List of protected branches
    #[serde(default)]
    pub protected_branches: Vec<String>,

    /// Default branch name
    #[serde(default = "default_branch_name")]
    pub default_branch: String,

    /// Natural language workflow rules
    #[serde(default)]
    pub rules: Vec<String>,

    /// Enforce conventional commits
    #[serde(default = "default_true")]
    pub conventional_commits: bool,

    /// Auto-push after commit
    #[serde(default)]
    pub auto_push: bool,
}

fn default_branch_name() -> String {
    "main".to_string()
}

fn default_true() -> bool {
    true
}

/// Configuration file containing all profiles
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Active profile name
    pub active_profile: String,

    /// All available profiles
    pub profiles: Vec<Profile>,
}

impl Config {
    /// Get the config file path (~/.config/flux/flux.toml)
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("flux");

        Ok(config_dir.join("flux.toml"))
    }

    /// Load config from file
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            anyhow::bail!(
                "Flux not initialized. Run 'flux init' to create your first profile.\nExpected config at: {}",
                path.display()
            );
        }

        let contents = fs::read_to_string(&path)
            .context("Failed to read config file")?;

        let config: Config = toml::from_str(&contents)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // Ensure config directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&path, contents)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Get the currently active profile
    pub fn get_active_profile(&self) -> Result<&Profile> {
        self.profiles
            .iter()
            .find(|p| p.name == self.active_profile)
            .context(format!("Active profile '{}' not found", self.active_profile))
    }

    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    /// Switch to a different profile
    pub fn switch_profile(&mut self, name: &str) -> Result<()> {
        if !self.profiles.iter().any(|p| p.name == name) {
            anyhow::bail!("Profile '{}' does not exist", name);
        }

        self.active_profile = name.to_string();
        Ok(())
    }

    /// Add a new profile
    pub fn add_profile(&mut self, profile: Profile) -> Result<()> {
        if self.profiles.iter().any(|p| p.name == profile.name) {
            anyhow::bail!("Profile '{}' already exists", profile.name);
        }

        self.profiles.push(profile);
        Ok(())
    }

    /// Check if config exists
    pub fn exists() -> bool {
        Self::config_path()
            .map(|p| p.exists())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_serialization() {
        let profile = Profile {
            name: "work".to_string(),
            user_name: "John Doe".to_string(),
            user_email: "john@example.com".to_string(),
            gpg_signing: true,
            gpg_key: Some("ABC123".to_string()),
            protected_branches: vec!["main".to_string(), "develop".to_string()],
            default_branch: "main".to_string(),
            rules: vec!["Use conventional commits".to_string()],
            conventional_commits: true,
            auto_push: false,
        };

        let toml = toml::to_string(&profile).unwrap();
        let deserialized: Profile = toml::from_str(&toml).unwrap();

        assert_eq!(profile.name, deserialized.name);
        assert_eq!(profile.user_email, deserialized.user_email);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            active_profile: "work".to_string(),
            profiles: vec![
                Profile {
                    name: "work".to_string(),
                    user_name: "John Doe".to_string(),
                    user_email: "john@example.com".to_string(),
                    gpg_signing: false,
                    gpg_key: None,
                    protected_branches: vec!["main".to_string()],
                    default_branch: "main".to_string(),
                    rules: vec![],
                    conventional_commits: true,
                    auto_push: false,
                },
            ],
        };

        let toml = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml).unwrap();

        assert_eq!(config.active_profile, deserialized.active_profile);
        assert_eq!(config.profiles.len(), deserialized.profiles.len());
    }
}
