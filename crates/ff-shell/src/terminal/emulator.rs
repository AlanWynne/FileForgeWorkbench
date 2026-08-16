//! VT100/ANSI terminal emulator state machine.
//!
//! Parses escape sequences, maintains a grid of cells, and provides
//! the rendered state for the GUI shell to paint.

use std::collections::VecDeque;

use crate::terminal::cell::{Cell, CellAttributes, TerminalColor};
use crate::terminal::grid::TerminalGrid;

/// Cursor state within the terminal emulator.
#[derive(Debug, Clone)]
pub struct CursorState {
    /// Current row (0-indexed).
    pub row: u16,
    /// Current column (0-indexed).
    pub col: u16,
    /// Whether the cursor is visible.
    pub visible: bool,
}

/// Parser state for multi-byte escape sequences.
#[derive(Debug, Clone, PartialEq)]
enum ParserState {
    /// Normal character input.
    Ground,
    /// Received ESC, waiting for next byte.
    Escape,
    /// In a CSI sequence (ESC [), collecting parameters.
    Csi { params: Vec<u16>, current: u16 },
    /// In an OSC sequence (ESC ]), collecting string.
    Osc { data: String },
}

/// VT100/ANSI terminal emulator state machine.
///
/// Parses escape sequences from raw byte streams, maintains a grid of cells,
/// and provides the rendered state for the GUI shell to paint.
pub struct TerminalEmulator {
    /// The visible cell grid (rows × columns).
    grid: TerminalGrid,
    /// Current cursor position and visibility.
    cursor: CursorState,
    /// Parser state for multi-byte escape sequences.
    parser_state: ParserState,
    /// Scrollback buffer above the visible grid.
    scrollback: VecDeque<Vec<Cell>>,
    /// Maximum scrollback lines (configurable).
    max_scrollback: usize,
    /// Current character attributes (applied to new characters).
    current_attrs: CellAttributes,
    /// Saved cursor position (for save/restore operations).
    saved_cursor: Option<CursorState>,
    /// Terminal title (set via OSC escape sequence).
    title: Option<String>,
}

impl std::fmt::Debug for TerminalEmulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalEmulator")
            .field("cols", &self.grid.cols())
            .field("rows", &self.grid.rows())
            .field("cursor", &self.cursor)
            .field("scrollback_len", &self.scrollback.len())
            .finish()
    }
}

impl TerminalEmulator {
    /// Creates a new emulator with given dimensions.
    pub fn new(cols: u16, rows: u16, max_scrollback: usize) -> Self {
        Self {
            grid: TerminalGrid::new(cols, rows),
            cursor: CursorState {
                row: 0,
                col: 0,
                visible: true,
            },
            parser_state: ParserState::Ground,
            scrollback: VecDeque::new(),
            max_scrollback,
            current_attrs: CellAttributes::default(),
            saved_cursor: None,
            title: None,
        }
    }

    /// Feeds raw bytes from the PTY into the emulator.
    ///
    /// Parses escape sequences and updates the grid.
    pub fn feed(&mut self, data: &[u8]) {
        for &byte in data {
            self.process_byte(byte);
        }
    }

    /// Returns a reference to the current visible grid.
    pub fn grid(&self) -> &TerminalGrid {
        &self.grid
    }

    /// Returns the cursor state.
    pub fn cursor(&self) -> &CursorState {
        &self.cursor
    }

