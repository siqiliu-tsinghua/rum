#!/usr/bin/env python3
"""Structured worker reports for rum briefs (schema rum-report/1).

A worker writes its report as JSON at .temp/brief-NNN/report.json (a staged
brief: report-<stage>.json) and runs `check` before it finishes. The director
reads `summary` and pulls one element with `show`. `render` turns a report
into the markdown record appended to the brief file.

  report.py schema                       print the schema and an example
  report.py check   NNN|PATH [--stage S] [--against-git]
  report.py summary NNN|PATH [--stage S]
  report.py show    NNN|PATH [--stage S] (--tag T | --check ID | --run ID |
                    --measurement ID | --question ID | --changes |
                    --deviations | --stop)
  report.py render  NNN|PATH [--stage S]
  report.py selftest

`check` is mechanical: every cited line must appear verbatim in its log, a
run's test counts must equal the sum of its log's `test result:` lines, a
FLAGGED row must be settled in its note, DONE carries no FAIL, and STOP
carries its stop. With --against-git (the director's gate) it also compares
`changes` with the working tree.
"""

import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

SCHEMA = "rum-report/1"
ROOT = Path(__file__).resolve().parent.parent
VERDICTS = {"DONE", "STOP"}
ROW_VERDICTS = {"PASS", "FAIL", "FLAGGED", "NA"}
CHANGE_KINDS = {"added", "modified", "deleted"}
TEST_RESULT = re.compile(
    r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored"
)

EXAMPLE = {
    "schema": SCHEMA,
    "brief": 631,
    "stage": "A",
    "worker": "grk",
    "verdict": "STOP",
    "summary": "One paragraph: what was done, what the numbers say, why it stopped.",
    "stop": {
        "rule": "the brief's stop rule that fired, quoted",
        "reason": "what happened",
        "evidence": [{"log": ".temp/brief-631/logs/census.log", "line": "frontier 1046 != 1040"}],
    },
    "changes": [
        {"path": "crates/rum-runtime/src/x.rs", "kind": "modified", "lines": "120-188", "why": "the arm"}
    ],
    "disclosures": [{"path": "crates/rum-runtime/src/y.rs", "why": "outside the allowed list: ..."}],
    "runs": [
        {
            "id": "ws",
            "cmd": "scripts/memory-scope.sh ws 14G cargo test --workspace --no-fail-fast",
            "log": ".temp/brief-631/logs/ws.log",
            "exit": 0,
            "attempt": 1,
            "tests": {"passed": 4013, "failed": 0, "ignored": 61},
            "verdict_lines": ["test result: ok. 1111 passed; 0 failed; 0 ignored"],
        }
    ],
    "checks": [
        {
            "id": "census",
            "bar": "frontier unchanged except the arm's lines",
            "expected": "1040",
            "actual": "1046",
            "verdict": "FLAGGED",
            "note": "how the flagged row is settled, or why it stops",
            "evidence": [{"log": ".temp/brief-631/logs/census.log", "line": "frontier 1046 != 1040"}],
        }
    ],
    "measurements": [
        {
            "id": "resume-panel",
            "bar": "no label apart from base in every rep",
            "result": "median +0.003%, max +0.16%, none apart",
            "verdict": "PASS",
            "raw": ".temp/brief-631/panel.tsv",
            "evidence": [{"log": ".temp/brief-631/logs/panel.log", "line": "apart: 0"}],
        }
    ],
    "deviations": [{"what": "what differs from the brief", "why": "why"}],
    "questions": [
        {"id": "Q1", "question": "...", "options": ["a", "b"], "recommendation": "a, because ..."}
    ],
    "sections": [{"tag": "shape", "title": "The arm's shape", "body": "markdown"}],
}

