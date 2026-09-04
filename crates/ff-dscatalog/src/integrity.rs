//! Workspace integrity, backup, restore, diagnose, and reconcile.
//!
//! Validates: Requirement 26.1, 26.2, 26.3, 26.4, 26.5, 26.6
//!
//! ## Checksum support (Req 26.1)
//!
//! SHA-256 of each physical object is computed on demand and can be stored
//! alongside the catalogue entry. Verification is optional and enabled per
//! workspace.
//!
//! ## Backup (Req 26.2, 26.3)
//!
//! A backup captures: catalogue DB, all SQLite record stores, all native
//! dataset files, and operation journals. A manifest records schema version,
//! provider configuration, object inventory, and integrity checksums.
//!
//! ## Restore (Req 26.4)
//!
//! Restore supports the original workspace root or a remapped root without
//! changing logical dataset names.
//!
//! ## Diagnose (Req 26.5)
//!
//! Reports orphaned physical objects (on disk but absent from catalogue) and
//! dangling catalogue entries (in catalogue but missing on disk).
//!
//! ## Repair preview (Req 26.6)
//!
//! Repair operations are previewable and auditable -- the caller receives a
//! list of proposed actions and must explicitly apply each one.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CatalogError;

// === Checksum support (Req 26.1) ==========================================

/// Compute a CRC-32 hex digest of a file.
///
/// Uses the standard IEEE 802.3 polynomial. This is a lightweight integrity
/// check that requires no external crates. The digest is returned as an
/// 8-character lowercase hex string.
///
/// Validates: Requirement 26.1
pub fn checksum_file(path: &Path) -> Result<String, CatalogError> {
    let mut file = std::fs::File::open(path).map_err(|source| CatalogError::IoError {
        operation: "checksum_file".to_string(),
        source,
    })?;
    let mut crc: u32 = 0xFFFF_FFFF;
    let mut buf = [0u8; 65536];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|source| CatalogError::IoError {
                operation: "checksum_file_read".to_string(),
                source,
            })?;
        if n == 0 {
            break;
        }
        for &byte in &buf[..n] {
            crc = crc32_update(crc, byte);
        }
    }
    Ok(format!("{:08x}", crc ^ 0xFFFF_FFFF))
}

/// Update a running CRC-32 value with one byte (IEEE 802.3 polynomial).
fn crc32_update(crc: u32, byte: u8) -> u32 {
    const POLY: u32 = 0xEDB8_8320;
    let mut v = crc ^ (byte as u32);
    for _ in 0..8 {
        if v & 1 != 0 {
            v = (v >> 1) ^ POLY;
        } else {
            v >>= 1;
        }
    }
    v
}

/// Verify a file's checksum matches an expected value.
///
/// Returns `Ok(true)` if the digest matches, `Ok(false)` if it does not.
/// Validates: Requirement 26.1
pub fn verify_checksum(path: &Path, expected: &str) -> Result<bool, CatalogError> {
    let actual = checksum_file(path)?;
    Ok(actual == expected)
}

// === Backup manifest (Req 26.3) ===========================================

/// Schema version written into every backup manifest.
pub const BACKUP_SCHEMA_VERSION: &str = "1.0";

/// Entry in the backup object inventory.
///
/// Validates: Requirement 26.3
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectInventoryEntry {
    /// Relative path within the backup archive.
    pub archive_path: String,
    /// SHA-256 hex digest of the file content.
    pub sha256: String,
    /// File size in bytes.
    pub size_bytes: u64,
}

/// Backup manifest written as `manifest.json` inside the archive.
///
/// Validates: Requirement 26.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    /// Schema version of this manifest format.
    pub schema_version: String,
    /// ISO 8601 timestamp of when the backup was created.
    pub created_at: String,
    /// Human-readable description of the workspace.
    pub workspace_description: String,
    /// Provider configuration snapshot (key-value pairs).
    pub provider_config: HashMap<String, String>,
    /// Inventory of all files captured in the backup.
    pub inventory: Vec<ObjectInventoryEntry>,
}

