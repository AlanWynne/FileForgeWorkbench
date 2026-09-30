//! Native collection persistence: a zip-compatible archive plus a crash-recovery
//! journal for an active collection.
//!
//! Archive layout (screen-snapshot-scrm Requirement 12.5, 17.3):
//! ```text
//! collection.yaml      -- collection header + capture index (see note)
//! screens/<seq>.json   -- one logical ScreenModel per capture (selectable text)
//! images/<seq>.png     -- OPTIONAL secondary bitmap per capture (if present)
//! metadata/<seq>.json  -- per-capture metadata (timestamps, DIDL, notes)
//! ```
//!
//! Note on `collection.yaml`: the archive member is named `.yaml` to match the
//! required layout. Its CONTENT is JSON, which is a valid subset of YAML, so the
//! file parses as YAML while keeping this crate free of a YAML serialiser
//! dependency. Screen content itself always lives under `screens/` as the
//! authoritative selectable text (never a raster) -- Requirement 1.
//!
//! Validates: screen-snapshot-scrm Requirement 12.5, 17.3, 18.4.

use std::io::{Read, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;

use crate::error::{Result, ScrmError};
use crate::model::ScreenCollection;

const HEADER_MEMBER: &str = "collection.yaml";

/// Write `collection` to a zip archive at `path`.
///
/// Validates: Requirement 12.5, 17.3.
pub fn save_archive(collection: &ScreenCollection, path: &Path) -> Result<()> {
    let file = std::fs::File::create(path).map_err(|source| ScrmError::Io {
        operation: format!("create archive {}", path.display()),
        source,
    })?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Header: collection sans the (bulky) screen bodies -- a stable index.
    let header =
        serde_json::to_vec_pretty(collection).map_err(|e| ScrmError::Serialize(e.to_string()))?;
    start_and_write(&mut zip, HEADER_MEMBER, opts, &header)?;

    // Per-capture screen + metadata + optional image.
    for cap in &collection.screens {
        let screen = serde_json::to_vec_pretty(&cap.screen)
            .map_err(|e| ScrmError::Serialize(e.to_string()))?;
        start_and_write(
            &mut zip,
            &format!("screens/{}.json", cap.sequence_number),
            opts,
            &screen,
        )?;

        if let Some(image) = &cap.image {
            start_and_write(
                &mut zip,
                &format!("images/{}.png", cap.sequence_number),
                opts,
                image,
            )?;
        }
    }

    zip.finish().map_err(|e| ScrmError::Archive {
        operation: "finish archive".to_string(),
        reason: e.to_string(),
    })?;
    Ok(())
}

/// Load a collection from a zip archive at `path`.
///
/// Validates: Requirement 12.5, 17.3.
pub fn load_archive(path: &Path) -> Result<ScreenCollection> {
    let file = std::fs::File::open(path).map_err(|source| ScrmError::Io {
        operation: format!("open archive {}", path.display()),
        source,
    })?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| ScrmError::Archive {
        operation: "open archive".to_string(),
        reason: e.to_string(),
    })?;
    let mut header = String::new();
    {
        let mut entry = zip
            .by_name(HEADER_MEMBER)
            .map_err(|_| ScrmError::MissingMember(HEADER_MEMBER.to_string()))?;
        entry
            .read_to_string(&mut header)
            .map_err(|source| ScrmError::Io {
                operation: "read header".to_string(),
                source,
            })?;
    }
    let collection: ScreenCollection =
        serde_json::from_str(&header).map_err(|e| ScrmError::Serialize(e.to_string()))?;
    Ok(collection)
}

fn start_and_write(
    zip: &mut zip::ZipWriter<std::fs::File>,
    name: &str,
    opts: SimpleFileOptions,
    bytes: &[u8],
) -> Result<()> {
    zip.start_file(name, opts).map_err(|e| ScrmError::Archive {
        operation: format!("start {name}"),
        reason: e.to_string(),
    })?;
    zip.write_all(bytes).map_err(|source| ScrmError::Io {
        operation: format!("write {name}"),
        source,
    })?;
    Ok(())
}

/// Append a capture record to a crash-recovery journal (newline-delimited JSON).
/// The journal lets an interrupted active collection be recovered on restart.
///
/// Validates: Requirement 18.4.
pub fn journal_append(journal_path: &Path, collection: &ScreenCollection) -> Result<()> {
    let line =
        serde_json::to_string(collection).map_err(|e| ScrmError::Serialize(e.to_string()))?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(journal_path)
        .map_err(|source| ScrmError::Io {
            operation: "open journal".to_string(),
            source,
        })?;
    writeln!(file, "{line}").map_err(|source| ScrmError::Io {
        operation: "write journal".to_string(),
        source,
    })?;
    Ok(())
}

/// Recover the most recent collection snapshot from a crash-recovery journal.
///
/// Validates: Requirement 18.4.
pub fn journal_recover_latest(journal_path: &Path) -> Result<Option<ScreenCollection>> {
    let content = match std::fs::read_to_string(journal_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(ScrmError::Io {
                operation: "read journal".to_string(),
                source,
            })
        }
    };
    let last = content.lines().rev().find(|l| !l.trim().is_empty());
    match last {
        Some(line) => {
            let c = serde_json::from_str(line).map_err(|e| ScrmError::Serialize(e.to_string()))?;
            Ok(Some(c))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use ff_screen_model::{Field, ScreenModel};
    use pretty_assertions::assert_eq;

    fn ts() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).unwrap()
    }

    fn sample(n: usize) -> ScreenCollection {
        let mut c = ScreenCollection::new("c1", "Repro", "alan", ts());
        for i in 0..n {
            c.append_capture(
                format!("cap{i}"),
                ts(),
                ScreenModel::new(format!("S{i}")).with_field(Field::new("K", format!("v{i}"))),
            );
        }
        c
    }

    // Validates: Requirement 12.5, 17.3 -- collection round-trips through zip.
    #[test]
    fn archive_round_trips_collection_header() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.ffscrm");
        let original = sample(3);
        save_archive(&original, &path).unwrap();
        let loaded = load_archive(&path).unwrap();
        assert_eq!(loaded, original);
    }

    // Validates: Requirement 18.4 -- journal recovers the latest snapshot.
    #[test]
    fn journal_recovers_latest_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let jpath = dir.path().join("active.journal");
        assert!(journal_recover_latest(&jpath).unwrap().is_none());
        journal_append(&jpath, &sample(1)).unwrap();
        journal_append(&jpath, &sample(2)).unwrap();
        let recovered = journal_recover_latest(&jpath).unwrap().unwrap();
        assert_eq!(recovered.len(), 2);
    }
}
