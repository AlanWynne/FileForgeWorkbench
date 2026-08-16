//! Top-level auto-indent service facade.
//!
//! The `AutoIndentService` is the central entry point for all auto-indentation
//! operations. It coordinates mode selection, pattern matching, block expansion,
//! and comment continuation into a unified API.

use std::collections::HashMap;
use std::sync::RwLock;

use crate::block::try_block_expansion;
use crate::comment::{compute_comment_continuation, CommentConfig};
use crate::config::IndentConfig;
use crate::decision::IndentDecision;
use crate::maintain::compute_maintain_indent;
use crate::mode::{resolve_effective_mode, AutoIndentMode};
use crate::patterns::IndentPatterns;
use crate::smart::{compute_decrease_on_type, compute_smart_indent, IndentContext};

/// The central auto-indentation service.
///
/// Thread-safe: all computation methods take `&self`.
/// Configuration and pattern caches are behind `RwLock`.
pub struct AutoIndentService {
    /// Cached indent configuration (updated on hot-reload).
    config: RwLock<IndentConfig>,
    /// Current auto-indent mode (updated on hot-reload).
    mode: RwLock<AutoIndentMode>,
    /// Compiled pattern cache, keyed by language_id.
    pattern_cache: RwLock<HashMap<String, IndentPatterns>>,
    /// Comment configuration cache, keyed by language_id.
    comment_cache: RwLock<HashMap<String, CommentConfig>>,
}

