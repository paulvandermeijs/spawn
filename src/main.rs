mod config;
mod fs;
mod repo;
mod spawn;
mod template;

use anyhow::{Error, Result};
use clap::Parser;
use clap_verbosity::Verbosity;
use log::error;

/// Create files and folders from templates
#[derive(Parser, Debug)]
#[command(name = "Spawn", version, about, long_about = None)]
struct Cli {
    /// Location of the template
    #[arg()]
    uri: Option<String>,
    #[command(flatten)]
    verbose: Verbosity,
}

fn main() {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    let result: Result<()> = match cli.uri {
        Some(uri) => spawn::spawn(uri),
        None => Err(Error::msg("Provide a location of a template")),
    };

    let code = match result {
        Ok(_) => 0,
        Err(message) => {
            error!("{message}");

            1
        }
    };

    std::process::exit(code);
}
