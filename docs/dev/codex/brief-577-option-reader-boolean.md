# brief-577 — G5 step A: a typed read-and-complete boundary, and the boolean readers

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

A delayed option value is evaluated when its builtin reads it; a suspension
inside it completes the request with the option instead of the builtin's
result (`Count[…, Heads :> (…; InputString["p: "]; …)]` resumes to `False`
where the queued path returns `3`). brief-576 designed the repair: a shared,
typed **read-and-complete** boundary entered at each existing read position —
the reader takes the caller's continuation and returns the completed
builtin's `Expr`, so a read that forgets its continuation does not compile —
with one generic option-read suspension frame and per-family frozen
continuation data. This step installs that protocol and migrates every
`system_option_bool` consumer: P0193, P0711, P0714, P0764, P0767, P0848,
P1385, P1389, P1393, P1401, P1410, P2354 (Trace, Cases, DeleteCases, Replace,
Count, MemberQ, FreeQ, Level, Position, Depth). Lost continuations
**43 -> 31**.

brief-576's report and `.temp/brief-576/draft-G5-A.md` are the raw material;
**this brief governs** where they differ.

## Director rulings on brief-576's questions

1. **The typed CPS surface and the kernel signature changes are approved.**
   No legacy getter that returns a bare `bool` survives: an escape hatch would
   keep the defect compilable.
2. **I04 is authorized, exactly:** the new public generic inputs —
   `Session::system_option_bool(next)` (and, in step C,
   `Session::option_rule_expr_for_symbol(next)`),
   `OptionReadContinuation::finish(value)`,
   `FrozenOptionReadContinuation::finish(value)`, and the generic frame's
   owned port if discovery reports one — enter the ownership inventories as
   `UnknownHostPayload` rows with disposition **`PublicInput`**, beside and
   distinct from the existing `OptionReentry` row (which is not touched). They
   are honest: a caller-supplied continuation is public input. **Any other new
   `UnknownHostPayload` row is a STOP.**
3. The cold-only reparse of pure data (step B's Names pattern, step C's
   ArrayFilter shape) and the A/B/C split are accepted.
4. **Performance:** instruction slopes on one-codegen-unit builds decide; **no
   timing suite** (`TEMPLATE.md` pitfall 8; briefs 566-575).

## Context pointers

- brief-576 and its report; `.temp/brief-576/inventory.md`, `design.md`,
  `inventory-plan.md` (I01-I08), `performance-plan.md`,
  `reader-references.tsv`, `g5-current-rows.json`, `cases.json`, `probe.rs`,
  `oracle.tsv`, `lifecycle.tsv`.
- `crates/rum-runtime/src/runtime/options/core.rs` (and the certified C01
  `:87-103` and C02 `:190-204`, which stay verbatim);
  `crates/rum-stdlib/src/tests/retention_closure/local/{certificates,production,contracts}.rs`.
- `suspension/core.rs:764-855`; the accepted continuation patterns of briefs
  563, 568, 572-575.

## Scope

**Objective:** the read-and-complete protocol, the boolean readers'
migration, the enforcement tests, the owning semantic tests, and the standing
and performance gates.

**Edit surface:** `crates/rum-runtime/src/runtime/options/{core.rs,read_phase.rs}`,
`runtime/options.rs`, `runtime/kernel_api.rs`; `debug_tools/trace.rs` and new
`trace/options.rs`; `kernel_data/rewrite_data/cases/{collect.rs,collect/options.rs,delete.rs,delete/options.rs}`;
`kernel_data/rewrite_data/replace/{once.rs,once/options.rs}`;
`tree_walkers/{count_like.rs,count_like/options.rs,position.rs,position/options.rs,level.rs,level/options.rs}`;
stdlib `structure_ops/{surface.rs,surface/options.rs}`; new
`options/tests/read_phase.rs` with its compile-fail and compile-pass controls;
new `crates/rum-stdlib/src/tests/options_suspension/read_inventory.rs` and its
module line; new `crates/rum-stdlib/tests/option_reader_continuations.rs` and
children (`support`, `boolean`, `lifecycle`); rum-runtime's frame lists.

**Inventory exceptions, authorized by name** (`inventory-plan.md`): I01 (the
boolean signature, the options-private helper, the exports), I02's F00, F01
and S01-S08 (frame lists: **+10** derive entries, **+1** `PureFrameAbandon`,
named frames **559 -> 560**), I03 for the affected source rows only, I04 as
ruled above, I05, I06's Cases / Level / Position / Replace / Trace-trio
evidence anchors, I07, I08. No addition to the `OptionReentry` domain.

**The certified bodies stay verbatim.** C01 and C02, the accounting drivers,
the `OptionReentry` domain and the 33-member consumer protocol are unchanged,
and both `local_production_acceptance` runs must earn the same two C01 and one
C02 marks **on their merits** — a refreshed fingerprint alone is not a pass.
If a certified body must change, STOP.

**Standing duty — always in scope** (`TEMPLATE.md`). STOP on any other
existing row change or disappearance, any `UnknownHostPayload` beyond I04, any
new row in untouched code, any self-reaching state.

**Fence:** no other suspension group (steps B and C keep their readers); no
`Expr`, matcher or evaluator redesign; no leases, `unsafe`, dependencies,
lockfile, catalog reclassification or request-policy change; the rum-host API
unchanged. No edits to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier
briefs or docs beyond the new protocol's own rustdoc.

**Disclosure duty:** every changed function, frame and root field, inventory
row, measured label, and any adjacent defect.

## Deliverable

brief-576 design.md §3: the reader takes `next: C` and returns the completed
`Expr`; ordinary success decodes and calls `next.finish`; an ordinary error
returns unfrozen; `InputRequired` freezes `next` in a cold, non-inlined helper,
inserts the frame and propagates. Every read stays at its current position:
Trace and Depth read before their arity checks, Heads defaults are still read
when an explicit override follows, Cases and DeleteCases converge both setup
entries on one post-read tail without re-evaluating target or pattern,
Count / MemberQ / FreeQ share one finish discriminator. A second suspension
installs the new continuation once; pure abandonment runs nothing pending.
The enforcement: compile-fail controls for a missing continuation, an
inaccessible raw getter and a non-rootable frozen capture (each with a
compile-pass twin), and a source gate that inventories every reader call,
requires tail use, and records only steps B and C's remaining legacy sites —
a new caller cannot join that list silently.

## Tests

Red first, recorded before any production edit: every `cases.json` witness in
this step's families (the Heads witnesses, both Cases / DeleteCases
after-target cases, their twice variants, Count / MemberQ / FreeQ / Position /
Replace after-target, TraceDetail and TracePrint, the Count option error,
first and second cancellation, answer-free abandonment) through the rum-vs-rum
oracle — value, error, termination, ordered messages, prints, `log`, actual
suspension, exact prompts in execution order, no owner after, `1 + 1`. Keep
the passing controls. Every earlier group's suite stays green.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. Every reader call is a tail call; a later unframed `?` discards the
   guarantee — the source gate enforces it.
3. Preserve read order, conditional reachability, the delayed-versus-ordinary
   policy, the current late default lookup and every completed guest prefix.
4. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression. Falling compile-error counts are progress.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-577/tmp …`. Bounds: focused 150 s; runtime,
ownership and retention 300 s; a one-unit release build 900 s; a metric
process 150 s; `cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test option_reader_continuations
cargo test -p rum-stdlib --lib options_suspension
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Performance gate.** Base: `.temp/brief-577/base/rum-cli` and
`rum-cli-alloc` — the one-codegen-unit build of the incoming source (hashes in
`sha256.txt`); do not rebuild them. Candidates: the working tree with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and `--features
rum-cli/alloc-count` (two arguments), fresh target directory each, hash
checked. Panel: `performance-plan.md`'s step-A labels — the small default
reads (`Cases[{1,2,3},_Integer]`, `Count[…]`, `MemberQ[…]`, `FreeQ[…]`,
`Position[…]`, `DeleteCases[…]`, `Replace[…]`, `Level[…]`, `Depth[…]`,
`Trace[1+2]` and its variants), the delayed-but-not-suspending defaults and
the explicit overrides — plus the neighbours `cases-wide-no-match`,
`count-wide-no-match`, `position-wide-no-match`, `deletecases-wide-no-match`,
`replace-wide-no-match`, `matchq-integer-*`, and the controls
`dot-three-no-input`, `differences-no-input`, `take-multispec-small`
(`.temp/brief-563/perf/`). Scratch workloads, each result checked first.
Allocation counts, then two paired samples of `taskset -c 2 perf stat -x, -e
cpu_core/instructions/u`, one label per process, stdin `/dev/null`,
`(count(2N) - count(N)) / N`, raw output saved first. A label regresses when
both samples exceed +0.05%. **No timing suite.** At most eight measured
candidates; report every residual with its callgrind attribution.

## Checkpointing — protect the scene

Everything under `.temp/brief-577/`; `PROGRESS.md` after the red capture, each
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

1. Changed files and functions, frame and root fields, size assessment.
2. Verbatim red and green output, compile controls, the source gate, ownership
   and certificate results (marks earned), the broad suites; every inventory
   row change, the I04 rows named.
3. Twelve sites repaired (43 -> 31 predicted); every measured row, both
   samples.
4. Stop-loss counters, residuals, open questions.

---

## Implementation report — 2026-10-06

The typed boolean read-and-complete protocol and all twelve step-A read sites are implemented. Final measured source: **candidate-05**. The instruction gate has **6 residual regressions** under the required two-sample +0.05% rule; this is not a performance pass. The allocation-count and allocation-byte slopes are identical on every measured row. No git commands or network access were used. No timing suite or external-engine comparison was run. No unsafe code, workspace dependency changes or root lockfile edits were introduced. Changes remain uncommitted.

### 1. Changed files, functions, state and sizes

`system_option_bool` now requires `OptionReadContinuation<bool>` and returns the completed `Expr`. The new public traits are `OptionReadContinuation<V>` (`Frozen`, `finish`, `freeze`) and `FrozenOptionReadContinuation<V>` (`FrameRootTrace + Send + Sync + 'static`, immutable `finish`). Success/fallback calls `finish`; ordinary errors return unfrozen; only `InputRequired` freezes in the cold, non-inlined helper. The private generic frame decodes once, keeps immutable rooted data, and forwards errors without constructing working data. Each migrated call is a tail. Cases and DeleteCases share their ordinary/resumed post-read tail, retaining completed target/spec work; Count/MemberQ/FreeQ share the existing finish discriminator. Trace and Depth retain their pre-arity read, and explicit overrides still follow the default read.

Both certified bodies were compared byte-for-byte with the incoming snapshot: C01 `merged_option_exprs_for_symbol` and C02 `evaluate_runtime_option_expr` are unchanged. The accounting drivers, 33-member consumer protocol, `OptionReentry` domain, rum-host API, matcher and evaluator bodies are unchanged.

The following table names every changed crate file and function; repeated `finish` entries refer to the working and frozen implementations. Functions embedded in compile-control strings are fixture functions. The full source manifest, positions and hashes are in [changed-files-final.json](../../../.temp/brief-577/changed-files-final.json); the release manifests match all production source. The sole later Rust-file change is the replayed, test-only C02 census position documented below.

