#!/usr/bin/env python3
"""probe-build.py — content-addressed probe builds (user ruling 2026-11-07).

The probe arm builds are one rustc process each on one core; the same
bytes were recompiled every round. This script builds one assembled tree
at one stable absolute path and reuses the binary across arms, builds and
briefs when the assembled tree, the Rust toolchain and the flags are
identical. Measurements are never reused: each round re-runs them, and one
arm per round is rebuilt with --force as a sha256 spot-check.

Usage:
  scripts/probe-build.py --tree SRC --out BIN [--force] [--timeout 880]
                         [--receipt PATH]

The caller assembles the complete source tree (workspace plus probe
inputs) in SRC. The script copies it to <repo>/.temp/probe/active — the
stable build path every round — hashes the tree, and:

  * cache hit (same tree, rustc and flags): copies the cached binary to
    BIN and prints a receipt with cache_hit=true. No cargo runs.
  * cache miss: cleans only the crates whose source changed (rum-runtime
    always changes in an arm), then runs
    `cargo test --offline --locked -j 8 -p rum-runtime --message-format=json
     --lib --no-run --release` in the stable root, stores the executable
    and a receipt under .temp/probe/cache/<key>/, and copies it to BIN.
  * --force: rebuilds even on a hit and compares the sha256 with the
    cached value; a mismatch is exit 3 (determinism failure).

A cargo failure or timeout is exit 1; bad arguments exit 2; a concurrent
build exits 4. The script never runs git and never touches the live tree.
"""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PROBE = ROOT / ".temp" / "probe"
ACTIVE = PROBE / "active"
TARGET = PROBE / "target"
CACHE = PROBE / "cache"
LOGS = PROBE / "logs"
TMP = PROBE / "tmp"
LOCK = PROBE / "lock"
STATE = PROBE / "state.json"

EXCLUDES = {".git", "target", ".temp", "__pycache__", ".mypy_cache"}
TARGET_NAME = "rum_runtime"
CARGO = [
    "cargo", "test", "--offline", "--locked", "-j", "8",
    "-p", "rum-runtime", "--message-format=json", "--lib", "--no-run",
    "--release",
]
ENV_DROP_PREFIX = ("RUM_PROFILE_",)
ENV_DROP = {"RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUST_MIN_STACK"}


def fail(message: str, code: int = 1) -> "NoReturn":
    print(message, file=sys.stderr)
    raise SystemExit(code)


def tree_hash(root: Path) -> str:
    digest = hashlib.sha256()
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = sorted(d for d in dirnames if d not in EXCLUDES)
        rel = Path(dirpath).relative_to(root)
        for name in sorted(filenames):
            path = Path(dirpath) / name
            key = str(rel / name).encode()
            if path.is_symlink():
                digest.update(b"L\0" + key + b"\0" + os.readlink(path).encode())
                continue
            digest.update(b"F\0" + key + b"\0")
            try:
                with path.open("rb") as handle:
                    for chunk in iter(lambda: handle.read(1 << 20), b""):
                        digest.update(chunk)
            except OSError as error:
                fail(f"cannot read {path}: {error}")
    return digest.hexdigest()


