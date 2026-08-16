//! Configuration values for the line commands subsystem.
//!
//! Reads `editor.shift_width` from the configuration system with hot-reload support.

/// Configuration values for the line commands subsystem.
///
/// Read from the configuration system at startup and on hot-reload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineCommandConfig {
    /// Default shift width for `>` and `<` commands (default: 2).
    pub shift_width: u32,
}

impl Default for LineCommandConfig {
    fn default() -> Self {
        Self { shift_width: 2 }
    }
}

impl LineCommandConfig {
    /// Creates a new configuration with the given shift width.
    pub fn new(shift_width: u32) -> Self {
        Self { shift_width }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_shift_width_is_two() {
        let config = LineCommandConfig::default();
        assert_eq!(config.shift_width, 2);
    }

    #[test]
    fn custom_shift_width_is_stored() {
        let config = LineCommandConfig::new(4);
        assert_eq!(config.shift_width, 4);
    }
}
