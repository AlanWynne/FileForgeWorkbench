//! Key map resolver — selects the active key map based on language profile state
//! and window context.
//!
//! Resolution priority (highest to lowest):
//! 1. Context_Key_Map (if active_context is set and a map exists for it)
//! 2. Profile_Key_Map (if a language profile is active)
//! 3. Global_Key_Map
//!
//! Each level is full-replacement: activating a higher-priority map suppresses
//! all lower-priority maps entirely.

use std::collections::HashMap;

use crate::key_map::KeyMap;

/// Selects the active key map based on the current window context and language profile.
///
/// Implements the full-replacement model at every level.
#[derive(Debug, Clone)]
pub struct KeyMapResolver {
    /// The loaded global key map.
    global_key_map: KeyMap,
    /// The currently active profile key map (if any).
    active_profile_key_map: Option<KeyMap>,
    /// The name of the currently active language profile (if any).
    active_profile_name: Option<String>,
    /// Per-context key maps keyed by context name (e.g. "pom", "editor").
    ///
    /// Validates: Requirement 14.1
    context_maps: HashMap<String, KeyMap>,
    /// The currently active context name (if any).
    ///
    /// Validates: Requirement 14.2
    active_context: Option<String>,
}

impl KeyMapResolver {
    /// Create a resolver with the given global key map and no active profile or context.
    pub fn new(global_key_map: KeyMap) -> Self {
        Self {
            global_key_map,
            active_profile_key_map: None,
            active_profile_name: None,
            context_maps: HashMap::new(),
            active_context: None,
        }
    }

    /// Get a reference to the currently effective key map.
    ///
    /// Priority: Context_Key_Map > Profile_Key_Map > Global_Key_Map.
    /// Each level is full-replacement.
    ///
    /// Validates: Requirement 14.3, 14.5
    pub fn active_key_map(&self) -> &KeyMap {
        // 1. Context map (highest priority)
        if let Some(ctx) = &self.active_context {
            if let Some(map) = self.context_maps.get(ctx) {
                return map;
            }
        }
        // 2. Profile map
        if let Some(map) = &self.active_profile_key_map {
            return map;
        }
        // 3. Global map
        &self.global_key_map
    }

    /// Set the active window context.
    ///
    /// Pass `None` to deactivate the context map and fall back to profile/global.
    ///
    /// Validates: Requirement 14.2, 14.4
    pub fn set_context(&mut self, context_name: Option<&str>) {
        self.active_context = context_name.map(|s| s.to_string());
    }

    /// Register a context key map for the given context name.
    ///
    /// Validates: Requirement 14.1
    pub fn set_context_map(&mut self, context_name: impl Into<String>, key_map: KeyMap) {
        self.context_maps.insert(context_name.into(), key_map);
    }

    /// Remove a context key map.
    pub fn remove_context_map(&mut self, context_name: &str) {
        self.context_maps.remove(context_name);
    }

    /// The currently active context name, if any.
    pub fn active_context(&self) -> Option<&str> {
        self.active_context.as_deref()
    }

    /// Set the active profile key map (language profile changed).
    ///
    /// Pass `None` for both parameters to deactivate the profile key map
    /// and fall back to the global map.
    pub fn set_profile_key_map(&mut self, profile_name: Option<&str>, key_map: Option<KeyMap>) {
        self.active_profile_name = profile_name.map(|s| s.to_string());
        self.active_profile_key_map = key_map;
    }

    /// Replace the global key map (configuration hot-reload).
    pub fn set_global_key_map(&mut self, key_map: KeyMap) {
        self.global_key_map = key_map;
    }

    /// Whether a profile key map is currently active.
    pub fn is_profile_active(&self) -> bool {
        self.active_profile_key_map.is_some()
    }

    /// The name of the active profile, if any.
    pub fn active_profile_name(&self) -> Option<&str> {
        self.active_profile_name.as_deref()
    }

