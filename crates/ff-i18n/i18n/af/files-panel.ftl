# Afrikaans (af) Files Panel keys (CR-NR-103 Task 10). REVIEWED POC locale.
# Keys + { $placeables } identical to en/files-panel.ftl; only prose translated.
# Non-ASCII Afrikaans letters via Fluent unicode escapes; em dash { "\u2014" },
# ellipsis { "\u2026" }, alignment spaces { "  " } preserved as in en.
# "FileForge Workbench" and the { $platform } name stay as-is (proper nouns).

# --- Shared labels ----------------------------------------------------------
files-panel-filter-label = Filter:
files-panel-filter-hint = naam{ "\u2026" }
files-panel-ctx-properties = Eienskappe

# --- Title bar --------------------------------------------------------------
files-panel-title-bar = FileForge Workbench { "\u2014" } Virtuele L{ "\u00ea" }erkatalogusse

# --- Toolbar ----------------------------------------------------------------
files-panel-btn-new-catalog = Nuwe Katalogus
files-panel-btn-open = Maak oop
files-panel-btn-refresh = Verfris
files-panel-btn-properties = Eienskappe

# --- Command field (prompt chrome; verb stays English) ----------------------
files-panel-command-label = Command ===>

# --- Tree section headers ---------------------------------------------------
files-panel-section-mainframe = Hoofraam-katalogusse
files-panel-section-posix = POSIX-katalogusse
files-panel-section-native = Inheemse katalogusse ({ $platform })

# --- Empty-section message --------------------------------------------------
files-panel-section-empty = { "  " }Geen katalogusse gedefinieer nie { "\u2014" } klik Nuwe Katalogus om een te skep

# --- Catalog-row context menu -----------------------------------------------
files-panel-ctx-allocate-dataset = Ken datastel toe
files-panel-ctx-delete-catalog = Skrap Katalogus

# --- Content area -----------------------------------------------------------
files-panel-content-no-catalog = Kies 'n katalogus om sy inhoud te blaai.
files-panel-content-no-entries = Geen inskrywings om te vertoon nie.

# Content-area sortable column headers.
files-panel-col-name = Naam
files-panel-col-type = Tipe
files-panel-col-size = Grootte
files-panel-col-modified = Gewysig
