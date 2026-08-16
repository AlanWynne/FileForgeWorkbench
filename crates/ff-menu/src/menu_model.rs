//! Menu model data structures — Menu, MenuEntry, and related types.
//!
//! These types represent the declarative menu tree structure that is
//! independent of any GUI framework.

use crate::menu_item::MenuItem;

/// A top-level menu or submenu containing ordered items.
///
/// # Examples
///
/// ```
/// use ff_menu::menu_model::{Menu, MenuEntry};
/// use ff_menu::menu_item::MenuItem;
///
/// let file_menu = Menu {
///     label: "File".to_string(),
///     access_key: Some('F'),
///     items: vec![
///         MenuEntry::Item(MenuItem::new("file_new", "New", "file.new")),
///         MenuEntry::Separator,
///         MenuEntry::Item(MenuItem::new("file_exit", "Exit", "workbench.exit")),
///     ],
///     is_open: false,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Menu {
    /// Display label (e.g., "File", "Edit").
    pub label: String,
    /// Access key character (underlined in UI, e.g., 'F' for File).
    pub access_key: Option<char>,
    /// Ordered list of items (menu items, separators, submenus).
    pub items: Vec<MenuEntry>,
    /// Whether this menu is currently open.
    pub is_open: bool,
}

impl Menu {
    /// Creates a new menu with the given label and access key.
    pub fn new(label: impl Into<String>, access_key: Option<char>) -> Self {
        Self {
            label: label.into(),
            access_key,
            items: Vec::new(),
            is_open: false,
        }
    }

    /// Adds an item to this menu, returning self for chaining.
    pub fn with_item(mut self, item: MenuItem) -> Self {
        self.items.push(MenuEntry::Item(item));
        self
    }

    /// Adds a separator to this menu, returning self for chaining.
    pub fn with_separator(mut self) -> Self {
        self.items.push(MenuEntry::Separator);
        self
    }

    /// Adds a submenu to this menu, returning self for chaining.
    pub fn with_submenu(mut self, submenu: Menu) -> Self {
        self.items.push(MenuEntry::Submenu(submenu));
        self
    }

    /// Returns the number of visible items (excluding hidden items).
    pub fn visible_item_count(&self) -> usize {
        self.items
            .iter()
            .filter(|entry| match entry {
                MenuEntry::Item(item) => item.is_visible,
                MenuEntry::Separator => true,
                MenuEntry::Submenu(_) => true,
            })
            .count()
    }
}

/// A single entry within a menu — an item, separator, or submenu.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MenuEntry {
    /// A clickable menu item bound to a command.
    Item(MenuItem),
    /// A visual separator between groups of items.
    Separator,
    /// A nested submenu.
    Submenu(Menu),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_item::MenuItem;

    #[test]
    fn menu_new_creates_empty_menu() {
        let menu = Menu::new("File", Some('F'));
        assert_eq!(menu.label, "File");
        assert_eq!(menu.access_key, Some('F'));
        assert!(menu.items.is_empty());
        assert!(!menu.is_open);
    }

    #[test]
    fn menu_builder_chaining_adds_items_in_order() {
        let menu = Menu::new("Edit", Some('E'))
            .with_item(MenuItem::new("edit_undo", "Undo", "edit.undo"))
            .with_separator()
            .with_item(MenuItem::new("edit_cut", "Cut", "edit.cut"));

        assert_eq!(menu.items.len(), 3);
        assert!(matches!(menu.items[0], MenuEntry::Item(_)));
        assert!(matches!(menu.items[1], MenuEntry::Separator));
        assert!(matches!(menu.items[2], MenuEntry::Item(_)));
    }

    #[test]
    fn visible_item_count_excludes_hidden_items() {
        let mut item = MenuItem::new("hidden", "Hidden", "cmd.hidden");
        item.is_visible = false;

        let menu = Menu::new("Test", None)
            .with_item(MenuItem::new("visible", "Visible", "cmd.visible"))
            .with_item(item)
            .with_separator();

        assert_eq!(menu.visible_item_count(), 2);
    }
}
