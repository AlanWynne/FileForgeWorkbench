#!/usr/bin/env python3
"""Reliable cargo gate runner (CR-NR-097 helper).

The interactive PowerShell session in this environment intermittently stops
flushing stdout, which makes it impossible to observe `cargo`/`ffwb-gate`
results. This driver runs the gate steps with ``subprocess.run`` (which captures
child output directly, independent of any shell), writes each step's full output
to ``tools/logs/`` and a compact summary to ``tools/logs/run_checks-summary.txt``,
and writes a ``tools/logs/run_checks-DONE.txt`` marker on completion.

Usage (from the repo root):
    C:\\tools\\python\\python.exe tools\\python\\run_checks.py [--full] [PACKAGES...]

- Default: fmt --check, clippy (-D warnings) + test for the given packages
  (default: ff-help ff-desktop).
- --full: run the whole workspace (fmt --check, clippy --workspace, test
  --workspace) -- the completion gate.

Overwrite behaviour: all log files listed below are overwritten each run.
Outputs (all under tools/logs/):
  run_checks-fmt.txt, run_checks-clippy.txt, run_checks-test.txt,
  run_checks-summary.txt, run_checks-DONE.txt
"""

import subprocess
import sys
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
LOG_DIR = REPO_ROOT / "tools" / "logs"


def log_path(name: str) -> Path:
    return LOG_DIR / f"run_checks-{name}.txt"


