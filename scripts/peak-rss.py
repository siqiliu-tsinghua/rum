#!/usr/bin/env python3
"""Run a command and report its peak resident set size, portably.

`/usr/bin/time -l` is a BSD/macOS interface. GNU `time` spells the flag `-v`,
labels the value differently, reports kilobytes rather than bytes, and is not
installed at all on many Linux distributions, so a benchmark that shells out
to it silently loses its RSS column off the platform it was written on. This
wrapper reads the same number from `getrusage(RUSAGE_CHILDREN)` instead, which
every Unix provides.

The child inherits stdout and stderr. After it exits, one line is appended to
stderr in the shape the benchmark's parser already expects:

    maximum resident set size: <bytes>

Exit status is the child's, or 128 + signal number if the child was signalled.
"""

from __future__ import annotations

import resource
import subprocess
import sys
from collections.abc import Sequence

USAGE_EXIT_CODE = 2
NOT_FOUND_EXIT_CODE = 127
SIGNAL_EXIT_OFFSET = 128


def peak_rss_bytes() -> int:
    """Peak RSS of all waited-for children, in bytes.

    `ru_maxrss` is bytes on macOS and kilobytes on Linux; the struct does not
    carry its own unit, so the platform decides it.
    """
    max_rss = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    if sys.platform == "darwin":
        return max_rss
    return max_rss * 1024


def main(argv: Sequence[str]) -> int:
    if not argv:
        print("usage: peak-rss.py <command> [args...]", file=sys.stderr)
        return USAGE_EXIT_CODE

    try:
        completed = subprocess.run(list(argv))
    except FileNotFoundError:
        print(f"peak-rss: command not found: {argv[0]}", file=sys.stderr)
        return NOT_FOUND_EXIT_CODE
    except PermissionError:
        print(f"peak-rss: not executable: {argv[0]}", file=sys.stderr)
        return NOT_FOUND_EXIT_CODE

    print(f"maximum resident set size: {peak_rss_bytes()}", file=sys.stderr)

    if completed.returncode < 0:
        return SIGNAL_EXIT_OFFSET - completed.returncode
    return completed.returncode


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
