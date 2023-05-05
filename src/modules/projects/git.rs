use git2::{BranchType, Error, Repository};

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
        let repo = Repository::clone(&self.url, &self.destination)?;

        // Check out the specified branch.
        // let branch = repo.find_branch(&self.branch, BranchType::Local)?;
        // repo.checkout_head(Some(branch))?;

        // Return success.
        Ok(())
    }
}