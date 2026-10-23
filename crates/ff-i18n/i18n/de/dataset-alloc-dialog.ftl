# German (de) Dataset Allocation dialog (CR-NR-103 Task 10).
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. Keys identical to
# en/dataset-alloc-dialog.ftl; only prose translated. DSORG/RECFM mnemonic codes
# (PS/PO/PDSE/GDG, FB/F/VB/V/U) and the abbreviations LRECL/BLKSIZE/GDG/HLQ stay
# as technical identity literals (Req 4.4). Trailing alignment spaces preserved
# with quoted placeables. { $lrecl } / { $blksize } / { $limit } / { $name } are
# placeables.

# --- Shared buttons ---------------------------------------------------------
dataset-alloc-btn-allocate = Zuweisen
dataset-alloc-btn-cancel = Abbrechen

# --- Allocate Dataset dialog labels -----------------------------------------
dataset-alloc-title = Dataset zuweisen
dataset-alloc-label-name = Dataset-Name:{ "      " }
dataset-alloc-label-dsorg = Dataset-Org. (DSORG):
dataset-alloc-label-recfm = Satzformat:{ "     " }
dataset-alloc-label-lrecl = LRECL:{ "             " }
dataset-alloc-label-blksize = { "  " }Blockgr{ "\u00f6" }{ "\u00df" }e:
dataset-alloc-label-dir-blocks = Verzeichnisbl{ "\u00f6" }cke:{ "  " }
dataset-alloc-label-gdg-limit = GDG-Grenze (1-255):{ " " }
dataset-alloc-label-description = Beschreibung:{ "       " }
dataset-alloc-chk-scratch = Beim Abrollen l{ "\u00f6" }schen

# --- Validation messages (Req 5.3, 5.8, 5.9) --------------------------------
dataset-alloc-err-name-required = Der Dataset-Name ist erforderlich.
dataset-alloc-err-lrecl-integer = LRECL muss eine positive Ganzzahl sein.
dataset-alloc-err-lrecl-range = LRECL muss zwischen 1 und 32760 liegen (erhalten { $lrecl }).
dataset-alloc-err-blksize-integer = Die Blockgr{ "\u00f6" }{ "\u00df" }e muss eine nicht-negative Ganzzahl sein.
dataset-alloc-err-blksize-range = Die Blockgr{ "\u00f6" }{ "\u00df" }e ({ $blksize }) muss >= LRECL ({ $lrecl }) sein, oder 0 f{ "\u00fc" }r systembestimmt.
dataset-alloc-err-dir-blocks-integer = Verzeichnisbl{ "\u00f6" }cke m{ "\u00fc" }ssen eine positive Ganzzahl sein.
dataset-alloc-err-gdg-limit-integer = Die GDG-Grenze muss eine Ganzzahl zwischen 1 und 255 sein.
dataset-alloc-err-gdg-limit-range = Die GDG-Grenze muss zwischen 1 und 255 liegen (erhalten { $limit }).
dataset-alloc-err-duplicate = Das Dataset '{ $name }' existiert bereits in diesem Katalog.
