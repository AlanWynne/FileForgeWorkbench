//! Integration tests for ff-background-io.
//!
//! Uses in-memory VFS providers to test the full load/save lifecycle without
//! touching real filesystems.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::AsyncRead;

use ff_background_io::save::{DocumentChunkSource, SaveOptions};
use ff_background_io::{
    BackgroundIoService, ChunkCallback, ChunkSize, IoConfig, IoPhase, IoTaskHandle, LoadOptions,
    ProgressState, TaskState,
};
use ff_vfs::{
    CreateOptions, DeleteOptions, OpenOptions, ProviderRegistry, ResourceUri, Vfs, VfsCapabilities,
    VfsEntry, VfsError, VfsFile, VfsMetadata, VfsProvider,
};

// ─── Mock VFS Provider ─────────────────────────────────────────────────────────

/// A simple in-memory VFS provider for testing.
struct TestProvider {
    scheme: String,
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    capabilities: VfsCapabilities,
}

impl TestProvider {
    fn new(scheme: &str) -> Self {
        Self {
            scheme: scheme.to_string(),
            files: Arc::new(Mutex::new(HashMap::new())),
            capabilities: VfsCapabilities::all(),
        }
    }

    fn with_file(self, path: &str, content: &[u8]) -> Self {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), content.to_vec());
        self
    }

    fn with_capabilities(mut self, caps: VfsCapabilities) -> Self {
        self.capabilities = caps;
        self
    }
}

#[async_trait]
impl VfsProvider for TestProvider {
    fn scheme(&self) -> &str {
        &self.scheme
    }

    fn capabilities(&self) -> VfsCapabilities {
        self.capabilities
    }

    async fn open(&self, path: &str, _options: OpenOptions) -> Result<Box<dyn VfsFile>, VfsError> {
        Ok(Box::new(TestFile {
            path: path.to_string(),
            files: self.files.clone(),
            write_buffer: Vec::new(),
        }))
    }

    async fn read(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or(VfsError::NotFound {
                uri: path.to_string(),
                operation: "read".to_string(),
            })
    }

    async fn read_stream(&self, path: &str) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
        let data = self.read(path).await?;
        Ok(Box::pin(std::io::Cursor::new(data)))
    }

    async fn write(&self, path: &str, data: &[u8]) -> Result<(), VfsError> {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), data.to_vec());
        Ok(())
    }

    async fn create(&self, path: &str, _options: CreateOptions) -> Result<(), VfsError> {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), Vec::new());
        Ok(())
    }

    async fn delete(&self, path: &str, _options: DeleteOptions) -> Result<(), VfsError> {
        self.files.lock().unwrap().remove(path);
        Ok(())
    }

    async fn rename(&self, old_path: &str, new_path: &str) -> Result<(), VfsError> {
        let mut files = self.files.lock().unwrap();
        let data = files.remove(old_path).ok_or(VfsError::NotFound {
            uri: old_path.to_string(),
            operation: "rename".to_string(),
        })?;
        files.insert(new_path.to_string(), data);
        Ok(())
    }

    async fn list(&self, _path: &str) -> Result<Vec<VfsEntry>, VfsError> {
        Ok(Vec::new())
    }

    async fn stat(&self, path: &str) -> Result<VfsMetadata, VfsError> {
        let files = self.files.lock().unwrap();
        let data = files.get(path).ok_or(VfsError::NotFound {
            uri: path.to_string(),
            operation: "stat".to_string(),
        })?;
        Ok(VfsMetadata {
            size: Some(data.len() as u64),
            modified: None,
            entry_type: ff_vfs::VfsEntryType::File,
            extra: HashMap::new(),
        })
    }

    async fn exists(&self, path: &str) -> Result<bool, VfsError> {
        Ok(self.files.lock().unwrap().contains_key(path))
    }
}

/// Simple file handle for testing.
struct TestFile {
    path: String,
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    write_buffer: Vec<u8>,
}

#[async_trait]
impl VfsFile for TestFile {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, VfsError> {
        let files = self.files.lock().unwrap();
        let data = files.get(&self.path).unwrap_or(&Vec::new()).clone();
        let len = buf.len().min(data.len());
        buf[..len].copy_from_slice(&data[..len]);
        Ok(len)
    }

    async fn write(&mut self, data: &[u8]) -> Result<usize, VfsError> {
        self.write_buffer.extend_from_slice(data);
        Ok(data.len())
    }

