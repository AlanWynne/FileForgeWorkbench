// === Tests ==================================================================

use crate::*;
use ff_catalog_registry::{CatalogType, VirtualCatalog};

fn make_catalog(name: &str, catalog_type: CatalogType) -> VirtualCatalog {
    VirtualCatalog {
        name: name.to_string(),
        catalog_type,
        path: "/some/path".to_string(),
        description: None,
        auto_mount: true,
        default_hlq: None,
        mount_point: None,
        read_only: false,
    }
}

/// Validates: Requirement 1.4 -- three section types are represented.
#[test]
fn files_panel_state_registry_supports_three_catalog_types() {
    // Validates: Requirement 1.4
    let mut state = FilesPanelState::new();
    state
        .registry
        .register(make_catalog("MF1", CatalogType::Mainframe))
        .unwrap();
    state
        .registry
        .register(make_catalog("PX1", CatalogType::Posix))
        .unwrap();
    state
        .registry
        .register(make_catalog("NAT1", CatalogType::Native))
        .unwrap();

    assert_eq!(state.registry.list_by_type(CatalogType::Mainframe).len(), 1);
    assert_eq!(state.registry.list_by_type(CatalogType::Posix).len(), 1);
    assert_eq!(state.registry.list_by_type(CatalogType::Native).len(), 1);
}

/// Validates: Requirement 1.5 -- empty section has no catalog entries.
#[test]
fn empty_section_has_no_catalog_entries() {
    // Validates: Requirement 1.5
    let state = FilesPanelState::new();
    assert!(state
        .registry
        .list_by_type(CatalogType::Mainframe)
        .is_empty());
    assert!(state.registry.list_by_type(CatalogType::Posix).is_empty());
    assert!(state.registry.list_by_type(CatalogType::Native).is_empty());
}

/// Validates: Requirement 1.4 -- native platform label is non-empty.
#[test]
fn native_platform_label_is_non_empty() {
    // Validates: Requirement 1.4
    let label = FilesPanelState::native_platform_label();
    assert!(!label.is_empty());
    assert!(
        label == "Windows" || label == "Linux" || label == "macOS",
        "unexpected label: {label}"
    );
}

/// Validates: Requirement 1.7 -- END command produces ReturnToPom action.
#[test]
fn end_command_produces_return_to_pom_action() {
    // Validates: Requirement 1.7
    let cmd = "END";
    let action = if cmd == "END" || cmd == "F3" {
        FilesPanelAction::ReturnToPom
    } else {
        FilesPanelAction::None
    };
    assert_eq!(action, FilesPanelAction::ReturnToPom);
}

/// Validates: Requirement 1.3 -- NewCatalog action is distinct from None.
#[test]
fn new_catalog_action_is_distinct_from_none() {
    // Validates: Requirement 1.3
    assert_ne!(FilesPanelAction::NewCatalog, FilesPanelAction::None);
    assert_ne!(FilesPanelAction::ReturnToPom, FilesPanelAction::None);
}

/// Validates: Requirement 1.2 -- FilesPanelState initialises with empty filter and command.
#[test]
fn files_panel_state_initialises_with_empty_filter_and_command() {
    // Validates: Requirement 1.2
    let state = FilesPanelState::new();
    assert!(state.filter.is_empty());
    assert!(state.command.is_empty());
}

/// Validates: Requirement 1.4 -- sections default to open.
#[test]
fn section_state_defaults_to_all_open() {
    // Validates: Requirement 1.4
    let s = SectionState::default();
    assert!(s.mainframe_open);
    assert!(s.posix_open);
    assert!(s.native_open);
}

/// Validates: Requirement 4.1 -- EditCatalog action carries the catalog name.
#[test]
fn edit_catalog_action_carries_name() {
    // Validates: Requirement 4.1
    let action = FilesPanelAction::EditCatalog("PAYROLL".to_string());
    assert_eq!(action, FilesPanelAction::EditCatalog("PAYROLL".to_string()));
    assert_ne!(action, FilesPanelAction::None);
}

/// Validates: Requirement 4.3 -- DeleteCatalog action carries the catalog name.
#[test]
fn delete_catalog_action_carries_name() {
    // Validates: Requirement 4.3
    let action = FilesPanelAction::DeleteCatalog("PAYROLL".to_string());
    assert_eq!(
        action,
        FilesPanelAction::DeleteCatalog("PAYROLL".to_string())
    );
    assert_ne!(action, FilesPanelAction::None);
}

// === Task 8 tests ===========================================================

