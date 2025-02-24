# Spawn

A command-line tool for creating files and folders from a template.

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

The template can be a collection of files and folders using Tera template
syntax. The command will scan these files for identifiers and ask for values
before creating the files and folders at their target location.

[Follow this link to learn more about Tera templates.](https://keats.github.io/tera/)

> [!TIP]  
> The `spwn` command was chosen for this tool because it should be easy to type
> on most keyboards by alternating between left and right hand.

### Adding an alias

To avoid having to enter the complete URI every time, it is also possible to add
an alias.

Use the following command to add an alias:

```bash
spwn alias add <NAME> <URI>
```

### Removing an alias

To remove an alias, use the following command:

```bash
spwn alias remove <NAME>
```

### Listing all aliases

To show all available aliases use:

```bash
spwn alias list
```