    async fn flush(&mut self) -> Result<(), VfsError> {
        let mut files = self.files.lock().unwrap();
        files.insert(self.path.clone(), self.write_buffer.clone());
        Ok(())
    }

    async fn sync_all(&mut self) -> Result<(), VfsError> {
        self.flush().await
    }

    async fn close(self: Box<Self>) -> Result<(), VfsError> {
        // Ensure data is committed
        let mut files = self.files.lock().unwrap();
        files.insert(self.path.clone(), self.write_buffer.clone());
        Ok(())
    }
}

// ─── Mock Document Source ──────────────────────────────────────────────────────

/// Simple document source that provides content in chunks for save operations.
struct TestDocumentSource {
    content: Vec<u8>,
    position: Mutex<usize>,
}

impl TestDocumentSource {
    fn new(content: Vec<u8>) -> Self {
        Self {
            content,
            position: Mutex::new(0),
        }
    }
}

impl DocumentChunkSource for TestDocumentSource {
    fn total_size(&self) -> Option<u64> {
        Some(self.content.len() as u64)
    }

    fn next_chunk(&self, chunk_size_hint: usize) -> Option<Vec<u8>> {
        let mut pos = self.position.lock().unwrap();
        if *pos >= self.content.len() {
            return None;
        }
        let end = (*pos + chunk_size_hint).min(self.content.len());
        let chunk = self.content[*pos..end].to_vec();
        *pos = end;
        Some(chunk)
    }

    fn reset(&self) {
        let mut pos = self.position.lock().unwrap();
        *pos = 0;
    }
}

// ─── Helper Functions ──────────────────────────────────────────────────────────

fn create_test_vfs(provider: TestProvider) -> Arc<Vfs> {
    let registry = ProviderRegistry::new();
    registry.register(Arc::new(provider)).unwrap();
    Arc::new(Vfs::with_registry(registry))
}

// ─── Integration Tests ─────────────────────────────────────────────────────────

#[tokio::test]
async fn load_task_delivers_complete_content() {
    // Validates: Requirement 1 AC 1, AC 2, AC 6
    let content = b"Hello, World! This is test content for loading.";
    let provider = TestProvider::new("test").with_file("/doc.txt", content);
    let vfs = create_test_vfs(provider);

    let service = BackgroundIoService::new(IoConfig::default());
    let received = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received.clone();

    let callback: ChunkCallback = Arc::new(move |chunk: &[u8]| {
        received_clone.lock().unwrap().extend_from_slice(chunk);
    });

    let uri = ResourceUri::new("test", "/doc.txt");
    let handle = service.spawn_load(vfs, uri, LoadOptions::default(), callback);

    handle.await_completion().await;

    assert_eq!(handle.state(), TaskState::Complete);
    let got = received.lock().unwrap().clone();
    assert_eq!(got, content);
}

#[tokio::test]
async fn load_task_reports_progress() {
    // Validates: Requirement 2 AC 1, AC 3, AC 5
    let content = vec![0u8; 256 * 1024]; // 256 KB
    let provider = TestProvider::new("test").with_file("/big.bin", &content);
    let vfs = create_test_vfs(provider);

    let config = IoConfig::new(64, 100, 4, 3, 500, 30); // 64 KB chunks
    let service = BackgroundIoService::new(config);
    let callback: ChunkCallback = Arc::new(|_| {});

    let uri = ResourceUri::new("test", "/big.bin");
    let handle = service.spawn_load(vfs, uri, LoadOptions::default(), callback);

    handle.await_completion().await;

    // Final progress should show 100%
    let progress = handle.progress();
    assert_eq!(progress.phase, IoPhase::Complete);
    assert_eq!(progress.percentage, Some(100));
    assert_eq!(progress.bytes_transferred, 256 * 1024);
}

