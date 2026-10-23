# French (fr) Files Panel (catalog tree + content area) (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/files-panel.ftl; only prose translated. Product name kept (Req 4.2); the
# { $platform } name is a non-translated argument (Req 9.2). Em dash / ellipsis
# use unicode-escape placeables so source stays ASCII.

# --- Shared labels ----------------------------------------------------------
files-panel-filter-label = Filtre :
files-panel-filter-hint = nom{ "\u2026" }
files-panel-ctx-properties = Propri{ "\u00e9" }t{ "\u00e9" }s

# --- Title bar --------------------------------------------------------------
files-panel-title-bar = FileForge Workbench { "\u2014" } Catalogues de fichiers virtuels

# --- Toolbar ----------------------------------------------------------------
files-panel-btn-new-catalog = Nouveau catalogue
files-panel-btn-open = Ouvrir
files-panel-btn-refresh = Actualiser
files-panel-btn-properties = Propri{ "\u00e9" }t{ "\u00e9" }s

# --- Command field ----------------------------------------------------------
files-panel-command-label = Command ===>

# --- Tree section headers ---------------------------------------------------
files-panel-section-mainframe = Catalogues Mainframe
files-panel-section-posix = Catalogues POSIX
files-panel-section-native = Catalogues Natif ({ $platform })

# --- Empty-section message --------------------------------------------------
files-panel-section-empty = { "  " }Aucun catalogue d{ "\u00e9" }fini { "\u2014" } cliquez sur Nouveau catalogue pour en cr{ "\u00e9" }er un

# --- Catalog-row context menu -----------------------------------------------
files-panel-ctx-allocate-dataset = Allouer un dataset
files-panel-ctx-delete-catalog = Supprimer le catalogue

# --- Content area -----------------------------------------------------------
files-panel-content-no-catalog = S{ "\u00e9" }lectionnez un catalogue pour parcourir son contenu.
files-panel-content-no-entries = Aucune entr{ "\u00e9" }e { "\u00e0" } afficher.

# Content-area sortable column headers.
files-panel-col-name = Nom
files-panel-col-type = Type
files-panel-col-size = Taille
files-panel-col-modified = Modifi{ "\u00e9" }
