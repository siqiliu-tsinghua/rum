#!/usr/bin/env python3
"""Rum-vs-rum instruction panels: one codegen unit, one source path.

  perf-panel.py arms --work DIR --base-src SRC --cand-src SRC [--only cand] [--no-scope] --json OUT
  perf-panel.py pathcal BIN_A BIN_B
  perf-panel.py structure ARM_A ARM_B [--json OUT]
  perf-panel.py panel ARM_A ARM_B --engine resume|cli [--labels a,b | --all] [--reps 15] --out TSV
  perf-panel.py summary TSV [--top 25] [--json OUT]
  perf-panel.py attr LABEL ARM_A ARM_B --out DIR [--runs 6]
  perf-panel.py callers DIR NEEDLE N
  perf-panel.py fnslope DIR FUNCTION N
  perf-panel.py shape BIN FUNCTION [--callee PATTERN ...]
  perf-panel.py selftest

An arm directory holds `rum-cli` and `resume`. The panels compare two rum
builds and nothing else. Stdlib only.
"""

import argparse
import collections
import hashlib
import json
import os
import re
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LABELS_JSON = ROOT / "tools/perf-panel/labels.json"
LABELS_DIR = ROOT / "tools/perf-panel/labels"
SCOPE = ROOT / "scripts/memory-scope.sh"
BINARIES = ("resume", "rum-cli")

DROP = re.compile(r"drop_in_place\$LT\$rum_core\.\.expr\.\.Expr\$GT\$17h[0-9a-f]{16}")
HDR = re.compile(r"^[0-9a-f]+ <(.+)>:$")
LEAD = re.compile(r"^\s*[0-9a-f]+:\s*")
ABS = re.compile(r"\b[0-9a-f]{5,}\b")
RIP = re.compile(r"-?0x[0-9a-f]+\(%rip\)")
COMMENT = re.compile(r"#\s*A <([^>+]+)(\+0x[0-9a-f]+)?>")
HASH = re.compile(r"17h[0-9a-f]{16}E")
INSTANCE = re.compile(r"#\d+$")
PADDING = re.compile(r"^(?:(?:cs|ds|data16)\s+)*(?:nop[wl]?|xchg\s+%ax,%ax|int3)\b")
HEX = re.compile(r"\$?-?0x[0-9a-f]+")
HEADER = re.compile(
    r"^# perf-panel engine=(\S+) reps=(\d+) binary=(\S+) sha_a=([0-9a-f]+) sha_b=([0-9a-f]+)$"
)
PERF_COUNT = re.compile(r"^(\d+),,cpu_core/instructions/u", re.M)
COST_LINE = re.compile(r"^(?:\*|[+-]?\d+)(?:\s+(?:\*|[+-]?\d+))+$")
PASS_LINE = "PASS {label} resumes={n}"


def load_labels():
    return json.loads(LABELS_JSON.read_text())


def label_index():
    return {row["label"]: row for row in load_labels()}


def clean_env():
    return {k: v for k, v in os.environ.items() if not k.startswith("RUM_PROFILE_")}


def command(row, arm_dir, n):
    """argv and the stdout expectation, the same rule as instr.command."""
    arm_dir = Path(arm_dir)
    if row["engine"] == "resume":
        cmd = [str(arm_dir / "resume"), row["harness_label"], str(n)]
        cmd.extend(row["args"])  # lhs- labels carry cand
        expect = re.compile(
            rf"PASS {re.escape(row['harness_label'])} resumes={n * row['resumes']}"
        )
        return cmd, expect
    cmd = [str(arm_dir / "rum-cli"), "--file", str(LABELS_DIR / f"{row['label']}-{n}.rum")]
    return cmd, ("Null" if row["kind"] == "repeat" else None)


def ok(expect, stdout):
    if expect is None:
        return True
    if isinstance(expect, str):
        return stdout.strip() == expect
    return expect.fullmatch(stdout.strip()) is not None


def sha256_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def binary_for(engine):
    return "resume" if engine == "resume" else "rum-cli"


# ---- pathcal -------------------------------------------------------------------

def hashes_from_nm_text(text):
    """The Expr drop-glue symbol hashes, sorted, duplicates kept (path-calibration.sh)."""
    return sorted(match.split("17h", 1)[1] for match in DROP.findall(text))


def pathcal_equal(hashes_a, hashes_b):
    joined_a = " ".join(hashes_a)
    joined_b = " ".join(hashes_b)
    return bool(joined_a) and joined_a == joined_b


def hashes_of_binary(path):
    out = subprocess.run(["nm", str(path)], capture_output=True, text=True)
    return hashes_from_nm_text(out.stdout)


def pathcal_lines(bin_a, bin_b):
    hashes_a, hashes_b = hashes_of_binary(bin_a), hashes_of_binary(bin_b)
    passed = pathcal_equal(hashes_a, hashes_b)
    lines = [f"a: {' '.join(hashes_a)}", f"b: {' '.join(hashes_b)}", "PASS" if passed else "FAIL"]
    return passed, lines, hashes_a, hashes_b


# ---- disassembly ---------------------------------------------------------------

def mask_hex(line):
    return HEX.sub("K", line)


def funcs_from_text(text):
    """fndiff.py's address normalization, on objdump -d text.

    A name can head several bodies (local copies under one mangled name, C
    statics). Each later one is kept as `name#k`, where fndiff.py kept only the
    last; fndiff3 groups the instances back under the name. The alignment
    padding after a body is dropped: its length follows the next function's
    address, so it moved `mi_bin` between 23 and 25 lines on unchanged code."""
    fs, name, body = {}, None, []

    def close(name, body):
        while body and PADDING.match(body[-1]):
            body.pop()
        fs[name] = body

    for line in text.splitlines():
        m = HDR.match(line)
        if m:
            if name:
                close(name, body)
            name, body = m.group(1), []
            k = 1
            while name in fs:
                k += 1
                name = f"{m.group(1)}#{k}"
        elif name and line.strip():
            normalized = LEAD.sub("", line)
            normalized = RIP.sub("R(%rip)", normalized)
            normalized = ABS.sub("A", normalized)
            normalized = COMMENT.sub(lambda match: "# <" + match.group(1) + ">", normalized)
            body.append(normalized)
    if name:
        close(name, body)
    return fs


