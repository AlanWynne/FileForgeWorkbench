//! # ff-config
//!
//! Central configuration management layer for the FileForgeWorkbench platform.
//!
//! This crate provides:
//! - TOML-based configuration files with a well-defined schema
//! - A six-layer override model: Defaults → System → User → Profile → Project → Workspace
//! - Hot-reload with debounced file watching and atomic change application
//! - Named user profiles with runtime switching
//! - Per-project and workspace-level overrides
//! - EditorConfig integration for per-file editor settings
//! - A typed access API with compile-time key definitions
//! - Plugin namespace scoping and isolation
//! - Runtime-queryable schema validation

pub mod access;
pub mod callback;
pub mod config_handle;
pub mod editorconfig;
pub mod error;
pub mod init;
pub mod keys;
pub mod layer;
pub mod loader;
pub mod merger;
pub mod namespace;
pub mod paths;
pub mod plugin_handle;
pub mod profile;
pub mod project;
pub mod provenance;
pub mod provider;
pub mod reload;
pub mod schema;
pub mod store;
pub mod validate;
pub mod value;
pub mod watcher;

// Public API re-exports
pub use callback::{CallbackHandle, CallbackRegistry, ReloadCallback};
pub use config_handle::ConfigHandle;
pub use error::ConfigError;
pub use init::{
    auto_detect_project_config, init, register_catalog_schema, register_core_schema, shutdown,
    ConfigInitOptions,
};
pub use layer::ConfigLayer;
pub use merger::merge_layers;
pub use namespace::{
    is_reserved_namespace, plugin_namespace_prefix, validate_plugin_name, RESERVED_NAMESPACES,
};
pub use plugin_handle::{
    create_plugin_config_handle, create_plugin_config_handle_with_callbacks,
    register_plugin_defaults, unload_plugin, PluginConfigHandle, PluginDefault,
};
pub use profile::{ProfileManager, UserProfile};
pub use provenance::{EffectiveValue, Provenance};
pub use reload::{ReloadEvent, ReloadManager};
pub use store::EffectiveStore;
pub use value::{ConfigTable, ConfigValue};
