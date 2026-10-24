//! Pure calendar-header formatting for the POM / Menu_Workspace calendar.
//!
//! Provides [`centered_calendar_header`], which centers the month+year label in
//! a constant-width monospace field so the `<` and `>` scroll buttons rendered
//! on either side land in the SAME columns for every month of the year
//! (menu-workspace Requirement 15.12, CR-CH-063). The field width is derived
//! from the ACTIVE LOCALE's month names via `ff_i18n::t` (B084), so switching
//! locale re-widens the field with no code change.

/// Number of spaces separating the month name from the year.
const MONTH_YEAR_GAP: usize = 2;

/// Character width of a four-digit year (the calendar only ever shows years in
/// the 1000..=9999 range for which this holds; the pad math is content-driven
/// via `chars().count()` so a wider year simply widens the field).
const YEAR_WIDTH: usize = 4;

/// Longest localized month name (in Unicode scalar values) over months 1..=12,
/// resolved through the active-locale i18n catalogue (keys
/// `ff-desktop-pom-month-01` .. `-12`, B084).
fn max_localized_month_len() -> usize {
    (1..=12)
        .map(|m| {
            let key = format!("ff-desktop-pom-month-{:02}", m);
            ff_i18n::t(&key).chars().count()
        })
        .max()
        .unwrap_or(0)
}

/// Build the centered, constant-width calendar header content for `month_name`
/// and `year`.
///
/// The returned string centers `"{month_name}  {year}"` within a field whose
/// width is `max_localized_month_len() + MONTH_YEAR_GAP + YEAR_WIDTH`. Padding
/// is split as evenly as possible; any odd remainder puts the extra space on
/// the RIGHT (matching the approved sketch where the right pad is >= the left
/// pad). Because the label is rendered monospace, equal character counts give
/// equal pixel widths, so `<` ends and `>` begins at fixed columns for every
/// month.
///
/// Validates: menu-workspace Requirement 15.12
pub fn centered_calendar_header(month_name: &str, year: i32) -> String {
    let content = format!("{}{}{}", month_name, " ".repeat(MONTH_YEAR_GAP), year);
    let field_width = max_localized_month_len() + MONTH_YEAR_GAP + YEAR_WIDTH;
    let content_len = content.chars().count();
    if content_len >= field_width {
        return content;
    }
    let total_pad = field_width - content_len;
    let left_pad = total_pad / 2;
    let right_pad = total_pad - left_pad;
    format!(
        "{}{}{}",
        " ".repeat(left_pad),
        content,
        " ".repeat(right_pad)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Validates: menu-workspace Requirement 15.12
    ///
    /// For month names of differing lengths the produced header strings MUST
    /// all have the SAME character length, and the content MUST be centered
    /// (balanced padding, within 1 char for odd remainders).
    #[test]
    fn centered_header_is_constant_width_and_centered() {
        let year = 2026;
        let names = ["May", "March", "September"];
        let headers: Vec<String> = names
            .iter()
            .map(|n| centered_calendar_header(n, year))
            .collect();

        // All the same character width.
        let widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
        let first = widths[0];
        for (name, w) in names.iter().zip(&widths) {
            assert_eq!(
                *w, first,
                "month '{}' produced width {} (expected {}): {:?}",
                name, w, first, headers
            );
        }

        // Centered: leading and trailing space counts are balanced to within 1
        // (odd remainder on the right), for every produced header.
        for (name, header) in names.iter().zip(&headers) {
            let left = header.chars().take_while(|c| *c == ' ').count();
            let right = header.chars().rev().take_while(|c| *c == ' ').count();
            assert!(
                right >= left && right - left <= 1,
                "month '{}' not centered: left={} right={} header={:?}",
                name,
                left,
                right,
                header
            );
        }
    }
}
