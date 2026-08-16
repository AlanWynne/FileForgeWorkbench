//! Filesystem-safe name encoding for DSN paths.
//!
//! Handles percent-encoding of national characters (`@`, `#`, `$`) in
//! physical directory names for cross-platform compatibility.

use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};

use crate::dsn::Dsn;

/// The set of characters that need percent-encoding in filesystem paths.
/// National characters @, #, $ are encoded for cross-platform safety.
const ENCODE_SET: &AsciiSet = &CONTROLS.add(b'@').add(b'#').add(b'$');

/// Convert a DSN to a filesystem-safe relative storage path.
///
/// Dots become directory separators (using forward slash), and national
/// characters are percent-encoded.
///
/// # Examples
///
/// ```
/// use ff_dscatalog::encoding::dsn_to_storage_path;
/// use ff_dscatalog::dsn::Dsn;
///
/// let dsn = Dsn::parse("PAYROLL.INPUT.FILE").unwrap();
/// assert_eq!(dsn_to_storage_path(&dsn), "PAYROLL/INPUT/FILE");
///
/// let dsn = Dsn::parse("#TEMP.$DATA").unwrap();
/// assert_eq!(dsn_to_storage_path(&dsn), "%23TEMP/%24DATA");
/// ```
pub fn dsn_to_storage_path(dsn: &Dsn) -> String {
    dsn.qualifiers()
        .iter()
        .map(|q| utf8_percent_encode(q, ENCODE_SET).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

/// Convert a filesystem path back to a DSN string.
///
/// Reverses the encoding: directory separators become dots, and
/// percent-encoded characters are decoded.
///
/// # Examples
///
/// ```
/// use ff_dscatalog::encoding::path_to_dsn_string;
///
/// assert_eq!(path_to_dsn_string("PAYROLL/INPUT/FILE"), "PAYROLL.INPUT.FILE");
/// assert_eq!(path_to_dsn_string("%23TEMP/%24DATA"), "#TEMP.$DATA");
/// ```
pub fn path_to_dsn_string(path: &str) -> String {
    // Normalize both forward and backward slashes
    let normalized = path.replace('\\', "/");
    normalized
        .split('/')
        .map(|segment| percent_decode_str(segment).decode_utf8_lossy().to_string())
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_dsn_to_path() {
        // Validates: Requirement 4 AC 2
        let dsn = Dsn::parse("PAYROLL.INPUT.FILE").unwrap();
        assert_eq!(dsn_to_storage_path(&dsn), "PAYROLL/INPUT/FILE");
    }

    #[test]
    fn national_chars_encoded() {
        // Validates: Requirement 4 AC 7
        let dsn = Dsn::parse("#TEMP.$DATA.@USER").unwrap();
        let path = dsn_to_storage_path(&dsn);
        assert_eq!(path, "%23TEMP/%24DATA/%40USER");
    }

    #[test]
    fn single_qualifier_dsn() {
        let dsn = Dsn::parse("SINGLE").unwrap();
        assert_eq!(dsn_to_storage_path(&dsn), "SINGLE");
    }

    #[test]
    fn path_to_dsn_round_trip() {
        // Validates: Requirement 4 AC 5
        let dsn = Dsn::parse("PAYROLL.INPUT.FILE").unwrap();
        let path = dsn_to_storage_path(&dsn);
        let recovered = path_to_dsn_string(&path);
        assert_eq!(recovered, dsn.as_str());
    }

    #[test]
    fn national_chars_round_trip() {
        // Validates: Requirement 4 AC 7
        let dsn = Dsn::parse("#TEMP.$DATA.@USER").unwrap();
        let path = dsn_to_storage_path(&dsn);
        let recovered = path_to_dsn_string(&path);
        assert_eq!(recovered, dsn.as_str());
    }

    #[test]
    fn path_with_backslashes_normalized() {
        let result = path_to_dsn_string("PAYROLL\\INPUT\\FILE");
        assert_eq!(result, "PAYROLL.INPUT.FILE");
    }
}
