//! `ExecutionContext` — ambient state available to commands during execution.
//!
//! Constructed by the dispatch layer before invoking the command handler.

use std::collections::BTreeMap;

/// A typed value stored in the [`CursorContext`] extras bag.
///
/// Mirrors the value kinds of `ParamValue` so workspace-specific context is
/// typed but open-ended. A command reads extras by key and interprets the
/// `ContextValue` it finds, ignoring keys/kinds it does not recognise.
///
/// Validates: command-framework Requirement 12.3 (CR-CH-028).
#[derive(Debug, Clone, PartialEq)]
pub enum ContextValue {
    /// A string value.
    String(String),
    /// A signed integer value.
    Int(i64),
    /// A floating-point value.
    Float(f64),
    /// A boolean value.
    Bool(bool),
}

/// A per-invocation snapshot of "where the cursor/focus was" when a command was
/// invoked, carried on the [`ExecutionContext`].
///
/// It has a fixed strongly-typed CORE (the common fields every workspace may
/// populate) and an open EXTRAS bag (a string-keyed map of [`ContextValue`]) so a
/// workspace MAY attach workspace-specific context a command may consult or
/// ignore. Every CORE field is optional: a workspace populates what applies.
///
/// A command MAY use this package or ignore it entirely; it is ADDITIVE and does
/// not by itself change any command's behaviour (command-framework
/// Requirement 12.1).
///
/// Validates: command-framework Requirement 12.1, 12.2, 12.3 (CR-CH-028).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CursorContext {
    /// The focused Workspace's stable per-kind context name (e.g. `"pom"`,
    /// `"editor"`, `"menu"`), or `None` when unknown.
    pub workspace_context: Option<String>,
    /// A semantic identity of the focused interior control (e.g. the command
    /// string / label of the focused Menu_Option, or `"command-line"`), or
    /// `None` when nothing interior is focused.
    pub focused_identity: Option<String>,
    /// The focused control's text where meaningful (e.g. the command-line text
    /// or the focused option's label), or `None`.
    pub focused_text: Option<String>,
    /// Editor cursor line (when an editor document is active), 1-indexed as the
    /// shell reports it, else `None`.
    pub cursor_line: Option<usize>,
    /// Editor cursor column (when an editor document is active), else `None`.
    pub cursor_column: Option<usize>,
    /// Current selection range `(start_line, start_col, end_line, end_col)`, else
    /// `None`.
    pub selection: Option<(usize, usize, usize, usize)>,
    /// The active scroll setting (e.g. `"CSR"`, `"HALF"`, `"PAGE"`), for a future
    /// cursor-relative scroll command to consult, else `None`.
    pub scroll_setting: Option<String>,
    /// Open extras bag: workspace-specific context keyed by name. Commands read
    /// by key and ignore what they do not recognise (Requirement 12.3).
    pub extras: BTreeMap<String, ContextValue>,
}

impl CursorContext {
    /// The empty package (all core `None`, empty extras).
    pub fn empty() -> Self {
        Self::default()
    }

    /// A builder for constructing a `CursorContext`.
    pub fn builder() -> CursorContextBuilder {
        CursorContextBuilder::default()
    }
}

/// Builder for [`CursorContext`].
#[derive(Debug, Default)]
pub struct CursorContextBuilder {
    inner: CursorContext,
}

impl CursorContextBuilder {
    /// Set the focused Workspace context name.
    pub fn workspace_context(mut self, s: impl Into<String>) -> Self {
        self.inner.workspace_context = Some(s.into());
        self
    }

    /// Set the focused-control semantic identity.
    pub fn focused_identity(mut self, s: impl Into<String>) -> Self {
        self.inner.focused_identity = Some(s.into());
        self
    }

    /// Set the focused-control text.
    pub fn focused_text(mut self, s: impl Into<String>) -> Self {
        self.inner.focused_text = Some(s.into());
        self
    }

    /// Set the editor cursor line.
    pub fn cursor_line(mut self, line: usize) -> Self {
        self.inner.cursor_line = Some(line);
        self
    }

    /// Set the editor cursor column.
    pub fn cursor_column(mut self, col: usize) -> Self {
        self.inner.cursor_column = Some(col);
        self
    }