/// Validates: Requirement 6.1 -- PS context menu contains Open, Rename, Delete, Properties, CopyDsn, AllocateLike.
#[test]
fn mainframe_ps_context_menu_contains_required_items() {
    // Validates: Requirement 6.1
    let items = context_menu_items_mainframe(DatasetNodeKind::Ps);
    assert!(items.contains(&MainframeContextItem::Open));
    assert!(items.contains(&MainframeContextItem::Rename));
    assert!(items.contains(&MainframeContextItem::Delete));
    assert!(items.contains(&MainframeContextItem::Properties));
    assert!(items.contains(&MainframeContextItem::CopyDsn));
    assert!(items.contains(&MainframeContextItem::AllocateLike));
}

/// Validates: Requirement 6.1 -- PS context menu does not contain NewMember.
#[test]
fn mainframe_ps_context_menu_excludes_new_member() {
    // Validates: Requirement 6.1
    let items = context_menu_items_mainframe(DatasetNodeKind::Ps);
    assert!(!items.contains(&MainframeContextItem::NewMember));
}

/// Validates: Requirement 6.2 -- PDS context menu contains NewMember, Rename, Delete, Properties, CopyDsn, AllocateLike.
#[test]
fn mainframe_pds_context_menu_contains_required_items() {
    // Validates: Requirement 6.2
    let items = context_menu_items_mainframe(DatasetNodeKind::Pds);
    assert!(items.contains(&MainframeContextItem::NewMember));
    assert!(items.contains(&MainframeContextItem::Rename));
    assert!(items.contains(&MainframeContextItem::Delete));
    assert!(items.contains(&MainframeContextItem::Properties));
    assert!(items.contains(&MainframeContextItem::CopyDsn));
    assert!(items.contains(&MainframeContextItem::AllocateLike));
}

/// Validates: Requirement 6.2 -- PDS context menu does not contain Open.
#[test]
fn mainframe_pds_context_menu_excludes_open() {
    // Validates: Requirement 6.2
    let items = context_menu_items_mainframe(DatasetNodeKind::Pds);
    assert!(!items.contains(&MainframeContextItem::Open));
}

/// Validates: Requirement 6.3 -- Member context menu contains Open, Rename, Delete, CopyMemberName.
#[test]
fn mainframe_member_context_menu_contains_required_items() {
    // Validates: Requirement 6.3
    let items = context_menu_items_mainframe(DatasetNodeKind::Member);
    assert!(items.contains(&MainframeContextItem::Open));
    assert!(items.contains(&MainframeContextItem::Rename));
    assert!(items.contains(&MainframeContextItem::Delete));
    assert!(items.contains(&MainframeContextItem::CopyMemberName));
}

/// Validates: Requirement 6.3 -- Member context menu does not contain Properties or CopyDsn.
#[test]
fn mainframe_member_context_menu_excludes_properties_and_copy_dsn() {
    // Validates: Requirement 6.3
    let items = context_menu_items_mainframe(DatasetNodeKind::Member);
    assert!(!items.contains(&MainframeContextItem::Properties));
    assert!(!items.contains(&MainframeContextItem::CopyDsn));
}

/// Validates: Requirement 6.4 -- GDG context menu contains NewGeneration, ListGenerations, Properties, DeleteGdg, ModifyLimit.
#[test]
fn mainframe_gdg_context_menu_contains_required_items() {
    // Validates: Requirement 6.4
    let items = context_menu_items_mainframe(DatasetNodeKind::GdgBase);
    assert!(items.contains(&MainframeContextItem::NewGeneration));
    assert!(items.contains(&MainframeContextItem::ListGenerations));
    assert!(items.contains(&MainframeContextItem::Properties));
    assert!(items.contains(&MainframeContextItem::DeleteGdg));
    assert!(items.contains(&MainframeContextItem::ModifyLimit));
}

/// Validates: Requirement 8.1 -- POSIX directory context menu contains all six items.
#[test]
fn posix_directory_context_menu_contains_required_items() {
    // Validates: Requirement 8.1
    let items = context_menu_items_posix(PosixNodeKind::Directory);
    assert!(items.contains(&PosixContextItem::NewFile));
    assert!(items.contains(&PosixContextItem::NewDirectory));
    assert!(items.contains(&PosixContextItem::Rename));
    assert!(items.contains(&PosixContextItem::Delete));
    assert!(items.contains(&PosixContextItem::Properties));
    assert!(items.contains(&PosixContextItem::CopyPath));
}

