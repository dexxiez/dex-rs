use anyhow::Result;
use dirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fs, io};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub search_paths: Vec<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().expect("Failed to get home directory");
        let mut search_paths = vec![home.clone()];

        // If Documents exists, prefer that over home
        let docs = home.join("Documents");
        if docs.exists() {
            search_paths = vec![docs];
        }

        Config { search_paths }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;

        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    fn get_config_path() -> io::Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No config directory found"))?;
        Ok(config_dir.join("dex").join("config.toml"))
    }
}
