//! # Toolchain Panel (thin adapter)
//!
//! The Toolchain Panel implementation now lives in the standalone
//! `ff-toolchain-panel` crate (decomposition Wave 2). This module re-exports
//! the crate's public API so existing `crate::toolchain_panel::*` references in
//! the shell (`shell/mod.rs`, `shell/render.rs`) resolve unchanged.
//!
//! There is no `WorkspaceContext` impl here: the panel is a bottom dock, and
//! the shell owns its `ToolchainPanelState` field, the `show_toolchain_panel`
//! visibility flag, and the editor-navigation side effect driven by `render`'s
//! return value.

pub use ff_toolchain_panel::{render, ToolchainPanelState};
