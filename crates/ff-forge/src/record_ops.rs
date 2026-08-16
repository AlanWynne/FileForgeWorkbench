//! Record insert and delete operations.
//!
//! Manages structural modifications to the record set in Grid_Edit_Mode:
//! inserting new blank records and deleting existing records.

use crate::byte_index::ByteOffsetIndex;
use crate::vb_reader;

/// Represents a record insert operation.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordInsert {
    /// Byte offset in the buffer where the new record will be inserted.
    pub insert_offset: u64,
    /// The complete record bytes to insert (including RDW for VB).
    pub record_bytes: Vec<u8>,
}

/// Represents a record delete operation.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordDelete {
    /// Byte offset of the record to remove.
    pub delete_offset: u64,
    /// Number of bytes to remove (record length, including RDW for VB).
    pub delete_length: usize,
}

/// Creates a blank fixed-width record of the given LRECL.
///
/// The record is filled with ASCII spaces (0x20).
pub fn create_blank_fb_record(lrecl: usize) -> Vec<u8> {
    vec![b' '; lrecl]
}

/// Creates a blank fixed-width record with EBCDIC spaces (0x40).
pub fn create_blank_ebcdic_record(lrecl: usize) -> Vec<u8> {
    vec![0x40; lrecl]
}

/// Creates a blank VB record with the given content length.
///
/// Returns RDW + space-filled content.
pub fn create_blank_vb_record(content_length: usize) -> Vec<u8> {
    let content = vec![b' '; content_length];
    vb_reader::write_vb_record(&content)
}

/// Prepares a record insert operation for a fixed-width file.
///
/// The new record is inserted after `after_index`. It will be exactly
/// `lrecl` bytes, filled with spaces.
pub fn prepare_fb_insert(
    index: &ByteOffsetIndex,
    after_index: usize,
    lrecl: usize,
) -> RecordInsert {
    let insert_offset = match index.offset_of(after_index) {
        Some(offset) => offset + lrecl as u64,
        None => {
            // Insert at end of file
            (index.record_count() * lrecl) as u64
        }
    };

    RecordInsert {
        insert_offset,
        record_bytes: create_blank_fb_record(lrecl),
    }
}

/// Prepares a record insert operation for a VB file.
///
/// Creates a blank record with RDW prefix.
pub fn prepare_vb_insert(
    index: &ByteOffsetIndex,
    after_index: usize,
    content_length: usize,
) -> RecordInsert {
    let insert_offset = match index {
        ByteOffsetIndex::Variable { offsets } => {
            if after_index < offsets.len() {
                // Insert after this record — need the record's end position
                // For VB, content_offset points to content start; we need to go past it
                // Since we don't know the record length from the index alone, we insert
                // after the next record's start (or at end)
                if after_index + 1 < offsets.len() {
                    offsets[after_index + 1] - 4 // Before next record's RDW
                } else {
                    // Insert at end — estimate from last known offset + some content
                    offsets[after_index] + content_length as u64
                }
            } else {
                0
            }
        }
        ByteOffsetIndex::FixedWidth {
            lrecl,
            record_count,
        } => ((after_index + 1) * lrecl).min(record_count * lrecl) as u64,
    };

    RecordInsert {
        insert_offset,
        record_bytes: create_blank_vb_record(content_length),
    }
}

/// Prepares a record delete operation for a fixed-width file.
pub fn prepare_fb_delete(
    index: &ByteOffsetIndex,
    record_index: usize,
    lrecl: usize,
) -> Option<RecordDelete> {
    index.offset_of(record_index).map(|offset| RecordDelete {
        delete_offset: offset,
        delete_length: lrecl,
    })
}

/// Prepares a record delete for a VB file at the given index.
///
/// For VB, the delete includes the RDW (4 bytes) + content.
pub fn prepare_vb_delete(
    index: &ByteOffsetIndex,
    record_index: usize,
    record_content_length: usize,
) -> Option<RecordDelete> {
    index.offset_of(record_index).map(|content_offset| {
        RecordDelete {
            delete_offset: content_offset - 4, // Include RDW
            delete_length: record_content_length + 4,
        }
    })
}

