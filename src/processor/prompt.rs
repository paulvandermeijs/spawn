use anyhow::Result;
use inquire::validator::Validation;

use crate::template::{Template, config::Var, plugins::Plugins};

pub(super) fn prompt<'a>(template: &'a Template<'a>, identifier: &str) -> Result<String> {
    let config = template.get_config()?;
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
            message.as_deref(),
            help_message.as_deref(),
            placeholder.as_deref(),
            initial_value.as_deref(),
            default.as_deref(),
        )?,
        Var::Select {
            identifier: _,
            message,
            options,
            help_message,
        } => prompt_select(message.as_deref(), options, help_message.as_deref())?,
    };

    Ok(value)
}

fn prompt_text(
    template: &Template,
    identifier: &str,
    message: Option<&str>,
    help_message: Option<&str>,
    placeholder: Option<&str>,
    initial_value: Option<&str>,
    default: Option<&str>,
) -> Result<String> {
    let plugins = template.get_plugins()?;
    let message = message.unwrap_or("");
    let prompt = inquire::Text::new(message);
    let prompt = match help_message {
        Some(help_message) => prompt.with_help_message(help_message),
        None => prompt,
    };
    let prompt = match placeholder {
        Some(placeholder) => prompt.with_placeholder(placeholder),
        None => prompt,
    };
    let prompt = match initial_value {
        Some(initial_value) => prompt.with_initial_value(initial_value),
        None => prompt,
    };
    let prompt = match default {
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
    let formatter = {
        let plugins = plugins.clone();
        let ident = identifier.to_string();

        move |input: &str| {
            plugins
                .format(&ident, input)
                .unwrap_or_else(|_| input.to_string())
        }
    };
    let prompt = prompt.with_formatter(&formatter);
    let validator = {
        let plugins = plugins.clone();
        let ident = identifier.to_string();

        move |input: &str| match plugins.validate(&ident, input)? {
            Ok(()) => Ok(Validation::Valid),
            Err(message) => Ok(Validation::Invalid(message.into())),
        }
    };
    let prompt = prompt.with_validator(validator);
    let value = prompt.prompt()?;

    Ok(value)
}

fn prompt_select(
    message: Option<&str>,
    options: Vec<String>,
    help_message: Option<&str>,
) -> Result<String> {
    let message = message.unwrap_or("");
    let prompt = inquire::Select::new(message, options);
    let prompt = match help_message {
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
        Ok(self
            .plugins
            .completion(&self.identifier, input, highlighted_suggestion.as_deref())?)
    }
}
