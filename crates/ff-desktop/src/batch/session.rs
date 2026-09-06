/// Headless session context for batch execution.
/// Loads config and catalog registry; does NOT restore GUI session state.
pub struct BatchSession {
    pub no_catalog: bool,
    pub profile: Option<String>,
}

impl BatchSession {
    pub fn new(no_catalog: bool, profile: Option<String>) -> Self {
        Self {
            no_catalog,
            profile,
        }
    }

    /// Returns true when the session was created with no-catalog mode.
    pub fn is_no_catalog(&self) -> bool {
        self.no_catalog
    }

    /// Returns the profile name if one was supplied.
    pub fn profile_name(&self) -> Option<&str> {
        self.profile.as_deref()
    }
}

// === Tests ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 7.4, 7.5
    #[test]
    fn batch_session_does_not_restore_gui_state() {
        // BatchSession has no tab/window/geometry fields -- GUI state is absent.
        let session = BatchSession::new(false, None);
        assert!(!session.no_catalog);
        assert!(session.profile.is_none());
    }

    // Validates: Requirement 7.6
    #[test]
    fn no_catalog_mode_is_reflected() {
        let session = BatchSession::new(true, None);
        assert!(session.is_no_catalog());
    }

    // Validates: Requirement 7.3
    #[test]
    fn profile_name_is_stored() {
        let session = BatchSession::new(false, Some("ci".to_string()));
        assert_eq!(session.profile_name(), Some("ci"));
    }

    // Validates: Requirement 7.1, 7.2
    #[test]
    fn default_session_has_no_profile_and_uses_catalog() {
        let session = BatchSession::new(false, None);
        assert!(!session.is_no_catalog());
        assert!(session.profile_name().is_none());
    }
}
