//! SDSF SET P2 commands and session persistence.
//!
//! Implements Requirement 18 AC 18.22-18.30:
//!   - SET BCOLOR <color>  (AC 18.22)
//!   - SET CONFIRM ON/OFF  (AC 18.23)
//!   - SET CURSOR <field>  (AC 18.24)
//!   - SET DATE <format>   (AC 18.25)
//!   - SET DELAY <seconds> (AC 18.26)
//!   - SET HEX ON/OFF      (AC 18.27)
//!   - SET SCHARS <chars>  (AC 18.28)
//!   - SET SCREEN <rows> <cols> (AC 18.29)
//!   - Persistence across sessions (AC 18.30)

// === DateFormat ==============================================================

/// Date display format for SDSF date columns.
///
/// Addresses: Requirement 18 AC 18.25
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DateFormat {
    /// Month/Day/Year (default).
    #[default]
    Mdy,
    /// Day/Month/Year.
    Dmy,
    /// Year/Month/Day.
    Ymd,
    /// Julian (year + day-of-year).
    Jul,
}

impl DateFormat {
    /// Parse a date format string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "MDY" => Some(Self::Mdy),
            "DMY" => Some(Self::Dmy),
            "YMD" => Some(Self::Ymd),
            "JUL" => Some(Self::Jul),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mdy => "MDY",
            Self::Dmy => "DMY",
            Self::Ymd => "YMD",
            Self::Jul => "JUL",
        }
    }
}

// === SetP2Settings ===========================================================

/// All SET P2 command settings, persisted across sessions.
///
/// Addresses: Requirement 18 AC 18.22-18.30
#[derive(Debug, Clone)]
pub struct SetP2Settings {
    /// Panel background colour name (AC 18.22).
    pub bcolor: String,
    /// Whether destructive actions require confirmation (AC 18.23).
    pub confirm: bool,
    /// Default cursor landing field on panel open (AC 18.24).
    pub cursor_field: String,
    /// Date display format (AC 18.25).
    pub date_format: DateFormat,
    /// Auto-refresh interval in seconds; 0 = disabled (AC 18.26).
    pub delay_seconds: u32,
    /// Whether hex display is active (AC 18.27).
    pub hex: bool,
    /// Special characters for field delimiters (AC 18.28).
    pub schars: String,
    /// Logical screen rows (AC 18.29).
    pub screen_rows: u32,
    /// Logical screen columns (AC 18.29).
    pub screen_cols: u32,
}

impl Default for SetP2Settings {
    fn default() -> Self {
        Self {
            bcolor: "DEFAULT".to_string(),
            confirm: true,
            cursor_field: String::new(),
            date_format: DateFormat::default(),
            delay_seconds: 2,
            hex: false,
            schars: " ,;".to_string(),
            screen_rows: 24,
            screen_cols: 80,
        }
    }
}

// === SetP2Command ============================================================

/// A parsed SET P2 command.
#[derive(Debug, Clone, PartialEq)]
pub enum SetP2Command {
    BColor(String),
    Confirm(bool),
    Cursor(String),
    Date(DateFormat),
    Delay(u32),
    Hex(bool),
    Schars(String),
    Screen { rows: u32, cols: u32 },
}

/// Result of applying a SET P2 command.
#[derive(Debug, Clone, PartialEq)]
pub enum SetP2Result {
    Applied(SetP2Command),
    InvalidValue(String),
    UnknownCommand(String),
}

