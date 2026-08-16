//! Terminal grid — fixed-size grid of cells representing the visible terminal area.
//!
//! Stored row-major for efficient line-based operations (scroll, insert, delete).

use crate::terminal::cell::Cell;

/// A fixed-size grid of terminal cells representing the visible terminal area.
///
/// Cells are stored row-major: `cells[row * cols + col]`.
/// Grid dimensions remain invariant after creation (only changed by explicit `resize()`).
#[derive(Debug, Clone)]
pub struct TerminalGrid {
    /// Grid cells stored row-major.
    cells: Vec<Cell>,
    /// Number of columns.
    cols: u16,
    /// Number of rows.
    rows: u16,
}

impl TerminalGrid {
    /// Creates a new grid with the given dimensions, filled with default cells.
    pub fn new(cols: u16, rows: u16) -> Self {
        let size = cols as usize * rows as usize;
        Self {
            cells: vec![Cell::default(); size],
            cols,
            rows,
        }
    }

    /// Returns the number of columns.
    pub fn cols(&self) -> u16 {
        self.cols
    }

    /// Returns the number of rows.
    pub fn rows(&self) -> u16 {
        self.rows
    }

    /// Gets a reference to the cell at (row, col). Returns None if out of bounds.
    pub fn get(&self, row: u16, col: u16) -> Option<&Cell> {
        if row < self.rows && col < self.cols {
            Some(&self.cells[row as usize * self.cols as usize + col as usize])
        } else {
            None
        }
    }

    /// Gets a mutable reference to the cell at (row, col). Returns None if out of bounds.
    pub fn get_mut(&mut self, row: u16, col: u16) -> Option<&mut Cell> {
        if row < self.rows && col < self.cols {
            Some(&mut self.cells[row as usize * self.cols as usize + col as usize])
        } else {
            None
        }
    }

    /// Sets the cell at (row, col). No-op if out of bounds.
    pub fn set(&mut self, row: u16, col: u16, cell: Cell) {
        if row < self.rows && col < self.cols {
            self.cells[row as usize * self.cols as usize + col as usize] = cell;
        }
    }

    /// Returns a slice representing an entire row.
    pub fn row(&self, row: u16) -> Option<&[Cell]> {
        if row < self.rows {
            let start = row as usize * self.cols as usize;
            let end = start + self.cols as usize;
            Some(&self.cells[start..end])
        } else {
            None
        }
    }

    /// Clears the entire grid (fills with default cells).
    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }

    /// Clears a single row (fills with default cells).
    pub fn clear_row(&mut self, row: u16) {
        if row < self.rows {
            let start = row as usize * self.cols as usize;
            let end = start + self.cols as usize;
            self.cells[start..end].fill(Cell::default());
        }
    }

    /// Scrolls the grid up by one line: removes the top row and adds a
    /// blank row at the bottom. Returns the removed top row.
    pub fn scroll_up(&mut self) -> Vec<Cell> {
        let top_row = self.cells[..self.cols as usize].to_vec();

        // Shift all rows up by one
        let cols = self.cols as usize;
        self.cells.rotate_left(cols);

        // Clear the bottom row
        let total = self.cells.len();
        self.cells[total - cols..].fill(Cell::default());

        top_row
    }

    /// Scrolls the grid down by one line: removes the bottom row and adds a
    /// blank row at the top.
    pub fn scroll_down(&mut self) {
        let cols = self.cols as usize;
        self.cells.rotate_right(cols);

        // Clear the top row
        self.cells[..cols].fill(Cell::default());
    }

    /// Resizes the grid to new dimensions, clearing all content.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        let size = cols as usize * rows as usize;
        self.cells = vec![Cell::default(); size];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 7.8
    #[test]
    fn new_grid_has_correct_dimensions() {
        let grid = TerminalGrid::new(80, 24);
        assert_eq!(grid.cols(), 80);
        assert_eq!(grid.rows(), 24);
    }

    // Validates: Requirement 7.8
    #[test]
    fn new_grid_cells_are_default() {
        let grid = TerminalGrid::new(80, 24);
        let cell = grid.get(0, 0).unwrap();
        assert_eq!(cell.character, ' ');
    }

    // Validates: Requirement 7.8
    #[test]
    fn set_and_get_cell() {
        let mut grid = TerminalGrid::new(80, 24);
        let cell = Cell::new('A');
        grid.set(5, 10, cell.clone());
        assert_eq!(grid.get(5, 10).unwrap().character, 'A');
    }

    // Validates: Requirement 7.8
    #[test]
    fn out_of_bounds_get_returns_none() {
        let grid = TerminalGrid::new(80, 24);
        assert!(grid.get(24, 0).is_none());
        assert!(grid.get(0, 80).is_none());
    }

    // Validates: Requirement 7.8
    #[test]
    fn scroll_up_shifts_content() {
        let mut grid = TerminalGrid::new(3, 3);
        grid.set(0, 0, Cell::new('A'));
        grid.set(1, 0, Cell::new('B'));
        grid.set(2, 0, Cell::new('C'));

        let removed = grid.scroll_up();
        assert_eq!(removed[0].character, 'A');
        assert_eq!(grid.get(0, 0).unwrap().character, 'B');
        assert_eq!(grid.get(1, 0).unwrap().character, 'C');
        assert_eq!(grid.get(2, 0).unwrap().character, ' '); // new blank row
    }

    // Validates: Requirement 7.8
    #[test]
    fn scroll_down_shifts_content() {
        let mut grid = TerminalGrid::new(3, 3);
        grid.set(0, 0, Cell::new('A'));
        grid.set(1, 0, Cell::new('B'));
        grid.set(2, 0, Cell::new('C'));

        grid.scroll_down();
        assert_eq!(grid.get(0, 0).unwrap().character, ' '); // new blank row
        assert_eq!(grid.get(1, 0).unwrap().character, 'A');
        assert_eq!(grid.get(2, 0).unwrap().character, 'B');
    }

    // Validates: Requirement 7.8
    #[test]
    fn clear_resets_all_cells() {
        let mut grid = TerminalGrid::new(3, 3);
        grid.set(1, 1, Cell::new('X'));
        grid.clear();
        assert_eq!(grid.get(1, 1).unwrap().character, ' ');
    }

    // Validates: Requirement 7.8
    #[test]
    fn resize_changes_dimensions_and_clears() {
        let mut grid = TerminalGrid::new(80, 24);
        grid.set(0, 0, Cell::new('X'));
        grid.resize(40, 12);
        assert_eq!(grid.cols(), 40);
        assert_eq!(grid.rows(), 12);
        assert_eq!(grid.get(0, 0).unwrap().character, ' ');
    }
}