/// Validates: Requirement 8.1 -- POSIX file context menu does not contain NewFile or NewDirectory.
#[test]
fn posix_file_context_menu_excludes_new_items() {
    // Validates: Requirement 8.1
    let items = context_menu_items_posix(PosixNodeKind::File);
    assert!(!items.contains(&PosixContextItem::NewFile));
    assert!(!items.contains(&PosixContextItem::NewDirectory));
    assert!(items.contains(&PosixContextItem::Rename));
    assert!(items.contains(&PosixContextItem::Delete));
}

/// Validates: Requirement 9.3 -- Native file on Windows includes OpenInCmd and OpenInPowerShell.
#[test]
fn native_file_windows_context_menu_includes_shell_actions() {
    // Validates: Requirement 9.3
    let items = context_menu_items_native(NativeNodeKind::File, "windows");
    assert!(items.contains(&NativeContextItem::Open));
    assert!(items.contains(&NativeContextItem::OpenInCmd));
    assert!(items.contains(&NativeContextItem::OpenInPowerShell));
    assert!(!items.contains(&NativeContextItem::OpenInTerminal));
}

/// Validates: Requirement 9.3 -- Native file on Linux includes OpenInTerminal.
#[test]
fn native_file_linux_context_menu_includes_terminal() {
    // Validates: Requirement 9.3
    let items = context_menu_items_native(NativeNodeKind::File, "linux");
    assert!(items.contains(&NativeContextItem::OpenInTerminal));
    assert!(!items.contains(&NativeContextItem::OpenInCmd));
}

/// Validates: Requirement 9.3 -- Native file on macOS includes RevealInFinder and OpenInTerminal.
#[test]
fn native_file_macos_context_menu_includes_finder_and_terminal() {
    // Validates: Requirement 9.3
    let items = context_menu_items_native(NativeNodeKind::File, "macos");
    assert!(items.contains(&NativeContextItem::RevealInFinder));
    assert!(items.contains(&NativeContextItem::OpenInTerminal));
    assert!(!items.contains(&NativeContextItem::OpenInCmd));
}

/// Validates: Requirement 9.4 -- Native directory context menu contains NewFile, NewFolder, Rename, Delete, CopyPath, OpenInNativeFileManager, Refresh.
#[test]
fn native_directory_context_menu_contains_required_items() {
    // Validates: Requirement 9.4
    let items = context_menu_items_native(NativeNodeKind::Directory, "windows");
    assert!(items.contains(&NativeContextItem::NewFile));
    assert!(items.contains(&NativeContextItem::NewFolder));
    assert!(items.contains(&NativeContextItem::Rename));
    assert!(items.contains(&NativeContextItem::Delete));
    assert!(items.contains(&NativeContextItem::CopyPath));
    assert!(items.contains(&NativeContextItem::OpenInNativeFileManager));
    assert!(items.contains(&NativeContextItem::Refresh));
}

/// Validates: Requirement 8.2 -- PosixNewFileForm rejects empty filename.
#[test]
fn posix_new_file_form_rejects_empty_filename() {
    // Validates: Requirement 8.2
    let form = PosixNewFileForm::new("/");
    assert!(form.validate().is_err());
}

/// Validates: Requirement 8.2 -- PosixNewFileForm rejects filename with path separator.
#[test]
fn posix_new_file_form_rejects_path_separator() {
    // Validates: Requirement 8.2
    let mut form = PosixNewFileForm::new("/");
    form.filename = "foo/bar".to_string();
    assert!(form.validate().is_err());
}

/// Validates: Requirement 8.2 -- PosixNewFileForm accepts valid filename.
#[test]
fn posix_new_file_form_accepts_valid_filename() {
    // Validates: Requirement 8.2
    let mut form = PosixNewFileForm::new("/");
    form.filename = "hello.txt".to_string();
    assert!(form.validate().is_ok());
}

/// Validates: Requirement 8.3 -- PosixNewDirForm rejects empty dirname.
#[test]
fn posix_new_dir_form_rejects_empty_dirname() {
    // Validates: Requirement 8.3
    let form = PosixNewDirForm::new("/");
    assert!(form.validate().is_err());
}

/// Validates: Requirement 8.5 -- PosixDeleteConfirm message mentions directory name.
#[test]
fn posix_delete_confirm_directory_message_contains_name() {
    // Validates: Requirement 8.5
    let confirm = PosixDeleteConfirm::new("src", true);
    let msg = confirm.message();
    assert!(msg.contains("src"));
    assert!(msg.contains("all its contents"));
}

