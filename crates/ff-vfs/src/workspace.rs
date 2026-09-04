//! Workspace backup, restore, reconcile, and diagnose commands.
//!
//! Addresses: Requirement 12, criteria 12.1-12.5

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::VfsError;

// === Checksum ===================================================

/// Computes a simple FNV-1a 32-bit checksum of `data`.
/// Used for manifest integrity verification (Req 12.2).
pub fn checksum(data: &[u8]) -> u32 {
    let mut hash: u32 = 2_166_136_261;
    for &byte in data {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    hash
}

// === BackupManifest ===================================================

/// An entry in the backup manifest describing one backed-up file.
///
/// Addresses: Requirement 12 AC 12.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    /// Relative path within the backup archive.
    pub archive_path: String,
    /// Original absolute path on the source filesystem.
    pub original_path: String,
    /// FNV-1a checksum of the file content.
    pub checksum: u32,
    /// File size in bytes.
    pub size: u64,
}

/// Manifest written alongside a workspace backup.
///
/// Contains schema version, provider configuration, object inventory,
/// and integrity information (Req 12.2).
#[derive(Debug, Clone)]
pub struct BackupManifest {
    /// Schema version for forward-compatibility.
    pub schema_version: u32,
    /// Provider configuration snapshot (scheme -> description).
    pub providers: HashMap<String, String>,
    /// Inventory of all backed-up objects.
    pub entries: Vec<ManifestEntry>,
}

impl BackupManifest {
    /// Creates a new manifest with the given schema version.
    pub fn new(schema_version: u32) -> Self {
        Self {
            schema_version,
            providers: HashMap::new(),
            entries: Vec::new(),
        }
    }

    /// Serialises the manifest to a plain-text string.
    pub fn serialise(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("schema_version={}\n", self.schema_version));
        for (scheme, desc) in &self.providers {
            out.push_str(&format!("provider:{scheme}={desc}\n"));
        }
        for e in &self.entries {
            out.push_str(&format!(
                "entry:{}|{}|{}|{}\n",
                e.archive_path, e.original_path, e.checksum, e.size
            ));
        }
        out
    }

    /// Parses a manifest from its serialised plain-text form.
    pub fn parse(text: &str) -> Result<Self, VfsError> {
        let mut schema_version = 1u32;
        let mut providers = HashMap::new();
        let mut entries = Vec::new();

        for line in text.lines() {
            if let Some(v) = line.strip_prefix("schema_version=") {
                schema_version = v.parse().unwrap_or(1);
            } else if let Some(rest) = line.strip_prefix("provider:") {
                if let Some((k, v)) = rest.split_once('=') {
                    providers.insert(k.to_string(), v.to_string());
                }
            } else if let Some(rest) = line.strip_prefix("entry:") {
                let parts: Vec<&str> = rest.splitn(4, '|').collect();
                if parts.len() == 4 {
                    let ck = parts[2].parse().unwrap_or(0);
                    let sz = parts[3].parse().unwrap_or(0);
                    entries.push(ManifestEntry {
                        archive_path: parts[0].to_string(),
                        original_path: parts[1].to_string(),
                        checksum: ck,
                        size: sz,
                    });
                }
            }
        }

        Ok(Self {
            schema_version,
            providers,
            entries,
        })
    }
}

// === workspace_backup ===================================================

/// Captures all files under `source_root` into `archive_dir`, writing a
/// manifest file alongside the archived content.
///
/// The archive layout is:
/// ```text
/// archive_dir/
///   manifest.txt
///   objects/
///     <relative-path-with-slashes-replaced-by-underscores>
/// ```
///
/// Addresses: Requirement 12 AC 12.1, 12.2
pub fn workspace_backup(
    source_root: &Path,
    archive_dir: &Path,
    providers: &HashMap<String, String>,
) -> Result<BackupManifest, VfsError> {
    let objects_dir = archive_dir.join("objects");
    std::fs::create_dir_all(&objects_dir).map_err(|e| VfsError::Io {
        uri: objects_dir.to_string_lossy().into_owned(),
        operation: "backup_mkdir".to_string(),
        source: e,
    })?;

    let mut manifest = BackupManifest::new(1);
    manifest.providers = providers.clone();

    collect_files(source_root, source_root, &objects_dir, &mut manifest)?;

    // Write manifest.
    let manifest_path = archive_dir.join("manifest.txt");
    std::fs::write(&manifest_path, manifest.serialise().as_bytes()).map_err(|e| VfsError::Io {
        uri: manifest_path.to_string_lossy().into_owned(),
        operation: "backup_manifest".to_string(),
        source: e,
    })?;

    Ok(manifest)
}

