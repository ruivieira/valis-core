use std::path::{Path, PathBuf};

use git2::{Branches, Error, Repository, RepositoryOpenFlags};

pub struct SimpleRepo {
    pub url: String,
    pub branch: Option<String>,
    pub destination: String,
}

pub trait GitOperations {
    fn clone(&self) -> Result<(), Error>;
}

impl GitOperations for SimpleRepo {
    fn clone(&self) -> Result<(), Error> {
        // Clone the repository.
        // let repo = Repository::clone(&self.url, &self.destination)?;

        // Check out the specified branch.
        // let branch = repo.find_branch(&self.branch, BranchType::Local)?;
        // repo.checkout_head(Some(branch))?;

        // Return success.
        Ok(())
    }
}

pub fn get_git_project_branches(path: PathBuf) -> Result<Vec<String>, Error> {
    let repo = Repository::open(path)?;

    let mut branch_names = Vec::new();

    let branches = repo.branches(None)?;

    for branch in branches {
        let (branch, _branch_type) = branch?;
        if let Some(name) = branch.name()? {
            branch_names.push(name.to_string());
        }
    }

    Ok(branch_names)
}


pub fn get_git_project_root_path(path: PathBuf) -> Option<PathBuf> {
    let empty_string_vec: Vec<String> = Vec::new();
    let repo = Repository::open_ext(path, RepositoryOpenFlags::empty(), empty_string_vec).ok()?;
    let workdir = repo.workdir()?;
    Some(workdir.to_path_buf())
}

/// Get the full path from a partial path, relative to the git project root.
/// # Arguments
/// * `partial_path` - A string slice that holds the partial path of the file.
/// # Returns
/// A string slice that holds the full path of the file.
pub fn from_root(partial_path: &str) -> Option<String> {
    if let Some(root_path) = get_git_project_root_path(PathBuf::from(".")) {
        let full_path = Path::new(&root_path).join(partial_path);
        if let Some(full_path_str) = full_path.to_str() {
            return Some(full_path_str.to_owned());
        }
    }
    None
}
