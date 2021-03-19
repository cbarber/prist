use crate::settings::Settings;

use log::debug;
use restson::{Error, RestClient, RestPath};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub display_name: String,
    pub nickname: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub pagelen: u32,
    pub values: Vec<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: u32,
    pub title: String,
    pub comment_count: u32,
    pub state: String,
    pub created_on: String,
    pub updated_on: String,
    pub author: User,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PullRequestActivity {
    Comment { comment: Comment },
    Approval { approval: Approval },
    Update { update: Update },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    pub user: User,
    pub created_on: String,
    pub content: CommentContent,
    pub inline: Option<Inline>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Inline {
    from: Option<u32>,
    to: Option<u32>,
    path: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CommentContent {
    pub raw: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Approval {
    pub user: User,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Update {
    pub author: User,
    pub date: String,
    pub source: Source,
    pub destination: Source,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    pub commit: PullRequestCommit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequestCommit {
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
    pub parents: Vec<CommitParent>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CommitParent {
    #[serde(rename = "commit")]
    Commit { hash: String },
}

impl RestPath<()> for Paginated<PullRequest> {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("pullrequests"))
    }
}

impl RestPath<u32> for Paginated<PullRequestActivity> {
    fn get_path(pull_request_id: u32) -> Result<String, Error> {
        Ok(format!("pullrequests/{}/activity", pull_request_id))
    }
}

impl RestPath<u32> for Paginated<PullRequestCommit> {
    fn get_path(pull_request_id: u32) -> Result<String, Error> {
        Ok(format!("pullrequests/{}/commits", pull_request_id))
    }
}

impl RestPath<String> for Commit {
    fn get_path(sha: String) -> Result<String, Error> {
        Ok(format!("commit/{}", sha))
    }
}

impl RestPath<(String, String)> for Commit {
    fn get_path(revspec: (String, String)) -> Result<String, Error> {
        Ok(format!("merge-base/{}..{}", revspec.0, revspec.1))
    }
}

impl RestPath<String> for Paginated<Comment> {
    fn get_path(sha: String) -> Result<String, Error> {
        Ok(format!("commit/{}/comments", sha))
    }
}

pub fn client(settings: Settings) -> RestClient {
    let url = format!(
        "https://api.bitbucket.org/2.0/repositories/{workspace}/{repo_slug}/",
        workspace = settings.endpoint.owner.unwrap(),
        repo_slug = settings.endpoint.name
    );
    debug!("Connecting to endpoint: {}", url);
    let mut client = RestClient::new(&url[..]).unwrap();
    client.set_auth(&settings.auth.username[..], &settings.auth.password[..]);
    client
}

// PR
//  - current commits
//  - activity
//    - revspc
//      - old commits
