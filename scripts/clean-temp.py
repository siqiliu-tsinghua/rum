#!/usr/bin/env python3
"""clean-temp.py — post-commit disk cleanup (user ruling 2026-11-07).

Run after every commit and push: delete ./target and prune .temp/.
Default is a dry-run; --apply performs the deletions.

Preconditions enforced before --apply (dry-run only warns):
  * git work tree clean;
  * HEAD == upstream;
  * no scripts/dispatch-brief.sh, cargo or rustc process running.

Deleted:
  * <repo>/target;
  * subdirectories of every .temp/brief-* root older than the newest
    KEEP_BRIEF_ROOTS (root-level reports, binaries and logs are kept);
  * .temp/director/gate-<number>* directories older than the newest
    KEEP_GATE_DIRS distinct numbers.

Never touched: .temp/logs/, .temp/director/state.md and every other
file, .temp/director/gate-state/ and the non-gate directories, and any
unrecognised top-level entry in .temp (reported, not deleted).
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

KEEP_BRIEF_ROOTS = 10
KEEP_GATE_DIRS = 5

ROOT = Path(subprocess.run(
    ["git", "rev-parse", "--show-toplevel"],
    text=True, capture_output=True, check=True,
).stdout.strip())
TEMP = ROOT / ".temp"
TARGET = ROOT / "target"


def human(n: int) -> str:
    value = float(n)
    for unit in ("B", "K", "M", "G", "T"):
        if value < 1024 or unit == "T":
            return f"{value:.0f}{unit}" if unit == "B" else f"{value:.1f}{unit}"
        value /= 1024
    return f"{value:.1f}T"


def size_of(path: Path) -> int:
    try:
        out = subprocess.run(
            ["du", "-s", "--block-size=1", "--", str(path)],
            text=True, capture_output=True, check=True,
        ).stdout
        return int(out.split("\t", 1)[0])
    except (subprocess.CalledProcessError, ValueError):
        return 0


def git(*args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=ROOT, text=True, capture_output=True, check=True,
    ).stdout.strip()


def pids_of(pattern: str, exact: bool = False) -> list[str]:
    flag = "-x" if exact else "-f"
    out = subprocess.run(
        ["pgrep", flag, pattern], text=True, capture_output=True,
    ).stdout
    skip = {str(os.getpid()), str(os.getppid())}
    return [pid for pid in out.split() if pid not in skip]


def preconditions() -> list[str]:
    problems: list[str] = []
    dirty = git("status", "--porcelain")
    if dirty:
        problems.append("work tree not clean:\n" + dirty)
    head = git("rev-parse", "HEAD")
    try:
        upstream = git("rev-parse", "@{u}")
    except subprocess.CalledProcessError:
        problems.append("current branch has no upstream")
    else:
        if head != upstream:
            problems.append(f"HEAD not pushed ({head[:12]} != {upstream[:12]})")
    for label, pattern, exact in (
        ("dispatch", r"scripts/dispatch-brief[.]sh", False),
        ("cargo", "cargo", True),
        ("rustc", "rustc", True),
    ):
        pids = pids_of(pattern, exact)
        if pids:
            problems.append(f"{label} still running: {', '.join(pids)}")
    return problems


def brief_key(path: Path) -> tuple[int, str]:
    match = re.match(r"brief-(\d+)", path.name)
    return (int(match.group(1)) if match else -1, path.name)


def gate_number(name: str) -> int | None:
    match = re.match(r"gate-(\d+)", name)
    return int(match.group(1)) if match else None


def assert_deletable(path: Path) -> None:
    resolved = path.resolve()
    if resolved == TARGET.resolve():
        return
    temp = TEMP.resolve()
    if resolved == temp or not resolved.is_relative_to(temp):
        raise SystemExit(f"refusing to delete outside .temp: {resolved}")


def plan() -> tuple[list[tuple[str, Path, int, str]], list[str]]:
    actions: list[tuple[str, Path, int, str]] = []
    skipped: list[str] = []

    if TARGET.exists():
        actions.append(("delete", TARGET, size_of(TARGET), "workspace target"))

    briefs = sorted(
        (p for p in TEMP.iterdir()
         if p.is_dir() and not p.is_symlink() and p.name.startswith("brief-")),
        key=brief_key,
    )
    for root in briefs[:-KEEP_BRIEF_ROOTS]:
        subs = [c for c in root.iterdir() if c.is_dir() and not c.is_symlink()]
        if subs:
            actions.append(
                ("prune", root, sum(size_of(c) for c in subs),
                 f"{len(subs)} subdir(s), root files kept"),
            )

    director = TEMP / "director"
    gates: dict[int, list[Path]] = {}
    if director.is_dir():
        for entry in director.iterdir():
            if entry.is_dir() and not entry.is_symlink():
                number = gate_number(entry.name)
                if number is not None:
                    gates.setdefault(number, []).append(entry)
    numbers = sorted(gates)
    if len(numbers) > KEEP_GATE_DIRS:
        threshold = numbers[-KEEP_GATE_DIRS]
        for number in numbers:
            if number < threshold:
                for entry in gates[number]:
                    actions.append(("delete", entry, size_of(entry), "old gate dir"))

    known = {"director", "logs"}
    for entry in sorted(TEMP.iterdir()):
        if entry.name in known or entry.name.startswith("brief-"):
            continue
        skipped.append(f".temp/{entry.name} ({human(size_of(entry))})")

    return actions, skipped


def main(argv: list[str]) -> int:
    apply = False
    for arg in argv:
        if arg == "--apply":
            apply = True
        elif arg in ("-h", "--help"):
            print(__doc__)
            return 0
        elif arg != "--dry-run":
            print(f"usage: {sys.argv[0]} [--apply]", file=sys.stderr)
            return 2

    problems = preconditions()
    if problems and apply:
        print("REFUSED:", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        return 1
    if problems:
        print("NOTE (dry-run continues; --apply would refuse):")
        for problem in problems:
            print(f"  - {problem}")

    actions, skipped = plan()
    free_before = shutil.disk_usage(ROOT).free

    print(f"repo: {ROOT}")
    print(f"free before: {human(free_before)}")
    print("plan:")
    total = 0
    for kind, path, size, detail in actions:
        total += size
        print(f"  {kind.upper():6} {path.relative_to(ROOT)} ({human(size)}) — {detail}")
    for note in skipped:
        print(f"  SKIP   {note} — unrecognised, left untouched")
    if not actions:
        print("  (nothing to do)")
    print(f"total to free: {human(total)}")

    if not apply:
        print("dry-run; pass --apply to delete.")
        return 0

    failed = 0
    for kind, path, size, detail in actions:
        assert_deletable(path)
        try:
            shutil.rmtree(path)
            print(f"deleted {path.relative_to(ROOT)} ({human(size)})")
        except OSError as error:
            failed += 1
            print(f"FAILED  {path.relative_to(ROOT)}: {error}", file=sys.stderr)

    free_after = shutil.disk_usage(ROOT).free
    print(f"free after: {human(free_after)} (freed {human(free_after - free_before)})")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
