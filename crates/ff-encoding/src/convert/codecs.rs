//! Encoding <-> UTF-8 codec implementations (single-byte, UTF-16, DBCS).
//!
//! Private helpers for the public conversion API in the parent module.

use crate::encoding::Encoding;
use crate::error::EncodingError;

use super::{ConversionIssue, ConversionResult, UnmappableAction};

/// Single-byte to UTF-8 conversion using ISO-8859-1 identity mapping.
pub(super) fn convert_single_byte_to_utf8(
    bytes: &[u8],
    source_encoding: &Encoding,
) -> Result<ConversionResult, EncodingError> {
    let table = get_single_byte_to_unicode_table(source_encoding);
    let mut result = Vec::with_capacity(bytes.len() * 2);
    let mut issues = Vec::new();

    for (offset, &byte) in bytes.iter().enumerate() {
        let cp = if let Some(table) = table {
            table[byte as usize]
        } else {
            // ISO-8859-1 identity mapping
            byte as u32
        };

        if cp == 0xFFFD {
            issues.push(ConversionIssue {
                source_offset: offset,
                original_bytes: vec![byte],
                description: format!("unmappable byte 0x{byte:02X} in {}", source_encoding.name),
            });
        }

        // Encode Unicode code point as UTF-8
        let mut buf = [0u8; 4];
        if let Some(ch) = char::from_u32(cp) {
            let encoded = ch.encode_utf8(&mut buf);
            result.extend_from_slice(encoded.as_bytes());
        } else {
            result.extend_from_slice("\u{FFFD}".as_bytes());
        }
    }

    Ok(ConversionResult {
        data: result,
        issues,
    })
}

/// UTF-16 to UTF-8 conversion.
pub(super) fn convert_utf16_to_utf8(
    bytes: &[u8],
    source_encoding: &Encoding,
) -> Result<ConversionResult, EncodingError> {
    let is_le = source_encoding.name.contains("le") || source_encoding.code_page == 1200;
    let is_utf32 = source_encoding.name.contains("32");
    let mut result = Vec::with_capacity(bytes.len());
    let mut issues = Vec::new();

    if is_utf32 {
        let unit_size = 4;
        let mut offset = 0;
        while offset + unit_size <= bytes.len() {
            let cp = if is_le {
                u32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ])
            } else {
                u32::from_be_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ])
            };

            if let Some(ch) = char::from_u32(cp) {
                let mut buf = [0u8; 4];
                let encoded = ch.encode_utf8(&mut buf);
                result.extend_from_slice(encoded.as_bytes());
            } else {
                result.extend_from_slice("\u{FFFD}".as_bytes());
                issues.push(ConversionIssue {
                    source_offset: offset,
                    original_bytes: bytes[offset..offset + unit_size].to_vec(),
                    description: format!("invalid UTF-32 code point 0x{cp:08X}"),
                });
            }
            offset += unit_size;
        }
    } else {
        // UTF-16
        let mut offset = 0;
        while offset + 2 <= bytes.len() {
            let unit = if is_le {
                u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
            } else {
                u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
            };

            if (0xD800..=0xDBFF).contains(&unit) {
                // High surrogate — need low surrogate
                if offset + 4 <= bytes.len() {
                    let low = if is_le {
                        u16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]])
                    } else {
                        u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]])
                    };
                    if (0xDC00..=0xDFFF).contains(&low) {
                        let cp = 0x10000 + ((unit as u32 - 0xD800) << 10) + (low as u32 - 0xDC00);
                        if let Some(ch) = char::from_u32(cp) {
                            let mut buf = [0u8; 4];
                            let encoded = ch.encode_utf8(&mut buf);
                            result.extend_from_slice(encoded.as_bytes());
                        }
                        offset += 4;
                        continue;
                    }
                }
                // Lone high surrogate
                result.extend_from_slice("\u{FFFD}".as_bytes());
                issues.push(ConversionIssue {
                    source_offset: offset,
                    original_bytes: bytes[offset..offset + 2].to_vec(),
                    description: "lone high surrogate".to_string(),
                });
                offset += 2;
            } else if (0xDC00..=0xDFFF).contains(&unit) {
                // Lone low surrogate
                result.extend_from_slice("\u{FFFD}".as_bytes());
                issues.push(ConversionIssue {
                    source_offset: offset,
                    original_bytes: bytes[offset..offset + 2].to_vec(),
                    description: "lone low surrogate".to_string(),
                });
                offset += 2;
            } else {
                if let Some(ch) = char::from_u32(unit as u32) {
                    let mut buf = [0u8; 4];
                    let encoded = ch.encode_utf8(&mut buf);
                    result.extend_from_slice(encoded.as_bytes());
                }
                offset += 2;
            }
        }
    }

    Ok(ConversionResult {
        data: result,
        issues,
    })
}

