//! Thread-safe provider registry for the VFS abstraction layer.
//!
//! The [`ProviderRegistry`] manages VFS provider instances keyed by their scheme
//! identifier. It supports runtime registration, deregistration, and discovery of
//! providers in a thread-safe manner using `std::sync::RwLock`.
//!
//! Addresses: Requirement 3, criteria 1–10

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::VfsError;
use crate::provider::VfsProvider;

/// Thread-safe registry of [`VfsProvider`] instances, indexed by scheme.
///
/// Supports runtime registration, deregistration, and discovery.
/// Uses `std::sync::RwLock` (not `tokio::sync::RwLock`) because:
/// 1. Registry operations are synchronous — no async work inside critical sections.
/// 2. Critical sections are very short (just `HashMap` operations).
/// 3. We never hold the lock across an `await` point.
///
/// Addresses: Requirement 3, criteria 1–10
pub struct ProviderRegistry {
    /// Provider storage keyed by scheme identifier.
    providers: Arc<RwLock<HashMap<String, Arc<dyn VfsProvider>>>>,
    /// The default provider scheme (typically "local").
    default_scheme: Arc<RwLock<String>>,
}

impl ProviderRegistry {
    /// Creates a new empty registry with `"local"` as the default scheme.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_vfs::ProviderRegistry;
    /// let registry = ProviderRegistry::new();
    /// assert_eq!(registry.default_scheme(), "local");
    /// ```
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            default_scheme: Arc::new(RwLock::new("local".to_string())),
        }
    }

    /// Registers a provider with its scheme.
    ///
    /// The scheme is obtained from [`VfsProvider::scheme()`]. Returns
    /// [`VfsError::DuplicateScheme`] if a provider with the same scheme is
    /// already registered.
    ///
    /// # Errors
    ///
    /// Returns `VfsError::DuplicateScheme` if the scheme is already registered.
    ///
    /// Addresses: Requirement 3 AC 2, AC 3
    pub fn register(&self, provider: Arc<dyn VfsProvider>) -> Result<(), VfsError> {
        let scheme = provider.scheme().to_string();
        let mut providers = self
            .providers
            .write()
            .expect("provider registry lock poisoned");

        if providers.contains_key(&scheme) {
            return Err(VfsError::DuplicateScheme {
                scheme: scheme.clone(),
            });
        }

        providers.insert(scheme.clone(), provider);
        eprintln!("[vfs] registry: registered provider for scheme '{scheme}'");
        Ok(())
    }

    /// Removes and returns the provider registered under the given scheme.
    ///
    /// # Errors
    ///
    /// Returns `VfsError::ProviderUnavailable` if no provider is registered
    /// for the given scheme.
    ///
    /// Addresses: Requirement 3 AC 10
    pub fn deregister(&self, scheme: &str) -> Result<Arc<dyn VfsProvider>, VfsError> {
        let mut providers = self
            .providers
            .write()
            .expect("provider registry lock poisoned");

        providers
            .remove(scheme)
            .ok_or_else(|| VfsError::ProviderUnavailable {
                scheme: scheme.to_string(),
            })
    }

    /// Looks up a provider by scheme using a read lock.
    ///
    /// Returns `None` if no provider is registered for the given scheme.
    ///
    /// Addresses: Requirement 3 AC 5, AC 6
    pub fn get(&self, scheme: &str) -> Option<Arc<dyn VfsProvider>> {
        let providers = self
            .providers
            .read()
            .expect("provider registry lock poisoned");
        providers.get(scheme).cloned()
    }

    /// Returns a sorted list of all registered scheme names.
    ///
    /// Addresses: Requirement 3 AC 7
    pub fn list_schemes(&self) -> Vec<String> {
        let providers = self
            .providers
            .read()
            .expect("provider registry lock poisoned");
        let mut schemes: Vec<String> = providers.keys().cloned().collect();
        schemes.sort();
        schemes
    }

    /// Returns a list of all registered providers as `(scheme, Arc<dyn VfsProvider>)` pairs.
    ///
    /// The list is sorted by scheme name.
    pub fn list_providers(&self) -> Vec<(String, Arc<dyn VfsProvider>)> {
        let providers = self
            .providers
            .read()
            .expect("provider registry lock poisoned");
        let mut pairs: Vec<(String, Arc<dyn VfsProvider>)> = providers
            .iter()
            .map(|(k, v)| (k.clone(), Arc::clone(v)))
            .collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        pairs
    }

    /// Returns the current default scheme name.
    ///
    /// Addresses: Requirement 3 AC 8
    pub fn default_scheme(&self) -> String {
        self.default_scheme
            .read()
            .expect("default_scheme lock poisoned")
            .clone()
    }

    /// Returns `true` if a provider is registered for the default scheme.
    ///
    /// Addresses: Requirement 3 AC 9
    pub fn has_default_provider(&self) -> bool {
        let scheme = self.default_scheme();
        self.get(&scheme).is_some()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::VfsFile;
    use crate::types::{
        CreateOptions, DeleteOptions, OpenOptions, VfsCapabilities, VfsEntry, VfsMetadata,
    };

    use async_trait::async_trait;
    use std::pin::Pin;
    use tokio::io::AsyncRead;

    /// A parameterized mock provider used in registry tests.
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

        async fn open(
            &self,
            _path: &str,
            _options: OpenOptions,
        ) -> Result<Box<dyn VfsFile>, VfsError> {
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

        async fn read_stream(
            &self,
            _path: &str,
        ) -> Result<Pin<Box<dyn AsyncRead + Send>>, VfsError> {
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

    // Validates: Requirement 3 AC 1
    #[test]
    fn new_registry_has_local_default_scheme() {
        let registry = ProviderRegistry::new();
        assert_eq!(registry.default_scheme(), "local");
    }

    // Validates: Requirement 3 AC 1
    #[test]
    fn new_registry_has_no_providers() {
        let registry = ProviderRegistry::new();
        assert!(registry.list_schemes().is_empty());
    }

    // Validates: Requirement 3 AC 2
    #[test]
    fn register_provider_succeeds() {
        let registry = ProviderRegistry::new();
        let provider: Arc<dyn VfsProvider> = Arc::new(MockProvider::new("local"));
        let result = registry.register(provider);
        assert!(result.is_ok());
    }

    // Validates: Requirement 3 AC 5
    #[test]
    fn get_returns_registered_provider() {
        let registry = ProviderRegistry::new();
        let provider: Arc<dyn VfsProvider> = Arc::new(MockProvider::new("local"));
        registry.register(provider).unwrap();

        let retrieved = registry.get("local");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().scheme(), "local");
    }

    // Validates: Requirement 3 AC 6
    #[test]
    fn get_returns_none_for_unregistered_scheme() {
        let registry = ProviderRegistry::new();
        let result = registry.get("nonexistent");
        assert!(result.is_none());
    }

    // Validates: Requirement 3 AC 3
    #[test]
    fn register_duplicate_scheme_returns_error() {
        let registry = ProviderRegistry::new();
        let provider1: Arc<dyn VfsProvider> = Arc::new(MockProvider::new("local"));
        let provider2: Arc<dyn VfsProvider> = Arc::new(MockProvider::new("local"));

        registry.register(provider1).unwrap();
        let result = registry.register(provider2);

        match result {
            Err(VfsError::DuplicateScheme { scheme }) => {
                assert_eq!(scheme, "local");
            }
            other => panic!("expected DuplicateScheme error, got: {other:?}"),
        }
    }

    // Validates: Requirement 3 AC 10
    #[test]
    fn deregister_removes_provider() {
        let registry = ProviderRegistry::new();
        let provider: Arc<dyn VfsProvider> = Arc::new(MockProvider::new("local"));
        registry.register(provider).unwrap();

        let removed = registry.deregister("local");
        assert!(removed.is_ok());
        assert_eq!(removed.unwrap().scheme(), "local");
        assert!(registry.get("local").is_none());
    }

    // Validates: Requirement 3 AC 10
    #[test]
    fn deregister_nonexistent_returns_error() {
        let registry = ProviderRegistry::new();
        let result = registry.deregister("nonexistent");

        match result {
            Err(VfsError::ProviderUnavailable { scheme }) => {
                assert_eq!(scheme, "nonexistent");
            }
            Err(other) => panic!("expected ProviderUnavailable error, got: {other}"),
            Ok(_) => panic!("expected ProviderUnavailable error, got Ok"),
        }
    }

    // Validates: Requirement 3 AC 7
    #[test]
    fn list_schemes_returns_sorted_schemes() {
        let registry = ProviderRegistry::new();
        registry
            .register(Arc::new(MockProvider::new("zebra")))
            .unwrap();
        registry
            .register(Arc::new(MockProvider::new("alpha")))
            .unwrap();
        registry
            .register(Arc::new(MockProvider::new("middle")))
            .unwrap();

        let schemes = registry.list_schemes();
        assert_eq!(schemes, vec!["alpha", "middle", "zebra"]);
    }

    // Validates: Requirement 3 AC 7
    #[test]
    fn list_providers_returns_sorted_pairs() {
        let registry = ProviderRegistry::new();
        registry
            .register(Arc::new(MockProvider::new("zebra")))
            .unwrap();
        registry
            .register(Arc::new(MockProvider::new("alpha")))
            .unwrap();

        let providers = registry.list_providers();
        assert_eq!(providers.len(), 2);
        assert_eq!(providers[0].0, "alpha");
        assert_eq!(providers[1].0, "zebra");
    }

    // Validates: Requirement 3 AC 9
    #[test]
    fn has_default_provider_returns_false_when_no_local_provider() {
        let registry = ProviderRegistry::new();
        assert!(!registry.has_default_provider());
    }

    // Validates: Requirement 3 AC 9
    #[test]
    fn has_default_provider_returns_true_when_local_provider_registered() {
        let registry = ProviderRegistry::new();
        registry
            .register(Arc::new(MockProvider::new("local")))
            .unwrap();
        assert!(registry.has_default_provider());
    }

    // Validates: Requirement 3 AC 4 (thread safety)
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_register_and_get_is_thread_safe() {
        let registry = Arc::new(ProviderRegistry::new());

        let mut handles = Vec::new();

        // Spawn tasks that register providers concurrently
        for i in 0..10 {
            let reg = Arc::clone(&registry);
            let handle = tokio::spawn(async move {
                let provider = Arc::new(MockProvider::new(&format!("scheme-{i}")));
                reg.register(provider)
            });
            handles.push(handle);
        }

        // All registrations should succeed (all schemes are unique)
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        // Verify all providers are registered
        let schemes = registry.list_schemes();
        assert_eq!(schemes.len(), 10);

        // Spawn tasks that read concurrently
        let mut read_handles = Vec::new();
        for i in 0..10 {
            let reg = Arc::clone(&registry);
            let handle = tokio::spawn(async move {
                let provider = reg.get(&format!("scheme-{i}"));
                assert!(provider.is_some());
            });
            read_handles.push(handle);
        }

        for handle in read_handles {
            handle.await.unwrap();
        }
    }

    // Validates: Requirement 3 AC 4 (thread safety — concurrent register/deregister)
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_deregister_is_thread_safe() {
        let registry = Arc::new(ProviderRegistry::new());

        // Pre-register providers
        for i in 0..10 {
            let provider = Arc::new(MockProvider::new(&format!("scheme-{i}")));
            registry.register(provider).unwrap();
        }

        // Spawn tasks that deregister concurrently
        let mut handles = Vec::new();
        for i in 0..10 {
            let reg = Arc::clone(&registry);
            let handle = tokio::spawn(async move { reg.deregister(&format!("scheme-{i}")) });
            handles.push(handle);
        }

        // All deregistrations should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        assert!(registry.list_schemes().is_empty());
    }

    // Validates: Requirement 3 AC 2, AC 10 (register/deregister lifecycle)
    #[test]
    fn register_deregister_register_lifecycle() {
        let registry = ProviderRegistry::new();
        let provider: Arc<dyn VfsProvider> = Arc::new(MockProvider::new("local"));

        // Register
        registry.register(Arc::clone(&provider)).unwrap();
        assert!(registry.get("local").is_some());

        // Deregister
        registry.deregister("local").unwrap();
        assert!(registry.get("local").is_none());

        // Re-register
        registry.register(provider).unwrap();
        assert!(registry.get("local").is_some());
    }
}