| File | Characters | Changed/added functions or data |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/debug_tools/trace.rs` | 26912 | `eval_trace_named` |
| `crates/rum-runtime/src/runtime/debug_tools/trace/options.rs` | 3859 | `finish`, `freeze`, `finish` (line 31), `finish_trace_option` |
| `crates/rum-runtime/src/runtime/kernel_api.rs` | 5177 | module/export or closed-list registration only |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs` | 16457 | `resume_cases_after_resolved_spec_suspended`, `resume_cases_after_resolved_spec`, `finish_cases_after_resolved_traversal` |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect/options.rs` | 3338 | `finish`, `freeze`, `finish` (line 96) |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs` | 6946 | `resume_delete_cases_after_pattern_suspended`, `resume_delete_cases_after_pattern` |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete/options.rs` | 3442 | `finish`, `freeze`, `finish` (line 95) |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs` | 33649 | `resume_replace_after_rule_exprs` |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once/options.rs` | 2999 | `finish`, `freeze`, `finish` (line 35), `finish_replace_option` |
| `crates/rum-runtime/src/runtime/options.rs` | 10230 | module/export or closed-list registration only |
| `crates/rum-runtime/src/runtime/options/core.rs` | 7478 | `system_option_bool`, `option_value_for_symbol` |
| `crates/rum-runtime/src/runtime/options/read_phase.rs` | 3246 | `finish`, `freeze`, `finish` (line 29), `decode`, `decode` (line 44), `resume`, `suspend_option_read` |
| `crates/rum-runtime/src/runtime/options/tests/read_phase.rs` | 5668 | `finish`, `finish` (line 17), `freeze`, `read_phase_success_fallback_and_ordinary_error_do_not_freeze`, `read_phase_compile_controls_pin_missing_private_and_unrooted_errors`, `consume`, `main`, `finish` (line 96), `main` (line 98), `consume` (line 103), `main` (line 106) |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` | 30342 | `suspension_protocol_repository_invariants_hold` |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | 172455 | module/export or closed-list registration only |
| `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs` | 11092 | `resume_count_after_pattern`, `resume_member_q_after_pattern`, `resume_free_q_after_pattern` |
| `crates/rum-runtime/src/runtime/tree_walkers/count_like/options.rs` | 1840 | `finish`, `freeze`, `finish` (line 46) |
| `crates/rum-runtime/src/runtime/tree_walkers/level.rs` | 4537 | `eval_level` |
| `crates/rum-runtime/src/runtime/tree_walkers/level/options.rs` | 3176 | `finish`, `freeze`, `finish` (line 27), `finish_level_option` |
| `crates/rum-runtime/src/runtime/tree_walkers/position.rs` | 10366 | `resume_position_after_pattern` |
| `crates/rum-runtime/src/runtime/tree_walkers/position/options.rs` | 2046 | `finish`, `freeze`, `finish` (line 28), `finish_position_option` |
| `crates/rum-stdlib/src/structure_ops/surface.rs` | 13961 | `eval_depth` |
| `crates/rum-stdlib/src/structure_ops/surface/options.rs` | 1887 | `finish`, `freeze`, `finish` (line 26), `finish_depth_option` |
| `crates/rum-stdlib/src/tests/options_suspension.rs` | 7594 | module/export or closed-list registration only |
| `crates/rum-stdlib/src/tests/options_suspension/read_inventory.rs` | 15370 | `position`, `tail_block`, `tail`, `test_only`, `function`, `record`, `visit_macro`, `contains_reader`, `visit_item_mod`, `visit_item_fn`, `visit_impl_item_fn`, `visit_expr`, `expected`, `discover`, `scan`, `read_inventory_is_closed_and_boolean_calls_are_tails`, `read_inventory_falsifies_new_callers_and_non_tail_use` |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv` | 20421 | inventory replay |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv` | 14111 | inventory replay |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv` | 23520 | inventory replay |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | 250549 | inventory replay |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 420119 | inventory replay |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 398682 | inventory replay |
| `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs` | 7939 | `baseline` |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 91646 | inventory replay |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv` | 111903 | inventory replay |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 18161 | inventory replay |
| `crates/rum-stdlib/tests/option_reader_continuations.rs` | 247 | module/export or closed-list registration only |
| `crates/rum-stdlib/tests/option_reader_continuations/boolean.rs` | 17670 | `cases_heads_option`, `deletecases_heads_option`, `count_heads_option`, `memberq_heads_option`, `freeq_heads_option`, `position_heads_option`, `level_heads_option`, `depth_heads_option`, `replace_heads_option`, `trace_heads_option`, `cases_heads_after_target`, `deletecases_heads_after_target`, `cases_heads_option_twice576`, `deletecases_heads_option_twice576`, `count_heads_option_twice576`, `memberq_heads_option_twice576`, `freeq_heads_option_twice576`, `position_heads_option_twice576`, `level_heads_option_twice576`, `depth_heads_option_twice576`, `replace_heads_option_twice576`, `trace_heads_option_twice576`, `count_heads_after_target576`, `memberq_heads_after_target576`, `freeq_heads_after_target576`, `position_heads_after_target576`, `replace_heads_after_target576`, `count_option_error_after_resume`, `arrayfilter_unread_default`, `testreport_unread_default`, `names_early_invalid_unread_later`, `tracedetail_heads_option`, `traceprint_heads_option`, `cases_heads_after_target_twice577`, `deletecases_heads_after_target_twice577`, `default_before_override_count`, `default_before_override_cases`, `default_before_override_deletecases`, `default_before_override_memberq`, `default_before_override_freeq`, `default_before_override_position`, `default_before_override_replace`, `default_before_override_level`, `default_before_override_depth`, `default_before_override_trace`, `depth_before_invalid_arity`, `trace_before_invalid_arity`, `trace_body_after_option` |
| `crates/rum-stdlib/tests/option_reader_continuations/lifecycle.rs` | 2106 | `cancel`, `cancel_first`, `cancel_second`, `abandon_first_without_answer`, `abandon_second_without_answer` |
| `crates/rum-stdlib/tests/option_reader_continuations/support.rs` | 4099 | `session`, `view`, `record`, `finish`, `run`, `compare` |

New retained types and every root field:

| Type | Fields | Protocol |
| --- | --- | --- |
| `OptionReadSuspensionFrame<D,F>` | `decoder: D; next: F` | FrameRootTrace + PureFrameAbandon; the only new SuspensionFrame |
| `BoolOptionDecode` | `none (boolean decoder)` | FrameRootTrace |
| `CasesOptionRead` | `target: Expr; spec: FrozenRewriteValue<CasesSpec>; call_args: SharedArgs` | FrameRootTrace |
| `DeleteCasesOptionRead` | `target: Expr; pattern: Expr; call_args: SharedArgs` | FrameRootTrace |
| `ReplaceOptionRead` | `target: Expr; rule_plan: RewriteRulePlan; levels_expr: Option<Expr>; heads_option_exprs: SharedArgs; call_args: SharedArgs` | FrameRootTrace |
| `CountLikeOptionRead` | `target: Expr; pattern: Expr; args: SharedArgs; finish: CountLikeFinish` | FrameRootTrace |
| `PositionOptionRead` | `target: Expr; pattern: Expr; args: SharedArgs` | FrameRootTrace |
| `LevelOptionRead` | `args: SharedArgs` | FrameRootTrace |
| `TraceOptionRead` | `args: SharedArgs; name: &'static str; output: TraceOutput` | FrameRootTrace |
| `DepthOptionRead` | `args: SharedArgs` | FrameRootTrace |

The eight family working descriptors use existing owned values or argument borrows. They retain no Session borrow, cancellation observer or native stack guard. The new frame is the only new named suspension frame: **559 → 560**; the lists add exactly **10 FrameRootTrace derives and 1 PureFrameAbandon derive**, with no stateful abandonment or finalizer addition.

All new production modules are below 4k characters. Warning-range parents include options, Cases collect, Count-like, Position and stdlib surface; the new source gate and boolean fixture file are coherent 15k/18k test modules. Trace (26.9k) and Replace once (33.6k) retain split-plan debt: future boundaries are trace capture/filter/rendering and remaining replacement setup/traversal. The 30.3k protocol-test parent changes only the required count; its future split is source-protocol checks versus root tests. The existing 172k declarative derive list is retained under the brief’s explicit list exception. No unrelated size cleanup was performed.

### 2. Tests, standing duty and certificate evidence

The real red run preceded every production edit. It ran 30 original step-A witnesses, three passing unread controls, two additional twice-after-target cases and four lifecycle cases. The oracle compares rum queued input with rum actual suspension: value/error/termination, ordered messages and prints, log, exact prompt sequence, live owner while suspended, no owner afterward and a subsequent `1+1`. No pre-fix result became an expected answer.

Red output, verbatim ([full log](../../../.temp/brief-577/red.log)):

```text
test result: FAILED. 8 passed; 31 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
```

Final semantic output, verbatim ([full log](../../../.temp/brief-577/boolean-final.log)):

```text
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
```

The suite grew to 52 cases, including default-before-override checks, the Trace/Depth arity boundary and a Trace body that suspends after its option. All first/second cancellation and answer-free abandonment paths run no remaining guest work.

Compile-pass/fail twins ran positive first and required exactly one intended diagnostic in each negative. The nested `Capture` type is the missing-root negative; the outer frozen descriptor still derives FrameRootTrace. Verbatim control output ([full log](../../../.temp/brief-577/compile-controls-final.log)):

```text
test runtime::options::read_phase_tests::read_phase_compile_controls_pin_missing_private_and_unrooted_errors ... missing-pass: success=true
missing-fail: success=false
error[E0061]: this method takes 4 arguments but 3 arguments were supplied
private-pass: success=true
private-fail: success=false
error[E0624]: method `option_value_for_symbol` is private
root-pass: success=true
root-fail: success=false
error[E0277]: the trait bound `Capture: FrameRootTrace` is not satisfied
```

