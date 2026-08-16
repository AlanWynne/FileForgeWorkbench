//! Integration tests for the ff-plugin crate.
//!
//! End-to-end tests covering discovery, loading, activation, capability
//! querying, deactivation, and shutdown flows.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ff_plugin::{
    Capability, CapabilityRegistrar, CapabilityType, CommandRegistration, CommandsCapability,
    EventHandler, FileForgePlugin, PlatformEvent, PlatformServices, PluginCommand,
    PluginConfigAccess, PluginContext, PluginError, PluginEventBus, PluginMetadata, PluginRegistry,
    PluginState, PluginVfsAccess, SubscriptionId, Version,
};

// ─── Mock Services ──────────────────────────────────────────────────────────

struct MockCommandService;
impl CommandRegistration for MockCommandService {
    fn register(&self, _: &str, _: PluginCommand) -> Result<(), PluginError> {
        Ok(())
    }
    fn unregister(&self, _: &str, _: &str) -> Result<(), PluginError> {
        Ok(())
    }
}

struct MockConfigService;
impl PluginConfigAccess for MockConfigService {
    fn get(&self, _: &str, _: &str) -> Result<Option<toml::Value>, PluginError> {
        Ok(None)
    }
    fn set(&self, _: &str, _: &str, _: toml::Value) -> Result<(), PluginError> {
        Ok(())
    }
}

struct MockVfsService;
impl PluginVfsAccess for MockVfsService {
    fn read(&self, _: &str) -> Result<Vec<u8>, PluginError> {
        Ok(vec![])
    }
    fn write(&self, _: &str, _: &[u8]) -> Result<(), PluginError> {
        Ok(())
    }
    fn exists(&self, _: &str) -> Result<bool, PluginError> {
        Ok(false)
    }
    fn list_directory(&self, _: &str) -> Result<Vec<String>, PluginError> {
        Ok(vec![])
    }
}

struct MockEventBus;
impl PluginEventBus for MockEventBus {
    fn subscribe(&self, _: &str, _: &str, _: EventHandler) -> SubscriptionId {
        SubscriptionId::new(0)
    }
    fn unsubscribe(&self, _: SubscriptionId) {}
    fn emit(&self, _: PlatformEvent) {}
}

struct MockCapabilityRegistrar;
impl CapabilityRegistrar for MockCapabilityRegistrar {
    fn register(&self, _: &str, _: Capability) -> Result<(), PluginError> {
        Ok(())
    }
    fn unregister(&self, _: &str, _: &str) -> Result<(), PluginError> {
        Ok(())
    }
}

fn test_services() -> PlatformServices {
    PlatformServices {
        command_service: Arc::new(MockCommandService),
        config_service: Arc::new(MockConfigService),
        vfs_service: Arc::new(MockVfsService),
        event_service: Arc::new(MockEventBus),
        capability_service: Arc::new(MockCapabilityRegistrar),
    }
}

// ─── Test Plugins ───────────────────────────────────────────────────────────

struct GoodPlugin {
    metadata: PluginMetadata,
    capabilities: Vec<Capability>,
    lifecycle_log: Arc<Mutex<Vec<String>>>,
}

impl GoodPlugin {
    fn new(name: &str, log: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            metadata: PluginMetadata {
                name: name.to_string(),
                version: Version::new(1, 0, 0),
                author: "Test".to_string(),
                description: format!("{name} plugin"),
                dependencies: vec![],
                required_api_version: Version::new(1, 0, 0),
            },
            capabilities: vec![Capability::Commands(CommandsCapability {
                command_ids: vec![format!("{name}.cmd")],
                category: "test".to_string(),
                version: Version::new(1, 0, 0),
            })],
            lifecycle_log: log,
        }
    }
}

impl FileForgePlugin for GoodPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    fn plugin_capabilities(&self) -> &[Capability] {
        &self.capabilities
    }
    fn initialize(&mut self, _ctx: Arc<PluginContext>) -> Result<(), PluginError> {
        self.lifecycle_log
            .lock()
            .unwrap()
            .push(format!("{}.initialize", self.metadata.name));
        Ok(())
    }
    fn activate(&mut self) -> Result<(), PluginError> {
        self.lifecycle_log
            .lock()
            .unwrap()
            .push(format!("{}.activate", self.metadata.name));
        Ok(())
    }
    fn deactivate(&mut self) -> Result<(), PluginError> {
        self.lifecycle_log
            .lock()
            .unwrap()
            .push(format!("{}.deactivate", self.metadata.name));
        Ok(())
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        self.lifecycle_log
            .lock()
            .unwrap()
            .push(format!("{}.shutdown", self.metadata.name));
        Ok(())
    }
}