impl AutoIndentService {
    /// Create a new `AutoIndentService` with the given initial configuration.
    pub fn new(config: IndentConfig, mode: AutoIndentMode) -> Self {
        Self {
            config: RwLock::new(config),
            mode: RwLock::new(mode),
            pattern_cache: RwLock::new(HashMap::new()),
            comment_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Update configuration after a hot-reload event.
    pub fn update_config(&self, config: IndentConfig) {
        *self.config.write().unwrap() = config;
    }

    /// Update the auto-indent mode after a hot-reload event.
    pub fn update_mode(&self, mode: AutoIndentMode) {
        *self.mode.write().unwrap() = mode;
    }

    /// Get the currently active indent configuration.
    pub fn config(&self) -> IndentConfig {
        *self.config.read().unwrap()
    }

    /// Get the currently active auto-indent mode.
    pub fn mode(&self) -> AutoIndentMode {
        *self.mode.read().unwrap()
    }

    /// Load and cache indent patterns for a language.
    pub fn set_language_patterns(&self, language_id: &str, patterns: IndentPatterns) {
        self.pattern_cache
            .write()
            .unwrap()
            .insert(language_id.to_string(), patterns);
    }

    /// Load and cache comment config for a language.
    pub fn set_comment_config(&self, language_id: &str, config: CommentConfig) {
        self.comment_cache
            .write()
            .unwrap()
            .insert(language_id.to_string(), config);
    }

    /// Get cached indent patterns for a language.
    pub fn get_patterns(&self, language_id: &str) -> Option<IndentPatterns> {
        self.pattern_cache.read().unwrap().get(language_id).cloned()
    }

    /// Get cached comment config for a language.
    pub fn get_comment_config(&self, language_id: &str) -> Option<CommentConfig> {
        self.comment_cache.read().unwrap().get(language_id).cloned()
    }

    /// Clear the pattern and comment caches.
    pub fn clear_cache(&self) {
        self.pattern_cache.write().unwrap().clear();
        self.comment_cache.write().unwrap().clear();
    }

    /// Resolve the effective mode for the current document context.
    pub fn resolve_effective_mode(
        &self,
        has_language_patterns: bool,
        language_mode_override: Option<AutoIndentMode>,
    ) -> AutoIndentMode {
        let global_mode = self.mode();
        resolve_effective_mode(global_mode, has_language_patterns, language_mode_override)
    }

    /// Compute the indentation for a newly created line after Enter.
    ///
    /// This is the primary entry point. It coordinates mode selection
    /// and delegates to the appropriate engine:
    /// - None → no indent
    /// - Maintain → copy reference whitespace
    /// - Smart → priority: block expansion → comment continuation → pattern-based
    pub fn compute_newline_indent(
        &self,
        context: &IndentContext,
        patterns: &IndentPatterns,
        comment_config: &CommentConfig,
    ) -> IndentDecision {
        let config = self.config();
        let effective_mode = self.resolve_effective_mode(!patterns.is_empty(), None);

        match effective_mode {
            AutoIndentMode::None => IndentDecision::no_indent(),
            AutoIndentMode::Maintain => {
                // Check for comment continuation first
                if let Some(decision) =
                    compute_comment_continuation(context, comment_config, &config)
                {
                    return decision;
                }
                compute_maintain_indent(&context.reference_line, context.caret_column, &config)
            }
            AutoIndentMode::Smart => {
                // Priority 1: Block expansion (Enter between braces)
                if let Some(decision) = try_block_expansion(context, patterns, &config) {
                    return decision;
                }

                // Priority 2: Comment continuation
                if let Some(decision) =
                    compute_comment_continuation(context, comment_config, &config)
                {
                    return decision;
                }

                // Priority 3: Smart indent (pattern-based)
                compute_smart_indent(context, patterns, &config)
            }
        }
    }

    /// Compute the indentation adjustment when a character is typed that
    /// completes a decrease pattern match.
    ///
    /// Returns `Some(new_indent_text)` if the line should be re-indented,
    /// or `None` if no adjustment is needed.
    pub fn compute_char_indent(
        &self,
        current_line_content: &str,
        caret_column: u64,
        patterns: &IndentPatterns,
    ) -> Option<String> {
        let config = self.config();
        compute_decrease_on_type(current_line_content, caret_column, patterns, &config)
    }
}

impl std::fmt::Debug for AutoIndentService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AutoIndentService")
            .field("mode", &self.mode())
            .field("config", &self.config())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::IndentStyle;
    use crate::patterns::c_like_patterns;

    fn make_service() -> AutoIndentService {
        AutoIndentService::new(
            IndentConfig::new(4, 4, IndentStyle::Spaces),
            AutoIndentMode::Smart,
        )
    }

    #[test]
    fn none_mode_returns_no_indent() {
        // Validates: Requirement 10.3 — None mode produces zero indent
        let service = AutoIndentService::new(
            IndentConfig::new(4, 4, IndentStyle::Spaces),
            AutoIndentMode::None,
        );
        let patterns = c_like_patterns();
        let comment = CommentConfig::empty();
        let ctx = IndentContext::simple("    if (true) {", 15);

        let result = service.compute_newline_indent(&ctx, &patterns, &comment);
        assert_eq!(result.indent_text, "");
        assert_eq!(result.indent_level, 0);
    }

    #[test]
    fn maintain_mode_copies_whitespace() {
        // Validates: Requirement 2.1
        let service = AutoIndentService::new(
            IndentConfig::new(4, 4, IndentStyle::Spaces),
            AutoIndentMode::Maintain,
        );
        let patterns = IndentPatterns::empty();
        let comment = CommentConfig::empty();
        let ctx = IndentContext::simple("        hello", 13);

        let result = service.compute_newline_indent(&ctx, &patterns, &comment);
        assert_eq!(result.indent_text, "        ");
    }

    #[test]
    fn smart_mode_increases_on_brace() {
        // Validates: Requirement 3.1
        let service = make_service();
        let patterns = c_like_patterns();
        let comment = CommentConfig::empty();
        let ctx = IndentContext::simple("    if (true) {", 15);

        let result = service.compute_newline_indent(&ctx, &patterns, &comment);
        assert_eq!(result.indent_text, "        ");
        assert_eq!(result.indent_level, 2);
    }

    #[test]
    fn smart_mode_block_expansion_takes_priority() {
        // Validates: Requirement 5.1 — block expansion has highest priority in smart mode
        let service = make_service();
        let patterns = c_like_patterns();
        let comment = CommentConfig::empty();
        let ctx = IndentContext {
            reference_line: "    fn main() {}".to_string(),
            caret_column: 15,
            in_comment: false,
            in_block_comment: false,
            is_empty_comment_continuation: false,
            text_before_caret: "    fn main() {".to_string(),
            text_after_caret: "}".to_string(),
        };

        let result = service.compute_newline_indent(&ctx, &patterns, &comment);
        assert!(result.block_expansion.is_some());
        assert_eq!(result.indent_text, "        "); // level 2
    }

    #[test]
    fn smart_mode_comment_continuation() {
        // Validates: Requirement 6.1
        let service = make_service();
        let patterns = c_like_patterns();
        let comment = CommentConfig::c_style();
        let ctx = IndentContext {
            reference_line: "     * some doc".to_string(),
            caret_column: 15,
            in_comment: true,
            in_block_comment: true,
            is_empty_comment_continuation: false,
            text_before_caret: "     * some doc".to_string(),
            text_after_caret: String::new(),
        };

        let result = service.compute_newline_indent(&ctx, &patterns, &comment);
        assert!(result.comment_continuation.is_some());
    }

    #[test]
    fn hot_reload_config_update() {
        // Validates: Requirement 1.4 — hot-reload updates config
        let service = make_service();
        assert_eq!(service.config().indent_size(), 4);

        service.update_config(IndentConfig::new(2, 4, IndentStyle::Spaces));
        assert_eq!(service.config().indent_size(), 2);
    }

    #[test]
    fn hot_reload_mode_update() {
        // Validates: Requirement 1.4 — hot-reload updates mode
        let service = make_service();
        assert_eq!(service.mode(), AutoIndentMode::Smart);

        service.update_mode(AutoIndentMode::Maintain);
        assert_eq!(service.mode(), AutoIndentMode::Maintain);
    }

    #[test]
    fn pattern_cache_operations() {
        // Validates: Requirement 9.5 — language change reloads patterns
        let service = make_service();
        assert!(service.get_patterns("rust").is_none());

        service.set_language_patterns("rust", c_like_patterns());
        assert!(service.get_patterns("rust").is_some());

        service.clear_cache();
        assert!(service.get_patterns("rust").is_none());
    }

    #[test]
    fn compute_char_indent_decrease() {
        // Validates: Requirement 4.1
        let service = make_service();
        let patterns = c_like_patterns();

        let result = service.compute_char_indent("        }", 9, &patterns);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "    ");
    }

    #[test]
    fn compute_char_indent_no_decrease() {
        // Validates: Requirement 4.3
        let service = make_service();
        let patterns = c_like_patterns();

        let result = service.compute_char_indent("        x", 9, &patterns);
        assert!(result.is_none());
    }

    #[test]
    fn resolve_effective_mode_falls_back_to_maintain() {
        // Validates: Requirement 1.2
        let service = make_service();
        let mode = service.resolve_effective_mode(false, None);
        assert_eq!(mode, AutoIndentMode::Maintain);
    }

    #[test]
    fn resolve_effective_mode_smart_with_patterns() {
        // Validates: Requirement 1.2
        let service = make_service();
        let mode = service.resolve_effective_mode(true, None);
        assert_eq!(mode, AutoIndentMode::Smart);
    }
}
