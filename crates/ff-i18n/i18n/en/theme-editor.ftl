# English Identity_Base keys for the Theme Editor Context
# (CR-NR-103, Localisation Phase 2 Slice 2, task 7.5, crate ff-theme-editor).
#
# These values are the EXACT current English literals from crate ff-theme-editor
# (lib.rs render + editable_surface.rs token labels), byte-for-byte, ASCII only.
# Render code resolves each displayed string through the ff_i18n::t()/t_args()
# seam (localization Req 4.1, 10.1).
#
# The 48 colour-token labels (theme-editor-token-*) are the DISPLAYED form of the
# EditableToken surface. The token's English `label` remains its stable IDENTITY
# (used by PartialEq, the positional all() ordering, and the crate's tests); only
# the DISPLAYED text is localised, via EditableToken::label_key() -> t(). NOT
# localised (Req 4.2, 4.4): egui id_salt values, the ThemeEditorAction enum, and
# hex colour values.

# --- Theme selector + lifecycle buttons -------------------------------------
theme-editor-label-theme = Theme:
theme-editor-selected-none = (none)
theme-editor-btn-set-active = Set Active
theme-editor-btn-reset = Reset to built-in
theme-editor-reset-confirm = Reset '{ $name }' to its baseline and discard edits?
theme-editor-btn-confirm-reset = Confirm reset
theme-editor-btn-cancel = Cancel

# --- Copy / Save As / Save + Import / Export --------------------------------
theme-editor-label-new-name = New name:
theme-editor-btn-copy = Copy
theme-editor-btn-save-as = Save As
theme-editor-btn-save = Save
theme-editor-btn-export = Export...
theme-editor-btn-import = Import...

# --- Colour token editor ----------------------------------------------------
theme-editor-hex-invalid = invalid hex
theme-editor-empty = Select a theme to edit its colours.

# --- Contrast advisory ------------------------------------------------------
theme-editor-advisory-heading = Contrast advisories (below AA):
# The ratio is pre-formatted to one decimal place in Rust and passed as text so
# the "4.5"-style rendering stays byte-identical (not via Fluent NUMBER()).
theme-editor-advisory-row = { $pair }: contrast { $ratio }:1 (below AA 4.5:1)

# --- Colour-token labels (displayed form of the EditableToken surface) -------
# egui chrome
theme-editor-token-chrome-panel-fill = Chrome: panel fill
theme-editor-token-chrome-window-fill = Chrome: window fill
theme-editor-token-chrome-panel-border = Chrome: panel border
theme-editor-token-chrome-extreme-input-bg = Chrome: extreme (input) bg
theme-editor-token-chrome-raised-button-bg = Chrome: raised (button) bg
theme-editor-token-chrome-hovered-bg = Chrome: hovered bg
theme-editor-token-chrome-menu-bar-text = Chrome: menu bar text
theme-editor-token-chrome-input-foreground = Chrome: input foreground
theme-editor-token-chrome-input-border = Chrome: input border
theme-editor-token-chrome-focus-ring = Chrome: focus ring
theme-editor-token-chrome-accent = Chrome: accent
theme-editor-token-chrome-title-band-bg = Chrome: title band bg
theme-editor-token-chrome-tab-active-bg = Chrome: tab active bg
theme-editor-token-chrome-tab-inactive-bg = Chrome: tab inactive bg
theme-editor-token-chrome-tab-active-text = Chrome: tab active text
theme-editor-token-chrome-tab-inactive-text = Chrome: tab inactive text
# domain: syntax
theme-editor-token-syntax-keyword = Syntax: keyword
theme-editor-token-syntax-comment = Syntax: comment
theme-editor-token-syntax-string = Syntax: string
theme-editor-token-syntax-number = Syntax: number
theme-editor-token-syntax-operator = Syntax: operator
theme-editor-token-syntax-type = Syntax: type
theme-editor-token-syntax-function = Syntax: function
theme-editor-token-syntax-macro = Syntax: macro
theme-editor-token-syntax-preprocessor = Syntax: preprocessor
theme-editor-token-syntax-default-text = Syntax: default text
# domain: editor content
theme-editor-token-editor-foreground = Editor: foreground
theme-editor-token-editor-muted = Editor: muted
theme-editor-token-editor-current-line-bg = Editor: current line bg
# domain: gutter
theme-editor-token-gutter-line-number-fg = Gutter: line number fg
theme-editor-token-gutter-line-number-bg = Gutter: line number bg
theme-editor-token-gutter-fold-margin-bg = Gutter: fold margin bg
theme-editor-token-gutter-fold-margin-fg = Gutter: fold margin fg
theme-editor-token-gutter-margin-separator = Gutter: margin separator
theme-editor-token-gutter-cursor-row-border = Gutter: cursor row border
theme-editor-token-gutter-cursor-column = Gutter: cursor column
# domain: file tree
theme-editor-token-file-tree-binary = File tree: binary
theme-editor-token-file-tree-structured = File tree: structured
theme-editor-token-file-tree-text = File tree: text
theme-editor-token-file-tree-unknown = File tree: unknown
theme-editor-token-file-tree-directory = File tree: directory
theme-editor-token-file-tree-symlink = File tree: symlink
# domain: decorations
theme-editor-token-decoration-search-highlight = Decoration: search highlight
theme-editor-token-decoration-error-underline = Decoration: error underline
theme-editor-token-decoration-warning-underline = Decoration: warning underline
theme-editor-token-decoration-info-underline = Decoration: info underline
theme-editor-token-decoration-change-added = Decoration: change added
theme-editor-token-decoration-change-modified = Decoration: change modified
theme-editor-token-decoration-change-deleted = Decoration: change deleted
theme-editor-token-decoration-bookmark = Decoration: bookmark
# domain: indicators
theme-editor-token-indicator-find-match = Indicator: find match
theme-editor-token-indicator-brace-match = Indicator: brace match
theme-editor-token-indicator-brace-mismatch = Indicator: brace mismatch
theme-editor-token-indicator-hotspot-underline = Indicator: hotspot underline
