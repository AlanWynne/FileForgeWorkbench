//! Key Label Bar display model.
//!
//! Provides the data model for the key label bar rendered in the workbench footer.
//! The model derives labels from the active key map and updates when the map changes.
//!
//! Layout: two rows of 12 slots each.
//! Row 0: F1–F12  (indices 0–11)
//! Row 1: F13–F24 (indices 12–23)

use crate::function_key::FunctionKey;
use crate::key_map::KeyMap;

/// A single slot in the Key Label Bar display model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyLabelSlot {
    /// The function key for this slot.
    pub key: FunctionKey,
    /// The display label (derived or explicit). `None` if unassigned.
    pub label: Option<String>,
}

/// The display model for the Key Label Bar.
///
/// Always contains exactly 24 slots (F1–F24) in two rows of 12.
/// Unassigned keys have `label: None` — the slot is still present so the
/// two-row grid layout is preserved.
///
/// Validates: Requirement 13.1, 13.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyLabelBarModel {
    /// Ordered slots for F1–F24 (all 24 keys, including F1).
    slots: Vec<KeyLabelSlot>,
}

impl KeyLabelBarModel {
    /// Build the label bar model from the active key map.
    ///
    /// Creates a slot for every function key F1–F24.
    /// Assigned keys get labels from `KeyBinding::display_label()`;
    /// unassigned keys get `None` (slot still present for grid layout).
    ///
    /// Validates: Requirement 13.1, 13.2
    pub fn from_key_map(key_map: &KeyMap) -> Self {
        let slots = FunctionKey::ALL
            .iter()
            .map(|&key| {
                let label = key_map
                    .get_plain(key)
                    .map(|binding| binding.display_label().to_string());
                KeyLabelSlot { key, label }
            })
            .collect();
        Self { slots }
    }

    /// Get all 24 slots in display order (F1–F24).
    pub fn slots(&self) -> &[KeyLabelSlot] {
        &self.slots
    }

    /// Row 0: F1–F12 (slots 0..12).
    ///
    /// Validates: Requirement 13.1
    pub fn row0(&self) -> &[KeyLabelSlot] {
        &self.slots[..12]
    }

    /// Row 1: F13–F24 (slots 12..24).
    ///
    /// Validates: Requirement 13.1
    pub fn row1(&self) -> &[KeyLabelSlot] {
        &self.slots[12..]
    }

    /// Get the slot for a specific key.
    pub fn slot_for(&self, key: FunctionKey) -> Option<&KeyLabelSlot> {
        self.slots.iter().find(|s| s.key == key)
    }

    /// Get only the assigned (non-blank) slots.
    pub fn assigned_slots(&self) -> impl Iterator<Item = &KeyLabelSlot> {
        self.slots.iter().filter(|s| s.label.is_some())
    }

    /// Update the model from a new key map.
    ///
    /// Replaces all slot labels with data from the new map.
    pub fn update(&mut self, key_map: &KeyMap) {
        for slot in &mut self.slots {
            slot.label = key_map
                .get_plain(slot.key)
                .map(|binding| binding.display_label().to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function_key::ModifiedKey;
    use crate::key_map::KeyBinding;

    fn make_test_map() -> KeyMap {
        let mut map = KeyMap::empty("test");
        map.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("END"));
        map.set(
            ModifiedKey::plain(FunctionKey::F5),
            KeyBinding::new("FIND 'ERROR' ALL"),
        );
        map.set(
            ModifiedKey::plain(FunctionKey::F7),
            KeyBinding::with_label("UP MAX", "UP"),
        );
        map
    }

    #[test]
    fn label_derived_from_first_token_of_command() {
        // Validates: Requirement 4.4
        let model = KeyLabelBarModel::from_key_map(&make_test_map());
        let slot = model.slot_for(FunctionKey::F5).unwrap();
        assert_eq!(slot.label.as_deref(), Some("FIND"));
    }

    #[test]
    fn explicit_label_overrides_derived() {
        // Validates: Requirement 4.5
        let model = KeyLabelBarModel::from_key_map(&make_test_map());
        let slot = model.slot_for(FunctionKey::F7).unwrap();
        assert_eq!(slot.label.as_deref(), Some("UP"));
    }

    #[test]
    fn unassigned_keys_have_blank_slots() {
        // Validates: Requirement 4.3
        let model = KeyLabelBarModel::from_key_map(&make_test_map());
        let slot = model.slot_for(FunctionKey::F4).unwrap();
        assert_eq!(slot.label, None);
    }

    #[test]
    fn f1_is_in_label_bar_with_no_label_by_default() {
        // Validates: Requirement 13.2 — all 24 slots present; F1 unassigned in test map
        let model = KeyLabelBarModel::from_key_map(&make_test_map());
        let slot = model.slot_for(FunctionKey::F1).unwrap();
        assert_eq!(slot.label, None);
    }

    #[test]
    fn assigned_slots_returns_only_non_blank() {
        let model = KeyLabelBarModel::from_key_map(&make_test_map());
        let assigned: Vec<_> = model.assigned_slots().collect();
        assert_eq!(assigned.len(), 3);
    }

    #[test]
    fn two_rows_of_twelve_slots_each() {
        // Validates: Requirement 13.1 — row0 = F1–F12, row1 = F13–F24
        let model = KeyLabelBarModel::from_key_map(&make_test_map());
        assert_eq!(model.row0().len(), 12);
        assert_eq!(model.row1().len(), 12);
        assert_eq!(model.row0()[0].key, FunctionKey::F1);
        assert_eq!(model.row0()[11].key, FunctionKey::F12);
        assert_eq!(model.row1()[0].key, FunctionKey::F13);
        assert_eq!(model.row1()[11].key, FunctionKey::F24);
    }

    #[test]
    fn unassigned_slots_present_with_none_label() {
        // Validates: Requirement 13.2 — blank slots preserve grid layout
        let model = KeyLabelBarModel::from_key_map(&make_test_map());
        // F13–F24 are all unassigned in the test map
        for slot in model.row1() {
            assert_eq!(slot.label, None, "F{} should be blank", slot.key.number());
        }
    }

    #[test]
    fn update_refreshes_labels_from_new_map() {
        // Validates: Requirement 4.6 — update on key map change
        let mut model = KeyLabelBarModel::from_key_map(&make_test_map());

        let mut new_map = KeyMap::empty("new");
        new_map.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("QUIT"));
        model.update(&new_map);

        assert_eq!(
            model.slot_for(FunctionKey::F3).unwrap().label.as_deref(),
            Some("QUIT")
        );
        // F5 was assigned but is now unassigned in new map
        assert_eq!(model.slot_for(FunctionKey::F5).unwrap().label, None);
    }

    #[test]
    fn empty_key_map_produces_all_blank_slots() {
        // Validates: Requirement 13.2 — all 24 slots present even when map is empty
        let model = KeyLabelBarModel::from_key_map(&KeyMap::empty("empty"));
        assert_eq!(model.assigned_slots().count(), 0);
        assert_eq!(model.slots().len(), 24);
    }
}
