//! # Configuration Provider Trait
//!
//! Defines the `ConfigProvider` trait that `ff-core` accepts as a dependency.
//! This trait is implemented by `ff-config` (when available) and keeps `ff-core`
//! decoupled from the configuration crate's internals.
//!
//! By accepting a `Box<dyn ConfigProvider>` rather than a concrete type,
//! `WorkbenchApp` can be constructed in tests with a mock provider and in
//! production with the real `ff-config` implementation.

/// Trait defining the configuration provider interface that `ff-core` accepts.
///
/// Implemented by `ff-config`. Keeps `ff-core` decoupled from the configuration
/// crate's internals. All lookups are namespace-scoped to support hierarchical
/// configuration (e.g., `("editor", "tab_size")` or `("logging", "level")`).
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow the configuration provider
/// to be shared across threads via `Arc` or stored in the `WorkbenchApp`.
pub trait ConfigProvider: Send + Sync {
    /// Retrieve a string value by namespace and key.
    ///
    /// Returns `None` if the key does not exist in the given namespace.
    fn get_string(&self, namespace: &str, key: &str) -> Option<String>;

    /// Retrieve an unsigned 64-bit integer value by namespace and key.
    ///
    /// Returns `None` if the key does not exist or cannot be interpreted
    /// as a `u64`.
    fn get_u64(&self, namespace: &str, key: &str) -> Option<u64>;

    /// Retrieve a boolean value by namespace and key.
    ///
    /// Returns `None` if the key does not exist or cannot be interpreted
    /// as a boolean.
    fn get_bool(&self, namespace: &str, key: &str) -> Option<bool>;
}
