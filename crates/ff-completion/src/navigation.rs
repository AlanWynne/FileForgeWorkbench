//! Selection state and keyboard navigation for the completion popup.

/// Manages the highlighted item index and scroll state for the popup.
///
/// Handles wrap-around, clamping, and page-based navigation.
#[derive(Debug, Clone)]
pub struct SelectionState {
    /// The currently highlighted item index.
    selected_index: usize,
    /// Number of items visible per page (used for PageUp/PageDown).
    page_size: usize,
    /// Total number of items in the current list.
    total_items: usize,
    /// Whether navigation wraps around at list boundaries.
    wrap_enabled: bool,
}

impl SelectionState {
    /// Creates a new selection state.
    ///
    /// # Panics
    ///
    /// Does not panic. If `total_items` is 0, navigation operations are no-ops.
    pub fn new(total_items: usize, page_size: usize, wrap_enabled: bool) -> Self {
        Self {
            selected_index: 0,
            page_size,
            total_items,
            wrap_enabled,
        }
    }

    /// Returns the currently selected index.
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Returns the total number of items.
    pub fn total_items(&self) -> usize {
        self.total_items
    }

    /// Returns the page size.
    pub fn page_size(&self) -> usize {
        self.page_size
    }

    /// Returns whether wrapping is enabled.
    pub fn wrap_enabled(&self) -> bool {
        self.wrap_enabled
    }

    /// Moves the selection down by one item.
    ///
    /// If at the last item:
    /// - With wrap enabled: moves to index 0
    /// - With wrap disabled: stays at last item (clamped)
    pub fn move_down(&mut self) {
        if self.total_items == 0 {
            return;
        }
        if self.selected_index >= self.total_items - 1 {
            if self.wrap_enabled {
                self.selected_index = 0;
            }
            // else: clamped, no change
        } else {
            self.selected_index += 1;
        }
    }

    /// Moves the selection up by one item.
    ///
    /// If at the first item:
    /// - With wrap enabled: moves to last item
    /// - With wrap disabled: stays at index 0 (clamped)
    pub fn move_up(&mut self) {
        if self.total_items == 0 {
            return;
        }
        if self.selected_index == 0 {
            if self.wrap_enabled {
                self.selected_index = self.total_items - 1;
            }
            // else: clamped, no change
        } else {
            self.selected_index -= 1;
        }
    }

    /// Moves the selection down by one page. Clamps at the end.
    pub fn page_down(&mut self) {
        if self.total_items == 0 {
            return;
        }
        self.selected_index = (self.selected_index + self.page_size).min(self.total_items - 1);
    }

    /// Moves the selection up by one page. Clamps at the start.
    pub fn page_up(&mut self) {
        if self.total_items == 0 {
            return;
        }
        self.selected_index = self.selected_index.saturating_sub(self.page_size);
    }

    /// Resets the selection to index 0 with a new total item count.
    ///
    /// Called when the list is re-filtered and the items change.
    pub fn reset(&mut self, new_total: usize) {
        self.total_items = new_total;
        self.selected_index = 0;
    }

    /// Sets the selected index directly (clamped to valid range).
    pub fn set_selected(&mut self, index: usize) {
        if self.total_items == 0 {
            self.selected_index = 0;
        } else {
            self.selected_index = index.min(self.total_items - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 4.1 (down arrow wraps)
    #[test]
    fn move_down_wraps_from_last_to_first() {
        let mut state = SelectionState::new(5, 5, true);
        state.set_selected(4); // last item
        state.move_down();
        assert_eq!(state.selected_index(), 0);
    }

    // Validates: Requirement 4.1 (down arrow no-wrap clamps)
    #[test]
    fn move_down_clamps_at_last_when_no_wrap() {
        let mut state = SelectionState::new(5, 5, false);
        state.set_selected(4);
        state.move_down();
        assert_eq!(state.selected_index(), 4);
    }

    // Validates: Requirement 4.2 (up arrow wraps)
    #[test]
    fn move_up_wraps_from_first_to_last() {
        let mut state = SelectionState::new(5, 5, true);
        state.move_up();
        assert_eq!(state.selected_index(), 4);
    }

    // Validates: Requirement 4.2 (up arrow no-wrap clamps)
    #[test]
    fn move_up_clamps_at_first_when_no_wrap() {
        let mut state = SelectionState::new(5, 5, false);
        state.move_up();
        assert_eq!(state.selected_index(), 0);
    }

    // Validates: Requirement 4.8 (page down)
    #[test]
    fn page_down_advances_by_page_size() {
        let mut state = SelectionState::new(20, 5, false);
        state.page_down();
        assert_eq!(state.selected_index(), 5);
        state.page_down();
        assert_eq!(state.selected_index(), 10);
    }

    #[test]
    fn page_down_clamps_at_end() {
        let mut state = SelectionState::new(20, 5, false);
        state.set_selected(18);
        state.page_down();
        assert_eq!(state.selected_index(), 19);
    }

    // Validates: Requirement 4.8 (page up)
    #[test]
    fn page_up_retreats_by_page_size() {
        let mut state = SelectionState::new(20, 5, false);
        state.set_selected(10);
        state.page_up();
        assert_eq!(state.selected_index(), 5);
    }

    #[test]
    fn page_up_clamps_at_start() {
        let mut state = SelectionState::new(20, 5, false);
        state.set_selected(2);
        state.page_up();
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn reset_sets_index_to_zero_with_new_total() {
        let mut state = SelectionState::new(10, 5, true);
        state.set_selected(7);
        state.reset(3);
        assert_eq!(state.selected_index(), 0);
        assert_eq!(state.total_items(), 3);
    }

    #[test]
    fn operations_on_empty_list_are_no_ops() {
        let mut state = SelectionState::new(0, 5, true);
        state.move_down();
        assert_eq!(state.selected_index(), 0);
        state.move_up();
        assert_eq!(state.selected_index(), 0);
        state.page_down();
        assert_eq!(state.selected_index(), 0);
        state.page_up();
        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn sequential_down_traverses_all_items() {
        let mut state = SelectionState::new(5, 5, true);
        for expected in 1..5 {
            state.move_down();
            assert_eq!(state.selected_index(), expected);
        }
        // Next down wraps to 0
        state.move_down();
        assert_eq!(state.selected_index(), 0);
    }
}