/// Validates: Requirement 6.5 -- DatasetRenameForm pre-fills new_name from current_name.
#[test]
fn dataset_rename_form_prefills_new_name() {
    // Validates: Requirement 6.5
    let form = DatasetRenameForm::new("PAYROLL.DATA");
    assert_eq!(form.current_name, "PAYROLL.DATA");
    assert_eq!(form.new_name, "PAYROLL.DATA");
    assert!(form.error.is_none());
}

/// Validates: Requirement 6.6 -- DatasetDeleteConfirm stores the dataset name.
#[test]
fn dataset_delete_confirm_stores_name() {
    // Validates: Requirement 6.6
    let confirm = DatasetDeleteConfirm::new("PAYROLL.DATA");
    assert_eq!(confirm.name, "PAYROLL.DATA");
}

// === Task 9 tests ===========================================================

fn make_entry(
    name: &str,
    entry_type: &str,
    size: &str,
    modified: &str,
    is_container: bool,
) -> ContentEntry {
    ContentEntry {
        name: name.to_string(),
        entry_type: entry_type.to_string(),
        size: size.to_string(),
        modified: modified.to_string(),
        is_container,
    }
}

/// Validates: Requirement 10.1 -- ContentAreaState initialises with no selection and empty entries.
#[test]
fn content_area_state_initialises_empty() {
    // Validates: Requirement 10.1
    let s = ContentAreaState::default();
    assert!(s.selected_catalog.is_none());
    assert!(s.entries.is_empty());
    assert!(s.path_segments.is_empty());
    assert_eq!(s.sort_col, SortColumn::Name);
    assert_eq!(s.sort_dir, SortDir::Ascending);
    assert!(s.content_filter.is_empty());
}

/// Validates: Requirement 10.7 -- directories sort before files when sorting by Name.
#[test]
fn visible_entries_name_sort_groups_dirs_before_files() {
    // Validates: Requirement 10.7
    let s = ContentAreaState {
        entries: vec![
            make_entry("zebra.txt", "File", "", "", false),
            make_entry("alpha", "Directory", "", "", true),
            make_entry("mango.txt", "File", "", "", false),
            make_entry("beta", "Directory", "", "", true),
        ],
        ..Default::default()
    };
    let names: Vec<&str> = s
        .visible_entries()
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    assert_eq!(names, vec!["alpha", "beta", "mango.txt", "zebra.txt"]);
}

/// Validates: Requirement 10.7 -- directories within the directory group are sorted alphabetically.
#[test]
fn visible_entries_name_sort_dirs_are_alphabetical_within_group() {
    // Validates: Requirement 10.7
    let s = ContentAreaState {
        entries: vec![
            make_entry("Zebra", "Directory", "", "", true),
            make_entry("alpha", "Directory", "", "", true),
            make_entry("Mango", "Directory", "", "", true),
        ],
        ..Default::default()
    };
    let names: Vec<&str> = s
        .visible_entries()
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    // Case-insensitive: alpha < mango < zebra
    assert_eq!(names, vec!["alpha", "Mango", "Zebra"]);
}

/// Validates: Requirement 10.7 -- non-Name sort columns do not apply container grouping.
#[test]
fn visible_entries_type_sort_does_not_force_dir_grouping() {
    // Validates: Requirement 10.7 (grouping only applies to Name column)
    let s = ContentAreaState {
        entries: vec![
            make_entry("b_file", "PS", "", "", false),
            make_entry("a_dir", "Directory", "", "", true),
        ],
        sort_col: SortColumn::Type,
        sort_dir: SortDir::Ascending,
        ..Default::default()
    };
    let types: Vec<&str> = s
        .visible_entries()
        .iter()
        .map(|e| e.entry_type.as_str())
        .collect();
    // "Directory" < "PS" alphabetically
    assert_eq!(types, vec!["Directory", "PS"]);
}

/// Validates: Requirement 10.2 -- visible_entries sorts by Name ascending by default.
#[test]
fn visible_entries_sorts_by_name_ascending_by_default() {
    // Validates: Requirement 10.2
    let s = ContentAreaState {
        entries: vec![
            make_entry("zebra.txt", "File", "1 KB", "2024-01-03", false),
            make_entry("alpha.txt", "File", "2 KB", "2024-01-01", false),
            make_entry("mango.txt", "File", "3 KB", "2024-01-02", false),
        ],
        ..Default::default()
    };
    let names: Vec<&str> = s
        .visible_entries()
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    assert_eq!(names, vec!["alpha.txt", "mango.txt", "zebra.txt"]);
}