SCHEMA_TEXT = """rum-report/1. Paths are relative to the repository root.

Required: schema, brief (int), stage (str, "" when the brief has none),
worker (str), verdict (DONE | STOP), summary (str, at most 1200 chars),
stop (null, or {rule, reason, evidence[]} when the verdict is STOP).
Lists (each may be empty, none may be missing):
  changes[]      {path, kind: added|modified|deleted, lines, why}
  disclosures[]  {path, why}: anything outside the brief's allowed list
  runs[]         {id, cmd, log, exit (int), attempt (int), tests (null |
                 {passed, failed, ignored}), verdict_lines[] (at least one)}
                 every run you made, failed attempts included
  checks[]       {id, bar, expected, actual, verdict, note, evidence[]}
  measurements[] {id, bar, result, verdict, raw, evidence[]}
  deviations[]   {what, why}
  questions[]    {id, question, options[], recommendation}
  sections[]     {tag, title, body (markdown)}: long reasoning goes here
Row verdicts: PASS | FAIL | FLAGGED | NA. A FLAGGED row needs a note that
settles it. A DONE report carries no FAIL.
Evidence: {log, line}; the line must appear verbatim in the log (a whole
line or a part of one). Quote every verdict line your instruments print.
"""


def fail(msg):
    print(f"report.py: {msg}", file=sys.stderr)
    sys.exit(2)


def locate(target, stage):
    p = Path(target)
    if p.suffix == ".json":
        return p if p.is_absolute() else ROOT / p
    m = re.fullmatch(r"(?:brief-)?(\d+)", target)
    if not m:
        fail(f"not a brief number or a .json path: {target}")
    d = ROOT / ".temp" / f"brief-{m.group(1)}"
    if stage:
        return d / f"report-{stage}.json"
    single = d / "report.json"
    if single.exists():
        return single
    staged = sorted(d.glob("report-*.json"), key=lambda q: q.stat().st_mtime)
    if staged:
        return staged[-1]
    return single


def load(path):
    if not path.exists():
        fail(f"no report at {path}")
    try:
        return json.loads(path.read_text())
    except json.JSONDecodeError as e:
        fail(f"{path}: not JSON: {e}")


def lines_of(rel, cache):
    if rel not in cache:
        p = ROOT / rel
        cache[rel] = p.read_text(errors="replace").splitlines() if p.is_file() else None
    return cache[rel]


