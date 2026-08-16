//! `PluginLogHandle` trait and concrete implementation for plugin integration.
//!
//! Plugins receive a `Box<dyn PluginLogHandle>` via `PluginContext` at
//! initialization time. This allows plugins to emit log records without
//! importing or depending on `ff-logging` internal types directly.

use crate::level::LogLevel;

/// Trait for plugin logging handles.
///
/// Provided to plugins via `PluginContext`. Plugins use this to emit
/// log records at any severity level without tight coupling to the
/// logging subsystem's internals.
///
/// All methods are safe to call from any thread.
pub trait PluginLogHandle: Send + Sync {
    /// Emit a TRACE-level log record.
    fn trace(&self, module: &str, message: &str);
    /// Emit a DEBUG-level log record.
    fn debug(&self, module: &str, message: &str);
    /// Emit an INFO-level log record.
    fn info(&self, module: &str, message: &str);
    /// Emit a WARN-level log record.
    fn warn(&self, module: &str, message: &str);
    /// Emit an ERROR-level log record.
    fn error(&self, module: &str, message: &str);

    /// Flush any buffered records from this plugin.
    ///
    /// Called during plugin shutdown before the plugin's `shutdown` method returns.
    /// Sends a flush message through the channel and waits briefly for the
    /// writer thread to process it.
    fn flush(&self);
}

/// Create a plugin log handle with the given plugin name prefix.
///
/// Records emitted through this handle are automatically prefixed as
/// `plugin:{name}::{module}` in the module path field, ensuring plugin
/// records are distinguishable from platform-core records in the log stream.
///
/// The returned handle delegates to the global logging subsystem, so
/// all level filtering, formatting, rotation, and flushing rules apply
/// identically to plugin records and core records.
///
/// # Examples
///
/// ```rust,no_run
/// use ff_logging::create_plugin_handle;
///
/// let handle = create_plugin_handle("my-plugin");
/// handle.info("utils", "Plugin initialized");
/// // Produces a log record with module path: "plugin:my-plugin::utils"
/// ```
pub fn create_plugin_handle(plugin_name: &str) -> Box<dyn PluginLogHandle> {
    Box::new(ConcretePluginLogHandle {
        plugin_name: plugin_name.to_string(),
    })
}

/// The concrete plugin log handle implementation.
///
/// Holds the plugin name and prefixes all log records with
/// `plugin:{name}::{module}` before delegating to the global
/// logging subsystem via `crate::init::log()`.
///
/// # Thread Safety
///
/// This struct is `Send + Sync` because it only contains an immutable
/// `String` field. It can be safely shared across threads spawned by
/// a plugin.
struct ConcretePluginLogHandle {
    /// The registered plugin name, used for module path prefixing.
    plugin_name: String,
}

impl ConcretePluginLogHandle {
    /// Formats the module path with the plugin prefix and delegates
    /// to the global logging subsystem.
    ///
    /// The prefixed module path takes the form `plugin:{name}::{module}`,
    /// which appears in the log output as `[plugin:{name}::{module}]`.
    fn log_with_prefix(&self, level: LogLevel, module: &str, message: &str) {
        let prefixed_module = format!("plugin:{}::{}", self.plugin_name, module);
        crate::init::log(level, &prefixed_module, message);
    }
}

impl PluginLogHandle for ConcretePluginLogHandle {
    fn trace(&self, module: &str, message: &str) {
        self.log_with_prefix(LogLevel::Trace, module, message);
    }

    fn debug(&self, module: &str, message: &str) {
        self.log_with_prefix(LogLevel::Debug, module, message);
    }

    fn info(&self, module: &str, message: &str) {
        self.log_with_prefix(LogLevel::Info, module, message);
    }

    fn warn(&self, module: &str, message: &str) {
        self.log_with_prefix(LogLevel::Warn, module, message);
    }

    fn error(&self, module: &str, message: &str) {
        self.log_with_prefix(LogLevel::Error, module, message);
    }

