#!/usr/bin/env bash
# Run a command under a resource watchdog and KILL it the moment it threatens the
# machine. Built 2026-06-29 after a cold `run_all.wls` ballooned to ~50 GB RSS and
# swap-thrashed for >2h undetected (rum's unbounded Module-gensym leak) — only
# noticed because the user opened Activity Monitor. This is deterministic and
# reacts in one poll interval; do NOT use an LLM poller for the auto-kill reflex.
#
# Usage:
#   scripts/guarded-test-run.sh [opts] -- <command> [args...]
# Options:
#   --rss-limit-gb N   kill when the process RSS exceeds N GB         (default 40)
#   --free-floor-gb N  kill when system free memory drops below N GB  (default 3)
#   --interval S       poll every S seconds                           (default 5)
#   --out-log PATH     command stdout+stderr here (default: a temp file, path printed)
# Metrics go to THIS script's stdout (one line per poll) so a Monitor/tail can
# stream them; the guarded command's own output goes to --out-log.
#
# Exit: 0 if the command finished on its own; 99 if the watchdog killed it.
set -uo pipefail

rss_limit_gb=40
free_floor_gb=3
interval=5
out_log=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --rss-limit-gb) rss_limit_gb=$2; shift 2 ;;
    --free-floor-gb) free_floor_gb=$2; shift 2 ;;
    --interval) interval=$2; shift 2 ;;
    --out-log) out_log=$2; shift 2 ;;
    --) shift; break ;;
    *) echo "guard: unknown option $1" >&2; exit 2 ;;
  esac
done
[[ $# -ge 1 ]] || { echo "guard: no command given (use -- <cmd>)" >&2; exit 2; }
[[ -n "$out_log" ]] || out_log=$(mktemp "${TMPDIR:-/tmp}/guarded_run.XXXXXX")

pagesize=$(getconf PAGESIZE 2>/dev/null || echo 4096)

# Free memory and swap use, read per platform. Each prints nothing when the
# value is unavailable: an unknown free-memory reading must never be taken for
# zero, which would trip the free-floor breach on the very first poll.
read_free_gb() {
  if [[ -r /proc/meminfo ]]; then
    awk '/^MemAvailable:/ { printf "%.2f", $2 / 1048576; found = 1 }
         END { if (!found) exit 1 }' /proc/meminfo
  elif command -v vm_stat >/dev/null 2>&1; then
    vm_stat | awk -v p="$pagesize" '
      /Pages free/ { gsub(/\./, ""); free = $3 }
      /Pages inactive/ { gsub(/\./, ""); inactive = $3 }
      END {
        if (free == "" && inactive == "") exit 1;
        printf "%.2f", (free + inactive) * p / 1073741824
      }'
  else
    return 1
  fi
}

read_swap_mb() {
  if [[ -r /proc/meminfo ]]; then
    awk '/^SwapTotal:/ { total = $2 } /^SwapFree:/ { free = $2 }
         END { if (total == "") exit 1; printf "%.1f", (total - free) / 1024 }' /proc/meminfo
  elif command -v sysctl >/dev/null 2>&1; then
    sysctl -n vm.swapusage 2>/dev/null | sed -E 's/.*used = ([0-9.]+)M.*/\1/'
  else
    return 1
  fi
}

# Launch the guarded command; its output goes to out_log.
"$@" > "$out_log" 2>&1 &
pid=$!
echo "[guard] pid=$pid rss_limit=${rss_limit_gb}GB free_floor=${free_floor_gb}GB interval=${interval}s out_log=$out_log"

killed=0
while kill -0 "$pid" 2>/dev/null; do
  rss_kb=$(ps -o rss= -p "$pid" 2>/dev/null | tr -d ' ')
  [[ -z "$rss_kb" ]] && break
  cpu=$(ps -o %cpu= -p "$pid" 2>/dev/null | tr -d ' ')
  free_gb=$(read_free_gb 2>/dev/null) || free_gb=""
  swap_used=$(read_swap_mb 2>/dev/null) || swap_used=""

  rss_gb=$(awk -v k="$rss_kb" 'BEGIN{printf "%.2f", k/1048576}')

  printf "[guard] rss=%sGB cpu=%s%% sys_free=%sGB swap_used=%sMB\n" \
    "$rss_gb" "${cpu:-?}" "${free_gb:-?}" "${swap_used:-?}"

  over_rss=$(awk -v a="$rss_gb" -v b="$rss_limit_gb" 'BEGIN{print (a>b)?1:0}')
  under_free=0
  if [[ -n "$free_gb" ]]; then
    under_free=$(awk -v a="$free_gb" -v b="$free_floor_gb" 'BEGIN{print (a<b)?1:0}')
  fi
  if [[ "$over_rss" == 1 || "$under_free" == 1 ]]; then
    reason="rss ${rss_gb}GB > ${rss_limit_gb}GB"
    [[ "$under_free" == 1 ]] && reason="sys_free ${free_gb}GB < ${free_floor_gb}GB"
    echo "[guard] !!! BREACH ($reason) — killing pid $pid and its tree"
    pkill -KILL -P "$pid" 2>/dev/null
    kill -KILL "$pid" 2>/dev/null
    killed=1
    break
  fi
  sleep "$interval"
done

if [[ "$killed" == 1 ]]; then
  echo "[guard] KILLED on resource breach. Partial output in $out_log"
  exit 99
fi
wait "$pid" 2>/dev/null
rc=$?
echo "[guard] command finished on its own (exit $rc). Output in $out_log"
exit "$rc"
