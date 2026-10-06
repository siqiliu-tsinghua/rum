# brief-582 — G7 step B: the structured trace filters resume, and the branch's groups close

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

`Trace`, `TraceDetail` and `TracePrint` with a pattern or a level / head
filter walk the captured trace tree and match each item; a pattern test that
asks for input resumes **past** the walk. Thirteen sites in
`crates/rum-runtime/src/runtime/debug_tools/trace.rs` — P0199, P0200, P0201,
P0202, P0203, P0204, P0205, P0206, P0208, P0209, P0210, P0211, P0213 — through
`filter_structured_trace_expr` / `_items` / `_leaf` and the detail family down
to `match_expr_owned`. brief-580's design: **one flat filter work stack and one
named frame**, preserving the trace structure, the candidate, pattern and
detail-field order, short-circuiting, the `Hold` and association
reconstruction, deduplication, printing, and `TracePrint`'s original return
value. Lost continuations **13 -> 0** — the last of brief-548's eight groups.

brief-580's report and `.temp/brief-580/draft-B-trace.md` are the raw
material; **this brief governs** where they differ. The rulings in brief-581
stand.

## Context pointers

- brief-580 and its report; `.temp/brief-580/{inventory.md,design.md,inventory-plan.md,performance-plan.md,performance-labels.json,probe.rs,cases.json,oracle.tsv,lifecycle.tsv,held-depth-96.rum}`.
- brief-581 and its report: the shared harness
  (`crates/rum-stdlib/tests/sow_trace_continuations/`) and the structural
  contract (`crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs`).
- `debug_tools/trace.rs` (`:61-151`, `:178-598`, `:652-767`),
  `trace/{builder,options,resume_frames}.rs`, `tree_walkers/level.rs:23-144`,
  `pattern_matching/outcome.rs`, `suspension/core.rs:764-855`; the existing
  trace shape and indentation contracts `runtime/tests/debug_tools.rs:4-266`.

## Scope

**Objective:** the flat trace filter, its frame, the tests, and the standing
and performance gates.

**Edit surface:** `debug_tools/trace.rs` (routing; the old recursive walkers
removed), new `trace/filter.rs`, `trace/filter/{leaf,state}.rs` and private
`trace/filter/tests.rs`, `trace/resume_frames.rs`; the shared harness
`sow_trace_continuations.rs` and its `{support,trace,lifecycle}` children;
`g7_continuation_contract.rs` (flat-state and edge gates with falsifiers);
rum-runtime's frame lists (five derive entries, one frame, named-frame count
**561 -> 562**).

**Inventory exceptions, authorized by name** (`inventory-plan.md`, I03):
retire the old `trace.rs` wrapping-map closures at `:552:26` and `:593:26` and
the `vec!` sites at `:519:23`, `:552:70`, `:559:57`, `:575:25`, `:593:70`,
`:596:32` — **only if each construct is actually gone** — with their
replacements discovered and disclosed in the new modules. `Unsplit:Trace`,
`Unsplit:TraceDetail`, `Unsplit:TracePrint` stay Unsplit/Pending. P0207 and
P0212's dependent routing may resolve without adding Lost counts.

**Standing duty — always in scope** (`TEMPLATE.md`): fingerprints and both
certificates (C01/C02 unchanged, marks earned on their merits), ownership
reconciliation, every row disclosed. **STOP** on any other row change or
disappearance, any new `UnknownHostPayload` or `self_reaching` row, any new
row in untouched code, any retention reclassification.

**Fence:** `trace/builder.rs`, `trace/options.rs`, `reap_state.rs`, Sow (step
A), the matcher, the level collector, the renderers, the suspension core and
public APIs unchanged; the existing capture frame keeps its custom
abandonment. No other group; no leases, `unsafe`, dependencies or lockfile; no
edits to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier briefs,
`scripts/` or docs. Keep `trace.rs` under 20k characters by extracting the
matching, not unrelated formatting; new modules under 10k each.

**Disclosure duty:** every changed function, state field and root, inventory
row, measured label, adjacent defect.

## Deliverable

brief-580 design.md §3: the flat driver owns the current level's remaining
siblings and emitted prefix, the parent levels, the pending original item, the
detail-field, candidate and pattern indices, the selected candidates and the
leaf's accepted prefix; it freezes only on the matcher's `InputRequired`. On
resume it applies the answer once: false advances pattern, then candidate,
then detail field; a detail match emits its original association once and
skips the rest of the search. Structured completion wraps original candidates
in `Hold` and keeps the existing deduplication and shape rules. Only root
completion renders and pushes `TracePrint` lines — exactly once — and returns
the saved body result. A second suspension freezes the new state once; pure
abandonment runs no missing match and prints nothing.

## Tests

Red first, recorded before any production edit: brief-580's remaining 38
cases (18 red, 20 controls: twelve trace cancellation cases, seven comparison
controls, and the 96-under-32 held-depth control) through the shared oracle —
value, error, termination, ordered messages, **exact print streams**, `log`,
actual suspension, exact prompts, no owner after, `1 + 1`. Step A's thirteen
stay green: 51 in all. The existing trace shape and indentation contracts
stay green. The structural contract extended to the filter modules, each check
with a falsifier.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. `TracePrint` prints once, at root completion — not per resume.
3. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression. If the gate cannot be met, report with
   attribution; do not switch to per-level frames on your own.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-582/tmp …`. Bounds: focused 150 s; runtime,
ownership and retention 300 s; a one-unit release build 900 s; a metric
process 150 s; `cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test sow_trace_continuations
cargo test -p rum-stdlib --lib g7_continuation_contract
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Performance gate.** Base: `.temp/brief-582/base/rum-cli` and
`rum-cli-alloc` — the director's copy of brief-581's selected one-unit
candidate, whose production source is the incoming tree (hashes in
`sha256.txt`); do not rebuild them. Candidates with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and `--features
rum-cli/alloc-count` (two arguments), fresh target directory each, hash
checked. Panel: `performance-plan.md`'s trace labels (unfiltered, pattern
filter, level and head filters, `TraceDetail`, `TracePrint`, a nested
evaluation) as scratch workloads, the Sow labels (`.temp/brief-581/perf/`) and
the controls. Each result checked first. **Noise floor first** (base against
itself, two pairs); a label regresses when both candidate samples exceed
`max(+0.05%, floor)`. Allocation counts, then two paired samples of `taskset -c
2 perf stat -x, -e cpu_core/instructions/u`, one label per process, stdin
`/dev/null`, `(count(2N) - count(N)) / N`, raw output first. No timing suite.
At most eight measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-582/`; `PROGRESS.md` after the red capture, each
compiling candidate and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the release `rum-cli` on passing programs, `taskset`/`perf`,
  `valgrind --tool=callgrind`. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed files and functions, state fields and roots, file sizes.
2. Verbatim red and green output (all 51 cases), the structural contract and
   falsifiers, ownership and certificate results, the broad suites; every
   inventory row change, the retirements named.
3. Thirteen sites repaired (13 -> 0 predicted); the noise floors and every
   measured row.
4. Stop-loss counters, residuals, open questions.

---

## Codex execution report — brief 582

All changes are uncommitted. No git command or network request was run. Scratch, raw measurements, source snapshots and checkpoints are under `.temp/brief-582/`; the standing certificate generator also uses its prescribed `.temp/brief-546/certificates/` output. The fingerprint generator uses its existing `target/brief-544/local-contracts.tsv` output, copied into this brief’s evidence directory. No timing suite was run, as directed.

Selected implementation: `candidate-08`. All 51 shared oracle fixtures pass; thirteen lost trace continuations are repaired (13 → 0 predicted). Performance verdict: **FAIL — residual instruction labels remain; this is not ready for performance acceptance.**

### 1. Changed files, functions, fields and roots

| File | Characters | Lines |
| --- | ---: | ---: |
| `crates/rum-runtime/src/runtime/debug_tools/trace.rs` | 15078 | 438 |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs` | 4971 | 150 |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs` | 5943 | 175 |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/state.rs` | 5891 | 191 |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 7101 | 206 |
| `crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs` | 2297 | 61 |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` | 30342 | 739 |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | 174476 | 1566 |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | 25519 | 654 |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | 253483 | 1494 |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 420926 | 2949 |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 399016 | 3139 |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv` | 20836 | 89 |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 91646 | 69 |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv` | 112156 | 318 |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 18162 | 3 |
| `crates/rum-stdlib/tests/sow_trace_continuations.rs` | 400 | 10 |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 7122 | 219 |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 9101 | 269 |

The other persistent change is this appended report. `trace.rs` now tail-transfers capture completion into the flat filter. Its eight recursive filter helpers were removed. The existing capture frame and its custom abandonment remain byte-identical. Builder, options, Sow/Reap, matcher, collector, renderer bodies, suspension core, public APIs, dependencies and lockfile were left unchanged. The frame lists add exactly five derived types and one pure-abandon frame; the named-frame assertion is 562.

Changed/new functions (including test functions and attribute changes); removed functions follow. Positions refer to the final tree.

| File | Line | Function | Change |
| --- | ---: | --- | --- |
| `crates/rum-runtime/src/runtime/debug_tools/trace.rs` | 92 | `finish_structured_trace_capture_expr` | changed |
| `crates/rum-runtime/src/runtime/debug_tools/trace.rs` | incoming 284 | `filter_structured_trace_expr` | removed |
| `crates/rum-runtime/src/runtime/debug_tools/trace.rs` | incoming 307 | `filter_structured_trace_detail_expr` | removed |
| `crates/rum-runtime/src/runtime/debug_tools/trace.rs` | incoming 330 | `filter_structured_trace_detail_items` | removed |
| `crates/rum-runtime/src/runtime/debug_tools/trace.rs` | incoming 374 | `trace_detail_item_matches` | removed |
| `crates/rum-runtime/src/runtime/debug_tools/trace.rs` | incoming 393 | `trace_detail_underlying_expr_matches` | removed |
| `crates/rum-runtime/src/runtime/debug_tools/trace.rs` | incoming 447 | `trace_detail_candidate_matches` | removed |
| `crates/rum-runtime/src/runtime/debug_tools/trace.rs` | incoming 465 | `filter_structured_trace_items` | removed |
| `crates/rum-runtime/src/runtime/debug_tools/trace.rs` | incoming 511 | `filter_structured_trace_leaf` | removed |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs` | 12 | `filter_trace_result` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs` | 40 | `drive_trace_filter` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs` | 87 | `finish_trace_filter_root` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs` | 104 | `append_child` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs` | 124 | `suspend_trace_filter` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs` | 136 | `resume_trace_filter` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs` | 4 | `current` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs` | 11 | `take_candidate` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs` | 20 | `select_candidates` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs` | 38 | `start_leaf` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs` | 90 | `apply_match` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs` | 106 | `next_field` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs` | 123 | `match_trace_leaf` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs` | 158 | `finish_leaf` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/state.rs` | 73 | `pattern_count` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/state.rs` | 80 | `pattern_at` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/state.rs` | 94 | `from_items` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/state.rs` | 101 | `freeze_level` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/state.rs` | 110 | `thaw_level` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/state.rs` | 120 | `freeze_filter` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/state.rs` | 160 | `thaw_filter` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 8 | `visit_expr` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 11 | `visit_symbol` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 15 | `roots` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 20 | `n` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 23 | `frozen` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 55 | `every_frozen_root_survives_copy_thaw_and_frame_tracing` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 79 | `omitted_parent_prefix_root_falsifier` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 90 | `false_advances_pattern_candidate_and_field_without_replaying` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 117 | `detail_true_emits_original_once_and_skips_later_search` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 129 | `flat_ancestor_copy_and_drop_use_shallow_expressions` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 140 | `error_frame_propagates_without_thaw_match_or_print` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 158 | `start` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 174 | `pending` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 181 | `successive_suspensions_print_only_after_root_completion` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/filter/tests.rs` | 192 | `pure_abandon_at_either_prompt_runs_no_missing_match` | new |
| `crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs` | 51 | `resume` | new |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` | 106 | `suspension_protocol_repository_invariants_hold` | changed |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | 335 | `trace_sources` | new |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | 347 | `trace_acyclic` | new |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | 361 | `trace_flat_call_graph_and_outbound_boundaries` | new |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | 386 | `trace_fields` | new |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | 432 | `trace_no_self_reaching` | new |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | 442 | `trace_complete_flat_fields_and_roots` | new |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | 451 | `trace_edges` | new |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | 530 | `trace_cold_edges_cursors_and_root_print_contract` | new |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | 536 | `rejected` | new |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | 544 | `trace_source_and_root_falsifiers_are_observed` | new |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | 598 | `trace_edge_falsifiers_are_observed` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 4 | `cancellation` | changed |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 102 | `trace_cancel_0_false` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 112 | `trace_cancel_0_true` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 122 | `trace_cancel_1_false` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 132 | `trace_cancel_1_true` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 142 | `tracedetail_cancel_0_false` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 152 | `tracedetail_cancel_0_true` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 162 | `tracedetail_cancel_1_false` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 172 | `tracedetail_cancel_1_true` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 182 | `traceprint_cancel_0_false` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 192 | `traceprint_cancel_0_true` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 202 | `traceprint_cancel_1_false` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 212 | `traceprint_cancel_1_true` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 4 | `case_trace_pattern` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 14 | `case_tracedetail_nested_pattern` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 24 | `case_traceprint_nested_pattern` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 34 | `case_trace_filter_levels` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 44 | `case_trace_filter_heads` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 54 | `case_tracedetail_filter_levels` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 64 | `case_tracedetail_filter_heads` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 74 | `case_trace_twice` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 84 | `case_tracedetail_twice` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 94 | `case_trace_pattern_alternatives` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 104 | `case_trace_effects_twice` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 114 | `case_trace_ordinary` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 124 | `case_trace_ordinary_filter` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 134 | `case_tracedetail_pattern_alternatives` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 144 | `case_tracedetail_effects_twice` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 154 | `case_tracedetail_ordinary` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 164 | `case_tracedetail_ordinary_filter` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 174 | `case_traceprint_pattern_alternatives` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 184 | `case_traceprint_effects_twice` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 194 | `case_traceprint_ordinary` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 204 | `case_traceprint_ordinary_filter` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 214 | `case_traceprint_error_after_resume` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 224 | `case_trace_definition_nesting` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 234 | `case_tracedetail_definition_nesting` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 244 | `case_traceprint_definition_nesting` | new |
| `crates/rum-stdlib/tests/sow_trace_continuations/trace.rs` | 254 | `captured_held_expression_exceeds_evaluation_depth` | new |

Frozen roots: `TraceFilterState` owns mode, saved body result, original pattern, level specification, Heads flag, current level, flat parents and pending leaf. Each level owns remaining siblings and its emitted prefix. The leaf owns its intact original item, remaining detail fields, field ordinal, ordered normalized patterns and index, original selected candidates and index, and accepted structured prefix. Candidates are `Single(Expr)` or `Parts(Vec<Expr>)`. The frame owns this state. All Expr fields reach the derived visitor; the private sentinel checks cover both candidate variants, current and parent prefixes, cold copies and the saved Print value. No Session borrow, observer, stack guard, iterator or callback is frozen.

Working fields and frozen declarations, verbatim (working iterators are consumed before capture):
```text
pub(in super::super) struct TraceFilterState {
    pub(super) mode: TraceOutput,
    pub(super) original_value: Expr,
    pub(super) pattern: Option<Expr>,
    pub(super) levels: Option<CasesLevelSpec>,
    pub(super) include_heads: bool,
    pub(super) current: TraceFilterLevel,
    pub(super) parents: Vec<TraceFilterLevel>,
    pub(super) pending: TraceFilterLeaf,
}

pub(super) struct TraceFilterLevel {
    remaining: Vec<Expr>,
    emitted: Vec<Expr>,
}

pub(super) struct TraceFilterLeaf {
    original: Expr,
    underlying: Vec<Expr>,
    field_index: usize,
    patterns: Vec<Expr>,
    pattern_index: usize,
    candidates: TraceFilterCandidates,
    candidate_index: usize,
    selected: Vec<Expr>,
}

pub(super) enum TraceFilterCandidates {
    Single(Expr),
    Parts(Vec<Expr>),
}

pub(super) struct FilterWork {
    pub(super) mode: TraceOutput,
    pub(super) original_value: Expr,
    pub(super) pattern: Option<Expr>,
    pub(super) levels: Option<CasesLevelSpec>,
    pub(super) include_heads: bool,
    pub(super) current: LevelWork,
    pub(super) parents: Vec<LevelWork>,
}

pub(super) struct LevelWork {
    pub(super) remaining: IntoIter<Expr>,
    pub(super) emitted: Vec<Expr>,
}

pub(super) struct LeafWork {
    pub(super) original: Expr,
    pub(super) underlying: IntoIter<Expr>,
    pub(super) field_index: usize,
    pub(super) patterns: PatternWork,
    pub(super) pattern_index: usize,
    pub(super) candidates: CandidateWork,
    pub(super) candidate_index: usize,
    pub(super) selected: Vec<Expr>,
}

pub(super) enum PatternWork {
    Single,
    Parts(Vec<Expr>),
}

pub(super) enum CandidateWork {
    Single(Option<Expr>),
    Parts(IntoIter<Expr>),
}
```


The ordinary path consumes iterator tails and freezes only on InputRequired. A resumed matcher outcome is decoded and applied once before continuing. Detail success emits its original association once; whole-leaf structured success retains the original Hold. Parts accumulate a separate accepted prefix and are wrapped in Hold at completion. Parent List collapse and adjacent leaf deduplication keep the original rules. Only root completion renders TracePrint and returns the saved body result. Pure abandonment performs no match and prints nothing.

Layout: all new modules are below 10k characters and `trace.rs` is below 20k. The existing G7 structural test file remains a cohesive closed-source/root/edge contract with its falsifiers; it is in the 20k–35k split-plan range. Its split plan is to extract the independent Sow and trace source/edge checks into sibling test modules while retaining shared scanner helpers and their falsifiers; this brief does not open that new module surface. The large pre-existing frame lists are touched only for the required closed-list additions/count. New files stay under the existing trace domain; no flat runtime-directory expansion was made.

### 2. Verification and every standing row change

Red capture preceded every production edit. The red log records all 51 cases: 33 passed and the 18 required trace witnesses failed. Expected values were never replaced with the broken matcher payload. All 34 queued comparison rows are checked against this original red capture, including value, log, status, termination, ordered messages and exact print strings.