impl SetP2Settings {
    /// Parse and apply a SET P2 command string.
    ///
    /// Input format: `SET <keyword> <value>` or `<keyword> <value>` (SET already stripped).
    ///
    /// Addresses: Requirement 18 AC 18.22-18.29
    pub fn apply(&mut self, input: &str) -> SetP2Result {
        let trimmed = input.trim();
        // Strip leading SET if present
        let rest = if trimmed.to_uppercase().starts_with("SET ") {
            trimmed[4..].trim()
        } else {
            trimmed
        };

        let mut parts = rest.splitn(2, char::is_whitespace);
        let keyword = parts.next().unwrap_or("").to_uppercase();
        let value = parts.next().unwrap_or("").trim();

        match keyword.as_str() {
            "BCOLOR" => {
                if value.is_empty() {
                    return SetP2Result::InvalidValue("BCOLOR requires a colour name".to_string());
                }
                self.bcolor = value.to_uppercase();
                SetP2Result::Applied(SetP2Command::BColor(self.bcolor.clone()))
            }
            "CONFIRM" => match value.to_uppercase().as_str() {
                "ON" => {
                    self.confirm = true;
                    SetP2Result::Applied(SetP2Command::Confirm(true))
                }
                "OFF" => {
                    self.confirm = false;
                    SetP2Result::Applied(SetP2Command::Confirm(false))
                }
                _ => SetP2Result::InvalidValue(format!("CONFIRM requires ON or OFF, got: {value}")),
            },
            "CURSOR" => {
                self.cursor_field = value.to_uppercase();
                SetP2Result::Applied(SetP2Command::Cursor(self.cursor_field.clone()))
            }
            "DATE" => match DateFormat::parse(value) {
                Some(fmt) => {
                    self.date_format = fmt.clone();
                    SetP2Result::Applied(SetP2Command::Date(fmt))
                }
                None => SetP2Result::InvalidValue(format!(
                    "DATE requires MDY, DMY, YMD, or JUL, got: {value}"
                )),
            },
            "DELAY" => match value.parse::<u32>() {
                Ok(n) => {
                    self.delay_seconds = n;
                    SetP2Result::Applied(SetP2Command::Delay(n))
                }
                Err(_) => SetP2Result::InvalidValue(format!(
                    "DELAY requires a non-negative integer, got: {value}"
                )),
            },
            "HEX" => match value.to_uppercase().as_str() {
                "ON" => {
                    self.hex = true;
                    SetP2Result::Applied(SetP2Command::Hex(true))
                }
                "OFF" => {
                    self.hex = false;
                    SetP2Result::Applied(SetP2Command::Hex(false))
                }
                _ => SetP2Result::InvalidValue(format!("HEX requires ON or OFF, got: {value}")),
            },
            "SCHARS" => {
                self.schars = value.to_string();
                SetP2Result::Applied(SetP2Command::Schars(self.schars.clone()))
            }
            "SCREEN" => {
                let mut dims = value.split_whitespace();
                let rows: u32 = match dims.next().and_then(|s| s.parse().ok()) {
                    Some(n) => n,
                    None => {
                        return SetP2Result::InvalidValue(
                            "SCREEN requires <rows> <cols>".to_string(),
                        )
                    }
                };
                let cols: u32 = match dims.next().and_then(|s| s.parse().ok()) {
                    Some(n) => n,
                    None => {
                        return SetP2Result::InvalidValue(
                            "SCREEN requires <rows> <cols>".to_string(),
                        )
                    }
                };
                self.screen_rows = rows;
                self.screen_cols = cols;
                SetP2Result::Applied(SetP2Command::Screen { rows, cols })
            }
            other => SetP2Result::UnknownCommand(other.to_string()),
        }
    }

    /// Serialize settings to a simple key=value string for session persistence.
    ///
    /// Addresses: Requirement 18 AC 18.30
    pub fn serialize(&self) -> String {
        format!(
            "bcolor={}\nconfirm={}\ncursor={}\ndate={}\ndelay={}\nhex={}\nschars={}\nscreen={}x{}",
            self.bcolor,
            if self.confirm { "ON" } else { "OFF" },
            self.cursor_field,
            self.date_format.as_str(),
            self.delay_seconds,
            if self.hex { "ON" } else { "OFF" },
            self.schars,
            self.screen_rows,
            self.screen_cols,
        )
    }

