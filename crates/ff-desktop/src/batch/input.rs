use std::io::{BufRead, BufReader, Read};

/// Maximum supported line length before truncation warning.
pub const MAX_LINE_LEN: usize = 32767;

/// Reads Batch_Commands from a source, skipping blanks and comments,
/// handling `-` continuation, and stripping UTF-8 BOM.
pub struct BatchInputSource {
    lines: Vec<String>,
    pos: usize,
    pub warnings: Vec<String>,
}

impl BatchInputSource {
    /// Build from any readable source (file or stdin).
    pub fn from_reader<R: Read>(reader: R) -> Self {
        let buf = BufReader::new(reader);
        let mut raw: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        for (idx, line_result) in buf.lines().enumerate() {
            let mut line = match line_result {
                Ok(l) => l,
                Err(_) => continue,
            };
            // Strip UTF-8 BOM on first line
            if idx == 0 {
                line = line.trim_start_matches('\u{FEFF}').to_string();
            }
            // Truncate overlong lines
            if line.len() > MAX_LINE_LEN {
                warnings.push(format!(
                    "Line {} truncated to {} characters",
                    idx + 1,
                    MAX_LINE_LEN
                ));
                line.truncate(MAX_LINE_LEN);
            }
            raw.push(line);
        }

        let lines = Self::process(raw, &mut warnings);
        Self {
            lines,
            pos: 0,
            warnings,
        }
    }

    /// Build from a string slice (used in tests).
    pub fn from_str(s: &str) -> Self {
        Self::from_reader(s.as_bytes())
    }

    /// Apply comment-skip, blank-skip, and `-` continuation to raw lines.
    fn process(raw: Vec<String>, _warnings: &mut Vec<String>) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut pending: Option<String> = None;

        for line in raw {
            let trimmed = line.trim_end();

            // Skip blank lines
            if trimmed.trim().is_empty() {
                if pending.is_some() {
                    // blank line ends a continuation -- flush
                    if let Some(p) = pending.take() {
                        out.push(p);
                    }
                }
                continue;
            }

            // Skip comment lines (* or /*)
            let first_nonws = trimmed.trim_start();
            if first_nonws.starts_with('*') || first_nonws.starts_with("/*") {
                continue;
            }

            // Continuation: line ends with bare `-`
            if trimmed.ends_with('-')
                && trimmed.trim_end_matches('-').trim_end() != trimmed.trim_end_matches('-')
            {
                // trailing `-` with possible spaces before it
                let body = trimmed.trim_end_matches('-').trim_end();
                let acc = match pending.take() {
                    Some(p) => format!("{} {}", p, body),
                    None => body.to_string(),
                };
                pending = Some(acc);
                continue;
            }
            // Simpler check: last non-whitespace char is `-`
            let last_nonws = trimmed.chars().rev().find(|c| !c.is_whitespace());
            if last_nonws == Some('-') && trimmed.trim() != "-" {
                let body = trimmed.trim_end();
                let body = &body[..body.rfind('-').unwrap()];
                let body = body.trim_end();
                let acc = match pending.take() {
                    Some(p) => format!("{} {}", p, body),
                    None => body.to_string(),
                };
                pending = Some(acc);
                continue;
            }

            // Normal line
            let full = match pending.take() {
                Some(p) => format!("{} {}", p, trimmed),
                None => trimmed.to_string(),
            };
            out.push(full);
        }

        // Flush any trailing continuation
        if let Some(p) = pending {
            out.push(p);
        }

        out
    }

    /// Return the next command, or None when exhausted.
    pub fn next_command(&mut self) -> Option<&str> {
        if self.pos < self.lines.len() {
            let cmd = &self.lines[self.pos];
            self.pos += 1;
            Some(cmd.as_str())
        } else {
            None
        }
    }

    /// Total number of commands available.
    pub fn command_count(&self) -> usize {
        self.lines.len()
    }
}

