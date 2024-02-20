use std::{fs::File, io::Read};

use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub account: Account,
    pub selection: Selection,
    pub query: Query,
    pub net: Option<Net>,
    pub target: Option<Target>,
}

#[derive(Debug, Deserialize)]
pub struct Account {
    pub ntustsecret: String,
}

#[derive(Debug, Deserialize)]
pub struct Selection {
    pub mode: String,
    pub custom_select_page_url: Option<String>,
    pub custom_select_api_url: Option<String>,
    pub login_retry_interval: Option<u64>,
    pub session_refresh_interval: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct Query {
    pub threads: u32,
    pub semester: String,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Net {
    pub interface: Option<Interface>,
}

#[derive(Debug, Deserialize)]
pub struct Interface {
    pub query: Option<String>,
    pub select: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Target {
    pub courses: Vec<Course>,
}

#[derive(Debug, Deserialize)]
pub struct Course {
    pub course_no: String,
    pub enabled: bool,
    pub force_grab: Option<bool>,
}

pub fn load_config(path: &str) -> anyhow::Result<Config> {
    let mut file = match File::open(&path) {
        Err(_) => {
            return Err(anyhow::Error::msg("Can't open config file"));
        }
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(_) => Err(anyhow::Error::msg("Can't read config file")),
        Ok(_) => Ok(toml::from_str(&s)?),
    }
}