    /// Resizes the terminal to new dimensions.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.grid.resize(cols, rows);
        // Clamp cursor to new bounds
        if self.cursor.row >= rows {
            self.cursor.row = rows.saturating_sub(1);
        }
        if self.cursor.col >= cols {
            self.cursor.col = cols.saturating_sub(1);
        }
    }

    /// Returns the scrollback buffer.
    pub fn scrollback(&self) -> &VecDeque<Vec<Cell>> {
        &self.scrollback
    }

    /// Returns the terminal title (set via OSC escape sequence).
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Processes a single byte through the parser state machine.
    fn process_byte(&mut self, byte: u8) {
        match &self.parser_state {
            ParserState::Ground => self.process_ground(byte),
            ParserState::Escape => self.process_escape(byte),
            ParserState::Csi { .. } => self.process_csi(byte),
            ParserState::Osc { .. } => self.process_osc(byte),
        }
    }

    /// Process a byte in the ground (normal) state.
    fn process_ground(&mut self, byte: u8) {
        match byte {
            0x1B => {
                // ESC
                self.parser_state = ParserState::Escape;
            }
            0x0D => {
                // CR — carriage return
                self.cursor.col = 0;
            }
            0x0A => {
                // LF — line feed
                self.line_feed();
            }
            0x08 => {
                // BS — backspace
                self.cursor.col = self.cursor.col.saturating_sub(1);
            }
            0x09 => {
                // HT — horizontal tab
                self.cursor.col = ((self.cursor.col / 8) + 1) * 8;
                if self.cursor.col >= self.grid.cols() {
                    self.cursor.col = self.grid.cols() - 1;
                }
            }
            0x07 => {
                // BEL — bell (ignore)
            }
            0x00..=0x1F => {
                // Other control characters — ignore
            }
            _ => {
                // Printable character
                self.put_char(byte as char);
            }
        }
    }

    /// Process a byte after ESC.
    fn process_escape(&mut self, byte: u8) {
        match byte {
            b'[' => {
                // CSI introducer
                self.parser_state = ParserState::Csi {
                    params: Vec::new(),
                    current: 0,
                };
            }
            b']' => {
                // OSC introducer
                self.parser_state = ParserState::Osc {
                    data: String::new(),
                };
            }
            b'7' => {
                // Save cursor
                self.saved_cursor = Some(self.cursor.clone());
                self.parser_state = ParserState::Ground;
            }
            b'8' => {
                // Restore cursor
                if let Some(saved) = self.saved_cursor.clone() {
                    self.cursor = saved;
                }
                self.parser_state = ParserState::Ground;
            }
            b'D' => {
                // Index (scroll up)
                self.line_feed();
                self.parser_state = ParserState::Ground;
            }
            b'M' => {
                // Reverse index (scroll down)
                if self.cursor.row == 0 {
                    self.grid.scroll_down();
                } else {
                    self.cursor.row -= 1;
                }
                self.parser_state = ParserState::Ground;
            }
            b'c' => {
                // Full reset
                self.reset();
                self.parser_state = ParserState::Ground;
            }
            _ => {
                // Unknown escape — return to ground
                self.parser_state = ParserState::Ground;
            }
        }
    }

    /// Process a byte in CSI state.
    fn process_csi(&mut self, byte: u8) {
        // Extract params from state (must clone due to borrow checker)
        let (mut params, mut current) = match &self.parser_state {
            ParserState::Csi { params, current } => (params.clone(), *current),
            _ => unreachable!(),
        };

        match byte {
            b'0'..=b'9' => {
                current = current
                    .saturating_mul(10)
                    .saturating_add((byte - b'0') as u16);
                self.parser_state = ParserState::Csi { params, current };
            }
            b';' => {
                params.push(current);
                self.parser_state = ParserState::Csi { params, current: 0 };
            }
            _ => {
                // Final byte — execute the CSI sequence
                params.push(current);
                self.execute_csi(byte, &params);
                self.parser_state = ParserState::Ground;
            }
        }
    }

    /// Process a byte in OSC state.
    fn process_osc(&mut self, byte: u8) {
        let mut data = match &self.parser_state {
            ParserState::Osc { data } => data.clone(),
            _ => unreachable!(),
        };

        match byte {
            0x07 | 0x1B => {
                // BEL or ESC terminates OSC
                // Parse OSC: typically "0;title" or "2;title"
                if let Some(title) = data.strip_prefix("0;").or_else(|| data.strip_prefix("2;")) {
                    self.title = Some(title.to_string());
                }
                self.parser_state = ParserState::Ground;
            }
            _ => {
                data.push(byte as char);
                self.parser_state = ParserState::Osc { data };
            }
        }
    }

    /// Execute a CSI sequence with the given final byte and parameters.
    fn execute_csi(&mut self, final_byte: u8, params: &[u16]) {
        match final_byte {
            b'A' => {
                // CUU — cursor up
                let n = params.first().copied().unwrap_or(1).max(1);
                self.cursor.row = self.cursor.row.saturating_sub(n);
            }
            b'B' => {
                // CUD — cursor down
                let n = params.first().copied().unwrap_or(1).max(1);
                self.cursor.row = (self.cursor.row + n).min(self.grid.rows() - 1);
            }
            b'C' => {
                // CUF — cursor forward (right)
                let n = params.first().copied().unwrap_or(1).max(1);
                self.cursor.col = (self.cursor.col + n).min(self.grid.cols() - 1);
            }
            b'D' => {
                // CUB — cursor backward (left)
                let n = params.first().copied().unwrap_or(1).max(1);
                self.cursor.col = self.cursor.col.saturating_sub(n);
            }
            b'H' | b'f' => {
                // CUP — cursor position (row;col, 1-indexed)
                let row = params.first().copied().unwrap_or(1).max(1) - 1;
                let col = params.get(1).copied().unwrap_or(1).max(1) - 1;
                self.cursor.row = row.min(self.grid.rows() - 1);
                self.cursor.col = col.min(self.grid.cols() - 1);
            }
            b'J' => {
                // ED — erase display
                let mode = params.first().copied().unwrap_or(0);
                self.erase_display(mode);
            }
            b'K' => {
                // EL — erase line
                let mode = params.first().copied().unwrap_or(0);
                self.erase_line(mode);
            }
            b'm' => {
                // SGR — set graphic rendition
                self.process_sgr(params);
            }
            b's' => {
                // Save cursor position
                self.saved_cursor = Some(self.cursor.clone());
            }
            b'u' => {
                // Restore cursor position
                if let Some(saved) = self.saved_cursor.clone() {
                    self.cursor = saved;
                }
            }
            b'h' | b'l' => {
                // DECSET/DECRST — set/reset mode (e.g., cursor visibility)
                if params.first().copied() == Some(25) && final_byte == b'h' {
                    self.cursor.visible = true;
                } else if params.first().copied() == Some(25) && final_byte == b'l' {
                    self.cursor.visible = false;
                }
            }
            _ => {
                // Unknown CSI sequence — ignore
            }
        }
    }

    /// Process SGR (Set Graphic Rendition) parameters.
    fn process_sgr(&mut self, params: &[u16]) {
        if params.is_empty() || (params.len() == 1 && params[0] == 0) {
            self.current_attrs.reset();
            return;
        }

        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => self.current_attrs.reset(),
                1 => self.current_attrs.bold = true,
                2 => self.current_attrs.dim = true,
                3 => self.current_attrs.italic = true,
                4 => self.current_attrs.underline = true,
                7 => self.current_attrs.inverse = true,
                9 => self.current_attrs.strikethrough = true,
                22 => {
                    self.current_attrs.bold = false;
                    self.current_attrs.dim = false;
                }
                23 => self.current_attrs.italic = false,
                24 => self.current_attrs.underline = false,
                27 => self.current_attrs.inverse = false,
                29 => self.current_attrs.strikethrough = false,
                30..=37 => {
                    self.current_attrs.foreground = TerminalColor::Ansi((params[i] - 30) as u8);
                }
                38 => {
                    // Extended foreground color
                    if i + 1 < params.len() {
                        match params[i + 1] {
                            5 if i + 2 < params.len() => {
                                self.current_attrs.foreground =
                                    TerminalColor::Palette(params[i + 2] as u8);
                                i += 2;
                            }
                            2 if i + 4 < params.len() => {
                                self.current_attrs.foreground = TerminalColor::Rgb(
                                    params[i + 2] as u8,
                                    params[i + 3] as u8,
                                    params[i + 4] as u8,
                                );
                                i += 4;
                            }
                            _ => {}
                        }
                    }
                }
                39 => self.current_attrs.foreground = TerminalColor::Default,
                40..=47 => {
                    self.current_attrs.background = TerminalColor::Ansi((params[i] - 40) as u8);
                }
                48 => {
                    // Extended background color
                    if i + 1 < params.len() {
                        match params[i + 1] {
                            5 if i + 2 < params.len() => {
                                self.current_attrs.background =
                                    TerminalColor::Palette(params[i + 2] as u8);
                                i += 2;
                            }
                            2 if i + 4 < params.len() => {
                                self.current_attrs.background = TerminalColor::Rgb(
                                    params[i + 2] as u8,
                                    params[i + 3] as u8,
                                    params[i + 4] as u8,
                                );
                                i += 4;
                            }
                            _ => {}
                        }
                    }
                }
                49 => self.current_attrs.background = TerminalColor::Default,
                90..=97 => {
                    self.current_attrs.foreground = TerminalColor::Ansi((params[i] - 90 + 8) as u8);
                }
                100..=107 => {
                    self.current_attrs.background =
                        TerminalColor::Ansi((params[i] - 100 + 8) as u8);
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// Places a character at the current cursor position and advances cursor.
    fn put_char(&mut self, ch: char) {
        if self.cursor.col >= self.grid.cols() {
            // Line wrap
            self.cursor.col = 0;
            self.line_feed();
        }

        let cell = Cell::with_attrs(ch, self.current_attrs);
        self.grid.set(self.cursor.row, self.cursor.col, cell);
        self.cursor.col += 1;
    }

    /// Performs a line feed (moves cursor down, scrolls if at bottom).
    fn line_feed(&mut self) {
        if self.cursor.row >= self.grid.rows() - 1 {
            // At bottom — scroll up
            let removed_row = self.grid.scroll_up();
            self.scrollback.push_back(removed_row);

            // Trim scrollback if over limit
            while self.scrollback.len() > self.max_scrollback {
                self.scrollback.pop_front();
            }
        } else {
            self.cursor.row += 1;
        }
    }

    /// Erase display (ED).
    fn erase_display(&mut self, mode: u16) {
        match mode {
            0 => {
                // Erase from cursor to end
                for col in self.cursor.col..self.grid.cols() {
                    self.grid.set(self.cursor.row, col, Cell::default());
                }
                for row in (self.cursor.row + 1)..self.grid.rows() {
                    self.grid.clear_row(row);
                }
            }
            1 => {
                // Erase from start to cursor
                for row in 0..self.cursor.row {
                    self.grid.clear_row(row);
                }
                for col in 0..=self.cursor.col {
                    self.grid.set(self.cursor.row, col, Cell::default());
                }
            }
            2 | 3 => {
                // Erase entire display
                self.grid.clear();
            }
            _ => {}
        }
    }

    /// Erase line (EL).
    fn erase_line(&mut self, mode: u16) {
        match mode {
            0 => {
                // Erase from cursor to end of line
                for col in self.cursor.col..self.grid.cols() {
                    self.grid.set(self.cursor.row, col, Cell::default());
                }
            }
            1 => {
                // Erase from start of line to cursor
                for col in 0..=self.cursor.col {
                    self.grid.set(self.cursor.row, col, Cell::default());
                }
            }
            2 => {
                // Erase entire line
                self.grid.clear_row(self.cursor.row);
            }
            _ => {}
        }
    }

    /// Resets the emulator to initial state.
    fn reset(&mut self) {
        let cols = self.grid.cols();
        let rows = self.grid.rows();
        self.grid = TerminalGrid::new(cols, rows);
        self.cursor = CursorState {
            row: 0,
            col: 0,
            visible: true,
        };
        self.current_attrs = CellAttributes::default();
        self.saved_cursor = None;
        self.title = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 7.8
    #[test]
    fn new_emulator_has_correct_dimensions() {
        let emu = TerminalEmulator::new(80, 24, 1000);
        assert_eq!(emu.grid().cols(), 80);
        assert_eq!(emu.grid().rows(), 24);
    }

    // Validates: Requirement 7.8
    #[test]
    fn feed_plain_text_updates_grid() {
        let mut emu = TerminalEmulator::new(80, 24, 1000);
        emu.feed(b"Hello");
        assert_eq!(emu.grid().get(0, 0).unwrap().character, 'H');
        assert_eq!(emu.grid().get(0, 1).unwrap().character, 'e');
        assert_eq!(emu.grid().get(0, 4).unwrap().character, 'o');
        assert_eq!(emu.cursor().col, 5);
    }

    // Validates: Requirement 7.8
    #[test]
    fn feed_newline_advances_row() {
        let mut emu = TerminalEmulator::new(80, 24, 1000);
        emu.feed(b"line1\r\nline2");
        assert_eq!(emu.grid().get(0, 0).unwrap().character, 'l');
        assert_eq!(emu.grid().get(1, 0).unwrap().character, 'l');
        assert_eq!(emu.cursor().row, 1);
    }

    // Validates: Requirement 7.8
    #[test]
    fn carriage_return_resets_column() {
        let mut emu = TerminalEmulator::new(80, 24, 1000);
        emu.feed(b"hello\rworld");
        // "world" overwrites "hello"
        assert_eq!(emu.grid().get(0, 0).unwrap().character, 'w');
        assert_eq!(emu.grid().get(0, 4).unwrap().character, 'd');
    }

    // Validates: Requirement 7.8
    #[test]
    fn csi_cursor_position() {
        let mut emu = TerminalEmulator::new(80, 24, 1000);
        // ESC[5;10H — move to row 5, col 10 (1-indexed)
        emu.feed(b"\x1b[5;10H");
        assert_eq!(emu.cursor().row, 4); // 0-indexed
        assert_eq!(emu.cursor().col, 9); // 0-indexed
    }

    // Validates: Requirement 7.8
    #[test]
    fn csi_erase_display() {
        let mut emu = TerminalEmulator::new(80, 24, 1000);
        emu.feed(b"Hello");
        emu.feed(b"\x1b[2J"); // Clear screen
        assert_eq!(emu.grid().get(0, 0).unwrap().character, ' ');
    }

    // Validates: Requirement 7.8
    #[test]
    fn sgr_bold_attribute() {
        let mut emu = TerminalEmulator::new(80, 24, 1000);
        emu.feed(b"\x1b[1m"); // Bold on
        emu.feed(b"X");
        assert!(emu.grid().get(0, 0).unwrap().attrs.bold);
    }

    // Validates: Requirement 7.8
    #[test]
    fn sgr_foreground_color() {
        let mut emu = TerminalEmulator::new(80, 24, 1000);
        emu.feed(b"\x1b[31m"); // Red foreground
        emu.feed(b"R");
        assert_eq!(
            emu.grid().get(0, 0).unwrap().attrs.foreground,
            TerminalColor::Ansi(1)
        );
    }

    // Validates: Requirement 7.8
    #[test]
    fn sgr_reset() {
        let mut emu = TerminalEmulator::new(80, 24, 1000);
        emu.feed(b"\x1b[1;31m"); // Bold + red
        emu.feed(b"\x1b[0m"); // Reset
        emu.feed(b"N");
        let cell = emu.grid().get(0, 0).unwrap();
        assert!(!cell.attrs.bold);
        assert_eq!(cell.attrs.foreground, TerminalColor::Default);
    }

    // Validates: Requirement 7.8 (Property 11 — dimensions invariant)
    #[test]
    fn grid_dimensions_invariant_after_feed() {
        let mut emu = TerminalEmulator::new(80, 24, 1000);
        // Feed a large amount of data
        for _ in 0..100 {
            emu.feed(b"This is a long line that might wrap around the terminal display.\n");
        }
        assert_eq!(emu.grid().cols(), 80);
        assert_eq!(emu.grid().rows(), 24);
    }

    // Validates: Requirement 7.8
    #[test]
    fn scrollback_captures_scrolled_lines() {
        let mut emu = TerminalEmulator::new(80, 3, 100);
        emu.feed(b"line1\nline2\nline3\nline4");
        // Grid has 3 rows, so line1 should be in scrollback
        assert_eq!(emu.scrollback().len(), 1);
        assert_eq!(emu.scrollback()[0][0].character, 'l');
    }

    // Validates: Requirement 7.8
    #[test]
    fn resize_changes_grid_dimensions() {
        let mut emu = TerminalEmulator::new(80, 24, 1000);
        emu.resize(40, 12);
        assert_eq!(emu.grid().cols(), 40);
        assert_eq!(emu.grid().rows(), 12);
    }
}
