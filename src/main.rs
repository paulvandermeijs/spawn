use clap::Parser;

/// Create files and folders from templates
#[derive(Parser, Debug)]
#[command(name = "Spawn", version, about, long_about = None)]
struct Cli {
    /// Path to a template
    #[arg()]
    path: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let result: Result<usize, String> = match cli.path {
        Some(_) => todo!("Implement `path`"),
        None => Err("Provide a path".to_string()),
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
