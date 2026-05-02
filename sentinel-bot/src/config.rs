use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

const DEFAULT_CONFIG: &str = include_str!("./resources/config.toml");

#[derive(Deserialize, Debug)]
pub struct Configuration {
  pub tokens: Tokens,
  pub guilds: HashMap<String, Channels>,
}

#[derive(Deserialize, Debug)]
pub struct Tokens {
  pub sentinel: String,
  pub factoids: String,
}

#[derive(Deserialize, Debug)]
pub struct Channels {
  #[serde(rename = "channel-report")]
  pub channel_report: Option<String>,

  #[serde(rename = "channel-mod-msg")]
  pub channel_mod_msg: Option<String>,

  #[serde(rename = "channel-appeals")]
  pub channel_appeals: Option<String>,
}

pub fn load_config(parent_folder: &str) -> Result<Configuration, String> {
  fs::create_dir_all(parent_folder).map_err(|_| format!("Failed to create directory: {}", parent_folder))?;

  let path = format!("{}/config.toml", parent_folder);
  let exists = fs::exists(path.clone());

  let source: String = if exists.is_err() || exists.unwrap() == false {
    fs::write(path.clone(), DEFAULT_CONFIG).map_err(|_| format!("Failed to create and write file: {}", path))?;
    String::from(DEFAULT_CONFIG)
  } else {
    String::from_utf8(fs::read(path.clone()).map_err(|_| format!("Failed to read from file: {}", path))?).map_err(|e| format!("Failed to get String from utf8 vec: {}", e))?
  };

  let config: Configuration = toml::from_str(source.as_str()).map_err(|e| format!("Failed to parse TOML: {}", e))?;
  Ok(config)
}
