use anyhow::Result;

use crate::template::{Template, config::Var};

pub(super) fn prompt<'a>(template: &'a Template<'a>, identifier: &str) -> Result<String> {
    let config = template.get_config()?;
    let var = config.get_var(identifier)?;
    let value = match var {
        Var::Text {
            identifier,
            message,
            placeholder,
            initial_value,
            default,
            ..
        } => prompt_text(
            template,
            identifier.as_ref(),
            message.as_deref(),
            placeholder.as_deref(),
            initial_value.as_deref(),
            default.as_deref(),
        )?,
        Var::Select {
            identifier: _,
            message,
            options,
            help_message,
        } => prompt_select(message.as_deref(), &options, help_message.as_deref())?,
    };

    Ok(value)
}

fn prompt_text(
    template: &Template,
    identifier: &str,
    message: Option<&str>,
    placeholder: Option<&str>,
    initial_value: Option<&str>,
    default: Option<&str>,
) -> Result<String> {
    let plugins = template.get_plugins()?;
    let message = message.unwrap_or("");
    let mut prompt = cliclack::input(message);
    if let Some(placeholder) = placeholder {
        prompt = prompt.placeholder(placeholder);
    }
    if let Some(value) = initial_value.or(default) {
        prompt = prompt.default_input(value);
    }
    let validator = {
        let plugins = plugins.clone();
        let ident = identifier.to_string();

        move |input: &String| match plugins.validate(&ident, input) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(message)) => Err(message),
            Err(e) => Err(e.to_string()),
        }
    };
    let mut prompt = prompt.validate(validator);
    let value: String = prompt.interact()?;

    Ok(value)
}

fn prompt_select(
    message: Option<&str>,
    options: &[String],
    help_message: Option<&str>,
) -> Result<String> {
    let message = message.unwrap_or("");
    let mut prompt = cliclack::select(message);
    for (i, option) in options.iter().enumerate() {
        let hint = if i == 0 {
            help_message.unwrap_or("")
        } else {
            ""
        };
        prompt = prompt.item(option.clone(), option, hint);
    }
    let value = prompt.interact()?;

    Ok(value)
}
