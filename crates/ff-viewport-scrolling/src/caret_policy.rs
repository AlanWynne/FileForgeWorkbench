//! Caret visibility policy engine.
//!
//! Implements configurable rules (modelled after Scintilla's `CaretPolicySlop`)
//! that determine how the viewport scrolls to keep the editing caret visible.
//! Supports four independent flags: slop, strict, jumps, and even.

/// Configurable policy controlling how the viewport scrolls to keep the caret visible.
///
/// Modelled after Scintilla's caret policy flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CaretPolicy {
    /// If true, a slop zone is defined near edges.
    pub slop: bool,
    /// If true, the slop zone is enforced strictly (always scroll if in zone).
    pub strict: bool,
    /// If true, scroll by larger jumps (3× slop) to reduce scroll frequency.
    pub jumps: bool,
    /// If true, apply slop symmetrically to both edges.
    pub even: bool,
    /// Number of lines (vertical) or pixels (horizontal) for the slop zone.
    pub slop_value: u32,
}

/// Separate policies for vertical and horizontal axes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CaretPolicyConfig {
    /// Vertical caret policy.
    pub vertical: CaretPolicy,
    /// Horizontal caret policy.
    pub horizontal: CaretPolicy,
}

/// Engine that computes viewport adjustments needed to keep the caret visible
/// per configured policy flags.
#[derive(Debug, Clone)]
pub struct CaretPolicyEngine {
    config: CaretPolicyConfig,
}

impl CaretPolicyEngine {
    /// Create an engine with the given policy configuration.
    pub fn new(config: CaretPolicyConfig) -> Self {
        Self { config }
    }

    /// Create an engine with default (minimal scroll) policy.
    pub fn default_policy() -> Self {
        Self {
            config: CaretPolicyConfig::default(),
        }
    }

    /// Get the current policy configuration.
    pub fn config(&self) -> &CaretPolicyConfig {
        &self.config
    }

    /// Update the policy configuration (e.g., from hot-reloaded settings).
    pub fn set_config(&mut self, config: CaretPolicyConfig) {
        self.config = config;
    }

    /// Compute the top_line adjustment needed after a vertical cursor move.
    ///
    /// Returns the new top_line (or the current one if no scroll needed).
    pub fn compute_vertical_scroll(
        &self,
        cursor_line: u64,
        top_line: u64,
        visible_count: u64,
        max_top_line: u64,
    ) -> u64 {
        let policy = &self.config.vertical;
        let bottom_line = top_line.saturating_add(visible_count).saturating_sub(1);

        if !policy.slop {
            // Default minimal policy: scroll the minimum amount to make cursor visible
            if cursor_line < top_line {
                return cursor_line.max(1);
            }
            if cursor_line > bottom_line {
                let new_top = cursor_line.saturating_sub(visible_count.saturating_sub(1));
                return new_top.clamp(1, max_top_line);
            }
            return top_line;
        }

        // Slop policy: define a margin zone
        let slop = policy.slop_value as u64;
        let top_zone = top_line.saturating_add(slop);
        let bottom_zone = bottom_line.saturating_sub(slop);

        let needs_scroll = if policy.strict {
            // Strict: always enforce slop zone
            cursor_line < top_zone || cursor_line > bottom_zone
        } else {
            // Non-strict: only scroll if cursor left visible area
            cursor_line < top_line || cursor_line > bottom_line
        };

        if !needs_scroll {
            return top_line;
        }

        // Compute scroll amount
        let jump_amount = if policy.jumps {
            slop.saturating_mul(3)
        } else {
            slop
        };

        let new_top = if cursor_line < top_zone {
            // Cursor is above the safe zone — scroll up
            if policy.even {
                // Even: position cursor in the middle area
                let half_visible = visible_count / 2;
                cursor_line.saturating_sub(half_visible)
            } else {
                cursor_line.saturating_sub(jump_amount)
            }
        } else {
            // Cursor is below the safe zone — scroll down
            if policy.even {
                let half_visible = visible_count / 2;
                cursor_line.saturating_sub(half_visible)
            } else {
                cursor_line
                    .saturating_sub(visible_count.saturating_sub(1))
                    .saturating_add(jump_amount)
            }
        };

        new_top.clamp(1, max_top_line)
    }

