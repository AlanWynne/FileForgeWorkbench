LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\tcr-update3.txt"
TCR = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
RED = b"\xf0\x9f\x94\xb4"
GREEN = b"\xe2\x9c\x85"

with open(TCR, "rb") as f:
    data = f.read()

reps = [
    (RED + b" | -- | Req 12.6: filter input narrows list by case-insensitive name substring |",
     GREEN + b" | filter_narrows_visible_entries | Req 12.6: filter input narrows list by case-insensitive name substring |"),
    (RED + b" | -- | Req 12.7: panel state (selection, filter) not persisted across sessions |",
     GREEN + b" | macro_library_tab_not_persisted_in_session | Req 12.7: panel state (selection, filter) not persisted across sessions |"),
    (RED + b" | -- | Req 12.8: panel list refreshes within one frame when macro inventory changes |",
     GREEN + b" | refresh_clears_stale_entries_before_scan | Req 12.8: panel list refreshes within one frame when macro inventory changes |"),
]
changed = 0
for old, new in reps:
    if old in data:
        data = data.replace(old, new, 1)
        changed += 1

with open(TCR, "wb") as f:
    f.write(data)

with open(LOG, "w") as lf:
    lf.write(f"Done: {changed}/3\n")
