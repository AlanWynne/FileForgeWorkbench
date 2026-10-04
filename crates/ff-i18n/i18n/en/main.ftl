# English Identity_Base message catalogue (CR-NR-103, localization Req 2.5).
#
# This is the always-present baseline catalogue. Every locale resolves against
# these keys as the identity base; a locale overlay layers its own translations
# on top (localization Req 3.2). The keys below are the initial baseline that
# proves the Catalogue_Lookup_Seam; per-crate extraction adds more keys as sites
# are converted (localization tasks.md, Phase 2).
#
# Messages MAY carry named placeable arguments ({ $arg }) for interpolation
# (localization Req 2.1, 9.1). A message that embeds a command verb keeps the
# verb as a NON-translated argument (localization Req 9.2); only the surrounding
# prose is translatable.

app-name = FileForge Workbench

files-panel-title = Files

ok-button = OK
cancel-button = Cancel

# Argument-bearing message: interpolates a runtime value (localization Req 9.1).
welcome-user = Welcome, { $name }.

# Verb-embedding message: the verb is passed as a non-translated argument so the
# prose localises while the verb stays English (localization Req 9.2).
profile-locked = Profile is locked -- use { $verb } to unlock.