/// DBCS to UTF-8 conversion (placeholder — full tables in production).
pub(super) fn convert_dbcs_to_utf8(
    bytes: &[u8],
    source_encoding: &Encoding,
) -> Result<ConversionResult, EncodingError> {
    // Simplified: treat as pass-through with replacement for now
    let mut result = Vec::with_capacity(bytes.len());
    let issues = Vec::new();

    // For Shift-JIS and other DBCS, we need code-page-specific mapping tables.
    // This is a simplified implementation that handles basic ASCII pass-through.
    for &byte in bytes {
        if byte < 0x80 {
            result.push(byte);
        } else {
            // Placeholder: replace high bytes with U+FFFD
            result.extend_from_slice("\u{FFFD}".as_bytes());
        }
    }

    let _ = source_encoding; // Will use for table selection in full impl

    Ok(ConversionResult {
        data: result,
        issues,
    })
}

/// UTF-8 to single-byte conversion.
pub(super) fn convert_utf8_to_single_byte(
    text: &str,
    target_encoding: &Encoding,
    unmappable_action: UnmappableAction,
) -> Result<ConversionResult, EncodingError> {
    let reverse_table = get_unicode_to_single_byte_table(target_encoding);
    let mut result = Vec::with_capacity(text.len());
    let mut issues = Vec::new();

    for (offset, ch) in text.char_indices() {
        let cp = ch as u32;

        let mapped = if let Some(table) = &reverse_table {
            table.get(&cp).copied()
        } else if cp <= 0xFF {
            // ISO-8859-1 identity
            Some(cp as u8)
        } else {
            None
        };

        if let Some(byte) = mapped {
            result.push(byte);
        } else {
            match unmappable_action {
                UnmappableAction::Abort => {
                    return Err(EncodingError::UnmappableCharacter {
                        code_point: cp,
                        offset,
                    });
                }
                UnmappableAction::ReplaceWithPlaceholder(placeholder) => {
                    if (placeholder as u32) <= 0xFF {
                        result.push(placeholder as u8);
                    } else {
                        result.push(b'?');
                    }
                    issues.push(ConversionIssue {
                        source_offset: offset,
                        original_bytes: ch.to_string().into_bytes(),
                        description: format!(
                            "unmappable character U+{cp:04X} replaced with '{placeholder}'"
                        ),
                    });
                }
                UnmappableAction::SwitchToUtf8 => {
                    // Switch to UTF-8 — just encode the whole remaining text as UTF-8
                    result.extend_from_slice(&text.as_bytes()[offset..]);
                    return Ok(ConversionResult {
                        data: result,
                        issues,
                    });
                }
            }
        }
    }

    Ok(ConversionResult {
        data: result,
        issues,
    })
}

/// UTF-8 to UTF-16 conversion.
pub(super) fn convert_utf8_to_utf16(
    text: &str,
    target_encoding: &Encoding,
    _unmappable_action: UnmappableAction,
) -> Result<ConversionResult, EncodingError> {
    let is_le = target_encoding.name.contains("le") || target_encoding.code_page == 1200;
    let is_utf32 = target_encoding.name.contains("32");
    let mut result = Vec::with_capacity(text.len() * 2);

    if is_utf32 {
        for ch in text.chars() {
            let cp = ch as u32;
            if is_le {
                result.extend_from_slice(&cp.to_le_bytes());
            } else {
                result.extend_from_slice(&cp.to_be_bytes());
            }
        }
    } else {
        // UTF-16
        let mut buf = [0u16; 2];
        for ch in text.chars() {
            let encoded = ch.encode_utf16(&mut buf);
            for unit in encoded.iter() {
                if is_le {
                    result.extend_from_slice(&unit.to_le_bytes());
                } else {
                    result.extend_from_slice(&unit.to_be_bytes());
                }
            }
        }
    }

    Ok(ConversionResult {
        data: result,
        issues: Vec::new(),
    })
}

/// UTF-8 to DBCS conversion (placeholder).
pub(super) fn convert_utf8_to_dbcs(
    text: &str,
    _target_encoding: &Encoding,
    _unmappable_action: UnmappableAction,
) -> Result<ConversionResult, EncodingError> {
    // Simplified placeholder: ASCII pass-through, rest replaced
    let mut result = Vec::with_capacity(text.len());
    for ch in text.chars() {
        if ch.is_ascii() {
            result.push(ch as u8);
        } else {
            result.push(b'?');
        }
    }
    Ok(ConversionResult {
        data: result,
        issues: Vec::new(),
    })
}

/// Get the single-byte-to-unicode mapping table for an encoding.
/// Returns None for ISO-8859-1 (identity mapping).
pub(super) fn get_single_byte_to_unicode_table(encoding: &Encoding) -> Option<&'static [u32; 256]> {
    match encoding.code_page {
        28591 => None, // ISO-8859-1 is identity
        _ => None,     // Simplified: treat all as identity for now
    }
}

/// Get the unicode-to-single-byte reverse mapping for an encoding.
pub(super) fn get_unicode_to_single_byte_table(
    _encoding: &Encoding,
) -> Option<std::collections::HashMap<u32, u8>> {
    None // ISO-8859-1 identity — handled inline
}