The AST source gate has a closed inventory of 12 typed boolean reads, nine remaining step-B/C legacy reader calls and ten existing raw/internal calls. It requires typed readers to be tails, rejects `?`/later work, closures, opaque macro use and aliased method references, and checks private getter visibility. This is a repository-local syntactic guarantee, not a whole-program proof. Actual inventory and result output: [options-source-green.log](../../../.temp/brief-577/options-source-green.log).

| Mandatory gate | Exit | Raw output |
| --- | --- | --- |
| fmt-census | 0 | [fmt-census.log](../../../.temp/brief-577/fmt-census.log) |
| boolean-final | 0 | [boolean-final.log](../../../.temp/brief-577/boolean-final.log) |
| options-source-green | 0 | [options-source-green.log](../../../.temp/brief-577/options-source-green.log) |
| runtime-final | 0 | [runtime-final.log](../../../.temp/brief-577/runtime-final.log) |
| ownership-green | 0 | [ownership-green.log](../../../.temp/brief-577/ownership-green.log) |
| certificates-true-final | 0 | [certificates-true-final.log](../../../.temp/brief-577/certificates-true-final.log) |
| certificates-false-final | 0 | [certificates-false-final.log](../../../.temp/brief-577/certificates-false-final.log) |
| workspace-sandbox | 0 | [workspace-sandbox.log](../../../.temp/brief-577/workspace-sandbox.log) |

Runtime, ownership, both feature-set certificates, and the workspace output below are verbatim test-result lines. Full logs above retain every test and diagnostic.

`options-source-green`:

```text
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 1078 filtered out; finished in 4.55s
```

`runtime-final`:

```text
test result: ok. 1122 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 27.81s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.11s
```

`ownership-green`:

```text
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1092 filtered out; finished in 60.10s
```

`certificates-true-final`:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1111 filtered out; finished in 17.96s
```

`certificates-false-final`:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1105 filtered out; finished in 18.18s
```

`workspace-sandbox`:

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.06s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.35s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.08s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 1122 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 26.78s
test result: ok. 1059 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1147.45s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.29s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.55s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.13s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.93s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Certificate acceptance, hash=true, verbatim:

```text
body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
```

Certificate acceptance, hash=false, verbatim:

```text
body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
```

Each feature set earned two C01 instance marks, one C02 mark and four separate probe marks, with zero rejections, before emitted cells were installed. The expected first replay failures were an expired source anchor and stale promoted-cell comparisons, after marks had been earned; they were followed by green acceptance runs. No certified body, precondition, exclusion, LocalCovered status, catalog class or domain membership was changed. All 68 contract fingerprint cells were replayed mechanically.

The first full workspace run included failures on nine native ZeroMQ tests because loopback bind returned PermissionDenied; its complete evidence is [workspace.log](../../../.temp/brief-577/workspace.log). The existing harness requires `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` to allow this sandbox skip. The final full workspace run used that flag and retained the same code and all other test contracts.

Native transport skips are skips, not evidence that those transports passed:

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_skips_uninitialized_module_bindings ... ok
```

Standing-duty reconciliation discovered the following counts. Every full row before/after, including source-position and fingerprint changes, is disclosed in [inventory-changes.txt](../../../.temp/brief-577/inventory-changes.txt). Structured classifications and anchor mappings are in [ownership-row-changes.json](../../../.temp/brief-577/ownership-row-changes.json), [obligation-row-changes.json](../../../.temp/brief-577/obligation-row-changes.json), [fingerprint-row-changes.json](../../../.temp/brief-577/fingerprint-row-changes.json) and [certificate-cell-changes-true.json](../../../.temp/brief-577/certificate-cell-changes-true.json) / [certificate-cell-changes-false.json](../../../.temp/brief-577/certificate-cell-changes-false.json).

| Ownership inventory | Before | After | Position moves | Removed | Added |
| --- | --- | --- | --- | --- | --- |
| `host_payloads.tsv` | 63 | 64 | 1 | 0 | 1 |
| `unknown_attributes.tsv` | 1465 | 1475 | 16 | 0 | 10 |
| `unknown_captures.tsv` | 2944 | 2944 | 43 | 1 | 1 |
| `unknown_macros.tsv` | 3140 | 3138 | 99 | 9 | 7 |
| `unknown_storage.tsv` | 88 | 88 | 0 | 0 | 0 |
| `self_reaching.tsv` | 59 | 59 | 1 | 0 | 0 |
| `host_payload_ledger.tsv` | 63 | 64 | 1 | 0 | 1 |

I04 emitted exactly this new public input row; its ledger disposition is **PublicInput**:

```text
UnknownHostPayload	crates/rum-runtime/src/runtime/options/core.rs	16	15	public input rum_runtime::runtime::options::core::impl(Session)@3::system_option_bool(next): opaque owned generic parameter C : OptionReadContinuation < bool >
```

Discovery did not emit separate rows for `OptionReadContinuation::finish(value)`, `FrozenOptionReadContinuation::finish(value)` or the generic frame port. No such rows were synthesized. The existing `OptionReentry` host row retains its identity and disposition; only its source positions move. There is no other new UnknownHostPayload, no new row in untouched code and no new self-reaching state. I03 removals/additions are solely the extracted post-read work. All other existing identities are preserved. The ledger’s position changes follow the host rows and are included in the full before/after ledger above.

The initial workspace run also found the C02 census locator still at line 199. The emitted census row identifies the same method-syntax rejection at line 209, columns 58–68, inside the byte-for-byte unchanged body. Under TEMPLATE’s standing maintenance duty (positions may move, always in scope whatever the fence says), only this numeric locator in `retention_closure/census/tests.rs::baseline` changed. The exact frontier contract remains 927 rejections (RET001=10, RET002=890, RET003=27) and 16 wrapper rejections; no checker or classification was weakened. Replay evidence: [census-position-replay.json](../../../.temp/brief-577/census-position-replay.json) and [census-anchor-change.json](../../../.temp/brief-577/census-anchor-change.json). This test-only edit is excluded from release builds; all other measured source hashes still match candidate 05. All 68 fingerprints and both feature-set certificates were replayed again afterward.

Focused census output, verbatim ([census-anchor-green.log](../../../.temp/brief-577/census-anchor-green.log)):

```text
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1111 filtered out; finished in 25.34s
```

Two scratch certificate-installer attempts also stopped before editing promoted rows: the first expected a fingerprint diagnostic before the expired anchor; the second parsed a range site as a point site. Their guards were corrected to validate the exact emitted source-range translation and unchanged non-fingerprint certificate contents. Original outputs: [final-gates.log](../../../.temp/brief-577/final-gates.log) and [final-gates-resume.log](../../../.temp/brief-577/final-gates-resume.log).

Earlier failed development checks remain available: seven test-only integer-literal compile errors in `runtime-01`, a mismatched cached-rlib harness and direct frame insertion in `runtime-02`, one test-only source-scanner borrow error in `options-source-01`, and the initial expanded arity fixtures in `boolean-green`. The harness now uses one local-dependency Cargo project; insertion uses the required frame macro. The initial Depth/Trace invalid-arity probes exposed residual-call reevaluation with an effectful delayed default, exhausting the queued answer. Their defaults now self-reset on the first read to isolate ordering; no evaluator change was made. These probes also emit duplicate arity diagnostics, and `Trace[]` retains the original 2–4 wording although the one-argument shortcut is accepted. Both queued and resumed outputs agree, as recorded in `boolean-final.log`; the extracted arity tail and evaluator are unchanged. These adjacent observations remain outside this repair.

### 3. Twelve repaired sites and every measured row

| Site | Owning phase |
| --- | --- |
| P0193 | `eval_trace_named` (Trace;TraceDetail;TracePrint) |
| P0711 | `resume_cases_after_resolved_spec_suspended` (Cases) |
| P0714 | `resume_cases_after_resolved_spec` (Cases) |
| P0764 | `resume_delete_cases_after_pattern_suspended` (DeleteCases) |
| P0767 | `resume_delete_cases_after_pattern` (DeleteCases) |
| P0848 | `resume_replace_after_rule_exprs` (Replace) |
| P1385 | `resume_count_after_pattern` (Count;MemberQ;FreeQ) |
| P1389 | `resume_member_q_after_pattern` (Count;MemberQ;FreeQ) |
| P1393 | `resume_free_q_after_pattern` (Count;MemberQ;FreeQ) |
| P1401 | `eval_level` (Level) |
| P1410 | `resume_position_after_pattern` (Position) |
| P2354 | `:eval_depth` (Depth) |

This repairs the twelve designated sites: the predicted lost-continuation count is **43 → 31**. The earlier catalog is not reclassified here; steps B and C remain unchanged.

Base binaries were hash-checked against the supplied sha256.txt and never rebuilt. Every candidate used fresh default and alloc-count target directories, `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, serial bounded Cargo, and the exact two arguments `--features rum-cli/alloc-count`. All 92 result checks and all 552 metric processes passed for every candidate. Allocation counts preceded two paired instruction samples, CPU 2, `cpu_core/instructions/u`, stdin `/dev/null`, one label per process, slopes `(count(2N)-count(N))/N`. No other CPU-heavy work ran beside measurement. Workloads and expected results: [perf/labels.json](../../../.temp/brief-577/perf/labels.json).