def check(rep, against_git=False):
    problems = []
    cited = 0
    cache = {}

    def need(obj, key, kind, where):
        if not isinstance(obj, dict) or key not in obj:
            problems.append(f"{where}: missing '{key}'")
            return None
        v = obj[key]
        if kind is not None and not isinstance(v, kind):
            problems.append(f"{where}.{key}: expected {kind.__name__ if isinstance(kind, type) else kind}")
            return None
        return v

    def evidence(items, where):
        nonlocal cited
        if not isinstance(items, list):
            problems.append(f"{where}: evidence must be a list")
            return
        for i, ev in enumerate(items):
            log = need(ev, "log", str, f"{where}.evidence[{i}]")
            line = need(ev, "line", str, f"{where}.evidence[{i}]")
            if log is None or line is None:
                continue
            text = lines_of(log, cache)
            if text is None:
                problems.append(f"{where}.evidence[{i}]: log {log} does not exist")
            elif not line.strip() or not any(line.strip() in t for t in text):
                problems.append(f"{where}.evidence[{i}]: line not found verbatim in {log}: {line[:80]!r}")
            else:
                cited += 1

    if rep.get("schema") != SCHEMA:
        problems.append(f"schema: expected {SCHEMA!r}")
    need(rep, "brief", int, "report")
    need(rep, "stage", str, "report")
    need(rep, "worker", str, "report")
    verdict = need(rep, "verdict", str, "report")
    if verdict is not None and verdict not in VERDICTS:
        problems.append(f"verdict: {verdict!r} is not DONE or STOP")
    summary = need(rep, "summary", str, "report")
    if summary is not None and len(summary) > 1200:
        problems.append(f"summary: {len(summary)} chars, at most 1200 (long text goes into sections)")
    if "stop" not in rep:
        problems.append("report: missing 'stop' (null when DONE)")
    stop = rep.get("stop")
    if verdict == "STOP":
        if not isinstance(stop, dict):
            problems.append("stop: a STOP report needs {rule, reason, evidence}")
        else:
            need(stop, "rule", str, "stop")
            need(stop, "reason", str, "stop")
            evidence(stop.get("evidence", []), "stop")
    elif verdict == "DONE" and stop is not None:
        problems.append("stop: a DONE report has stop null")

    lists = {}
    for key in ("changes", "disclosures", "runs", "checks", "measurements", "deviations", "questions", "sections"):
        v = need(rep, key, list, "report")
        lists[key] = v if v is not None else []

    for i, c in enumerate(lists["changes"]):
        w = f"changes[{i}]"
        path, kind = need(c, "path", str, w), need(c, "kind", str, w)
        need(c, "lines", str, w), need(c, "why", str, w)
        if kind is not None and kind not in CHANGE_KINDS:
            problems.append(f"{w}.kind: {kind!r}")
        if path is not None and kind is not None:
            exists = (ROOT / path).exists()
            if kind == "deleted" and exists:
                problems.append(f"{w}: {path} is reported deleted but exists")
            if kind != "deleted" and not exists:
                problems.append(f"{w}: {path} is reported {kind} but does not exist")
    for i, d in enumerate(lists["disclosures"]):
        need(d, "path", str, f"disclosures[{i}]"), need(d, "why", str, f"disclosures[{i}]")

    for i, r in enumerate(lists["runs"]):
        w = f"runs[{i}]"
        rid, log = need(r, "id", str, w), need(r, "log", str, w)
        need(r, "cmd", str, w), need(r, "exit", int, w), need(r, "attempt", int, w)
        vlines = need(r, "verdict_lines", list, w)
        w = f"runs[{rid or i}]"
        if log is None:
            continue
        text = lines_of(log, cache)
        if text is None:
            problems.append(f"{w}: log {log} does not exist")
            continue
        if vlines is not None:
            if not vlines:
                problems.append(f"{w}: verdict_lines is empty (quote what the run printed)")
            for v in vlines:
                if not isinstance(v, str) or not v.strip() or not any(v.strip() in t for t in text):
                    problems.append(f"{w}: verdict line not found verbatim in {log}: {str(v)[:80]!r}")
                else:
                    cited += 1
        if "tests" not in r:
            problems.append(f"{w}: missing 'tests' (null for a run that is not cargo test)")
        elif r["tests"] is not None:
            t = r["tests"]
            found = [tuple(map(int, m.groups())) for line in text for m in [TEST_RESULT.search(line)] if m]
            if not found:
                problems.append(f"{w}: tests reported but {log} has no 'test result:' line")
            elif isinstance(t, dict):
                got = tuple(sum(x[k] for x in found) for k in range(3))
                want = (t.get("passed"), t.get("failed"), t.get("ignored"))
                if got != want:
                    problems.append(f"{w}: tests {want} but {log} sums to {got} over {len(found)} binaries")
            else:
                problems.append(f"{w}.tests: expected an object or null")

    def rows(key, extra):
        ids = set()
        for i, r in enumerate(lists[key]):
            w = f"{key}[{i}]"
            rid = need(r, "id", str, w)
            if rid is not None:
                if rid in ids:
                    problems.append(f"{key}: duplicate id {rid!r}")
                ids.add(rid)
                w = f"{key}[{rid}]"
            for k in extra:
                need(r, k, str, w)
            v = need(r, "verdict", str, w)
            if v is not None and v not in ROW_VERDICTS:
                problems.append(f"{w}.verdict: {v!r}")
            if v == "FLAGGED" and not str(r.get("note", "")).strip():
                problems.append(f"{w}: FLAGGED with no note settling it")
            if v == "FAIL" and verdict == "DONE":
                problems.append(f"{w}: FAIL in a DONE report")
            evidence(r.get("evidence", []), w)

    rows("checks", ("bar", "expected", "actual", "note"))
    rows("measurements", ("bar", "result", "raw"))
    for key in ("questions", "sections"):
        ids = set()
        idk = "id" if key == "questions" else "tag"
        for i, q in enumerate(lists[key]):
            v = need(q, idk, str, f"{key}[{i}]")
            if v in ids:
                problems.append(f"{key}: duplicate {idk} {v!r}")
            ids.add(v)
    for i, s in enumerate(lists["sections"]):
        need(s, "title", str, f"sections[{i}]"), need(s, "body", str, f"sections[{i}]")

    if against_git:
        problems += git_scope(lists["changes"])
    return problems, cited


