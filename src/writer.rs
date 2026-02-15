mod diff;
mod prompt;

use anyhow::Result;
use log::info;
use prompt::PromptResult;
use std::fs::File;

use crate::processor::{
    ProcessResult,
    actions::{Action, Write},
};

pub(crate) struct Writer<'a> {
    process_result: &'a ProcessResult,
}

impl<'a> Writer<'a> {
    pub(crate) fn from_process_result(process_result: &'a ProcessResult) -> Self {
        Writer { process_result }
    }

    pub(crate) fn write(&self) -> Result<()> {
        let tera = &self.process_result.tera;
        let context = &self.process_result.context;
        let mut replace_all = false;

        for action in &self.process_result.actions {
            let (name, target) = match action {
                Action::Create(write) => (&write.name, &write.target),
                Action::Replace(write) => {
                    if !replace_all {
                        let prompt_result = self.prompt(write)?;

                        if let PromptResult::No = prompt_result {
                            continue;
                        }

                        if let PromptResult::All = prompt_result {
                            replace_all = true;
                        }
                    }

                    (&write.name, &write.target)
                }
            };

            info!("Writing to {target:?}");

            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let file = File::create(target)?;

            tera.render_to(name, context, file)?;
        }

        Ok(())
    }

    fn prompt(&self, write: &Write) -> Result<PromptResult> {
        let prompt_result = loop {
            let prompt_result = prompt::prompt(write)?;

            if let PromptResult::Diff = prompt_result {
                let tera = &self.process_result.tera;
                let context = &self.process_result.context;
                diff::diff(tera, context, write)?;
            } else {
                break prompt_result;
            }
        };

        Ok(prompt_result)
    }
}
