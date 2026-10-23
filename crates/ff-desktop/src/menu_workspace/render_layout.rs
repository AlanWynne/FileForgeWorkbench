//! Pure layout helpers + colours for the shared Menu_Workspace renderer.
//!
//! Split out of `render.rs` (CR-NR-109 Task 11.5) as a PURE REFACTOR with no
//! behaviour change: these items hold the menu option-column colours, the fixed
//! calendar-fit constants, and the deterministic column-width / natural-width
//! measurement helpers the frame loop in `render.rs` consumes. Keeping them in
//! their own file keeps both files under the 400-line rule before the command
//! -column localisation change grows `render.rs`.
//!
//! Validates: Requirement 2.1a (menu-workspace); Requirement 13.4-13.8;
//! menu-workspace Requirement 16.7 (CR-CH-032)

use eframe::egui;

/// Minimum width (px) the calendar column needs to render fully: the
/// `<  Month YYYY  >` header plus the 7-column day grid and the time/day-of-year
/// footer. If the workspace cannot spare this alongside the option list, the
/// calendar is omitted for the frame (menu-workspace Req 16.3, CR-CH-026, B060).
pub(crate) const CALENDAR_MIN_WIDTH: f32 = 180.0;

// CR-CH-032 removed the fixed OPTION_LIST_MIN_WIDTH reserve: the option-list
// width is now driven by the descriptions' natural one-line width
// (`natural_option_list_width`), not a hard minimum. The calendar-fit decision
// (Tier 1) uses that natural width instead of a constant floor.

/// Horizontal gap (px) between the option list and the calendar column.
pub(crate) const CALENDAR_GAP: f32 = 32.0;

/// The semantic colours the shared menu renderer uses for the option columns
/// and the calendar panel. The caller resolves these from the active palette
/// (Legacy uses ISPF semantics; other themes inherit egui colours via
/// PLACEHOLDER). Mirrors the POM's colour scheme so every menu -- POM, Settings
/// and any other -- shares one look.
///
/// Validates: menu-workspace Requirement 2.1a, 2.1b; Requirement 13.4-13.8
#[derive(Debug, Clone, Copy)]
pub struct MenuColours {
    /// Option key column text (White in Legacy).
    pub option_key: egui::Color32,
    /// Option command column text (Turquoise in Legacy).
    pub option_command: egui::Color32,
    /// Option description column text (Green in Legacy).
    pub description: egui::Color32,
    /// Calendar body text colour (Turquoise in Legacy).
    pub calendar_fg: egui::Color32,
    /// Today-cell background (when `use_today_reverse`).
    pub today_bg: egui::Color32,
    /// Today-cell foreground (when `use_today_reverse`).
    pub today_fg: egui::Color32,
    /// Whether to draw today's cell reversed (Legacy) vs a plain marker.
    pub use_today_reverse: bool,
}

impl Default for MenuColours {
    fn default() -> Self {
        Self {
            option_key: egui::Color32::PLACEHOLDER,
            option_command: egui::Color32::PLACEHOLDER,
            description: egui::Color32::PLACEHOLDER,
            calendar_fg: egui::Color32::PLACEHOLDER,
            today_bg: egui::Color32::PLACEHOLDER,
            today_fg: egui::Color32::PLACEHOLDER,
            use_today_reverse: false,
        }
    }
}

/// Width of the command column: the widest command among the options, clamped
/// to a sane range so descriptions align without the column dominating.
///
/// Validates: Requirement 2.1a (menu-workspace)
pub(crate) fn command_column_width(options: &[super::MenuOption]) -> usize {
    options
        .iter()
        .map(|o| o.command.len())
        .max()
        .unwrap_or(0)
        .clamp(4, 24)
}

/// Format one option as three aligned columns: key | command | description.
/// This plain-text form is the alignment reference for `option_row_job` (which
/// produces the same layout with per-column colours) and is exercised by tests.
///
/// Validates: Requirement 2.1a (menu-workspace)
#[cfg(test)]
pub(crate) fn format_option_row(option: &super::MenuOption, cmd_width: usize) -> String {
    format!(
        "{:<4}  {:<width$}  {}",
        option.key,
        option.command,
        option.description,
        width = cmd_width,
    )
}

