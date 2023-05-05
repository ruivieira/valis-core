use git2::{
    Branch,
    CloneOptions,
    Commit,
    Error,
    Repository,
    RepositoryCreateOptions,
};

pub struct SimpleRepo {
    pub url: String,
    pub branch: String,
    pub destination: String,
}

pub trait GitOperations {
    fn clone(&self) -> Result<(), Error>;
}

impl GitOperations for SimpleRepo {
    fn clone(&self) -> Result<(), Error> {
        // Clone the repository.
        let repo = Repository::clone_options(self.url, self.destination)?;

        // Check out the specified branch.
        let branch = repo.find_branch(self.branch)?;
        repo.checkout_head(&branch)?;

        // Return success.
        Ok(())
    }
}