mod commands;
mod config;
mod fs;
mod processor;
mod repo;
mod template;
mod writer;

use anyhow::{Error, Result};
use clap::{Parser, Subcommand};
use clap_verbosity::Verbosity;
use config::Config;
use log::error;

use commands::{alias, spawn};

/// Create files and folders from templates
#[derive(Parser, Debug)]
#[command(name = "Spawn", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Location of the template
    #[arg()]
    uri: Option<String>,
    #[command(flatten)]
    verbose: Verbosity,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage aliases
    Alias {
        #[command(subcommand)]
        command: AliasCommands,
    },
}

#[derive(Debug, Subcommand)]
enum AliasCommands {
    /// Add an alias for a URI
    Add {
        /// A name for the alias
        name: String,
        /// The URI to use for the alias
        uri: String,
    },
    /// Remove an existing alias
    Remove {
        /// The name of the alias
        name: String,
    },
    /// List all current aliases
    #[command(visible_alias = "ls")]
    List,
}

fn main() {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    let result: Result<()> = (|| {
        config::init()?;

        let mut config = Config::read()?;

        match (cli.uri, cli.command) {
            (Some(uri), None) => spawn::spawn(&config, uri),
            (None, Some(Commands::Alias { command })) => match command {
                AliasCommands::Add { name, uri } => alias::add(&mut config, name, uri),
                AliasCommands::Remove { name } => alias::remove(&mut config, &name),
                AliasCommands::List => {
                    alias::list(&config);
                    Ok(())
                }
            },
            _ => Err(Error::msg(
                "Provide either a command or location of a template",
            )),
        }
    })();

    let code = match result {
        Ok(()) => 0,
        Err(message) => {
            error!("{message}");

            1
        }
    };

    std::process::exit(code);
}
