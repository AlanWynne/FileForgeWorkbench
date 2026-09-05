//! FFTest script parser.
//!
//! Parses `.fftest` plain-text scripts into a sequence of [`Command`] values.
//!
//! Validates: Requirement 3.1, 3.2, 3.3, 3.4, 3.5, 3.6 (automated-dialog-testing)

use thiserror::Error;

// === ParseError =============================================================

/// Errors produced by the FFTest parser.
///
/// # Errors
///
/// Every variant carries the 1-based line number where the error occurred.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    /// A command keyword was not recognised.
    #[error("line {line}: unknown command '{keyword}'")]
    UnknownCommand { line: usize, keyword: String },

    /// A command was missing one or more required arguments.
    #[error("line {line}: command '{command}' requires {required} argument(s), got {got}")]
    MissingArgument {
        line: usize,
        command: String,
        required: usize,
        got: usize,
    },

    /// A quoted string was not properly terminated.
    #[error("line {line}: unterminated quoted string")]
    UnterminatedString { line: usize },
}

// === Command ================================================================

/// A single parsed FFTest script command.
///
/// Validates: Requirement 3.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// `OPEN FILE "<path>"`
    OpenFile { path: String },
    /// `WAIT WINDOW "<title>"`
    WaitWindow { title: String },
    /// `CLICK MENU "<dot-path>"`
    ClickMenu { path: String },
    /// `CLICK BUTTON "<automation-id>"`
    ClickButton { id: String },
    /// `SELECT MENUITEM "<label>"`
    SelectMenuItem { label: String },
    /// `TYPE TEXT "<value>"`
    TypeText { value: String },
    /// `PRESS KEY <keyname>`
    PressKey { key: String },
    /// `ASSERT WINDOW EXISTS "<title>"`
    AssertWindowExists { title: String },
    /// `ASSERT TEXT EXISTS "<text>"`
    AssertTextExists { text: String },
    /// `ASSERT STATUSBAR CONTAINS "<text>"`
    AssertStatusbarContains { text: String },
    /// `ASSERT FILE OPEN`
    AssertFileOpen,
    /// `ASSERT CONTROL VALUE "<id>" "<expected>"`
    AssertControlValue { id: String, expected: String },
    /// `CHECKPOINT "<name>"`
    Checkpoint { name: String },
    /// `CLOSE WINDOW`
    CloseWindow,
    /// `LOAD PLUGIN "<name>"`
    LoadPlugin { name: String },
    /// `VARIABLE <name> "<value>"`
    Variable { name: String, value: String },
}

// === ParsedScript ===========================================================

/// The result of parsing a `.fftest` file: an ordered list of commands with
/// their source line numbers for diagnostic reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedScript {
    /// Commands in file order, each paired with its 1-based source line number.
    pub commands: Vec<(usize, Command)>,
}

// === parse ==================================================================

/// Parse a `.fftest` script from source text.
///
/// Returns a [`ParsedScript`] on success, or a [`ParseError`] describing the
/// first error encountered.
///
/// # Errors
///
/// Returns [`ParseError::UnknownCommand`] for unrecognised keywords,
/// [`ParseError::MissingArgument`] for commands with too few arguments, and
/// [`ParseError::UnterminatedString`] for unclosed quoted strings.
///
/// Validates: Requirement 3.1, 3.2, 3.3, 3.4
pub fn parse(source: &str) -> Result<ParsedScript, ParseError> {
    let mut commands = Vec::new();

    for (zero_idx, raw_line) in source.lines().enumerate() {
        let line_no = zero_idx + 1;
        let trimmed = raw_line.trim();

        // Req 3.4 -- lines beginning with '#' are comments; blank lines ignored
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let tokens = tokenise(trimmed, line_no)?;
        if tokens.is_empty() {
            continue;
        }

        let cmd = parse_command(&tokens, line_no)?;
        commands.push((line_no, cmd));
    }

    Ok(ParsedScript { commands })
}

// === tokenise ===============================================================