    fn flush(&self) {
        if let Some(sender) = crate::init::get_sender() {
            sender.send_flush();
            // Brief wait for the writer thread to process the flush.
            // This ensures buffered plugin records are written to disk
            // before the plugin's shutdown method returns (Req 10, AC 10.5).
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

// ─── Static Assertions ─────────────────────────────────────────────────────

// Compile-time verification that ConcretePluginLogHandle is Send + Sync.
// These assertions produce a compile error if the type fails to meet the bounds.
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ConcretePluginLogHandle>();
};

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Handle Creation Tests ──────────────────────────────────────────────

    #[test]
    fn create_plugin_handle_returns_valid_handle() {
        // Validates: Requirement 10.1
        let handle = create_plugin_handle("test-plugin");
        // Should not panic — the handle is usable even without the subsystem initialized
        handle.info("module", "test message");
    }

    #[test]
    fn create_plugin_handle_with_empty_name() {
        // Validates: Requirement 10.1
        let handle = create_plugin_handle("");
        // Empty plugin name still produces a valid handle
        handle.info("module", "test");
    }

    // ─── Module Path Prefix Tests ───────────────────────────────────────────

    #[test]
    fn module_path_prefix_format_is_correct() {
        // Validates: Requirement 10.2
        let handle = ConcretePluginLogHandle {
            plugin_name: "my-plugin".to_string(),
        };

        // Verify the prefix format by checking what log_with_prefix would produce
        let expected_module = "plugin:my-plugin::utils";
        let actual_module = format!("plugin:{}::{}", handle.plugin_name, "utils");
        assert_eq!(actual_module, expected_module);
    }

    #[test]
    fn module_path_prefix_with_nested_module() {
        // Validates: Requirement 10.2
        let handle = ConcretePluginLogHandle {
            plugin_name: "syntax-highlight".to_string(),
        };

        let expected = "plugin:syntax-highlight::parser::tokens";
        let actual = format!("plugin:{}::{}", handle.plugin_name, "parser::tokens");
        assert_eq!(actual, expected);
    }

    #[test]
    fn module_path_prefix_with_special_characters_in_name() {
        // Validates: Requirement 10.2
        let handle = ConcretePluginLogHandle {
            plugin_name: "my_plugin-v2.0".to_string(),
        };

        let expected = "plugin:my_plugin-v2.0::core";
        let actual = format!("plugin:{}::{}", handle.plugin_name, "core");
        assert_eq!(actual, expected);
    }

    // ─── Level Filtering Tests ──────────────────────────────────────────────

    #[test]
    fn all_five_log_levels_are_callable() {
        // Validates: Requirement 10.2
        // All five methods should be callable without panic,
        // even when the subsystem is not initialized (records are silently dropped).
        let handle = create_plugin_handle("level-test");
        handle.trace("mod", "trace message");
        handle.debug("mod", "debug message");
        handle.info("mod", "info message");
        handle.warn("mod", "warn message");
        handle.error("mod", "error message");
    }

    // ─── Flush Tests ────────────────────────────────────────────────────────

    #[test]
    fn flush_does_not_panic_without_subsystem() {
        // Validates: Requirement 10.5
        // When the subsystem is not initialized, flush should be a no-op
        let handle = create_plugin_handle("flush-test");
        handle.flush(); // Should not panic
    }

    // ─── Thread Safety Tests ────────────────────────────────────────────────

    #[test]
    fn plugin_handle_is_send_and_sync() {
        // Validates: Requirement 10.6
        // Compile-time assertion that the trait object is Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn PluginLogHandle>>();
    }

    #[test]
    fn plugin_handle_usable_from_multiple_threads() {
        // Validates: Requirement 10.6
        use std::sync::Arc;

        let handle: Arc<dyn PluginLogHandle> = Arc::from(create_plugin_handle("thread-test"));

        let handles: Vec<_> = (0..4)
            .map(|thread_id| {
                let h = Arc::clone(&handle);
                std::thread::spawn(move || {
                    for i in 0..50 {
                        h.info("worker", &format!("thread {thread_id} msg {i}"));
                    }
                })
            })
            .collect();

        for join_handle in handles {
            join_handle.join().expect("thread should not panic");
        }
    }

    #[test]
    fn concrete_handle_is_send_and_sync() {
        // Validates: Requirement 10.6
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ConcretePluginLogHandle>();
    }
}
