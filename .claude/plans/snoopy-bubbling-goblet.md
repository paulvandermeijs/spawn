# Replace `inquire` with `cliclack`

## Context

Replace the `inquire` prompt library with `cliclack` for a more modern, minimal CLI aesthetic. Cliclack doesn't support autocomplete, custom formatters, or help messages on text inputs, so those features and the associated plugin methods will be removed.

## Changes

### 1. `Cargo.toml` — swap dependency

Replace `inquire = "0.9.3"` with `cliclack = "0.3"`.

### 2. `src/processor/prompt.rs` — rewrite prompts

**Text prompt** (`prompt_text`):
- `inquire::Text::new(message)` → `cliclack::input(message)`
- `.with_placeholder()` → `.placeholder()`
- `.with_initial_value()` / `.with_default()` → `.default_input()` (initial_value takes priority)
- `.with_validator()` → `.validate()` with `Result<(), String>` return instead of `Validation::Valid/Invalid`
- `.prompt()` → `.interact()`
- Remove: `help_message` parameter (no equivalent), autocomplete, formatter
- Remove: `Autocomplete` struct and `impl inquire::Autocomplete`
- Remove: `use inquire::validator::Validation`

**Select prompt** (`prompt_select`):
- `inquire::Select::new(message, options)` → `cliclack::select(message)` with `.item()` per option
- Use `help_message` as hint on the first item only (empty hint `""` for the rest)
- `.prompt()` → `.interact()`

### 3. `src/writer/prompt.rs` — replace `CustomType` with `select`

Replace `inquire::CustomType<PromptResult>` with `cliclack::select()` using four `.item()` calls (Yes/No/All/Diff with descriptive hints).

Remove `impl Display` and `impl FromStr` for `PromptResult` — no longer needed.

Add `Eq` and `PartialEq` derives to `PromptResult` (required by `cliclack::Select<T>`).

### 4. `src/template/plugins.rs` — remove unused methods

Delete these methods and their associated constants:
- `suggestions()` / `FUNCTION_SUGGESTIONS`
- `completion()` / `FUNCTION_COMPLETION`
- `format()` / `FUNCTION_FORMAT`

### 5. `src/processor/prompt.rs` — remove `Plugins` import

The `Plugins` type is no longer used directly in `prompt.rs` (was only needed for the `Autocomplete` struct). Remove from imports.

## Verification

1. `cargo clippy --all-targets --all-features` — zero warnings/errors
2. `cargo fmt -- --check` — no formatting issues