def funcs_of_binary(path):
    out = subprocess.run(
        ["objdump", "-d", "--no-show-raw-insn", "-w", str(path)],
        capture_output=True, text=True, check=True,
    )
    return funcs_from_text(out.stdout)


def fndiff_lines(fa, fb):
    only_a = sorted(set(fa) - set(fb))
    only_b = sorted(set(fb) - set(fa))
    diff = sorted(name for name in set(fa) & set(fb) if fa[name] != fb[name])
    lines = [
        f"functions a={len(fa)} b={len(fb)} common={len(set(fa) & set(fb))} "
        f"differ={len(diff)} only_a={len(only_a)} only_b={len(only_b)}"
    ]
    lines += [f"ONLY_A {len(fa[name])}\t{name}" for name in only_a]
    lines += [f"ONLY_B {len(fb[name])}\t{name}" for name in only_b]
    lines += [f"DIFF {len(fa[name])}->{len(fb[name])}\t{name}" for name in diff]
    return lines, only_a, only_b, diff


def grouped_bodies(funcs, mask):
    groups = collections.defaultdict(list)
    for name, body in funcs.items():
        lines = [HASH.sub("", line) for line in body]
        if mask:
            lines = [mask_hex(line) for line in lines]
        groups[HASH.sub("", INSTANCE.sub("", name))].append(tuple(lines))
    return {name: sorted(bodies) for name, bodies in groups.items()}


def fndiff3_lines(fa, fb):
    plain_a, plain_b = grouped_bodies(fa, False), grouped_bodies(fb, False)
    masked_a, masked_b = grouped_bodies(fa, True), grouped_bodies(fb, True)
    diff = sorted(name for name in set(plain_a) & set(plain_b) if plain_a[name] != plain_b[name])
    layout = [name for name in diff if masked_a[name] == masked_b[name]]
    code = [name for name in diff if masked_a[name] != masked_b[name]]
    lines = [f"FNDIFF3 differ={len(diff)} layout_only={len(layout)} code={len(code)}"]
    for name in code:
        lines.append(
            f"CODE {[len(body) for body in plain_a[name]]} -> {[len(body) for body in plain_b[name]]} {name}"
        )
    return lines, diff, layout, code, plain_a, plain_b


def analyze_pair(fa, fb):
    """fndiff's lines stay the raw log. The verdict's names are fndiff3's, without
    symbol hashes: a crate whose metadata moved renames every symbol it has, and
    a renamed function is neither new nor gone."""
    fd_lines, _only_a, _only_b, _diff = fndiff_lines(fa, fb)
    f3_lines, diff, layout, code, plain_a, plain_b = fndiff3_lines(fa, fb)
    only_a = sorted(set(plain_a) - set(plain_b))
    only_b = sorted(set(plain_b) - set(plain_a))
    size_a = {name: sum(len(body) for body in plain_a[name]) for name in only_a}
    size_b = {name: sum(len(body) for body in plain_b[name]) for name in only_b}
    code_rows = [
        {
            "name": name,
            "counts_a": [len(body) for body in plain_a[name]],
            "counts_b": [len(body) for body in plain_b[name]],
        }
        for name in code
    ]
    notes = [line for line in f3_lines if line.startswith("CODE ")]
    notes += [f"ONLY_A {size_a[name]}\t{name}" for name in only_a]
    notes += [f"ONLY_B {size_b[name]}\t{name}" for name in only_b]
    report = {
        "functions_a": len(fa),
        "functions_b": len(fb),
        "differ": len(diff),
        "layout_only": len(layout),
        "code": code_rows,
        "only_a": [{"name": name, "count": size_a[name]} for name in only_a],
        "only_b": [{"name": name, "count": size_b[name]} for name in only_b],
        "notes": notes,
    }
    return fd_lines, f3_lines, report


# ---- slopes, apart, TSV --------------------------------------------------------

def slope(counts_n, counts_2n, n):
    return (statistics.median(counts_2n) - statistics.median(counts_n)) / n


def rep_slopes(counts_n, counts_2n, n):
    return [(b - a) / n for a, b in zip(counts_n, counts_2n)]


def apart_direction(base_n, base_2n, cand_n, cand_2n, n):
    """A tie is not apart: the candidate must sit strictly beyond the base."""
    base = rep_slopes(base_n, base_2n, n)
    cand = rep_slopes(cand_n, cand_2n, n)
    above = min(cand) > max(base)
    below = max(cand) < min(base)
    return above, below, base, cand


def format_header(header):
    return (
        f"# perf-panel engine={header['engine']} reps={header['reps']} "
        f"binary={header['binary']} sha_a={header['sha_a']} sha_b={header['sha_b']}"
    )


def parse_header(line):
    match = HEADER.match(line.strip())
    if not match:
        return None
    return {
        "engine": match.group(1),
        "reps": int(match.group(2)),
        "binary": match.group(3),
        "sha_a": match.group(4),
        "sha_b": match.group(5),
    }


def header_matches(old, new):
    if old is None or new is None:
        return False
    if old["sha_a"] != new["sha_a"] or old["sha_b"] != new["sha_b"]:
        return False
    return (
        old["engine"] == new["engine"]
        and old["reps"] == new["reps"]
        and old["binary"] == new["binary"]
    )


def data_rows(path):
    rows = []
    header = None
    text = Path(path).read_text()
    lines = text.splitlines()
    if lines and lines[0].startswith("#"):
        header = parse_header(lines[0])
        lines = lines[1:]
    for line in lines:
        if line:
            rows.append(line.split("\t"))
    return header, rows