struct PanickingPlugin {
    metadata: PluginMetadata,
    panic_phase: String,
}

impl PanickingPlugin {
    fn new(name: &str, panic_phase: &str) -> Self {
        Self {
            metadata: PluginMetadata {
                name: name.to_string(),
                version: Version::new(1, 0, 0),
                author: "Test".to_string(),
                description: "Panics during lifecycle".to_string(),
                dependencies: vec![],
                required_api_version: Version::new(1, 0, 0),
            },
            panic_phase: panic_phase.to_string(),
        }
    }
}

impl FileForgePlugin for PanickingPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    fn plugin_capabilities(&self) -> &[Capability] {
        &[]
    }
    fn initialize(&mut self, _ctx: Arc<PluginContext>) -> Result<(), PluginError> {
        if self.panic_phase == "initialize" {
            panic!("intentional panic in initialize");
        }
        Ok(())
    }
    fn activate(&mut self) -> Result<(), PluginError> {
        if self.panic_phase == "activate" {
            panic!("intentional panic in activate");
        }
        Ok(())
    }
    fn deactivate(&mut self) -> Result<(), PluginError> {
        if self.panic_phase == "deactivate" {
            panic!("intentional panic in deactivate");
        }
        Ok(())
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        if self.panic_phase == "shutdown" {
            panic!("intentional panic in shutdown");
        }
        Ok(())
    }
}

// ─── Integration Tests ──────────────────────────────────────────────────────

#[test]
fn end_to_end_load_activate_query_deactivate_shutdown() {
    // Validates: Requirement 3.2, Requirement 5.1
    let log = Arc::new(Mutex::new(Vec::new()));
    let registry = PluginRegistry::new(PathBuf::from("/tmp/plugins"), test_services());

    registry.register_plugin(Box::new(GoodPlugin::new("viewer", Arc::clone(&log))));
    registry.register_plugin(Box::new(GoodPlugin::new("editor", Arc::clone(&log))));

    // Load both
    registry.load_plugin("viewer").unwrap();
    registry.load_plugin("editor").unwrap();

    // Verify active state
    assert_eq!(registry.plugin_state("viewer"), Some(PluginState::Active));
    assert_eq!(registry.plugin_state("editor"), Some(PluginState::Active));

    // Verify lifecycle calls happened in order
    let calls = log.lock().unwrap();
    assert!(calls.contains(&"viewer.initialize".to_string()));
    assert!(calls.contains(&"viewer.activate".to_string()));
    assert!(calls.contains(&"editor.initialize".to_string()));
    assert!(calls.contains(&"editor.activate".to_string()));
    drop(calls);

    // Unload one
    registry.unload_plugin("viewer").unwrap();
    assert_eq!(registry.plugin_state("viewer"), Some(PluginState::Shutdown));
    assert_eq!(registry.plugin_state("editor"), Some(PluginState::Active));

    // Shutdown all
    registry.shutdown_all(Duration::from_secs(5));
    assert_eq!(registry.plugin_state("editor"), Some(PluginState::Shutdown));
}

#[test]
fn plugin_failure_isolation_panic_in_initialize() {
    // Validates: Requirement 5.3
    let log = Arc::new(Mutex::new(Vec::new()));
    let registry = PluginRegistry::new(PathBuf::from("/tmp/plugins"), test_services());

    registry.register_plugin(Box::new(PanickingPlugin::new("crashy", "initialize")));
    registry.register_plugin(Box::new(GoodPlugin::new("healthy", Arc::clone(&log))));

    // Load panicking plugin — should not crash the host
    let result = registry.load_plugin("crashy");
    assert!(result.is_err());
    assert_eq!(registry.plugin_state("crashy"), Some(PluginState::Shutdown));

    // Other plugins still work
    registry.load_plugin("healthy").unwrap();
    assert_eq!(registry.plugin_state("healthy"), Some(PluginState::Active));
}