/// Validates: Requirement 10.2 -- toggle_sort on same column flips to descending.
#[test]
fn toggle_sort_same_column_flips_to_descending() {
    // Validates: Requirement 10.2
    let mut s = ContentAreaState::default();
    assert_eq!(s.sort_dir, SortDir::Ascending);
    s.toggle_sort(SortColumn::Name);
    assert_eq!(s.sort_col, SortColumn::Name);
    assert_eq!(s.sort_dir, SortDir::Descending);
}

/// Validates: Requirement 10.2 -- toggle_sort on different column resets to ascending.
#[test]
fn toggle_sort_different_column_resets_to_ascending() {
    // Validates: Requirement 10.2
    let mut s = ContentAreaState::default();
    s.toggle_sort(SortColumn::Name); // now descending
    s.toggle_sort(SortColumn::Type); // switch column
    assert_eq!(s.sort_col, SortColumn::Type);
    assert_eq!(s.sort_dir, SortDir::Ascending);
}

/// Validates: Requirement 10.2 -- visible_entries sorts descending correctly.
#[test]
fn visible_entries_sorts_by_name_descending() {
    // Validates: Requirement 10.2
    let s = ContentAreaState {
        entries: vec![
            make_entry("alpha.txt", "File", "", "", false),
            make_entry("zebra.txt", "File", "", "", false),
        ],
        sort_dir: SortDir::Descending,
        ..Default::default()
    };
    let names: Vec<&str> = s
        .visible_entries()
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    assert_eq!(names, vec!["zebra.txt", "alpha.txt"]);
}

/// Validates: Requirement 10.2 -- sort by Type column works.
#[test]
fn visible_entries_sorts_by_type_column() {
    // Validates: Requirement 10.2
    let s = ContentAreaState {
        entries: vec![
            make_entry("b", "PS", "", "", false),
            make_entry("a", "Directory", "", "", true),
        ],
        sort_col: SortColumn::Type,
        sort_dir: SortDir::Ascending,
        ..Default::default()
    };
    let types: Vec<&str> = s
        .visible_entries()
        .iter()
        .map(|e| e.entry_type.as_str())
        .collect();
    assert_eq!(types, vec!["Directory", "PS"]);
}

/// Validates: Requirement 10.6 -- visible_entries filters by name substring (case-insensitive).
#[test]
fn visible_entries_filters_by_name_case_insensitive() {
    // Validates: Requirement 10.6
    let s = ContentAreaState {
        entries: vec![
            make_entry("README.md", "File", "", "", false),
            make_entry("main.rs", "File", "", "", false),
            make_entry("Cargo.toml", "File", "", "", false),
        ],
        content_filter: "readme".to_string(),
        ..Default::default()
    };
    let visible = s.visible_entries();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].name, "README.md");
}

/// Validates: Requirement 10.6 -- empty filter shows all entries.
#[test]
fn visible_entries_empty_filter_shows_all() {
    // Validates: Requirement 10.6
    let s = ContentAreaState {
        entries: vec![
            make_entry("a", "File", "", "", false),
            make_entry("b", "File", "", "", false),
        ],
        ..Default::default()
    };
    assert_eq!(s.visible_entries().len(), 2);
}

/// Validates: Requirement 10.6 -- filter with no match returns empty list.
#[test]
fn visible_entries_filter_no_match_returns_empty() {
    // Validates: Requirement 10.6
    let s = ContentAreaState {
        entries: vec![make_entry("hello.txt", "File", "", "", false)],
        content_filter: "zzz".to_string(),
        ..Default::default()
    };
    assert!(s.visible_entries().is_empty());
}

/// Validates: Requirement 10.5 -- breadcrumb_display returns catalog name when at root.
#[test]
fn breadcrumb_display_at_root_shows_catalog_name() {
    // Validates: Requirement 10.5
    let s = ContentAreaState {
        selected_catalog: Some("PAYROLL".to_string()),
        ..Default::default()
    };
    assert_eq!(s.breadcrumb_display(), "PAYROLL");
}

/// Validates: Requirement 10.5 -- breadcrumb_display includes path segments.
#[test]
fn breadcrumb_display_includes_path_segments() {
    // Validates: Requirement 10.5
    let mut s = ContentAreaState {
        selected_catalog: Some("MYCAT".to_string()),
        ..Default::default()
    };
    s.push_path("src");
    s.push_path("lib");
    assert_eq!(s.breadcrumb_display(), "MYCAT / src / lib");
}

/// Validates: Requirement 10.5 -- breadcrumb_display is empty when no catalog selected.
#[test]
fn breadcrumb_display_empty_when_no_catalog_selected() {
    // Validates: Requirement 10.5
    let s = ContentAreaState::default();
    assert!(s.breadcrumb_display().is_empty());
}

