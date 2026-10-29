#!/usr/bin/env python3
"""The director's gate for a rum brief: one command, one table.

  director-gate.py run NNN [--stage S] [--steps a,b | --from STEP] [--continue] [--perf resume,cli]
  director-gate.py show NNN [--stage S] STEP
  director-gate.py rule NNN [--stage S] STEP --why TEXT
  director-gate.py pair BASE_TSV CAND_TSV [--rev REV]
  director-gate.py base-frontier [--rev REV]
  director-gate.py selftest

`run` replays the standing inventories and runs the gate, in this order. Logs
go to .temp/director/gate-NNN<stage>/logs/, every step's result to gate.json
there, and one table with a verdict per step goes to stdout. `show` prints a
step's notes (the rows behind its verdict).

  scope         report.py check --against-git, before any replay edits the tree
  fmt           cargo fmt --check
  check         cargo check --workspace --all-targets; warnings new against HEAD
  ownership     production_inventory_is_exact, replayed for up to three rounds
  census        the rejection census; its frontier paired with the base's
  oracle        exact_types_and_constructor_claims_match_rustc
  standing      LP01 (suspension_oracle logical_counter_port), typed_slot_census
  fingerprints  regenerate_local_contract_fingerprints, copied in
  certificates  local_production_acceptance in both feature builds, merged
  inventories   the inventory tests in one run, on the final tree
  workspace     cargo fmt --check, then cargo test --workspace --no-fail-fast
  arms          both arms from one source path, one codegen unit (needs --perf)
  structure     function diff of the two arms (needs --perf)
  panels        one interleaved perf panel per engine named in --perf

The steps up to `standing` can need a ruling (a moved count, a new unknown
row, a new warning). A FLAGGED or FAIL there stops the run before the replays
that hash the tree. Rule, edit, then rerun with --from. --continue runs on.

Verdicts: PASS; FLAGGED (read the notes and rule); RULED (a FLAGGED step a
human has accepted); FAIL; SKIP. A filtered test run that runs no test is a
FAIL: a check that ran nothing checked nothing.

Base artifacts are keyed by the git tree of crates/. A green run stores its
final frontier and its warnings under .temp/director/gate-state/<tree>/, so
the next gate, whose HEAD is that tree, pairs against them. When the key is
missing, `census` dumps HEAD's frontier in a worktree (base-frontier).

Every cargo build or test runs in scripts/memory-scope.sh at 14G, with
CARGO_BUILD_JOBS=8, RUST_TEST_THREADS=1 and --offline. Steps run serially.
With --perf, three steps follow workspace: arms (both arms from one source
path, one codegen unit), structure (the function diff) and panels (one perf
panel per named engine). Without --perf they are SKIP. `rule` turns a
FLAGGED step into RULED; GREEN counts RULED as passing.
"""

import argparse
import collections
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCOPE = ROOT / "scripts/memory-scope.sh"
STATE = ROOT / ".temp/director/gate-state"
PERF_WORK = ROOT / ".temp/director/perf"
PERF_PANEL = ROOT / "scripts/perf-panel.py"
OWNERSHIP = Path("crates/rum-stdlib/src/tests/ownership_graph/inventory")
CONTRACTS = Path("crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv")
PROMOTED = Path("crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv")
REGENERATED = Path("target/brief-544/local-contracts.tsv")
CERTIFICATES = Path(".temp/brief-546/certificates")
TEST_RESULT = re.compile(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored")
SCOPE_LINE = re.compile(r"# memory-scope: unit=\S+ max=\S* peak=(\d*) oom=\S+ oom_kill=(\S+) rc=")
STEPS = ["scope", "fmt", "check", "ownership", "census", "oracle", "standing",
         "fingerprints", "certificates", "inventories", "workspace",
         "arms", "structure", "panels"]
PASSING = ("PASS", "SKIP", "RULED")
RULING = STEPS[: STEPS.index("standing") + 1]
INVENTORY_TESTS = ["production_rejection_census_is_complete", "production_inventory_is_exact",
                   "host_payload_ledger", "local_production_acceptance",
                   "exact_types_and_constructor_claims_match_rustc"]


# ---- running -------------------------------------------------------------------

def git(*args, env=None, cwd=ROOT, check=True):
    out = subprocess.run(["git", *args], cwd=cwd, env=env, capture_output=True, text=True)
    if check and out.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)}: {out.stderr.strip()}")
    return out.stdout


class Gate:
    def __init__(self, brief, stage):
        self.brief, self.stage = brief, stage
        self.dir = ROOT / f".temp/director/gate-{brief}{(stage or '').lower()}"
        self.logs = self.dir / "logs"
        self.logs.mkdir(parents=True, exist_ok=True)
        self.json = self.dir / "gate.json"
        self.state = json.loads(self.json.read_text()) if self.json.exists() else {"steps": {}}
        self.state.update(brief=brief, stage=stage)

    def log(self, name):
        path = self.logs / name
        if path.exists():
            old = self.logs / "prev"
            old.mkdir(exist_ok=True)
            path.rename(old / f"{path.stem}.{int(path.stat().st_mtime)}{path.suffix}")
        return path

    def cargo(self, name, args, env=(), cwd=ROOT):
        """Run cargo in its own memory scope; returns (rc, log text, scope facts)."""
        path = self.log(f"{name}.log")
        unit = f"gate{self.brief}{(self.stage or '').lower()}-{name}-{os.getpid()}"
        cmd = [str(SCOPE), unit, "14G", "env", "CARGO_BUILD_JOBS=8", "RUST_TEST_THREADS=1",
               *env, "cargo", *args]
        with path.open("w") as sink:
            sink.write(f"$ {' '.join(cmd)}\n")
            sink.flush()
            rc = subprocess.run(cmd, cwd=cwd, stdout=sink, stderr=subprocess.STDOUT,
                                stdin=subprocess.DEVNULL).returncode
        text = path.read_text(errors="replace")
        return rc, text, scope_facts(text), path

    def save(self, step, result):
        self.state["steps"][step] = result
        self.json.write_text(json.dumps(self.state, indent=2, ensure_ascii=False) + "\n")


def scope_facts(text):
    m = None
    for m in SCOPE_LINE.finditer(text):
        pass
    if m is None:
        return {"peak_gb": None, "oom_kill": None}
    peak = int(m.group(1)) / 1e9 if m.group(1) else None
    return {"peak_gb": peak, "oom_kill": m.group(2)}


def test_totals(text):
    rows = [tuple(map(int, m.groups())) for m in TEST_RESULT.finditer(text)]
    return {"binaries": len(rows), "passed": sum(r[0] for r in rows),
            "failed": sum(r[1] for r in rows), "ignored": sum(r[2] for r in rows)}


def failures(text):
    names = re.findall(r"^test (\S+) \.\.\. FAILED$", text, re.M)
    names += [m.strip() for m in re.findall(r"^(.*signal: \d+.*)$", text, re.M)]
    return names


