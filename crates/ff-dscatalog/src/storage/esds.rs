//! Native append-only storage for VSAM entry-sequenced datasets.
//!
//! An insert frame's byte offset is its stable record address. Updates append
//! replacement frames and deletes append tombstones; old frames are untouched.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use uuid::Uuid;

use super::{ObjectId, ObjectStat, ProviderCapability, StorageProvider};
use crate::error::CatalogError;

const OBJECTS_DIR: &str = "datasets/objects";
const DATA_SUFFIX: &str = ".esds";
const DATA_MAGIC: &[u8; 8] = b"FFWESDS1";
const INDEX_MAGIC: &[u8; 8] = b"FFWESIDX";
const FRAME_HEADER_LEN: u64 = 17;

#[derive(Debug, Clone, PartialEq, Eq)]
struct IndexEntry {
    latest_offset: u64,
    data_len: u64,
    active: bool,
}

#[derive(Debug)]
struct EsdsState {
    file: File,
    records: BTreeMap<u64, IndexEntry>,
}

/// A stable byte-offset address and the latest visible payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EsdsRecord {
    pub address: u64,
    pub data: Vec<u8>,
}

/// The stable address returned by [`NativeEsdsProvider::append`].
pub type EsdsRecordAddress = u64;

/// Native-file provider for VSAM ESDS datasets.
pub struct NativeEsdsProvider {
    data_path: PathBuf,
    index_path: PathBuf,
    dataset_id: Uuid,
    state: Mutex<EsdsState>,
}

impl std::fmt::Debug for NativeEsdsProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativeEsdsProvider")
            .field("data_path", &self.data_path)
            .field("index_path", &self.index_path)
            .field("dataset_id", &self.dataset_id)
            .finish_non_exhaustive()
    }
}