    /// Deserialize settings from a serialized string.
    ///
    /// Addresses: Requirement 18 AC 18.30
    pub fn deserialize(s: &str) -> Self {
        let mut settings = Self::default();
        for line in s.lines() {
            let mut kv = line.splitn(2, '=');
            let key = kv.next().unwrap_or("").trim();
            let val = kv.next().unwrap_or("").trim();
            match key {
                "bcolor" => settings.bcolor = val.to_string(),
                "confirm" => settings.confirm = val.eq_ignore_ascii_case("ON"),
                "cursor" => settings.cursor_field = val.to_string(),
                "date" => {
                    if let Some(fmt) = DateFormat::parse(val) {
                        settings.date_format = fmt;
                    }
                }
                "delay" => {
                    if let Ok(n) = val.parse() {
                        settings.delay_seconds = n;
                    }
                }
                "hex" => settings.hex = val.eq_ignore_ascii_case("ON"),
                "schars" => settings.schars = val.to_string(),
                "screen" => {
                    let mut dims = val.splitn(2, 'x');
                    if let (Some(r), Some(c)) = (dims.next(), dims.next()) {
                        if let (Ok(rows), Ok(cols)) = (r.parse(), c.parse()) {
                            settings.screen_rows = rows;
                            settings.screen_cols = cols;
                        }
                    }
                }
                _ => {}
            }
        }
        settings
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> SetP2Settings {
        SetP2Settings::default()
    }

    // --- DateFormat ---------------------------------------------------------

    // Validates: Requirement 18.25
    #[test]
    fn date_format_parse_all_variants() {
        assert_eq!(DateFormat::parse("MDY"), Some(DateFormat::Mdy));
        assert_eq!(DateFormat::parse("DMY"), Some(DateFormat::Dmy));
        assert_eq!(DateFormat::parse("YMD"), Some(DateFormat::Ymd));
        assert_eq!(DateFormat::parse("JUL"), Some(DateFormat::Jul));
    }

    // Validates: Requirement 18.25
    #[test]
    fn date_format_parse_case_insensitive() {
        assert_eq!(DateFormat::parse("mdy"), Some(DateFormat::Mdy));
    }

    // Validates: Requirement 18.25
    #[test]
    fn date_format_parse_invalid_returns_none() {
        assert_eq!(DateFormat::parse("BOGUS"), None);
    }

    // --- Default values -----------------------------------------------------

    // Validates: Requirement 18.22-18.29
    #[test]
    fn default_settings_have_expected_values() {
        let s = settings();
        assert_eq!(s.bcolor, "DEFAULT");
        assert!(s.confirm);
        assert_eq!(s.date_format, DateFormat::Mdy);
        assert_eq!(s.delay_seconds, 2);
        assert!(!s.hex);
        assert_eq!(s.screen_rows, 24);
        assert_eq!(s.screen_cols, 80);
    }

    // --- SET BCOLOR ---------------------------------------------------------

    // Validates: Requirement 18.22
    #[test]
    fn set_bcolor_updates_colour() {
        let mut s = settings();
        let result = s.apply("BCOLOR BLUE");
        assert_eq!(
            result,
            SetP2Result::Applied(SetP2Command::BColor("BLUE".to_string()))
        );
        assert_eq!(s.bcolor, "BLUE");
    }

    // Validates: Requirement 18.22
    #[test]
    fn set_bcolor_with_set_prefix() {
        let mut s = settings();
        s.apply("SET BCOLOR GREEN");
        assert_eq!(s.bcolor, "GREEN");
    }

    // --- SET CONFIRM --------------------------------------------------------

    // Validates: Requirement 18.23
    #[test]
    fn set_confirm_off_disables_confirmation() {
        let mut s = settings();
        s.apply("CONFIRM OFF");
        assert!(!s.confirm);
    }

    // Validates: Requirement 18.23
    #[test]
    fn set_confirm_on_enables_confirmation() {
        let mut s = settings();
        s.confirm = false;
        s.apply("CONFIRM ON");
        assert!(s.confirm);
    }

    // Validates: Requirement 18.23
    #[test]
    fn set_confirm_invalid_value_returns_error() {
        let mut s = settings();
        let result = s.apply("CONFIRM MAYBE");
        assert!(matches!(result, SetP2Result::InvalidValue(_)));
    }

    // --- SET CURSOR ---------------------------------------------------------

    // Validates: Requirement 18.24
    #[test]
    fn set_cursor_stores_field_name() {
        let mut s = settings();
        s.apply("CURSOR JOBNAME");
        assert_eq!(s.cursor_field, "JOBNAME");
    }

    // --- SET DATE -----------------------------------------------------------

    // Validates: Requirement 18.25
    #[test]
    fn set_date_updates_format() {
        let mut s = settings();
        s.apply("DATE YMD");
        assert_eq!(s.date_format, DateFormat::Ymd);
    }

    // Validates: Requirement 18.25
    #[test]
    fn set_date_invalid_format_returns_error() {
        let mut s = settings();
        let result = s.apply("DATE BOGUS");
        assert!(matches!(result, SetP2Result::InvalidValue(_)));
    }

    // --- SET DELAY ----------------------------------------------------------

    // Validates: Requirement 18.26
    #[test]
    fn set_delay_updates_interval() {
        let mut s = settings();
        s.apply("DELAY 5");
        assert_eq!(s.delay_seconds, 5);
    }

    // Validates: Requirement 18.26
    #[test]
    fn set_delay_zero_disables_refresh() {
        let mut s = settings();
        s.apply("DELAY 0");
        assert_eq!(s.delay_seconds, 0);
    }

    // Validates: Requirement 18.26
    #[test]
    fn set_delay_invalid_returns_error() {
        let mut s = settings();
        let result = s.apply("DELAY FAST");
        assert!(matches!(result, SetP2Result::InvalidValue(_)));
    }

    // --- SET HEX ------------------------------------------------------------

    // Validates: Requirement 18.27
    #[test]
    fn set_hex_on_enables_hex_display() {
        let mut s = settings();
        s.apply("HEX ON");
        assert!(s.hex);
    }

    // Validates: Requirement 18.27
    #[test]
    fn set_hex_off_disables_hex_display() {
        let mut s = settings();
        s.hex = true;
        s.apply("HEX OFF");
        assert!(!s.hex);
    }

    // --- SET SCHARS ---------------------------------------------------------

    // Validates: Requirement 18.28
    #[test]
    fn set_schars_updates_delimiter_chars() {
        let mut s = settings();
        s.apply("SCHARS ,;:");
        assert_eq!(s.schars, ",;:");
    }

    // --- SET SCREEN ---------------------------------------------------------

    // Validates: Requirement 18.29
    #[test]
    fn set_screen_updates_dimensions() {
        let mut s = settings();
        s.apply("SCREEN 43 132");
        assert_eq!(s.screen_rows, 43);
        assert_eq!(s.screen_cols, 132);
    }

    // Validates: Requirement 18.29
    #[test]
    fn set_screen_missing_cols_returns_error() {
        let mut s = settings();
        let result = s.apply("SCREEN 43");
        assert!(matches!(result, SetP2Result::InvalidValue(_)));
    }

    // --- Unknown command ----------------------------------------------------

    #[test]
    fn unknown_set_command_returns_unknown() {
        let mut s = settings();
        let result = s.apply("BOGUSCMD value");
        assert!(matches!(result, SetP2Result::UnknownCommand(_)));
    }

    // --- Persistence --------------------------------------------------------

    // Validates: Requirement 18.30
    #[test]
    fn settings_round_trip_through_serialize_deserialize() {
        let mut original = settings();
        original.apply("BCOLOR BLUE");
        original.apply("CONFIRM OFF");
        original.apply("DATE YMD");
        original.apply("DELAY 10");
        original.apply("HEX ON");
        original.apply("SCREEN 43 132");

        let serialized = original.serialize();
        let restored = SetP2Settings::deserialize(&serialized);

        assert_eq!(restored.bcolor, "BLUE");
        assert!(!restored.confirm);
        assert_eq!(restored.date_format, DateFormat::Ymd);
        assert_eq!(restored.delay_seconds, 10);
        assert!(restored.hex);
        assert_eq!(restored.screen_rows, 43);
        assert_eq!(restored.screen_cols, 132);
    }

    // Validates: Requirement 18.30
    #[test]
    fn deserialize_empty_string_returns_defaults() {
        let s = SetP2Settings::deserialize("");
        assert_eq!(s.bcolor, "DEFAULT");
        assert!(s.confirm);
        assert_eq!(s.date_format, DateFormat::Mdy);
    }

    // Validates: Requirement 18.30
    #[test]
    fn serialize_contains_all_keys() {
        let s = settings();
        let serialized = s.serialize();
        assert!(serialized.contains("bcolor="));
        assert!(serialized.contains("confirm="));
        assert!(serialized.contains("date="));
        assert!(serialized.contains("delay="));
        assert!(serialized.contains("hex="));
        assert!(serialized.contains("schars="));
        assert!(serialized.contains("screen="));
    }
}
