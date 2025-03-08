mod config;
pub(crate) mod plugins;

use anyhow::{Error, Result};
use log::info;
use plugins::Plugins;
use std::{path::PathBuf, sync::OnceLock};

use crate::config::cache_dir;
use crate::template::config::Config;

const IGNORE_FILENAME: &str = ".spwnignore";
const CONFIG_DIR: &str = ".spwn";
const CONFIG_FILENAME: &str = "config.toml";
const PLUGINS_FILENAME: &str = "plugins.scm";
const INFO_FILENAME: &str = "info.txt";

#[derive(Default)]
pub(crate) struct Template {
    pub uri: String,
    pub hash: String,
    config: OnceLock<Config>,
    plugins: OnceLock<Plugins>,
    info: OnceLock<Option<String>>,
}

impl Template {
    pub fn from_uri(uri: String) -> Self {
        let hash = create_hash(&uri);

        Template {
            uri,
            hash,
            ..Default::default()
        }
    }

    pub fn init(&self) -> Result<&Self> {
        let cache_dir = self.cache_dir()?;

        if cache_dir.is_dir() {
            return Ok(self);
        }

        crate::repo::clone(&self.uri, &cache_dir)?;

        Ok(self)
    }

    pub fn cache_dir(&self) -> Result<PathBuf> {
        let Some(mut cache_dir) = cache_dir() else {
            return Err(Error::msg("No cache directory"));
        };

        cache_dir.push(&self.hash);

        Ok(cache_dir)
    }

    pub fn config_dir(&self) -> Result<PathBuf> {
        let config_dir = self.cache_dir()?.as_path().join(CONFIG_DIR);

        Ok(config_dir)
    }

    pub fn get_ignore(&self) -> Result<Vec<String>> {
        let template_ignore_file = self.cache_dir()?.as_path().join(IGNORE_FILENAME);

        if !template_ignore_file.is_file() {
            return Err(Error::msg("No template ignore file"));
        }

        info!(
            "Using template ignore file {:?}",
            template_ignore_file.display()
        );

        let lines = crate::fs::read_lines(template_ignore_file)?;

        Ok(lines)
    }

    pub fn get_config(&self) -> &Config {
        let config = self.config.get_or_init(|| {
            let config_path = self.config_dir().unwrap().as_path().join(CONFIG_FILENAME);

            if !config_path.is_file() {
                return Config::default();
            }

            info!("Using project config file {config_path:?}");

            let config = Config::try_from_file(&config_path).unwrap();

            config
        });

        config
    }

    pub fn get_plugins(&self) -> &Plugins {
        let plugins = self.plugins.get_or_init(|| {
            let plugins_path = self.config_dir().unwrap().as_path().join(PLUGINS_FILENAME);

            Plugins::try_from_file(&plugins_path).unwrap()
        });

        plugins
    }

    pub fn get_info(&self) -> Option<&String> {
        let info = self.info.get_or_init(|| {
            let info_path = self.config_dir().unwrap().as_path().join(INFO_FILENAME);

            if !info_path.is_file() {
                return None;
            }

            info!("Using info from {info_path:?}");

            let info = std::fs::read_to_string(&info_path).unwrap();

            Some(info)
        });

        info.as_ref()
    }
}

fn create_hash(path: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    hasher.update(&path);

    let result = hasher.finalize();
    let hash = format!("{:x}", result);

    hash
}
