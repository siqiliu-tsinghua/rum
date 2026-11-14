#!/usr/bin/env python3
"""Director campaign gate for brief 623: content-addressed bookkeeping checks.

One PASS/FAIL line per check; evidence reading stays human. Repo-relative paths.
"""
import argparse
import hashlib
import json
import os
import subprocess
import sys
from datetime import datetime
from pathlib import Path

R = Path(__file__).resolve().parent.parent
SETTLEMENT_COLUMNS = {
    "verdict", "tail_observed", "disposition", "mutant_result", "ledger_kind",
    "settlement_evidence",
}


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def val(p):
    p = Path(p)
    return "symlink:" + os.readlink(p) if p.is_symlink() else sha(p)


def repo_rel(p):
    return str(Path(p).resolve().relative_to(R))


def manifest(path):
    d = json.loads(Path(path).read_text())
    return d["files"] if isinstance(d, dict) and "files" in d else d


def check_live(scene):
    man = manifest(R / scene / "entry-hashes.json")
    missing, changed = [], []
    for rel, h in man.items():
        p = R / rel
        if p.is_symlink():
            v = val(p)
        elif not p.exists():
            missing.append(rel)
            continue
        else:
            v = sha(p)
        if v != h:
            changed.append(rel)
    ok = not missing and not changed
    print(f"GATE live files={len(man)} missing={len(missing)} changed={len(changed)} "
          f"{'PASS' if ok else 'FAIL'}")
    for rel in (missing + changed)[:5]:
        print(" ", rel)
    return ok


def is_mutable(scene, p, seal):
    name = p.name
    if name in seal.get("mutable_exclusions", []):
        return True
    rel = p.relative_to(scene)
    if rel.parts and rel.parts[0] == "logs":
        return any(name.startswith(pre) for pre in seal.get("mutable_log_prefixes", []))
    return False


def check_refresh(spec):
    p = R / spec["path"]
    actual = sha(p)
    if actual != spec["actual_sha256"]:
        return False, f"actual {actual} != spec"
    raw = p.read_bytes()
    old, new = spec["old"].encode(), spec["new"].encode()
    if raw.count(new) != 1:
        return False, "new value not unique"
    rebuilt = raw.replace(new, old)
    if hashlib.sha256(rebuilt).hexdigest() != spec["sealed_sha256"]:
        return False, "reconstruction does not reach the sealed hash"
    rec = json.loads(rebuilt)
    if rec.get(spec["field"]) != spec["old"]:
        return False, "field value"
    for key, h in spec.get("outputs", {}).items():
        if rec.get("outputs", {}).get(key) != h:
            return False, f"output {key}"
    return True, ""


def check_seal(scene, allow_refresh=None):
    scene = (R / scene).resolve()
    seal = json.loads((scene / "artifact-hashes.json").read_text())
    files = seal["files"]
    missing, changed = [], []
    for rel, h in files.items():
        p = R / rel
        if not p.exists():
            missing.append(rel)
            continue
        if val(p) != h:
            changed.append(rel)
    extra_mutable, extra = [], []
    for parent, _dirs, names in os.walk(scene):
        for n in sorted(names):
            p = Path(parent) / n
            rel = repo_rel(p)
            if rel in files:
                continue
            (extra_mutable if is_mutable(scene, p, seal) else extra).append(rel)
    allowed_changed = None
    refresh_ok, why = True, ""
    spec = json.loads(Path(allow_refresh).read_text()) if allow_refresh else None
    if spec:
        allowed_changed = spec["path"]
        refresh_ok, why = check_refresh(spec)
    unexpected_changed = [x for x in changed if x != allowed_changed]
    allowed_extra = set(spec.get("extra_members", [])) if spec else set()
    unexpected_extra = [x for x in extra if x not in allowed_extra]
    ok = (not missing and not unexpected_changed and not unexpected_extra
          and (not changed or refresh_ok))
    print(f"GATE seal scene={scene.relative_to(R)} sealed={len(files)} missing={len(missing)} "
          f"changed={len(changed)} changed_allowed={1 if changed and refresh_ok else 0} "
          f"extras_mutable={len(extra_mutable)} extras_unexpected={len(unexpected_extra)} "
          f"{'PASS' if ok else 'FAIL'}")
    if why:
        print("  refresh:", why)
    for rel in (missing + unexpected_changed + unexpected_extra)[:5]:
        print(" ", rel)
    return ok


def read_tsv(path):
    rows = [line.rstrip("\n").split("\t") for line in Path(path).read_text().splitlines()]
    head = rows[0]
    return head, {r[0]: r for r in rows[1:] if r and r[0]}