impl NativeEsdsProvider {
    /// Open or create an ESDS and rebuild its sidecar index from the data file.
    pub fn open(repository_root: impl AsRef<Path>, dataset_id: Uuid) -> Result<Self, CatalogError> {
        let data_path = data_path(repository_root.as_ref(), dataset_id)?;
        let index_path = index_path(&data_path);
        fs::create_dir_all(data_path.parent().expect("ESDS path has a parent"))
            .map_err(|source| io_error("open_esds", source))?;
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&data_path)
            .map_err(|source| io_error("open_esds", source))?;
        let records = scan_data_file(&mut file, &data_path)?;
        let length = file
            .metadata()
            .map_err(|source| io_error("open_esds", source))?
            .len();
        write_sidecar(&index_path, length, &records)?;
        file.seek(SeekFrom::End(0))
            .map_err(|source| io_error("open_esds", source))?;
        Ok(Self {
            data_path,
            index_path,
            dataset_id,
            state: Mutex::new(EsdsState { file, records }),
        })
    }

    /// Open an existing ESDS without creating a missing data file.
    pub fn open_existing(
        repository_root: impl AsRef<Path>,
        dataset_id: Uuid,
    ) -> Result<Self, CatalogError> {
        let path = data_path(repository_root.as_ref(), dataset_id)?;
        if !path.is_file() {
            return Err(CatalogError::DatasetNotFound {
                dsn: dataset_id.to_string(),
                operation: "open_esds".into(),
            });
        }
        Self::open(repository_root, dataset_id)
    }

    pub fn dataset_id(&self) -> Uuid {
        self.dataset_id
    }
    pub fn data_path(&self) -> &Path {
        &self.data_path
    }
    pub fn index_path(&self) -> &Path {
        &self.index_path
    }

    /// Append a record and return its stable byte-offset address.
    pub fn append(&self, data: &[u8]) -> Result<EsdsRecordAddress, CatalogError> {
        let mut state = self.state("append_esds")?;
        let address = state
            .file
            .seek(SeekFrom::End(0))
            .map_err(|source| io_error("append_esds", source))?;
        write_frame(&mut state.file, 0, address, data)
            .map_err(|source| io_error("append_esds", source))?;
        state
            .file
            .sync_data()
            .map_err(|source| io_error("append_esds", source))?;
        state.records.insert(
            address,
            IndexEntry {
                latest_offset: address,
                data_len: data.len() as u64,
                active: true,
            },
        );
        self.persist_index(&state, "append_esds")?;
        Ok(address)
    }

    /// Read the latest active version at a stable address.
    pub fn read(&self, address: EsdsRecordAddress) -> Result<Option<EsdsRecord>, CatalogError> {
        let state = self.state("read_esds")?;
        let Some(entry) = state.records.get(&address) else {
            return Ok(None);
        };
        if !entry.active {
            return Ok(None);
        }
        let data = read_payload(&state.file, entry.latest_offset, entry.data_len)
            .map_err(|source| io_error("read_esds", source))?;
        Ok(Some(EsdsRecord { address, data }))
    }

    /// Append a replacement while retaining the original address.
    /// Returns `false` for an unknown or deleted address.
    pub fn update(&self, address: EsdsRecordAddress, data: &[u8]) -> Result<bool, CatalogError> {
        let mut state = self.state("update_esds")?;
        if !state
            .records
            .get(&address)
            .is_some_and(|entry| entry.active)
        {
            return Ok(false);
        }
        let offset = append_frame(&mut state.file, 1, address, data)
            .map_err(|source| io_error("update_esds", source))?;
        state
            .file
            .sync_data()
            .map_err(|source| io_error("update_esds", source))?;
        state.records.insert(
            address,
            IndexEntry {
                latest_offset: offset,
                data_len: data.len() as u64,
                active: true,
            },
        );
        self.persist_index(&state, "update_esds")?;
        Ok(true)
    }

    /// Append a tombstone; the address is never reused.
    /// Returns `false` for an unknown or already deleted address.
    pub fn delete_record(&self, address: EsdsRecordAddress) -> Result<bool, CatalogError> {
        let mut state = self.state("delete_esds")?;
        if !state
            .records
            .get(&address)
            .is_some_and(|entry| entry.active)
        {
            return Ok(false);
        }
        let offset = append_frame(&mut state.file, 2, address, &[])
            .map_err(|source| io_error("delete_esds", source))?;
        state
            .file
            .sync_data()
            .map_err(|source| io_error("delete_esds", source))?;
        state.records.insert(
            address,
            IndexEntry {
                latest_offset: offset,
                data_len: 0,
                active: false,
            },
        );
        self.persist_index(&state, "delete_esds")?;
        Ok(true)
    }

    pub fn append_record(&self, data: &[u8]) -> Result<EsdsRecordAddress, CatalogError> {
        self.append(data)
    }
    pub fn read_record(
        &self,
        address: EsdsRecordAddress,
    ) -> Result<Option<EsdsRecord>, CatalogError> {
        self.read(address)
    }
    pub fn update_record(
        &self,
        address: EsdsRecordAddress,
        data: &[u8],
    ) -> Result<bool, CatalogError> {
        self.update(address, data)
    }

    /// Read active records in original insertion order.
    pub fn sequential_read(&self) -> Result<Vec<EsdsRecord>, CatalogError> {
        let state = self.state("sequential_esds")?;
        state
            .records
            .iter()
            .filter(|(_, entry)| entry.active)
            .map(|(&address, entry)| {
                read_payload(&state.file, entry.latest_offset, entry.data_len)
                    .map(|data| EsdsRecord { address, data })
                    .map_err(|source| io_error("sequential_esds", source))
            })
            .collect()
    }

    pub fn records(&self) -> Result<Vec<EsdsRecord>, CatalogError> {
        self.sequential_read()
    }

    pub fn len(&self) -> Result<usize, CatalogError> {
        Ok(self
            .state("count_esds")?
            .records
            .values()
            .filter(|entry| entry.active)
            .count())
    }

    pub fn is_empty(&self) -> Result<bool, CatalogError> {
        Ok(self.len()? == 0)
    }

    fn state(&self, operation: &'static str) -> Result<MutexGuard<'_, EsdsState>, CatalogError> {
        self.state
            .lock()
            .map_err(|_| CatalogError::RepositoryCorrupt {
                path: self.data_path.display().to_string(),
                reason: "ESDS file mutex is poisoned".into(),
                operation: operation.into(),
            })
    }

    fn persist_index(&self, state: &EsdsState, operation: &str) -> Result<(), CatalogError> {
        let length = state
            .file
            .metadata()
            .map_err(|source| io_error(operation, source))?
            .len();
        write_sidecar(&self.index_path, length, &state.records)
    }
}

impl StorageProvider for NativeEsdsProvider {
    fn capabilities(&self) -> &[ProviderCapability] {
        static CAPABILITIES: &[ProviderCapability] = &[
            ProviderCapability::RecordRead,
            ProviderCapability::RecordWrite,
            ProviderCapability::AppendOnly,
        ];
        CAPABILITIES
    }

