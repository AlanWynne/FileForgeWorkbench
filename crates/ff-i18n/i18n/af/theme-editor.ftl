# Afrikaans (af) Theme Editor Context (CR-NR-103 Task 10). REVIEWED POC.
# The token ENGLISH label stays the stable identity in code; only the DISPLAYED
# text is localised here (via label_key()). { $name }/{ $pair }/{ $ratio } are
# DATA placeables; hex colour values and "AA" / ":1" contrast notation kept.
# Domain prefixes: Chrome->Chroom, Syntax->Sintaksis, Editor->Redigeerder,
# Gutter->Kantlyn, File tree->L{ e }erboom, Decoration->Versiering,
# Indicator->Aanwyser.

# --- Theme selector + lifecycle buttons -------------------------------------
theme-editor-label-theme = Tema:
theme-editor-selected-none = (geen)
theme-editor-btn-set-active = Stel Aktief
theme-editor-btn-reset = Herstel na ingeboude
theme-editor-reset-confirm = Herstel '{ $name }' na sy basislyn en gooi redigerings weg?
theme-editor-btn-confirm-reset = Bevestig herstel
theme-editor-btn-cancel = Kanselleer

# --- Copy / Save As / Save + Import / Export --------------------------------
theme-editor-label-new-name = Nuwe naam:
theme-editor-btn-copy = Kopieer
theme-editor-btn-save-as = Stoor As
theme-editor-btn-save = Stoor
theme-editor-btn-export = Voer uit...
theme-editor-btn-import = Voer in...

# --- Colour token editor ----------------------------------------------------
theme-editor-hex-invalid = ongeldige hex
theme-editor-empty = Kies 'n tema om sy kleure te redigeer.

# --- Contrast advisory ------------------------------------------------------
theme-editor-advisory-heading = Kontraswenke (onder AA):
theme-editor-advisory-row = { $pair }: kontras { $ratio }:1 (onder AA 4.5:1)

# --- Colour-token labels ----------------------------------------------------
# egui chrome
theme-editor-token-chrome-panel-fill = Chroom: paneelvulling
theme-editor-token-chrome-window-fill = Chroom: venstervulling
theme-editor-token-chrome-panel-border = Chroom: paneelrand
theme-editor-token-chrome-extreme-input-bg = Chroom: uiterste (invoer) agtergrond
theme-editor-token-chrome-raised-button-bg = Chroom: verhewe (knoppie) agtergrond
theme-editor-token-chrome-hovered-bg = Chroom: sweef-agtergrond
theme-editor-token-chrome-menu-bar-text = Chroom: kieslysbalkteks
theme-editor-token-chrome-input-foreground = Chroom: invoervoorgrond
theme-editor-token-chrome-input-border = Chroom: invoerrand
theme-editor-token-chrome-focus-ring = Chroom: fokusring
theme-editor-token-chrome-accent = Chroom: aksent
theme-editor-token-chrome-title-band-bg = Chroom: titelbalk-agtergrond
theme-editor-token-chrome-tab-active-bg = Chroom: oortjie aktief agtergrond
theme-editor-token-chrome-tab-inactive-bg = Chroom: oortjie onaktief agtergrond
theme-editor-token-chrome-tab-active-text = Chroom: oortjie aktief teks
theme-editor-token-chrome-tab-inactive-text = Chroom: oortjie onaktief teks
# domain: syntax
theme-editor-token-syntax-keyword = Sintaksis: sleutelwoord
theme-editor-token-syntax-comment = Sintaksis: kommentaar
theme-editor-token-syntax-string = Sintaksis: string
theme-editor-token-syntax-number = Sintaksis: getal
theme-editor-token-syntax-operator = Sintaksis: operator
theme-editor-token-syntax-type = Sintaksis: tipe
theme-editor-token-syntax-function = Sintaksis: funksie
theme-editor-token-syntax-macro = Sintaksis: makro
theme-editor-token-syntax-preprocessor = Sintaksis: voorverwerker
theme-editor-token-syntax-default-text = Sintaksis: verstekteks
# domain: editor content
theme-editor-token-editor-foreground = Redigeerder: voorgrond
theme-editor-token-editor-muted = Redigeerder: gedemp
theme-editor-token-editor-current-line-bg = Redigeerder: huidige re{ "\u00eb" }l agtergrond
# domain: gutter
theme-editor-token-gutter-line-number-fg = Kantlyn: re{ "\u00eb" }lnommer voorgrond
theme-editor-token-gutter-line-number-bg = Kantlyn: re{ "\u00eb" }lnommer agtergrond
theme-editor-token-gutter-fold-margin-bg = Kantlyn: voumarge agtergrond
theme-editor-token-gutter-fold-margin-fg = Kantlyn: voumarge voorgrond
theme-editor-token-gutter-margin-separator = Kantlyn: margeskeier
theme-editor-token-gutter-cursor-row-border = Kantlyn: wyser-ryrand
theme-editor-token-gutter-cursor-column = Kantlyn: wyserkolom
# domain: file tree
theme-editor-token-file-tree-binary = L{ "\u00ea" }erboom: bin{ "\u00ea" }r
theme-editor-token-file-tree-structured = L{ "\u00ea" }erboom: gestruktureer
theme-editor-token-file-tree-text = L{ "\u00ea" }erboom: teks
theme-editor-token-file-tree-unknown = L{ "\u00ea" }erboom: onbekend
theme-editor-token-file-tree-directory = L{ "\u00ea" }erboom: gids
theme-editor-token-file-tree-symlink = L{ "\u00ea" }erboom: simboliese skakel
# domain: decorations
theme-editor-token-decoration-search-highlight = Versiering: soekuitlig
theme-editor-token-decoration-error-underline = Versiering: foutonderstreep
theme-editor-token-decoration-warning-underline = Versiering: waarskuwingonderstreep
theme-editor-token-decoration-info-underline = Versiering: inligtingonderstreep
theme-editor-token-decoration-change-added = Versiering: verandering bygevoeg
theme-editor-token-decoration-change-modified = Versiering: verandering gewysig
theme-editor-token-decoration-change-deleted = Versiering: verandering geskrap
theme-editor-token-decoration-bookmark = Versiering: boekmerk
# domain: indicators
theme-editor-token-indicator-find-match = Aanwyser: vind-passing
theme-editor-token-indicator-brace-match = Aanwyser: hakie-passing
theme-editor-token-indicator-brace-mismatch = Aanwyser: hakie-wanpassing
theme-editor-token-indicator-hotspot-underline = Aanwyser: brandpunt-onderstreep
