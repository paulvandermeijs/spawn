use anyhow::Result;

use crate::template::{config::Var, plugins::Plugins, Template};

pub(super) fn prompt<'a>(template: &'a Template<'a>, identifier: &str) -> Result<String> {
    let config = template.get_config();
    let var = config.get_var(identifier)?;
    let value = match var {
        Var::Text {
            identifier,
            message,
            help_message,
            placeholder,
            initial_value,
            default,
        } => prompt_text(
            template,
            identifier.as_ref(),
            message,
            help_message,
            placeholder,
            initial_value,
            default,
        )?,
        Var::Select {
            identifier: _,
            message,
            options,
            help_message,
        } => prompt_select(message, options, help_message)?,
    };

    Ok(value)
}

fn prompt_text(
    template: &Template,
    identifier: &str,
    message: Option<String>,
    help_message: Option<String>,
    placeholder: Option<String>,
    initial_value: Option<String>,
    default: Option<String>,
) -> Result<String> {
    let plugins = template.get_plugins();
    let message = message.as_ref().map_or_else(|| "", |s| s.as_str());
    let prompt = inquire::Text::new(message);
    let prompt = match help_message.as_ref() {
        Some(help_message) => prompt.with_help_message(help_message),
        None => prompt,
    };
    let prompt = match placeholder.as_ref() {
        Some(placeholder) => prompt.with_placeholder(placeholder),
        None => prompt,
    };
    let prompt = match initial_value.as_ref() {
        Some(initial_value) => prompt.with_initial_value(initial_value),
        None => prompt,
    };
    let prompt = match default.as_ref() {
        Some(default) => prompt.with_default(default),
        None => prompt,
    };
    let prompt = prompt.with_autocomplete({
        let plugins = plugins.clone();

        Autocomplete {
            plugins,
            identifier: identifier.to_string(),
        }
    });
    let formatter = move |input: &str| plugins.format(&identifier, input).unwrap();
    let prompt = prompt.with_formatter(&formatter);
    let validator = {
        let plugins = plugins.clone();
        let ident = identifier.to_string();

        use inquire::validator::Validation;

        move |input: &str| match plugins.validate(&ident, input)? {
            Ok(_) => Ok(Validation::Valid),
            Err(message) => Ok(Validation::Invalid(message.into())),
        }
    };
    let prompt = prompt.with_validator(validator);
    let value = prompt.prompt()?;

    Ok(value)
}

fn prompt_select(
    message: Option<String>,
    options: Vec<String>,
    help_message: Option<String>,
) -> Result<String> {
    let message = message.as_ref().map_or_else(|| "", |s| s.as_str());
    let prompt = inquire::Select::new(message, options);
    let prompt = match help_message.as_ref() {
        Some(help_message) => prompt.with_help_message(help_message),
        None => prompt,
    };

    let value = prompt.prompt()?;

    Ok(value)
}

#[derive(Clone)]
struct Autocomplete {
    plugins: Plugins,
    identifier: String,
}

impl inquire::Autocomplete for Autocomplete {
    fn get_suggestions(
        &mut self,
        input: &str,
    ) -> std::result::Result<Vec<String>, inquire::CustomUserError> {
        self.plugins
            .suggestions(&self.identifier, input)
            .map_err(|e| e.to_string().into())
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> std::result::Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        Ok(self.plugins.completion(
            &self.identifier,
            input,
            highlighted_suggestion.as_ref().map(|v| v.as_str()),
        )?)
    }
}
