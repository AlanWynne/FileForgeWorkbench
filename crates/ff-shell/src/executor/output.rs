//! Output capture data structure.
//!
//! Represents captured stdout/stderr from command execution, supporting
//! both incremental streaming and final collection.

/// Captured output from a command execution.
///
/// Supports incremental streaming (lines appended as they arrive)
/// and final collection (all output available after process exit).
#[derive(Debug, Clone)]
pub struct OutputCapture {
    /// Accumulated stdout lines.
    pub stdout_lines: Vec<String>,
    /// Accumulated stderr lines.
    pub stderr_lines: Vec<String>,
    /// Whether the capture is still receiving data.
    pub is_streaming: bool,
    /// Total bytes received so far.
    pub bytes_received: usize,
}

impl OutputCapture {
    /// Creates a new empty capture in streaming mode.
    pub fn new_streaming() -> Self {
        Self {
            stdout_lines: Vec::new(),
            stderr_lines: Vec::new(),
            is_streaming: true,
            bytes_received: 0,
        }
    }

    /// Appends a stdout line.
    pub fn append_stdout(&mut self, line: String) {
        self.bytes_received += line.len();
        self.stdout_lines.push(line);
    }

    /// Appends a stderr line.
    pub fn append_stderr(&mut self, line: String) {
        self.bytes_received += line.len();
        self.stderr_lines.push(line);
    }

    /// Marks the capture as complete (no more data expected).
    pub fn finalize(&mut self) {
        self.is_streaming = false;
    }

    /// Returns true if stdout produced any output.
    pub fn has_stdout(&self) -> bool {
        !self.stdout_lines.is_empty()
    }

    /// Returns true if stderr produced any output.
    pub fn has_stderr(&self) -> bool {
        !self.stderr_lines.is_empty()
    }

    /// Returns the total number of output lines (stdout + stderr).
    pub fn total_lines(&self) -> usize {
        self.stdout_lines.len() + self.stderr_lines.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 4.1
    #[test]
    fn new_streaming_capture_is_empty() {
        let capture = OutputCapture::new_streaming();
        assert!(capture.is_streaming);
        assert!(!capture.has_stdout());
        assert!(!capture.has_stderr());
        assert_eq!(capture.total_lines(), 0);
        assert_eq!(capture.bytes_received, 0);
    }

    // Validates: Requirement 4.1
    #[test]
    fn append_stdout_adds_lines() {
        let mut capture = OutputCapture::new_streaming();
        capture.append_stdout("hello".to_string());
        capture.append_stdout("world".to_string());
        assert!(capture.has_stdout());
        assert_eq!(capture.stdout_lines.len(), 2);
        assert_eq!(capture.bytes_received, 10);
    }

    // Validates: Requirement 4.1
    #[test]
    fn finalize_stops_streaming() {
        let mut capture = OutputCapture::new_streaming();
        assert!(capture.is_streaming);
        capture.finalize();
        assert!(!capture.is_streaming);
    }
}
