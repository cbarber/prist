extern crate clap;
use clap::{App, Arg};
use git2::Repository;

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