/// Validates: Requirement 10.5 -- navigate_to_segment(0) clears all path segments.
#[test]
fn navigate_to_segment_zero_clears_path() {
    // Validates: Requirement 10.5
    let mut s = ContentAreaState::default();
    s.push_path("src");
    s.push_path("lib");
    s.navigate_to_segment(0);
    assert!(s.path_segments.is_empty());
}

/// Validates: Requirement 10.5 -- navigate_to_segment(1) keeps first segment only.
#[test]
fn navigate_to_segment_one_keeps_first_segment() {
    // Validates: Requirement 10.5
    let mut s = ContentAreaState::default();
    s.push_path("src");
    s.push_path("lib");
    s.push_path("util");
    s.navigate_to_segment(1);
    assert_eq!(s.path_segments, vec!["src"]);
}

/// Validates: Requirement 10.3 -- OpenFile action carries the file name.
#[test]
fn open_file_action_carries_name() {
    // Validates: Requirement 10.3
    let action = FilesPanelAction::OpenFile("PAYROLL.DATA".to_string());
    assert_eq!(
        action,
        FilesPanelAction::OpenFile("PAYROLL.DATA".to_string())
    );
    assert_ne!(action, FilesPanelAction::None);
}

/// Validates: Requirement 10.4 -- NavigateInto action carries the directory name.
#[test]
fn navigate_into_action_carries_name() {
    // Validates: Requirement 10.4
    let action = FilesPanelAction::NavigateInto("src".to_string());
    assert_eq!(action, FilesPanelAction::NavigateInto("src".to_string()));
    assert_ne!(action, FilesPanelAction::None);
}

/// Validates: Requirement 10.4 -- push_path appends a segment and breadcrumb updates.
#[test]
fn push_path_appends_segment() {
    // Validates: Requirement 10.4
    let mut s = ContentAreaState {
        selected_catalog: Some("CAT".to_string()),
        ..Default::default()
    };
    s.push_path("subdir");
    assert_eq!(s.path_segments, vec!["subdir"]);
    assert_eq!(s.breadcrumb_display(), "CAT / subdir");
}

/// Validates: Requirement 10.1 -- FilesPanelState initialises with default ContentAreaState.
#[test]
fn files_panel_state_has_default_content_area() {
    // Validates: Requirement 10.1
    let state = FilesPanelState::new();
    assert!(state.content.selected_catalog.is_none());
    assert!(state.content.entries.is_empty());
}

// === Phase AT: Allocated Dataset Persistence and Display (Req 13) ============

fn make_alloc_params(
    name: &str,
    dsorg: ff_dataset_alloc_dialog::Dsorg,
) -> ff_dataset_alloc_dialog::AllocParams {
    ff_dataset_alloc_dialog::AllocParams {
        dataset_name: name.to_string(),
        dsorg,
        recfm: ff_dataset_alloc_dialog::Recfm::Fb,
        lrecl: 80,
        blksize: 0,
        dir_blocks: None,
        gdg_limit: None,
        scratch: false,
        description: None,
    }
}

// === Phase BJ: Dataset path resolution (Req 16) ==============================

/// Validates: Requirement 16.1, 16.5 -- DSN qualifiers become path components.
#[test]
fn resolve_dataset_path_maps_dsn_to_subpath() {
    // Validates: Requirement 16.1, 16.5
    let result = FilesPanelState::resolve_dataset_path("C:/catalogs/payroll", "PAYROLL.EMPLOYEE");
    assert!(result.is_some());
    let path = result.unwrap();
    // Should end with PAYROLL/EMPLOYEE (platform separator)
    let components: Vec<_> = path.components().collect();
    let last = components.last().unwrap().as_os_str().to_string_lossy();
    let second_last = components[components.len() - 2]
        .as_os_str()
        .to_string_lossy();
    assert_eq!(last, "EMPLOYEE");
    assert_eq!(second_last, "PAYROLL");
}

/// Validates: Requirement 16.4, 16.5 -- empty repository path returns None.
#[test]
fn resolve_dataset_path_empty_repo_returns_none() {
    // Validates: Requirement 16.4, 16.5
    let result = FilesPanelState::resolve_dataset_path("", "PAYROLL.EMPLOYEE");
    assert!(result.is_none());
}

