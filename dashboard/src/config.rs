use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, env};

#[derive(Deserialize, Default, Debug)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_password")]
    pub password: String,
    pub hosts: HashMap<String, Host>,
}

fn deserialize_password<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if std::path::Path::new(&s).exists() {
        std::fs::read_to_string(&s)
            .map(|v| v.trim().to_string())
            .map_err(serde::de::Error::custom)
    } else {
        Ok(s)
    }
}

impl Config {
    pub fn load() -> Option<Self> {
        let path = Self::config_path()?;
        let content = std::fs::read_to_string(path).ok()?;

        serde_yaml::from_str(&content).ok()
    }

    fn config_path() -> Option<std::path::PathBuf> {
        if let Some(arg_path) = env::args().nth(1) {
            return Some(std::path::PathBuf::from(arg_path));
        }
        if let Ok(env_path) = env::var("MOXAPI_CONFIG") {
            return Some(std::path::PathBuf::from(env_path));
        }
        if let Some(home) = dirs::config_dir() {
            let fallback = home.join("mox/moxapi/config.yaml");
            if fallback.exists() {
                return Some(fallback);
            }
        }
        let etc_path = std::path::PathBuf::from("/etc/moxapi/config.yaml");
        if etc_path.exists() {
            return Some(etc_path);
        }

        None
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Host {
    pub ip: String,
    #[serde(deserialize_with = "deserialize_api_key")]
    pub api_key: String,
}

fn deserialize_api_key<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if std::path::Path::new(&s).exists() {
        std::fs::read_to_string(&s)
            .map(|v| v.trim().to_string())
            .map_err(serde::de::Error::custom)
    } else {
        Ok(s)
    }
}
