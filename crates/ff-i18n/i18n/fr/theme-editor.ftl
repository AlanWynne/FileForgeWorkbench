# French (fr) Theme Editor Context (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/theme-editor.ftl; only the DISPLAYED prose translated. The token English
# `label` remains the stable identity; only the displayed text localises. NOT
# localised (Req 4.4): egui id_salt, the ThemeEditorAction enum, hex values.
# { $name } / { $pair } / { $ratio } are placeables.

# --- Theme selector + lifecycle buttons -------------------------------------
theme-editor-label-theme = Th{ "\u00e8" }me :
theme-editor-selected-none = (aucun)
theme-editor-btn-set-active = D{ "\u00e9" }finir comme actif
theme-editor-btn-reset = R{ "\u00e9" }initialiser au th{ "\u00e8" }me int{ "\u00e9" }gr{ "\u00e9" }
theme-editor-reset-confirm = R{ "\u00e9" }initialiser '{ $name }' { "\u00e0" } sa r{ "\u00e9" }f{ "\u00e9" }rence et abandonner les modifications ?
theme-editor-btn-confirm-reset = Confirmer la r{ "\u00e9" }initialisation
theme-editor-btn-cancel = Annuler

# --- Copy / Save As / Save + Import / Export --------------------------------
theme-editor-label-new-name = Nouveau nom :
theme-editor-btn-copy = Copier
theme-editor-btn-save-as = Enregistrer sous
theme-editor-btn-save = Enregistrer
theme-editor-btn-export = Exporter...
theme-editor-btn-import = Importer...

# --- Colour token editor ----------------------------------------------------
theme-editor-hex-invalid = hex invalide
theme-editor-empty = S{ "\u00e9" }lectionnez un th{ "\u00e8" }me pour modifier ses couleurs.

# --- Contrast advisory ------------------------------------------------------
theme-editor-advisory-heading = Avertissements de contraste (sous AA) :
theme-editor-advisory-row = { $pair } : contraste { $ratio }:1 (sous AA 4.5:1)

