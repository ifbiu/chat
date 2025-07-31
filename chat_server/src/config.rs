use std::env;
use std::fs::File;
use anyhow::{bail, Error};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize,Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self,Error> {
        let ret = match (
            File::open("chat_server/app.yaml"),
            File::open("/etc/config/app.yaml"),
            env::var("CHAT_CONFIG"),
            ) {
            (Ok(reader),_,_) => serde_yaml::from_reader(reader),
            (_,Ok(reader),_) => serde_yaml::from_reader(reader),
            (_,_,Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            _ => bail!("Config file not found"),
        };
        Ok(ret?)
    }
}

#[derive(Serialize, Deserialize,Debug)]
pub struct ServerConfig {
    pub port: u16,
}