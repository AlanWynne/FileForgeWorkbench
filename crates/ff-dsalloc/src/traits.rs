//! Public trait interface for the Dataset Allocator service.
//!
//! The `AllocatorService` trait defines the complete public API contract that
//! dependent crates (ff-idcams) code against. This enables trait-based coupling
//! and mock implementations for testing.

use crate::config::ResolveMode;
use crate::error::JclResolverError;
use crate::pipeline::ResolveOutput;
use crate::symbols::SymbolTable;

// ─── AllocatorService Trait ─────────────────────────────────────────────────

/// The primary interface for dataset allocation workflow operations.
///
/// This trait defines the complete set of allocation operations available to
/// external consumers (ff-idcams). Dependent crates depend on this trait rather
/// than concrete implementation types, enabling mock implementations for testing.
///
/// # Errors
///
/// All fallible methods return `Result<T, JclResolverError>`.
pub trait AllocatorService: Send + Sync {
    /// Resolve all DD statements in a complete JCL job text.
    ///
    /// Runs the full four-stage pipeline: parse → substitute → resolve → validate.
    ///
    /// # Errors
    ///
    /// Returns `JclResolverError::SyntaxError` if the JCL cannot be parsed.
    fn resolve_job(
        &self,
        jcl_text: &str,
        mode: ResolveMode,
    ) -> Result<ResolveOutput, JclResolverError>;

    /// Resolve a standalone DSN against mounted catalogs.
    ///
    /// Convenience method bypassing JCL parsing — useful for REPRO/IMPORT/EXPORT
    /// commands that need to locate datasets without a full JCL context.
    ///
    /// # Errors
    ///
    /// Returns `JclResolverError::DatasetNotFound` if the DSN cannot be resolved.
    fn resolve_dsn(&self, dsn: &str) -> Result<String, JclResolverError>;

    /// Perform symbolic substitution on arbitrary text.
    ///
    /// Utility method for consumers that need symbol expansion outside of
    /// the full JCL resolution pipeline.
    ///
    /// # Errors
    ///
    /// Returns `JclResolverError::UnresolvedSymbolic` if any symbol cannot be resolved.
    fn substitute_symbols(
        &self,
        text: &str,
        symbol_table: &SymbolTable,
    ) -> Result<String, JclResolverError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 17 AC 7 — AllocatorService is object-safe
    #[test]
    fn allocator_service_is_object_safe() {
        // This test verifies that Box<dyn AllocatorService> compiles,
        // proving object safety of the trait.
        fn _assert_object_safe(_: Box<dyn AllocatorService>) {}
    }
}
