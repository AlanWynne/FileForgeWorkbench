# Command-verb display surfaces -- Afrikaans (af) POC (CR-NR-109, localization
# Requirement 11, Task 11.7).
#
# The Afrikaans surface for the FIND command verb. An alias row in
# af/aliases/FFCMD.toml maps the typeable surface SOEK to the root verb FIND and
# references this key, so the menu-kind command column and the typeable form both
# show "SOEK" under the af locale while the ROOT dispatched/persisted verb stays
# FIND (Requirement 11.9).

verb-find = SOEK

# --- POM + Settings menu-root command surfaces (CR-NR-111 full af menu DATA) ---
# DRAFT Afrikaans surfaces, pending fluent review. Each maps 1:1 to an FFCMD root
# referenced by a row in af/aliases/FFCMD.toml. The ROOT dispatched/persisted verb
# is unchanged (Req 11.9); only the displayed/typeable surface localises.
verb-settings = INSTELLINGS
verb-catalogs = KATALOGUSSE
verb-files = LEERS
verb-help = HULP
verb-config = KONFIG
verb-aliases = ALIASSE
verb-theme = TEMA
verb-menus = KIESLYSTE
verb-keys = SLEUTELS
verb-kinds = SOORTE
verb-reset-bare = HERSTEL KAAL
