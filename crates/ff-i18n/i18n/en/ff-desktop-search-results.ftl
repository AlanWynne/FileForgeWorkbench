# English Identity_Base keys for the Search Results Context panel
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/search_results_panel/render.rs, byte-for-byte, ASCII
# only. Resolved through ff_i18n::t() / t_args() (Req 4.1, 9.1, 10.1, 10.2).
#
# NOT localised (Req 4.2, 4.4): the "Aa"/"W"/".*" checkbox mnemonic captions
# (their HOVER text IS localised); the "**/*.rs" / "**/target/**" glob example
# hints (technical examples); the "v" history-button glyph and the
# ">"/"v"-prefixed Replace-expand and file-collapse labels (directional UI
# affordances embedding data); the per-file header / per-match row formatting
# (file names, counts, line numbers, line text -- DATA); the query_field_id()
# id. Counts in the status/confirm messages are passed as TEXT placeables
# (pre-formatted in Rust), NOT Fluent NUMBER(), so rendering stays byte-
# identical for counts >= 1000 (no locale grouping); the result-row data is
# not routed through the catalogue.

ff-desktop-search-results-search-label = Search:
ff-desktop-search-results-search-hint = Search across files...
ff-desktop-search-results-history-hover = Search history
ff-desktop-search-results-search-button = Search
ff-desktop-search-results-cancel = Cancel
ff-desktop-search-results-hover-case = Case sensitive
ff-desktop-search-results-hover-word = Whole word
ff-desktop-search-results-hover-regex = Use regex
ff-desktop-search-results-include-label = Include:
ff-desktop-search-results-exclude-label = Exclude:
ff-desktop-search-results-replace-hint = Replacement text
ff-desktop-search-results-replace-all = Replace All
ff-desktop-search-results-confirm-title = Confirm Replace All
ff-desktop-search-results-confirm-body = Replace { $mc } occurrence(s) in { $fc } file(s)?
ff-desktop-search-results-replace = Replace
ff-desktop-search-results-running = Searching... { $files } files scanned, { $matches } matches
ff-desktop-search-results-done = { $matches } matches in { $files } files
ff-desktop-search-results-cancelled = Search cancelled.
