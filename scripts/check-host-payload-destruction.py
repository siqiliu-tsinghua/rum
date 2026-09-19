#!/usr/bin/env python3
"""Check the declared destruction boundary from independently rooted crates."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from typing import NamedTuple


REPO = Path(__file__).resolve().parent.parent
FIXTURES = REPO / "crates/rum-host/tests/fixtures/host_payload_destruction_contract"


class Case(NamedTuple):
    name: str
    type_name: str
    trait_name: str
    impl_line: str
    declared: bool = False


CASES = (
    Case("legacy_impl", "LegacyEntropy", "EntropySource", "impl EntropySource for LegacyEntropy {"),
    Case("legacy_new", "LegacyEntropy", "EntropySource", "impl EntropySource for LegacyEntropy {"),
    Case("legacy_from_arc", "LegacyEntropy", "EntropySource", "impl EntropySource for LegacyEntropy {"),
    Case("declared_new", "LegacyEntropy", "EntropySource", "impl EntropySource for LegacyEntropy {", True),
    Case("declared_from_arc", "LegacyEntropy", "EntropySource", "impl EntropySource for LegacyEntropy {", True),
    Case("legacy_file_io_impl", "LegacyFileIo", "FileIo", "impl FileIo for LegacyFileIo {"),
    Case("legacy_file_io_new", "LegacyFileIo", "FileIo", "impl FileIo for LegacyFileIo {"),
    Case("legacy_file_io_from_arc", "LegacyFileIo", "FileIo", "impl FileIo for LegacyFileIo {"),
    Case("declared_file_io_new", "LegacyFileIo", "FileIo", "impl FileIo for LegacyFileIo {", True),
    Case("declared_file_io_from_arc", "LegacyFileIo", "FileIo", "impl FileIo for LegacyFileIo {", True),
)
# The 2026-09-19 cold ten-case run took 60.71s, at most 6.31s per check.
# Ten times that per-check maximum, rounded up to a minute, is 120s.
# Full output: docs/dev/codex/report-482-bounded-destruction-file-io.md.
CHECK_TIMEOUT_SECONDS = 120


def missing_declaration(message: dict, source: Path, case: Case) -> bool:
    """Admit only this case's exact missing-bound E0277 at its implementation."""
    return (
        (message.get("code") or {}).get("code") == "E0277"
        and message.get("message")
        == f"the trait bound `{case.type_name}: BoundedDestruction` is not satisfied"
        and any(
            span.get("is_primary")
            and (source.parent.parent / span["file_name"]).resolve() == source.resolve()
            and any(
                line["text"].strip() == case.impl_line
                for line in span.get("text", [])
            )
            for span in message.get("spans", [])
        )
        and any(
            child.get("level") == "note"
            and child.get("message") == f"required by a bound in `{case.trait_name}`"
            for child in message.get("children", [])
        )
    )


def check_case(case: Case, work_dir: Path, baseline: bool, timeout: float) -> bool:
    consumer = work_dir / case.name
    source = consumer / "src/lib.rs"
    source.parent.mkdir(parents=True, exist_ok=True)
    source.write_bytes((FIXTURES / f"{case.name}.rs").read_bytes())
    package = f"host-payload-{case.name.replace('_', '-')}"
    (consumer / "Cargo.toml").write_text(
        f'[package]\nname = "{package}"\nversion = "0.0.0"\nedition = "2024"\n'
        'publish = false\n\n[workspace]\n\n[dependencies]\n'
        f'rum-runtime = {{ path = {json.dumps(str(REPO / "crates/rum-runtime"))} }}\n',
        encoding="utf-8",
    )
    command = [
        os.environ.get("CARGO", "cargo"),
        "check",
        "--offline",
        "--manifest-path",
        str(consumer / "Cargo.toml"),
        "--target-dir",
        str(consumer / "target"),
        "--message-format=json",
        "--color=never",
    ]
    print(f"CASE {case.name}: {' '.join(command)}", flush=True)
    started = time.monotonic()
    result = subprocess.run(
        [sys.executable, str(REPO / "scripts/run-bounded.py"), "--timeout", str(timeout),
         "--", *command],
        cwd=REPO,
        env={**os.environ, "CARGO_NET_OFFLINE": "true"},
        capture_output=True,
        text=True,
        check=False,
    )
    elapsed = time.monotonic() - started
    (consumer / "cargo.stdout.jsonl").write_text(result.stdout, encoding="utf-8")
    (consumer / "cargo.stderr.txt").write_text(result.stderr, encoding="utf-8")
    try:
        messages = [json.loads(line) for line in result.stdout.splitlines()]
    except json.JSONDecodeError:
        print(result.stdout, end="")
        print(result.stderr, end="")
        print(f"FAIL {case.name}: malformed Cargo diagnostics", flush=True)
        return False
    errors = [
        item["message"] for item in messages
        if item.get("reason") == "compiler-message" and item["message"]["level"] == "error"
    ]
    finished = [item["success"] for item in messages if item.get("reason") == "build-finished"]
    expect_success = baseline or case.declared
    if expect_success:
        passed = result.returncode == 0 and not errors and finished == [True]
        detail = "compiled" if passed else "expected a successful compile"
    else:
        passed = (
            result.returncode == 101
            and finished == [False]
            and bool(errors)
            and all(missing_declaration(error, source, case) for error in errors)
        )
        if passed:
            detail = f"rejected only for missing BoundedDestruction ({len(errors)} E0277)"
        elif result.returncode == 0:
            detail = "compiled without BoundedDestruction; boundary is not enforced"
        else:
            detail = "failure was not solely the missing BoundedDestruction declaration"
    print(result.stderr, end="")
    for error in errors:
        print(error.get("rendered") or error["message"], end="\n")
    print(f"{'PASS' if passed else 'FAIL'} {case.name}: {detail} ({elapsed:.2f}s)", flush=True)
    return passed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--legacy-baseline", action="store_true",
        help="require the unchanged legacy fixtures to compile on the pre-change tree",
    )
    parser.add_argument(
        "--case", action="append", choices=[case.name for case in CASES],
        help="select a case (repeatable); defaults to all applicable cases",
    )
    parser.add_argument(
        "--work-dir", type=Path, default=REPO / "target/host-payload-destruction-contract",
        help="generated consumer root, which must be inside this repository's target/",
    )
    parser.add_argument(
        "--timeout", type=float, default=CHECK_TIMEOUT_SECONDS, help="seconds per cargo check",
    )
    args = parser.parse_args()
    work_dir = args.work_dir.resolve()
    target = (REPO / "target").resolve()
    if not work_dir.is_relative_to(target) or work_dir == target:
        parser.error("--work-dir must be a directory beneath repository-local target/")
    if args.timeout <= 0:
        parser.error("--timeout must be positive")
    work_dir.mkdir(parents=True, exist_ok=True)
    temporary = work_dir / "tmp"
    temporary.mkdir(exist_ok=True)
    os.environ["TMPDIR"] = str(temporary)
    started = time.monotonic()
    cases = [
        case for case in CASES
        if (not args.legacy_baseline or not case.declared)
        and (args.case is None or case.name in args.case)
    ]
    if not cases:
        parser.error("no applicable cases selected")
    results = [check_case(case, work_dir, args.legacy_baseline, args.timeout) for case in cases]
    passed = sum(results)
    print(f"{'PASS' if all(results) else 'FAIL'}: {passed}/{len(cases)} cases "
          f"({time.monotonic() - started:.2f}s)", flush=True)
    return 0 if all(results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
