# French (fr) Dataset Allocation dialog (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/dataset-alloc-dialog.ftl; only prose translated. DSORG/RECFM mnemonic codes
# (PS/PO/PDSE/GDG, FB/F/VB/V/U) and the abbreviations LRECL/BLKSIZE/GDG/HLQ stay
# as technical identity literals (Req 4.4). Trailing alignment spaces preserved
# with quoted placeables. { $lrecl } / { $blksize } / { $limit } / { $name } are
# placeables.

# --- Shared buttons ---------------------------------------------------------
dataset-alloc-btn-allocate = Allouer
dataset-alloc-btn-cancel = Annuler

# --- Allocate Dataset dialog labels -----------------------------------------
dataset-alloc-title = Allouer un dataset
dataset-alloc-label-name = Nom du dataset :{ "      " }
dataset-alloc-label-dsorg = Org. dataset (DSORG) :
dataset-alloc-label-recfm = Format d'enreg. :{ "     " }
dataset-alloc-label-lrecl = LRECL :{ "             " }
dataset-alloc-label-blksize = { "  " }Taille de bloc :
dataset-alloc-label-dir-blocks = Blocs de r{ "\u00e9" }pertoire :{ "  " }
dataset-alloc-label-gdg-limit = Limite GDG (1-255) :{ " " }
dataset-alloc-label-description = Description :{ "       " }
dataset-alloc-chk-scratch = Supprimer au d{ "\u00e9" }bordement

# --- Validation messages (Req 5.3, 5.8, 5.9) --------------------------------
dataset-alloc-err-name-required = Le nom du dataset est requis.
dataset-alloc-err-lrecl-integer = LRECL doit { "\u00ea" }tre un entier positif.
dataset-alloc-err-lrecl-range = LRECL doit { "\u00ea" }tre compris entre 1 et 32760 (re{ "\u00e7" }u { $lrecl }).
dataset-alloc-err-blksize-integer = La taille de bloc doit { "\u00ea" }tre un entier non n{ "\u00e9" }gatif.
dataset-alloc-err-blksize-range = La taille de bloc ({ $blksize }) doit { "\u00ea" }tre >= LRECL ({ $lrecl }), ou 0 pour d{ "\u00e9" }termin{ "\u00e9" }e par le syst{ "\u00e8" }me.
dataset-alloc-err-dir-blocks-integer = Les blocs de r{ "\u00e9" }pertoire doivent { "\u00ea" }tre un entier positif.
dataset-alloc-err-gdg-limit-integer = La limite GDG doit { "\u00ea" }tre un entier entre 1 et 255.
dataset-alloc-err-gdg-limit-range = La limite GDG doit { "\u00ea" }tre comprise entre 1 et 255 (re{ "\u00e7" }u { $limit }).
dataset-alloc-err-duplicate = Le dataset '{ $name }' existe d{ "\u00e9" }j{ "\u00e0" } dans ce catalogue.
