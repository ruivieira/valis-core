use std::{env, path::PathBuf};

use git2::Repository;

#[derive(Debug)]
pub struct VirtualEnv {
    name: String,
    location: PathBuf,
    root: PathBuf,
    requirements: PathBuf
}

/// Returns a `VirtualEnv` struct for the specified `PathBuf` path.
/// Assumes that the virtualenvs are located in `~/.virtualenvs` and that
/// the virtualenv name is the same as the project name.
///
/// # Arguments
/// * `path` - A `PathBuf` object that holds the path to the project root.
pub fn get_venv_info(path: PathBuf) -> VirtualEnv {
    let repo = Repository::discover(path);
    let mut root = PathBuf::from(repo.ok().unwrap().path());
    root.pop();
    let name = root.file_name().unwrap();
    let mut requirements = root.clone();
    requirements.push("requirements.txt");
    let mut virtualvenv = home::home_dir().unwrap();
    virtualvenv.push(".virtualenvs");
    virtualvenv.push(name);
    let virtualenv = VirtualEnv{
        name:  name.to_str().unwrap().to_string(),
        location: virtualvenv,
        root,
        requirements,
    };
    return virtualenv;

}

pub fn rebuild(venv: VirtualEnv) {
    // delete the original environment
    
}

/// Prints the status of the virtualenv for the current project at `path`.
/// Assumes that the virtualenvs are located in `~/.virtualenvs` and that
/// the virtualenv name is the same as the project name.
///
/// # Arguments
/// * `path` - A `PathBuf` object that holds the path to the project root.
/// # Example
/// ```
/// use std::env;
/// use valis_core::modules::projects::venv::status;
/// let cwd = env::current_dir().ok().unwrap();
/// status(cwd);
/// ```
pub fn status(path: PathBuf) {
    let virtualenv = get_venv_info(path);

    let requirements_exist = virtualenv.requirements.exists();
    let requirements_icon = if requirements_exist { "👍" } else {"👎"};


    let virtualenv_exists = virtualenv.location.exists();
    let virtualenv_icon = if virtualenv_exists { "👍" } else {"👎"};

    println!("🌳 Project root:\t {:}", virtualenv.root.display());
    println!("{} requirements.txt:\t {:}", requirements_icon, virtualenv.requirements.display());
    println!("{} virtualenv:\t\t {:}", virtualenv_icon, virtualenv.location.display());
}