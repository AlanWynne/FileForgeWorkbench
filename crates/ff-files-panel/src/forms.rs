//! # Files Panel inline-form structs
//!
//! Inline dataset/member rename + delete forms and POSIX new-file/new-dir +
//! delete-confirm forms, with their validation rules and confirmation messages.
//!
//! Validates: Requirement 6.5, 6.6, 8.2, 8.3, 8.5

/// Inline rename form for a dataset or PDS member.
///
/// Validates: Requirement 6.5
#[derive(Debug, Clone)]
pub struct DatasetRenameForm {
    pub current_name: String,
    pub new_name: String,
    pub error: Option<String>,
}

impl DatasetRenameForm {
    pub fn new(current_name: impl Into<String>) -> Self {
        let current_name = current_name.into();
        let new_name = current_name.clone();
        Self {
            current_name,
            new_name,
            error: None,
        }
    }
}

/// Delete confirmation for a dataset or PDS member.
///
/// Validates: Requirement 6.6
#[derive(Debug, Clone)]
pub struct DatasetDeleteConfirm {
    pub name: String,
}

impl DatasetDeleteConfirm {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Inline new-file form for a POSIX catalog.
///
/// Validates: Requirement 8.2
#[derive(Debug, Clone)]
pub struct PosixNewFileForm {
    pub parent_path: String,
    pub filename: String,
    pub error: Option<String>,
}

impl PosixNewFileForm {
    pub fn new(parent_path: impl Into<String>) -> Self {
        Self {
            parent_path: parent_path.into(),
            filename: String::new(),
            error: None,
        }
    }

    /// Validate: 1-255 chars, no path separators, no null bytes.
    ///
    /// Validates: Requirement 8.2
    pub fn validate(&self) -> Result<(), String> {
        if self.filename.is_empty() {
            return Err("Filename cannot be empty".to_string());
        }
        if self.filename.len() > 255 {
            return Err("Filename must be 255 characters or fewer".to_string());
        }
        if self.filename.contains('/') || self.filename.contains('\\') {
            return Err("Filename must not contain path separators".to_string());
        }
        if self.filename.contains('\0') {
            return Err("Filename must not contain null bytes".to_string());
        }
        Ok(())
    }
}

/// Inline new-directory form for a POSIX catalog.
///
/// Validates: Requirement 8.3
#[derive(Debug, Clone)]
pub struct PosixNewDirForm {
    pub parent_path: String,
    pub dirname: String,
    pub error: Option<String>,
}

impl PosixNewDirForm {
    pub fn new(parent_path: impl Into<String>) -> Self {
        Self {
            parent_path: parent_path.into(),
            dirname: String::new(),
            error: None,
        }
    }

    /// Validate: same rules as filename.
    ///
    /// Validates: Requirement 8.3
    pub fn validate(&self) -> Result<(), String> {
        if self.dirname.is_empty() {
            return Err("Directory name cannot be empty".to_string());
        }
        if self.dirname.len() > 255 {
            return Err("Directory name must be 255 characters or fewer".to_string());
        }
        if self.dirname.contains('/') || self.dirname.contains('\\') {
            return Err("Directory name must not contain path separators".to_string());
        }
        if self.dirname.contains('\0') {
            return Err("Directory name must not contain null bytes".to_string());
        }
        Ok(())
    }
}

/// Delete confirmation for a POSIX file or directory.
///
/// Validates: Requirement 8.5
#[derive(Debug, Clone)]
pub struct PosixDeleteConfirm {
    pub name: String,
    pub is_directory: bool,
}

impl PosixDeleteConfirm {
    pub fn new(name: impl Into<String>, is_directory: bool) -> Self {
        Self {
            name: name.into(),
            is_directory,
        }
    }

    /// Confirmation message shown to the user.
    ///
    /// Validates: Requirement 8.5
    pub fn message(&self) -> String {
        if self.is_directory {
            format!(
                "Delete directory \"{}\" and all its contents? This cannot be undone.",
                self.name
            )
        } else {
            format!("Delete \"{}\"? This cannot be undone.", self.name)
        }
    }
}
