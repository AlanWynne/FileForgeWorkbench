//! HELP command — context-sensitive online documentation for commands,
//! line commands, and the macro API.
//!
//! The HELP command is registered with Command_ID `"help.show"` and is
//! valid in all editor modes without modifying document state.

/// A single help topic entry.
#[derive(Debug, Clone)]
pub struct HelpTopic {
    /// The topic identifier (command name, "LINECOMMANDS", "MACRO", "API").
    pub key: String,
    /// Category for grouping.
    pub category: String,
    /// One-line description.
    pub summary: String,
    /// Full help text (syntax, modifiers, examples).
    pub full_text: String,
}

/// Context-sensitive help system for commands, line commands, and macro API.
pub struct HelpEngine {
    /// Registry of help topics.
    topics: Vec<HelpTopic>,
}

impl HelpEngine {
    /// Create a new HelpEngine and register built-in topics.
    pub fn new() -> Self {
        let mut engine = Self { topics: Vec::new() };
        engine.register_builtin_topics();
        engine
    }

    /// Register a help topic.
    pub fn register_topic(&mut self, topic: HelpTopic) {
        self.topics.push(topic);
    }

    /// Show all commands grouped by category.
    pub fn show_all(&self) -> String {
        let mut categories: std::collections::BTreeMap<&str, Vec<&HelpTopic>> =
            std::collections::BTreeMap::new();

        for topic in &self.topics {
            categories.entry(&topic.category).or_default().push(topic);
        }

        let mut output = String::from("Available Commands:\n\n");
        for (category, topics) in &categories {
            output.push_str(&format!("  {}:\n", category));
            for topic in topics {
                output.push_str(&format!("    {:<16} {}\n", topic.key, topic.summary));
            }
            output.push('\n');
        }
        output
    }

    /// Show help for a specific command.
    pub fn show_command(&self, name: &str) -> Option<String> {
        let upper = name.to_uppercase();
        self.topics
            .iter()
            .find(|t| t.key.to_uppercase() == upper)
            .map(|t| t.full_text.clone())
    }

    /// Show all line commands.
    pub fn show_line_commands(&self) -> String {
        let mut output = String::from("Line Commands:\n\n");
        output.push_str("  Single-line commands:\n");
        output.push_str("    C    - Copy line\n");
        output.push_str("    M    - Move line\n");
        output.push_str("    D    - Delete line\n");
        output.push_str("    R    - Repeat (duplicate) line\n");
        output.push_str("    X    - Exclude line from display\n");
        output.push_str("    I    - Insert lines after\n");
        output.push_str("    A    - After (destination for copy/move)\n");
        output.push_str("    B    - Before (destination for copy/move)\n");
        output.push_str("    O    - Overlay\n");
        output.push_str("    W    - Show/reveal excluded line\n");
        output.push_str("    S    - Select line\n");
        output.push_str("    T    - Tag line\n");
        output.push_str("    >    - Shift right\n");
        output.push_str("    <    - Shift left\n");
        output.push_str("    (    - Indent in\n");
        output.push_str("    )    - Indent out\n");
        output.push_str("    ]    - Set bounds\n");
        output.push('\n');
        output.push_str("  Block commands (paired):\n");
        output.push_str("    CC   - Copy block\n");
        output.push_str("    MM   - Move block\n");
        output.push_str("    DD   - Delete block\n");
        output.push_str("    RR   - Repeat block\n");
        output.push_str("    XX   - Exclude block\n");
        output.push_str("    TT   - Tag block\n");
        output.push('\n');
        output.push_str("  Counts: append digits to single-line commands (e.g., D5, C3)\n");
        output.push_str("  Maximum count: 99999\n");
        output
    }

    /// Show macro API help.
    pub fn show_macro_api(&self) -> String {
        let mut output = String::from("Macro API (Lua):\n\n");
        output.push_str("  Available functions:\n");
        output.push_str("    ff.execute(cmd)       - Execute a primary command\n");
        output.push_str("    ff.line_count()       - Get document line count\n");
        output.push_str("    ff.get_line(n)        - Get text of line n\n");
        output.push_str("    ff.set_line(n, text)  - Set text of line n\n");
        output.push_str("    ff.insert_line(n, t)  - Insert line at position n\n");
        output.push_str("    ff.delete_line(n)     - Delete line n\n");
        output.push_str("    ff.cursor_line()      - Get current cursor line\n");
        output.push_str("    ff.cursor_col()       - Get current cursor column\n");
        output.push_str("    ff.set_cursor(l, c)   - Set cursor position\n");
        output.push_str("    ff.message(text)      - Display a status message\n");
        output.push('\n');
        output.push_str("  For detailed API documentation, see the macro scripting guide.\n");
        output
    }

    /// Find close matches for an unknown topic.
    pub fn suggest_matches(&self, query: &str) -> Vec<String> {
        let upper = query.to_uppercase();
        self.topics
            .iter()
            .filter(|t| {
                let key_upper = t.key.to_uppercase();
                key_upper.contains(&upper)
                    || upper.contains(&key_upper)
                    || Self::levenshtein(&key_upper, &upper) <= 2
            })
            .map(|t| t.key.clone())
            .collect()
    }