Verbatim red output (the pre-edit checkpoint records exit 101):
```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.27s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-98b7354458f5211c)

running 51 tests
test lifecycle::sow_cancel_0_false ... lifecycle.tsv	Sow-cancel-0-false	stop=0	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::sow_cancel_0_true ... lifecycle.tsv	Sow-cancel-0-true	stop=0	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::sow_cancel_1_false ... lifecycle.tsv	Sow-cancel-1-false	stop=1	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::sow_cancel_1_true ... lifecycle.tsv	Sow-cancel-1-true	stop=1	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::trace_cancel_0_false ... lifecycle.tsv	Trace-cancel-0-false	stop=0	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::trace_cancel_0_true ... lifecycle.tsv	Trace-cancel-0-true	stop=0	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::trace_cancel_1_false ... lifecycle.tsv	Trace-cancel-1-false	stop=1	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::trace_cancel_1_true ... lifecycle.tsv	Trace-cancel-1-true	stop=1	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::tracedetail_cancel_0_false ... lifecycle.tsv	TraceDetail-cancel-0-false	stop=0	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::tracedetail_cancel_0_true ... lifecycle.tsv	TraceDetail-cancel-0-true	stop=0	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::tracedetail_cancel_1_false ... lifecycle.tsv	TraceDetail-cancel-1-false	stop=1	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::tracedetail_cancel_1_true ... lifecycle.tsv	TraceDetail-cancel-1-true	stop=1	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::traceprint_cancel_0_false ... lifecycle.tsv	TracePrint-cancel-0-false	stop=0	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::traceprint_cancel_0_true ... lifecycle.tsv	TracePrint-cancel-0-true	stop=0	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::traceprint_cancel_1_false ... lifecycle.tsv	TracePrint-cancel-1-false	stop=1	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::traceprint_cancel_1_true ... lifecycle.tsv	TracePrint-cancel-1-true	stop=1	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test sow::case_sow_direct_return ... suspensions.tsv	Sow-direct-return	1	"value: "	[]	[]
oracle.tsv	Sow-direct-return	"List[77, List[List[77]]]"	"List[2]"	"List[77, List[List[77]]]"	"List[2]"	["value: "]	["value: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-direct-return	"List[77, List[List[77]]]"	"List[2]"	"List[77, List[List[77]]]"	"List[2]"	["value: "]	["value: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test sow::case_sow_error_after_resume ... suspensions.tsv	Sow-error-after-resume	1	"error: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]
oracle.tsv	Sow-error-after-resume	"<error>"	"List[1, 8]"	"<error>"	"List[1, 8]"	["error: "]	["error: "]	true	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
ROW	Sow-error-after-resume	"<error>"	"List[1, 8]"	"<error>"	"List[1, 8]"	["error: "]	["error: "]	true	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
ok
test sow::case_sow_false_then_next_pattern ... suspensions.tsv	Sow-false-then-next-pattern	1	"decline: "	[]	[]
suspensions.tsv	Sow-false-then-next-pattern	2	"accept: "	[]	[]
oracle.tsv	Sow-false-then-next-pattern	"List[7, List[List[], List[List[7]]]]"	"List[1, 2]"	"List[7, List[List[], List[List[7]]]]"	"List[1, 2]"	["decline: ", "accept: "]	["decline: ", "accept: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-false-then-next-pattern	"List[7, List[List[], List[List[7]]]]"	"List[1, 2]"	"List[7, List[List[], List[List[7]]]]"	"List[1, 2]"	["decline: ", "accept: "]	["decline: ", "accept: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test sow::case_sow_later_pattern_error ... suspensions.tsv	Sow-later-pattern-error	1	"first: "	[]	[]
oracle.tsv	Sow-later-pattern-error	"<error>"	"List[1, 2]"	"<error>"	"List[1, 2]"	["first: "]	["first: "]	true	"Other { message: \"uncaught throw: \\\"g7 later\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 later\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 later\"] returned to top level." }]	[RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 later\"] returned to top level." }]	[]	[]
ROW	Sow-later-pattern-error	"<error>"	"List[1, 2]"	"<error>"	"List[1, 2]"	["first: "]	["first: "]	true	"Other { message: \"uncaught throw: \\\"g7 later\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 later\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 later\"] returned to top level." }]	[RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 later\"] returned to top level." }]	[]	[]
ok
test sow::case_sow_match ... suspensions.tsv	Sow-match	1	"p: "	[]	[]
oracle.tsv	Sow-match	"List[Null, List[List[1], List[2], List[3]]]"	"List[1, 2, 3]"	"List[Null, List[List[1], List[2], List[3]]]"	"List[1, 2, 3]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-match	"List[Null, List[List[1], List[2], List[3]]]"	"List[1, 2, 3]"	"List[Null, List[List[1], List[2], List[3]]]"	"List[1, 2, 3]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test sow::case_sow_nested_patterns ... suspensions.tsv	Sow-nested-patterns	1	"outer: "	[]	[]
suspensions.tsv	Sow-nested-patterns	2	"inner: "	[]	[]
oracle.tsv	Sow-nested-patterns	"List[List[Null, List[List[List[2]], List[List[1], List[2], List[3]]]], List[List[1], List[2], List[3]]]"	"List[List[\"outer\", 1], List[\"innerA\", 1], List[\"innerB\", 1], List[\"outer\", 2], List[\"innerA\", 2], List[\"innerB\", 2], List[\"outer\", 3], List[\"innerA\", 3], List[\"innerB\", 3]]"	"List[List[Null, List[List[List[2]], List[List[1], List[2], List[3]]]], List[List[1], List[2], List[3]]]"	"List[List[\"outer\", 1], List[\"innerA\", 1], List[\"innerB\", 1], List[\"outer\", 2], List[\"innerA\", 2], List[\"innerB\", 2], List[\"outer\", 3], List[\"innerA\", 3], List[\"innerB\", 3]]"	["outer: ", "inner: "]	["outer: ", "inner: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-nested-patterns	"List[List[Null, List[List[List[2]], List[List[1], List[2], List[3]]]], List[List[1], List[2], List[3]]]"	"List[List[\"outer\", 1], List[\"innerA\", 1], List[\"innerB\", 1], List[\"outer\", 2], List[\"innerA\", 2], List[\"innerB\", 2], List[\"outer\", 3], List[\"innerA\", 3], List[\"innerB\", 3]]"	"List[List[Null, List[List[List[2]], List[List[1], List[2], List[3]]]], List[List[1], List[2], List[3]]]"	"List[List[\"outer\", 1], List[\"innerA\", 1], List[\"innerB\", 1], List[\"outer\", 2], List[\"innerA\", 2], List[\"innerB\", 2], List[\"outer\", 3], List[\"innerA\", 3], List[\"innerB\", 3]]"	["outer: ", "inner: "]	["outer: ", "inner: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test sow::case_sow_ordinary ... oracle.tsv	Sow-ordinary	"List[List[Null, List[List[List[1], List[2], List[3]], List[List[2]]]], List[List[1], List[2], List[3]]]"	"List[]"	"List[List[Null, List[List[List[1], List[2], List[3]], List[List[2]]]], List[List[1], List[2], List[3]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-ordinary	"List[List[Null, List[List[List[1], List[2], List[3]], List[List[2]]]], List[List[1], List[2], List[3]]]"	"List[]"	"List[List[Null, List[List[List[1], List[2], List[3]], List[List[2]]]], List[List[1], List[2], List[3]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test sow::case_sow_same_tag_twice ... suspensions.tsv	Sow-same-tag-twice	1	"same: "	[]	[]
oracle.tsv	Sow-same-tag-twice	"List[Null, List[List[1, 2, 3]]]"	"List[9, 9, 9]"	"List[Null, List[List[1, 2, 3]]]"	"List[9, 9, 9]"	["same: "]	["same: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-same-tag-twice	"List[Null, List[List[1, 2, 3]]]"	"List[9, 9, 9]"	"List[Null, List[List[1, 2, 3]]]"	"List[9, 9, 9]"	["same: "]	["same: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test sow::case_sow_twice ... suspensions.tsv	Sow-twice	1	"p: "	[]	[]
suspensions.tsv	Sow-twice	2	"p: "	[]	[]
oracle.tsv	Sow-twice	"List[Null, List[List[1], List[2], List[3], List[4], List[5]]]"	"List[1, 2, 3, 4, 5]"	"List[Null, List[List[1], List[2], List[3], List[4], List[5]]]"	"List[1, 2, 3, 4, 5]"	["p: ", "p: "]	["p: ", "p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-twice	"List[Null, List[List[1], List[2], List[3], List[4], List[5]]]"	"List[1, 2, 3, 4, 5]"	"List[Null, List[List[1], List[2], List[3], List[4], List[5]]]"	"List[1, 2, 3, 4, 5]"	["p: ", "p: "]	["p: ", "p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::captured_held_expression_exceeds_evaluation_depth ... depth.tsv	held96_recursion32	List[1, List[Hold[x]]]
ok
test trace::case_trace_definition_nesting ... suspensions.tsv	Trace-definition-nesting	1	"tree: "	[]	[]
oracle.tsv	Trace-definition-nesting	"List[True, List[]]"	"List[5, 5]"	"List[Hold[f[3]], List[Hold[Plus[g[3], 1]], Hold[g[3]], List[Hold[Plus[3, 1]], Hold[4]], Hold[4], Hold[5]], Hold[5]]"	"List[5, 5, 4, 4, 4, 4, 5, 5]"	["tree: "]	["tree: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-definition-nesting	"List[True, List[]]"	"List[5, 5]"	"List[Hold[f[3]], List[Hold[Plus[g[3], 1]], Hold[g[3]], List[Hold[Plus[3, 1]], Hold[4]], Hold[4], Hold[5]], Hold[5]]"	"List[5, 5, 4, 4, 4, 4, 5, 5]"	["tree: "]	["tree: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'trace::case_trace_definition_nesting' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[True, List[]]", termination: "None", messages: [], prints: [] }, "List[5, 5]")
 right: (View { status: "OK", value: "List[Hold[f[3]], List[Hold[Plus[g[3], 1]], Hold[g[3]], List[Hold[Plus[3, 1]], Hold[4]], Hold[4], Hold[5]], Hold[5]]", termination: "None", messages: [], prints: [] }, "List[5, 5, 4, 4, 4, 4, 5, 5]")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test trace::case_trace_effects_twice ... suspensions.tsv	Trace-effects-twice	1	"first: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]
oracle.tsv	Trace-effects-twice	"List[True, List[]]"	"List[f[3, 7], 3]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["first: "]	["first: ", "second: "]	false	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]	["before", "after"]
ROW	Trace-effects-twice	"List[True, List[]]"	"List[f[3, 7], 3]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["first: "]	["first: ", "second: "]	false	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]	["before", "after"]

thread 'trace::case_trace_effects_twice' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:126:5:
assertion `left == right` failed: exact prompts and order
  left: ["first: "]
 right: ["first: ", "second: "]
FAILED
test trace::case_trace_filter_heads ... suspensions.tsv	Trace-filter-heads	1	"p: "	[]	[]
oracle.tsv	Trace-filter-heads	"List[True, List[]]"	"List[f, Plus]"	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[7], Hold[f], Hold[3], Hold[7], Hold[f[3, 7]]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, f[3, 7], Plus, 1, 2, 3, 3, Plus, 3, 4, 7, 7, f, 3, 7, f[3, 7]]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-filter-heads	"List[True, List[]]"	"List[f, Plus]"	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[7], Hold[f], Hold[3], Hold[7], Hold[f[3, 7]]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, f[3, 7], Plus, 1, 2, 3, 3, Plus, 3, 4, 7, 7, f, 3, 7, f[3, 7]]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'trace::case_trace_filter_heads' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[True, List[]]", termination: "None", messages: [], prints: [] }, "List[f, Plus]")
 right: (View { status: "OK", value: "List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[7], Hold[f], Hold[3], Hold[7], Hold[f[3, 7]]]", termination: "None", messages: [], prints: [] }, "List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, f[3, 7], Plus, 1, 2, 3, 3, Plus, 3, 4, 7, 7, f, 3, 7, f[3, 7]]")
FAILED
test trace::case_trace_filter_levels ... suspensions.tsv	Trace-filter-levels	1	"p: "	[]	[]
oracle.tsv	Trace-filter-levels	"List[True, List[]]"	"List[1, 2]"	"List[Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[1], Hold[2], Hold[3], Hold[4], Hold[3], Hold[7]]"	"List[1, 2, 3, 3, 4, 7, 1, 2, 3, 4, 3, 7]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-filter-levels	"List[True, List[]]"	"List[1, 2]"	"List[Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[1], Hold[2], Hold[3], Hold[4], Hold[3], Hold[7]]"	"List[1, 2, 3, 3, 4, 7, 1, 2, 3, 4, 3, 7]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'trace::case_trace_filter_levels' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[True, List[]]", termination: "None", messages: [], prints: [] }, "List[1, 2]")
 right: (View { status: "OK", value: "List[Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[1], Hold[2], Hold[3], Hold[4], Hold[3], Hold[7]]", termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 3, 4, 7, 1, 2, 3, 4, 3, 7]")
FAILED
test trace::case_trace_ordinary ... oracle.tsv	Trace-ordinary	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-ordinary	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_trace_ordinary_filter ... oracle.tsv	Trace-ordinary-filter	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[Plus], Hold[1], Hold[2], Hold[Plus], Hold[3], Hold[4], Hold[f], Hold[3], Hold[7]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[Plus], Hold[1], Hold[2], Hold[Plus], Hold[3], Hold[4], Hold[f], Hold[3], Hold[7]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-ordinary-filter	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[Plus], Hold[1], Hold[2], Hold[Plus], Hold[3], Hold[4], Hold[f], Hold[3], Hold[7]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[Plus], Hold[1], Hold[2], Hold[Plus], Hold[3], Hold[4], Hold[f], Hold[3], Hold[7]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_trace_pattern ... suspensions.tsv	Trace-pattern	1	"p: "	[]	[]
oracle.tsv	Trace-pattern	"List[True, List[]]"	"List[3, 3]"	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[3, 3]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-pattern	"List[True, List[]]"	"List[3, 3]"	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[3, 3]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'trace::case_trace_pattern' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[True, List[]]", termination: "None", messages: [], prints: [] }, "List[3, 3]")
 right: (View { status: "OK", value: "List[Hold[Plus[1, 2]], Hold[3]]", termination: "None", messages: [], prints: [] }, "List[3, 3]")
FAILED
test trace::case_trace_pattern_alternatives ... suspensions.tsv	Trace-pattern-alternatives	1	"decline: "	[]	[]
oracle.tsv	Trace-pattern-alternatives	"List[False, List[]]"	"List[List[1, f[3, 7]]]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[List[1, f[3, 7]], List[2, f[3, 7]], List[1, 3], List[2, 3], List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	["decline: "]	["decline: ", "accept: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-pattern-alternatives	"List[False, List[]]"	"List[List[1, f[3, 7]]]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[List[1, f[3, 7]], List[2, f[3, 7]], List[1, 3], List[2, 3], List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	["decline: "]	["decline: ", "accept: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'trace::case_trace_pattern_alternatives' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:126:5:
assertion `left == right` failed: exact prompts and order
  left: ["decline: "]
 right: ["decline: ", "accept: "]
FAILED
test trace::case_trace_twice ... suspensions.tsv	Trace-twice	1	"p: "	[]	[]
oracle.tsv	Trace-twice	"List[True, List[]]"	"List[f[3, 7], 3]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["p: "]	["p: ", "p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-twice	"List[True, List[]]"	"List[f[3, 7], 3]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["p: "]	["p: ", "p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'trace::case_trace_twice' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:126:5:
assertion `left == right` failed: exact prompts and order
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED
test trace::case_tracedetail_definition_nesting ... suspensions.tsv	TraceDetail-definition-nesting	1	"tree: "	[]	[]
oracle.tsv	TraceDetail-definition-nesting	"List[True, List[]]"	"List[5, 4]"	"List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[f[3]]], Rule[\"RuleLeft\", Hold[f[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[g[3], 1]]]], List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[g[3]]], Rule[\"RuleLeft\", Hold[g[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[3, 1]]]], List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 1]]], Rule[\"After\", Hold[4]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[g[3]]], Rule[\"After\", Hold[4]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[g[3], 1]]], Rule[\"After\", Hold[5]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[3]]], Rule[\"After\", Hold[5]]]]"	"List[5, 4, 4, 4, 5, 5]"	["tree: "]	["tree: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-definition-nesting	"List[True, List[]]"	"List[5, 4]"	"List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[f[3]]], Rule[\"RuleLeft\", Hold[f[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[g[3], 1]]]], List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[g[3]]], Rule[\"RuleLeft\", Hold[g[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[3, 1]]]], List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 1]]], Rule[\"After\", Hold[4]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[g[3]]], Rule[\"After\", Hold[4]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[g[3], 1]]], Rule[\"After\", Hold[5]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[3]]], Rule[\"After\", Hold[5]]]]"	"List[5, 4, 4, 4, 5, 5]"	["tree: "]	["tree: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'trace::case_tracedetail_definition_nesting' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[True, List[]]", termination: "None", messages: [], prints: [] }, "List[5, 4]")
 right: (View { status: "OK", value: "List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[f[3]]], Rule[\"RuleLeft\", Hold[f[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[g[3], 1]]]], List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[g[3]]], Rule[\"RuleLeft\", Hold[g[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[3, 1]]]], List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 1]]], Rule[\"After\", Hold[4]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[g[3]]], Rule[\"After\", Hold[4]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[g[3], 1]]], Rule[\"After\", Hold[5]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[3]]], Rule[\"After\", Hold[5]]]]", termination: "None", messages: [], prints: [] }, "List[5, 4, 4, 4, 5, 5]")
FAILED
test trace::case_tracedetail_effects_twice ... suspensions.tsv	TraceDetail-effects-twice	1	"first: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]
oracle.tsv	TraceDetail-effects-twice	"List[True, List[]]"	"List[3, 7]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[5, 6]]], Rule[\"After\", Hold[11]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4], Plus[5, 6]]]], Rule[\"After\", Hold[f[3, 7, 11]]]]]"	"List[3, 7, 11, f[3, 7, 11]]"	["first: "]	["first: ", "second: "]	false	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]	["before", "after"]
ROW	TraceDetail-effects-twice	"List[True, List[]]"	"List[3, 7]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[5, 6]]], Rule[\"After\", Hold[11]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4], Plus[5, 6]]]], Rule[\"After\", Hold[f[3, 7, 11]]]]]"	"List[3, 7, 11, f[3, 7, 11]]"	["first: "]	["first: ", "second: "]	false	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]	["before", "after"]

thread 'trace::case_tracedetail_effects_twice' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:126:5:
assertion `left == right` failed: exact prompts and order
  left: ["first: "]
 right: ["first: ", "second: "]
FAILED
test trace::case_tracedetail_filter_heads ... suspensions.tsv	TraceDetail-filter-heads	1	"p: "	[]	[]
oracle.tsv	TraceDetail-filter-heads	"List[True, List[]]"	"List[Plus, Plus]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-filter-heads	"List[True, List[]]"	"List[Plus, Plus]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'trace::case_tracedetail_filter_heads' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[True, List[]]", termination: "None", messages: [], prints: [] }, "List[Plus, Plus]")
 right: (View { status: "OK", value: "List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]", termination: "None", messages: [], prints: [] }, "List[Plus, Plus, f]")
FAILED
test trace::case_tracedetail_filter_levels ... suspensions.tsv	TraceDetail-filter-levels	1	"p: "	[]	[]
oracle.tsv	TraceDetail-filter-levels	"List[True, List[]]"	"List[1, 3]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[1, 3, 1]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-filter-levels	"List[True, List[]]"	"List[1, 3]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[1, 3, 1]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'trace::case_tracedetail_filter_levels' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[True, List[]]", termination: "None", messages: [], prints: [] }, "List[1, 3]")
 right: (View { status: "OK", value: "List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]", termination: "None", messages: [], prints: [] }, "List[1, 3, 1]")
FAILED
test trace::case_tracedetail_nested_pattern ... suspensions.tsv	TraceDetail-nested-pattern	1	"p: "	[]	[]
oracle.tsv	TraceDetail-nested-pattern	"List[True, List[]]"	"List[3, 7]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[3, 7, f[3, 7]]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-nested-pattern	"List[True, List[]]"	"List[3, 7]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[3, 7, f[3, 7]]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'trace::case_tracedetail_nested_pattern' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[True, List[]]", termination: "None", messages: [], prints: [] }, "List[3, 7]")
 right: (View { status: "OK", value: "List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]", termination: "None", messages: [], prints: [] }, "List[3, 7, f[3, 7]]")
FAILED
test trace::case_tracedetail_ordinary ... oracle.tsv	TraceDetail-ordinary	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-ordinary	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_tracedetail_ordinary_filter ... oracle.tsv	TraceDetail-ordinary-filter	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-ordinary-filter	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_tracedetail_pattern_alternatives ... suspensions.tsv	TraceDetail-pattern-alternatives	1	"decline: "	[]	[]
oracle.tsv	TraceDetail-pattern-alternatives	"List[False, List[]]"	"List[List[1, 3]]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	["decline: "]	["decline: ", "accept: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-pattern-alternatives	"List[False, List[]]"	"List[List[1, 3]]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	["decline: "]	["decline: ", "accept: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'trace::case_tracedetail_pattern_alternatives' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:126:5:
assertion `left == right` failed: exact prompts and order
  left: ["decline: "]
 right: ["decline: ", "accept: "]
FAILED
test trace::case_tracedetail_twice ... suspensions.tsv	TraceDetail-twice	1	"p: "	[]	[]
oracle.tsv	TraceDetail-twice	"List[False, List[]]"	"List[3, 3]"	"List[]"	"List[3, 3, 7, 7, 11, 11, f[3, 7, 11], f[3, 7, 11]]"	["p: "]	["p: ", "p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-twice	"List[False, List[]]"	"List[3, 3]"	"List[]"	"List[3, 3, 7, 7, 11, 11, f[3, 7, 11], f[3, 7, 11]]"	["p: "]	["p: ", "p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'trace::case_tracedetail_twice' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:126:5:
assertion `left == right` failed: exact prompts and order
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED
test trace::case_traceprint_definition_nesting ... suspensions.tsv	TracePrint-definition-nesting	1	"tree: "	[]	[]
oracle.tsv	TracePrint-definition-nesting	"List[True, List[]]"	"List[5, 5]"	"5"	"List[5, 5, 4, 4, 4, 4, 5, 5]"	["tree: "]	["tree: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	["f[3]", "  Plus[g[3], 1]", "  g[3]", "    Plus[3, 1]", "    4", "  4", "  5", "5"]
ROW	TracePrint-definition-nesting	"List[True, List[]]"	"List[5, 5]"	"5"	"List[5, 5, 4, 4, 4, 4, 5, 5]"	["tree: "]	["tree: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	["f[3]", "  Plus[g[3], 1]", "  g[3]", "    Plus[3, 1]", "    4", "  4", "  5", "5"]

thread 'trace::case_traceprint_definition_nesting' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[True, List[]]", termination: "None", messages: [], prints: [] }, "List[5, 5]")
 right: (View { status: "OK", value: "5", termination: "None", messages: [], prints: ["f[3]", "  Plus[g[3], 1]", "  g[3]", "    Plus[3, 1]", "    4", "  4", "  5", "5"] }, "List[5, 5, 4, 4, 4, 4, 5, 5]")
FAILED
test trace::case_traceprint_effects_twice ... suspensions.tsv	TracePrint-effects-twice	1	"first: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]
oracle.tsv	TracePrint-effects-twice	"List[True, List[]]"	"List[f[3, 7], 3]"	"f[3, 7]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["first: "]	["first: ", "second: "]	false	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]	["before", "after", "f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]
ROW	TracePrint-effects-twice	"List[True, List[]]"	"List[f[3, 7], 3]"	"f[3, 7]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["first: "]	["first: ", "second: "]	false	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]	["before", "after", "f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]

thread 'trace::case_traceprint_effects_twice' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:126:5:
assertion `left == right` failed: exact prompts and order
  left: ["first: "]
 right: ["first: ", "second: "]
FAILED
test trace::case_traceprint_error_after_resume ... suspensions.tsv	TracePrint-error-after-resume	1	"error: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]
oracle.tsv	TracePrint-error-after-resume	"<error>"	"List[f[3, 7]]"	"<error>"	"List[f[3, 7]]"	["error: "]	["error: "]	true	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
ROW	TracePrint-error-after-resume	"<error>"	"List[f[3, 7]]"	"<error>"	"List[f[3, 7]]"	["error: "]	["error: "]	true	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
ok
test trace::case_traceprint_nested_pattern ... suspensions.tsv	TracePrint-nested-pattern	1	"p: "	[]	[]
oracle.tsv	TracePrint-nested-pattern	"List[True, List[]]"	"List[f[3, 7], 3]"	"f[3, 7]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]
ROW	TracePrint-nested-pattern	"List[True, List[]]"	"List[f[3, 7], 3]"	"f[3, 7]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]

thread 'trace::case_traceprint_nested_pattern' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[True, List[]]", termination: "None", messages: [], prints: [] }, "List[f[3, 7], 3]")
 right: (View { status: "OK", value: "f[3, 7]", termination: "None", messages: [], prints: ["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"] }, "List[f[3, 7], 3, 3, 7, 7, f[3, 7]]")
FAILED
test trace::case_traceprint_ordinary ... oracle.tsv	TracePrint-ordinary	"f[3, 7]"	"List[]"	"f[3, 7]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]
ROW	TracePrint-ordinary	"f[3, 7]"	"List[]"	"f[3, 7]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]
ok
test trace::case_traceprint_ordinary_filter ... oracle.tsv	TracePrint-ordinary-filter	"f[3, 7]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	"f[3, 7]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f", "Plus", "1", "2", "Plus[1, 2]", "Plus", "3", "4", "Plus[3, 4]", "Plus", "1", "2", "Plus", "3", "4", "f", "3", "7"]	["f", "Plus", "1", "2", "Plus[1, 2]", "Plus", "3", "4", "Plus[3, 4]", "Plus", "1", "2", "Plus", "3", "4", "f", "3", "7"]
ROW	TracePrint-ordinary-filter	"f[3, 7]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	"f[3, 7]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f", "Plus", "1", "2", "Plus[1, 2]", "Plus", "3", "4", "Plus[3, 4]", "Plus", "1", "2", "Plus", "3", "4", "f", "3", "7"]	["f", "Plus", "1", "2", "Plus[1, 2]", "Plus", "3", "4", "Plus[3, 4]", "Plus", "1", "2", "Plus", "3", "4", "f", "3", "7"]
ok
test trace::case_traceprint_pattern_alternatives ... suspensions.tsv	TracePrint-pattern-alternatives	1	"decline: "	[]	[]
oracle.tsv	TracePrint-pattern-alternatives	"List[False, List[]]"	"List[List[1, f[3, 7]]]"	"f[3, 7]"	"List[List[1, f[3, 7]], List[2, f[3, 7]], List[1, 3], List[2, 3], List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	["decline: "]	["decline: ", "accept: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]
ROW	TracePrint-pattern-alternatives	"List[False, List[]]"	"List[List[1, f[3, 7]]]"	"f[3, 7]"	"List[List[1, f[3, 7]], List[2, f[3, 7]], List[1, 3], List[2, 3], List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	["decline: "]	["decline: ", "accept: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]

thread 'trace::case_traceprint_pattern_alternatives' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:126:5:
assertion `left == right` failed: exact prompts and order
  left: ["decline: "]
 right: ["decline: ", "accept: "]
FAILED

failures:

failures:
    trace::case_trace_definition_nesting
    trace::case_trace_effects_twice
    trace::case_trace_filter_heads
    trace::case_trace_filter_levels
    trace::case_trace_pattern
    trace::case_trace_pattern_alternatives
    trace::case_trace_twice
    trace::case_tracedetail_definition_nesting
    trace::case_tracedetail_effects_twice
    trace::case_tracedetail_filter_heads
    trace::case_tracedetail_filter_levels
    trace::case_tracedetail_nested_pattern
    trace::case_tracedetail_pattern_alternatives
    trace::case_tracedetail_twice
    trace::case_traceprint_definition_nesting
    trace::case_traceprint_effects_twice
    trace::case_traceprint_nested_pattern
    trace::case_traceprint_pattern_alternatives

test result: FAILED. 33 passed; 18 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s

error: test failed, to rerun pass `-p rum-stdlib --test sow_trace_continuations`
```

Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-582/tmp cargo test -p rum-stdlib --test sow_trace_continuations -- --nocapture`; exit 0. Raw: `.temp/brief-582/green-08.log`.
```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.23s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-98b7354458f5211c)

