# Afrikaans (af) Dataset Allocation dialog (CR-NR-103 Task 10). REVIEWED POC.
# Keys + { $placeables } + alignment spaces identical to en. DSORG/RECFM codes
# (PS/PO/FB/...) and the (DSORG)/(LRECL)/(GDG) mnemonics stay English (technical
# mainframe format codes). { $lrecl }/{ $blksize }/{ $limit }/{ $name } are DATA.

# --- Shared buttons ---------------------------------------------------------
dataset-alloc-btn-allocate = Ken toe
dataset-alloc-btn-cancel = Kanselleer

# --- Allocate Dataset dialog labels -----------------------------------------
dataset-alloc-title = Ken Datastel Toe
dataset-alloc-label-name = Datastelnaam:{ "      " }
dataset-alloc-label-dsorg = Datastelorg. (DSORG):
dataset-alloc-label-recfm = Rekordformaat:{ "     " }
dataset-alloc-label-lrecl = LRECL:{ "             " }
dataset-alloc-label-blksize = { "  " }Blokgrootte:
dataset-alloc-label-dir-blocks = Gidsblokke:{ "  " }
dataset-alloc-label-gdg-limit = GDG-limiet (1-255):{ " " }
dataset-alloc-label-description = Beskrywing:{ "       " }
dataset-alloc-chk-scratch = Skrap met afrol

# --- Validation messages ----------------------------------------------------
dataset-alloc-err-name-required = Datastelnaam word vereis.
dataset-alloc-err-lrecl-integer = LRECL moet 'n positiewe heelgetal wees.
dataset-alloc-err-lrecl-range = LRECL moet tussen 1 en 32760 wees (gekry { $lrecl }).
dataset-alloc-err-blksize-integer = Blokgrootte moet 'n nie-negatiewe heelgetal wees.
dataset-alloc-err-blksize-range = Blokgrootte ({ $blksize }) moet >= LRECL ({ $lrecl }) wees, of 0 vir stelselbepaald.
dataset-alloc-err-dir-blocks-integer = Gidsblokke moet 'n positiewe heelgetal wees.
dataset-alloc-err-gdg-limit-integer = GDG-limiet moet 'n heelgetal tussen 1 en 255 wees.
dataset-alloc-err-gdg-limit-range = GDG-limiet moet tussen 1 en 255 wees (gekry { $limit }).
dataset-alloc-err-duplicate = Datastel '{ $name }' bestaan reeds in hierdie katalogus.
