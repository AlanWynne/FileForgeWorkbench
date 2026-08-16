//! IDC message catalogue and formatting.
//!
//! All IDCAMS messages follow the pattern `IDCnnnnX text` where `nnnn` is a 4-digit
//! message number and `X` is the severity indicator.

use std::fmt;

/// All IDC message codes used by ff-idcams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageCode {
    // ─── Success messages (I) ────────────────────────────────────────────
    /// Dataset/object created successfully.
    IDC0001I,
    /// Dataset/object deleted / final MAXCC summary.
    IDC0002I,
    /// Dataset altered successfully.
    IDC0003I,
    /// Export completed.
    IDC0004I,
    /// Import completed.
    IDC0005I,
    /// BLDINDEX completed.
    IDC0006I,

    // ─── Warning messages (W) ────────────────────────────────────────────
    /// LISTCAT no entries found.
    IDC0565W,
    /// REPRO duplicate key skipped.
    IDC0580W,
    /// BLDINDEX duplicate keys found.
    IDC0622W,
    /// Empty input (no commands).
    IDC0640I,

    // ─── Error messages — parser (E) ─────────────────────────────────────
    /// Invalid/unrecognized command verb.
    IDC0001E,
    /// Malformed parameter syntax.
    IDC0002E,
    /// Invalid IF condition operand.
    IDC0630E,

    // ─── Error messages — execution (E) ──────────────────────────────────
    /// KEYS required for INDEXED.
    IDC0503E,
    /// RELATE base cluster not found.
    IDC0510E,
    /// RELATE target not a VSAM cluster.
    IDC0511E,
    /// PATHENTRY AIX not found.
    IDC0512E,
    /// Duplicate dataset name.
    IDC0514E,
    /// LIMIT required for GDG.
    IDC0520E,
    /// DELETE entry not found.
    IDC0550E,
    /// DELETE type mismatch.
    IDC0551E,
    /// ALTER entry not found.
    IDC0560E,
    /// ALTER attribute not modifiable.
    IDC0561E,
    /// PRINT dataset not found.
    IDC0570E,
    /// PRINT key selection requires KSDS.
    IDC0571E,
    /// REPRO source not found.
    IDC0581E,
    /// REPRO target not found.
    IDC0582E,
    /// VERIFY dataset consistent (info).
    IDC0590I,
    /// VERIFY dataset access failure.
    IDC0591E,
    /// VERIFY non-VSAM dataset.
    IDC0592E,
    /// EXPORT source not found.
    IDC0600E,
    /// EXPORT output write failure.
    IDC0601E,
    /// IMPORT invalid source.
    IDC0610E,
    /// IMPORT target already exists.
    IDC0611E,
    /// BLDINDEX base cluster not found.
    IDC0620E,
    /// BLDINDEX output not a valid AIX.
    IDC0621E,

    // ─── Severe messages (S) ─────────────────────────────────────────────
    /// Rollback partial failure (inconsistency warning).
    IDC0700W,
    /// Rollback failed — manual intervention required.
    IDC0701S,
}

