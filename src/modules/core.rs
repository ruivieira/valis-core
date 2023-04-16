use std::process::Command;
use std::process::Stdio;
use std::io::BufRead;
use std::io::BufReader;

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