    fn allocate(
        &self,
        workspace_root: &Path,
        _is_container: bool,
    ) -> Result<(ObjectId, String), CatalogError> {
        let id = Uuid::new_v4();
        Self::open(workspace_root, id)?;
        Ok((id, locator(id)))
    }

    fn open(&self, workspace_root: &Path, locator: &str) -> Result<PathBuf, CatalogError> {
        let path = validate_locator(workspace_root, locator)?;
        if path.is_file() {
            Ok(path)
        } else {
            Err(CatalogError::DatasetNotFound {
                dsn: locator.into(),
                operation: "open_esds".into(),
            })
        }
    }

    fn stat(&self, workspace_root: &Path, locator: &str) -> Result<ObjectStat, CatalogError> {
        let path = self.open(workspace_root, locator)?;
        let size = fs::metadata(path)
            .map_err(|source| io_error("stat_esds", source))?
            .len();
        Ok(ObjectStat {
            size,
            is_container: false,
            locator: locator.into(),
        })
    }

    fn rename(
        &self,
        _workspace_root: &Path,
        _locator: &str,
        _new_locator: &str,
    ) -> Result<(), CatalogError> {
        Ok(())
    }

    fn delete(&self, workspace_root: &Path, locator: &str) -> Result<(), CatalogError> {
        let path = self.open(workspace_root, locator)?;
        fs::remove_file(&path).map_err(|source| io_error("delete_esds", source))?;
        let sidecar = index_path(&path);
        if sidecar.exists() {
            fs::remove_file(sidecar).map_err(|source| io_error("delete_esds", source))?;
        }
        Ok(())
    }

    fn list(&self, workspace_root: &Path, _locator: &str) -> Result<Vec<String>, CatalogError> {
        let directory = workspace_root.join(OBJECTS_DIR);
        if !directory.is_dir() {
            return Ok(Vec::new());
        }
        let mut result = Vec::new();
        for entry in fs::read_dir(directory).map_err(|source| io_error("list_esds", source))? {
            let entry = entry.map_err(|source| io_error("list_esds", source))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(DATA_SUFFIX)
                && Uuid::parse_str(name.trim_end_matches(DATA_SUFFIX)).is_ok()
            {
                result.push(format!("{OBJECTS_DIR}/{name}"));
            }
        }
        result.sort();
        Ok(result)
    }

    fn reconcile(
        &self,
        workspace_root: &Path,
        known_locators: &[String],
    ) -> Result<Vec<String>, CatalogError> {
        let mut discrepancies = Vec::new();
        for locator in known_locators {
            match validate_locator(workspace_root, locator) {
                Ok(path) if !path.is_file() => {
                    discrepancies.push(format!("missing physical object for locator '{locator}'"))
                }
                Err(error) => discrepancies.push(format!("invalid locator '{locator}': {error}")),
                Ok(_) => {}
            }
        }
        Ok(discrepancies)
    }
}

fn locator(id: Uuid) -> String {
    format!("{OBJECTS_DIR}/{id}{DATA_SUFFIX}")
}

fn data_path(root: &Path, id: Uuid) -> Result<PathBuf, CatalogError> {
    validate_locator(root, &locator(id))
}

fn index_path(data_path: &Path) -> PathBuf {
    let mut path = data_path.to_path_buf();
    path.set_extension("idx");
    path
}

fn validate_locator(root: &Path, locator: &str) -> Result<PathBuf, CatalogError> {
    let relative = Path::new(locator);
    let mut components = relative.components();
    if components.next() != Some(std::path::Component::Normal("datasets".as_ref()))
        || components.next() != Some(std::path::Component::Normal("objects".as_ref()))
        || components.next().is_none()
        || components.next().is_some()
    {
        return Err(corrupt_locator(
            locator,
            "ESDS locator must be datasets/objects/<uuid>.esds",
        ));
    }
    let filename = relative
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let uuid_text = filename.strip_suffix(DATA_SUFFIX).unwrap_or_default();
    if Uuid::parse_str(uuid_text).is_err() {
        return Err(corrupt_locator(locator, "ESDS locator must contain a UUID"));
    }
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let path = root.join(relative);
    if !path.starts_with(&root) {
        return Err(corrupt_locator(
            locator,
            "path traversal outside workspace root rejected",
        ));
    }
    Ok(path)
}