/// The monospace font used for menu option rows.
pub(crate) fn option_font() -> egui::FontId {
    egui::FontId::monospace(14.0)
}

/// Build the NON-WRAPPING key+command prefix as a two-colour monospace
/// `LayoutJob`: the key column (left-justified in 4) + gutter in `key_col`, then
/// the command column (left-justified in `cmd_width`) + gutter in `command_col`.
/// This is the fixed left part of an option row; the description is laid out
/// SEPARATELY (B065) so wrapped description lines hang-indent under the
/// description column rather than the row's left edge.
///
/// `command_text` is the DISPLAYED command-column text (CR-NR-109 Task 11.5: the
/// active-locale alias surface for the option's root, resolved shell-side), which
/// at `en` / no overlay is byte-identical to `option.command`.
///
/// Validates: Requirement 2.1a; Requirement 13.4, 13.5; localization Req 11.7
pub(crate) fn option_prefix_job(
    option: &super::MenuOption,
    command_text: &str,
    cmd_width: usize,
    key_col: egui::Color32,
    command_col: egui::Color32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let fmt = |color: egui::Color32| egui::TextFormat {
        font_id: option_font(),
        color,
        ..Default::default()
    };
    // Never wrap the fixed columns.
    job.wrap.max_width = f32::INFINITY;
    job.append(&format!("{:<4}", option.key), 0.0, fmt(key_col));
    job.append("  ", 0.0, fmt(key_col));
    job.append(
        &format!("{command_text:<cmd_width$}"),
        0.0,
        fmt(command_col),
    );
    job.append("  ", 0.0, fmt(command_col));
    job
}

/// The prefix TEXT (key + gutter + command + gutter) for one option, matching
/// exactly what `option_prefix_job` paints. Kept separate so the natural-width
/// measurement lays out the same string the renderer draws. `command_text` is
/// the displayed command-column text (CR-NR-109 Task 11.5).
pub(crate) fn option_prefix_text(
    option: &super::MenuOption,
    command_text: &str,
    cmd_width: usize,
) -> String {
    format!("{:<4}  {command_text:<cmd_width$}  ", option.key)
}

/// Reduce per-row natural widths to the option list's Natural_Option_Width: the
/// widest row (prefix + single-line description) plus a scrollbar allowance so
/// the calendar-fit decision leaves room for the option-list vertical scrollbar
/// when one is shown. Pure and deterministic (no `Ui`), so it is unit-testable.
///
/// `row_widths` yields each row's full single-line width in px (prefix galley
/// width + description galley width). Empty -> just the scrollbar allowance.
///
/// Validates: menu-workspace Requirement 16.7 (CR-CH-032)
pub(crate) fn natural_width_from_rows(
    row_widths: impl IntoIterator<Item = f32>,
    scrollbar_allowance: f32,
) -> f32 {
    let widest = row_widths
        .into_iter()
        .fold(0.0_f32, |acc, w| if w > acc { w } else { acc });
    widest + scrollbar_allowance
}

