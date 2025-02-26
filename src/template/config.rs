use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Default, Deserialize)]
pub(crate) struct Config {
    #[serde(rename = "var", default = "Vec::new")]
    vars: Vec<Var>,
}

impl Config {
    pub(super) fn try_from_file(path: &Path) -> Result<Config> {
        let config_data = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&config_data)?;

        Ok(config)
    }

    pub(crate) fn get_var(&self, identifier: &str) -> Option<&Var> {
        let identifier = self.vars.iter().find(|v| v.identifier == identifier);

        identifier
    }
}

#[derive(Deserialize)]
pub(crate) struct Var {
    pub identifier: String,
    pub message: Option<String>,
    pub help_message: Option<String>,
}
