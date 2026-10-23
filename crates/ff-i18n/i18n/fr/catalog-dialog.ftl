# French (fr) Virtual Catalog New/Edit/Delete dialogs (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/catalog-dialog.ftl; only prose translated. Catalog type DISPLAY labels
# (Mainframe/POSIX/Native) are kept as proper identity terms. { $name } /
# { $path } / { $error } are placeables. Trailing alignment spaces are preserved
# with quoted placeables so column widths stay identical to en.

# --- Catalog type display labels (shared New + Edit; Req 4.1) ---------------
catalog-dialog-type-mainframe = Mainframe
catalog-dialog-type-posix = POSIX
catalog-dialog-type-native = Natif

# --- Shared buttons ---------------------------------------------------------
catalog-dialog-btn-ok = OK
catalog-dialog-btn-cancel = Annuler
catalog-dialog-btn-save = Enregistrer

# --- New Catalog dialog -----------------------------------------------------
catalog-dialog-new-title = Nouveau catalogue
catalog-dialog-new-label-type = Type de catalogue :
catalog-dialog-new-label-name = Nom du catalogue :{ "  " }
catalog-dialog-new-label-description = Description :{ "   " }
catalog-dialog-new-label-repository-path = Chemin du d{ "\u00e9" }p{ "\u00f4" }t :
catalog-dialog-new-label-default-hlq = HLQ par d{ "\u00e9" }faut :{ "    " }
catalog-dialog-new-label-root-directory = R{ "\u00e9" }pertoire racine :{ " " }
catalog-dialog-new-label-mount-point = Point de montage :{ "    " }
catalog-dialog-new-label-root-path = Chemin racine :{ "      " }
catalog-dialog-new-chk-auto-mount = Monter automatiquement au d{ "\u00e9" }marrage
catalog-dialog-new-chk-read-only = Lecture seule
catalog-dialog-new-chk-create-repository = Cr{ "\u00e9" }er le d{ "\u00e9" }p{ "\u00f4" }t maintenant

# New Catalog validation messages (Req 3.3-3.8).
catalog-dialog-new-err-name-format = Le nom du catalogue doit comporter 1 { "\u00e0" } 32 caract{ "\u00e8" }res (alphanum{ "\u00e9" }riques, tiret, soulignement).
catalog-dialog-new-err-duplicate = Un catalogue nomm{ "\u00e9" } '{ $name }' existe d{ "\u00e9" }j{ "\u00e0" }.
catalog-dialog-new-err-description-length = La description doit comporter au maximum 120 caract{ "\u00e8" }res.
catalog-dialog-new-err-repository-path-required = Le chemin du d{ "\u00e9" }p{ "\u00f4" }t est requis pour les catalogues Mainframe.
catalog-dialog-new-err-root-directory-required = Le r{ "\u00e9" }pertoire racine est requis pour les catalogues POSIX.
catalog-dialog-new-err-root-path-required = Le chemin racine est requis pour les catalogues Natif.
catalog-dialog-new-err-create-repository-failed = { "\u00c9" }chec de la cr{ "\u00e9" }ation du d{ "\u00e9" }p{ "\u00f4" }t : { $error }

# --- Edit Catalog dialog ----------------------------------------------------
catalog-dialog-edit-title = Modifier le catalogue
catalog-dialog-edit-label-name = Nom du catalogue :{ "  " }
catalog-dialog-edit-label-type = Type de catalogue :{ "  " }
catalog-dialog-edit-label-repository-path = Chemin du d{ "\u00e9" }p{ "\u00f4" }t :
catalog-dialog-edit-label-description = Description :{ "   " }
catalog-dialog-edit-label-default-hlq = HLQ par d{ "\u00e9" }faut :{ "   " }
catalog-dialog-edit-chk-auto-mount = Monter automatiquement au d{ "\u00e9" }marrage
catalog-dialog-edit-chk-read-only = Lecture seule
catalog-dialog-edit-err-description-length = La description doit comporter au maximum 120 caract{ "\u00e8" }res.

# --- Delete Catalog dialog --------------------------------------------------
catalog-dialog-delete-title = Supprimer le catalogue
catalog-dialog-delete-confirm = Supprimer le catalogue "{ $name }" ? Il sera d{ "\u00e9" }mont{ "\u00e9" }.
catalog-dialog-delete-optional-files = Supprimer { "\u00e9" }galement tous les fichiers sous-jacents (facultatif).
catalog-dialog-delete-btn-catalog-only = Supprimer le catalogue uniquement
catalog-dialog-delete-btn-catalog-and-files = Supprimer le catalogue et les fichiers
catalog-dialog-delete-err-home-protected = Le catalogue Home ne peut pas { "\u00ea" }tre supprim{ "\u00e9" }. Renommez-le ou modifiez-le { "\u00e0" } la place.
catalog-dialog-delete-err-delete-failed = { "\u00c9" }chec de la suppression de '{ $path }' : { $error }
