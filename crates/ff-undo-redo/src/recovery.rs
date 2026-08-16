//! Recovery file system — periodic persistence of undo state for crash recovery.
//!
//! Serializes the undo state (stacks, save point, scrap, record IDs) with a
//! CRC32 checksum for corruption detection.

use std::path::{Path, PathBuf};

use crate::error::UndoError;
use crate::scrap::ScrapStack;

/// Magic bytes identifying an ff-undo-redo recovery file.
const RECOVERY_MAGIC: &[u8; 8] = b"FFUNDO01";

/// Recovery file format version.
const RECOVERY_VERSION: u32 = 1;

/// Computes the recovery file path for a given source file.
///
/// Format: `.<source_stem>.recovery` in the same directory.
pub fn recovery_path_for(source_path: &Path) -> PathBuf {
    let parent = source_path.parent().unwrap_or(Path::new("."));
    let stem = source_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    parent.join(format!(".{}.recovery", stem))
}

/// Computes the recovery file path for an unsaved document.
///
/// Format: `~/.fileforgewb/recovery/<session_id>.recovery`
pub fn unsaved_recovery_path(session_id: &str) -> PathBuf {
    let recovery_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".fileforgewb")
        .join("recovery");
    recovery_dir.join(format!("{}.recovery", session_id))
}

/// Serializes undo state for recovery.
///
/// Format:
/// - 8 bytes: magic "FFUNDO01"
/// - 4 bytes: version (u32 LE)
/// - 4 bytes: payload length (u32 LE)
/// - N bytes: JSON payload
/// - 4 bytes: CRC32 checksum of payload
pub fn serialize_for_recovery(
    scrap: &ScrapStack,
    save_point: usize,
    current_action: usize,
    record_id_data: Option<&[u8]>,
) -> Result<Vec<u8>, UndoError> {
    let payload = RecoveryPayload {
        save_point,
        current_action,
        scrap_data: scrap.as_bytes().to_vec(),
        record_id_data: record_id_data.map(|d| d.to_vec()),
    };

    let json = serde_json::to_vec(&payload).map_err(|e| UndoError::Serialization(e.to_string()))?;

    let checksum = crc32fast::hash(&json);

    let mut output = Vec::with_capacity(8 + 4 + 4 + json.len() + 4);
    output.extend_from_slice(RECOVERY_MAGIC);
    output.extend_from_slice(&RECOVERY_VERSION.to_le_bytes());
    output.extend_from_slice(&(json.len() as u32).to_le_bytes());
    output.extend_from_slice(&json);
    output.extend_from_slice(&checksum.to_le_bytes());

    Ok(output)
}

/// Deserializes and validates a recovery file.
///
/// Verifies magic, version, and CRC32 checksum before accepting.
pub fn deserialize_recovery(data: &[u8]) -> Result<RecoveryPayload, UndoError> {
    if data.len() < 20 {
        return Err(UndoError::RecoveryCorrupted);
    }

    // Check magic
    if &data[0..8] != RECOVERY_MAGIC {
        return Err(UndoError::RecoveryCorrupted);
    }

    // Check version
    let version = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    if version != RECOVERY_VERSION {
        return Err(UndoError::RecoveryCorrupted);
    }

    // Read payload length
    let payload_len = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;

    // Verify total length
    let expected_total = 16 + payload_len + 4;
    if data.len() < expected_total {
        return Err(UndoError::RecoveryCorrupted);
    }

    let payload_bytes = &data[16..16 + payload_len];
    let checksum_bytes = &data[16 + payload_len..16 + payload_len + 4];

    // Verify CRC32
    let stored_checksum = u32::from_le_bytes([
        checksum_bytes[0],
        checksum_bytes[1],
        checksum_bytes[2],
        checksum_bytes[3],
    ]);
    let computed_checksum = crc32fast::hash(payload_bytes);

    if stored_checksum != computed_checksum {
        return Err(UndoError::RecoveryCorrupted);
    }

    // Deserialize payload
    let payload: RecoveryPayload =
        serde_json::from_slice(payload_bytes).map_err(|_| UndoError::RecoveryCorrupted)?;

    Ok(payload)
}

/// The data payload stored in a recovery file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecoveryPayload {
    /// The save point position.
    pub save_point: usize,
    /// The current action position.
    pub current_action: usize,
    /// Raw scrap stack data.
    pub scrap_data: Vec<u8>,
    /// Serialized record ID map (optional).
    pub record_id_data: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_path_for_normal_file() {
        let path = Path::new("/home/user/project/main.cbl");
        let recovery = recovery_path_for(path);
        assert_eq!(
            recovery,
            PathBuf::from("/home/user/project/.main.cbl.recovery")
        );
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let mut scrap = ScrapStack::new();
        scrap.push(b"hello world");

        let data = serialize_for_recovery(&scrap, 5, 10, None).unwrap();
        let payload = deserialize_recovery(&data).unwrap();

        assert_eq!(payload.save_point, 5);
        assert_eq!(payload.current_action, 10);
        assert_eq!(payload.scrap_data, b"hello world");
        assert!(payload.record_id_data.is_none());
    }

    #[test]
    fn corrupted_magic_detected() {
        let mut scrap = ScrapStack::new();
        scrap.push(b"test");
        let mut data = serialize_for_recovery(&scrap, 0, 0, None).unwrap();
        // Corrupt magic bytes
        data[0] = b'X';
        assert!(matches!(
            deserialize_recovery(&data),
            Err(UndoError::RecoveryCorrupted)
        ));
    }

    #[test]
    fn corrupted_checksum_detected() {
        let mut scrap = ScrapStack::new();
        scrap.push(b"test");
        let mut data = serialize_for_recovery(&scrap, 0, 0, None).unwrap();
        // Corrupt last byte (checksum)
        let last = data.len() - 1;
        data[last] ^= 0xFF;
        assert!(matches!(
            deserialize_recovery(&data),
            Err(UndoError::RecoveryCorrupted)
        ));
    }

    #[test]
    fn truncated_data_detected() {
        let data = b"FFUNDO01";
        assert!(matches!(
            deserialize_recovery(data),
            Err(UndoError::RecoveryCorrupted)
        ));
    }

    #[test]
    fn roundtrip_with_record_id_data() {
        let scrap = ScrapStack::new();
        let record_data = b"some serialized record ids";

        let data = serialize_for_recovery(&scrap, 3, 7, Some(record_data)).unwrap();
        let payload = deserialize_recovery(&data).unwrap();

        assert_eq!(payload.save_point, 3);
        assert_eq!(payload.current_action, 7);
        assert_eq!(
            payload.record_id_data.as_deref(),
            Some(b"some serialized record ids".as_slice())
        );
    }
}
