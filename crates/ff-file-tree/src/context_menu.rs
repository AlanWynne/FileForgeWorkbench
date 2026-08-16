//! ContextMenuBuilder — node-type-aware context menu construction.

use crate::node::NodeType;

/// Actions available in context menus, mapped to command IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContextAction {
    Open,
    OpenWith,
    Rename,
    Delete,
    NewFile,
    NewFolder,
    CopyPath,
    CopyDsn,
    RevealInExplorer,
    Refresh,
    ExpandCollapse,
    NewMember,
    Properties,
    Unmount,
    AddRootFolder,
    RefreshAll,
    ShowAll,
}

impl ContextAction {
    /// Returns the command ID string for this action.
    pub fn command_id(&self) -> &'static str {
        match self {
            ContextAction::Open => "file_tree.open",
            ContextAction::OpenWith => "file_tree.open_with",
            ContextAction::Rename => "file_tree.rename",
            ContextAction::Delete => "file_tree.delete",
            ContextAction::NewFile => "file_tree.new_file",
            ContextAction::NewFolder => "file_tree.new_folder",
            ContextAction::CopyPath => "file_tree.copy_path",
            ContextAction::CopyDsn => "file_tree.copy_dsn",
            ContextAction::RevealInExplorer => "file_tree.reveal_in_explorer",
            ContextAction::Refresh => "file_tree.refresh",
            ContextAction::ExpandCollapse => "file_tree.expand_collapse",
            ContextAction::NewMember => "file_tree.new_member",
            ContextAction::Properties => "file_tree.properties",
            ContextAction::Unmount => "file_tree.unmount_catalog",
            ContextAction::AddRootFolder => "file_tree.add_root",
            ContextAction::RefreshAll => "file_tree.refresh_all",
            ContextAction::ShowAll => "file_tree.show_all",
        }
    }

    /// Returns the display label for the menu item.
    pub fn label(&self) -> &'static str {
        match self {
            ContextAction::Open => "Open",
            ContextAction::OpenWith => "Open With...",
            ContextAction::Rename => "Rename",
            ContextAction::Delete => "Delete",
            ContextAction::NewFile => "New File",
            ContextAction::NewFolder => "New Folder",
            ContextAction::CopyPath => "Copy Path",
            ContextAction::CopyDsn => "Copy DSN",
            ContextAction::RevealInExplorer => "Reveal in System Explorer",
            ContextAction::Refresh => "Refresh",
            ContextAction::ExpandCollapse => "Expand/Collapse",
            ContextAction::NewMember => "New Member",
            ContextAction::Properties => "Properties",
            ContextAction::Unmount => "Unmount",
            ContextAction::AddRootFolder => "Add Root Folder",
            ContextAction::RefreshAll => "Refresh All",
            ContextAction::ShowAll => "Show All",
        }
    }
}

/// Builds context menu items based on node type and root category.
pub struct ContextMenuBuilder;