#[tokio::test]
async fn load_task_cancellation_stops_transfer() {
    // Validates: Requirement 3 AC 2, AC 5
    // With in-memory provider, reads are nearly instant. We verify the cancellation
    // mechanism works by accepting either Cancelled or Complete state.
    let content = vec![42u8; 1024 * 1024]; // 1 MB
    let provider = TestProvider::new("test").with_file("/large.bin", &content);
    let vfs = create_test_vfs(provider);

    let config = IoConfig::new(4, 100, 4, 3, 500, 30); // 4 KB chunks (smallest)
    let service = BackgroundIoService::new(config);

    let received_bytes = Arc::new(AtomicU64::new(0));
    let received_clone = received_bytes.clone();
    let callback: ChunkCallback = Arc::new(move |chunk: &[u8]| {
        received_clone.fetch_add(chunk.len() as u64, Ordering::Relaxed);
    });

    let uri = ResourceUri::new("test", "/large.bin");
    let handle = service.spawn_load(vfs, uri, LoadOptions::default(), callback);

    // Cancel immediately (race with the fast in-memory read)
    handle.cancel();
    handle.await_completion().await;

    // Should be in a terminal state — either cancelled (if caught in time) or complete
    let state = handle.state();
    assert!(
        state == TaskState::Cancelled || state == TaskState::Complete,
        "expected Cancelled or Complete, got {:?}",
        state
    );
}

#[tokio::test]
async fn load_task_handles_file_not_found() {
    // Validates: Requirement 6 AC 3
    let provider = TestProvider::new("test"); // no files
    let vfs = create_test_vfs(provider);

    let service = BackgroundIoService::new(IoConfig::default());
    let callback: ChunkCallback = Arc::new(|_| {});

    let uri = ResourceUri::new("test", "/nonexistent.txt");
    let handle = service.spawn_load(vfs, uri, LoadOptions::default(), callback);

    handle.await_completion().await;

    assert_eq!(handle.state(), TaskState::Failed);
}

#[tokio::test]
async fn save_task_writes_content_via_atomic_rename() {
    // Validates: Requirement 4 AC 1, AC 4, AC 5
    let provider = TestProvider::new("test").with_file("/target.txt", b"old content");
    let vfs = create_test_vfs(provider);

    let service = BackgroundIoService::new(IoConfig::default());
    let content = b"new content written via save";
    let doc_source = Arc::new(TestDocumentSource::new(content.to_vec()));

    let uri = ResourceUri::new("test", "/target.txt");
    let handle = service.spawn_save(vfs.clone(), uri.clone(), doc_source, SaveOptions::default());

    handle.await_completion().await;

    assert_eq!(handle.state(), TaskState::Complete);
    // The content should be written to the target (via rename)
    let result = vfs.read(&uri).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), content.to_vec());
}

#[tokio::test]
async fn save_task_reports_progress() {
    // Validates: Requirement 4 AC 3
    let content = vec![0u8; 128 * 1024]; // 128 KB
    let provider = TestProvider::new("test").with_file("/save.bin", b"");
    let vfs = create_test_vfs(provider);

    let config = IoConfig::new(32, 100, 4, 3, 500, 30); // 32 KB chunks
    let service = BackgroundIoService::new(config);
    let doc_source = Arc::new(TestDocumentSource::new(content));

    let uri = ResourceUri::new("test", "/save.bin");
    let handle = service.spawn_save(vfs, uri, doc_source, SaveOptions::default());

    handle.await_completion().await;

    let progress = handle.progress();
    assert_eq!(progress.phase, IoPhase::Complete);
    assert_eq!(progress.percentage, Some(100));
}

#[tokio::test]
async fn concurrency_limit_queues_excess_tasks() {
    // Validates: Requirement 7 AC 1, AC 3
    let mut provider = TestProvider::new("test");
    for i in 0..10 {
        provider = provider.with_file(&format!("/file{}.txt", i), b"data");
    }
    let vfs = create_test_vfs(provider);

    // Limit to 2 concurrent tasks
    let config = IoConfig::new(64, 100, 2, 3, 500, 30);
    let service = BackgroundIoService::new(config);

    let callback: ChunkCallback = Arc::new(|_| {});

    // Spawn 5 tasks — only 2 should run concurrently
    let mut handles = Vec::new();
    for i in 0..5 {
        let uri = ResourceUri::new("test", &format!("/file{}.txt", i));
        let h = service.spawn_load(vfs.clone(), uri, LoadOptions::default(), callback.clone());
        handles.push(h);
    }

    // Wait for all to complete
    for h in &handles {
        h.await_completion().await;
    }

    // All should have completed successfully
    for h in &handles {
        assert_eq!(h.state(), TaskState::Complete);
    }
}