impl MessageCode {
    /// Returns the string representation of this message code (e.g., "IDC0001I").
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IDC0001I => "IDC0001I",
            Self::IDC0002I => "IDC0002I",
            Self::IDC0003I => "IDC0003I",
            Self::IDC0004I => "IDC0004I",
            Self::IDC0005I => "IDC0005I",
            Self::IDC0006I => "IDC0006I",
            Self::IDC0565W => "IDC0565W",
            Self::IDC0580W => "IDC0580W",
            Self::IDC0622W => "IDC0622W",
            Self::IDC0640I => "IDC0640I",
            Self::IDC0001E => "IDC0001E",
            Self::IDC0002E => "IDC0002E",
            Self::IDC0630E => "IDC0630E",
            Self::IDC0503E => "IDC0503E",
            Self::IDC0510E => "IDC0510E",
            Self::IDC0511E => "IDC0511E",
            Self::IDC0512E => "IDC0512E",
            Self::IDC0514E => "IDC0514E",
            Self::IDC0520E => "IDC0520E",
            Self::IDC0550E => "IDC0550E",
            Self::IDC0551E => "IDC0551E",
            Self::IDC0560E => "IDC0560E",
            Self::IDC0561E => "IDC0561E",
            Self::IDC0570E => "IDC0570E",
            Self::IDC0571E => "IDC0571E",
            Self::IDC0581E => "IDC0581E",
            Self::IDC0582E => "IDC0582E",
            Self::IDC0590I => "IDC0590I",
            Self::IDC0591E => "IDC0591E",
            Self::IDC0592E => "IDC0592E",
            Self::IDC0600E => "IDC0600E",
            Self::IDC0601E => "IDC0601E",
            Self::IDC0610E => "IDC0610E",
            Self::IDC0611E => "IDC0611E",
            Self::IDC0620E => "IDC0620E",
            Self::IDC0621E => "IDC0621E",
            Self::IDC0700W => "IDC0700W",
            Self::IDC0701S => "IDC0701S",
        }
    }

    /// Returns the severity of this message code.
    pub fn severity(&self) -> Severity {
        match self {
            Self::IDC0001I
            | Self::IDC0002I
            | Self::IDC0003I
            | Self::IDC0004I
            | Self::IDC0005I
            | Self::IDC0006I
            | Self::IDC0640I
            | Self::IDC0590I => Severity::Informational,

            Self::IDC0565W | Self::IDC0580W | Self::IDC0622W | Self::IDC0700W => Severity::Warning,

            Self::IDC0001E
            | Self::IDC0002E
            | Self::IDC0630E
            | Self::IDC0503E
            | Self::IDC0510E
            | Self::IDC0511E
            | Self::IDC0512E
            | Self::IDC0514E
            | Self::IDC0520E
            | Self::IDC0550E
            | Self::IDC0551E
            | Self::IDC0560E
            | Self::IDC0561E
            | Self::IDC0570E
            | Self::IDC0571E
            | Self::IDC0581E
            | Self::IDC0582E
            | Self::IDC0591E
            | Self::IDC0592E
            | Self::IDC0600E
            | Self::IDC0601E
            | Self::IDC0610E
            | Self::IDC0611E
            | Self::IDC0620E
            | Self::IDC0621E => Severity::Error,

            Self::IDC0701S => Severity::Severe,
        }
    }
}

impl fmt::Display for MessageCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Message severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Informational — operation succeeded.
    Informational,
    /// Warning — operation completed with minor issues.
    Warning,
    /// Error — operation failed.
    Error,
    /// Severe — catastrophic failure.
    Severe,
}

impl Severity {
    /// Returns the single-character severity indicator.
    pub fn indicator(&self) -> char {
        match self {
            Self::Informational => 'I',
            Self::Warning => 'W',
            Self::Error => 'E',
            Self::Severe => 'S',
        }
    }

    /// Returns the default condition code for this severity.
    pub fn default_condition_code(&self) -> ConditionCode {
        match self {
            Self::Informational => ConditionCode::Success,
            Self::Warning => ConditionCode::Warning,
            Self::Error => ConditionCode::Error,
            Self::Severe => ConditionCode::Catastrophic,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.indicator())
    }
}

/// Condition code values matching z/OS semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ConditionCode {
    /// Successful completion.
    Success = 0,
    /// Warning — operation completed with minor issues.
    Warning = 4,
    /// Error — operation failed but processing continues.
    Error = 8,
    /// Severe error — the specific command failed.
    Severe = 12,
    /// Catastrophic error — processing should terminate.
    Catastrophic = 16,
}

impl ConditionCode {
    /// Returns the numeric value of this condition code.
    pub fn value(&self) -> u8 {
        *self as u8
    }

    /// Creates a condition code from a numeric value.
    ///
    /// Values are clamped to the nearest valid code:
    /// 0, 1-3→4, 4, 5-7→8, 8, 9-11→12, 12, 13-15→16, 16+→16
    pub fn from_value(n: u8) -> Self {
        match n {
            0 => Self::Success,
            1..=4 => Self::Warning,
            5..=8 => Self::Error,
            9..=12 => Self::Severe,
            _ => Self::Catastrophic,
        }
    }
}