/// Compute the option list's Natural_Option_Width in px: the width needed to
/// render the WIDEST option row (fixed key+command prefix + its single-line
/// description) with NO wrapping, plus an allowance for the option-list vertical
/// scrollbar. Uncapped -- a very long description simply increases it (which may
/// drop the layout to Tier 2/3; menu-workspace Req 16.7, 16.8).
///
/// Measures against the live `ui` fonts so the result matches what the renderer
/// paints (the prefix and description both use `option_font()`). `command_texts`
/// supplies each option's DISPLAYED command-column text (CR-NR-109 Task 11.5),
/// positionally aligned with `options`; a short/absent slice falls back to
/// `option.command` so the measurement never panics.
///
/// Validates: menu-workspace Requirement 16.7 (CR-CH-032)
pub(crate) fn natural_option_list_width(
    options: &[super::MenuOption],
    command_texts: &[String],
    cmd_width: usize,
    ui: &egui::Ui,
) -> f32 {
    let font = option_font();
    let measure = |text: &str| -> f32 {
        let mut job = egui::text::LayoutJob::simple_singleline(
            text.to_string(),
            font.clone(),
            egui::Color32::WHITE,
        );
        job.wrap.max_width = f32::INFINITY;
        ui.painter().layout_job(job).size().x
    };
    let row_widths = options.iter().enumerate().map(|(i, o)| {
        let command_text = command_texts
            .get(i)
            .map(String::as_str)
            .unwrap_or(&o.command);
        let prefix_w = measure(&option_prefix_text(o, command_text, cmd_width));
        let desc_w = measure(&o.description);
        prefix_w + desc_w
    });
    // Scrollbar allowance: the theme's scrollbar width plus a small pad, so the
    // Tier-1 calendar fit accounts for a scrollbar appearing on the option list.
    let scrollbar_allowance = ui.spacing().scroll.bar_width + 4.0;
    natural_width_from_rows(row_widths, scrollbar_allowance)
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_workspace::MenuOption;

    fn opt(key: &str, command: &str, description: &str) -> MenuOption {
        MenuOption {
            key: key.to_string(),
            command: command.to_string(),
            description: description.to_string(),
            enabled: true,
            group: None,
            show_in_menu_bar: true,
            target: None,
        }
    }

    // Validates: Requirement 2.1a -- three aligned columns key | command | description
    #[test]
    fn format_option_row_produces_three_aligned_columns() {
        let o = opt("1", "FILES", "Browse files");
        let row = format_option_row(&o, 8);
        // key left-padded to 4, then command left-padded to width 8, then desc.
        assert_eq!(row, "1     FILES     Browse files");
        // Contains all three fields.
        assert!(row.contains("FILES"));
        assert!(row.contains("Browse files"));
        assert!(row.trim_start().starts_with('1'));
    }

    // Validates: Requirement 2.1a -- command column widens to the widest command, clamped
    #[test]
    fn command_column_width_uses_widest_command_clamped() {
        // Empty -> minimum 4.
        assert_eq!(command_column_width(&[]), 4);
        // Short command still clamps up to 4.
        assert_eq!(command_column_width(&[opt("1", "GO", "x")]), 4);
        // Widest command wins.
        let opts = vec![opt("1", "FILES", "a"), opt("2", "SETTINGS", "b")];
        assert_eq!(command_column_width(&opts), "SETTINGS".len());
        // Very long command clamps to 24.
        let long = "X".repeat(40);
        assert_eq!(command_column_width(&[opt("1", &long, "b")]), 24);
    }

    // Validates: Requirement 2.1a -- rows sharing one column width align on the description
    #[test]
    fn format_option_rows_align_descriptions_at_common_width() {
        let opts = vec![opt("1", "FILES", "Browse"), opt("2", "SETTINGS", "Config")];
        let w = command_column_width(&opts);
        let r0 = format_option_row(&opts[0], w);
        let r1 = format_option_row(&opts[1], w);
        // The description starts at the same character index in both rows.
        let idx0 = r0.find("Browse").unwrap();
        let idx1 = r1.find("Config").unwrap();
        assert_eq!(idx0, idx1, "descriptions must align across rows");
    }

    // Validates: Requirement 2.1a, 2.1b -- menu colours default to inherited placeholders
    #[test]
    fn menu_colours_default_is_inherited() {
        let c = MenuColours::default();
        assert_eq!(c.option_key, egui::Color32::PLACEHOLDER);
        assert_eq!(c.option_command, egui::Color32::PLACEHOLDER);
        assert_eq!(c.description, egui::Color32::PLACEHOLDER);
        assert_eq!(c.calendar_fg, egui::Color32::PLACEHOLDER);
        assert_eq!(c.today_bg, egui::Color32::PLACEHOLDER);
        assert_eq!(c.today_fg, egui::Color32::PLACEHOLDER);
        assert!(!c.use_today_reverse);
    }

    // Validates: Requirement 2.1a (B065) -- the key+command PREFIX job holds the
    // fixed columns (key left-justified in 4 + 2 gutter, command in cmd_width +
    // 2 gutter) and NEVER wraps. The description is a SEPARATE wrapping Label
    // (hanging indent under the description column), so it is not part of the
    // prefix job -- the wrap alignment itself is a rendered-widget property
    // (verified manually / by egui layout, test-plan 2.2c).
    #[test]
    fn option_prefix_job_holds_fixed_columns_and_does_not_wrap() {
        let o = opt("1", "FILES", "Browse the catalog");
        let w = command_column_width(std::slice::from_ref(&o));
        let job = option_prefix_job(
            &o,
            &o.command,
            w,
            egui::Color32::WHITE,
            egui::Color32::WHITE,
        );
        // The prefix text is exactly key(4) + "  " + command(w) + "  " -- i.e.
        // the format_option_row prefix WITHOUT the description.
        let expected_prefix = &format_option_row(&o, w)[..(4 + 2 + w + 2)];
        assert_eq!(job.text, expected_prefix);
        // The prefix never wraps.
        assert_eq!(job.wrap.max_width, f32::INFINITY);
        // No newlines in the prefix (single physical line).
        assert!(!job.text.contains('\n'));
    }

    // Validates: Requirement 2.1a -- prefixes of different options share the same
    // column width, so descriptions (rendered to the right of the prefix) all
    // start at the SAME x-offset (the description column).
    #[test]
    fn option_prefix_jobs_share_width_across_rows() {
        let opts = vec![opt("1", "FILES", "Browse"), opt("22", "SETTINGS", "Config")];
        let w = command_column_width(&opts);
        let a = option_prefix_job(
            &opts[0],
            &opts[0].command,
            w,
            egui::Color32::WHITE,
            egui::Color32::WHITE,
        );
        let b = option_prefix_job(
            &opts[1],
            &opts[1].command,
            w,
            egui::Color32::WHITE,
            egui::Color32::WHITE,
        );
        // Same character length -> same monospace width -> descriptions align.
        assert_eq!(a.text.chars().count(), b.text.chars().count());
    }

    // === CR-CH-032: description-driven layout natural width (Req 16.7) =====

    // Validates: menu-workspace Requirement 16.7 -- the prefix TEXT used for the
    // natural-width measurement matches the painted prefix job (key(4) + gutter +
    // command(cmd_width) + gutter), so measuring it reproduces the drawn layout.
    #[test]
    fn option_prefix_text_matches_prefix_job_text() {
        let o = opt("1", "FILES", "Browse the catalog");
        let w = command_column_width(std::slice::from_ref(&o));
        let job = option_prefix_job(
            &o,
            &o.command,
            w,
            egui::Color32::WHITE,
            egui::Color32::WHITE,
        );
        assert_eq!(option_prefix_text(&o, &o.command, w), job.text);
    }

    // Validates: menu-workspace Requirement 16.7 -- Natural_Option_Width is the
    // WIDEST row plus the scrollbar allowance.
    #[test]
    fn natural_width_is_widest_row_plus_scrollbar() {
        // Rows of widths 100, 250, 180 -> widest 250 + allowance 12 = 262.
        let w = natural_width_from_rows([100.0_f32, 250.0, 180.0], 12.0);
        assert_eq!(w, 262.0);
    }

    // Validates: menu-workspace Requirement 16.7 -- a longer single-line
    // description increases the natural width (uncapped).
    #[test]
    fn natural_width_grows_with_longer_description() {
        let short = natural_width_from_rows([120.0_f32], 10.0);
        let long = natural_width_from_rows([120.0_f32, 400.0], 10.0);
        assert!(
            long > short,
            "a wider row must increase the natural width (uncapped): {long} !> {short}"
        );
        assert_eq!(long, 410.0);
    }

    // Validates: menu-workspace Requirement 16.7 -- empty option list yields just
    // the scrollbar allowance (no rows to measure).
    #[test]
    fn natural_width_empty_is_scrollbar_allowance_only() {
        let w = natural_width_from_rows(std::iter::empty(), 15.0);
        assert_eq!(w, 15.0);
    }
}
