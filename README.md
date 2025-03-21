# Spawn

A scriptable command-line tool for creating files and folders from a template.

## Installation

Use Cargo to install the application.

```bash
cargo install spawn-cli
```

## Usage

Copy files from a template to the current directory using the following command
providing the location of a Git repository as `URI`.

```bash
spwn <URI>
```

[Follow this link to learn more about using the command.](https://github.com/paulvandermeijs/spawn/wiki/Creating-A-Template)

> [!TIP]  
> The `spwn` command was chosen for this tool because it should be easy to type
> on most keyboards by alternating between left and right hand.

The template can be a collection of files and folders using Tera template
syntax. The command will scan these files for identifiers and ask for values
before creating the files and folders at their target location.

[Follow this link to learn to learn how to create your own templates.](https://keats.github.io/tera/)