def file_hash(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def crate_hashes(root: Path) -> dict[str, str]:
    hashes: dict[str, str] = {}
    crates = root / "crates"
    if not crates.is_dir():
        return hashes
    for entry in sorted(crates.iterdir()):
        if entry.is_dir() and not entry.is_symlink():
            hashes[entry.name] = tree_hash(entry)
    return hashes


def manifest_hash(root: Path) -> str:
    digest = hashlib.sha256()
    paths = [root / "Cargo.toml", root / "Cargo.lock"]
    crates = root / "crates"
    if crates.is_dir():
        paths.extend(sorted(crates.glob("*/Cargo.toml")))
    for path in paths:
        if not path.is_file():
            continue
        digest.update(str(path.relative_to(root)).encode() + b"\0")
        digest.update(path.read_bytes())
    return digest.hexdigest()


def build_env() -> dict[str, str]:
    env = {
        name: value for name, value in os.environ.items()
        if name not in ENV_DROP and not name.startswith(ENV_DROP_PREFIX)
    }
    env.update(
        CARGO_BUILD_JOBS="8",
        CARGO_NET_OFFLINE="true",
        CARGO_PROFILE_RELEASE_CODEGEN_UNITS="1",
        CARGO_TARGET_DIR=str(TARGET),
        TMPDIR=str(TMP),
        PYTHONDONTWRITEBYTECODE="1",
        DEBUGINFOD_URLS="",
    )
    return env


def run_bounded(command: list[str], timeout: float, log_path: Path | None) -> subprocess.CompletedProcess[str]:
    wrapped = [
        sys.executable, "-I", str(ROOT / "scripts" / "run-bounded.py"),
        "--timeout", str(timeout), "--", *command,
    ]
    result = subprocess.run(
        wrapped, cwd=ACTIVE, env=build_env(), text=True,
        stdin=subprocess.DEVNULL, capture_output=True,
    )
    if log_path is not None:
        log_path.parent.mkdir(parents=True, exist_ok=True)
        log_path.write_text(
            "$ " + " ".join(command) + "\n"
            + f"exit={result.returncode}\n"
            + "--- stdout ---\n" + result.stdout
            + "--- stderr ---\n" + result.stderr
        )
    return result


def touch_crates(root: Path, names: list[str]) -> None:
    """Make the named crates' sources newer than their artifacts.

    `cargo clean -p` removed nothing for the test-unit fingerprints in the
    calibration (it reported "Removed 0 files"), so the rebuild trigger is
    the source mtime, which cargo's fingerprint does honour.
    """
    now = time.time()
    for name in names:
        crate = root / "crates" / name
        if not crate.is_dir():
            continue
        for dirpath, dirnames, filenames in os.walk(crate):
            dirnames[:] = sorted(d for d in dirnames if d not in EXCLUDES)
            for filename in filenames:
                try:
                    os.utime(Path(dirpath) / filename, (now, now))
                except OSError:
                    pass


def clean_target() -> None:
    if TARGET.exists():
        shutil.rmtree(TARGET)
    try:
        STATE.unlink()
    except OSError:
        pass


def find_executable(stdout: str) -> str | None:
    for line in stdout.splitlines():
        if not line.startswith("{"):
            continue
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if (
            message.get("reason") == "compiler-artifact"
            and message.get("executable")
            and message.get("target", {}).get("name") == TARGET_NAME
        ):
            return message["executable"]
    return None


def read_state() -> dict | None:
    try:
        return json.loads(STATE.read_text())
    except (OSError, json.JSONDecodeError):
        return None


def write_state(value: dict) -> None:
    STATE.parent.mkdir(parents=True, exist_ok=True)
    STATE.write_text(json.dumps(value, indent=2) + "\n")


def rustc_version() -> str:
    result = subprocess.run(["rustc", "-V"], text=True, capture_output=True)
    if result.returncode != 0:
        fail(f"rustc -V failed: {result.stderr.strip()}")
    return result.stdout.strip()


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tree", required=True, type=Path,
                        help="assembled source tree to build")
    parser.add_argument("--out", required=True, type=Path,
                        help="where to copy the resulting probe binary")
    parser.add_argument("--force", action="store_true",
                        help="rebuild and sha256-compare against the cache")
    parser.add_argument("--timeout", type=float, default=880.0,
                        help="per-command wall timeout (default 880 s)")
    parser.add_argument("--receipt", type=Path, default=None,
                        help="optional path for the JSON receipt")
    args = parser.parse_args(argv)

    source = args.tree.resolve()
    if not source.is_dir():
        fail(f"--tree is not a directory: {source}", 2)

    PROBE.mkdir(parents=True, exist_ok=True)
    for directory in (TMP, LOGS, CACHE):
        directory.mkdir(parents=True, exist_ok=True)
    lock = LOCK.open("w")
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        fail("another probe build is running", 4)

    started = time.monotonic()
    if ACTIVE.exists():
        shutil.rmtree(ACTIVE)
    shutil.copytree(source, ACTIVE, symlinks=True)

    tree = tree_hash(ACTIVE)
    rustc = rustc_version()
    key = hashlib.sha256(
        json.dumps(
            {"tree": tree, "rustc": rustc, "cargo": CARGO, "path": str(ACTIVE)},
            sort_keys=True,
        ).encode()
    ).hexdigest()
    cache_dir = CACHE / key
    cached_binary = cache_dir / "rum_runtime"
    cached_receipt = cache_dir / "receipt.json"
    cached_sha = None
    cached_ok = False
    if cached_binary.is_file() and cached_receipt.is_file():
        try:
            cached_sha = json.loads(cached_receipt.read_text()).get("sha256")
        except json.JSONDecodeError:
            cached_sha = None
        if cached_sha and file_hash(cached_binary) == cached_sha:
            cached_ok = True

    hit = cached_ok and not args.force
    cargo_exit = None
    cargo_log = None
    spot = "none"
    binary = None
    sha = None

    if hit:
        binary = cached_binary
        sha = cached_sha
    else:
        state = read_state()
        crates = crate_hashes(ACTIVE)
        manifests = manifest_hash(ACTIVE)
        log_path = LOGS / f"{key}.log"
        cargo_log = str(log_path.relative_to(ROOT))
        full_clean = state is None or state.get("manifests") != manifests
        changed = [] if full_clean else [
            name for name, value in crates.items()
            if state.get("crates", {}).get(name) != value
        ]
        # The spot-check must recompile the crate even when the tree is
        # unchanged; a normal miss recompiles whatever changed.
        if args.force and not full_clean and "rum-runtime" not in changed:
            changed.append("rum-runtime")
        if full_clean:
            clean_target()
        else:
            touch_crates(ACTIVE, sorted(set(changed)))
        result = run_bounded(CARGO, args.timeout, log_path)
        cargo_exit = result.returncode
        if cargo_exit != 0:
            fail(f"cargo build failed: exit={cargo_exit}; see {cargo_log}")
        executable = find_executable(result.stdout)
        if not executable:
            fail("cargo reported no rum_runtime test executable")
        built = cache_dir / "rum_runtime.new"
        cache_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(executable, built)
        sha = file_hash(built)
        if args.force and cached_ok:
            spot = "pass" if sha == cached_sha else "fail"
            if spot == "fail":
                built.unlink(missing_ok=True)
                fail(
                    "determinism spot-check failed: rebuilt sha256 differs "
                    f"({sha} != {cached_sha})",
                    3,
                )
        built.replace(cache_dir / "rum_runtime")
        binary = cache_dir / "rum_runtime"
        write_state({"tree": tree, "crates": crates, "manifests": manifests})
        receipt = {
            "key": key, "tree": tree, "rustc": rustc, "cargo": CARGO,
            "path": str(ACTIVE), "sha256": sha, "cargo_exit": cargo_exit,
            "cargo_log": cargo_log, "cache_hit": False, "spot_check": spot,
            "source": str(source), "active": str(ACTIVE),
            "elapsed_seconds": round(time.monotonic() - started, 3),
        }
        cached_receipt.write_text(json.dumps(receipt, indent=2) + "\n")

    args.out.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(binary, args.out)
    out_sha = file_hash(args.out)
    if out_sha != sha:
        fail(f"copied binary hash mismatch: {out_sha} != {sha}")

    receipt = {
        "key": key, "tree": tree, "rustc": rustc, "cargo": CARGO,
        "path": str(ACTIVE), "sha256": sha, "cargo_exit": cargo_exit,
        "cargo_log": cargo_log, "cache_hit": hit, "spot_check": spot,
        "source": str(source), "out": str(args.out),
        "elapsed_seconds": round(time.monotonic() - started, 3),
    }
    if args.receipt is not None:
        args.receipt.parent.mkdir(parents=True, exist_ok=True)
        args.receipt.write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, sort_keys=True))
    print(
        f"PROBE_BUILD verdict=PASS cache_hit={str(hit).lower()} "
        f"spot_check={spot} sha256={sha} out={args.out}",
        flush=True,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
