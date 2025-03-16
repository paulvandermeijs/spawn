use anyhow::Result;
use log::info;
use std::{env, path::PathBuf, str::FromStr};

use crate::{config::Config, processor::Processor, template::Template};

pub(crate) fn spawn(config: &Config, uri: String) -> Result<()> {
    let uri = config.resolve_alias(uri);

    info!("Using template {uri:?}");

    let template = Template::from_uri(uri);
    let plugins = template.get_plugins();
    let info = template.get_info().map(|x| x.as_str());
    let info = plugins.info(info)?;

    if let Some(info) = info {
        println!("{info}");
    }

    let cwd = env::current_dir()?;
    let cwd = plugins.cwd(cwd.to_string_lossy().to_string())?;
    let cwd = PathBuf::from_str(&cwd)?;

    info!("The current directory is {:?}", cwd.display());

    let processor = Processor::from_template(&template);

    processor.process(cwd)
}
