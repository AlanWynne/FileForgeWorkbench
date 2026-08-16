//! `ExecutionContext` — ambient state available to commands during execution.
//!
//! Constructed by the dispatch layer before invoking the command handler.

/// The ambient state available to a command during execution.
///
/// Constructed by the dispatch layer before invoking the command handler.
/// Contains the active document, cursor position, selection, and panel.
///
/// # Examples
///
/// ```
/// use ff_command::ExecutionContext;
///
/// let ctx = ExecutionContext::empty();
/// assert!(ctx.active_document.is_none());
/// ```
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// The URI of the currently active document (if any).
    pub active_document: Option<String>,
    /// Current cursor position (line, column) — 0-indexed.
    pub cursor_position: Option<(usize, usize)>,
    /// Current selection range: (start_line, start_col, end_line, end_col).
    pub selection: Option<(usize, usize, usize, usize)>,
    /// The identifier of the currently focused panel.
    pub active_panel: Option<String>,
}

impl ExecutionContext {
    /// Creates an empty execution context with no active document or selection.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates a builder for constructing an `ExecutionContext` in tests.
    pub fn builder() -> ExecutionContextBuilder {
        ExecutionContextBuilder::default()
    }
}

/// Builder for constructing `ExecutionContext` instances.
#[derive(Debug, Default)]
pub struct ExecutionContextBuilder {
    active_document: Option<String>,
    cursor_position: Option<(usize, usize)>,
    selection: Option<(usize, usize, usize, usize)>,
    active_panel: Option<String>,
}

impl ExecutionContextBuilder {
    /// Sets the active document URI.
    pub fn active_document(mut self, doc: impl Into<String>) -> Self {
        self.active_document = Some(doc.into());
        self
    }

    /// Sets the cursor position (line, column).
    pub fn cursor_position(mut self, line: usize, col: usize) -> Self {
        self.cursor_position = Some((line, col));
        self
    }

    /// Sets the selection range.
    pub fn selection(
        mut self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Self {
        self.selection = Some((start_line, start_col, end_line, end_col));
        self
    }

    /// Sets the active panel ID.
    pub fn active_panel(mut self, panel: impl Into<String>) -> Self {
        self.active_panel = Some(panel.into());
        self
    }

    /// Builds the `ExecutionContext`.
    pub fn build(self) -> ExecutionContext {
        ExecutionContext {
            active_document: self.active_document,
            cursor_position: self.cursor_position,
            selection: self.selection,
            active_panel: self.active_panel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.3
    #[test]
    fn empty_context_has_all_fields_none() {
        let ctx = ExecutionContext::empty();
        assert!(ctx.active_document.is_none());
        assert!(ctx.cursor_position.is_none());
        assert!(ctx.selection.is_none());
        assert!(ctx.active_panel.is_none());
    }

    // Validates: Requirement 2.3
    #[test]
    fn builder_sets_active_document() {
        let ctx = ExecutionContext::builder()
            .active_document("/path/to/file.txt")
            .build();
        assert_eq!(ctx.active_document.as_deref(), Some("/path/to/file.txt"));
    }

    // Validates: Requirement 2.3
    #[test]
    fn builder_sets_cursor_position() {
        let ctx = ExecutionContext::builder().cursor_position(10, 5).build();
        assert_eq!(ctx.cursor_position, Some((10, 5)));
    }

    // Validates: Requirement 2.3
    #[test]
    fn builder_sets_selection() {
        let ctx = ExecutionContext::builder().selection(1, 0, 3, 10).build();
        assert_eq!(ctx.selection, Some((1, 0, 3, 10)));
    }

    // Validates: Requirement 2.3
    #[test]
    fn builder_sets_active_panel() {
        let ctx = ExecutionContext::builder()
            .active_panel("editor_main")
            .build();
        assert_eq!(ctx.active_panel.as_deref(), Some("editor_main"));
    }
}
