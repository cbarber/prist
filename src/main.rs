extern crate clap;
use clap::{App, Arg};

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
    let current_dir = current_dir.to_str().unwrap();

    let path = matches.value_of("path").unwrap_or(current_dir);

    println!("{}", path);
}
