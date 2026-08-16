//! Decoration events — click and hover notifications.
//!
//! Defines the event types emitted by the decoration system
//! when the user interacts with decorated text.

use crate::IndicatorNumber;

/// Events emitted by the text-decorations system.
///
/// Addresses: Requirement 11 AC 5
#[derive(Debug, Clone)]
pub enum DecorationEvent {
    /// A click occurred on a decorated position.
    Click {
        /// Character position that was clicked.
        position: u64,
        /// Indicator numbers active at the click position.
        indicators: Vec<IndicatorNumber>,
    },
    /// Hover entered a dynamic indicator range.
    HoverEnter {
        /// Character position where hover entered.
        position: u64,
        /// The dynamic indicator being hovered.
        indicator: IndicatorNumber,
    },
    /// Hover left a dynamic indicator range.
    HoverLeave {
        /// Character position where hover left.
        position: u64,
        /// The dynamic indicator that was left.
        indicator: IndicatorNumber,
    },
}

/// Trait for receiving decoration events.
pub trait DecorationEventListener: Send + Sync {
    /// Called when a decoration event occurs.
    fn on_decoration_event(&self, event: &DecorationEvent);
}