Base hashes, verbatim ([base/sha256.txt](../../../.temp/brief-577/base/sha256.txt)):

```text
86be5519d715462ece77af4ad96300d2d73cb3ab8e8d2b0bdb019662d70cd6bf  rum-cli
4704630d518097b639f104e12784da2f49933a9895e7f37a739cc1210c071166  rum-cli-alloc
```

| Candidate | Default SHA256 | Alloc SHA256 | Regressing labels | Worst persistent % | Stagnant |
| --- | --- | --- | --- | --- | --- |
| candidate-01 | `73dcaa950944aa0693fcc0c75f4e4bd2301d1e34cbe1b444b2e623d8ab707e83` | `7291c1efef118922b91c1eadf2bc7829786bd45ebf18d40b27e9133da9bb685c` | 8 | +2.779257 | 0 |
| candidate-02 | `2ec733c9b4083d839fa6b45484694831d9f1d5369a849af9eeb68f12671f26d1` | `be02a0173d41b961c3082a0fa6ea40bd407c3a9eae5c4d128327af229c9e5f0b` | 9 | +1.454289 | 0 |
| candidate-03 | `88786429123b0f921abd9e8e54750f92747ab3af3e44ed9bd927d6779aef55f8` | `24f983e397cb8ae75b2c91230443f647edaec643b7ff7f090a5b5a059a0249a0` | 9 | +1.328395 | 0 |
| candidate-04 | `3936c9e645d1a86ef59bb01f1173107bf1d4c8da0eafd913618730f2338f3db8` | `7c49edf29b2331042f42eab9e822f5016cb75ef7751922fde9560fd562eb5a3b` | 14 | +1.580619 | 1 |
| candidate-05 | `e451f8e445198b378402c2f1ec681bf64b84a23462a60d20a26dab3a80512327` | `8bc24a8d28566d743ab794870b7f1774cef14677e771752edddb056c6f893cb6` | 6 | +0.977770 | 0 |
| candidate-06 | `7dccec608f7a1df95231fe6bf8f3e25bc2582b3d6252be5ac80b45fb5d981235` | `e60a688da1da1722dcb1bc604f82c288ad7d2898ff0d8a4503830011699fcf0f` | 10 | +1.143950 | 1 |
| candidate-07 | `374268f445630b6816284b12f02331423b5e66e0db264d8d7e2072e9555a4d81` | `3b1a61f331154aeb8b7bdce027c72aaeda0bee77bd63eacba01aee39dabdc058` | 11 | +1.492368 | 2 |

Every measured label appears below with **sample 1 / sample 2 percentage change** for every candidate. `R` means both samples exceed +0.05%; a single positive sample does not change that rule. Absolute base/candidate instruction slopes and both allocation slopes are retained in each linked TSV.

