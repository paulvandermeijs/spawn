use anyhow::Result;
use log::info;
use std::fs::File;

use crate::processor::{actions::Action, ProcessResult};

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

        for action in &self.process_result.actions {
            let (name, target) = match action {
                Action::Create(write) => (&write.name, &write.target),
                Action::Replace(write) => {
                    if !self.prompt(write)? {
                        continue;
                    }

                    (&write.name, &write.target)
                }
            };

            info!("Writing to {target:?}");

            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let file = File::create(target)?;

            tera.render_to(&name, context, file)?;
        }

        Ok(())
    }

    fn prompt(&self, write: &crate::processor::actions::Write) -> Result<bool> {
        let message = format!("Are you sure you wish to replace '{}'?", write.name);
        let result = inquire::Confirm::new(&message)
            .with_default(true)
            .prompt()?;

        Ok(result)
    }
}
