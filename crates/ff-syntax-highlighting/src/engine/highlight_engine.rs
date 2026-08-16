//! HighlightEngine: per-document orchestrator managing style buffer, per-line state,
//! fold levels, lexer binding, incremental re-highlighting, and demand-driven styling.

use crate::engine::idle_styling::{IdleStylingConfig, IdleStylingResult};
use crate::error::SyntaxHighlightError;
use crate::fold::store::FoldData;
use crate::keywords::word_list::WordList;
use crate::lexer::traits::Lexer;
use crate::state::per_line::PerLineState;
use crate::style::buffer::StyleBuffer;
use crate::style::context::StyleContext;
use crate::style::sub_styles::{SubStyleAllocator, SubStyleRange};
use crate::types::{
    BytePosition, FoldFlags, FoldLevel, HighlightSpan, KeywordSetIndex, LineNumber, StyleSlotIndex,
    SyntaxHighlighter,
};

/// Per-document highlighting engine that implements SyntaxHighlighter.
/// Manages style buffer, per-line state, fold levels, and lexer binding.
/// Addresses: Requirements 2, 3, 4, 8, 11, 13
pub struct HighlightEngine {
    /// The style buffer parallel to document text.
    style_buffer: StyleBuffer,
    /// Per-line lexer state for incremental re-highlighting.
    per_line_state: PerLineState,
    /// Per-line fold level data.
    fold_data: FoldData,
    /// The bound lexer (None if no language assigned).
    lexer: Option<Box<dyn Lexer>>,
    /// Keyword sets (up to 9).
    keyword_sets: Vec<WordList>,
    /// Sub-style allocator.
    sub_style_allocator: SubStyleAllocator,
    /// Current styling position (furthest styled byte offset).
    styling_position: BytePosition,
    /// Document text content for demand-driven styling.
    text: String,
    /// Line start byte offsets.
    line_starts: Vec<usize>,
}

impl HighlightEngine {
    /// Create a new engine for a document with the given initial text.
    /// Addresses: Requirement 11, criterion 11.6
    pub fn new(text: &str) -> Self {
        let line_starts = compute_line_starts(text);
        let line_count = line_starts.len();

        Self {
            style_buffer: StyleBuffer::new(text.len()),
            per_line_state: PerLineState::new(line_count),
            fold_data: FoldData::new(line_count),
            lexer: None,
            keyword_sets: Vec::new(),
            sub_style_allocator: SubStyleAllocator::new(0),
            styling_position: BytePosition(0),
            text: text.to_string(),
            line_starts,
        }
    }

    /// Create a new engine with explicit length and line count (no text stored).
    pub fn new_empty(document_length: usize, line_count: usize) -> Self {
        Self {
            style_buffer: StyleBuffer::new(document_length),
            per_line_state: PerLineState::new(line_count),
            fold_data: FoldData::new(line_count),
            lexer: None,
            keyword_sets: Vec::new(),
            sub_style_allocator: SubStyleAllocator::new(0),
            styling_position: BytePosition(0),
            text: String::new(),
            line_starts: vec![0],
        }
    }

    /// Bind a lexer to this engine for a specific language.
    /// Populates keyword sets and properties from the language definition.
    /// Addresses: Requirement 13, criterion 13.1
    pub fn bind_lexer(
        &mut self,
        mut lexer: Box<dyn Lexer>,
        keyword_sets: &[Vec<String>],
        properties: &[(&str, &str)],
    ) {
        // Set properties
        for &(key, value) in properties {
            lexer.set_property(key, value);
        }

        let base_count = lexer.style_slot_count();

        // Build keyword sets
        self.keyword_sets.clear();
        for (i, words) in keyword_sets.iter().enumerate() {
            if i > KeywordSetIndex::MAX as usize {
                break;
            }
            let style = StyleSlotIndex((i as u8) + 1); // keyword sets get styles 1..=9
            let mut wl = WordList::new(style, false);
            for word in words {
                wl.add(word);
            }
            self.keyword_sets.push(wl);
        }

        self.sub_style_allocator = SubStyleAllocator::new(base_count);
        self.lexer = Some(lexer);

        // Invalidate all styling
        self.styling_position = BytePosition(0);
    }