def result(verdict, detail, notes=(), logs=()):
    return {"verdict": verdict, "detail": detail, "notes": list(notes),
            "logs": [str(Path(p).relative_to(ROOT)) for p in logs]}


def filtered_run(gate, name, args, want=1, env=()):
    """A filtered test run: PASS only if it ran at least `want` tests and none failed."""
    rc, text, facts, path = gate.cargo(name, args, env)
    t = test_totals(text)
    ok = rc == 0 and t["failed"] == 0 and t["passed"] >= want
    why = "" if ok else (f"rc {rc}" if t["passed"] >= want else f"ran {t['passed']} tests, wanted {want}")
    return ok, t, text, path, why


# ---- diffs and the census pairing ----------------------------------------------

def parse_hunks(diff_text):
    hunks = []
    for line in diff_text.splitlines():
        m = re.match(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@", line)
        if m:
            hunks.append((int(m[1]), int(m[2] or 1), int(m[3]), int(m[4] or 1)))
    return hunks


def tree_hunks(rev):
    """hunks(file): the -U0 hunks from `rev` to the working tree; None when the
    file is missing on one side (every row there is new or retired)."""
    cache = {}

    def hunks(rel):
        if rel not in cache:
            at_rev = subprocess.run(["git", "cat-file", "-e", f"{rev}:{rel}"], cwd=ROOT,
                                    capture_output=True).returncode == 0
            if not at_rev or not (ROOT / rel).exists():
                cache[rel] = None
            else:
                cache[rel] = parse_hunks(git("diff", "-U0", "--no-color", "--no-ext-diff",
                                             "--no-renames", rev, "--", rel))
        return cache[rel]
    return hunks


def old_to_new(hunks, old):
    """The new line of an unchanged old line; None when the diff removed or changed it."""
    off = 0
    for os_, oc, ns, nc in hunks:
        if oc == 0:  # a pure insertion after line os_
            if old > os_:
                off += nc
            continue
        if old < os_:
            break
        if old < os_ + oc:
            return None
        off += nc - oc
    return old + off


def new_changed(hunks, new):
    return any(nc and ns <= new < ns + nc for _, _, ns, nc in hunks)


def load_frontier(path):
    rows = []
    for line in Path(path).read_text().splitlines():
        if line.startswith("#") or line.startswith("kind\t") or not line:
            continue
        r = line.split("\t")
        rows.append((r[0], r[1], int(r[2]), int(r[3]), int(r[4]), int(r[5]), r[6], r[7]))
    return rows


def frontier_counts(path):
    m = re.search(r"^# counts (.*) len (\d+)$", Path(path).read_text(), re.M)
    return f"{m.group(2)} {m.group(1)}" if m else "?"


def pair(base, cand, hunks):
    """Pair base rows with candidate rows of the same kind, columns, callee and
    reason at the shifted lines, as multisets. Returns (counts, rows): rows are
    (class, row, note) for every row that did not pair."""
    pool = collections.Counter(cand)
    rows, counts = [], collections.Counter()
    for r in base:
        h = hunks(r[1])
        ns, ne = (None, None) if h is None else (old_to_new(h, r[2]), old_to_new(h, r[4]))
        if ns is None or ne is None:
            counts["retired"] += 1
            rows.append(("RETIRED", r, "file missing on one side" if h is None else "line changed"))
            continue
        key = (r[0], r[1], ns, r[3], ne, r[5], r[6], r[7])
        if pool[key] > 0:
            pool[key] -= 1
            counts["paired"] += 1
        else:
            counts["base_only"] += 1
            rows.append(("BASE-ONLY", r, f"expected at {ns}-{ne}"))
    for r, n in pool.items():
        h = hunks(r[1])
        cls = "NEW" if h is None or new_changed(h, r[2]) or new_changed(h, r[4]) else "CAND-ONLY"
        counts["new" if cls == "NEW" else "cand_only"] += n
        rows += [(cls, r, "line added or changed" if cls == "NEW" else "unchanged line")] * n
    counts["base"], counts["cand"] = len(base), len(cand)
    order = {"BASE-ONLY": 0, "CAND-ONLY": 1, "RETIRED": 2, "NEW": 3}
    rows.sort(key=lambda x: (order[x[0]], x[1][1], x[1][2], x[1][3]))
    return counts, rows


def pair_line(c):
    return (f"paired {c['paired']}, retired {c['retired']}, new {c['new']}, "
            f"base-only {c['base_only']}, cand-only {c['cand_only']}")


def pair_groups(rows):
    groups = collections.Counter((cls, r[0], r[1]) for cls, r, _ in rows)
    return [f"{n:4d} {cls:9s} {kind} {file}" for (cls, kind, file), n in sorted(groups.items())]


def write_pairs(path, counts, rows):
    with Path(path).open("w") as f:
        for cls, r, note in rows:
            f.write(f"{cls}\t{r[0]}\t{r[1]}:{r[2]}:{r[3]}-{r[4]}:{r[5]}\t{r[6]}\t{r[7]}\t{note}\n")
        f.write(f"CENSUSPAIR base={counts['base']} cand={counts['cand']} {pair_line(counts)}\n")


# ---- trees and base artifacts --------------------------------------------------

def crates_tree_of_worktree():
    """The git tree crates/ would have if the working tree were committed now."""
    with tempfile.TemporaryDirectory(dir=ROOT / ".temp") as d:
        env = dict(os.environ, GIT_INDEX_FILE=str(Path(d) / "index"))
        git("read-tree", "HEAD", env=env)
        git("add", "-A", "--", "crates", env=env)
        return git("write-tree", "--prefix=crates/", env=env).strip()


def crates_tree_of(rev):
    return git("rev-parse", f"{rev}:crates").strip()


def head_worktree(rev):
    """The worktree base_frontier uses, checked out at `rev` (factored out of it).

    `rev` is resolved in the main tree first: inside the worktree, HEAD names
    the worktree's own checkout, so the last gate's commit would stay put."""
    commit = git("rev-parse", "--verify", f"{rev}^{{commit}}").strip()
    wt = STATE / "worktree"
    if wt.exists():
        git("checkout", "--detach", "--force", commit, cwd=wt)
    else:
        STATE.mkdir(parents=True, exist_ok=True)
        git("worktree", "add", "--detach", str(wt), commit)
    if git("rev-parse", "HEAD", cwd=wt).strip() != commit:
        raise RuntimeError(f"worktree {wt} is not at {rev} ({commit[:12]})")
    return wt


def base_frontier(rev, gate=None):
    """Dump the census frontier of `rev` in a worktree; store it by its tree."""
    tree = crates_tree_of(rev)
    out = STATE / tree / "frontier.tsv"
    if out.exists():
        return out
    wt = head_worktree(rev)
    if "RUM_CENSUS_DUMP" not in (wt / "crates/rum-stdlib/src/tests/retention_closure/census/tests.rs").read_text():
        raise RuntimeError(f"{rev} predates RUM_CENSUS_DUMP: seed {out} by hand")
    out.parent.mkdir(parents=True, exist_ok=True)
    gate = gate or Gate("base", None)
    rc, text, _, path = gate.cargo(
        "base-frontier", ["test", "-p", "rum-stdlib", "--offline", "--lib",
                          "production_rejection_census_is_complete"],
        env=[f"CARGO_TARGET_DIR={STATE / 'target'}", f"RUM_CENSUS_DUMP={out}"], cwd=wt)
    if not out.exists():
        raise RuntimeError(f"no frontier written at {rev} (rc {rc}); see {path}")
    return out


# ---- the ownership replay ------------------------------------------------------

def own_key(row):
    c = row.split("\t")
    return (c[0], c[1], re.sub(r"\d+", "#", "\t".join(c[4:] if c[0].startswith("Unknown") else c[3:])))


def own_target(row):
    if row.startswith("UnknownHostPayload"):
        return "host_payloads.tsv"
    if not row.startswith("UnknownConstruct"):
        return "self_reaching.tsv"
    c = row.split("\t")[4]
    for prefix, f in [("unresolved attribute", "unknown_attributes.tsv"),
                      ("closure captures", "unknown_captures.tsv"),
                      ("unexpanded macro", "unknown_macros.tsv")]:
        if c.startswith(prefix):
            return f
    return "unknown_storage.tsv"


def own_sortkey(line):
    c = line.split("\t")
    if c[0].startswith("Unknown"):
        return (c[1], int(c[2]), int(c[3]), line)
    return (c[1], int(c[2]), line)


def own_replay(log_text, inv):
    """Replay a failing exact-inventory run into the inventory files at `inv`.

    A stale row pairs with a missing row of the same kind, file and construct
    (digits ignored) and moves in place; else with one of the same kind and
    construct in another file (MOVE-FILE). Unpaired missing rows are added to
    their construct's file (NEW), unpaired stale rows removed (REMOVED). The
    ledger follows a moved host payload. Returns the actions as (kind, file, row).
    """
    stale = re.findall(r"^stale inventory row: (.*)$", log_text, re.M)
    missing = re.findall(r"^missing inventory row: (.*)$", log_text, re.M)
    files = {f.name: f.read_text().split("\n") for f in Path(inv).iterdir() if f.suffix == ".tsv"}

    def locate(row):
        hits = [(f, i) for f, ls in files.items() for i, l in enumerate(ls)
                if f != "host_payload_ledger.tsv" and l == row]
        if len(hits) != 1:
            raise RuntimeError(f"stale row found {len(hits)} times: {row}")
        return hits[0]

    def move(s, m):
        f, i = locate(s)
        files[f][i] = m
        if s.startswith("UnknownHostPayload"):
            led = files["host_payload_ledger.tsv"]
            hit = [j for j, l in enumerate(led) if l.startswith(s + "\t")]
            if len(hit) != 1:
                raise RuntimeError(f"ledger row found {len(hit)} times: {s}")
            led[hit[0]] = m + led[hit[0]][len(s):]
        return f

    actions, left = [], list(missing)
    unpaired = []
    for s in stale:
        m = next((m for m in left if own_key(m) == own_key(s)), None)
        if m is None:
            unpaired.append(s)
            continue
        left.remove(m)
        actions.append(("MOVE", move(s, m), m))
    for s in unpaired:
        hits = [m for m in left if (own_key(m)[0], own_key(m)[2]) == (own_key(s)[0], own_key(s)[2])]
        if len(hits) == 1:
            left.remove(hits[0])
            actions.append(("MOVE-FILE", move(s, hits[0]), f"{s}  ->  {hits[0]}"))
        else:
            f, i = locate(s)
            del files[f][i]
            actions.append(("REMOVED", f, s))
    for m in left:
        files[own_target(m)].append(m)
        actions.append(("NEW", own_target(m), m))
    for f, ls in files.items():
        body = [l for l in ls if l]
        if f == "host_payload_ledger.tsv":
            body = body[:1] + sorted(body[1:], key=own_sortkey)
        elif f != "self_reaching.tsv":
            body = sorted(body, key=own_sortkey)
        (Path(inv) / f).write_text("\n".join(body) + "\n")
    return actions


def own_flags(actions, hunks):
    """The actions that need a ruling: a row moved across files, a row gone,
    a new host payload, a new row on a line the diff did not add or change."""
    flags = []
    for kind, f, row in actions:
        if kind in ("MOVE-FILE", "REMOVED"):
            flags.append(f"{kind} {f}: {row}")
        elif kind == "NEW":
            c = row.split("\t")
            h = hunks(c[1])
            if row.startswith("UnknownHostPayload"):
                flags.append(f"NEW host payload (needs a disposition): {row}")
            elif h is not None and not new_changed(h, int(c[2])):
                flags.append(f"NEW on an unchanged line: {row}")
    return flags


def own_rows(log_text):
    """The exact inventory's own counts, as the passing run prints them."""
    rows = re.findall(r"EXACT INVENTORY (.+?): (\d+) rows", log_text)
    return ", ".join(f"{kind} {n}" for kind, n in rows) or "no counts printed"


# ---- fingerprints and certificates ---------------------------------------------

def fingerprint_diff(old_text, new_text):
    """(changed, problems): rows whose last column moved; any other difference is a problem."""
    old, new = old_text.splitlines(), new_text.splitlines()
    if len(old) != len(new):
        return 0, [f"row count {len(old)} -> {len(new)}"]
    changed, problems = 0, []
    for a, b in zip(old, new):
        if a.startswith("#") or not a:
            if a != b:
                problems.append(f"comment changed: {a!r}")
            continue
        if a.rsplit("\t", 1)[0] != b.rsplit("\t", 1)[0]:
            problems.append(f"a column other than the fingerprint moved: {a.split(chr(9))[0]}")
        changed += a != b
    return changed, problems


def merge_promoted(inv_text, hash_true, hash_false):
    """Column 4 from the hash build, column 5 from the no-hash build; every other
    column must agree across all three. Returns (text, changed)."""
    rows = [l.split("\t") for l in inv_text.rstrip("\n").split("\n")]
    t = [l.split("\t") for l in hash_true.rstrip("\n").split("\n")]
    f = [l.split("\t") for l in hash_false.rstrip("\n").split("\n")]
    if not len(rows) == len(t) == len(f):
        raise RuntimeError(f"promoted rows {len(rows)}/{len(t)}/{len(f)}")
    out, changed = [], 0
    for i, a, b in zip(rows, t, f):
        if i[0].startswith("#"):
            if not i == a == b:
                raise RuntimeError(f"promoted comment differs: {i}")
            out.append(i)
            continue
        for k in range(len(i)):
            if k not in (4, 5) and not i[k] == a[k] == b[k]:
                raise RuntimeError(f"promoted {i[0]} column {k} differs")
        n = i[:4] + [a[4], b[5]] + i[6:]
        changed += n != i
        out.append(n)
    return "".join("\t".join(r) + "\n" for r in out), changed


# ---- warnings ------------------------------------------------------------------

def warnings_of(text):
    """Distinct (message, file) pairs of compiler warnings; line numbers dropped."""
    found, lines = set(), text.splitlines()
    for i, line in enumerate(lines):
        m = re.match(r"^warning: (.*)$", line)
        if not m or re.search(r"generated \d+ warnings?|warnings? emitted", m.group(1)):
            continue
        loc = next((re.sub(r":\d+:\d+$", "", l.split("--> ", 1)[1].strip())
                    for l in lines[i + 1:i + 4] if "--> " in l), "-")
        found.add(f"{m.group(1)} @ {loc}")
    return sorted(found)


# ---- the steps -----------------------------------------------------------------

def step_scope(g, a):
    stage = ["--stage", a.stage] if a.stage else []
    names = [f"report-{a.stage}.json"] if a.stage else ["report.json"]
    if not any((ROOT / f".temp/brief-{a.brief}" / n).exists() for n in names):
        return result("SKIP", "no worker report (a director change)")
    path = g.log("scope.log")
    out = subprocess.run([sys.executable, "-I", str(ROOT / "scripts/report.py"), "check", str(a.brief),
                          *stage, "--against-git"], cwd=ROOT, capture_output=True, text=True)
    path.write_text(out.stdout + out.stderr)
    problems = [l for l in out.stdout.splitlines() if l.startswith("PROBLEM")]
    last = (out.stdout.strip().splitlines() or ["?"])[-1]
    return result("PASS" if out.returncode == 0 else "FLAGGED", last, problems, [path])


def step_fmt(g, a):
    path = g.log("fmt.log")
    out = subprocess.run(["cargo", "fmt", "--check"], cwd=ROOT, capture_output=True, text=True)
    path.write_text(out.stdout + out.stderr)
    files = sorted(set(re.findall(r"^Diff in (.+?)(?::\d+)?(?: at line \d+)?:\s*$", out.stdout, re.M)))
    if out.returncode == 0:
        return result("PASS", "formatted", logs=[path])
    return result("FAIL", f"{len(files)} files need rustfmt", files, [path])


def step_check(g, a):
    rc, text, facts, path = g.cargo("check", ["check", "--workspace", "--all-targets", "--offline"])
    found = warnings_of(text)
    base_file = STATE / crates_tree_of("HEAD") / "warnings.json"
    g.state["warnings"] = found
    if rc != 0:
        return result("FAIL", f"cargo check rc {rc}", [l for l in text.splitlines() if l.startswith("error")][:20], [path])
    if not base_file.exists():
        return result("PASS", f"{len(found)} warnings; no baseline for HEAD", found, [path])
    base = set(json.loads(base_file.read_text()))
    new = [w for w in found if w not in base]
    gone = [w for w in base if w not in found]
    notes = [f"NEW {w}" for w in new] + [f"GONE {w}" for w in gone]
    detail = f"{len(found)} warnings, {len(new)} new against HEAD, {len(gone)} gone"
    return result("FLAGGED" if new else "PASS", detail, notes, [path])


def step_ownership(g, a):
    inv = ROOT / OWNERSHIP
    hunks = tree_hunks("HEAD")
    actions, logs = [], []
    for k in (1, 2, 3):
        ok, t, text, path, why = filtered_run(
            g, f"ownership-{k}", ["test", "-p", "rum-stdlib", "--offline", "--lib",
                                  "production_inventory_is_exact", "--", "--nocapture"])
        logs.append(path)
        if ok:
            break
        if t["passed"] + t["failed"] == 0:
            return result("FAIL", f"round {k}: {why}", failures(text), logs)
        done = own_replay(text, inv)
        if not done:
            return result("FAIL", f"round {k} failed with nothing to replay", failures(text), logs)
        actions += done
    else:
        return result("FAIL", "still failing after three replay rounds", failures(text), logs)
    tally = collections.Counter(kind for kind, _, _ in actions)
    flags = own_flags(actions, hunks)
    notes = flags + [f"{kind} {f}: {row}" for kind, f, row in actions]
    detail = (f"{own_rows(text)}; replayed {k - 1} rounds: "
              + (", ".join(f"{n} {kind.lower()}" for kind, n in sorted(tally.items())) or "nothing"))
    return result("FLAGGED" if flags else "PASS", detail, notes, logs)


def step_census(g, a):
    dump = g.dir / "frontier.tsv"
    dump.unlink(missing_ok=True)
    rc, text, facts, path = g.cargo(
        "census", ["test", "-p", "rum-stdlib", "--offline", "--lib", "production_rejection_census_is_complete"],
        env=[f"RUM_CENSUS_DUMP={dump}"])
    if not dump.exists():
        return result("FAIL", f"no frontier written (rc {rc})", failures(text), [path])
    base = base_frontier("HEAD", g)
    counts, rows = pair(load_frontier(base), load_frontier(dump), tree_hunks("HEAD"))
    write_pairs(g.dir / "pairs.tsv", counts, rows)
    notes = [f"base {frontier_counts(base)} ({base.relative_to(ROOT)})",
             f"cand {frontier_counts(dump)}"] + pair_groups(rows)
    detail = f"{frontier_counts(dump)}; {pair_line(counts)}"
    # The test's own verdict waits for `inventories`: any edit under crates/
    # stales the contract fingerprints, and the census test certifies with
    # them after it has dumped and counted. Here the pairing decides: every
    # row paired, or a ruling on what moved (the counts and their comment).
    if rc != 0:
        panic = re.search(r"panicked at (\S+):\n(.{0,240})", text)
        notes.insert(0, "the test itself fails (decided at inventories, after the fingerprints): "
                     + (f"{panic.group(1)} {panic.group(2)}" if panic else f"rc {rc}"))
    if counts["paired"] == counts["base"] == counts["cand"]:
        return result("PASS", detail, notes, [path, g.dir / "pairs.tsv"])
    asserted = re.findall(r"^\s*(left|right): (.*)$", text, re.M)
    notes = [f"assert {side}: {v}" for side, v in asserted] + notes
    return result("FLAGGED", detail, notes, [path, g.dir / "pairs.tsv"])


def step_oracle(g, a):
    ok, t, text, path, why = filtered_run(
        g, "oracle", ["test", "-p", "rum-stdlib", "--offline", "--lib",
                      "exact_types_and_constructor_claims_match_rustc"])
    if ok:
        return result("PASS", f"{t['passed']} passed", logs=[path])
    asserted = [f"{s}: {v}" for s, v in re.findall(r"^\s*(left|right): (.*)$", text, re.M)]
    return result("FLAGGED", f"fails ({why})", asserted + failures(text), [path])


def step_standing(g, a):
    runs = [("lp01", ["test", "-p", "rum-stdlib", "--offline", "--test", "suspension_oracle",
                      "--", "logical_counter_port"]),
            ("typed-slot", ["test", "-p", "rum-stdlib", "--offline", "--test", "typed_slot_census"])]
    parts, notes, logs, bad = [], [], [], False
    for name, args in runs:
        ok, t, text, path, why = filtered_run(g, name, args)
        logs.append(path)
        parts.append(f"{name} {t['passed']}/{t['failed']}" + ("" if ok else f" ({why})"))
        if not ok:
            bad = True
            notes += [f"{name}: {f}" for f in failures(text)]
    return result("FLAGGED" if bad else "PASS", "; ".join(parts), notes, logs)


def step_fingerprints(g, a):
    out = ROOT / REGENERATED
    start = time.time()
    ok, t, text, path, why = filtered_run(
        g, "fingerprints", ["test", "-p", "rum-stdlib", "--offline", "--lib",
                            "regenerate_local_contract_fingerprints", "--", "--ignored", "--nocapture"])
    if not ok or not out.exists() or out.stat().st_mtime < start:
        return result("FAIL", f"not regenerated ({why or 'no fresh file'})", failures(text), [path])
    inv = ROOT / CONTRACTS
    changed, problems = fingerprint_diff(inv.read_text(), out.read_text())
    if problems:
        return result("FAIL", f"{len(problems)} rows moved outside the fingerprint", problems, [path])
    shutil.copyfile(out, inv)
    rows = sum(1 for l in inv.read_text().splitlines() if l and not l.startswith("#"))
    return result("PASS", f"{rows} contracts, {changed} fingerprints changed, copied in", logs=[path])


def step_certificates(g, a):
    builds = [("hash", []), ("nohash", ["--no-default-features"])]
    cert = ROOT / CERTIFICATES
    logs, merged = [], None
    for k in (1, 2):
        start = time.time()
        passed = {}
        for tag, feat in builds:
            ok, t, text, path, why = filtered_run(
                g, f"certificates-{k}-{tag}", ["test", "-p", "rum-stdlib", "--offline", "--lib",
                                               "local_production_acceptance", *feat, "--", "--nocapture"])
            logs.append(path)
            passed[tag] = ok
        if k == 2:
            break
        files = [cert / "promoted-hash-true.tsv", cert / "promoted-hash-false.tsv"]
        if any(not f.exists() or f.stat().st_mtime < start for f in files):
            return result("FAIL", "certificate cells not written this run", logs=logs)
        inv = ROOT / PROMOTED
        try:
            text, merged = merge_promoted(inv.read_text(), *(f.read_text() for f in files))
        except RuntimeError as e:
            return result("FAIL", f"merge refused: {e}", logs=logs)
        inv.write_text(text)
        if all(passed.values()) and merged == 0:
            break
    if not all(passed.values()):
        return result("FAIL", f"round {k}: " + ", ".join(f"{t} {'ok' if v else 'FAIL'}" for t, v in passed.items()),
                      logs=logs)
    return result("PASS", f"both builds pass; {merged} promoted rows merged; {k} rounds", logs=logs)


def step_inventories(g, a):
    dump = g.dir / "frontier-final.tsv"
    dump.unlink(missing_ok=True)
    ok, t, text, path, why = filtered_run(
        g, "inventories", ["test", "-p", "rum-stdlib", "--offline", "--lib", "--no-fail-fast", "--",
                           *INVENTORY_TESTS], want=len(INVENTORY_TESTS), env=[f"RUM_CENSUS_DUMP={dump}"])
    detail = f"{t['passed']} passed, {t['failed']} failed"
    if not ok:
        return result("FAIL", f"{detail} ({why})", failures(text), [path])
    return result("PASS", detail + f"; frontier {frontier_counts(dump)}", logs=[path])


def step_workspace(g, a):
    fmt = subprocess.run(["cargo", "fmt", "--check"], cwd=ROOT, capture_output=True, text=True)
    if fmt.returncode != 0:
        return result("FAIL", "cargo fmt --check fails")
    since = time.strftime("%Y-%m-%d %H:%M:%S")
    rc, text, facts, path = g.cargo("workspace", ["test", "--workspace", "--offline", "--no-fail-fast"])
    t = test_totals(text)
    kern = subprocess.run(["journalctl", "-k", "--since", since, "--no-pager"], capture_output=True, text=True)
    ooms = len(re.findall(r"oom-kill", kern.stdout, re.I))
    peak = f"{facts['peak_gb']:.1f} GB" if facts["peak_gb"] is not None else "?"
    detail = (f"{t['binaries']} binaries, {t['passed']} passed, {t['failed']} failed, {t['ignored']} ignored; "
              f"peak {peak} of 14, oom_kill {facts['oom_kill']}, kernel OOM lines {ooms}")
    ok = rc == 0 and t["failed"] == 0 and t["passed"] > 0 and ooms == 0 and facts["oom_kill"] in ("0", None)
    return result("PASS" if ok else "FAIL", detail, failures(text), [path])


def perf_engines(a):
    return [engine for engine in (getattr(a, "perf", "") or "").split(",") if engine]


def green(verdicts):
    return all(verdict in PASSING for verdict in verdicts)


def arms_verdict(receipt):
    """PASS when both arms built, both copies match their crates, and pathcal passes."""
    parts, ok = [], True
    for arm in ("base", "cand"):
        info = receipt["arms"][arm]
        rc_cli = info["builds"]["rum-cli"]["rc"]
        rc_resume = info["builds"]["resume"]["rc"]
        equal = info["source_equal"]
        parts.append(f"{arm} rc {rc_cli}/{rc_resume} source-equal={'yes' if equal else 'no'}")
        if rc_cli != 0 or rc_resume != 0 or not equal:
            ok = False
    for binary in ("resume", "rum-cli"):
        passed = receipt["pathcal"][binary]["pass"]
        parts.append(f"pathcal {binary} {'PASS' if passed else 'FAIL'}")
        if not passed:
            ok = False
    return ("PASS" if ok else "FAIL"), ", ".join(parts)


def structure_verdict(report):
    """Informational. new is only in the candidate; gone is only in the base."""
    parts, notes = [], []
    for name in ("resume", "rum-cli"):
        binary = report["binaries"][name]
        parts.append(
            f"{name}: differ {binary['differ']} (layout {binary['layout_only']}, code {len(binary['code'])}), "
            f"new {len(binary['only_b'])}, gone {len(binary['only_a'])}"
        )
        notes.extend(binary.get("notes") or [])
    return "PASS", "; ".join(parts), notes


def panels_verdict(summaries, failed):
    notes = []
    for summary in summaries:
        notes.extend(summary.get("apart_above_rows") or [])
        notes.extend(summary.get("apart_below_rows") or [])
    if failed:
        return "FAIL", "failed run: " + ", ".join(failed), notes
    detail = "; ".join(
        f"{summary['engine']}: {summary['labels']} labels, median {summary['median_pct']}, "
        f"max {summary['max_pct']}, apart above {summary['apart_above']}/below {summary['apart_below']}"
        for summary in summaries
    )
    flagged = any(summary["apart_above"] for summary in summaries)
    return ("FLAGGED" if flagged else "PASS"), detail, notes


def apply_rule(step, why):
    """Turn a FLAGGED step into RULED. A rerun of the step replaces this result."""
    if step.get("verdict") != "FLAGGED":
        raise ValueError(f"rule refuses {step.get('verdict')}: not FLAGGED")
    updated = dict(step)
    updated["verdict"] = "RULED"
    notes = list(step.get("notes") or [])
    notes.append(f"RULED: {why}")
    updated["notes"] = notes
    return updated


def stale_arms(receipt, digest):
    """Why the arms on disk are not the ones `receipt` built, or None.

    structure and panels read PERF_WORK, which every gate shares, so a run
    --from structure must not measure an earlier gate's binaries."""
    if receipt is None:
        return "no arms receipt in this gate: run the arms step first"
    for arm in ("base", "cand"):
        for name in ("resume", "rum-cli"):
            want = receipt["arms"].get(arm, {}).get("sha256", {}).get(name)
            got = digest(PERF_WORK / "arms" / arm / name)
            if want is None or got != want:
                return f"arms/{arm}/{name} is not the binary this gate's arms step built"
    return None


def file_sha256(path):
    if not path.is_file():
        return None
    return hashlib.sha256(path.read_bytes()).hexdigest()


def arms_receipt(g):
    path = g.dir / "arms.json"
    return json.loads(path.read_text()) if path.is_file() else None


def run_perf(g, name, args, json_path):
    log = g.log(f"{name}.log")
    cmd = [sys.executable, str(PERF_PANEL), *args]
    with log.open("w") as sink:
        sink.write("$ " + " ".join(cmd) + "\n")
        sink.flush()
        rc = subprocess.run(
            cmd, cwd=ROOT, stdout=sink, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL,
        ).returncode
    data = None
    if json_path is not None and Path(json_path).is_file():
        data = json.loads(Path(json_path).read_text())
    return rc, data, log


def step_arms(g, a):
    if not perf_engines(a):
        return result("SKIP", "not requested")
    receipt_path = g.dir / "arms.json"
    rc, receipt, log = run_perf(g, "arms", [
        "arms", "--work", str(PERF_WORK),
        "--base-src", str(head_worktree("HEAD")), "--cand-src", str(ROOT),
        "--json", str(receipt_path),
    ], receipt_path)
    if receipt is None:
        return result("FAIL", f"arms produced no receipt (rc {rc})", logs=[log])
    verdict, detail = arms_verdict(receipt)
    if rc != 0 and verdict == "PASS":
        verdict, detail = "FAIL", detail + f"; perf-panel rc {rc}"
    return result(verdict, detail, logs=[log])


def step_structure(g, a):
    if not perf_engines(a):
        return result("SKIP", "not requested")
    stale = stale_arms(arms_receipt(g), file_sha256)
    if stale:
        return result("FAIL", stale)
    report_path = g.dir / "structure.json"
    rc, report, log = run_perf(g, "structure", [
        "structure", str(PERF_WORK / "arms" / "base"), str(PERF_WORK / "arms" / "cand"),
        "--json", str(report_path),
    ], report_path)
    if report is None:
        return result("FAIL", f"structure produced no report (rc {rc})", logs=[log])
    verdict, detail, notes = structure_verdict(report)
    if rc != 0:
        return result("FAIL", f"structure rc {rc}", notes, [log])
    return result(verdict, detail, notes, [log])


def step_panels(g, a):
    engines = perf_engines(a)
    if not engines:
        return result("SKIP", "not requested")
    stale = stale_arms(arms_receipt(g), file_sha256)
    if stale:
        return result("FAIL", stale)
    summaries, failed, logs = [], [], []
    for engine in engines:
        if engine not in ("resume", "cli"):
            return result("FAIL", f"unknown engine {engine}")
        tsv = g.dir / f"panel-{engine}.tsv"
        rc, _data, log = run_perf(g, f"panel-{engine}", [
            "panel", str(PERF_WORK / "arms" / "base"), str(PERF_WORK / "arms" / "cand"),
            "--engine", engine, "--all", "--out", str(tsv),
        ], None)
        logs.append(log)
        if rc != 0:
            failed.append(engine)
            continue
        summary_path = g.dir / f"panel-{engine}.json"
        rc, summary, summary_log = run_perf(g, f"summary-{engine}", [
            "summary", str(tsv), "--json", str(summary_path),
        ], summary_path)
        logs.append(summary_log)
        if rc != 0 or summary is None:
            failed.append(engine)
        else:
            summaries.append(summary)
    verdict, detail, notes = panels_verdict(summaries, failed)
    return result(verdict, detail, notes, logs)


STEP_FNS = {name: globals()[f"step_{name}"] for name in STEPS}


def store_base(g):
    """After a green run: the final frontier and the warnings, keyed by the tree."""
    tree = crates_tree_of_worktree()
    d = STATE / tree
    d.mkdir(parents=True, exist_ok=True)
    final = g.dir / "frontier-final.tsv"
    if final.exists():
        shutil.copyfile(final, d / "frontier.tsv")
    if "warnings" in g.state:
        (d / "warnings.json").write_text(json.dumps(g.state["warnings"], indent=1) + "\n")
    return tree


def table(g, names):
    lines = [f"GATE brief-{g.brief}{' stage ' + g.stage if g.stage else ''}  "
             f"({g.dir.relative_to(ROOT)})", f"{'step':13s} {'verdict':8s} {'secs':>6s}  detail"]
    for name in names:
        r = g.state["steps"].get(name)
        if r:
            lines.append(f"{name:13s} {r['verdict']:8s} {r.get('secs', 0):6.0f}  {r['detail']}")
    return "\n".join(lines)


def run(a):
    g = Gate(a.brief, a.stage)
    names = a.steps.split(",") if a.steps else STEPS[STEPS.index(a.from_) if a.from_ else 0:]
    for n in names:
        if n not in STEP_FNS:
            sys.exit(f"unknown step {n!r}; steps: {', '.join(STEPS)}")
    stopped = None
    for name in names:
        start = time.time()
        try:
            r = STEP_FNS[name](g, a)
        except Exception as e:  # a broken step is a FAIL with its reason, never a crash mid-table
            r = result("FAIL", f"{type(e).__name__}: {e}")
        r["secs"] = time.time() - start
        g.save(name, r)
        print(f"  {name}: {r['verdict']} ({r['secs']:.0f}s)", file=sys.stderr, flush=True)
        if r["verdict"] in ("FAIL", "FLAGGED") and name in RULING and not a.continue_:
            stopped = name
            break
        if r["verdict"] == "FAIL" and not a.continue_:
            stopped = name
            break
    print(table(g, STEPS))
    verdicts = [g.state["steps"][n]["verdict"] for n in STEPS if n in g.state["steps"]]
    complete = all(n in g.state["steps"] for n in STEPS if n != "scope")
    if stopped:
        print(f"VERDICT: STOPPED at {stopped} ({g.state['steps'][stopped]['verdict']}). "
              f"Read: director-gate.py show {a.brief}{' --stage ' + a.stage if a.stage else ''} {stopped}; "
              f"rule, then rerun with --from {stopped}.")
        return 1
    if complete and green(verdicts):
        tree = store_base(g)
        print(f"VERDICT: GREEN. Base artifacts stored for crates tree {tree[:12]}.")
        return 0
    flagged = [n for n in STEPS if g.state["steps"].get(n, {}).get("verdict") in ("FAIL", "FLAGGED")]
    print(f"VERDICT: {'NOT GREEN: ' + ', '.join(flagged) if flagged else 'INCOMPLETE (steps not run)'}.")
    return 1


def show(a):
    g = Gate(a.brief, a.stage)
    r = g.state["steps"].get(a.step)
    if r is None:
        sys.exit(f"no result for {a.step}")
    print(f"{a.step}: {r['verdict']}  {r['detail']}")
    for n in r["notes"]:
        print(f"  {n}")
    for p in r["logs"]:
        print(f"  log: {p}")
    return 0


def rule(a):
    g = Gate(a.brief, a.stage)
    current = g.state["steps"].get(a.step)
    if current is None:
        sys.exit(f"no result for {a.step}")
    try:
        updated = apply_rule(current, a.why)
    except ValueError as exc:
        sys.exit(str(exc))
    g.save(a.step, updated)
    print(f"{a.step}: RULED  {updated['detail']}")
    return 0


# ---- selftest ------------------------------------------------------------------

def selftest():
    fails = []

    def expect(name, got, want):
        if got != want:
            fails.append(f"{name}: got {got!r}, want {want!r}")

    # A diff that inserts two lines after 15 and changes line 30.
    hunks = {"a.rs": parse_hunks("@@ -15,0 +16,2 @@\n+x\n+y\n@@ -30 +32 @@\n-a\n+b\n"), "gone.rs": None}
    expect("insertion shifts", old_to_new(hunks["a.rs"], 20), 22)
    expect("before the insertion", old_to_new(hunks["a.rs"], 10), 10)
    expect("changed line", old_to_new(hunks["a.rs"], 30), None)
    expect("added line", new_changed(hunks["a.rs"], 17), True)
    expect("unchanged line", new_changed(hunks["a.rs"], 25), False)
    expect("file head insertion", old_to_new(parse_hunks("@@ -0,0 +1,3 @@"), 1), 4)
    # Then lines 40-41 deleted: a line past all three hunks is back where it was.
    mixed = parse_hunks("@@ -15,0 +16,2 @@\n@@ -30 +32 @@\n@@ -40,2 +41,0 @@\n")
    expect("after a deletion", old_to_new(mixed, 50), 50)
    expect("a deleted line", old_to_new(mixed, 41), None)

    def row(line, callee="f", file="a.rs"):
        return ("RET002", file, line, 5, line, 9, callee, "why")
    base = [row(10), row(20), row(30), row(25, "lost"), row(3, file="gone.rs")]
    cand = [row(10), row(22), row(32, "g"), row(17, "added"), row(40, "stray")]
    counts, rows = pair(base, cand, hunks.get)
    expect("pair counts", {k: counts[k] for k in ("paired", "retired", "new", "base_only", "cand_only")},
           {"paired": 2, "retired": 2, "new": 2, "base_only": 1, "cand_only": 1})
    expect("pair classes", sorted((c, r[6]) for c, r, _ in rows),
           [("BASE-ONLY", "lost"), ("CAND-ONLY", "stray"), ("NEW", "added"), ("NEW", "g"),
            ("RETIRED", "f"), ("RETIRED", "f")])
    # A shared body's rows count once per witness: two equal rows need two partners.
    counts, _ = pair([row(10), row(10)], [row(10)], hunks.get)
    expect("multiset", (counts["paired"], counts["base_only"]), (1, 1))

    log = ("test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured\n"
           "test x::y ... FAILED\n"
           "test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured\n"
           "# memory-scope: unit=u max=15032385536 peak=10500000000 oom=0 oom_kill=0 rc=101\n")
    expect("totals", test_totals(log), {"binaries": 2, "passed": 7, "failed": 1, "ignored": 1})
    expect("failures", failures(log), ["x::y"])
    expect("scope line", scope_facts(log), {"peak_gb": 10.5, "oom_kill": "0"})

    warn = ("warning: unused import: `x`\n  --> crates/a/src/b.rs:3:5\n"
            "warning: `rum-a` (lib) generated 1 warning\n"
            "warning: field `y` is never read\n --> crates/a/src/c.rs:9:1\n")
    expect("warnings", warnings_of(warn),
           ["field `y` is never read @ crates/a/src/c.rs", "unused import: `x` @ crates/a/src/b.rs"])

    expect("fingerprint only", fingerprint_diff("# h\na\tb\tF1\n", "# h\na\tb\tF2\n"), (1, []))
    expect("fingerprint other column", fingerprint_diff("a\tb\tF1\n", "a\tc\tF2\n")[1] != [], True)
    inv = "# h\nr\t1\t2\t3\tT0\tF0\tz\n"
    expect("merge", merge_promoted(inv, "# h\nr\t1\t2\t3\tT1\tX\tz\n", "# h\nr\t1\t2\t3\tX\tF1\tz\n"),
           ("# h\nr\t1\t2\t3\tT1\tF1\tz\n", 1))
    try:
        merge_promoted(inv, "# h\nr\t1\t9\t3\tT1\tX\tz\n", "# h\nr\t1\t2\t3\tX\tF1\tz\n")
        fails.append("merge: a differing column was accepted")
    except RuntimeError:
        pass

    with tempfile.TemporaryDirectory(dir=ROOT / ".temp") as d:
        inv = Path(d)
        storage = ["UnknownConstruct\ta.rs\t10\t4\tthing/arg0: foreign x", "UnknownConstruct\tb.rs\t5\t1\tgone/arg1: y"]
        (inv / "unknown_storage.tsv").write_text("\n".join(storage) + "\n")
        (inv / "unknown_macros.tsv").write_text("")
        (inv / "self_reaching.tsv").write_text("")
        (inv / "host_payloads.tsv").write_text("")
        (inv / "host_payload_ledger.tsv").write_text("head\n")
        log = ("stale inventory row: UnknownConstruct\ta.rs\t10\t4\tthing/arg0: foreign x\n"
               "missing inventory row: UnknownConstruct\ta.rs\t12\t4\tthing/arg0: foreign x\n"
               "stale inventory row: UnknownConstruct\tb.rs\t5\t1\tgone/arg1: y\n"
               "missing inventory row: UnknownConstruct\ta.rs\t17\t2\tunexpanded macro format\n"
               "missing inventory row: UnknownConstruct\ta.rs\t25\t2\tunexpanded macro debug_assert\n")
        actions = own_replay(log, inv)
        expect("replay kinds", sorted(k for k, _, _ in actions), ["MOVE", "NEW", "NEW", "REMOVED"])
        expect("replay storage", (inv / "unknown_storage.tsv").read_text(),
               "UnknownConstruct\ta.rs\t12\t4\tthing/arg0: foreign x\n")
        expect("replay macros", (inv / "unknown_macros.tsv").read_text().count("\n"), 2)
        flags = own_flags(actions, hunks.get)
        expect("replay flags", sorted(f.split(":")[0] for f in flags),
               ["NEW on an unchanged line", "REMOVED unknown_storage.tsv"])

    receipt = {
        "arms": {
            "base": {"source_equal": True, "builds": {"rum-cli": {"rc": 0}, "resume": {"rc": 0}}},
            "cand": {"source_equal": True, "builds": {"rum-cli": {"rc": 0}, "resume": {"rc": 0}}},
        },
        "pathcal": {"resume": {"pass": True}, "rum-cli": {"pass": True}},
    }
    expect("arms detail", arms_verdict(receipt), (
        "PASS",
        "base rc 0/0 source-equal=yes, cand rc 0/0 source-equal=yes, pathcal resume PASS, pathcal rum-cli PASS",
    ))
    bad = {
        "arms": {
            "base": {"source_equal": True, "builds": {"rum-cli": {"rc": 0}, "resume": {"rc": 0}}},
            "cand": {"source_equal": False, "builds": {"rum-cli": {"rc": 1}, "resume": {"rc": 0}}},
        },
        "pathcal": {"resume": {"pass": False}, "rum-cli": {"pass": True}},
    }
    verdict, detail = arms_verdict(bad)
    expect("arms fail", verdict, "FAIL")
    expect("arms fail detail", "rc 1/0" in detail and "source-equal=no" in detail and "pathcal resume FAIL" in detail, True)

    structure = {"binaries": {
        "resume": {"differ": 3, "layout_only": 0, "code": [{}, {}, {}], "only_a": [], "only_b": [{}],
                   "notes": ["CODE [502] -> [520] abandon_frames_profiled"]},
        "rum-cli": {"differ": 3, "layout_only": 0, "code": [{}, {}, {}], "only_a": [], "only_b": [{}],
                    "notes": ["ONLY_B 44\treport_abandoned_cleanup_error"]},
    }}
    verdict, detail, notes = structure_verdict(structure)
    expect("structure verdict", verdict, "PASS")
    expect("structure detail", detail,
           "resume: differ 3 (layout 0, code 3), new 1, gone 0; rum-cli: differ 3 (layout 0, code 3), new 1, gone 0")
    expect("structure notes", notes,
           ["CODE [502] -> [520] abandon_frames_profiled", "ONLY_B 44\treport_abandoned_cleanup_error"])

    quiet = {"engine": "resume", "labels": 121, "median_pct": "+0.0031%", "max_pct": "+0.1642%",
             "apart_above": 0, "apart_below": 0, "apart_above_rows": [], "apart_below_rows": []}
    expect("panels pass", panels_verdict([quiet], [])[:2], (
        "PASS", "resume: 121 labels, median +0.0031%, max +0.1642%, apart above 0/below 0",
    ))
    hot = dict(quiet, apart_above=1, apart_above_rows=["  hot +1.0/iter +1.0000% base 1-2 cand 4-5"])
    verdict, _detail, notes = panels_verdict([hot], [])
    expect("panels flagged", verdict, "FLAGGED")
    expect("panels flagged note", notes, hot["apart_above_rows"])
    cold = dict(quiet, apart_below=1, apart_below_rows=["  cold -1.0/iter -1.0000% base 4-5 cand 1-2"])
    verdict, detail, notes = panels_verdict([cold], [])
    expect("panels below", verdict, "PASS")
    expect("panels below detail", "apart above 0/below 1" in detail, True)
    expect("panels below note", notes, cold["apart_below_rows"])
    expect("panels failed run", panels_verdict([], ["cli"])[0], "FAIL")

    flagged = {"verdict": "FLAGGED",
               "detail": "resume: 1 labels, median +1.0000%, max +1.0000%, apart above 1/below 0",
               "notes": ["  hot +1.0/iter +1.0000% base 1-2 cand 4-5"], "secs": 3}
    ruled = apply_rule(flagged, "layout effect")
    expect("rule verdict", ruled["verdict"], "RULED")
    expect("rule detail", ruled["detail"], flagged["detail"])
    expect("rule note", ruled["notes"][-1], "RULED: layout effect")
    expect("rule keeps earlier notes", ruled["notes"][0], flagged["notes"][0])
    try:
        apply_rule({"verdict": "PASS", "detail": "ok", "notes": []}, "no")
        fails.append("rule refuses: a PASS step was ruled")
    except ValueError:
        pass
    state = {"panels": ruled}
    state["panels"] = {"verdict": "PASS", "detail": "fresh", "notes": []}
    expect("rerun replaces ruling", state["panels"]["verdict"], "PASS")
    built = {"arms": {arm: {"sha256": {"resume": f"{arm}-r", "rum-cli": f"{arm}-c"}}
                      for arm in ("base", "cand")}}
    current = {PERF_WORK / "arms" / arm / name: built["arms"][arm]["sha256"][name]
               for arm in ("base", "cand") for name in ("resume", "rum-cli")}
    expect("arms current", stale_arms(built, current.get), None)
    expect("arms no receipt", stale_arms(None, current.get) is not None, True)
    current[PERF_WORK / "arms/cand/resume"] = "older"
    expect("arms stale", stale_arms(built, current.get), "arms/cand/resume is not the binary this gate's arms step built")
    expect("green counts RULED", green(["PASS", "SKIP", "RULED"]), True)
    expect("green rejects FLAGGED", green(["PASS", "FLAGGED"]), False)

    for f in fails:
        print(f"SELFTEST FAIL {f}")
    print(f"selftest: {'PASS' if not fails else f'{len(fails)} failures'}")
    return 1 if fails else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("brief")
    r.add_argument("--stage")
    r.add_argument("--steps")
    r.add_argument("--from", dest="from_")
    r.add_argument("--continue", dest="continue_", action="store_true")
    r.add_argument("--perf", default="", help="comma list of perf engines, e.g. resume,cli")
    s = sub.add_parser("show")
    s.add_argument("brief")
    s.add_argument("--stage")
    s.add_argument("step")
    rule_cmd = sub.add_parser("rule")
    rule_cmd.add_argument("brief")
    rule_cmd.add_argument("--stage")
    rule_cmd.add_argument("step")
    rule_cmd.add_argument("--why", required=True)
    p = sub.add_parser("pair")
    p.add_argument("base")
    p.add_argument("cand")
    p.add_argument("--rev", default="HEAD")
    b = sub.add_parser("base-frontier")
    b.add_argument("--rev", default="HEAD")
    sub.add_parser("selftest")
    a = ap.parse_args()
    if a.cmd == "run":
        return run(a)
    if a.cmd == "show":
        return show(a)
    if a.cmd == "rule":
        return rule(a)
    if a.cmd == "pair":
        counts, rows = pair(load_frontier(a.base), load_frontier(a.cand), tree_hunks(a.rev))
        print("\n".join(pair_groups(rows)))
        print(f"CENSUSPAIR base={counts['base']} cand={counts['cand']} {pair_line(counts)}")
        return 0
    if a.cmd == "base-frontier":
        print(base_frontier(a.rev).relative_to(ROOT))
        return 0
    return selftest()


if __name__ == "__main__":
    sys.exit(main())