    /// Set the selection range.
    pub fn selection(
        mut self,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Self {
        self.inner.selection = Some((start_line, start_col, end_line, end_col));
        self
    }

    /// Set the active scroll setting.
    pub fn scroll_setting(mut self, s: impl Into<String>) -> Self {
        self.inner.scroll_setting = Some(s.into());
        self
    }

    /// Attach a workspace-specific extra value under `key`.
    pub fn extra(mut self, key: impl Into<String>, value: ContextValue) -> Self {
        self.inner.extras.insert(key.into(), value);
        self
    }

    /// Build the `CursorContext`.
    pub fn build(self) -> CursorContext {
        self.inner
    }
}

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
    /// The per-invocation Cursor_Context package (CR-CH-028, Requirement 12).
    /// Additive: a handler that ignores it behaves exactly as before.
    pub cursor_context: CursorContext,
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
    cursor_context: CursorContext,
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

    /// Sets the Cursor_Context package (CR-CH-028).
    pub fn cursor_context(mut self, cc: CursorContext) -> Self {
        self.cursor_context = cc;
        self
    }

    /// Builds the `ExecutionContext`.
    pub fn build(self) -> ExecutionContext {
        ExecutionContext {
            active_document: self.active_document,
            cursor_position: self.cursor_position,
            selection: self.selection,
            active_panel: self.active_panel,
            cursor_context: self.cursor_context,
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

    // === CR-CH-028: Cursor_Context package (Requirement 12) ================

    // Validates: command-framework Requirement 12.1 -- an empty ExecutionContext
    // carries an empty Cursor_Context (all core None, extras empty); additive so
    // existing empty() semantics are unchanged.
    #[test]
    fn empty_context_has_empty_cursor_context() {
        let ctx = ExecutionContext::empty();
        let cc = &ctx.cursor_context;
        assert!(cc.workspace_context.is_none());
        assert!(cc.focused_identity.is_none());
        assert!(cc.focused_text.is_none());
        assert!(cc.cursor_line.is_none());
        assert!(cc.cursor_column.is_none());
        assert!(cc.selection.is_none());
        assert!(cc.scroll_setting.is_none());
        assert!(cc.extras.is_empty());
    }

    // Validates: command-framework Requirement 12.2 -- the Cursor_Context CORE
    // fields round-trip through the builder.
    #[test]
    fn builder_sets_cursor_context_core() {
        let cc = CursorContext::builder()
            .workspace_context("pom")
            .focused_identity("FILES")
            .focused_text("File Explorer")
            .cursor_line(3)
            .cursor_column(7)
            .selection(1, 0, 2, 4)
            .scroll_setting("CSR")
            .build();
        let ctx = ExecutionContext::builder().cursor_context(cc).build();
        let cc = &ctx.cursor_context;
        assert_eq!(cc.workspace_context.as_deref(), Some("pom"));
        assert_eq!(cc.focused_identity.as_deref(), Some("FILES"));
        assert_eq!(cc.focused_text.as_deref(), Some("File Explorer"));
        assert_eq!(cc.cursor_line, Some(3));
        assert_eq!(cc.cursor_column, Some(7));
        assert_eq!(cc.selection, Some((1, 0, 2, 4)));
        assert_eq!(cc.scroll_setting.as_deref(), Some("CSR"));
    }

    // Validates: command-framework Requirement 12.3 -- the open EXTRAS bag holds
    // typed, string-keyed Context_Values a workspace may attach.
    #[test]
    fn cursor_context_extras_bag_holds_typed_values() {
        let cc = CursorContext::builder()
            .extra("catalog", ContextValue::String("HOME".to_string()))
            .extra("row", ContextValue::Int(12))
            .extra("dirty", ContextValue::Bool(true))
            .extra("ratio", ContextValue::Float(0.5))
            .build();
        assert_eq!(
            cc.extras.get("catalog"),
            Some(&ContextValue::String("HOME".to_string()))
        );
        assert_eq!(cc.extras.get("row"), Some(&ContextValue::Int(12)));
        assert_eq!(cc.extras.get("dirty"), Some(&ContextValue::Bool(true)));
        assert_eq!(cc.extras.get("ratio"), Some(&ContextValue::Float(0.5)));
        // A key a command does not recognise is simply absent.
        assert_eq!(cc.extras.get("unknown"), None);
    }

    // Validates: command-framework Requirement 12.2/12.3 -- default Cursor_Context
    // is the empty package.
    #[test]
    fn cursor_context_default_is_empty() {
        let cc = CursorContext::default();
        assert!(cc.workspace_context.is_none());
        assert!(cc.extras.is_empty());
    }
}