| Label / N | candidate-01 | candidate-02 | candidate-03 | candidate-04 | candidate-05 | candidate-06 | candidate-07 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `g5-cases-small-default` / 20000 | +1.53399 / +1.86560 **R** | +1.27342 / +0.09462 **R** | +0.83900 / +1.11114 **R** | +1.58062 / +0.88296 **R** | +0.62584 / +0.39733 **R** | +1.12441 / +0.74544 **R** | +1.48309 / +1.48548 **R** |
| `g5-cases-small-delayed` / 20000 | +1.45920 / +0.08765 **R** | +0.62572 / +1.33840 **R** | +0.09555 / +1.06753 **R** | +0.91786 / +0.88572 **R** | +0.97777 / +0.94320 **R** | +0.68172 / +0.79174 **R** | +1.48089 / +1.45558 **R** |
| `g5-cases-small-explicit` / 20000 | +0.79513 / +1.33002 **R** | +1.13429 / +0.75154 **R** | +0.15259 / +0.78907 **R** | +0.49353 / +0.55118 **R** | +0.92490 / -0.96300 | +0.82711 / +1.14395 **R** | +1.49237 / +1.47082 **R** |
| `g5-count-small-default` / 20000 | -0.32702 / -0.25381 | -0.56492 / -0.07271 | -0.25958 / -0.60783 | +0.66928 / +0.60878 **R** | -0.89179 / +0.16639 | -0.39193 / -0.26286 | -0.11600 / +0.45217 |
| `g5-count-small-delayed` / 20000 | -0.10973 / -0.35054 | -0.07827 / -0.28141 | -0.23293 / -1.01885 | +0.90349 / -0.24904 | +0.13911 / -0.80795 | -0.43595 / -0.55918 | +0.49121 / -0.09437 |
| `g5-count-small-explicit` / 20000 | -0.30708 / +0.55268 | +0.04290 / -0.43803 | +0.08085 / +0.01282 | +0.57731 / +0.16461 **R** | -0.85498 / +0.08172 | -0.24628 / -0.24731 | -0.69963 / -0.11003 |
| `g5-memberq-small-default` / 20000 | -0.52694 / +0.06608 | +0.05775 / -0.33862 | -1.03603 / -0.10566 | +0.48962 / +0.58572 **R** | -0.02524 / +0.60489 | -0.30576 / -0.30618 | -0.38643 / -0.24157 |
| `g5-memberq-small-delayed` / 20000 | +0.14137 / -0.03723 | +0.05201 / +0.03534 | -0.41134 / -1.06789 | -0.54450 / -0.14476 | -0.12741 / +0.23368 | -0.41115 / +0.35742 | -0.76750 / +0.16217 |
| `g5-memberq-small-explicit` / 20000 | -0.36176 / +0.11046 | -0.79182 / -0.32445 | -0.52589 / +0.18164 | -0.53027 / -0.05618 | +2.14275 / -1.13511 | -0.49456 / -0.29323 | +0.24554 / -0.93180 |
| `g5-freeq-small-default` / 20000 | -0.34229 / -0.00143 | +1.50955 / -0.33728 | +1.32839 / +0.46029 **R** | -0.45068 / -0.56243 | -0.27426 / -0.27442 | +0.52800 / +0.30535 **R** | -0.79602 / -0.64582 |
| `g5-freeq-small-delayed` / 20000 | -0.18097 / -0.02495 | -0.18947 / +0.80689 | -0.40310 / -0.71069 | -0.47929 / +0.09256 | -0.26640 / -0.65895 | -0.04923 / -0.28179 | -0.28117 / -0.38747 |
| `g5-freeq-small-explicit` / 20000 | -0.11040 / -0.11060 | +0.27887 / -0.33515 | -0.47932 / +0.11629 | -0.04292 / -0.15313 | -0.04453 / -0.26275 | +0.04477 / -0.26154 | +0.09520 / -0.18283 |
| `g5-position-small-default` / 20000 | +0.64926 / +0.37878 **R** | +0.36508 / -0.12333 | -0.12538 / +0.63302 | +1.17308 / +0.20772 **R** | +0.37072 / -0.11973 | -0.29617 / -0.53503 | +0.62281 / -0.02167 |
| `g5-position-small-delayed` / 20000 | -0.01068 / +0.32497 | +0.70944 / +0.20210 **R** | +1.09415 / -0.58580 | -0.78132 / -0.11994 | -0.20078 / +0.31994 | -0.19997 / -0.25820 | -0.26151 / +0.15524 |
| `g5-position-small-explicit` / 20000 | +0.28854 / +0.00023 | -0.40840 / -0.37258 | +0.86827 / -0.05168 | +0.46098 / -0.12401 | -0.21989 / -0.05910 | +0.34980 / +0.65882 **R** | -0.11464 / +0.04626 |
| `g5-deletecases-small-default` / 20000 | +2.77926 / +0.54319 **R** | +1.45429 / +0.53648 **R** | -0.19799 / +0.78014 | +0.72173 / +0.15168 **R** | +0.46922 / -0.21710 | -0.15322 / +0.20453 | +0.05309 / +0.47777 **R** |
| `g5-deletecases-small-delayed` / 20000 | +0.59596 / +0.02467 | +0.41742 / +0.91016 **R** | +0.09352 / +0.30348 **R** | +0.10891 / +0.30365 **R** | +0.42675 / +0.41216 **R** | +0.42568 / +0.04061 | +0.08227 / +0.38841 **R** |
| `g5-deletecases-small-explicit` / 20000 | +1.20515 / +0.57020 **R** | +0.27519 / +0.27797 **R** | +0.53493 / +0.13556 **R** | +0.15099 / +0.24468 **R** | +0.72768 / +0.19398 **R** | +0.40636 / +0.14201 **R** | +0.24327 / +0.10424 **R** |
| `g5-replace-small-default` / 20000 | +0.16484 / -0.52965 | -0.00862 / +0.02275 | -0.25294 / +0.03142 | +0.02669 / +0.02703 | +0.31392 / +0.68534 **R** | +0.02660 / +0.02570 | +0.26929 / +0.43216 **R** |
| `g5-replace-small-delayed` / 20000 | +0.04380 / +0.20443 | +0.22969 / +0.26947 **R** | -0.02523 / -0.11754 | -0.03186 / +0.04764 | +0.01939 / +0.04726 | +0.12470 / +0.36073 **R** | -0.06723 / +0.12012 |
| `g5-replace-small-explicit` / 20000 | +0.18862 / +0.26644 **R** | +0.29826 / -0.01478 | -0.16280 / +0.32973 | +0.06026 / +0.04747 | +0.03568 / -0.15233 | +0.19861 / -0.40705 | -0.15698 / +0.20206 |
| `g5-level-small-default` / 20000 | -0.63859 / -0.36733 | -0.33385 / -0.12757 | -0.45686 / +1.49227 | +0.09633 / +0.76399 **R** | -0.72378 / -0.83703 | -0.12975 / -0.12163 | +0.99005 / -0.12815 |
| `g5-level-small-delayed` / 20000 | +0.14992 / +0.15921 **R** | -0.35309 / -0.11412 | +0.05059 / -0.18833 | -0.13720 / -0.73476 | -0.20556 / -0.07182 | +0.51079 / +0.71226 **R** | -0.20440 / -0.46365 |
| `g5-level-small-explicit` / 20000 | +0.03339 / -0.07123 | -0.34742 / -0.57778 | -0.19023 / -0.15256 | +0.09260 / -0.16601 | -0.18518 / -0.24292 | -0.02840 / +0.29580 | +0.22671 / -1.03258 |
| `g5-depth-small-default` / 20000 | +0.67112 / +0.03267 | -0.40187 / -0.62437 | +0.50017 / +0.34204 **R** | +0.95169 / +0.73301 **R** | -0.10252 / -2.53790 | -0.29799 / +1.07676 | -0.07179 / +0.18154 |
| `g5-depth-small-delayed` / 20000 | -0.13148 / +1.10164 | +0.34253 / -0.94755 | +0.64247 / +0.66597 **R** | +0.48890 / +0.27380 **R** | -0.05692 / -0.23797 | +0.92265 / -0.26133 | +0.35636 / -0.10633 |
| `g5-depth-small-explicit` / 20000 | +1.66051 / -0.05881 | +0.49769 / -0.43175 | +0.86452 / +1.03925 **R** | -0.06875 / -0.23197 | -0.12415 / -0.23161 | -0.69923 / -0.39815 | -0.22229 / +0.56244 |
| `g5-trace-small-default` / 20000 | +1.22079 / -0.26457 | +0.18762 / -0.19139 | -0.15320 / -0.25938 | -0.50951 / -0.49515 | -0.26334 / +0.50386 | +0.64749 / +0.77363 **R** | +0.73951 / -0.41051 |
| `g5-trace-small-delayed` / 20000 | +0.03512 / +0.11509 | -0.11768 / -0.01004 | +0.25746 / -0.05644 | +0.46051 / -0.07648 | +0.58158 / +0.39888 **R** | -0.33622 / +0.21760 | -0.05530 / -0.16892 |
| `g5-trace-small-explicit` / 20000 | -0.00000 / +0.00222 | -0.04080 / -0.11608 | -0.36797 / +0.66552 | -0.10862 / +0.03973 | -0.07438 / +0.04063 | -0.28170 / -0.13291 | +0.59403 / +0.28213 **R** |
| `g5-tracedetail-small-default` / 20000 | -0.32143 / +0.00831 | +0.00821 / +0.25373 | -0.08692 / +0.24999 | -0.21530 / -0.16804 | -0.12153 / +0.00767 | +0.00980 / +0.02601 | -0.02079 / -0.14075 |
| `g5-tracedetail-small-delayed` / 20000 | +0.11564 / +0.02913 | +0.25871 / +0.10879 **R** | -0.33090 / -0.19793 | -0.19654 / +0.06951 | -0.26805 / -0.22205 | -0.12891 / -0.33989 | -0.83324 / -0.36473 |
| `g5-tracedetail-small-explicit` / 20000 | +0.01779 / +0.21129 | +0.18429 / -0.31780 | -0.14541 / -0.03746 | +0.18406 / +0.19448 **R** | -0.20585 / +0.38572 | +0.11396 / +0.02232 | +0.07255 / +0.32470 **R** |
| `g5-traceprint-small-default` / 2000 | -0.69550 / +0.46352 | -0.12739 / -0.23043 | -0.43763 / +0.24966 | +0.13481 / -0.05377 | -0.82079 / -0.05481 | +0.31456 / +0.24916 **R** | -0.43298 / +0.06297 |
| `g5-traceprint-small-delayed` / 2000 | -0.08640 / -0.05610 | +0.30637 / -0.29905 | +0.15192 / -0.18661 | +0.61203 / -0.06739 | -0.02630 / -0.11350 | -0.26245 / -0.38566 | -0.24002 / -0.17629 |
| `g5-traceprint-small-explicit` / 2000 | +0.26419 / -0.15424 | +0.64872 / -0.05420 | -0.22336 / -0.07418 | -0.25199 / -0.08007 | -0.25302 / -0.08638 | -1.33571 / -0.27259 | +0.54489 / -0.08512 |
| `matchq-integer-success` / 20000 | +0.00388 / -0.00190 | -0.00031 / -0.00161 | -0.01442 / -0.02277 | -0.00170 / -0.00207 | -0.00126 / +0.00113 | -0.00213 / +0.00504 | -0.01933 / -0.02186 |
| `matchq-integer-fail` / 20000 | -0.00320 / +0.00066 | +0.00078 / -0.00058 | -0.01830 / -0.01635 | +0.00108 / +0.00063 | +0.00048 / -0.00403 | +0.00542 / -0.00070 | -0.01899 / -0.01883 |
| `cases-wide-no-match` / 1000 | -0.32294 / -0.32504 | -0.33106 / -0.32625 | -0.32628 / -0.32714 | -0.32873 / -0.32598 | -0.45932 / -0.46207 | -0.32453 / -0.32683 | +1.13538 / +1.12990 **R** |
| `count-wide-no-match` / 1000 | -0.00296 / +0.00303 | +0.00060 / -0.00178 | -0.00066 / -0.00175 | -0.00773 / +0.00211 | -0.00164 / +0.00109 | -0.00252 / -0.00041 | +0.28934 / +0.29323 **R** |
| `position-wide-no-match` / 1000 | +0.00276 / -0.00025 | +0.00482 / +0.00001 | +0.00202 / +0.00385 | +0.00186 / -0.00004 | +0.00014 / +0.00252 | +0.00201 / +0.00531 | +0.00017 / +0.00283 |
| `deletecases-wide-no-match` / 1000 | +0.00293 / +0.00160 | +0.00140 / +0.00360 | +0.00262 / -0.00113 | +0.00255 / -0.00547 | +0.00283 / +0.00155 | +0.00054 / +0.00088 | +0.00308 / +0.00163 |
| `replace-wide-no-match` / 1000 | +0.00257 / +0.01089 | +0.00220 / +0.01087 | +0.00184 / +0.00164 | +0.00624 / +0.01121 | -0.00381 / +0.00059 | +0.00283 / +0.00277 | +0.00150 / +0.00224 |
| `dot-three-no-input` / 20000 | -0.00356 / -0.00370 | -0.00532 / -0.00382 | -0.00512 / -0.00490 | -0.00534 / -0.00490 | -0.00493 / -0.00451 | -0.00313 / -0.00508 | -0.00485 / -0.00355 |
| `differences-no-input` / 20000 | -0.01715 / +0.02989 | -0.02105 / -0.17983 | -0.02956 / -0.16101 | -0.09522 / -0.09742 | -0.01888 / -0.16771 | -0.01887 / +0.03039 | -0.09700 / -0.04045 |
| `take-multispec-small` / 20000 | +0.01473 / -0.03390 | -0.01772 / +0.01506 | -0.00302 / +0.01424 | -0.03432 / -0.00222 | -0.01758 / -0.00282 | +0.01334 / -0.00176 | -0.00266 / -0.00265 |

