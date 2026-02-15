use anyhow::Result;
use log::info;
use serde::Deserialize;
use std::path::Path;

use super::{CONFIG_FILENAME, Template};

pub(crate) struct Config<'a> {
    template: &'a Template<'a>,
    config_file: ConfigFile,
}

impl<'a> Config<'a> {
    pub(super) fn try_from_template(template: &'a Template<'a>) -> Result<Config<'a>> {
        let config_path = template.config_dir()?.as_path().join(CONFIG_FILENAME);

        if !config_path.is_file() {
            return Ok(Self {
                template,
                config_file: ConfigFile::default(),
            });
        }

        info!("Using template config file {config_path:?}");

        let config_file = ConfigFile::try_from_file(&config_path)?;

        Ok(Config {
            template,
            config_file,
        })
    }

    pub(crate) fn get_var(&self, identifier: &str) -> Result<Var> {
        let var = self.config_file.get_var(identifier);
        let plugins = self.template.get_plugins()?;
        let default_message = format!("Provide a value for '{identifier}':");

        let var = match var {
            Var::Text {
                identifier: _,
                message,
                help_message,
                placeholder,
                initial_value,
                default,
            } => {
                let message = match message {
                    Some(message) => message,
                    None => default_message,
                };
                let message = plugins.message(identifier, &message)?;
                let help_message = plugins.help_message(identifier, help_message.as_deref())?;
                let placeholder = plugins.placeholder(identifier, placeholder.as_deref())?;
                let initial_value = plugins.initial_value(identifier, initial_value.as_deref())?;
                let default = plugins.default(identifier, default.as_deref())?;

                Var::Text {
                    identifier: identifier.to_string(),
                    message: Some(message),
                    help_message,
                    placeholder,
                    initial_value,
                    default,
                }
            }
            Var::Select {
                identifier: _,
                message,
                options,
                help_message,
            } => {
                let message = match message {
                    Some(message) => message,
                    None => default_message,
                };
                let message = plugins.message(identifier, &message)?;
                let options = plugins.options(identifier, &options)?;
                let help_message = plugins.help_message(identifier, help_message.as_deref())?;

                Var::Select {
                    identifier: identifier.to_string(),
                    message: Some(message),
                    options,
                    help_message,
                }
            }
        };

        Ok(var)
    }
}

#[derive(Default, Deserialize)]
struct ConfigFile {
    #[serde(rename = "var", default = "Vec::new")]
    vars: Vec<Var>,
}

impl ConfigFile {
    fn try_from_file(path: &Path) -> Result<ConfigFile> {
        let config_data = std::fs::read_to_string(path)?;
        let config: ConfigFile = toml::from_str(&config_data)?;

        Ok(config)
    }

    fn get_var(&self, identifier: &str) -> Var {
        self.vars
            .iter()
            .find(|v| Var::has_identifier(v, identifier))
            .cloned()
            .unwrap_or_default()
    }
}

#[derive(Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub(crate) enum Var {
    Text {
        identifier: String,
        message: Option<String>,
        help_message: Option<String>,
        placeholder: Option<String>,
        initial_value: Option<String>,
        default: Option<String>,
    },
    Select {
        identifier: String,
        message: Option<String>,
        options: Vec<String>,
        help_message: Option<String>,
    },
}

impl Var {
    fn has_identifier(&self, name: &str) -> bool {
        match self {
            Var::Text { identifier, .. } | Var::Select { identifier, .. } => identifier == name,
        }
    }
}

impl Default for Var {
    fn default() -> Self {
        Var::Text {
            identifier: String::new(),
            message: None,
            help_message: None,
            placeholder: None,
            initial_value: None,
            default: None,
        }
    }
}
