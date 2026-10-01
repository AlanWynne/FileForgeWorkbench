//! # Files Panel context-menu item sets
//!
//! Pure functions returning the context-menu item list for a given node kind,
//! for the Mainframe, POSIX and Native catalog sections.
//!
//! Validates: Requirement 6.1-6.4, 8.1, 9.3-9.4

// === Context menu types =====================================================

/// The kind of a Mainframe dataset node in the tree.
///
/// Validates: Requirement 6.1-6.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetNodeKind {
    Ps,
    Pds,
    Member,
    GdgBase,
}

/// Context menu items for a Mainframe dataset node.
///
/// Validates: Requirement 6.1-6.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainframeContextItem {
    Open,
    NewMember,
    NewGeneration,
    Rename,
    Delete,
    DeleteGdg,
    Properties,
    CopyDsn,
    CopyMemberName,
    AllocateLike,
    ListGenerations,
    ModifyLimit,
}

/// Returns the context menu items for a given Mainframe dataset node kind.
///
/// Validates: Requirement 6.1-6.4
pub fn context_menu_items_mainframe(kind: DatasetNodeKind) -> Vec<MainframeContextItem> {
    match kind {
        DatasetNodeKind::Ps => vec![
            MainframeContextItem::Open,
            MainframeContextItem::Rename,
            MainframeContextItem::Delete,
            MainframeContextItem::Properties,
            MainframeContextItem::CopyDsn,
            MainframeContextItem::AllocateLike,
        ],
        DatasetNodeKind::Pds => vec![
            MainframeContextItem::NewMember,
            MainframeContextItem::Rename,
            MainframeContextItem::Delete,
            MainframeContextItem::Properties,
            MainframeContextItem::CopyDsn,
            MainframeContextItem::AllocateLike,
        ],
        DatasetNodeKind::Member => vec![
            MainframeContextItem::Open,
            MainframeContextItem::Rename,
            MainframeContextItem::Delete,
            MainframeContextItem::CopyMemberName,
        ],
        DatasetNodeKind::GdgBase => vec![
            MainframeContextItem::NewGeneration,
            MainframeContextItem::ListGenerations,
            MainframeContextItem::Properties,
            MainframeContextItem::DeleteGdg,
            MainframeContextItem::ModifyLimit,
        ],
    }
}

/// The kind of a POSIX node in the tree.
///
/// Validates: Requirement 8.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosixNodeKind {
    File,
    Directory,
}

/// Context menu items for a POSIX node.
///
/// Validates: Requirement 8.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosixContextItem {
    NewFile,
    NewDirectory,
    Rename,
    Delete,
    Properties,
    CopyPath,
}

/// Returns the context menu items for a POSIX node.
///
/// Validates: Requirement 8.1
pub fn context_menu_items_posix(kind: PosixNodeKind) -> Vec<PosixContextItem> {
    match kind {
        PosixNodeKind::Directory => vec![
            PosixContextItem::NewFile,
            PosixContextItem::NewDirectory,
            PosixContextItem::Rename,
            PosixContextItem::Delete,
            PosixContextItem::Properties,
            PosixContextItem::CopyPath,
        ],
        PosixNodeKind::File => vec![
            PosixContextItem::Rename,
            PosixContextItem::Delete,
            PosixContextItem::Properties,
            PosixContextItem::CopyPath,
        ],
    }
}

/// The kind of a Native catalog node.
///
/// Validates: Requirement 9.3-9.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeNodeKind {
    File,
    Directory,
}

/// Context menu items for a Native catalog node.
///
/// Validates: Requirement 9.3-9.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeContextItem {
    Open,
    Rename,
    Delete,
    CopyPath,
    NewFile,
    NewFolder,
    OpenInNativeFileManager,
    Refresh,
    OpenInCmd,
    OpenInPowerShell,
    OpenInTerminal,
    RevealInFinder,
}

/// Returns the context menu items for a Native catalog node.
///
/// `os` should be `std::env::consts::OS` -- passed in for testability.
///
/// Validates: Requirement 9.3-9.4
pub fn context_menu_items_native(kind: NativeNodeKind, os: &str) -> Vec<NativeContextItem> {
    match kind {
        NativeNodeKind::File => {
            let mut items = vec![
                NativeContextItem::Open,
                NativeContextItem::Rename,
                NativeContextItem::Delete,
                NativeContextItem::CopyPath,
            ];
            match os {
                "windows" => {
                    items.push(NativeContextItem::OpenInCmd);
                    items.push(NativeContextItem::OpenInPowerShell);
                }
                "macos" => {
                    items.push(NativeContextItem::RevealInFinder);
                    items.push(NativeContextItem::OpenInTerminal);
                }
                _ => items.push(NativeContextItem::OpenInTerminal),
            }
            items
        }
        NativeNodeKind::Directory => {
            let mut items = vec![
                NativeContextItem::NewFile,
                NativeContextItem::NewFolder,
                NativeContextItem::Rename,
                NativeContextItem::Delete,
                NativeContextItem::CopyPath,
                NativeContextItem::OpenInNativeFileManager,
                NativeContextItem::Refresh,
            ];
            match os {
                "windows" => {
                    items.push(NativeContextItem::OpenInCmd);
                    items.push(NativeContextItem::OpenInPowerShell);
                }
                "macos" => {
                    items.push(NativeContextItem::RevealInFinder);
                    items.push(NativeContextItem::OpenInTerminal);
                }
                _ => items.push(NativeContextItem::OpenInTerminal),
            }
            items
        }
    }
}
