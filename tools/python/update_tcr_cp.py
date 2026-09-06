import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\update_tcr_cp.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

path = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"

with open(path, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

# Detect line separator
sep = b"\r\n" if b"\r\n" in data else b"\n"
log(f"Line separator: {repr(sep)}")

lines = data.split(sep)
log(f"Total lines: {len(lines)}")

# Find the Phase CP section header
header = b"### Phase CP -- Batch Command Execution (CR-NR-041)"
start_idx = None
for i, line in enumerate(lines):
    if header in line:
        start_idx = i
        log(f"Found Phase CP header at line {i}: {line[:80]}")
        break

if start_idx is None:
    log("ERROR: Phase CP header not found")
    sys.exit(1)

# Find the end of the Phase CP table (next ### heading or end of file)
end_idx = len(lines)
for i in range(start_idx + 1, len(lines)):
    stripped = lines[i].strip()
    if stripped.startswith(b"###") or stripped.startswith(b"##"):
        end_idx = i
        log(f"Phase CP section ends at line {i}: {lines[i][:80]}")
        break

log(f"Phase CP section: lines {start_idx} to {end_idx-1} ({end_idx - start_idx} lines)")

# Build the replacement section
new_section_lines = [
    b"### Phase CP -- Batch Command Execution (CR-NR-041)",
    b"",
    b"| Crate | Status | Test files | Notes |",
    b"|-------|--------|-----------|-------|",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::cli` unit tests, `main.rs` | Req 1.1: ffwb --batch <file> executes in headless batch mode; no GUI window opened |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::cli` unit tests | Req 1.2: ffwb --batch - reads commands from stdin |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::cli` unit tests | Req 1.3: --batch combined with file path arguments rejected with error |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::cli` unit tests, `main.rs` | Req 1.4: without --batch, workbench starts in normal interactive GUI mode |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 1.5: --batch documented in ffwb --help |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::mod::run_batch()` | Req 1.6: missing/unreadable batch file exits with return code 12 |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::input` unit tests | Req 2.1: Batch_Input_Source accepts UTF-8 with or without BOM; BOM stripped |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::input` unit tests | Req 2.2: each non-blank non-comment line submitted as one Batch_Command in order |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::input` unit tests | Req 2.3: lines beginning with * treated as comments and skipped |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::input` unit tests | Req 2.4: lines beginning with /* treated as comments and skipped |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::input` unit tests | Req 2.5: blank/whitespace-only lines skipped |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::input` unit tests | Req 2.6: line ending with - continues onto next line |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::input` unit tests | Req 2.7: lines exceeding 32767 chars truncated with warning |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 3.1: Batch_Commands dispatched through ff-command-semantics pipeline unchanged |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 3.2: Batch_Session provides same catalog registry and config as interactive session |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 3.3: command output written to Batch_Output_Sink |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 3.4: document modifications applied to real filesystem |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 3.5: GUI-requiring commands fail with Step_Return_Code 8 and diagnostic message |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::runner` unit tests | Req 3.6: commands executed sequentially |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::mod::run_batch()` | Req 4.1: default output to stdout |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::cli` unit tests, `batch::mod::run_batch()` | Req 4.2: --batch-output <file> writes output to specified file |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::cli` unit tests, `batch::mod::run_batch()` | Req 4.3: --batch-output-append <file> appends output to specified file |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::runner` unit tests | Req 4.4: --batch-echo prefixes each command output with ===> <command text> |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 4.5: without --batch-echo, command text not written to output |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 4.6: BatchRunner diagnostic output written to stderr |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::mod::run_batch()` | Req 4.7: unwritable output file exits with return code 12 before executing commands |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::return_code` unit tests | Req 5.1: all commands succeed -> exit code 0 |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::return_code` unit tests | Req 5.2: Batch_Return_Code is maximum Step_Return_Code across all commands |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::return_code` unit tests | Req 5.3: Step_Return_Code values 0/4/8/12/16 used per z/OS convention |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 5.4: BatchRunner init failure exits with code 12 |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::mod::run_batch()` | Req 5.5: final summary line \"FFWB BATCH RETURN CODE: N\" written to stderr |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::return_code` unit tests | Req 6.1: default mode continues after command failure (best-effort) |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::cli` unit tests, `batch::return_code` unit tests | Req 6.2: --batch-abort-on-error <threshold> stops on Step_Return_Code >= threshold |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 6.3: abort writes message identifying aborting command and return code |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 6.4: Batch_Return_Code reflects aborting command's Step_Return_Code |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 6.5: CONTROL ERRORS CANCEL / NOCANCEL inline commands override abort policy |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 7.1: Batch_Session loads same config layers as interactive session |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 7.2: Batch_Session loads catalog registry from same catalogs.toml |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::cli` unit tests | Req 7.3: --batch-profile <name> loads named config profile |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 7.4: Batch_Session does NOT restore GUI session state |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 7.5: batch run does NOT overwrite interactive session state file |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::cli` unit tests | Req 7.6: --batch-no-catalog starts with empty catalog registry |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::runner` unit tests | Req 8.1: --batch-dry-run parses/validates commands without modifying filesystem |",
    b"| `ff-desktop` | \xe2\x9c\x85 | `batch::runner` unit tests | Req 8.2: dry-run writes [DRY-RUN] <command> -> OK|ERROR: reason for each command |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 8.3: dry-run return code 0 if all valid, 8 if any syntax error or missing resource |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 8.4: read-only commands execute normally in dry-run mode |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 9.1: Batch_Input_Source format identical to .ffcmd format |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 9.2: .ffcmd file usable as --batch input without modification |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 9.3: .ffcmd extension recognised; other extensions also accepted |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 9.4: .ffcmd via --batch does NOT invoke Lua engine |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 10.1: structured log written to ff-logging: start, each command+RC+duration, final RC |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 10.2: --batch-log <file> redirects structured log to specified file |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 10.3: log includes wall-clock duration of each command |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 10.4: Step_Return_Code >= 8 logged at ERROR level with full error detail |",
    b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 10.5: log format matches ff-logging structured format (timestamp, level, component, message) |",
]

new_lines = lines[:start_idx] + new_section_lines + lines[end_idx:]
new_data = sep.join(new_lines)

with open(path, "wb") as f:
    f.write(new_data)

log(f"Written {len(new_data)} bytes ({len(new_lines)} lines)")
log("Done")
