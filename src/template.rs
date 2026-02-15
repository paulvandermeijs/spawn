pub(crate) mod config;
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

pub(crate) struct Template<'a> {
    pub uri: String,
    pub hash: String,
    config: OnceLock<Config<'a>>,
    plugins: OnceLock<Plugins>,
    info: OnceLock<Option<String>>,
}

impl<'a> Template<'a> {
    pub fn new(uri: String) -> Self {
        let hash = create_hash(&uri);

        Template {
            uri,
            hash,
            config: OnceLock::new(),
            plugins: OnceLock::new(),
            info: OnceLock::new(),
        }
    }

    pub fn init(self) -> Result<Self> {
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

    pub fn get_config(&'a self) -> Result<&'a Config<'a>> {
        if let Some(config) = self.config.get() {
            return Ok(config);
        }

        let config = Config::try_from_template(self)?;
        let _ = self.config.set(config);

        Ok(self.config.get().unwrap())
    }

    pub fn get_plugins(&self) -> Result<&Plugins> {
        if let Some(plugins) = self.plugins.get() {
            return Ok(plugins);
        }

        let plugins_path = self.config_dir()?.as_path().join(PLUGINS_FILENAME);
        let plugins = Plugins::try_from_file(&plugins_path)?;
        let _ = self.plugins.set(plugins);

        Ok(self.plugins.get().unwrap())
    }

    pub fn get_info(&self) -> Result<Option<&String>> {
        if let Some(info) = self.info.get() {
            return Ok(info.as_ref());
        }

        let info_path = self.config_dir()?.as_path().join(INFO_FILENAME);

        if !info_path.is_file() {
            let _ = self.info.set(None);
            return Ok(None);
        }

        info!("Using info from {info_path:?}");

        let info = std::fs::read_to_string(&info_path)?;
        let _ = self.info.set(Some(info));

        Ok(self.info.get().and_then(|i| i.as_ref()))
    }
}

fn create_hash(path: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    hasher.update(path);

    format!("{:x}", hasher.finalize())
}
