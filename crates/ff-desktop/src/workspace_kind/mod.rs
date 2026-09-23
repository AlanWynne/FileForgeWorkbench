//! Configurable Workspace Kinds (CR-NR-090, Slice B.1).
//!
//! A Workspace Kind's presentation and profile are data (`KindConfig`) resolved
//! through a `KindRegistry`. A Kind is either BUILT-IN (compiled, maps to a core
//! command) or USER-created (a flat override "modelled on" a built-in base,
//! sharing the base's command/behaviour but overriding title / menu bar / key
//! list / profile). There is deliberately NO separate "Definition" layer: a user
//! preset IS just a new Kind modelled on a base.
//!
//! B.1 delivers the model, the registry with compiled built-in defaults, TOML
//! round-trip, and title-derived-from-config (fixing the Catalog Explorer vs File
//! Explorer shared-`[FILES]` label smell). Menu bar + key list wiring (B.2),
//! profile application (B.3), and the config dialog (B.4) are later slices.
//!
//! The model is EXTERNAL-READY: `BaseKind::External(name)` is accepted by the
//! schema for a future Lua/REXX-provided base, but is unresolved in v1.

use ff_edit_operations::EditProfile;
use serde::{Deserialize, Serialize};

use crate::tab_state::TabKind;

/// The compiled built-in Workspace Kinds, keyed by the SAME stable names used by
/// `context_name_for_kind` (function-keys Req 14.6). One variant per stable Kind
/// identity. `Pom` and `Menu` are distinct here because the shell splits them by
/// `is_home` even though both render as `TabKind::MenuWorkspace`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinKind {
    Editor,
    Files,
    Catalogs,
    Config,
    Search,
    Plugins,
    Log,
    Macros,
    Menu,
    Pom,
    Commands,
    Theme,
    Menus,
    Keys,
    Kinds,
}

impl BuiltinKind {
    /// All built-in kinds (registry seeding + tests).
    pub const ALL: [BuiltinKind; 15] = [
        BuiltinKind::Editor,
        BuiltinKind::Files,
        BuiltinKind::Catalogs,
        BuiltinKind::Config,
        BuiltinKind::Search,
        BuiltinKind::Plugins,
        BuiltinKind::Log,
        BuiltinKind::Macros,
        BuiltinKind::Menu,
        BuiltinKind::Pom,
        BuiltinKind::Commands,
        BuiltinKind::Theme,
        BuiltinKind::Menus,
        BuiltinKind::Keys,
        BuiltinKind::Kinds,
    ];

