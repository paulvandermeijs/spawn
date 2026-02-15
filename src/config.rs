use anyhow::{Error, Result};
use directories::ProjectDirs;
use log::info;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

const IGNORE_GLOBAL_FILENAME: &str = ".spwnignore_global";

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    aliases: HashMap<String, String>,
}

impl Config {
    pub fn read() -> Result<Config> {
        let config_path = config_path()?;
        let Ok(config_data) = std::fs::read_to_string(&config_path) else {
            return Ok(Config::default());
        };

        info!("Using config file {}", config_path.display());

        let config: Config = toml::from_str(&config_data)?;

        Ok(config)
    }

    pub fn write(&self) -> Result<()> {
        let config_data = toml::to_string(self)?;
        let config_path = config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        info!("Writing config to {}", config_path.display());

        std::fs::write(config_path, config_data)?;

        Ok(())
    }

    pub fn resolve_alias(&self, uri: String) -> String {
        if let Some(uri) = self.aliases.get(&uri) {
            uri.clone()
        } else {
            uri
        }
    }

    pub fn get_aliases(&self) -> &HashMap<String, String> {
        &self.aliases
    }

    pub fn add_alias(&mut self, name: String, uri: String) -> &Self {
        self.aliases.insert(name, uri);

        self
    }

    pub fn remove_alias(&mut self, name: &str) -> &Self {
        self.aliases.remove(name);

        self
    }
}

pub(crate) fn init() -> Result<()> {
    let Some(config_dir) = config_dir() else {
        return Err(Error::msg("No config directory"));
    };

    if config_dir.is_dir() {
        return Ok(());
    }

    let config_dir = config_dir.as_path();

    info!("Initializing config at {}", config_dir.display());

    std::fs::create_dir_all(config_dir)?;
    std::fs::write(
        config_dir.join(IGNORE_GLOBAL_FILENAME),
        include_str!("../.spwnignore_global"),
    )?;

    Ok(())
}

pub(crate) fn config_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "paulvandermeijs", "spwn")
        .map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
}

pub(crate) fn cache_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "paulvandermeijs", "spwn")
        .map(|proj_dirs| proj_dirs.cache_dir().to_path_buf())
}

pub(crate) fn get_global_ignore() -> Result<Vec<String>> {
    let Some(mut global_ignore_file) = config_dir() else {
        return Err(Error::msg("No config directory"));
    };

    global_ignore_file.push(IGNORE_GLOBAL_FILENAME);

    if !global_ignore_file.is_file() {
        return Err(Error::msg("No global ignore file"));
    }

    info!(
        "Using global ignore file {:?}",
        global_ignore_file.display()
    );

    let lines = crate::fs::read_lines(global_ignore_file)?;

    Ok(lines)
}

fn config_path() -> Result<PathBuf> {
    let Some(mut config_path) = config_dir() else {
        return Err(Error::msg("No config directory"));
    };

    config_path.push("config.toml");

    Ok(config_path)
}