fn scan_data_file(file: &mut File, path: &Path) -> Result<BTreeMap<u64, IndexEntry>, CatalogError> {
    file.seek(SeekFrom::Start(0))
        .map_err(|source| io_error("scan_esds", source))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|source| io_error("scan_esds", source))?;
    if bytes.len() < DATA_MAGIC.len() {
        if bytes.is_empty() {
            file.seek(SeekFrom::Start(0))
                .and_then(|_| file.write_all(DATA_MAGIC))
                .and_then(|_| file.sync_data())
                .map_err(|source| io_error("initialize_esds", source))?;
            return Ok(BTreeMap::new());
        }
        return Err(corrupt_file(path, "truncated ESDS header"));
    }
    if &bytes[..DATA_MAGIC.len()] != DATA_MAGIC {
        return Err(corrupt_file(path, "invalid ESDS header"));
    }
    let mut records = BTreeMap::new();
    let mut cursor = DATA_MAGIC.len() as u64;
    while cursor < bytes.len() as u64 {
        let start = cursor;
        let end = cursor
            .checked_add(FRAME_HEADER_LEN)
            .ok_or_else(|| corrupt_file(path, "ESDS frame offset overflow"))?;
        if end > bytes.len() as u64 {
            return Err(corrupt_file(path, "truncated ESDS frame header"));
        }
        let kind = bytes[cursor as usize];
        let token = read_u64(&bytes, cursor as usize + 1);
        let payload_len = read_u64(&bytes, cursor as usize + 9);
        let payload_end = end
            .checked_add(payload_len)
            .ok_or_else(|| corrupt_file(path, "ESDS payload length overflow"))?;
        if payload_end > bytes.len() as u64 {
            return Err(corrupt_file(path, "truncated ESDS frame payload"));
        }
        match kind {
            0 if token == start && !records.contains_key(&token) => {
                records.insert(
                    token,
                    IndexEntry {
                        latest_offset: start,
                        data_len: payload_len,
                        active: true,
                    },
                );
            }
            1 if records.get(&token).is_some_and(|entry| entry.active) => {
                records.insert(
                    token,
                    IndexEntry {
                        latest_offset: start,
                        data_len: payload_len,
                        active: true,
                    },
                );
            }
            2 if payload_len == 0 && records.get(&token).is_some_and(|entry| entry.active) => {
                records.insert(
                    token,
                    IndexEntry {
                        latest_offset: start,
                        data_len: 0,
                        active: false,
                    },
                );
            }
            _ => return Err(corrupt_file(path, "invalid ESDS frame or record address")),
        }
        cursor = payload_end;
    }
    Ok(records)
}

fn append_frame(file: &mut File, kind: u8, token: u64, data: &[u8]) -> io::Result<u64> {
    let offset = file.seek(SeekFrom::End(0))?;
    write_frame(file, kind, token, data)?;
    Ok(offset)
}

fn write_frame(file: &mut File, kind: u8, token: u64, data: &[u8]) -> io::Result<()> {
    file.write_all(&[kind])?;
    file.write_all(&token.to_le_bytes())?;
    file.write_all(&(data.len() as u64).to_le_bytes())?;
    file.write_all(data)
}

fn read_payload(file: &File, offset: u64, data_len: u64) -> io::Result<Vec<u8>> {
    let mut file = file.try_clone()?;
    file.seek(SeekFrom::Start(offset + FRAME_HEADER_LEN))?;
    let mut data = vec![
        0;
        usize::try_from(data_len).map_err(|_| io::Error::new(
            io::ErrorKind::InvalidData,
            "ESDS record is too large"
        ))?
    ];
    file.read_exact(&mut data)?;
    Ok(data)
}

