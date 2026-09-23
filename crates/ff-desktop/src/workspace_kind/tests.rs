//! Unit tests for the Workspace Kind model + registry (CR-NR-090, Slice B.1).

use super::*;

// === Model: BaseKind tag round-trip =========================================

/// Validates: workspace-kinds Requirement 1.2, 1.6; design P3 -- a built-in base
/// round-trips through its stable-name tag.
#[test]
fn base_kind_builtin_tag_round_trips() {
    let b = BaseKind::Builtin(BuiltinKind::Editor);
    assert_eq!(b.to_tag(), "editor");
    assert_eq!(BaseKind::from_tag("editor"), b);
}

/// Validates: workspace-kinds Requirement 1.2 -- an External base round-trips
/// through the `ext:` tag (external-ready schema; unresolved in v1).
#[test]
fn base_kind_external_tag_round_trips() {
    let e = BaseKind::External("my-lua-kind".to_string());
    assert_eq!(e.to_tag(), "ext:my-lua-kind");
    assert_eq!(BaseKind::from_tag("ext:my-lua-kind"), e);
}

/// Validates: workspace-kinds Requirement 1.2 -- an unknown bare tag becomes an
/// External (round-trips, flagged at resolve time) rather than being lost.
#[test]
fn base_kind_unknown_tag_becomes_external() {
    assert_eq!(
        BaseKind::from_tag("totally-unknown"),
        BaseKind::External("totally-unknown".to_string())
    );
}

// === Model: KindConfig TOML round-trip ======================================

/// Validates: workspace-kinds Requirement 1.6; design P3 -- a KindConfig
/// round-trips to TOML and back, including modelled_on in both forms.
#[test]
fn kind_config_toml_round_trips_builtin_base() {
    let cfg = KindConfig {
        name: "mainframe-editor".to_string(),
        modelled_on: BaseKind::Builtin(BuiltinKind::Editor),
        title: "[MF-EDIT]".to_string(),
        menu_bar: Some("MB-MF".to_string()),
        key_list: Some("mf".to_string()),
        profile: KindProfile::default(),
    };
    let toml_str = toml::to_string(&KindConfigToml::from(&cfg)).expect("serialise");
    let back: KindConfig = toml::from_str::<KindConfigToml>(&toml_str)
        .expect("parse")
        .into();
    assert_eq!(back, cfg);
    // The tag is the stable name, not a debug form.
    assert!(toml_str.contains("modelled_on = \"editor\""));
}

/// Validates: workspace-kinds Requirement 1.2, 1.6 -- an External-based Kind
/// round-trips with its `ext:` tag.
#[test]
fn kind_config_toml_round_trips_external_base() {
    let cfg = KindConfig {
        name: "lua-thing".to_string(),
        modelled_on: BaseKind::External("lua-thing-provider".to_string()),
        title: "[LUA]".to_string(),
        menu_bar: None,
        key_list: None,
        profile: KindProfile::default(),
    };
    let toml_str = toml::to_string(&KindConfigToml::from(&cfg)).expect("serialise");
    assert!(toml_str.contains("modelled_on = \"ext:lua-thing-provider\""));
    let back: KindConfig = toml::from_str::<KindConfigToml>(&toml_str)
        .expect("parse")
        .into();
    assert_eq!(back, cfg);
}

// === BuiltinKind <-> stable name / TabKind mapping ==========================

/// Validates: workspace-kinds Requirement 1.5 -- stable names round-trip and are
/// unique across all built-in kinds.
#[test]
fn builtin_stable_names_are_unique_and_round_trip() {
    let mut seen = std::collections::HashSet::new();
    for k in BuiltinKind::ALL {
        let n = k.stable_name();
        assert!(seen.insert(n), "duplicate stable name {n}");
        assert_eq!(BuiltinKind::from_stable_name(n), Some(k));
    }
}

/// Validates: workspace-kinds Requirement 1.4 -- every runtime TabKind maps to a
/// built-in kind, and the POM split (is_home) resolves to Pom vs Menu.
#[test]
fn tab_kind_maps_to_builtin_kind() {
    use crate::tab_state::TabKind;
    assert_eq!(
        BuiltinKind::from_tab_kind(TabKind::FilesPanel, false),
        BuiltinKind::Catalogs,
        "the Virtual Catalog Manager (option 1) is the Catalogs kind"
    );
    assert_eq!(
        BuiltinKind::from_tab_kind(TabKind::FileExplorerPanel, false),
        BuiltinKind::Files,
        "the File Explorer (option 2) is the Files kind"
    );
    assert_eq!(
        BuiltinKind::from_tab_kind(TabKind::MenuWorkspace, true),
        BuiltinKind::Pom
    );
    assert_eq!(
        BuiltinKind::from_tab_kind(TabKind::MenuWorkspace, false),
        BuiltinKind::Menu
    );
    assert_eq!(
        BuiltinKind::from_tab_kind(TabKind::FileEditor, false),
        BuiltinKind::Editor
    );
}

