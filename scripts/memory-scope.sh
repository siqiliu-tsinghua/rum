#!/bin/bash
# Run a command in its own memory-capped systemd user scope.
#
#   scripts/memory-scope.sh <unit-name> <memory-max> <command> [args...]
#
# The scope has MemoryMax=<memory-max>, no swap, and OOMPolicy=continue. When
# the scope reaches its cap, the kernel OOM-kills the largest process inside it
# (a test binary, a rustc) and nothing outside it. With OOMPolicy=continue
# systemd leaves the rest of the scope running, so a dispatched worker sees its
# command die with SIGKILL (exit 137) and carries on. The default policy, stop,
# would tear the whole scope down, worker included.
#
# No MemoryHigh is set on purpose. Throttling just below the cap stalls the
# scope for long periods, and systemd-oomd watches the user slice's memory
# pressure; a long stall there invites oomd to kill the whole scope. At
# MemoryMax the kernel reclaims page cache and then kills one process, quickly.
#
# Why the scope is made here and not by the worker: the agy and grk sandboxes
# hide the user bus, so `systemd-run --user` fails inside them (probed
# 2026-10-27, .temp/director/scope-probe/). Without a scope of its own, a
# worker runs in the director's terminal cgroup, and an oomd kill there would
# take the director's session with it.
#
# When the command exits, one line goes to stderr:
#
#   # memory-scope: unit=<unit> max=<bytes> peak=<bytes> oom=<n> oom_kill=<n> rc=<rc>
#
# The exit status is the command's.
set -u

if [[ "${1:-}" == --inside ]]; then
  unit=$2
  shift 2
  cg=/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup | head -n 1)
  "$@"
  rc=$?
  events=$(cat "$cg/memory.events" 2>/dev/null)
  oom=$(awk '$1 == "oom" { print $2 }' <<< "$events")
  oom_kill=$(awk '$1 == "oom_kill" { print $2 }' <<< "$events")
  echo "# memory-scope: unit=$unit max=$(cat "$cg/memory.max" 2>/dev/null) \
peak=$(cat "$cg/memory.peak" 2>/dev/null) oom=${oom:-?} oom_kill=${oom_kill:-?} rc=$rc" >&2
  exit "$rc"
fi

if [[ $# -lt 3 ]]; then
  echo "usage: $0 <unit-name> <memory-max> <command> [args...]" >&2
  exit 2
fi
unit=$1
memory_max=$2
shift 2
exec systemd-run --user --scope --quiet --unit="$unit" \
  -p MemoryMax="$memory_max" -p MemorySwapMax=0 -p OOMPolicy=continue \
  "$0" --inside "$unit" "$@"
