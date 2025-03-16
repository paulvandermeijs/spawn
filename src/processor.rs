use anyhow::Result;
use log::{info, warn};
use std::{fs::File, path::PathBuf};
use tera::{Context, Tera};

use crate::template::{plugins::Plugins, Template};

const FILENAME_TEMPLATE_NAME: &str = "__filename_template";

pub(crate) struct Processor<'a> {
    template: &'a Template,
}

impl<'a> Processor<'a> {
    pub(crate) fn from_template(template: &'a Template) -> Self {
        Self { template }
    }

    pub(crate) fn process(&self, cwd: PathBuf) -> Result<()> {
        let plugins = self.template.get_plugins();
        let cache_dir = self.template.init()?.cache_dir()?;
        let ignore = self.get_ignore();
        let mut tera = Tera::default();
        let context = Context::new();
        let mut context = plugins.context(context)?;

        info!("Initial context {context:?}");

        for path in walkdir::WalkDir::new(&cache_dir)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|path| {
                let path = pathdiff::diff_paths(path.path(), &cache_dir).unwrap();

                !ignore.is_match(&path)
            })
        {
            let path = path.path();
            let name = pathdiff::diff_paths(path, &cache_dir).unwrap();
            let name = name.to_str().unwrap();
            let name = self.process_filename(&mut tera, &mut context, name)?;

            if path.is_dir() {
                continue;
            }

            tera.add_template_file(path, Some(&name))?;
            self.collect_vars(tera.get_template(&name)?, &mut context)?;

            let mut target = cwd.clone();
            target.push(std::path::Path::new(&name));

            info!("Writing to {target:?}");

            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let file = File::create(target)?;

            tera.render_to(&name, &context, file)?;
        }

        Ok(())
    }

    fn get_ignore(&self) -> globset::GlobSet {
        use globset::{Glob, GlobSetBuilder};

        let mut builder = GlobSetBuilder::new();

        match self.template.get_ignore() {
            Ok(lines) => {
                for line in lines {
                    builder.add(Glob::new(&line).unwrap());
                }
            }
            Err(e) => warn!("Not using template ignore: {:?}", e),
        };

        use crate::config::get_global_ignore;

        match get_global_ignore() {
            Ok(lines) => {
                for line in lines {
                    builder.add(Glob::new(&line).unwrap());
                }
            }
            Err(e) => warn!("Not using global ignore: {:?}", e),
        };

        let ignore = builder.build().unwrap();

        ignore
    }

    fn process_filename(
        &self,
        tera: &mut Tera,
        context: &mut Context,
        name: &str,
    ) -> Result<String, anyhow::Error> {
        tera.add_raw_template(FILENAME_TEMPLATE_NAME, &name)?;
        self.collect_vars(tera.get_template(FILENAME_TEMPLATE_NAME)?, context)?;
        let result = tera.render(FILENAME_TEMPLATE_NAME, &*context);
        tera.templates.remove(FILENAME_TEMPLATE_NAME);
        let name = result?;
        Ok(name)
    }

    fn collect_vars(
        &self,
        tera_template: &tera::Template,
        context: &mut tera::Context,
    ) -> Result<()> {
        let template_config = self.template.get_config();
        use tera::ast::{ExprVal, Node};
        let ast = &tera_template.ast;
        let plugins = self.template.get_plugins();

        for node in ast {
            let Node::VariableBlock(_, expr) = node else {
                continue;
            };
            let ExprVal::Ident(ident) = &expr.val else {
                continue;
            };

            if context.contains_key(ident) {
                continue;
            }

            let message = format!("Provide a value for '{ident}':");
            let (message, help_message, placeholder, initial_value, default) =
                match template_config.get_var(&ident) {
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
            let message = plugins.message(ident, message)?;
            let help_message = plugins.help_message(ident, help_message)?;
            let placeholder = plugins.placeholder(ident, placeholder)?;
            let initial_value = plugins.initial_value(ident, initial_value)?;
            let default = plugins.default(ident, default)?;

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
                    identifier: ident.to_string(),
                }
            });
            let formatter = move |input: &str| plugins.format(&ident, input).unwrap();
            let prompt = prompt.with_formatter(&formatter);
            let validator = {
                let plugins = plugins.clone();
                let ident = ident.to_string();

                use inquire::validator::Validation;

                move |input: &str| match plugins.validate(&ident, input)? {
                    Ok(_) => Ok(Validation::Valid),
                    Err(message) => Ok(Validation::Invalid(message.into())),
                }
            };
            let prompt = prompt.with_validator(validator);
            let value = prompt.prompt()?;

            info!("Collected value {value:?} for {ident:?}");

            context.insert(ident, &value);
        }

        Ok(())
    }
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
