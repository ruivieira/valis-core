use std::process::Command;
use std::process::Stdio;
use std::io::BufRead;
use std::io::BufReader;
use std::env;


pub fn run(command: &str) {
    /// Run a command string and outputs realtime output to stdout.
    ///
    /// # Arguments
    ///
    /// * `command` - A string slice that holds the command to be executed
    ///
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

pub fn get_os() -> String {
    /// Return the OS name on which we are running.
    return env::consts::OS.to_owned();
}
