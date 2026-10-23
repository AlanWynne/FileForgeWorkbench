# German (de) message catalogue (CR-NR-103 Task 10).
#
# DRAFT -- AI-DRAFTED, PENDING NATIVE-SPEAKER REVIEW. This locale layers on top of
# the English Identity_Base; any key NOT present here falls back to the English
# value (localization Req 3.2). Keys and { $placeables } are IDENTICAL to
# en/main.ftl -- only the prose is translated. Product name "FileForge Workbench"
# is a proper noun, kept as-is (Req 4.2). Embedded command verbs stay English via
# non-translated args (Req 9.2). Non-ASCII German letters are written as Fluent
# unicode escapes to keep the source ASCII-clean, e.g. { "\u00fc" } = u-umlaut,
# { "\u00df" } = sharp-s.

app-name = FileForge Workbench

files-panel-title = Dateien

ok-button = OK
cancel-button = Abbrechen

# Argument-bearing message: interpolates a runtime value (localization Req 9.1).
welcome-user = Willkommen, { $name }.

# Verb-embedding message: the verb is passed as a non-translated argument so the
# prose localises while the verb stays English (localization Req 9.2).
profile-locked = Profil ist gesperrt -- verwenden Sie { $verb } zum Entsperren.
