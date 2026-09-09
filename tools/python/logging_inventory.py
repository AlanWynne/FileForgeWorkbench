#!/usr/bin/env python3
"""Logging Inventory and Gap Report tool for FileForgeWorkbench.

Implements logging-subsystem Requirement 12 (CR-NR-055).

This is a READ-ONLY maintenance tool. It statically scans every ``*.rs`` file
under ``crates/`` and regenerates two things into a single Markdown report:

  1. A Logging Inventory  -- every ff-logging call site, grouped by crate, with
     file, 1-based line, level (where statically determinable) and the
     enclosing item.
  2. A Logging Gap report -- crates with non-test source but zero log calls,
     plus "silent error" candidate sites (let _ =, .ok(), unwrap(), expect()).

It writes exactly one artefact -- ``docs/quality/logging-inventory.md`` -- and
mirrors its stdout/stderr to ``tools/logs/logging-inventory.txt`` per the
project tooling standard. It touches no other file and never mutates source.

Usage:
    C:\\tools\\python\\python.exe tools\\python\\logging_inventory.py

The detection is line-oriented regex matching. Full Rust parsing is not
required because the logging call forms are textually regular. Silent-error
detection cannot prove the discarded expression is a Result, so those sites are
reported as REVIEW CANDIDATES, not confirmed defects.
"""

from __future__ import annotations

import datetime
import re
import sys
from pathlib import Path

TOOL_VERSION = "1.0.0"

# --- Path resolution (repo root is two levels up from tools/python) ----------
SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent.parent
CRATES_DIR = REPO_ROOT / "crates"
REPORT_PATH = REPO_ROOT / "docs" / "quality" / "logging-inventory.md"
LOG_PATH = REPO_ROOT / "tools" / "logs" / "logging-inventory.txt"

# --- stdout mirroring (tooling.md MANDATORY STDOUT CAPTURE) ------------------
_log_handle = None


def _open_log():
    global _log_handle
    LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
    # Overwrite the log at the start of each run so stale output never misleads.
    _log_handle = open(LOG_PATH, "w", encoding="utf-8")


def log(msg: str = "") -> None:
    """Print to stdout and mirror to the run log."""
    print(msg, flush=True)
    if _log_handle is not None:
        _log_handle.write(msg + "\n")
        _log_handle.flush()


# --- Detection patterns ------------------------------------------------------
# Log call sites (Req 12.1).
MACRO_RE = re.compile(r"ff_logging::log_(trace|debug|info|warn|error)\s*!")
LOG_FN_RE = re.compile(r"ff_logging::(log|log_lazy)\s*\(")
LOGLEVEL_ARG_RE = re.compile(r"LogLevel::(Trace|Debug|Info|Warn|Error)")
# PluginLogHandle method calls: <recv>.trace(/.debug(/.info(/.warn(/.error(
# Heuristic -- may over-match generic method names; flagged as such in report.
HANDLE_METHOD_RE = re.compile(r"\.(trace|debug|info|warn|error)\s*\(")

# Silent-error candidate sites (Req 12.4), non-test lines only.
LET_UNDERSCORE_RE = re.compile(r"\blet\s+_\s*=")
DOT_OK_RE = re.compile(r"\.ok\(\)\s*;?\s*$")
UNWRAP_RE = re.compile(r"\.unwrap\(\)")
EXPECT_RE = re.compile(r"\.expect\s*\(")

# Enclosing-item detection (best effort, for context only).
ITEM_RE = re.compile(
    r"^\s*(?:pub\s+)?(?:async\s+)?(?:unsafe\s+)?"
    r"(?:fn|struct|enum|trait|impl|mod)\s+([A-Za-z_][A-Za-z0-9_]*)"
)
IMPL_RE = re.compile(r"^\s*impl(?:\s*<[^>]*>)?\s+(.+?)\s*(?:\{|where)")

LEVEL_ORDER = ["trace", "debug", "info", "warn", "error", "dynamic"]