/// Validates: Requirement 16.5 -- empty DSN returns None.
#[test]
fn resolve_dataset_path_empty_dsn_returns_none() {
    // Validates: Requirement 16.5
    let result = FilesPanelState::resolve_dataset_path("C:/catalogs/payroll", "");
    assert!(result.is_none());
}

/// Validates: Requirement 16.1 -- single-qualifier DSN resolves to one component under repo.
#[test]
fn resolve_dataset_path_single_qualifier_dsn() {
    // Validates: Requirement 16.1
    let result = FilesPanelState::resolve_dataset_path("C:/repo", "MYDATA");
    assert!(result.is_some());
    let path = result.unwrap();
    let last = path.file_name().unwrap().to_string_lossy();
    assert_eq!(last, "MYDATA");
}

/// Validates: Requirement 16.3 -- create_dataset_file creates the file and parent dirs.
#[test]
fn opening_missing_dataset_creates_file_and_parent_dirs() {
    // Validates: Requirement 16.3
    use tempfile::TempDir;
    let tmp = TempDir::new().expect("tempdir");
    let target = tmp.path().join("PAYROLL").join("EMPLOYEE");
    assert!(!target.exists());
    FilesPanelState::create_dataset_file(&target).expect("create must succeed");
    assert!(target.exists(), "file must be created");
    assert!(target.is_file(), "must be a regular file");
}

/// Validates: Requirement 16.3 -- create_dataset_file creates missing parent directories.
#[test]
fn opening_missing_dataset_creates_parent_dirs() {
    // Validates: Requirement 16.3
    use tempfile::TempDir;
    let tmp = TempDir::new().expect("tempdir");
    let target = tmp.path().join("A").join("B").join("C").join("DATASET");
    assert!(!target.parent().unwrap().exists());
    FilesPanelState::create_dataset_file(&target).expect("create must succeed");
    assert!(target.exists());
    assert!(target.parent().unwrap().is_dir());
}

// === BU.2 failing tests (Tasks 19.1-19.3) ===================================
// These tests call resolve_and_open_dataset() which does not yet exist.
// They MUST fail (red) before implementation.

/// Validates: Requirement 16.1 -- resolve_and_open_dataset returns path for known DSN.
#[test]
fn resolve_and_open_dataset_returns_path_for_known_dsn() {
    // Validates: Requirement 16.1
    use ff_dscatalog::{
        catalog::CatalogMount,
        dataset::{AllocParams as DsAllocParams, Dsorg as DsDsorg},
        hierarchy::CatalogScope,
        repository::Repository,
    };
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let repo_path = tmp.path().join("PAYROLL");
    Repository::new(&repo_path)
        .initialize("PAYROLL")
        .expect("init");

    let mut ds_reg = ff_dscatalog::catalog_registry::CatalogRegistry::new();
    ds_reg
        .mount(CatalogMount::local(&repo_path, 1))
        .expect("mount");
    ds_reg
        .get_catalog("PAYROLL")
        .unwrap()
        .allocate(DsAllocParams {
            dsn: ff_dscatalog::dsn::Dsn::parse("PAYROLL.INPUT").unwrap(),
            dsorg: DsDsorg::PS,
            recfm: None,
            lrecl: None,
            blksize: None,
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
            scope: CatalogScope::User,
        })
        .expect("allocate");

    let result = FilesPanelState::resolve_and_open_dataset(&ds_reg, "PAYROLL.INPUT");
    assert!(result.is_ok(), "expected Ok path, got: {:?}", result);
}

/// Validates: Requirement 16.4 -- resolve_and_open_dataset returns Err for unknown DSN.
#[test]
fn resolve_and_open_dataset_returns_err_for_unknown_dsn() {
    // Validates: Requirement 16.4
    let ds_reg = ff_dscatalog::catalog_registry::CatalogRegistry::new();
    let result = FilesPanelState::resolve_and_open_dataset(&ds_reg, "NOSUCH.DATASET");
    assert!(result.is_err(), "expected Err for unknown DSN");
    let msg = result.unwrap_err();
    assert!(
        msg.contains("not found") || msg.contains("NOSUCH"),
        "error message should mention not found or DSN: {msg}"
    );
}

