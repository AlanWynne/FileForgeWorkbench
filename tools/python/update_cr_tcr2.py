LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\tcr-update2.txt"
TCR = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"

with open(LOG, "w", encoding="utf-8") as lf:
    lf.write("start\n")

def log(msg):
    with open(LOG, "a", encoding="utf-8") as lf:
        lf.write(msg + "\n")

with open(TCR, "rb") as f:
    data = f.read()

log(f"TCR size: {len(data)}")

RED = b"\xf0\x9f\x94\xb4"
GREEN = b"\xe2\x9c\x85"
MANUAL = b"\xf0\x9f\x94\xb2"

replacements = [
    # Req 16.1 -- theme.follow_os key registered
    (
        RED + b" | -- | Req 16.1: theme.follow_os config key (boolean, default false) |",
        GREEN + b" | theme_follow_os_key_is_registered_in_schema | Req 16.1: theme.follow_os config key (boolean, default false) |"
    ),
    # Req 16.2 -- follow_os=true + OS dark -> Dark (manual: needs egui context)
    (
        RED + b" | -- | Req 16.2: follow_os=true + OS dark -> Visual_Mode set to Dark |",
        MANUAL + b" | manual: egui ctx required | Req 16.2: follow_os=true + OS dark -> Visual_Mode set to Dark |"
    ),
    # Req 16.3 -- follow_os=true + OS light -> Light (manual)
    (
        RED + b" | -- | Req 16.3: follow_os=true + OS light -> Visual_Mode set to Light |",
        MANUAL + b" | manual: egui ctx required | Req 16.3: follow_os=true + OS light -> Visual_Mode set to Light |"
    ),
    # Req 16.4 -- follow_os=false -> OS ignored
    (
        RED + b" | -- | Req 16.4: follow_os=false -> OS preference ignored; theme.mode used |",
        GREEN + b" | theme_follow_os_false_does_not_change_palette | Req 16.4: follow_os=false -> OS preference ignored; theme.mode used |"
    ),
    # Req 16.5 -- OS preference change within one frame (manual)
    (
        RED + b" | -- | Req 16.5: OS preference change detected within one egui frame |",
        MANUAL + b" | manual: egui ctx required | Req 16.5: OS preference change detected within one egui frame |"
    ),
    # Req 16.6 -- read from ctx.style().visuals.dark_mode (manual)
    (
        RED + b" | -- | Req 16.6: OS preference read from egui ctx.style().visuals.dark_mode |",
        MANUAL + b" | manual: egui ctx required | Req 16.6: OS preference read from egui ctx.style().visuals.dark_mode |"
    ),
    # Req 16.7 -- auto-applied mode not persisted (manual)
    (
        RED + b" | -- | Req 16.7: auto-applied mode not persisted to theme.mode config key |",
        MANUAL + b" | manual: egui ctx required | Req 16.7: auto-applied mode not persisted to theme.mode config key |"
    ),
    # Req 16.8 -- Settings panel checkbox (manual UI)
    (
        RED + b" | -- | Req 16.8: Settings panel exposes theme.follow_os checkbox |",
        MANUAL + b" | manual: UI verification | Req 16.8: Settings panel exposes theme.follow_os checkbox |"
    ),
    # Req 12.1 -- Macro Library panel routing
    (
        RED + b" | -- | Req 12.1: Macro Library panel via POM option 6, MACROS command, =6 fastpath |",
        GREEN + b" | option_6_routes_to_macro_library | Req 12.1: Macro Library panel via POM option 6, MACROS command, =6 fastpath |"
    ),
    # Req 12.2 -- panel lists macros
    (
        RED + b" | -- | Req 12.2: panel lists all macros with name, source directory, full path |",
        GREEN + b" | refresh_discovers_lua_files_in_directory | Req 12.2: panel lists all macros with name, source directory, full path |"
    ),
    # Req 12.3 -- Run action (manual: Lua not yet wired)
    (
        RED + b" | -- | Req 12.3: Run action dispatches MACRO <name> and shows result in status bar |",
        MANUAL + b" | manual: Lua execution deferred | Req 12.3: Run action dispatches MACRO <name> and shows result in status bar |"
    ),
    # Req 12.4 -- Edit action opens file
    (
        RED + b" | -- | Req 12.4: Edit action opens .lua file in new editor tab |",
        MANUAL + b" | manual: UI verification | Req 12.4: Edit action opens .lua file in new editor tab |"
    ),
    # Req 12.5 -- Delete action
    (
        RED + b" | -- | Req 12.5: Delete action prompts confirmation then removes file and inventory entry |",
        MANUAL + b" | manual: UI verification | Req 12.5: Delete action prompts confirmation then removes file and inventory entry |"
    ),
]

changed = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        changed += 1
        log(f"  OK: {old[:60]}")
    else:
        log(f"  MISS: {old[:60]}")

with open(TCR, "wb") as f:
    f.write(data)

log(f"Done: {changed}/{len(replacements)} replacements")