def git_scope(changes):
    out = subprocess.run(
        ["git", "status", "--porcelain", "--untracked-files=all"], cwd=ROOT, capture_output=True, text=True
    )
    if out.returncode != 0:
        return [f"git status failed: {out.stderr.strip()}"]
    tree = {line[3:].split(" -> ")[-1] for line in out.stdout.splitlines() if line.strip()}
    reported = {c.get("path") for c in changes if isinstance(c, dict)}
    problems = [f"scope: {p} changed in the tree but not in changes" for p in sorted(tree - reported)]
    problems += [f"scope: {p} in changes but unchanged in the tree" for p in sorted(reported - tree)]
    return problems


def counts(t):
    return "-" if not t else f"{t.get('passed')}/{t.get('failed')}/{t.get('ignored')}"


def summary(rep, path):
    out = []
    stage = f" stage {rep.get('stage')}" if rep.get("stage") else ""
    out.append(f"brief-{rep.get('brief')}{stage} ({rep.get('worker')}): {rep.get('verdict')}")
    stop = rep.get("stop")
    if stop:
        out.append(f"  STOP rule: {stop.get('rule')}")
        out.append(f"  STOP reason: {stop.get('reason')}")
    out.append(f"summary: {rep.get('summary')}")
    ch = rep.get("changes", [])
    out.append(
        f"changes: {len(ch)} files ("
        + ", ".join(f"{sum(1 for c in ch if c.get('kind') == k)} {k}" for k in ("added", "modified", "deleted"))
        + f"); disclosures: {len(rep.get('disclosures', []))}"
    )
    for d in rep.get("disclosures", []):
        out.append(f"  DISCLOSED {d.get('path')}: {d.get('why')}")
    out.append("runs:")
    for r in rep.get("runs", []):
        out.append(f"  {r.get('id'):<18} try {r.get('attempt')}  exit {r.get('exit'):<3} {counts(r.get('tests')):<14} {r.get('log')}")
    for key in ("checks", "measurements"):
        rs = rep.get(key, [])
        tally = {v: sum(1 for r in rs if r.get("verdict") == v) for v in ("PASS", "FLAGGED", "FAIL", "NA")}
        out.append(f"{key}: " + ", ".join(f"{n} {v}" for v, n in tally.items() if n) if rs else f"{key}: none")
        for r in rs:
            if r.get("verdict") not in ("PASS", "NA"):
                shown = r.get("actual", r.get("result"))
                out.append(f"  {r.get('verdict')} {r.get('id')}: {shown} (bar: {r.get('bar')}) -- {r.get('note', '')}")
    q = rep.get("questions", [])
    out.append(f"deviations: {len(rep.get('deviations', []))}; questions: " + (", ".join(x.get("id", "?") for x in q) or "none"))
    for x in q:
        out.append(f"  {x.get('id')}: {x.get('question')}")
    secs = rep.get("sections", [])
    out.append("sections: " + (", ".join(f"{s.get('tag')}({len(s.get('body', '')) // 100 / 10}k)" for s in secs) or "none"))
    problems, cited = check(rep)
    out.append(f"check: OK, {cited} citations verified" if not problems else f"check: {len(problems)} PROBLEMS (run check)")
    out.append(f"file: {path.relative_to(ROOT) if path.is_relative_to(ROOT) else path}")
    return "\n".join(out)