def summary_of(rows, top, header=None):
    pct = [float(row[5]) for row in rows]
    apart, below = [], []
    above_rows, below_rows = [], []
    for row in rows:
        n = int(row[1])
        runs = [list(map(int, row[k].split(","))) for k in (6, 7, 8, 9)]
        above, under, base, cand = apart_direction(runs[0], runs[1], runs[2], runs[3], n)
        item = (row[0], float(row[4]), float(row[5]), min(base), max(base), min(cand), max(cand))
        rendered = "  %s %+.1f/iter %+.4f%% base %.0f-%.0f cand %.0f-%.0f" % item
        if above:
            apart.append(item)
            above_rows.append(rendered)
        if under:
            below.append(row[0])
            below_rows.append(rendered)
    lines = [
        "labels %d median %+.4f%% max %+.4f%% min %+.4f%% above +0.1%%: %d"
        % (len(rows), statistics.median(pct), max(pct), min(pct), sum(p > 0.1 for p in pct)),
        "cand above base in every rep: %d  below in every rep: %d" % (len(apart), len(below)),
    ]
    for item in sorted(apart, key=lambda row: -row[2])[:top]:
        lines.append("  %s %+.1f/iter %+.4f%% base %.0f-%.0f cand %.0f-%.0f" % item)
    seconds = None
    if rows and len(rows[0]) >= 11:
        seconds = sum(float(row[10]) for row in rows)
        lines.append("total seconds: %.3f" % seconds)
    payload = {
        "labels": len(rows),
        "median_pct": "%+.4f%%" % statistics.median(pct),
        "max_pct": "%+.4f%%" % max(pct),
        "min_pct": "%+.4f%%" % min(pct),
        "above_0_1": sum(p > 0.1 for p in pct),
        "apart_above": len(apart),
        "apart_below": len(below),
        "apart_above_rows": above_rows,
        "apart_below_rows": below_rows,
        "seconds": seconds,
        "engine": None if header is None else header["engine"],
        "reps": None if header is None else header["reps"],
    }
    return lines, payload


# ---- measuring -----------------------------------------------------------------

def count_instructions(cmd, expect):
    proc = subprocess.run(
        ["taskset", "-c", "2", "perf", "stat", "-x,", "-e", "cpu_core/instructions/u", *cmd],
        env=clean_env(), stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=900,
    )
    if proc.returncode != 0 or not ok(expect, proc.stdout):
        raise SystemExit(
            f"perf run failed rc={proc.returncode} cmd={cmd} "
            f"stdout={proc.stdout[-300:]!r} stderr={proc.stderr[-300:]!r}"
        )
    match = PERF_COUNT.search(proc.stderr)
    if not match:
        raise SystemExit(f"no instruction count in perf stderr: {proc.stderr[-300:]!r}")
    return int(match.group(1))


def select_rows(engine, labels, take_all):
    rows = [row for row in load_labels() if row["engine"] == engine]
    if take_all and not labels:
        return rows
    if labels and not take_all:
        wanted = labels.split(",")
        by_name = {row["label"]: row for row in rows}
        missing = [name for name in wanted if name not in by_name]
        if missing:
            raise SystemExit(f"unknown {engine} label: {', '.join(missing)}")
        return [by_name[name] for name in wanted]
    raise SystemExit("panel needs --labels a,b or --all")


def prepare_tsv(path, header):
    """Resume only when the header names these arms, this engine and this rep count."""
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists() and path.stat().st_size:
        first = path.read_text().splitlines()[0]
        if header_matches(parse_header(first), header):
            done = set()
            for line in path.read_text().splitlines()[1:]:
                if line:
                    done.add(line.split("\t", 1)[0])
            return done
        aside = Path(f"{path}.{int(path.stat().st_mtime)}")
        suffix = 0
        while aside.exists():
            suffix += 1
            aside = Path(f"{path}.{int(path.stat().st_mtime)}.{suffix}")
        path.rename(aside)
        print(f"moved {path} aside to {aside.name}: header does not match these arms", flush=True)
    path.write_text(format_header(header) + "\n")
    return set()


def format_panel_row(label, n, slope_a, slope_b, runs, secs):
    delta = slope_b - slope_a
    pct = delta / slope_a * 100
    cells = [
        label, str(n), f"{slope_a:.2f}", f"{slope_b:.2f}", f"{delta:+.2f}", f"{pct:+.4f}",
    ]
    for key in (("a", n), ("a", 2 * n), ("b", n), ("b", 2 * n)):
        cells.append(",".join(str(value) for value in runs[key]))
    cells.append(f"{secs:.3f}")
    return "\t".join(cells), delta, pct


# ---- callgrind -----------------------------------------------------------------

def exclusive(path):
    fn_names, ob_names = {}, {}
    cur_ob, cur = "?", None
    pending = False
    costs = collections.Counter()
    summary = None
    for line in Path(path).read_text().splitlines():
        if line.startswith("summary:"):
            summary = int(line.split()[1])
            continue
        if line.startswith("ob=") or line.startswith("cob="):
            rest = line.split("=", 1)[1].strip()
            ident, _, name = rest.partition(")")
            ident = ident[1:] if ident.startswith("(") else ident
            name = name.strip()
            if name:
                ob_names[ident] = name
            if line.startswith("ob="):
                cur_ob = ob_names.get(ident, ident)
            continue
        if line.startswith("fn=") or line.startswith("cfn="):
            rest = line.split("=", 1)[1].strip()
            ident, _, name = rest.partition(")")
            ident = ident[1:] if ident.startswith("(") else ident
            name = name.strip()
            if name:
                fn_names[ident] = name
            if line.startswith("fn="):
                obj = cur_ob.rsplit("/", 1)[-1]
                cur = f"{obj}!{fn_names.get(ident, ident)}"
                pending = False
            continue
        if line.startswith("calls="):
            pending = True
            continue
        if COST_LINE.match(line):
            ir = int(line.split()[-1])
            if pending:
                pending = False
            else:
                costs[cur] += ir
    if summary is None or sum(costs.values()) != summary:
        raise SystemExit(f"exclusive sum mismatch in {path}: {sum(costs.values())} vs {summary}")
    return summary, costs


