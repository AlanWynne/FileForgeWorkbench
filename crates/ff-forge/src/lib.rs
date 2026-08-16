//! # ff-forge — FileForge Flat-File Processing Engine
//!
//! This crate implements the FileForge domain logic for FileForgeWorkbench:
//!
//! - **Structure parsing**: Load `.ffs` structure definition files with legacy compatibility
//! - **Record format engines**: Fixed-width (F/FB), variable-length binary (VB) with RDW
//! - **EBCDIC codec**: Code page decode/encode for mainframe binary files
//! - **COMP-3 packed decimal**: Decode, format, and encode IBM packed decimal fields
//! - **ASA carriage control**: Auto-detection and display of printer control characters
//! - **Record classification**: Multi-type record identification and filtering
//! - **Field extraction**: Byte-level field slicing with type-aware conversion
//! - **Navigation**: O(1) record seek via byte-offset index
//!
//! The crate is **GUI-independent** — it produces data models for rendering by the
//! shell layer. All file access flows through the VFS abstraction.

pub mod asa;
pub mod byte_index;
pub mod classifier;
pub mod comp3;
pub mod convert;
pub mod ebcdic;
pub mod error;
pub mod fb_reader;
pub mod field_def;
pub mod field_display;
pub mod field_edit;
pub mod field_validation;
pub mod navigation;
pub mod record_format;
pub mod record_ops;
pub mod record_structure;
pub mod structure_file;
pub mod vb_reader;
pub mod window;

pub use asa::AsaControl;
pub use byte_index::ByteOffsetIndex;
pub use classifier::{ClassificationStats, RecordClassification};
pub use comp3::{Comp3Sign, Comp3Value};
pub use convert::OutputFormat;
pub use error::FileForgeError;
pub use field_def::{DataType, FieldDefinition};
pub use field_display::{DisplayMode, FieldValue};
pub use field_validation::FieldValidator;
pub use navigation::RecordNavigator;
pub use record_format::RecordFormat;
pub use record_structure::RecordStructure;
pub use structure_file::StructureFile;
pub use vb_reader::RdwHeader;
pub use window::RecordWindow;
