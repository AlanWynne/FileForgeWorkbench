//! Keyboard shortcut management — chord definitions, registry, conflict detection.

pub mod chord;
pub mod conflict;
pub mod registry;
pub mod reserved;
pub mod sequence;

pub use chord::{KeyChord, KeyCode, Modifiers};
pub use registry::ShortcutRegistry;
pub use sequence::ShortcutBinding;
