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

impl std::fmt::Display for GroupedActions<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.create.is_empty() {
            writeln!(f, "Create the following files:")?;

            for write in &self.create {
                writeln!(f, "- {}", write.name)?;
            }

            if !self.replace.is_empty() {
                writeln!(f)?;
            }
        }

        if !self.replace.is_empty() {
            writeln!(f, "Replace the following files:")?;

            for write in &self.replace {
                writeln!(f, "- {}", write.name)?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
