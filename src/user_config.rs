use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug)]
pub struct UserConfig {
    pub connections: Vec<Connection>,
}

#[derive(Debug, Deserialize)]
pub struct Connection {
    pub name: String,
    pub user: String,
    pub host: String,
    pub port: u64,
}

impl UserConfig {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        #[derive(Debug, Deserialize)]
        pub struct ConfigFormat {
            pub conn: HashMap<String, Connection>,
        }

        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;

        let config: Result<ConfigFormat, toml::de::Error> = toml::from_str(&contents);
        match config {
            Ok(config) => Ok(UserConfig {
                connections: config
                    .conn
                    .iter()
                    .map(|(name, conn)| Connection {
                        name: name.to_string(),
                        user: conn.user.to_string(),
                        host: conn.host.to_string(),
                        port: conn.port,
                    })
                    .collect::<Vec<Connection>>(),
            }),
            Err(e) => panic!("fail to parse config file: {}", e),
        }
    }
}
