//! Completion provider trait and registry.
//!
//! This module defines the `CompletionProvider` trait that all candidate sources
//! implement (both built-in and plugin-contributed), and the `ProviderRegistry`
//! that manages provider registration, lookup, and lifecycle.

pub mod command_name;
pub mod file_path;
pub mod keyword;
pub mod line_command;
pub mod macro_name;

use std::sync::{Arc, RwLock};

use crate::candidate::CompletionCandidate;
use crate::context::CompletionContext;
use crate::error::CompletionError;

/// A unique identifier for a registered provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProviderId(pub String);

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The trait that completion candidate sources implement.
///
/// Both built-in providers and plugin-registered providers use this trait.
/// Providers are queried when the completion engine needs candidates for
/// a given context.
pub trait CompletionProvider: Send + Sync {
    /// Returns a stable identifier for this provider (for logging and deregistration).
    fn id(&self) -> &str;

    /// Returns true if this provider can produce candidates for the given context.
    ///
    /// Called before `provide_candidates` as a fast filter. Implementations
    /// should be cheap — no I/O or expensive computation.
    fn is_applicable(&self, context: &CompletionContext) -> bool;

    /// Generates completion candidates for the given context.
    ///
    /// # Errors
    ///
    /// Returns `CompletionError` if candidate generation fails.
    fn provide_candidates(
        &self,
        context: &CompletionContext,
    ) -> Result<Vec<CompletionCandidate>, CompletionError>;
}

/// A registration entry in the provider registry.
struct ProviderEntry {
    provider: Box<dyn CompletionProvider>,
}

/// The registry of all completion providers.
///
/// Thread-safe storage for provider registration, lookup, and removal.
/// Built-in providers are registered at engine initialization;
/// plugin providers can be added/removed dynamically.
pub struct ProviderRegistry {
    entries: RwLock<Vec<ProviderEntry>>,
}

