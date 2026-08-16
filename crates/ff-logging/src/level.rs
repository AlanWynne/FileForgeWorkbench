//! Log level enum, ordering, and parsing.
//!
//! Defines the five severity levels (Trace, Debug, Info, Warn, Error) with
//! comparison operators and case-insensitive string parsing.

use std::fmt;
use std::str::FromStr;

/// Severity levels for log records, in ascending order.
///
/// Level ordering: `Trace < Debug < Info < Warn < Error`.
/// A record passes the level filter if its level is >= the configured minimum.
///
/// # Examples
///
/// ```
/// use ff_logging::LogLevel;
///
/// assert!(LogLevel::Error > LogLevel::Warn);
/// assert_eq!(LogLevel::Info.as_str(), "INFO");
/// assert_eq!(LogLevel::Info.as_u8(), 2);
/// assert_eq!(LogLevel::from_u8(3), Some(LogLevel::Warn));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LogLevel {
    /// Finest-grained diagnostic information.
    Trace = 0,
    /// Detailed diagnostic information for debugging.
    Debug = 1,
    /// General informational messages about application progress.
    Info = 2,
    /// Potentially harmful situations that deserve attention.
    Warn = 3,
    /// Error conditions that may allow the application to continue.
    Error = 4,
}

impl LogLevel {
    /// Returns the uppercase display name of the level as a static string slice.
    ///
    /// The returned name is always uppercase and suitable for log output formatting.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_logging::LogLevel;
    ///
    /// assert_eq!(LogLevel::Trace.as_str(), "TRACE");
    /// assert_eq!(LogLevel::Info.as_str(), "INFO");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// Returns the numeric value of the level as a `u8`.
    ///
    /// Useful for atomic operations where the level must be stored as an integer.
    /// The values are: Trace=0, Debug=1, Info=2, Warn=3, Error=4.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_logging::LogLevel;
    ///
    /// assert_eq!(LogLevel::Trace.as_u8(), 0);
    /// assert_eq!(LogLevel::Error.as_u8(), 4);
    /// ```
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// Converts a numeric value back to a `LogLevel`, returning `None` if the
    /// value does not correspond to a valid level.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_logging::LogLevel;
    ///
    /// assert_eq!(LogLevel::from_u8(0), Some(LogLevel::Trace));
    /// assert_eq!(LogLevel::from_u8(4), Some(LogLevel::Error));
    /// assert_eq!(LogLevel::from_u8(5), None);
    /// ```
    pub fn from_u8(val: u8) -> Option<LogLevel> {
        match val {
            0 => Some(LogLevel::Trace),
            1 => Some(LogLevel::Debug),
            2 => Some(LogLevel::Info),
            3 => Some(LogLevel::Warn),
            4 => Some(LogLevel::Error),
            _ => None,
        }
    }
}

impl LogLevel {
    /// Parses a level string using case-insensitive comparison with leading and
    /// trailing whitespace trimmed.
    ///
    /// Returns `Some(LogLevel)` for recognized level names ("trace", "debug",
    /// "info", "warn", "error") and `None` for anything else. Only exact level
    /// names are accepted — no partial matches or abbreviations.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_logging::LogLevel;
    ///
    /// assert_eq!(LogLevel::from_str_lenient("info"), Some(LogLevel::Info));
    /// assert_eq!(LogLevel::from_str_lenient("  WARN  "), Some(LogLevel::Warn));
    /// assert_eq!(LogLevel::from_str_lenient("tRaCe"), Some(LogLevel::Trace));
    /// assert_eq!(LogLevel::from_str_lenient("unknown"), None);
    /// ```
    pub fn from_str_lenient(s: &str) -> Option<LogLevel> {
        s.parse().ok()
    }
}

impl FromStr for LogLevel {
    type Err = ParseLogLevelError;

    /// Parses a log level from a string with case-insensitive comparison and
    /// whitespace trimming.
    ///
    /// # Errors
    ///
    /// Returns [`ParseLogLevelError`] if the input (after trimming and
    /// lowercasing) does not match one of the five valid level names.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_logging::LogLevel;
    ///
    /// let level: LogLevel = "ERROR".parse().unwrap();
    /// assert_eq!(level, LogLevel::Error);
    ///
    /// let level: LogLevel = "  debug\t".parse().unwrap();
    /// assert_eq!(level, LogLevel::Debug);
    ///
    /// assert!("nope".parse::<LogLevel>().is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(ParseLogLevelError {
                input: s.to_owned(),
            }),
        }
    }
}

/// Error returned when parsing an unrecognized log level string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseLogLevelError {
    /// The original input that could not be parsed.
    pub input: String,
}

impl fmt::Display for ParseLogLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unrecognized log level '{}'; expected one of: trace, debug, info, warn, error",
            self.input
        )
    }
}

