//! `KeyChord` and related types — modifier keys and key codes.

use std::fmt;

/// Modifier keys for keyboard shortcuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Modifiers {
    /// Ctrl key pressed.
    pub ctrl: bool,
    /// Alt key pressed.
    pub alt: bool,
    /// Shift key pressed.
    pub shift: bool,
    /// Super/Win/Cmd key pressed.
    pub super_key: bool,
}

impl Modifiers {
    /// No modifiers pressed.
    pub fn none() -> Self {
        Self::default()
    }

    /// Ctrl modifier only.
    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Self::default()
        }
    }

    /// Ctrl+Shift modifiers.
    pub fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            shift: true,
            ..Self::default()
        }
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ctrl {
            f.write_str("Ctrl+")?;
        }
        if self.alt {
            f.write_str("Alt+")?;
        }
        if self.shift {
            f.write_str("Shift+")?;
        }
        if self.super_key {
            f.write_str("Super+")?;
        }
        Ok(())
    }
}

/// A single keyboard chord: modifiers plus a primary key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyChord {
    /// The modifier keys.
    pub modifiers: Modifiers,
    /// The primary key.
    pub key: KeyCode,
}

impl KeyChord {
    /// Creates a new key chord with the given modifiers and key.
    pub fn new(modifiers: Modifiers, key: KeyCode) -> Self {
        Self { modifiers, key }
    }
}

impl fmt::Display for KeyChord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.modifiers, self.key)
    }
}

/// Platform-independent key codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum KeyCode {
    // Letters
    /// A key.
    A,
    /// B key.
    B,
    /// C key.
    C,
    /// D key.
    D,
    /// E key.
    E,
    /// F key.
    F,
    /// G key.
    G,
    /// H key.
    H,
    /// I key.
    I,
    /// J key.
    J,
    /// K key.
    K,
    /// L key.
    L,
    /// M key.
    M,
    /// N key.
    N,
    /// O key.
    O,
    /// P key.
    P,
    /// Q key.
    Q,
    /// R key.
    R,
    /// S key.
    S,
    /// T key.
    T,
    /// U key.
    U,
    /// V key.
    V,
    /// W key.
    W,
    /// X key.
    X,
    /// Y key.
    Y,
    /// Z key.
    Z,
    // Numbers
    /// 0 key.
    Key0,
    /// 1 key.
    Key1,
    /// 2 key.
    Key2,
    /// 3 key.
    Key3,
    /// 4 key.
    Key4,
    /// 5 key.
    Key5,
    /// 6 key.
    Key6,
    /// 7 key.
    Key7,
    /// 8 key.
    Key8,
    /// 9 key.
    Key9,
    // Function keys
    /// F1 key.
    F1,
    /// F2 key.
    F2,
    /// F3 key.
    F3,
    /// F4 key.
    F4,
    /// F5 key.
    F5,
    /// F6 key.
    F6,
    /// F7 key.
    F7,
    /// F8 key.
    F8,
    /// F9 key.
    F9,
    /// F10 key.
    F10,
    /// F11 key.
    F11,
    /// F12 key.
    F12,
    /// F13 key.
    F13,
    /// F14 key.
    F14,
    /// F15 key.
    F15,
    /// F16 key.
    F16,
    /// F17 key.
    F17,
    /// F18 key.
    F18,
    /// F19 key.
    F19,
    /// F20 key.
    F20,
    /// F21 key.
    F21,
    /// F22 key.
    F22,
    /// F23 key.
    F23,
    /// F24 key.
    F24,
    // Special keys
    /// Tab key.
    Tab,
    /// Space key.
    Space,
    /// Enter/Return key.
    Enter,
    /// Escape key.
    Escape,
    /// Backspace key.
    Backspace,
    /// Delete key.
    Delete,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page Up key.
    PageUp,
    /// Page Down key.
    PageDown,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Plus/equals key.
    Plus,
    /// Minus/hyphen key.
    Minus,
    // Punctuation
    /// Comma key.
    Comma,
    /// Period key.
    Period,
    /// Semicolon key.
    Semicolon,
    /// Slash key.
    Slash,
    /// Backslash key.
    Backslash,
    /// Left bracket key.
    LeftBracket,
    /// Right bracket key.
    RightBracket,
    /// Grave/backtick key.
    Grave,
    /// Equals key.
    Equals,
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::H => "H",
            Self::I => "I",
            Self::J => "J",
            Self::K => "K",
            Self::L => "L",
            Self::M => "M",
            Self::N => "N",
            Self::O => "O",
            Self::P => "P",
            Self::Q => "Q",
            Self::R => "R",
            Self::S => "S",
            Self::T => "T",
            Self::U => "U",
            Self::V => "V",
            Self::W => "W",
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
            Self::Key0 => "0",
            Self::Key1 => "1",
            Self::Key2 => "2",
            Self::Key3 => "3",
            Self::Key4 => "4",
            Self::Key5 => "5",
            Self::Key6 => "6",
            Self::Key7 => "7",
            Self::Key8 => "8",
            Self::Key9 => "9",
            Self::F1 => "F1",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::F7 => "F7",
            Self::F8 => "F8",
            Self::F9 => "F9",
            Self::F10 => "F10",
            Self::F11 => "F11",
            Self::F12 => "F12",
            Self::F13 => "F13",
            Self::F14 => "F14",
            Self::F15 => "F15",
            Self::F16 => "F16",
            Self::F17 => "F17",
            Self::F18 => "F18",
            Self::F19 => "F19",
            Self::F20 => "F20",
            Self::F21 => "F21",
            Self::F22 => "F22",
            Self::F23 => "F23",
            Self::F24 => "F24",
            Self::Tab => "Tab",
            Self::Space => "Space",
            Self::Enter => "Enter",
            Self::Escape => "Escape",
            Self::Backspace => "Backspace",
            Self::Delete => "Delete",
            Self::Home => "Home",
            Self::End => "End",
            Self::PageUp => "PageUp",
            Self::PageDown => "PageDown",
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Plus => "Plus",
            Self::Minus => "Minus",
            Self::Comma => "Comma",
            Self::Period => "Period",
            Self::Semicolon => "Semicolon",
            Self::Slash => "Slash",
            Self::Backslash => "Backslash",
            Self::LeftBracket => "LeftBracket",
            Self::RightBracket => "RightBracket",
            Self::Grave => "Grave",
            Self::Equals => "Equals",
        };
        f.write_str(name)
    }
}
