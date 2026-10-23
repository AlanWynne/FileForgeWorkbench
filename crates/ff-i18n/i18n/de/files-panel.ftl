# German (de) Files Panel (catalog tree + content area) (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/files-panel.ftl; only prose translated. Product name kept (Req 4.2); the
# { $platform } name is a non-translated argument (Req 9.2). Em dash / ellipsis
# use unicode-escape placeables so source stays ASCII.

# --- Shared labels ----------------------------------------------------------
files-panel-filter-label = Filter:
files-panel-filter-hint = Name{ "\u2026" }
files-panel-ctx-properties = Eigenschaften

# --- Title bar --------------------------------------------------------------
files-panel-title-bar = FileForge Workbench { "\u2014" } Virtuelle Dateikataloge

# --- Toolbar ----------------------------------------------------------------
files-panel-btn-new-catalog = Neuer Katalog
files-panel-btn-open = { "\u00d6" }ffnen
files-panel-btn-refresh = Aktualisieren
files-panel-btn-properties = Eigenschaften

# --- Command field ----------------------------------------------------------
files-panel-command-label = Command ===>

# --- Tree section headers ---------------------------------------------------
files-panel-section-mainframe = Mainframe-Kataloge
files-panel-section-posix = POSIX-Kataloge
files-panel-section-native = Native Kataloge ({ $platform })

# --- Empty-section message --------------------------------------------------
files-panel-section-empty = { "  " }Keine Kataloge definiert { "\u2014" } klicken Sie auf Neuer Katalog, um einen zu erstellen

# --- Catalog-row context menu -----------------------------------------------
files-panel-ctx-allocate-dataset = Dataset zuweisen
files-panel-ctx-delete-catalog = Katalog l{ "\u00f6" }schen

# --- Content area -----------------------------------------------------------
files-panel-content-no-catalog = W{ "\u00e4" }hlen Sie einen Katalog, um seinen Inhalt zu durchsuchen.
files-panel-content-no-entries = Keine Eintr{ "\u00e4" }ge zum Anzeigen.

# Content-area sortable column headers.
files-panel-col-name = Name
files-panel-col-type = Typ
files-panel-col-size = Gr{ "\u00f6" }{ "\u00df" }e
files-panel-col-modified = Ge{ "\u00e4" }ndert