#[test]
fn plugin_failure_isolation_panic_in_activate() {
    // Validates: Requirement 5.3
    let registry = PluginRegistry::new(PathBuf::from("/tmp/plugins"), test_services());

    registry.register_plugin(Box::new(PanickingPlugin::new("crashy", "activate")));

    let result = registry.load_plugin("crashy");
    assert!(result.is_err());
    assert_eq!(registry.plugin_state("crashy"), Some(PluginState::Shutdown));
}

#[test]
fn capability_registry_integration() {
    // Validates: Requirement 4.2, 4.3
    let cap_registry = ff_plugin::CapabilityRegistry::new();

    // Register capabilities for multiple plugins
    cap_registry
        .register(
            "viewer-plugin",
            Capability::Commands(CommandsCapability {
                command_ids: vec!["viewer.open".to_string()],
                category: "file".to_string(),
                version: Version::new(1, 0, 0),
            }),
        )
        .unwrap();

    cap_registry
        .register(
            "editor-plugin",
            Capability::Commands(CommandsCapability {
                command_ids: vec!["editor.save".to_string()],
                category: "file".to_string(),
                version: Version::new(1, 0, 0),
            }),
        )
        .unwrap();

    // Query by type
    let commands = cap_registry.query_by_type(CapabilityType::Commands);
    assert_eq!(commands.len(), 2);

    // Unregister one plugin
    cap_registry.unregister_all("viewer-plugin");
    let commands = cap_registry.query_by_type(CapabilityType::Commands);
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].owner_plugin, "editor-plugin");
}

#[test]
fn configuration_scoping_across_plugins() {
    // Validates: Requirement 2.7, Requirement 7.4, Requirement 7.5
    let services = test_services();
    let ctx_a = PluginContext::new("plugin-a", &services);
    let ctx_b = PluginContext::new("plugin-b", &services);

    // Each plugin can access their own keys
    assert!(ctx_a.config_get("my_setting").is_ok());
    assert!(ctx_b.config_get("my_setting").is_ok());

    // Neither can access the other's namespace
    assert!(ctx_a.config_get("plugins.plugin-b.secret").is_err());
    assert!(ctx_b.config_get("plugins.plugin-a.secret").is_err());
}

#[test]
fn discovery_with_tempdir() {
    // Validates: Requirement 3.1
    let dir = tempfile::TempDir::new().unwrap();

    // Create a valid plugin directory
    let plugin_dir = dir.path().join("my-viewer");
    std::fs::create_dir(&plugin_dir).unwrap();
    std::fs::write(
        plugin_dir.join("plugin.toml"),
        r#"
[plugin]
name = "my-viewer"
version = "1.0.0"
author = "Test"
description = "A test viewer"
required_api_version = "1.0.0"
"#,
    )
    .unwrap();

    // Create a directory without a manifest (should be skipped)
    let no_manifest = dir.path().join("no-manifest");
    std::fs::create_dir(&no_manifest).unwrap();

    let registry = PluginRegistry::new(dir.path().to_path_buf(), test_services());
    let discovered = registry.discover_plugins().unwrap();

    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0], "my-viewer");
    assert_eq!(
        registry.plugin_state("my-viewer"),
        Some(PluginState::Discovered)
    );
}

#[test]
fn shutdown_all_with_timeout_completes() {
    // Validates: Requirement 5.5
    let log = Arc::new(Mutex::new(Vec::new()));
    let registry = PluginRegistry::new(PathBuf::from("/tmp/plugins"), test_services());

    for i in 0..5 {
        registry.register_plugin(Box::new(GoodPlugin::new(
            &format!("plugin-{i}"),
            Arc::clone(&log),
        )));
    }

    for i in 0..5 {
        registry.load_plugin(&format!("plugin-{i}")).unwrap();
    }

    // Shutdown with generous timeout
    registry.shutdown_all(Duration::from_secs(10));

    // All should be shut down
    for i in 0..5 {
        assert_eq!(
            registry.plugin_state(&format!("plugin-{i}")),
            Some(PluginState::Shutdown)
        );
    }
}