class Site:
    __slots__ = ("crate", "rel_path", "line", "kind", "level", "item")

    def __init__(self, crate, rel_path, line, kind, level, item):
        self.crate = crate
        self.rel_path = rel_path
        self.line = line
        self.kind = kind  # "macro" | "fn" | "handle"
        self.level = level  # trace/debug/info/warn/error/dynamic
        self.item = item


class SilentSite:
    __slots__ = ("crate", "rel_path", "line", "category", "text")

    def __init__(self, crate, rel_path, line, category, text):
        self.crate = crate
        self.rel_path = rel_path
        self.line = line
        self.category = category  # "let _ =" | ".ok()" | "unwrap()" | "expect()"
        self.text = text


def crate_name_for(path: Path) -> str:
    """Return the crate directory name for a file under crates/."""
    rel = path.relative_to(CRATES_DIR)
    return rel.parts[0]


def is_test_file(path: Path) -> bool:
    """A file anywhere under a tests/ directory is entirely test code."""
    parts = path.relative_to(REPO_ROOT).parts
    return "tests" in parts


def enclosing_item(lines, idx, cache):
    """Best-effort nearest enclosing fn/impl/struct name at or above line idx."""
    if idx in cache:
        return cache[idx]
    item = "-"
    for j in range(idx, -1, -1):
        m = ITEM_RE.match(lines[j])
        if m:
            item = m.group(1)
            break
        mi = IMPL_RE.match(lines[j])
        if mi:
            item = "impl " + mi.group(1)
            break
    cache[idx] = item
    return item


def scan_file(path: Path, crate: str, sites, silent, unreadable):
    """Scan a single .rs file, appending to sites/silent, recording read errors."""
    rel_path = path.relative_to(REPO_ROOT).as_posix()
    try:
        text = path.read_text(encoding="utf-8", errors="strict")
    except (OSError, UnicodeError) as exc:  # Req 12.8: record and continue
        unreadable.append((rel_path, str(exc)))
        return
    lines = text.splitlines()
    file_is_test = is_test_file(path)
    item_cache = {}

    # Track #[cfg(test)] module regions via brace depth so non-test detection
    # (Req 12.4) excludes unit-test modules.
    in_cfg_test = False
    cfg_test_depth = 0
    pending_cfg_test = False

    for i, raw in enumerate(lines):
        line_no = i + 1
        stripped = raw.strip()

        # cfg(test) region bookkeeping
        if re.search(r"#\[cfg\(test\)\]", raw):
            pending_cfg_test = True
        if pending_cfg_test and "{" in raw:
            in_cfg_test = True
            pending_cfg_test = False
            cfg_test_depth = raw.count("{") - raw.count("}")
            # The opening line itself is inside the test region from here on.
            continue
        if in_cfg_test:
            cfg_test_depth += raw.count("{") - raw.count("}")
            if cfg_test_depth <= 0:
                in_cfg_test = False
            continue

        is_test_ctx = file_is_test or in_cfg_test

        # --- Log call sites (counted regardless of test context, but the
        #     report separates test vs non-test via the crate gap logic). ---
        m = MACRO_RE.search(raw)
        if m:
            sites.append(
                Site(crate, rel_path, line_no, "macro", m.group(1),
                     enclosing_item(lines, i, item_cache))
            )
            continue

        mf = LOG_FN_RE.search(raw)
        if mf:
            lvl_m = LOGLEVEL_ARG_RE.search(raw)
            level = lvl_m.group(1).lower() if lvl_m else "dynamic"
            sites.append(
                Site(crate, rel_path, line_no, "fn", level,
                     enclosing_item(lines, i, item_cache))
            )
            continue

        # Handle-method calls: only when the receiver looks like a log handle,
        # to reduce over-matching. Accept `.info(`, `handle.info(`, `log.warn(`,
        # `self.log().info(`, `ctx.log().error(` etc.
        hm = HANDLE_METHOD_RE.search(raw)
        if hm and re.search(r"(log|handle|logger)\s*(\(\))?\s*\.(trace|debug|info|warn|error)\s*\(", raw):
            sites.append(
                Site(crate, rel_path, line_no, "handle", hm.group(1),
                     enclosing_item(lines, i, item_cache))
            )
            continue

        # --- Silent-error candidates (non-test only, Req 12.4) ---
        if is_test_ctx:
            continue
        if LET_UNDERSCORE_RE.search(raw):
            silent.append(SilentSite(crate, rel_path, line_no, "let _ =", stripped))
        if DOT_OK_RE.search(raw):
            silent.append(SilentSite(crate, rel_path, line_no, ".ok()", stripped))
        if UNWRAP_RE.search(raw):
            silent.append(SilentSite(crate, rel_path, line_no, "unwrap()", stripped))
        if EXPECT_RE.search(raw):
            silent.append(SilentSite(crate, rel_path, line_no, "expect()", stripped))


