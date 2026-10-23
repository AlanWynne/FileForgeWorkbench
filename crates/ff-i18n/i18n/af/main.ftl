# Afrikaans (af) message catalogue (CR-NR-103 Task 10).
#
# REVIEWED POC LOCALE: Afrikaans is the hand-checked proof-of-concept locale for
# the localisation mechanism. It layers on top of the English Identity_Base; any
# key NOT present here falls back to the English value (localization Req 3.2).
# Keys and { $placeables } are IDENTICAL to en/main.ftl -- only the prose is
# translated. Product name "FileForge Workbench" is a proper noun, kept as-is
# (Req 4.2). Embedded command verbs stay English via non-translated args (Req 9.2).
# Non-ASCII Afrikaans letters are written as Fluent unicode escapes to keep the
# source ASCII-clean (matching the en convention), e.g. { "\u00eb" } = e-diaeresis.

app-name = FileForge Workbench

files-panel-title = L{ "\u00ea" }ers

ok-button = OK
cancel-button = Kanselleer

# Argument-bearing message: interpolates a runtime value (localization Req 9.1).
welcome-user = Welkom, { $name }.

# Verb-embedding message: the verb is passed as a non-translated argument so the
# prose localises while the verb stays English (localization Req 9.2).
profile-locked = Profiel is gesluit -- gebruik { $verb } om te ontsluit.