/// Split a line into tokens. Quoted strings become single tokens (without
/// surrounding quotes). Unquoted tokens are whitespace-delimited.
///
/// Validates: Requirement 3.3 (case-insensitive keywords handled by caller)
fn tokenise(line: &str, line_no: usize) -> Result<Vec<String>, ParseError> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if ch == '"' {
            chars.next(); // consume opening quote
            let mut buf = String::new();
            let mut closed = false;
            for c in chars.by_ref() {
                if c == '"' {
                    closed = true;
                    break;
                }
                buf.push(c);
            }
            if !closed {
                return Err(ParseError::UnterminatedString { line: line_no });
            }
            tokens.push(buf);
        } else {
            let mut buf = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    break;
                }
                buf.push(c);
                chars.next();
            }
            tokens.push(buf);
        }
    }

    Ok(tokens)
}

// === parse_command ==========================================================

/// Dispatch on the first token (uppercased) to build a [`Command`].
///
/// Validates: Requirement 3.2, 3.3
fn parse_command(tokens: &[String], line_no: usize) -> Result<Command, ParseError> {
    // Req 3.3 -- keywords are case-insensitive
    let kw = tokens[0].to_uppercase();

    match kw.as_str() {
        "OPEN" => {
            let sub = sub(tokens, 1);
            match sub.as_str() {
                "FILE" => {
                    let path = require_arg(tokens, 2, "OPEN FILE", line_no)?;
                    Ok(Command::OpenFile { path })
                }
                _ => Err(ParseError::UnknownCommand {
                    line: line_no,
                    keyword: format!("OPEN {sub}"),
                }),
            }
        }
        "WAIT" => {
            let sub = sub(tokens, 1);
            match sub.as_str() {
                "WINDOW" => {
                    let title = require_arg(tokens, 2, "WAIT WINDOW", line_no)?;
                    Ok(Command::WaitWindow { title })
                }
                _ => Err(ParseError::UnknownCommand {
                    line: line_no,
                    keyword: format!("WAIT {sub}"),
                }),
            }
        }
        "CLICK" => {
            let sub = sub(tokens, 1);
            match sub.as_str() {
                "MENU" => {
                    let path = require_arg(tokens, 2, "CLICK MENU", line_no)?;
                    Ok(Command::ClickMenu { path })
                }
                "BUTTON" => {
                    let id = require_arg(tokens, 2, "CLICK BUTTON", line_no)?;
                    Ok(Command::ClickButton { id })
                }
                _ => Err(ParseError::UnknownCommand {
                    line: line_no,
                    keyword: format!("CLICK {sub}"),
                }),
            }
        }
        "SELECT" => {
            let sub = sub(tokens, 1);
            match sub.as_str() {
                "MENUITEM" => {
                    let label = require_arg(tokens, 2, "SELECT MENUITEM", line_no)?;
                    Ok(Command::SelectMenuItem { label })
                }
                _ => Err(ParseError::UnknownCommand {
                    line: line_no,
                    keyword: format!("SELECT {sub}"),
                }),
            }
        }
        "TYPE" => {
            let sub = sub(tokens, 1);
            match sub.as_str() {
                "TEXT" => {
                    let value = require_arg(tokens, 2, "TYPE TEXT", line_no)?;
                    Ok(Command::TypeText { value })
                }
                _ => Err(ParseError::UnknownCommand {
                    line: line_no,
                    keyword: format!("TYPE {sub}"),
                }),
            }
        }
        "PRESS" => {
            let sub = sub(tokens, 1);
            match sub.as_str() {
                "KEY" => {
                    let key = require_arg(tokens, 2, "PRESS KEY", line_no)?;
                    Ok(Command::PressKey { key })
                }
                _ => Err(ParseError::UnknownCommand {
                    line: line_no,
                    keyword: format!("PRESS {sub}"),
                }),
            }
        }
        "ASSERT" => parse_assert(tokens, line_no),
        "CHECKPOINT" => {
            let name = require_arg(tokens, 1, "CHECKPOINT", line_no)?;
            Ok(Command::Checkpoint { name })
        }
        "CLOSE" => {
            let sub = sub(tokens, 1);
            match sub.as_str() {
                "WINDOW" => Ok(Command::CloseWindow),
                _ => Err(ParseError::UnknownCommand {
                    line: line_no,
                    keyword: format!("CLOSE {sub}"),
                }),
            }
        }
        "LOAD" => {
            let sub = sub(tokens, 1);
            match sub.as_str() {
                "PLUGIN" => {
                    let name = require_arg(tokens, 2, "LOAD PLUGIN", line_no)?;
                    Ok(Command::LoadPlugin { name })
                }
                _ => Err(ParseError::UnknownCommand {
                    line: line_no,
                    keyword: format!("LOAD {sub}"),
                }),
            }
        }
        "VARIABLE" => {
            let name = require_arg(tokens, 1, "VARIABLE", line_no)?;
            let value = require_arg(tokens, 2, "VARIABLE", line_no)?;
            Ok(Command::Variable { name, value })
        }
        _ => Err(ParseError::UnknownCommand {
            line: line_no,
            keyword: kw,
        }),
    }
}

