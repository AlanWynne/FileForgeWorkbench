//! Tokenizer for IDCAMS control statements.
//!
//! Transforms input text into a flat token stream, handling continuation lines,
//! comments, case-insensitive keywords, and nested parentheses.

use super::token::{CmpOp, LogOp, Token, Verb};

/// The IDCAMS lexer. Tokenizes input text into a sequence of tokens.
pub struct Lexer;

impl Lexer {
    /// Tokenizes the input text into a sequence of tokens.
    ///
    /// Handles:
    /// - Case-insensitive verb and keyword recognition
    /// - Continuation lines (hyphen at end of line)
    /// - Block comments (`/* ... */`)
    /// - Single-line comments (`//`)
    /// - Semicolon command separators
    /// - Parenthesised parameters
    /// - Dataset names (1-44 chars with qualifier rules)
    pub fn tokenize(input: &str) -> Vec<Token> {
        let preprocessed = Self::preprocess(input);
        let mut tokens = Vec::new();
        let chars: Vec<char> = preprocessed.chars().collect();
        let mut pos = 0;

        while pos < chars.len() {
            // Skip whitespace
            if chars[pos].is_whitespace() {
                pos += 1;
                continue;
            }

            // Block comments: /* ... */
            if pos + 1 < chars.len() && chars[pos] == '/' && chars[pos + 1] == '*' {
                let start = pos;
                pos += 2;
                while pos + 1 < chars.len() && !(chars[pos] == '*' && chars[pos + 1] == '/') {
                    pos += 1;
                }
                if pos + 1 < chars.len() {
                    pos += 2; // skip */
                }
                let comment: String = chars[start..pos].iter().collect();
                tokens.push(Token::Comment(comment));
                continue;
            }

            // Single-line comments: //
            if pos + 1 < chars.len() && chars[pos] == '/' && chars[pos + 1] == '/' {
                let start = pos;
                while pos < chars.len() && chars[pos] != '\n' {
                    pos += 1;
                }
                let comment: String = chars[start..pos].iter().collect();
                tokens.push(Token::Comment(comment));
                continue;
            }

            // Semicolon
            if chars[pos] == ';' {
                tokens.push(Token::Semicolon);
                pos += 1;
                continue;
            }

            // Parentheses
            if chars[pos] == '(' {
                tokens.push(Token::OpenParen);
                pos += 1;
                continue;
            }
            if chars[pos] == ')' {
                tokens.push(Token::CloseParen);
                pos += 1;
                continue;
            }

            // Wildcard
            if chars[pos] == '*' {
                tokens.push(Token::Wildcard);
                pos += 1;
                continue;
            }

            // Hyphen — only a continuation token at end of line
            // In other contexts, it's part of a word
            if chars[pos] == '-' {
                // Check if it's followed only by whitespace then newline
                let mut peek = pos + 1;
                while peek < chars.len() && chars[peek] == ' ' {
                    peek += 1;
                }
                if peek >= chars.len() || chars[peek] == '\n' {
                    tokens.push(Token::Hyphen);
                    pos = peek;
                    if pos < chars.len() {
                        pos += 1; // skip newline
                    }
                    continue;
                }
                // Otherwise treat as part of a word (e.g., negative number or part of name)
                // Fall through to word/number handling
            }

            // Numbers (including negative)
            if chars[pos].is_ascii_digit()
                || (chars[pos] == '-' && pos + 1 < chars.len() && chars[pos + 1].is_ascii_digit())
            {
                let start = pos;
                if chars[pos] == '-' {
                    pos += 1;
                }
                while pos < chars.len() && chars[pos].is_ascii_digit() {
                    pos += 1;
                }
                // If followed by a letter or dot, it's part of a word (e.g., dataset qualifier)
                if pos < chars.len()
                    && (chars[pos].is_alphabetic()
                        || chars[pos] == '.'
                        || chars[pos] == '#'
                        || chars[pos] == '@'
                        || chars[pos] == '$')
                {
                    // Backtrack and handle as a word
                    pos = start;
                } else {
                    let num_str: String = chars[start..pos].iter().collect();
                    if let Ok(n) = num_str.parse::<i64>() {
                        tokens.push(Token::Number(n));
                        continue;
                    }
                    pos = start; // fallback to word
                }
            }

            // Words: keywords, verbs, dataset names, comparison ops, logical ops
            if chars[pos].is_alphabetic()
                || chars[pos] == '@'
                || chars[pos] == '#'
                || chars[pos] == '$'
                || chars[pos] == '\''
                || (chars[pos].is_ascii_digit()
                    && pos > 0
                    && (chars[pos - 1].is_alphabetic() || chars[pos - 1] == '.'))
            {
                let start = pos;
                // Handle quoted strings
                if chars[pos] == '\'' {
                    pos += 1;
                    while pos < chars.len() && chars[pos] != '\'' {
                        pos += 1;
                    }
                    if pos < chars.len() {
                        pos += 1;
                    }
                    let word: String = chars[start + 1..pos - 1].iter().collect();
                    tokens.push(Token::StringLit(word));
                    continue;
                }

                while pos < chars.len()
                    && (chars[pos].is_alphanumeric()
                        || chars[pos] == '.'
                        || chars[pos] == '-'
                        || chars[pos] == '@'
                        || chars[pos] == '#'
                        || chars[pos] == '$'
                        || chars[pos] == '_')
                {
                    // Don't include hyphen if it's a continuation
                    if chars[pos] == '-' {
                        let mut peek = pos + 1;
                        while peek < chars.len() && chars[peek] == ' ' {
                            peek += 1;
                        }
                        if peek >= chars.len() || chars[peek] == '\n' {
                            break;
                        }
                    }
                    pos += 1;
                }

                let word: String = chars[start..pos].iter().collect();
                let upper = word.to_uppercase();

                // Check if it's a verb
                if let Some(verb) = Verb::from_str_ci(&upper) {
                    tokens.push(Token::Verb(verb));
                    continue;
                }

                // Check if it's a comparison operator
                if let Some(op) = CmpOp::from_str_ci(&upper) {
                    tokens.push(Token::CompareOp(op));
                    continue;
                }

                // Check if it's a logical operator
                if let Some(op) = LogOp::from_str_ci(&upper) {
                    tokens.push(Token::LogicalOp(op));
                    continue;
                }

                // Dataset names contain dots; everything else is a keyword
                if word.contains('.') {
                    tokens.push(Token::StringLit(word.to_uppercase()));
                } else {
                    tokens.push(Token::Keyword(upper));
                }
                continue;
            }

            // Handle comparison operator symbols
            if chars[pos] == '>' || chars[pos] == '<' || chars[pos] == '=' {
                let start = pos;
                pos += 1;
                if pos < chars.len() && chars[pos] == '=' {
                    pos += 1;
                }
                let op_str: String = chars[start..pos].iter().collect();
                if let Some(op) = CmpOp::from_str_ci(&op_str) {
                    tokens.push(Token::CompareOp(op));
                } else {
                    tokens.push(Token::Keyword(op_str));
                }
                continue;
            }

            // Skip anything else
            pos += 1;
        }

        tokens.push(Token::Eof);
        // Filter out Comment and Hyphen tokens (they're structural markers, not semantic)
        tokens
            .into_iter()
            .filter(|t| !matches!(t, Token::Comment(_) | Token::Hyphen))
            .collect()
    }

