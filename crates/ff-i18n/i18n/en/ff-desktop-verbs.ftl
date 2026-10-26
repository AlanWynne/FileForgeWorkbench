# Command-verb display surfaces (CR-NR-109, localization Requirement 11).
#
# These keys supply the localised DISPLAYED/typeable surface for a command verb
# that an alias row references via its `localisation_key`. The English
# Identity_Base value is the root verb itself, so the `en` display is
# behaviour-neutral (the surface equals the root). Non-English catalogues
# (e.g. af/ff-desktop-verbs.ftl) translate the same key to the local surface.
#
# This is the MINIMAL verb-key set the CR-NR-109 Afrikaans POC references
# (Task 11.7); it is NOT the full command-string extraction (out of scope).

verb-find = FIND

# --- POM + Settings menu-root command surfaces (en Identity_Base) ---
# Identity values = the root verb itself, so the en display equals the root.
# Non-English catalogues (af/ff-desktop-verbs.ftl) translate these same keys.
verb-settings = SETTINGS
verb-catalogs = CATALOGS
verb-files = FILES
verb-help = HELP
verb-config = CONFIG
verb-aliases = ALIASES
verb-theme = THEME
verb-menus = MENUS
verb-keys = KEYS
verb-kinds = KINDS
verb-reset-bare = RESET BARE
