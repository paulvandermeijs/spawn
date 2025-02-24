use anyhow::{Error, Result};
use log::info;
use std::path::PathBuf;

use crate::config::cache_dir;

pub(crate) struct Template {
    pub uri: String,
    pub hash: String,
}

impl Template {
    pub fn from_uri(uri: String) -> Self {
        let hash = create_hash(&uri);
        Template { uri, hash }
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

    pub fn get_ignore(&self) -> Result<Vec<String>> {
        let Some(mut template_ignore_file) = cache_dir() else {
            return Err(Error::msg("No cache directory"));
        };

        template_ignore_file.push(".spwnignore");

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
}

fn create_hash(path: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    hasher.update(&path);

    let result = hasher.finalize();
    let hash = format!("{:x}", result);

    hash
}
