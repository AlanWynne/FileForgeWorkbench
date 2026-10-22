# English Identity_Base keys for the Files Panel (catalog tree + content area)
# (CR-NR-103, Localisation Phase 2 Slice 2, task 7.3, crate ff-files-panel).
#
# These values are the EXACT current English literals from crate ff-files-panel
# (tree.rs, content.rs), byte-for-byte. Render code resolves each displayed
# string through the ff_i18n::t()/t_args() seam keyed by these ids (localization
# Req 4.1, 10.1). NOT localised and left as source literals (Req 4.2, 4.4):
# command verbs (the END/F3 command field), enum logic keys (the context-item
# enums, whose display labels are produced in ff-desktop, task 8), egui id_salt
# values, the breadcrumb " / " separator, the sort-direction glyphs, the
# folder/file row icons, the row format/layout string, and the platform name
# (Windows/macOS/Linux -- a proper noun passed as a non-translated argument).
#
# A few values contain non-ASCII typographic characters in the original UI prose
# (an em dash and an ellipsis). The .ftl source stays ASCII (documentation-charset
# rule) by expressing those as Fluent unicode-escape string placeables
# ({ "\u2014" } = em dash, { "\u2026" } = ellipsis), so the RENDERED text remains
# byte-identical to the original literal.

# --- Shared labels (reused across tree + content) ---------------------------
files-panel-filter-label = Filter:
files-panel-filter-hint = name{ "\u2026" }
files-panel-ctx-properties = Properties

# --- Title bar --------------------------------------------------------------
files-panel-title = FileForge Workbench { "\u2014" } Virtual File Catalogs

# --- Toolbar ----------------------------------------------------------------
files-panel-btn-new-catalog = New Catalog
files-panel-btn-open = Open
files-panel-btn-refresh = Refresh
files-panel-btn-properties = Properties

# --- Command field ----------------------------------------------------------
files-panel-command-label = Command ===>

# --- Tree section headers ---------------------------------------------------
files-panel-section-mainframe = Mainframe Catalogs
files-panel-section-posix = POSIX Catalogs
files-panel-section-native = Native Catalogs ({ $platform })

# --- Empty-section message --------------------------------------------------
files-panel-section-empty = { "  " }No catalogs defined { "\u2014" } click New Catalog to create one

# --- Catalog-row context menu -----------------------------------------------
files-panel-ctx-allocate-dataset = Allocate Dataset
files-panel-ctx-delete-catalog = Delete Catalog

# --- Content area -----------------------------------------------------------
files-panel-content-no-catalog = Select a catalog to browse its contents.
files-panel-content-no-entries = No entries to display.

# Content-area sortable column headers.
files-panel-col-name = Name
files-panel-col-type = Type
files-panel-col-size = Size
files-panel-col-modified = Modified
