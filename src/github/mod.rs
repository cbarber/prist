use git_url_parse::GitUrl;

use crate::Endpoint;

#[derive(Debug)]
pub struct GithubEndpoint {
    name: String,
    owner: Option<String>,
}

impl GithubEndpoint {
    pub fn new(url: GitUrl) -> Self {
        Self {
            name: url.name,
            owner: url.owner,
        }
    }
}

impl Endpoint for GithubEndpoint {}
