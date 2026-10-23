# Afrikaans (af) Virtual Catalog New/Edit/Delete dialog keys (CR-NR-103 Task 10).
# REVIEWED POC locale. Keys + { $placeables } + alignment spaces identical to
# en/catalog-dialog.ftl; only prose translated. POSIX/Native/Mainframe are kept
# recognisable; "FileForge Workbench" / HLQ stay as-is.

# --- Catalog type display labels --------------------------------------------
catalog-dialog-type-mainframe = Hoofraam
catalog-dialog-type-posix = POSIX
catalog-dialog-type-native = Inheems

# --- Shared buttons ---------------------------------------------------------
catalog-dialog-btn-ok = OK
catalog-dialog-btn-cancel = Kanselleer
catalog-dialog-btn-save = Stoor

# --- New Catalog dialog -----------------------------------------------------
catalog-dialog-new-title = Nuwe Katalogus
catalog-dialog-new-label-type = Katalogustipe:
catalog-dialog-new-label-name = Katalogusnaam:{ "  " }
catalog-dialog-new-label-description = Beskrywing:{ "   " }
catalog-dialog-new-label-repository-path = Bewaarplekpad:
catalog-dialog-new-label-default-hlq = Verstek-HLQ:{ "    " }
catalog-dialog-new-label-root-directory = Wortelgids:{ " " }
catalog-dialog-new-label-mount-point = Hegpunt:{ "    " }
catalog-dialog-new-label-root-path = Wortelpad:{ "      " }
catalog-dialog-new-chk-auto-mount = Heg outomaties met opstart
catalog-dialog-new-chk-read-only = Leesalleen
catalog-dialog-new-chk-create-repository = Skep bewaarplek nou

# New Catalog validation messages.
catalog-dialog-new-err-name-format = Katalogusnaam moet 1-32 karakters wees (alfanumeries, koppelteken, onderstreep).
catalog-dialog-new-err-duplicate = 'n Katalogus genaamd '{ $name }' bestaan reeds.
catalog-dialog-new-err-description-length = Beskrywing moet 120 karakters of minder wees.
catalog-dialog-new-err-repository-path-required = Bewaarplekpad word vir Hoofraam-katalogusse vereis.
catalog-dialog-new-err-root-directory-required = Wortelgids word vir POSIX-katalogusse vereis.
catalog-dialog-new-err-root-path-required = Wortelpad word vir Inheemse katalogusse vereis.
catalog-dialog-new-err-create-repository-failed = Kon nie bewaarplek skep nie: { $error }

# --- Edit Catalog dialog ----------------------------------------------------
catalog-dialog-edit-title = Redigeer Katalogus
catalog-dialog-edit-label-name = Katalogusnaam:{ "  " }
catalog-dialog-edit-label-type = Katalogustipe:{ "  " }
catalog-dialog-edit-label-repository-path = Bewaarplekpad:
catalog-dialog-edit-label-description = Beskrywing:{ "   " }
catalog-dialog-edit-label-default-hlq = Verstek-HLQ:{ "   " }
catalog-dialog-edit-chk-auto-mount = Heg outomaties met opstart
catalog-dialog-edit-chk-read-only = Leesalleen
catalog-dialog-edit-err-description-length = Beskrywing moet 120 karakters of minder wees.

# --- Delete Catalog dialog --------------------------------------------------
catalog-dialog-delete-title = Skrap Katalogus
catalog-dialog-delete-confirm = Skrap katalogus "{ $name }"? Dit sal dit ontheg.
catalog-dialog-delete-optional-files = Skrap opsioneel alle onderliggende l{ "\u00ea" }ers.
catalog-dialog-delete-btn-catalog-only = Skrap slegs Katalogus
catalog-dialog-delete-btn-catalog-and-files = Skrap Katalogus en L{ "\u00ea" }ers
catalog-dialog-delete-err-home-protected = Die Tuis-katalogus kan nie geskrap word nie. Hernoem of redigeer dit eerder.
catalog-dialog-delete-err-delete-failed = Kon nie '{ $path }' skrap nie: { $error }
