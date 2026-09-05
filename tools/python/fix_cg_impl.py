import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

# Clear log
open(LOG, "w").close()
log("fix_cg_impl.py started")

replacements = [
    (
        r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\lua-macro-engine\tasks.md",
        [
            (b"- [ ] 21. ISPF host command environments", b"- [x] 21. ISPF host command environments"),
            (b"  - [ ] 21.1 Implement ISREDIT", b"  - [x] 21.1 Implement ISREDIT"),
            (b"  - [ ] 21.2 Implement ISPEXEC", b"  - [x] 21.2 Implement ISPEXEC"),
            (b"  - [ ] 21.3 Implement IMACRO initial", b"  - [x] 21.3 Implement IMACRO initial"),
            (b"  - [ ] 21.4 Implement IMACRO edit profile", b"  - [x] 21.4 Implement IMACRO edit profile"),
            (b"  - [ ] 21.5 Implement LINENUM", b"  - [x] 21.5 Implement LINENUM"),
            (b"  - [ ] 21.6 Extend cursor API", b"  - [x] 21.6 Extend cursor API"),
            (b"  - [ ] 21.7 Write unit tests for ISREDIT", b"  - [x] 21.7 Write unit tests for ISREDIT"),
            (b"- [ ] 22. REXX execution bridge", b"- [x] 22. REXX execution bridge"),
            (b"  - [ ] 22.1 Implement EXEC command:", b"  - [x] 22.1 Implement EXEC command:"),
            (b"  - [ ] 22.2 Implement implicit exec", b"  - [x] 22.2 Implement implicit exec"),
            (b"  - [ ] 22.3 Implement % prefix:", b"  - [x] 22.3 Implement % prefix:"),
            (b"  - [ ] 22.4 Implement argument passing:", b"  - [x] 22.4 Implement argument passing:"),
            (b"  - [ ] 22.5 Implement TSO host command", b"  - [x] 22.5 Implement TSO host command"),
            (b"  - [ ] 22.6 Implement ADDRESS <environment-name>:", b"  - [x] 22.6 Implement ADDRESS <environment-name>:"),
            (b"  - [ ] 22.7 Implement ISPEXEC and ISREDIT as ADDRESS", b"  - [x] 22.7 Implement ISPEXEC and ISREDIT as ADDRESS"),
            (b"  - [ ] 22.8 Implement RC special variable:", b"  - [x] 22.8 Implement RC special variable:"),
            (b"  - [ ] 22.9 Write unit tests for exec location", b"  - [x] 22.9 Write unit tests for exec location"),
            (b"- [ ] 23. REXX built-in functions", b"- [x] 23. REXX built-in functions"),
            (b"  - [ ] 23.1 Implement LISTDSI:", b"  - [x] 23.1 Implement LISTDSI:"),
            (b"  - [ ] 23.2 Implement MSG:", b"  - [x] 23.2 Implement MSG:"),
            (b"  - [ ] 23.3 Implement MVSVAR:", b"  - [x] 23.3 Implement MVSVAR:"),
            (b"  - [ ] 23.4 Implement OUTTRAP:", b"  - [x] 23.4 Implement OUTTRAP:"),
            (b"  - [ ] 23.5 Implement PROMPT:", b"  - [x] 23.5 Implement PROMPT:"),
            (b"  - [ ] 23.6 Implement SYSDSN:", b"  - [x] 23.6 Implement SYSDSN:"),
            (b"  - [ ] 23.7 Implement SYSVAR:", b"  - [x] 23.7 Implement SYSVAR:"),
            (b"  - [ ] 23.8 Implement USERID:", b"  - [x] 23.8 Implement USERID:"),
            (b"  - [ ] 23.9 Write unit tests for each built-in", b"  - [x] 23.9 Write unit tests for each built-in"),
            (b"- [ ] 24. EXECIO I/O operations", b"- [x] 24. EXECIO I/O operations"),
            (b"  - [ ] 24.1 Implement EXECIO DISKR:", b"  - [x] 24.1 Implement EXECIO DISKR:"),
            (b"  - [ ] 24.2 Implement EXECIO DISKW:", b"  - [x] 24.2 Implement EXECIO DISKW:"),
            (b"  - [ ] 24.3 Implement EXECIO FINIS", b"  - [x] 24.3 Implement EXECIO FINIS"),
            (b"  - [ ] 24.4 Implement EXECIO SKIP:", b"  - [x] 24.4 Implement EXECIO SKIP:"),
            (b"  - [ ] 24.5 Implement EXECIO return codes:", b"  - [x] 24.5 Implement EXECIO return codes:"),
            (b"  - [ ] 24.6 Implement FFCMD command files:", b"  - [x] 24.6 Implement FFCMD command files:"),
            (b"  - [ ] 24.7 Implement FFCMD transaction wrapping:", b"  - [x] 24.7 Implement FFCMD transaction wrapping:"),
            (b"  - [ ] 24.8 Write unit tests for DISKR/DISKW", b"  - [x] 24.8 Write unit tests for DISKR/DISKW"),
        ]
    ),
    (
        r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md",
        [
            (b"- [ ] CG.impl lua-macro-engine:", b"- [x] CG.impl lua-macro-engine:"),
        ]
    ),
]

for path, pairs in replacements:
    with open(path, "rb") as f:
        data = f.read()
    log(f"File: {path} ({len(data)} bytes)")
    count = 0
    for old, new in pairs:
        if old in data:
            data = data.replace(old, new, 1)
            count += 1
        else:
            log(f"  WARNING: pattern not found: {old[:60]}")
    with open(path, "wb") as f:
        f.write(data)
    log(f"  {count}/{len(pairs)} replacements made")

log("Done")