/// Validates: workspace-kinds Requirement 2.4; design P4 -- the compiled default
/// titles for the Catalog Explorer and File Explorer are DISTINCT (the smell).
#[test]
fn builtin_default_titles_distinguish_catalogs_from_files() {
    assert_eq!(BuiltinKind::Catalogs.default_title(), "[CATALOGS]");
    assert_eq!(BuiltinKind::Files.default_title(), "[FILES]");
    assert_ne!(
        BuiltinKind::Catalogs.default_title(),
        BuiltinKind::Files.default_title(),
        "Catalog Explorer and File Explorer must not share a title label"
    );
}

/// Validates: menu-and-statusbar Requirement 17.13 (CR-CH-045) -- each Kind has a
/// descriptive Title-Case display name for the centered Title_Line, DISTINCT from
/// the terse bracketed `[XXX]` Tab_Header tag (`default_title`).
#[test]
fn builtin_display_titles_are_descriptive_title_case_and_distinct_from_tab_tags() {
    // The covered editor/config + read-only panel Kinds.
    assert_eq!(BuiltinKind::Config.display_title(), "Configuration");
    assert_eq!(BuiltinKind::Theme.display_title(), "Theme Editor");
    assert_eq!(BuiltinKind::Menus.display_title(), "Menu Editor");
    assert_eq!(BuiltinKind::Keys.display_title(), "Key Assignments Editor");
    assert_eq!(BuiltinKind::Kinds.display_title(), "Workspace Kinds Editor");
    assert_eq!(
        BuiltinKind::Commands.display_title(),
        "Command Configurator"
    );
    assert_eq!(BuiltinKind::Catalogs.display_title(), "Catalog Explorer");
    assert_eq!(BuiltinKind::Files.display_title(), "File Explorer");
    assert_eq!(BuiltinKind::Search.display_title(), "Search Results");
    assert_eq!(BuiltinKind::Plugins.display_title(), "Plugin Manager");
    assert_eq!(BuiltinKind::Log.display_title(), "Event Log");
    assert_eq!(BuiltinKind::Macros.display_title(), "Macro Library");

    // The display title is descriptive (no brackets), unlike the tab tag.
    for k in [
        BuiltinKind::Config,
        BuiltinKind::Theme,
        BuiltinKind::Menus,
        BuiltinKind::Keys,
        BuiltinKind::Kinds,
        BuiltinKind::Commands,
        BuiltinKind::Catalogs,
        BuiltinKind::Files,
        BuiltinKind::Search,
        BuiltinKind::Plugins,
        BuiltinKind::Log,
        BuiltinKind::Macros,
    ] {
        assert!(
            !k.display_title().contains('['),
            "{} display title must not be a bracketed tag",
            k.stable_name()
        );
        assert_ne!(
            k.display_title(),
            k.default_title(),
            "{} display title must differ from its [XXX] tab tag",
            k.stable_name()
        );
    }
}

// === Registry ===============================================================

/// Validates: workspace-kinds Requirement 2.1; design P1 -- the built-in registry
/// resolves every built-in kind and `effective` is total (unknown -> safe default).
#[test]
fn registry_builtin_defaults_resolve_every_kind() {
    let reg = KindRegistry::with_builtin_defaults();
    for k in BuiltinKind::ALL {
        let cfg = reg.effective(k.stable_name());
        assert_eq!(cfg.name, k.stable_name());
        assert_eq!(cfg.title, k.default_title());
    }
    // Totality: an unknown name never panics and yields a config.
    let fallback = reg.effective("no-such-kind");
    assert!(!fallback.title.is_empty());
}

/// Validates: workspace-kinds Requirement 1.4; design P2 -- resolve_base is a
/// single hop; built-in resolves to itself.
#[test]
fn registry_resolve_base_single_hop_for_builtin() {
    let reg = KindRegistry::with_builtin_defaults();
    assert_eq!(reg.resolve_base("catalogs"), BuiltinKind::Catalogs);
    assert_eq!(reg.resolve_base("editor"), BuiltinKind::Editor);
    // Unknown -> safe POM fallback.
    assert_eq!(reg.resolve_base("no-such-kind"), BuiltinKind::Pom);
}