fn write_sidecar(
    path: &Path,
    data_len: u64,
    records: &BTreeMap<u64, IndexEntry>,
) -> Result<(), CatalogError> {
    let temporary = path.with_extension("idx.tmp");
    let _ = fs::remove_file(&temporary);
    let result = (|| -> io::Result<()> {
        let mut file = File::create(&temporary)?;
        file.write_all(INDEX_MAGIC)?;
        file.write_all(&data_len.to_le_bytes())?;
        file.write_all(&(records.len() as u64).to_le_bytes())?;
        for (&address, entry) in records {
            file.write_all(&address.to_le_bytes())?;
            file.write_all(&entry.latest_offset.to_le_bytes())?;
            file.write_all(&entry.data_len.to_le_bytes())?;
            file.write_all(&[u8::from(entry.active)])?;
        }
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    result.map_err(|source| io_error("write_esds_index", source))
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("checked frame bounds"),
    )
}

fn corrupt_locator(locator: &str, reason: &str) -> CatalogError {
    CatalogError::RepositoryCorrupt {
        path: locator.into(),
        reason: reason.into(),
        operation: "resolve_esds_path".into(),
    }
}

fn corrupt_file(path: &Path, reason: &str) -> CatalogError {
    CatalogError::RepositoryCorrupt {
        path: path.display().to_string(),
        reason: reason.into(),
        operation: "scan_esds".into(),
    }
}

fn io_error(operation: &str, source: io::Error) -> CatalogError {
    CatalogError::IoError {
        operation: operation.into(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn provider() -> (TempDir, NativeEsdsProvider) {
        let directory = tempfile::tempdir().expect("tempdir");
        let provider = NativeEsdsProvider::open(directory.path(), Uuid::new_v4()).unwrap();
        (directory, provider)
    }

    #[test]
    fn appends_records_in_insertion_order() {
        let (_directory, provider) = provider();
        let first = provider.append(b"one").unwrap();
        let second = provider.append(b"two").unwrap();
        assert!(second > first);
        assert_eq!(
            provider.sequential_read().unwrap(),
            vec![
                EsdsRecord {
                    address: first,
                    data: b"one".to_vec()
                },
                EsdsRecord {
                    address: second,
                    data: b"two".to_vec()
                },
            ]
        );
    }

    #[test]
    fn addresses_remain_stable_across_updates_and_reopen() {
        let (directory, provider) = provider();
        let address = provider.append(b"before").unwrap();
        let second = provider.append(b"second").unwrap();
        assert!(provider.update(address, b"after").unwrap());
        assert_eq!(provider.read(address).unwrap().unwrap().data, b"after");
        let id = provider.dataset_id();
        drop(provider);
        let reopened = NativeEsdsProvider::open_existing(directory.path(), id).unwrap();
        assert_eq!(reopened.read(address).unwrap().unwrap().data, b"after");
        assert_eq!(reopened.read(second).unwrap().unwrap().data, b"second");
    }

    #[test]
    fn rebuilds_sidecar_index_from_data_file() {
        let (directory, provider) = provider();
        let address = provider.append(b"value").unwrap();
        let index_path = provider.index_path().to_path_buf();
        let id = provider.dataset_id();
        drop(provider);
        fs::remove_file(index_path).unwrap();
        let reopened = NativeEsdsProvider::open_existing(directory.path(), id).unwrap();
        assert!(reopened.index_path().is_file());
        assert_eq!(reopened.read(address).unwrap().unwrap().data, b"value");
    }

    #[test]
    fn rejects_truncated_data_during_open() {
        let (directory, provider) = provider();
        let id = provider.dataset_id();
        let data_path = provider.data_path().to_path_buf();
        drop(provider);
        OpenOptions::new()
            .append(true)
            .open(data_path)
            .unwrap()
            .write_all(&[0xff])
            .unwrap();
        assert!(matches!(
            NativeEsdsProvider::open_existing(directory.path(), id),
            Err(CatalogError::RepositoryCorrupt { .. })
        ));
    }

    #[test]
    fn updates_append_and_deletes_tombstone() {
        let (_directory, provider) = provider();
        let address = provider.append(b"one").unwrap();
        let original_len = fs::metadata(provider.data_path()).unwrap().len();
        assert!(provider.update(address, b"updated").unwrap());
        assert!(fs::metadata(provider.data_path()).unwrap().len() > original_len);
        assert!(provider.delete_record(address).unwrap());
        assert!(provider.read(address).unwrap().is_none());
        assert!(!provider.delete_record(address).unwrap());
    }

    #[test]
    fn rejects_invalid_locators() {
        let (directory, provider) = provider();
        assert!(matches!(
            provider.open(directory.path(), "../../outside.esds"),
            Err(CatalogError::RepositoryCorrupt { .. })
        ));
        assert!(matches!(
            provider.open(directory.path(), "datasets/objects/not-a-uuid.esds"),
            Err(CatalogError::RepositoryCorrupt { .. })
        ));
    }

    #[test]
    fn capabilities_advertise_append_only_record_access() {
        let (_directory, provider) = provider();
        assert!(provider
            .capabilities()
            .contains(&ProviderCapability::AppendOnly));
        assert!(provider
            .capabilities()
            .contains(&ProviderCapability::RecordRead));
        assert!(provider
            .capabilities()
            .contains(&ProviderCapability::RecordWrite));
    }
}