/// Recursively collects files from `dir` into `objects_dir`, adding entries
/// to `manifest`.
fn collect_files(
    root: &Path,
    dir: &Path,
    objects_dir: &Path,
    manifest: &mut BackupManifest,
) -> Result<(), VfsError> {
    let read_dir = std::fs::read_dir(dir).map_err(|e| VfsError::Io {
        uri: dir.to_string_lossy().into_owned(),
        operation: "backup_readdir".to_string(),
        source: e,
    })?;

    for entry in read_dir {
        let entry = entry.map_err(|e| VfsError::Io {
            uri: dir.to_string_lossy().into_owned(),
            operation: "backup_entry".to_string(),
            source: e,
        })?;
        let path = entry.path();
        let meta = std::fs::metadata(&path).map_err(|e| VfsError::Io {
            uri: path.to_string_lossy().into_owned(),
            operation: "backup_stat".to_string(),
            source: e,
        })?;

        if meta.is_dir() {
            collect_files(root, &path, objects_dir, manifest)?;
        } else {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let archive_name = rel_str.replace('/', "_");

            let data = std::fs::read(&path).map_err(|e| VfsError::Io {
                uri: path.to_string_lossy().into_owned(),
                operation: "backup_read".to_string(),
                source: e,
            })?;

            let ck = checksum(&data);
            let size = data.len() as u64;

            let dest = objects_dir.join(&archive_name);
            std::fs::write(&dest, &data).map_err(|e| VfsError::Io {
                uri: dest.to_string_lossy().into_owned(),
                operation: "backup_write".to_string(),
                source: e,
            })?;

            manifest.entries.push(ManifestEntry {
                archive_path: archive_name,
                original_path: path.to_string_lossy().into_owned(),
                checksum: ck,
                size,
            });
        }
    }
    Ok(())
}

// === workspace_restore ===================================================

/// Restores a workspace from `archive_dir` to `restore_root`.
///
/// If `restore_root` differs from the original paths in the manifest, files
/// are remapped: the original root prefix is replaced with `restore_root`.
///
/// Validates checksums before writing each file (Req 12.3).
///
/// Addresses: Requirement 12 AC 12.3
pub fn workspace_restore(
    archive_dir: &Path,
    restore_root: &Path,
    original_root: &Path,
) -> Result<BackupManifest, VfsError> {
    let manifest_path = archive_dir.join("manifest.txt");
    let manifest_text = std::fs::read_to_string(&manifest_path).map_err(|e| VfsError::Io {
        uri: manifest_path.to_string_lossy().into_owned(),
        operation: "restore_manifest".to_string(),
        source: e,
    })?;

    let manifest = BackupManifest::parse(&manifest_text)?;
    let objects_dir = archive_dir.join("objects");

    for entry in &manifest.entries {
        let src = objects_dir.join(&entry.archive_path);
        let data = std::fs::read(&src).map_err(|e| VfsError::Io {
            uri: src.to_string_lossy().into_owned(),
            operation: "restore_read".to_string(),
            source: e,
        })?;

        // Verify checksum before writing.
        let actual = checksum(&data);
        if actual != entry.checksum {
            return Err(VfsError::Io {
                uri: entry.original_path.clone(),
                operation: "restore_checksum".to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "checksum mismatch: expected {}, got {}",
                        entry.checksum, actual
                    ),
                ),
            });
        }

        // Remap original path to restore root.
        let dest = remap_path(&entry.original_path, original_root, restore_root);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| VfsError::Io {
                uri: parent.to_string_lossy().into_owned(),
                operation: "restore_mkdir".to_string(),
                source: e,
            })?;
        }
        std::fs::write(&dest, &data).map_err(|e| VfsError::Io {
            uri: dest.to_string_lossy().into_owned(),
            operation: "restore_write".to_string(),
            source: e,
        })?;
    }

    Ok(manifest)
}

/// Remaps `original_path` by replacing the `original_root` prefix with
/// `new_root`. Falls back to joining `new_root` with the filename if the
/// prefix does not match.
fn remap_path(original_path: &str, original_root: &Path, new_root: &Path) -> PathBuf {
    let orig = Path::new(original_path);
    if let Ok(rel) = orig.strip_prefix(original_root) {
        new_root.join(rel)
    } else {
        // Fallback: use just the filename under new_root.
        let name = orig.file_name().unwrap_or_default();
        new_root.join(name)
    }
}

