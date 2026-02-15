# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build              # Build the project
cargo run -- <URI>       # Run with a template URI
cargo run -- alias ls    # Run alias subcommand
cargo clippy             # Lint
cargo fmt                # Format
```

The binary is named `spwn` (installed via `cargo install spawn-cli`).

There are no tests in this codebase.

## Architecture

Spawn is a CLI tool that creates files and folders from Git-hosted templates using Tera templating. The pipeline flows through four stages:

1. **Template** (`src/template.rs`) — Clones a Git repo (via `gix`) into a cache directory (hashed by URI). Lazily loads config, plugins, and info from the `.spwn/` directory within the template repo.

2. **Processor** (`src/processor.rs`) — Walks the cached template files, processes filenames as Tera templates, parses the AST to discover template variables, and interactively prompts the user for values. Produces a `ProcessResult` containing the Tera engine, context, and a list of actions (Create/Replace).

3. **Writer** (`src/writer.rs`) — Renders templates to their target paths. For files that already exist (Replace actions), prompts the user with Yes/No/All/Diff options.

4. **Commands** (`src/commands/`) — `spawn` orchestrates the full pipeline; `alias` manages URI aliases stored in the app config.

### Plugin System

Templates can include a `.spwn/plugins.scm` file containing Steel (Scheme) scripts. The `Plugins` struct (`src/template/plugins.rs`) runs these scripts in a Steel VM and exposes hook functions that customize behavior: `cwd`, `info`, `context`, `message`, `help-message`, `placeholder`, `initial-value`, `default`, `suggestions`, `completion`, `format`, `validate`, and `options`.

### Template Config

Templates can include `.spwn/config.toml` to configure variables with input types (`text` or `select`), prompt messages, defaults, placeholders, and help messages. See `src/template/config.rs`.

### Tera Extensions

Custom case-conversion filters are registered in `src/processor/tera_extensions.rs`: `camel_case`, `kebab_case`, `pascal_case`, `snake_case`, `title_case`, `train_case`, `upper_kebab_case`, `upper_snake_case`.

### App Config

User-level config (`src/config.rs`) stores aliases in a TOML file and manages a global `.spwnignore_global` file for patterns to always exclude from templates.
