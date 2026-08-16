//! # ff-lua — Lua Macro Engine for FileForgeWorkbench
//!
//! This crate is the **scripting and automation layer** for the
//! FileForgeWorkbench platform. It embeds a Lua 5.4 runtime (via `mlua`),
//! exposes a rich editor API, manages event hooks, provides per-buffer
//! state isolation, auto-reloads modified scripts, enforces security modes,
//! and registers primary commands (MACRO, EXEC, RUN).
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │           LuaMacroEngine (this crate)        │
//! │  Lua runtime, editor API, hooks, commands   │
//! ├─────────────────────────────────────────────┤
//! │  ff-command │ ff-plugin │ ff-config          │
//! └─────────────────────────────────────────────┘
//! ```
//!
//! ## Key Types
//!
//! - [`LuaMacroEngine`] — core engine owning the Lua runtime
//! - [`HookEvent`] — supported event hook types
//! - [`HookRegistry`] — event-to-handler mapping
//! - [`SecurityGate`] — security mode enforcement
//! - [`BufferStateManager`] — per-buffer Lua table swap
//! - [`EngineConfig`] — configuration model
//! - [`LuaEngineError`] — error types

// ─── Modules ────────────────────────────────────────────────────────────────

pub mod buffer_state;
pub mod config;
pub mod engine;
pub mod error;
pub mod hooks;
pub mod scanner;
pub mod security;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use buffer_state::{BufferId, BufferStateManager};
pub use config::EngineConfig;
pub use engine::LuaMacroEngine;
pub use error::{LuaEngineError, LuaResult};
pub use hooks::event::HookEvent;
pub use hooks::registry::{HookDispatchResult, HookHandler, HookRegistry};
pub use scanner::{DirectoryPriority, MacroScript};
pub use security::{SecurityDecision, SecurityGate, SecurityMode, SecurityPermission, StdlibSet};
