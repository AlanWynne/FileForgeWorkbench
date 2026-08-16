//! Plugin extensibility — menu and status bar contributions.
//!
//! Plugins can contribute menu items to existing menus or create new
//! top-level menus. This module defines the contribution descriptors
//! and the registry that manages plugin lifecycle.

use crate::error::MenuError;
use crate::menu_bar::MenuBar;
use crate::menu_item::MenuItem;
use crate::menu_model::{Menu, MenuEntry};

/// Where to insert a contributed menu item within the target menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuInsertPosition {
    /// Insert at the end of the menu.
    End,
    /// Insert before a specific item ID.
    Before(String),
    /// Insert after a specific item ID.
    After(String),
}

/// Descriptor for a plugin-contributed menu item.
///
/// Specifies where the item should be inserted and what command it binds to.
#[derive(Debug, Clone)]
pub struct MenuContribution {
    /// The target menu path (e.g., "File", "Tools", "View").
    pub menu_path: String,
    /// The Command_ID to bind this menu item to.
    pub command_id: String,
    /// Display label for the menu item.
    pub label: String,
    /// Desired position within the target menu.
    pub position: MenuInsertPosition,
    /// Whether to insert a separator before this item.
    pub separator_before: bool,
    /// Whether to insert a separator after this item.
    pub separator_after: bool,
    /// The plugin that contributed this item.
    pub plugin_name: String,
}

/// Registry for plugin-contributed menu items and submenus.
///
/// Tracks all contributions and applies them to the menu bar model.
/// When a plugin is unloaded, its contributions are removed and empty
/// menus are collapsed.
#[derive(Debug, Clone)]
pub struct MenuContributionRegistry {
    /// All registered contributions.
    contributions: Vec<MenuContribution>,
}

impl MenuContributionRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            contributions: Vec::new(),
        }
    }

    /// Registers a plugin menu contribution.
    pub fn register(&mut self, contribution: MenuContribution) -> Result<(), MenuError> {
        self.contributions.push(contribution);
        Ok(())
    }

    /// Removes all contributions from a specific plugin.
    pub fn remove_plugin(&mut self, plugin_name: &str) {
        self.contributions.retain(|c| c.plugin_name != plugin_name);
    }

    /// Returns all contributions from a specific plugin.
    pub fn contributions_for(&self, plugin_name: &str) -> Vec<&MenuContribution> {
        self.contributions
            .iter()
            .filter(|c| c.plugin_name == plugin_name)
            .collect()
    }

    /// Returns the total number of registered contributions.
    pub fn contribution_count(&self) -> usize {
        self.contributions.len()
    }

    /// Applies all registered contributions to a menu bar model.
    ///
    /// Creates new top-level menus as needed (inserted before Help).
    /// Items are inserted at the specified position within the target menu.
    pub fn apply_to(&self, menu_bar: &mut MenuBar) {
        for contribution in &self.contributions {
            self.apply_contribution(menu_bar, contribution);
        }

        // Remove empty top-level menus that were created by plugins
        // but whose contributions have all been removed
        menu_bar.menus.retain(|menu| {
            let has_items = menu
                .items
                .iter()
                .any(|entry| matches!(entry, MenuEntry::Item(_) | MenuEntry::Submenu(_)));
            has_items
        });
    }

    /// Applies a single contribution to the menu bar.
    fn apply_contribution(&self, menu_bar: &mut MenuBar, contribution: &MenuContribution) {
        // Find or create the target menu
        let menu_index = menu_bar
            .menus
            .iter()
            .position(|m| m.label == contribution.menu_path);

        let menu_index = match menu_index {
            Some(idx) => idx,
            None => {
                // Create new top-level menu before Help (last menu)
                let insert_pos = if menu_bar.menus.is_empty() {
                    0
                } else {
                    // Insert before the last menu (Help)
                    menu_bar.menus.len().saturating_sub(1)
                };
                let new_menu = Menu::new(&contribution.menu_path, None);
                menu_bar.menus.insert(insert_pos, new_menu);
                insert_pos
            }
        };

        let menu = &mut menu_bar.menus[menu_index];
        let item = MenuItem::new(
            format!(
                "plugin_{}_{}",
                contribution.plugin_name, contribution.command_id
            ),
            &contribution.label,
            &contribution.command_id,
        )
        .with_plugin(&contribution.plugin_name);

        // Build entries to insert
        let mut new_entries = Vec::new();
        if contribution.separator_before {
            new_entries.push(MenuEntry::Separator);
        }
        new_entries.push(MenuEntry::Item(item));
        if contribution.separator_after {
            new_entries.push(MenuEntry::Separator);
        }

        // Determine insertion position
        match &contribution.position {
            MenuInsertPosition::End => {
                menu.items.extend(new_entries);
            }
            MenuInsertPosition::Before(ref_id) => {
                if let Some(pos) = find_item_position(&menu.items, ref_id) {
                    for (i, entry) in new_entries.into_iter().enumerate() {
                        menu.items.insert(pos + i, entry);
                    }
                } else {
                    menu.items.extend(new_entries);
                }
            }
            MenuInsertPosition::After(ref_id) => {
                if let Some(pos) = find_item_position(&menu.items, ref_id) {
                    let insert_at = pos + 1;
                    for (i, entry) in new_entries.into_iter().enumerate() {
                        menu.items.insert(insert_at + i, entry);
                    }
                } else {
                    menu.items.extend(new_entries);
                }
            }
        }
    }
}