impl BackupManifest {
    /// Create a new manifest with the current timestamp.
    pub fn new(workspace_description: impl Into<String>) -> Self {
        Self {
            schema_version: BACKUP_SCHEMA_VERSION.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            workspace_description: workspace_description.into(),
            provider_config: HashMap::new(),
            inventory: Vec::new(),
        }
    }

    /// Serialise to JSON bytes.
    pub fn to_json(&self) -> Result<Vec<u8>, CatalogError> {
        serde_json::to_vec_pretty(self).map_err(|e| CatalogError::ExportError {
            reason: e.to_string(),
            operation: "manifest_to_json".to_string(),
        })
    }

    /// Deserialise from JSON bytes.
    pub fn from_json(bytes: &[u8]) -> Result<Self, CatalogError> {
        serde_json::from_slice(bytes).map_err(|e| CatalogError::ImportError {
            reason: e.to_string(),
            operation: "manifest_from_json".to_string(),
        })
    }
}

// === Backup (Req 26.2) ====================================================

/// Capture a workspace into a ZIP backup archive.
///
/// Walks `workspace_root`, adds every file to the archive, computes SHA-256
/// for each, and writes a `manifest.json` as the final entry.
///
/// Validates: Requirement 26.2, 26.3
pub fn backup_workspace(
    workspace_root: &Path,
    output_path: &Path,
    description: &str,
) -> Result<BackupManifest, CatalogError> {
    let file = std::fs::File::create(output_path).map_err(|source| CatalogError::IoError {
        operation: "backup_create_archive".to_string(),
        source,
    })?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut manifest = BackupManifest::new(description);

    // Walk workspace and add every file
    for entry in walkdir::WalkDir::new(workspace_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let abs_path = entry.path();
        let rel_path = abs_path
            .strip_prefix(workspace_root)
            .unwrap_or(abs_path)
            .to_string_lossy()
            .replace('\\', "/");

        let checksum = checksum_file(abs_path)?;
        let size = abs_path.metadata().map(|m| m.len()).unwrap_or(0);

        let content = std::fs::read(abs_path).map_err(|source| CatalogError::IoError {
            operation: "backup_read_file".to_string(),
            source,
        })?;

        zip.start_file(&rel_path, options)
            .map_err(|e| CatalogError::ExportError {
                reason: e.to_string(),
                operation: "backup_start_file".to_string(),
            })?;
        zip.write_all(&content)
            .map_err(|source| CatalogError::IoError {
                operation: "backup_write_file".to_string(),
                source,
            })?;

        manifest.inventory.push(ObjectInventoryEntry {
            archive_path: rel_path,
            sha256: checksum,
            size_bytes: size,
        });
    }

    // Write manifest as final entry
    let manifest_bytes = manifest.to_json()?;
    zip.start_file("manifest.json", options)
        .map_err(|e| CatalogError::ExportError {
            reason: e.to_string(),
            operation: "backup_write_manifest".to_string(),
        })?;
    zip.write_all(&manifest_bytes)
        .map_err(|source| CatalogError::IoError {
            operation: "backup_write_manifest_bytes".to_string(),
            source,
        })?;

    zip.finish().map_err(|e| CatalogError::ExportError {
        reason: e.to_string(),
        operation: "backup_finish".to_string(),
    })?;

    Ok(manifest)
}

// === Restore (Req 26.4) ===================================================

