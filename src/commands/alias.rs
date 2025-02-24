use anyhow::Result;
use log::{info, warn};

use crate::config::Config;

pub(crate) fn add(config: &mut Config, name: String, uri: String) -> Result<()> {
    info!("Adding alias {name:?} for {uri:?}");

    config.add_alias(name, uri).write()?;

    Ok(())
}

pub(crate) fn remove(config: &mut Config, name: String) -> Result<()> {
    info!("Removing alias {name:?}");

    config.remove_alias(name).write()?;

    Ok(())
}

pub(crate) fn list(config: &Config) -> Result<()> {
    use comfy_table::modifiers::UTF8_ROUND_CORNERS;
    use comfy_table::presets::UTF8_FULL;
    use comfy_table::*;

    let aliases = config.get_aliases();
    let mut table = Table::new();

    if aliases.len() < 1 {
        warn!("No aliases configured");
    }

    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Name", "URI"]);

    for alias in aliases {
        table.add_row(vec![alias.0, alias.1]);
    }

    println!("{table}");

    Ok(())
}
