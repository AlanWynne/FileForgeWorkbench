# English Identity_Base keys for the File Explorer view context menu
# (CR-NR-103, Localisation Phase 2 Slice 2, task 7.4, crate ff-explorer-view).
#
# These values are the EXACT current English literals from crate
# ff-explorer-view (lib.rs context_menu_ui), byte-for-byte, ASCII only. Render
# code resolves each displayed string through the ff_i18n::t() seam keyed by
# these ids (localization Req 4.1, 10.1).
#
# NOT localised and left as source literals (Req 4.2, 4.4): the row disclosure/
# bullet glyphs (row_glyph), and the clipboard text-tree format produced by
# build_selection_text_tree -- the "[DIR] " prefix and "|-- " connector are a
# fixed interchange/clipboard format defined by file-tree-panel Req 19.5/19.6,
# not translatable UI chrome.

# --- Row context menu -------------------------------------------------------
explorer-view-ctx-open = Open
explorer-view-ctx-copy = Copy
explorer-view-ctx-paste = Paste
explorer-view-ctx-copy-full-path = Copy Full Path
explorer-view-ctx-reveal = Reveal in File Manager
explorer-view-ctx-rename = Rename
explorer-view-ctx-delete = Delete
explorer-view-ctx-new-file = New File
explorer-view-ctx-new-folder = New Folder