def scan_workspace():
    sites = []
    silent = []
    unreadable = []
    crate_has_nontest_src = {}
    files_scanned = 0

    crate_dirs = sorted(
        [d for d in CRATES_DIR.iterdir() if d.is_dir()],
        key=lambda p: p.name,
    )
    for crate_dir in crate_dirs:
        crate = crate_dir.name
        crate_has_nontest_src.setdefault(crate, False)
        rs_files = sorted(crate_dir.rglob("*.rs"), key=lambda p: p.as_posix())
        for path in rs_files:
            files_scanned += 1
            if not is_test_file(path):
                crate_has_nontest_src[crate] = True
            scan_file(path, crate, sites, silent, unreadable)
        log(f"  scanned crate {crate}: {len(rs_files)} .rs files")

    # Deterministic ordering (Req 12.9)
    sites.sort(key=lambda s: (s.crate, s.rel_path, s.line))
    silent.sort(key=lambda s: (s.crate, s.rel_path, s.line, s.category))
    unreadable.sort()
    return sites, silent, unreadable, crate_has_nontest_src, files_scanned


def build_report(sites, silent, unreadable, crate_has_nontest_src, files_scanned):
    now = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    all_crates = sorted(crate_has_nontest_src.keys())

    # Per-crate level counts
    counts = {c: {lvl: 0 for lvl in LEVEL_ORDER} for c in all_crates}
    for s in sites:
        counts.setdefault(s.crate, {lvl: 0 for lvl in LEVEL_ORDER})
        counts[s.crate][s.level] = counts[s.crate].get(s.level, 0) + 1

    gaps = [
        c for c in all_crates
        if crate_has_nontest_src[c] and sum(counts[c].values()) == 0
    ]

    out = []
    a = out.append
    a("# Logging Inventory and Gap Report")
    a("")
    a("> Generated by `tools/python/logging_inventory.py` "
      "(logging-subsystem Requirement 12, CR-NR-055).")
    a("> This file is regenerated on demand; do not edit by hand.")
    a("")
    a(f"- Generated: {now}")
    a(f"- Tool version: {TOOL_VERSION}")
    a(f"- Workspace root: `{REPO_ROOT.as_posix()}`")
    a(f"- Crates scanned: {len(all_crates)}")
    a(f"- Rust files scanned: {files_scanned}")
    a(f"- Total log call sites: {len(sites)}")
    a(f"- Crates with zero log calls: {len(gaps)}")
    a(f"- Silent-error review candidates: {len(silent)}")
    a("")

    # --- Summary table (Req 12.2) ---
    a("## 1. Per-crate log call summary")
    a("")
    a("Counts of log call sites by level. `dynamic` = `log()`/`log_lazy()` "
      "call whose level was not a literal `LogLevel::X`.")
    a("")
    a("| Crate | trace | debug | info | warn | error | dynamic | total | gap |")
    a("|-------|-------|-------|------|------|-------|---------|-------|-----|")
    for c in all_crates:
        cc = counts[c]
        total = sum(cc.values())
        gap = "YES" if (c in gaps) else ""
        a(f"| `{c}` | {cc['trace']} | {cc['debug']} | {cc['info']} | "
          f"{cc['warn']} | {cc['error']} | {cc['dynamic']} | {total} | {gap} |")
    a("")

    # --- Gaps (Req 12.3) ---
    a("## 2. Logging gaps (crates with non-test source but zero log calls)")
    a("")
    if gaps:
        for c in gaps:
            a(f"- `{c}`")
    else:
        a("None. Every crate with non-test source emits at least one log call.")
    a("")

    # --- Silent-error candidates (Req 12.4) ---
    a("## 3. Silent-error review candidates (non-test code)")
    a("")
    a("Discarded results / panics that emit no log record. These are "
      "REVIEW CANDIDATES -- a text scan cannot prove the discarded value is a "
      "`Result`, so confirm before acting.")
    a("")
    if silent:
        cat_totals = {}
        for s in silent:
            cat_totals[s.category] = cat_totals.get(s.category, 0) + 1
        a("Totals by category: "
          + ", ".join(f"`{k}` = {cat_totals[k]}" for k in sorted(cat_totals)))
        a("")
        a("| Crate | File:Line | Category | Source |")
        a("|-------|-----------|----------|--------|")
        for s in silent:
            snippet = s.text.replace("|", "\\|")
            if len(snippet) > 100:
                snippet = snippet[:97] + "..."
            a(f"| `{s.crate}` | `{s.rel_path}:{s.line}` | `{s.category}` "
              f"| `{snippet}` |")
    else:
        a("None detected.")
    a("")

    # --- Unreadable files (Req 12.8) ---
    a("## 4. Unreadable or unparseable files")
    a("")
    if unreadable:
        a("| File | Error |")
        a("|------|-------|")
        for rel_path, err in unreadable:
            a(f"| `{rel_path}` | {err} |")
    else:
        a("None. All scanned files were read successfully.")
    a("")

    # --- Full inventory (Req 12.1) ---
    a("## 5. Full log call inventory")
    a("")
    a("Kinds: `macro` = `log_*!`; `fn` = `log()`/`log_lazy()`; "
      "`handle` = `PluginLogHandle` method (heuristic, may over-match).")
    a("")
    if sites:
        a("| Crate | File:Line | Level | Kind | Enclosing item |")
        a("|-------|-----------|-------|------|----------------|")
        for s in sites:
            item = s.item.replace("|", "\\|")
            a(f"| `{s.crate}` | `{s.rel_path}:{s.line}` | {s.level} | "
              f"{s.kind} | `{item}` |")
    else:
        a("No log call sites found.")
    a("")

    return "\n".join(out) + "\n"


