//! Focused-editable-Context SEAM (CR-CH-071, command-environments Req 20a).
//!
//! The ISPF Save/End/Cancel/Return/Refresh verbs for an editable NON-buffer
//! Context are handled by the FFCMD base (`try_commands_a`), which owns NO
//! working copy of its own. It routes to the focused Context that DOES own the
//! working copy through this small dispatch SEAM, keyed by the active tab's
//! `KindTag` (Req 20a.6). The seam is realised the SAME way FFEDIT is -- a
//! dispatch helper on `WorkbenchShell`, not a free-standing `dyn` object --
//! because the editable Contexts are render-entangled panels on the shell, and
//! it mirrors the existing `apply_<editor>_action` / `pending_action` pattern
//! (framework-conformance: no new public framework type, single front door).
//!
//! The three seam operations:
//!   - `editable_persist`  -> validate + write the working copy, return ok/err
//!   - `editable_discard`  -> drop the unsaved working edits (no write)
//!   - `editable_reload`   -> re-read the last-saved state from the backing store
//!
//! v1 wires ONLY the Alias Editor (the required deliverable). The five sibling
//! config editors (Settings / Menus / Theme / Keys / Kinds) are the fast-follow
//! set: each has a DIFFERENT working-copy shape and save entry point, so wiring
//! all five is real scope bloat that would risk the v1 deliverable. The seam is
//! SHAPED so each plugs in by adding its `EditableKind` arm; until then they are
//! NOT editable here and fall through to today's plain navigation (Req 20a.14).

use super::WorkbenchShell;
use crate::tab_state::KindTag;

/// Which focused editable NON-buffer Context owns the working copy the FFCMD
/// editable-Context seam routes to (CR-CH-071, Req 20a.6). An internal routing
/// marker only -- NOT a public framework type (framework-conformance). v1 wires
/// only [`EditableKind::AliasEditor`]; the sibling config editors are the
/// fast-follow set (see the module docs and the TODO below).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Each variant names a concrete editable Context whose product name ends in
// "Editor" (Alias/Theme/Menus/Keys/Kinds Editor), so the shared "Editor" suffix
// is intentional and matches the Context names; renaming them would diverge from
// the UI vocabulary for no benefit.
#[allow(clippy::enum_variant_names)]
pub(super) enum EditableKind {
    /// The Alias Editor Context (the v1 deliverable).
    AliasEditor,
    /// The Theme Editor Context (project-master Req 11, B081 fix). Wired into the
    /// shared seam so typed SAVE / END / RETURN / REFRESH persist/discard/reload
    /// the working theme, matching the convention the Alias Editor already meets.
    ThemeEditor,
    /// The Menus Editor Context (project-master Req 11, B081 second fix). Typed
    /// SAVE persists + stays, END persists + exits, RETURN/REFRESH discard the
    /// working menu and re-open the persisted copy.
    MenusEditor,
    /// The Keys Editor Context (project-master Req 11, B081 second fix). Typed
    /// SAVE persists + stays, END persists + exits, RETURN/REFRESH discard the
    /// working key rows and re-open the persisted copy.
    KeysEditor,
    /// The Kinds Editor Context (project-master Req 11, B081 second fix). Typed
    /// SAVE persists + stays, END persists + exits, RETURN/REFRESH discard the
    /// working Kind config and re-open the persisted copy.
    KindsEditor,
    /// The Config / Settings Context (project-master Req 11, B081 third fix).
    /// Unlike the sibling editors it used to AUTOSAVE on every edit; it now owns
    /// a WORKING-COPY BUFFER in `ConfigPanelState`, so typed SAVE flushes the
    /// buffer + stays, END flushes + exits, and RETURN/REFRESH drop the buffer
    /// (reverting all pending edits). The `ConfigHandle` store semantics are
    /// unchanged; the working copy lives in the panel.
    ConfigPanel,
    /// The Command Configurator Context (project-master Req 11, B081 -- the LAST
    /// non-compliant editable Context). It used to write `commands.toml` on every
    /// add / edit / delete (autosave); it now owns a WHOLE-STORE WORKING COPY in
    /// `CommandConfiguratorState`, so typed SAVE flushes the working copy + stays,
    /// END flushes + exits, and RETURN/REFRESH drop the working copy (reverting
    /// all pending edits, including a staged delete).
    CommandConfigurator,
}

