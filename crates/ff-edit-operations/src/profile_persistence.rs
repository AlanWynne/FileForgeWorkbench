//! Edit profile persistence -- TOML serialisation/deserialisation.
//!
//! Provides `serialize_profile` and `deserialize_profile` helpers used by
//! the session system to persist per-file edit profile settings.

use crate::profile::EditProfile;

/// Serialise an `EditProfile` to a TOML string.
///
/// # Validates
/// Requirement 16.9
pub fn serialize_profile(profile: &EditProfile) -> Result<String, ProfilePersistError> {
    toml::to_string(profile).map_err(|e| ProfilePersistError::Serialise(e.to_string()))
}

/// Deserialise an `EditProfile` from a TOML string.
///
/// # Validates
/// Requirement 16.9
pub fn deserialize_profile(toml_str: &str) -> Result<EditProfile, ProfilePersistError> {
    toml::from_str(toml_str).map_err(|e| ProfilePersistError::Deserialise(e.to_string()))
}

/// Errors produced by profile persistence operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProfilePersistError {
    #[error("[profile-persist] serialise failed: {0}")]
    Serialise(String),
    #[error("[profile-persist] deserialise failed: {0}")]
    Deserialise(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{CapsMode, HiliteMode, NullsMode, ProfileLock, StatsMode};

    #[test]
    fn edit_profile_round_trips_through_toml_defaults() {
        // Validates: Requirement 16.9
        let profile = EditProfile::new();
        let toml_str = serialize_profile(&profile).expect("serialise");
        let restored = deserialize_profile(&toml_str).expect("deserialise");
        assert_eq!(profile, restored);
    }

    #[test]
    fn edit_profile_round_trips_with_all_flags_on() {
        // Validates: Requirement 16.9
        let mut profile = EditProfile::new();
        profile.caps = CapsMode::On;
        profile.nulls = NullsMode::On;
        profile.stats = StatsMode::On;
        profile.lock = ProfileLock::On;
        profile.hilite = HiliteMode::Logic;

        let toml_str = serialize_profile(&profile).expect("serialise");
        let restored = deserialize_profile(&toml_str).expect("deserialise");
        assert_eq!(profile, restored);
    }

    #[test]
    fn deserialise_empty_string_uses_defaults() {
        // Validates: Requirement 16.9 -- missing keys fall back to defaults
        let restored = deserialize_profile("").expect("deserialise empty");
        assert_eq!(restored, EditProfile::new());
    }

    #[test]
    fn deserialise_invalid_toml_returns_error() {
        let result = deserialize_profile("not = [valid toml");
        assert!(result.is_err());
    }
}