def check_move(scene, baseline):
    h1, r1 = read_tsv(R / scene / "settled.tsv")
    h2, r2 = read_tsv(R / baseline / "settled.tsv")
    if h1 != h2:
        print("GATE move FAIL headers differ")
        return False
    added = [i for i in r1 if i not in r2]
    removed = [i for i in r2 if i not in r1]
    moved, unexpected = [], []
    for i in r1:
        if i not in r2:
            continue
        a, b = r1[i], r2[i]
        if len(a) != len(b):
            unexpected.append((i, ["<column-count>"]))
            continue
        cols = [h1[k] for k, (x, y) in enumerate(zip(a, b)) if x != y]
        if cols:
            moved.append((i, cols))
            if set(cols) - SETTLEMENT_COLUMNS:
                unexpected.append((i, sorted(set(cols) - SETTLEMENT_COLUMNS)))
    tuples = (R / scene / "required-tuples.tsv", R / baseline / "required-tuples.tsv")
    tuples_equal = all(p.exists() for p in tuples) and sha(tuples[0]) == sha(tuples[1])
    ok = not removed and not unexpected
    print(f"GATE move scene={scene} baseline={baseline} rows={len(r1)} added={len(added)} "
          f"removed={len(removed)} moved={len(moved)} unexpected_columns={len(unexpected)} "
          f"required_tuples_identical={tuples_equal} {'PASS' if ok else 'FAIL'}")
    for i, cols in moved[:8]:
        print(f"  {i}: {','.join(cols)}")
    for i, cols in (added[:5] + unexpected[:5]):
        print("  problem", i, cols)
    return ok


def check_protected(scene, roots, allows):
    expected = manifest(R / scene / "protected-entry.json")
    if not roots:
        roots = sorted({"/".join(rel.split("/")[:3]) if rel.startswith(".temp/g15-c2/")
                        else "/".join(rel.split("/")[:2]) for rel in expected})
    allowed = dict(a.split("=", 1) for a in allows)
    seen, diffs, missing = set(), [], []
    for root in roots:
        for parent, dirs, names in os.walk(R / root):
            dirs.sort()
            for n in sorted(names):
                p = Path(parent) / n
                rel = repo_rel(p)
                seen.add(rel)
                v = val(p)
                if rel in allowed:
                    if v != allowed[rel]:
                        diffs.append(("publication_mismatch", rel))
                    continue
                e = expected.get(rel)
                if e is None:
                    diffs.append(("extra", rel))
                elif v != e:
                    diffs.append(("changed", rel))
    missing = [rel for rel in expected if rel not in seen]
    ok = not diffs and not missing
    print(f"GATE protected scene={scene} expected={len(expected)} seen={len(seen)} "
          f"diffs={len(diffs)} missing={len(missing)} allowed={len(allowed)} "
          f"{'PASS' if ok else 'FAIL'}")
    for kind, rel in diffs[:5]:
        print(" ", kind, rel)
    for rel in missing[:5]:
        print("  missing", rel)
    return ok


def check_mtime(roots, since):
    ts = datetime.strptime(since, "%Y-%m-%d %H:%M:%S").timestamp()
    newer = []
    for root in roots:
        for parent, _dirs, names in os.walk(R / root):
            for n in names:
                p = Path(parent) / n
                try:
                    if p.stat().st_mtime > ts:
                        newer.append(repo_rel(p))
                except OSError:
                    pass
    print(f"GATE mtime roots={len(roots)} since={since} newer={len(newer)}")
    for rel in newer[:10]:
        print(" ", rel)
    return True


def check_report(brief, scene):
    live = R / f".temp/brief-{brief}/report.json"
    copy = R / scene / "report.json"
    same = live.exists() and copy.exists() and sha(live) == sha(copy)
    proc = subprocess.run([sys.executable, str(R / "scripts/report.py"), "check", str(brief)],
                          capture_output=True, text=True)
    ok = same and proc.returncode == 0
    tail = (proc.stdout.strip().splitlines() or proc.stderr.strip().splitlines() or [""])[-1]
    print(f"GATE report brief={brief} live_equals_scene={same} check_rc={proc.returncode} "
          f"sha256={sha(live) if live.exists() else 'missing'} {'PASS' if ok else 'FAIL'}")
    print(" ", tail[:160])
    return ok


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("live"); p.add_argument("scene")
    p = sub.add_parser("seal"); p.add_argument("scene"); p.add_argument("--allow-seal-refresh")
    p = sub.add_parser("move"); p.add_argument("scene"); p.add_argument("baseline")
    p = sub.add_parser("protected"); p.add_argument("scene")
    p.add_argument("--root", action="append", default=[])
    p.add_argument("--allow", action="append", default=[])
    p = sub.add_parser("mtime"); p.add_argument("--root", action="append", default=[])
    p.add_argument("--since", required=True)
    p = sub.add_parser("report"); p.add_argument("--brief", required=True)
    p.add_argument("--scene", required=True)
    a = ap.parse_args()
    if a.cmd == "live":
        ok = check_live(a.scene)
    elif a.cmd == "seal":
        ok = check_seal(a.scene, a.allow_seal_refresh)
    elif a.cmd == "move":
        ok = check_move(a.scene, a.baseline)
    elif a.cmd == "protected":
        ok = check_protected(a.scene, a.root, a.allow)
    elif a.cmd == "mtime":
        ok = check_mtime(a.root, a.since)
    else:
        ok = check_report(a.brief, a.scene)
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
