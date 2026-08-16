//! Context menu registry and types.
//!
//! Context menus are popup menus triggered by right-click or context-menu key.
//! Each context type (editor area, tab header, etc.) has its own menu definition.

use crate::menu_model::Menu;

/// The type of UI element that a context menu is associated with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ContextType {
    /// Right-click in the editor text area.
    EditorArea,
    /// Right-click on a tab header.
    TabHeader,
    /// Right-click on a panel header.
    PanelHeader,
    /// Right-click on a file tree node.
    FileTreeNode,
}

/// Registry for context-specific popup menus.
///
/// Stores menu definitions per `ContextType` and supports plugin contributions.
#[derive(Debug, Clone)]
pub struct ContextMenuRegistry {
    /// Editor area context menu items.
    editor_menu: Menu,
    /// Tab header context menu items.
    tab_menu: Menu,
    /// Panel header context menu items.
    panel_menu: Menu,
    /// File tree node context menu items.
    file_tree_menu: Menu,
}

impl ContextMenuRegistry {
    /// Creates a new registry with empty context menus.
    pub fn new() -> Self {
        Self {
            editor_menu: Menu::new("Editor Context", None),
            tab_menu: Menu::new("Tab Context", None),
            panel_menu: Menu::new("Panel Context", None),
            file_tree_menu: Menu::new("File Tree Context", None),
        }
    }

    /// Returns the menu for the given context type.
    pub fn get_menu(&self, context_type: ContextType) -> &Menu {
        match context_type {
            ContextType::EditorArea => &self.editor_menu,
            ContextType::TabHeader => &self.tab_menu,
            ContextType::PanelHeader => &self.panel_menu,
            ContextType::FileTreeNode => &self.file_tree_menu,
        }
    }

    /// Returns a mutable reference to the menu for the given context type.
    pub fn get_menu_mut(&mut self, context_type: ContextType) -> &mut Menu {
        match context_type {
            ContextType::EditorArea => &mut self.editor_menu,
            ContextType::TabHeader => &mut self.tab_menu,
            ContextType::PanelHeader => &mut self.panel_menu,
            ContextType::FileTreeNode => &mut self.file_tree_menu,
        }
    }

    /// Sets the menu for a specific context type.
    pub fn set_menu(&mut self, context_type: ContextType, menu: Menu) {
        match context_type {
            ContextType::EditorArea => self.editor_menu = menu,
            ContextType::TabHeader => self.tab_menu = menu,
            ContextType::PanelHeader => self.panel_menu = menu,
            ContextType::FileTreeNode => self.file_tree_menu = menu,
        }
    }
}

impl Default for ContextMenuRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_item::MenuItem;

    #[test]
    fn new_registry_has_empty_menus() {
        let registry = ContextMenuRegistry::new();
        assert_eq!(registry.get_menu(ContextType::EditorArea).items.len(), 0);
        assert_eq!(registry.get_menu(ContextType::TabHeader).items.len(), 0);
    }

    #[test]
    fn set_menu_replaces_context_menu() {
        let mut registry = ContextMenuRegistry::new();
        let menu = Menu::new("Editor", None).with_item(MenuItem::new("cut", "Cut", "edit.cut"));

        registry.set_menu(ContextType::EditorArea, menu);
        assert_eq!(registry.get_menu(ContextType::EditorArea).items.len(), 1);
    }
}
