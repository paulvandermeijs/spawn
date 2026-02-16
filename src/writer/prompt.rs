use anyhow::Result;

use crate::processor::actions::Write;

#[derive(Clone, Eq, PartialEq)]
pub(super) enum PromptResult {
    Yes,
    No,
    All,
    Diff,
}

pub(super) fn prompt(write: &Write) -> Result<PromptResult> {
    let message = format!("Are you sure you wish to replace '{}'?", write.name);
    let result = cliclack::select(&message)
        .item(PromptResult::Yes, "Yes", "Replace this file")
        .item(PromptResult::No, "No", "Skip this file")
        .item(PromptResult::All, "All", "Replace all remaining files")
        .item(PromptResult::Diff, "Diff", "Show differences first")
        .interact()?;

    Ok(result)
}
