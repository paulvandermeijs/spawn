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

impl ToString for PromptResult {
    fn to_string(&self) -> String {
        match self {
            PromptResult::Yes => "Yes".to_string(),
            PromptResult::No => "No".to_string(),
            PromptResult::All => "All".to_string(),
            PromptResult::Diff => "Diff".to_string(),
        }
    }
}

impl std::str::FromStr for PromptResult {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let s = s.as_str();
        let result = match s {
            "yes" | "y" => PromptResult::Yes,
            "no" | "n" => PromptResult::No,
            "all" | "a" => PromptResult::All,
            "diff" | "d" => PromptResult::Diff,
            _ => return Err(()),
        };

        Ok(result)
    }
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
