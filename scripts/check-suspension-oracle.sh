#!/usr/bin/env bash
set -euo pipefail
python3 - "$0" "$@" <<'PYTHON'
from pathlib import Path
import os
import subprocess
import sys
root = Path(sys.argv[1]).resolve().parent.parent
if len(sys.argv) > 3:
    raise SystemExit("usage: scripts/check-suspension-oracle.sh [timeout-seconds-per-build]")
timeout = int(sys.argv[2]) if len(sys.argv) == 3 else 900
if timeout <= 0:
    raise SystemExit("timeout must be positive")
scratch = root / ".temp/suspension-oracle"
(scratch / "tmp").mkdir(parents=True, exist_ok=True)
for label, features in [("default", []), ("no-default", ["--no-default-features"])]:
    command = ["python3", "scripts/run-bounded.py", "--timeout", str(timeout), "--", "env",
        "CARGO_BUILD_JOBS=1", "RUST_TEST_THREADS=1", "CARGO_NET_OFFLINE=true",
        f"TMPDIR={scratch / 'tmp'}", "cargo", "test", "-p", "rum-stdlib", *features,
        "--test", "suspension_oracle", "--", "--nocapture"]
    result = subprocess.run(command, cwd=root, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (scratch / f"gate-{label}.txt").write_bytes(result.stdout)
    sys.stdout.buffer.write(result.stdout)
    sys.stdout.buffer.flush()
    if result.returncode:
        raise SystemExit(result.returncode)
PYTHON