impl Default for MenuContributionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Finds the position of a menu item by ID within a flat list of entries.
fn find_item_position(entries: &[MenuEntry], item_id: &str) -> Option<usize> {
    entries
        .iter()
        .position(|entry| matches!(entry, MenuEntry::Item(item) if item.id == item_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_contribution(plugin: &str, menu: &str, cmd: &str, label: &str) -> MenuContribution {
        MenuContribution {
            menu_path: menu.to_string(),
            command_id: cmd.to_string(),
            label: label.to_string(),
            position: MenuInsertPosition::End,
            separator_before: false,
            separator_after: false,
            plugin_name: plugin.to_string(),
        }
    }

    #[test]
    fn new_registry_is_empty() {
        let registry = MenuContributionRegistry::new();
        assert_eq!(registry.contribution_count(), 0);
    }

    #[test]
    fn register_adds_contribution() {
        let mut registry = MenuContributionRegistry::new();
        registry
            .register(make_contribution(
                "myplugin", "Tools", "tool.run", "Run Tool",
            ))
            .unwrap();
        assert_eq!(registry.contribution_count(), 1);
    }

    #[test]
    fn remove_plugin_clears_contributions() {
        let mut registry = MenuContributionRegistry::new();
        registry
            .register(make_contribution("pluginA", "Tools", "a.cmd1", "A1"))
            .unwrap();
        registry
            .register(make_contribution("pluginA", "Tools", "a.cmd2", "A2"))
            .unwrap();
        registry
            .register(make_contribution("pluginB", "Tools", "b.cmd1", "B1"))
            .unwrap();

        registry.remove_plugin("pluginA");
        assert_eq!(registry.contribution_count(), 1);
        assert_eq!(registry.contributions_for("pluginB").len(), 1);
    }

    #[test]
    fn apply_to_creates_new_top_level_menu_before_help() {
        let mut registry = MenuContributionRegistry::new();
        registry
            .register(make_contribution("myplugin", "Tools", "tool.run", "Run"))
            .unwrap();

        let mut bar = MenuBar::with_menus(vec![
            Menu::new("File", Some('F')).with_item(MenuItem::new("file_new", "New", "file.new")),
            Menu::new("Help", Some('H')).with_item(MenuItem::new(
                "help_about",
                "About",
                "help.about",
            )),
        ]);

        registry.apply_to(&mut bar);

        assert_eq!(bar.menu_count(), 3);
        assert_eq!(bar.menus[0].label, "File");
        assert_eq!(bar.menus[1].label, "Tools");
        assert_eq!(bar.menus[2].label, "Help");
    }

    #[test]
    fn apply_to_removes_empty_menus_after_plugin_unload() {
        let mut registry = MenuContributionRegistry::new();
        registry
            .register(make_contribution("myplugin", "Tools", "tool.run", "Run"))
            .unwrap();

        let mut bar = MenuBar::with_menus(vec![
            Menu::new("File", Some('F')).with_item(MenuItem::new("file_new", "New", "file.new")),
            Menu::new("Help", Some('H')).with_item(MenuItem::new(
                "help_about",
                "About",
                "help.about",
            )),
        ]);

        // Apply contributions to create "Tools" menu
        registry.apply_to(&mut bar);
        assert_eq!(bar.menu_count(), 3);

        // Unload the plugin and re-apply
        registry.remove_plugin("myplugin");
        // Reset the bar and re-apply (simulate refresh)
        let mut bar2 = MenuBar::with_menus(vec![
            Menu::new("File", Some('F')).with_item(MenuItem::new("file_new", "New", "file.new")),
            Menu::new("Tools", None), // empty after unload
            Menu::new("Help", Some('H')).with_item(MenuItem::new(
                "help_about",
                "About",
                "help.about",
            )),
        ]);
        registry.apply_to(&mut bar2);
        // The empty "Tools" menu should be removed
        assert_eq!(bar2.menu_count(), 2);
    }
}