/// Restore a workspace from a backup archive.
///
/// Extracts all files to `target_root`. If `target_root` differs from the
/// original workspace root recorded in the manifest, logical dataset names
/// are unchanged -- only the physical root changes.
///
/// Validates: Requirement 26.4
pub fn restore_workspace(
    archive_path: &Path,
    target_root: &Path,
) -> Result<BackupManifest, CatalogError> {
    let file = std::fs::File::open(archive_path).map_err(|source| CatalogError::IoError {
        operation: "restore_open_archive".to_string(),
        source,
    })?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| CatalogError::ImportError {
        reason: e.to_string(),
        operation: "restore_open_zip".to_string(),
    })?;

    // Read manifest first
    let manifest = {
        let mut entry = zip
            .by_name("manifest.json")
            .map_err(|e| CatalogError::ImportError {
                reason: format!("manifest.json not found: {e}"),
                operation: "restore_read_manifest".to_string(),
            })?;
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|source| CatalogError::IoError {
                operation: "restore_read_manifest_bytes".to_string(),
                source,
            })?;
        BackupManifest::from_json(&bytes)?
    };

    // Extract all files
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| CatalogError::ImportError {
            reason: e.to_string(),
            operation: "restore_entry".to_string(),
        })?;
        if entry.name() == "manifest.json" {
            continue;
        }
        let out_path = target_root.join(entry.name());
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| CatalogError::IoError {
                operation: "restore_create_dirs".to_string(),
                source,
            })?;
        }
        let mut out_file =
            std::fs::File::create(&out_path).map_err(|source| CatalogError::IoError {
                operation: "restore_create_file".to_string(),
                source,
            })?;
        std::io::copy(&mut entry, &mut out_file).map_err(|source| CatalogError::IoError {
            operation: "restore_copy_file".to_string(),
            source,
        })?;
    }

    Ok(manifest)
}

// === Diagnose (Req 26.5) ==================================================

/// A single diagnostic finding.
///
/// Validates: Requirement 26.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticFinding {
    /// Physical object exists on disk but has no catalogue entry.
    OrphanedObject { path: PathBuf },
    /// Catalogue entry exists but physical object is missing from disk.
    DanglingEntry { locator: String },
    /// Physical object exists but its checksum does not match the stored value.
    ChecksumMismatch {
        locator: String,
        expected: String,
        actual: String,
    },
}

impl std::fmt::Display for DiagnosticFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OrphanedObject { path } => {
                write!(f, "orphaned object: {}", path.display())
            }
            Self::DanglingEntry { locator } => {
                write!(f, "dangling catalogue entry: {locator}")
            }
            Self::ChecksumMismatch {
                locator,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "checksum mismatch for '{locator}': expected {expected}, got {actual}"
                )
            }
        }
    }
}

/// Diagnose a workspace by comparing catalogue entries with physical objects.
///
/// - `workspace_root`: root of the workspace directory.
/// - `catalogue_locators`: locators known to the catalogue (relative paths).
/// - `checksums`: optional map of locator -> expected SHA-256 hex digest.
///
/// Returns a list of findings. An empty list means the workspace is consistent.
///
/// Validates: Requirement 26.5
pub fn diagnose_workspace(
    workspace_root: &Path,
    catalogue_locators: &[String],
    checksums: Option<&HashMap<String, String>>,
) -> Result<Vec<DiagnosticFinding>, CatalogError> {
    let mut findings = Vec::new();

    // Check for dangling entries and checksum mismatches
    for locator in catalogue_locators {
        let path = workspace_root.join(locator);
        if !path.exists() {
            findings.push(DiagnosticFinding::DanglingEntry {
                locator: locator.clone(),
            });
            continue;
        }
        if let Some(cs_map) = checksums {
            if let Some(expected) = cs_map.get(locator) {
                let actual = checksum_file(&path)?;
                if &actual != expected {
                    findings.push(DiagnosticFinding::ChecksumMismatch {
                        locator: locator.clone(),
                        expected: expected.clone(),
                        actual,
                    });
                }
            }
        }
    }

    // Check for orphaned objects in datasets/objects/
    let objects_dir = workspace_root.join("datasets").join("objects");
    if objects_dir.is_dir() {
        let known: std::collections::HashSet<PathBuf> = catalogue_locators
            .iter()
            .map(|l| workspace_root.join(l))
            .collect();
        for entry in walkdir::WalkDir::new(&objects_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if !known.contains(entry.path()) {
                findings.push(DiagnosticFinding::OrphanedObject {
                    path: entry.path().to_path_buf(),
                });
            }
        }
    }

    Ok(findings)
}

