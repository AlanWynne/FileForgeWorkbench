//! IndentDecision type and related result structures.
//!
//! Represents the computed indentation to apply after a newline insertion
//! or character-typed event.

/// The result of an auto-indent computation.
///
/// Describes what indentation to apply after a newline insertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndentDecision {
    /// The indentation string to prepend to the new line.
    pub indent_text: String,
    /// Optional comment continuation marker to insert after the indent.
    pub comment_continuation: Option<CommentContinuation>,
    /// If block expansion is needed, the additional line(s) to insert.
    pub block_expansion: Option<BlockExpansion>,
    /// The logical indent level of the new line (for debugging/logging).
    pub indent_level: u32,
}

impl IndentDecision {
    /// Create a decision with no indentation (mode = None).
    pub fn no_indent() -> Self {
        Self {
            indent_text: String::new(),
            comment_continuation: None,
            block_expansion: None,
            indent_level: 0,
        }
    }

    /// Create a maintain-indent decision copying reference whitespace.
    pub fn maintain(indent_text: String, indent_level: u32) -> Self {
        Self {
            indent_text,
            comment_continuation: None,
            block_expansion: None,
            indent_level,
        }
    }

    /// Create a smart-indent decision with computed indent level.
    pub fn smart(indent_text: String, indent_level: u32) -> Self {
        Self {
            indent_text,
            comment_continuation: None,
            block_expansion: None,
            indent_level,
        }
    }

    /// Create a block expansion decision.
    pub fn block_expand(indent_text: String, indent_level: u32, expansion: BlockExpansion) -> Self {
        Self {
            indent_text,
            comment_continuation: None,
            block_expansion: Some(expansion),
            indent_level,
        }
    }

    /// The total text to insert at the start of the new line
    /// (indent + optional comment continuation marker).
    pub fn full_prefix(&self) -> String {
        match &self.comment_continuation {
            Some(cont) => format!("{}{}", self.indent_text, cont.marker),
            None => self.indent_text.clone(),
        }
    }
}

/// Describes the comment continuation marker for a new line inside a comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentContinuation {
    /// The continuation marker text (e.g., " * " or "// ").
    pub marker: String,
    /// The type of comment being continued.
    pub kind: CommentKind,
}

/// The kind of comment being continued.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    /// Inside a block comment (e.g., /* ... */).
    Block,
    /// A line comment continuation (e.g., // ...).
    Line,
}

/// Describes additional lines inserted during Enter-between-braces expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockExpansion {
    /// The closing line content (e.g., `}`).
    pub closing_text: String,
    /// The indent string for the closing line (same level as the opening).
    pub closing_indent: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_indent_returns_empty_text() {
        let decision = IndentDecision::no_indent();
        assert_eq!(decision.indent_text, "");
        assert_eq!(decision.indent_level, 0);
        assert!(decision.comment_continuation.is_none());
        assert!(decision.block_expansion.is_none());
    }

    #[test]
    fn maintain_stores_whitespace() {
        let decision = IndentDecision::maintain("    ".to_string(), 1);
        assert_eq!(decision.indent_text, "    ");
        assert_eq!(decision.indent_level, 1);
    }

    #[test]
    fn full_prefix_without_comment() {
        let decision = IndentDecision::smart("    ".to_string(), 1);
        assert_eq!(decision.full_prefix(), "    ");
    }

    #[test]
    fn full_prefix_with_comment_continuation() {
        let mut decision = IndentDecision::smart(" ".to_string(), 0);
        decision.comment_continuation = Some(CommentContinuation {
            marker: "* ".to_string(),
            kind: CommentKind::Block,
        });
        assert_eq!(decision.full_prefix(), " * ");
    }
}