running 51 tests
test lifecycle::sow_cancel_0_false ... lifecycle.tsv	Sow-cancel-0-false	stop=0	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::sow_cancel_0_true ... lifecycle.tsv	Sow-cancel-0-true	stop=0	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::sow_cancel_1_false ... lifecycle.tsv	Sow-cancel-1-false	stop=1	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::sow_cancel_1_true ... lifecycle.tsv	Sow-cancel-1-true	stop=1	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::trace_cancel_0_false ... lifecycle.tsv	Trace-cancel-0-false	stop=0	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::trace_cancel_0_true ... lifecycle.tsv	Trace-cancel-0-true	stop=0	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::trace_cancel_1_false ... lifecycle.tsv	Trace-cancel-1-false	stop=1	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::trace_cancel_1_true ... lifecycle.tsv	Trace-cancel-1-true	stop=1	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::tracedetail_cancel_0_false ... lifecycle.tsv	TraceDetail-cancel-0-false	stop=0	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::tracedetail_cancel_0_true ... lifecycle.tsv	TraceDetail-cancel-0-true	stop=0	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::tracedetail_cancel_1_false ... lifecycle.tsv	TraceDetail-cancel-1-false	stop=1	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::tracedetail_cancel_1_true ... lifecycle.tsv	TraceDetail-cancel-1-true	stop=1	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::traceprint_cancel_0_false ... lifecycle.tsv	TracePrint-cancel-0-false	stop=0	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::traceprint_cancel_0_true ... lifecycle.tsv	TracePrint-cancel-0-true	stop=0	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::traceprint_cancel_1_false ... lifecycle.tsv	TracePrint-cancel-1-false	stop=1	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::traceprint_cancel_1_true ... lifecycle.tsv	TracePrint-cancel-1-true	stop=1	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test sow::case_sow_direct_return ... suspensions.tsv	Sow-direct-return	1	"value: "	[]	[]
oracle.tsv	Sow-direct-return	"List[77, List[List[77]]]"	"List[2]"	"List[77, List[List[77]]]"	"List[2]"	["value: "]	["value: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-direct-return	"List[77, List[List[77]]]"	"List[2]"	"List[77, List[List[77]]]"	"List[2]"	["value: "]	["value: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test sow::case_sow_error_after_resume ... suspensions.tsv	Sow-error-after-resume	1	"error: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]
oracle.tsv	Sow-error-after-resume	"<error>"	"List[1, 8]"	"<error>"	"List[1, 8]"	["error: "]	["error: "]	true	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
ROW	Sow-error-after-resume	"<error>"	"List[1, 8]"	"<error>"	"List[1, 8]"	["error: "]	["error: "]	true	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
ok
test sow::case_sow_false_then_next_pattern ... suspensions.tsv	Sow-false-then-next-pattern	1	"decline: "	[]	[]
suspensions.tsv	Sow-false-then-next-pattern	2	"accept: "	[]	[]
oracle.tsv	Sow-false-then-next-pattern	"List[7, List[List[], List[List[7]]]]"	"List[1, 2]"	"List[7, List[List[], List[List[7]]]]"	"List[1, 2]"	["decline: ", "accept: "]	["decline: ", "accept: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-false-then-next-pattern	"List[7, List[List[], List[List[7]]]]"	"List[1, 2]"	"List[7, List[List[], List[List[7]]]]"	"List[1, 2]"	["decline: ", "accept: "]	["decline: ", "accept: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test sow::case_sow_later_pattern_error ... suspensions.tsv	Sow-later-pattern-error	1	"first: "	[]	[]
oracle.tsv	Sow-later-pattern-error	"<error>"	"List[1, 2]"	"<error>"	"List[1, 2]"	["first: "]	["first: "]	true	"Other { message: \"uncaught throw: \\\"g7 later\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 later\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 later\"] returned to top level." }]	[RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 later\"] returned to top level." }]	[]	[]
ROW	Sow-later-pattern-error	"<error>"	"List[1, 2]"	"<error>"	"List[1, 2]"	["first: "]	["first: "]	true	"Other { message: \"uncaught throw: \\\"g7 later\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 later\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 later\"] returned to top level." }]	[RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 later\"] returned to top level." }]	[]	[]
ok
test sow::case_sow_match ... suspensions.tsv	Sow-match	1	"p: "	[]	[]
oracle.tsv	Sow-match	"List[Null, List[List[1], List[2], List[3]]]"	"List[1, 2, 3]"	"List[Null, List[List[1], List[2], List[3]]]"	"List[1, 2, 3]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-match	"List[Null, List[List[1], List[2], List[3]]]"	"List[1, 2, 3]"	"List[Null, List[List[1], List[2], List[3]]]"	"List[1, 2, 3]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test sow::case_sow_nested_patterns ... suspensions.tsv	Sow-nested-patterns	1	"outer: "	[]	[]
suspensions.tsv	Sow-nested-patterns	2	"inner: "	[]	[]
oracle.tsv	Sow-nested-patterns	"List[List[Null, List[List[List[2]], List[List[1], List[2], List[3]]]], List[List[1], List[2], List[3]]]"	"List[List[\"outer\", 1], List[\"innerA\", 1], List[\"innerB\", 1], List[\"outer\", 2], List[\"innerA\", 2], List[\"innerB\", 2], List[\"outer\", 3], List[\"innerA\", 3], List[\"innerB\", 3]]"	"List[List[Null, List[List[List[2]], List[List[1], List[2], List[3]]]], List[List[1], List[2], List[3]]]"	"List[List[\"outer\", 1], List[\"innerA\", 1], List[\"innerB\", 1], List[\"outer\", 2], List[\"innerA\", 2], List[\"innerB\", 2], List[\"outer\", 3], List[\"innerA\", 3], List[\"innerB\", 3]]"	["outer: ", "inner: "]	["outer: ", "inner: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-nested-patterns	"List[List[Null, List[List[List[2]], List[List[1], List[2], List[3]]]], List[List[1], List[2], List[3]]]"	"List[List[\"outer\", 1], List[\"innerA\", 1], List[\"innerB\", 1], List[\"outer\", 2], List[\"innerA\", 2], List[\"innerB\", 2], List[\"outer\", 3], List[\"innerA\", 3], List[\"innerB\", 3]]"	"List[List[Null, List[List[List[2]], List[List[1], List[2], List[3]]]], List[List[1], List[2], List[3]]]"	"List[List[\"outer\", 1], List[\"innerA\", 1], List[\"innerB\", 1], List[\"outer\", 2], List[\"innerA\", 2], List[\"innerB\", 2], List[\"outer\", 3], List[\"innerA\", 3], List[\"innerB\", 3]]"	["outer: ", "inner: "]	["outer: ", "inner: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test sow::case_sow_ordinary ... oracle.tsv	Sow-ordinary	"List[List[Null, List[List[List[1], List[2], List[3]], List[List[2]]]], List[List[1], List[2], List[3]]]"	"List[]"	"List[List[Null, List[List[List[1], List[2], List[3]], List[List[2]]]], List[List[1], List[2], List[3]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-ordinary	"List[List[Null, List[List[List[1], List[2], List[3]], List[List[2]]]], List[List[1], List[2], List[3]]]"	"List[]"	"List[List[Null, List[List[List[1], List[2], List[3]], List[List[2]]]], List[List[1], List[2], List[3]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test sow::case_sow_same_tag_twice ... suspensions.tsv	Sow-same-tag-twice	1	"same: "	[]	[]
oracle.tsv	Sow-same-tag-twice	"List[Null, List[List[1, 2, 3]]]"	"List[9, 9, 9]"	"List[Null, List[List[1, 2, 3]]]"	"List[9, 9, 9]"	["same: "]	["same: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-same-tag-twice	"List[Null, List[List[1, 2, 3]]]"	"List[9, 9, 9]"	"List[Null, List[List[1, 2, 3]]]"	"List[9, 9, 9]"	["same: "]	["same: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test sow::case_sow_twice ... suspensions.tsv	Sow-twice	1	"p: "	[]	[]
suspensions.tsv	Sow-twice	2	"p: "	[]	[]
oracle.tsv	Sow-twice	"List[Null, List[List[1], List[2], List[3], List[4], List[5]]]"	"List[1, 2, 3, 4, 5]"	"List[Null, List[List[1], List[2], List[3], List[4], List[5]]]"	"List[1, 2, 3, 4, 5]"	["p: ", "p: "]	["p: ", "p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-twice	"List[Null, List[List[1], List[2], List[3], List[4], List[5]]]"	"List[1, 2, 3, 4, 5]"	"List[Null, List[List[1], List[2], List[3], List[4], List[5]]]"	"List[1, 2, 3, 4, 5]"	["p: ", "p: "]	["p: ", "p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::captured_held_expression_exceeds_evaluation_depth ... depth.tsv	held96_recursion32	List[1, List[Hold[x]]]
ok
test trace::case_trace_definition_nesting ... suspensions.tsv	Trace-definition-nesting	1	"tree: "	[]	[]
oracle.tsv	Trace-definition-nesting	"List[Hold[f[3]], List[Hold[Plus[g[3], 1]], Hold[g[3]], List[Hold[Plus[3, 1]], Hold[4]], Hold[4], Hold[5]], Hold[5]]"	"List[5, 5, 4, 4, 4, 4, 5, 5]"	"List[Hold[f[3]], List[Hold[Plus[g[3], 1]], Hold[g[3]], List[Hold[Plus[3, 1]], Hold[4]], Hold[4], Hold[5]], Hold[5]]"	"List[5, 5, 4, 4, 4, 4, 5, 5]"	["tree: "]	["tree: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-definition-nesting	"List[Hold[f[3]], List[Hold[Plus[g[3], 1]], Hold[g[3]], List[Hold[Plus[3, 1]], Hold[4]], Hold[4], Hold[5]], Hold[5]]"	"List[5, 5, 4, 4, 4, 4, 5, 5]"	"List[Hold[f[3]], List[Hold[Plus[g[3], 1]], Hold[g[3]], List[Hold[Plus[3, 1]], Hold[4]], Hold[4], Hold[5]], Hold[5]]"	"List[5, 5, 4, 4, 4, 4, 5, 5]"	["tree: "]	["tree: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_trace_effects_twice ... suspensions.tsv	Trace-effects-twice	1	"first: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]
suspensions.tsv	Trace-effects-twice	2	"second: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before", "after"]
oracle.tsv	Trace-effects-twice	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["first: ", "second: "]	["first: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before", "after"]	["before", "after"]
ROW	Trace-effects-twice	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["first: ", "second: "]	["first: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before", "after"]	["before", "after"]
ok
test trace::case_trace_filter_heads ... suspensions.tsv	Trace-filter-heads	1	"p: "	[]	[]
oracle.tsv	Trace-filter-heads	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[7], Hold[f], Hold[3], Hold[7], Hold[f[3, 7]]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, f[3, 7], Plus, 1, 2, 3, 3, Plus, 3, 4, 7, 7, f, 3, 7, f[3, 7]]"	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[7], Hold[f], Hold[3], Hold[7], Hold[f[3, 7]]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, f[3, 7], Plus, 1, 2, 3, 3, Plus, 3, 4, 7, 7, f, 3, 7, f[3, 7]]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-filter-heads	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[7], Hold[f], Hold[3], Hold[7], Hold[f[3, 7]]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, f[3, 7], Plus, 1, 2, 3, 3, Plus, 3, 4, 7, 7, f, 3, 7, f[3, 7]]"	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[7], Hold[f], Hold[3], Hold[7], Hold[f[3, 7]]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, f[3, 7], Plus, 1, 2, 3, 3, Plus, 3, 4, 7, 7, f, 3, 7, f[3, 7]]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_trace_filter_levels ... suspensions.tsv	Trace-filter-levels	1	"p: "	[]	[]
oracle.tsv	Trace-filter-levels	"List[Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[1], Hold[2], Hold[3], Hold[4], Hold[3], Hold[7]]"	"List[1, 2, 3, 3, 4, 7, 1, 2, 3, 4, 3, 7]"	"List[Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[1], Hold[2], Hold[3], Hold[4], Hold[3], Hold[7]]"	"List[1, 2, 3, 3, 4, 7, 1, 2, 3, 4, 3, 7]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-filter-levels	"List[Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[1], Hold[2], Hold[3], Hold[4], Hold[3], Hold[7]]"	"List[1, 2, 3, 3, 4, 7, 1, 2, 3, 4, 3, 7]"	"List[Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[1], Hold[2], Hold[3], Hold[4], Hold[3], Hold[7]]"	"List[1, 2, 3, 3, 4, 7, 1, 2, 3, 4, 3, 7]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_trace_ordinary ... oracle.tsv	Trace-ordinary	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-ordinary	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_trace_ordinary_filter ... oracle.tsv	Trace-ordinary-filter	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[Plus], Hold[1], Hold[2], Hold[Plus], Hold[3], Hold[4], Hold[f], Hold[3], Hold[7]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[Plus], Hold[1], Hold[2], Hold[Plus], Hold[3], Hold[4], Hold[f], Hold[3], Hold[7]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-ordinary-filter	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[Plus], Hold[1], Hold[2], Hold[Plus], Hold[3], Hold[4], Hold[f], Hold[3], Hold[7]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	"List[Hold[f], Hold[Plus], Hold[1], Hold[2], Hold[Plus[1, 2]], Hold[Plus], Hold[3], Hold[4], Hold[Plus[3, 4]], Hold[Plus], Hold[1], Hold[2], Hold[Plus], Hold[3], Hold[4], Hold[f], Hold[3], Hold[7]]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_trace_pattern ... suspensions.tsv	Trace-pattern	1	"p: "	[]	[]
oracle.tsv	Trace-pattern	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[3, 3]"	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[3, 3]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-pattern	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[3, 3]"	"List[Hold[Plus[1, 2]], Hold[3]]"	"List[3, 3]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_trace_pattern_alternatives ... suspensions.tsv	Trace-pattern-alternatives	1	"decline: "	[]	[]
suspensions.tsv	Trace-pattern-alternatives	2	"accept: "	[]	[]
oracle.tsv	Trace-pattern-alternatives	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[List[1, f[3, 7]], List[2, f[3, 7]], List[1, 3], List[2, 3], List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[List[1, f[3, 7]], List[2, f[3, 7]], List[1, 3], List[2, 3], List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	["decline: ", "accept: "]	["decline: ", "accept: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-pattern-alternatives	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[List[1, f[3, 7]], List[2, f[3, 7]], List[1, 3], List[2, 3], List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[List[1, f[3, 7]], List[2, f[3, 7]], List[1, 3], List[2, 3], List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	["decline: ", "accept: "]	["decline: ", "accept: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_trace_twice ... suspensions.tsv	Trace-twice	1	"p: "	[]	[]
suspensions.tsv	Trace-twice	2	"p: "	[]	[]
oracle.tsv	Trace-twice	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["p: ", "p: "]	["p: ", "p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Trace-twice	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	"List[Hold[f[Plus[1, 2], Plus[3, 4]]], Hold[Plus[1, 2]], Hold[3], Hold[Plus[3, 4]], Hold[7], Hold[f[3, 7]]]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["p: ", "p: "]	["p: ", "p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_tracedetail_definition_nesting ... suspensions.tsv	TraceDetail-definition-nesting	1	"tree: "	[]	[]
oracle.tsv	TraceDetail-definition-nesting	"List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[f[3]]], Rule[\"RuleLeft\", Hold[f[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[g[3], 1]]]], List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[g[3]]], Rule[\"RuleLeft\", Hold[g[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[3, 1]]]], List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 1]]], Rule[\"After\", Hold[4]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[g[3]]], Rule[\"After\", Hold[4]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[g[3], 1]]], Rule[\"After\", Hold[5]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[3]]], Rule[\"After\", Hold[5]]]]"	"List[5, 4, 4, 4, 5, 5]"	"List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[f[3]]], Rule[\"RuleLeft\", Hold[f[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[g[3], 1]]]], List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[g[3]]], Rule[\"RuleLeft\", Hold[g[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[3, 1]]]], List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 1]]], Rule[\"After\", Hold[4]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[g[3]]], Rule[\"After\", Hold[4]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[g[3], 1]]], Rule[\"After\", Hold[5]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[3]]], Rule[\"After\", Hold[5]]]]"	"List[5, 4, 4, 4, 5, 5]"	["tree: "]	["tree: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-definition-nesting	"List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[f[3]]], Rule[\"RuleLeft\", Hold[f[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[g[3], 1]]]], List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[g[3]]], Rule[\"RuleLeft\", Hold[g[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[3, 1]]]], List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 1]]], Rule[\"After\", Hold[4]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[g[3]]], Rule[\"After\", Hold[4]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[g[3], 1]]], Rule[\"After\", Hold[5]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[3]]], Rule[\"After\", Hold[5]]]]"	"List[5, 4, 4, 4, 5, 5]"	"List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[f[3]]], Rule[\"RuleLeft\", Hold[f[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[g[3], 1]]]], List[Association[Rule[\"Step\", \"RuleApplied\"], Rule[\"Source\", \"DownValue\"], Rule[\"Input\", Hold[g[3]]], Rule[\"RuleLeft\", Hold[g[Pattern[z, Blank[]]]]], Rule[\"Result\", Hold[Plus[3, 1]]]], List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 1]]], Rule[\"After\", Hold[4]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[g[3]]], Rule[\"After\", Hold[4]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[g[3], 1]]], Rule[\"After\", Hold[5]]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[3]]], Rule[\"After\", Hold[5]]]]"	"List[5, 4, 4, 4, 5, 5]"	["tree: "]	["tree: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_tracedetail_effects_twice ... suspensions.tsv	TraceDetail-effects-twice	1	"first: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]
suspensions.tsv	TraceDetail-effects-twice	2	"second: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before", "after"]
oracle.tsv	TraceDetail-effects-twice	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[5, 6]]], Rule[\"After\", Hold[11]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4], Plus[5, 6]]]], Rule[\"After\", Hold[f[3, 7, 11]]]]]"	"List[3, 7, 11, f[3, 7, 11]]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[5, 6]]], Rule[\"After\", Hold[11]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4], Plus[5, 6]]]], Rule[\"After\", Hold[f[3, 7, 11]]]]]"	"List[3, 7, 11, f[3, 7, 11]]"	["first: ", "second: "]	["first: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before", "after"]	["before", "after"]
ROW	TraceDetail-effects-twice	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[5, 6]]], Rule[\"After\", Hold[11]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4], Plus[5, 6]]]], Rule[\"After\", Hold[f[3, 7, 11]]]]]"	"List[3, 7, 11, f[3, 7, 11]]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[5, 6]]], Rule[\"After\", Hold[11]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4], Plus[5, 6]]]], Rule[\"After\", Hold[f[3, 7, 11]]]]]"	"List[3, 7, 11, f[3, 7, 11]]"	["first: ", "second: "]	["first: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before", "after"]	["before", "after"]
ok
test trace::case_tracedetail_filter_heads ... suspensions.tsv	TraceDetail-filter-heads	1	"p: "	[]	[]
oracle.tsv	TraceDetail-filter-heads	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-filter-heads	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_tracedetail_filter_levels ... suspensions.tsv	TraceDetail-filter-levels	1	"p: "	[]	[]
oracle.tsv	TraceDetail-filter-levels	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[1, 3, 1]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[1, 3, 1]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-filter-levels	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[1, 3, 1]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[1, 3, 1]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_tracedetail_nested_pattern ... suspensions.tsv	TraceDetail-nested-pattern	1	"p: "	[]	[]
oracle.tsv	TraceDetail-nested-pattern	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[3, 7, f[3, 7]]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[3, 7, f[3, 7]]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-nested-pattern	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[3, 7, f[3, 7]]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[3, 7, f[3, 7]]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_tracedetail_ordinary ... oracle.tsv	TraceDetail-ordinary	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-ordinary	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_tracedetail_ordinary_filter ... oracle.tsv	TraceDetail-ordinary-filter	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-ordinary-filter	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[Plus, Plus, f]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_tracedetail_pattern_alternatives ... suspensions.tsv	TraceDetail-pattern-alternatives	1	"decline: "	[]	[]
suspensions.tsv	TraceDetail-pattern-alternatives	2	"accept: "	[]	[]
oracle.tsv	TraceDetail-pattern-alternatives	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	["decline: ", "accept: "]	["decline: ", "accept: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-pattern-alternatives	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	"List[Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[1, 2]]], Rule[\"After\", Hold[3]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[Plus[3, 4]]], Rule[\"After\", Hold[7]]], Association[Rule[\"Step\", \"EvaluationPass\"], Rule[\"Before\", Hold[f[Plus[1, 2], Plus[3, 4]]]], Rule[\"After\", Hold[f[3, 7]]]]]"	"List[List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	["decline: ", "accept: "]	["decline: ", "accept: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_tracedetail_twice ... suspensions.tsv	TraceDetail-twice	1	"p: "	[]	[]
suspensions.tsv	TraceDetail-twice	2	"p: "	[]	[]
oracle.tsv	TraceDetail-twice	"List[]"	"List[3, 3, 7, 7, 11, 11, f[3, 7, 11], f[3, 7, 11]]"	"List[]"	"List[3, 3, 7, 7, 11, 11, f[3, 7, 11], f[3, 7, 11]]"	["p: ", "p: "]	["p: ", "p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	TraceDetail-twice	"List[]"	"List[3, 3, 7, 7, 11, 11, f[3, 7, 11], f[3, 7, 11]]"	"List[]"	"List[3, 3, 7, 7, 11, 11, f[3, 7, 11], f[3, 7, 11]]"	["p: ", "p: "]	["p: ", "p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test trace::case_traceprint_definition_nesting ... suspensions.tsv	TracePrint-definition-nesting	1	"tree: "	[]	[]
oracle.tsv	TracePrint-definition-nesting	"5"	"List[5, 5, 4, 4, 4, 4, 5, 5]"	"5"	"List[5, 5, 4, 4, 4, 4, 5, 5]"	["tree: "]	["tree: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f[3]", "  Plus[g[3], 1]", "  g[3]", "    Plus[3, 1]", "    4", "  4", "  5", "5"]	["f[3]", "  Plus[g[3], 1]", "  g[3]", "    Plus[3, 1]", "    4", "  4", "  5", "5"]
ROW	TracePrint-definition-nesting	"5"	"List[5, 5, 4, 4, 4, 4, 5, 5]"	"5"	"List[5, 5, 4, 4, 4, 4, 5, 5]"	["tree: "]	["tree: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f[3]", "  Plus[g[3], 1]", "  g[3]", "    Plus[3, 1]", "    4", "  4", "  5", "5"]	["f[3]", "  Plus[g[3], 1]", "  g[3]", "    Plus[3, 1]", "    4", "  4", "  5", "5"]
ok
test trace::case_traceprint_effects_twice ... suspensions.tsv	TracePrint-effects-twice	1	"first: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]
suspensions.tsv	TracePrint-effects-twice	2	"second: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before", "after"]
oracle.tsv	TracePrint-effects-twice	"f[3, 7]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	"f[3, 7]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["first: ", "second: "]	["first: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before", "after", "f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]	["before", "after", "f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]
ROW	TracePrint-effects-twice	"f[3, 7]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	"f[3, 7]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["first: ", "second: "]	["first: ", "second: "]	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before", "after", "f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]	["before", "after", "f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]
ok
test trace::case_traceprint_error_after_resume ... suspensions.tsv	TracePrint-error-after-resume	1	"error: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]
oracle.tsv	TracePrint-error-after-resume	"<error>"	"List[f[3, 7]]"	"<error>"	"List[f[3, 7]]"	["error: "]	["error: "]	true	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
ROW	TracePrint-error-after-resume	"<error>"	"List[f[3, 7]]"	"<error>"	"List[f[3, 7]]"	["error: "]	["error: "]	true	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
ok
test trace::case_traceprint_nested_pattern ... suspensions.tsv	TracePrint-nested-pattern	1	"p: "	[]	[]
oracle.tsv	TracePrint-nested-pattern	"f[3, 7]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	"f[3, 7]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]
ROW	TracePrint-nested-pattern	"f[3, 7]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	"f[3, 7]"	"List[f[3, 7], 3, 3, 7, 7, f[3, 7]]"	["p: "]	["p: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]
ok
test trace::case_traceprint_ordinary ... oracle.tsv	TracePrint-ordinary	"f[3, 7]"	"List[]"	"f[3, 7]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]
ROW	TracePrint-ordinary	"f[3, 7]"	"List[]"	"f[3, 7]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]
ok
test trace::case_traceprint_ordinary_filter ... oracle.tsv	TracePrint-ordinary-filter	"f[3, 7]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	"f[3, 7]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f", "Plus", "1", "2", "Plus[1, 2]", "Plus", "3", "4", "Plus[3, 4]", "Plus", "1", "2", "Plus", "3", "4", "f", "3", "7"]	["f", "Plus", "1", "2", "Plus[1, 2]", "Plus", "3", "4", "Plus[3, 4]", "Plus", "1", "2", "Plus", "3", "4", "f", "3", "7"]
ROW	TracePrint-ordinary-filter	"f[3, 7]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	"f[3, 7]"	"List[f, Plus, 1, 2, 3, Plus, 3, 4, 7, Plus, 1, 2, Plus, 3, 4, f, 3, 7]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f", "Plus", "1", "2", "Plus[1, 2]", "Plus", "3", "4", "Plus[3, 4]", "Plus", "1", "2", "Plus", "3", "4", "f", "3", "7"]	["f", "Plus", "1", "2", "Plus[1, 2]", "Plus", "3", "4", "Plus[3, 4]", "Plus", "1", "2", "Plus", "3", "4", "f", "3", "7"]
ok
test trace::case_traceprint_pattern_alternatives ... suspensions.tsv	TracePrint-pattern-alternatives	1	"decline: "	[]	[]
suspensions.tsv	TracePrint-pattern-alternatives	2	"accept: "	[]	[]
oracle.tsv	TracePrint-pattern-alternatives	"f[3, 7]"	"List[List[1, f[3, 7]], List[2, f[3, 7]], List[1, 3], List[2, 3], List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	"f[3, 7]"	"List[List[1, f[3, 7]], List[2, f[3, 7]], List[1, 3], List[2, 3], List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	["decline: ", "accept: "]	["decline: ", "accept: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]
ROW	TracePrint-pattern-alternatives	"f[3, 7]"	"List[List[1, f[3, 7]], List[2, f[3, 7]], List[1, 3], List[2, 3], List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	"f[3, 7]"	"List[List[1, f[3, 7]], List[2, f[3, 7]], List[1, 3], List[2, 3], List[1, 3], List[2, 3], List[1, 7], List[2, 7], List[1, 7], List[2, 7], List[1, f[3, 7]], List[2, f[3, 7]]]"	["decline: ", "accept: "]	["decline: ", "accept: "]	true	"OK"	"OK"	"None"	"None"	[]	[]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]	["f[Plus[1, 2], Plus[3, 4]]", "Plus[1, 2]", "3", "Plus[3, 4]", "7", "f[3, 7]"]
ok

test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
```

Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-582/tmp cargo test -p rum-stdlib --lib g7_continuation_contract -- --nocapture`; exit 0. Raw: `.temp/brief-582/structural-final.log`.
```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 35.55s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 18 tests
test tests::association_depth_contract::g7_continuation_contract::cross_file_recursion_falsifier - should panic ... 
thread 'tests::association_depth_contract::g7_continuation_contract::cross_file_recursion_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:19:5:
recursive combined Sow modules
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
ok
test tests::association_depth_contract::g7_continuation_contract::direct_recursion_falsifier - should panic ... 
thread 'tests::association_depth_contract::g7_continuation_contract::direct_recursion_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:13:9:
recursive Sow file: crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs
ok
test tests::association_depth_contract::g7_continuation_contract::doubled_cursor_falsifier - should panic ... 
thread 'tests::association_depth_contract::g7_continuation_contract::doubled_cursor_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:280:5:
resumed Sow answer advances once
ok
test tests::association_depth_contract::g7_continuation_contract::eager_capture_falsifier - should panic ... 
thread 'tests::association_depth_contract::g7_continuation_contract::eager_capture_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:239:5:
one cold Sow capture
ok
test tests::association_depth_contract::g7_continuation_contract::early_bucket_commit_falsifier - should panic ... 
thread 'tests::association_depth_contract::g7_continuation_contract::early_bucket_commit_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:271:5:
Sow commit follows all matching
ok
test tests::association_depth_contract::g7_continuation_contract::lost_pending_accept_falsifier - should panic ... 
thread 'tests::association_depth_contract::g7_continuation_contract::lost_pending_accept_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:280:5:
resumed Sow answer advances once
ok
test tests::association_depth_contract::g7_continuation_contract::missing_root_falsifier - should panic ... 
thread 'tests::association_depth_contract::g7_continuation_contract::missing_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:152:5:
assertion `left == right` failed: complete flat Sow fields and roots
  left: {"SowMatchState": {"matched": "Vec<(usize,usize)>", "pattern_count": "usize", "pattern_index": "usize", "scope_index": "usize", "scope_limit": "usize", "value": "Expr"}, "SowMatchSuspensionFrame": {"state": "SowMatchState"}}
 right: {"SowMatchState": {"matched": "Vec<(usize,usize)>", "pattern_count": "usize", "pattern_index": "usize", "scope_index": "usize", "scope_limit": "usize", "tag": "Expr", "value": "Expr"}, "SowMatchSuspensionFrame": {"state": "SowMatchState"}}
ok
test tests::association_depth_contract::g7_continuation_contract::recursive_field_falsifier - should panic ... 
thread 'tests::association_depth_contract::g7_continuation_contract::recursive_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:152:5:
assertion `left == right` failed: complete flat Sow fields and roots
  left: {"SowMatchState": {"matched": "Vec<SowMatchState>", "pattern_count": "usize", "pattern_index": "usize", "scope_index": "usize", "scope_limit": "usize", "tag": "Expr", "value": "Expr"}, "SowMatchSuspensionFrame": {"state": "SowMatchState"}}
 right: {"SowMatchState": {"matched": "Vec<(usize,usize)>", "pattern_count": "usize", "pattern_index": "usize", "scope_index": "usize", "scope_limit": "usize", "tag": "Expr", "value": "Expr"}, "SowMatchSuspensionFrame": {"state": "SowMatchState"}}
ok
test tests::association_depth_contract::g7_continuation_contract::self_reaching_inventory_falsifier - should panic ... 
thread 'tests::association_depth_contract::g7_continuation_contract::self_reaching_inventory_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:155:5:
self-reaching Sow state
ok
test tests::association_depth_contract::g7_continuation_contract::skipped_root_falsifier - should panic ... 
thread 'tests::association_depth_contract::g7_continuation_contract::skipped_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:119:21:
Sow root cannot skip tracing
ok
test tests::association_depth_contract::g7_continuation_contract::sow_cold_capture_cursor_and_commit_contract ... G7_SOW_CONTINUATION cold capture, decode, one cursor advance and final commit passed
ok
test tests::association_depth_contract::g7_continuation_contract::sow_flat_fields_roots_and_ownership_are_complete ... G7_SOW_ROOTS value, tag and state roots; exact scalar/pair fields; no self-reaching Sow state
ok
test tests::association_depth_contract::g7_continuation_contract::sow_recursion_scanner_per_file_and_combined ... G7_SOW_SOURCE per-file and combined recursion, closed functions and outbound calls passed
ok
test tests::association_depth_contract::g7_continuation_contract::trace_cold_edges_cursors_and_root_print_contract ... G7_TRACE_CONTINUATION one cold capture, exact answer/cursor, short circuit, parent restore and root print passed
ok
test tests::association_depth_contract::g7_continuation_contract::trace_complete_flat_fields_and_roots ... G7_TRACE_ROOTS all five derived types, closed fields and candidates, no self-reaching control state
ok
test tests::association_depth_contract::g7_continuation_contract::trace_edge_falsifiers_are_observed ... 
thread 'tests::association_depth_contract::g7_continuation_contract::trace_edge_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:461:5:
trace match owns cold capture
G7_TRACE_FALSIFIER lost pending match: rejected

thread 'tests::association_depth_contract::g7_continuation_contract::trace_edge_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:486:5:
detail short circuit and pattern advance
G7_TRACE_FALSIFIER doubled cursor: rejected

thread 'tests::association_depth_contract::g7_continuation_contract::trace_edge_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:483:5:
apply trace answer once without replay
G7_TRACE_FALSIFIER cursor reset replay: rejected

thread 'tests::association_depth_contract::g7_continuation_contract::trace_edge_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:506:5:
restore parent prefixes
G7_TRACE_FALSIFIER omitted parent prefix: rejected

thread 'tests::association_depth_contract::g7_continuation_contract::trace_edge_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:490:5:
false continues next pattern then candidate
G7_TRACE_FALSIFIER false ends search: rejected

thread 'tests::association_depth_contract::g7_continuation_contract::trace_edge_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:456:5:
assertion `left == right` failed: print at completed root only
  left: 2
 right: 1
G7_TRACE_FALSIFIER early print completion: rejected

thread 'tests::association_depth_contract::g7_continuation_contract::trace_edge_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:514:13:
no streaming trace lines
G7_TRACE_FALSIFIER stream before final success: rejected

thread 'tests::association_depth_contract::g7_continuation_contract::trace_edge_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:466:5:
one named cold filter frame
G7_TRACE_FALSIFIER double freeze: rejected
ok
test tests::association_depth_contract::g7_continuation_contract::trace_flat_call_graph_and_outbound_boundaries ... G7_TRACE_SOURCE closed functions, outbound boundaries and per-file/combined acyclic walk passed
ok
test tests::association_depth_contract::g7_continuation_contract::trace_source_and_root_falsifiers_are_observed ... 
thread 'tests::association_depth_contract::g7_continuation_contract::trace_source_and_root_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:430:5:
assertion `left == right` failed: complete flat trace fields and roots
  left: ["TraceFilterLeaf.candidate_index:usize", "TraceFilterLeaf.candidates:TraceFilterCandidates", "TraceFilterLeaf.field_index:usize", "TraceFilterLeaf.original:Expr", "TraceFilterLeaf.pattern_index:usize", "TraceFilterLeaf.patterns:Vec<Expr>", "TraceFilterLeaf.selected:Vec<Expr>", "TraceFilterLeaf.underlying:Vec<Expr>", "TraceFilterLevel.emitted:Vec<Expr>", "TraceFilterLevel.remaining:Vec<Expr>", "TraceFilterState.current:TraceFilterLevel", "TraceFilterState.include_heads:bool", "TraceFilterState.levels:Option<CasesLevelSpec>", "TraceFilterState.mode:TraceOutput", "TraceFilterState.parents:Vec<TraceFilterLevel>", "TraceFilterState.pattern:Option<Expr>", "TraceFilterState.pending:TraceFilterLeaf", "TraceFilterSuspensionFrame.state:filter::TraceFilterState"]
 right: ["TraceFilterLeaf.candidate_index:usize", "TraceFilterLeaf.candidates:TraceFilterCandidates", "TraceFilterLeaf.field_index:usize", "TraceFilterLeaf.original:Expr", "TraceFilterLeaf.pattern_index:usize", "TraceFilterLeaf.patterns:Vec<Expr>", "TraceFilterLeaf.selected:Vec<Expr>", "TraceFilterLeaf.underlying:Vec<Expr>", "TraceFilterLevel.emitted:Vec<Expr>", "TraceFilterLevel.remaining:Vec<Expr>", "TraceFilterState.current:TraceFilterLevel", "TraceFilterState.include_heads:bool", "TraceFilterState.levels:Option<CasesLevelSpec>", "TraceFilterState.mode:TraceOutput", "TraceFilterState.original_value:Expr", "TraceFilterState.parents:Vec<TraceFilterLevel>", "TraceFilterState.pattern:Option<Expr>", "TraceFilterState.pending:TraceFilterLeaf", "TraceFilterSuspensionFrame.state:filter::TraceFilterState"]
G7_TRACE_FALSIFIER missing saved root: rejected

thread 'tests::association_depth_contract::g7_continuation_contract::trace_source_and_root_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:416:17:
trace root cannot skip tracing
G7_TRACE_FALSIFIER skipped root: rejected

thread 'tests::association_depth_contract::g7_continuation_contract::trace_source_and_root_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:430:5:
assertion `left == right` failed: complete flat trace fields and roots
  left: ["TraceFilterLeaf.candidate_index:usize", "TraceFilterLeaf.candidates:TraceFilterCandidates", "TraceFilterLeaf.field_index:usize", "TraceFilterLeaf.original:Expr", "TraceFilterLeaf.pattern_index:usize", "TraceFilterLeaf.patterns:Vec<Expr>", "TraceFilterLeaf.selected:Vec<Expr>", "TraceFilterLeaf.underlying:Vec<Expr>", "TraceFilterLevel.emitted:Vec<Expr>", "TraceFilterLevel.remaining:Vec<Expr>", "TraceFilterState.current:TraceFilterLevel", "TraceFilterState.include_heads:bool", "TraceFilterState.levels:Option<CasesLevelSpec>", "TraceFilterState.mode:TraceOutput", "TraceFilterState.original_value:Expr", "TraceFilterState.parents:Vec<TraceFilterState>", "TraceFilterState.pattern:Option<Expr>", "TraceFilterState.pending:TraceFilterLeaf", "TraceFilterSuspensionFrame.state:filter::TraceFilterState"]
 right: ["TraceFilterLeaf.candidate_index:usize", "TraceFilterLeaf.candidates:TraceFilterCandidates", "TraceFilterLeaf.field_index:usize", "TraceFilterLeaf.original:Expr", "TraceFilterLeaf.pattern_index:usize", "TraceFilterLeaf.patterns:Vec<Expr>", "TraceFilterLeaf.selected:Vec<Expr>", "TraceFilterLeaf.underlying:Vec<Expr>", "TraceFilterLevel.emitted:Vec<Expr>", "TraceFilterLevel.remaining:Vec<Expr>", "TraceFilterState.current:TraceFilterLevel", "TraceFilterState.include_heads:bool", "TraceFilterState.levels:Option<CasesLevelSpec>", "TraceFilterState.mode:TraceOutput", "TraceFilterState.original_value:Expr", "TraceFilterState.parents:Vec<TraceFilterLevel>", "TraceFilterState.pattern:Option<Expr>", "TraceFilterState.pending:TraceFilterLeaf", "TraceFilterSuspensionFrame.state:filter::TraceFilterState"]
G7_TRACE_FALSIFIER recursive state: rejected

thread 'tests::association_depth_contract::g7_continuation_contract::trace_source_and_root_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:393:21:
assertion `left == right` failed: closed candidate variants
  left: "Single(usize),Parts(Vec<Expr>),"
 right: "Single(Expr),Parts(Vec<Expr>),"
G7_TRACE_FALSIFIER candidate root variant: rejected

thread 'tests::association_depth_contract::g7_continuation_contract::trace_source_and_root_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:349:9:
recursive trace file
G7_TRACE_FALSIFIER direct recursion: rejected

thread 'tests::association_depth_contract::g7_continuation_contract::trace_source_and_root_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:355:5:
recursive combined trace modules
G7_TRACE_FALSIFIER cross-helper recursion: rejected

thread 'tests::association_depth_contract::g7_continuation_contract::trace_source_and_root_falsifiers_are_observed' panicked at crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:433:5:
self-reaching trace state
G7_TRACE_FALSIFIER self-reaching inventory: rejected
ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1114 filtered out; finished in 0.34s
```

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-582/tmp cargo test -p rum-runtime`; exit 0. Raw: `.temp/brief-582/runtime-final.log`.
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 28.85s
test result: ok. 1138 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 29.23s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.62s
```

Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-582/tmp cargo fmt --check`; exit 0. Raw: `.temp/brief-582/fmt-final.log`.
```text

```

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-582/tmp cargo test -p rum-stdlib --lib ownership_graph -- --nocapture`; exit 0. Raw: `.temp/brief-582/ownership-final.log`.
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.45s
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7671 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1112 filtered out; finished in 59.79s
```

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-582/tmp cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture`; exit 0. Raw: `.temp/brief-582/fingerprints-replay.log`.
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.36s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 0.18s
```

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-582/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`; exit 101. Raw: `.temp/brief-582/certificates-true-replay.log`.
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.63s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 17.90s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-582/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`; exit 0. Raw: `.temp/brief-582/certificates-true-final.log`.
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 17.81s
```

Freshly earned replay cells for hash=true:
```text
# body	status	identity	source	hash=true certificates	hash=false certificates	preconditions	strength	exclusions
C01	LocalCovered	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6	instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:30ab7b10ba2475f8", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:bea87d08d68e91d9;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:a4f75775f419f2a7;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:ec086d8b45c2eb18;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:30ab7b10ba2475f8", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:3d09ac85c9045e0f;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:5ef688116a348788;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:6802bedd40d9930f;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}	UNRECORDED	exact recorded build and instance; current own-body lint and direct marks; repository declaration equals compiled and installed runtime domain and registrations; sealed accounting modes; checked concrete Copy/FrameRootTrace descriptor; private typed owners and borrowed inputs; each direct contract caller precondition remains required	local retention across may-run-guest/may-grow calls and branch/error cleanup within admitted syntax; exact resolution and current direct marks; typed lease/frame transfer only at discharged boundaries; Runtime-enforced or Test-checked as recorded per contract; no inferred receipt from a return type	local only; callers and whole option consumer; arbitrary external R and host-installed declarations asserting host trust through the public certified-domain installer; guest-private and host-private allocation; process RSS and allocator overhead; interner storage; stack-safe destruction of deep Expr; universal allocation bounds derived from tests; between-checkpoint peaks; four escaping consumer copies; remaining N class; P bound unchanged
C02	LocalCovered	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6	instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:b2b55d2367924bca", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:f4caeadcdf433392;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:bafc9eb7b5411c85;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:14d665d89bf32844;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:ce8728e44345b4f0;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:920938e1b91a3d20;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:ebf1771746ea2733;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:d081f3e28da43831;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:bd94e844a3bca044;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}	UNRECORDED	exact recorded build and instance; current own-body lint and direct marks; repository declaration equals compiled and installed runtime domain and registrations; sealed accounting modes; checked concrete Copy/FrameRootTrace descriptor; private typed owners and borrowed inputs; each direct contract caller precondition remains required	local retention across may-run-guest/may-grow calls and branch/error cleanup within admitted syntax; exact resolution and current direct marks; typed lease/frame transfer only at discharged boundaries; Runtime-enforced or Test-checked as recorded per contract; no inferred receipt from a return type	local only; callers and whole option consumer; arbitrary external R and host-installed declarations asserting host trust through the public certified-domain installer; guest-private and host-private allocation; process RSS and allocator overhead; interner storage; stack-safe destruction of deep Expr; universal allocation bounds derived from tests; between-checkpoint peaks; four escaping consumer copies; remaining N class; P bound unchanged
```

Every promoted.tsv certificate cell change for hash=true:
```text
[
  {
    "file": "crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv",
    "row": 2,
    "id": "C01",
    "feature": "true",
    "column": 5,
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:0803ea849d9eb978\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:96f190d809c6f5a4;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:05e9b62028622c96;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:dde2f645389368b9;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:0803ea849d9eb978\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:463a040a5d268e56;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:32e43df16e3e35c9;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:59d5cd937e911b4e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:30ab7b10ba2475f8\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:bea87d08d68e91d9;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:a4f75775f419f2a7;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:ec086d8b45c2eb18;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:30ab7b10ba2475f8\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:3d09ac85c9045e0f;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:5ef688116a348788;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:6802bedd40d9930f;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "source_before": "crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6",
    "source_after": "crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6"
  },
  {
    "file": "crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv",
    "row": 3,
    "id": "C02",
    "feature": "true",
    "column": 5,
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:adfb9da067e2474a\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:187e8a84efe1df10;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:4e95a983bad51e78;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:867b06284138a315;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:a1369488b5dd6aa1;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:1b2ac220a12e6131;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:fa8a497e1810a682;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:3a78bafac1bbbe74;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:2f39889449021b15;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:b2b55d2367924bca\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:f4caeadcdf433392;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:bafc9eb7b5411c85;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:14d665d89bf32844;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:ce8728e44345b4f0;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:920938e1b91a3d20;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:ebf1771746ea2733;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:d081f3e28da43831;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:bd94e844a3bca044;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "source_before": "crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6",
    "source_after": "crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6"
  }
]
```

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-582/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`; exit 101. Raw: `.temp/brief-582/certificates-false-replay.log`.
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 41.13s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1125 filtered out; finished in 17.95s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-582/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`; exit 0. Raw: `.temp/brief-582/certificates-false-final.log`.
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.23s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1125 filtered out; finished in 18.09s
```

Freshly earned replay cells for hash=false:
```text
# body	status	identity	source	hash=true certificates	hash=false certificates	preconditions	strength	exclusions
C01	LocalCovered	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6	UNRECORDED	instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:b559052e7e3dd95e", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:77d59dfebd26af05;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:a4f75775f419f2a7;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:ec086d8b45c2eb18;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:b559052e7e3dd95e", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:611e7183534f3393;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:5ef688116a348788;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:6802bedd40d9930f;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}	exact recorded build and instance; current own-body lint and direct marks; repository declaration equals compiled and installed runtime domain and registrations; sealed accounting modes; checked concrete Copy/FrameRootTrace descriptor; private typed owners and borrowed inputs; each direct contract caller precondition remains required	local retention across may-run-guest/may-grow calls and branch/error cleanup within admitted syntax; exact resolution and current direct marks; typed lease/frame transfer only at discharged boundaries; Runtime-enforced or Test-checked as recorded per contract; no inferred receipt from a return type	local only; callers and whole option consumer; arbitrary external R and host-installed declarations asserting host trust through the public certified-domain installer; guest-private and host-private allocation; process RSS and allocator overhead; interner storage; stack-safe destruction of deep Expr; universal allocation bounds derived from tests; between-checkpoint peaks; four escaping consumer copies; remaining N class; P bound unchanged
C02	LocalCovered	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr	crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6	UNRECORDED	instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:d4c17b025c5226a0", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:6c13470263e5106e;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:bafc9eb7b5411c85;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:14d665d89bf32844;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:ce8728e44345b4f0;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:920938e1b91a3d20;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:ebf1771746ea2733;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:d081f3e28da43831;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:bd94e844a3bca044;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}	exact recorded build and instance; current own-body lint and direct marks; repository declaration equals compiled and installed runtime domain and registrations; sealed accounting modes; checked concrete Copy/FrameRootTrace descriptor; private typed owners and borrowed inputs; each direct contract caller precondition remains required	local retention across may-run-guest/may-grow calls and branch/error cleanup within admitted syntax; exact resolution and current direct marks; typed lease/frame transfer only at discharged boundaries; Runtime-enforced or Test-checked as recorded per contract; no inferred receipt from a return type	local only; callers and whole option consumer; arbitrary external R and host-installed declarations asserting host trust through the public certified-domain installer; guest-private and host-private allocation; process RSS and allocator overhead; interner storage; stack-safe destruction of deep Expr; universal allocation bounds derived from tests; between-checkpoint peaks; four escaping consumer copies; remaining N class; P bound unchanged
```

Every promoted.tsv certificate cell change for hash=false:
```text
[
  {
    "file": "crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv",
    "row": 2,
    "id": "C01",
    "feature": "false",
    "column": 6,
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:2d850ccf5dc3fcab\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:434fe62bdfbf766a;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:05e9b62028622c96;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:dde2f645389368b9;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:2d850ccf5dc3fcab\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:18cfd631754a3fd4;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:32e43df16e3e35c9;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:59d5cd937e911b4e;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:b559052e7e3dd95e\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:77d59dfebd26af05;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:a4f75775f419f2a7;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:ec086d8b45c2eb18;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:b559052e7e3dd95e\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:611e7183534f3393;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:5ef688116a348788;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:6802bedd40d9930f;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "source_before": "crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6",
    "source_after": "crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6"
  },
  {
    "file": "crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv",
    "row": 3,
    "id": "C02",
    "feature": "false",
    "column": 6,
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:478cd341dd84a5a9\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:7827f070bc028adf;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:4e95a983bad51e78;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:867b06284138a315;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:a1369488b5dd6aa1;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:1b2ac220a12e6131;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:fa8a497e1810a682;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:3a78bafac1bbbe74;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:2f39889449021b15;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:d4c17b025c5226a0\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:6c13470263e5106e;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:bafc9eb7b5411c85;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:14d665d89bf32844;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:ce8728e44345b4f0;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:920938e1b91a3d20;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:ebf1771746ea2733;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:d081f3e28da43831;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:bd94e844a3bca044;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "source_before": "crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6",
    "source_after": "crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6"
  }
]
```

Command: `python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-582/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture`; exit 0. Raw: `.temp/brief-582/workspace-final.log`.
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4m 11s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.48s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.79s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.74s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.41s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.97s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.61s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.58s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.01s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
test result: ok. 1138 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.00s
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7671 rows
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
PRODUCTION_WALL_SECONDS=17.359
test result: ok. 1079 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1202.58s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.92s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.55s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.43s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.76s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Workspace result-line totals (including doctests): 3306 passed, 0 failed, 58 ignored, across 46 completed binaries/doctest groups. 9 native ZeroMQ skip notices are included in the harness pass counts; they are skips, not successful transport execution. Full raw broad-suite output remains in the named log. Intentional falsifier panics and compiler errors from negative proof fixtures are expected rejection evidence; surrounding test results, rejection markers and each command’s exit status distinguish them from failed gates.
```text
{
  "baseline": "red.log",
  "selected": "green-08.log",
  "cases": 34,
  "all_queued_values_logs_errors_terminations_messages_prints_and_expected_prompts_unchanged": true,
  "all_final_resumed_comparisons_equal": true
}
```


Ownership reconciliation, verbatim before/after rows (including source-position moves, all new constructs and the eight named retirements):
```text
{
  "host_payloads.tsv": {
    "before_count": 65,
    "after_count": 65,
    "changed": [],
    "removed": [],
    "added": []
  },
  "unknown_attributes.tsv": {
    "before_count": 1489,
    "after_count": 1494,
    "changed": [
      {
        "row": 108,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t12\t1\tunresolved attribute/expansion derive (Clone , FrameRootTrace)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t13\t1\tunresolved attribute/expansion derive (Clone , FrameRootTrace)",
        "reason": "source position only"
      },
      {
        "row": 109,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t21\t1\tunresolved attribute/expansion derive (Clone , Copy , PartialEq , Eq , FrameRootTrace)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t22\t1\tunresolved attribute/expansion derive (Clone , Copy , PartialEq , Eq , FrameRootTrace)",
        "reason": "source position only"
      }
    ],
    "removed": [],
    "added": [
      "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace/filter/state.rs\t5\t1\tunresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace)",
      "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace/filter/state.rs\t17\t1\tunresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace)",
      "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace/filter/state.rs\t23\t1\tunresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace)",
      "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace/filter/state.rs\t35\t1\tunresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace)",
      "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs\t45\t1\tunresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)"
    ]
  },
  "unknown_captures.tsv": {
    "before_count": 2951,
    "after_count": 2949,
    "changed": [
      {
        "row": 250,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t643\t18\tclosure captures: |(key , value)| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t307\t18\tclosure captures: |(key , value)| (capture types/modes unresolved)",
        "reason": "source position only"
      },
      {
        "row": 251,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t666\t14\tclosure captures: |field| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t330\t14\tclosure captures: |field| (capture types/modes unresolved)",
        "reason": "source position only"
      },
      {
        "row": 252,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t673\t19\tclosure captures: |(entry_key , value)| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t337\t19\tclosure captures: |(entry_key , value)| (capture types/modes unresolved)",
        "reason": "source position only"
      },
      {
        "row": 253,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t704\t33\tclosure captures: |current| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t368\t33\tclosure captures: |current| (capture types/modes unresolved)",
        "reason": "source position only"
      },
      {
        "row": 254,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t711\t33\tclosure captures: |current| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t375\t33\tclosure captures: |current| (capture types/modes unresolved)",
        "reason": "source position only"
      },
      {
        "row": 255,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t729\t38\tclosure captures: |(depth , _)| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t393\t38\tclosure captures: |(depth , _)| (capture types/modes unresolved)",
        "reason": "source position only"
      },
      {
        "row": 256,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t732\t14\tclosure captures: |(depth , line)| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t396\t14\tclosure captures: |(depth , line)| (capture types/modes unresolved)",
        "reason": "source position only"
      }
    ],
    "removed": [
      {
        "row": 248,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t552\t26\tclosure captures: |candidate| (capture types/modes unresolved)",
        "reason": "brief-582 I03 named removed recursive filter construct"
      },
      {
        "row": 249,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t593\t26\tclosure captures: |candidate| (capture types/modes unresolved)",
        "reason": "brief-582 I03 named removed recursive filter construct"
      }
    ],
    "added": []
  },
  "unknown_macros.tsv": {
    "before_count": 3142,
    "after_count": 3139,
    "changed": [
      {
        "row": 166,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t70\t23\tunexpanded macro crate :: resume_or_suspend! (ends at 78:10)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t71\t23\tunexpanded macro crate :: resume_or_suspend! (ends at 79:10)",
        "reason": "source position only"
      },
      {
        "row": 167,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t186\t32\tunexpanded macro vec! (ends at 186:54)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t166\t32\tunexpanded macro vec! (ends at 166:54)",
        "reason": "source position only"
      },
      {
        "row": 168,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t232\t32\tunexpanded macro vec! (ends at 232:54)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t212\t32\tunexpanded macro vec! (ends at 212:54)",
        "reason": "source position only"
      },
      {
        "row": 175,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t607\t30\tunexpanded macro vec! (ends at 613:6)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t271\t30\tunexpanded macro vec! (ends at 277:6)",
        "reason": "source position only"
      },
      {
        "row": 176,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t617\t30\tunexpanded macro vec! (ends at 621:6)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t281\t30\tunexpanded macro vec! (ends at 285:6)",
        "reason": "source position only"
      },
      {
        "row": 177,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t628\t9\tunexpanded macro format! (ends at 628:44)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t292\t9\tunexpanded macro format! (ends at 292:44)",
        "reason": "source position only"
      },
      {
        "row": 178,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t630\t30\tunexpanded macro vec! (ends at 635:6)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t294\t30\tunexpanded macro vec! (ends at 299:6)",
        "reason": "source position only"
      },
      {
        "row": 179,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t643\t65\tunexpanded macro vec! (ends at 643:95)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t307\t65\tunexpanded macro vec! (ends at 307:95)",
        "reason": "source position only"
      },
      {
        "row": 180,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t649\t37\tunexpanded macro vec! (ends at 649:55)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t313\t37\tunexpanded macro vec! (ends at 313:55)",
        "reason": "source position only"
      },
      {
        "row": 181,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t732\t30\tunexpanded macro format! (ends at 732:97)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t396\t30\tunexpanded macro format! (ends at 396:97)",
        "reason": "source position only"
      },
      {
        "row": 182,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t766\t18\tunexpanded macro vec! (ends at 766:37)",
        "after": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t430\t18\tunexpanded macro vec! (ends at 430:37)",
        "reason": "source position only"
      }
    ],
    "removed": [
      {
        "row": 169,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t519\t23\tunexpanded macro vec! (ends at 519:33)",
        "reason": "brief-582 I03 named removed recursive filter construct"
      },
      {
        "row": 170,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t552\t70\tunexpanded macro vec! (ends at 552:85)",
        "reason": "brief-582 I03 named removed recursive filter construct"
      },
      {
        "row": 171,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t559\t57\tunexpanded macro vec! (ends at 559:67)",
        "reason": "brief-582 I03 named removed recursive filter construct"
      },
      {
        "row": 172,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t575\t25\tunexpanded macro vec! (ends at 575:74)",
        "reason": "brief-582 I03 named removed recursive filter construct"
      },
      {
        "row": 173,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t593\t70\tunexpanded macro vec! (ends at 593:85)",
        "reason": "brief-582 I03 named removed recursive filter construct"
      },
      {
        "row": 174,
        "before": "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs\t596\t32\tunexpanded macro vec! (ends at 596:81)",
        "reason": "brief-582 I03 named removed recursive filter construct"
      }
    ],
    "added": [
      "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace/filter.rs\t131\t5\tunexpanded macro crate :: resume_or_suspend! (ends at 131:84)",
      "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs\t91\t34\tunexpanded macro matches! (ends at 91:85)",
      "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs\t171\t49\tunexpanded macro vec! (ends at 171:64)"
    ]
  },
  "unknown_storage.tsv": {
    "before_count": 88,
    "after_count": 89,
    "changed": [],
    "removed": [],
    "added": [
      "UnknownConstruct\tcrates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs\t47\t23\trum_runtime::runtime::debug_tools::trace::resume_frames::TraceFilterSuspensionFrame.state: unresolved/ambiguous type (): filter :: TraceFilterState"
    ]
  },
  "self_reaching.tsv": {
    "changed": [],
    "before_count": 59,
    "after_count": 59
  },
  "host_payload_ledger.tsv": {
    "changed": [],
    "added": []
  }
}
```

The retired constructs are trace.rs 552:26 and 593:26 closures, and vec! at 519:23, 552:70, 559:57, 575:25, 593:70 and 596:32. The deleted bodies and absence of source mappings were checked. Replacements are the disclosed new driver/leaf/state/frame rows. Unknown constructs remain unknown. There is no new UnknownHostPayload, self-reaching row, row in untouched code, changed host disposition or retention reclassification.

Retention evidence cells only; all three trace rows remain Unsplit/Pending:
```text
[
  {
    "row": 301,
    "before": "Unsplit:Trace\tTrace\tUnsplit\tPending\tcrates/rum-stdlib/src/catalog/memory_growth_contract.rs\tInherited membership; subclass not re-audited\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs:106,143,144\trum_runtime::runtime::builtin_adapters::builtin_trace",
    "after": "Unsplit:Trace\tTrace\tUnsplit\tPending\tcrates/rum-stdlib/src/catalog/memory_growth_contract.rs\tInherited membership; subclass not re-audited\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs:108,114;crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:130,131\trum_runtime::runtime::builtin_adapters::builtin_trace"
  },
  {
    "row": 302,
    "before": "Unsplit:TraceDetail\tTraceDetail\tUnsplit\tPending\tcrates/rum-stdlib/src/catalog/memory_growth_contract.rs\tInherited membership; subclass not re-audited\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs:106,143,144\trum_runtime::runtime::builtin_adapters::builtin_trace_detail",
    "after": "Unsplit:TraceDetail\tTraceDetail\tUnsplit\tPending\tcrates/rum-stdlib/src/catalog/memory_growth_contract.rs\tInherited membership; subclass not re-audited\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs:108,114;crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:130,131\trum_runtime::runtime::builtin_adapters::builtin_trace_detail"
  },
  {
    "row": 303,
    "before": "Unsplit:TracePrint\tTracePrint\tUnsplit\tPending\tcrates/rum-stdlib/src/catalog/memory_growth_contract.rs\tInherited membership; subclass not re-audited\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs:106,143,144\trum_runtime::runtime::builtin_adapters::builtin_trace_print",
    "after": "Unsplit:TracePrint\tTracePrint\tUnsplit\tPending\tcrates/rum-stdlib/src/catalog/memory_growth_contract.rs\tInherited membership; subclass not re-audited\tcrates/rum-runtime/src/runtime/debug_tools/trace.rs:108,114;crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:130,131\trum_runtime::runtime::builtin_adapters::builtin_trace_print"
  }
]
```

Every refreshed contract fingerprint (other columns unchanged):
```text
[
  {
    "row": 2,
    "id": "B01",
    "before": "fnv1a64:70b3be864e1d27f8",
    "after": "fnv1a64:dd1ab3ba48892605"
  },
  {
    "row": 3,
    "id": "B02",
    "before": "fnv1a64:9e288efb617c5781",
    "after": "fnv1a64:7480471aa298c050"
  },
  {
    "row": 4,
    "id": "B03",
    "before": "fnv1a64:2bbc7efdbe2492fa",
    "after": "fnv1a64:0977ccb66394cadb"
  },
  {
    "row": 5,
    "id": "B04",
    "before": "fnv1a64:5423c67c834646f0",
    "after": "fnv1a64:550ad67307ba59ad"
  },
  {
    "row": 6,
    "id": "B05",
    "before": "fnv1a64:2f39889449021b15",
    "after": "fnv1a64:bd94e844a3bca044"
  },
  {
    "row": 7,
    "id": "B06",
    "before": "fnv1a64:ac682771911f0274",
    "after": "fnv1a64:427160595d077c31"
  },
  {
    "row": 8,
    "id": "B07",
    "before": "fnv1a64:da1a9e31b6f45fc5",
    "after": "fnv1a64:7d94979446aaa714"
  },
  {
    "row": 9,
    "id": "B08",
    "before": "fnv1a64:b866e65e5d2ebe1e",
    "after": "fnv1a64:832d8589c38181bf"
  },
  {
    "row": 10,
    "id": "B09",
    "before": "fnv1a64:e71769fb7f5a43cc",
    "after": "fnv1a64:6970137158bf9cd9"
  },
  {
    "row": 11,
    "id": "B10",
    "before": "fnv1a64:4ae584cb878a018b",
    "after": "fnv1a64:6c59f4063fe314b6"
  },
  {
    "row": 12,
    "id": "B11",
    "before": "fnv1a64:157b77d99d3a6cf0",
    "after": "fnv1a64:166287d021ae7fad"
  },
  {
    "row": 13,
    "id": "B12",
    "before": "fnv1a64:a623e9f75b62d8cc",
    "after": "fnv1a64:287c936d34c831d9"
  },
  {
    "row": 14,
    "id": "I01",
    "before": "fnv1a64:4e95a983bad51e78",
    "after": "fnv1a64:bafc9eb7b5411c85"
  },
  {
    "row": 15,
    "id": "I02",
    "before": "fnv1a64:867b06284138a315",
    "after": "fnv1a64:14d665d89bf32844"
  },
  {
    "row": 16,
    "id": "I03",
    "before": "fnv1a64:2cd5639c2907540c",
    "after": "fnv1a64:8193d31309477699"
  },
  {
    "row": 17,
    "id": "I04",
    "before": "fnv1a64:a1369488b5dd6aa1",
    "after": "fnv1a64:ce8728e44345b4f0"
  },
  {
    "row": 18,
    "id": "P01",
    "before": "fnv1a64:ae4e36eb37420814",
    "after": "fnv1a64:43c5e12ed1ef5251"
  },
  {
    "row": 19,
    "id": "P02",
    "before": "fnv1a64:5d14933a776d609b",
    "after": "fnv1a64:d6c1771ca9287b26"
  },
  {
    "row": 20,
    "id": "P03",
    "before": "fnv1a64:1e1a72cdf9b66642",
    "after": "fnv1a64:18f207d36ab28973"
  },
  {
    "row": 21,
    "id": "P04",
    "before": "fnv1a64:9303ad10e6257381",
    "after": "fnv1a64:695b65302741dc50"
  },
  {
    "row": 22,
    "id": "P05",
    "before": "fnv1a64:9937d43ddc1e1248",
    "after": "fnv1a64:3bd45f6a1726aa35"
  },
  {
    "row": 23,
    "id": "P06",
    "before": "fnv1a64:f6a30ec7e792724c",
    "after": "fnv1a64:78fbb83dc0f7cb59"
  },
  {
    "row": 24,
    "id": "P070",
    "before": "fnv1a64:30f0ab09c607ec4c",
    "after": "fnv1a64:b349547f9f6d4559"
  },
  {
    "row": 25,
    "id": "P080",
    "before": "fnv1a64:7818c3ee3e536ccb",
    "after": "fnv1a64:2143e68b1c3d5276"
  },
  {
    "row": 26,
    "id": "P090",
    "before": "fnv1a64:3e0f22bc06a715c1",
    "after": "fnv1a64:084a666c51183b10"
  },
  {
    "row": 27,
    "id": "P100",
    "before": "fnv1a64:70fc79b2c6eb1e60",
    "after": "fnv1a64:3baee80a61fe7e7d"
  },
  {
    "row": 28,
    "id": "P110",
    "before": "fnv1a64:8737a7e77b066c33",
    "after": "fnv1a64:5a89f727ce2a610e"
  },
  {
    "row": 29,
    "id": "P120",
    "before": "fnv1a64:c93b56bf3120f9be",
    "after": "fnv1a64:bff5f31523e45cdf"
  },
  {
    "row": 30,
    "id": "P130",
    "before": "fnv1a64:7de7f78ee5a839c5",
    "after": "fnv1a64:2161f0f1755e8114"
  },
  {
    "row": 31,
    "id": "P140",
    "before": "fnv1a64:381c9c1d1f870fc9",
    "after": "fnv1a64:642ee63d1b7d6188"
  },
  {
    "row": 32,
    "id": "P150",
    "before": "fnv1a64:228b7470fe89bae1",
    "after": "fnv1a64:61317430935327b0"
  },
  {
    "row": 33,
    "id": "P071",
    "before": "fnv1a64:96721f61c1537de5",
    "after": "fnv1a64:c6be0cda740d96b4"
  },
  {
    "row": 34,
    "id": "P081",
    "before": "fnv1a64:ef5b309a6f6b11ec",
    "after": "fnv1a64:d2bea43c999cfdf9"
  },
  {
    "row": 35,
    "id": "P091",
    "before": "fnv1a64:5ef7452263fe8c54",
    "after": "fnv1a64:bd07c37a271e6911"
  },
  {
    "row": 36,
    "id": "P101",
    "before": "fnv1a64:7f3a98b4b06a0933",
    "after": "fnv1a64:528ce7f5038dfe0e"
  },
  {
    "row": 37,
    "id": "P111",
    "before": "fnv1a64:53cfd9c8b4590abc",
    "after": "fnv1a64:674f013cccdc3869"
  },
  {
    "row": 38,
    "id": "P121",
    "before": "fnv1a64:151c0ec728e86c27",
    "after": "fnv1a64:50cbaa9c687761d2"
  },
  {
    "row": 39,
    "id": "P131",
    "before": "fnv1a64:f1cde8e2b99a6870",
    "after": "fnv1a64:f2b4f8d93e0e7b2d"
  },
  {
    "row": 40,
    "id": "P141",
    "before": "fnv1a64:c637bad8b05bb7b4",
    "after": "fnv1a64:b7d1f23f0ad432f1"
  },
  {
    "row": 41,
    "id": "P151",
    "before": "fnv1a64:a7b3d5b5369eee7e",
    "after": "fnv1a64:6b80406265f9ac1f"
  },
  {
    "row": 42,
    "id": "B13",
    "before": "fnv1a64:3a78bafac1bbbe74",
    "after": "fnv1a64:d081f3e28da43831"
  },
  {
    "row": 43,
    "id": "R01",
    "before": "fnv1a64:1b2ac220a12e6131",
    "after": "fnv1a64:920938e1b91a3d20"
  },
  {
    "row": 44,
    "id": "R02",
    "before": "fnv1a64:fa8a497e1810a682",
    "after": "fnv1a64:ebf1771746ea2733"
  },
  {
    "row": 45,
    "id": "R03",
    "before": "fnv1a64:e5210878fe71e53d",
    "after": "fnv1a64:845bd136f9cd5ddc"
  },
  {
    "row": 46,
    "id": "R040",
    "before": "fnv1a64:9eb7275245b0ffd1",
    "after": "fnv1a64:daa0a8440b6706c0"
  },
  {
    "row": 47,
    "id": "R050",
    "before": "fnv1a64:89a5c4b53e2f2ea8",
    "after": "fnv1a64:16d70e24e25d2795"
  },
  {
    "row": 48,
    "id": "R060",
    "before": "fnv1a64:9b5a39bd2b01a439",
    "after": "fnv1a64:a97fb10338312698"
  },
  {
    "row": 49,
    "id": "R070",
    "before": "fnv1a64:57c8f8b4510cd0ec",
    "after": "fnv1a64:3b2c6c567b3ebcf9"
  },
  {
    "row": 50,
    "id": "R080",
    "before": "fnv1a64:683afd7b856233a1",
    "after": "fnv1a64:958b91d712ca7df0"
  },
  {
    "row": 51,
    "id": "R090",
    "before": "fnv1a64:64593d65eef666a0",
    "after": "fnv1a64:690a4ef5a6f6223d"
  },
  {
    "row": 52,
    "id": "R100",
    "before": "fnv1a64:f0e2235067fa3d7a",
    "after": "fnv1a64:ce9d71090d6a755b"
  },
  {
    "row": 53,
    "id": "R110",
    "before": "fnv1a64:c42f329df973fda3",
    "after": "fnv1a64:78b604a5002daa5e"
  },
  {
    "row": 54,
    "id": "R041",
    "before": "fnv1a64:20b21e24d8e37d74",
    "after": "fnv1a64:b6bb570ca4cbf731"
  },
  {
    "row": 55,
    "id": "R051",
    "before": "fnv1a64:cc3150d193d52241",
    "after": "fnv1a64:966c9481de464790"
  },
  {
    "row": 56,
    "id": "R061",
    "before": "fnv1a64:7e4436065256d66e",
    "after": "fnv1a64:7e863ee20445712f"
  },
  {
    "row": 57,
    "id": "R071",
    "before": "fnv1a64:d92dc06f3939307f",
    "after": "fnv1a64:275e3dd25b74877a"
  },
  {
    "row": 58,
    "id": "R081",
    "before": "fnv1a64:2ce6e3076296f360",
    "after": "fnv1a64:f799515efdaa537d"
  },
  {
    "row": 59,
    "id": "R091",
    "before": "fnv1a64:ef653f211e3c1289",
    "after": "fnv1a64:9dda44f9757fc7c8"
  },
  {
    "row": 60,
    "id": "R101",
    "before": "fnv1a64:346c0a27af893179",
    "after": "fnv1a64:0b2a5581e52b4658"
  },
  {
    "row": 61,
    "id": "R111",
    "before": "fnv1a64:75e7dc81a3324ef6",
    "after": "fnv1a64:f59ccec5dcf7bf87"
  },
  {
    "row": 62,
    "id": "R12",
    "before": "fnv1a64:3e7b22dc6b7585b6",
    "after": "fnv1a64:99a34d0f078ef0c7"
  },
  {
    "row": 63,
    "id": "R13",
    "before": "fnv1a64:a420650e35bdf68a",
    "after": "fnv1a64:bfce75b544491f8b"
  },
  {
    "row": 64,
    "id": "R14",
    "before": "fnv1a64:8b548828a39bde60",
    "after": "fnv1a64:5606f6803eaf3e7d"
  },
  {
    "row": 65,
    "id": "R15",
    "before": "fnv1a64:3300a89ef1c1b804",
    "after": "fnv1a64:8229575fa1a4a7a1"
  },
  {
    "row": 66,
    "id": "O010",
    "before": "fnv1a64:59d5cd937e911b4e",
    "after": "fnv1a64:6802bedd40d9930f"
  },
  {
    "row": 67,
    "id": "O0200",
    "before": "fnv1a64:32e43df16e3e35c9",
    "after": "fnv1a64:5ef688116a348788"
  },
  {
    "row": 68,
    "id": "O011",
    "before": "fnv1a64:dde2f645389368b9",
    "after": "fnv1a64:ec086d8b45c2eb18"
  },
  {
    "row": 69,
    "id": "O0210",
    "before": "fnv1a64:05e9b62028622c96",
    "after": "fnv1a64:a4f75775f419f2a7"
  }
]
```

C01 and C02 bodies, rules, classifications and finite consumer domain are unchanged. Three marks per feature configuration were freshly earned and installed, then replayed to green. The certificate files above contain the actual cells, not hand-created marks.

### 3. Repaired sites and every measured row

```text
id	old_caller	old_callee	live_match	owner	frame	status
P0199	finish_structured_trace_capture_expr	filter_structured_trace_expr	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame	repaired; remaining Lost 13 -> 0 predicted
P0200	filter_structured_trace_expr	filter_structured_trace_items	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame	repaired; remaining Lost 13 -> 0 predicted
P0201	filter_structured_trace_detail_expr	filter_structured_trace_detail_items	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame	repaired; remaining Lost 13 -> 0 predicted
P0202	filter_structured_trace_detail_items	filter_structured_trace_detail_items	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame	repaired; remaining Lost 13 -> 0 predicted
P0203	filter_structured_trace_detail_items	trace_detail_item_matches	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame	repaired; remaining Lost 13 -> 0 predicted
P0204	trace_detail_item_matches	trace_detail_underlying_expr_matches	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame	repaired; remaining Lost 13 -> 0 predicted
P0205	trace_detail_underlying_expr_matches	trace_detail_candidate_matches	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame	repaired; remaining Lost 13 -> 0 predicted
P0206	trace_detail_underlying_expr_matches	trace_detail_candidate_matches	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame	repaired; remaining Lost 13 -> 0 predicted
P0208	trace_detail_candidate_matches	match_expr_owned	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame	repaired; remaining Lost 13 -> 0 predicted
P0209	filter_structured_trace_items	filter_structured_trace_items	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame	repaired; remaining Lost 13 -> 0 predicted
P0210	filter_structured_trace_items	filter_structured_trace_leaf	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame	repaired; remaining Lost 13 -> 0 predicted
P0211	filter_structured_trace_leaf	match_expr_owned	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame	repaired; remaining Lost 13 -> 0 predicted
P0213	filter_structured_trace_leaf	match_expr_owned	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	crates/rum-runtime/src/runtime/debug_tools/trace/resume_frames.rs:TraceFilterSuspensionFrame	repaired; remaining Lost 13 -> 0 predicted
P0207	dependent local-tail routing	old recursive walker	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	TraceFilterSuspensionFrame	retired helper; no added Lost count
P0212	dependent local-tail routing	old recursive walker	crates/rum-runtime/src/runtime/debug_tools/trace/filter/leaf.rs:141	crates/rum-runtime/src/runtime/debug_tools/trace/filter.rs:76	TraceFilterSuspensionFrame	retired helper; no added Lost count
```

P0207/P0212 were dependent local-tail routes retired with the old walkers; they do not add Lost counts. P0198 and the existing capture abandonment identity remain intact. This local repair does not recensus the separate Unsettled population or claim retention coverage or universal stack safety.

Supplied base hashes (verified, never rebuilt):
```text
c49b3a1d0a695e7cf5d9fe620749f914d3994f72322d1f3e60473c8c8bf522f9  rum-cli
01b3a5b1a20928cdd3eaf2e0332dce88155305651698234b2e0cadb4a06f3251  rum-cli-alloc
```

The panel contains all 33 trace statements, nine Sow statements and the supplied Dot, Differences and Take controls. Each exact result and print stream was checked before measurement. Per-label N is fixed after the passing 2N calibration. Raw files precede reduction; slopes are (count(2N) - count(N)) / N. Two independent base/base pairs set each label’s floor. Both candidate samples must exceed max(+0.05%, floor) to regress. Allocations and bytes have their own strict comparison. All perf-stat instruction processes use CPU 2, cpu_core/instructions/u and /dev/null stdin, serially with profiling flags removed. No timing suite was run.

Every noise-floor row:
```text
label	N	floor	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.0026832673071019997	4166.56215	4166.67395	0.0026832673071019997	4166.5094	4166.50495	-0.00010680403121909876	False
g7-sow-default-tag	20000	0.053219646861402126	16569.29455	16562.4249	-0.04146012359951268	16558.0768	16566.88895	0.053219646861402126	False
g7-sow-tag-hit	20000	0.0761705788903666	19867.4478	19852.31465	-0.0761705788903666	19868.00525	19876.6139	0.04332921142147228	False
g7-sow-tag-decline	20000	0.031637928628791556	17147.7758	17147.7842	4.898594485602814e-05	17148.72065	17143.29515	-0.031637928628791556	False
g7-sow-predicate-hit	20000	0.05842085887823867	33268.42565	33287.86135	0.05842085887823867	33282.32265	33285.78645	0.010407326545158746	False
g7-sow-predicate-decline	20000	0.05822286054277814	30630.06495	30612.23125	-0.05822286054277814	30630.862	30614.90935	-0.052080316903913015	False
g7-sow-pattern-list	20000	0.06664157314812957	41306.94805	41334.47565	0.06664157314812957	41315.24155	41306.57905	-0.020966838568559076	False
g7-sow-nested-scopes	20000	0.04908701694480433	46391.89845	46389.8437	-0.004429113851023292	46404.42915	46381.6506	-0.04908701694480433	False
g7-sow-repeated-tag	10000	0.2613144753496277	50779.4412	50782.7485	0.0065130689149928145	50909.1966	50776.1635	-0.2613144753496277	False
g7-trace-plain	2000	0.34826975444315345	28003.1495	27905.623	-0.34826975444315345	27905.794	27958.959	0.1905159910518872	False
g7-trace-pattern-hit	2000	0.11862567989811001	29278.652	29313.384	0.11862567989811001	29316.814	29318.095	0.0043695061816784175	False
g7-trace-pattern-decline	2000	0.036326623799620705	26426.8985	26436.4985	0.036326623799620705	26488.1165	26485.4155	-0.010197025522751257	False
g7-trace-alternatives	2000	0.5084391171986558	32988.4985	33116.358	0.38758811650672076	33256.2925	33087.2045	-0.5084391171986558	False
g7-trace-predicate-hit	2000	0.1337796438685146	154955.189	155162.4875	0.1337796438685146	154980.551	154964.986	-0.010043195678144379	False
g7-trace-predicate-decline	2000	0.31619069451558346	139102.291	139542.1195	0.31619069451558346	139108.624	139332.6385	0.16103566663128177	False
g7-trace-levels	2000	0.01763534787542209	96139.3	96156.2545	0.01763534787542209	96201.9975	96190.369	-0.012087586850773047	False
g7-trace-negative-levels	2000	0.23982683112320213	99378.28	99363.938	-0.014431724920183076	99376.287	99614.618	0.23982683112320213	False
g7-trace-heads	2000	0.06624326505909606	114462.957	114387.133	-0.06624326505909606	114357.2365	114362.2165	0.004354774697623576	False
g7-trace-nested	2000	0.48524986699812755	91322.024	90878.884	-0.48524986699812755	91267.9625	91358.882	0.0996181984450395	False
g7-trace-nested-filter	2000	0.07699978796587015	106343.1465	106267.829	-0.07082496849010189	106202.2405	106284.016	0.07699978796587015	False
g7-tracedetail-plain	2000	0.19662089083332113	36450.2465	36402.7775	-0.1302295719728619	36402.2865	36473.861	0.19662089083332113	False
g7-tracedetail-pattern-hit	2000	0.1844833388357614	40601.7695	40526.866	-0.1844833388357614	40521.9075	40525.7555	0.009496097882366605	False
g7-tracedetail-pattern-decline	2000	0.9269690649497342	32731.513	33034.924	0.9269690649497342	32689.8575	32740.5375	0.15503279572264272	False
g7-tracedetail-alternatives	2000	0.7124896924413648	44686.2605	44367.8755	-0.7124896924413648	44364.2095	44368.4515	0.00956176171695855	False
g7-tracedetail-predicate-hit	2000	0.2116941011157225	149546.5975	149299.437	-0.16527323532051774	149751.929	149434.913	-0.2116941011157225	False
g7-tracedetail-predicate-decline	2000	0.0298367056246418	158289.9285	158337.157	0.0298367056246418	158305.592	158346.6715	0.025949493938282053	False
g7-tracedetail-levels	2000	1.2538831561790809	95496.5775	94299.162	-1.2538831561790809	94225.371	94208.133	-0.018294435794785624	False
g7-tracedetail-negative-levels	2000	0.11894438130271023	95079.3125	95192.404	0.11894438130271023	95217.964	95286.924	0.07242330869414726	False
g7-tracedetail-heads	2000	0.1350390526104528	96173.6605	96303.5325	0.1350390526104528	96310.793	96316.668	0.006100043221524132	False
g7-tracedetail-nested	2000	0.03457663379686782	179607.065	179578.5135	-0.01589664638191879	179666.1305	179728.253	0.03457663379686782	False
g7-tracedetail-nested-filter	2000	0.13737786126142293	206538.643	206361.923	-0.08556268087808183	206361.125	206644.6195	0.13737786126142293	False
g7-traceprint-plain	2000	0.19779990078052645	30622.1655	30636.53	0.04690883144760072	30670.3895	30609.7235	-0.19779990078052645	False
g7-traceprint-pattern-hit	2000	0.5541069117105613	29362.8615	29402.461	0.13486253715428997	29251.3045	29413.388	0.5541069117105613	False
g7-traceprint-pattern-decline	2000	1.3117046813320377	25254.6175	25585.8835	1.3117046813320377	25304.0085	25273.5135	-0.12051450267256403	False
g7-traceprint-alternatives	2000	0.20413214834034132	33119.7515	33145.787	0.07861019126305102	33057.0175	33124.4975	0.20413214834034132	False
g7-traceprint-predicate-hit	2000	0.3097151647886598	170392.7315	170228.8335	-0.09618837526528257	170342.1595	170869.735	0.3097151647886598	False
g7-traceprint-predicate-decline	2000	0.16378302963143687	138391.892	138420.073	0.02036318717284491	138620.2835	138393.247	-0.16378302963143687	False
g7-traceprint-levels	2000	0.35921481706462366	107828.375	108215.7105	0.35921481706462366	108338.2825	108278.181	-0.05547577330294873	False
g7-traceprint-negative-levels	2000	0.6931639432600001	113107.297	113891.316	0.6931639432600001	113049.5255	113076.568	0.023920931892806863	False
g7-traceprint-heads	2000	0.012223858486204442	117231.2855	117241.0775	0.008352719121207386	117237.941	117252.272	0.012223858486204442	False
g7-traceprint-nested	2000	0.09411825290817077	103827.3545	103826.7485	-0.0005836612161802535	103772.6445	103870.3135	0.09411825290817077	False
g7-traceprint-nested-filter	2000	0.42001189902265423	116303.4075	115937.2645	-0.3148170873669365	116522.175	116032.768	-0.42001189902265423	False
dot-three-no-input	20000	0.0009633022971966554	253029.87495	253030.00305	5.062643295694613e-05	253030.64335	253028.2059	-0.0009633022971966554	False
differences-no-input	20000	0.0725462958578671	9416.04345	9416.07355	0.00031966717399800615	9408.9573	9415.78315	0.0725462958578671	False
take-multispec-small	20000	0.03114976305030881	49199.8802	49207.95415	0.016410507438591182	49199.25065	49183.9252	-0.03114976305030881	False
```

Candidate sequence: 01 flat filter; 02 direct unfiltered-leaf emission; 03 accepted whole-leaf completion from the original Hold; 04 inline apply_match and finish_leaf; 05 scan and immediately process ordinary leaves; 06 inline working single patterns; 07 refer to the root-owned single pattern during a match; 08 inline select_candidates.

candidate-01 build identity:
```text
[
  {
    "binary": "rum-cli",
    "sha256": "a29ff978cba5839d586b4a82f219b047623709ab1c8840ad27cfacdf32ba1e61",
    "base_sha256": "c49b3a1d0a695e7cf5d9fe620749f914d3994f72322d1f3e60473c8c8bf522f9",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-01/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "d67a1647ce35678d8ba76f897a91ff931043dc3c6f75e034f781942bf76a13fd",
    "base_sha256": "01b3a5b1a20928cdd3eaf2e0332dce88155305651698234b2e0cadb4a06f3251",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-01/target-alloc"
  }
]
```

Every allocation, byte and paired instruction row:
```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.0026832673071019997	2.0	2.0	168.0	168.0	4166.5171	4153.61625	-0.30963151453284876	4166.76265	4166.72395	-0.0009287786046585644	False
g7-sow-default-tag	20000	0.053219646861402126	21.0	21.0	1464.0	1464.0	16568.8867	16559.5311	-0.056464867974492794	16569.0108	16559.29095	-0.05866282614772622	False
g7-sow-tag-hit	20000	0.0761705788903666	20.0	20.0	1416.0	1416.0	19855.3933	19845.7806	-0.04841354615726656	19866.5336	19846.15015	-0.10260194561569902	False
g7-sow-tag-decline	20000	0.031637928628791556	13.0	13.0	848.0	848.0	17154.33135	17155.7892	0.008498436751946947	17150.5676	17155.9431	0.031342985989590844	False
g7-sow-predicate-hit	20000	0.05842085887823867	35.0	35.0	2332.0	2332.0	33285.80425	33289.36395	0.010694348777828822	33285.7022	33274.67415	-0.033131492716420574	False
g7-sow-predicate-decline	20000	0.05822286054277814	28.0	28.0	1764.0	1764.0	30614.899	30601.9087	-0.04243130117790317	30604.8613	30592.9917	-0.038783381122531324	False
g7-sow-pattern-list	20000	0.06664157314812957	37.0	37.0	2768.0	2768.0	41326.0149	41314.51005	-0.027839243701199834	41317.6039	41310.4701	-0.01726576404882385	False
g7-sow-nested-scopes	20000	0.04908701694480433	49.0	49.0	3552.0	3552.0	46374.80465	46362.4794	-0.02657747044547687	46381.68865	46362.93755	-0.0404278079254361	False
g7-sow-repeated-tag	10000	0.2613144753496277	48.0	48.0	3417.0	3417.0	50783.1179	50796.9079	0.027154693469499946	50793.218	50797.306	0.008048318576703295	False
g7-trace-plain	2000	0.34826975444315345	33.0005	33.0005	2917.704	2917.704	27907.2725	28755.428	3.0391916658999873	27903.84	28707.1175	2.878734611436995	True
g7-trace-pattern-hit	2000	0.11862567989811001	32.0005	32.0005	2893.704	2893.704	29321.066	30191.011	2.9669623880659746	29311.4555	29963.1175	2.2232331656133475	True
g7-trace-pattern-decline	2000	0.036326623799620705	27.0005	27.0005	2150.856	2150.856	26432.424	27074.2145	2.4280425435064146	26490.1325	27071.113	2.193195900398015	True
g7-trace-alternatives	2000	0.5084391171986558	34.0005	34.0005	3149.704	3149.704	33163.0875	33853.4665	2.08176937687119	33035.961	33886.022	2.573138405145814	True
g7-trace-predicate-hit	2000	0.1337796438685146	182.0	182.0	10704.0	10704.0	154979.906	156844.805	1.203316641578045	154960.7175	157518.8225	1.6508086960813184	True
g7-trace-predicate-decline	2000	0.31619069451558346	156.0005	156.0005	10333.704	10333.704	139284.763	140954.9375	1.199107830624646	139252.974	140908.9835	1.1892094311752421	True
g7-trace-levels	2000	0.01763534787542209	107.0	107.0	8320.0	8320.0	95782.1995	96958.858	1.2284730421125767	96203.793	97163.0975	0.9971587087008071	True
g7-trace-negative-levels	2000	0.23982683112320213	116.0	116.0	8481.0	8481.0	98953.744	100517.557	1.5803474803338435	100234.157	100440.246	0.20560755551621845	False
g7-trace-heads	2000	0.06624326505909606	125.0005	107.0005	11827.4	11107.4	114476.4715	106174.4205	-7.2521898091521875	114391.995	106151.43	-7.203795160666626	False
g7-trace-nested	2000	0.48524986699812755	118.0	119.0	7672.0	7896.0	91227.5225	94425.672	3.5056849209075036	90776.834	94424.948	4.01877201401406	True
g7-trace-nested-filter	2000	0.07699978796587015	117.0	118.0	8000.0	8224.0	106634.241	109100.194	2.312533926133553	106284.471	109309.037	2.845727105326601	True
g7-tracedetail-plain	2000	0.19662089083332113	55.0005	55.0005	5114.4	5114.4	36400.3125	36775.9385	1.031930701144379	36399.783	36819.0805	1.1519230760249188	True
g7-tracedetail-pattern-hit	2000	0.1844833388357614	58.0005	57.0005	5242.4	5210.4	40479.7275	40716.279	0.5843702875717227	40502.872	40756.4325	0.6260309145484699	True
g7-tracedetail-pattern-decline	2000	0.9269690649497342	52.0005	51.0005	3600.856	3568.856	32753.7515	32985.36	0.7071205263311731	32850.381	33011.736	0.49118151780338515	False
g7-tracedetail-alternatives	2000	0.7124896924413648	60.0005	59.0005	5498.4	5434.4	44363.4575	44565.553	0.45554497189495	44618.311	44890.391	0.6097944854972281	False
g7-tracedetail-predicate-hit	2000	0.2116941011157225	207.0	207.0	12049.0	12049.0	149465.523	149947.1405	0.32222648429767187	149960.7475	150156.9275	0.13082090031593108	False
g7-tracedetail-predicate-decline	2000	0.0298367056246418	232.0005	229.0005	14683.704	14587.704	158322.178	158915.806	0.37494936432720394	158414.034	158615.958	0.12746597943462668	True
g7-tracedetail-levels	2000	1.2538831561790809	152.0	152.0	9701.0	9701.0	94383.754	94758.245	0.39677485173983396	94210.2515	94782.518	0.6074354869968657	False
g7-tracedetail-negative-levels	2000	0.11894438130271023	155.0	155.0	9494.0	9494.0	95243.3875	95793.018	0.5770799573881158	95143.741	95748.114	0.6352209758075489	True
g7-tracedetail-heads	2000	0.1350390526104528	158.0	158.0	10853.0	10853.0	96282.0	96933.6495	0.6768134230697331	96582.473	96858.052	0.2853302379200784	True
g7-tracedetail-nested	2000	0.03457663379686782	327.0	328.0	20618.0	20842.0	179608.2315	181709.3255	1.1698205491211144	179671.0385	181643.452	1.0977915619940104	True
g7-tracedetail-nested-filter	2000	0.13737786126142293	340.0	337.0	21546.0	21514.0	206437.216	207247.6635	0.3925878849286635	206455.1095	207279.2685	0.39919525459843896	True
g7-traceprint-plain	2000	0.19779990078052645	40.0	40.0	2174.0	2174.0	30595.179	31471.6145	2.8646196186660644	30624.3465	31443.6125	2.675211371449193	True
g7-traceprint-pattern-hit	2000	0.5541069117105613	35.0	35.0	2112.0	2112.0	29363.3585	29966.9875	2.055721929764953	29336.1035	29992.1745	2.2363944823142523	True
g7-traceprint-pattern-decline	2000	1.3117046813320377	27.0	27.0	1688.0	1688.0	25291.3245	25976.757	2.7101486915009243	25303.8855	25902.799	2.3668835365224794	True
g7-traceprint-alternatives	2000	0.20413214834034132	37.0	37.0	2368.0	2368.0	33142.711	33949.885	2.4354495321761593	33239.089	33878.5525	1.9238298017132616	True
g7-traceprint-predicate-hit	2000	0.3097151647886598	206.0005	206.0005	12228.704	12228.704	170303.9495	172258.366	1.147604917993994	170370.119	172740.715	1.3914388355859453	True
g7-traceprint-predicate-decline	2000	0.16378302963143687	156.0005	156.0005	10333.704	10333.704	138448.5935	139994.0055	1.1162352472724901	138424.149	140208.378	1.2889578970790572	True
g7-traceprint-levels	2000	0.35921481706462366	130.0005	130.0005	9838.856	9838.856	108278.53	109311.876	0.9543406250528186	108287.2885	109434.2935	1.0592240473358938	True
g7-traceprint-negative-levels	2000	0.6931639432600001	141.0005	141.0005	10015.856	10015.856	113047.446	114112.9065	0.9424896693376095	113979.7575	114672.7405	0.6079877823919722	False
g7-traceprint-heads	2000	0.012223858486204442	135.0005	117.0005	10620.856	9900.856	117226.91	109038.4815	-6.985109903519604	117172.133	108955.844	-7.012152795750504	False
g7-traceprint-nested	2000	0.09411825290817077	153.0	154.0	8343.0	8567.0	103827.13	106998.4325	3.0544063964784485	103886.1515	107048.5225	3.0440736848356575	True
g7-traceprint-nested-filter	2000	0.42001189902265423	144.0	145.0	8605.0	8829.0	115910.7045	118857.4995	2.5422975494036493	115972.7865	118957.4305	2.5735727234595718	True
dot-three-no-input	20000	0.0009633022971966554	596.0	596.0	23472.0	23472.0	253028.1493	253031.7131	0.0014084598926489278	253028.31925	253030.93595	0.0010341530180424385	False
differences-no-input	20000	0.0725462958578671	13.0	13.0	672.0	672.0	9396.52595	9399.0695	0.027069046725713974	9417.99505	9399.233	-0.19921490614925874	False
take-multispec-small	20000	0.03114976305030881	109.0	109.0	5824.0	5824.0	49206.7826	49190.15775	-0.03378568791042591	49192.04065	49190.25945	-0.003620910977608638	False
```

```text
{
  "candidate": "candidate-01",
  "labels": 45,
  "regressions": [
    "g7-trace-plain",
    "g7-trace-pattern-hit",
    "g7-trace-pattern-decline",
    "g7-trace-alternatives",
    "g7-trace-predicate-hit",
    "g7-trace-predicate-decline",
    "g7-trace-levels",
    "g7-trace-nested",
    "g7-trace-nested-filter",
    "g7-tracedetail-plain",
    "g7-tracedetail-pattern-hit",
    "g7-tracedetail-predicate-decline",
    "g7-tracedetail-negative-levels",
    "g7-tracedetail-heads",
    "g7-tracedetail-nested",
    "g7-tracedetail-nested-filter",
    "g7-traceprint-plain",
    "g7-traceprint-pattern-hit",
    "g7-traceprint-pattern-decline",
    "g7-traceprint-alternatives",
    "g7-traceprint-predicate-hit",
    "g7-traceprint-predicate-decline",
    "g7-traceprint-levels",
    "g7-traceprint-nested",
    "g7-traceprint-nested-filter"
  ],
  "allocation_regressions": [
    "g7-trace-nested",
    "g7-trace-nested-filter",
    "g7-tracedetail-nested",
    "g7-traceprint-nested",
    "g7-traceprint-nested-filter"
  ],
  "byte_regressions": [
    "g7-trace-nested",
    "g7-trace-nested-filter",
    "g7-tracedetail-nested",
    "g7-traceprint-nested",
    "g7-traceprint-nested-filter"
  ],
  "worst": 4.01877201401406,
  "worst_residual": 4.01877201401406
}
```


candidate-02 build identity:
```text
[
  {
    "binary": "rum-cli",
    "sha256": "16df78f128e576dc231b125bbcfbb8ef9e1b49f5a3fd5db00396d90a02a689a8",
    "base_sha256": "c49b3a1d0a695e7cf5d9fe620749f914d3994f72322d1f3e60473c8c8bf522f9",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-02/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "bcd47bc456ed865e825bc0c243296817a3439a6dbb1a4a237a2ffc020c11869e",
    "base_sha256": "01b3a5b1a20928cdd3eaf2e0332dce88155305651698234b2e0cadb4a06f3251",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-02/target-alloc"
  }
]
```

Every allocation, byte and paired instruction row:
```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.0026832673071019997	2.0	2.0	168.0	168.0	4166.71395	4166.73375	0.00047519460748546294	4166.69155	4166.6229	-0.0016475901605694787	False
g7-sow-default-tag	20000	0.053219646861402126	21.0	21.0	1464.0	1464.0	16562.2413	16559.56585	-0.01615391269539357	16555.18255	16559.7221	0.027420718474635564	False
g7-sow-tag-hit	20000	0.0761705788903666	20.0	20.0	1416.0	1416.0	19857.4686	19844.7593	-0.06400261914552008	19852.7583	19845.29465	-0.03759502779017154	False
g7-sow-tag-decline	20000	0.031637928628791556	13.0	13.0	848.0	848.0	17148.1821	17155.6476	0.0435352269789524	17148.6311	17155.93595	0.04259727763342713	False
g7-sow-predicate-hit	20000	0.05842085887823867	35.0	35.0	2332.0	2332.0	33282.28515	33292.48515	0.030646934109324064	33288.9613	33296.79805	0.02354158764332759	False
g7-sow-predicate-decline	20000	0.05822286054277814	28.0	28.0	1764.0	1764.0	30638.5721	30602.42565	-0.11797694057681385	30611.9879	30588.2312	-0.07760587152199072	False
g7-sow-pattern-list	20000	0.06664157314812957	37.0	37.0	2768.0	2768.0	41341.2333	41308.0895	-0.08017128990681632	41317.41475	41308.57715	-0.02138952800769811	False
g7-sow-nested-scopes	20000	0.04908701694480433	49.0	49.0	3552.0	3552.0	46394.68125	46362.7976	-0.06872264048587429	46349.43185	46362.63055	0.028476508714736504	False
g7-sow-repeated-tag	10000	0.2613144753496277	48.0	48.0	3417.0	3417.0	50780.6465	50795.5897	0.02942695894978531	50793.3039	50848.3263	0.10832608980964054	False
g7-trace-plain	2000	0.34826975444315345	33.0005	25.0005	2917.704	2613.704	27938.269	24634.364	-11.825732653658672	27906.855	24741.541	-11.342424647994187	False
g7-trace-pattern-hit	2000	0.11862567989811001	32.0005	32.0005	2893.704	2893.704	29260.095	29963.121	2.4026784602032114	29289.145	30034.749	2.54566666251268	True
g7-trace-pattern-decline	2000	0.036326623799620705	27.0005	27.0005	2150.856	2150.856	26481.014	27028.281	2.0666391400268935	26533.7155	27071.012	2.024957643040981	True
g7-trace-alternatives	2000	0.5084391171986558	34.0005	34.0005	3149.704	3149.704	33203.295	33879.176	2.0355841189857893	33341.0085	33881.4035	1.620811799978994	True
g7-trace-predicate-hit	2000	0.1337796438685146	182.0	182.0	10704.0	10704.0	155072.778	156766.377	1.0921317215327297	154905.9415	156932.983	1.308562783565037	True
g7-trace-predicate-decline	2000	0.31619069451558346	156.0005	156.0005	10333.704	10333.704	139299.5345	140882.8915	1.1366563468307778	139168.084	140845.8515	1.2055691590896478	True
g7-trace-levels	2000	0.01763534787542209	107.0	107.0	8320.0	8320.0	95768.8995	96837.1015	1.115395504779726	96102.9885	97146.511	1.085837720852978	True
g7-trace-negative-levels	2000	0.23982683112320213	116.0	116.0	8481.0	8481.0	99378.6875	100328.4735	0.9557240328817995	99363.6235	100461.799	1.1052087890091933	True
g7-trace-heads	2000	0.06624326505909606	125.0005	107.0005	11827.4	11107.4	114396.648	106087.166	-7.263746049622021	114335.332	106125.804	-7.180219671728416	False
g7-trace-nested	2000	0.48524986699812755	118.0	87.0	7672.0	6680.0	90882.725	77746.265	-14.454298107808727	91266.9015	78030.648	-14.50279705178772	False
g7-trace-nested-filter	2000	0.07699978796587015	117.0	118.0	8000.0	8224.0	106322.2035	109318.5275	2.8181545353318294	106284.6335	109315.8885	2.852016232431187	True
g7-tracedetail-plain	2000	0.19662089083332113	55.0005	45.0005	5114.4	4253.4	36608.019	33538.0995	-8.385920855209362	36509.3775	33497.3855	-8.249913327062352	False
g7-tracedetail-pattern-hit	2000	0.1844833388357614	58.0005	57.0005	5242.4	5210.4	40438.543	40905.955	1.1558576677700882	40526.6155	40766.0075	0.5907031639491356	True
g7-tracedetail-pattern-decline	2000	0.9269690649497342	52.0005	51.0005	3600.856	3568.856	32760.6365	32966.2645	0.6276679026062215	32733.842	33114.777	1.1637344617231316	False
g7-tracedetail-alternatives	2000	0.7124896924413648	60.0005	59.0005	5498.4	5434.4	44328.637	44544.041	0.4859251593952507	44335.6765	44691.1185	0.8017064992794198	False
g7-tracedetail-predicate-hit	2000	0.2116941011157225	207.0	207.0	12049.0	12049.0	149549.1735	150219.2685	0.4480766989996221	149578.578	150248.0915	0.44759985617726095	True
g7-tracedetail-predicate-decline	2000	0.0298367056246418	232.0005	229.0005	14683.704	14587.704	158386.9565	159062.908	0.4267722007777808	158332.14	159015.712	0.4317329381134938	True
g7-tracedetail-levels	2000	1.2538831561790809	152.0	152.0	9701.0	9701.0	94190.0025	94744.946	0.589174525183811	94288.0395	94789.97	0.5323374021367844	False
g7-tracedetail-negative-levels	2000	0.11894438130271023	155.0	155.0	9494.0	9494.0	95066.6565	95816.4035	0.788654011409351	95249.1235	95752.1165	0.5280814998785743	True
g7-tracedetail-heads	2000	0.1350390526104528	158.0	158.0	10853.0	10853.0	96271.2105	96861.9835	0.6136548994571944	96297.001	96802.376	0.52480865940987	True
g7-tracedetail-nested	2000	0.03457663379686782	327.0	254.0	20618.0	13332.0	179559.8735	158245.404	-11.87039681223655	178685.6305	158304.506	-11.406135145265639	False
g7-tracedetail-nested-filter	2000	0.13737786126142293	340.0	337.0	21546.0	21514.0	206357.0465	207035.54	0.3287958960005888	206711.733	207202.0195	0.2371836822634421	True
g7-traceprint-plain	2000	0.19779990078052645	40.0	32.0	2174.0	1870.0	30623.165	27347.5155	-10.696639292509447	30633.926	27318.015	-10.824309623258866	False
g7-traceprint-pattern-hit	2000	0.5541069117105613	35.0	35.0	2112.0	2112.0	29493.6215	29994.1095	1.6969364036898638	29336.321	30055.2835	2.45075890736266	True
g7-traceprint-pattern-decline	2000	1.3117046813320377	27.0	27.0	1688.0	1688.0	25303.7975	25944.9015	2.5336276106382893	25304.321	25933.4945	2.4864271204906085	True
g7-traceprint-alternatives	2000	0.20413214834034132	37.0	37.0	2368.0	2368.0	33142.7465	33950.879	2.4383389590238114	33286.142	33905.06	1.8593864077128597	True
g7-traceprint-predicate-hit	2000	0.3097151647886598	206.0005	206.0005	12228.704	12228.704	170283.793	172266.8995	1.1645891044956835	170553.7745	172397.107	1.080792556719401	True
g7-traceprint-predicate-decline	2000	0.16378302963143687	156.0005	156.0005	10333.704	10333.704	138434.8645	140050.724	1.1672345011035867	138489.881	139995.968	1.0875068915684905	True
g7-traceprint-levels	2000	0.35921481706462366	130.0005	130.0005	9838.856	9838.856	108271.1805	109356.244	1.0021720415249513	108354.5465	109332.0215	0.9021079701533319	True
g7-traceprint-negative-levels	2000	0.6931639432600001	141.0005	141.0005	10015.856	10015.856	113024.9665	114241.6025	1.0764311971727114	113096.6745	114285.5885	1.0512369220900597	True
g7-traceprint-heads	2000	0.012223858486204442	135.0005	117.0005	10620.856	9900.856	117226.833	108938.0135	-7.070752734572294	117212.887	108901.505	-7.090843176655137	False
g7-traceprint-nested	2000	0.09411825290817077	153.0	122.0	8343.0	7351.0	103442.2025	90330.393	-12.675493350985057	103801.095	90281.935	-13.02410152802338	False
g7-traceprint-nested-filter	2000	0.42001189902265423	144.0	145.0	8605.0	8829.0	115936.156	120092.4265	3.584964900854559	116172.1565	119014.4405	2.446613789079488	True
dot-three-no-input	20000	0.0009633022971966554	596.0	596.0	23472.0	23472.0	253030.029	253034.6477	0.0018253564678660794	253030.33195	253031.03945	0.0002796107464897446	False
differences-no-input	20000	0.0725462958578671	13.0	13.0	672.0	672.0	9409.58995	9399.3467	-0.10885968521933131	9397.22555	9398.7894	0.016641613970835856	False
take-multispec-small	20000	0.03114976305030881	109.0	109.0	5824.0	5824.0	49183.8477	49190.2906	0.01309962579443269	49200.5323	49190.37935	-0.02063585397428236	False
```

```text
{
  "candidate": "candidate-02",
  "labels": 45,
  "regressions": [
    "g7-trace-pattern-hit",
    "g7-trace-pattern-decline",
    "g7-trace-alternatives",
    "g7-trace-predicate-hit",
    "g7-trace-predicate-decline",
    "g7-trace-levels",
    "g7-trace-negative-levels",
    "g7-trace-nested-filter",
    "g7-tracedetail-pattern-hit",
    "g7-tracedetail-predicate-hit",
    "g7-tracedetail-predicate-decline",
    "g7-tracedetail-negative-levels",
    "g7-tracedetail-heads",
    "g7-tracedetail-nested-filter",
    "g7-traceprint-pattern-hit",
    "g7-traceprint-pattern-decline",
    "g7-traceprint-alternatives",
    "g7-traceprint-predicate-hit",
    "g7-traceprint-predicate-decline",
    "g7-traceprint-levels",
    "g7-traceprint-negative-levels",
    "g7-traceprint-nested-filter"
  ],
  "allocation_regressions": [
    "g7-trace-nested-filter",
    "g7-traceprint-nested-filter"
  ],
  "byte_regressions": [
    "g7-trace-nested-filter",
    "g7-traceprint-nested-filter"
  ],
  "worst": 3.584964900854559,
  "worst_residual": 3.584964900854559
}
```


candidate-03 build identity:
```text
[
  {
    "binary": "rum-cli",
    "sha256": "a6638515a4c9f2259a25c9e7cb84fd8fea6567dfbf1ff00d7d8a5c49b8a9e519",
    "base_sha256": "c49b3a1d0a695e7cf5d9fe620749f914d3994f72322d1f3e60473c8c8bf522f9",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-03/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "52bc969f966d8b925ad58e8df09db1289388cb794283b85a642db8a2f2611e94",
    "base_sha256": "01b3a5b1a20928cdd3eaf2e0332dce88155305651698234b2e0cadb4a06f3251",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-03/target-alloc"
  }
]
```

Every allocation, byte and paired instruction row:
```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.0026832673071019997	2.0	2.0	168.0	168.0	4166.5953	4166.6237	0.000681611674657745	4166.6848	4153.7292	-0.31093304681938383	False
g7-sow-default-tag	20000	0.053219646861402126	21.0	21.0	1464.0	1464.0	16570.9675	16560.104	-0.06555742747066384	16558.407	16560.09095	0.010169758479805502	False
g7-sow-tag-hit	20000	0.0761705788903666	20.0	20.0	1416.0	1416.0	19867.32255	19845.50605	-0.10981097198726752	19853.08455	19845.01775	-0.040632476931656836	False
g7-sow-tag-decline	20000	0.031637928628791556	13.0	13.0	848.0	848.0	17141.7282	17154.971	0.0772547542785107	17149.48225	17156.44035	0.04057323654771405	False
g7-sow-predicate-hit	20000	0.05842085887823867	35.0	35.0	2332.0	2332.0	33278.0351	33301.10095	0.06931253582336261	33279.7099	33290.3243	0.031894508791974197	False
g7-sow-predicate-decline	20000	0.05822286054277814	28.0	28.0	1764.0	1764.0	30615.02375	30602.87225	-0.039691296989441405	30608.39425	30570.7784	-0.12289390189098581	False
g7-sow-pattern-list	20000	0.06664157314812957	37.0	37.0	2768.0	2768.0	41306.0758	41315.91185	0.023812598533012874	41316.35535	41315.13895	-0.0029441125425799797	False
g7-sow-nested-scopes	20000	0.04908701694480433	49.0	49.0	3552.0	3552.0	46426.17165	46362.26825	-0.13764520684960146	46364.1984	46362.49875	-0.0036658673257750074	False
g7-sow-repeated-tag	10000	0.2613144753496277	48.0	48.0	3417.0	3417.0	50763.0648	50798.8938	0.07058084483504423	50787.1358	50796.6595	0.01875218960467606	False
g7-trace-plain	2000	0.34826975444315345	33.0005	25.0005	2917.704	2613.704	28003.5475	24644.9805	-11.993362626645776	28036.31	24737.2435	-11.767120922831865	False
g7-trace-pattern-hit	2000	0.11862567989811001	32.0005	28.0005	2893.704	2741.704	29265.3665	28268.074	-3.407756742086243	29310.6895	28294.796	-3.465948830715848	False
g7-trace-pattern-decline	2000	0.036326623799620705	27.0005	27.0005	2150.856	2150.856	26418.6845	27036.341	2.3379532769695732	26532.7425	27021.918	1.8436673103053858	True
g7-trace-alternatives	2000	0.5084391171986558	34.0005	30.0005	3149.704	2997.704	33109.2545	32162.0125	-2.8609584066593863	33291.6445	32191.8665	-3.303465528715477	False
g7-trace-predicate-hit	2000	0.1337796438685146	182.0	158.0	10704.0	9792.0	155104.4695	146665.001	-5.441151068828498	154953.618	146905.7355	-5.193736425050743	False
g7-trace-predicate-decline	2000	0.31619069451558346	156.0005	156.0005	10333.704	10333.704	139247.493	141045.9175	1.291530972123156	139244.559	140890.8105	1.182273484739893	True
g7-trace-levels	2000	0.01763534787542209	107.0	107.0	8320.0	8320.0	96202.113	97038.613	0.8695235207567542	96165.922	97094.7635	0.9658738570613279	True
g7-trace-negative-levels	2000	0.23982683112320213	116.0	116.0	8481.0	8481.0	100023.083	100319.7985	0.2966470249672337	99362.5255	100947.8765	1.595522046186315	True
g7-trace-heads	2000	0.06624326505909606	125.0005	107.0005	11827.4	11107.4	114393.3675	105987.396	-7.34830321347083	114388.8155	105396.322	-7.861339817790136	False
g7-trace-nested	2000	0.48524986699812755	118.0	87.0	7672.0	6680.0	91263.862	77651.238	-14.915678234173345	91268.238	77450.111	-15.140126842374224	False
g7-trace-nested-filter	2000	0.07699978796587015	117.0	94.0	8000.0	7312.0	106953.605	99194.274	-7.254856907347806	106328.6575	99191.2425	-6.712597683272737	False
g7-tracedetail-plain	2000	0.19662089083332113	55.0005	45.0005	5114.4	4253.4	36451.157	33589.0405	-7.851922231165387	36448.063	33707.0915	-7.52021170507744	False
g7-tracedetail-pattern-hit	2000	0.1844833388357614	58.0005	57.0005	5242.4	5210.4	40400.605	40997.2695	1.4768702102357034	40541.693	40728.1145	0.4598266283551755	True
g7-tracedetail-pattern-decline	2000	0.9269690649497342	52.0005	51.0005	3600.856	3568.856	32661.9805	33083.951	1.291931761455789	32741.7065	32979.0895	0.7250171887039691	False
g7-tracedetail-alternatives	2000	0.7124896924413648	60.0005	59.0005	5498.4	5434.4	44467.1495	44551.534	0.1897681793162853	44369.32	44658.5235	0.6518096288155917	False
g7-tracedetail-predicate-hit	2000	0.2116941011157225	207.0	207.0	12049.0	12049.0	149483.549	150176.036	0.46325298310918583	149447.4855	150178.697	0.4892765492531259	True
g7-tracedetail-predicate-decline	2000	0.0298367056246418	232.0005	229.0005	14683.704	14587.704	158611.9935	158904.591	0.18447375481727146	158483.077	158981.2925	0.3143651104149159	True
g7-tracedetail-levels	2000	1.2538831561790809	152.0	152.0	9701.0	9701.0	94516.0795	94784.824	0.284337333310547	94220.225	94841.911	0.6598222409254451	False
g7-tracedetail-negative-levels	2000	0.11894438130271023	155.0	155.0	9494.0	9494.0	95485.114	95757.731	0.2855073304934175	95472.0965	95780.7625	0.32330493548970995	True
g7-tracedetail-heads	2000	0.1350390526104528	158.0	158.0	10853.0	10853.0	96270.724	96859.224	0.6112969504623145	96368.2915	96905.7775	0.5577415471768488	True
g7-tracedetail-nested	2000	0.03457663379686782	327.0	254.0	20618.0	13332.0	179579.6925	158012.158	-12.010007478991536	180229.0115	158102.353	-12.27696823937804	False
g7-tracedetail-nested-filter	2000	0.13737786126142293	340.0	337.0	21546.0	21514.0	206617.259	207235.788	0.29935979355917297	206456.8605	207234.868	0.37683780433150904	True
g7-traceprint-plain	2000	0.19779990078052645	40.0	32.0	2174.0	1870.0	30613.616	27306.135	-10.803954031434914	30549.888	27210.6065	-10.93058508103204	False
g7-traceprint-pattern-hit	2000	0.5541069117105613	35.0	31.0	2112.0	1960.0	29444.7575	28436.2645	-3.425034150816142	29240.474	28304.6545	-3.2004252051454407	False
g7-traceprint-pattern-decline	2000	1.3117046813320377	27.0	27.0	1688.0	1688.0	25304.4545	25902.128	2.3619299914171332	25245.9245	25962.401	2.837988761314736	True
g7-traceprint-alternatives	2000	0.20413214834034132	37.0	33.0	2368.0	2216.0	33169.8135	32295.5075	-2.6358484047551145	33155.6745	32233.867	-2.7802405286612486	False
g7-traceprint-predicate-hit	2000	0.3097151647886598	206.0005	182.0005	12228.704	11316.704	171284.0605	162169.22	-5.321476191884178	170342.519	162269.8805	-4.739062535525851	False
g7-traceprint-predicate-decline	2000	0.16378302963143687	156.0005	156.0005	10333.704	10333.704	138223.994	140038.9075	1.313023482739184	138494.96	140258.866	1.2736246864145873	True
g7-traceprint-levels	2000	0.35921481706462366	130.0005	130.0005	9838.856	9838.856	108309.553	109209.533	0.8309331680096621	108209.6705	109195.5245	0.9110590536360652	True
g7-traceprint-negative-levels	2000	0.6931639432600001	141.0005	141.0005	10015.856	10015.856	113034.4535	113994.378	0.8492317786983339	113034.4685	113967.6515	0.8255738381253153	True
g7-traceprint-heads	2000	0.012223858486204442	135.0005	117.0005	10620.856	9900.856	117449.285	108696.4105	-7.4524715071701	117249.2215	108384.285	-7.560763633726985	False
g7-traceprint-nested	2000	0.09411825290817077	153.0	122.0	8343.0	7351.0	103771.742	90301.505	-12.98064072201852	103826.9515	90390.227	-12.941461061774506	False
g7-traceprint-nested-filter	2000	0.42001189902265423	144.0	121.0	8605.0	7917.0	116028.829	109082.0805	-5.987088346810776	115997.8805	108633.852	-6.348416426453585	False
dot-three-no-input	20000	0.0009633022971966554	596.0	596.0	23472.0	23472.0	253028.29305	253031.1377	0.001124241864691733	253030.4835	253067.0507	0.01445169747700259	False
differences-no-input	20000	0.0725462958578671	13.0	13.0	672.0	672.0	9411.16445	9399.43	-0.12468648340322463	9403.55815	9398.93175	-0.04919839837435047	False
take-multispec-small	20000	0.03114976305030881	109.0	109.0	5824.0	5824.0	49191.97365	49190.3932	-0.003212820878562006	49207.78895	49190.1208	-0.035905189761642475	False
```

```text
{
  "candidate": "candidate-03",
  "labels": 45,
  "regressions": [
    "g7-trace-pattern-decline",
    "g7-trace-predicate-decline",
    "g7-trace-levels",
    "g7-trace-negative-levels",
    "g7-tracedetail-pattern-hit",
    "g7-tracedetail-predicate-hit",
    "g7-tracedetail-predicate-decline",
    "g7-tracedetail-negative-levels",
    "g7-tracedetail-heads",
    "g7-tracedetail-nested-filter",
    "g7-traceprint-pattern-decline",
    "g7-traceprint-predicate-decline",
    "g7-traceprint-levels",
    "g7-traceprint-negative-levels"
  ],
  "allocation_regressions": [],
  "byte_regressions": [],
  "worst": 2.837988761314736,
  "worst_residual": 2.837988761314736
}
```


candidate-04 build identity:
```text
[
  {
    "binary": "rum-cli",
    "sha256": "510c071ffddc539e0e771139a1940ece6fe17e4d11983f6b8db0bda605cc6fc6",
    "base_sha256": "c49b3a1d0a695e7cf5d9fe620749f914d3994f72322d1f3e60473c8c8bf522f9",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-04/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "62a95fab9102847dcb992753f8d851f148a826e93141404954265b5944cb9d91",
    "base_sha256": "01b3a5b1a20928cdd3eaf2e0332dce88155305651698234b2e0cadb4a06f3251",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-04/target-alloc"
  }
]
```

Every allocation, byte and paired instruction row:
```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.0026832673071019997	2.0	2.0	168.0	168.0	4166.6357	4166.71295	0.0018540137790346023	4166.60165	4166.61975	0.00043440677848227693	False
g7-sow-default-tag	20000	0.053219646861402126	21.0	21.0	1464.0	1464.0	16558.07595	16559.85945	0.010771179002833087	16568.64835	16559.76405	-0.053621151299276804	False
g7-sow-tag-hit	20000	0.0761705788903666	20.0	20.0	1416.0	1416.0	19878.6903	19845.23275	-0.16830862343079067	19865.8073	19845.0411	-0.10453237407573868	False
g7-sow-tag-decline	20000	0.031637928628791556	13.0	13.0	848.0	848.0	17150.52765	17155.5391	0.029220383782191384	17153.8952	17155.4484	0.009054503259431357	False
g7-sow-predicate-hit	20000	0.05842085887823867	35.0	35.0	2332.0	2332.0	33287.9622	33290.8197	0.00858418422500229	33285.63275	33290.61915	0.014980637554495857	False
g7-sow-predicate-decline	20000	0.05822286054277814	28.0	28.0	1764.0	1764.0	30586.6403	30601.5556	0.0487641004494499	30611.41175	30590.71555	-0.06760942673609271	False
g7-sow-pattern-list	20000	0.06664157314812957	37.0	37.0	2768.0	2768.0	41326.1912	41308.97665	-0.04165530260626937	41334.4051	41312.9825	-0.051827527088332825	False
g7-sow-nested-scopes	20000	0.04908701694480433	49.0	49.0	3552.0	3552.0	46350.0962	46362.33805	0.026411703542472686	46376.9335	46362.8386	-0.030392048236649316	False
g7-sow-repeated-tag	10000	0.2613144753496277	48.0	48.0	3417.0	3417.0	50778.481	50770.9571	-0.01481710333162134	50819.521	50799.6997	-0.039003319216646126	False
g7-trace-plain	2000	0.34826975444315345	33.0005	25.0005	2917.704	2613.704	27907.4065	24594.8995	-11.86963396258266	27967.9	24746.733	-11.517371701128798	False
g7-trace-pattern-hit	2000	0.11862567989811001	32.0005	28.0005	2893.704	2741.704	29556.0745	28165.369	-4.7053119317316705	29458.876	28070.686	-4.7122979165939665	False
g7-trace-pattern-decline	2000	0.036326623799620705	27.0005	27.0005	2150.856	2150.856	26429.4045	27044.0945	2.325780741673533	26441.1955	26988.138	2.0685240952891037	True
g7-trace-alternatives	2000	0.5084391171986558	34.0005	30.0005	3149.704	2997.704	33180.6365	32045.046	-3.4224494156403606	33110.0415	32048.904	-3.204881214057076	False
g7-trace-predicate-hit	2000	0.1337796438685146	182.0	158.0	10704.0	9792.0	154786.3735	146334.1455	-5.460576282575658	155515.821	146542.166	-5.770252146886068	False
g7-trace-predicate-decline	2000	0.31619069451558346	156.0005	156.0005	10333.704	10333.704	139264.573	140625.0585	0.9769070989791517	139419.3035	140703.262	0.9209330901584822	True
g7-trace-levels	2000	0.01763534787542209	107.0	107.0	8320.0	8320.0	96154.353	96819.221	0.6914590751809335	96135.8745	96441.592	0.31800563690715933	True
g7-trace-negative-levels	2000	0.23982683112320213	116.0	116.0	8481.0	8481.0	99545.0615	100044.7445	0.5019666395002398	99370.7605	99990.7295	0.6238947924726679	True
g7-trace-heads	2000	0.06624326505909606	125.0005	107.0005	11827.4	11107.4	114392.903	105484.718	-7.787358102101849	114389.08	105599.411	-7.684010571638488	False
g7-trace-nested	2000	0.48524986699812755	118.0	87.0	7672.0	6680.0	90883.7785	78067.821	-14.101479616629275	91311.5775	77661.334	-14.949082990051288	False
g7-trace-nested-filter	2000	0.07699978796587015	117.0	94.0	8000.0	7312.0	106282.831	98824.853	-7.017105142786417	106257.681	98730.947	-7.083472864422857	False
g7-tracedetail-plain	2000	0.19662089083332113	55.0005	45.0005	5114.4	4253.4	36648.0985	33568.924	-8.402003449101192	36546.5525	33500.64	-8.334336049891434	False
g7-tracedetail-pattern-hit	2000	0.1844833388357614	58.0005	57.0005	5242.4	5210.4	40521.6615	40705.3305	0.45326127607081546	40670.3055	40655.927	-0.03535380377214237	False
g7-tracedetail-pattern-decline	2000	0.9269690649497342	52.0005	51.0005	3600.856	3568.856	32784.978	32954.624	0.5174504006072667	32544.565	33107.27	1.729029102094315	False
g7-tracedetail-alternatives	2000	0.7124896924413648	60.0005	59.0005	5498.4	5434.4	44368.9225	44565.547	0.44315815873148257	44330.301	44559.3935	0.5167853473406314	False
g7-tracedetail-predicate-hit	2000	0.2116941011157225	207.0	207.0	12049.0	12049.0	149608.0375	150055.5985	0.2991557188229299	149448.453	150007.627	0.3741584397665143	True
g7-tracedetail-predicate-decline	2000	0.0298367056246418	232.0005	229.0005	14683.704	14587.704	158312.851	158898.9575	0.3702204188085645	158308.2775	158756.3765	0.28305468739624473	True
g7-tracedetail-levels	2000	1.2538831561790809	152.0	152.0	9701.0	9701.0	94366.945	94775.2835	0.4327134888175044	94244.16	94662.666	0.4440657118701008	False
g7-tracedetail-negative-levels	2000	0.11894438130271023	155.0	155.0	9494.0	9494.0	95541.009	95623.283	0.08611380689940162	95630.5025	95602.988	-0.028771677739547474	False
g7-tracedetail-heads	2000	0.1350390526104528	158.0	158.0	10853.0	10853.0	96262.956	96637.275	0.3888505148335497	96314.8	96738.619	0.44003517631765465	True
g7-tracedetail-nested	2000	0.03457663379686782	327.0	254.0	20618.0	13332.0	179608.4485	157831.3475	-12.124764275774025	179607.4255	158281.382	-11.87369811723068	False
g7-tracedetail-nested-filter	2000	0.13737786126142293	340.0	337.0	21546.0	21514.0	206360.506	206822.454	0.22385484943518996	205977.9845	206822.789	0.41014310439571044	True
g7-traceprint-plain	2000	0.19779990078052645	40.0	32.0	2174.0	1870.0	30636.873	27409.903	-10.532961376312787	30586.769	27309.9295	-10.713258075738574	False
g7-traceprint-pattern-hit	2000	0.5541069117105613	35.0	31.0	2112.0	1960.0	29336.65	28260.7485	-3.667431352932249	29361.307	28230.9815	-3.8497111181051924	False
g7-traceprint-pattern-decline	2000	1.3117046813320377	27.0	27.0	1688.0	1688.0	25261.219	25972.287	2.814860201322822	25281.2745	25782.2995	1.981802776596564	True
g7-traceprint-alternatives	2000	0.20413214834034132	37.0	33.0	2368.0	2216.0	33156.895	32180.3345	-2.9452712625835376	33142.04	32090.467	-3.1729277980474313	False
g7-traceprint-predicate-hit	2000	0.3097151647886598	206.0005	182.0005	12228.704	11316.704	170303.6795	161941.008	-4.910446752854803	170188.5605	161868.8735	-4.888511293331021	False
g7-traceprint-predicate-decline	2000	0.16378302963143687	156.0005	156.0005	10333.704	10333.704	138417.097	139808.1745	1.004989650953303	138459.764	139819.0875	0.9817462205121252	True
g7-traceprint-levels	2000	0.35921481706462366	130.0005	130.0005	9838.856	9838.856	108330.7355	108920.9335	0.5448112184191833	108268.5885	109660.64	1.2857390303929295	True
g7-traceprint-negative-levels	2000	0.6931639432600001	141.0005	141.0005	10015.856	10015.856	113051.2115	113720.0265	0.5916035672028253	113276.966	113692.789	0.36708522013204625	False
g7-traceprint-heads	2000	0.012223858486204442	135.0005	117.0005	10620.856	9900.856	117255.827	108286.6195	-7.649263776033921	117261.352	108179.308	-7.7451298702406195	False
g7-traceprint-nested	2000	0.09411825290817077	153.0	122.0	8343.0	7351.0	103795.5315	90326.6915	-12.976319698309934	103831.358	90365.715	-12.96876325165659	False
g7-traceprint-nested-filter	2000	0.42001189902265423	144.0	121.0	8605.0	7917.0	115600.2235	108351.6045	-6.270419537726923	115980.0875	108232.554	-6.680054884421427	False
dot-three-no-input	20000	0.0009633022971966554	596.0	596.0	23472.0	23472.0	253030.04435	253035.17085	0.0020260439874464353	253030.65335	253035.4569	0.0018984063536953144	False
differences-no-input	20000	0.0725462958578671	13.0	13.0	672.0	672.0	9416.6023	9399.3378	-0.18334107621812423	9410.92725	9399.1764	-0.12486389160005773	False
take-multispec-small	20000	0.03114976305030881	109.0	109.0	5824.0	5824.0	49207.7612	49190.38695	-0.03530794650337121	49198.5825	49190.3373	-0.016759019429057886	False
```

```text
{
  "candidate": "candidate-04",
  "labels": 45,
  "regressions": [
    "g7-trace-pattern-decline",
    "g7-trace-predicate-decline",
    "g7-trace-levels",
    "g7-trace-negative-levels",
    "g7-tracedetail-predicate-hit",
    "g7-tracedetail-predicate-decline",
    "g7-tracedetail-heads",
    "g7-tracedetail-nested-filter",
    "g7-traceprint-pattern-decline",
    "g7-traceprint-predicate-decline",
    "g7-traceprint-levels"
  ],
  "allocation_regressions": [],
  "byte_regressions": [],
  "worst": 2.814860201322822,
  "worst_residual": 2.814860201322822
}
```


candidate-05 build identity:
```text
[
  {
    "binary": "rum-cli",
    "sha256": "c63ac7cd3af7894f06fad8574c367f68f9e5aa90e49544c9281de7738b6af650",
    "base_sha256": "c49b3a1d0a695e7cf5d9fe620749f914d3994f72322d1f3e60473c8c8bf522f9",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-05/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "90aaa86ce87c673ca4e8b32183c55c504a53e669a436e7be644c0b4bd1e1aa31",
    "base_sha256": "01b3a5b1a20928cdd3eaf2e0332dce88155305651698234b2e0cadb4a06f3251",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-05/target-alloc"
  }
]
```

Every allocation, byte and paired instruction row:
```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.0026832673071019997	2.0	2.0	168.0	168.0	4166.63415	4166.501	-0.003195624938645558	4166.7333	4166.4409	-0.007017487776339593	False
g7-sow-default-tag	20000	0.053219646861402126	21.0	21.0	1464.0	1464.0	16570.9278	16559.64025	-0.06811658427479106	16558.33925	16559.81375	0.008904878549342854	False
g7-sow-tag-hit	20000	0.0761705788903666	20.0	20.0	1416.0	1416.0	19875.9808	19844.7706	-0.15702470390794865	19867.73485	19845.02235	-0.11431851779520885	False
g7-sow-tag-decline	20000	0.031637928628791556	13.0	13.0	848.0	848.0	17150.81375	17155.54625	0.027593442905859966	17154.2561	17155.57325	0.007678269417943362	False
g7-sow-predicate-hit	20000	0.05842085887823867	35.0	35.0	2332.0	2332.0	33289.25665	33288.1749	-0.0032495468774706993	33368.0745	33290.7425	-0.2317544573931074	False
g7-sow-predicate-decline	20000	0.05822286054277814	28.0	28.0	1764.0	1764.0	30604.77385	30623.8277	0.06225777093922247	30587.33895	30602.8424	0.050685841044706414	False
g7-sow-pattern-list	20000	0.06664157314812957	37.0	37.0	2768.0	2768.0	41330.71885	41308.0835	-0.054766407722417476	41326.18935	41314.67595	-0.02785981524329184	False
g7-sow-nested-scopes	20000	0.04908701694480433	49.0	49.0	3552.0	3552.0	46360.1138	46362.65115	0.005473131517641505	46386.5633	46362.9054	-0.05100162270481601	False
g7-sow-repeated-tag	10000	0.2613144753496277	48.0	48.0	3417.0	3417.0	50788.9589	50799.3584	0.020475907018435713	50792.1653	50798.1884	0.011858324929492348	False
g7-trace-plain	2000	0.34826975444315345	33.0005	25.0005	2917.704	2613.704	27907.1725	24560.588	-11.991843673879899	27809.278	24565.22	-11.665380165569195	False
g7-trace-pattern-hit	2000	0.11862567989811001	32.0005	28.0005	2893.704	2741.704	29361.077	28089.9055	-4.329444386525738	29278.4545	28030.0845	-4.263783800473487	False
g7-trace-pattern-decline	2000	0.036326623799620705	27.0005	27.0005	2150.856	2150.856	26525.63	26899.4225	1.4091748244999147	26538.3935	26948.1725	1.5440987413198215	True
g7-trace-alternatives	2000	0.5084391171986558	34.0005	30.0005	3149.704	2997.704	33111.4285	31966.5775	-3.4575705484890307	32912.189	31901.008	-3.072360212807468	False
g7-trace-predicate-hit	2000	0.1337796438685146	182.0	158.0	10704.0	9792.0	154912.2635	146065.8465	-5.710598244534715	154989.8495	146411.828	-5.534569862267014	False
g7-trace-predicate-decline	2000	0.31619069451558346	156.0005	156.0005	10333.704	10333.704	139168.5505	140392.13	0.879206900987306	139427.4915	140390.859	0.6909451569671266	True
g7-trace-levels	2000	0.01763534787542209	107.0	107.0	8320.0	8320.0	96154.416	96553.5955	0.4151442196892896	96200.732	96498.0185	0.3090272743454747	True
g7-trace-negative-levels	2000	0.23982683112320213	116.0	116.0	8481.0	8481.0	99372.1555	99726.343	0.3564252966214543	99377.422	99773.111	0.39816790578446337	True
g7-trace-heads	2000	0.06624326505909606	125.0005	107.0005	11827.4	11107.4	114386.9825	105290.5455	-7.952335835067603	114455.1835	105213.2615	-8.07470812363863	False
g7-trace-nested	2000	0.48524986699812755	118.0	87.0	7672.0	6680.0	91248.8515	77283.2065	-15.305009071812814	91269.487	77379.2515	-15.218925794992133	False
g7-trace-nested-filter	2000	0.07699978796587015	117.0	94.0	8000.0	7312.0	106283.46	98348.0265	-7.466292027000254	106225.2145	98350.4035	-7.413316167038664	False
g7-tracedetail-plain	2000	0.19662089083332113	55.0005	45.0005	5114.4	4253.4	36499.41	33447.444	-8.361685846428745	36450.923	33641.094	-7.7085263382768225	False
g7-tracedetail-pattern-hit	2000	0.1844833388357614	58.0005	57.0005	5242.4	5210.4	40447.4635	40650.138	0.5010808650584453	40493.507	40884.7625	0.9662178679658462	True
g7-tracedetail-pattern-decline	2000	0.9269690649497342	52.0005	51.0005	3600.856	3568.856	32727.888	32876.727	0.45477728352041336	32923.335	32987.193	0.19395969454492068	False
g7-tracedetail-alternatives	2000	0.7124896924413648	60.0005	59.0005	5498.4	5434.4	44368.314	44425.772	0.12950232907205095	44412.0755	44404.6165	-0.01679498180624517	False
g7-tracedetail-predicate-hit	2000	0.2116941011157225	207.0	207.0	12049.0	12049.0	149497.872	149917.837	0.28091704208337287	149562.0675	149877.0615	0.21061088902103453	False
g7-tracedetail-predicate-decline	2000	0.0298367056246418	232.0005	229.0005	14683.704	14587.704	158509.9705	158602.121	0.05813545968706979	158431.555	158560.867	0.08162010402537057	True
g7-tracedetail-levels	2000	1.2538831561790809	152.0	152.0	9701.0	9701.0	94217.8585	94799.202	0.6170204982954397	94250.6475	94514.36	0.2797991387804455	False
g7-tracedetail-negative-levels	2000	0.11894438130271023	155.0	155.0	9494.0	9494.0	95232.869	95618.207	0.404627104114641	95330.376	95474.1945	0.15086324635915638	True
g7-tracedetail-heads	2000	0.1350390526104528	158.0	158.0	10853.0	10853.0	96292.6835	96708.262	0.43157848020716205	96269.0825	96766.308	0.5164955218099232	True
g7-tracedetail-nested	2000	0.03457663379686782	327.0	254.0	20618.0	13332.0	179607.722	160094.04	-10.864611934669488	179607.0955	157986.2005	-12.037884661410825	False
g7-tracedetail-nested-filter	2000	0.13737786126142293	340.0	337.0	21546.0	21514.0	206359.8685	206499.8855	0.06785088642367043	206359.4915	206501.6725	0.06889966580478912	False
g7-traceprint-plain	2000	0.19779990078052645	40.0	32.0	2174.0	1870.0	30638.1415	27261.894	-11.019752944218242	30840.419	27264.523	-11.594835984556507	False
g7-traceprint-pattern-hit	2000	0.5541069117105613	35.0	31.0	2112.0	1960.0	29422.686	28137.3175	-4.36863072256558	29305.183	28175.674	-3.854297719280586	False
g7-traceprint-pattern-decline	2000	1.3117046813320377	27.0	27.0	1688.0	1688.0	25389.3705	25735.7825	1.364397750625601	25302.87	25734.749	1.7068379990095917	True
g7-traceprint-alternatives	2000	0.20413214834034132	37.0	33.0	2368.0	2216.0	33168.0485	32020.5895	-3.4595312413390866	33237.761	32081.2985	-3.479363426435367	False
g7-traceprint-predicate-hit	2000	0.3097151647886598	206.0005	182.0005	12228.704	11316.704	170363.083	161263.2595	-5.341429222667937	170305.5615	161802.7855	-4.9926590330404474	False
g7-traceprint-predicate-decline	2000	0.16378302963143687	156.0005	156.0005	10333.704	10333.704	138585.7305	139511.8675	0.6682773158958044	138381.5755	139684.181	0.9413142575472433	True
g7-traceprint-levels	2000	0.35921481706462366	130.0005	130.0005	9838.856	9838.856	108386.254	109013.9925	0.5791680003997435	108239.6485	108652.41	0.38134039210226955	True
g7-traceprint-negative-levels	2000	0.6931639432600001	141.0005	141.0005	10015.856	10015.856	112574.5675	113427.9405	0.7580513245142972	113045.803	113428.6835	0.3386950155062296	False
g7-traceprint-heads	2000	0.012223858486204442	135.0005	117.0005	10620.856	9900.856	117316.5405	107976.877	-7.961079878587118	117215.414	108059.1665	-7.8114705118901835	False
g7-traceprint-nested	2000	0.09411825290817077	153.0	122.0	8343.0	7351.0	103828.5155	90384.051	-12.948720720176322	103823.4375	89998.634	-13.315686547172934	False
g7-traceprint-nested-filter	2000	0.42001189902265423	144.0	121.0	8605.0	7917.0	115997.6615	108220.7515	-6.704367915210085	115938.495	107803.906	-7.016296873613892	False
dot-three-no-input	20000	0.0009633022971966554	596.0	596.0	23472.0	23472.0	253030.37255	253031.2421	0.0003436543966062189	253028.6397	253031.85395	0.0012703107457712193	False
differences-no-input	20000	0.0725462958578671	13.0	13.0	672.0	672.0	9411.99835	9399.52915	-0.1324819611767114	9417.9743	9398.8829	-0.20271238157869576	False
take-multispec-small	20000	0.03114976305030881	109.0	109.0	5824.0	5824.0	49206.64875	49190.5265	-0.032764373127525115	49200.534	49190.256	-0.020890017169317154	False
```

```text
{
  "candidate": "candidate-05",
  "labels": 45,
  "regressions": [
    "g7-trace-pattern-decline",
    "g7-trace-predicate-decline",
    "g7-trace-levels",
    "g7-trace-negative-levels",
    "g7-tracedetail-pattern-hit",
    "g7-tracedetail-predicate-decline",
    "g7-tracedetail-negative-levels",
    "g7-tracedetail-heads",
    "g7-traceprint-pattern-decline",
    "g7-traceprint-predicate-decline",
    "g7-traceprint-levels"
  ],
  "allocation_regressions": [],
  "byte_regressions": [],
  "worst": 1.7068379990095917,
  "worst_residual": 1.7068379990095917
}
```


candidate-06 build identity:
```text
[
  {
    "binary": "rum-cli",
    "sha256": "482b1229b0fe8bc61f4158b72a5c9a3c70c46215c007f7ff1d6b761e04329fef",
    "base_sha256": "c49b3a1d0a695e7cf5d9fe620749f914d3994f72322d1f3e60473c8c8bf522f9",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-06/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "de239599b38a430da23ceb87214b9f4f9ced10477d2c72cc0c32cdc1995a0b58",
    "base_sha256": "01b3a5b1a20928cdd3eaf2e0332dce88155305651698234b2e0cadb4a06f3251",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-06/target-alloc"
  }
]
```

Every allocation, byte and paired instruction row:
```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.0026832673071019997	2.0	2.0	168.0	168.0	4166.7066	4166.6784	-0.0006767935136320347	4166.6284	4166.60775	-0.0004956045516202323	False
g7-sow-default-tag	20000	0.053219646861402126	21.0	21.0	1464.0	1464.0	16559.7977	16559.58105	-0.001308288929147139	16570.60515	16560.4534	-0.061263604485806145	False
g7-sow-tag-hit	20000	0.0761705788903666	20.0	20.0	1416.0	1416.0	19878.99625	19845.66445	-0.16767345584663973	19867.39935	19844.71215	-0.11419310399073401	False
g7-sow-tag-decline	20000	0.031637928628791556	13.0	13.0	848.0	848.0	17151.468	17155.92615	0.02599281880710791	17143.22195	17154.9212	0.06824417273558758	False
g7-sow-predicate-hit	20000	0.05842085887823867	35.0	35.0	2332.0	2332.0	33291.68725	33290.9521	-0.0022082088975583503	33345.0787	33301.9435	-0.12936001857448698	False
g7-sow-predicate-decline	20000	0.05822286054277814	28.0	28.0	1764.0	1764.0	30630.68465	30601.67115	-0.0947203770713001	30610.2913	30602.90285	-0.024137143706315367	False
g7-sow-pattern-list	20000	0.06664157314812957	37.0	37.0	2768.0	2768.0	41308.1607	41317.11595	0.02167913034190061	41332.885	41309.13005	-0.05747227661462295	False
g7-sow-nested-scopes	20000	0.04908701694480433	49.0	49.0	3552.0	3552.0	46357.71995	46362.44615	0.010195065687224059	46381.61815	46362.7265	-0.040730898906782276	False
g7-sow-repeated-tag	10000	0.2613144753496277	48.0	48.0	3417.0	3417.0	50790.9026	50798.8963	0.015738448404722938	50783.2372	50778.0563	-0.010201988462454903	False
g7-trace-plain	2000	0.34826975444315345	33.0005	25.0005	2917.704	2613.704	27909.0335	24783.7735	-11.198023034369864	27908.795	24560.489	-11.997314824950333	False
g7-trace-pattern-hit	2000	0.11862567989811001	32.0005	26.0005	2893.704	2677.704	29412.534	27842.7075	-5.337270498352842	29343.3315	27953.2865	-4.73717512273616	False
g7-trace-pattern-decline	2000	0.036326623799620705	27.0005	25.0005	2150.856	2086.856	26402.6945	26769.4915	1.389240783738943	26443.3615	26658.843	0.8148793790834885	True
g7-trace-alternatives	2000	0.5084391171986558	34.0005	30.0005	3149.704	2997.704	33118.818	32303.6705	-2.461281981742225	33108.8035	32255.2465	-2.5780363823778796	False
g7-trace-predicate-hit	2000	0.1337796438685146	182.0	152.0	10704.0	9600.0	154813.042	145568.4355	-5.971464923478475	155031.7975	145597.7805	-6.0852142283907895	False
g7-trace-predicate-decline	2000	0.31619069451558346	156.0005	150.0005	10333.704	10141.704	139344.5635	139891.6265	0.39259730430747375	139157.524	139969.6275	0.5835857642882525	True
g7-trace-levels	2000	0.01763534787542209	107.0	101.0	8320.0	8128.0	96185.1655	95981.3565	-0.21189234217204866	96190.77	96354.6215	0.1703401480204203	False
g7-trace-negative-levels	2000	0.23982683112320213	116.0	110.0	8481.0	8289.0	99378.96	99299.976	-0.07947758761011059	99983.9145	99300.6505	-0.6833739241125625	False
g7-trace-heads	2000	0.06624326505909606	125.0005	101.0005	11827.4	10915.4	116323.8015	104818.563	-9.890700227846327	114393.032	104883.4715	-8.31305922549549	False
g7-trace-nested	2000	0.48524986699812755	118.0	87.0	7672.0	6680.0	91205.313	77320.2435	-15.223970011483868	91263.4305	77292.074	-15.308822409431578	False
g7-trace-nested-filter	2000	0.07699978796587015	117.0	94.0	8000.0	7312.0	106409.9945	98958.213	-7.002896236405687	106952.193	99196.6155	-7.251443175176409	False
g7-tracedetail-plain	2000	0.19662089083332113	55.0005	45.0005	5114.4	4253.4	36661.5435	33575.215	-8.418435792262823	36497.9155	33544.719	-8.091411412249027	False
g7-tracedetail-pattern-hit	2000	0.1844833388357614	58.0005	56.0005	5242.4	5178.4	40618.948	40534.4925	-0.20792143607459845	40561.8645	40562.8535	0.0024382508353237853	False
g7-tracedetail-pattern-decline	2000	0.9269690649497342	52.0005	50.0005	3600.856	3536.856	32757.997	32802.214	0.1349807804182923	32796.809	32814.9575	0.055336176150544425	False
g7-tracedetail-alternatives	2000	0.7124896924413648	60.0005	59.0005	5498.4	5434.4	44314.1425	44559.257	0.553129285983589	44438.515	44495.8585	0.12904009056109444	False
g7-tracedetail-predicate-hit	2000	0.2116941011157225	207.0	204.0	12049.0	11953.0	149491.9545	149561.964	0.04683161728280538	149491.0645	149570.197	0.05293460198751987	False
g7-tracedetail-predicate-decline	2000	0.0298367056246418	232.0005	226.0005	14683.704	14491.704	158495.7545	158312.457	-0.1156482081039023	158371.2915	158620.866	0.15758822046356702	False
g7-tracedetail-levels	2000	1.2538831561790809	152.0	149.0	9701.0	9605.0	94251.721	94160.649	-0.09662635231880712	94214.164	94362.9745	0.15794918054996554	False
g7-tracedetail-negative-levels	2000	0.11894438130271023	155.0	152.0	9494.0	9398.0	95232.8705	95204.303	-0.02999752065648842	95499.704	95238.2305	-0.2737950894591168	False
g7-tracedetail-heads	2000	0.1350390526104528	158.0	155.0	10853.0	10757.0	96277.59	96308.1715	0.03176388191685753	96298.011	96371.6195	0.07643823505347758	False
g7-tracedetail-nested	2000	0.03457663379686782	327.0	254.0	20618.0	13332.0	179521.2295	157958.678	-12.01114294953064	179700.877	157987.595	-12.083013929865238	False
g7-tracedetail-nested-filter	2000	0.13737786126142293	340.0	337.0	21546.0	21514.0	206357.617	206723.6495	0.17737775097490527	206419.4195	207194.2535	0.37536875255093083	True
g7-traceprint-plain	2000	0.19779990078052645	40.0	32.0	2174.0	1870.0	30505.6765	27238.4235	-10.71031157102843	30745.194	27263.9245	-11.32297132358312	False
g7-traceprint-pattern-hit	2000	0.5541069117105613	35.0	29.0	2112.0	1896.0	29337.2105	28125.8235	-4.129182629684591	29360.2455	28057.2835	-4.437844363392673	False
g7-traceprint-pattern-decline	2000	1.3117046813320377	27.0	25.0	1688.0	1624.0	25302.356	25641.5795	1.340679500359565	25398.676	25677.5755	1.0980867664125427	False
g7-traceprint-alternatives	2000	0.20413214834034132	37.0	33.0	2368.0	2216.0	33023.69	32587.642	-1.3204096816558097	33321.012	32320.969	-3.0012383777539564	False
g7-traceprint-predicate-hit	2000	0.3097151647886598	206.0005	176.0005	12228.704	11124.704	170447.6315	161354.831	-5.3346593437409995	170784.094	161020.551	-5.716892464236167	False
g7-traceprint-predicate-decline	2000	0.16378302963143687	156.0005	150.0005	10333.704	10141.704	138405.954	139008.8555	0.4356037313250294	138426.002	139428.122	0.7239391339208145	True
g7-traceprint-levels	2000	0.35921481706462366	130.0005	124.0005	9838.856	9646.856	108335.872	108175.3715	-0.14815083594842404	108255.5005	108208.409	-0.04350033003633902	False
g7-traceprint-negative-levels	2000	0.6931639432600001	141.0005	135.0005	10015.856	9823.856	113049.48	112979.767	-0.061665918321773816	113124.453	112980.8045	-0.1269827134545265	False
g7-traceprint-heads	2000	0.012223858486204442	135.0005	111.0005	10620.856	9708.856	117251.0935	107695.4705	-8.149709068598154	117274.72	107609.683	-8.241364379296744	False
g7-traceprint-nested	2000	0.09411825290817077	153.0	122.0	8343.0	7351.0	104547.54	89999.7505	-13.914999339056667	103827.0855	89971.1975	-13.345157415595576	False
g7-traceprint-nested-filter	2000	0.42001189902265423	144.0	121.0	8605.0	7917.0	115936.6595	108685.504	-6.254411271872118	115938.071	108653.349	-6.283287221502931	False
dot-three-no-input	20000	0.0009633022971966554	596.0	596.0	23472.0	23472.0	253027.86075	253035.29255	0.002937146912596056	253029.9	253035.04835	0.0020346804863846657	False
differences-no-input	20000	0.0725462958578671	13.0	13.0	672.0	672.0	9410.90035	9399.3562	-0.12266785929785362	9410.9361	9399.3183	-0.1234499934602673	False
take-multispec-small	20000	0.03114976305030881	109.0	109.0	5824.0	5824.0	49199.94345	49190.3205	-0.01955886394418016	49199.75355	49190.2587	-0.019298572279136117	False
```

```text
{
  "candidate": "candidate-06",
  "labels": 45,
  "regressions": [
    "g7-trace-pattern-decline",
    "g7-trace-predicate-decline",
    "g7-tracedetail-nested-filter",
    "g7-traceprint-predicate-decline"
  ],
  "allocation_regressions": [],
  "byte_regressions": [],
  "worst": 1.389240783738943,
  "worst_residual": 1.389240783738943
}
```


candidate-07 build identity:
```text
[
  {
    "binary": "rum-cli",
    "sha256": "52586210649b0a093e2c59840424098eb7ed4ad9cbd2a1acd158acd304f59851",
    "base_sha256": "c49b3a1d0a695e7cf5d9fe620749f914d3994f72322d1f3e60473c8c8bf522f9",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-07/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "155b56f3aae48770c67a9ae251a067ab0c446a22cf256e4a745bfb84cf02371d",
    "base_sha256": "01b3a5b1a20928cdd3eaf2e0332dce88155305651698234b2e0cadb4a06f3251",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-07/target-alloc"
  }
]
```

Every allocation, byte and paired instruction row:
```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.0026832673071019997	2.0	2.0	168.0	168.0	4166.825	4166.6395	-0.004451830830420889	4166.68405	4166.82095	0.0032855862925451618	False
g7-sow-default-tag	20000	0.053219646861402126	21.0	21.0	1464.0	1464.0	16568.848	16559.79715	-0.05462570481666873	16565.3987	16560.08405	-0.03208283782508703	False
g7-sow-tag-hit	20000	0.0761705788903666	20.0	20.0	1416.0	1416.0	19868.8821	19845.5565	-0.1173976466446458	19852.8681	19845.75735	-0.03581724295039779	False
g7-sow-tag-decline	20000	0.031637928628791556	13.0	13.0	848.0	848.0	17152.6813	17156.2492	0.02080082954725171	17154.2584	17155.59535	0.007793691623536603	False
g7-sow-predicate-hit	20000	0.05842085887823867	35.0	35.0	2332.0	2332.0	33285.687	33298.17195	0.03750846422370113	33330.38475	33290.1836	-0.12061411922345	False
g7-sow-predicate-decline	20000	0.05822286054277814	28.0	28.0	1764.0	1764.0	30594.91935	30632.0909	0.12149582607086096	30615.0432	30604.27345	-0.03517796767309456	False
g7-sow-pattern-list	20000	0.06664157314812957	37.0	37.0	2768.0	2768.0	41333.54265	41314.63165	-0.04575218766059708	41315.4892	41319.14065	0.008837968690933451	False
g7-sow-nested-scopes	20000	0.04908701694480433	49.0	49.0	3552.0	3552.0	46400.3705	46363.0517	-0.08042780606675182	46404.9316	46362.5128	-0.09141011210973549	False
g7-sow-repeated-tag	10000	0.2613144753496277	48.0	48.0	3417.0	3417.0	50788.8787	50798.8792	0.019690334293609446	50753.5449	50798.7138	0.08899654219030051	False
g7-trace-plain	2000	0.34826975444315345	33.0005	25.0005	2917.704	2613.704	27907.678	24557.111	-12.005896728491706	27908.1155	24656.828	-11.649971493059063	False
g7-trace-pattern-hit	2000	0.11862567989811001	32.0005	26.0005	2893.704	2677.704	29167.743	27772.2255	-4.7844548685169075	29319.094	27776.5995	-5.261057862156315	False
g7-trace-pattern-decline	2000	0.036326623799620705	27.0005	25.0005	2150.856	2086.856	26496.8845	26627.546	0.49312023834349183	26437.0265	26647.345	0.7955452176136468	True
g7-trace-alternatives	2000	0.5084391171986558	34.0005	30.0005	3149.704	2997.704	33106.7805	32175.0025	-2.814462735209189	33042.1095	32138.0495	-2.736084389527238	False
g7-trace-predicate-hit	2000	0.1337796438685146	182.0	152.0	10704.0	9600.0	154641.1115	145210.1185	-6.098632445486518	154949.1505	145343.329	-6.199337956357487	False
g7-trace-predicate-decline	2000	0.31619069451558346	156.0005	150.0005	10333.704	10141.704	139256.046	139800.9945	0.3913284310829823	139357.622	139574.501	0.15562765558669867	False
g7-trace-levels	2000	0.01763534787542209	107.0	101.0	8320.0	8128.0	95783.913	95894.0105	0.11494362315309647	96153.932	95757.757	-0.412021632147086	False
g7-trace-negative-levels	2000	0.23982683112320213	116.0	110.0	8481.0	8289.0	101653.2535	99035.1355	-2.5755378306706156	99442.2435	99033.5725	-0.41096317381456027	False
g7-trace-heads	2000	0.06624326505909606	125.0005	101.0005	11827.4	10915.4	114008.2095	104472.7785	-8.36381085346314	114388.703	104996.9305	-8.210402123363526	False
g7-trace-nested	2000	0.48524986699812755	118.0	87.0	7672.0	6680.0	91239.59	77292.0605	-15.286707776744713	91270.017	77091.7795	-15.534386829357116	False
g7-trace-nested-filter	2000	0.07699978796587015	117.0	94.0	8000.0	7312.0	106282.566	99048.685	-6.8062724417097815	106284.3785	99054.8345	-6.802075810228314	False
g7-tracedetail-plain	2000	0.19662089083332113	55.0005	45.0005	5114.4	4253.4	36537.263	33549.0685	-8.178484797835017	36450.8965	33439.02	-8.262832438154177	False
g7-tracedetail-pattern-hit	2000	0.1844833388357614	58.0005	56.0005	5242.4	5178.4	40499.3845	40529.019	0.0731727169829055	40523.582	40583.6725	0.14828526264039077	False
g7-tracedetail-pattern-decline	2000	0.9269690649497342	52.0005	50.0005	3600.856	3536.856	32686.444	32758.225	0.21960480008165195	32728.977	32749.128	0.06156929377902909	False
g7-tracedetail-alternatives	2000	0.7124896924413648	60.0005	59.0005	5498.4	5434.4	44424.2725	44512.0705	0.19763520044138172	44368.7405	44463.985	0.21466577353035365	False
g7-tracedetail-predicate-hit	2000	0.2116941011157225	207.0	204.0	12049.0	11953.0	149779.8995	149519.893	-0.17359238513842135	149483.26	149523.824	0.02713614889051641	False
g7-tracedetail-predicate-decline	2000	0.0298367056246418	232.0005	226.0005	14683.704	14491.704	158225.876	158413.09	0.11832072271162897	158295.3065	157995.9395	-0.18911931542329175	False
g7-tracedetail-levels	2000	1.2538831561790809	152.0	149.0	9701.0	9605.0	94303.9435	94153.385	-0.15965239035841128	94407.134	94133.0425	-0.2903292244842537	False
g7-tracedetail-negative-levels	2000	0.11894438130271023	155.0	152.0	9494.0	9398.0	95257.9945	95055.3775	-0.21270340727149728	95259.3015	95247.128	-0.012779329481016344	False
g7-tracedetail-heads	2000	0.1350390526104528	158.0	155.0	10853.0	10757.0	96524.6165	96210.2075	-0.3257293438715725	96300.631	96210.3805	-0.09371745445779789	False
g7-tracedetail-nested	2000	0.03457663379686782	327.0	254.0	20618.0	13332.0	179443.171	158662.1685	-11.580826611674178	179603.619	158634.1685	-11.675405326882638	False
g7-tracedetail-nested-filter	2000	0.13737786126142293	340.0	337.0	21546.0	21514.0	207150.824	207138.381	-0.006006734494090615	206647.925	207074.458	0.20640565348044593	False
g7-traceprint-plain	2000	0.19779990078052645	40.0	32.0	2174.0	1870.0	30624.7645	27276.0495	-10.934663677168842	30623.076	27360.969	-10.652447193743697	False
g7-traceprint-pattern-hit	2000	0.5541069117105613	35.0	29.0	2112.0	1896.0	29556.428	27942.724	-5.459739586935209	29347.909	28027.5805	-4.498884400929548	False
g7-traceprint-pattern-decline	2000	1.3117046813320377	27.0	25.0	1688.0	1624.0	25303.773	25549.504	0.971123950566577	25291.9415	25521.738	0.9085759588681741	False
g7-traceprint-alternatives	2000	0.20413214834034132	37.0	33.0	2368.0	2216.0	33126.352	32202.5865	-2.7886122202649966	33279.1455	32201.989	-3.23673124359517	False
g7-traceprint-predicate-hit	2000	0.3097151647886598	206.0005	176.0005	12228.704	11124.704	170279.6985	160990.7555	-5.4551089071842584	170292.482	160848.5545	-5.5457101740991614	False
g7-traceprint-predicate-decline	2000	0.16378302963143687	156.0005	150.0005	10333.704	10141.704	138444.708	138794.2265	0.2524607152192271	138522.8695	138840.194	0.22907733657653306	True
g7-traceprint-levels	2000	0.35921481706462366	130.0005	124.0005	9838.856	9646.856	108268.212	107947.1435	-0.29654918472283853	108225.267	107941.2615	-0.2624206970078591	False
g7-traceprint-negative-levels	2000	0.6931639432600001	141.0005	135.0005	10015.856	9823.856	112995.788	113150.5	0.13691837787794814	113021.2815	112383.791	-0.5640446573772051	False
g7-traceprint-heads	2000	0.012223858486204442	135.0005	111.0005	10620.856	9708.856	117235.1235	107292.2255	-8.481159658606918	117240.611	107526.353	-8.285744945495043	False
g7-traceprint-nested	2000	0.09411825290817077	153.0	122.0	8343.0	7351.0	103922.807	89514.9915	-13.863959140364635	103610.0875	90097.619	-13.041653400784925	False
g7-traceprint-nested-filter	2000	0.42001189902265423	144.0	121.0	8605.0	7917.0	115966.8405	108549.2855	-6.3962723896060725	116032.553	109025.197	-6.0391293812177	False
dot-three-no-input	20000	0.0009633022971966554	596.0	596.0	23472.0	23472.0	253028.37265	253031.98805	0.0014288516193339262	253030.885	253031.4596	0.00022708690283756994	False
differences-no-input	20000	0.0725462958578671	13.0	13.0	672.0	672.0	9397.6365	9399.96025	0.02472696193345225	9397.60115	9399.2247	0.017276217346173794	False
take-multispec-small	20000	0.03114976305030881	109.0	109.0	5824.0	5824.0	49200.60365	49190.3671	-0.020805740662888095	49198.52375	49190.332	-0.01665039797053014	False
```

```text
{
  "candidate": "candidate-07",
  "labels": 45,
  "regressions": [
    "g7-trace-pattern-decline",
    "g7-traceprint-predicate-decline"
  ],
  "allocation_regressions": [],
  "byte_regressions": [],
  "worst": 0.971123950566577,
  "worst_residual": 0.7955452176136468
}
```


candidate-08 build identity:
```text
[
  {
    "binary": "rum-cli",
    "sha256": "7c387acd61d0956f4d89e4c4c1f1ac940599f8ccb1c1609625e65ae25cb90508",
    "base_sha256": "c49b3a1d0a695e7cf5d9fe620749f914d3994f72322d1f3e60473c8c8bf522f9",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-08/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "017ebaa291acdcfd50948e14ffea7a436f2d67a2639f70c5b4eae4b2764f14e3",
    "base_sha256": "01b3a5b1a20928cdd3eaf2e0332dce88155305651698234b2e0cadb4a06f3251",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-582/candidate-08/target-alloc"
  }
]
```

Every allocation, byte and paired instruction row:
```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.0026832673071019997	2.0	2.0	168.0	168.0	4166.59175	4166.5958	9.720174769523027e-05	4166.49615	4153.5131	-0.3116059521619863	False
g7-sow-default-tag	20000	0.053219646861402126	21.0	21.0	1464.0	1464.0	16567.1121	16559.89955	-0.04353534856567087	16555.354	16559.54725	0.025328664068435813	False
g7-sow-tag-hit	20000	0.0761705788903666	20.0	20.0	1416.0	1416.0	19849.4531	19845.4006	-0.020416179627624764	19868.5743	19844.59265	-0.1207014133872808	False
g7-sow-tag-decline	20000	0.031637928628791556	13.0	13.0	848.0	848.0	17150.73255	17155.57025	0.02820695842522536	17142.10125	17154.8187	0.07418839624460105	False
g7-sow-predicate-hit	20000	0.05842085887823867	35.0	35.0	2332.0	2332.0	33285.59395	33293.17035	0.022761799027470886	33290.0253	33333.19335	0.12967262599226093	False
g7-sow-predicate-decline	20000	0.05822286054277814	28.0	28.0	1764.0	1764.0	30611.6872	30602.49935	-0.03001419013586304	30615.66235	30602.76215	-0.042135949412180906	False
g7-sow-pattern-list	20000	0.06664157314812957	37.0	37.0	2768.0	2768.0	41308.06055	41310.9725	0.007049350565546675	41317.9677	41306.04035	-0.028867223302464495	False
g7-sow-nested-scopes	20000	0.04908701694480433	49.0	49.0	3552.0	3552.0	46360.24005	46362.4736	0.004817813707580498	46373.8035	46362.4869	-0.02440300157824371	False
g7-sow-repeated-tag	10000	0.2613144753496277	48.0	48.0	3417.0	3417.0	50817.6962	50799.9763	-0.03486954609326354	50839.7344	50845.24	0.010829324867600576	False
g7-trace-plain	2000	0.34826975444315345	33.0005	25.0005	2917.704	2613.704	27959.7855	24566.9775	-12.134599530457779	27907.7555	24558.881	-11.999798765615521	False
g7-trace-pattern-hit	2000	0.11862567989811001	32.0005	26.0005	2893.704	2677.704	29685.834	27739.0785	-6.5578602238360535	29295.092	27734.8585	-5.3259211474741335	False
g7-trace-pattern-decline	2000	0.036326623799620705	27.0005	25.0005	2150.856	2086.856	26611.244	26463.8225	-0.5539819934761381	26426.8235	26624.9105	0.7495679531821287	False
g7-trace-alternatives	2000	0.5084391171986558	34.0005	30.0005	3149.704	2997.704	33114.743	32051.405	-3.211071274205579	33109.85	31970.5715	-3.4409050478936054	False
g7-trace-predicate-hit	2000	0.1337796438685146	182.0	152.0	10704.0	9600.0	155025.8625	145536.481	-6.121160267694037	154996.776	145166.8445	-6.342023204405233	False
g7-trace-predicate-decline	2000	0.31619069451558346	156.0005	150.0005	10333.704	10141.704	139223.644	139427.95	0.1467466258820327	139173.5945	139420.0335	0.17707310132022425	False
g7-trace-levels	2000	0.01763534787542209	107.0	101.0	8320.0	8128.0	96164.7315	95675.5115	-0.5087312077609263	96163.8135	95710.857	-0.4710259332633471	False
g7-trace-negative-levels	2000	0.23982683112320213	116.0	110.0	8481.0	8289.0	99364.8225	98567.807	-0.8021103243051608	99377.006	99776.209	0.40170560179686454	False
g7-trace-heads	2000	0.06624326505909606	125.0005	101.0005	11827.4	10915.4	114384.0955	104601.6325	-8.552293006504552	114394.0905	104676.1675	-8.495126765311367	False
g7-trace-nested	2000	0.48524986699812755	118.0	87.0	7672.0	6680.0	91751.41	77320.7235	-15.728026958931762	91085.583	77320.864	-15.111852552999515	False
g7-trace-nested-filter	2000	0.07699978796587015	117.0	94.0	8000.0	7312.0	106330.7355	99198.018	-6.708048680806877	106284.4725	99006.2675	-6.84785352818118	False
g7-tracedetail-plain	2000	0.19662089083332113	55.0005	45.0005	5114.4	4253.4	36487.246	33544.2045	-8.065945837622268	36401.669	33480.745	-8.02414856307825	False
g7-tracedetail-pattern-hit	2000	0.1844833388357614	58.0005	56.0005	5242.4	5178.4	40496.262	40495.6545	-0.001500138457233291	40496.779	40497.158	0.0009358769002432155	False
g7-tracedetail-pattern-decline	2000	0.9269690649497342	52.0005	50.0005	3600.856	3536.856	32676.1295	32809.4425	0.4079828365229021	32733.723	32728.837	-0.014926502555190346	False
g7-tracedetail-alternatives	2000	0.7124896924413648	60.0005	59.0005	5498.4	5434.4	44345.9955	44487.925	0.3200503188613757	44370.6515	44488.899	0.2664993548719963	False
g7-tracedetail-predicate-hit	2000	0.2116941011157225	207.0	204.0	12049.0	11953.0	149349.9055	149330.026	-0.013310688033862927	149428.0405	149442.246	0.009506582534624108	False
g7-tracedetail-predicate-decline	2000	0.0298367056246418	232.0005	226.0005	14683.704	14491.704	158427.4635	157882.3635	-0.34406913293792973	158469.1585	158357.032	-0.07075603925793539	False
g7-tracedetail-levels	2000	1.2538831561790809	152.0	149.0	9701.0	9605.0	94151.472	94237.131	0.09097999020131198	94241.984	93855.1185	-0.4105022873881814	False
g7-tracedetail-negative-levels	2000	0.11894438130271023	155.0	152.0	9494.0	9398.0	95619.3535	94863.1875	-0.7908085260166509	95437.1595	95057.871	-0.3974222430624619	False
g7-tracedetail-heads	2000	0.1350390526104528	158.0	155.0	10853.0	10757.0	96314.5065	96164.684	-0.15555548737614666	96270.986	96460.002	0.1963374510363769	False
g7-tracedetail-nested	2000	0.03457663379686782	327.0	254.0	20618.0	13332.0	179606.686	157815.844	-12.132533863466522	179895.496	157991.72	-12.175833462778861	False
g7-tracedetail-nested-filter	2000	0.13737786126142293	340.0	337.0	21546.0	21514.0	206409.246	207122.7665	0.34568243129959875	206358.555	207065.179	0.3424253479580752	True
g7-traceprint-plain	2000	0.19779990078052645	40.0	32.0	2174.0	1870.0	30528.59	27227.2165	-10.814038578263851	30637.1475	27223.8455	-11.141056784088665	False
g7-traceprint-pattern-hit	2000	0.5541069117105613	35.0	29.0	2112.0	1896.0	29412.0215	27892.9015	-5.164962904708881	29324.8655	27804.4335	-5.184787633552834	False
g7-traceprint-pattern-decline	2000	1.3117046813320377	27.0	25.0	1688.0	1624.0	25400.9545	25645.066	0.9610327832365373	25304.6235	25465.053	0.6339928353409352	False
g7-traceprint-alternatives	2000	0.20413214834034132	37.0	33.0	2368.0	2216.0	33113.553	32106.9635	-3.0398112217073114	33161.076	32167.4665	-2.996312604572915	False
g7-traceprint-predicate-hit	2000	0.3097151647886598	206.0005	176.0005	12228.704	11124.704	170323.035	160880.823	-5.543708166073957	170311.465	160798.9025	-5.585391740949442	False
g7-traceprint-predicate-decline	2000	0.16378302963143687	156.0005	150.0005	10333.704	10141.704	138423.181	139419.2155	0.7195575862398274	139115.743	138675.934	-0.316146102889292	False
g7-traceprint-levels	2000	0.35921481706462366	130.0005	124.0005	9838.856	9646.856	108298.8335	107800.3285	-0.46030505028477053	108229.2425	107394.871	-0.7709298159413791	False
g7-traceprint-negative-levels	2000	0.6931639432600001	141.0005	135.0005	10015.856	9823.856	113027.368	112574.867	-0.40034640105925945	113031.0065	112583.4745	-0.39593737493615055	False
g7-traceprint-heads	2000	0.012223858486204442	135.0005	111.0005	10620.856	9708.856	117272.602	107144.4015	-8.636459264372753	117242.8265	107186.7285	-8.577154185207226	False
g7-traceprint-nested	2000	0.09411825290817077	153.0	122.0	8343.0	7351.0	103825.347	90063.1195	-13.255171205929116	103827.178	90003.4205	-13.314199390067216	False
g7-traceprint-nested-filter	2000	0.42001189902265423	144.0	121.0	8605.0	7917.0	115937.4915	108411.0665	-6.491795624196339	115772.4275	108412.3235	-6.357389370625411	False
dot-three-no-input	20000	0.0009633022971966554	596.0	596.0	23472.0	23472.0	253030.3081	253031.93945	0.0006447251367891482	253032.60795	253031.22395	-0.0005469650774303503	False
differences-no-input	20000	0.0725462958578671	13.0	13.0	672.0	672.0	9408.7808	9399.8901	-0.09449364576544772	9408.9556	9399.41285	-0.10142198991776041	False
take-multispec-small	20000	0.03114976305030881	109.0	109.0	5824.0	5824.0	49198.9136	49190.4241	-0.017255462323872983	49207.7798	49190.3189	-0.03548402319910604	False
```

```text
{
  "candidate": "candidate-08",
  "labels": 45,
  "regressions": [
    "g7-tracedetail-nested-filter"
  ],
  "allocation_regressions": [],
  "byte_regressions": [],
  "worst": 0.9610327832365373,
  "worst_residual": 0.34568243129959875
}
```

Callgrind attribution, diagnostic only: `.temp/brief-582/candidate-01/attribution/slopes.tsv`
```text
label	base_total	candidate-01_total	base_trace_self	candidate-01_trace_self	candidate-01_continuation_self
g7-trace-plain	28069.09	28684.57	1864.0	2438.0	0
g7-trace-nested-filter	107834.84	110581.04	6810.0	8749.0	0
g7-tracedetail-plain	36641.91	37131.44	1782.0	2043.0	0
```

Callgrind attribution, diagnostic only: `.temp/brief-582/candidate-06/attribution/slopes.tsv`
```text
label	base_total	candidate-06_total	base_trace_self	candidate-06_trace_self	candidate-06_continuation_self
g7-trace-pattern-decline	26322.52	26566.14	2011.0	2304.0	0
g7-tracedetail-nested-filter	212901.99	214293.34	9485.0	10271.0	0
```

Callgrind attribution, diagnostic only: `.temp/brief-582/candidate-08/attribution/slopes.tsv`
```text
label	base_total	candidate-08_total	base_trace_self	candidate-08_trace_self	candidate-08_continuation_self
g7-tracedetail-nested-filter	212696.96	214379.04	9485.0	10082.0	0
```

The selected residual profile uses the same passing nested TraceDetail statement at N=100 and 200. Callgrind total slope is 212696.96 → 214379.04, while trace-named function self cost is 9485 → 10082 (+597). The common driver has 4512 self instructions in place of the removed detail walkers/match helpers; other differences include symbol lookup, vector conversion/drop, allocator routines and unresolved addresses. The frame has zero self cost in this non-suspending workload. These diagnostic counts do not replace the CPU-2 hardware instruction gate or fully explain every code-generation/allocator delta. No further candidate was tried.
```text
{
  "g7-tracedetail-nested-filter": {
    "row": {
      "label": "g7-tracedetail-nested-filter",
      "base_total": 212696.96,
      "base_trace_self": 9485.0,
      "base_continuation_self": 0,
      "base_trace_functions": {
        "rum_runtime::runtime::debug_tools::trace::trace_detail_association": 2046.0,
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::trace_detail_item_matches": 1986.0,
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::finish_structured_trace_capture_expr": 1605.0,
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::trace_detail_candidate_matches": 1076.0,
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::filter_structured_trace_detail_items": 787.0,
        "rum_runtime::runtime::debug_tools::trace::trace_detail_hold": 686.0,
        "rum_runtime::runtime::debug_tools::trace::builder::StructuredTraceBuilder::push_item": 323.0,
        "rum_runtime::runtime::debug_tools::trace::normalize_trace_patterns": 290.0,
        "rum_runtime::runtime::debug_tools::trace::builder::StructuredTraceBuilder::close_to_depth": 228.0,
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::eval_trace_named": 150.0,
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::eval_structured_trace_capture_expr": 95.0,
        "rum_runtime::runtime::debug_tools::trace::options::<impl rum_runtime::runtime::model::session::Session>::finish_trace_option": 84.0,
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::truncate_trace_from": 73.0,
        "rum_runtime::runtime::debug_tools::trace::builder::StructuredTraceBuilder::finish": 34.0,
        "core::ptr::drop_in_place<rum_runtime::runtime::debug_tools::trace::builder::StructuredTraceBuilder>": 22.0
      },
      "candidate-08_total": 214379.04,
      "candidate-08_trace_self": 10082.0,
      "candidate-08_continuation_self": 0,
      "candidate-08_trace_functions": {
        "rum_runtime::runtime::debug_tools::trace::filter::<impl rum_runtime::runtime::model::session::Session>::drive_trace_filter": 4512.0,
        "rum_runtime::runtime::debug_tools::trace::trace_detail_association": 2046.0,
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::finish_structured_trace_capture_expr": 1598.0,
        "rum_runtime::runtime::debug_tools::trace::trace_detail_hold": 686.0,
        "rum_runtime::runtime::debug_tools::trace::builder::StructuredTraceBuilder::push_item": 323.0,
        "rum_runtime::runtime::debug_tools::trace::builder::StructuredTraceBuilder::close_to_depth": 228.0,
        "rum_runtime::runtime::debug_tools::trace::normalize_trace_patterns": 174.0,
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::eval_trace_named": 150.0,
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::eval_structured_trace_capture_expr": 95.0,
        "rum_runtime::runtime::debug_tools::trace::options::<impl rum_runtime::runtime::model::session::Session>::finish_trace_option": 84.0,
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::truncate_trace_from": 73.0,
        "rum_runtime::runtime::debug_tools::trace::builder::StructuredTraceBuilder::finish": 34.0,
        "core::ptr::drop_in_place<alloc::vec::Vec<rum_runtime::runtime::debug_tools::trace::filter::state::LevelWork>>": 29.0,
        "rum_runtime::runtime::debug_tools::trace::filter::<impl rum_runtime::runtime::model::session::Session>::finish_trace_filter_root": 28.0,
        "core::ptr::drop_in_place<rum_runtime::runtime::debug_tools::trace::builder::StructuredTraceBuilder>": 22.0
      }
    },
    "top_self_deltas": [
      [
        "rum_runtime::runtime::debug_tools::trace::filter::<impl rum_runtime::runtime::model::session::Session>::drive_trace_filter",
        4512.0
      ],
      [
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::trace_detail_item_matches",
        -1986.0
      ],
      [
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::trace_detail_candidate_matches",
        -1076.0
      ],
      [
        "rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::filter_structured_trace_detail_items",
        -787.0
      ],
      [
        "rum_core::expr::Symbol::system",
        401.57000000000016
      ],
      [
        "<T as alloc::slice::hack::ConvertVec>::to_vec",
        -392.0
      ],
      [
        "core::ptr::drop_in_place<rum_runtime::runtime::kernel_data::text_rendering::transparent_hold_form_expr::Work>",
        216.0
      ],
      [
        "0x00000000001b1980",
        180.0
      ],
      [
        "0x00000000001b1220",
        170.14000000000033
      ],
      [
        "hashbrown::map::HashMap<K,V,S,A>::insert",
        127.43000000000029
      ],
      [
        "rum_runtime::runtime::debug_tools::trace::normalize_trace_patterns",
        -116.0
      ],
      [
        "mi_theap_malloc_aligned",
        -64.14000000000033
      ],
      [
        "mi_free",
        -59.85999999999967
      ],
      [
        "rum_runtime::runtime::evaluation_cache::stability::association_front::<impl rum_runtime::runtime::model::session::Session>::begin_association_call_cache_visit",
        58.789999999999964
      ],
      [
        "<rum_core::expr::Atom as core::clone::Clone>::clone",
        58.0
      ],
      [
        "alloc::raw_vec::RawVec<T,A>::grow_one",
        44.0
      ],
      [
        "rum_core::expr::Symbol::is_system_name",
        38.0
      ],
      [
        "_mi_page_malloc_zero",
        -37.51000000000022
      ],
      [
        "mi_theap_malloc_zero_aligned_at_overalloc",
        32.75999999999999
      ],
      [
        "mi_page_free_list_extend",
        29.559999999999945
      ],
      [
        "core::ptr::drop_in_place<alloc::vec::Vec<rum_runtime::runtime::debug_tools::trace::filter::state::LevelWork>>",
        29.0
      ],
      [
        "rum_runtime::runtime::debug_tools::trace::filter::<impl rum_runtime::runtime::model::session::Session>::finish_trace_filter_root",
        28.0
      ],
      [
        "alloc::raw_vec::finish_grow",
        27.0
      ],
      [
        "rum_core::expr::intern_symbol_text",
        -25.15000000000009
      ],
      [
        "mi_bin",
        21.0
      ],
      [
        "mi_bitmap_setN",
        20.959999999999994
      ],
      [
        "core::ptr::drop_in_place<rum_core::expr::Atom>",
        20.0
      ],
      [
        "mi_theap_malloc_zero_aligned_at_generic",
        18.83000000000004
      ],
      [
        "mi_bbitmap_try_find_and_clear",
        18.58
      ],
      [
        "mi_malloc_aligned",
        -15.0
      ],
      [
        "rum_runtime::runtime::evaluation_cache::stability::<impl rum_runtime::runtime::model::session::Session>::cached_expr_stability_with_bloom",
        14.099999999999909
      ],
      [
        "mi_arenas_page_alloc_fresh",
        12.64
      ],
      [
        "mi_option_get",
        12.32
      ],
      [
        "_mi_page_free",
        11.840000000000003
      ],
      [
        "_mi_theap_malloc_zero",
        8.900000000000006
      ]
    ]
  }
}
```

Final selected-source/base/candidate hash verification after the workspace gate:
```text
{
  "source_files": 1275,
  "selected_source_matches_build": true,
  "binaries": [
    {
      "path": ".temp/brief-582/candidate-08/rum-cli",
      "sha256": "7c387acd61d0956f4d89e4c4c1f1ac940599f8ccb1c1609625e65ae25cb90508"
    },
    {
      "path": ".temp/brief-582/base/rum-cli",
      "sha256": "c49b3a1d0a695e7cf5d9fe620749f914d3994f72322d1f3e60473c8c8bf522f9"
    },
    {
      "path": ".temp/brief-582/candidate-08/rum-cli-alloc",
      "sha256": "017ebaa291acdcfd50948e14ffea7a436f2d67a2639f70c5b4eae4b2764f14e3"
    },
    {
      "path": ".temp/brief-582/base/rum-cli-alloc",
      "sha256": "01b3a5b1a20928cdd3eaf2e0332dce88155305651698234b2e0cadb4a06f3251"
    }
  ]
}
```


### 4. Stop-loss, residuals and open questions

```text
{
  "selected": "candidate-08",
  "rule": "fewest distinct failing labels, then lowest worst failing sample; all measurements retained",
  "restored": [],
  "performance_ok": false,
  "summary": {
    "candidate": "candidate-08",
    "labels": 45,
    "regressions": [
      "g7-tracedetail-nested-filter"
    ],
    "allocation_regressions": [],
    "byte_regressions": [],
    "worst": 0.9610327832365373,
    "worst_residual": 0.34568243129959875
  },
  "source_manifest_verified": true,
  "binary_hashes_verified": true
}
```

```text
{
  "measured_candidates": 8,
  "cap": 8,
  "tuning_stopped": true,
  "stop_reason": "eight measured candidates cap reached",
  "consecutive_no_progress": 0,
  "history": [
    {
      "candidate": "candidate-01",
      "repaired_sites": 13,
      "passing_fixtures": 51,
      "regressing_labels": 25,
      "worst_regression": 4.01877201401406,
      "progress": true,
      "no_progress_streak": 0
    },
    {
      "candidate": "candidate-02",
      "repaired_sites": 13,
      "passing_fixtures": 51,
      "regressing_labels": 22,
      "worst_regression": 3.584964900854559,
      "progress": true,
      "no_progress_streak": 0
    },
    {
      "candidate": "candidate-03",
      "repaired_sites": 13,
      "passing_fixtures": 51,
      "regressing_labels": 14,
      "worst_regression": 2.837988761314736,
      "progress": true,
      "no_progress_streak": 0
    },
    {
      "candidate": "candidate-04",
      "repaired_sites": 13,
      "passing_fixtures": 51,
      "regressing_labels": 11,
      "worst_regression": 2.814860201322822,
      "progress": true,
      "no_progress_streak": 0
    },
    {
      "candidate": "candidate-05",
      "repaired_sites": 13,
      "passing_fixtures": 51,
      "regressing_labels": 11,
      "worst_regression": 1.7068379990095917,
      "progress": true,
      "no_progress_streak": 0
    },
    {
      "candidate": "candidate-06",
      "repaired_sites": 13,
      "passing_fixtures": 51,
      "regressing_labels": 4,
      "worst_regression": 1.389240783738943,
      "progress": true,
      "no_progress_streak": 0
    },
    {
      "candidate": "candidate-07",
      "repaired_sites": 13,
      "passing_fixtures": 51,
      "regressing_labels": 2,
      "worst_regression": 0.7955452176136468,
      "progress": true,
      "no_progress_streak": 0
    },
    {
      "candidate": "candidate-08",
      "repaired_sites": 13,
      "passing_fixtures": 51,
      "regressing_labels": 1,
      "worst_regression": 0.34568243129959875,
      "progress": true,
      "no_progress_streak": 0
    }
  ],
  "selected": "candidate-08",
  "selected_performance_ok": false
}
```

Disclosed iteration failures: the intentional 18 red cases; a malformed initial closed-list insertion caught by fmt and fixed before production compilation; the first structural build’s quoted token syntax error; then an omitted constructor allowlist entry and a trailing-comma-sensitive root token check, fixed with all falsifiers retained. The first reconciliation script retained a step-A-only source anchor and was corrected before any inventory write. The first noise run rejected a correct nested Print stream because its scratch expectation stripped leading indentation: all original bytes and counts were revalidated, the original false verdict was saved, and only unmeasured processes resumed. No recorded perf-stat sample was discarded or rerun to improve its number. The first standing-gate launcher stopped before running a gate because scratch select.py shadowed Python’s standard-library select module; it was renamed select-candidate.py and its cached module removed. Certificate replays initially rejected stale cells as expected; both final replays are green.
Focused runs beyond the design plan’s nominal eight were required by the recorded structural failures and successive measured performance defects; each production change was followed by the same 51-case gate. The dispatched eight-measured-candidate cap and two-consecutive-no-progress rule remain binding. Required broad checks are recorded above; any nonzero or missing gate is reported as such. No scope extension, per-level frame fallback, unsafe code, lease, dependency or lockfile change was taken.
Residual `g7-tracedetail-nested-filter`: instruction samples 0.34568243129959875% and 0.3424253479580752% against floor 0.13737786126142293%; allocations 340.0 → 337.0 and bytes 21546.0 → 21514.0.
The open acceptance issue is the remaining nested TraceDetail instruction regression. The local continuation repair is green, but the required performance gate is not. The eight-candidate cap prevents further tuning under this brief. All changes are left uncommitted for the director’s authoritative gate.
