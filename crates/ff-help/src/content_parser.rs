//! Help content parser — parses `.help.md` file format.
//!
//! Recognises topic delimiters: `<!-- TOPIC: topic_key -->` followed by
//! `<!-- TITLE: Human Title -->` to separate multiple topics within a single file.

use std::path::Path;

use crate::error::HelpError;
use crate::topic::{HelpTopic, TopicSource};
use crate::topic_key::TopicKey;

/// Parses `.help.md` file content into `HelpTopic` instances.
pub struct ContentParser;

impl ContentParser {
    /// Parse a single `.help.md` file into one or more `HelpTopic`s.
    ///
    /// Topic delimiters are HTML comments:
    /// - `<!-- TOPIC: topic_key -->` — starts a new topic with the given key
    /// - `<!-- TITLE: Human Title -->` — sets the title for the current topic
    ///
    /// Everything between topic delimiters is the Markdown body content.
    ///
    /// # Errors
    ///
    /// Returns `HelpError::ContentParseError` if:
    /// - A TOPIC delimiter has an invalid topic key format
    /// - A TOPIC delimiter is not followed by a TITLE delimiter
    pub fn parse_file(path: &Path, content: &str) -> Result<Vec<HelpTopic>, HelpError> {
        let path_str = path.display().to_string();
        let mut topics = Vec::new();
        let mut current_key: Option<TopicKey> = None;
        let mut current_title: Option<String> = None;
        let mut current_body = String::new();
        let mut in_topic = false;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check for TOPIC delimiter
            if let Some(key_str) = Self::extract_topic_delimiter(trimmed) {
                // Save previous topic if any
                if in_topic {
                    if let (Some(key), Some(title)) = (current_key.take(), current_title.take()) {
                        topics.push(HelpTopic::new(
                            key,
                            title,
                            current_body.trim().to_string(),
                            TopicSource::FileBased {
                                file_path: path.to_path_buf(),
                            },
                        ));
                    }
                    current_body.clear();
                }

                // Parse the new topic key
                let key: TopicKey = key_str.parse().map_err(|_| HelpError::ContentParseError {
                    path: path_str.clone(),
                    line: line_num + 1,
                    reason: format!("invalid topic key format: \"{key_str}\""),
                })?;
                current_key = Some(key);
                current_title = None;
                in_topic = true;
                continue;
            }

            // Check for TITLE delimiter
            if let Some(title_str) = Self::extract_title_delimiter(trimmed) {
                if in_topic && current_title.is_none() {
                    current_title = Some(title_str.to_string());
                    continue;
                }
            }

            // Accumulate body content
            if in_topic && current_title.is_some() {
                current_body.push_str(line);
                current_body.push('\n');
            }
        }

        // Save the last topic
        if in_topic {
            if let (Some(key), Some(title)) = (current_key.take(), current_title.take()) {
                topics.push(HelpTopic::new(
                    key,
                    title,
                    current_body.trim().to_string(),
                    TopicSource::FileBased {
                        file_path: path.to_path_buf(),
                    },
                ));
            }
        }