// === Tests ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.5
    #[test]
    fn blank_lines_are_skipped() {
        let mut src = BatchInputSource::from_str("EDIT foo.txt\n\n\nSAVE\n");
        assert_eq!(src.next_command(), Some("EDIT foo.txt"));
        assert_eq!(src.next_command(), Some("SAVE"));
        assert_eq!(src.next_command(), None);
    }

    // Validates: Requirement 2.3
    #[test]
    fn asterisk_comment_lines_are_skipped() {
        let mut src = BatchInputSource::from_str("* this is a comment\nFIND ERROR\n");
        assert_eq!(src.next_command(), Some("FIND ERROR"));
        assert_eq!(src.next_command(), None);
    }

    // Validates: Requirement 2.4
    #[test]
    fn slash_asterisk_comment_lines_are_skipped() {
        let mut src = BatchInputSource::from_str("/* JCL-style comment\nLOCATE 10\n");
        assert_eq!(src.next_command(), Some("LOCATE 10"));
        assert_eq!(src.next_command(), None);
    }

    // Validates: Requirement 2.2
    #[test]
    fn commands_returned_in_order() {
        let mut src = BatchInputSource::from_str("EDIT a.txt\nFIND ERROR\nSAVE\n");
        assert_eq!(src.next_command(), Some("EDIT a.txt"));
        assert_eq!(src.next_command(), Some("FIND ERROR"));
        assert_eq!(src.next_command(), Some("SAVE"));
        assert_eq!(src.next_command(), None);
    }

    // Validates: Requirement 2.6
    #[test]
    fn continuation_joins_lines() {
        let input = "CHANGE /OLD -\n/NEW\n";
        let mut src = BatchInputSource::from_str(input);
        let cmd = src.next_command().unwrap();
        assert!(
            cmd.contains("CHANGE") && cmd.contains("OLD") && cmd.contains("NEW"),
            "continuation not joined: {cmd}"
        );
        assert_eq!(src.next_command(), None);
    }

    // Validates: Requirement 2.1
    #[test]
    fn utf8_bom_stripped() {
        // BOM is U+FEFF = 0xEF 0xBB 0xBF in UTF-8
        let bom = "\u{FEFF}EDIT file.txt\n";
        let mut src = BatchInputSource::from_str(bom);
        let cmd = src.next_command().unwrap();
        assert!(!cmd.starts_with('\u{FEFF}'), "BOM not stripped: {cmd:?}");
        assert_eq!(cmd, "EDIT file.txt");
    }

    // Validates: Requirement 2.7
    #[test]
    fn overlong_line_produces_warning() {
        let long_line = "X".repeat(MAX_LINE_LEN + 10) + "\n";
        let src = BatchInputSource::from_str(&long_line);
        assert!(!src.warnings.is_empty(), "expected truncation warning");
        assert_eq!(src.lines[0].len(), MAX_LINE_LEN);
    }

    // Validates: Requirement 2.2
    #[test]
    fn command_count_matches_non_blank_non_comment_lines() {
        let input = "* comment\nEDIT a.txt\n\nSAVE\n/* another comment\nEXIT\n";
        let src = BatchInputSource::from_str(input);
        assert_eq!(src.command_count(), 3);
    }

    // Validates: Requirement 2.5
    #[test]
    fn whitespace_only_lines_skipped() {
        let mut src = BatchInputSource::from_str("   \n\t\nEDIT x\n");
        assert_eq!(src.next_command(), Some("EDIT x"));
        assert_eq!(src.next_command(), None);
    }

    // Validates: Requirement 2.3 -- asterisk must be first non-whitespace
    #[test]
    fn asterisk_not_first_nonws_is_not_a_comment() {
        let mut src = BatchInputSource::from_str("FIND * ERROR\n");
        assert_eq!(src.next_command(), Some("FIND * ERROR"));
    }

    // Validates: Requirement 9.1, 9.2, 9.3 -- .ffcmd format identical to batch format
    #[test]
    fn ffcmd_format_parsed_identically_to_batch_format() {
        // A typical .ffcmd file: * comments, /* comments, continuation, blanks
        let ffcmd = "* FFCMD script\n\
                     /* allocate and edit */\n\
                     EDIT myfile.txt\n\
                     FIND /ERROR -\n\
                     /FIRST\n\
                     \n\
                     SAVE\n";
        let mut src = BatchInputSource::from_str(ffcmd);
        let cmd1 = src.next_command().unwrap();
        assert_eq!(cmd1, "EDIT myfile.txt");
        let cmd2 = src.next_command().unwrap();
        assert!(
            cmd2.contains("FIND") && cmd2.contains("ERROR") && cmd2.contains("FIRST"),
            "continuation not joined: {cmd2}"
        );
        let cmd3 = src.next_command().unwrap();
        assert_eq!(cmd3, "SAVE");
        assert_eq!(src.next_command(), None);
    }

    // Validates: Requirement 9.4 -- BatchInputSource does not invoke Lua engine
    #[test]
    fn batch_input_source_has_no_lua_dependency() {
        // BatchInputSource is a pure text reader with no reference to ff-lua
        // or any scripting engine. This test confirms the type compiles and
        // operates without any Lua context.
        let mut src = BatchInputSource::from_str("LISTCAT\n");
        assert_eq!(src.next_command(), Some("LISTCAT"));
        // If this test compiles and passes, the Lua engine is not involved.
    }
}