    /// Get a reference to the global key map (for diagnostics or comparison).
    pub fn global_key_map(&self) -> &KeyMap {
        &self.global_key_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function_key::FunctionKey;
    use crate::function_key::ModifiedKey;
    use crate::key_map::KeyBinding;

    fn make_global_map() -> KeyMap {
        let mut map = KeyMap::empty("global");
        map.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("END"));
        map.set(ModifiedKey::plain(FunctionKey::F5), KeyBinding::new("FIND"));
        map.set(
            ModifiedKey::plain(FunctionKey::F7),
            KeyBinding::with_label("UP MAX", "UP"),
        );
        map
    }

    fn make_profile_map() -> KeyMap {
        let mut map = KeyMap::empty("cobol");
        map.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("END"));
        map.set(
            ModifiedKey::plain(FunctionKey::F10),
            KeyBinding::with_label("MACRO cobol_check", "CHECK"),
        );
        map
    }

    #[test]
    fn global_only_resolution_returns_global_map() {
        // Validates: Requirement 1.2 — Global_Key_Map applies when no profile
        let resolver = KeyMapResolver::new(make_global_map());
        assert!(!resolver.is_profile_active());
        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F3)
                .unwrap()
                .command(),
            "END"
        );
        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F5)
                .unwrap()
                .command(),
            "FIND"
        );
    }

    #[test]
    fn profile_override_replaces_global_map() {
        // Validates: Requirement 2.2 — Profile fully replaces global
        let mut resolver = KeyMapResolver::new(make_global_map());
        resolver.set_profile_key_map(Some("cobol"), Some(make_profile_map()));

        assert!(resolver.is_profile_active());
        assert_eq!(resolver.active_profile_name(), Some("cobol"));

        // Profile has F3 and F10
        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F3)
                .unwrap()
                .command(),
            "END"
        );
        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F10)
                .unwrap()
                .command(),
            "MACRO cobol_check"
        );
    }

    #[test]
    fn full_replacement_no_inheritance_from_global() {
        // Validates: Requirement 2.5 — Keys not in profile are unassigned
        let mut resolver = KeyMapResolver::new(make_global_map());
        resolver.set_profile_key_map(Some("cobol"), Some(make_profile_map()));

        // F5 is in global but NOT in profile — should be None
        assert_eq!(resolver.active_key_map().get_plain(FunctionKey::F5), None);
        // F7 is in global but NOT in profile — should be None
        assert_eq!(resolver.active_key_map().get_plain(FunctionKey::F7), None);
    }

    #[test]
    fn profile_removal_falls_back_to_global() {
        // Validates: Requirement 2.4 — Fallback on profile removal
        let mut resolver = KeyMapResolver::new(make_global_map());
        resolver.set_profile_key_map(Some("cobol"), Some(make_profile_map()));
        assert!(resolver.is_profile_active());

        // Remove profile
        resolver.set_profile_key_map(None, None);
        assert!(!resolver.is_profile_active());
        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F5)
                .unwrap()
                .command(),
            "FIND"
        );
    }

    #[test]
    fn empty_global_map_leaves_all_keys_unassigned() {
        // Validates: Requirement 1.4 — Empty global map means all unassigned
        let resolver = KeyMapResolver::new(KeyMap::empty("global"));
        for key in FunctionKey::ALL {
            assert_eq!(resolver.active_key_map().get_plain(key), None);
        }
    }

    #[test]
    fn profile_switch_updates_active_map() {
        // Validates: Requirement 2.6 — Profile switch recomputes active map
        let mut resolver = KeyMapResolver::new(make_global_map());

        let mut cobol_map = KeyMap::empty("cobol");
        cobol_map.set(
            ModifiedKey::plain(FunctionKey::F10),
            KeyBinding::new("MACRO cobol_check"),
        );
        resolver.set_profile_key_map(Some("cobol"), Some(cobol_map));

        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F10)
                .unwrap()
                .command(),
            "MACRO cobol_check"
        );

        // Switch to a different profile
        let mut abap_map = KeyMap::empty("abap");
        abap_map.set(
            ModifiedKey::plain(FunctionKey::F11),
            KeyBinding::new("MACRO abap_check"),
        );
        resolver.set_profile_key_map(Some("abap"), Some(abap_map));

        assert_eq!(resolver.active_profile_name(), Some("abap"));
        assert_eq!(resolver.active_key_map().get_plain(FunctionKey::F10), None);
        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F11)
                .unwrap()
                .command(),
            "MACRO abap_check"
        );
    }

    #[test]
    fn set_global_key_map_updates_when_no_profile_active() {
        // Validates: Requirement 11.7 — Hot-reload of global_key_map
        let mut resolver = KeyMapResolver::new(make_global_map());
        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F3)
                .unwrap()
                .command(),
            "END"
        );

        let mut new_global = KeyMap::empty("global");
        new_global.set(ModifiedKey::plain(FunctionKey::F3), KeyBinding::new("QUIT"));
        resolver.set_global_key_map(new_global);

        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F3)
                .unwrap()
                .command(),
            "QUIT"
        );
    }

    #[test]
    fn context_map_overrides_global_map() {
        // Validates: Requirement 14.2, 14.5
        let mut resolver = KeyMapResolver::new(make_global_map());
        let mut ctx_map = KeyMap::empty("pom");
        ctx_map.set(
            ModifiedKey::plain(FunctionKey::F3),
            KeyBinding::new("RETURN"),
        );
        resolver.set_context_map("pom", ctx_map);
        resolver.set_context(Some("pom"));

        assert_eq!(resolver.active_context(), Some("pom"));
        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F3)
                .unwrap()
                .command(),
            "RETURN"
        );
        // F5 is in global but not in context map — full replacement
        assert_eq!(resolver.active_key_map().get_plain(FunctionKey::F5), None);
    }

    #[test]
    fn unknown_context_falls_back_to_global() {
        // Validates: Requirement 14.3
        let mut resolver = KeyMapResolver::new(make_global_map());
        resolver.set_context(Some("unknown_context"));
        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F3)
                .unwrap()
                .command(),
            "END"
        );
    }

    #[test]
    fn clearing_context_falls_back_to_global() {
        // Validates: Requirement 14.4
        let mut resolver = KeyMapResolver::new(make_global_map());
        let mut ctx_map = KeyMap::empty("pom");
        ctx_map.set(
            ModifiedKey::plain(FunctionKey::F3),
            KeyBinding::new("RETURN"),
        );
        resolver.set_context_map("pom", ctx_map);
        resolver.set_context(Some("pom"));
        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F3)
                .unwrap()
                .command(),
            "RETURN"
        );

        resolver.set_context(None);
        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F3)
                .unwrap()
                .command(),
            "END"
        );
    }

    #[test]
    fn context_map_takes_priority_over_profile_map() {
        // Validates: Requirement 14.5 — context > profile > global
        let mut resolver = KeyMapResolver::new(make_global_map());
        resolver.set_profile_key_map(Some("cobol"), Some(make_profile_map()));

        let mut ctx_map = KeyMap::empty("editor");
        ctx_map.set(
            ModifiedKey::plain(FunctionKey::F3),
            KeyBinding::new("CONTEXT_CMD"),
        );
        resolver.set_context_map("editor", ctx_map);
        resolver.set_context(Some("editor"));

        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F3)
                .unwrap()
                .command(),
            "CONTEXT_CMD"
        );
    }

    #[test]
    fn profile_map_used_when_no_context_map_registered() {
        // Validates: Requirement 14.3 — profile still works when no context map
        let mut resolver = KeyMapResolver::new(make_global_map());
        resolver.set_profile_key_map(Some("cobol"), Some(make_profile_map()));
        resolver.set_context(Some("editor")); // no map registered for "editor"

        assert_eq!(
            resolver
                .active_key_map()
                .get_plain(FunctionKey::F10)
                .unwrap()
                .command(),
            "MACRO cobol_check"
        );
    }
}