def ensure_debuginfod(dest):
    dest = Path(dest)
    if dest.exists() and any(dest.iterdir()):
        return "present"
    home = Path.home() / ".cache" / "debuginfod_client"
    try:
        if not home.is_dir():
            raise FileNotFoundError(home)
        shutil.copytree(home, dest, dirs_exist_ok=True)
        return f"copied {home}"
    except (PermissionError, FileNotFoundError) as exc:
        # A sandboxed worker cannot read the home cache; brief-629 kept a copy.
        fallback = ROOT / ".temp/brief-629/debuginfod"
        if not fallback.is_dir():
            raise SystemExit(f"debuginfod cache unreadable ({exc}) and {fallback} is absent")
        shutil.copytree(fallback, dest, dirs_exist_ok=True)
        return f"home cache failed ({exc}); copied {fallback}"


def attr_env(cache):
    env = clean_env()
    env.update(
        # The debuginfod client ignores a relative cache path and leaves libc
        # locals (memcmp) as addresses. The reference passes an absolute path.
        DEBUGINFOD_CACHE_PATH=str(Path(cache).resolve()),
        # An empty DEBUGINFOD_URLS skips the cache. A closed localhost URL makes
        # the client read the copied cache and fail fast on a miss. No network.
        DEBUGINFOD_URLS="http://127.0.0.1:9",
        DEBUGINFOD_TIMEOUT="1",
    )
    return env


def median_costs(samples):
    keys = set().union(*(sample.keys() for sample in samples))
    return {key: statistics.median([sample[key] for sample in samples]) for key in keys}


def parse_calls(path, needle):
    names = {}

    def resolve(token):
        match = re.match(r"\((\d+)\)(?: (.*))?", token)
        if not match:
            return token
        ident, name = match.group(1), match.group(2)
        if name is not None:
            names[ident] = name
        return names.get(ident, "?" + ident)

    cur = callee = None
    out = collections.defaultdict(int)
    for line in Path(path).read_text().splitlines():
        if line.startswith("fn="):
            cur = resolve(line[3:].strip())
            callee = None
        elif line.startswith("cfn="):
            callee = resolve(line[4:].strip())
        elif line.startswith("calls=") and callee is not None:
            if needle in callee:
                out[cur] += int(line.split("=")[1].split()[0])
            callee = None
    return out


def parse_self_ir(path, target):
    names = {}

    def resolve(token):
        match = re.match(r"\((\d+)\)(?: (.*))?", token)
        if not match:
            return token
        if match.group(2) is not None:
            names[match.group(1)] = match.group(2)
        return names.get(match.group(1), "?")

    cur, pos, skip = None, 0, False
    out = collections.defaultdict(int)
    for line in Path(path).read_text().splitlines():
        if line.startswith("fn="):
            cur = resolve(line[3:])
            continue
        if line.startswith("cfn="):
            resolve(line[4:])
            continue
        if line.startswith(("cfi=", "cob=", "fl=", "fi=", "fe=", "ob=")):
            continue
        if line.startswith("calls="):
            skip = True
            continue
        if not line or line[0] not in "+-*0":
            continue
        toks = line.split()
        head = toks[0]
        if head.startswith("0x"):
            pos = int(head, 16)
        elif head.startswith("+"):
            pos += int(head[1:], 16) if head[1:].startswith("0x") else int(head[1:])
        elif head.startswith("-"):
            pos -= int(head[1:], 16) if head[1:].startswith("0x") else int(head[1:])
        elif head == "*":
            pass
        else:
            try:
                pos = int(head)
            except ValueError:
                continue
        if skip:
            skip = False
            continue
        if cur == target and len(toks) > 2:
            out[pos] += int(toks[2])
    return out


def instruction_slope_lines(path_n, path_2n, function, n):
    before, after = parse_self_ir(path_n, function), parse_self_ir(path_2n, function)
    slopes = {key: (after.get(key, 0) - before.get(key, 0)) / n for key in set(before) | set(after)}
    lines = [f"# total self slope {sum(slopes.values()):.3f}"]
    for key in sorted(slopes):
        if abs(slopes[key]) > 1e-9:
            lines.append(f"{key:x}\t{slopes[key]:.3f}")
    return lines


# ---- shape ---------------------------------------------------------------------

def count_matching_lines(pattern, text):
    return sum(1 for line in text.splitlines() if re.search(pattern, line))


def shape_line(binary, function, callees):
    binary = str(binary)
    nm_sized = subprocess.run(["nm", "-S", binary], capture_output=True, text=True).stdout
    addr = size = None
    for line in nm_sized.splitlines():
        if re.search(function, line):
            addr, size = line.split()[:2]
            break
    if addr is None:
        raise SystemExit(f"symbol not found: {function}")
    start = int(addr, 16)
    stop = start + int(size, 16)
    dump = subprocess.run(
        [
            "objdump", "-d", "--no-show-raw-insn",
            f"--start-address=0x{addr}", f"--stop-address=0x{stop:x}", binary,
        ],
        capture_output=True, text=True, check=True,
    ).stdout
    frame_match = re.search(r"sub +\$0x[0-9a-f]+,%rsp", dump)
    frame = re.search(r"0x[0-9a-f]+", frame_match.group(0)).group(0) if frame_match else ""
    relocations = subprocess.run(["readelf", "-rW", binary], capture_output=True, text=True).stdout
    got = []
    for line in relocations.splitlines():
        parts = line.split()
        if len(parts) >= 4 and parts[2] == "R_X86_64_RELATIVE":
            got.append((parts[0], parts[3]))
    nm_plain = subprocess.run(["nm", binary], capture_output=True, text=True).stdout

    def sites(pattern):
        target = None
        for line in nm_plain.splitlines():
            if re.search(pattern, line):
                target = line.split()[0]
                break
        if not target:
            return 0
        symbol = format(int(target, 16), "x")
        slots = [
            format(int(offset, 16), "x")
            for offset, value in got
            if format(int(value, 16), "x") == symbol
        ]
        found = count_matching_lines(rf"call +{symbol} <", dump)
        for slot in slots:
            found += count_matching_lines(rf"# {slot} <", dump)
        return found

    parts = [
        f"size={int(size, 16)}",
        f"frame={frame}",
        f"memcpy_sites={count_matching_lines(r'call.*memcpy', dump)}",
    ]
    parts += [f"calls[{pattern}]={sites(pattern)}" for pattern in callees]
    return " ".join(parts)