impl ContextMenuBuilder {
    /// Build the context menu for a given node type.
    pub fn build(node_type: NodeType) -> Vec<ContextAction> {
        match node_type {
            NodeType::File => vec![
                ContextAction::Open,
                ContextAction::OpenWith,
                ContextAction::Rename,
                ContextAction::Delete,
                ContextAction::NewFile,
                ContextAction::NewFolder,
                ContextAction::CopyPath,
                ContextAction::RevealInExplorer,
            ],
            NodeType::Directory | NodeType::BookmarkedRoot => vec![
                ContextAction::ExpandCollapse,
                ContextAction::NewFile,
                ContextAction::NewFolder,
                ContextAction::Rename,
                ContextAction::Delete,
                ContextAction::CopyPath,
                ContextAction::RevealInExplorer,
                ContextAction::Refresh,
            ],
            NodeType::DatasetSequential => vec![
                ContextAction::Open,
                ContextAction::Rename,
                ContextAction::Delete,
                ContextAction::Properties,
                ContextAction::CopyDsn,
            ],
            NodeType::DatasetPartitioned => vec![
                ContextAction::ExpandCollapse,
                ContextAction::NewMember,
                ContextAction::Rename,
                ContextAction::Delete,
                ContextAction::Properties,
                ContextAction::CopyDsn,
            ],
            NodeType::PdsMember => vec![
                ContextAction::Open,
                ContextAction::Rename,
                ContextAction::Delete,
                ContextAction::CopyDsn,
            ],
            NodeType::CatalogRoot => vec![
                ContextAction::Unmount,
                ContextAction::Refresh,
                ContextAction::Properties,
            ],
            NodeType::RootCategory => vec![ContextAction::AddRootFolder, ContextAction::RefreshAll],
            NodeType::SymbolicLink => vec![
                ContextAction::Open,
                ContextAction::CopyPath,
                ContextAction::RevealInExplorer,
            ],
            NodeType::GdgBase | NodeType::GdgGeneration => vec![
                ContextAction::Open,
                ContextAction::Properties,
                ContextAction::CopyDsn,
            ],
            NodeType::OverflowIndicator => vec![ContextAction::ShowAll],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_node_menu_has_required_actions() {
        // Validates: Requirement 6.1 — file node context menu
        let menu = ContextMenuBuilder::build(NodeType::File);
        assert!(menu.contains(&ContextAction::Open));
        assert!(menu.contains(&ContextAction::OpenWith));
        assert!(menu.contains(&ContextAction::Rename));
        assert!(menu.contains(&ContextAction::Delete));
        assert!(menu.contains(&ContextAction::NewFile));
        assert!(menu.contains(&ContextAction::NewFolder));
        assert!(menu.contains(&ContextAction::CopyPath));
        assert!(menu.contains(&ContextAction::RevealInExplorer));
    }

    #[test]
    fn directory_node_menu_has_required_actions() {
        // Validates: Requirement 6.2 — directory node context menu
        let menu = ContextMenuBuilder::build(NodeType::Directory);
        assert!(menu.contains(&ContextAction::ExpandCollapse));
        assert!(menu.contains(&ContextAction::NewFile));
        assert!(menu.contains(&ContextAction::NewFolder));
        assert!(menu.contains(&ContextAction::Rename));
        assert!(menu.contains(&ContextAction::Delete));
        assert!(menu.contains(&ContextAction::CopyPath));
        assert!(menu.contains(&ContextAction::RevealInExplorer));
        assert!(menu.contains(&ContextAction::Refresh));
    }

    #[test]
    fn dataset_sequential_menu_has_required_actions() {
        // Validates: Requirement 6.3 — dataset node context menu
        let menu = ContextMenuBuilder::build(NodeType::DatasetSequential);
        assert!(menu.contains(&ContextAction::Open));
        assert!(menu.contains(&ContextAction::Rename));
        assert!(menu.contains(&ContextAction::Delete));
        assert!(menu.contains(&ContextAction::Properties));
        assert!(menu.contains(&ContextAction::CopyDsn));
    }

    #[test]
    fn pds_dataset_menu_has_new_member() {
        // Validates: Requirement 6.4 — PDS dataset context menu
        let menu = ContextMenuBuilder::build(NodeType::DatasetPartitioned);
        assert!(menu.contains(&ContextAction::ExpandCollapse));
        assert!(menu.contains(&ContextAction::NewMember));
        assert!(menu.contains(&ContextAction::Rename));
        assert!(menu.contains(&ContextAction::Delete));
        assert!(menu.contains(&ContextAction::Properties));
        assert!(menu.contains(&ContextAction::CopyDsn));
    }

    #[test]
    fn pds_member_menu_has_required_actions() {
        // Validates: Requirement 6.5 — PDS member context menu
        let menu = ContextMenuBuilder::build(NodeType::PdsMember);
        assert!(menu.contains(&ContextAction::Open));
        assert!(menu.contains(&ContextAction::Rename));
        assert!(menu.contains(&ContextAction::Delete));
        assert!(menu.contains(&ContextAction::CopyDsn));
    }

    #[test]
    fn catalog_root_menu_has_unmount() {
        // Validates: Requirement 6.6 — catalog root context menu
        let menu = ContextMenuBuilder::build(NodeType::CatalogRoot);
        assert!(menu.contains(&ContextAction::Unmount));
        assert!(menu.contains(&ContextAction::Refresh));
        assert!(menu.contains(&ContextAction::Properties));
    }

    #[test]
    fn root_category_menu_has_add_root_folder() {
        // Validates: Requirement 6.7 — Local Files section header menu
        let menu = ContextMenuBuilder::build(NodeType::RootCategory);
        assert!(menu.contains(&ContextAction::AddRootFolder));
        assert!(menu.contains(&ContextAction::RefreshAll));
    }

    #[test]
    fn all_actions_have_command_ids() {
        // Validates: Requirement 6.8 — all actions dispatched as commands
        let all_actions = [
            ContextAction::Open,
            ContextAction::OpenWith,
            ContextAction::Rename,
            ContextAction::Delete,
            ContextAction::NewFile,
            ContextAction::NewFolder,
            ContextAction::CopyPath,
            ContextAction::CopyDsn,
            ContextAction::RevealInExplorer,
            ContextAction::Refresh,
            ContextAction::ExpandCollapse,
            ContextAction::NewMember,
            ContextAction::Properties,
            ContextAction::Unmount,
            ContextAction::AddRootFolder,
            ContextAction::RefreshAll,
            ContextAction::ShowAll,
        ];
        for action in &all_actions {
            assert!(!action.command_id().is_empty());
            assert!(action.command_id().starts_with("file_tree."));
        }
    }

    #[test]
    fn overflow_indicator_menu_has_show_all() {
        // Validates: Requirement 4.8 — overflow indicator Show All action
        let menu = ContextMenuBuilder::build(NodeType::OverflowIndicator);
        assert!(menu.contains(&ContextAction::ShowAll));
    }

    #[test]
    fn placeholder_node_has_empty_menu() {
        // Validates: Requirement 2.5 — placeholder nodes are non-interactive
        let menu = ContextMenuBuilder::build(NodeType::Placeholder);
        assert!(menu.is_empty());
    }
}
