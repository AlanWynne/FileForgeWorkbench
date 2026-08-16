//! Primary command parser — tokenises command-line text into structured command representation.
//!
//! Handles bare words, quoted strings (single and double quotes with escape handling),
//! hex literals (X'hh...'), and case-insensitive command name normalization.

use crate::error::ParseError;

/// A single lexical unit from the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandToken {
    /// A bare word (unquoted whitespace-delimited string).
    Word(String),
    /// A quoted string with the quote character stripped.
    QuotedString {
        value: String,
        quote_style: QuoteStyle,
    },
    /// A hex literal: X'hh...' decoded into a byte sequence.
    HexLiteral(Vec<u8>),
}

/// Which quote character enclosed a quoted string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteStyle {
    /// Single quote (`'`).
    Single,
    /// Double quote (`"`).
    Double,
}

/// The result of parsing a command-line string.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedCommand {
    /// No command was entered (empty/whitespace-only input).
    Empty,
    /// A command name followed by zero or more argument tokens.
    Command {
        /// The normalized command name (uppercase).
        name: String,
        /// The argument tokens following the command name.
        args: Vec<CommandToken>,
    },
}

/// Tokenises command-line text into structured command representation.
pub struct PrimaryCommandParser;

impl PrimaryCommandParser {
    /// Parse command-line text into a ParsedCommand.
    ///
    /// Returns `Ok(ParsedCommand::Empty)` for empty/whitespace-only input.
    /// Returns `Err(ParseError)` for malformed input (unclosed quotes, invalid hex).
    pub fn parse(input: &str) -> Result<ParsedCommand, ParseError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(ParsedCommand::Empty);
        }

        let tokens = Self::tokenize(trimmed)?;
        if tokens.is_empty() {
            return Ok(ParsedCommand::Empty);
        }

        // First token is the command name (must be a bare word)
        let name = match &tokens[0] {
            CommandToken::Word(w) => w.to_uppercase(),
            CommandToken::QuotedString { value, .. } => value.to_uppercase(),
            CommandToken::HexLiteral(_) => {
                return Err(ParseError::InvalidHexLiteral {
                    position: 0,
                    detail: "hex literal cannot be a command name".to_string(),
                });
            }
        };

        let args = tokens.into_iter().skip(1).collect();
        Ok(ParsedCommand::Command { name, args })
    }

    /// Reconstruct command-line text from a ParsedCommand (for round-trip testing).
    pub fn reconstruct(command: &ParsedCommand) -> String {
        match command {
            ParsedCommand::Empty => String::new(),
            ParsedCommand::Command { name, args } => {
                let mut parts = vec![name.clone()];
                for token in args {
                    parts.push(token.reconstruct());
                }
                parts.join(" ")
            }
        }
    }

    /// Tokenize input into a list of CommandTokens.
    fn tokenize(input: &str) -> Result<Vec<CommandToken>, ParseError> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            // Skip whitespace
            if chars[i].is_whitespace() {
                i += 1;
                continue;
            }

            // Check for hex literal: X' or x'
            if (chars[i] == 'X' || chars[i] == 'x') && i + 1 < len && chars[i + 1] == '\'' {
                let start = i;
                i += 2; // skip X'
                let mut hex_str = String::new();
                loop {
                    if i >= len {
                        return Err(ParseError::UnclosedQuote { position: start });
                    }
                    if chars[i] == '\'' {
                        i += 1;
                        break;
                    }
                    hex_str.push(chars[i]);
                    i += 1;
                }
                // Validate hex content
                if !hex_str.len().is_multiple_of(2) {
                    return Err(ParseError::InvalidHexLiteral {
                        position: start,
                        detail: "odd number of hex digits".to_string(),
                    });
                }
                let mut bytes = Vec::with_capacity(hex_str.len() / 2);
                let hex_chars: Vec<char> = hex_str.chars().collect();
                let mut j = 0;
                while j < hex_chars.len() {
                    let hi =
                        hex_chars[j]
                            .to_digit(16)
                            .ok_or_else(|| ParseError::InvalidHexLiteral {
                                position: start,
                                detail: format!("invalid hex character '{}'", hex_chars[j]),
                            })?;
                    let lo = hex_chars[j + 1].to_digit(16).ok_or_else(|| {
                        ParseError::InvalidHexLiteral {
                            position: start,
                            detail: format!("invalid hex character '{}'", hex_chars[j + 1]),
                        }
                    })?;
                    bytes.push((hi * 16 + lo) as u8);
                    j += 2;
                }
                tokens.push(CommandToken::HexLiteral(bytes));
                continue;
            }

            // Check for quoted string
            if chars[i] == '\'' || chars[i] == '"' {
                let quote_char = chars[i];
                let quote_style = if quote_char == '\'' {
                    QuoteStyle::Single
                } else {
                    QuoteStyle::Double
                };
                let start = i;
                i += 1; // skip opening quote
                let mut value = String::new();
                loop {
                    if i >= len {
                        return Err(ParseError::UnclosedQuote { position: start });
                    }
                    if chars[i] == quote_char {
                        // Check for escaped quote (doubled)
                        if i + 1 < len && chars[i + 1] == quote_char {
                            value.push(quote_char);
                            i += 2;
                        } else {
                            i += 1; // skip closing quote
                            break;
                        }
                    } else {
                        value.push(chars[i]);
                        i += 1;
                    }
                }
                tokens.push(CommandToken::QuotedString { value, quote_style });
                continue;
            }

            // Bare word: read until whitespace
            let mut word = String::new();
            while i < len && !chars[i].is_whitespace() {
                word.push(chars[i]);
                i += 1;
            }
            tokens.push(CommandToken::Word(word));
        }

        Ok(tokens)
    }
}