Full raw/absolute slope ledgers: [candidate-01](../../../.temp/brief-577/candidate-01/perf/slopes.tsv), [candidate-02](../../../.temp/brief-577/candidate-02/perf/slopes.tsv), [candidate-03](../../../.temp/brief-577/candidate-03/perf/slopes.tsv), [candidate-04](../../../.temp/brief-577/candidate-04/perf/slopes.tsv), [candidate-05](../../../.temp/brief-577/candidate-05/perf/slopes.tsv), [candidate-06](../../../.temp/brief-577/candidate-06/perf/slopes.tsv), [candidate-07](../../../.temp/brief-577/candidate-07/perf/slopes.tsv). Each adjacent `perf/paired-results.json`, command file, stdout, stderr and perf CSV was retained before reduction. Allocation counts and bytes are equal on every row in every candidate; this claim is checked in [report-data.json](../../../.temp/brief-577/report-data.json).

### 4. Stop-loss counters, residual attribution and open items

Candidate 01 introduced the repair; 02 inlined the private raw getter; 03 inlined the typed reader; 04 inlined the Cases working finish; 05 inlined its ordinary resolved-traversal finisher; 06 inlined `finish_cases_with_traversal`; 07 inlined `collect_cases_owned`. Each measured candidate made one production change. Candidates 06 and 07 reached the consecutive-stagnation limit; their two annotations were removed by restoring the saved candidate-05 Cases file. Every source hash then matched that already measured build. Candidate 05 is retained as the best measured source; there was no eighth candidate and no repeat sample. Any eighth candidate, if present, is described in the checkpoint. Compile and test fixes preceded measurement; failed checks were retained. The stagnant counter uses the best prior regressing-label count and worst persistent regression, not a favorable reread of a sample. The checkpoint preserves each decision: [PROGRESS.md](../../../.temp/brief-577/PROGRESS.md).

Measured candidates: **7/8**. Final repaired sites: **12**. Passing semantic fixtures: **52**. Final regressing labels: **6**; worst persistent **+0.977770%**; worst single sample over all labels **+2.142748%**; consecutive stagnant candidates at stop **2**.

Every final residual has an independent N=10/20 base/candidate callgrind profile and small-N perf calibration. Callgrind self costs sum exactly to each profile total. These profiles locate work; the mandatory paired hardware slopes decide the gate. A profile that does not reproduce a positive total delta does not erase a hardware regression. Unresolved libc entries are reported by address, not guessed as named routines. Full symbols, self deltas, edges and calibration: [candidate-05/attribution/details.json](../../../.temp/brief-577/candidate-05/attribution/details.json), [candidate-05/attribution/slopes.tsv](../../../.temp/brief-577/candidate-05/attribution/slopes.tsv), [candidate-05/attribution/calibration.tsv](../../../.temp/brief-577/candidate-05/attribution/calibration.tsv).

