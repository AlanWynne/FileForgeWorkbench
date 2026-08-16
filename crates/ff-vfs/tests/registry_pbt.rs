//! Property-based tests for the ProviderRegistry.
//!
//! Tests Properties 3, 4, and 10 from the VFS spec.

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

/// A parameterized mock provider for property tests.
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
        VfsCapabilities::none()
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
        Err(VfsError::UnsupportedOperation {
            operation: "rename".to_string(),
            provider: self.scheme_name.clone(),
        })
    }

    async fn list(&self, _path: &str) -> Result<Vec<VfsEntry>, VfsError> {
        Err(VfsError::UnsupportedOperation {
            operation: "list".to_string(),
            provider: self.scheme_name.clone(),
        })
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

/// Strategy that generates a Vec of unique scheme names.
fn unique_schemes_strategy(max_len: usize) -> impl Strategy<Value = Vec<String>> {
    proptest::collection::hash_set(scheme_strategy(), 1..=max_len)
        .prop_map(|set| set.into_iter().collect::<Vec<_>>())
}

// Feature: ff-vfs, Property 3: Registry uniqueness — duplicate scheme rejected
// **Validates: Requirements 3.2, 3.3**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn registry_uniqueness_duplicate_scheme_rejected(
        schemes in unique_schemes_strategy(10),
    ) {
        let registry = ProviderRegistry::new();

        // Register all unique schemes — all should succeed
        for scheme in &schemes {
            let provider = Arc::new(MockProvider::new(scheme));
            let result = registry.register(provider);
            prop_assert!(
                result.is_ok(),
                "registration failed for unique scheme '{}': {:?}",
                scheme,
                result.err()
            );
        }

        // Attempt to register a duplicate of the first scheme — must fail
        let duplicate_scheme = &schemes[0];
        let duplicate_provider = Arc::new(MockProvider::new(duplicate_scheme));
        let result = registry.register(duplicate_provider);

        match result {
            Err(VfsError::DuplicateScheme { scheme }) => {
                prop_assert_eq!(&scheme, duplicate_scheme);
            }
            other => {
                prop_assert!(
                    false,
                    "expected DuplicateScheme for '{}', got: {:?}",
                    duplicate_scheme,
                    other.err()
                );
            }
        }
    }
}

// Feature: ff-vfs, Property 4: Registry routing — registered schemes route correctly,
// unregistered return None
// **Validates: Requirements 3.5, 3.6**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn registry_routing_correctness(
        registered_schemes in unique_schemes_strategy(8),
        unregistered_schemes in unique_schemes_strategy(5),
    ) {
        let registry = ProviderRegistry::new();

        // Register all "registered" schemes
        for scheme in &registered_schemes {
            let provider = Arc::new(MockProvider::new(scheme));
            registry.register(provider).unwrap();
        }

        // All registered schemes should route correctly
        for scheme in &registered_schemes {
            let provider = registry.get(scheme);
            prop_assert!(
                provider.is_some(),
                "get() returned None for registered scheme '{}'",
                scheme
            );
            let p = provider.unwrap();
            prop_assert_eq!(p.scheme(), scheme.as_str());
        }

        // All unregistered schemes (that aren't also in registered) should return None
        for scheme in &unregistered_schemes {
            if !registered_schemes.contains(scheme) {
                let provider = registry.get(scheme);
                prop_assert!(
                    provider.is_none(),
                    "get() returned Some for unregistered scheme '{}'",
                    scheme
                );
            }
        }
    }
}

// Feature: ff-vfs, Property 10: Deregistration completeness — after deregister, get() returns None
// **Validates: Requirement 3.10**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn deregistration_completeness(
        schemes in unique_schemes_strategy(8),
    ) {
        let registry = ProviderRegistry::new();

        // Register all schemes
        for scheme in &schemes {
            let provider = Arc::new(MockProvider::new(scheme));
            registry.register(provider).unwrap();
        }

        // Deregister each one and verify it's gone
        for scheme in &schemes {
            // Before deregister: get() returns Some
            prop_assert!(
                registry.get(scheme).is_some(),
                "get() returned None before deregister for scheme '{}'",
                scheme
            );

            // Deregister
            let result = registry.deregister(scheme);
            prop_assert!(
                result.is_ok(),
                "deregister failed for scheme '{}': {:?}",
                scheme,
                result.err()
            );

            // After deregister: get() returns None
            prop_assert!(
                registry.get(scheme).is_none(),
                "get() returned Some after deregister for scheme '{}'",
                scheme
            );
        }

        // All schemes removed — list should be empty
        prop_assert!(
            registry.list_schemes().is_empty(),
            "list_schemes() should be empty after deregistering all, got: {:?}",
            registry.list_schemes()
        );
    }
}
