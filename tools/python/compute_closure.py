#!/usr/bin/env python3
"""Compute the authoritative build closure of the ff-desktop binary.

Reads cargo metadata JSON and walks the resolved dependency graph from the
ff-desktop root node, following normal and build dependencies (NOT dev-only),
restricted to WORKSPACE MEMBER crates (path deps under crates/). Emits the
in-closure set and the parked set (workspace members not reached).

Usage:
    python compute_closure.py <metadata.json> <log.txt>

Outputs are written to the log file (and echoed to stdout).
"""
import json
import sys

LOG = sys.argv[2] if len(sys.argv) > 2 else "closure-compute.txt"
_log_fh = open(LOG, "w", encoding="utf-8")


def log(msg):
    print(msg, flush=True)
    _log_fh.write(str(msg) + "\n")


def main():
    meta_path = sys.argv[1]
    with open(meta_path, "r", encoding="utf-8") as f:
        meta = json.load(f)

    # --- Identify workspace members (ids) ---
    workspace_member_ids = set(meta["workspace_members"])

    # Map package id -> package record for every package.
    pkg_by_id = {p["id"]: p for p in meta["packages"]}

    # Workspace member id -> name, restricted to path deps under crates/.
    member_name_by_id = {}
    for pid in workspace_member_ids:
        p = pkg_by_id[pid]
        manifest = p.get("manifest_path", "").replace("\\", "/")
        if "/crates/" in manifest:
            member_name_by_id[pid] = p["name"]
    member_ids = set(member_name_by_id.keys())
    all_member_names = set(member_name_by_id.values())

    log("Total workspace members (path deps under crates/): %d" % len(member_ids))

    # --- Build resolved graph from resolve.nodes ---
    # Each node has "id" and "deps": list of {name, pkg, dep_kinds:[{kind, target}]}
    # dep_kinds kind is null for normal, "build", or "dev".
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}

    # Find the ff-desktop root id.
    root_ids = [pid for pid in member_ids if member_name_by_id[pid] == "ff-desktop"]
    if not root_ids:
        log("ERROR: ff-desktop not found among workspace members")
        return 2
    root = root_ids[0]
    log("ff-desktop root id: %s" % root)

    # Walk closure following normal + build dep_kinds only (exclude dev).
    def edge_kinds(dep):
        kinds = set()
        for dk in dep.get("dep_kinds", []):
            k = dk.get("kind")
            kinds.add("normal" if k is None else k)
        return kinds

    visited = set()
    stack = [root]
    # Track whether a reached member was pulled in via a non-dev edge somewhere.
    while stack:
        cur = stack.pop()
        if cur in visited:
            continue
        visited.add(cur)
        node = nodes.get(cur)
        if node is None:
            continue
        for dep in node.get("deps", []):
            kinds = edge_kinds(dep)
            # Follow only normal/build edges; skip pure-dev edges.
            if kinds <= {"dev"}:
                continue
            stack.append(dep["pkg"])

    # Closure restricted to workspace members.
    closure_member_ids = visited & member_ids
    closure_names = sorted(member_name_by_id[i] for i in closure_member_ids)
    parked_names = sorted(all_member_names - set(closure_names))

    # --- Ambiguity analysis -------------------------------------------------
    # A member reachable ONLY via dev edges from the root closure would be
    # dev-only. Compute full reachability INCLUDING dev to detect those.
    visited_all = set()
    stack = [root]
    while stack:
        cur = stack.pop()
        if cur in visited_all:
            continue
        visited_all.add(cur)
        node = nodes.get(cur)
        if node is None:
            continue
        for dep in node.get("deps", []):
            stack.append(dep["pkg"])
    dev_only_members = sorted(
        member_name_by_id[i]
        for i in (visited_all & member_ids) - closure_member_ids
    )

    # Optional-feature-gated detection: a member listed as a dependency in a
    # closure package manifest but marked optional (behind a feature).
    optional_member_deps = {}
    closure_names_set = set(closure_names)
    for pid in closure_member_ids:
        p = pkg_by_id[pid]
        for d in p.get("dependencies", []):
            if d.get("optional") and d.get("name") in all_member_names:
                optional_member_deps.setdefault(d["name"], []).append(p["name"])

    # --- Emit ---------------------------------------------------------------
    log("")
    log("=== IN-CLOSURE WORKSPACE MEMBERS (normal+build from ff-desktop) ===")
    log("Count: %d" % len(closure_names))
    for n in closure_names:
        log("  " + n)

    log("")
    log("=== PARKED WORKSPACE MEMBERS (not in ffwb build closure) ===")
    log("Count: %d" % len(parked_names))
    for n in parked_names:
        log("  " + n)

    log("")
    log("=== AMBIGUITY NOTES ===")
    log("Members reachable ONLY via dev-dependency edges from the closure: %d"
        % len(dev_only_members))
    for n in dev_only_members:
        log("  dev-only: " + n)
    log("Optional (feature-gated) member deps inside the closure: %d"
        % len(optional_member_deps))
    for n, parents in sorted(optional_member_deps.items()):
        log("  optional: %s (optional dep of: %s)" % (n, ", ".join(sorted(set(parents)))))

    log("")
    log("Sanity: closure(%d) + parked(%d) = %d ; total members = %d"
        % (len(closure_names), len(parked_names),
           len(closure_names) + len(parked_names), len(member_ids)))
    return 0


if __name__ == "__main__":
    rc = main()
    _log_fh.close()
    sys.exit(rc)
