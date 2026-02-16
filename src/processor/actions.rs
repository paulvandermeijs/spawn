use std::path::PathBuf;

pub(crate) enum Action {
    Create(Write),
    Replace(Write),
}

pub(crate) struct Write {
    /// The name of the template.
    pub(crate) name: String,
    /// The target path to write the contents of the template to.
    pub(crate) target: PathBuf,
}

pub(crate) trait ActionVec {
    fn get_grouped_actions(&self) -> GroupedActions<'_>;
}

impl From<Write> for Action {
    fn from(value: Write) -> Self {
        if value.target.is_file() {
            Action::Replace(value)
        } else {
            Action::Create(value)
        }
    }
}

impl ActionVec for Vec<Action> {
    fn get_grouped_actions(&self) -> GroupedActions<'_> {
        let mut grouped_actions = GroupedActions::default();

        for action in self {
            match action {
                Action::Create(write) => grouped_actions.create.push(write),
                Action::Replace(write) => grouped_actions.replace.push(write),
            }
        }

        grouped_actions
    }
}

#[derive(Default)]
pub(crate) struct GroupedActions<'a> {
    create: Vec<&'a Write>,
    replace: Vec<&'a Write>,
}

impl GroupedActions<'_> {
    pub(crate) fn log(&self) -> std::io::Result<()> {
        if !self.create.is_empty() {
            let files = self
                .create
                .iter()
                .map(|w| format!("- {}", w.name))
                .collect::<Vec<_>>()
                .join("\n");
            cliclack::log::info(format!("Create the following files:\n{files}"))?;
        }

        if !self.replace.is_empty() {
            let files = self
                .replace
                .iter()
                .map(|w| format!("- {}", w.name))
                .collect::<Vec<_>>()
                .join("\n");
            cliclack::log::info(format!("Replace the following files:\n{files}"))?;
        }

        Ok(())
    }
}