# --- Colour-token labels (displayed form of the EditableToken surface) -------
# egui chrome
theme-editor-token-chrome-panel-fill = Ch{ "\u00e2" }ssis : remplissage du panneau
theme-editor-token-chrome-window-fill = Ch{ "\u00e2" }ssis : remplissage de la fen{ "\u00ea" }tre
theme-editor-token-chrome-panel-border = Ch{ "\u00e2" }ssis : bordure du panneau
theme-editor-token-chrome-extreme-input-bg = Ch{ "\u00e2" }ssis : fond (saisie) extr{ "\u00ea" }me
theme-editor-token-chrome-raised-button-bg = Ch{ "\u00e2" }ssis : fond (bouton) en relief
theme-editor-token-chrome-hovered-bg = Ch{ "\u00e2" }ssis : fond survol{ "\u00e9" }
theme-editor-token-chrome-menu-bar-text = Ch{ "\u00e2" }ssis : texte de la barre de menus
theme-editor-token-chrome-input-foreground = Ch{ "\u00e2" }ssis : premier plan de saisie
theme-editor-token-chrome-input-border = Ch{ "\u00e2" }ssis : bordure de saisie
theme-editor-token-chrome-focus-ring = Ch{ "\u00e2" }ssis : anneau de focus
theme-editor-token-chrome-accent = Ch{ "\u00e2" }ssis : accentuation
theme-editor-token-chrome-title-band-bg = Ch{ "\u00e2" }ssis : fond du bandeau de titre
theme-editor-token-chrome-tab-active-bg = Ch{ "\u00e2" }ssis : fond onglet actif
theme-editor-token-chrome-tab-inactive-bg = Ch{ "\u00e2" }ssis : fond onglet inactif
theme-editor-token-chrome-tab-active-text = Ch{ "\u00e2" }ssis : texte onglet actif
theme-editor-token-chrome-tab-inactive-text = Ch{ "\u00e2" }ssis : texte onglet inactif
# domain: syntax
theme-editor-token-syntax-keyword = Syntaxe : mot-cl{ "\u00e9" }
theme-editor-token-syntax-comment = Syntaxe : commentaire
theme-editor-token-syntax-string = Syntaxe : cha{ "\u00ee" }ne
theme-editor-token-syntax-number = Syntaxe : nombre
theme-editor-token-syntax-operator = Syntaxe : op{ "\u00e9" }rateur
theme-editor-token-syntax-type = Syntaxe : type
theme-editor-token-syntax-function = Syntaxe : fonction
theme-editor-token-syntax-macro = Syntaxe : macro
theme-editor-token-syntax-preprocessor = Syntaxe : pr{ "\u00e9" }processeur
theme-editor-token-syntax-default-text = Syntaxe : texte par d{ "\u00e9" }faut
# domain: editor content
theme-editor-token-editor-foreground = { "\u00c9" }diteur : premier plan
theme-editor-token-editor-muted = { "\u00c9" }diteur : att{ "\u00e9" }nu{ "\u00e9" }
theme-editor-token-editor-current-line-bg = { "\u00c9" }diteur : fond de la ligne courante
# domain: gutter
theme-editor-token-gutter-line-number-fg = Goutti{ "\u00e8" }re : num{ "\u00e9" }ro de ligne (av.-plan)
theme-editor-token-gutter-line-number-bg = Goutti{ "\u00e8" }re : num{ "\u00e9" }ro de ligne (fond)
theme-editor-token-gutter-fold-margin-bg = Goutti{ "\u00e8" }re : fond de la marge de pliage
theme-editor-token-gutter-fold-margin-fg = Goutti{ "\u00e8" }re : av.-plan de la marge de pliage
theme-editor-token-gutter-margin-separator = Goutti{ "\u00e8" }re : s{ "\u00e9" }parateur de marge
theme-editor-token-gutter-cursor-row-border = Goutti{ "\u00e8" }re : bordure de la ligne du curseur
theme-editor-token-gutter-cursor-column = Goutti{ "\u00e8" }re : colonne du curseur
# domain: file tree
theme-editor-token-file-tree-binary = Arborescence : binaire
theme-editor-token-file-tree-structured = Arborescence : structur{ "\u00e9" }
theme-editor-token-file-tree-text = Arborescence : texte
theme-editor-token-file-tree-unknown = Arborescence : inconnu
theme-editor-token-file-tree-directory = Arborescence : r{ "\u00e9" }pertoire
theme-editor-token-file-tree-symlink = Arborescence : lien symbolique
# domain: decorations
theme-editor-token-decoration-search-highlight = D{ "\u00e9" }coration : surlignage de recherche
theme-editor-token-decoration-error-underline = D{ "\u00e9" }coration : soulignement d'erreur
theme-editor-token-decoration-warning-underline = D{ "\u00e9" }coration : soulignement d'avertissement
theme-editor-token-decoration-info-underline = D{ "\u00e9" }coration : soulignement d'information
theme-editor-token-decoration-change-added = D{ "\u00e9" }coration : changement ajout{ "\u00e9" }
theme-editor-token-decoration-change-modified = D{ "\u00e9" }coration : changement modifi{ "\u00e9" }
theme-editor-token-decoration-change-deleted = D{ "\u00e9" }coration : changement supprim{ "\u00e9" }
theme-editor-token-decoration-bookmark = D{ "\u00e9" }coration : signet
# domain: indicators
theme-editor-token-indicator-find-match = Indicateur : correspondance trouv{ "\u00e9" }e
theme-editor-token-indicator-brace-match = Indicateur : accolade correspondante
theme-editor-token-indicator-brace-mismatch = Indicateur : accolade non correspondante
theme-editor-token-indicator-hotspot-underline = Indicateur : soulignement de point actif
