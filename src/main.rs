extern crate clap;
extern crate log;
extern crate pretty_env_logger;
extern crate serde;
#[macro_use]
extern crate prettytable;

mod bitbucket;
mod github;
mod settings;

use crate::bitbucket::{Approval, Comment, PullRequestActivity, Update};
use crate::settings::{Auth, Endpoint, Settings};

use anyhow::{Context, Result};
use clap::{App, Arg, SubCommand};
use git2::Repository;
use git_url_parse::GitUrl;
use prettytable::Table;
use std::io::prelude::*;
use std::io::{stdin, stdout};

fn main() -> Result<()> {
    pretty_env_logger::init();

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
        .subcommand(
            SubCommand::with_name("pr")
                .help("Operate on pull requests")
                .arg(Arg::with_name("id").index(1).required(false)),
        )
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

    let mut client = bitbucket::client(settings);
    match matches.subcommand() {
        ("pr", Some(cmd)) if cmd.is_present("id") => {
            let id: u32 = cmd.value_of("id").unwrap().parse().unwrap();
            let query = vec![("pagelen", "50")];
            let pullrequest_activities: bitbucket::Paginated<bitbucket::PullRequestActivity> =
                client.get_with(id, &query).unwrap();

            let mut table = Table::new();
            table.add_row(row!["Type", "User", "Date", "Content"]);
            for pullrequest_activity in pullrequest_activities.values {
                match pullrequest_activity {
                    PullRequestActivity::Comment {
                        comment:
                            Comment {
                                user,
                                created_on,
                                content,
                            },
                    } => table.add_row(row!["Comment", user.display_name, created_on, content.raw]),

                    PullRequestActivity::Approval {
                        approval: Approval { user, date },
                    } => table.add_row(row!["Approval", user.display_name, date, ""]),
                    PullRequestActivity::Update {
                        update:
                            Update {
                                author,
                                date,
                                source,
                                destination,
                            },
                    } => table.add_row(row![
                        "Update",
                        author.display_name,
                        date,
                        format!("{}..{}", destination.commit.hash, source.commit.hash)
                    ]),
                };
            }
            table.printstd();
        }
        ("pr", Some(_)) => {
            let pullrequests: bitbucket::Paginated<bitbucket::PullRequest> =
                client.get(()).unwrap();
            let mut table = Table::new();
            table.add_row(row![
                "Id", "Title", "Author", "State", "Comments", "Created", "Updated"
            ]);
            for pullrequest in pullrequests.values {
                table.add_row(row![
                    pullrequest.id,
                    pullrequest.title,
                    pullrequest.author.display_name,
                    pullrequest.state,
                    pullrequest.comment_count,
                    pullrequest.created_on,
                    pullrequest.updated_on
                ]);
            }
            table.printstd();
        }
        _ => {
            println!("Done");
        }
    };

    Ok(())
}

fn get_origin_url(path: &str) -> Result<String, git2::Error> {
    let repo = Repository::open(path)?;
    let origin = repo.find_remote("origin")?;
    let url = origin.url().unwrap();
    Ok(url.to_owned())
}
