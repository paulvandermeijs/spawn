use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::{info, warn};
use std::{env, fs::File};
use tera::{Context, Tera};

use crate::{
    config::{get_global_ignore, Config},
    template::Template,
};

const FILENAME_TEMPLATE_NAME: &str = "__filename_template";

pub(crate) fn spawn(config: &Config, uri: String) -> Result<()> {
    let uri = config.resolve_alias(uri);

    info!("Using template {uri:?}");

    let cwd = env::current_dir()?;

    info!("The current directory is {:?}", cwd.display());

    let template = Template::from_uri(uri);
    let cache_dir = template.init()?.cache_dir()?;
    let ignore = get_ignore(&template);
    let mut tera = Tera::default();
    let mut context = Context::new();

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
        let name = process_filename(&mut tera, &mut context, name)?;

        if path.is_dir() {
            continue;
        }

        tera.add_template_file(path, Some(&name))?;
        collect_vars(tera.get_template(&name)?, &mut context)?;

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

fn get_ignore(template: &Template) -> GlobSet {
    let mut builder = GlobSetBuilder::new();

    match template.get_ignore() {
        Ok(lines) => {
            for line in lines {
                builder.add(Glob::new(&line).unwrap());
            }
        }
        Err(e) => warn!("Not using template ignore: {:?}", e),
    };

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
    tera: &mut Tera,
    context: &mut Context,
    name: &str,
) -> Result<String, anyhow::Error> {
    tera.add_raw_template(FILENAME_TEMPLATE_NAME, &name)?;
    collect_vars(tera.get_template(FILENAME_TEMPLATE_NAME)?, context)?;
    let result = tera.render(FILENAME_TEMPLATE_NAME, &*context);
    tera.templates.remove(FILENAME_TEMPLATE_NAME);
    let name = result?;
    Ok(name)
}

fn collect_vars(template: &tera::Template, context: &mut tera::Context) -> Result<()> {
    use tera::ast::{ExprVal, Node};
    let ast = &template.ast;

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

        let value = inquire::Text::new(&format!("Provide a value for '{ident}':")).prompt()?;

        context.insert(ident, &value);
    }

    Ok(())
}