// === workspace_diagnose ===================================================

/// A finding produced by `workspace_diagnose` or `workspace_reconcile`.
///
/// Addresses: Requirement 12 AC 12.4, 12.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceFinding {
    /// Short category label (e.g. "orphaned", "dangling", "mismatch").
    pub kind: String,
    /// Human-readable description of the finding.
    pub description: String,
    /// Path or identifier of the affected object.
    pub path: String,
}

/// Compares `physical_paths` (files actually on disk) against
/// `catalogue_paths` (entries registered in the catalogue) and reports:
/// - Orphaned objects: on disk but not in catalogue.
/// - Dangling entries: in catalogue but not on disk.
///
/// Does NOT automatically correct anything (Req 12.5).
///
/// Addresses: Requirement 12 AC 12.5
pub fn workspace_diagnose(
    physical_paths: &[String],
    catalogue_paths: &[String],
) -> Vec<WorkspaceFinding> {
    use std::collections::HashSet;

    let physical: HashSet<&str> = physical_paths.iter().map(|s| s.as_str()).collect();
    let catalogue: HashSet<&str> = catalogue_paths.iter().map(|s| s.as_str()).collect();

    let mut findings = Vec::new();

    for p in &physical {
        if !catalogue.contains(p) {
            findings.push(WorkspaceFinding {
                kind: "orphaned".to_string(),
                description: format!(
                    "physical object '{p}' exists on disk but has no catalogue entry"
                ),
                path: p.to_string(),
            });
        }
    }

    for c in &catalogue {
        if !physical.contains(c) {
            findings.push(WorkspaceFinding {
                kind: "dangling".to_string(),
                description: format!("catalogue entry '{c}' has no corresponding physical object"),
                path: c.to_string(),
            });
        }
    }

    findings
}

// === workspace_reconcile ===================================================

/// Compares VFS provider state with catalogue state and reports proposed
/// corrections without automatically applying them (Req 12.4).
///
/// `provider_entries` maps logical name -> checksum of current content.
/// `catalogue_entries` maps logical name -> expected checksum.
///
/// Reports: missing from provider, missing from catalogue, checksum mismatches.
///
/// Addresses: Requirement 12 AC 12.4
pub fn workspace_reconcile(
    provider_entries: &HashMap<String, u32>,
    catalogue_entries: &HashMap<String, u32>,
) -> Vec<WorkspaceFinding> {
    let mut findings = Vec::new();

    for (name, &cat_ck) in catalogue_entries {
        match provider_entries.get(name) {
            None => findings.push(WorkspaceFinding {
                kind: "missing_from_provider".to_string(),
                description: format!("'{name}' is in catalogue but missing from provider"),
                path: name.clone(),
            }),
            Some(&prov_ck) if prov_ck != cat_ck => findings.push(WorkspaceFinding {
                kind: "checksum_mismatch".to_string(),
                description: format!(
                    "'{name}' checksum mismatch: catalogue={cat_ck}, provider={prov_ck}"
                ),
                path: name.clone(),
            }),
            _ => {}
        }
    }

    for name in provider_entries.keys() {
        if !catalogue_entries.contains_key(name) {
            findings.push(WorkspaceFinding {
                kind: "missing_from_catalogue".to_string(),
                description: format!("'{name}' exists in provider but has no catalogue entry"),
                path: name.clone(),
            });
        }
    }

    findings
}

