use anyhow::Result;

use crate::template::{plugins::Plugins, Template};

pub(super) fn prompt(template: &Template, identifier: &str) -> Result<String> {
    let template_config = template.get_config();
    let plugins = template.get_plugins();
    let message = format!("Provide a value for '{identifier}':");
    let (message, help_message, placeholder, initial_value, default) =
        match template_config.get_var(&identifier) {
            Some(var) => {
                let message = match &var.message {
                    Some(message) => message.as_str(),
                    None => &message,
                };
                let help_message = var.help_message.as_ref().map(|s| s.as_str());
                let placeholder = var.placeholder.as_ref().map(|s| s.as_str());
                let initial_value = var.initial_value.as_ref().map(|s| s.as_str());
                let default = var.default.as_ref().map(|s| s.as_str());

                (message, help_message, placeholder, initial_value, default)
            }
            None => (message.as_str(), None, None, None, None),
        };
    let message = plugins.message(identifier, message)?;
    let help_message = plugins.help_message(identifier, help_message)?;
    let placeholder = plugins.placeholder(identifier, placeholder)?;
    let initial_value = plugins.initial_value(identifier, initial_value)?;
    let default = plugins.default(identifier, default)?;

    let prompt = inquire::Text::new(&message);
    let prompt = match &help_message {
        Some(help_message) => prompt.with_help_message(help_message),
        None => prompt,
    };
    let prompt = match &placeholder {
        Some(placeholder) => prompt.with_placeholder(placeholder),
        None => prompt,
    };
    let prompt = match &initial_value {
        Some(initial_value) => prompt.with_initial_value(initial_value),
        None => prompt,
    };
    let prompt = match &default {
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
