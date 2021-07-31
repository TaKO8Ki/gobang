use crate::Key;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub conn: Vec<Connection>,
    #[serde(default)]
    pub key_config: KeyConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            conn: vec![Connection {
                name: None,
                user: "root".to_string(),
                host: "localhost".to_string(),
                port: 3306,
                database: None,
            }],
            key_config: KeyConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Connection {
    name: Option<String>,
    user: String,
    host: String,
    port: u64,
    pub database: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeyConfig {
    pub move_up: Key,
    pub move_down: Key,
    pub move_right: Key,
    pub move_left: Key,
    pub copy: Key,
    pub enter: Key,
    pub exit: Key,
    pub quit: Key,
    pub exit_popup: Key,
    pub focus_right: Key,
    pub focus_left: Key,
    pub open_help: Key,
    pub filter: Key,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            move_up: Key::Char('k'),
            move_down: Key::Char('j'),
            move_right: Key::Char('l'),
            move_left: Key::Char('h'),
            copy: Key::Char('y'),
            enter: Key::Enter,
            exit: Key::Ctrl('c'),
            quit: Key::Char('q'),
            exit_popup: Key::Esc,
            focus_right: Key::Right,
            focus_left: Key::Left,
            open_help: Key::Char('?'),
            filter: Key::Char('/'),
        }
    }
}

impl Config {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        if let Ok(file) = File::open(path) {
            let mut buf_reader = BufReader::new(file);
            let mut contents = String::new();
            buf_reader.read_to_string(&mut contents)?;

            let config: Result<Config, toml::de::Error> = toml::from_str(&contents);
            match config {
                Ok(config) => return Ok(config),
                Err(e) => panic!("fail to parse config file: {}", e),
            }
        }
        Ok(Config::default())
    }
}

impl Connection {
    pub fn database_url(&self) -> String {
        match &self.database {
            Some(database) => format!(
                "mysql://{user}:@{host}:{port}/{database}",
                user = self.user,
                host = self.host,
                port = self.port,
                database = database
            ),
            None => format!(
                "mysql://{user}:@{host}:{port}",
                user = self.user,
                host = self.host,
                port = self.port,
            ),
        }
    }
}
