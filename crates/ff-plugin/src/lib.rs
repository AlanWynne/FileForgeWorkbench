//! # ff-plugin — Plugin Architecture for FileForgeWorkbench
//!
//! This crate defines the plugin extensibility framework. Every optional
//! feature (viewers, language services, connectors, macro engines, the
//! database tool) is implemented as a plugin that interacts with the core
//! exclusively through traits and a context object defined here.
//!
//! ## Key Types
//!
//! - [`FileForgePlugin`] — the primary trait all plugins implement
//! - [`PluginContext`] — sandboxed gateway to platform services
//! - [`PluginRegistry`] — manages plugin lifecycle and state
//! - [`CapabilityRegistry`] — dynamic index of active capabilities
//! - [`PluginMetadata`] — plugin identity, versioning, dependencies
//! - [`Capability`] — typed service a plugin provides
//!
//! ## Architecture
//!
//! ```text
//! Plugin Crates → implement FileForgePlugin
//!                → use PluginContext for services
//!                → declare Capabilities
//!
//! Platform Core → constructs PluginRegistry
//!              → provides PlatformServices
//!              → calls discover/load/shutdown
//! ```

pub mod capability;
pub mod capability_registry;
pub mod context;
pub mod dependency;
pub mod error;
pub mod event;
pub mod lifecycle;
pub mod loader;
pub mod metadata;
pub mod registry;
pub mod security;
pub mod traits;
pub mod version;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use capability::{
    Capability, CapabilityDescriptor, CapabilityFilter, CapabilityType, CommandsCapability,
    LanguageSupportCapability, ProvidersCapability, ThemeCapability, ViewersCapability,
};
pub use capability_registry::CapabilityRegistry;
pub use context::{PlatformServices, PluginContext};
pub use dependency::DependencyGraph;
pub use error::PluginError;
pub use event::{EventHandler, PlatformEvent, SubscriptionId};
pub use lifecycle::PluginState;
pub use metadata::{parse_manifest, PluginDependency, PluginMetadata};
pub use registry::{PluginLoadResult, PluginRegistry};
pub use traits::{
    CapabilityRegistrar, CommandRegistration, FileForgePlugin, PluginCommand, PluginConfigAccess,
    PluginEventBus, PluginVfsAccess,
};
pub use version::{
    check_api_compatibility, is_api_compatible, Version, VersionReq, PLUGIN_API_VERSION,
};
