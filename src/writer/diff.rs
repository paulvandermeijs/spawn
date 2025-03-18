use anyhow::Result;
use console::Style;
use similar::{ChangeTag, TextDiff};
use tera::{Context, Tera};

use crate::writer::Write;

pub(super) fn diff(tera: &Tera, context: &Context, write: &Write) -> Result<()> {
    let new = tera.render(&write.name, context)?;
    let old = std::fs::read_to_string(&write.target)?;
    let diff = TextDiff::from_lines(&old, &new);

    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let (sign, style) = match change.tag() {
                ChangeTag::Delete => ("-", Style::new().red()),
                ChangeTag::Insert => ("+", Style::new().green()),
                ChangeTag::Equal => (" ", Style::new()),
            };
            print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
        }
    }

    Ok(())
}
