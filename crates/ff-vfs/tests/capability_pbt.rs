// Feature: virtual-file-system, Property 5: Capability-gated operations
//!
//! Property-based tests verifying that the Vfs facade gates operations based on
//! provider-declared capabilities. When a provider declares `watch: false`, the
//! Vfs facade MUST return `VfsError::UnsupportedOperation` without delegating to
//! the provider's watch method.

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use proptest::prelude::*;
use tokio::io::AsyncRead;

use ff_vfs::error::VfsError;
use ff_vfs::provider::{VfsFile, VfsProvider};
use ff_vfs::registry::ProviderRegistry;
use ff_vfs::types::{
    CreateOptions, DeleteOptions, OpenOptions, VfsCapabilities, VfsEntry, VfsMetadata, WatchOptions,
};
use ff_vfs::uri::ResourceUri;
use ff_vfs::watch::WatchHandle;
use ff_vfs::Vfs;

/// A mock provider that returns the given capabilities and tracks whether
/// its `watch()` method was actually called. This allows the test to verify
/// that the facade gates the operation without delegating.
struct CapabilityMockProvider {
    caps: VfsCapabilities,
    watch_called: Arc<AtomicBool>,
}

impl CapabilityMockProvider {
    fn new(caps: VfsCapabilities, watch_called: Arc<AtomicBool>) -> Self {
        Self { caps, watch_called }
    }
}

#[async_trait]
impl VfsProvider for CapabilityMockProvider {
    fn scheme(&self) -> &str {
        "captest"
    }

    fn capabilities(&self) -> VfsCapabilities {
        self.caps
    }

    async fn open(&self, _path: &str, _options: OpenOptions) -> Result<Box<dyn VfsFile>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "open".to_string(),
            provider: "captest".to_string(),
        })
    }

    async fn read(&self, _path: &str) -> Result<Vec<u8>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "read".to_string(),
            provider: "captest".to_string(),
        })
    }

    async fn read_stream(&self, _path: &str) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "read_stream".to_string(),
            provider: "captest".to_string(),
        })
    }

    async fn write(&self, _path: &str, _data: &[u8]) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "write".to_string(),
            provider: "captest".to_string(),
        })
    }

    async fn create(&self, _path: &str, _options: CreateOptions) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "create".to_string(),
            provider: "captest".to_string(),
        })
    }

    async fn delete(&self, _path: &str, _options: DeleteOptions) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "delete".to_string(),
            provider: "captest".to_string(),
        })
    }

    async fn rename(&self, _old_path: &str, _new_path: &str) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "rename".to_string(),
            provider: "captest".to_string(),
        })
    }

    async fn list(&self, _path: &str) -> Result<Vec<VfsEntry>, VfsError> {
        Ok(Vec::new())
    }

    async fn stat(&self, _path: &str) -> Result<VfsMetadata, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "stat".to_string(),
            provider: "captest".to_string(),
        })
    }

    async fn exists(&self, _path: &str) -> Result<bool, VfsError> {
        Ok(false)
    }

    async fn watch(&self, _path: &str, _options: WatchOptions) -> Result<WatchHandle, VfsError> {
        // Track that the provider's watch method was actually invoked.
        self.watch_called.store(true, Ordering::SeqCst);
        Err(VfsError::UnsupportedOperation {
            operation: "watch".to_string(),
            provider: "captest".to_string(),
        })
    }
}

/// Strategy that generates random VfsCapabilities (random bool for each of the 10 fields).
fn capabilities_strategy() -> impl Strategy<Value = VfsCapabilities> {
    (
        any::<bool>(), // read
        any::<bool>(), // write
        any::<bool>(), // watch
        any::<bool>(), // search
        any::<bool>(), // random_access
        any::<bool>(), // append
        any::<bool>(), // rename
        any::<bool>(), // delete
        any::<bool>(), // list
        any::<bool>(), // create_directory
    )
        .prop_map(
            |(
                read,
                write,
                watch,
                search,
                random_access,
                append,
                rename,
                delete,
                list,
                create_directory,
            )| {
                VfsCapabilities {
                    read,
                    write,
                    watch,
                    search,
                    random_access,
                    append,
                    rename,
                    delete,
                    list,
                    create_directory,
                }
            },
        )
}

/// Strategy for generating valid path components (must start with /).
fn path_strategy() -> impl Strategy<Value = String> {
    "/[a-z][a-z0-9/_.-]{1,30}".prop_filter("path must start with /", |p| p.starts_with('/'))
}

// Feature: virtual-file-system, Property 5: Capability-gated operations
// **Validates: Requirements 1.2**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// When a provider declares watch: false, calling Vfs::watch() must return
    /// UnsupportedOperation. When watch: true, the call delegates to the provider.
    #[test]
    fn watch_capability_gate_is_enforced(
        caps in capabilities_strategy(),
        path in path_strategy(),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let watch_called = Arc::new(AtomicBool::new(false));
            let provider: Arc<dyn VfsProvider> =
                Arc::new(CapabilityMockProvider::new(caps, watch_called.clone()));
            let registry = ProviderRegistry::new();
            registry.register(provider).unwrap();

            let vfs = Vfs::with_registry(registry);
            let uri = ResourceUri::new("captest", &path);

            let result = vfs.watch(&uri, WatchOptions::default()).await;

            // The result must always be UnsupportedOperation for "watch" on "captest"
            match result {
                Err(VfsError::UnsupportedOperation { operation, provider }) => {
                    assert_eq!(operation, "watch");
                    assert_eq!(provider, "captest");
                }
                other => {
                    panic!(
                        "Expected UnsupportedOperation (caps.watch={}), got: {:?}",
                        caps.watch, other
                    );
                }
            }

            // The KEY property: when watch capability is false, the provider's
            // watch() method must NOT have been called (facade short-circuits).
            if !caps.watch {
                assert!(
                    !watch_called.load(Ordering::SeqCst),
                    "Provider's watch() must not be called when capabilities.watch == false"
                );
            } else {
                // When watch is true, the facade delegates to the provider
                assert!(
                    watch_called.load(Ordering::SeqCst),
                    "Provider's watch() must be called when capabilities.watch == true"
                );
            }
        });
    }
}
