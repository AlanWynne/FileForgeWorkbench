//! # ff-large-file-performance — Rendering Optimisation Infrastructure
//!
//! This crate provides **GUI-independent rendering optimisation** for the
//! FileForgeWorkbench editor, ensuring responsive behaviour (60fps scrolling,
//! sub-frame layout computation) when working with documents containing very
//! long lines (>10,000 characters), exceeding one million lines, or both.
//!
//! ## Core Capabilities
//!
//! 1. **PositionCache** — Hash-table caching of character x-positions by
//!    (style_slot, text_content), avoiding redundant platform measurement calls.
//! 2. **LineLayoutCache** — Per-line layout result caching with LRU eviction,
//!    configurable scope (Viewport/Page/Document), and memory budget enforcement.
//! 3. **ChunkRenderer** — Subdivides visible portions of long lines into
//!    render chunks (≤300 chars) for efficient text drawing.
//! 4. **LazyLayoutManager** — Demand-driven layout engine; only measures lines
//!    within the viewport + overscan buffer.
//! 5. **InvalidationCoordinator** — Batches cache invalidation events within
//!    a frame to avoid per-keystroke invalidation storms.
//!
//! ## Design
//!
//! Adapted from Scintilla's `PositionCache`, `LineLayoutCache`, and `LineLayout`
//! concepts into a trait-based, cache-invalidation-aware Rust design.

pub mod chunk_renderer;
pub mod config;
pub mod error;
pub mod invalidation;
pub mod line_layout;
pub mod line_layout_cache;
pub mod position_cache;
pub mod scroll_predictor;
pub mod status;
pub mod surface;
pub mod types;

pub use chunk_renderer::ChunkRenderer;
pub use config::PerfConfig;
pub use error::LargeFilePerfError;
pub use invalidation::{InvalidationCoordinator, InvalidationEvent};
pub use line_layout::LineLayout;
pub use line_layout_cache::LineLayoutCache;
pub use position_cache::PositionCache;
pub use scroll_predictor::{ScrollDirection, ScrollPredictor};
pub use status::StatusIndicator;
pub use surface::Surface;
pub use types::{
    CacheLevel, CharOffset, ChunkRange, ClockValue, LongLineThreshold, RenderChunkSize, StyleSlot,
    ValidLevel, XPosition,
};
