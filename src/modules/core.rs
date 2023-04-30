use globmatch::Matcher;
use std::{env, fs};
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

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
