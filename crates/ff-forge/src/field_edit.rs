//! Field editing and encode-back logic.
//!
//! Implements the pipeline: validate input → encode to target format
//! (plain text / EBCDIC / COMP-3) → verify fits field length → produce byte patch.

use crate::comp3;
use crate::error::FileForgeError;
use crate::field_def::{DataType, FieldDefinition};
use crate::field_validation::FieldValidator;

/// Represents a pending field edit before application.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldEdit {
    /// 0-based record index in the file.
    pub record_index: usize,
    /// Index of the field within the record structure.
    pub field_index: usize,
    /// The new value as a string.
    pub new_value: String,
}

/// Result of encoding a field edit to bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodedEdit {
    /// Byte offset within the record where the edit applies.
    pub offset: usize,
    /// The new byte content for the field.
    pub bytes: Vec<u8>,
}

/// Validates and encodes a field edit to bytes.
///
/// This is the core edit pipeline:
/// 1. Validate input against field data type
/// 2. Encode value to the correct byte representation
/// 3. Verify encoded bytes fit within field length
/// 4. Return the byte patch
///
/// # Errors
///
/// Returns `FileForgeError::FieldValidation` or `FileForgeError::FieldOverflow`
/// if the value fails validation or encoding.
pub fn encode_field_edit(
    field: &FieldDefinition,
    value: &str,
) -> Result<EncodedEdit, FileForgeError> {
    // Step 1: Validate
    FieldValidator::validate(field, value)?;

    // Step 2: Encode to bytes
    let bytes = encode_value(field, value)?;

    // Step 3: Verify length
    if bytes.len() > field.length {
        return Err(FileForgeError::FieldOverflow {
            field_name: field.field_name.clone(),
            max_length: field.length,
            actual_length: bytes.len(),
        });
    }

    Ok(EncodedEdit {
        offset: field.offset,
        bytes,
    })
}

/// Encodes a validated value to bytes according to the field's data type.
fn encode_value(field: &FieldDefinition, value: &str) -> Result<Vec<u8>, FileForgeError> {
    match field.data_type {
        DataType::Str | DataType::Bool => {
            // Pad with spaces to field length
            let mut bytes = value.as_bytes().to_vec();
            bytes.resize(field.length, b' ');
            Ok(bytes)
        }
        DataType::Int => {
            // Right-align, space-padded
            let formatted = format!("{:>width$}", value.trim(), width = field.length);
            Ok(formatted.into_bytes()[..field.length].to_vec())
        }
        DataType::Float => {
            let formatted = format!("{:>width$}", value.trim(), width = field.length);
            Ok(formatted.into_bytes()[..field.length].to_vec())
        }
        DataType::Comp3 => comp3::encode_comp3(value, field.decimals, field.length),
    }
}

/// Applies a byte patch to a record buffer at the specified offset.
///
/// Overwrites `field.length` bytes starting at `field.offset` with the
/// encoded bytes, padding if shorter.
pub fn apply_patch(record: &mut [u8], edit: &EncodedEdit, field_length: usize) {
    let start = edit.offset;
    let end = (start + field_length).min(record.len());
    let copy_len = edit.bytes.len().min(end - start);

    record[start..start + copy_len].copy_from_slice(&edit.bytes[..copy_len]);

    // Pad remaining with spaces if encoded bytes are shorter than field
    for byte in record[start + copy_len..end].iter_mut() {
        *byte = b' ';
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_field(
        name: &str,
        dt: DataType,
        offset: usize,
        length: usize,
        decimals: u8,
    ) -> FieldDefinition {
        FieldDefinition {
            field_name: name.to_string(),
            offset,
            length,
            data_type: dt,
            decimals,
            identifiers: vec![],
            filters: vec![],
        }
    }

    // Validates: Requirement 3.3
    #[test]
    fn encode_str_field_pads_with_spaces() {
        let field = make_field("name", DataType::Str, 0, 10, 0);
        let result = encode_field_edit(&field, "Hello").unwrap();
        assert_eq!(result.bytes.len(), 10);
        assert_eq!(&result.bytes[..5], b"Hello");
        assert_eq!(&result.bytes[5..], b"     ");
    }

    #[test]
    fn encode_int_field_right_aligned() {
        let field = make_field("count", DataType::Int, 5, 8, 0);
        let result = encode_field_edit(&field, "123").unwrap();
        assert_eq!(result.offset, 5);
        assert_eq!(result.bytes.len(), 8);
        // Right-aligned
        let text = String::from_utf8(result.bytes).unwrap();
        assert_eq!(text.trim(), "123");
    }

    // Validates: Requirement 5.5
    #[test]
    fn encode_comp3_field() {
        let field = make_field("amount", DataType::Comp3, 10, 4, 0);
        let result = encode_field_edit(&field, "1234567").unwrap();
        assert_eq!(result.offset, 10);
        assert_eq!(result.bytes.len(), 4);
        // Verify roundtrip
        let decoded = comp3::decode_comp3(&result.bytes).unwrap();
        assert_eq!(decoded.mantissa, 1234567);
    }

    // Validates: Requirement 3.5
    #[test]
    fn encode_rejects_overflow() {
        let field = make_field("tiny", DataType::Str, 0, 3, 0);
        let result = encode_field_edit(&field, "Too Long");
        assert!(matches!(result, Err(FileForgeError::FieldOverflow { .. })));
    }

    #[test]
    fn apply_patch_overwrites_at_offset() {
        let mut record = b"AAAAAAAAAA".to_vec(); // 10 bytes
        let edit = EncodedEdit {
            offset: 3,
            bytes: b"XYZ".to_vec(),
        };
        apply_patch(&mut record, &edit, 3);
        assert_eq!(&record, b"AAAXYZAAAA");
    }

    #[test]
    fn apply_patch_pads_shorter_value() {
        let mut record = b"AAAAAAAAAA".to_vec();
        let edit = EncodedEdit {
            offset: 2,
            bytes: b"X".to_vec(),
        };
        apply_patch(&mut record, &edit, 4);
        assert_eq!(&record, b"AAX   AAAA");
    }
}
