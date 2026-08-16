//! Terminal session manager.
//!
//! Manages the lifecycle of interactive terminal sessions: creation,
//! input routing, output polling, and destruction.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::ShellError;
use crate::terminal::emulator::TerminalEmulator;
use crate::terminal::pty::{MockPtyHandle, PtyHandle};

/// Unique identifier for a terminal session (tab).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(u64);

impl SessionId {
    /// Creates a new unique session ID.
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Returns the raw numeric value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Session({})", self.0)
    }
}

/// Represents an active interactive terminal session.
///
/// Manages the PTY connection and emulator state.
pub struct TerminalSession {
    /// Unique session identifier.
    pub id: SessionId,
    /// The shell profile used for this session.
    pub profile_name: Option<String>,
    /// Terminal emulator state.
    emulator: TerminalEmulator,
    /// Platform PTY handle (read/write to child process).
    pty: Box<dyn PtyHandle>,
    /// Working directory at session start.
    pub working_directory: PathBuf,
    /// Whether this session is currently focused.
    pub is_focused: bool,
    /// Display title (shell name or custom).
    pub title: String,
}

impl TerminalSession {
    /// Returns a reference to the terminal emulator.
    pub fn emulator(&self) -> &TerminalEmulator {
        &self.emulator
    }

    /// Returns a mutable reference to the terminal emulator.
    pub fn emulator_mut(&mut self) -> &mut TerminalEmulator {
        &mut self.emulator
    }

    /// Returns whether the PTY process is still alive.
    pub fn is_alive(&self) -> bool {
        self.pty.is_alive()
    }

    /// Returns the exit code if the session has ended.
    pub fn exit_code(&self) -> Option<i32> {
        self.pty.exit_code()
    }
}

impl std::fmt::Debug for TerminalSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalSession")
            .field("id", &self.id)
            .field("profile_name", &self.profile_name)
            .field("working_directory", &self.working_directory)
            .field("is_focused", &self.is_focused)
            .field("title", &self.title)
            .finish()
    }
}

/// Manages the lifecycle of interactive terminal sessions.
///
/// Creates, destroys, and routes I/O for multiple concurrent sessions
/// (displayed as tabs in the Terminal Panel).
#[derive(Debug)]
pub struct TerminalManager {
    sessions: HashMap<SessionId, TerminalSession>,
}

impl TerminalManager {
    /// Creates a new empty terminal manager.
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Opens a new terminal session with a mock PTY (for testing/development).
    ///
    /// In production, this would use platform-specific PTY spawning.
    pub fn open_session_mock(
        &mut self,
        working_dir: PathBuf,
        profile_name: Option<String>,
        dimensions: (u16, u16),
    ) -> SessionId {
        let id = SessionId::new();
        let pty = Box::new(MockPtyHandle::new(dimensions.0, dimensions.1));
        let emulator = TerminalEmulator::new(dimensions.0, dimensions.1, 1000);

        let title = profile_name.as_deref().unwrap_or("Terminal").to_string();

        let session = TerminalSession {
            id,
            profile_name,
            emulator,
            pty,
            working_directory: working_dir,
            is_focused: false,
            title,
        };

        self.sessions.insert(id, session);
        id
    }

    /// Closes a terminal session, terminating its process.
    pub fn close_session(&mut self, id: SessionId) -> Result<(), ShellError> {
        if let Some(mut session) = self.sessions.remove(&id) {
            session.pty.close()?;
            Ok(())
        } else {
            Err(ShellError::SessionNotFound { id: id.as_u64() })
        }
    }

    /// Gets an immutable reference to a session.
    pub fn session(&self, id: SessionId) -> Option<&TerminalSession> {
        self.sessions.get(&id)
    }

    /// Gets a mutable reference to a session.
    pub fn session_mut(&mut self, id: SessionId) -> Option<&mut TerminalSession> {
        self.sessions.get_mut(&id)
    }

    /// Lists all active session IDs.
    pub fn active_sessions(&self) -> Vec<SessionId> {
        self.sessions.keys().copied().collect()
    }

    /// Returns the number of active sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Writes keyboard input to the specified terminal session.
    pub fn write_input(&mut self, id: SessionId, data: &[u8]) -> Result<(), ShellError> {
        let session = self
            .sessions
            .get_mut(&id)
            .ok_or(ShellError::SessionNotFound { id: id.as_u64() })?;
        session.pty.write(data)?;
        Ok(())
    }

    /// Polls all sessions for new output and feeds it to their emulators.
    pub fn poll_output(&mut self) -> Result<(), ShellError> {
        let ids: Vec<SessionId> = self.sessions.keys().copied().collect();
        for id in ids {
            if let Some(session) = self.sessions.get_mut(&id) {
                let mut buf = [0u8; 4096];
                loop {
                    let n = session.pty.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    session.emulator.feed(&buf[..n]);
                }
            }
        }
        Ok(())
    }
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 7.7
    #[test]
    fn open_session_creates_new_session() {
        let mut manager = TerminalManager::new();
        let id = manager.open_session_mock(PathBuf::from("/tmp"), None, (80, 24));
        assert_eq!(manager.session_count(), 1);
        assert!(manager.session(id).is_some());
    }

    // Validates: Requirement 7.7
    #[test]
    fn multiple_sessions_can_be_opened() {
        let mut manager = TerminalManager::new();
        let _id1 = manager.open_session_mock(PathBuf::from("/tmp"), None, (80, 24));
        let _id2 =
            manager.open_session_mock(PathBuf::from("/home"), Some("bash".to_string()), (80, 24));
        assert_eq!(manager.session_count(), 2);
    }

    // Validates: Requirement 7.3
    #[test]
    fn close_session_removes_it() {
        let mut manager = TerminalManager::new();
        let id = manager.open_session_mock(PathBuf::from("/tmp"), None, (80, 24));
        manager.close_session(id).unwrap();
        assert_eq!(manager.session_count(), 0);
        assert!(manager.session(id).is_none());
    }

    // Validates: Requirement 7
    #[test]
    fn close_nonexistent_session_returns_error() {
        let mut manager = TerminalManager::new();
        let result = manager.close_session(SessionId::new());
        assert!(matches!(result, Err(ShellError::SessionNotFound { .. })));
    }

    // Validates: Requirement 7.4
    #[test]
    fn write_input_routes_to_pty() {
        let mut manager = TerminalManager::new();
        let id = manager.open_session_mock(PathBuf::from("/tmp"), None, (80, 24));
        manager.write_input(id, b"hello").unwrap();
        // We can verify through the mock that data was written
        // (the mock stores written data)
    }

    // Validates: Requirement 7
    #[test]
    fn active_sessions_lists_all_ids() {
        let mut manager = TerminalManager::new();
        let id1 = manager.open_session_mock(PathBuf::from("/tmp"), None, (80, 24));
        let id2 = manager.open_session_mock(PathBuf::from("/tmp"), None, (80, 24));
        let ids = manager.active_sessions();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    // Validates: Requirement 7
    #[test]
    fn session_ids_are_unique() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);
    }
}
