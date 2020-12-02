use crate::hardware::{DevBoard, Target};
use serde_derive::Deserialize;
use std::path::{Path, PathBuf};

pub const CONFIG_FILE_NAME: &str = ".hdlman.toml";

#[derive(Deserialize)]
pub struct Config {
    #[serde(alias = "default-target")]
    pub default_target: Option<Target>,
    #[serde(alias = "default-dev-board")]
    pub default_dev_board: Option<DevBoard>,
}

pub fn config_file_path() -> Option<PathBuf> {
    directories::BaseDirs::new()
        .map(|base_dir| base_dir.home_dir().join(Path::new(CONFIG_FILE_NAME)))
}

pub fn get_persisted_config() -> Option<Config> {
    config_file_path().and_then(|path| {
        let config_as_str = std::fs::read_to_string(path);
        match config_as_str {
            Ok(config_contents) => match toml::from_str::<Config>(&config_contents) {
                Ok(config) => Some(config),
                Err(e) => {
                    colour::red_ln!("failed trying to parse hdlman config file, but pressing onwards.  error is: {}", e);
                    None
                }
            }
            Err(_) => {
                // The user may just not have a hdlman config file, so don't bother logging
                // anything here.  In the future, it might be useful to have a "debug log" flag or
                // something for things like this.
                None
            }
        }
    })
}
