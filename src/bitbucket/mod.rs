use git_url_parse::GitUrl;

use crate::Endpoint;

#[derive(Debug)]
pub struct BitbucketEndpoint {
    name: String,
    owner: Option<String>,
}

impl BitbucketEndpoint {
    pub fn new(url: GitUrl) -> Self {
        Self {
            name: url.name,
            owner: url.owner,
        }
    }
}

impl Endpoint for BitbucketEndpoint {}