impl ProviderRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
        }
    }

    /// Registers a provider.
    ///
    /// # Errors
    ///
    /// Returns `CompletionError::DuplicateProvider` if a provider with the
    /// same ID is already registered.
    pub fn register(
        &self,
        provider: Box<dyn CompletionProvider>,
    ) -> Result<ProviderId, CompletionError> {
        let id = provider.id().to_string();
        let mut entries = self
            .entries
            .write()
            .expect("provider registry lock poisoned");

        if entries.iter().any(|e| e.provider.id() == id) {
            return Err(CompletionError::DuplicateProvider { provider_id: id });
        }

        entries.push(ProviderEntry { provider });
        Ok(ProviderId(id))
    }

    /// Deregisters a provider by ID. Returns true if removed.
    pub fn deregister(&self, id: &str) -> bool {
        let mut entries = self
            .entries
            .write()
            .expect("provider registry lock poisoned");
        let len_before = entries.len();
        entries.retain(|e| e.provider.id() != id);
        entries.len() < len_before
    }

    /// Finds all providers applicable to the given context and collects their candidates.
    ///
    /// Provider errors are logged and skipped — a single provider failure does not
    /// prevent other providers from contributing candidates.
    pub fn provide_candidates(&self, context: &CompletionContext) -> Vec<CompletionCandidate> {
        let entries = self
            .entries
            .read()
            .expect("provider registry lock poisoned");
        let mut all_candidates = Vec::new();

        for entry in entries.iter() {
            if !entry.provider.is_applicable(context) {
                continue;
            }

            match entry.provider.provide_candidates(context) {
                Ok(candidates) => {
                    all_candidates.extend(candidates);
                }
                Err(_err) => {
                    // Provider failure is isolated — log and continue
                    // In production this would log via ff-logging
                }
            }
        }

        all_candidates
    }

    /// Returns the number of registered providers.
    pub fn count(&self) -> usize {
        let entries = self
            .entries
            .read()
            .expect("provider registry lock poisoned");
        entries.len()
    }

    /// Returns the IDs of all registered providers.
    pub fn provider_ids(&self) -> Vec<String> {
        let entries = self
            .entries
            .read()
            .expect("provider registry lock poisoned");
        entries
            .iter()
            .map(|e| e.provider.id().to_string())
            .collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a provider registry pre-loaded with all built-in providers.
///
/// Built-in providers:
/// - `CommandNameProvider` — command name completion
/// - `FilePathProvider` — file path completion (mock VFS)
/// - `KeywordProvider` — keyword/modifier completion
/// - `MacroNameProvider` — Lua macro name completion
/// - `LineCommandProvider` — line command prefix-area completion
pub fn create_default_registry() -> Arc<ProviderRegistry> {
    let registry = ProviderRegistry::new();

    // Register built-in providers
    let _ = registry.register(Box::new(command_name::CommandNameProvider::new()));
    let _ = registry.register(Box::new(file_path::FilePathProvider::new()));
    let _ = registry.register(Box::new(keyword::KeywordProvider::new()));
    let _ = registry.register(Box::new(macro_name::MacroNameProvider::new()));
    let _ = registry.register(Box::new(line_command::LineCommandProvider::new()));

    Arc::new(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{CompletionContextBuilder, CompletionField};

    struct TestProvider {
        id: String,
        applicable: bool,
        candidates: Vec<CompletionCandidate>,
        should_fail: bool,
    }

    impl CompletionProvider for TestProvider {
        fn id(&self) -> &str {
            &self.id
        }

        fn is_applicable(&self, _context: &CompletionContext) -> bool {
            self.applicable
        }

        fn provide_candidates(
            &self,
            _context: &CompletionContext,
        ) -> Result<Vec<CompletionCandidate>, CompletionError> {
            if self.should_fail {
                Err(CompletionError::ProviderFailed {
                    provider_id: self.id.clone(),
                    reason: "test failure".to_string(),
                })
            } else {
                Ok(self.candidates.clone())
            }
        }
    }

    fn make_provider(
        id: &str,
        applicable: bool,
        candidates: Vec<CompletionCandidate>,
    ) -> Box<dyn CompletionProvider> {
        Box::new(TestProvider {
            id: id.to_string(),
            applicable,
            candidates,
            should_fail: false,
        })
    }

    fn make_failing_provider(id: &str) -> Box<dyn CompletionProvider> {
        Box::new(TestProvider {
            id: id.to_string(),
            applicable: true,
            candidates: vec![],
            should_fail: true,
        })
    }

    // Validates: Requirement 10.1, 10.2 (registration)
    #[test]
    fn register_and_count_providers() {
        let registry = ProviderRegistry::new();
        let _ = registry
            .register(make_provider("test1", true, vec![]))
            .unwrap();
        let _ = registry
            .register(make_provider("test2", true, vec![]))
            .unwrap();
        assert_eq!(registry.count(), 2);
    }

    // Validates: Requirement 10.2 (duplicate detection)
    #[test]
    fn duplicate_registration_returns_error() {
        let registry = ProviderRegistry::new();
        let _ = registry
            .register(make_provider("dup", true, vec![]))
            .unwrap();
        let result = registry.register(make_provider("dup", true, vec![]));
        assert!(result.is_err());
    }

    // Validates: Requirement 10.3 (deregistration)
    #[test]
    fn deregister_removes_provider() {
        let registry = ProviderRegistry::new();
        let _ = registry
            .register(make_provider("removable", true, vec![]))
            .unwrap();
        assert_eq!(registry.count(), 1);
        assert!(registry.deregister("removable"));
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn deregister_nonexistent_returns_false() {
        let registry = ProviderRegistry::new();
        assert!(!registry.deregister("nonexistent"));
    }

    // Validates: Requirement 10.4, 10.6 (context-based provider lookup)
    #[test]
    fn provide_candidates_filters_by_applicability() {
        use crate::candidate::CompletionKind;

        let registry = ProviderRegistry::new();
        let c1 = CompletionCandidate::new("FIND", "find", CompletionKind::Command);
        let c2 = CompletionCandidate::new("secret", "secret", CompletionKind::Plugin);

        let _ = registry.register(make_provider("applicable", true, vec![c1.clone()]));
        let _ = registry.register(make_provider("not_applicable", false, vec![c2]));

        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .prefix("fi")
            .build();

        let candidates = registry.provide_candidates(&ctx);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].label, "FIND");
    }

    // Validates: Requirement 10.5 (provider failure isolation)
    #[test]
    fn failing_provider_does_not_prevent_other_providers() {
        use crate::candidate::CompletionKind;

        let registry = ProviderRegistry::new();
        let c1 = CompletionCandidate::new("GOOD", "good", CompletionKind::Command);

        let _ = registry.register(make_failing_provider("bad_provider"));
        let _ = registry.register(make_provider("good_provider", true, vec![c1]));

        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .prefix("g")
            .build();

        let candidates = registry.provide_candidates(&ctx);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].label, "GOOD");
    }

    // Validates: Requirement 10.6 (built-in providers use same trait)
    #[test]
    fn default_registry_has_all_builtin_providers() {
        let registry = create_default_registry();
        let ids = registry.provider_ids();
        assert!(ids.contains(&"command_name".to_string()));
        assert!(ids.contains(&"file_path".to_string()));
        assert!(ids.contains(&"keyword".to_string()));
        assert!(ids.contains(&"macro_name".to_string()));
        assert!(ids.contains(&"line_command".to_string()));
    }
}