    /// Unbind the current lexer, resetting all style data to default.
    /// Addresses: Requirement 13, criterion 13.3
    pub fn unbind_lexer(&mut self) {
        self.lexer = None;
        self.keyword_sets.clear();
        self.sub_style_allocator = SubStyleAllocator::new(0);
        // Reset style buffer to default
        self.style_buffer = StyleBuffer::new(self.text.len());
        self.styling_position = BytePosition(self.text.len());
    }

    /// Returns true if a lexer is currently bound.
    pub fn has_lexer(&self) -> bool {
        self.lexer.is_some()
    }

    /// Update the text content held by the engine.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.line_starts = compute_line_starts(text);
    }

    /// Get reference to the stored text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Notify the engine of a text insertion at the given position.
    /// Addresses: Requirement 2, criteria 2.6–2.7; Requirement 3, criterion 3.8
    pub fn notify_insert(&mut self, position: BytePosition, new_text: &str) {
        let length = new_text.len();
        let lines_inserted = new_text.chars().filter(|&c| c == '\n').count();

        // Update style buffer
        self.style_buffer.insert(position, length);

        // Update text
        let pos = position.0.min(self.text.len());
        self.text.insert_str(pos, new_text);
        self.line_starts = compute_line_starts(&self.text);

        // Update per-line state
        if lines_inserted > 0 {
            let line = line_at_position(&self.line_starts, position.0);
            self.per_line_state
                .insert_lines(LineNumber(line + 1), lines_inserted);
            self.fold_data
                .insert_lines(LineNumber(line + 1), lines_inserted);
        }

        // Invalidate styling from modified position
        self.styling_position = BytePosition(self.styling_position.0.min(position.0));
    }

    /// Notify the engine of a text deletion at the given position.
    /// Addresses: Requirement 2, criterion 2.8; Requirement 3, criterion 3.9
    pub fn notify_delete(&mut self, position: BytePosition, length: usize) {
        let pos = position.0.min(self.text.len());
        let end = (pos + length).min(self.text.len());
        let deleted_text = &self.text[pos..end];
        let lines_deleted = deleted_text.chars().filter(|&c| c == '\n').count();

        // Update style buffer
        self.style_buffer.delete(position, length);

        // Update text
        self.text.drain(pos..end);
        self.line_starts = compute_line_starts(&self.text);

        // Update per-line state
        if lines_deleted > 0 {
            let line = line_at_position(&self.line_starts, position.0);
            self.per_line_state
                .delete_lines(LineNumber(line + 1), lines_deleted);
            self.fold_data
                .delete_lines(LineNumber(line + 1), lines_deleted);
        }

        // Invalidate styling from modified position
        self.styling_position = BytePosition(self.styling_position.0.min(position.0));
    }

    /// Update a keyword set at runtime.
    /// Addresses: Requirement 5, criterion 5.8
    pub fn set_keywords(
        &mut self,
        set_index: KeywordSetIndex,
        words: &[&str],
        case_insensitive: bool,
    ) {
        let idx = set_index.value() as usize;
        let style = StyleSlotIndex(idx as u8 + 1);
        let mut wl = WordList::new(style, case_insensitive);
        for word in words {
            wl.add(word);
        }

        // Extend keyword_sets if needed
        while self.keyword_sets.len() <= idx {
            self.keyword_sets
                .push(WordList::new(StyleSlotIndex::DEFAULT, false));
        }
        self.keyword_sets[idx] = wl;

        // Invalidate all styling (keyword change may affect any position)
        self.styling_position = BytePosition(0);
    }

    /// Update a lexer property at runtime.
    /// Addresses: Requirement 10, criterion 10.3
    pub fn set_lexer_property(&mut self, key: &str, value: &str) {
        if let Some(ref mut lexer) = self.lexer {
            lexer.set_property(key, value);
            // Invalidate all styling
            self.styling_position = BytePosition(0);
        }
    }

    /// Allocate sub-styles for a base style.
    /// Addresses: Requirement 7, criterion 7.2
    pub fn allocate_sub_styles(
        &mut self,
        base_style: StyleSlotIndex,
        count: u8,
    ) -> Result<SubStyleRange, SyntaxHighlightError> {
        self.sub_style_allocator.allocate(base_style, count)
    }

    /// Free sub-styles for a base style.
    /// Addresses: Requirement 7, criterion 7.5
    pub fn free_sub_styles(&mut self, base_style: StyleSlotIndex) {
        self.sub_style_allocator.free(base_style);
    }

    /// Get the base style for a sub-style index.
    /// Addresses: Requirement 7, criterion 7.7
    pub fn sub_style_base(&self, sub_style: StyleSlotIndex) -> Option<StyleSlotIndex> {
        self.sub_style_allocator.base_for(sub_style)
    }

    /// Perform one idle styling increment. Returns the result status.
    /// Addresses: Requirement 9, criterion 9.3
    pub fn idle_style_increment(&mut self, config: &IdleStylingConfig) -> IdleStylingResult {
        if self.is_fully_styled() {
            return IdleStylingResult::Complete;
        }

        let start_time = std::time::Instant::now();
        let mut lines_styled = 0;

        while !self.is_fully_styled()
            && lines_styled < config.lines_per_slice
            && start_time.elapsed().as_millis() < config.time_budget_ms as u128
        {
            // Style one line at a time
            let current_line = line_at_position(&self.line_starts, self.styling_position.0);
            let line_end = if current_line + 1 < self.line_starts.len() {
                self.line_starts[current_line + 1]
            } else {
                self.text.len()
            };

            self.style_line(current_line);
            self.styling_position = BytePosition(line_end);
            lines_styled += 1;
        }

        if self.is_fully_styled() {
            IdleStylingResult::Complete
        } else {
            IdleStylingResult::MoreWork
        }
    }

    /// Check if idle styling is complete (entire document styled).
    /// Addresses: Requirement 9, criterion 9.5
    pub fn is_fully_styled(&self) -> bool {
        self.styling_position.0 >= self.text.len()
    }

    /// Internal: style a single line using the bound lexer.
    fn style_line(&mut self, line_idx: usize) {
        if self.lexer.is_none() {
            return;
        }

        let line_start = self.line_starts.get(line_idx).copied().unwrap_or(0);
        let line_end = if line_idx + 1 < self.line_starts.len() {
            self.line_starts[line_idx + 1]
        } else {
            self.text.len()
        };

        if line_start >= line_end {
            return;
        }

        let initial_state = self
            .per_line_state
            .state_at_line_start(LineNumber(line_idx));

        // Create a StyleContext and invoke the lexer
        let style_data = self.style_buffer.data_mut();
        let mut ctx = StyleContext::new(
            &self.text,
            line_start,
            line_end,
            initial_state,
            style_data,
            0,
        );

        if let Some(ref lexer) = self.lexer {
            lexer.style_text(&mut ctx);
        }

        // Save end-of-line state
        let end_state = ctx.state();
        self.per_line_state
            .set_state(LineNumber(line_idx), end_state);
    }

    /// Perform incremental re-highlighting from the given line until convergence.
    /// Addresses: Requirement 3, criterion 3.4
    #[allow(dead_code)]
    fn rehighlight_from(&mut self, start_line: usize) {
        if self.lexer.is_none() {
            return;
        }

        let total_lines = self.line_starts.len();
        let mut current_line = start_line;

        while current_line < total_lines {
            let prev_state = self
                .per_line_state
                .state_at_line_end(LineNumber(current_line));
            self.style_line(current_line);
            let new_state = self
                .per_line_state
                .state_at_line_end(LineNumber(current_line));

            current_line += 1;

            // State convergence: if the end-of-line state didn't change,
            // subsequent styling remains valid
            if current_line < total_lines
                && new_state == prev_state
                && current_line > start_line + 1
            {
                break;
            }
        }
    }
}

