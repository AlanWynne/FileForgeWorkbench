//! Edge column indicator computation.

use crate::modes::EdgeMode;
use crate::types::{ColourRGBA, EdgeInfo, EdgeProperties};

/// Configuration for edge column indicator.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeConfig {
    /// The active edge mode.
    pub mode: EdgeMode,
    /// The single-edge column position (used by `Line` and `Background` modes).
    pub column: u32,
    /// The colour for the single-edge indicator.
    pub colour: ColourRGBA,
    /// Multi-edge entries (used by `MultiLine` mode).
    pub multi_edges: Vec<EdgeProperties>,
}

impl Default for EdgeConfig {
    fn default() -> Self {
        Self {
            mode: EdgeMode::None,
            column: 80,
            colour: ColourRGBA::default(),
            multi_edges: Vec::new(),
        }
    }
}

/// Compute the edge indicator based on the active mode.
///
/// Returns `None` when mode is `None`, otherwise returns the appropriate
/// `EdgeInfo` variant.
///
/// Addresses: Requirement 5 AC 5.1–5.5
pub fn compute_edge_indicator(config: &EdgeConfig) -> Option<EdgeInfo> {
    match config.mode {
        EdgeMode::None => None,
        EdgeMode::Line => Some(EdgeInfo::Line {
            column: config.column,
            colour: config.colour,
        }),
        EdgeMode::Background => Some(EdgeInfo::Background {
            column: config.column,
            colour: config.colour,
        }),
        EdgeMode::MultiLine => Some(EdgeInfo::MultiLine {
            edges: config.multi_edges.clone(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_mode_returns_none() {
        // Validates: Requirement 5.1
        let config = EdgeConfig {
            mode: EdgeMode::None,
            ..Default::default()
        };
        assert_eq!(compute_edge_indicator(&config), None);
    }

    #[test]
    fn line_mode_returns_single_column() {
        // Validates: Requirement 5.3
        let config = EdgeConfig {
            mode: EdgeMode::Line,
            column: 80,
            colour: ColourRGBA {
                r: 128,
                g: 128,
                b: 128,
                a: 255,
            },
            multi_edges: Vec::new(),
        };
        let result = compute_edge_indicator(&config);
        assert_eq!(
            result,
            Some(EdgeInfo::Line {
                column: 80,
                colour: ColourRGBA {
                    r: 128,
                    g: 128,
                    b: 128,
                    a: 255
                },
            })
        );
    }

    #[test]
    fn background_mode_returns_shading_start() {
        // Validates: Requirement 5.4
        let config = EdgeConfig {
            mode: EdgeMode::Background,
            column: 120,
            colour: ColourRGBA {
                r: 50,
                g: 50,
                b: 50,
                a: 128,
            },
            multi_edges: Vec::new(),
        };
        let result = compute_edge_indicator(&config);
        assert_eq!(
            result,
            Some(EdgeInfo::Background {
                column: 120,
                colour: ColourRGBA {
                    r: 50,
                    g: 50,
                    b: 50,
                    a: 128
                },
            })
        );
    }

    #[test]
    fn multi_line_mode_returns_all_entries() {
        // Validates: Requirement 5.5
        let edges = vec![
            EdgeProperties {
                column: 80,
                colour: ColourRGBA {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            },
            EdgeProperties {
                column: 120,
                colour: ColourRGBA {
                    r: 0,
                    g: 255,
                    b: 0,
                    a: 255,
                },
            },
        ];
        let config = EdgeConfig {
            mode: EdgeMode::MultiLine,
            column: 80,
            colour: ColourRGBA::default(),
            multi_edges: edges.clone(),
        };
        let result = compute_edge_indicator(&config);
        assert_eq!(result, Some(EdgeInfo::MultiLine { edges }));
    }
}