`g5-cases-small-default`: hardware +0.625838% / +0.397329%; callgrind slopes 16808.8 → 17227.9 Ir/iteration. Largest absolute self-cost changes (Ir/iteration):

| Function | Delta |
| --- | --- |
| `rum_runtime::runtime::kernel_data::rewrite_data::cases::collect::<impl rum_runtime::runtime::model::session::Session>::finish_cases_with_traversal` | +531.0 |
| `rum_runtime::runtime::kernel_data::rewrite_data::<impl rum_runtime::runtime::model::session::Session>::eval_rewrite_data_builtin` | -352.0 |
| `hashbrown::map::HashMap<K,V,S,A>::insert` | +206.9 |
| `rum_core::expr::intern_text` | +157.1 |
| `rum_runtime::runtime::options::core::<impl rum_runtime::runtime::model::session::Session>::system_option_bool` | -154.0 |
| `0x00000000001b1980` | +60.0 |
| `rum_core::expr::Symbol::system` | -31.8 |
| `core::ptr::drop_in_place<[rum_core::expr::Expr]>` | +13.0 |

`g5-cases-small-delayed`: hardware +0.977770% / +0.943200%; callgrind slopes 15639.1 → 15650.7 Ir/iteration. Largest absolute self-cost changes (Ir/iteration):

| Function | Delta |
| --- | --- |
| `rum_runtime::runtime::kernel_data::rewrite_data::cases::collect::<impl rum_runtime::runtime::model::session::Session>::finish_cases_with_traversal` | +531.0 |
| `rum_runtime::runtime::kernel_data::rewrite_data::<impl rum_runtime::runtime::model::session::Session>::eval_rewrite_data_builtin` | -325.0 |
| `rum_runtime::runtime::options::core::<impl rum_runtime::runtime::model::session::Session>::system_option_bool` | -179.0 |
| `hashbrown::map::HashMap<K,V,S,A>::insert` | -150.8 |
| `rum_core::expr::Symbol::system` | +110.6 |
| `rum_core::expr::intern_text` | -104.9 |
| `0x00000000001b1980` | +60.0 |
| `0x00000000001b1220` | +56.6 |

`g5-deletecases-small-delayed`: hardware +0.426746% / +0.412164%; callgrind slopes 13311.0 → 14475.2 Ir/iteration. Largest absolute self-cost changes (Ir/iteration):

| Function | Delta |
| --- | --- |
| `rum_core::expr::Symbol::system` | +726.1 |
| `rum_runtime::runtime::options::core::<impl rum_runtime::runtime::model::session::Session>::system_option_bool` | -179.0 |
| `<rum_runtime::runtime::kernel_data::rewrite_data::cases::delete::options::DeleteCasesOptionWorking as rum_runtime::runtime::options::read_phase::OptionReadContinuation<bool>>::finish` | +158.0 |
| `hashbrown::map::HashMap<K,V,S,A>::insert` | +152.9 |
| `rum_core::expr::intern_text` | +114.7 |
| `rum_core::expr::intern_symbol_text` | +88.8 |
| `rum_runtime::runtime::kernel_data::rewrite_data::<impl rum_runtime::runtime::model::session::Session>::eval_rewrite_data_builtin` | +52.0 |
| `std::sync::poison::once::Once::call_once_force::{{closure}}` | +25.3 |

`g5-deletecases-small-explicit`: hardware +0.727682% / +0.193980%; callgrind slopes 16576.1 → 16502.2 Ir/iteration. Largest absolute self-cost changes (Ir/iteration):

| Function | Delta |
| --- | --- |
| `rum_core::expr::intern_symbol_text` | -207.2 |
| `<rum_runtime::runtime::kernel_data::rewrite_data::cases::delete::options::DeleteCasesOptionWorking as rum_runtime::runtime::options::read_phase::OptionReadContinuation<bool>>::finish` | +158.0 |
| `rum_runtime::runtime::options::core::<impl rum_runtime::runtime::model::session::Session>::system_option_bool` | -154.0 |
| `hashbrown::map::HashMap<K,V,S,A>::insert` | +74.7 |
| `rum_core::expr::intern_text` | +69.1 |
| `rum_runtime::runtime::kernel_data::rewrite_data::<impl rum_runtime::runtime::model::session::Session>::eval_rewrite_data_builtin` | +25.0 |
| `0x00000000001b1220` | -24.5 |
| `std::sync::poison::once::Once::call_once_force::{{closure}}` | +18.8 |

`g5-replace-small-default`: hardware +0.313923% / +0.685339%; callgrind slopes 21148.5 → 21058.6 Ir/iteration. Largest absolute self-cost changes (Ir/iteration):

| Function | Delta |
| --- | --- |
| `rum_runtime::runtime::options::core::<impl rum_runtime::runtime::model::session::Session>::system_option_bool` | -154.0 |
| `rum_runtime::runtime::kernel_data::rewrite_data::replace::once::options::<impl rum_runtime::runtime::model::session::Session>::finish_replace_option` | +85.0 |
| `rum_core::expr::intern_symbol_text` | -83.6 |
| `rum_runtime::runtime::definitions::definition_catalog::rewrite_rules::RewriteRuleExprContinuation::resume` | +80.0 |
| `hashbrown::map::HashMap<K,V,S,A>::insert` | -38.3 |
| `rum_core::expr::intern_text` | -33.3 |
| `std::sync::poison::once::Once::call_once_force::{{closure}}` | +21.0 |
| `0x00000000001b1220` | -15.1 |

`g5-trace-small-delayed`: hardware +0.581581% / +0.398881%; callgrind slopes 27850.6 → 27848.9 Ir/iteration. Largest absolute self-cost changes (Ir/iteration):

| Function | Delta |
| --- | --- |
| `rum_runtime::runtime::options::core::<impl rum_runtime::runtime::model::session::Session>::system_option_bool` | -179.0 |
| `rum_runtime::runtime::debug_tools::trace::<impl rum_runtime::runtime::model::session::Session>::eval_trace_named` | +106.0 |
| `rum_core::expr::intern_symbol_text` | -70.3 |
| `rum_runtime::runtime::debug_tools::trace::options::<impl rum_runtime::runtime::model::session::Session>::finish_trace_option` | +57.0 |
| `hashbrown::map::HashMap<K,V,S,A>::insert` | +39.7 |
| `std::sync::poison::once::Once::call_once_force::{{closure}}` | -34.7 |
| `rum_core::expr::intern_text` | +20.0 |
| `rum_core::expr::Symbol::system` | +19.9 |

Open items: the instruction regressions above remain a failed performance gate; this change is not represented as release-ready. The adjacent invalid-arity residual-call behavior is unchanged. The source gate cannot prove an arbitrary host continuation’s semantics. No timing suite, third-party engine comparison or deep abort probe was run. Native ZeroMQ skips remain environmental coverage gaps.