/// Validates: workspace-kinds Requirement 2.2 -- a user Kind file whose name is
/// NEW is added; a user file whose name equals a built-in OVERRIDES it.
#[test]
fn registry_load_adds_user_kind_and_overrides_builtin_by_name() {
    let dir = tempfile::tempdir().expect("tempdir");
    // A new user kind modelled on Editor.
    std::fs::write(
        dir.path().join("mainframe-editor.toml"),
        "name = \"mainframe-editor\"\nmodelled_on = \"editor\"\ntitle = \"[MF-EDIT]\"\n",
    )
    .unwrap();
    // Override the built-in Catalogs title.
    std::fs::write(
        dir.path().join("catalogs.toml"),
        "name = \"catalogs\"\nmodelled_on = \"catalogs\"\ntitle = \"[MY CATALOGS]\"\n",
    )
    .unwrap();

    let reg = KindRegistry::load(dir.path());
    assert_eq!(reg.effective("mainframe-editor").title, "[MF-EDIT]");
    assert_eq!(
        reg.resolve_base("mainframe-editor"),
        BuiltinKind::Editor,
        "a user kind resolves to its built-in base"
    );
    assert_eq!(
        reg.effective("catalogs").title,
        "[MY CATALOGS]",
        "a user file named like a built-in overrides its config"
    );
}

/// Validates: workspace-kinds Requirement 2.3 -- an unparseable user file is
/// skipped with a notice and does not crash; built-ins remain intact.
#[test]
fn registry_load_skips_unparseable_with_notice() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("broken.toml"), "this is = not valid = toml").unwrap();
    let reg = KindRegistry::load(dir.path());
    assert!(
        reg.notices().iter().any(|n| n.contains("broken")),
        "an unparseable user file must produce a notice"
    );
    // Built-ins still resolve.
    assert_eq!(reg.effective("editor").title, "[EDITOR]");
}

/// Validates: workspace-kinds Requirement 1.2 -- an External-based user Kind is
/// kept (round-trip) but flagged with a notice and resolves to a safe fallback
/// (v1: external bases are unresolved).
#[test]
fn registry_load_flags_external_base_and_falls_back() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("lua-kind.toml"),
        "name = \"lua-kind\"\nmodelled_on = \"ext:my-provider\"\ntitle = \"[LUA]\"\n",
    )
    .unwrap();
    let reg = KindRegistry::load(dir.path());
    assert_eq!(reg.effective("lua-kind").title, "[LUA]");
    assert_eq!(
        reg.resolve_base("lua-kind"),
        BuiltinKind::Pom,
        "an unresolved external base falls back to a safe built-in"
    );
    assert!(
        reg.notices().iter().any(|n| n.contains("lua-kind")),
        "an external base must be flagged in v1"
    );
}

/// Validates: workspace-kinds Requirement 2.3 -- an absent user directory is
/// silent (no notices, built-ins intact).
#[test]
fn registry_load_absent_dir_is_silent() {
    let reg = KindRegistry::load(std::path::Path::new("/no/such/dir/at/all"));
    assert!(reg.notices().is_empty());
    assert_eq!(reg.effective("pom").title, "[POM]");
}

// === CR-NR-095: per-Kind command-line position (Top | Bottom) ===============

/// Validates: workspace-kinds Requirement 8.1 -- the default command-line
/// position is Top so every existing Kind's layout is unchanged.
#[test]
fn command_line_position_defaults_to_top() {
    assert_eq!(
        KindProfile::default().command_line_position,
        CommandLinePosition::Top
    );
    // And the compiled built-in default carries Top.
    let cfg = KindConfig::builtin_default(BuiltinKind::Editor);
    assert_eq!(cfg.profile.command_line_position, CommandLinePosition::Top);
}

/// Validates: workspace-kinds Requirement 8.2 -- `command_line_position`
/// round-trips through the Kind TOML with the stable `"top"`/`"bottom"` spelling,
/// and (via `#[serde(default)]`) a file with no such key loads as Top.
#[test]
fn kind_profile_toml_roundtrips_command_line_position() {
    // Bottom serialises to "bottom" and round-trips.
    let mut cfg = KindConfig::builtin_default(BuiltinKind::Editor);
    cfg.profile.command_line_position = CommandLinePosition::Bottom;
    let toml_str = toml::to_string(&KindConfigToml::from(&cfg)).expect("serialise");
    assert!(
        toml_str.contains("command_line_position = \"bottom\""),
        "expected stable lowercase spelling in TOML: {toml_str}"
    );
    let back: KindConfig = toml::from_str::<KindConfigToml>(&toml_str)
        .expect("parse")
        .into();
    assert_eq!(
        back.profile.command_line_position,
        CommandLinePosition::Bottom
    );

    // A profile table WITHOUT the key loads as Top (serde default; no schema break).
    let legacy = r#"
name = "editor"
modelled_on = "editor"
title = "[EDIT]"

[profile]
tab_size = 8
line_end_mode = "default"
"#;
    let parsed: KindConfig = toml::from_str::<KindConfigToml>(legacy)
        .expect("parse legacy")
        .into();
    assert_eq!(
        parsed.profile.command_line_position,
        CommandLinePosition::Top,
        "a Kind file predating the attribute must load as Top"
    );
}