impl CommandToken {
    /// Reconstruct text that, when re-parsed, yields the same token.
    pub fn reconstruct(&self) -> String {
        match self {
            CommandToken::Word(w) => w.clone(),
            CommandToken::QuotedString { value, quote_style } => {
                let quote_char = match quote_style {
                    QuoteStyle::Single => '\'',
                    QuoteStyle::Double => '"',
                };
                // Escape internal quotes by doubling
                let escaped = value.replace(quote_char, &format!("{}{}", quote_char, quote_char));
                format!("{}{}{}", quote_char, escaped, quote_char)
            }
            CommandToken::HexLiteral(bytes) => {
                let hex: String = bytes.iter().map(|b| format!("{:02X}", b)).collect();
                format!("X'{}'", hex)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 3.1
    #[test]
    fn parse_single_command_no_args() {
        let result = PrimaryCommandParser::parse("FIND").unwrap();
        assert_eq!(
            result,
            ParsedCommand::Command {
                name: "FIND".to_string(),
                args: vec![],
            }
        );
    }

    // Validates: Requirement 3.1
    #[test]
    fn parse_command_with_bare_word_args() {
        let result = PrimaryCommandParser::parse("CHANGE foo bar").unwrap();
        assert_eq!(
            result,
            ParsedCommand::Command {
                name: "CHANGE".to_string(),
                args: vec![
                    CommandToken::Word("foo".to_string()),
                    CommandToken::Word("bar".to_string()),
                ],
            }
        );
    }

    // Validates: Requirement 3.5
    #[test]
    fn parse_empty_input_returns_empty() {
        assert_eq!(
            PrimaryCommandParser::parse("").unwrap(),
            ParsedCommand::Empty
        );
    }

    // Validates: Requirement 3.5
    #[test]
    fn parse_whitespace_only_returns_empty() {
        assert_eq!(
            PrimaryCommandParser::parse("   ").unwrap(),
            ParsedCommand::Empty
        );
        assert_eq!(
            PrimaryCommandParser::parse("\t\n").unwrap(),
            ParsedCommand::Empty
        );
    }

    // Validates: Requirement 3.4
    #[test]
    fn parse_normalizes_command_name_to_uppercase() {
        let result = PrimaryCommandParser::parse("find").unwrap();
        match result {
            ParsedCommand::Command { name, .. } => assert_eq!(name, "FIND"),
            _ => panic!("expected Command"),
        }

        let result = PrimaryCommandParser::parse("Find").unwrap();
        match result {
            ParsedCommand::Command { name, .. } => assert_eq!(name, "FIND"),
            _ => panic!("expected Command"),
        }
    }

    // Validates: Requirement 3.2
    #[test]
    fn parse_single_quoted_string() {
        let result = PrimaryCommandParser::parse("FIND 'hello world'").unwrap();
        assert_eq!(
            result,
            ParsedCommand::Command {
                name: "FIND".to_string(),
                args: vec![CommandToken::QuotedString {
                    value: "hello world".to_string(),
                    quote_style: QuoteStyle::Single,
                }],
            }
        );
    }

    // Validates: Requirement 3.2
    #[test]
    fn parse_double_quoted_string() {
        let result = PrimaryCommandParser::parse("FIND \"hello world\"").unwrap();
        assert_eq!(
            result,
            ParsedCommand::Command {
                name: "FIND".to_string(),
                args: vec![CommandToken::QuotedString {
                    value: "hello world".to_string(),
                    quote_style: QuoteStyle::Double,
                }],
            }
        );
    }

    // Validates: Requirement 3.7
    #[test]
    fn parse_escaped_quotes_in_single_quoted_string() {
        let result = PrimaryCommandParser::parse("FIND 'it''s'").unwrap();
        assert_eq!(
            result,
            ParsedCommand::Command {
                name: "FIND".to_string(),
                args: vec![CommandToken::QuotedString {
                    value: "it's".to_string(),
                    quote_style: QuoteStyle::Single,
                }],
            }
        );
    }

    // Validates: Requirement 3.7
    #[test]
    fn parse_escaped_quotes_in_double_quoted_string() {
        let result = PrimaryCommandParser::parse("FIND \"say \"\"hi\"\"\"").unwrap();
        assert_eq!(
            result,
            ParsedCommand::Command {
                name: "FIND".to_string(),
                args: vec![CommandToken::QuotedString {
                    value: "say \"hi\"".to_string(),
                    quote_style: QuoteStyle::Double,
                }],
            }
        );
    }

    // Validates: Requirement 3.3
    #[test]
    fn parse_hex_literal() {
        let result = PrimaryCommandParser::parse("FIND X'48454C4C4F'").unwrap();
        assert_eq!(
            result,
            ParsedCommand::Command {
                name: "FIND".to_string(),
                args: vec![CommandToken::HexLiteral(vec![0x48, 0x45, 0x4C, 0x4C, 0x4F])],
            }
        );
    }

    // Validates: Requirement 3.3
    #[test]
    fn parse_hex_literal_lowercase_x() {
        let result = PrimaryCommandParser::parse("FIND x'ff00'").unwrap();
        assert_eq!(
            result,
            ParsedCommand::Command {
                name: "FIND".to_string(),
                args: vec![CommandToken::HexLiteral(vec![0xFF, 0x00])],
            }
        );
    }

    // Validates: Requirement 3.8
    #[test]
    fn parse_unclosed_quote_returns_error() {
        let result = PrimaryCommandParser::parse("FIND 'hello");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::UnclosedQuote { .. }
        ));
    }

    // Validates: Requirement 3.3
    #[test]
    fn parse_invalid_hex_odd_digits_returns_error() {
        let result = PrimaryCommandParser::parse("FIND X'ABC'");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidHexLiteral { .. }
        ));
    }

    // Validates: Requirement 3.3
    #[test]
    fn parse_invalid_hex_non_hex_chars_returns_error() {
        let result = PrimaryCommandParser::parse("FIND X'GHIJ'");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidHexLiteral { .. }
        ));
    }

    // Validates: Requirement 3.6
    #[test]
    fn round_trip_basic_command() {
        let input = "FIND foo bar";
        let parsed = PrimaryCommandParser::parse(input).unwrap();
        let reconstructed = PrimaryCommandParser::reconstruct(&parsed);
        let reparsed = PrimaryCommandParser::parse(&reconstructed).unwrap();
        assert_eq!(parsed, reparsed);
    }

    // Validates: Requirement 3.6
    #[test]
    fn round_trip_quoted_strings() {
        let input = "CHANGE 'hello world' 'goodbye world'";
        let parsed = PrimaryCommandParser::parse(input).unwrap();
        let reconstructed = PrimaryCommandParser::reconstruct(&parsed);
        let reparsed = PrimaryCommandParser::parse(&reconstructed).unwrap();
        assert_eq!(parsed, reparsed);
    }

    // Validates: Requirement 3.6
    #[test]
    fn round_trip_hex_literal() {
        let input = "FIND X'48454C4C4F'";
        let parsed = PrimaryCommandParser::parse(input).unwrap();
        let reconstructed = PrimaryCommandParser::reconstruct(&parsed);
        let reparsed = PrimaryCommandParser::parse(&reconstructed).unwrap();
        assert_eq!(parsed, reparsed);
    }

    // Validates: Requirement 3.1
    #[test]
    fn parse_multiple_consecutive_spaces() {
        let result = PrimaryCommandParser::parse("FIND    foo    bar").unwrap();
        assert_eq!(
            result,
            ParsedCommand::Command {
                name: "FIND".to_string(),
                args: vec![
                    CommandToken::Word("foo".to_string()),
                    CommandToken::Word("bar".to_string()),
                ],
            }
        );
    }

    // Validates: Requirement 3.2
    #[test]
    fn parse_mixed_quote_types() {
        let result = PrimaryCommandParser::parse("CHANGE 'single' \"double\"").unwrap();
        assert_eq!(
            result,
            ParsedCommand::Command {
                name: "CHANGE".to_string(),
                args: vec![
                    CommandToken::QuotedString {
                        value: "single".to_string(),
                        quote_style: QuoteStyle::Single,
                    },
                    CommandToken::QuotedString {
                        value: "double".to_string(),
                        quote_style: QuoteStyle::Double,
                    },
                ],
            }
        );
    }

    // Validates: Requirement 3.4
    #[test]
    fn command_name_with_digits_preserved() {
        let result = PrimaryCommandParser::parse("cmd123 arg").unwrap();
        match result {
            ParsedCommand::Command { name, .. } => assert_eq!(name, "CMD123"),
            _ => panic!("expected Command"),
        }
    }
}
