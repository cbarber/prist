extern crate clap;
extern crate serde;

mod bitbucket;
mod github;
mod settings;

use anyhow::{Context, Result};
use clap::{App, Arg, SubCommand};
use git2::Repository;
use git_url_parse::GitUrl;
use settings::{Auth, Endpoint, Settings};
use std::io::prelude::*;
use std::io::{stdin, stdout};

fn main() -> Result<()> {
    let matches = App::new("prist")
        .version("0.1.0")
        .author("Craig Barber <craigb@mojotech.com>")
        .about("Pull Request CLI")
        .arg(
            Arg::with_name("path")
                .index(1)
                .help("Sets path for the git repository"),
        )
        .subcommand(SubCommand::with_name("init").help("Initializes configuration for a path"))
        .get_matches();

    let current_dir = std::env::current_dir()?;
    let current_dir_str = current_dir.to_str().unwrap();

    let repos_path = matches.value_of("path").unwrap_or(current_dir_str);

    let url = get_origin_url(repos_path)
        .with_context(|| format!("Failed to find remotes from path: {}", repos_path))?;

    let url = GitUrl::parse(url.as_str())
        .with_context(|| format!("failed to parse git origin remote: {}", url))?;

    let endpoint = Endpoint::new(url).unwrap();
    println!("{:?}", endpoint);

    let settings = match matches.subcommand() {
        ("init", Some(_)) => {
            let mut username = String::new();
            print!("username: ");
            stdout().flush()?;
            stdin().read_line(&mut username)?;
            username = (&username[..username.len() - 1]).to_string();

            let mut password = String::new();
            print!("password: ");
            stdout().flush()?;
            stdin().read_line(&mut password)?;
            password = (&password[..password.len() - 1]).to_string();

            let settings = Settings::new(Auth::new(username, password), endpoint);
            settings.save(repos_path)?;
            settings
        }
        _ => settings::Settings::load(repos_path).with_context(|| {
            format!(
                "Failed to open settings in: {}. Did you forget to init?",
                repos_path
            )
        })?,
    };

    println!("{:?}", settings);

    Ok(())
}

fn get_origin_url(path: &str) -> Result<String, git2::Error> {
    let repo = Repository::open(path)?;
    let origin = repo.find_remote("origin")?;
    let url = origin.url().unwrap();
    Ok(url.to_owned())
}
