//! Markdown file scanning, rendering, and live-reload watching.
//!
//! This library provides three independent building blocks for Markdown
//! tooling: [`render_to_html`] converts Markdown to an HTML fragment,
//! [`Scanner`] discovers `.md` files in a directory tree, and [`FileWatcher`]
//! reports debounced changes to Markdown files on disk.

pub mod error;
pub mod renderer;
pub mod scanner;
pub mod watcher;

pub use error::MdViewerError;
pub use renderer::render_to_html;
pub use scanner::{FileEntry, Scanner};
pub use watcher::{coalesce_burst, is_markdown_path, FileWatcher};