impl fmt::Display for ConditionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

/// A single output message in IDCnnnnX format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdcamsMessage {
    /// The message code (e.g., IDC0001I).
    pub code: MessageCode,
    /// The severity level.
    pub severity: Severity,
    /// The human-readable message text.
    pub text: String,
    /// The line number in the output stream.
    pub line_number: u32,
}

impl IdcamsMessage {
    /// Creates a new IDCAMS message.
    pub fn new(code: MessageCode, text: impl Into<String>, line_number: u32) -> Self {
        Self {
            severity: code.severity(),
            code,
            text: text.into(),
            line_number,
        }
    }

    /// Formats the message in z/OS IDCAMS format: `IDCnnnnX text`.
    pub fn format(&self) -> String {
        format!("{} {}", self.code, self.text)
    }
}

impl fmt::Display for IdcamsMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.code, self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_code_ordering_is_correct() {
        assert!(ConditionCode::Success < ConditionCode::Warning);
        assert!(ConditionCode::Warning < ConditionCode::Error);
        assert!(ConditionCode::Error < ConditionCode::Severe);
        assert!(ConditionCode::Severe < ConditionCode::Catastrophic);
    }

    #[test]
    fn condition_code_values_match_zos() {
        assert_eq!(ConditionCode::Success.value(), 0);
        assert_eq!(ConditionCode::Warning.value(), 4);
        assert_eq!(ConditionCode::Error.value(), 8);
        assert_eq!(ConditionCode::Severe.value(), 12);
        assert_eq!(ConditionCode::Catastrophic.value(), 16);
    }

    #[test]
    fn condition_code_from_value_clamps_correctly() {
        assert_eq!(ConditionCode::from_value(0), ConditionCode::Success);
        assert_eq!(ConditionCode::from_value(3), ConditionCode::Warning);
        assert_eq!(ConditionCode::from_value(4), ConditionCode::Warning);
        assert_eq!(ConditionCode::from_value(7), ConditionCode::Error);
        assert_eq!(ConditionCode::from_value(8), ConditionCode::Error);
        assert_eq!(ConditionCode::from_value(12), ConditionCode::Severe);
        assert_eq!(ConditionCode::from_value(16), ConditionCode::Catastrophic);
        assert_eq!(ConditionCode::from_value(20), ConditionCode::Catastrophic);
    }

    #[test]
    fn message_code_display_format() {
        assert_eq!(MessageCode::IDC0001I.to_string(), "IDC0001I");
        assert_eq!(MessageCode::IDC0514E.to_string(), "IDC0514E");
        assert_eq!(MessageCode::IDC0701S.to_string(), "IDC0701S");
    }

    #[test]
    fn message_code_severity_mapping() {
        assert_eq!(MessageCode::IDC0001I.severity(), Severity::Informational);
        assert_eq!(MessageCode::IDC0565W.severity(), Severity::Warning);
        assert_eq!(MessageCode::IDC0514E.severity(), Severity::Error);
        assert_eq!(MessageCode::IDC0701S.severity(), Severity::Severe);
    }

    #[test]
    fn severity_default_condition_codes() {
        assert_eq!(
            Severity::Informational.default_condition_code(),
            ConditionCode::Success
        );
        assert_eq!(
            Severity::Warning.default_condition_code(),
            ConditionCode::Warning
        );
        assert_eq!(
            Severity::Error.default_condition_code(),
            ConditionCode::Error
        );
        assert_eq!(
            Severity::Severe.default_condition_code(),
            ConditionCode::Catastrophic
        );
    }

    #[test]
    fn idcams_message_format() {
        let msg = IdcamsMessage::new(MessageCode::IDC0001I, "ENTRY MY.CLUSTER DEFINED", 1);
        assert_eq!(msg.format(), "IDC0001I ENTRY MY.CLUSTER DEFINED");
        assert_eq!(msg.line_number, 1);
    }
}
