//! Custom operation types for provider-specific operations.
//!
//! Provides helper types for the `custom_operation` escape hatch on the
//! `ConnectorPlugin` trait. These types document the expected patterns for
//! z/OS-specific operations (JES spool access, job submission) and other
//! provider-specific operations that don't map to standard VFS methods.
//!
//! ## z/OS-Specific Custom Operation Patterns
//!
//! ### JES Spool Access
//! ```text
//! custom_operation("jes_spool", &CustomOperationRequest {
//!     operation_name: "jes_spool".to_string(),
//!     parameters: /* JesSpoolParams as Box<dyn Any> */,
//! })
//! ```
//!
//! ### Job Submission
//! ```text
//! custom_operation("submit_job", &CustomOperationRequest {
//!     operation_name: "submit_job".to_string(),
//!     parameters: /* JobSubmitParams as Box<dyn Any> */,
//! })
//! ```
//!
//! ## Usage
//!
//! Future connector implementations use `custom_operation` for operations
//! that cannot be expressed through the standard VFS interface. The operation
//! name serves as the dispatch key, and the params are downcast by the
//! connector implementation.

use std::any::Any;
use std::fmt;

/// A custom operation request passed to `ConnectorPlugin::custom_operation`.
///
/// Carries a named operation and type-erased parameters that the connector
/// implementation can downcast to the expected type.
///
/// Addresses: Requirement 6 AC 3, AC 6
#[derive(Debug)]
pub struct CustomOperationRequest {
    /// The name of the custom operation (e.g., "jes_spool", "submit_job").
    pub operation_name: String,
    /// A human-readable description of the operation for logging.
    pub description: Option<String>,
}

/// A custom operation response from `ConnectorPlugin::custom_operation`.
///
/// Wraps the type-erased result with metadata about the operation that produced it.
///
/// Addresses: Requirement 6 AC 3, AC 6
pub struct CustomOperationResponse {
    /// The name of the operation that produced this response.
    pub operation_name: String,
    /// The type-erased result value.
    pub result: Box<dyn Any + Send>,
}

impl fmt::Debug for CustomOperationResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomOperationResponse")
            .field("operation_name", &self.operation_name)
            .field("result", &"<dyn Any>")
            .finish()
    }
}