impl SyntaxHighlighter for HighlightEngine {
    /// Guarantee all text up to `position` has valid style data.
    /// Addresses: Requirement 4, criterion 4.1
    fn ensure_styled_to(&mut self, position: BytePosition) {
        // Early return if already styled past this position
        if position.0 <= self.styling_position.0 {
            return;
        }

        // No-op if no lexer bound
        if self.lexer.is_none() {
            self.styling_position = BytePosition(self.text.len());
            return;
        }

        // Style from current position to requested position
        let target = position.0.min(self.text.len());
        while self.styling_position.0 < target {
            let current_line = line_at_position(&self.line_starts, self.styling_position.0);
            let line_end = if current_line + 1 < self.line_starts.len() {
                self.line_starts[current_line + 1]
            } else {
                self.text.len()
            };

            self.style_line(current_line);
            self.styling_position = BytePosition(line_end);
        }
    }

    /// Returns the current end-of-styled-text position.
    /// Addresses: Requirement 4, criterion 4.4
    fn styling_position(&self) -> BytePosition {
        self.styling_position
    }

    /// Get the style index at a specific byte position. O(1).
    /// Addresses: Requirement 2, criterion 2.3
    fn style_at(&self, position: BytePosition) -> StyleSlotIndex {
        self.style_buffer.get(position)
    }

