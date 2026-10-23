# Afrikaans (af) shell STATUS / ERROR messages (CR-NR-103 Task 10). REVIEWED POC.
# Keys + { $placeables } identical to en. Command VERBS/keywords (SWAP, WORKSPACE,
# CREATE, REPLACE, COMPARE, UNSPLIT, FOCUS, SUBMIT, STATUS, EDIT, DOCK, COMMAND,
# THEME, HILITE, PFSHOW, CAPTURE, LOCK OFF, TOP/BOTTOM/ON/OFF/..., the SAVE AS
# <path> usage phrases) stay ENGLISH (command surface, Req 4.2/9.2); only the
# surrounding prose localises. { $error } / { $dsn } / { $path } / counts are DATA.

# --- File Explorer node operations ---
ff-desktop-shell-msg-new-file-failed = Nuwe l{ "\u00ea" }er/gids: { $error }
ff-desktop-shell-msg-create-failed = Skep het misluk: { $error }
ff-desktop-shell-msg-paste-failed = Plak: { $error }
ff-desktop-shell-msg-paste-count-failed = Plak: { $count } l{ "\u00ea" }er(s) kon nie gekopieer word nie
ff-desktop-shell-msg-delete-provider-failed = Skrap: { $error }
ff-desktop-shell-msg-delete-failed = Skrap het misluk: { $error }
ff-desktop-shell-msg-rename-provider-failed = Hernoem: { $error }
ff-desktop-shell-msg-rename-failed = Hernoem het misluk: { $error }

# --- Workspace open/save ---
ff-desktop-shell-msg-ws-open-failed = Kan nie werkspasie oopmaak nie: { $error }
ff-desktop-shell-msg-ws-root-not-found = Werkspasie-waarskuwing: wortel '{ $root }' nie op skyf gevind nie
ff-desktop-shell-msg-ws-none-to-save = Geen aktiewe werkspasie om te stoor nie
ff-desktop-shell-msg-ws-no-path = Geen werkspasie-l{ "\u00ea" }erpad gestel nie -- gebruik WORKSPACE SAVE AS <path>
ff-desktop-shell-msg-ws-save-failed = Kan nie werkspasie stoor nie: { $error }

# --- Session restore ---
ff-desktop-shell-msg-restore-failed = Kon nie herstel nie: { $error }

# --- Theme set/activate ---
ff-desktop-shell-msg-theme-mode-save-failed = Tema na { $mode } verander vir hierdie sessie, maar kon nie gestoor word nie: { $error }
ff-desktop-shell-msg-theme-name-save-failed = Tema '{ $name }' toegepas vir hierdie sessie, maar kon nie gestoor word nie: { $error }
ff-desktop-shell-msg-theme-load-failed = Tema '{ $name }' kon nie gelaai word nie

# --- Macro library / search apply ---
ff-desktop-shell-msg-lua-not-available = Lua-uitvoering is nog nie beskikbaar nie
ff-desktop-shell-msg-replace-summary = { $count } voorkoms(te) in { $files } l{ "\u00ea" }er(s) vervang
ff-desktop-shell-msg-replace-failed = Vervang het misluk: { $error }

# --- External program execution ---
ff-desktop-shell-msg-external-started-detached = '{ $program }' begin (losgemaak).
ff-desktop-shell-msg-external-finished = '{ $program }' voltooi (uitgang { $code }). Sien die Afvoerpaneel.

# --- Startup workspace restore ---
ff-desktop-shell-msg-workspace-not-found = Werkspasie '{ $path }' nie gevind nie -- begin sonder werkspasie

# --- SCRM capture/snapshot command status ---
ff-desktop-shell-msg-scrm-captured-screen = Skerm { $seq } vasgevang.
ff-desktop-shell-msg-scrm-screen-not-capturable = CAPTURE SCREEN: die aktiewe werkspasie kan nog nie vasgevang word nie.
ff-desktop-shell-msg-scrm-unknown-subcommand = CAPTURE: onbekende subopdrag '{ $subcommand }'.
ff-desktop-shell-msg-scrm-no-collection-replay = Geen skermversameling om te herspeel nie.
ff-desktop-shell-msg-scrm-snapshot-copied = Kiekie na knipbord gekopieer ({ $format }).
ff-desktop-shell-msg-scrm-snapshot-no-clipboard = Kiekie gelewer ({ $format }) maar die knipbord was onbeskikbaar.