def main():
    _open_log()
    log("=== Logging Inventory and Gap Report tool ===")
    log(f"Tool version: {TOOL_VERSION}")
    log(f"Workspace root: {REPO_ROOT}")

    if not CRATES_DIR.is_dir():
        log(f"ERROR: crates directory not found at {CRATES_DIR}")
        return 1

    log("Scanning crates...")
    sites, silent, unreadable, crate_has_nontest_src, files_scanned = scan_workspace()
    log(f"Scan complete: {files_scanned} files, {len(sites)} log call sites, "
        f"{len(silent)} silent-error candidates, {len(unreadable)} unreadable.")

    report = build_report(
        sites, silent, unreadable, crate_has_nontest_src, files_scanned
    )

    REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    REPORT_PATH.write_text(report, encoding="utf-8")  # Req 12.5: overwrite
    log(f"Report written to {REPORT_PATH.relative_to(REPO_ROOT).as_posix()} "
        f"({len(report)} bytes)")

    gaps = sum(
        1 for c in crate_has_nontest_src
        if crate_has_nontest_src[c]
        and not any(s.crate == c for s in sites)
    )
    log(f"Summary: {len(crate_has_nontest_src)} crates, {gaps} with zero log calls.")
    log("Done.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    finally:
        if _log_handle is not None:
            _log_handle.close()