/// Parse an ASSERT sub-command.
fn parse_assert(tokens: &[String], line_no: usize) -> Result<Command, ParseError> {
    let sub1 = sub(tokens, 1);
    let sub2 = sub(tokens, 2);

    match (sub1.as_str(), sub2.as_str()) {
        ("WINDOW", "EXISTS") => {
            let title = require_arg(tokens, 3, "ASSERT WINDOW EXISTS", line_no)?;
            Ok(Command::AssertWindowExists { title })
        }
        ("TEXT", "EXISTS") => {
            let text = require_arg(tokens, 3, "ASSERT TEXT EXISTS", line_no)?;
            Ok(Command::AssertTextExists { text })
        }
        ("STATUSBAR", "CONTAINS") => {
            let text = require_arg(tokens, 3, "ASSERT STATUSBAR CONTAINS", line_no)?;
            Ok(Command::AssertStatusbarContains { text })
        }
        ("FILE", "OPEN") => Ok(Command::AssertFileOpen),
        ("CONTROL", "VALUE") => {
            let id = require_arg(tokens, 3, "ASSERT CONTROL VALUE", line_no)?;
            let expected = require_arg(tokens, 4, "ASSERT CONTROL VALUE", line_no)?;
            Ok(Command::AssertControlValue { id, expected })
        }
        _ => Err(ParseError::UnknownCommand {
            line: line_no,
            keyword: format!("ASSERT {} {}", sub1, sub2),
        }),
    }
}

// === helpers ================================================================

/// Return the token at `index` uppercased, or empty string if out of bounds.
fn sub(tokens: &[String], index: usize) -> String {
    tokens
        .get(index)
        .map(|s| s.to_uppercase())
        .unwrap_or_default()
}

/// Return the token at `index` as-is (preserving case for argument values),
/// or a [`ParseError::MissingArgument`] if out of bounds.
fn require_arg(
    tokens: &[String],
    index: usize,
    command: &str,
    line_no: usize,
) -> Result<String, ParseError> {
    tokens
        .get(index)
        .cloned()
        .ok_or_else(|| ParseError::MissingArgument {
            line: line_no,
            command: command.to_string(),
            required: index,
            got: tokens.len().saturating_sub(1),
        })
}

// === Variable substitution ==================================================