def show(rep, a):
    def pick(key, field, value):
        for x in rep.get(key, []):
            if x.get(field) == value:
                return x
        fail(f"no {key} entry with {field} {value!r}")

    if a.tag:
        s = pick("sections", "tag", a.tag)
        return f"## {s.get('title')}\n\n{s.get('body')}"
    for key, flag in (("checks", a.check), ("runs", a.run), ("measurements", a.measurement), ("questions", a.question)):
        if flag:
            return json.dumps(pick(key, "id", flag), indent=2, ensure_ascii=False)
    for key, flag in (("changes", a.changes), ("deviations", a.deviations)):
        if flag:
            return json.dumps(rep.get(key, []), indent=2, ensure_ascii=False)
    if a.stop:
        return json.dumps(rep.get("stop"), indent=2, ensure_ascii=False)
    fail("show needs one of --tag/--check/--run/--measurement/--question/--changes/--deviations/--stop")


def render(rep):
    def cell(s):
        return str(s).replace("|", "\\|").replace("\n", " ")

    stage = f"{rep.get('stage')} " if rep.get("stage") else ""
    out = [f"## {stage}report ({rep.get('worker')}, {rep.get('verdict')})", "", rep.get("summary", ""), ""]
    stop = rep.get("stop")
    if stop:
        out += [f"**STOP.** {stop.get('reason')} (rule: {stop.get('rule')})", ""]
    if rep.get("changes"):
        out += ["| File | Kind | Lines | Why |", "| --- | --- | --- | --- |"]
        out += [f"| `{c.get('path')}` | {c.get('kind')} | {cell(c.get('lines'))} | {cell(c.get('why'))} |" for c in rep["changes"]]
        out.append("")
    for d in rep.get("disclosures", []):
        out.append(f"- Disclosed: `{d.get('path')}`: {d.get('why')}")
    if rep.get("runs"):
        if out[-1] != "":
            out.append("")
        out += ["| Run | Try | Exit | Tests | Log |", "| --- | ---: | ---: | --- | --- |"]
        out += [f"| {cell(r.get('id'))} | {r.get('attempt')} | {r.get('exit')} | {counts(r.get('tests'))} | `{r.get('log')}` |" for r in rep["runs"]]
    for key, shown in (("checks", "actual"), ("measurements", "result")):
        if rep.get(key):
            out += ["", f"| {key.capitalize()[:-1]} | Bar | Result | Verdict | Note |", "| --- | --- | --- | --- | --- |"]
            out += [
                f"| {cell(r.get('id'))} | {cell(r.get('bar'))} | {cell(r.get(shown))} | {r.get('verdict')} | {cell(r.get('note', ''))} |"
                for r in rep[key]
            ]
    if rep.get("deviations"):
        out += ["", "Deviations:"] + [f"- {d.get('what')}: {d.get('why')}" for d in rep["deviations"]]
    if rep.get("questions"):
        out += ["", "Open questions:"]
        out += [f"- **{q.get('id')}.** {q.get('question')} Recommendation: {q.get('recommendation')}" for q in rep["questions"]]
    for s in rep.get("sections", []):
        out += ["", f"### {s.get('title')}", "", s.get("body", "")]
    return "\n".join(out).rstrip() + "\n"


