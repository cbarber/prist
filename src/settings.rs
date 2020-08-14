use config::{Config, ConfigError, Environment, File};
use git_url_parse::GitUrl;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use toml::to_string_pretty;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub auth: Auth,
    pub endpoint: Endpoint,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoint {
    pub kind: EndpointKind,
    pub name: String,
    pub owner: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EndpointKind {
    Github,
    Bitbucket,
}

impl Settings {
    pub fn new(auth: Auth, endpoint: Endpoint) -> Self {
        Self { auth, endpoint }
    }

    pub fn load(path: &str) -> Result<Self, ConfigError> {
        let mut s = Config::new();

        let path = Path::new(path).join(".prist/config.toml");

        if !path.is_file() {
            return Err(ConfigError::Message("No config file found".to_string()));
        }

        s.merge(File::with_name(path.to_str().unwrap()))?;

        s.merge(Environment::with_prefix("PRIST"))?;

        s.try_into()
    }

    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let path = Path::new(path).join(".prist/config.toml");

        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix)?;

        let mut file = OpenOptions::new().create(true).write(true).open(path)?;
        file.set_len(0)?;
        let data = self.save_as_string();
        file.write_all(data.as_bytes())?;

        Ok(())
    }

    pub fn save_as_string(&self) -> String {
        to_string_pretty(self).unwrap()
    }
}

impl Auth {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

impl Endpoint {
    pub fn new(git_url: GitUrl) -> Option<Self> {
        let kind = match git_url.host.clone() {
            Some(host) if host == "github.com" => Some(EndpointKind::Github),
            Some(host) if host == "bitbucket.org" => Some(EndpointKind::Bitbucket),
            Some(host) => {
                println!("unsupported host: {}", host);
                None
            }
            None => {
                println!("no host defined for url");
                None
            }
        };

        kind.map(|kind| Self {
            kind,
            name: git_url.name,
            owner: git_url.owner,
        })
    }
}
