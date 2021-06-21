use serde::Deserialize;

use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug, Deserialize)]
pub struct UserConfig {
    pub conn: Vec<Connection>,
}

#[derive(Debug, Deserialize)]
pub struct Connection {
    pub name: Option<String>,
    pub user: String,
    pub host: String,
    pub port: u64,
}

impl UserConfig {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;

        let config: Result<UserConfig, toml::de::Error> = toml::from_str(&contents);
        match config {
            Ok(config) => Ok(config),
            Err(e) => panic!("fail to parse config file: {}", e),
        }
    }
}

impl Connection {
    pub fn database_url(&self) -> String {
        format!(
            "mysql://{user}:@{host}:{port}",
            user = self.user,
            host = self.host,
            port = self.port
        )
    }
}
