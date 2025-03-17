mod prompt;

use anyhow::Result;
use log::{info, warn};
use std::{fs::File, path::PathBuf};
use tera::{Context, Tera};

use crate::template::Template;

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
        use tera::ast::{ExprVal, Node};
        let ast = &tera_template.ast;

        for node in ast {
            let Node::VariableBlock(_, expr) = node else {
                continue;
            };
            let ExprVal::Ident(identifier) = &expr.val else {
                continue;
            };

            if context.contains_key(identifier) {
                continue;
            }

            let value = prompt::prompt(self.template, identifier)?;

            info!("Collected value {value:?} for {identifier:?}");

            context.insert(identifier, &value);
        }

        Ok(())
    }
}