# ---- arms ----------------------------------------------------------------------

def run_logged(cmd, log_path, cwd, env):
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with log_path.open("w") as sink:
        sink.write("$ " + " ".join(cmd) + "\n")
        sink.flush()
        start = time.monotonic()
        rc = subprocess.run(
            cmd, cwd=cwd, env=env, stdout=sink, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL,
        ).returncode
        seconds = time.monotonic() - start
    return rc, seconds


def rsync_tree(src, dest, log_path):
    dest.mkdir(parents=True, exist_ok=True)
    cmd = [
        "rsync", "-rlpD", "--checksum", "--delete",
        "--exclude", "/target/", "--exclude", "/.temp/", "--exclude", "/.git",
        str(src).rstrip("/") + "/", str(dest).rstrip("/") + "/",
    ]
    return run_logged(cmd, log_path, ROOT, os.environ.copy())


def build_arm(work, arm, no_scope, logs):
    """Build rum-cli and the resume harness at work/same/src. No -t on the copy."""
    src = work / "same" / "src"
    env = os.environ.copy()
    env.update(
        CARGO_PROFILE_RELEASE_CODEGEN_UNITS="1",
        CARGO_BUILD_JOBS="8",
        CARGO_NET_OFFLINE="true",
    )
    builds = {}
    cli_env = dict(env, CARGO_TARGET_DIR=str(work / "same" / f"target-{arm}"))
    cli_cmd = ["cargo", "build", "--release", "-p", "rum-cli", "--offline"]
    if not no_scope:
        cli_cmd = [str(SCOPE), f"perfpanel-{arm}-cli-{os.getpid()}", "14G", *cli_cmd]
    rc, seconds = run_logged(cli_cmd, logs / f"build-{arm}-cli.log", src, cli_env)
    builds["rum-cli"] = {"rc": rc, "seconds": round(seconds, 3)}
    if rc != 0:
        builds["resume"] = {"rc": None, "seconds": 0}
        return builds
    harness_env = dict(env, CARGO_TARGET_DIR=str(work / "same" / f"target-{arm}-harness"))
    manifest = src / "tools/perf-panel/harness/Cargo.toml"
    harness_cmd = ["cargo", "build", "--release", "--offline", "--manifest-path", str(manifest)]
    if not no_scope:
        harness_cmd = [str(SCOPE), f"perfpanel-{arm}-harness-{os.getpid()}", "14G", *harness_cmd]
    rc, seconds = run_logged(harness_cmd, logs / f"build-{arm}-harness.log", src, harness_env)
    builds["resume"] = {"rc": rc, "seconds": round(seconds, 3)}
    if rc != 0:
        return builds
    dest = work / "arms" / arm
    dest.mkdir(parents=True, exist_ok=True)
    shutil.copy2(work / "same" / f"target-{arm}" / "release" / "rum-cli", dest / "rum-cli")
    shutil.copy2(work / "same" / f"target-{arm}-harness" / "release" / "resume", dest / "resume")
    return builds


def cmd_arms(args):
    work = Path(args.work)
    if not work.is_absolute():
        work = (Path.cwd() / work).resolve()
    logs = work / "logs"
    logs.mkdir(parents=True, exist_ok=True)
    cand_src = Path(args.cand_src).resolve()
    harness = cand_src / "tools/perf-panel/harness"
    if not (harness / "Cargo.toml").is_file():
        raise SystemExit(f"candidate has no harness at {harness}")
    arms = ("cand",) if args.only == "cand" else ("base", "cand")
    sources = {}
    if "base" in arms:
        if not args.base_src:
            raise SystemExit("arms needs --base-src unless --only cand")
        sources["base"] = Path(args.base_src).resolve()
    sources["cand"] = cand_src
    receipt = {"arms": {}, "pathcal": {}}
    ok_all = True
    for arm in arms:
        src = sources[arm]
        for name in BINARIES:  # a failed build must not leave the last gate's binary behind
            (work / "arms" / arm / name).unlink(missing_ok=True)
        rc, _seconds = rsync_tree(src, work / "same" / "src", logs / f"rsync-{arm}.log")
        overlay, _seconds = rsync_tree(harness, work / "same" / "src" / "tools/perf-panel/harness", logs / f"rsync-{arm}-harness.log")
        diff = subprocess.run(
            ["diff", "-rq", str(work / "same" / "src" / "crates"), str(src / "crates")],
            capture_output=True, text=True,
        )
        (logs / f"diff-{arm}.log").write_text(diff.stdout + diff.stderr)
        equal = rc == 0 and overlay == 0 and diff.returncode == 0
        info = {"source": str(src), "source_equal": equal, "builds": {}, "sha256": {}}
        if not equal:
            ok_all = False
            info["builds"] = {
                "rum-cli": {"rc": None, "seconds": 0},
                "resume": {"rc": None, "seconds": 0},
            }
            receipt["arms"][arm] = info
            continue
        info["builds"] = build_arm(work, arm, args.no_scope, logs)
        if any(build["rc"] != 0 for build in info["builds"].values()):
            ok_all = False
        else:
            for name in BINARIES:
                info["sha256"][name] = sha256_file(work / "arms" / arm / name)
        receipt["arms"][arm] = info
    base_dir, cand_dir = work / "arms" / "base", work / "arms" / "cand"
    if all((base_dir / name).is_file() and (cand_dir / name).is_file() for name in BINARIES):
        for name in BINARIES:
            passed, lines, hashes_a, hashes_b = pathcal_lines(base_dir / name, cand_dir / name)
            (logs / f"pathcal-{name}.log").write_text("\n".join(lines) + "\n")
            receipt["pathcal"][name] = {
                "pass": passed,
                "a": " ".join(hashes_a),
                "b": " ".join(hashes_b),
            }
            if not passed:
                ok_all = False
            print("\n".join(lines), flush=True)
    else:
        ok_all = False
    Path(args.json).write_text(json.dumps(receipt, indent=2) + "\n")
    print(f"wrote {args.json}", flush=True)
    return 0 if ok_all else 1


