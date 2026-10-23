# German (de) Virtual Catalog New/Edit/Delete dialogs (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/catalog-dialog.ftl; only prose translated. Catalog type DISPLAY labels
# (Mainframe/POSIX/Native) are kept as proper identity terms. { $name } /
# { $path } / { $error } are placeables. Trailing alignment spaces are preserved
# with quoted placeables so column widths stay identical to en.

# --- Catalog type display labels (shared New + Edit; Req 4.1) ---------------
catalog-dialog-type-mainframe = Mainframe
catalog-dialog-type-posix = POSIX
catalog-dialog-type-native = Nativ

# --- Shared buttons ---------------------------------------------------------
catalog-dialog-btn-ok = OK
catalog-dialog-btn-cancel = Abbrechen
catalog-dialog-btn-save = Speichern

# --- New Catalog dialog -----------------------------------------------------
catalog-dialog-new-title = Neuer Katalog
catalog-dialog-new-label-type = Katalogtyp:
catalog-dialog-new-label-name = Katalogname:{ "  " }
catalog-dialog-new-label-description = Beschreibung:{ "   " }
catalog-dialog-new-label-repository-path = Repository-Pfad:
catalog-dialog-new-label-default-hlq = Standard-HLQ:{ "    " }
catalog-dialog-new-label-root-directory = Stammverzeichnis:{ " " }
catalog-dialog-new-label-mount-point = Einh{ "\u00e4" }ngepunkt:{ "    " }
catalog-dialog-new-label-root-path = Stammpfad:{ "      " }
catalog-dialog-new-chk-auto-mount = Beim Start automatisch einh{ "\u00e4" }ngen
catalog-dialog-new-chk-read-only = Schreibgesch{ "\u00fc" }tzt
catalog-dialog-new-chk-create-repository = Repository jetzt erstellen

# New Catalog validation messages (Req 3.3-3.8).
catalog-dialog-new-err-name-format = Der Katalogname muss 1-32 Zeichen lang sein (alphanumerisch, Bindestrich, Unterstrich).
catalog-dialog-new-err-duplicate = Ein Katalog namens '{ $name }' existiert bereits.
catalog-dialog-new-err-description-length = Die Beschreibung darf h{ "\u00f6" }chstens 120 Zeichen lang sein.
catalog-dialog-new-err-repository-path-required = Der Repository-Pfad ist f{ "\u00fc" }r Mainframe-Kataloge erforderlich.
catalog-dialog-new-err-root-directory-required = Das Stammverzeichnis ist f{ "\u00fc" }r POSIX-Kataloge erforderlich.
catalog-dialog-new-err-root-path-required = Der Stammpfad ist f{ "\u00fc" }r Native-Kataloge erforderlich.
catalog-dialog-new-err-create-repository-failed = Repository konnte nicht erstellt werden: { $error }

# --- Edit Catalog dialog ----------------------------------------------------
catalog-dialog-edit-title = Katalog bearbeiten
catalog-dialog-edit-label-name = Katalogname:{ "  " }
catalog-dialog-edit-label-type = Katalogtyp:{ "  " }
catalog-dialog-edit-label-repository-path = Repository-Pfad:
catalog-dialog-edit-label-description = Beschreibung:{ "   " }
catalog-dialog-edit-label-default-hlq = Standard-HLQ:{ "   " }
catalog-dialog-edit-chk-auto-mount = Beim Start automatisch einh{ "\u00e4" }ngen
catalog-dialog-edit-chk-read-only = Schreibgesch{ "\u00fc" }tzt
catalog-dialog-edit-err-description-length = Die Beschreibung darf h{ "\u00f6" }chstens 120 Zeichen lang sein.

# --- Delete Catalog dialog --------------------------------------------------
catalog-dialog-delete-title = Katalog l{ "\u00f6" }schen
catalog-dialog-delete-confirm = Katalog "{ $name }" l{ "\u00f6" }schen? Er wird ausgeh{ "\u00e4" }ngt.
catalog-dialog-delete-optional-files = Optional alle zugrunde liegenden Dateien l{ "\u00f6" }schen.
catalog-dialog-delete-btn-catalog-only = Nur Katalog l{ "\u00f6" }schen
catalog-dialog-delete-btn-catalog-and-files = Katalog und Dateien l{ "\u00f6" }schen
catalog-dialog-delete-err-home-protected = Der Home-Katalog kann nicht gel{ "\u00f6" }scht werden. Benennen Sie ihn stattdessen um oder bearbeiten Sie ihn.
catalog-dialog-delete-err-delete-failed = '{ $path }' konnte nicht gel{ "\u00f6" }scht werden: { $error }
