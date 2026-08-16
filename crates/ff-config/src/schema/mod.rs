//! Configuration schema and validation.
//!
//! Defines the schema registry, schema entries, and constraint metadata used
//! to validate configuration values, provide defaults, and support settings UI
//! generation.

pub mod constraint;
pub mod entry;
pub mod registry;

pub use constraint::Constraints;
pub use entry::SchemaEntry;
pub use registry::SchemaRegistry;