/// Apply `${NAME}` substitution to a string using the provided variable map.
///
/// Validates: Requirement 3.6
pub fn substitute_vars(s: &str, vars: &std::collections::HashMap<String, String>) -> String {
    let mut result = s.to_string();
    for (name, value) in vars {
        let placeholder = format!("${{{name}}}");
        result = result.replace(&placeholder, value);
    }
    result
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Validates: Requirement 3.2 -- OPEN FILE command parses correctly
    #[test]
    fn parse_open_file_command() {
        let script = parse(r#"OPEN FILE "C:\test.txt""#).expect("parse ok");
        assert_eq!(script.commands.len(), 1);
        assert_eq!(
            script.commands[0].1,
            Command::OpenFile {
                path: r"C:\test.txt".to_string()
            }
        );
    }

    // Validates: Requirement 3.3 -- keywords are case-insensitive
    #[test]
    fn keywords_are_case_insensitive() {
        let script = parse(r#"open file "test.txt""#).expect("parse ok");
        assert_eq!(
            script.commands[0].1,
            Command::OpenFile {
                path: "test.txt".to_string()
            }
        );
    }

    // Validates: Requirement 3.4 -- comment lines are ignored
    #[test]
    fn comment_lines_are_ignored() {
        let src = "# This is a comment\nOPEN FILE \"test.txt\"";
        let script = parse(src).expect("parse ok");
        assert_eq!(script.commands.len(), 1);
    }

    // Validates: Requirement 3.4 -- blank lines are ignored
    #[test]
    fn blank_lines_are_ignored() {
        let src = "\n\nOPEN FILE \"test.txt\"\n\n";
        let script = parse(src).expect("parse ok");
        assert_eq!(script.commands.len(), 1);
    }

    // Validates: Requirement 3.5 -- unknown command produces error with line number
    #[test]
    fn unknown_command_returns_error_with_line_number() {
        let src = "# comment\nFLORP SOMETHING";
        let err = parse(src).expect_err("should fail");
        assert_eq!(
            err,
            ParseError::UnknownCommand {
                line: 2,
                keyword: "FLORP".to_string()
            }
        );
    }

    // Validates: Requirement 3.5 -- missing argument produces error
    #[test]
    fn missing_argument_returns_error() {
        let err = parse("OPEN FILE").expect_err("should fail");
        assert!(matches!(err, ParseError::MissingArgument { .. }));
    }

    // Validates: Requirement 3.2 -- all command variants parse
    #[test]
    fn all_command_variants_parse() {
        let src = r#"
WAIT WINDOW "My Window"
CLICK MENU "file.open"
CLICK BUTTON "button.save"
SELECT MENUITEM "Save"
TYPE TEXT "hello"
PRESS KEY ENTER
ASSERT WINDOW EXISTS "My Window"
ASSERT TEXT EXISTS "hello"
ASSERT STATUSBAR CONTAINS "Ready"
ASSERT FILE OPEN
ASSERT CONTROL VALUE "textbox.cmd" "FIND"
CHECKPOINT "after_open"
CLOSE WINDOW
LOAD PLUGIN "my-plugin"
VARIABLE MYVAR "value"
"#;
        let script = parse(src).expect("parse ok");
        assert_eq!(script.commands.len(), 15);
    }

    // Validates: Requirement 3.6 -- variable substitution replaces ${NAME}
    #[test]
    fn variable_substitution_replaces_placeholder() {
        let mut vars = HashMap::new();
        vars.insert("TESTFILE".to_string(), "C:\\test.txt".to_string());
        let result = substitute_vars("${TESTFILE}", &vars);
        assert_eq!(result, "C:\\test.txt");
    }

    // Validates: Requirement 3.6 -- unknown variable is left as-is
    #[test]
    fn unknown_variable_left_unchanged() {
        let vars = HashMap::new();
        let result = substitute_vars("${UNKNOWN}", &vars);
        assert_eq!(result, "${UNKNOWN}");
    }

    // Validates: Requirement 3.2 -- PRESS KEY with unquoted key name
    #[test]
    fn press_key_unquoted_key_name() {
        let script = parse("PRESS KEY ENTER").expect("parse ok");
        assert_eq!(
            script.commands[0].1,
            Command::PressKey {
                key: "ENTER".to_string()
            }
        );
    }

    // Validates: Requirement 3.4 -- line number is recorded correctly
    #[test]
    fn line_numbers_are_recorded() {
        let src = "# comment\n\nOPEN FILE \"test.txt\"";
        let script = parse(src).expect("parse ok");
        assert_eq!(script.commands[0].0, 3);
    }

    // Validates: Requirement 3.3 -- unterminated string produces error
    #[test]
    fn unterminated_string_returns_error() {
        let err = parse("OPEN FILE \"unclosed").expect_err("should fail");
        assert!(matches!(err, ParseError::UnterminatedString { .. }));
    }
}
