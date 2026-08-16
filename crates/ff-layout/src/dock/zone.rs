//! Dock zone enum and zone content management.

/// Designated areas within the primary window where panels can be attached.
///
/// Standard zones are left, right, bottom, and center. The `Floating` variant
/// indicates a panel that has been detached into its own OS-level window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum DockZone {
    /// Left side panel area (e.g., file tree, explorer).
    Left,
    /// Right side panel area (e.g., properties, outline).
    Right,
    /// Bottom panel area (e.g., output, terminal, problems).
    Bottom,
    /// Center editor area containing tab groups.
    Center,
    /// Detached into a floating OS-level window.
    Floating,
}

impl DockZone {
    /// Returns true if this is a valid docking target (not Floating).
    pub fn is_dockable(&self) -> bool {
        !matches!(self, DockZone::Floating)
    }

    /// Returns all standard dock zones (excluding Floating).
    pub fn standard_zones() -> &'static [DockZone] {
        &[
            DockZone::Left,
            DockZone::Right,
            DockZone::Bottom,
            DockZone::Center,
        ]
    }
}

impl std::fmt::Display for DockZone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DockZone::Left => write!(f, "Left"),
            DockZone::Right => write!(f, "Right"),
            DockZone::Bottom => write!(f, "Bottom"),
            DockZone::Center => write!(f, "Center"),
            DockZone::Floating => write!(f, "Floating"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dock_zone_is_dockable_excludes_floating() {
        assert!(DockZone::Left.is_dockable());
        assert!(DockZone::Right.is_dockable());
        assert!(DockZone::Bottom.is_dockable());
        assert!(DockZone::Center.is_dockable());
        assert!(!DockZone::Floating.is_dockable());
    }

    #[test]
    fn standard_zones_returns_four_zones() {
        let zones = DockZone::standard_zones();
        assert_eq!(zones.len(), 4);
        assert!(!zones.contains(&DockZone::Floating));
    }

    #[test]
    fn dock_zone_display_format() {
        assert_eq!(DockZone::Left.to_string(), "Left");
        assert_eq!(DockZone::Right.to_string(), "Right");
        assert_eq!(DockZone::Bottom.to_string(), "Bottom");
        assert_eq!(DockZone::Center.to_string(), "Center");
        assert_eq!(DockZone::Floating.to_string(), "Floating");
    }

    #[test]
    fn dock_zone_serialization_round_trip() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Wrapper {
            zone: DockZone,
        }

        let zones = [
            DockZone::Left,
            DockZone::Right,
            DockZone::Bottom,
            DockZone::Center,
            DockZone::Floating,
        ];
        for zone in &zones {
            let wrapper = Wrapper { zone: *zone };
            let serialized = toml::to_string(&wrapper).unwrap();
            let deserialized: Wrapper = toml::from_str(&serialized).unwrap();
            assert_eq!(*zone, deserialized.zone);
        }
    }
}
