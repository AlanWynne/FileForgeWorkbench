//! Terminal subsystem: emulation, PTY abstraction, and session management.
//!
//! Provides VT100/ANSI terminal emulation, platform-specific PTY implementations,
//! and multi-session terminal lifecycle management.

pub mod cell;
pub mod emulator;
pub mod grid;
pub mod manager;
pub mod pty;