impl std::error::Error for ParseLogLevelError {}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── FromStr Tests ──────────────────────────────────────────────────────

    #[test]
    fn from_str_accepts_lowercase_level_names() {
        // Validates: Requirement 3.1
        assert_eq!("trace".parse::<LogLevel>(), Ok(LogLevel::Trace));
        assert_eq!("debug".parse::<LogLevel>(), Ok(LogLevel::Debug));
        assert_eq!("info".parse::<LogLevel>(), Ok(LogLevel::Info));
        assert_eq!("warn".parse::<LogLevel>(), Ok(LogLevel::Warn));
        assert_eq!("error".parse::<LogLevel>(), Ok(LogLevel::Error));
    }

    #[test]
    fn from_str_accepts_uppercase_level_names() {
        // Validates: Requirement 3.1
        assert_eq!("TRACE".parse::<LogLevel>(), Ok(LogLevel::Trace));
        assert_eq!("DEBUG".parse::<LogLevel>(), Ok(LogLevel::Debug));
        assert_eq!("INFO".parse::<LogLevel>(), Ok(LogLevel::Info));
        assert_eq!("WARN".parse::<LogLevel>(), Ok(LogLevel::Warn));
        assert_eq!("ERROR".parse::<LogLevel>(), Ok(LogLevel::Error));
    }

    #[test]
    fn from_str_accepts_mixed_case_level_names() {
        // Validates: Requirement 3.1
        assert_eq!("Trace".parse::<LogLevel>(), Ok(LogLevel::Trace));
        assert_eq!("dEbUg".parse::<LogLevel>(), Ok(LogLevel::Debug));
        assert_eq!("iNfO".parse::<LogLevel>(), Ok(LogLevel::Info));
        assert_eq!("wArN".parse::<LogLevel>(), Ok(LogLevel::Warn));
        assert_eq!("eRrOr".parse::<LogLevel>(), Ok(LogLevel::Error));
    }

    #[test]
    fn from_str_trims_leading_and_trailing_whitespace() {
        // Validates: Requirement 3.1
        assert_eq!("  info  ".parse::<LogLevel>(), Ok(LogLevel::Info));
        assert_eq!("\tWARN\n".parse::<LogLevel>(), Ok(LogLevel::Warn));
        assert_eq!("   trace   ".parse::<LogLevel>(), Ok(LogLevel::Trace));
        assert_eq!("\r\ndebug\r\n".parse::<LogLevel>(), Ok(LogLevel::Debug));
    }

    #[test]
    fn from_str_rejects_unrecognized_values() {
        // Validates: Requirement 3.4
        assert!("".parse::<LogLevel>().is_err());
        assert!("warning".parse::<LogLevel>().is_err());
        assert!("err".parse::<LogLevel>().is_err());
        assert!("information".parse::<LogLevel>().is_err());
        assert!("fatal".parse::<LogLevel>().is_err());
        assert!("INFO!".parse::<LogLevel>().is_err());
        assert!("in fo".parse::<LogLevel>().is_err());
    }

    #[test]
    fn from_str_error_contains_original_input() {
        // Validates: Requirement 3.4
        let err = "nope".parse::<LogLevel>().unwrap_err();
        assert_eq!(err.input, "nope");
        assert!(err.to_string().contains("nope"));
    }

    // ─── from_str_lenient Tests ─────────────────────────────────────────────

    #[test]
    fn from_str_lenient_returns_some_for_valid_levels() {
        // Validates: Requirement 3.1
        assert_eq!(LogLevel::from_str_lenient("trace"), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_str_lenient("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str_lenient("info"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str_lenient("warn"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str_lenient("error"), Some(LogLevel::Error));
    }

    #[test]
    fn from_str_lenient_returns_none_for_invalid_values() {
        // Validates: Requirement 3.4
        assert_eq!(LogLevel::from_str_lenient(""), None);
        assert_eq!(LogLevel::from_str_lenient("unknown"), None);
        assert_eq!(LogLevel::from_str_lenient("inf"), None);
    }

    #[test]
    fn from_str_lenient_is_case_insensitive_with_whitespace_trimming() {
        // Validates: Requirement 3.1
        assert_eq!(
            LogLevel::from_str_lenient("  ERROR  "),
            Some(LogLevel::Error)
        );
        assert_eq!(LogLevel::from_str_lenient("DeBuG"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str_lenient("\twarn\n"), Some(LogLevel::Warn));
    }

    // ─── Round-trip test ────────────────────────────────────────────────────

    #[test]
    fn from_str_round_trips_with_as_str() {
        // Validates: Requirement 3.1
        let levels = [
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ];
        for level in levels {
            let parsed = LogLevel::from_str_lenient(level.as_str());
            assert_eq!(parsed, Some(level));
        }
    }
}