    /// Get contiguous styled spans within a range.
    /// Addresses: Requirement 2, criterion 2.4
    fn styled_spans(&self, start: BytePosition, end: BytePosition) -> Vec<HighlightSpan> {
        self.style_buffer.spans(start, end)
    }

    /// Get the fold level and flags for a specific line.
    /// Addresses: Requirement 8, criterion 8.5
    fn fold_level_at(&self, line: LineNumber) -> (FoldLevel, FoldFlags) {
        self.fold_data.fold_level_at(line)
    }

    /// Get fold levels for a range of lines (bulk query).
    /// Addresses: Requirement 15, criterion 15.6
    fn fold_level_range(
        &self,
        start_line: LineNumber,
        end_line: LineNumber,
    ) -> Vec<(LineNumber, FoldLevel, FoldFlags)> {
        self.fold_data.fold_level_range(start_line, end_line)
    }

    /// Get the number of base style slots the active lexer uses.
    /// Addresses: Requirement 12, criterion 12.4
    fn style_slot_count(&self) -> u8 {
        self.lexer
            .as_ref()
            .map(|l| l.style_slot_count())
            .unwrap_or(1)
    }
}

/// Compute line start byte offsets from text.
fn compute_line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// Find which line a byte position belongs to.
fn line_at_position(line_starts: &[usize], position: usize) -> usize {
    match line_starts.binary_search(&position) {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fold::context::FoldContext;
    use crate::types::{KeywordSetDescriptor, LexerState, PropertyDescriptor};
    use std::collections::HashMap;

    /// A simple test lexer that assigns style 1 to alphabetic chars and style 0 to others.
    struct SimpleTestLexer {
        properties: HashMap<String, String>,
    }

    impl SimpleTestLexer {
        fn new() -> Self {
            Self {
                properties: HashMap::new(),
            }
        }
    }

    impl Lexer for SimpleTestLexer {
        fn name(&self) -> &str {
            "simple_test"
        }

        fn style_text(&self, context: &mut StyleContext) {
            while context.more() {
                let ch = context.ch();
                if ch.is_alphabetic() {
                    context.set_style(StyleSlotIndex(1));
                } else {
                    context.set_style(StyleSlotIndex(0));
                }
                context.forward();
            }
            context.set_state(LexerState::INITIAL);
        }

        fn fold_text(&self, _context: &mut FoldContext) {}

        fn default_style(&self) -> StyleSlotIndex {
            StyleSlotIndex::DEFAULT
        }

        fn keyword_sets(&self) -> &[KeywordSetDescriptor] {
            &[]
        }

        fn sub_style_bases(&self) -> &[StyleSlotIndex] {
            &[]
        }

        fn get_property(&self, key: &str) -> Option<&str> {
            self.properties.get(key).map(|v| v.as_str())
        }

        fn set_property(&mut self, key: &str, value: &str) {
            self.properties.insert(key.to_string(), value.to_string());
        }

        fn property_names(&self) -> &[PropertyDescriptor] {
            &[]
        }

        fn style_slot_count(&self) -> u8 {
            2
        }
    }

    #[test]
    fn new_engine_has_correct_initial_state() {
        let engine = HighlightEngine::new("hello\nworld\n");
        assert_eq!(engine.styling_position(), BytePosition(0));
        assert!(!engine.has_lexer());
        assert!(!engine.is_fully_styled());
    }

    #[test]
    fn unbind_resets_to_default_style() {
        // Validates: Requirement 13, criterion 13.3
        let mut engine = HighlightEngine::new("hello");
        engine.bind_lexer(Box::new(SimpleTestLexer::new()), &[], &[]);
        engine.ensure_styled_to(BytePosition(5));
        assert_eq!(engine.style_at(BytePosition(0)), StyleSlotIndex(1));

        engine.unbind_lexer();
        assert!(!engine.has_lexer());
        assert_eq!(engine.style_at(BytePosition(0)), StyleSlotIndex::DEFAULT);
    }

    #[test]
    fn ensure_styled_to_with_no_lexer_is_noop() {
        // Validates: Requirement 4, criterion 4.7
        let mut engine = HighlightEngine::new("hello world");
        engine.ensure_styled_to(BytePosition(5));
        // No lexer means all text is treated as default-styled
        assert!(engine.is_fully_styled());
        assert_eq!(engine.style_at(BytePosition(0)), StyleSlotIndex::DEFAULT);
    }

    #[test]
    fn ensure_styled_to_early_return_when_already_styled() {
        // Validates: Requirement 4, criterion 4.2
        let mut engine = HighlightEngine::new("hello");
        engine.bind_lexer(Box::new(SimpleTestLexer::new()), &[], &[]);
        engine.ensure_styled_to(BytePosition(5));
        let pos1 = engine.styling_position();
        // Second call should be a no-op
        engine.ensure_styled_to(BytePosition(3));
        assert_eq!(engine.styling_position(), pos1);
    }

    #[test]
    fn ensure_styled_to_styles_text() {
        // Validates: Requirement 4, criterion 4.1
        let mut engine = HighlightEngine::new("hi 12");
        engine.bind_lexer(Box::new(SimpleTestLexer::new()), &[], &[]);
        engine.ensure_styled_to(BytePosition(5));
        assert_eq!(engine.style_at(BytePosition(0)), StyleSlotIndex(1)); // 'h'
        assert_eq!(engine.style_at(BytePosition(1)), StyleSlotIndex(1)); // 'i'
        assert_eq!(engine.style_at(BytePosition(2)), StyleSlotIndex(0)); // ' '
        assert_eq!(engine.style_at(BytePosition(3)), StyleSlotIndex(0)); // '1'
        assert_eq!(engine.style_at(BytePosition(4)), StyleSlotIndex(0)); // '2'
    }

    #[test]
    fn notify_insert_grows_style_buffer() {
        // Validates: Requirement 2, criterion 2.6
        let mut engine = HighlightEngine::new("hello");
        assert_eq!(engine.style_buffer.len(), 5);
        engine.notify_insert(BytePosition(2), " world");
        assert_eq!(engine.style_buffer.len(), 11);
        assert_eq!(engine.text(), "he worldllo");
    }

    #[test]
    fn notify_delete_shrinks_style_buffer() {
        // Validates: Requirement 2, criterion 2.8
        let mut engine = HighlightEngine::new("hello world");
        assert_eq!(engine.style_buffer.len(), 11);
        engine.notify_delete(BytePosition(5), 6);
        assert_eq!(engine.style_buffer.len(), 5);
        assert_eq!(engine.text(), "hello");
    }

    #[test]
    fn style_buffer_length_equals_text_length_invariant() {
        // Validates: Requirement 2, criterion 2.6 (length invariant)
        let mut engine = HighlightEngine::new("abc");
        assert_eq!(engine.style_buffer.len(), engine.text().len());
        engine.notify_insert(BytePosition(1), "XY");
        assert_eq!(engine.style_buffer.len(), engine.text().len());
        engine.notify_delete(BytePosition(0), 2);
        assert_eq!(engine.style_buffer.len(), engine.text().len());
    }

    #[test]
    fn idle_style_increment_progress() {
        // Validates: Requirement 9, criterion 9.3
        let text = "hello\nworld\nfoo\nbar\n";
        let mut engine = HighlightEngine::new(text);
        engine.bind_lexer(Box::new(SimpleTestLexer::new()), &[], &[]);

        let config = IdleStylingConfig {
            lines_per_slice: 2,
            time_budget_ms: 1000,
        };

        // First increment styles 2 lines
        let result = engine.idle_style_increment(&config);
        assert_eq!(result, IdleStylingResult::MoreWork);
        assert!(engine.styling_position().0 > 0);

        // Keep going until complete
        loop {
            let result = engine.idle_style_increment(&config);
            if result == IdleStylingResult::Complete {
                break;
            }
        }
        assert!(engine.is_fully_styled());
    }

    #[test]
    fn style_slot_count_with_no_lexer() {
        // Validates: Requirement 12, criterion 12.4
        let engine = HighlightEngine::new("test");
        assert_eq!(engine.style_slot_count(), 1);
    }

    #[test]
    fn style_slot_count_with_lexer() {
        let mut engine = HighlightEngine::new("test");
        engine.bind_lexer(Box::new(SimpleTestLexer::new()), &[], &[]);
        assert_eq!(engine.style_slot_count(), 2);
    }

    #[test]
    fn compute_line_starts_basic() {
        assert_eq!(compute_line_starts(""), vec![0]);
        assert_eq!(compute_line_starts("hello"), vec![0]);
        assert_eq!(compute_line_starts("hello\n"), vec![0, 6]);
        assert_eq!(compute_line_starts("a\nb\nc"), vec![0, 2, 4]);
    }

    #[test]
    fn line_at_position_finds_correct_line() {
        let starts = vec![0, 5, 10, 15];
        assert_eq!(line_at_position(&starts, 0), 0);
        assert_eq!(line_at_position(&starts, 3), 0);
        assert_eq!(line_at_position(&starts, 5), 1);
        assert_eq!(line_at_position(&starts, 7), 1);
        assert_eq!(line_at_position(&starts, 10), 2);
        assert_eq!(line_at_position(&starts, 14), 2);
        assert_eq!(line_at_position(&starts, 15), 3);
    }

    #[test]
    fn engine_is_send_sync() {
        // Validates: Requirement 11, criterion 11.5
        fn assert_send_sync<T: Send + Sync>() {}
        // HighlightEngine needs to be Send+Sync for thread safety
        // The Lexer trait is Send+Sync, so dyn Lexer is Send+Sync
        assert_send_sync::<HighlightEngine>();
    }

    #[test]
    fn fold_level_at_default() {
        let engine = HighlightEngine::new("line1\nline2\n");
        let (level, flags) = engine.fold_level_at(LineNumber(0));
        assert_eq!(level, FoldLevel::MIN);
        assert_eq!(flags, FoldFlags::NONE);
    }

    #[test]
    fn set_lexer_property_invalidates_styling() {
        // Validates: Requirement 10, criterion 10.4
        let mut engine = HighlightEngine::new("hello");
        engine.bind_lexer(Box::new(SimpleTestLexer::new()), &[], &[]);
        engine.ensure_styled_to(BytePosition(5));
        assert!(engine.styling_position().0 >= 5);

        engine.set_lexer_property("fold.comment", "1");
        assert_eq!(engine.styling_position(), BytePosition(0));
    }

    #[test]
    fn set_keywords_invalidates_styling() {
        // Validates: Requirement 5, criterion 5.9
        let mut engine = HighlightEngine::new("hello");
        engine.bind_lexer(Box::new(SimpleTestLexer::new()), &[], &[]);
        engine.ensure_styled_to(BytePosition(5));

        engine.set_keywords(KeywordSetIndex(0), &["hello"], false);
        assert_eq!(engine.styling_position(), BytePosition(0));
    }
}