impl WorkbenchShell {
    /// Classify the FOCUSED Context as an editable non-buffer Context the seam
    /// can persist / discard / reload (Req 20a.6, 20a.11). Returns `Some` only
    /// when the active tab's kind is a WIRED editable Context. An editor BUFFER
    /// (`FileEditor` / `Untitled` -> FFEDIT) is NEVER editable here -- that is the
    /// active-wins boundary (Req 20a.7): FFEDIT claims SAVE first, so the FFCMD
    /// seam is only reached on a non-buffer Context.
    ///
    /// v1 WIRES only the Alias Editor. The five sibling config editors return
    /// `None` (non-editable fall-through -> today's plain navigation, Req 20a.14)
    /// until the fast-follow slices wire them.
    ///
    /// Validates: command-environments Requirement 20a.6, 20a.11, 20a.14
    pub(super) fn editable_context_kind(&self) -> Option<EditableKind> {
        match self.tabs.active_tab().kind.tag() {
            KindTag::AliasEditor => Some(EditableKind::AliasEditor),
            // B081 / project-master Req 11: the Theme Editor is now wired into the
            // shared seam, so typed SAVE persists, END persists-then-exits, and
            // RETURN/REFRESH discard the working theme and revert the live preview.
            KindTag::ThemeEditor => Some(EditableKind::ThemeEditor),
            // B081 second fix / project-master Req 11: the Menus / Keys / Kinds
            // editors are now wired into the shared seam (each already owned a
            // working copy + an in-panel Save button; the only gap was the ISPF
            // command verbs). Each persist delegates to that editor's EXISTING
            // save function; each discard/reload re-opens the persisted copy.
            KindTag::MenusEditor => Some(EditableKind::MenusEditor),
            KindTag::KeysEditor => Some(EditableKind::KeysEditor),
            KindTag::KindsEditor => Some(EditableKind::KindsEditor),
            // B081 third fix / project-master Req 11: the Config / Settings panel
            // is now wired into the shared seam. It differs from the five sibling
            // editors in that it previously AUTOSAVED on every edit; it now owns a
            // WORKING-COPY BUFFER in `ConfigPanelState`, so persist flushes the
            // buffer, discard drops it, and reload re-snapshots from the store.
            // See save-convention-audit findings 4.3.
            KindTag::ConfigPanel => Some(EditableKind::ConfigPanel),
            // B081 / project-master Req 11 (the LAST editable Context): the
            // Command Configurator is now wired into the shared seam. It used to
            // autosave each add/edit/delete; it now owns a whole-store working
            // copy, so persist flushes it, discard drops it, reload re-reads the
            // store from disk.
            KindTag::CommandConfigurator => Some(EditableKind::CommandConfigurator),
            _ => None,
        }
    }

    /// SEAM PERSIST (Req 20a.6): persist the focused editable Context's working
    /// copy, returning ok/err so the caller's END can branch save-then-leave vs
    /// stay-with-error. Routes by the active `KindTag` to the owning Context.
    ///
    /// Validates: command-environments Requirement 20a.2, 20a.6
    pub(super) fn editable_persist(&mut self) -> Result<(), String> {
        match self.editable_context_kind() {
            Some(EditableKind::AliasEditor) => self.alias_editor_persist(),
            Some(EditableKind::ThemeEditor) => self.theme_editor_persist(),
            Some(EditableKind::MenusEditor) => self.menus_editor_persist(),
            Some(EditableKind::KeysEditor) => self.keys_editor_persist(),
            Some(EditableKind::KindsEditor) => self.kinds_editor_persist(),
            Some(EditableKind::ConfigPanel) => self.config_panel_persist(),
            Some(EditableKind::CommandConfigurator) => self.command_configurator_persist(),
            None => Ok(()),
        }
    }

    /// SEAM DISCARD (Req 20a.6): drop the focused editable Context's unsaved
    /// working edits WITHOUT writing the backing store. A no-op on a Context the
    /// seam does not own.
    ///
    /// Validates: command-environments Requirement 20a.3, 20a.6
    pub(super) fn editable_discard(&mut self) {
        match self.editable_context_kind() {
            Some(EditableKind::AliasEditor) => self.alias_editor_discard(),
            Some(EditableKind::ThemeEditor) => self.theme_editor_discard(),
            Some(EditableKind::MenusEditor) => self.menus_editor_discard(),
            Some(EditableKind::KeysEditor) => self.keys_editor_discard(),
            Some(EditableKind::KindsEditor) => self.kinds_editor_discard(),
            Some(EditableKind::ConfigPanel) => self.config_panel_discard(),
            Some(EditableKind::CommandConfigurator) => self.command_configurator_discard(),
            None => {}
        }
    }

    /// SEAM RELOAD (Req 20a.6): re-read the focused editable Context's last-saved
    /// state from its backing store into the working copy. A no-op on a Context
    /// the seam does not own.
    ///
    /// Validates: command-environments Requirement 20a.4, 20a.6
    pub(super) fn editable_reload(&mut self) {
        match self.editable_context_kind() {
            Some(EditableKind::AliasEditor) => self.alias_editor_reload(),
            Some(EditableKind::ThemeEditor) => self.theme_editor_reload(),
            Some(EditableKind::MenusEditor) => self.menus_editor_reload(),
            Some(EditableKind::KeysEditor) => self.keys_editor_reload(),
            Some(EditableKind::KindsEditor) => self.kinds_editor_reload(),
            Some(EditableKind::ConfigPanel) => self.config_panel_reload(),
            Some(EditableKind::CommandConfigurator) => self.command_configurator_reload(),
            None => {}
        }
    }
}
