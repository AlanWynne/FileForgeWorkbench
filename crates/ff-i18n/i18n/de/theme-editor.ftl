# German (de) Theme Editor Context (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/theme-editor.ftl; only the DISPLAYED prose translated. The token English
# `label` remains the stable identity; only the displayed text localises. NOT
# localised (Req 4.4): egui id_salt, the ThemeEditorAction enum, hex values.
# { $name } / { $pair } / { $ratio } are placeables.

# --- Theme selector + lifecycle buttons -------------------------------------
theme-editor-label-theme = Theme:
theme-editor-selected-none = (keins)
theme-editor-btn-set-active = Als aktiv festlegen
theme-editor-btn-reset = Auf integriertes zur{ "\u00fc" }cksetzen
theme-editor-reset-confirm = '{ $name }' auf seine Basislinie zur{ "\u00fc" }cksetzen und { "\u00c4" }nderungen verwerfen?
theme-editor-btn-confirm-reset = Zur{ "\u00fc" }cksetzen best{ "\u00e4" }tigen
theme-editor-btn-cancel = Abbrechen

# --- Copy / Save As / Save + Import / Export --------------------------------
theme-editor-label-new-name = Neuer Name:
theme-editor-btn-copy = Kopieren
theme-editor-btn-save-as = Speichern unter
theme-editor-btn-save = Speichern
theme-editor-btn-export = Exportieren...
theme-editor-btn-import = Importieren...

# --- Colour token editor ----------------------------------------------------
theme-editor-hex-invalid = ung{ "\u00fc" }ltiges Hex
theme-editor-empty = W{ "\u00e4" }hlen Sie ein Theme, um seine Farben zu bearbeiten.

# --- Contrast advisory ------------------------------------------------------
theme-editor-advisory-heading = Kontrasthinweise (unter AA):
theme-editor-advisory-row = { $pair }: Kontrast { $ratio }:1 (unter AA 4.5:1)

# --- Colour-token labels (displayed form of the EditableToken surface) -------
# egui chrome
theme-editor-token-chrome-panel-fill = Chrom: Panel-F{ "\u00fc" }llung
theme-editor-token-chrome-window-fill = Chrom: Fenster-F{ "\u00fc" }llung
theme-editor-token-chrome-panel-border = Chrom: Panel-Rahmen
theme-editor-token-chrome-extreme-input-bg = Chrom: extremer (Eingabe-)Hintergrund
theme-editor-token-chrome-raised-button-bg = Chrom: erhabener (Schaltfl{ "\u00e4" }chen-)Hintergrund
theme-editor-token-chrome-hovered-bg = Chrom: Hover-Hintergrund
theme-editor-token-chrome-menu-bar-text = Chrom: Men{ "\u00fc" }leistentext
theme-editor-token-chrome-input-foreground = Chrom: Eingabe-Vordergrund
theme-editor-token-chrome-input-border = Chrom: Eingabe-Rahmen
theme-editor-token-chrome-focus-ring = Chrom: Fokusring
theme-editor-token-chrome-accent = Chrom: Akzent
theme-editor-token-chrome-title-band-bg = Chrom: Titelband-Hintergrund
theme-editor-token-chrome-tab-active-bg = Chrom: Tab aktiv Hintergrund
theme-editor-token-chrome-tab-inactive-bg = Chrom: Tab inaktiv Hintergrund
theme-editor-token-chrome-tab-active-text = Chrom: Tab aktiv Text
theme-editor-token-chrome-tab-inactive-text = Chrom: Tab inaktiv Text
# domain: syntax
theme-editor-token-syntax-keyword = Syntax: Schl{ "\u00fc" }sselwort
theme-editor-token-syntax-comment = Syntax: Kommentar
theme-editor-token-syntax-string = Syntax: Zeichenkette
theme-editor-token-syntax-number = Syntax: Zahl
theme-editor-token-syntax-operator = Syntax: Operator
theme-editor-token-syntax-type = Syntax: Typ
theme-editor-token-syntax-function = Syntax: Funktion
theme-editor-token-syntax-macro = Syntax: Makro
theme-editor-token-syntax-preprocessor = Syntax: Pr{ "\u00e4" }prozessor
theme-editor-token-syntax-default-text = Syntax: Standardtext
# domain: editor content
theme-editor-token-editor-foreground = Editor: Vordergrund
theme-editor-token-editor-muted = Editor: ged{ "\u00e4" }mpft
theme-editor-token-editor-current-line-bg = Editor: Hintergrund der aktuellen Zeile
# domain: gutter
theme-editor-token-gutter-line-number-fg = Randleiste: Zeilennummer (Vordergrund)
theme-editor-token-gutter-line-number-bg = Randleiste: Zeilennummer (Hintergrund)
theme-editor-token-gutter-fold-margin-bg = Randleiste: Faltrand-Hintergrund
theme-editor-token-gutter-fold-margin-fg = Randleiste: Faltrand-Vordergrund
theme-editor-token-gutter-margin-separator = Randleiste: Randtrenner
theme-editor-token-gutter-cursor-row-border = Randleiste: Cursorzeilen-Rahmen
theme-editor-token-gutter-cursor-column = Randleiste: Cursorspalte
# domain: file tree
theme-editor-token-file-tree-binary = Dateibaum: bin{ "\u00e4" }r
theme-editor-token-file-tree-structured = Dateibaum: strukturiert
theme-editor-token-file-tree-text = Dateibaum: Text
theme-editor-token-file-tree-unknown = Dateibaum: unbekannt
theme-editor-token-file-tree-directory = Dateibaum: Verzeichnis
theme-editor-token-file-tree-symlink = Dateibaum: symbolischer Link
# domain: decorations
theme-editor-token-decoration-search-highlight = Dekoration: Suchhervorhebung
theme-editor-token-decoration-error-underline = Dekoration: Fehlerunterstreichung
theme-editor-token-decoration-warning-underline = Dekoration: Warnunterstreichung
theme-editor-token-decoration-info-underline = Dekoration: Info-Unterstreichung
theme-editor-token-decoration-change-added = Dekoration: { "\u00c4" }nderung hinzugef{ "\u00fc" }gt
theme-editor-token-decoration-change-modified = Dekoration: { "\u00c4" }nderung ge{ "\u00e4" }ndert
theme-editor-token-decoration-change-deleted = Dekoration: { "\u00c4" }nderung gel{ "\u00f6" }scht
theme-editor-token-decoration-bookmark = Dekoration: Lesezeichen
# domain: indicators
theme-editor-token-indicator-find-match = Indikator: Suchtreffer
theme-editor-token-indicator-brace-match = Indikator: Klammer passt
theme-editor-token-indicator-brace-mismatch = Indikator: Klammer passt nicht
theme-editor-token-indicator-hotspot-underline = Indikator: Hotspot-Unterstreichung