def cmd_pathcal(args):
    passed, lines, _a, _b = pathcal_lines(args.bin_a, args.bin_b)
    print("\n".join(lines))
    return 0 if passed else 1


def cmd_structure(args):
    report = {"binaries": {}}
    for name in BINARIES:
        print(f"===== fndiff {name} =====")
        fa, fb = funcs_of_binary(Path(args.arm_a) / name), funcs_of_binary(Path(args.arm_b) / name)
        fd_lines, f3_lines, one = analyze_pair(fa, fb)
        print("\n".join(fd_lines))
        print(f"===== fndiff3 {name} =====")
        print("\n".join(f3_lines))
        print(
            f"STRUCTURE {name} differ={one['differ']} layout_only={one['layout_only']} "
            f"code={len(one['code'])} only_a={len(one['only_a'])} only_b={len(one['only_b'])}"
        )
        report["binaries"][name] = one
    if args.json:
        Path(args.json).write_text(json.dumps(report, indent=2) + "\n")
    return 0


def cmd_panel(args):
    rows = select_rows(args.engine, args.labels, args.all)
    binary = binary_for(args.engine)
    header = {
        "engine": args.engine,
        "reps": args.reps,
        "binary": binary,
        "sha_a": sha256_file(Path(args.arm_a) / binary),
        "sha_b": sha256_file(Path(args.arm_b) / binary),
    }
    done = prepare_tsv(args.out, header)
    arms = (("a", Path(args.arm_a)), ("b", Path(args.arm_b)))
    for row in rows:
        if row["label"] in done:
            continue
        n = row["N"]
        runs = {(arm, size): [] for arm, _dir in arms for size in (n, 2 * n)}
        start = time.monotonic()
        for _rep in range(args.reps):
            for size in (n, 2 * n):
                for arm, directory in arms:
                    argv, expect = command(row, directory, size)
                    runs[(arm, size)].append(count_instructions(argv, expect))
        slope_a = slope(runs[("a", n)], runs[("a", 2 * n)], n)
        slope_b = slope(runs[("b", n)], runs[("b", 2 * n)], n)
        secs = time.monotonic() - start
        line, delta, pct = format_panel_row(row["label"], n, slope_a, slope_b, runs, secs)
        with open(args.out, "a") as fh:
            fh.write(line + "\n")
            fh.flush()
        print(row["label"], f"{delta:+.2f}", f"{pct:+.4f}%", f"{secs:.0f}s", flush=True)
    return 0


def cmd_summary(args):
    header, rows = data_rows(args.tsv)
    if not rows:
        raise SystemExit(f"no rows in {args.tsv}")
    lines, payload = summary_of(rows, args.top, header)
    print("\n".join(lines))
    if args.json:
        Path(args.json).write_text(json.dumps(payload, indent=2) + "\n")
    return 0


def cmd_attr(args):
    row = label_index()[args.label]
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    note = ensure_debuginfod(out / "debuginfod")
    print(f"debuginfod: {note}", flush=True)
    binary = binary_for(row["engine"])
    sha_a = sha256_file(Path(args.arm_a) / binary)
    sha_b = sha256_file(Path(args.arm_b) / binary)
    cache = out / args.label
    stamp = f"{binary}\t{sha_a}\t{sha_b}\n"
    if cache.exists():
        current = (cache / "arms.sha256").read_text() if (cache / "arms.sha256").is_file() else ""
        if current != stamp:
            aside = Path(f"{cache}.{int(cache.stat().st_mtime)}")
            suffix = 0
            while aside.exists():
                suffix += 1
                aside = Path(f"{cache}.{int(cache.stat().st_mtime)}.{suffix}")
            cache.rename(aside)
            print(f"moved {cache.name} aside to {aside.name}: arm sha256 changed", flush=True)
    cache.mkdir(parents=True, exist_ok=True)
    (cache / "arms.sha256").write_text(stamp)
    env = attr_env(out / "debuginfod")
    arms = {"base": Path(args.arm_a), "cand": Path(args.arm_b)}
    n = row["N"]
    slopes, totals = {}, {}
    for arm, directory in arms.items():
        by_size, total_by_size = {}, {}
        for size in (n, 2 * n):
            samples = []
            for rep in range(1, args.runs + 1):
                stem = cache / f"{arm}-{size}-{rep}"
                out_file = stem.with_suffix(".out")
                argv, expect = command(row, directory, size)
                if not out_file.exists():
                    proc = subprocess.run(
                        ["valgrind", "--tool=callgrind", "--vgdb=no", f"--callgrind-out-file={out_file}", *argv],
                        env=env, stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=900,
                    )
                    stem.with_suffix(".stdout").write_text(proc.stdout)
                    stem.with_suffix(".stderr").write_text(proc.stderr)
                    if proc.returncode != 0 or not ok(expect, proc.stdout):
                        raise SystemExit(
                            f"callgrind failed {arm} {row['label']} N={size} rep={rep} rc={proc.returncode} "
                            f"stdout={proc.stdout[-200:]!r} stderr={proc.stderr[-300:]!r}"
                        )
                elif not ok(expect, stem.with_suffix(".stdout").read_text()):
                    raise SystemExit(f"stale bad stdout {stem}")
                _summary, costs = exclusive(out_file)
                samples.append(costs)
            by_size[size] = median_costs(samples)
            total_by_size[size] = [sum(sample.values()) for sample in samples]
            print(f"{args.label} {arm} N={size} totals={total_by_size[size]}", flush=True)
        slopes[arm] = {}
        for key in set(by_size[n]) | set(by_size[2 * n]):
            slopes[arm][key] = (by_size[2 * n].get(key, 0) - by_size[n].get(key, 0)) / n
        totals[arm] = (statistics.median(total_by_size[2 * n]) - statistics.median(total_by_size[n])) / n
    rows = []
    for key in set(slopes["base"]) | set(slopes["cand"]):
        base, cand = slopes["base"].get(key, 0.0), slopes["cand"].get(key, 0.0)
        if base or cand:
            rows.append((cand - base, base, cand, key))
    rows.sort(key=lambda item: -abs(item[0]))
    dest = out / f"attr-{args.label}.tsv"
    with dest.open("w") as fh:
        fh.write("delta\tbase_slope\tcand_slope\tfunction\n")
        fh.write(f"{totals['cand'] - totals['base']:+.4f}\t{totals['base']:.4f}\t{totals['cand']:.4f}\tTOTAL\n")
        for delta, base, cand, key in rows:
            if abs(delta) < 0.05 and abs(base) < 0.05 and abs(cand) < 0.05:
                continue
            fh.write(f"{delta:+.4f}\t{base:.4f}\t{cand:.4f}\t{key}\n")
    print(
        f"TOTAL base={totals['base']:.2f} cand={totals['cand']:.2f} "
        f"delta={totals['cand'] - totals['base']:+.2f}",
        flush=True,
    )
    print(f"wrote {dest}", flush=True)
    return 0