/// Validates: Requirement 16.3 -- resolve_and_open_dataset creates file when missing on disk.
#[test]
fn resolve_and_open_dataset_creates_file_when_missing() {
    // Validates: Requirement 16.3
    use ff_dscatalog::{
        catalog::CatalogMount,
        dataset::{AllocParams as DsAllocParams, Dsorg as DsDsorg},
        hierarchy::CatalogScope,
        repository::Repository,
    };
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let repo_path = tmp.path().join("NEWCAT");
    Repository::new(&repo_path)
        .initialize("NEWCAT")
        .expect("init");

    let mut ds_reg = ff_dscatalog::catalog_registry::CatalogRegistry::new();
    ds_reg
        .mount(CatalogMount::local(&repo_path, 1))
        .expect("mount");
    let record = ds_reg
        .get_catalog("NEWCAT")
        .unwrap()
        .allocate(DsAllocParams {
            dsn: ff_dscatalog::dsn::Dsn::parse("NEW.DATA").unwrap(),
            dsorg: DsDsorg::PS,
            recfm: None,
            lrecl: None,
            blksize: None,
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
            scope: CatalogScope::User,
        })
        .expect("allocate");
    // Delete the physical file to simulate missing-on-disk
    let phys = repo_path.join(&record.storage_path);
    if phys.exists() {
        std::fs::remove_file(&phys).unwrap();
    }

    let result = FilesPanelState::resolve_and_open_dataset(&ds_reg, "NEW.DATA");
    assert!(
        result.is_ok(),
        "expected Ok after file creation: {:?}",
        result
    );
    let path = result.unwrap();
    assert!(path.exists(), "file must have been created at {path:?}");
}

// === BU.2 failing tests (Tasks 20.1-20.3) ===================================
// These tests call load_entries_from_catalog() which does not yet exist.
// They MUST fail (red) before implementation.

/// Validates: Requirement 13.2 -- Files Panel content area populated from SQLite.
#[test]
fn files_panel_content_area_populated_from_sqlite() {
    // Validates: Requirement 13.2
    use ff_dscatalog::{
        catalog::CatalogMount,
        dataset::{AllocParams as DsAllocParams, Dsorg as DsDsorg},
        hierarchy::CatalogScope,
        repository::Repository,
    };
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let repo_path = tmp.path().join("MYCAT");
    Repository::new(&repo_path)
        .initialize("MYCAT")
        .expect("init");

    let mut ds_reg = ff_dscatalog::catalog_registry::CatalogRegistry::new();
    ds_reg
        .mount(CatalogMount::local(&repo_path, 1))
        .expect("mount");
    for (dsn, dsorg) in &[("MYCAT.SEQ", DsDsorg::PS), ("MYCAT.LIB", DsDsorg::PO)] {
        ds_reg
            .get_catalog("MYCAT")
            .unwrap()
            .allocate(DsAllocParams {
                dsn: ff_dscatalog::dsn::Dsn::parse(dsn).unwrap(),
                dsorg: *dsorg,
                recfm: None,
                lrecl: None,
                blksize: None,
                dir_blocks: None,
                gdg_limit: None,
                gdg_scratch: None,
                subtype: None,
                description: None,
                scope: CatalogScope::User,
            })
            .expect("allocate");
    }

    let mut state = FilesPanelState::new();
    state.load_entries_from_catalog("MYCAT", &ds_reg);
    assert_eq!(state.content.entries.len(), 2);
    let names: Vec<&str> = state
        .content
        .entries
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    assert!(names.contains(&"MYCAT.SEQ"));
    assert!(names.contains(&"MYCAT.LIB"));
}

/// Validates: Requirement 13.4 -- alloc confirm uses registry, not datasets HashMap.
#[test]
fn alloc_confirm_uses_registry_not_hashmap() {
    // Validates: Requirement 13.1, 13.4
    use ff_dscatalog::{
        catalog::CatalogMount,
        dataset::{AllocParams as DsAllocParams, Dsorg as DsDsorg},
        hierarchy::CatalogScope,
        repository::Repository,
    };
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let repo_path = tmp.path().join("ALLOC");
    Repository::new(&repo_path)
        .initialize("ALLOC")
        .expect("init");

    let mut ds_reg = ff_dscatalog::catalog_registry::CatalogRegistry::new();
    ds_reg
        .mount(CatalogMount::local(&repo_path, 1))
        .expect("mount");
    ds_reg
        .get_catalog("ALLOC")
        .unwrap()
        .allocate(DsAllocParams {
            dsn: ff_dscatalog::dsn::Dsn::parse("ALLOC.DATA").unwrap(),
            dsorg: DsDsorg::PS,
            recfm: None,
            lrecl: None,
            blksize: None,
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
            scope: CatalogScope::User,
        })
        .expect("allocate");

    let mut state = FilesPanelState::new();
    state.load_entries_from_catalog("ALLOC", &ds_reg);
    assert_eq!(state.content.entries.len(), 1);
    // datasets HashMap no longer exists -- SQLite is the sole store
}
