use std::collections::HashMap;
use std::env;

use reqwest::header;
use serde::Deserialize;

#[derive(Deserialize)]
struct Milestone {
    number: i32,
    title: String,
}

#[derive(Deserialize)]
struct Issue {
    title: String,
}

pub fn github_get_milestones(user: String, token: String, org: String, repo: String) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.github.com/repos/{}/{}/milestones", org, repo);

    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("reqwest"));
    headers.insert("Authorization", header::HeaderValue::from_str(&format!("token {}", token))?);

    let resp = client.get(&url).headers(headers).send()?.json::<Vec<Milestone>>()?;

    let mut map = HashMap::new();
    for milestone in resp {
        map.insert(milestone.number.to_string(), milestone.title);
    }
    Ok(map)
}


pub fn github_get_milestone_issues(user: String, token: String, org: String, repo: String, milestone_number: i32) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://api.github.com/repos/{}/{}/issues?milestone={}", org, repo, milestone_number);

    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static("reqwest"));
    headers.insert("Authorization", header::HeaderValue::from_str(&format!("token {}", token))?);

    let resp = client.get(&url).headers(headers).send()?.json::<Vec<Issue>>()?;

    let mut vec = Vec::new();
    for issue in resp {
        vec.push(issue.title);
    }
    Ok(vec)
}
