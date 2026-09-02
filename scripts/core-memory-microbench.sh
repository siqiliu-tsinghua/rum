#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/core-memory-microbench.sh [options]

Collect a core memory microbenchmark baseline for rum. Each workload runs in
its own fresh process.

Options:
  --output-dir PATH    Output directory. Defaults to a /tmp directory.
  --rum-cli PATH       Existing rum-cli binary. Defaults to target/release/rum-cli.
  --no-rss             Skip the peak-RSS wrapper and leave RSS columns empty.
                       Use this in sandboxes that deny its process probes.
  -h, --help           Show this help.

Outputs:
  measurements.tsv     One row per workload measurement.
  rum.tsv              rum-local memory and RSS metrics by label.
  raw/                 Per-workload stdout/stderr logs.

MemoryInUse[] and MaxMemoryUsed[] are reported as baseline-subtracted proxy
deltas: retained, peak, transient, and post-cleanup. Post-cleanup is measured
after each workload's cleanup form; this script does not clear process caches.
The allocation columns are empty unless the provided rum-cli already supports
the alloc-count profile and emits it.
EOF
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)

output_dir=${RUM_CORE_MEMORY_OUTPUT_DIR:-}
rum_cli_bin=${RUM_CLI_BIN:-"${repo_root}/target/release/rum-cli"}
collect_rss=true

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-dir)
      [[ $# -ge 2 ]] || { echo "error: --output-dir expects a path" >&2; exit 2; }
      output_dir=$2
      shift 2
      ;;
    --rum-cli)
      [[ $# -ge 2 ]] || { echo "error: --rum-cli expects a path" >&2; exit 2; }
      rum_cli_bin=$2
      shift 2
      ;;
    --no-rss)
      collect_rss=false
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --*)
      echo "error: unknown option $1" >&2
      exit 2
      ;;
    *)
      echo "error: unexpected positional argument $1" >&2
      exit 2
      ;;
  esac
done

peak_rss_runner="${script_dir}/peak-rss.py"
if [[ "${collect_rss}" == "true" && ! -x "${peak_rss_runner}" ]]; then
  echo "error: ${peak_rss_runner} is not executable" >&2
  exit 2
fi

if [[ ! -x "${rum_cli_bin}" ]]; then
  (
    cd "${repo_root}"
    cargo build --release -p rum-cli >/dev/null
  )
fi
if [[ ! -x "${rum_cli_bin}" ]]; then
  echo "error: rum-cli binary is not executable: ${rum_cli_bin}" >&2
  exit 2
fi

if [[ -z "${output_dir}" ]]; then
  output_dir=$(mktemp -d /tmp/rum-core-memory-microbench.XXXXXX)
else
  mkdir -p "${output_dir}"
fi
raw_dir="${output_dir}/raw"
mkdir -p "${raw_dir}"

measurements_tsv="${output_dir}/measurements.tsv"
rum_tsv="${output_dir}/rum.tsv"

printf 'label\tmem_before\tmem_after\tmem_delta\tmax_memory\tmem_postcleanup\tpeak_rss_bytes\talloc_count\talloc_bytes\talloc_peak_live_bytes\n' > "${measurements_tsv}"
printf 'label\trum_retained\trum_peak\trum_transient\trum_post_cleanup\trum_peak_rss\trum_rss_delta\trum_alloc_count\trum_alloc_bytes\trum_alloc_peak_live_bytes\n' > "${rum_tsv}"

wide_exact_match_expr() {
  local width=$1
  local args=""
  local index

  for ((index = 1; index <= width; index++)); do
    if [[ -n "${args}" ]]; then
      args+=", "
    fi
    args+="${index}"
  done

  printf 'MatchQ[gMemoryWideMatcher[%s], gMemoryWideMatcher[%s]]' "${args}" "${args}"
}

labels=(
  baseline
  matcher-single-wide-small
  matcher-single-wide-large
  module-leak-20000
  retain-range-1000
  retain-range-10000
  retain-range-100000
  transient-total-200000
)

