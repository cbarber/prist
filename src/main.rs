extern crate clap;
use clap::{App, Arg};
use git2::Repository;
use git_url_parse::GitUrl;

mod bitbucket;
mod github;

pub trait Endpoint: std::fmt::Debug {}

pub fn create_endpoint(url: GitUrl) -> Option<Box<dyn Endpoint + 'static>> {
    if let Some(host) = url.host.clone() {
        match host.as_str() {
            "github.com" => Some(Box::new(github::GithubEndpoint::new(url))),
            "bitbucket.org" => Some(Box::new(bitbucket::BitbucketEndpoint::new(url))),
            host => {
                println!("unsupported host: {}", host);
                None
            }
        }
    } else {
        println!("no host defined for url: {}", url);
        None
    }
}

fn main() {
    let matches = App::new("prist")
        .version("0.1.0")
        .author("Craig Barber <craigb@mojotech.com>")
        .about("Pull Request CLI")
        .arg(
            Arg::with_name("path")
                .index(1)
                .help("Sets path for the git repository"),
        )
        .get_matches();

    let current_dir = std::env::current_dir().unwrap();
    let current_dir_str = current_dir.to_str().unwrap();

    let path = matches.value_of("path").unwrap_or(current_dir_str);

    match get_origin_url(path) {
        Ok(url) => {
            println!("{}", url);
            let url = GitUrl::parse(url.as_str()).unwrap();

            let endpoint = create_endpoint(url);
            println!("{:?}", endpoint);
        }
        Err(error) => println!("Failed to find remotes from path: {}", error),
    }
}

fn get_origin_url(path: &str) -> Result<String, git2::Error> {
    let repo = Repository::open(path)?;
    let origin = repo.find_remote("origin")?;
    let url = origin.url().unwrap();
    Ok(url.to_owned())
}
