//! Platform-abstracted pseudo-terminal handle.
//!
//! Defines the `PtyHandle` trait and provides platform-specific implementations
//! for Windows (ConPTY) and Unix (PTY).

use crate::error::ShellError;

/// Platform-abstracted pseudo-terminal handle.
///
/// Implemented separately for Windows (ConPTY) and Unix (PTY).
/// The trait provides a unified interface for reading/writing to the terminal
/// and managing the child process lifecycle.
pub trait PtyHandle: Send + Sync {
    /// Write bytes to the PTY (user keyboard input → child process stdin).
    fn write(&mut self, data: &[u8]) -> Result<usize, ShellError>;

    /// Read available bytes from the PTY (child process stdout → emulator).
    /// Returns Ok(0) if no data is currently available (non-blocking).
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ShellError>;

    /// Resize the PTY to new dimensions.
    fn resize(&mut self, cols: u16, rows: u16) -> Result<(), ShellError>;

    /// Close the PTY, terminating the child process.
    fn close(&mut self) -> Result<(), ShellError>;

    /// Check if the child process has exited.
    fn is_alive(&self) -> bool;

    /// Get the exit code if the process has exited.
    fn exit_code(&self) -> Option<i32>;
}

/// A mock PTY handle for testing purposes.
#[derive(Debug)]
pub struct MockPtyHandle {
    /// Buffer of data "written" to the PTY by tests.
    pub written: Vec<u8>,
    /// Buffer of data available for "reading" from the PTY.
    pub read_buffer: Vec<u8>,
    /// Whether the mock process is still "alive".
    pub alive: bool,
    /// Mock exit code.
    pub mock_exit_code: Option<i32>,
    /// Current dimensions.
    pub cols: u16,
    /// Current dimensions.
    pub rows: u16,
}

impl MockPtyHandle {
    /// Creates a new mock PTY handle.
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            written: Vec::new(),
            read_buffer: Vec::new(),
            alive: true,
            mock_exit_code: None,
            cols,
            rows,
        }
    }

    /// Enqueues data for the next read.
    pub fn enqueue_output(&mut self, data: &[u8]) {
        self.read_buffer.extend_from_slice(data);
    }
}

impl PtyHandle for MockPtyHandle {
    fn write(&mut self, data: &[u8]) -> Result<usize, ShellError> {
        self.written.extend_from_slice(data);
        Ok(data.len())
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ShellError> {
        let n = buf.len().min(self.read_buffer.len());
        if n == 0 {
            return Ok(0);
        }
        buf[..n].copy_from_slice(&self.read_buffer[..n]);
        self.read_buffer.drain(..n);
        Ok(n)
    }

    fn resize(&mut self, cols: u16, rows: u16) -> Result<(), ShellError> {
        self.cols = cols;
        self.rows = rows;
        Ok(())
    }

    fn close(&mut self) -> Result<(), ShellError> {
        self.alive = false;
        self.mock_exit_code = Some(0);
        Ok(())
    }

    fn is_alive(&self) -> bool {
        self.alive
    }

    fn exit_code(&self) -> Option<i32> {
        self.mock_exit_code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 7.5/7.6
    #[test]
    fn mock_pty_write_stores_data() {
        let mut pty = MockPtyHandle::new(80, 24);
        let n = pty.write(b"hello").unwrap();
        assert_eq!(n, 5);
        assert_eq!(pty.written, b"hello");
    }

    // Validates: Requirement 7.5/7.6
    #[test]
    fn mock_pty_read_returns_enqueued_data() {
        let mut pty = MockPtyHandle::new(80, 24);
        pty.enqueue_output(b"output");
        let mut buf = [0u8; 10];
        let n = pty.read(&mut buf).unwrap();
        assert_eq!(n, 6);
        assert_eq!(&buf[..n], b"output");
    }

    // Validates: Requirement 7.5/7.6
    #[test]
    fn mock_pty_read_returns_zero_when_empty() {
        let mut pty = MockPtyHandle::new(80, 24);
        let mut buf = [0u8; 10];
        let n = pty.read(&mut buf).unwrap();
        assert_eq!(n, 0);
    }

    // Validates: Requirement 7.5/7.6
    #[test]
    fn mock_pty_close_marks_not_alive() {
        let mut pty = MockPtyHandle::new(80, 24);
        assert!(pty.is_alive());
        pty.close().unwrap();
        assert!(!pty.is_alive());
        assert_eq!(pty.exit_code(), Some(0));
    }

    // Validates: Requirement 7.5/7.6
    #[test]
    fn mock_pty_resize() {
        let mut pty = MockPtyHandle::new(80, 24);
        pty.resize(120, 40).unwrap();
        assert_eq!(pty.cols, 120);
        assert_eq!(pty.rows, 40);
    }
}