def selftest():
    base = ROOT / ".temp" / "report-selftest"
    base.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=base) as tmp:
        rel = Path(tmp).relative_to(ROOT)
        log = Path(tmp) / "ws.log"
        log.write_text(
            "running 3 tests\ntest result: ok. 3 passed; 0 failed; 1 ignored; 0 measured\n"
            "running 2 tests\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured\nverdict: PASS\n"
        )
        (Path(tmp) / "a.rs").write_text("fn a() {}\n")

        def report(**over):
            r = json.loads(json.dumps(EXAMPLE))
            r.update(verdict="DONE", stop=None, disclosures=[], deviations=[], questions=[], sections=[])
            r["changes"] = [{"path": f"{rel}/a.rs", "kind": "modified", "lines": "1", "why": "x"}]
            r["runs"] = [{"id": "ws", "cmd": "cargo test", "log": f"{rel}/ws.log", "exit": 0, "attempt": 1,
                          "tests": {"passed": 5, "failed": 0, "ignored": 1}, "verdict_lines": ["verdict: PASS"]}]
            r["checks"] = [{"id": "c", "bar": "b", "expected": "1", "actual": "1", "verdict": "PASS", "note": "",
                            "evidence": [{"log": f"{rel}/ws.log", "line": "verdict: PASS"}]}]
            r["measurements"] = []
            r.update(over)
            return r

        cases = [
            ("a consistent report", report(), ""),
            ("a fabricated evidence line", report(checks=[{**report()["checks"][0], "evidence": [{"log": f"{rel}/ws.log", "line": "verdict: GREEN"}]}]), "not found verbatim"),
            ("a missing log", report(runs=[{**report()["runs"][0], "log": f"{rel}/nope.log"}]), "does not exist"),
            ("test counts that disagree with the log", report(runs=[{**report()["runs"][0], "tests": {"passed": 6, "failed": 0, "ignored": 1}}]), "sums to (5, 0, 1)"),
            ("a FAIL in a DONE report", report(checks=[{**report()["checks"][0], "verdict": "FAIL"}]), "FAIL in a DONE report"),
            ("a FLAGGED row with no note", report(checks=[{**report()["checks"][0], "verdict": "FLAGGED"}]), "FLAGGED with no note"),
            ("a STOP with no stop", report(verdict="STOP"), "a STOP report needs"),
            ("a changed file that does not exist", report(changes=[{"path": f"{rel}/b.rs", "kind": "added", "lines": "1", "why": "x"}]), "does not exist"),
            ("a missing list", {k: v for k, v in report().items() if k != "deviations"}, "missing 'deviations'"),
        ]
        bad = 0
        for name, rep, want in cases:
            problems = check(rep)[0]
            ok = not problems if want == "" else len(problems) == 1 and want in problems[0]
            bad += not ok
            print(f"{'ok  ' if ok else 'FAIL'} {name}: {problems[0] if problems else 'no problems'}")
    real = ROOT / ".temp/director/gate-630b3/logs/ws.log"
    if real.exists():
        found = [tuple(map(int, m.groups())) for line in real.read_text(errors="replace").splitlines() for m in [TEST_RESULT.search(line)] if m]
        got = tuple(sum(x[k] for x in found) for k in range(3))
        ok = got == (4013, 0, 61) and len(found) == 59
        bad += not ok
        print(f"{'ok  ' if ok else 'FAIL'} the B3 gate's workspace log sums to {got} over {len(found)} binaries (known: 4013/0/61 over 59)")
    print("selftest:", "PASS" if not bad else f"{bad} FAILED")
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("cmd", choices=["schema", "check", "summary", "show", "render", "selftest"])
    ap.add_argument("target", nargs="?")
    ap.add_argument("--stage")
    ap.add_argument("--against-git", action="store_true")
    for flag in ("tag", "check", "run", "measurement", "question"):
        ap.add_argument(f"--{flag}")
    for flag in ("changes", "deviations", "stop"):
        ap.add_argument(f"--{flag}", action="store_true")
    a = ap.parse_args()
    if a.cmd == "schema":
        print(SCHEMA_TEXT)
        print("Example:")
        print(json.dumps(EXAMPLE, indent=2, ensure_ascii=False))
        return 0
    if a.cmd == "selftest":
        return selftest()
    if not a.target:
        fail(f"{a.cmd} needs a brief number or a report path")
    path = locate(a.target, a.stage)
    rep = load(path)
    if a.cmd == "check":
        problems, cited = check(rep, a.against_git)
        for p in problems:
            print(f"PROBLEM {p}")
        print(f"check: {'OK' if not problems else f'{len(problems)} problems'}, {cited} citations verified ({path.name})")
        return 1 if problems else 0
    if a.cmd == "summary":
        print(summary(rep, path))
    elif a.cmd == "show":
        print(show(rep, a))
    elif a.cmd == "render":
        print(render(rep), end="")
    return 0


if __name__ == "__main__":
    sys.exit(main())
