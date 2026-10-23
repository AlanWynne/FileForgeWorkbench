# German (de) shell STATUS / ERROR messages (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/ff-desktop-shell-status-messages.ftl; only prose translated. Command
# VERBS/keywords stay English baked into the value (D1); the one mid-sentence
# case uses the non-translated { $verb } placeable (Req 9.2). { $error } and all
# names/ids/counts are non-translated DATA placeables (Req 9.3, 4.2).

# --- File Explorer node operations ---
ff-desktop-shell-msg-new-file-failed = Neue Datei/neuer Ordner: { $error }
ff-desktop-shell-msg-create-failed = Erstellen fehlgeschlagen: { $error }
ff-desktop-shell-msg-paste-failed = Einf{ "\u00fc" }gen: { $error }
ff-desktop-shell-msg-paste-count-failed = Einf{ "\u00fc" }gen: { $count } Datei(en) konnten nicht kopiert werden
ff-desktop-shell-msg-delete-provider-failed = L{ "\u00f6" }schen: { $error }
ff-desktop-shell-msg-delete-failed = L{ "\u00f6" }schen fehlgeschlagen: { $error }
ff-desktop-shell-msg-rename-provider-failed = Umbenennen: { $error }
ff-desktop-shell-msg-rename-failed = Umbenennen fehlgeschlagen: { $error }

# --- Workspace open/save ---
ff-desktop-shell-msg-ws-open-failed = Arbeitsbereich kann nicht ge{ "\u00f6" }ffnet werden: { $error }
ff-desktop-shell-msg-ws-root-not-found = Arbeitsbereich-Warnung: Wurzel '{ $root }' nicht auf der Festplatte gefunden
ff-desktop-shell-msg-ws-none-to-save = Kein aktiver Arbeitsbereich zum Speichern
ff-desktop-shell-msg-ws-no-path = Kein Arbeitsbereich-Dateipfad gesetzt -- verwenden Sie WORKSPACE SAVE AS <path>
ff-desktop-shell-msg-ws-save-failed = Arbeitsbereich kann nicht gespeichert werden: { $error }

# --- Session restore ---
ff-desktop-shell-msg-restore-failed = Wiederherstellung nicht m{ "\u00f6" }glich: { $error }

# --- Theme set/activate ---
ff-desktop-shell-msg-theme-mode-save-failed = Theme f{ "\u00fc" }r diese Sitzung auf { $mode } ge{ "\u00e4" }ndert, konnte aber nicht gespeichert werden: { $error }
ff-desktop-shell-msg-theme-name-save-failed = Theme '{ $name }' f{ "\u00fc" }r diese Sitzung angewendet, konnte aber nicht gespeichert werden: { $error }
ff-desktop-shell-msg-theme-load-failed = Theme '{ $name }' konnte nicht geladen werden

# --- Macro library / search apply ---
ff-desktop-shell-msg-lua-not-available = Lua-Ausf{ "\u00fc" }hrung noch nicht verf{ "\u00fc" }gbar
ff-desktop-shell-msg-replace-summary = { $count } Vorkommen in { $files } Datei(en) ersetzt
ff-desktop-shell-msg-replace-failed = Ersetzen fehlgeschlagen: { $error }

# --- External program execution ---
ff-desktop-shell-msg-external-started-detached = '{ $program }' gestartet (abgekoppelt).
ff-desktop-shell-msg-external-finished = '{ $program }' beendet (Exit { $code }). Siehe Ausgabe-Panel.

# --- Startup workspace restore ---
ff-desktop-shell-msg-workspace-not-found = Arbeitsbereich '{ $path }' nicht gefunden -- Start ohne Arbeitsbereich

# --- SCRM capture/snapshot command status ---
ff-desktop-shell-msg-scrm-captured-screen = Bildschirm { $seq } erfasst.
ff-desktop-shell-msg-scrm-screen-not-capturable = CAPTURE SCREEN: der aktive Arbeitsbereich kann noch nicht erfasst werden.
ff-desktop-shell-msg-scrm-unknown-subcommand = CAPTURE: unbekannter Unterbefehl '{ $subcommand }'.
ff-desktop-shell-msg-scrm-no-collection-replay = Keine Bildschirmsammlung zum Wiedergeben.
ff-desktop-shell-msg-scrm-snapshot-copied = Schnappschuss in die Zwischenablage kopiert ({ $format }).
ff-desktop-shell-msg-scrm-snapshot-no-clipboard = Schnappschuss gerendert ({ $format }), aber die Zwischenablage war nicht verf{ "\u00fc" }gbar.

