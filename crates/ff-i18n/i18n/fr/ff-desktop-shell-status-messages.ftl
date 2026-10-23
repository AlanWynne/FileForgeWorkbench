# French (fr) shell STATUS / ERROR messages (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/ff-desktop-shell-status-messages.ftl; only prose translated. Command
# VERBS/keywords stay English baked into the value (D1); the one mid-sentence
# case uses the non-translated { $verb } placeable (Req 9.2). { $error } and all
# names/ids/counts are non-translated DATA placeables (Req 9.3, 4.2).

# --- File Explorer node operations ---
ff-desktop-shell-msg-new-file-failed = Nouveau fichier/dossier : { $error }
ff-desktop-shell-msg-create-failed = { "\u00c9" }chec de la cr{ "\u00e9" }ation : { $error }
ff-desktop-shell-msg-paste-failed = Coller : { $error }
ff-desktop-shell-msg-paste-count-failed = Coller : { $count } fichier(s) n'ont pas pu { "\u00ea" }tre copi{ "\u00e9" }s
ff-desktop-shell-msg-delete-provider-failed = Supprimer : { $error }
ff-desktop-shell-msg-delete-failed = { "\u00c9" }chec de la suppression : { $error }
ff-desktop-shell-msg-rename-provider-failed = Renommer : { $error }
ff-desktop-shell-msg-rename-failed = { "\u00c9" }chec du renommage : { $error }

# --- Workspace open/save ---
ff-desktop-shell-msg-ws-open-failed = Impossible d'ouvrir l'espace de travail : { $error }
ff-desktop-shell-msg-ws-root-not-found = Avertissement d'espace de travail : racine '{ $root }' introuvable sur le disque
ff-desktop-shell-msg-ws-none-to-save = Aucun espace de travail actif { "\u00e0" } enregistrer
ff-desktop-shell-msg-ws-no-path = Aucun chemin de fichier d'espace de travail d{ "\u00e9" }fini -- utilisez WORKSPACE SAVE AS <path>
ff-desktop-shell-msg-ws-save-failed = Impossible d'enregistrer l'espace de travail : { $error }

# --- Session restore ---
ff-desktop-shell-msg-restore-failed = Restauration impossible : { $error }

# --- Theme set/activate ---
ff-desktop-shell-msg-theme-mode-save-failed = Th{ "\u00e8" }me chang{ "\u00e9" } en { $mode } pour cette session, mais n'a pas pu { "\u00ea" }tre enregistr{ "\u00e9" } : { $error }
ff-desktop-shell-msg-theme-name-save-failed = Th{ "\u00e8" }me '{ $name }' appliqu{ "\u00e9" } pour cette session, mais n'a pas pu { "\u00ea" }tre enregistr{ "\u00e9" } : { $error }
ff-desktop-shell-msg-theme-load-failed = Le th{ "\u00e8" }me '{ $name }' n'a pas pu { "\u00ea" }tre charg{ "\u00e9" }

# --- Macro library / search apply ---
ff-desktop-shell-msg-lua-not-available = L'ex{ "\u00e9" }cution Lua n'est pas encore disponible
ff-desktop-shell-msg-replace-summary = { $count } occurrence(s) remplac{ "\u00e9" }e(s) dans { $files } fichier(s)
ff-desktop-shell-msg-replace-failed = { "\u00c9" }chec du remplacement : { $error }

# --- External program execution ---
ff-desktop-shell-msg-external-started-detached = '{ $program }' d{ "\u00e9" }marr{ "\u00e9" } (d{ "\u00e9" }tach{ "\u00e9" }).
ff-desktop-shell-msg-external-finished = '{ $program }' termin{ "\u00e9" } (sortie { $code }). Voir le panneau Sortie.

# --- Startup workspace restore ---
ff-desktop-shell-msg-workspace-not-found = Espace de travail '{ $path }' introuvable -- d{ "\u00e9" }marrage sans espace de travail

# --- SCRM capture/snapshot command status ---
ff-desktop-shell-msg-scrm-captured-screen = { "\u00c9" }cran { $seq } captur{ "\u00e9" }.
ff-desktop-shell-msg-scrm-screen-not-capturable = CAPTURE SCREEN : l'espace de travail actif ne peut pas encore { "\u00ea" }tre captur{ "\u00e9" }.
ff-desktop-shell-msg-scrm-unknown-subcommand = CAPTURE : sous-commande inconnue '{ $subcommand }'.
ff-desktop-shell-msg-scrm-no-collection-replay = Aucune collection d'{ "\u00e9" }crans { "\u00e0" } rejouer.
ff-desktop-shell-msg-scrm-snapshot-copied = Instantan{ "\u00e9" } copi{ "\u00e9" } dans le presse-papiers ({ $format }).
ff-desktop-shell-msg-scrm-snapshot-no-clipboard = Instantan{ "\u00e9" } rendu ({ $format }) mais le presse-papiers { "\u00e9" }tait indisponible.

