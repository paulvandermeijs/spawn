use anyhow::Result;
use log::info;
use std::{
    env::{self, set_current_dir},
    path::PathBuf,
    str::FromStr,
};

use crate::{config::Config, processor::Processor, template::Template, writer::Writer};

pub(crate) fn spawn(config: &Config, uri: String) -> Result<()> {
    let uri = config.resolve_alias(uri);

    info!("Using template {uri:?}");

    let template = Template::from_uri(uri).init()?;
    let plugins = template.get_plugins();
    let info = template.get_info().map(|x| x.as_str());
    let info = plugins.info(info)?;

    if let Some(info) = info {
        println!("{info}");
    }

    let cwd = env::current_dir()?;
    let cwd = plugins.cwd(cwd.to_string_lossy().to_string())?;
    let cwd = PathBuf::from_str(&cwd)?;

    set_current_dir(&cwd)?;

    info!("The current directory is {:?}", cwd.display());

    let processor = Processor::from_template(&template);
    let process_result = processor.process(cwd)?;

    println!("");
    print!("{process_result}");

    let writer = Writer::from_process_result(&process_result);

    writer.write()
}