# === verb-embedding command-ladder messages ===
ff-desktop-shell-msg-detach-max-reached = Maximale Anzahl abgekoppelter Arbeitsbereiche (16) erreicht.
ff-desktop-shell-msg-unsplit-not-split = UNSPLIT: der Arbeitsbereich ist nicht geteilt.
ff-desktop-shell-msg-focus-not-split = FOCUS: der Arbeitsbereich ist nicht geteilt.
ff-desktop-shell-msg-swap-out-of-range = SWAP: Tab-Nummer { $n } au{ "\u00df" }erhalb des Bereichs (g{ "\u00fc" }ltig: 1 bis { $count })
ff-desktop-shell-msg-swap-bad-arg = SWAP: ung{ "\u00fc" }ltiges Argument '{ $arg }'. Verwendung: SWAP <n>, SWAP LIST

# --- JES / dataset stubs ---
ff-desktop-shell-msg-submit-not-available = SUBMIT: JES-Subsystem noch nicht verf{ "\u00fc" }gbar
ff-desktop-shell-msg-time-status = Datum: { $date }  Zeit: { $time }  Tag: { $day }
ff-desktop-shell-msg-status-routing-filter = STATUS: Weiterleitung zum JES-Panel (Filter: { $filter })
ff-desktop-shell-msg-status-routing = STATUS: Weiterleitung zum JES-Jobstatus-Panel
ff-desktop-shell-msg-create-not-available = CREATE { $dsn }: Dataset-Erstellung noch nicht verf{ "\u00fc" }gbar
ff-desktop-shell-msg-replace-dsn-not-available = REPLACE { $dsn }: Dataset-Ersetzung noch nicht verf{ "\u00fc" }gbar
ff-desktop-shell-msg-compare-not-available = COMPARE { $dsn }: Vergleichsansicht noch nicht verf{ "\u00fc" }gbar

# --- WORKSPACE subcommands ---
ff-desktop-shell-msg-workspace-open-requires-path = WORKSPACE OPEN erfordert einen Pfad
ff-desktop-shell-msg-workspace-saveas-requires-path = WORKSPACE SAVE AS erfordert einen Pfad
ff-desktop-shell-msg-workspace-addroot-requires-path = WORKSPACE ADD ROOT erfordert einen Pfad
ff-desktop-shell-msg-workspace-no-active-open-first = Kein aktiver Arbeitsbereich -- verwenden Sie zuerst WORKSPACE OPEN
ff-desktop-shell-msg-workspace-removeroot-requires-path = WORKSPACE REMOVE ROOT erfordert einen Pfad
ff-desktop-shell-msg-workspace-no-active = Kein aktiver Arbeitsbereich
ff-desktop-shell-msg-workspace-unknown-subcommand = WORKSPACE: unbekannter Unterbefehl. Verwenden Sie OPEN/SAVE/SAVE AS/CLOSE/ADD ROOT/REMOVE ROOT

# --- verb-prefixed cluster B ---
ff-desktop-shell-msg-edit-requires-path = EDIT erfordert einen Dateipfad
ff-desktop-shell-msg-dock-not-detached = DOCK: der aktuelle Arbeitsbereich ist nicht abgekoppelt
ff-desktop-shell-msg-command-bad-position = COMMAND: '{ $arg }' ist keine g{ "\u00fc" }ltige Position (TOP oder BOTTOM)
ff-desktop-shell-msg-theme-not-exist = THEME: '{ $name }' existiert nicht
ff-desktop-shell-msg-chain-depth-exceeded = Verketteter Pfad { "\u00fc" }berschreitet die maximale Tiefe von 4 Segmenten.
ff-desktop-shell-msg-command-not-defined = Befehl '{ $id }' ist nicht definiert.
ff-desktop-shell-msg-custom-ws-not-runnable = Benutzerdefiniertes Arbeitsbereich-Ziel '{ $target }' ist { "\u00fc" }ber eine Befehlsbindung noch nicht ausf{ "\u00fc" }hrbar.
ff-desktop-shell-msg-macro-not-runnable = Makro-Ziele sind { "\u00fc" }ber eine Befehlsbindung noch nicht ausf{ "\u00fc" }hrbar.
ff-desktop-shell-msg-external-cancelled = Externe Ausf{ "\u00fc" }hrung abgebrochen.

# --- mid-sentence verb (Req 9.2 canonical) + HILITE/PFSHOW ---
ff-desktop-shell-msg-profile-locked = Profil ist gesperrt -- verwenden Sie { $verb } zum Entsperren
ff-desktop-shell-msg-hilite-unknown-mode = HILITE: unbekannter Modus '{ $keyword }'
ff-desktop-shell-msg-pfshow-bad-arg = Unbekanntes PFSHOW-Argument '{ $arg }' -- erwartet ON, OFF, BASE, SHIFT, CTRL oder ALT.

# === thiserror DISPLAY-BOUNDARY keys (D2) ===
ff-desktop-shell-msg-profile-unknown-key = Unbekannter Profilschl{ "\u00fc" }ssel: { $key }
ff-desktop-shell-msg-profile-unknown-value = Unbekannter Wert f{ "\u00fc" }r { $key }: { $value }
ff-desktop-shell-msg-shell-disabled = Shell-Zugriff ist durch die Konfiguration deaktiviert