    /// Preprocesses input: handles continuation lines and strips sequence numbers.
    fn preprocess(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let lines: Vec<&str> = input.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let mut line = lines[i].to_string();

            // Strip sequence numbers from columns 73-80 (0-indexed: 72-79)
            // Only if line is exactly 80 chars and cols 73-80 are numeric
            if line.len() >= 80 {
                let seq_area = &line[72..80];
                if seq_area.chars().all(|c| c.is_ascii_digit() || c == ' ') {
                    line = line[..72].to_string();
                }
            }

            let trimmed = line.trim_end();
            if let Some(without_hyphen) = trimmed.strip_suffix('-') {
                // Join with next line
                result.push_str(without_hyphen);
                // Don't add newline — continuation
            } else {
                result.push_str(&line);
                result.push('\n');
            }

            i += 1;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple_define_verb() {
        let tokens = Lexer::tokenize("DEFINE");
        assert_eq!(tokens[0], Token::Verb(Verb::Define));
    }

    #[test]
    fn tokenize_case_insensitive_verbs() {
        let tokens = Lexer::tokenize("define");
        assert_eq!(tokens[0], Token::Verb(Verb::Define));

        let tokens = Lexer::tokenize("Define");
        assert_eq!(tokens[0], Token::Verb(Verb::Define));

        let tokens = Lexer::tokenize("LISTCAT");
        assert_eq!(tokens[0], Token::Verb(Verb::Listcat));
    }

    #[test]
    fn tokenize_parentheses() {
        let tokens = Lexer::tokenize("NAME(MY.CLUSTER)");
        assert_eq!(tokens[0], Token::Keyword("NAME".to_string()));
        assert_eq!(tokens[1], Token::OpenParen);
        assert_eq!(tokens[2], Token::StringLit("MY.CLUSTER".to_string()));
        assert_eq!(tokens[3], Token::CloseParen);
    }

    #[test]
    fn tokenize_numbers() {
        let tokens = Lexer::tokenize("KEYS(8 0)");
        assert_eq!(tokens[0], Token::Keyword("KEYS".to_string()));
        assert_eq!(tokens[1], Token::OpenParen);
        assert_eq!(tokens[2], Token::Number(8));
        assert_eq!(tokens[3], Token::Number(0));
        assert_eq!(tokens[4], Token::CloseParen);
    }

    #[test]
    fn tokenize_semicolons_separate_commands() {
        let tokens = Lexer::tokenize("DEFINE; DELETE");
        assert!(tokens.contains(&Token::Verb(Verb::Define)));
        assert!(tokens.contains(&Token::Semicolon));
        assert!(tokens.contains(&Token::Verb(Verb::Delete)));
    }

    #[test]
    fn tokenize_block_comments_excluded() {
        let tokens = Lexer::tokenize("DEFINE /* a comment */ CLUSTER");
        // Comments are filtered out
        assert!(!tokens.iter().any(|t| matches!(t, Token::Comment(_))));
        assert_eq!(tokens[0], Token::Verb(Verb::Define));
        assert_eq!(tokens[1], Token::Keyword("CLUSTER".to_string()));
    }

    #[test]
    fn tokenize_line_comments_excluded() {
        let tokens = Lexer::tokenize("// this is a comment\nDEFINE");
        assert!(!tokens.iter().any(|t| matches!(t, Token::Comment(_))));
        assert_eq!(tokens[0], Token::Verb(Verb::Define));
    }

    #[test]
    fn tokenize_continuation_lines() {
        let input = "DEFINE -\n  CLUSTER";
        let tokens = Lexer::tokenize(input);
        assert_eq!(tokens[0], Token::Verb(Verb::Define));
        assert_eq!(tokens[1], Token::Keyword("CLUSTER".to_string()));
    }

    #[test]
    fn tokenize_dataset_name() {
        let tokens = Lexer::tokenize("MY.DATA.SET1");
        assert_eq!(tokens[0], Token::StringLit("MY.DATA.SET1".to_string()));
    }

    #[test]
    fn tokenize_wildcard() {
        let tokens = Lexer::tokenize("MY.DATA.*");
        assert_eq!(tokens[0], Token::StringLit("MY.DATA.".to_string()));
        assert_eq!(tokens[1], Token::Wildcard);
    }

    #[test]
    fn tokenize_comparison_operators() {
        let tokens = Lexer::tokenize("EQ NE GT LT GE LE");
        assert_eq!(tokens[0], Token::CompareOp(CmpOp::Eq));
        assert_eq!(tokens[1], Token::CompareOp(CmpOp::Ne));
        assert_eq!(tokens[2], Token::CompareOp(CmpOp::Gt));
        assert_eq!(tokens[3], Token::CompareOp(CmpOp::Lt));
        assert_eq!(tokens[4], Token::CompareOp(CmpOp::Ge));
        assert_eq!(tokens[5], Token::CompareOp(CmpOp::Le));
    }

    #[test]
    fn tokenize_logical_operators() {
        let tokens = Lexer::tokenize("AND OR");
        assert_eq!(tokens[0], Token::LogicalOp(LogOp::And));
        assert_eq!(tokens[1], Token::LogicalOp(LogOp::Or));
    }

    #[test]
    fn tokenize_all_verbs() {
        let verbs = "DEFINE DELETE ALTER LISTCAT PRINT REPRO VERIFY EXPORT IMPORT BLDINDEX SET IF";
        let tokens = Lexer::tokenize(verbs);
        assert_eq!(tokens[0], Token::Verb(Verb::Define));
        assert_eq!(tokens[1], Token::Verb(Verb::Delete));
        assert_eq!(tokens[2], Token::Verb(Verb::Alter));
        assert_eq!(tokens[3], Token::Verb(Verb::Listcat));
        assert_eq!(tokens[4], Token::Verb(Verb::Print));
        assert_eq!(tokens[5], Token::Verb(Verb::Repro));
        assert_eq!(tokens[6], Token::Verb(Verb::Verify));
        assert_eq!(tokens[7], Token::Verb(Verb::Export));
        assert_eq!(tokens[8], Token::Verb(Verb::Import));
        assert_eq!(tokens[9], Token::Verb(Verb::Bldindex));
        assert_eq!(tokens[10], Token::Verb(Verb::Set));
        assert_eq!(tokens[11], Token::Verb(Verb::If));
    }
}