    /// The stable name for this built-in kind (the id used for keymaps, keymap
    /// context, session descriptor, and the default Kind name).
    pub fn stable_name(self) -> &'static str {
        match self {
            BuiltinKind::Editor => "editor",
            BuiltinKind::Files => "files",
            BuiltinKind::Catalogs => "catalogs",
            BuiltinKind::Config => "config",
            BuiltinKind::Search => "search",
            BuiltinKind::Plugins => "plugins",
            BuiltinKind::Log => "log",
            BuiltinKind::Macros => "macros",
            BuiltinKind::Menu => "menu",
            BuiltinKind::Pom => "pom",
            BuiltinKind::Commands => "commands",
            BuiltinKind::Theme => "theme",
            BuiltinKind::Menus => "menus",
            BuiltinKind::Keys => "keys",
            BuiltinKind::Kinds => "kinds",
        }
    }

    /// Parse a stable name to a built-in kind (`None` if not a built-in name).
    pub fn from_stable_name(name: &str) -> Option<BuiltinKind> {
        BuiltinKind::ALL
            .into_iter()
            .find(|k| k.stable_name() == name)
    }

    /// The compiled default title label for this built-in kind. Distinct labels
    /// for distinct kinds (Req 2.4): Catalogs is `[CATALOGS]`, Files (the File
    /// Explorer / navigator) is `[FILES]` -- they no longer collide.
    pub fn default_title(self) -> &'static str {
        match self {
            BuiltinKind::Editor => "[EDITOR]",
            BuiltinKind::Files => "[FILES]",
            BuiltinKind::Catalogs => "[CATALOGS]",
            BuiltinKind::Config => "[CONFIG]",
            BuiltinKind::Search => "[SEARCH]",
            BuiltinKind::Plugins => "[PLUGINS]",
            BuiltinKind::Log => "[LOG]",
            BuiltinKind::Macros => "[MACROS]",
            BuiltinKind::Menu => "[MENU]",
            BuiltinKind::Pom => "[POM]",
            BuiltinKind::Commands => "[COMMANDS]",
            BuiltinKind::Theme => "[THEME]",
            BuiltinKind::Menus => "[MENUS]",
            BuiltinKind::Keys => "[KEYS]",
            BuiltinKind::Kinds => "[KINDS]",
        }
    }

    /// The descriptive, Title-Case DISPLAY name for the centered Title_Line
    /// (CR-CH-045, menu-and-statusbar Req 17.13). This is DISTINCT from
    /// [`default_title`], which is the terse bracketed `[XXX]` Tab_Header tag
    /// (the ISPF tab label, B016) and stays unchanged. `Editor`, `Pom`, and
    /// `Menu` retain their existing Title_Line derivation (file path / Menu_Title)
    /// and are not routed through this helper, so their display names simply
    /// mirror the tab tag here for completeness.
    pub fn display_title(self) -> &'static str {
        match self {
            BuiltinKind::Editor => "Editor",
            BuiltinKind::Files => "File Explorer",
            BuiltinKind::Catalogs => "Catalog Explorer",
            BuiltinKind::Config => "Configuration",
            BuiltinKind::Search => "Search Results",
            BuiltinKind::Plugins => "Plugin Manager",
            BuiltinKind::Log => "Event Log",
            BuiltinKind::Macros => "Macro Library",
            BuiltinKind::Menu => "Menu",
            BuiltinKind::Pom => "Primary Option Menu",
            BuiltinKind::Commands => "Command Configurator",
            BuiltinKind::Theme => "Theme Editor",
            BuiltinKind::Menus => "Menu Editor",
            BuiltinKind::Keys => "Key Assignments Editor",
            BuiltinKind::Kinds => "Workspace Kinds Editor",
        }
    }

    /// Map a runtime `TabKind` (+ `is_home` for the POM split) to a built-in kind.
    /// A `MenuWorkspace` that is the Home Context maps to `Pom`, otherwise `Menu`.
    pub fn from_tab_kind(kind: TabKind, is_home: bool) -> BuiltinKind {
        match kind {
            TabKind::FileEditor | TabKind::Untitled => BuiltinKind::Editor,
            TabKind::FilesPanel => BuiltinKind::Catalogs,
            TabKind::FileExplorerPanel => BuiltinKind::Files,
            TabKind::ConfigPanel => BuiltinKind::Config,
            TabKind::SearchResults => BuiltinKind::Search,
            TabKind::PluginManager => BuiltinKind::Plugins,
            TabKind::EventLog => BuiltinKind::Log,
            TabKind::MacroLibrary => BuiltinKind::Macros,
            TabKind::CommandConfigurator => BuiltinKind::Commands,
            TabKind::ThemeEditor => BuiltinKind::Theme,
            TabKind::MenusEditor => BuiltinKind::Menus,
            TabKind::KeysEditor => BuiltinKind::Keys,
            TabKind::KindsEditor => BuiltinKind::Kinds,
            TabKind::MenuWorkspace => {
                if is_home {
                    BuiltinKind::Pom
                } else {
                    BuiltinKind::Menu
                }
            }
        }
    }
}

/// What a Kind is modelled on. Open model: `Builtin` is resolvable in v1;
/// `External` is accepted by the persisted schema (future Lua/REXX Kinds) but has
/// no resolver in v1 (unresolved -> a safe built-in fallback, with a notice).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseKind {
    /// A compiled built-in base (the only resolvable form in v1).
    Builtin(BuiltinKind),
    /// A future external (Lua/REXX-registered) base by name. Unresolved in v1.
    External(String),
}

impl BaseKind {
    /// Serialise to the on-disk string tag: `"editor"` for a built-in, or
    /// `"ext:<name>"` for an external base.
    pub fn to_tag(&self) -> String {
        match self {
            BaseKind::Builtin(k) => k.stable_name().to_string(),
            BaseKind::External(name) => format!("ext:{name}"),
        }
    }

    /// Parse a string tag back to a `BaseKind`. An `ext:` prefix yields an
    /// `External`; a known built-in name yields `Builtin`; anything else is an
    /// `External` (so an unknown tag round-trips and is flagged at resolve time
    /// rather than lost).
    pub fn from_tag(tag: &str) -> BaseKind {
        if let Some(name) = tag.strip_prefix("ext:") {
            BaseKind::External(name.to_string())
        } else if let Some(k) = BuiltinKind::from_stable_name(tag) {
            BaseKind::Builtin(k)
        } else {
            BaseKind::External(tag.to_string())
        }
    }
}