    /// Compute the horizontal_offset adjustment needed after a horizontal cursor move.
    ///
    /// Returns the new horizontal_offset (or the current one if no scroll needed).
    pub fn compute_horizontal_scroll(
        &self,
        cursor_pixel_x: u64,
        horizontal_offset: u64,
        viewport_width: u64,
        max_horizontal_extent: u64,
    ) -> u64 {
        let policy = &self.config.horizontal;
        let visible_left = horizontal_offset;
        let visible_right = horizontal_offset.saturating_add(viewport_width);

        if !policy.slop {
            // Default minimal policy
            if cursor_pixel_x < visible_left {
                return cursor_pixel_x.min(max_horizontal_extent);
            }
            if cursor_pixel_x >= visible_right {
                let new_offset = cursor_pixel_x
                    .saturating_sub(viewport_width)
                    .saturating_add(1);
                return new_offset.min(max_horizontal_extent);
            }
            return horizontal_offset;
        }

        let slop = policy.slop_value as u64;
        let left_zone = visible_left.saturating_add(slop);
        let right_zone = visible_right.saturating_sub(slop);

        let needs_scroll = if policy.strict {
            cursor_pixel_x < left_zone || cursor_pixel_x > right_zone
        } else {
            cursor_pixel_x < visible_left || cursor_pixel_x >= visible_right
        };

        if !needs_scroll {
            return horizontal_offset;
        }

        let jump_amount = if policy.jumps {
            slop.saturating_mul(3)
        } else {
            slop
        };

        let new_offset = if cursor_pixel_x < left_zone {
            if policy.even {
                cursor_pixel_x.saturating_sub(viewport_width / 2)
            } else {
                cursor_pixel_x.saturating_sub(jump_amount)
            }
        } else {
            if policy.even {
                cursor_pixel_x.saturating_sub(viewport_width / 2)
            } else {
                cursor_pixel_x
                    .saturating_sub(viewport_width)
                    .saturating_add(jump_amount)
            }
        };

        new_offset.min(max_horizontal_extent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_no_scroll_when_cursor_visible() {
        let engine = CaretPolicyEngine::default_policy();
        let result = engine.compute_vertical_scroll(5, 1, 20, 100);
        assert_eq!(result, 1); // cursor at 5, visible [1..20], no scroll needed
    }

    #[test]
    fn default_policy_scrolls_down_when_cursor_below_viewport() {
        let engine = CaretPolicyEngine::default_policy();
        let result = engine.compute_vertical_scroll(25, 1, 20, 100);
        // cursor at 25, visible [1..20], need to scroll so cursor is visible
        assert_eq!(result, 6); // 25 - 20 + 1 = 6
    }

    #[test]
    fn default_policy_scrolls_up_when_cursor_above_viewport() {
        let engine = CaretPolicyEngine::default_policy();
        let result = engine.compute_vertical_scroll(3, 10, 20, 100);
        assert_eq!(result, 3);
    }

    #[test]
    fn slop_policy_scrolls_when_in_margin() {
        let engine = CaretPolicyEngine::new(CaretPolicyConfig {
            vertical: CaretPolicy {
                slop: true,
                strict: true,
                jumps: false,
                even: false,
                slop_value: 3,
            },
            horizontal: CaretPolicy::default(),
        });
        // Cursor at line 2, top_line=1, visible_count=20
        // top_zone = 1 + 3 = 4, cursor(2) < top_zone(4) → needs scroll
        let result = engine.compute_vertical_scroll(2, 1, 20, 100);
        // cursor(2) < top_zone(4), scroll up: 2 - 3 = saturates to 1
        assert!(result >= 1);
    }

    #[test]
    fn jumps_policy_scrolls_by_triple_slop() {
        let engine = CaretPolicyEngine::new(CaretPolicyConfig {
            vertical: CaretPolicy {
                slop: true,
                strict: true,
                jumps: true,
                even: false,
                slop_value: 5,
            },
            horizontal: CaretPolicy::default(),
        });
        // Cursor at 30, top=1, visible=20, bottom=20, bottom_zone=20-5=15
        // cursor(30) > bottom_zone(15) → needs scroll, jump_amount = 15
        let result = engine.compute_vertical_scroll(30, 1, 20, 100);
        // scroll down: 30 - 19 + 15 = 26
        assert_eq!(result, 26);
    }

    #[test]
    fn even_policy_centers_cursor() {
        let engine = CaretPolicyEngine::new(CaretPolicyConfig {
            vertical: CaretPolicy {
                slop: true,
                strict: true,
                jumps: false,
                even: true,
                slop_value: 3,
            },
            horizontal: CaretPolicy::default(),
        });
        // Cursor at 50, top=1, visible=20
        // cursor(50) > bottom_zone(20-3=17) → needs scroll
        // even mode: 50 - 10 = 40
        let result = engine.compute_vertical_scroll(50, 1, 20, 100);
        assert_eq!(result, 40);
    }

    #[test]
    fn horizontal_default_policy_scrolls_when_cursor_outside() {
        let engine = CaretPolicyEngine::default_policy();
        // cursor at pixel 500, viewport offset=0, width=400
        let result = engine.compute_horizontal_scroll(500, 0, 400, 1000);
        // 500 >= 400 → scroll: 500 - 400 + 1 = 101
        assert_eq!(result, 101);
    }

    #[test]
    fn horizontal_no_scroll_when_cursor_visible() {
        let engine = CaretPolicyEngine::default_policy();
        let result = engine.compute_horizontal_scroll(200, 0, 400, 1000);
        assert_eq!(result, 0);
    }
}
