# English Identity_Base keys for the built-in menu titles and option
# descriptions (CR-NR-103, localization Req 7.1, 7.4).
#
# These values are the EXACT current English prose from the code-only menu
# content in ff-desktop (menu_workspace/defaults.rs: DEFAULT_POM_TOML and
# DEFAULT_SETTINGS_TOML). Built-in menus are code-only TOML and are NEVER
# written to disk; render code localises title/description out of the parsed
# MenuFile by looking up these keys (localization Req 7.1, 7.4). English base
# only -- locale overlays (fr/de) layer on top later.

# --- Primary Option Menu (POM) ---------------------------------------------
menu-pom-title = FileForge Workbench -- Primary Option Menu
menu-pom-opt-settings-desc = FFWB Settings and Client Parameters
menu-pom-opt-catalogs-desc = Virtual File Catalogs -- Mainframe, POSIX, Native
menu-pom-opt-files-desc = File Explorer -- Browse catalogs and files in a tree view
menu-pom-opt-help-desc = Help -- workbench help and topics
menu-pom-opt-x-desc = Collapse to the visual root / close workspace (exit when last)

# --- Settings menu ----------------------------------------------------------
menu-settings-title = Settings
menu-settings-opt-config-desc = All settings -- browse every configuration key
menu-settings-opt-aliases-desc = Alias editor -- create, edit and save command aliases
menu-settings-opt-theme-desc = Theme editor -- copy, edit, save and select themes
menu-settings-opt-menus-desc = Menus editor -- create, change and save menus
menu-settings-opt-keys-desc = Keys -- edit and save per-workspace key assignments
menu-settings-opt-kinds-desc = Workspace kinds -- configure title, menu bar, key list, profile
menu-settings-opt-reset-bare-desc = Reset to barebones -- archive config and start fresh
