use std::io::BufRead;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::{env, fs};

use globmatch::Matcher;

use dirs;

/// Run a command string and outputs realtime output to stdout.
///
/// # Arguments
///
/// * `command` - A string slice that holds the command to be executed
///
pub fn run(command: &str) {
    let mut tokens = command.split(" ").collect::<Vec<&str>>();
    let location = tokens[0];
    tokens.remove(0);

    let mut child = Command::new(location)
        .args(tokens)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();

    // Stream output.
    let lines = BufReader::new(stdout).lines();
    for line in lines {
        println!("{}", line.unwrap());
    }
}

pub fn run_buffered(command: &str) -> String {
    let mut tokens = command.split(" ").collect::<Vec<&str>>();
    let location = tokens[0];
    tokens.remove(0);
    let output = Command::new(location)
        .args(tokens)
        .output()
        .expect("Failed to execute process.");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    print!("{}", stdout.to_owned());

    return stdout.to_owned();
}

/// Return the OS name on which we are running.
pub fn get_os() -> String {
    return env::consts::OS.to_owned();
}

/// Return a list of `Matcher` objects that match the given pattern.
pub fn get_files<'a>(root: PathBuf, pattern: &'a &str) -> Result<Matcher<'a, PathBuf>, String> {
    return globmatch::Builder::new(&pattern).build(root.to_str().unwrap().to_owned());
}

/// Check if a program is in the PATH.
pub fn in_path(program: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(":") {
            let p_str = format!("{}/{}", p, program);
            if fs::metadata(&p_str).is_ok() {
                return true;
            }
        }
    }

    return false;
}

/// Check if a directory exists.
fn directory_exists(path: &Path) -> bool {
    // Check if the directory exists.
    Path::new(path).exists()
}

/// Set the current working directory.
/// # Arguments
/// * `dir` - A string slice that holds the directory to be set as the current working directory.
pub fn set_dir(dir: &str) -> Result<(), std::io::Error> {
    let path = Path::new(dir);
    env::set_current_dir(&path)
}

/// Get the current working directory.
/// # Returns
/// A string slice that holds the current working directory.
pub fn get_dir() -> Result<String, std::io::Error> {
    let current_dir = env::current_dir()?;
    let current_dir_str = current_dir.to_str().unwrap_or("");
    Ok(current_dir_str.to_owned())
}

fn get_home_dir() -> Option<PathBuf> {
    if let Some(home_dir) = dirs::home_dir() {
        Some(home_dir)
    } else {
        None
    }
}

/// Return the full path of a file in the user's home directory.
/// # Arguments
/// * `partial_path` - A string slice that holds the partial path of the file.
pub fn from_home(partial_path: &str) -> Option<String> {
    if let Some(home_dir) = get_home_dir() {
        let full_path = Path::new(&home_dir).join(partial_path);
        if let Some(full_path_str) = full_path.to_str() {
            return Some(full_path_str.to_owned());
        }
    }
    None
}
