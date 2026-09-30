# The three-level model

FFWB follows a documented three-level UI model, and the code matches it:

1. Workbench — the application window. This is WorkbenchShell (mod.rs), which implements eframe::App. It's the single point of contact with egui and owns everything: the Tokio runtime, the active theme palette, the command registry/engine, and the TabManager that holds all open tabs. It renders the menu bar, the tab bar, the central panel, the command line, and the status/key-label bars.

2. Workspace — one tab in the tab bar. Modeled as TabState (tab_state.rs), managed by TabManager (tab_manager.rs). Each TabState carries its own document handle, viewport (scroll), cursor, undo stack, edit profile, and a per-tab navigation stack. Switching tabs preserves all of that.

3. Context — the content inside a workspace. This is the TabKind enum on each TabState. render.rs matches on tabs.active_tab().kind to draw the right panel. The kinds that exist: FileEditor, Untitled, FilesPanel (Catalog Manager / [CATALOGS]), ConfigPanel, FileExplorerPanel ([FILES]), SearchResults, PluginManager, EventLog, MacroLibrary, MenuWorkspace (this is also the Home Context / POM when is_home is true), CommandConfigurator, ThemeEditor, MenusEditor, KeysEditor, and KindsEditor.

##  Menu bar

The menu bar is data-driven (CR-NR-080). It's rendered from the compiled default Menu_Bar rather than a hardcoded label list, with the top-level entries coming from the default menu bar's option descriptions.

## Opening and closing workspaces.

Opening: new POM tabs, untitled files, or file editors are added through TabManager (deferred flags like pending_new_pom / pending_new_file, or file.open). POM options (typed digits or clicks) route through handle_command, which opens the target context in place.

Closing: the CLOSE command calls tabs.close_tab(active_index). END pops one level of the tab's navigation stack and closes the workspace when the stack is empty; RETURN collapses to the root in one step. TabManager::close_tab always keeps at least one tab open, so the caller decides the last-workspace termination case.
The command line — important correction
Your mental model was "each workspace has its own command line." That's only partly true in the current code. There is one shared Command ===> field owned by the shell (command_text on WorkbenchShell), acting on whichever tab is active. The command field has a single stable egui id (cmd_field_id), and everything flows through handle_command().

The one case where a command line is genuinely per-window is a Detached Workspace — a tab detached into its own OS window (DETACH command, formerly SPLIT DETACH). Each FloatingTab owns a WorkspaceCommandContext: its own command buffer, SCROLL field, status line, and focus latches. The shell swaps that context in, runs the same command pipeline, and swaps it back, so a detached window's command line acts only on its own tab. Docked tabs all share the shell's one command line.

## Split — this has real history worth knowing

The word "split" means two different things, and one of them is deprecated:

SPLIT DETACH — deprecated alias of DETACH. Detaches the active workspace into a floating OS window. The old "SPLIT splits the editor screen" behaviour (an inert SplitScreenState that never rendered) was retired.

SPLIT / SPLIT RIGHT / SPLIT DOWN — the real in-window split, built out across CR-NR-091/092/093 (B046). This is what splits a workspace region into two regions. Under the hood TabManager now models arrangement as a TabGroupTree (from ff-layout): a single Leaf when unsplit, or a recursive Split tree of arbitrary depth when split. Key commands:

SPLIT / SPLIT RIGHT → split_focused(Horizontal) (side by side)
SPLIT DOWN → split_focused(Vertical) (stacked)
UNSPLIT → collapses the split around the focused leaf
FOCUS / FOCUS OTHER → cycles focus to the next region
END while split → collapses the split first (acts as "close this region")
Each split leaf (Tab_Group) holds its own set of tabs and its own active tab, and splitting nests to arbitrary depth. The flat tab store stays authoritative for tab content; the tree is authoritative for arrangement, reconciled by sync_layout after every mutation. The layout is even persisted structurally (identity-free LayoutDescriptor) so a split arrangement survives a restart.

One nuance on your question "a workspace has two regions each with their own command line": with the in-window split, the regions are Tab_Groups within the shell window, and they still share the shell's single Command ===> line (it acts on the focused region's active tab). Independent per-window command lines exist only for detached workspaces, not for in-window split regions.

## Summary

Workbench (WorkbenchShell / eframe::App)
├── Menu bar (data-driven)
├── Tab bar  ──> TabManager
│     └── Workspace (TabState)  [one per tab]
│           ├── Context (TabKind: POM/Editor/Config/Files/…)
│           ├── own document / viewport / cursor / undo / nav-stack
│           └── may be split into regions (TabGroupTree of Tab_Groups)
├── Command ===> line  [ONE shared, acts on the focused/active tab]
├── SCROLL / status / key-label bars
└── Detached Workspaces (FloatingTab)  [each an OS window with its OWN command line]

So: one Workbench, many Workspaces (tabs), each showing one Context; in-window SPLIT gives multiple regions sharing the shell's command line; DETACH gives a separate OS window that does get its own independent command line.

Want me to go deeper on any piece — for example how the split render walk and drag-between-regions works, or how the shared command line resolves commands through the resolution chain?