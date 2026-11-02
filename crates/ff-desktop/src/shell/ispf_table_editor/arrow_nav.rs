//! Shared ISPF Table Editor -- full arrow-key focus navigation (CR-CH-078,
//! ispf-table-editor Requirement 11).
//!
//! egui provides no vertical / grid arrow traversal for a field matrix, so the
//! shared component OWNS the walk: a pure [`arrow_target`] computes the adjacent
//! cell (clamped, no wrap on either axis), and [`apply_arrow_walk`] CONSUMES the
//! pressed arrow BEFORE egui traverses (the render_menu_bar / render_chrome
//! B056-B059 lesson) and requests focus on the target cell's stable same-frame
//! id. It lives here so every host (Alias / Menu / Kinds / future) inherits it
//! without re-implementing anything (Requirement 11.6).

/// A directional arrow press across the field matrix (Requirement 11.1, 11.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowDir {
    /// Up -- same column, row above (clamped at the first row).
    Up,
    /// Down -- same column, row below (clamped at the last row).
    Down,
    /// Left -- previous column, same row (clamped at the first column).
    Left,
    /// Right -- next column, same row (clamped at the last column).
    Right,
}

/// Compute the target cell `(row, col)` for an arrow press from `cur` over a
/// matrix of `rows` x `cols`, CLAMPED at every edge with NO wrap on either axis
/// (Requirement 11.1, 11.2). Up/Down move within the same column; Left/Right
/// move within the same row. A zero-sized matrix or an out-of-range `cur` is
/// returned unchanged (nothing to move to).
///
/// Validates: ispf-table-editor Requirement 11.1, 11.2
pub fn arrow_target(
    rows: usize,
    cols: usize,
    cur: (usize, usize),
    dir: ArrowDir,
) -> (usize, usize) {
    let (r, c) = cur;
    if rows == 0 || cols == 0 || r >= rows || c >= cols {
        return cur;
    }
    match dir {
        ArrowDir::Up => (r.saturating_sub(1), c),
        ArrowDir::Down => ((r + 1).min(rows - 1), c),
        ArrowDir::Left => (r, c.saturating_sub(1)),
        ArrowDir::Right => (r, (c + 1).min(cols - 1)),
    }
}

/// Apply the arrow walk against a field `matrix` of stable cell ids (row-major,
/// visual order) given the currently `focused` id. For each pressed arrow this
/// CONSUMES the key (so egui's own traversal does not also act on it) and, when
/// the focused id is a matrix cell, requests focus on the clamped target cell.
/// Does nothing when no matrix cell holds focus (so the `Command ===>` line keeps
/// its CR-NR-096 Up/Down history gesture, Requirement 11.5).
///
/// The matrix rows need not be uniform length; `cols` is taken from the focused
/// cell's own row so a short row clamps against its own width.
///
/// Validates: ispf-table-editor Requirement 11.1, 11.2, 11.3, 11.5
pub fn apply_arrow_walk(ui: &egui::Ui, matrix: &[Vec<egui::Id>], focused: Option<egui::Id>) {
    // Always drain any arrow presses this frame so they never leak into egui's
    // default focus traversal, regardless of whether a cell is focused.
    let pressed = [
        (ArrowDir::Up, egui::Key::ArrowUp),
        (ArrowDir::Down, egui::Key::ArrowDown),
        (ArrowDir::Left, egui::Key::ArrowLeft),
        (ArrowDir::Right, egui::Key::ArrowRight),
    ];

    let Some(focused) = focused else {
        return;
    };
    // Locate the focused id in the matrix (its row, col). Not found -> the
    // command line or some other control holds focus: do not consume anything.
    let Some(cur) = find_cell(matrix, focused) else {
        return;
    };

    let rows = matrix.len();
    let cols = matrix[cur.0].len();
    for (dir, key) in pressed {
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
            let (tr, tc) = arrow_target(rows, cols, cur, dir);
            if let Some(row) = matrix.get(tr) {
                if let Some(&target) = row.get(tc) {
                    ui.ctx().memory_mut(|m| m.request_focus(target));
                }
            }
        }
    }
}

/// Find the `(row, col)` of `id` in the row-major `matrix`, if present.
fn find_cell(matrix: &[Vec<egui::Id>], id: egui::Id) -> Option<(usize, usize)> {
    for (r, row) in matrix.iter().enumerate() {
        if let Some(c) = row.iter().position(|&cell| cell == id) {
            return Some((r, c));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: ispf-table-editor Requirement 11.1 -- Down from (0,c) moves to
    // (1,c) in the same column.
    #[test]
    fn arrow_target_down_moves_to_next_row_same_column() {
        assert_eq!(arrow_target(3, 4, (0, 2), ArrowDir::Down), (1, 2));
    }

    // Validates: ispf-table-editor Requirement 11.1 -- Up on the first row clamps
    // (no wrap).
    #[test]
    fn arrow_target_up_on_first_row_clamps() {
        assert_eq!(arrow_target(3, 4, (0, 2), ArrowDir::Up), (0, 2));
    }

    // Validates: ispf-table-editor Requirement 11.1 -- Down on the last row clamps
    // (no wrap).
    #[test]
    fn arrow_target_down_on_last_row_clamps() {
        assert_eq!(arrow_target(3, 4, (2, 1), ArrowDir::Down), (2, 1));
    }

    // Validates: ispf-table-editor Requirement 11.1 -- Up from an interior row
    // moves to the row above in the same column.
    #[test]
    fn arrow_target_up_moves_to_previous_row_same_column() {
        assert_eq!(arrow_target(3, 4, (2, 1), ArrowDir::Up), (1, 1));
    }

    // Validates: ispf-table-editor Requirement 11.2 -- Right from (r,0) moves to
    // (r,1) in the same row.
    #[test]
    fn arrow_target_right_moves_to_next_column_same_row() {
        assert_eq!(arrow_target(3, 4, (1, 0), ArrowDir::Right), (1, 1));
    }

    // Validates: ispf-table-editor Requirement 11.2 -- Left on the first column
    // clamps (no wrap).
    #[test]
    fn arrow_target_left_on_first_column_clamps() {
        assert_eq!(arrow_target(3, 4, (1, 0), ArrowDir::Left), (1, 0));
    }

    // Validates: ispf-table-editor Requirement 11.2 -- Right on the last column
    // clamps (no wrap to another row).
    #[test]
    fn arrow_target_right_on_last_column_clamps() {
        assert_eq!(arrow_target(3, 4, (1, 3), ArrowDir::Right), (1, 3));
    }

    // Validates: ispf-table-editor Requirement 11.2 -- Left from an interior
    // column moves to the column to its left in the same row.
    #[test]
    fn arrow_target_left_moves_to_previous_column_same_row() {
        assert_eq!(arrow_target(3, 4, (1, 3), ArrowDir::Left), (1, 2));
    }

    // Validates: ispf-table-editor Requirement 11.1, 11.2 -- a zero-sized matrix
    // or an out-of-range current cell is returned unchanged.
    #[test]
    fn arrow_target_degenerate_matrices_return_current() {
        assert_eq!(arrow_target(0, 0, (0, 0), ArrowDir::Down), (0, 0));
        assert_eq!(arrow_target(0, 4, (0, 0), ArrowDir::Down), (0, 0));
        assert_eq!(arrow_target(3, 0, (0, 0), ArrowDir::Right), (0, 0));
        assert_eq!(arrow_target(3, 4, (5, 5), ArrowDir::Down), (5, 5));
    }
}