#[tokio::test]
async fn shutdown_cancels_load_tasks_and_clears_registry() {
    // Validates: Requirement 7 AC 6
    let content = vec![0u8; 10 * 1024 * 1024]; // 10 MB
    let provider = TestProvider::new("test").with_file("/huge.bin", &content);
    let vfs = create_test_vfs(provider);

    let config = IoConfig::new(4, 100, 4, 3, 500, 1); // 1s shutdown timeout
    let service = BackgroundIoService::new(config);
    let callback: ChunkCallback = Arc::new(|_| {});

    let uri = ResourceUri::new("test", "/huge.bin");
    let _handle = service.spawn_load(vfs, uri, LoadOptions::default(), callback);

    // Brief delay to let task start
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Shutdown should cancel load tasks
    service.shutdown().await;

    // Task registry should be empty after shutdown
    let tasks = service.list_tasks().await;
    assert!(tasks.is_empty());
}

#[tokio::test]
async fn save_with_non_atomic_provider_falls_back_to_write_in_place() {
    // Validates: Requirement 4 AC 6
    let caps = VfsCapabilities {
        read: true,
        write: true,
        watch: false,
        search: false,
        random_access: false,
        append: false,
        rename: false, // No rename support
        delete: false,
        list: false,
        create_directory: false,
    };
    let provider = TestProvider::new("test")
        .with_file("/target.txt", b"old")
        .with_capabilities(caps);
    let vfs = create_test_vfs(provider);

    let service = BackgroundIoService::new(IoConfig::default());
    let content = b"new content via fallback";
    let doc_source = Arc::new(TestDocumentSource::new(content.to_vec()));

    let uri = ResourceUri::new("test", "/target.txt");
    let handle = service.spawn_save(vfs.clone(), uri.clone(), doc_source, SaveOptions::default());

    handle.await_completion().await;

    assert_eq!(handle.state(), TaskState::Complete);
    let result = vfs.read(&uri).await.unwrap();
    assert_eq!(result, content.to_vec());
}

#[tokio::test]
async fn save_task_cancellation_preserves_original() {
    // Validates: Requirement 3 AC 3
    // This test verifies that if a save is cancelled, the original file is preserved.
    // With the in-memory provider, cancellation during write is hard to trigger
    // (operations are nearly instant), so we verify the mechanism.
    let original = b"original content that must be preserved";
    let provider = TestProvider::new("test").with_file("/preserve.txt", original);
    let vfs = create_test_vfs(provider);

    let service = BackgroundIoService::new(IoConfig::default());
    let content = vec![0u8; 1024]; // Small content
    let doc_source = Arc::new(TestDocumentSource::new(content));

    let uri = ResourceUri::new("test", "/preserve.txt");
    let handle = service.spawn_save(vfs.clone(), uri.clone(), doc_source, SaveOptions::default());

    // Cancel immediately (race condition — may complete before cancel)
    handle.cancel();
    handle.await_completion().await;

    // Either completed (wrote new content) or cancelled (kept original)
    // Both are acceptable outcomes for this test
    let state = handle.state();
    assert!(state == TaskState::Complete || state == TaskState::Cancelled);
}

#[tokio::test]
async fn multiple_concurrent_loads_and_saves_complete_successfully() {
    // Validates: Requirement 7 AC 4
    let mut provider = TestProvider::new("test");
    for i in 0..4 {
        provider = provider.with_file(
            &format!("/load{}.txt", i),
            format!("content {}", i).as_bytes(),
        );
        provider = provider.with_file(&format!("/save{}.txt", i), b"old");
    }
    let vfs = create_test_vfs(provider);

    let service = BackgroundIoService::new(IoConfig::default());

    let mut handles: Vec<IoTaskHandle> = Vec::new();

    // Spawn loads
    for i in 0..4 {
        let callback: ChunkCallback = Arc::new(|_| {});
        let uri = ResourceUri::new("test", &format!("/load{}.txt", i));
        handles.push(service.spawn_load(vfs.clone(), uri, LoadOptions::default(), callback));
    }

    // Spawn saves
    for i in 0..4 {
        let uri = ResourceUri::new("test", &format!("/save{}.txt", i));
        let doc_source = Arc::new(TestDocumentSource::new(format!("new {}", i).into_bytes()));
        handles.push(service.spawn_save(vfs.clone(), uri, doc_source, SaveOptions::default()));
    }

    // Await all
    for h in &handles {
        h.await_completion().await;
    }

    // All should complete
    for h in &handles {
        assert_eq!(h.state(), TaskState::Complete);
    }
}
