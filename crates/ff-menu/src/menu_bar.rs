//! Menu bar model and operations.
//!
//! The `MenuBar` is the top-level container for all menus. It holds
//! the ordered list of top-level menus and the keyboard navigation state.

use crate::keyboard_nav::MenuNavState;
use crate::menu_model::Menu;

/// The complete menu bar model. A list of top-level menus rendered left-to-right.
///
/// The menu bar delegates all command activation to the `ff-command` dispatcher
/// and never directly mutates application state.
#[derive(Debug, Clone)]
pub struct MenuBar {
    /// Ordered list of top-level menus.
    pub menus: Vec<Menu>,
    /// Current keyboard navigation state.
    pub nav_state: MenuNavState,
}

impl MenuBar {
    /// Creates a new empty menu bar.
    pub fn new() -> Self {
        Self {
            menus: Vec::new(),
            nav_state: MenuNavState::Inactive,
        }
    }

    /// Creates a menu bar from a list of menus.
    pub fn with_menus(menus: Vec<Menu>) -> Self {
        Self {
            menus,
            nav_state: MenuNavState::Inactive,
        }
    }

    /// Adds a top-level menu to the menu bar.
    pub fn add_menu(&mut self, menu: Menu) {
        self.menus.push(menu);
    }

    /// Returns the number of top-level menus.
    pub fn menu_count(&self) -> usize {
        self.menus.len()
    }

    /// Returns a reference to the menu at the given index, if it exists.
    pub fn get_menu(&self, index: usize) -> Option<&Menu> {
        self.menus.get(index)
    }

    /// Returns a mutable reference to the menu at the given index, if it exists.
    pub fn get_menu_mut(&mut self, index: usize) -> Option<&mut Menu> {
        self.menus.get_mut(index)
    }

    /// Opens the menu at the given index, closing any other open menu.
    pub fn open_menu(&mut self, index: usize) {
        for (i, menu) in self.menus.iter_mut().enumerate() {
            menu.is_open = i == index;
        }
        self.nav_state = MenuNavState::Open {
            menu_index: index,
            item_index: None,
            submenu_stack: Vec::new(),
        };
    }

    /// Closes all open menus and returns to inactive state.
    pub fn close_all(&mut self) {
        for menu in &mut self.menus {
            menu.is_open = false;
        }
        self.nav_state = MenuNavState::Inactive;
    }

    /// Returns the index of the currently open menu, if any.
    pub fn open_menu_index(&self) -> Option<usize> {
        self.menus.iter().position(|m| m.is_open)
    }

    /// Finds a menu item by its ID across all menus and submenus.
    pub fn find_item(&self, item_id: &str) -> Option<&crate::menu_item::MenuItem> {
        for menu in &self.menus {
            if let Some(item) = find_item_in_entries(&menu.items, item_id) {
                return Some(item);
            }
        }
        None
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Recursively searches menu entries for an item with the given ID.
fn find_item_in_entries<'a>(
    entries: &'a [crate::menu_model::MenuEntry],
    item_id: &str,
) -> Option<&'a crate::menu_item::MenuItem> {
    for entry in entries {
        match entry {
            crate::menu_model::MenuEntry::Item(item) if item.id == item_id => {
                return Some(item);
            }
            crate::menu_model::MenuEntry::Submenu(submenu) => {
                if let Some(found) = find_item_in_entries(&submenu.items, item_id) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_item::MenuItem;
    use crate::menu_model::MenuEntry;

    #[test]
    fn new_menu_bar_is_empty_and_inactive() {
        let bar = MenuBar::new();
        assert_eq!(bar.menu_count(), 0);
        assert_eq!(bar.nav_state, MenuNavState::Inactive);
    }

    #[test]
    fn add_menu_increases_count() {
        let mut bar = MenuBar::new();
        bar.add_menu(Menu::new("File", Some('F')));
        bar.add_menu(Menu::new("Edit", Some('E')));
        assert_eq!(bar.menu_count(), 2);
    }

    #[test]
    fn open_menu_sets_correct_state() {
        let mut bar = MenuBar::with_menus(vec![
            Menu::new("File", Some('F')),
            Menu::new("Edit", Some('E')),
        ]);

        bar.open_menu(1);
        assert!(!bar.menus[0].is_open);
        assert!(bar.menus[1].is_open);
        assert_eq!(bar.open_menu_index(), Some(1));
    }

    #[test]
    fn close_all_returns_to_inactive() {
        let mut bar = MenuBar::with_menus(vec![Menu::new("File", Some('F'))]);
        bar.open_menu(0);
        bar.close_all();

        assert!(!bar.menus[0].is_open);
        assert_eq!(bar.nav_state, MenuNavState::Inactive);
    }

    #[test]
    fn find_item_locates_item_in_submenu() {
        let submenu = Menu::new("Recent", None).with_item(MenuItem::new(
            "recent_file1",
            "file1.txt",
            "file.open_recent",
        ));

        let file_menu = Menu {
            label: "File".to_string(),
            access_key: Some('F'),
            items: vec![
                MenuEntry::Item(MenuItem::new("file_new", "New", "file.new")),
                MenuEntry::Submenu(submenu),
            ],
            is_open: false,
        };

        let bar = MenuBar::with_menus(vec![file_menu]);

        assert!(bar.find_item("file_new").is_some());
        assert!(bar.find_item("recent_file1").is_some());
        assert!(bar.find_item("nonexistent").is_none());
    }
}
