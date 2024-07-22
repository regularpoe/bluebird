#[macro_use]
extern crate clap;
use clap::App;

use colored::*;

extern crate dotenv;
use dotenv::dotenv;

extern crate regex;
use regex::Regex;

use reqwest::header::USER_AGENT;

use serde::{Deserialize, Serialize};

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Vars {
    variable_type: Option<String>,
    key: Option<String>,
    value: Option<String>,
    protected: Option<bool>,
    masked: Option<bool>,
    environment_scope: Option<String>,
}

#[tokio::main]
async fn get_project_vars(url: &str, id: &str, pat: &str) -> Result<String, reqwest::Error> {
    let url = format!(
        "{}/api/v4/projects/{}/variables?private_token={}",
        url, id, pat
    );

    let rsp = reqwest::Client::new()
        .get(url)
        .header(USER_AGENT, "blue.bird")
        .send()
        .await?;

    let payload = rsp.text().await?;

    Ok(payload)
}

fn read_file<P>(filename: P) -> Result<String, ::std::io::Error>
where
    P: AsRef<Path>,
{
    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(content)
}

fn main() {
    dotenv().ok();

    let yaml = load_yaml!("cli.yaml");
    let opts = App::from_yaml(yaml).get_matches();

    println!("\nblue.bird {}", env!("CARGO_PKG_VERSION"));

    let url = match env::var("url") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("You need to specify an URL in .env file, url=<your gitlab instance>");
            std::process::exit(1);
        }
    };

    let project_id = match env::var("project_id") {
        Ok(id) => id,
        Err(_) => {
            eprintln!("You need to specify project id in .env file, project_id=<your_project_id>");
            std::process::exit(1);
        }
    };

    let pat = match env::var("token") {
        Ok(pat) => pat,
        Err(_) => {
            eprintln!(
                "You need to specify personal token in .env file, token=<your_personal_token>"
            );
            std::process::exit(1);
        }
    };

    let input_file = opts.value_of("filename").unwrap();

    let regex = Regex::new(r"(?m)\$[a-zA-Z0-9_]*").unwrap();

    let file = match read_file(input_file) {
        Ok(data) => data,
        Err(why) => panic!("Cannot read file, error: {}", why),
    };

    let raw = match get_project_vars(&url, &project_id, &pat) {
        Ok(v) => v,
        Err(why) => panic!(
            "Got an error: {}, while fetching variables from GitLab",
            why
        ),
    };

    let parsed: Vec<Vars> = match serde_json::from_str(&raw) {
        Ok(data) => data,
        Err(why) => panic!(
            "Got an error: {}, while parsing vars from GitLab response",
            why
        ),
    };

    let mut remote_vars = parsed
        .into_iter()
        .map(|e| String::from(e.key.as_ref().unwrap()))
        .collect::<Vec<String>>();

    let mut file_vars = regex
        .captures_iter(&file)
        .into_iter()
        .map(|e| String::from(e.get(0).unwrap().as_str()).replace("$", ""))
        .filter(|e| e.len() > 0)
        .collect::<Vec<String>>();

    remote_vars.sort();
    file_vars.sort();
    file_vars.dedup();

    println!(
        "{}",
        "These variables were found in your GitLab project:".green()
    );
    for i in &remote_vars {
        println!("Remote vars: {i}");
    }

    println!("{}", "These variables were found in your CI file:".green());
    for i in &file_vars {
        println!("File vars: {i}");
    }

    let remote_set: HashSet<String> = HashSet::from_iter(remote_vars);
    let file_set: HashSet<String> = HashSet::from_iter(file_vars);

    println!("{}", "These variables were found in your CI file, but they are NOT defined in your GitLab project!".yellow());
    for x in file_set.difference(&remote_set) {
        println!("{x}");
    }
}
