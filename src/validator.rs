use anyhow::Result;

use crate::context::GitContext;
use crate::config::Profile;

/// Result of safety validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether operation is safe to proceed
    pub is_safe: bool,

    /// Critical violations that block operation
    pub violations: Vec<String>,

    /// Non-blocking warnings
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn safe() -> Self {
        Self {
            is_safe: true,
            violations: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    pub fn with_violation(mut self, violation: String) -> Self {
        self.is_safe = false;
        self.violations.push(violation);
        self
    }
}

/// Safety validator for git operations
pub struct Validator;

impl Validator {
    /// Validate commit operation
    pub fn validate_commit(profile: &Profile, context: &GitContext) -> Result<ValidationResult> {
        let mut result = ValidationResult::safe();

        // Check if there are staged files to commit
        if !context.has_staged_files() {
            result = result.with_violation(
                "No staged files to commit. Use 'git add' or 'flux commit --all'.".to_string(),
            );
        }

        // Warn about untracked files
        if !context.untracked_files.is_empty() {
            result = result.with_warning(format!(
                "{} untracked file(s) not included in commit",
                context.untracked_files.len()
            ));
        }

        // Warn about unstaged changes
        if !context.unstaged_files.is_empty() {
            result = result.with_warning(format!(
                "{} unstaged change(s) not included in commit",
                context.unstaged_files.len()
            ));
        }

        // Warn if committing directly to protected branch
        if context.is_protected_branch {
            result = result.with_warning(format!(
                "Committing directly to protected branch '{}'",
                context.current_branch
            ));
        }

        Ok(result)
    }

    /// Validate force push operation
    pub fn validate_force_push(
        profile: &Profile,
        context: &GitContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::safe();

        // Block force push to protected branches
        if context.is_protected_branch {
            result = result.with_violation(format!(
                "Force push to protected branch '{}' is not allowed",
                context.current_branch
            ));
        }

        Ok(result)
    }

    /// Validate checkout operation to protected branch
    pub fn validate_checkout_protected(
        branch_name: &str,
        profile: &Profile,
        context: &GitContext,
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult::safe();

        let is_protected = profile
            .protected_branches
            .iter()
            .any(|b| b == branch_name);

        if is_protected {
            result = result.with_warning(format!(
                "Checking out protected branch '{}'. Be careful with commits.",
                branch_name
            ));
        }

        // Warn about uncommitted changes
        if context.has_uncommitted_changes {
            result = result.with_warning(
                "You have uncommitted changes that may be lost".to_string(),
            );
        }

        Ok(result)
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
            protected_branches: vec!["main".to_string(), "develop".to_string()],
            default_branch: "main".to_string(),
            rules: vec![],
            conventional_commits: true,
            auto_push: false,
        }
    }

    #[test]
    fn test_validate_commit_no_staged_files() {
        let profile = create_test_profile();
        let context = GitContext {
            repo_path: PathBuf::from("/test"),
            current_branch: "feature".to_string(),
            staged_files: vec![],
            unstaged_files: vec![],
            untracked_files: vec![],
            is_protected_branch: false,
            has_uncommitted_changes: false,
        };

        let result = Validator::validate_commit(&profile, &context).unwrap();
        assert!(!result.is_safe);
        assert_eq!(result.violations.len(), 1);
    }

    #[test]
    fn test_validate_commit_with_staged_files() {
        let profile = create_test_profile();
        let context = GitContext {
            repo_path: PathBuf::from("/test"),
            current_branch: "feature".to_string(),
            staged_files: vec!["test.rs".to_string()],
            unstaged_files: vec![],
            untracked_files: vec![],
            is_protected_branch: false,
            has_uncommitted_changes: true,
        };

        let result = Validator::validate_commit(&profile, &context).unwrap();
        assert!(result.is_safe);
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    fn test_validate_commit_protected_branch_warning() {
        let profile = create_test_profile();
        let context = GitContext {
            repo_path: PathBuf::from("/test"),
            current_branch: "main".to_string(),
            staged_files: vec!["test.rs".to_string()],
            unstaged_files: vec![],
            untracked_files: vec![],
            is_protected_branch: true,
            has_uncommitted_changes: true,
        };

        let result = Validator::validate_commit(&profile, &context).unwrap();
        assert!(result.is_safe);
        assert!(result.warnings.iter().any(|w| w.contains("protected")));
    }

    #[test]
    fn test_validate_force_push_protected() {
        let profile = create_test_profile();
        let context = GitContext {
            repo_path: PathBuf::from("/test"),
            current_branch: "main".to_string(),
            staged_files: vec![],
            unstaged_files: vec![],
            untracked_files: vec![],
            is_protected_branch: true,
            has_uncommitted_changes: false,
        };

        let result = Validator::validate_force_push(&profile, &context).unwrap();
        assert!(!result.is_safe);
        assert_eq!(result.violations.len(), 1);
    }

    #[test]
    fn test_validate_force_push_feature_branch() {
        let profile = create_test_profile();
        let context = GitContext {
            repo_path: PathBuf::from("/test"),
            current_branch: "feature/test".to_string(),
            staged_files: vec![],
            unstaged_files: vec![],
            untracked_files: vec![],
            is_protected_branch: false,
            has_uncommitted_changes: false,
        };

        let result = Validator::validate_force_push(&profile, &context).unwrap();
        assert!(result.is_safe);
        assert_eq!(result.violations.len(), 0);
    }
}
