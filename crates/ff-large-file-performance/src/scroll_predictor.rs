//! Scroll direction and velocity prediction.

use std::collections::VecDeque;

/// Scroll direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Down,
    Up,
    Stationary,
}

/// Predicts scroll direction from recent scroll events for pre-fetch prioritisation.
pub struct ScrollPredictor {
    /// Recent scroll deltas (ring buffer, positive = down, negative = up).
    recent_deltas: VecDeque<i64>,
    /// Maximum number of deltas to track.
    window_size: usize,
}

impl ScrollPredictor {
    /// Create a new ScrollPredictor.
    pub fn new(window_size: usize) -> Self {
        Self {
            recent_deltas: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    /// Record a scroll event (positive = down, negative = up).
    pub fn record_scroll(&mut self, delta: i64) {
        if self.recent_deltas.len() >= self.window_size {
            self.recent_deltas.pop_front();
        }
        self.recent_deltas.push_back(delta);
    }

    /// Get the predicted scroll direction.
    pub fn predicted_direction(&self) -> ScrollDirection {
        if self.recent_deltas.is_empty() {
            return ScrollDirection::Stationary;
        }
        let sum: i64 = self.recent_deltas.iter().sum();
        if sum > 0 {
            ScrollDirection::Down
        } else if sum < 0 {
            ScrollDirection::Up
        } else {
            ScrollDirection::Stationary
        }
    }

    /// Get the current scroll velocity (lines per frame, absolute value).
    pub fn velocity(&self) -> f64 {
        if self.recent_deltas.is_empty() {
            return 0.0;
        }
        let sum: i64 = self.recent_deltas.iter().map(|d| d.abs()).sum();
        sum as f64 / self.recent_deltas.len() as f64
    }

    /// Whether scrolling is considered "fast" (> 20 lines/frame).
    pub fn is_fast_scrolling(&self) -> bool {
        self.velocity() > 20.0
    }

    /// Whether scrolling is considered "slow" (< 5 lines/frame).
    pub fn is_slow_scrolling(&self) -> bool {
        self.velocity() < 5.0 && !self.recent_deltas.is_empty()
    }

    /// Reset the predictor (no recent scroll events).
    pub fn reset(&mut self) {
        self.recent_deltas.clear();
    }
}

impl Default for ScrollPredictor {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stationary_when_no_events() {
        let predictor = ScrollPredictor::default();
        assert_eq!(predictor.predicted_direction(), ScrollDirection::Stationary);
        assert_eq!(predictor.velocity(), 0.0);
    }

    #[test]
    fn down_direction_from_positive_deltas() {
        let mut predictor = ScrollPredictor::default();
        predictor.record_scroll(5);
        predictor.record_scroll(3);
        assert_eq!(predictor.predicted_direction(), ScrollDirection::Down);
    }

    #[test]
    fn up_direction_from_negative_deltas() {
        let mut predictor = ScrollPredictor::default();
        predictor.record_scroll(-5);
        predictor.record_scroll(-3);
        assert_eq!(predictor.predicted_direction(), ScrollDirection::Up);
    }

    #[test]
    fn fast_scrolling_detection() {
        // Validates: Requirement 8 AC 4
        let mut predictor = ScrollPredictor::default();
        for _ in 0..5 {
            predictor.record_scroll(25);
        }
        assert!(predictor.is_fast_scrolling());
    }

    #[test]
    fn slow_scrolling_detection() {
        // Validates: Requirement 8 AC 4
        let mut predictor = ScrollPredictor::default();
        for _ in 0..5 {
            predictor.record_scroll(2);
        }
        assert!(predictor.is_slow_scrolling());
        assert!(!predictor.is_fast_scrolling());
    }

    #[test]
    fn window_size_limits_history() {
        let mut predictor = ScrollPredictor::new(3);
        predictor.record_scroll(-10);
        predictor.record_scroll(-10);
        predictor.record_scroll(-10);
        predictor.record_scroll(5); // Pushes out first -10
        predictor.record_scroll(5); // Pushes out second -10
        predictor.record_scroll(5); // Pushes out third -10
                                    // Now all 3 are +5
        assert_eq!(predictor.predicted_direction(), ScrollDirection::Down);
    }

    #[test]
    fn reset_clears_history() {
        let mut predictor = ScrollPredictor::default();
        predictor.record_scroll(10);
        predictor.reset();
        assert_eq!(predictor.predicted_direction(), ScrollDirection::Stationary);
    }
}