// === Tests ===================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Validates: Requirement 12.1 -- backup captures files from source root
    #[test]
    fn backup_captures_all_files_from_source_root() {
        let src = TempDir::new().unwrap();
        std::fs::write(src.path().join("a.txt"), b"alpha").unwrap();
        std::fs::write(src.path().join("b.txt"), b"beta").unwrap();

        let archive = TempDir::new().unwrap();
        let mut providers = HashMap::new();
        providers.insert("posix".to_string(), "POSIX native provider".to_string());

        let manifest = workspace_backup(src.path(), archive.path(), &providers).unwrap();

        assert_eq!(manifest.entries.len(), 2);
        assert!(archive.path().join("manifest.txt").exists());
        assert!(archive.path().join("objects").is_dir());
    }

    // Validates: Requirement 12.2 -- manifest contains schema version and provider config
    #[test]
    fn backup_manifest_contains_schema_version_and_providers() {
        let src = TempDir::new().unwrap();
        std::fs::write(src.path().join("f.txt"), b"data").unwrap();

        let archive = TempDir::new().unwrap();
        let mut providers = HashMap::new();
        providers.insert("local".to_string(), "local fs".to_string());

        let manifest = workspace_backup(src.path(), archive.path(), &providers).unwrap();

        assert_eq!(manifest.schema_version, 1);
        assert!(manifest.providers.contains_key("local"));

        let text = std::fs::read_to_string(archive.path().join("manifest.txt")).unwrap();
        assert!(text.contains("schema_version=1"));
        assert!(text.contains("provider:local=local fs"));
    }

    // Validates: Requirement 12.2 -- manifest entries carry checksums and sizes
    #[test]
    fn backup_manifest_entries_have_checksums_and_sizes() {
        let src = TempDir::new().unwrap();
        std::fs::write(src.path().join("c.txt"), b"content").unwrap();

        let archive = TempDir::new().unwrap();
        let manifest = workspace_backup(src.path(), archive.path(), &HashMap::new()).unwrap();

        let entry = &manifest.entries[0];
        assert_eq!(entry.size, 7);
        assert_eq!(entry.checksum, checksum(b"content"));
    }

    // Validates: Requirement 12.3 -- restore round-trip produces identical content
    #[test]
    fn restore_round_trip_produces_identical_content() {
        let src = TempDir::new().unwrap();
        std::fs::write(src.path().join("hello.txt"), b"hello world").unwrap();

        let archive = TempDir::new().unwrap();
        workspace_backup(src.path(), archive.path(), &HashMap::new()).unwrap();

        let restore = TempDir::new().unwrap();
        workspace_restore(archive.path(), restore.path(), src.path()).unwrap();

        let restored = std::fs::read(restore.path().join("hello.txt")).unwrap();
        assert_eq!(restored, b"hello world");
    }

    // Validates: Requirement 12.3 -- restore to remapped root uses new root
    #[test]
    fn restore_to_remapped_root_places_files_under_new_root() {
        let src = TempDir::new().unwrap();
        std::fs::write(src.path().join("data.bin"), b"bytes").unwrap();

        let archive = TempDir::new().unwrap();
        workspace_backup(src.path(), archive.path(), &HashMap::new()).unwrap();

        let new_root = TempDir::new().unwrap();
        workspace_restore(archive.path(), new_root.path(), src.path()).unwrap();

        assert!(new_root.path().join("data.bin").exists());
    }

    // Validates: Requirement 12.3 -- restore rejects corrupted archive entry
    #[test]
    fn restore_rejects_corrupted_archive_entry() {
        let src = TempDir::new().unwrap();
        std::fs::write(src.path().join("ok.txt"), b"good").unwrap();

        let archive = TempDir::new().unwrap();
        workspace_backup(src.path(), archive.path(), &HashMap::new()).unwrap();

        // Corrupt the archived object.
        let objects_dir = archive.path().join("objects");
        let archived = std::fs::read_dir(&objects_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        std::fs::write(&archived, b"corrupted data").unwrap();

        let restore = TempDir::new().unwrap();
        let result = workspace_restore(archive.path(), restore.path(), src.path());
        assert!(result.is_err(), "restore must fail on checksum mismatch");
    }

    // Validates: Requirement 12.5 -- diagnose reports orphaned objects
    #[test]
    fn diagnose_reports_orphaned_physical_objects() {
        let physical = vec!["obj_a".to_string(), "obj_b".to_string()];
        let catalogue = vec!["obj_a".to_string()];

        let findings = workspace_diagnose(&physical, &catalogue);
        let orphaned: Vec<_> = findings.iter().filter(|f| f.kind == "orphaned").collect();
        assert_eq!(orphaned.len(), 1);
        assert_eq!(orphaned[0].path, "obj_b");
    }

    // Validates: Requirement 12.5 -- diagnose reports dangling catalogue entries
    #[test]
    fn diagnose_reports_dangling_catalogue_entries() {
        let physical = vec!["obj_a".to_string()];
        let catalogue = vec!["obj_a".to_string(), "obj_missing".to_string()];

        let findings = workspace_diagnose(&physical, &catalogue);
        let dangling: Vec<_> = findings.iter().filter(|f| f.kind == "dangling").collect();
        assert_eq!(dangling.len(), 1);
        assert_eq!(dangling[0].path, "obj_missing");
    }

    // Validates: Requirement 12.5 -- diagnose returns empty for clean workspace
    #[test]
    fn diagnose_returns_empty_for_clean_workspace() {
        let paths = vec!["a".to_string(), "b".to_string()];
        let findings = workspace_diagnose(&paths, &paths);
        assert!(findings.is_empty());
    }

    // Validates: Requirement 12.4 -- reconcile reports missing from provider
    #[test]
    fn reconcile_reports_missing_from_provider() {
        let mut catalogue = HashMap::new();
        catalogue.insert("ds1".to_string(), 0xABCD_u32);
        catalogue.insert("ds2".to_string(), 0x1234_u32);

        let mut provider = HashMap::new();
        provider.insert("ds1".to_string(), 0xABCD_u32);
        // ds2 missing from provider

        let findings = workspace_reconcile(&provider, &catalogue);
        let missing: Vec<_> = findings
            .iter()
            .filter(|f| f.kind == "missing_from_provider")
            .collect();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].path, "ds2");
    }

    // Validates: Requirement 12.4 -- reconcile reports checksum mismatch
    #[test]
    fn reconcile_reports_checksum_mismatch() {
        let mut catalogue = HashMap::new();
        catalogue.insert("ds1".to_string(), 0xAAAA_u32);

        let mut provider = HashMap::new();
        provider.insert("ds1".to_string(), 0xBBBB_u32);

        let findings = workspace_reconcile(&provider, &catalogue);
        let mismatches: Vec<_> = findings
            .iter()
            .filter(|f| f.kind == "checksum_mismatch")
            .collect();
        assert_eq!(mismatches.len(), 1);
        assert!(
            mismatches[0].description.contains("0xAAAA")
                || mismatches[0].description.contains("43690")
        );
    }

    // Validates: Requirement 12.4 -- reconcile reports missing from catalogue
    #[test]
    fn reconcile_reports_missing_from_catalogue() {
        let mut catalogue = HashMap::new();
        catalogue.insert("ds1".to_string(), 0x1111_u32);

        let mut provider = HashMap::new();
        provider.insert("ds1".to_string(), 0x1111_u32);
        provider.insert("ds_extra".to_string(), 0x2222_u32);

        let findings = workspace_reconcile(&provider, &catalogue);
        let extra: Vec<_> = findings
            .iter()
            .filter(|f| f.kind == "missing_from_catalogue")
            .collect();
        assert_eq!(extra.len(), 1);
        assert_eq!(extra[0].path, "ds_extra");
    }

    // Validates: Requirement 12.4 -- reconcile returns empty when state matches
    #[test]
    fn reconcile_returns_empty_when_state_matches() {
        let mut entries = HashMap::new();
        entries.insert("ds1".to_string(), 0x1234_u32);
        entries.insert("ds2".to_string(), 0x5678_u32);

        let findings = workspace_reconcile(&entries, &entries);
        assert!(findings.is_empty());
    }

    // Validates: Requirement 12.2 -- manifest serialise/parse round-trip
    #[test]
    fn manifest_serialise_parse_round_trip() {
        let mut m = BackupManifest::new(1);
        m.providers
            .insert("posix".to_string(), "native".to_string());
        m.entries.push(ManifestEntry {
            archive_path: "file_txt".to_string(),
            original_path: "/tmp/file.txt".to_string(),
            checksum: 12345,
            size: 42,
        });

        let text = m.serialise();
        let parsed = BackupManifest::parse(&text).unwrap();

        assert_eq!(parsed.schema_version, 1);
        assert_eq!(
            parsed.providers.get("posix").map(|s| s.as_str()),
            Some("native")
        );
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].checksum, 12345);
        assert_eq!(parsed.entries[0].size, 42);
    }

    // Validates: Requirement 12.1 -- backup handles empty source root
    #[test]
    fn backup_empty_source_root_produces_empty_manifest() {
        let src = TempDir::new().unwrap();
        let archive = TempDir::new().unwrap();
        let manifest = workspace_backup(src.path(), archive.path(), &HashMap::new()).unwrap();
        assert!(manifest.entries.is_empty());
        assert!(archive.path().join("manifest.txt").exists());
    }

    // Validates: Requirement 12.1 -- backup recurses into subdirectories
    #[test]
    fn backup_recurses_into_subdirectories() {
        let src = TempDir::new().unwrap();
        let sub = src.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("nested.txt"), b"nested").unwrap();

        let archive = TempDir::new().unwrap();
        let manifest = workspace_backup(src.path(), archive.path(), &HashMap::new()).unwrap();

        assert_eq!(manifest.entries.len(), 1);
        assert!(manifest.entries[0].archive_path.contains("nested"));
    }
}