def cmd_callers(args):
    directory, needle, n = args.directory, args.needle, args.n
    res = {}
    for arm in ("base", "cand"):
        per = {}
        for size in (n, 2 * n):
            paths = sorted(Path(directory).glob(f"{arm}-{size}-*.out"))
            runs = [parse_calls(path, needle) for path in paths]
            keys = set().union(*runs) if runs else set()
            per[size] = {key: statistics.median([run.get(key, 0) for run in runs]) for key in keys}
        keys = set(per[n]) | set(per[2 * n])
        res[arm] = {key: (per[2 * n].get(key, 0) - per[n].get(key, 0)) / n for key in keys}
    keys = set(res["base"]) | set(res["cand"])
    ordered = sorted(keys, key=lambda key: -abs(res["cand"].get(key, 0) - res["base"].get(key, 0)))
    print("delta\tbase\tcand\tcaller")
    for key in ordered:
        base, cand = res["base"].get(key, 0), res["cand"].get(key, 0)
        if base or cand:
            print(f"{cand - base:+.3f}\t{base:.3f}\t{cand:.3f}\t{key}")
    return 0


def cmd_fnslope(args):
    directory = Path(args.directory)
    found = False
    for arm in ("base", "cand"):
        rep = 1
        while True:
            path_n = directory / f"{arm}-{args.n}-{rep}.out"
            path_2n = directory / f"{arm}-{2 * args.n}-{rep}.out"
            if not path_n.is_file() or not path_2n.is_file():
                break
            found = True
            print(f"# arm={arm} rep={rep}")
            print("\n".join(instruction_slope_lines(path_n, path_2n, args.function, args.n)))
            rep += 1
    if not found:
        raise SystemExit(f"no callgrind pair in {directory} at N={args.n}")
    return 0


def cmd_shape(args):
    print(shape_line(args.binary, args.function, args.callee or []))
    return 0


# ---- selftest ------------------------------------------------------------------

