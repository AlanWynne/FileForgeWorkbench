# English Identity_Base keys for the Help Context panel
# (CR-NR-103, Localisation Phase 2, task 8.1, crate ff-desktop).
#
# Values are the EXACT current English literals from
# crates/ff-desktop/src/help_context.rs, byte-for-byte, ASCII only.
# Resolved through ff_i18n::t() (localization Req 4.1, 10.1, 10.2).
#
# NOT localised (Req 4.2, 4.4): topic.title() / topic.body() (help content,
# localised via the per-locale help/ directory in Task 6); the breadcrumb
# b.label values (navigation DATA); the help_search_field_id() id.

ff-desktop-help-context-search-label = Search:
ff-desktop-help-context-search-hint = type to search help
ff-desktop-help-context-no-topic = No help topic is open.