def run_step(name: str, args: list[str]) -> tuple[int, float]:
    """Run one cargo step, capture combined output to a log, return (exit, secs)."""
    start = time.monotonic()
    proc = subprocess.run(
        args,
        cwd=str(REPO_ROOT),
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    elapsed = time.monotonic() - start
    log_path(name).write_text(proc.stdout or "", encoding="utf-8")
    return proc.returncode, elapsed


def tail(name: str, n: int) -> str:
    text = log_path(name).read_text(encoding="utf-8", errors="replace")
    lines = text.splitlines()
    return "\n".join(lines[-n:])


def kill_stale_cargo() -> str:
    """Kill any lingering cargo/rustc/clippy processes that may hold the target
    build lock (left behind by terminated terminals), so `cargo fmt` does not
    block on `.cargo-lock`. Uses taskkill; ignores 'not found' results. Does NOT
    kill this Python process."""
    notes = []
    for image in ("cargo.exe", "rustc.exe", "cargo-clippy.exe", "rustfmt.exe", "cargo-nextest.exe"):
        try:
            r = subprocess.run(
                ["taskkill", "/F", "/IM", image],
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                encoding="utf-8",
                errors="replace",
            )
            notes.append(f"{image}: rc={r.returncode}")
        except Exception as exc:  # noqa: BLE001 - best effort cleanup
            notes.append(f"{image}: {exc}")
    # Remove a stale build lock if present.
    lock = REPO_ROOT / "target" / "debug" / ".cargo-lock"
    if lock.exists():
        try:
            lock.unlink()
            notes.append("removed stale target/debug/.cargo-lock")
        except OSError as exc:
            notes.append(f"could not remove lock: {exc}")
    return "; ".join(notes)


def main() -> int:
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    # Clear the completion marker up front so a stale one never misleads.
    done = LOG_DIR / "run_checks-DONE.txt"
    if done.exists():
        done.unlink()

    if "--kill-stale" in sys.argv:
        note = kill_stale_cargo()
        (LOG_DIR / "run_checks-killstale.txt").write_text(note + "\n", encoding="utf-8")
        time.sleep(2)

    if "--clippy-fix" in sys.argv:
        # Apply clippy's machine-applicable suggestions across the whole
        # workspace (CR-CH-048 mechanical lint sweep). `--allow-dirty` because we
        # have uncommitted work; `--keep-going` to fix as many crates as possible
        # in one pass. Non-machine-applicable lints are left for manual fixing.
        fix_exit, fix_secs = run_step(
            "clippyfix",
            [
                "cargo",
                "clippy",
                "--fix",
                "--workspace",
                "--all-targets",
                "--allow-dirty",
                "--allow-no-vcs",
                "--keep-going",
            ],
        )
        (LOG_DIR / "run_checks-clippyfix-summary.txt").write_text(
            f"clippy --fix: exit={fix_exit} ({fix_secs:.1f}s)\n", encoding="utf-8"
        )
        done.write_text("FIX-DONE\n", encoding="utf-8")
        print(f"clippy --fix done: exit={fix_exit} ({fix_secs:.1f}s)")
        return 0

    full = "--full" in sys.argv
    pkgs = [a for a in sys.argv[1:] if not a.startswith("--")]
    if not pkgs:
        pkgs = ["ff-help", "ff-desktop"]

    def pkg_args(base: list[str]) -> list[str]:
        if full:
            return base + ["--workspace"]
        out = list(base)
        for p in pkgs:
            out += ["-p", p]
        return out

    summary: list[str] = []
    summary.append(f"scope={'FULL workspace' if full else 'packages: ' + ' '.join(pkgs)}")

    # 1. fmt --check (always whole workspace; formatting is global)
    fmt_exit, fmt_secs = run_step("fmt", ["cargo", "fmt", "--all", "--", "--check"])
    summary.append(f"fmt --check: exit={fmt_exit} ({fmt_secs:.1f}s)")

    # 2. clippy -D warnings. `--keep-going` so one pass reports lints from every
    # independently-checkable crate at once (clippy otherwise aborts the build
    # early), minimising fix/re-run round-trips.
    clippy_exit, clippy_secs = run_step(
        "clippy",
        pkg_args(["cargo", "clippy", "--all-targets", "--keep-going"]) + ["--", "-D", "warnings"],
    )
    summary.append(f"clippy -D warnings: exit={clippy_exit} ({clippy_secs:.1f}s)")

    if "--clippy-only" in sys.argv:
        clean = fmt_exit == 0 and clippy_exit == 0
        summary.append("")
        summary.append("VERDICT: " + ("CLEAN" if clean else "ISSUES") + " (clippy-only)")
        if clippy_exit != 0:
            summary.append("")
            summary.append("--- clippy tail ---")
            summary.append(tail("clippy", 200))
        summary_text = "\n".join(summary) + "\n"
        log_path("summary").write_text(summary_text, encoding="utf-8")
        done.write_text("CLEAN\n" if clean else "ISSUES\n", encoding="utf-8")
        print(summary_text)
        return 0 if clean else 1

    # 3. test -- prefer cargo-nextest (process-per-test isolation) to match the
    # project gate; several ff-desktop tests rely on env-var isolation that only
    # holds under nextest, NOT under serial `cargo test` (steering B048). Fall
    # back to `cargo test` if nextest is not installed.
    have_nextest = (
        subprocess.run(
            ["cargo", "nextest", "--version"],
            cwd=str(REPO_ROOT),
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        ).returncode
        == 0
    )
    if have_nextest:
        test_exit, test_secs = run_step("test", pkg_args(["cargo", "nextest", "run"]))
        runner = "nextest"
    else:
        test_exit, test_secs = run_step("test", pkg_args(["cargo", "test"]))
        runner = "cargo-test"
    summary.append(f"test ({runner}): exit={test_exit} ({test_secs:.1f}s)")

    # Extract the "test result:" lines for a quick pass/fail view.
    test_results = [
        ln for ln in log_path("test").read_text(encoding="utf-8", errors="replace").splitlines()
        if "test result:" in ln
    ]

    clean = fmt_exit == 0 and clippy_exit == 0 and test_exit == 0
    summary.append("")
    summary.append("VERDICT: " + ("CLEAN" if clean else "ISSUES"))
    summary.append("")
    summary.append("--- test result lines ---")
    summary.extend(test_results)
    if fmt_exit != 0:
        summary.append("")
        summary.append("--- fmt tail ---")
        summary.append(tail("fmt", 30))
    if clippy_exit != 0:
        summary.append("")
        summary.append("--- clippy tail ---")
        summary.append(tail("clippy", 40))
    if test_exit != 0:
        summary.append("")
        summary.append("--- test tail ---")
        summary.append(tail("test", 60))

    summary_text = "\n".join(summary) + "\n"
    log_path("summary").write_text(summary_text, encoding="utf-8")
    done.write_text("CLEAN\n" if clean else "ISSUES\n", encoding="utf-8")
    print(summary_text)
    return 0 if clean else 1


if __name__ == "__main__":
    sys.exit(main())