def selftest():
    fails = []

    def expect(name, got, want):
        if got != want:
            fails.append(f"{name}: got {got!r}, want {want!r}")

    expect("slope arithmetic", slope([10, 30, 20], [40, 60, 50], 10), 3.0)
    expect(
        "apart tie",
        apart_direction([0, 0], [1, 3], [0, 0], [3, 4], 1)[:2],
        (False, False),
    )
    expect("apart above", apart_direction([0, 0], [1, 2], [0, 0], [3, 4], 1)[0], True)
    expect("apart below", apart_direction([0, 0], [5, 6], [0, 0], [1, 2], 1)[1], True)

    header = {
        "engine": "resume", "reps": 15, "binary": "resume",
        "sha_a": "ab" * 32, "sha_b": "cd" * 32,
    }
    expect("header round trip", parse_header(format_header(header)), header)
    other = dict(header, sha_a="ef" * 32)
    expect("re-entry sha", header_matches(header, other), False)
    expect("re-entry same", header_matches(header, dict(header)), True)
    expect("re-entry reps", header_matches(header, dict(header, reps=3)), False)

    labels = label_index()
    resume_cmd, resume_expect = command(labels["resume-table"], Path("/arm"), 1000)
    expect("resume argv", resume_cmd[1:], ["table", "1000"])
    expect("resume expect", resume_expect.pattern, r"PASS table resumes=1000")
    lhs_cmd, lhs_expect = command(labels["lhs-arg-cycle"], Path("/arm"), 100)
    expect("lhs- args", lhs_cmd[1:], ["lhs-arg-cycle", "100", "cand"])
    expect("lhs- expect", lhs_expect.pattern, r"PASS lhs\-arg\-cycle resumes=100")
    twice_cmd, twice_expect = command(labels["resume-argument-twice"], Path("/arm"), 1000)
    expect("two-prompt argv", twice_cmd[1:], ["argument-twice", "1000"])
    expect("two-prompt expect", twice_expect.pattern, r"PASS argument\-twice resumes=2000")
    repeat_cmd, repeat_expect = command(labels["lazy-range-take-small"], Path("/arm"), 20000)
    expect(
        "repeat argv",
        repeat_cmd[1:],
        ["--file", str(LABELS_DIR / "lazy-range-take-small-20000.rum")],
    )
    expect("repeat expect", repeat_expect, "Null")
    _once_cmd, once_expect = command(labels["do-count-null"], Path("/arm"), 20000)
    expect("once expect", once_expect, None)
    _inc_cmd, inc_expect = command(labels["appendto-list-incremental-small"], Path("/arm"), 250)
    expect("incremental expect", inc_expect, None)
    expect("ok null", ok("Null", "Null\n"), True)
    expect("ok none", ok(None, "anything"), True)
    expect("ok pass", ok(resume_expect, "PASS table resumes=1000\n"), True)

    text_a = (
        "0000000000401000 <alpha17h0123456789abcdefE>:\n"
        "  401000:\tmov    $0x10,%eax\n"
        "  401005:\tlea    -0x70(%rip),%rax\n"
        "  401008:\t# 401234 <other+0x4>\n"
        "  40100a:\tcall   401200 <other>\n"
        "\n"
        "0000000000401200 <beta>:\n"
        "  401200:\tret\n"
    )
    text_b = text_a.replace("$0x10", "$0x20")
    text_c = text_a.replace("mov    $0x10", "add    $0x10")
    fa, fb, fc = funcs_from_text(text_a), funcs_from_text(text_b), funcs_from_text(text_c)
    body = fa["alpha17h0123456789abcdefE"]
    expect("normalize rip", any("R(%rip)" in line for line in body), True)
    expect("normalize comment", any("# <other>" in line for line in body), True)
    _fd, _f3, layout = analyze_pair(fa, fb)
    expect("layout-only", (layout["layout_only"], len(layout["code"])), (1, 0))
    _fd, _f3, code = analyze_pair(fa, fc)
    expect("code split", (code["layout_only"], len(code["code"])), (0, 1))
    twin = "0000000000402000 <beta>:\n  402000:\tnop\n  402001:\tret\n"
    fd, fe = funcs_from_text(text_a + twin), funcs_from_text(text_a + twin.replace("nop", "int3"))
    expect("duplicate kept", sorted(fd), ["alpha17h0123456789abcdefE", "beta", "beta#2"])
    _fd, _f3, dup = analyze_pair(fd, fe)
    expect("duplicate instance differs", [row["name"] for row in dup["code"]], ["beta"])
    _fd, _f3, gone = analyze_pair(fd, funcs_from_text(text_a))
    expect("duplicate instance gone", ([row["name"] for row in gone["code"]], gone["only_a"]), (["beta"], []))
    _fd, _f3, renamed = analyze_pair(fa, funcs_from_text(text_a.replace("0123456789abcdef", "fedcba9876543210")))
    expect("hash rename", (renamed["differ"], renamed["only_a"], renamed["only_b"]), (0, [], []))
    _fd, _f3, new = analyze_pair(fa, funcs_from_text(text_a + twin.replace("<beta>", "<gamma17h0000000000000000E>")))
    expect("new function", new["only_b"], [{"name": "gamma", "count": 2}])
    padded = text_a.replace("  401200:\tret\n", "  401200:\tret\n  401201:\tcs nopw 0x0(%rax,%rax,1)\n"
                            "  40120b:\tnopl   0x0(%rax,%rax,1)\n")
    fp = funcs_from_text(padded)
    expect("padding dropped", fp["beta"], ["ret"])
    _fd, _f3, pad = analyze_pair(fa, fp)
    expect("padding is not code", (pad["differ"], len(pad["code"])), (0, 0))
    inner = text_a.replace("  401005:", "  401004:\tnop\n  401005:")
    expect("inner nop kept", len(funcs_from_text(inner)["alpha17h0123456789abcdefE"]), 5)

    nm_text = (
        "000000 t foo_drop_in_place$LT$rum_core..expr..Expr$GT$17h0123456789abcdefE\n"
        "000000 t foo_drop_in_place$LT$rum_core..expr..Expr$GT$17habcdefabcdefabcdE\n"
        "000000 t other17hffffffffffffffffE\n"
    )
    extracted = hashes_from_nm_text(nm_text)
    expect("pathcal extract", extracted, ["0123456789abcdef", "abcdefabcdefabcd"])
    expect("pathcal sets", pathcal_equal(["0123456789abcdef", "abcdefabcdefabcd"], ["0123456789abcdef", "ffffffffffffffff"]), False)
    expect("pathcal equal", pathcal_equal(extracted, list(extracted)), True)
    expect("pathcal empty", pathcal_equal([], []), False)

    for failure in fails:
        print(f"SELFTEST FAIL {failure}")
    print(f"selftest: {'PASS' if not fails else f'{len(fails)} failures'}")
    return 1 if fails else 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="cmd", required=True)

    arms = sub.add_parser("arms")
    arms.add_argument("--work", required=True)
    arms.add_argument("--base-src")
    arms.add_argument("--cand-src", required=True)
    arms.add_argument("--only", choices=("cand",))
    arms.add_argument("--no-scope", action="store_true")
    arms.add_argument("--json", required=True)
    arms.set_defaults(func=cmd_arms)

    pathcal = sub.add_parser("pathcal")
    pathcal.add_argument("bin_a")
    pathcal.add_argument("bin_b")
    pathcal.set_defaults(func=cmd_pathcal)

    structure = sub.add_parser("structure")
    structure.add_argument("arm_a")
    structure.add_argument("arm_b")
    structure.add_argument("--json")
    structure.set_defaults(func=cmd_structure)

    panel = sub.add_parser("panel")
    panel.add_argument("arm_a")
    panel.add_argument("arm_b")
    panel.add_argument("--engine", required=True, choices=("resume", "cli"))
    panel.add_argument("--labels", default="")
    panel.add_argument("--all", action="store_true")
    panel.add_argument("--reps", type=int, default=15)
    panel.add_argument("--out", required=True)
    panel.set_defaults(func=cmd_panel)

    summary = sub.add_parser("summary")
    summary.add_argument("tsv")
    summary.add_argument("--top", type=int, default=25)
    summary.add_argument("--json")
    summary.set_defaults(func=cmd_summary)

    attr = sub.add_parser("attr")
    attr.add_argument("label")
    attr.add_argument("arm_a")
    attr.add_argument("arm_b")
    attr.add_argument("--out", required=True)
    attr.add_argument("--runs", type=int, default=6)
    attr.set_defaults(func=cmd_attr)

    callers = sub.add_parser("callers")
    callers.add_argument("directory")
    callers.add_argument("needle")
    callers.add_argument("n", type=int)
    callers.set_defaults(func=cmd_callers)

    fnslope = sub.add_parser("fnslope")
    fnslope.add_argument("directory")
    fnslope.add_argument("function")
    fnslope.add_argument("n", type=int)
    fnslope.set_defaults(func=cmd_fnslope)

    shape = sub.add_parser("shape")
    shape.add_argument("binary")
    shape.add_argument("function")
    shape.add_argument("--callee", action="append")
    shape.set_defaults(func=cmd_shape)

    test = sub.add_parser("selftest")
    test.set_defaults(func=lambda _args: selftest())

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
