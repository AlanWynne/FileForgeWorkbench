//! Panel display state types.
//!
//! Defines the visual states a panel can have within its dock zone.

/// Display states for panels within dock zones.
///
/// Tracks whether a panel is minimized (collapsed), at normal size, or
/// maximized (filling the primary window content area).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum PanelDisplayState {
    /// Collapsed to tab/icon in dock zone header.
    Minimized,
    /// Rendered at assigned size.
    #[default]
    Normal,
    /// Expanded to fill entire primary window content area.
    Maximized,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_display_state_is_normal() {
        assert_eq!(PanelDisplayState::default(), PanelDisplayState::Normal);
    }

    #[test]
    fn display_state_serialization_round_trip() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Wrapper {
            state: PanelDisplayState,
        }

        let states = [
            PanelDisplayState::Minimized,
            PanelDisplayState::Normal,
            PanelDisplayState::Maximized,
        ];
        for state in &states {
            let wrapper = Wrapper { state: *state };
            let serialized = toml::to_string(&wrapper).unwrap();
            let deserialized: Wrapper = toml::from_str(&serialized).unwrap();
            assert_eq!(*state, deserialized.state);
        }
    }
}
