//! Menu Kind snapshot content (CR-NR-117, menu-workspace Requirement 23).
//!
//! Pure, GUI-independent builders for the CONTENT AREA of every Menu Kind
//! snapshot (POM, Settings, SCRM and user menus alike -- there is NO menu-name
//! branch here). The option rows are modelled ONCE as [`MenuRowDisplay`] values
//! (the texts the live panel displays) and both the box body lines and the
//! clipboard `Option | Command | Description` table are derived from that single
//! list (Requirement 23.13). The frame (title, Command line, rule, status band,
//! PF-key line) is owned by the shell common frame and is NOT drawn here.
//!
//! Validates: menu-workspace Requirement 23.1-23.14.

use ff_screen_model::{MarkdownSection, Table};

use crate::menu_workspace::{GroupSeparator, MenuFile};

/// The gap, in spaces, between the widest option row and the calendar column
/// (the live panel's 32 px gap is about four monospace cells).
pub const MENU_COLUMN_GAP: usize = 4;

/// One option row as the LIVE panel displays it (active-locale surfaces).
///
/// Validates: menu-workspace Requirement 23.2, 23.13.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuRowDisplay {
    /// Displayed key-column mnemonic.
    pub key: String,
    /// Displayed command-column surface.
    pub command: String,
    /// Displayed (localised) description.
    pub description: String,
    /// Option group label, if any.
    pub group: Option<String>,
}

/// Build the rows from a menu's STORED values (used when the shell supplies no
/// localised surfaces, so the stored values are the displayed values).
pub fn rows_from_menu(menu: &MenuFile) -> Vec<MenuRowDisplay> {
    menu.options
        .iter()
        .map(|o| MenuRowDisplay {
            key: o.key.clone(),
            command: o.command.clone(),
            description: o.description.clone(),
            group: o.group.clone(),
        })
        .collect()
}

/// The group label of a row, ignoring empty labels (as the live panel does).
fn non_empty_group(row: &MenuRowDisplay) -> Option<&str> {
    row.group.as_deref().filter(|g| !g.is_empty())
}

/// Format the box body rows: `<key> <command> : <description>` with the key and
/// command padded to the widest so the colon aligns. Group boundaries follow the
/// live rule (only between two DIFFERENT non-empty groups): a `Line` separator is
/// a rule of `-` as wide as the widest option row, `Space` is an empty row, `None`
/// is nothing. With `group_headers`, the group name precedes its first option.
///
/// Validates: menu-workspace Requirement 23.2, 23.3, 23.4, 23.5, 23.7.
pub fn format_menu_rows(
    rows: &[MenuRowDisplay],
    separator: GroupSeparator,
    group_headers: bool,
) -> Vec<String> {
    let key_w = rows
        .iter()
        .map(|r| r.key.chars().count())
        .max()
        .unwrap_or(0);
    let cmd_w = rows
        .iter()
        .map(|r| r.command.chars().count())
        .max()
        .unwrap_or(0);
    let option_lines: Vec<String> = rows
        .iter()
        .map(|r| {
            format!(
                "{:<key_w$} {:<cmd_w$} : {}",
                r.key, r.command, r.description
            )
            .trim_end()
            .to_string()
        })
        .collect();
    let rule_w = option_lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0);

    let mut out = Vec::new();
    let mut last_group: Option<&str> = None;
    for (row, line) in rows.iter().zip(option_lines) {
        let current = non_empty_group(row);
        if current.is_some() && last_group.is_some() && current != last_group {
            match separator {
                GroupSeparator::Line => out.push("-".repeat(rule_w)),
                GroupSeparator::Space => out.push(String::new()),
                GroupSeparator::None => {}
            }
        }
        if group_headers && current.is_some() && current != last_group {
            if let Some(g) = current {
                out.push(g.to_string());
            }
        }
        if current.is_some() {
            last_group = current;
        }
        out.push(line);
    }
    out
}

/// Compose the two-column body: `left` rows on the LEFT padded to the widest
/// left line plus [`MENU_COLUMN_GAP`], `right` (calendar block) on the RIGHT of
/// the same lines. The calendar column is DERIVED from the rows, never fixed.
/// With an empty `right` the left rows are returned unchanged (no padding).
///
/// Validates: menu-workspace Requirement 23.6.
pub fn compose_with_calendar(left: &[String], right: &[String]) -> Vec<String> {
    if right.is_empty() {
        return left.to_vec();
    }
    let left_width = left.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let rows = left.len().max(right.len());
    (0..rows)
        .map(|i| {
            let l = left.get(i).map(String::as_str).unwrap_or("");
            match right.get(i) {
                Some(r) if !r.is_empty() => {
                    let pad = left_width - l.chars().count() + MENU_COLUMN_GAP;
                    format!("{l}{}{r}", " ".repeat(pad))
                }
                _ => l.to_string(),
            }
        })
        .collect()
}

/// The ONE clipboard table section: `Option | Command | Description`, one row per
/// option in order, no heading. `None` when there are no options (load-error and
/// empty menus emit no table). Cells are the displayed surfaces; the calendar,
/// group rules and group headers are never part of the table.
///
/// Validates: menu-workspace Requirement 23.10, 23.11, 23.12.
pub fn menu_table_section(rows: &[MenuRowDisplay]) -> Option<MarkdownSection> {
    if rows.is_empty() {
        return None;
    }
    Some(MarkdownSection {
        heading: None,
        table: Table {
            caption: None,
            headers: vec![
                "Option".to_string(),
                "Command".to_string(),
                "Description".to_string(),
            ],
            rows: rows
                .iter()
                .map(|r| vec![r.key.clone(), r.command.clone(), r.description.clone()])
                .collect(),
        },
    })
}

#[cfg(test)]
mod tests;