/// Updates a fixed-width ByteOffsetIndex after a record insert.
pub fn update_index_after_insert(index: &ByteOffsetIndex, _insert_index: usize) -> ByteOffsetIndex {
    match index {
        ByteOffsetIndex::FixedWidth {
            lrecl,
            record_count,
        } => ByteOffsetIndex::FixedWidth {
            lrecl: *lrecl,
            record_count: record_count + 1,
        },
        ByteOffsetIndex::Variable { offsets } => {
            // For variable-length, we'd need to rebuild the index
            // This is a simplified version — in production, shift offsets
            ByteOffsetIndex::Variable {
                offsets: offsets.clone(),
            }
        }
    }
}

/// Updates a fixed-width ByteOffsetIndex after a record delete.
pub fn update_index_after_delete(index: &ByteOffsetIndex, _delete_index: usize) -> ByteOffsetIndex {
    match index {
        ByteOffsetIndex::FixedWidth {
            lrecl,
            record_count,
        } => ByteOffsetIndex::FixedWidth {
            lrecl: *lrecl,
            record_count: record_count.saturating_sub(1),
        },
        ByteOffsetIndex::Variable { offsets } => ByteOffsetIndex::Variable {
            offsets: offsets.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 11.1
    #[test]
    fn create_blank_fb_record_filled_with_spaces() {
        let record = create_blank_fb_record(80);
        assert_eq!(record.len(), 80);
        assert!(record.iter().all(|&b| b == b' '));
    }

    // Validates: Requirement 11.6
    #[test]
    fn create_blank_fb_record_exact_lrecl_length() {
        let record = create_blank_fb_record(120);
        assert_eq!(record.len(), 120);
    }

    // Validates: Requirement 11.5
    #[test]
    fn create_blank_vb_record_includes_rdw() {
        let record = create_blank_vb_record(76);
        assert_eq!(record.len(), 80); // 4 RDW + 76 content
                                      // Verify RDW
        let rdw_len = u16::from_be_bytes([record[0], record[1]]);
        assert_eq!(rdw_len, 80); // Total length including RDW
        assert_eq!(record[2], 0);
        assert_eq!(record[3], 0);
        // Content should be spaces
        assert!(record[4..].iter().all(|&b| b == b' '));
    }

    #[test]
    fn prepare_fb_insert_after_first_record() {
        let index = ByteOffsetIndex::FixedWidth {
            lrecl: 80,
            record_count: 10,
        };
        let op = prepare_fb_insert(&index, 0, 80);
        assert_eq!(op.insert_offset, 80); // After first record
        assert_eq!(op.record_bytes.len(), 80);
    }

    #[test]
    fn prepare_fb_delete_returns_correct_offset() {
        let index = ByteOffsetIndex::FixedWidth {
            lrecl: 80,
            record_count: 10,
        };
        let op = prepare_fb_delete(&index, 5, 80).unwrap();
        assert_eq!(op.delete_offset, 400); // 5 * 80
        assert_eq!(op.delete_length, 80);
    }

    // Validates: Requirement 11.7
    #[test]
    fn update_index_after_insert_increments_count() {
        let index = ByteOffsetIndex::FixedWidth {
            lrecl: 80,
            record_count: 100,
        };
        let new_index = update_index_after_insert(&index, 50);
        assert_eq!(new_index.record_count(), 101);
    }

    #[test]
    fn update_index_after_delete_decrements_count() {
        let index = ByteOffsetIndex::FixedWidth {
            lrecl: 80,
            record_count: 100,
        };
        let new_index = update_index_after_delete(&index, 50);
        assert_eq!(new_index.record_count(), 99);
    }

    // Validates: Requirement 11.1 (EBCDIC space for EBCDIC files)
    #[test]
    fn create_blank_ebcdic_record_uses_ebcdic_space() {
        let record = create_blank_ebcdic_record(80);
        assert_eq!(record.len(), 80);
        assert!(record.iter().all(|&b| b == 0x40)); // EBCDIC space
    }

    #[test]
    fn prepare_fb_insert_at_end_of_file() {
        let index = ByteOffsetIndex::FixedWidth {
            lrecl: 80,
            record_count: 10,
        };
        let op = prepare_fb_insert(&index, 9, 80);
        assert_eq!(op.insert_offset, 800); // After last record (10 * 80)
    }
}
