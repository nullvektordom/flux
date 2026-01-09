use anyhow::Result;
use std::path::PathBuf;

use crate::config::Profile;
use crate::git::GitOps;

/// Snapshot of current repository state
#[derive(Debug, Clone)]
pub struct GitContext {
    /// Path to repository
    pub repo_path: PathBuf,

    /// Active branch name
    pub current_branch: String,

    /// Files in staging area
    pub staged_files: Vec<String>,

    /// Modified files (not staged)
    pub unstaged_files: Vec<String>,

    /// New files (not tracked)
    pub untracked_files: Vec<String>,

    /// Is current branch protected
    pub is_protected_branch: bool,

    /// Has any uncommitted changes
    pub has_uncommitted_changes: bool,
}

impl GitContext {
    /// Gather git context from current repository
    pub fn gather(profile: &Profile) -> Result<Self> {
        let repo = GitOps::open_repo(None)?;
        let repo_path = repo.path().parent().unwrap_or(repo.path()).to_path_buf();

        let current_branch = GitOps::current_branch(&repo)?;
        let staged_files = GitOps::staged_files(&repo)?;
        let unstaged_files = GitOps::unstaged_files(&repo)?;
        let untracked_files = GitOps::untracked_files(&repo)?;
        let has_uncommitted_changes = GitOps::has_uncommitted_changes(&repo)?;

        let is_protected_branch =
            GitOps::is_protected_branch(&current_branch, &profile.protected_branches);

        Ok(GitContext {
            repo_path,
            current_branch,
            staged_files,
            unstaged_files,
            untracked_files,
            is_protected_branch,
            has_uncommitted_changes,
        })
    }

    /// Total count of changed files
    pub fn total_changes(&self) -> usize {
        self.staged_files.len() + self.unstaged_files.len() + self.untracked_files.len()
    }

    /// Check if there are any staged files
    pub fn has_staged_files(&self) -> bool {
        !self.staged_files.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Profile;
    use git2::Repository;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_profile() -> Profile {
        Profile {
            name: "test".to_string(),
            user_name: "Test User".to_string(),
            user_email: "test@example.com".to_string(),
            gpg_signing: false,
            gpg_key: None,
            protected_branches: vec!["main".to_string(), "master".to_string()],
            default_branch: "main".to_string(),
            rules: vec![],
            conventional_commits: true,
            auto_push: false,
        }
    }

    #[test]
    fn test_git_context_total_changes() {
        let context = GitContext {
            repo_path: PathBuf::from("/test"),
            current_branch: "main".to_string(),
            staged_files: vec!["a.rs".to_string()],
            unstaged_files: vec!["b.rs".to_string()],
            untracked_files: vec!["c.rs".to_string()],
            is_protected_branch: true,
            has_uncommitted_changes: true,
        };

        assert_eq!(context.total_changes(), 3);
        assert!(context.has_staged_files());
        assert!(context.is_protected_branch);
    }

    #[test]
    fn test_git_context_gather() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        // Configure repo
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();

        // Create and commit initial file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "initial").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = repo.signature().unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();

        // Change directory to repo for context gathering
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let profile = create_test_profile();
        let context = GitContext::gather(&profile).unwrap();

        // Should detect main/master as protected
        assert!(context.is_protected_branch);
        assert!(!context.has_uncommitted_changes);
    }
}
