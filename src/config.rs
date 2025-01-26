use anyhow::{Error, Result};
use directories::ProjectDirs;
use log::info;
use std::path::Path;

pub(crate) fn config_dir() -> Option<std::path::PathBuf> {
    ProjectDirs::from("com", "paulvandermeijs", "spwn")
        .map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
}

pub(crate) fn cache_dir() -> Option<std::path::PathBuf> {
    ProjectDirs::from("com", "paulvandermeijs", "spwn")
        .map(|proj_dirs| proj_dirs.cache_dir().to_path_buf())
}

pub(crate) fn get_global_ignore() -> Result<Vec<String>> {
    let Some(config_dir) = config_dir() else {
        return Err(Error::msg("No config directory"));
    };

    let global_ignore_file = format!("{}/.spwnignore_global", config_dir.display());
    let global_ignore_file = Path::new(&global_ignore_file);

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
