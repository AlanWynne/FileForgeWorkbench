//! # ff-database-tool — Integrated Database IDE Plugin
//!
//! This crate provides a full-featured integrated Database IDE delivered as a
//! workbench plugin. It adapts DBeaver Community Edition capabilities to the
//! FileForgeWorkbench ecosystem.
//!
//! ## Features
//!
//! - **Connection Management** — Create, edit, test, and manage database connections
//!   with credential security and SSH tunnelling.
//! - **Driver Registry** — Registry of available Rust database drivers with
//!   capability detection.
//! - **SQL Editor** — Multi-statement SQL editing with dialect-aware syntax
//!   highlighting, auto-complete, and query execution.
//! - **Result Grid** — Scrollable, sortable, filterable grid with cell editing.
//! - **Schema Browser** — Hierarchical tree of database objects with lazy loading.
//! - **ER Diagram** — Visual entity-relationship diagrams on a zoomable canvas.
//! - **Data Transfer** — Wizard-driven import/export/migration workflows.
//! - **Administration** — Session monitoring, lock inspection, performance dashboards.
//!
//! ## Architecture
//!
//! The database tool integrates with the workbench platform through:
//! - Plugin Architecture (`ff-plugin`): registers as a `FileForgePlugin`
//! - Command Framework (`ff-command`): all operations are registered commands
//! - Layout and Docking (`ff-layout`): all panels implement `DockablePanel`
//! - Workflow Engine (`ff-workflow`): data transfer operations are workflows
//! - Virtual File System (`ff-vfs`): SQL scripts and export files via VFS

pub mod connection;
pub mod driver;
pub mod error;
pub mod sql;
pub mod types;

pub use connection::{ConnectionDescriptor, ConnectionState, ConnectionType};
pub use driver::{DriverCapabilities, DriverDefinition, DriverRegistry};
pub use error::DatabaseToolError;
pub use sql::{SqlParser, StatementBoundary};
pub use types::{ConnectionId, ExecutionId, IsolationLevel, SqlDialect, SqlType, SslMode};