workload_expr() {
  case "$1" in
    baseline) printf '%s' 'Null' ;;
    matcher-single-wide-small) wide_exact_match_expr 800 ;;
    matcher-single-wide-large) wide_exact_match_expr 1600 ;;
    module-leak-20000) printf '%s' 'Do[Module[{x}, x = i^2; x], {i, 1, 20000}]' ;;
    retain-range-1000) printf '%s' 'memGlobal = Range[1000]' ;;
    retain-range-10000) printf '%s' 'memGlobal = Range[10000]' ;;
    retain-range-100000) printf '%s' 'memGlobal = Range[100000]' ;;
    transient-total-200000) printf '%s' 'memTransient = Join[Range[100000], Range[100001, 200000]]; mxProbe = MaxMemoryUsed[]; Total[memTransient]; ClearAll[memTransient, mxProbe]' ;;
    *) echo "error: unknown workload label $1" >&2; exit 2 ;;
  esac
}

workload_cleanup() {
  case "$1" in
    baseline) printf '%s' 'Null' ;;
    matcher-single-wide-small | matcher-single-wide-large) printf '%s' 'Null' ;;
    module-leak-20000) printf '%s' 'Null' ;;
    retain-range-1000) printf '%s' 'ClearAll[memGlobal]' ;;
    retain-range-10000) printf '%s' 'ClearAll[memGlobal]' ;;
    retain-range-100000) printf '%s' 'ClearAll[memGlobal]' ;;
    transient-total-200000) printf '%s' 'ClearAll[memTransient, mxProbe]' ;;
    *) echo "error: unknown workload label $1" >&2; exit 2 ;;
  esac
}

parse_peak_rss() {
  local stderr_path=$1
  awk '
    /maximum resident set size/ {
      for (i = 1; i <= NF; i++) {
        if ($i ~ /^[0-9]+$/) {
          print $i;
          exit;
        }
      }
    }
  ' "${stderr_path}" || true
}

parse_alloc_metric() {
  local stderr_path=$1
  local metric=$2
  awk -v metric="${metric}" '
    /rum-profile alloc / {
      for (i = 1; i <= NF; i++) {
        if (index($i, metric "=") == 1) {
          sub("^" metric "=", "", $i);
          print $i;
          exit;
        }
      }
    }
  ' "${stderr_path}" || true
}