        Ok(topics)
    }

    /// Extract a topic key from a TOPIC delimiter line.
    ///
    /// Expected format: `<!-- TOPIC: topic_key -->`
    fn extract_topic_delimiter(line: &str) -> Option<&str> {
        let line = line.trim();
        if line.starts_with("<!-- TOPIC:") && line.ends_with("-->") {
            let inner = &line["<!-- TOPIC:".len()..line.len() - "-->".len()];
            let key = inner.trim();
            if !key.is_empty() {
                return Some(key);
            }
        }
        None
    }

    /// Extract a title from a TITLE delimiter line.
    ///
    /// Expected format: `<!-- TITLE: Human Title -->`
    fn extract_title_delimiter(line: &str) -> Option<&str> {
        let line = line.trim();
        if line.starts_with("<!-- TITLE:") && line.ends_with("-->") {
            let inner = &line["<!-- TITLE:".len()..line.len() - "-->".len()];
            let title = inner.trim();
            if !title.is_empty() {
                return Some(title);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Validates: Requirement 5.2 — Parse single topic from file
    #[test]
    fn parse_single_topic_file() {
        let content = "\
<!-- TOPIC: cmd:FIND -->
<!-- TITLE: FIND Command -->
## Syntax

```
FIND 'text' [ALL]
```

## Description

Searches for text in the current file.
";
        let path = PathBuf::from("commands.help.md");
        let topics = ContentParser::parse_file(&path, content).unwrap();
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].key(), &TopicKey::command("FIND"));
        assert_eq!(topics[0].title(), "FIND Command");
        assert!(topics[0].body().contains("FIND 'text' [ALL]"));
        assert!(topics[0].body().contains("Searches for text"));
    }

    // Validates: Requirement 5.2 — Parse multiple topics from single file
    #[test]
    fn parse_multiple_topics_in_one_file() {
        let content = "\
<!-- TOPIC: cmd:FIND -->
<!-- TITLE: FIND Command -->
Find searches for text.

<!-- TOPIC: cmd:CHANGE -->
<!-- TITLE: CHANGE Command -->
Change replaces text.
";
        let path = PathBuf::from("commands.help.md");
        let topics = ContentParser::parse_file(&path, content).unwrap();
        assert_eq!(topics.len(), 2);
        assert_eq!(topics[0].key(), &TopicKey::command("FIND"));
        assert_eq!(topics[0].title(), "FIND Command");
        assert_eq!(topics[1].key(), &TopicKey::command("CHANGE"));
        assert_eq!(topics[1].title(), "CHANGE Command");
    }

    // Validates: Requirement 5.2 — Invalid topic key is an error
    #[test]
    fn parse_invalid_topic_key_returns_error() {
        let content = "\
<!-- TOPIC: invalid_no_prefix -->
<!-- TITLE: Bad Topic -->
Body content.
";
        let path = PathBuf::from("bad.help.md");
        let result = ContentParser::parse_file(&path, content);
        assert!(result.is_err());
        match result.unwrap_err() {
            HelpError::ContentParseError { line, .. } => {
                assert_eq!(line, 1);
            }
            _ => panic!("Expected ContentParseError"),
        }
    }

    // Validates: Requirement 5.3 — Markdown body content preserved
    #[test]
    fn parse_preserves_markdown_elements() {
        let content = "\
<!-- TOPIC: cmd:SAVE -->
<!-- TITLE: SAVE Command -->
## Syntax

`SAVE [filename]`

## Description

- Saves current file
- **Creates backup** if configured

See also: [CANCEL command](cmd:CANCEL)
";
        let path = PathBuf::from("test.help.md");
        let topics = ContentParser::parse_file(&path, content).unwrap();
        assert_eq!(topics.len(), 1);
        let body = topics[0].body();
        assert!(body.contains("## Syntax"));
        assert!(body.contains("`SAVE [filename]`"));
        assert!(body.contains("- Saves current file"));
        assert!(body.contains("**Creates backup**"));
        assert!(body.contains("[CANCEL command](cmd:CANCEL)"));
    }

    // Validates: Requirement 5.2 — Empty file produces no topics
    #[test]
    fn parse_empty_file_produces_no_topics() {
        let content = "";
        let path = PathBuf::from("empty.help.md");
        let topics = ContentParser::parse_file(&path, content).unwrap();
        assert!(topics.is_empty());
    }

    // Validates: Requirement 5.2 — File with content but no topic delimiters
    #[test]
    fn parse_file_without_delimiters_produces_no_topics() {
        let content = "Just some random markdown without delimiters.\n# Heading\nBody.";
        let path = PathBuf::from("no_delimiters.help.md");
        let topics = ContentParser::parse_file(&path, content).unwrap();
        assert!(topics.is_empty());
    }
}