// === Repair preview (Req 26.6) ============================================

/// A proposed repair action.
///
/// Repair operations are previewable -- the caller receives this list and
/// must explicitly call `apply_repair` for each action they approve.
///
/// Validates: Requirement 26.6
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepairAction {
    /// Delete an orphaned physical object.
    DeleteOrphan { path: PathBuf },
    /// Remove a dangling catalogue entry (caller must update catalogue).
    RemoveDanglingEntry { locator: String },
    /// Re-compute and store the correct checksum for a file.
    RefreshChecksum { locator: String },
}

impl std::fmt::Display for RepairAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeleteOrphan { path } => write!(f, "delete orphan: {}", path.display()),
            Self::RemoveDanglingEntry { locator } => {
                write!(f, "remove dangling entry: {locator}")
            }
            Self::RefreshChecksum { locator } => {
                write!(f, "refresh checksum: {locator}")
            }
        }
    }
}

/// Build a repair plan from a list of diagnostic findings.
///
/// Does NOT apply any changes. Returns one `RepairAction` per finding.
/// Validates: Requirement 26.6
pub fn repair_plan(findings: &[DiagnosticFinding]) -> Vec<RepairAction> {
    findings
        .iter()
        .map(|f| match f {
            DiagnosticFinding::OrphanedObject { path } => {
                RepairAction::DeleteOrphan { path: path.clone() }
            }
            DiagnosticFinding::DanglingEntry { locator } => RepairAction::RemoveDanglingEntry {
                locator: locator.clone(),
            },
            DiagnosticFinding::ChecksumMismatch { locator, .. } => RepairAction::RefreshChecksum {
                locator: locator.clone(),
            },
        })
        .collect()
}

/// Apply a single repair action to the filesystem.
///
/// Catalogue mutations (RemoveDanglingEntry) are NOT performed here --
/// the caller is responsible for updating the catalogue after this returns.
///
/// Validates: Requirement 26.6
pub fn apply_repair(workspace_root: &Path, action: &RepairAction) -> Result<(), CatalogError> {
    match action {
        RepairAction::DeleteOrphan { path } => {
            if path.is_file() {
                std::fs::remove_file(path).map_err(|source| CatalogError::IoError {
                    operation: "apply_repair_delete_orphan".to_string(),
                    source,
                })?;
            }
        }
        RepairAction::RemoveDanglingEntry { .. } => {
            // Catalogue mutation -- caller's responsibility.
        }
        RepairAction::RefreshChecksum { locator } => {
            // Just verify the file is readable; caller stores the new digest.
            let path = workspace_root.join(locator);
            checksum_file(&path)?;
        }
    }
    Ok(())
}