    /// Register built-in help topics.
    fn register_builtin_topics(&mut self) {
        self.register_topic(HelpTopic {
            key: "HELP".to_string(),
            category: "General".to_string(),
            summary: "Display help information".to_string(),
            full_text: "HELP [topic]\n\n\
                Display help for a command or topic.\n\n\
                Syntax:\n\
                  HELP                  - Show all available commands\n\
                  HELP <command>        - Show help for a specific command\n\
                  HELP LINECOMMANDS     - Show all line commands\n\
                  HELP MACRO            - Show macro API functions\n\
                  HELP API              - Same as HELP MACRO\n"
                .to_string(),
        });

        self.register_topic(HelpTopic {
            key: "LINECOMMANDS".to_string(),
            category: "Reference".to_string(),
            summary: "Line command reference".to_string(),
            full_text: self.show_line_commands(),
        });

        self.register_topic(HelpTopic {
            key: "MACRO".to_string(),
            category: "Scripting".to_string(),
            summary: "Lua macro API reference".to_string(),
            full_text: self.show_macro_api(),
        });

        self.register_topic(HelpTopic {
            key: "API".to_string(),
            category: "Scripting".to_string(),
            summary: "Lua macro API reference (alias for MACRO)".to_string(),
            full_text: self.show_macro_api(),
        });
    }

    /// Simple Levenshtein distance for fuzzy matching.
    fn levenshtein(a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let m = a_chars.len();
        let n = b_chars.len();

        let mut dp = vec![vec![0usize; n + 1]; m + 1];

        for (i, row) in dp.iter_mut().enumerate() {
            row[0] = i;
        }
        for (j, cell) in dp[0].iter_mut().enumerate() {
            *cell = j;
        }

        for i in 1..=m {
            for j in 1..=n {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                dp[i][j] = (dp[i - 1][j] + 1)
                    .min(dp[i][j - 1] + 1)
                    .min(dp[i - 1][j - 1] + cost);
            }
        }

        dp[m][n]
    }
}

impl Default for HelpEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 7.1
    #[test]
    fn show_all_lists_available_commands() {
        let engine = HelpEngine::new();
        let output = engine.show_all();
        assert!(output.contains("Available Commands:"));
        assert!(output.contains("HELP"));
    }

    // Validates: Requirement 7.2
    #[test]
    fn show_command_returns_full_text_for_known_command() {
        let engine = HelpEngine::new();
        let result = engine.show_command("HELP");
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(text.contains("HELP"));
        assert!(text.contains("Syntax:"));
    }

    // Validates: Requirement 7.2
    #[test]
    fn show_command_case_insensitive() {
        let engine = HelpEngine::new();
        assert!(engine.show_command("help").is_some());
        assert!(engine.show_command("Help").is_some());
    }

    // Validates: Requirement 7.3
    #[test]
    fn show_line_commands_lists_all_kinds() {
        let engine = HelpEngine::new();
        let output = engine.show_line_commands();
        assert!(output.contains("Line Commands:"));
        assert!(output.contains("C    - Copy line"));
        assert!(output.contains("CC   - Copy block"));
        assert!(output.contains(">    - Shift right"));
    }

    // Validates: Requirement 7.4
    #[test]
    fn show_macro_api_lists_functions() {
        let engine = HelpEngine::new();
        let output = engine.show_macro_api();
        assert!(output.contains("Macro API"));
        assert!(output.contains("ff.execute"));
        assert!(output.contains("ff.get_line"));
    }

    // Validates: Requirement 7.5
    #[test]
    fn suggest_matches_finds_close_names() {
        let engine = HelpEngine::new();
        let suggestions = engine.suggest_matches("HALP");
        assert!(suggestions.contains(&"HELP".to_string()));
    }

    // Validates: Requirement 7.5
    #[test]
    fn show_command_returns_none_for_unknown_topic() {
        let engine = HelpEngine::new();
        assert!(engine.show_command("NOSUCHCMD").is_none());
    }

    // Validates: Requirement 7.6
    #[test]
    fn help_engine_does_not_require_mutable_document() {
        // HELP is read-only — all methods take &self, no document mutation
        let engine = HelpEngine::new();
        let _ = engine.show_all();
        let _ = engine.show_command("HELP");
        let _ = engine.show_line_commands();
        let _ = engine.show_macro_api();
        // If this compiles, it proves no &mut is needed
    }

    // Validates: Requirement 7.8
    #[test]
    fn help_registered_with_builtin_topics() {
        let engine = HelpEngine::new();
        assert!(engine.show_command("HELP").is_some());
        assert!(engine.show_command("LINECOMMANDS").is_some());
        assert!(engine.show_command("MACRO").is_some());
        assert!(engine.show_command("API").is_some());
    }

    #[test]
    fn register_custom_topic() {
        let mut engine = HelpEngine::new();
        engine.register_topic(HelpTopic {
            key: "FIND".to_string(),
            category: "Editing".to_string(),
            summary: "Find text in document".to_string(),
            full_text: "FIND 'text' [modifiers]\n\nFind text in the document.".to_string(),
        });
        assert!(engine.show_command("FIND").is_some());
        let output = engine.show_all();
        assert!(output.contains("FIND"));
    }
}