# === verb-embedding command-ladder messages ===
ff-desktop-shell-msg-detach-max-reached = Nombre maximum d'espaces de travail d{ "\u00e9" }tach{ "\u00e9" }s (16) atteint.
ff-desktop-shell-msg-unsplit-not-split = UNSPLIT : l'espace de travail n'est pas divis{ "\u00e9" }.
ff-desktop-shell-msg-focus-not-split = FOCUS : l'espace de travail n'est pas divis{ "\u00e9" }.
ff-desktop-shell-msg-swap-out-of-range = SWAP : num{ "\u00e9" }ro d'onglet { $n } hors plage (valide : 1 { "\u00e0" } { $count })
ff-desktop-shell-msg-swap-bad-arg = SWAP : argument invalide '{ $arg }'. Utilisation : SWAP <n>, SWAP LIST

# --- JES / dataset stubs ---
ff-desktop-shell-msg-submit-not-available = SUBMIT : sous-syst{ "\u00e8" }me JES pas encore disponible
ff-desktop-shell-msg-time-status = Date : { $date }  Heure : { $time }  Jour : { $day }
ff-desktop-shell-msg-status-routing-filter = STATUS : redirection vers le panneau JES (filtre : { $filter })
ff-desktop-shell-msg-status-routing = STATUS : redirection vers le panneau de statut des travaux JES
ff-desktop-shell-msg-create-not-available = CREATE { $dsn } : cr{ "\u00e9" }ation de dataset pas encore disponible
ff-desktop-shell-msg-replace-dsn-not-available = REPLACE { $dsn } : remplacement de dataset pas encore disponible
ff-desktop-shell-msg-compare-not-available = COMPARE { $dsn } : vue de comparaison pas encore disponible

# --- WORKSPACE subcommands ---
ff-desktop-shell-msg-workspace-open-requires-path = WORKSPACE OPEN n{ "\u00e9" }cessite un chemin
ff-desktop-shell-msg-workspace-saveas-requires-path = WORKSPACE SAVE AS n{ "\u00e9" }cessite un chemin
ff-desktop-shell-msg-workspace-addroot-requires-path = WORKSPACE ADD ROOT n{ "\u00e9" }cessite un chemin
ff-desktop-shell-msg-workspace-no-active-open-first = Aucun espace de travail actif -- utilisez d'abord WORKSPACE OPEN
ff-desktop-shell-msg-workspace-removeroot-requires-path = WORKSPACE REMOVE ROOT n{ "\u00e9" }cessite un chemin
ff-desktop-shell-msg-workspace-no-active = Aucun espace de travail actif
ff-desktop-shell-msg-workspace-unknown-subcommand = WORKSPACE : sous-commande inconnue. Utilisez OPEN/SAVE/SAVE AS/CLOSE/ADD ROOT/REMOVE ROOT

# --- verb-prefixed cluster B ---
ff-desktop-shell-msg-edit-requires-path = EDIT n{ "\u00e9" }cessite un chemin de fichier
ff-desktop-shell-msg-dock-not-detached = DOCK : l'espace de travail actuel n'est pas d{ "\u00e9" }tach{ "\u00e9" }
ff-desktop-shell-msg-command-bad-position = COMMAND : '{ $arg }' n'est pas une position valide (TOP ou BOTTOM)
ff-desktop-shell-msg-theme-not-exist = THEME : '{ $name }' n'existe pas
ff-desktop-shell-msg-chain-depth-exceeded = Le chemin cha{ "\u00ee" }n{ "\u00e9" } d{ "\u00e9" }passe la profondeur maximale de 4 segments.
ff-desktop-shell-msg-command-not-defined = La commande '{ $id }' n'est pas d{ "\u00e9" }finie.
ff-desktop-shell-msg-custom-ws-not-runnable = La cible d'espace de travail personnalis{ "\u00e9" } '{ $target }' n'est pas encore ex{ "\u00e9" }cutable via une liaison de commande.
ff-desktop-shell-msg-macro-not-runnable = Les cibles macro ne sont pas encore ex{ "\u00e9" }cutables via une liaison de commande.
ff-desktop-shell-msg-external-cancelled = Ex{ "\u00e9" }cution externe annul{ "\u00e9" }e.

# --- mid-sentence verb (Req 9.2 canonical) + HILITE/PFSHOW ---
ff-desktop-shell-msg-profile-locked = Le profil est verrouill{ "\u00e9" } -- utilisez { $verb } pour le d{ "\u00e9" }verrouiller
ff-desktop-shell-msg-hilite-unknown-mode = HILITE : mode inconnu '{ $keyword }'
ff-desktop-shell-msg-pfshow-bad-arg = Argument PFSHOW inconnu '{ $arg }' -- attendu ON, OFF, BASE, SHIFT, CTRL ou ALT.

# === thiserror DISPLAY-BOUNDARY keys (D2) ===
ff-desktop-shell-msg-profile-unknown-key = Cl{ "\u00e9" } de profil inconnue : { $key }
ff-desktop-shell-msg-profile-unknown-value = Valeur inconnue pour { $key } : { $value }
ff-desktop-shell-msg-shell-disabled = L'acc{ "\u00e8" }s au shell est d{ "\u00e9" }sactiv{ "\u00e9" } par la configuration
