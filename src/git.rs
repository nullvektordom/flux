use anyhow::{Context, Result};
use git2::{Repository, StatusOptions};
use std::path::Path;

/// Git operations using git2
pub struct GitOps;

impl GitOps {
    /// Open a git repository at the given path (or current directory)
    pub fn open_repo(path: Option<&Path>) -> Result<Repository> {
        let repo = if let Some(p) = path {
            Repository::open(p)
        } else {
            Repository::open_from_env()
        }
        .context("Not a git repository. Run this command from inside a git repository.")?;

        Ok(repo)
    }

    /// Get the current branch name
    pub fn current_branch(repo: &Repository) -> Result<String> {
        let head = repo.head().context("Failed to get HEAD reference")?;

        if head.is_branch() {
            let branch_name = head
                .shorthand()
                .context("Failed to get branch name")?
                .to_string();
            Ok(branch_name)
        } else if let Some(oid) = head.target() {
            // Detached HEAD state
            Ok(format!("detached at {}", &oid.to_string()[..7]))
        } else {
            Ok("unknown".to_string())
        }
    }

    /// Get staged files (in index, ready to commit)
    pub fn staged_files(repo: &Repository) -> Result<Vec<String>> {
        let mut files = Vec::new();
        let mut opts = StatusOptions::new();
        opts.include_untracked(false);

        let statuses = repo.statuses(Some(&mut opts))?;

        for entry in statuses.iter() {
            let status = entry.status();
            // Check if file is in the index (staged)
            if status.is_index_new()
                || status.is_index_modified()
                || status.is_index_deleted()
                || status.is_index_renamed()
                || status.is_index_typechange()
            {
                if let Some(path) = entry.path() {
                    files.push(path.to_string());
                }
            }
        }

        Ok(files)
    }

    /// Get unstaged modified files (modified in working tree but not staged)
    pub fn unstaged_files(repo: &Repository) -> Result<Vec<String>> {
        let mut files = Vec::new();
        let mut opts = StatusOptions::new();
        opts.include_untracked(false);

        let statuses = repo.statuses(Some(&mut opts))?;

        for entry in statuses.iter() {
            let status = entry.status();
            // Check if file has working tree changes
            if status.is_wt_modified()
                || status.is_wt_deleted()
                || status.is_wt_renamed()
                || status.is_wt_typechange()
            {
                if let Some(path) = entry.path() {
                    files.push(path.to_string());
                }
            }
        }

        Ok(files)
    }

    /// Get untracked files (new files not in git)
    pub fn untracked_files(repo: &Repository) -> Result<Vec<String>> {
        let mut files = Vec::new();
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);

        let statuses = repo.statuses(Some(&mut opts))?;

        for entry in statuses.iter() {
            if entry.status().is_wt_new() {
                if let Some(path) = entry.path() {
                    files.push(path.to_string());
                }
            }
        }

        Ok(files)
    }

    /// Check if repository has any uncommitted changes
    pub fn has_uncommitted_changes(repo: &Repository) -> Result<bool> {
        let staged = Self::staged_files(repo)?;
        let unstaged = Self::unstaged_files(repo)?;
        let untracked = Self::untracked_files(repo)?;

        Ok(!staged.is_empty() || !unstaged.is_empty() || !untracked.is_empty())
    }

    /// Check if current branch is in the protected list
    pub fn is_protected_branch(current_branch: &str, protected_branches: &[String]) -> bool {
        protected_branches.iter().any(|b| b == current_branch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn init_test_repo() -> (TempDir, Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        // Configure repo
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();

        (temp_dir, repo)
    }

    #[test]
    fn test_current_branch_main() {
        let (_temp, repo) = init_test_repo();

        // Create initial commit to establish branch
        let sig = repo.signature().unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();

        let branch = GitOps::current_branch(&repo).unwrap();
        // git2 creates "master" by default, or "main" depending on git config
        assert!(branch == "master" || branch == "main");
    }

    #[test]
    fn test_is_protected_branch() {
        let protected = vec!["main".to_string(), "develop".to_string()];

        assert!(GitOps::is_protected_branch("main", &protected));
        assert!(GitOps::is_protected_branch("develop", &protected));
        assert!(!GitOps::is_protected_branch("feature/test", &protected));
    }

    #[test]
    fn test_untracked_files() {
        let (temp_dir, repo) = init_test_repo();

        // Create a new file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        let untracked = GitOps::untracked_files(&repo).unwrap();
        assert_eq!(untracked.len(), 1);
        assert_eq!(untracked[0], "test.txt");
    }

    #[test]
    fn test_has_uncommitted_changes() {
        let (temp_dir, repo) = init_test_repo();

        // Initially no changes
        assert!(!GitOps::has_uncommitted_changes(&repo).unwrap());

        // Add untracked file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Now has changes
        assert!(GitOps::has_uncommitted_changes(&repo).unwrap());
    }
}