parse_measurement() {
  local stdout_path=$1
  local peak_rss=$2
  local alloc_count=$3
  local alloc_bytes=$4
  local alloc_peak_live_bytes=$5

  awk -v peak_rss="${peak_rss}" -v alloc_count="${alloc_count}" -v alloc_bytes="${alloc_bytes}" -v alloc_peak_live_bytes="${alloc_peak_live_bytes}" '
    function trim(x) {
      gsub(/^[[:space:]]+/, "", x);
      gsub(/[[:space:]]+$/, "", x);
      return x;
    }
    {
      line = $0;
      gsub(/\r/, "", line);
      sub(/^"/, "", line);
      sub(/"$/, "", line);
      gsub(/\\t/, "\t", line);
      split(line, fields, "\t");
    }
    fields[1] == "RUM_MEM" {
      before = trim(fields[3]);
      after = trim(fields[4]);
      max_memory = trim(fields[5]);
      postcleanup = trim(fields[6]);
      print trim(fields[2]) "\t" before "\t" after "\t" (after - before) "\t" max_memory "\t" postcleanup "\t" peak_rss "\t" alloc_count "\t" alloc_bytes "\t" alloc_peak_live_bytes;
    }
  ' "${stdout_path}"
}

run_workload() {
  local label=$1
  local expr
  local cleanup
  local stdout_path="${raw_dir}/${label}.stdout"
  local stderr_path="${raw_dir}/${label}.stderr"
  local program parsed peak_rss alloc_count alloc_bytes alloc_peak_live_bytes exit_code

  expr=$(workload_expr "${label}")
  cleanup=$(workload_cleanup "${label}")

  # The b0/b1/mx ownvalues add a tiny constant; compare deltas.
  program="b0 = MemoryInUse[]; ${expr}; b1 = MemoryInUse[]; mx = MaxMemoryUsed[]; ${cleanup}; bp = MemoryInUse[]; Print[\"RUM_MEM\t${label}\t\", b0, \"\t\", b1, \"\t\", mx, \"\t\", bp]"

  set +e
  if [[ "${collect_rss}" == "true" ]]; then
    "${peak_rss_runner}" "${rum_cli_bin}" --eval "${program}" >"${stdout_path}" 2>"${stderr_path}"
  else
    "${rum_cli_bin}" --eval "${program}" >"${stdout_path}" 2>"${stderr_path}"
  fi
  exit_code=$?
  set -e

  peak_rss=$(parse_peak_rss "${stderr_path}")
  alloc_count=$(parse_alloc_metric "${stderr_path}" count)
  alloc_bytes=$(parse_alloc_metric "${stderr_path}" bytes_total)
  alloc_peak_live_bytes=$(parse_alloc_metric "${stderr_path}" peak_live_bytes)
  if [[ "${exit_code}" != "0" ]]; then
    echo "error: ${label} exited with ${exit_code}; see ${stderr_path}" >&2
    exit "${exit_code}"
  fi

  parsed=$(parse_measurement \
    "${stdout_path}" \
    "${peak_rss}" \
    "${alloc_count}" \
    "${alloc_bytes}" \
    "${alloc_peak_live_bytes}")
  if [[ -z "${parsed}" ]]; then
    echo "error: ${label} did not emit a RUM_MEM row; see ${stdout_path}" >&2
    exit 1
  fi
  printf '%s\n' "${parsed}" >> "${measurements_tsv}"
}

for label in "${labels[@]}"; do
  run_workload "${label}"
done

label_order="${labels[*]}"
awk -F '\t' -v label_order="${label_order}" '
  BEGIN {
    label_count = split(label_order, labels, " ");
  }
  NR == 1 { next }
  {
    label = $1;
    mem_before[label] = $2;
    mem_after[label] = $3;
    mem_delta[label] = $4;
    max_memory[label] = $5;
    mem_postcleanup[label] = $6;
    peak_rss[label] = $7;
    alloc_count[label] = $8;
    alloc_bytes[label] = $9;
    alloc_peak_live_bytes[label] = $10;
    if (label == "baseline" && $7 != "") {
      baseline_rss = $7;
    }
  }
  function rss_delta(label, value) {
    value = peak_rss[label];
    if (value == "" || baseline_rss == "") {
      return "";
    }
    return (value + 0) - (baseline_rss + 0);
  }
  function metric_peak(label) {
    if (max_memory[label] == "" || mem_before[label] == "") {
      return "";
    }
    return (max_memory[label] + 0) - (mem_before[label] + 0);
  }
  function metric_transient(label) {
    if (max_memory[label] == "" || mem_after[label] == "") {
      return "";
    }
    return (max_memory[label] + 0) - (mem_after[label] + 0);
  }
  function metric_post_cleanup(label) {
    if (mem_postcleanup[label] == "" || mem_before[label] == "") {
      return "";
    }
    return (mem_postcleanup[label] + 0) - (mem_before[label] + 0);
  }
  END {
    for (i = 1; i <= label_count; i++) {
      label = labels[i];
      if (label == "") {
        continue;
      }
      rum_retained = mem_delta[label];
      rum_peak = metric_peak(label);
      rum_transient = metric_transient(label);
      rum_post_cleanup = metric_post_cleanup(label);
      rum_peak_rss = peak_rss[label];
      rum_rss = rss_delta(label);

      print label "\t" rum_retained "\t" rum_peak "\t" rum_transient "\t" rum_post_cleanup "\t" rum_peak_rss "\t" rum_rss "\t" alloc_count[label] "\t" alloc_bytes[label] "\t" alloc_peak_live_bytes[label];
    }
  }
' "${measurements_tsv}" >> "${rum_tsv}"

echo "core memory microbench output: ${output_dir}"