/// Where a Workspace Kind places its `Command ===>` line (CR-NR-095, Req 8).
/// `Top` (the default) puts it under the Title_Line, above the Context body (the
/// existing behaviour); `Bottom` puts it below the body, at the foot of the
/// Workspace. Serialises to the stable lowercase spelling `"top"` / `"bottom"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CommandLinePosition {
    /// Command line under the Title_Line, above the body (current default).
    #[default]
    Top,
    /// Command line at the foot of the Workspace, below the body.
    Bottom,
}

impl CommandLinePosition {
    /// The opposite position (for the bare `COMMAND` toggle, Req 8.9).
    pub fn toggled(self) -> Self {
        match self {
            CommandLinePosition::Top => CommandLinePosition::Bottom,
            CommandLinePosition::Bottom => CommandLinePosition::Top,
        }
    }
}

/// The profile attributes a Kind carries (applied on open in B.3; inert in B.1).
/// Extensible: theme override / default view-edit mode / working dir are future.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KindProfile {
    /// ISPF edit-profile defaults (CAPS/NULLS/STATS/LOCK/HILITE).
    pub edit_profile: EditProfile,
    /// Default tab size (columns).
    pub tab_size: u8,
    /// Line-end mode name ("default" | "unicode"), kept as a string so the
    /// persisted schema is stable without depending on ff-document-model here.
    pub line_end_mode: String,
    /// Where this Kind places its `Command ===>` line (CR-NR-095, Req 8.1).
    /// Defaults to `Top`; round-trips via the struct-level `#[serde(default)]`
    /// so a Kind file predating this attribute loads as `Top` (Req 8.2).
    pub command_line_position: CommandLinePosition,
}

impl Default for KindProfile {
    fn default() -> Self {
        Self {
            edit_profile: EditProfile::default(),
            tab_size: 8,
            line_end_mode: "default".to_string(),
            command_line_position: CommandLinePosition::default(),
        }
    }
}

/// The configurable record for a Workspace Kind (a compiled built-in default or a
/// user file). See module docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindConfig {
    /// Stable unique id (built-in: "editor"...; user: "mainframe-editor").
    pub name: String,
    /// What this Kind is modelled on (built-in base in v1).
    pub modelled_on: BaseKind,
    /// Tab / Title_Line label, e.g. "[CATALOGS]".
    pub title: String,
    /// Named menu bar; `None` => the base Kind's default (wired in B.2).
    pub menu_bar: Option<String>,
    /// Named key list; `None` => the base Kind's default (wired in B.2).
    pub key_list: Option<String>,
    /// Profile attributes (applied on open in B.3).
    pub profile: KindProfile,
}

impl KindConfig {
    /// The compiled default config for a built-in kind: name = stable name,
    /// modelled on itself, its default title, no menu/key overrides, default
    /// profile.
    pub fn builtin_default(kind: BuiltinKind) -> Self {
        Self {
            name: kind.stable_name().to_string(),
            modelled_on: BaseKind::Builtin(kind),
            title: kind.default_title().to_string(),
            menu_bar: None,
            key_list: None,
            profile: KindProfile::default(),
        }
    }
}

/// The on-disk serde shape for a `KindConfig` (stable TOML schema). `modelled_on`
/// is a string tag so both `Builtin` and `External` bases round-trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KindConfigToml {
    pub name: String,
    pub modelled_on: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub menu_bar: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_list: Option<String>,
    #[serde(default)]
    pub profile: KindProfile,
}

impl From<&KindConfig> for KindConfigToml {
    fn from(c: &KindConfig) -> Self {
        Self {
            name: c.name.clone(),
            modelled_on: c.modelled_on.to_tag(),
            title: c.title.clone(),
            menu_bar: c.menu_bar.clone(),
            key_list: c.key_list.clone(),
            profile: c.profile.clone(),
        }
    }
}

impl From<KindConfigToml> for KindConfig {
    fn from(t: KindConfigToml) -> Self {
        Self {
            name: t.name,
            modelled_on: BaseKind::from_tag(&t.modelled_on),
            title: t.title,
            menu_bar: t.menu_bar,
            key_list: t.key_list,
            profile: t.profile,
        }
    }
}

mod registry;
pub use registry::KindRegistry;

#[cfg(test)]
mod tests;
