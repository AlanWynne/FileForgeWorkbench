//! Token types for the IDCAMS lexer.

/// A token produced by the IDCAMS lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A command verb (DEFINE, DELETE, etc.).
    Verb(Verb),
    /// A parameter keyword (NAME, KEYS, RECORDSIZE, etc.).
    Keyword(String),
    /// An opening parenthesis.
    OpenParen,
    /// A closing parenthesis.
    CloseParen,
    /// A numeric literal.
    Number(i64),
    /// A string literal (dataset names, values).
    StringLit(String),
    /// A semicolon command separator.
    Semicolon,
    /// A hyphen (continuation at end of line).
    Hyphen,
    /// A comment (block or line).
    Comment(String),
    /// A wildcard character (*).
    Wildcard,
    /// A comparison operator.
    CompareOp(CmpOp),
    /// A logical operator.
    LogicalOp(LogOp),
    /// End of input.
    Eof,
}

/// IDCAMS command verbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Verb {
    /// DEFINE command.
    Define,
    /// DELETE command.
    Delete,
    /// ALTER command.
    Alter,
    /// LISTCAT command.
    Listcat,
    /// PRINT command.
    Print,
    /// REPRO command.
    Repro,
    /// VERIFY command.
    Verify,
    /// EXPORT command.
    Export,
    /// IMPORT command.
    Import,
    /// BLDINDEX command.
    Bldindex,
    /// SET command.
    Set,
    /// IF command.
    If,
}

impl Verb {
    /// Tries to parse a verb from a string (case-insensitive).
    pub fn from_str_ci(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "DEFINE" => Some(Self::Define),
            "DELETE" => Some(Self::Delete),
            "ALTER" => Some(Self::Alter),
            "LISTCAT" | "LISTC" => Some(Self::Listcat),
            "PRINT" => Some(Self::Print),
            "REPRO" => Some(Self::Repro),
            "VERIFY" => Some(Self::Verify),
            "EXPORT" => Some(Self::Export),
            "IMPORT" => Some(Self::Import),
            "BLDINDEX" => Some(Self::Bldindex),
            "SET" => Some(Self::Set),
            "IF" => Some(Self::If),
            _ => None,
        }
    }
}

/// Comparison operators for IF conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    /// Equal.
    Eq,
    /// Not equal.
    Ne,
    /// Greater than.
    Gt,
    /// Less than.
    Lt,
    /// Greater than or equal.
    Ge,
    /// Less than or equal.
    Le,
}

impl CmpOp {
    /// Tries to parse a comparison operator from a string (case-insensitive).
    pub fn from_str_ci(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "EQ" | "=" => Some(Self::Eq),
            "NE" | "\\=" | "^=" => Some(Self::Ne),
            "GT" | ">" => Some(Self::Gt),
            "LT" | "<" => Some(Self::Lt),
            "GE" | ">=" => Some(Self::Ge),
            "LE" | "<=" => Some(Self::Le),
            _ => None,
        }
    }

    /// Evaluates this comparison on two values.
    pub fn evaluate(&self, left: u8, right: u8) -> bool {
        match self {
            Self::Eq => left == right,
            Self::Ne => left != right,
            Self::Gt => left > right,
            Self::Lt => left < right,
            Self::Ge => left >= right,
            Self::Le => left <= right,
        }
    }
}

/// Logical operators for compound conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogOp {
    /// Logical AND.
    And,
    /// Logical OR.
    Or,
}

impl LogOp {
    /// Tries to parse a logical operator from a string (case-insensitive).
    pub fn from_str_ci(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "AND" | "&&" => Some(Self::And),
            "OR" | "||" => Some(Self::Or),
            _ => None,
        }
    }
}
