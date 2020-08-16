use crate::settings::Settings;

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
    pub commit: Commit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
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

pub fn client(settings: Settings) -> RestClient {
    let url = format!(
        "https://api.bitbucket.org/2.0/repositories/{workspace}/{repo_slug}/",
        workspace = settings.endpoint.owner.unwrap(),
        repo_slug = settings.endpoint.name
    );
    println!("Connecting to endpoint: {}", url);
    let mut client = RestClient::new(&url[..]).unwrap();
    client.set_auth(&settings.auth.username[..], &settings.auth.password[..]);
    client
}
