//! Property-based tests for the Vfs facade.
//!
//! Tests Property 7: cross-provider rename rejection.

use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use proptest::prelude::*;
use tokio::io::AsyncRead;

use ff_vfs::error::VfsError;
use ff_vfs::provider::{VfsFile, VfsProvider};
use ff_vfs::registry::ProviderRegistry;
use ff_vfs::types::{
    CreateOptions, DeleteOptions, OpenOptions, VfsCapabilities, VfsEntry, VfsMetadata,
};
use ff_vfs::uri::ResourceUri;
use ff_vfs::Vfs;

/// A minimal mock provider for property tests.
struct MockProvider {
    scheme_name: String,
}

impl MockProvider {
    fn new(scheme: &str) -> Self {
        Self {
            scheme_name: scheme.to_string(),
        }
    }
}

#[async_trait]
impl VfsProvider for MockProvider {
    fn scheme(&self) -> &str {
        &self.scheme_name
    }

    fn capabilities(&self) -> VfsCapabilities {
        VfsCapabilities::all()
    }

    async fn open(&self, _path: &str, _options: OpenOptions) -> Result<Box<dyn VfsFile>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "open".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn read(&self, _path: &str) -> Result<Vec<u8>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "read".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn read_stream(&self, _path: &str) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "read_stream".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn write(&self, _path: &str, _data: &[u8]) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "write".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn create(&self, _path: &str, _options: CreateOptions) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "create".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn delete(&self, _path: &str, _options: DeleteOptions) -> Result<(), VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "delete".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn rename(&self, _old_path: &str, _new_path: &str) -> Result<(), VfsError> {
        Ok(())
    }

    async fn list(&self, _path: &str) -> Result<Vec<VfsEntry>, VfsError> {
        Ok(Vec::new())
    }

    async fn stat(&self, _path: &str) -> Result<VfsMetadata, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "stat".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn exists(&self, _path: &str) -> Result<bool, VfsError> {
        Ok(false)
    }
}

/// Strategy that generates valid scheme names: 1–20 chars of [a-z0-9_-].
fn scheme_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{0,19}".prop_filter("scheme must not be empty", |s| !s.is_empty())
}

/// Strategy that generates a pair of DIFFERENT scheme names.
fn different_schemes_strategy() -> impl Strategy<Value = (String, String)> {
    (scheme_strategy(), scheme_strategy()).prop_filter("schemes must be different", |(a, b)| a != b)
}

/// Strategy for generating valid path components.
fn path_strategy() -> impl Strategy<Value = String> {
    "/[a-z][a-z0-9/_.-]{1,30}".prop_filter("path must start with /", |p| p.starts_with('/'))
}

// Feature: ff-vfs, Property 7: Cross-provider rename rejected
// **Validates: Requirement 5.6**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn cross_provider_rename_always_returns_unsupported_operation(
        (scheme_a, scheme_b) in different_schemes_strategy(),
        path_a in path_strategy(),
        path_b in path_strategy(),
    ) {
        // Create a Tokio runtime for the async test
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let registry = ProviderRegistry::new();
            let provider_a: Arc<dyn VfsProvider> = Arc::new(MockProvider::new(&scheme_a));
            let provider_b: Arc<dyn VfsProvider> = Arc::new(MockProvider::new(&scheme_b));
            registry.register(provider_a).unwrap();
            registry.register(provider_b).unwrap();

            let vfs = Vfs::with_registry(registry);

            let old_uri = ResourceUri::new(&scheme_a, &path_a);
            let new_uri = ResourceUri::new(&scheme_b, &path_b);

            let result = vfs.rename(&old_uri, &new_uri).await;

            match result {
                Err(VfsError::UnsupportedOperation { operation, provider }) => {
                    assert_eq!(operation, "rename");
                    assert!(
                        provider.contains(&scheme_a),
                        "provider field should contain source scheme '{}', got '{}'",
                        scheme_a, provider
                    );
                    assert!(
                        provider.contains(&scheme_b),
                        "provider field should contain dest scheme '{}', got '{}'",
                        scheme_b, provider
                    );
                }
                other => {
                    panic!(
                        "expected UnsupportedOperation for cross-provider rename ({} -> {}), got: {:?}",
                        scheme_a, scheme_b, other
                    );
                }
            }
        });
    }
}
