#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/core-language-microbench.sh [options]

Collect a bottom-up core-language microbenchmark baseline for rum. The suite
runs small core-language statements and writes machine-readable TSV files.

Options:
  --output-dir PATH    Output directory. Defaults to a /tmp directory.
  --timeout SECONDS    Run timeout. Defaults to 180.
  --build-timeout SEC  rum-cli build timeout when the binary is missing.
                       Defaults to 300.
  --scale N            Multiply benchmark iteration counts. Defaults to 1.
  --rum-cli PATH       Existing rum-cli binary. Defaults to target/release/rum-cli.
  -h, --help           Show this help.

Outputs:
  core-language-microbench.rum  Generated benchmark source.
  runs.tsv                      One row for the rum execution.
  measurements.tsv              Parsed rum timing rows.
  timings.tsv                   rum timing by label.
  raw/                          rum stdout/stderr logs.
EOF
}

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd "${script_dir}/.." && pwd)
bounded_runner="${repo_root}/scripts/run-bounded.py"

output_dir=${RUM_CORE_MICRO_OUTPUT_DIR:-}
timeout_seconds=${RUM_CORE_MICRO_TIMEOUT:-180}
build_timeout_seconds=${RUM_CORE_MICRO_BUILD_TIMEOUT:-300}
scale=${RUM_CORE_MICRO_SCALE:-1}
reps=${RUM_CORE_MICRO_REPS:-7}
retained_module_gc_batches=${RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES:-3}
rum_cli_bin=${RUM_CLI_BIN:-"${repo_root}/target/release/rum-cli"}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-dir)
      [[ $# -ge 2 ]] || { echo "error: --output-dir expects a path" >&2; exit 2; }
      output_dir=$2
      shift 2
      ;;
    --timeout)
      [[ $# -ge 2 ]] || { echo "error: --timeout expects seconds" >&2; exit 2; }
      timeout_seconds=$2
      shift 2
      ;;
    --build-timeout)
      [[ $# -ge 2 ]] || { echo "error: --build-timeout expects seconds" >&2; exit 2; }
      build_timeout_seconds=$2
      shift 2
      ;;
    --scale)
      [[ $# -ge 2 ]] || { echo "error: --scale expects a positive integer" >&2; exit 2; }
      scale=$2
      shift 2
      ;;
    --reps)
      [[ $# -ge 2 ]] || { echo "error: --reps expects a positive integer" >&2; exit 2; }
      reps=$2
      shift 2
      ;;
    --rum-cli)
      [[ $# -ge 2 ]] || { echo "error: --rum-cli expects a path" >&2; exit 2; }
      rum_cli_bin=$2
      shift 2
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

for number in "${timeout_seconds}" "${build_timeout_seconds}" "${scale}" "${reps}" "${retained_module_gc_batches}"; do
  if ! [[ "${number}" =~ ^[1-9][0-9]*$ ]]; then
    echo "error: numeric options expect positive integers" >&2
    exit 2
  fi
done

if [[ ! -x "${bounded_runner}" ]]; then
  echo "error: bounded runner is not executable: ${bounded_runner}" >&2
  exit 2
fi

if [[ ! -x "${rum_cli_bin}" ]]; then
  (
    cd "${repo_root}"
    "${bounded_runner}" --timeout "${build_timeout_seconds}" -- cargo build --release -p rum-cli >/dev/null
  )
fi
if [[ ! -x "${rum_cli_bin}" ]]; then
  echo "error: rum-cli binary is not executable: ${rum_cli_bin}" >&2
  exit 2
fi

if [[ -z "${output_dir}" ]]; then
  output_dir=$(mktemp -d /tmp/rum-core-microbench.XXXXXX)
else
  mkdir -p "${output_dir}"
fi
raw_dir="${output_dir}/raw"
mkdir -p "${raw_dir}"

bench_rum="${output_dir}/core-language-microbench.rum"
runs_tsv="${output_dir}/runs.tsv"
measurements_tsv="${output_dir}/measurements.tsv"
timings_tsv="${output_dir}/timings.tsv"

module_chain_expr() {
  local width=$1
  local locals=""
  local index
  local name
  local next_name

  for ((index = 1; index <= width; index++)); do
    name="gMicroModuleChain${index}"
    if [[ -n "${locals}" ]]; then
      locals+=", "
    fi
    if ((index < width)); then
      next_name="gMicroModuleChain$((index + 1))"
      locals+="${name} = Hold[${next_name}]"
    else
      locals+="${name} = 1"
    fi
  done

  printf 'Module[{%s}, Hold[gMicroModuleChain1]]' "${locals}"
}

wide_symbol_call_expr() {
  local width=$1
  local args=""
  local index

  for ((index = 1; index <= width; index++)); do
    if [[ -n "${args}" ]]; then
      args+=", "
    fi
    args+="gMicroUpvalueArg${index}"
  done
  args+=", gMicroUpvalueNonce"

  printf 'gMicroUpvalueProbe[%s]' "${args}"
}

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

  printf 'MatchQ[gMicroWideMatcher[%s], gMicroWideMatcher[%s]]' "${args}" "${args}"
}

value_q_ownvalue_definitions() {
  local first=$1
  local last=$2
  local index

  for ((index = first; index <= last; index++)); do
    printf 'gMicroValueQDefinition%s = %s;\n' "${index}" "${index}"
  done
}

module_chain_small_width=96
module_chain_large_width=192
module_chain_small_expr=$(module_chain_expr "${module_chain_small_width}")
module_chain_large_expr=$(module_chain_expr "${module_chain_large_width}")
upvalue_candidates_small_width=1600
upvalue_candidates_large_width=3200
upvalue_candidates_small_expr=$(wide_symbol_call_expr "${upvalue_candidates_small_width}")
upvalue_candidates_large_expr=$(wide_symbol_call_expr "${upvalue_candidates_large_width}")
matcher_single_small_width=800
matcher_single_large_width=1600
matcher_single_small_expr=$(wide_exact_match_expr "${matcher_single_small_width}")
matcher_single_large_expr=$(wide_exact_match_expr "${matcher_single_large_width}")
value_q_definitions_small=$(value_q_ownvalue_definitions 1 2000)
value_q_definitions_large=$(value_q_ownvalue_definitions 2001 8000)

cat > "${bench_rum}" <<EOF
ClearAll[
  rumMicroBench,
  rumMicroOnce,
  rumMicroEmit,
  rumMicroN,
  rumMicroSink,
  gMicroGuardedOwnValue,
  fExact,
  fPattern,
  fGuarded,
  fGuardedFallback,
  gMicroWideMatcher,
  gMicroUpvalueTag,
  gMicroUpvalueMarker,
  gMicroUpvalueProbe,
  gMicroUpvalueNonce,
  gMicroGuardedUpvalueTag,
  gMicroGuardedUpvalueHead,
  gMicroSubvalue,
  gMicroGuardedSubvalue,
  gMicroDefaultHit,
  gMicroDefaultMiss,
  gMicroGuardedDefault,
  gMicroDNestedSinSmall,
  gMicroDVariable,
  list10,
  rulesSimple,
  gMicroGuardedRewriteRule,
  gMicroGuardedRewriteFallbackRules,
  gMicroWideTraversal,
  gMicroDeleteCasesLeafDeleteSmall,
  gMicroWideAssociation,
  gMicroAssociationPartWidth4000,
  gMicroAssociationPartWidth32000,
  gMicroReplacePartRules,
  gMicroGuardedReplacePartRule,
  gMicroGuardedReplacePartFallbackRules,
  gMicroStringLiteralHaystack,
  gMicroStringLiteralNeedle,
  gMicroStringLiteralRule,
  gMicroStringPatternRule,
  gMicroGuardedStringRule,
  gMicroGuardedStringFallbackRules,
  gMicroStringTrimLiteralList4000,
  gMicroStringTrimLiteralList32000,
  gMicroStringTrimSingleLiteral4000,
  gMicroStringTrimSingleLiteral32000,
  gMicroStringTrimWhitespace4000,
  gMicroStringTrimWhitespace32000,
  gMicroAppendListSmall,
  gMicroAppendListLarge,
  gMicroAppendPartOuter1000Small,
  gMicroAppendPartOuter1000Large,
  gMicroAssociationSetKeySmall,
  gMicroAssociationSetKeyLarge,
  gMicroAssociationSmallLiteralRepeat,
  gMicroSetPartFixed100Len100,
  gMicroSetPartFixed100Len800,
  gMicroSetPartFillSmall,
  gMicroSetPartFillLarge,
  gMicroSortCallbackSmall,
  gMicroSortCallbackLarge,
  gMicroSortByCallbackSmall,
  gMicroSortByCallbackLarge,
  gMicroReverseSortCallbackSmall,
  gMicroReverseSortCallbackLarge,
  gMicroSortDefaultWide,
  gMicroSortByDefaultWide,
  gMicroReverseSortDefaultWide,
  gMicroSortSymbolWidth4000,
  gMicroSortSymbolWidth16000,
  gMicroSortSymbolExpectedWidth4000,
  gMicroSortSymbolExpectedWidth16000,
  gMicroKeySortCallbackSmall,
  gMicroKeySortCallbackLarge,
  gMicroKeySortByCallbackSmall,
  gMicroKeySortByCallbackLarge,
  gMicroKeySortDefaultWide,
  gMicroKeySortByDefaultWide,
  gMicroRetainedModuleGcRoots,
  gMicroRetainedModuleGcBatch,
  gMicroRetainedModuleGcBatchSize,
  gMicroRetainedModuleGcSeconds
];

rumMicroScale = ${scale};
rumMicroReps = ${reps};
rumMicroN[n_Integer] := n * rumMicroScale;
rumMicroSink = Null;

SetAttributes[rumMicroBench, HoldAll];
rumMicroEmit[label_String, n_, seconds_] := Print[StringJoin[
  "RUM_CORE_MICRO\t",
  label,
  "\t",
  ToString[n],
  "\t",
  ToString[seconds]
]];

(* Timing source is AbsoluteTiming, which reads the kernel's MONOTONIC clock
   (Instant::now). Do NOT go back to differencing AbsoluteTime[], which reads
   the wall clock (SystemTime::now). A wall clock tracks civil time, so it is
   exposed to the outside world — NTP/time-daemon correction, manual clock
   changes, and host clock-hardware behavior can all slew or step it during a
   measurement window; a monotonic clock exists precisely to measure elapsed
   intervals and is immune to those. Wall clocks are also coarser on some
   platforms (microsecond resolution on macOS), which is real quantization
   noise on the shortest labels here (tens of microseconds). Reading the
   monotonic clock inside the kernel additionally drops the two per-rep
   AbsoluteTime[] evaluations from the measured region. Note that timings.tsv
   produced before this change is not directly comparable with timings.tsv
   produced after it.

   Min-of-reps: the timed loop runs the bare expression (NO per-iteration
   assignment — a per-iteration result-equals-expr injects one ownvalue Set
   plus cache invalidation per iteration, inflating rum time about 57 percent
   on cheap primitives. We time it
   rumMicroReps times and report the MINIMUM, which rejects OS scheduling
   spikes; sub-millisecond labels otherwise jitter 20-78 percent run-to-run.
   One untimed warm eval precedes the reps so the first timed rep is warm. *)
rumMicroBench[label_String, nExpr_, expr_] := Module[
  {i, k, n, seconds, best},
  n = nExpr;
  rumMicroSink = expr;
  best = -1;
  k = 1;
  While[k <= rumMicroReps,
    seconds = First[AbsoluteTiming[
      Do[
        expr,
        {i, 1, n}
      ]
    ]];
    If[best < 0 || seconds < best, best = seconds];
    k = k + 1
  ];
  rumMicroEmit[label, n, best];
];

SetAttributes[rumMicroOnce, HoldAll];
rumMicroOnce[label_String, nExpr_, expr_] := Module[
  {k, n, result, seconds, best},
  n = nExpr;
  result = expr;
  rumMicroSink = result;
  best = -1;
  k = 1;
  While[k <= rumMicroReps,
    seconds = First[AbsoluteTiming[result = expr]];
    If[best < 0 || seconds < best, best = seconds];
    k = k + 1
  ];
  rumMicroSink = result;
  rumMicroEmit[label, n, best];
];

xOwn = 123;
gMicroGuardedOwnValue := 123 /; True;
list10 = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};
rulesSimple = {a -> 1, b -> 2, c -> 3};
gMicroGuardedRewriteRule = x_Integer :> x + 1 /; x > 0;
gMicroGuardedRewriteFallbackRules = {
  x_Integer :> x + 1 /; x > 0,
  x_Integer :> x
};
gMicroWideTraversal = Range[rumMicroN[1600]];
gMicroDeleteCasesLeafDeleteSmall = Nest[List, 1, 8];
gMicroReplaceLeafRebuildSmall = Nest[List, 1, 8];
gMicroWideAssociation = AssociationThread[
  Range[rumMicroN[1600]] -> Range[rumMicroN[1600]]
];
gMicroAssociationPartWidth4000 = AssociationThread[Range[4000], Range[4000]];
gMicroAssociationPartWidth32000 = AssociationThread[Range[32000], Range[32000]];
gMicroReplacePartRules = Table[{1} -> i, {i, 1, rumMicroN[1600]}];
gMicroGuardedReplacePartRule = 1 :> 1 /; True;
gMicroGuardedReplacePartFallbackRules = {1 -> 0, 1 :> 1 /; False};
gMicroStringLiteralHaystack = StringJoin[
  StringRepeat["z", 300],
  StringRepeat["a", 16]
];
gMicroStringLiteralNeedle = StringRepeat["a", 16];
gMicroStringLiteralRule = gMicroStringLiteralNeedle -> "x";
gMicroStringPatternRule = x_ :> x;
gMicroGuardedStringRule = "a" :> ("x" /; True);
gMicroGuardedStringFallbackRules = {
  "a" :> ("x" /; False),
  "a" -> "y"
};
gMicroStringTrimLiteralList4000 = StringJoin[
  "1", StringRepeat["x", 3998], "2"
];
gMicroStringTrimLiteralList32000 = StringJoin[
  "1", StringRepeat["x", 31998], "2"
];
gMicroStringTrimSingleLiteral4000 = StringJoin[
  "1", StringRepeat["x", 3998], "1"
];
gMicroStringTrimSingleLiteral32000 = StringJoin[
  "1", StringRepeat["x", 31998], "1"
];
gMicroStringTrimWhitespace4000 = StringJoin[
  " ", StringRepeat["x", 3998], " "
];
gMicroStringTrimWhitespace32000 = StringJoin[
  " ", StringRepeat["x", 31998], " "
];
fExact[1] = 2;
fPattern[x_Integer] := x + 1;
fGuarded[x_Integer] := x + 1 /; x > 0;
fGuardedFallback[x_Integer] := x + 1 /; x > 0;
fGuardedFallback[x_] := x;
gMicroGuardedUpvalueTag /: gMicroGuardedUpvalueHead[
  gMicroGuardedUpvalueTag,
  x_Integer
] := x + 1 /; x > 0;
gMicroSubvalue[1][x_Integer] := x + 1;
gMicroGuardedSubvalue[1][x_Integer] := x + 1 /; x > 0;
Default[gMicroDefaultHit] = 1;
gMicroDefaultHit[x_.] := x;
gMicroDefaultMiss[x_.] := x;
Default[gMicroGuardedDefault] := 1 /; True;
gMicroGuardedDefault[x_.] := x;
gMicroDNestedSinSmall = Nest[Sin, gMicroDVariable, 8];

Print["RUM_CORE_MICRO_BEGIN"];

rumMicroOnce[
  "association-part-key-read-width4000",
  rumMicroN[40],
  Do[
    Part[gMicroAssociationPartWidth4000, System\`Key[3999]],
    {i, 1, rumMicroN[40]}
  ]
];
If[SameQ[Part[gMicroAssociationPartWidth4000, System\`Key[3999]], 3999], Null, Exit[2]];
rumMicroOnce[
  "association-part-key-read-width32000",
  rumMicroN[40],
  Do[
    Part[gMicroAssociationPartWidth32000, System\`Key[31999]],
    {i, 1, rumMicroN[40]}
  ]
];
If[SameQ[Part[gMicroAssociationPartWidth32000, System\`Key[31999]], 31999], Null, Exit[2]];
rumMicroOnce[
  "association-part-positional-read-width4000",
  rumMicroN[40],
  Do[Part[gMicroAssociationPartWidth4000, 3999], {i, 1, rumMicroN[40]}]
];
If[SameQ[Part[gMicroAssociationPartWidth4000, 3999], 3999], Null, Exit[2]];
rumMicroOnce[
  "association-part-positional-read-width32000",
  rumMicroN[40],
  Do[Part[gMicroAssociationPartWidth32000, 31999], {i, 1, rumMicroN[40]}]
];
If[SameQ[Part[gMicroAssociationPartWidth32000, 31999], 31999], Null, Exit[2]];

rumMicroBench["loop-empty", rumMicroN[5000], Null];
rumMicroOnce["do-count-null", rumMicroN[10000], Do[Null, {10000 * rumMicroScale}]];
rumMicroOnce["do-iterator-null", rumMicroN[1000], Do[Null, {j, 1000 * rumMicroScale}]];
rumMicroOnce["table-count-null", rumMicroN[5000], Table[Null, {5000 * rumMicroScale}]];
rumMicroOnce["table-iterator-null", rumMicroN[1000], Table[Null, {j, 1000 * rumMicroScale}]];
rumMicroOnce["table-iterator-symbol", rumMicroN[1000], Table[j, {j, 1000 * rumMicroScale}]];

rumMicroBench["literal-integer", rumMicroN[2000], 123];
rumMicroBench["literal-string", rumMicroN[2000], "abc"];
rumMicroBench["symbol-ownvalue-read", rumMicroN[2000], xOwn];
rumMicroBench[
  "symbol-ownvalue-guarded-read",
  rumMicroN[1000],
  gMicroGuardedOwnValue
];
rumMicroBench["set-ownvalue", rumMicroN[1000], xOwn = 123];

rumMicroBench["list-literal-small", rumMicroN[1000], {1, 2, 3, 4}];
rumMicroBench["head-list", rumMicroN[2000], Head[list10]];
rumMicroBench["length-list", rumMicroN[2000], Length[list10]];
rumMicroBench["part-list", rumMicroN[2000], list10[[5]]];
rumMicroBench["sameq-integer", rumMicroN[2000], SameQ[1, 1]];
rumMicroBench["sameq-list", rumMicroN[1000], SameQ[list10, list10]];

rumMicroBench["if-true", rumMicroN[2000], If[True, 1, 2]];
rumMicroBench["module-local", rumMicroN[500], Module[{x = 1}, x + 1]];
rumMicroOnce[
  "module-chain-escape-small",
  ${module_chain_small_width},
  ${module_chain_small_expr}
];
rumMicroOnce[
  "module-chain-escape-large",
  ${module_chain_large_width},
  ${module_chain_large_expr}
];
rumMicroBench["block-local", rumMicroN[500], Block[{xOwn = 1}, xOwn + 1]];
rumMicroOnce["block-symbol-only-repeat", rumMicroN[1000], Do[Block[{b}, Null], {1000 * rumMicroScale}]];
rumMicroOnce["block-init-null-repeat", rumMicroN[1000], Do[Block[{b = 1}, Null], {1000 * rumMicroScale}]];
rumMicroOnce["block-init-read-repeat", rumMicroN[1000], Do[Block[{b = 1}, b], {1000 * rumMicroScale}]];
rumMicroBench["with-local", rumMicroN[500], With[{x = 1}, x + 1]];
rumMicroBench["table-small", rumMicroN[200], Table[i, {i, 1, 5}]];
rumMicroOnce["table-iterator-plus", rumMicroN[1000], Table[j + 1, {j, 1000 * rumMicroScale}]];
rumMicroOnce["table-iterator-pair-list", rumMicroN[1000], Table[{j, j}, {j, 1000 * rumMicroScale}]];
rumMicroBench["do-small", rumMicroN[200], Module[{s = 0}, Do[s = i, {i, 1, 5}]; s]];
rumMicroBench[
  "d-nested-sin-small",
  rumMicroN[200],
  D[gMicroDNestedSinSmall, gMicroDVariable]
];
If[SameQ[Length[rumMicroSink], 8], Null, Exit[2]];

rumMicroBench["downvalue-exact-hit", rumMicroN[2000], fExact[1]];
rumMicroBench["downvalue-exact-miss", rumMicroN[2000], fExact[2]];
rumMicroBench["downvalue-pattern-hit", rumMicroN[1000], fPattern[1]];
rumMicroBench["downvalue-pattern-miss", rumMicroN[1000], fPattern["x"]];
rumMicroOnce["downvalue-exact-miss-repeat", rumMicroN[5000], Do[fExact[2], {5000 * rumMicroScale}]];
rumMicroOnce["downvalue-pattern-miss-repeat", rumMicroN[5000], Do[fPattern["x"], {5000 * rumMicroScale}]];
rumMicroBench["downvalue-guarded-hit", rumMicroN[1000], fGuarded[1]];
rumMicroBench[
  "downvalue-guarded-decline-fallback",
  rumMicroN[1000],
  fGuardedFallback[-1]
];
rumMicroBench[
  "upvalue-guarded-hit",
  rumMicroN[1000],
  gMicroGuardedUpvalueHead[gMicroGuardedUpvalueTag, 1]
];
rumMicroBench["subvalue-unguarded-hit", rumMicroN[1000], gMicroSubvalue[1][1]];
rumMicroBench[
  "subvalue-unguarded-miss",
  rumMicroN[1000],
  gMicroSubvalue[1]["x"]
];
rumMicroBench[
  "subvalue-guarded-hit",
  rumMicroN[1000],
  gMicroGuardedSubvalue[1][1]
];
rumMicroBench["default-unguarded-hit", rumMicroN[1000], gMicroDefaultHit[]];
rumMicroBench["default-unguarded-miss", rumMicroN[1000], gMicroDefaultMiss[]];
rumMicroBench[
  "default-guarded-hit",
  rumMicroN[1000],
  gMicroGuardedDefault[]
];

rumMicroBench["matchq-integer-success", rumMicroN[1000], MatchQ[1, _Integer]];
rumMicroBench["matchq-integer-fail", rumMicroN[1000], MatchQ["x", _Integer]];
rumMicroBench[
  "matcher-single-tiny",
  rumMicroN[1000],
  MatchQ[gMicroWideMatcher[1, 2, 3, 4], gMicroWideMatcher[1, 2, 3, 4]]
];
If[SameQ[rumMicroSink, True], Null, Exit[2]];
rumMicroOnce[
  "matcher-single-wide-small",
  ${matcher_single_small_width},
  ${matcher_single_small_expr}
];
If[SameQ[rumMicroSink, True], Null, Exit[2]];
rumMicroOnce[
  "matcher-single-wide-large",
  ${matcher_single_large_width},
  ${matcher_single_large_expr}
];
If[SameQ[rumMicroSink, True], Null, Exit[2]];
rumMicroBench["replaceall-simple", rumMicroN[500], a /. rulesSimple];
rumMicroBench[
  "rewrite-guarded-hit",
  rumMicroN[500],
  Replace[1, gMicroGuardedRewriteRule]
];
rumMicroBench[
  "rewrite-guarded-decline-fallback",
  rumMicroN[500],
  Replace[-1, gMicroGuardedRewriteFallbackRules]
];
rumMicroOnce[
  "cases-wide-no-match",
  rumMicroN[1600],
  Cases[gMicroWideTraversal, _String, {1}]
];
rumMicroOnce[
  "count-wide-no-match",
  rumMicroN[1600],
  Count[gMicroWideTraversal, _String, {1}]
];
rumMicroOnce[
  "position-wide-no-match",
  rumMicroN[1600],
  Position[gMicroWideTraversal, _String, {1}]
];
rumMicroOnce[
  "deletecases-wide-no-match",
  rumMicroN[1600],
  DeleteCases[gMicroWideTraversal, _String, {1}]
];
rumMicroOnce[
  "deletecases-wide-association-no-match",
  rumMicroN[1600],
  DeleteCases[gMicroWideAssociation, _String, {1}]
];
rumMicroBench[
  "deletecases-leaf-delete-small",
  rumMicroN[500],
  DeleteCases[gMicroDeleteCasesLeafDeleteSmall, 1, Infinity]
];
If[SameQ[rumMicroSink, Nest[List, {}, 7]], Null, Exit[2]];
rumMicroOnce[
  "replace-wide-no-match",
  rumMicroN[1600],
  Replace[gMicroWideTraversal, z -> 0, {1}]
];
rumMicroBench[
  "replace-leaf-rebuild-small",
  rumMicroN[500],
  Replace[gMicroReplaceLeafRebuildSmall, 1 -> 2, {0, Infinity}]
];
If[SameQ[rumMicroSink, Nest[List, 2, 8]], Null, Exit[2]];
rumMicroOnce[
  "replacepart-wide-rule-list",
  rumMicroN[1600],
  ReplacePart[{0}, gMicroReplacePartRules]
];
rumMicroBench[
  "replacepart-guarded-hit",
  rumMicroN[500],
  ReplacePart[{0}, gMicroGuardedReplacePartRule]
];
rumMicroBench[
  "replacepart-guarded-decline-fallback",
  rumMicroN[500],
  ReplacePart[{0}, gMicroGuardedReplacePartFallbackRules]
];

rumMicroBench["string-repeat-small", rumMicroN[500], StringRepeat["a", 3]];
rumMicroBench["character-range-small", rumMicroN[200], CharacterRange["a", "z"]];
rumMicroBench[
  "stringtrim-literal-list-4000",
  rumMicroN[100],
  StringTrim[gMicroStringTrimLiteralList4000, {"1", "2"}]
];
If[SameQ[StringLength[rumMicroSink], 3998], Null, Exit[2]];
rumMicroBench[
  "stringtrim-literal-list-32000",
  rumMicroN[100],
  StringTrim[gMicroStringTrimLiteralList32000, {"1", "2"}]
];
If[SameQ[StringLength[rumMicroSink], 31998], Null, Exit[2]];
rumMicroBench[
  "stringtrim-single-literal-4000",
  rumMicroN[100],
  StringTrim[gMicroStringTrimSingleLiteral4000, "1"]
];
If[SameQ[StringLength[rumMicroSink], 3998], Null, Exit[2]];
rumMicroBench[
  "stringtrim-single-literal-32000",
  rumMicroN[100],
  StringTrim[gMicroStringTrimSingleLiteral32000, "1"]
];
If[SameQ[StringLength[rumMicroSink], 31998], Null, Exit[2]];
rumMicroBench[
  "stringtrim-whitespace-4000",
  rumMicroN[100],
  StringTrim[gMicroStringTrimWhitespace4000]
];
If[SameQ[StringLength[rumMicroSink], 3998], Null, Exit[2]];
rumMicroBench[
  "stringtrim-whitespace-32000",
  rumMicroN[100],
  StringTrim[gMicroStringTrimWhitespace32000]
];
If[SameQ[StringLength[rumMicroSink], 31998], Null, Exit[2]];
rumMicroBench[
  "stringreplace-single-literal-unguarded",
  rumMicroN[200],
  StringReplace[gMicroStringLiteralHaystack, gMicroStringLiteralRule]
];
rumMicroBench[
  "stringreplace-pattern-unguarded",
  rumMicroN[500],
  StringReplace["abc", gMicroStringPatternRule]
];
rumMicroBench[
  "stringreplace-guarded-hit",
  rumMicroN[500],
  StringReplace["abc", gMicroGuardedStringRule]
];
rumMicroBench[
  "stringreplace-guarded-decline-fallback",
  rumMicroN[500],
  StringReplace["abc", gMicroGuardedStringFallbackRules]
];

rumMicroOnce[
  "appendto-list-incremental-small",
  rumMicroN[250],
  (
    gMicroAppendListSmall = {};
    Do[AppendTo[gMicroAppendListSmall, i], {i, 1, 250 * rumMicroScale}]
  )
];
rumMicroOnce[
  "appendto-list-incremental-large",
  rumMicroN[500],
  (
    gMicroAppendListLarge = {};
    Do[AppendTo[gMicroAppendListLarge, i], {i, 1, 500 * rumMicroScale}]
  )
];
rumMicroOnce[
  "appendto-part-outer1000-small",
  rumMicroN[100],
  (
    gMicroAppendPartOuter1000Small = Table[{}, {1000}];
    Do[
      AppendTo[gMicroAppendPartOuter1000Small[[1]], i],
      {i, 1, 100 * rumMicroScale}
    ]
  )
];
rumMicroOnce[
  "appendto-part-outer1000-large",
  rumMicroN[200],
  (
    gMicroAppendPartOuter1000Large = Table[{}, {1000}];
    Do[
      AppendTo[gMicroAppendPartOuter1000Large[[1]], i],
      {i, 1, 200 * rumMicroScale}
    ]
  )
];
rumMicroOnce[
  "association-set-key-incremental-small",
  rumMicroN[100],
  (
    gMicroAssociationSetKeySmall = <||>;
    Do[gMicroAssociationSetKeySmall[i] = i, {i, 1, 100 * rumMicroScale}]
  )
];
rumMicroOnce[
  "association-set-key-incremental-large",
  rumMicroN[200],
  (
    gMicroAssociationSetKeyLarge = <||>;
    Do[gMicroAssociationSetKeyLarge[i] = i, {i, 1, 200 * rumMicroScale}]
  )
];
rumMicroOnce[
  "association-small-literal-repeat",
  rumMicroN[500],
  Do[
    <|a -> gMicroAssociationSmallLiteralRepeat, b -> 2|>,
    {gMicroAssociationSmallLiteralRepeat, 1, 500 * rumMicroScale}
  ]
];

gMicroSetPartFixed100Len100 = Table[0, {100}];
rumMicroOnce[
  "set-part-fixed100-len100",
  100,
  Do[gMicroSetPartFixed100Len100[[1]] = i, {i, 1, 100}]
];
gMicroSetPartFixed100Len800 = Table[0, {800}];
rumMicroOnce[
  "set-part-fixed100-len800",
  100,
  Do[gMicroSetPartFixed100Len800[[1]] = i, {i, 1, 100}]
];
gMicroSetPartFillSmall = Table[0, {200}];
rumMicroOnce[
  "set-part-fill-small",
  200,
  Do[gMicroSetPartFillSmall[[i]] = i, {i, 1, 200}]
];
gMicroSetPartFillLarge = Table[0, {400}];
rumMicroOnce[
  "set-part-fill-large",
  400,
  Do[gMicroSetPartFillLarge[[i]] = i, {i, 1, 400}]
];

gMicroUpvalueNonce = 0;
gMicroUpvalueTag /: gMicroUpvalueMarker[gMicroUpvalueTag] := 1;
rumMicroBench[
  "upvalue-candidates-tiny",
  rumMicroN[1000],
  (
    gMicroUpvalueNonce = gMicroUpvalueNonce + 1;
    gMicroUpvalueProbe[a, b, c, d, gMicroUpvalueNonce]
  )
];
rumMicroOnce[
  "upvalue-candidates-wide-small",
  ${upvalue_candidates_small_width},
  (
    gMicroUpvalueNonce = gMicroUpvalueNonce + 1;
    ${upvalue_candidates_small_expr}
  )
];
rumMicroOnce[
  "upvalue-candidates-wide-large",
  ${upvalue_candidates_large_width},
  (
    gMicroUpvalueNonce = gMicroUpvalueNonce + 1;
    ${upvalue_candidates_large_expr}
  )
];
ClearAll[
  gMicroUpvalueTag,
  gMicroUpvalueMarker,
  gMicroUpvalueProbe,
  gMicroUpvalueNonce
];

gMicroSortCallbackSmall = Reverse[Range[rumMicroN[100]]];
gMicroSortCallbackLarge = Reverse[Range[rumMicroN[200]]];
gMicroSortByCallbackSmall = Reverse[Range[rumMicroN[100]]];
gMicroSortByCallbackLarge = Reverse[Range[rumMicroN[200]]];
gMicroReverseSortCallbackSmall = Range[rumMicroN[100]];
gMicroReverseSortCallbackLarge = Range[rumMicroN[200]];
gMicroSortDefaultWide = Reverse[Range[rumMicroN[1600]]];
gMicroSortByDefaultWide = Reverse[Range[rumMicroN[1600]]];
gMicroReverseSortDefaultWide = Range[rumMicroN[1600]];
gMicroSortSymbolWidth4000 = Reverse[Table[
  Symbol[StringJoin["gMicroSortSymbol", ToString[i]]],
  {i, 1, 4000}
]];
gMicroSortSymbolWidth16000 = Reverse[Table[
  Symbol[StringJoin["gMicroSortSymbol", ToString[i]]],
  {i, 1, 16000}
]];
gMicroSortSymbolExpectedWidth4000 = Sort[gMicroSortSymbolWidth4000];
gMicroSortSymbolExpectedWidth16000 = Sort[gMicroSortSymbolWidth16000];
gMicroKeySortCallbackSmall = AssociationThread[
  Range[rumMicroN[50]] -> Range[rumMicroN[50]]
];
gMicroKeySortCallbackLarge = AssociationThread[
  Range[rumMicroN[100]] -> Range[rumMicroN[100]]
];
gMicroKeySortByCallbackSmall = AssociationThread[
  Range[rumMicroN[25]] -> Range[rumMicroN[25]]
];
gMicroKeySortByCallbackLarge = AssociationThread[
  Range[rumMicroN[50]] -> Range[rumMicroN[50]]
];
gMicroKeySortDefaultWide = AssociationThread[
  Reverse[Range[rumMicroN[1600]]] -> Reverse[Range[rumMicroN[1600]]]
];
gMicroKeySortByDefaultWide = AssociationThread[
  Reverse[Range[rumMicroN[1600]]] -> Reverse[Range[rumMicroN[1600]]]
];

rumMicroOnce[
  "sort-callback-small",
  rumMicroN[100],
  Sort[gMicroSortCallbackSmall, Less]
];
If[SameQ[rumMicroSink, Range[rumMicroN[100]]], Null, Exit[2]];
rumMicroOnce[
  "sort-callback-large",
  rumMicroN[200],
  Sort[gMicroSortCallbackLarge, Less]
];
If[SameQ[rumMicroSink, Range[rumMicroN[200]]], Null, Exit[2]];
rumMicroOnce[
  "sortby-callback-small",
  rumMicroN[100],
  SortBy[gMicroSortByCallbackSmall, Identity, Less]
];
If[SameQ[rumMicroSink, Range[rumMicroN[100]]], Null, Exit[2]];
rumMicroOnce[
  "sortby-callback-large",
  rumMicroN[200],
  SortBy[gMicroSortByCallbackLarge, Identity, Less]
];
If[SameQ[rumMicroSink, Range[rumMicroN[200]]], Null, Exit[2]];
rumMicroOnce[
  "reversesort-callback-small",
  rumMicroN[100],
  ReverseSort[gMicroReverseSortCallbackSmall, Less]
];
If[SameQ[rumMicroSink, Reverse[Range[rumMicroN[100]]]], Null, Exit[2]];
rumMicroOnce[
  "reversesort-callback-large",
  rumMicroN[200],
  ReverseSort[gMicroReverseSortCallbackLarge, Less]
];
If[SameQ[rumMicroSink, Reverse[Range[rumMicroN[200]]]], Null, Exit[2]];

rumMicroOnce[
  "keysort-callback-small",
  rumMicroN[50],
  KeySort[gMicroKeySortCallbackSmall, Less]
];
If[SameQ[Keys[rumMicroSink], Range[rumMicroN[50]]] && Length[rumMicroSink] == rumMicroN[50], Null, Exit[2]];
rumMicroOnce[
  "keysort-callback-large",
  rumMicroN[100],
  KeySort[gMicroKeySortCallbackLarge, Less]
];
If[SameQ[Keys[rumMicroSink], Range[rumMicroN[100]]] && Length[rumMicroSink] == rumMicroN[100], Null, Exit[2]];
rumMicroOnce[
  "keysortby-callback-small",
  rumMicroN[25],
  KeySortBy[gMicroKeySortByCallbackSmall, Identity, Less]
];
If[SameQ[Keys[rumMicroSink], Range[rumMicroN[25]]] && Length[rumMicroSink] == rumMicroN[25], Null, Exit[2]];
rumMicroOnce[
  "keysortby-callback-large",
  rumMicroN[50],
  KeySortBy[gMicroKeySortByCallbackLarge, Identity, Less]
];
If[SameQ[Keys[rumMicroSink], Range[rumMicroN[50]]] && Length[rumMicroSink] == rumMicroN[50], Null, Exit[2]];

rumMicroOnce[
  "sort-default-wide",
  rumMicroN[1600],
  Sort[gMicroSortDefaultWide]
];
If[SameQ[rumMicroSink, Range[rumMicroN[1600]]], Null, Exit[2]];
rumMicroOnce[
  "sort-symbol-default-width4000",
  4000,
  Sort[gMicroSortSymbolWidth4000]
];
If[SameQ[rumMicroSink, gMicroSortSymbolExpectedWidth4000], Null, Exit[2]];
rumMicroOnce[
  "sort-symbol-default-width16000",
  16000,
  Sort[gMicroSortSymbolWidth16000]
];
If[SameQ[rumMicroSink, gMicroSortSymbolExpectedWidth16000], Null, Exit[2]];
rumMicroOnce[
  "sortby-default-wide",
  rumMicroN[1600],
  SortBy[gMicroSortByDefaultWide, Identity]
];
If[SameQ[rumMicroSink, Range[rumMicroN[1600]]], Null, Exit[2]];
rumMicroOnce[
  "reversesort-default-wide",
  rumMicroN[1600],
  ReverseSort[gMicroReverseSortDefaultWide]
];
If[SameQ[rumMicroSink, Reverse[Range[rumMicroN[1600]]]], Null, Exit[2]];
rumMicroOnce[
  "keysort-default-wide",
  rumMicroN[1600],
  KeySort[gMicroKeySortDefaultWide]
];
If[SameQ[Keys[rumMicroSink], Range[rumMicroN[1600]]] && Length[rumMicroSink] == rumMicroN[1600], Null, Exit[2]];
rumMicroOnce[
  "keysortby-default-wide",
  rumMicroN[1600],
  KeySortBy[gMicroKeySortByDefaultWide, Identity]
];
If[SameQ[Keys[rumMicroSink], Range[rumMicroN[1600]]] && Length[rumMicroSink] == rumMicroN[1600], Null, Exit[2]];

${value_q_definitions_small}
If[SameQ[gMicroValueQDefinition1, 1] && SameQ[gMicroValueQDefinition2000, 2000], Null, Exit[2]];
rumMicroBench[
  "valueq-undefined-symbol-definitions-2000",
  rumMicroN[20],
  ValueQ[gMicroValueQUndefined]
];
If[SameQ[rumMicroSink, False], Null, Exit[2]];

${value_q_definitions_large}
If[SameQ[gMicroValueQDefinition8000, 8000], Null, Exit[2]];
rumMicroBench[
  "valueq-undefined-symbol-definitions-8000",
  rumMicroN[20],
  ValueQ[gMicroValueQUndefined]
];
If[SameQ[rumMicroSink, False], Null, Exit[2]];

(* Keep every Module gensym live while a low RUM_GENSYM_GC_THRESHOLD forces
   the candidate scheduler to run. The environment-selected batch count lets
   the same stable row compare an early and a later retained batch. *)
gMicroRetainedModuleGcRoots = {};
gMicroRetainedModuleGcBatch = 1;
gMicroRetainedModuleGcBatchSize = rumMicroN[40];
gMicroRetainedModuleGcSeconds = 0;
While[gMicroRetainedModuleGcBatch <= ${retained_module_gc_batches},
  gMicroRetainedModuleGcSeconds = First[AbsoluteTiming[
    gMicroRetainedModuleGcRoots = Append[
      gMicroRetainedModuleGcRoots,
      With[{n = gMicroRetainedModuleGcBatchSize},
        Reap[Do[
          Module[{x = 1}, Sow[Hold[x]]],
          {n}
        ]]
      ]
    ]
  ]];
  gMicroRetainedModuleGcBatch = gMicroRetainedModuleGcBatch + 1
];
If[
  SameQ[
    Length[First[Last[Last[gMicroRetainedModuleGcRoots]]]],
    gMicroRetainedModuleGcBatchSize
  ],
  Null,
  Exit[2]
];
rumMicroEmit[
  "module-retained-gensym-gc-batch",
  gMicroRetainedModuleGcBatchSize,
  gMicroRetainedModuleGcSeconds
];

Print["RUM_CORE_MICRO_END"];
EOF

printf 'engine\tstatus\texit_code\telapsed_seconds\tstdout\tstderr\n' > "${runs_tsv}"
printf 'engine\tlabel\titerations\tseconds\tseconds_raw\tstdout\n' > "${measurements_tsv}"
rm -f "${timings_tsv}"

disable_profile_env=(
  RUM_PROFILE_LEXICAL_SUBSTITUTION=0
  RUM_PROFILE_LEXICAL_DEFER=0
  RUM_PROFILE_LEXICAL_DETAIL=0
  RUM_PROFILE_LEXICAL_SHAPES=0
  RUM_PROFILE_LEXICAL_BINDINGS=0
  RUM_PROFILE_LEXICAL_SOURCES=0
  RUM_PROFILE_LEXICAL_SOURCE_FINGERPRINTS=0
  RUM_PROFILE_LEXICAL_PROVENANCE=0
  RUM_PROFILE_EVALUATION_CACHE=0
  RUM_PROFILE_EVALUATION_CACHE_DEFER=0
  RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL=0
  RUM_PROFILE_RULE_RHS_TIMING=0
  RUM_PROFILE_DEFINITION_RULE_TIMING=0
  RUM_PROFILE_BUILTIN_TIMING=0
)

engine_exit_code=0

run_engine() {
  local engine_name=rum
  local stdout_path="${raw_dir}/${engine_name}.stdout"
  local stderr_path="${raw_dir}/${engine_name}.stderr"
  local start elapsed exit_code status

  start=$(perl -MTime::HiRes=time -e 'printf "%.6f\n", time')
  set +e
  # stdin must be closed explicitly. rum-cli binds non-TTY stdin as
  # Global`$ScriptInput when given a program argument, so an inherited pipe that
  # never reaches EOF leaves the benchmark waiting until its timeout, producing
  # an empty run that looks like a slow machine rather than a stuck read.
  env "${disable_profile_env[@]}" \
    "${bounded_runner}" --timeout "${timeout_seconds}" -- \
    "${rum_cli_bin}" --file "${bench_rum}" \
    >"${stdout_path}" 2>"${stderr_path}" </dev/null
  exit_code=$?
  set -e
  elapsed=$(perl -MTime::HiRes=time -e 'printf "%.6f\n", time - $ARGV[0]' "${start}")

  if [[ "${exit_code}" == "0" ]]; then
    status=ok
  else
    status=failed
  fi

  printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${engine_name}" "${status}" "${exit_code}" "${elapsed}" \
    "${stdout_path}" "${stderr_path}" >> "${runs_tsv}"

  awk -v engine="${engine_name}" -v stdout="${stdout_path}" '
    function trim(x) {
      gsub(/^[[:space:]]+/, "", x);
      gsub(/[[:space:]]+$/, "", x);
      return x;
    }
    function normalize(x, y, parts) {
      y = trim(x);
      gsub(/\*\^/, "e", y);
      if (y ~ /^-?[0-9]+\/[0-9]+$/) {
        split(y, parts, "/");
        return parts[1] / parts[2];
      }
      return y + 0;
    }
    {
      line = $0;
      gsub(/\r/, "", line);
      sub(/^"/, "", line);
      sub(/"$/, "", line);
      gsub(/\\t/, "\t", line);
      split(line, fields, "\t");
    }
    fields[1] == "RUM_CORE_MICRO" {
      print engine "\t" fields[2] "\t" fields[3] "\t" normalize(fields[4]) "\t" trim(fields[4]) "\t" stdout;
    }
  ' "${stdout_path}" >> "${measurements_tsv}"

  if [[ "${exit_code}" != "0" ]]; then
    echo "warning: ${engine_name} exited with ${exit_code}; see ${stderr_path}" >&2
  fi
  engine_exit_code=${exit_code}
}

run_engine

if [[ "${engine_exit_code}" != "0" ]]; then
  echo "error: core microbench failed because rum exited with ${engine_exit_code}; no timings.tsv written; see ${raw_dir}/rum.stderr" >&2
  exit "${engine_exit_code}"
fi

measurement_rows=$(awk 'END { print NR - 1 }' "${measurements_tsv}")
if [[ "${measurement_rows}" == "0" ]]; then
  echo "error: rum exited successfully but emitted zero RUM_CORE_MICRO measurement rows; no timings.tsv written; see ${raw_dir}/rum.stderr" >&2
  exit 1
fi

printf 'label\titerations\trum_seconds\n' > "${timings_tsv}"
awk -F '\t' '
  NR == 1 { next }
  {
    print $2 "\t" $3 "\t" $4;
  }
' "${measurements_tsv}" >> "${timings_tsv}"

echo "core microbench output: ${output_dir}"
