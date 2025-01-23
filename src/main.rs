use clap::{Parser, Subcommand};

/// Create files and folders from templates
#[derive(Parser, Debug)]
#[command(name = "Spawn", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Path to a template
    #[arg()]
    path: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add an alias for a URL
    Alias { name: String, url: String },
}

fn main() {
    let cli = Cli::parse();

    let result: Result<usize, String> = match (cli.path, cli.command) {
        (Some(_), None) => todo!("Implement `path`"),
        (None, Some(Commands::Alias { name: _, url: _ })) => todo!("Implement `alias`"),
        _ => Err("Provide either a command or path".to_string()),
    };

    let code = match result {
        Ok(_) => 0,
        Err(message) => {
            eprintln!("{}", message);

            1
        }
    };

    std::process::exit(code);
}
