//! Path resolution and native path handling.
//!
//! This module provides:
//! - [`PathResolver`] — resolves tilde, environment variables, relative paths, and canonicalization
//! - [`NativePath`] — a validated, platform-native filesystem path wrapper
//! - Platform-specific path handling (conditional compilation)

pub mod native;
pub mod platform;
pub mod resolver;

pub use native::NativePath;
pub use resolver::PathResolver;