# --- detach / split / swap ---
ff-desktop-shell-msg-detach-max-reached = Maksimum aantal losgemaakte werkspasies (16) bereik.
ff-desktop-shell-msg-unsplit-not-split = UNSPLIT: die werkspasie is nie verdeel nie.
ff-desktop-shell-msg-focus-not-split = FOCUS: die werkspasie is nie verdeel nie.
ff-desktop-shell-msg-swap-out-of-range = SWAP: oortjienommer { $n } buite bereik (geldig: 1 tot { $count })
ff-desktop-shell-msg-swap-bad-arg = SWAP: ongeldige argument '{ $arg }'. Gebruik: SWAP <n>, SWAP LIST

# --- JES / dataset stubs ---
ff-desktop-shell-msg-submit-not-available = SUBMIT: JES-substelsel nog nie beskikbaar nie
ff-desktop-shell-msg-time-status = Datum: { $date }  Tyd: { $time }  Dag: { $day }
ff-desktop-shell-msg-status-routing-filter = STATUS: stuur na JES-paneel (filter: { $filter })
ff-desktop-shell-msg-status-routing = STATUS: stuur na JES-taakstatuspaneel
ff-desktop-shell-msg-create-not-available = CREATE { $dsn }: datastelskepping nog nie beskikbaar nie
ff-desktop-shell-msg-replace-dsn-not-available = REPLACE { $dsn }: datastelvervanging nog nie beskikbaar nie
ff-desktop-shell-msg-compare-not-available = COMPARE { $dsn }: vergelykingsaansig nog nie beskikbaar nie

# --- WORKSPACE subcommands ---
ff-desktop-shell-msg-workspace-open-requires-path = WORKSPACE OPEN vereis 'n pad
ff-desktop-shell-msg-workspace-saveas-requires-path = WORKSPACE SAVE AS vereis 'n pad
ff-desktop-shell-msg-workspace-addroot-requires-path = WORKSPACE ADD ROOT vereis 'n pad
ff-desktop-shell-msg-workspace-no-active-open-first = Geen aktiewe werkspasie nie -- gebruik eers WORKSPACE OPEN
ff-desktop-shell-msg-workspace-removeroot-requires-path = WORKSPACE REMOVE ROOT vereis 'n pad
ff-desktop-shell-msg-workspace-no-active = Geen aktiewe werkspasie nie
ff-desktop-shell-msg-workspace-unknown-subcommand = WORKSPACE: onbekende subopdrag. Gebruik OPEN/SAVE/SAVE AS/CLOSE/ADD ROOT/REMOVE ROOT

# --- verb-prefixed cluster B ---
ff-desktop-shell-msg-edit-requires-path = EDIT vereis 'n l{ "\u00ea" }erpad
ff-desktop-shell-msg-dock-not-detached = DOCK: die huidige werkspasie is nie losgemaak nie
ff-desktop-shell-msg-command-bad-position = COMMAND: '{ $arg }' is nie 'n geldige posisie nie (TOP of BOTTOM)
ff-desktop-shell-msg-theme-not-exist = THEME: '{ $name }' bestaan nie
ff-desktop-shell-msg-chain-depth-exceeded = Geketting pad oorskry die maksimum diepte van 4 segmente.
ff-desktop-shell-msg-command-not-defined = Opdrag '{ $id }' is nie gedefinieer nie.
ff-desktop-shell-msg-custom-ws-not-runnable = Pasgemaakte werkspasieteiken '{ $target }' is nog nie via 'n opdragbinding uitvoerbaar nie.
ff-desktop-shell-msg-macro-not-runnable = Makroteikens is nog nie via 'n opdragbinding uitvoerbaar nie.
ff-desktop-shell-msg-external-cancelled = Eksterne uitvoering gekanselleer.

# --- mid-sentence verb (Req 9.2 canonical) + HILITE/PFSHOW ---
ff-desktop-shell-msg-profile-locked = Profiel is gesluit -- gebruik { $verb } om te ontsluit
ff-desktop-shell-msg-hilite-unknown-mode = HILITE: onbekende modus '{ $keyword }'
ff-desktop-shell-msg-pfshow-bad-arg = Onbekende PFSHOW-argument '{ $arg }' -- verwag ON, OFF, BASE, SHIFT, CTRL, of ALT.

# --- thiserror display-boundary keys ---
ff-desktop-shell-msg-profile-unknown-key = Onbekende profielsleutel: { $key }
ff-desktop-shell-msg-profile-unknown-value = Onbekende waarde vir { $key }: { $value }
ff-desktop-shell-msg-shell-disabled = Dopkastoegang is deur konfigurasie gedeaktiveer
