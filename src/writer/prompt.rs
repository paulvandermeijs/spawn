use std::fmt;

use anyhow::Result;
use inquire::CustomType;

use crate::processor::actions::Write;

#[derive(Clone)]
pub(super) enum PromptResult {
    Yes,
    No,
    All,
    Diff,
}

pub(super) fn prompt(write: &Write) -> Result<PromptResult> {
    let message = format!("Are you sure you wish to replace '{}'?", write.name);
    let formatter = &|ans| match ans {
        PromptResult::Yes => "Y/n/a/d".to_string(),
        PromptResult::No => "y/N/a/d".to_string(),
        PromptResult::All => "y/n/A/d".to_string(),
        PromptResult::Diff => "y/n/a/D".to_string(),
    };
    let result = CustomType::<PromptResult>::new(&message)
        .with_default(PromptResult::Yes)
        .with_default_value_formatter(formatter)
        .prompt()?;

    Ok(result)
}

impl fmt::Display for PromptResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PromptResult::Yes => write!(f, "Yes"),
            PromptResult::No => write!(f, "No"),
            PromptResult::All => write!(f, "All"),
            PromptResult::Diff => write!(f, "Diff"),
        }
    }
}

impl std::str::FromStr for PromptResult {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let result = match s.as_str() {
            "yes" | "y" => PromptResult::Yes,
            "no" | "n" => PromptResult::No,
            "all" | "a" => PromptResult::All,
            "diff" | "d" => PromptResult::Diff,
            _ => return Err(()),
        };

        Ok(result)
    }
}