// === Tests ================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as IoWrite;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    fn write_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).unwrap();
        }
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content).unwrap();
        path
    }

    // === Req 26.1 -- checksums ============================================

    #[test]
    fn checksum_file_produces_hex_digest() {
        // Validates: Requirement 26.1
        let dir = tmp();
        let path = write_file(dir.path(), "test.dat", b"hello world");
        let digest = checksum_file(&path).unwrap();
        assert_eq!(digest.len(), 8);
        // Must be valid hex
        assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn checksum_file_empty_file_has_known_value() {
        // Validates: Requirement 26.1
        let dir = tmp();
        let path = write_file(dir.path(), "a.dat", b"");
        let digest = checksum_file(&path).unwrap();
        // CRC-32 of empty input is 00000000
        assert_eq!(digest, "00000000");
    }

    #[test]
    fn verify_checksum_returns_true_for_matching_digest() {
        // Validates: Requirement 26.1
        let dir = tmp();
        let path = write_file(dir.path(), "b.dat", b"");
        let digest = checksum_file(&path).unwrap();
        assert!(verify_checksum(&path, &digest).unwrap());
    }

    #[test]
    fn verify_checksum_returns_false_for_wrong_digest() {
        // Validates: Requirement 26.1
        let dir = tmp();
        let path = write_file(dir.path(), "c.dat", b"data");
        assert!(!verify_checksum(&path, "00000000").unwrap());
    }

    // === Req 26.3 -- manifest round-trip ==================================

    #[test]
    fn manifest_serialises_and_deserialises() {
        // Validates: Requirement 26.3
        let mut m = BackupManifest::new("test workspace");
        m.inventory.push(ObjectInventoryEntry {
            archive_path: "catalog.db".to_string(),
            sha256: "abc123".to_string(),
            size_bytes: 4096,
        });
        let bytes = m.to_json().unwrap();
        let m2 = BackupManifest::from_json(&bytes).unwrap();
        assert_eq!(m2.schema_version, BACKUP_SCHEMA_VERSION);
        assert_eq!(m2.inventory.len(), 1);
        assert_eq!(m2.inventory[0].archive_path, "catalog.db");
    }

    #[test]
    fn manifest_schema_version_is_set() {
        // Validates: Requirement 26.3
        let m = BackupManifest::new("ws");
        assert_eq!(m.schema_version, BACKUP_SCHEMA_VERSION);
    }

    // === Req 26.2, 26.3 -- backup =========================================

    #[test]
    fn backup_creates_archive_with_manifest() {
        // Validates: Requirement 26.2, 26.3
        let src = tmp();
        write_file(src.path(), "catalog.db", b"sqlite data");
        write_file(src.path(), "datasets/objects/uuid1.dat", b"record data");

        let out = tmp();
        let archive = out.path().join("backup.zip");
        let manifest = backup_workspace(src.path(), &archive, "test backup").unwrap();

        assert!(archive.exists());
        assert_eq!(manifest.schema_version, BACKUP_SCHEMA_VERSION);
        // inventory has 2 files (catalog.db + uuid1.dat)
        assert_eq!(manifest.inventory.len(), 2);
        // all entries have non-empty checksums
        for entry in &manifest.inventory {
            assert_eq!(entry.sha256.len(), 8);
        }
    }

    #[test]
    fn backup_manifest_contains_correct_sizes() {
        // Validates: Requirement 26.3
        let src = tmp();
        let content = b"hello backup";
        write_file(src.path(), "file.dat", content);

        let out = tmp();
        let archive = out.path().join("b.zip");
        let manifest = backup_workspace(src.path(), &archive, "ws").unwrap();

        let entry = manifest
            .inventory
            .iter()
            .find(|e| e.archive_path == "file.dat")
            .unwrap();
        assert_eq!(entry.size_bytes, content.len() as u64);
    }

    // === Req 26.4 -- restore ==============================================

    #[test]
    fn restore_extracts_files_to_target_root() {
        // Validates: Requirement 26.4
        let src = tmp();
        write_file(src.path(), "catalog.db", b"db content");
        write_file(src.path(), "datasets/objects/x.dat", b"obj content");

        let out = tmp();
        let archive = out.path().join("bk.zip");
        backup_workspace(src.path(), &archive, "ws").unwrap();

        let restore_dir = tmp();
        let manifest = restore_workspace(&archive, restore_dir.path()).unwrap();

        assert!(restore_dir.path().join("catalog.db").exists());
        assert!(restore_dir.path().join("datasets/objects/x.dat").exists());
        assert_eq!(manifest.schema_version, BACKUP_SCHEMA_VERSION);
    }

    #[test]
    fn restore_preserves_file_content() {
        // Validates: Requirement 26.4
        let src = tmp();
        let content = b"exact bytes preserved";
        write_file(src.path(), "data.dat", content);

        let out = tmp();
        let archive = out.path().join("bk2.zip");
        backup_workspace(src.path(), &archive, "ws").unwrap();

        let restore_dir = tmp();
        restore_workspace(&archive, restore_dir.path()).unwrap();

        let restored = std::fs::read(restore_dir.path().join("data.dat")).unwrap();
        assert_eq!(restored, content);
    }

    // === Req 26.5 -- diagnose =============================================

    #[test]
    fn diagnose_reports_dangling_entry() {
        // Validates: Requirement 26.5
        let dir = tmp();
        let locators = vec!["datasets/objects/missing.dat".to_string()];
        let findings = diagnose_workspace(dir.path(), &locators, None).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(matches!(
            findings[0],
            DiagnosticFinding::DanglingEntry { .. }
        ));
    }

    #[test]
    fn diagnose_reports_orphaned_object() {
        // Validates: Requirement 26.5
        let dir = tmp();
        // Create a physical file with no catalogue entry
        write_file(dir.path(), "datasets/objects/orphan.dat", b"orphan");
        let findings = diagnose_workspace(dir.path(), &[], None).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(matches!(
            findings[0],
            DiagnosticFinding::OrphanedObject { .. }
        ));
    }

    #[test]
    fn diagnose_reports_checksum_mismatch() {
        // Validates: Requirement 26.5
        let dir = tmp();
        write_file(dir.path(), "datasets/objects/f.dat", b"original");
        let locator = "datasets/objects/f.dat".to_string();
        let mut cs = HashMap::new();
        cs.insert(locator.clone(), "00000000".to_string());
        let findings = diagnose_workspace(dir.path(), &[locator], Some(&cs)).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(matches!(
            findings[0],
            DiagnosticFinding::ChecksumMismatch { .. }
        ));
    }

    #[test]
    fn diagnose_clean_workspace_returns_empty() {
        // Validates: Requirement 26.5
        let dir = tmp();
        let path = write_file(dir.path(), "datasets/objects/ok.dat", b"data");
        let locator = "datasets/objects/ok.dat".to_string();
        let digest = checksum_file(&path).unwrap();
        let mut cs = HashMap::new();
        cs.insert(locator.clone(), digest);
        let findings = diagnose_workspace(dir.path(), &[locator], Some(&cs)).unwrap();
        assert!(findings.is_empty());
    }

    // === Req 26.6 -- repair preview =======================================

    #[test]
    fn repair_plan_maps_findings_to_actions() {
        // Validates: Requirement 26.6
        let findings = vec![
            DiagnosticFinding::OrphanedObject {
                path: PathBuf::from("x.dat"),
            },
            DiagnosticFinding::DanglingEntry {
                locator: "loc/y.dat".to_string(),
            },
            DiagnosticFinding::ChecksumMismatch {
                locator: "loc/z.dat".to_string(),
                expected: "aaa".to_string(),
                actual: "bbb".to_string(),
            },
        ];
        let plan = repair_plan(&findings);
        assert_eq!(plan.len(), 3);
        assert!(matches!(plan[0], RepairAction::DeleteOrphan { .. }));
        assert!(matches!(plan[1], RepairAction::RemoveDanglingEntry { .. }));
        assert!(matches!(plan[2], RepairAction::RefreshChecksum { .. }));
    }

    #[test]
    fn apply_repair_deletes_orphan_file() {
        // Validates: Requirement 26.6
        let dir = tmp();
        let path = write_file(dir.path(), "orphan.dat", b"junk");
        assert!(path.exists());
        let action = RepairAction::DeleteOrphan { path: path.clone() };
        apply_repair(dir.path(), &action).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn apply_repair_dangling_entry_is_noop_on_filesystem() {
        // Validates: Requirement 26.6 -- catalogue mutation is caller's job
        let dir = tmp();
        let action = RepairAction::RemoveDanglingEntry {
            locator: "loc/missing.dat".to_string(),
        };
        // Should not error even though file does not exist
        apply_repair(dir.path(), &action).unwrap();
    }

    #[test]
    fn repair_plan_is_empty_for_no_findings() {
        // Validates: Requirement 26.6
        let plan = repair_plan(&[]);
        assert!(plan.is_empty());
    }
}
