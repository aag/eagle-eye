extern crate rustc_serialize;
extern crate toml;

use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, RustcDecodable)]
pub struct Config {
    pub settings: Option<SettingsConfig>,
    pub watchers: Option<Vec<WatcherSettings>>,
}

#[derive(Debug, RustcDecodable)]
pub struct SettingsConfig {
    pub quiet: Option<bool>,
}

#[derive(Debug, RustcDecodable)]
pub struct WatcherSettings {
    pub action_type: String,
    pub execute: String,
    pub path: String,
}

pub fn parse(config_content: String) -> Option<Config> {
    toml::decode_str(&config_content)
}

pub fn parse_file(path: &str) -> Option<Config> {
    let file_path = Path::new(path);
    if !file_path.exists() {
        println!("ERROR: Config file not found: {}", path);
        return None;
    }

    let mut config_content = String::new();
    File::open(&path)
        .and_then(|mut f| f.read_to_string(&mut config_content))
        .unwrap();

    parse(config_content)
}
