extern crate log;
extern crate pretty_env_logger;
extern crate serde;
#[macro_use]
extern crate prettytable;

mod bitbucket;
mod github;
mod settings;

use crate::bitbucket::{
    Comment, Commit, CommitParent, Paginated, PullRequest, PullRequestActivity, PullRequestCommit,
};
use crate::settings::{Auth, Endpoint, Settings};

use anyhow::{Context, Result};
use clap::Clap;
use git2::Repository;
use git_url_parse::GitUrl;
use log::debug;
use prettytable::Table;
use std::io::prelude::*;
use std::io::{stdin, stdout};
use textwrap::{termwidth, Wrapper};

#[derive(Clap)]
#[clap(
    version = "0.1.0",
    author = "Craig Barber <craigb@mojotech.com>",
    about = "Pull Request CLI"
)]
struct Opts {
    #[clap(about = "Sets path for the git repository", index = 1)]
    path: Option<String>,

    #[clap(subcommand)]
    command: OptCommand,
}

#[derive(Clap)]
enum OptCommand {
    #[clap(about = "Initializes configuration for a path")]
    Init,
    #[clap(about = "Operate on pull requests")]
    PR { id: Option<u32> },
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let opts = Opts::parse();

    let current_dir = std::env::current_dir()?;
    let current_dir_str = current_dir.to_str().unwrap();

    let repos_path = opts.path.unwrap_or_else(|| current_dir_str.to_string());

    let url = get_origin_url(&repos_path)
        .with_context(|| format!("Failed to find remotes from path: {}", repos_path))?;

    let url = GitUrl::parse(url.as_str())
        .with_context(|| format!("failed to parse git origin remote: {}", url))?;

    let endpoint = Endpoint::new(url).unwrap();
    debug!("{:?}", endpoint);

    let settings = match opts.command {
        OptCommand::Init => {
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
            settings.save(&repos_path)?;
            settings
        }
        _ => settings::Settings::load(&repos_path).with_context(|| {
            format!(
                "Failed to open settings in: {}. Did you forget to init?",
                repos_path
            )
        })?,
    };
    debug!("{:?}", settings);

    let mut client = bitbucket::client(settings);
    match opts.command {
        OptCommand::PR { id: Some(id) } => show_pr(&mut client, id),
        OptCommand::PR { .. } => list_pr(&mut client),
        _ => {}
    };

    Ok(())
}

fn traverse_commits(
    client: &mut restson::RestClient,
    source_hash: String,
    destination_hash: String,
) -> Vec<Commit> {
    let mut hash = source_hash.clone();
    let mut results = vec![];

    let merge_base: Commit = client.get((source_hash, destination_hash)).unwrap();

    while !merge_base.hash.starts_with(&hash[..]) {
        let commit: Commit = client.get(hash).unwrap();
        hash = match commit.parents.first() {
            Some(CommitParent::Commit { hash }) => hash.clone(),
            _ => merge_base.hash.clone(),
        };
        results.push(commit);
    }
    results
}

static TABLE_BORDER: usize = 8;

fn add_commit_comments(
    client: &mut restson::RestClient,
    table: &mut Table,
    commit_hashes: Vec<String>,
) {
    let wrapper = Wrapper::new(termwidth() - TABLE_BORDER);
    for commit in commit_hashes {
        let comments: Paginated<Comment> = client.get(commit.clone()).unwrap();
        for comment in comments.values {
            table.add_row(row![comment.user.display_name, comment.created_on, commit,]);
            table.add_row(row![H3 -> wrapper.fill(&comment.content.raw[..])]);
        }
    }
}

fn show_pr(client: &mut restson::RestClient, id: u32) {
    let mut table = Table::new();

    let mut hashes = vec![];

    let pullrequest_commits: Paginated<PullRequestCommit> = client.get(id).unwrap();
    let pullrequest_commits: Vec<String> = pullrequest_commits
        .values
        .iter()
        .map(|commit| commit.hash.clone())
        .collect();
    hashes.extend(pullrequest_commits);

    let query = vec![("pagelen", "50")];
    let pullrequest_activities: Paginated<PullRequestActivity> =
        client.get_with(id, &query).unwrap();

    for activity in pullrequest_activities.values.iter() {
        hashes.extend(match activity {
            PullRequestActivity::Update { update } => traverse_commits(
                client,
                update.source.commit.hash.clone(),
                update.destination.commit.hash.clone(),
            )
            .iter()
            .map(|c| c.hash.clone())
            .collect(),
            _ => vec![],
        });
    }

    hashes.sort();
    hashes.dedup_by(|a, b| a.starts_with(&b[..]) || b.starts_with(&a[..]));

    let wrapper = Wrapper::new(termwidth() - TABLE_BORDER);
    add_commit_comments(client, &mut table, hashes);

    let pr_comments: Vec<&Comment> = pullrequest_activities
        .values
        .iter()
        .filter_map(|activity| match activity {
            PullRequestActivity::Comment { comment } => Some(comment),
            _ => None,
        })
        .collect();
    for comment in pr_comments {
        table.add_row(row![comment.user.display_name, comment.created_on,]);
        table.add_row(row![H3 -> wrapper.fill(&comment.content.raw[..])]);
    }

    table.printstd();
}

fn list_pr(client: &mut restson::RestClient) {
    let pullrequests: Paginated<PullRequest> = client.get(()).unwrap();
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

fn get_origin_url(path: &str) -> Result<String, git2::Error> {
    let repo = Repository::open(path)?;
    let origin = repo.find_remote("origin")?;
    let url = origin.url().unwrap();
    Ok(url.to_owned())
}
