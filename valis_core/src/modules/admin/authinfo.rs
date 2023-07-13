use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct AuthInfo {
    pub machine: String,
    pub login: Login,
    pub password: String,
    pub port: Option<u16>,
}

#[derive(Clone, Debug)]
pub struct Login {
    pub name: String,
    pub domain: Option<String>,
}

pub fn parse_auth_info(line: &str) -> Result<AuthInfo, &str> {
    let mut parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 6 {
        return Err("Invalid auth info format");
    }

    let machine = parts[1].to_string();
    let password = parts[5].to_string();
    let mut port = None;

    if parts.len() > 6 {
        if parts[6] == "port" {
            port = parts.get(7).and_then(|s| s.parse::<u16>().ok());
            parts.truncate(6); // to make sure we ignore anything beyond the port info
        }
    }

    let login_parts: Vec<&str> = parts[3].split('^').collect();
    let login = if login_parts.len() == 2 {
        Login {
            name: login_parts[0].to_string(),
            domain: Some(login_parts[1].to_string()),
        }
    } else if login_parts.len() == 1 {
        Login {
            name: login_parts[0].to_string(),
            domain: None,
        }
    } else {
        return Err("Invalid login format");
    };

    Ok(AuthInfo {
        machine,
        login,
        password,
        port,
    })
}

pub fn read_auth_file(file_path: &Path) -> Result<Vec<AuthInfo>, Box<dyn std::error::Error>> {
    let file = File::open(&file_path)?;
    let reader = io::BufReader::new(file);
    let mut auth_infos = Vec::new();

    for line in reader.lines() {
        let line = line?.trim().to_string(); // trim white spaces
        if line.is_empty() {
            continue; // Skip empty lines
        }

        match parse_auth_info(&line) {
            Ok(auth_info) => auth_infos.push(auth_info),
            Err(e) => eprintln!("Skipping line due to error: {}", e),
        }
    }
    Ok(auth_infos)
}


pub fn find_auth_info_for_machine(machine: &str, auth_infos: Vec<AuthInfo>) -> Vec<AuthInfo> {
    auth_infos.into_iter().filter(|info| info.machine == machine).collect()
}
