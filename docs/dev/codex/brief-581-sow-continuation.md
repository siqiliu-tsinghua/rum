# brief-581 — G7 step A: Sow resumes its tag matching before it commits

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

`Sow[value, tag]` matches its tag against every enclosing `Reap` pattern and
then records the value into each accepted bucket. A pattern test that asks for
input resumes past that work: `Reap[Do[Sow[x, x], {x, {1, 2, 3}}],
_?(…InputString…)]` resumes to `{Null, {{1}, {3}}}` where the queued path
returns `{Null, {{1}, {2}, {3}}}`. Repair P0125
(`crates/rum-runtime/src/runtime/debug_tools/reap.rs`, `eval_sow` →
`match_expr_owned`) per brief-580's design: finish every scope and pattern
match — resumed, accepted or declined — then record the value exactly once into
each accepted bucket and return the original value; nothing is committed
before all matches finish. Lost continuations **14 -> 13**; the trace filters
are step B.

brief-580's report and `.temp/brief-580/draft-A-sow.md` are the raw material;
**this brief governs** where they differ.

## Director rulings on brief-580's questions

1. The two-step split and the flat filter choice are approved (step B).
2. I01-I07 are authorized as named in `.temp/brief-580/inventory-plan.md`;
   for this step: I01 (P0125), I02's `SowMatchState` and
   `SowMatchSuspensionFrame`, I03's step-A position moves and additions, I04
   (standing replay), I05 (Sow layout and tests). **Not I06's harness edit:
   `scripts/core-language-microbench.sh` stays unchanged**; the labels are
   scratch workloads.
3. The base is supplied below.
4. The deep held-expression boundary stays with the hardening line; the
   96-under-32 case is a regression control in step B.

## Context pointers

- brief-580 and its report; `.temp/brief-580/{inventory.md,design.md,inventory-plan.md,probe.rs,cases.json,oracle.tsv,lifecycle.tsv,g7-sites.tsv,performance-plan.md,performance-labels.json}`.
- `debug_tools/reap.rs` (`:21-60`, `:62-114`, `:341-368`, `:390-411`),
  `pattern_matching/{core.rs,outcome.rs}`, `suspension/core.rs`.
- briefs 563, 568, 572-579 (the accepted protocol, the noise-floor gate).

## Scope

**Objective:** the resumable Sow matching loop, its frame, its tests, and the
standing and performance gates.

**Edit surface:** `debug_tools/reap.rs` (a child declaration and the
relocation of `eval_sow`) and new `debug_tools/reap/sow.rs` with private
`sow/tests.rs`; new `crates/rum-stdlib/tests/sow_trace_continuations.rs` with
children `support`, `sow`, `lifecycle` (the Sow subset only; the trace
fixtures wait for step B); new
`crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs`
and its `mod` line; rum-runtime's frame lists (two derive entries, one frame,
named-frame count **560 -> 561**).

**Standing duty — always in scope** (`TEMPLATE.md`): replay the fingerprints
and both certificates (C01/C02 unchanged, marks earned on their merits),
reconcile ownership discovery, disclose every row. No old construct retires in
this step; every Reap render frame keeps its identity. **STOP** on any other row
change or disappearance, any new `UnknownHostPayload` or `self_reaching` row,
any new row in untouched code, any retention reclassification.

**Fence:** no change to trace, options, the matcher, the suspension core,
`reap_state.rs` or the Reap render and handler bodies; no other group; no
leases, `unsafe`, dependencies or lockfile; no edits to `AGENTS.md`, `TODO.md`,
root `PROGRESS.md`, earlier briefs, `scripts/` or docs.

**Disclosure duty:** every changed function, state field and root, inventory
row, measured label, adjacent defect.

## Deliverable

The ordinary two-phase Sow algorithm is unchanged in shape. The frozen state
(only on `InputRequired`, on the error edge) holds the value and tag, the
scope limit and index, the pattern index and count, and the ordered accepted
pairs. On resume, a true answer adds the pending pair once and advances once;
a false answer advances without recording; a later error commits nothing of
this Sow. Completion runs the existing bucket and occurrence-charging loop
once and returns the original value; scope cleanup stays with the existing
Reap frames. Pure abandonment runs no missing match and no bucket commit. A
second suspension freezes the new state once.

## Tests

Red first, recorded before any production edit: brief-580's thirteen Sow cases
(seven red, six controls — `Sow-match`, `Sow-twice`, nested `Reap` scopes with
different tag patterns, an ordinary error after a resume, cancellation at a
suspended tag test, and the controls) through the rum-vs-rum oracle (value,
error, termination, ordered messages and prints, `log`, actual suspension,
exact prompts, no owner after, `1 + 1`). The structural contract for the new
module: the recursion scanner per file and combined, no self-reaching Sow
state, root completeness, each with a falsifier. Every earlier group's suite
stays green.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. Nothing is recorded into a bucket until every match has finished.
3. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-581/tmp …`. Bounds: focused 150 s; runtime,
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

**Performance gate.** Base: `.temp/brief-581/base/rum-cli` and
`rum-cli-alloc` — the director's copy of brief-579's selected one-unit
candidate, whose production source is the incoming tree (hashes in
`sha256.txt`); do not rebuild them. Candidates with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and `--features
rum-cli/alloc-count` (two arguments), fresh target directory each, hash
checked. Panel: `performance-plan.md`'s Sow labels (Reap and Sow with no tag,
a matching tag, a declining tag, nested scopes) as scratch workloads, plus the
controls `dot-three-no-input`, `differences-no-input`, `take-multispec-small`
(`.temp/brief-563/perf/`). Each result checked first. **Noise floor first**
(the base against itself, two pairs); a label regresses when both candidate
samples exceed `max(+0.05%, floor)`. Allocation counts, then two paired
samples of `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, one label
per process, stdin `/dev/null`, `(count(2N) - count(N)) / N`, raw output
first. No timing suite. At most eight measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-581/`; `PROGRESS.md` after the red capture, each
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

1. Changed files and functions, state fields and roots.
2. Verbatim red and green output, the structural contract and falsifiers,
   ownership and certificate results, the broad suites; every inventory row
   change.
3. P0125 repaired (14 -> 13 predicted); the noise floors and every measured
   row.
4. Stop-loss counters, residuals, open questions.

---

## Director resume note (2026-10-06) — read this first

The first run was cut by an upstream "selected model is at capacity" error
while building candidate 02's alloc-count binary; nothing in the brief
changes. Resume from `.temp/brief-581/PROGRESS.md`: the thirteen Sow cases are
green, candidate 01 is measured (two labels above their floors), candidate 02
(the one `#[inline(always)]` on `drive_sow_matches`) has its default build.
Build its alloc-count variant (`--features` and `rum-cli/alloc-count` as two
arguments), measure it, continue within the same candidate cap and stop-loss,
then run the standing duty and every verification command. The working tree
holds the run's uncommitted edits; keep them.

---

## Codex execution report — brief 581

P0125 is repaired by candidate-03: a resumed tag match applies its answer once, finishes the remaining scopes/patterns, commits the saved accepted pairs once, and returns the original value. The thirteen required cases, structural/falsifier checks, runtime, standing ownership/retention checks and workspace gate pass. The selected 12-label allocation/instruction panel FAILS the ruled per-label test for g7-sow-default-tag, g7-sow-predicate-hit. Frozen Lost **14 → 13 predicted**; this is the authorized P0125 repair, not a new whole-repository census.

### 1. Changed files, functions, state and roots

| File | Characters | Lines | Change |
| --- | ---: | ---: | --- |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs` | 5794 | 166 | new Sow contract/tests |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs` | 4808 | 152 | entry, resumable match driver, cold capture/resume and one frame |
| `crates/rum-runtime/src/runtime/debug_tools/reap.rs` | 18517 | 506 | declare sow child; relocate eval_sow only |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | 173916 | 1560 | two root-trace entries and one pure-abandon entry |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` | 30342 | 739 | named-frame assertion 560 → 561 |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs` | 11094 | 329 | new Sow contract/tests |
| `crates/rum-stdlib/src/tests/association_depth_contract/mod.rs` | 443 | 22 | wire G7 contract child |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs` | 3101 | 91 | new Sow contract/tests |
| `crates/rum-stdlib/tests/sow_trace_continuations/sow.rs` | 2856 | 91 | new Sow contract/tests |
| `crates/rum-stdlib/tests/sow_trace_continuations/support.rs` | 4110 | 132 | new Sow contract/tests |
| `crates/rum-stdlib/tests/sow_trace_continuations.rs` | 344 | 8 | new Sow contract/tests |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv` | 20751 | 66 | standing replay / source positions |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | 252638 | 1489 | standing replay / source positions |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 421204 | 2951 | standing replay / source positions |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 399306 | 3142 | standing replay / source positions |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 91646 | 69 | standing replay / source positions |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv` | 111967 | 318 | standing replay / source positions |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 18162 | 3 | standing replay / source positions |

The document change is this appended report in docs/dev/codex/brief-581-sow-continuation.md; earlier briefs and other documents were not edited. All run-owned scratch scripts, sources, logs and candidate snapshots are under .temp/brief-581/.

Every new or semantically changed function is listed below. The old `Session::eval_sow` moved from `reap.rs` into `reap/sow.rs`; the remaining Reap render/handler/accounting bodies are byte-identical after that removal and the child declaration. The existing frame-count assertion is the only changed function body outside the new Sow/contract/test files.

| File:line | Function declaration |
| --- | --- |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:7` | `fn visit_expr(&mut self, expr: &Expr) {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:10` | `fn visit_symbol(&mut self, _: &Symbol) {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:14` | `fn assert_roots(state: &impl FrameRootTrace, expected: &[Expr]) {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:19` | `fn frozen() -> SowMatchState {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:31` | `fn state_and_frame_trace_value_and_tag_through_cold_copy() {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:48` | `fn missing_tag_root_falsifier() {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:53` | `fn session_with_patterns(patterns: &[&str]) -> Session {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:71` | `fn start(session: &mut Session) -> Result<Expr, EvalError> {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:77` | `fn pending(outcome: &Result<Expr, EvalError>, prompt: &str) {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:83` | `fn assert_uncommitted(session: &Session) {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:93` | `fn successive_match_suspensions_commit_only_after_the_last_answer() {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:117` | `fn later_error_discards_the_accepted_prefix() {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:133` | `fn pure_abandon_runs_no_remaining_match_or_commit() {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow/tests.rs:158` | `fn early_commit_falsifier() {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:7` | `fn sources() -> BTreeMap<String, String> {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:11` | `fn assert_acyclic(sources: &BTreeMap<String, String>) {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:25` | `fn sow_recursion_scanner_per_file_and_combined() {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:78` | `fn direct_recursion_falsifier() {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:88` | `fn cross_file_recursion_falsifier() {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:98` | `fn compact(tokens: impl ToTokens) -> String {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:105` | `fn assert_fields(source: &str) {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:154` | `fn assert_no_self_reaching(inventory: &str) {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:163` | `fn sow_flat_fields_roots_and_ownership_are_complete() {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:174` | `fn missing_root_falsifier() {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:179` | `fn skipped_root_falsifier() {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:184` | `fn recursive_field_falsifier() {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:192` | `fn self_reaching_inventory_falsifier() {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:196` | `fn functions(source: &str) -> BTreeMap<String, String> {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:215` | `fn assert_continuation_edges(source: &str) {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:293` | `fn sow_cold_capture_cursor_and_commit_contract() {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:301` | `fn doubled_cursor_falsifier() {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:308` | `fn lost_pending_accept_falsifier() {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:316` | `fn eager_capture_falsifier() {` |
| `crates/rum-stdlib/src/tests/association_depth_contract/g7_continuation_contract.rs:324` | `fn early_bucket_commit_falsifier() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs:4` | `fn cancellation(label: &str, source: &str, stop: usize, without_answer: bool) {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs:54` | `fn sow_cancel_0_false() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs:64` | `fn sow_cancel_0_true() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs:74` | `fn sow_cancel_1_false() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/lifecycle.rs:84` | `fn sow_cancel_1_true() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/sow.rs:4` | `fn case_sow_match() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/sow.rs:14` | `fn case_sow_twice() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/sow.rs:24` | `fn case_sow_nested_patterns() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/sow.rs:34` | `fn case_sow_false_then_next_pattern() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/sow.rs:44` | `fn case_sow_same_tag_twice() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/sow.rs:54` | `fn case_sow_error_after_resume() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/sow.rs:64` | `fn case_sow_later_pattern_error() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/sow.rs:74` | `fn case_sow_direct_return() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/sow.rs:84` | `fn case_sow_ordinary() {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/support.rs:4` | `pub(super) fn session() -> Session {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/support.rs:18` | `fn view(out: Result<SessionEvalResponse, SessionRequestFailure>) -> View {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/support.rs:36` | `pub(super) fn record(file: &str, line: &str) {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/support.rs:39` | `pub(super) fn finish(` |
| `crates/rum-stdlib/tests/sow_trace_continuations/support.rs:56` | `fn run(label: &str, source: &str, inputs: &[&str], queued: bool) -> (View, String, Vec<String>) {` |
| `crates/rum-stdlib/tests/sow_trace_continuations/support.rs:98` | `pub(super) fn compare(label: &str, source: &str, inputs: &[&str], expected_prompts: &[&str]) {` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs:21` | `pub(in crate::runtime) fn eval_sow(` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs:41` | `fn drive_sow_matches(` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs:108` | `fn resume(` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs:122` | `fn suspend_sow_match(` |
| `crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs:132` | `fn resume_sow_match(` |

Existing changed assertion owner: `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs:106` — `fn suspension_protocol_repository_invariants_hold() {`; assertion at line 483.

| Frozen owner | Field | Root/meaning |
| --- | --- | --- |
| `SowMatchState` | `value: Expr,` | Expr root; original Sow return/committed value |
| `SowMatchState` | `tag: Expr,` | Expr root; original evaluated tag |
| `SowMatchState` | `scope_limit: usize,` | scalar; entry enclosing-scope count |
| `SowMatchState` | `scope_index: usize,` | scalar; scope with unanswered match |
| `SowMatchState` | `pattern_index: usize,` | scalar; unanswered pattern, advanced once on answer |
| `SowMatchState` | `pattern_count: usize,` | scalar; active scope pattern limit |
| `SowMatchState` | `matched: Vec<(usize, usize)>,` | ordered accepted (scope, pattern) index pairs; no guest roots |
| `SowMatchSuspensionFrame` | `state: SowMatchState,` | SowMatchState root graph; immutable frame-owned state |

The error arm constructs state only for InputRequired. A cold non-inlined helper installs the data-only frame; successful resume cold-copies state, decodes CompiledMatchOutcome, conditionally appends the pending pair and advances once. Error resume returns directly without cloning. Pure abandonment runs no match/commit. Reap cleanup stays in the existing Reap frames. No Session borrow, cancellation observer, NativeStackDepth, callback, lease, unsafe or new dependency is stored/introduced.

Size/split assessment: the parent shrinks from 20,002 to 18,517 characters and remains cohesive scope/render/accounting code, fenced from further splitting. New Sow implementation/private tests and G7 contract stay below their applicable 10k/15k targets. The existing 173k derived identity list and 30k frame test file receive only the mandated entries/count; restructuring that closed inventory is outside this brief. `debug_tools/` gains the reap domain child (18 direct children, below 20). No public behavior contract was redefined.

### 2. Verbatim verification and standing row reconciliation

Red capture preceded every production edit, as checkpointed by the incoming source hashes. Seven failures and six controls are preserved; no failing output became an expected answer. The oracle compares queued rum input with actual rum suspension, including value/error/termination, ordered messages/prints, log, prompts, owner cleanup and subsequent 1+1.

Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-581/tmp cargo test -p rum-stdlib --test sow_trace_continuations -- --nocapture`; exit 101. Raw: `.temp/brief-581/red.log`.

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.04s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-98b7354458f5211c)

running 13 tests
test lifecycle::sow_cancel_0_false ... lifecycle.tsv	Sow-cancel-0-false	stop=0	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::sow_cancel_0_true ... lifecycle.tsv	Sow-cancel-0-true	stop=0	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::sow_cancel_1_false ... lifecycle.tsv	Sow-cancel-1-false	stop=1	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::sow_cancel_1_true ... lifecycle.tsv	Sow-cancel-1-true	stop=1	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test sow::case_sow_direct_return ... suspensions.tsv	Sow-direct-return	1	"value: "	[]	[]
oracle.tsv	Sow-direct-return	"List[List[True, List[]], List[]]"	"List[2]"	"List[77, List[List[77]]]"	"List[2]"	["value: "]	["value: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-direct-return	"List[List[True, List[]], List[]]"	"List[2]"	"List[77, List[List[77]]]"	"List[2]"	["value: "]	["value: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'sow::case_sow_direct_return' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[List[True, List[]], List[]]", termination: "None", messages: [], prints: [] }, "List[2]")
 right: (View { status: "OK", value: "List[77, List[List[77]]]", termination: "None", messages: [], prints: [] }, "List[2]")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test sow::case_sow_error_after_resume ... suspensions.tsv	Sow-error-after-resume	1	"error: "	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }]	["before"]
oracle.tsv	Sow-error-after-resume	"<error>"	"List[1, 8]"	"<error>"	"List[1, 8]"	["error: "]	["error: "]	true	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
ROW	Sow-error-after-resume	"<error>"	"List[1, 8]"	"<error>"	"List[1, 8]"	["error: "]	["error: "]	true	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g7 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 boom\"] returned to top level." }]	["before", "after"]	["before", "after"]
ok
test sow::case_sow_false_then_next_pattern ... suspensions.tsv	Sow-false-then-next-pattern	1	"decline: "	[]	[]
oracle.tsv	Sow-false-then-next-pattern	"List[List[False, List[]], List[List[], List[]]]"	"List[1]"	"List[7, List[List[], List[List[7]]]]"	"List[1, 2]"	["decline: "]	["decline: ", "accept: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-false-then-next-pattern	"List[List[False, List[]], List[List[], List[]]]"	"List[1]"	"List[7, List[List[], List[List[7]]]]"	"List[1, 2]"	["decline: "]	["decline: ", "accept: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'sow::case_sow_false_then_next_pattern' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:126:5:
assertion `left == right` failed: exact prompts and order
  left: ["decline: "]
 right: ["decline: ", "accept: "]
FAILED
test sow::case_sow_later_pattern_error ... suspensions.tsv	Sow-later-pattern-error	1	"first: "	[]	[]
oracle.tsv	Sow-later-pattern-error	"List[List[True, List[]], List[List[], List[], List[]]]"	"List[1]"	"<error>"	"List[1, 2]"	["first: "]	["first: "]	false	"OK"	"Other { message: \"uncaught throw: \\\"g7 later\\\" with tag None\" }"	"None"	"None"	[]	[RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 later\"] returned to top level." }]	[]	[]
ROW	Sow-later-pattern-error	"List[List[True, List[]], List[List[], List[], List[]]]"	"List[1]"	"<error>"	"List[1, 2]"	["first: "]	["first: "]	false	"OK"	"Other { message: \"uncaught throw: \\\"g7 later\\\" with tag None\" }"	"None"	"None"	[]	[RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 later\"] returned to top level." }]	[]	[]

thread 'sow::case_sow_later_pattern_error' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[List[True, List[]], List[List[], List[], List[]]]", termination: "None", messages: [], prints: [] }, "List[1]")
 right: (View { status: "Other { message: \"uncaught throw: \\\"g7 later\\\" with tag None\" }", value: "<error>", termination: "None", messages: [RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g7 later\"] returned to top level." }], prints: [] }, "List[1, 2]")
FAILED
test sow::case_sow_match ... suspensions.tsv	Sow-match	1	"p: "	[]	[]
oracle.tsv	Sow-match	"List[Null, List[List[1], List[3]]]"	"List[1, 2, 3]"	"List[Null, List[List[1], List[2], List[3]]]"	"List[1, 2, 3]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-match	"List[Null, List[List[1], List[3]]]"	"List[1, 2, 3]"	"List[Null, List[List[1], List[2], List[3]]]"	"List[1, 2, 3]"	["p: "]	["p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'sow::case_sow_match' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[Null, List[List[1], List[3]]]", termination: "None", messages: [], prints: [] }, "List[1, 2, 3]")
 right: (View { status: "OK", value: "List[Null, List[List[1], List[2], List[3]]]", termination: "None", messages: [], prints: [] }, "List[1, 2, 3]")
FAILED
test sow::case_sow_nested_patterns ... suspensions.tsv	Sow-nested-patterns	1	"outer: "	[]	[]
oracle.tsv	Sow-nested-patterns	"List[List[Null, List[List[], List[List[1], List[3]]]], List[List[1], List[3]]]"	"List[List[\"outer\", 1], List[\"innerA\", 1], List[\"innerB\", 1], List[\"outer\", 2], List[\"outer\", 3], List[\"innerA\", 3], List[\"innerB\", 3]]"	"List[List[Null, List[List[List[2]], List[List[1], List[2], List[3]]]], List[List[1], List[2], List[3]]]"	"List[List[\"outer\", 1], List[\"innerA\", 1], List[\"innerB\", 1], List[\"outer\", 2], List[\"innerA\", 2], List[\"innerB\", 2], List[\"outer\", 3], List[\"innerA\", 3], List[\"innerB\", 3]]"	["outer: "]	["outer: ", "inner: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-nested-patterns	"List[List[Null, List[List[], List[List[1], List[3]]]], List[List[1], List[3]]]"	"List[List[\"outer\", 1], List[\"innerA\", 1], List[\"innerB\", 1], List[\"outer\", 2], List[\"outer\", 3], List[\"innerA\", 3], List[\"innerB\", 3]]"	"List[List[Null, List[List[List[2]], List[List[1], List[2], List[3]]]], List[List[1], List[2], List[3]]]"	"List[List[\"outer\", 1], List[\"innerA\", 1], List[\"innerB\", 1], List[\"outer\", 2], List[\"innerA\", 2], List[\"innerB\", 2], List[\"outer\", 3], List[\"innerA\", 3], List[\"innerB\", 3]]"	["outer: "]	["outer: ", "inner: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'sow::case_sow_nested_patterns' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:126:5:
assertion `left == right` failed: exact prompts and order
  left: ["outer: "]
 right: ["outer: ", "inner: "]
FAILED
test sow::case_sow_ordinary ... oracle.tsv	Sow-ordinary	"List[List[Null, List[List[List[1], List[2], List[3]], List[List[2]]]], List[List[1], List[2], List[3]]]"	"List[]"	"List[List[Null, List[List[List[1], List[2], List[3]], List[List[2]]]], List[List[1], List[2], List[3]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-ordinary	"List[List[Null, List[List[List[1], List[2], List[3]], List[List[2]]]], List[List[1], List[2], List[3]]]"	"List[]"	"List[List[Null, List[List[List[1], List[2], List[3]], List[List[2]]]], List[List[1], List[2], List[3]]]"	"List[]"	[]	[]	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test sow::case_sow_same_tag_twice ... suspensions.tsv	Sow-same-tag-twice	1	"same: "	[]	[]
oracle.tsv	Sow-same-tag-twice	"List[Null, List[List[1, 3]]]"	"List[9, 9, 9]"	"List[Null, List[List[1, 2, 3]]]"	"List[9, 9, 9]"	["same: "]	["same: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-same-tag-twice	"List[Null, List[List[1, 3]]]"	"List[9, 9, 9]"	"List[Null, List[List[1, 2, 3]]]"	"List[9, 9, 9]"	["same: "]	["same: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'sow::case_sow_same_tag_twice' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[Null, List[List[1, 3]]]", termination: "None", messages: [], prints: [] }, "List[9, 9, 9]")
 right: (View { status: "OK", value: "List[Null, List[List[1, 2, 3]]]", termination: "None", messages: [], prints: [] }, "List[9, 9, 9]")
FAILED
test sow::case_sow_twice ... suspensions.tsv	Sow-twice	1	"p: "	[]	[]
suspensions.tsv	Sow-twice	2	"p: "	[]	[]
oracle.tsv	Sow-twice	"List[Null, List[List[1], List[3], List[5]]]"	"List[1, 2, 3, 4, 5]"	"List[Null, List[List[1], List[2], List[3], List[4], List[5]]]"	"List[1, 2, 3, 4, 5]"	["p: ", "p: "]	["p: ", "p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Sow-twice	"List[Null, List[List[1], List[3], List[5]]]"	"List[1, 2, 3, 4, 5]"	"List[Null, List[List[1], List[2], List[3], List[4], List[5]]]"	"List[1, 2, 3, 4, 5]"	["p: ", "p: "]	["p: ", "p: "]	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'sow::case_sow_twice' panicked at crates/rum-stdlib/tests/sow_trace_continuations/support.rs:127:5:
assertion `left == right` failed: rum value/error/termination/messages/prints/log equality
  left: (View { status: "OK", value: "List[Null, List[List[1], List[3], List[5]]]", termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]")
 right: (View { status: "OK", value: "List[Null, List[List[1], List[2], List[3], List[4], List[5]]]", termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]")
FAILED

failures:

failures:
    sow::case_sow_direct_return
    sow::case_sow_false_then_next_pattern
    sow::case_sow_later_pattern_error
    sow::case_sow_match
    sow::case_sow_nested_patterns
    sow::case_sow_same_tag_twice
    sow::case_sow_twice

test result: FAILED. 6 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

error: test failed, to rerun pass `-p rum-stdlib --test sow_trace_continuations`
```

Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-581/tmp cargo test -p rum-stdlib --test sow_trace_continuations -- --nocapture`; exit 0. Raw: `.temp/brief-581/green-03.log`.

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.98s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-98b7354458f5211c)

running 13 tests
test lifecycle::sow_cancel_0_false ... lifecycle.tsv	Sow-cancel-0-false	stop=0	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::sow_cancel_0_true ... lifecycle.tsv	Sow-cancel-0-true	stop=0	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1]
ok
test lifecycle::sow_cancel_1_false ... lifecycle.tsv	Sow-cancel-1-false	stop=1	without_answer=false	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
ok
test lifecycle::sow_cancel_1_true ... lifecycle.tsv	Sow-cancel-1-true	stop=1	without_answer=true	View { status: "OK", value: "TerminatedEvaluation[\"Cancelled\"]", termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G7 message" }, RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }	List[1, 2]
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

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-581/tmp cargo test -p rum-stdlib --lib g7_continuation_contract -- --nocapture`; exit 0. Raw: `.temp/brief-581/structural-final.log`.

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 38.54s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 13 tests
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

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1114 filtered out; finished in 0.10s
```

Structural scope is the new Sow module, scanned per file and combined. The closed function/outbound-call set, exact scalar/pair state, both Expr roots, immutable frame and final commit boundary are checked. Observed negative controls challenge direct/cross-file recursion, missing/skipped roots, recursive state fields, self-reaching inventory entries, lost acceptance, doubled cursor, eager capture and early commit. Six private runtime tests separately exercise traced cold copies, successive accepted/declined suspension, later error, pure abandon and root/commit falsifiers. This is not a claim about Expr/matcher stack safety or deep held expressions.

Command: `python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-581/tmp cargo fmt --check`; exit 0. Raw: `.temp/brief-581/fmt-final.log`.

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-581/tmp cargo test -p rum-runtime`; exit 0. Raw: `.temp/brief-581/runtime-final.log`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 25.04s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)
test result: ok. 1130 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 30.49s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.66s
```

Private Sow runtime output:

```text
test runtime::debug_tools::reap::sow::tests::early_commit_falsifier - should panic ... ok
test runtime::debug_tools::reap::sow::tests::later_error_discards_the_accepted_prefix ... ok
test runtime::debug_tools::reap::sow::tests::missing_tag_root_falsifier - should panic ... ok
test runtime::debug_tools::reap::sow::tests::pure_abandon_runs_no_remaining_match_or_commit ... ok
test runtime::debug_tools::reap::sow::tests::state_and_frame_trace_value_and_tag_through_cold_copy ... ok
test runtime::debug_tools::reap::sow::tests::successive_match_suspensions_commit_only_after_the_last_answer ... ok
```

Ownership discovery was replayed against the final source; every old construct keeps its identity and classification. No retirement, new UnknownHostPayload, self-reaching row, untouched-code row or retention reclassification occurred. C01/C02 production source remains byte-identical. Exact before/after inventory records are in `.temp/brief-581/ownership-row-changes.json` and `.temp/brief-581/obligation-row-changes.json`.

| Inventory | Before | After | Position moves | Additions | Retirements |
| --- | ---: | ---: | ---: | ---: | ---: |
| `host_payloads.tsv` | 65 | 65 | 0 | 0 | 0 |
| `unknown_attributes.tsv` | 1487 | 1489 | 8 | 2 | 0 |
| `unknown_captures.tsv` | 2951 | 2951 | 6 | 0 | 0 |
| `unknown_macros.tsv` | 3141 | 3142 | 13 | 1 | 0 |
| `unknown_storage.tsv` | 88 | 88 | 0 | 0 | 0 |
| `self_reaching.tsv` | 59 | 59 | 0 | 0 | 0 |
| `host_payload_ledger.tsv` | unchanged | unchanged | 1 | 0 | 0 |

Every moved/added row (source position changes include end positions embedded in witnesses):

`ownership_graph/inventory/unknown_attributes.tsv`:

```text
old row 58: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	7	1	unresolved attribute/expansion derive (FrameRootTrace)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	9	1	unresolved attribute/expansion derive (FrameRootTrace)
```

```text
old row 59: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	14	1	unresolved attribute/expansion derive (FrameRootTrace)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	16	1	unresolved attribute/expansion derive (FrameRootTrace)
```

```text
old row 60: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	390	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	351	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace)
```

```text
old row 61: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	414	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	375	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
```

```text
old row 62: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	435	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	396	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
```

```text
old row 63: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	458	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	419	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
```

```text
old row 64: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	479	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	440	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
```

```text
old row 65: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	500	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	461	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
```

```text
added: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs	9	1	unresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace)
```

```text
added: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs	102	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
```

`ownership_graph/inventory/unknown_captures.tsv`:

```text
old row 163: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	146	41	closure captures: || (capture types/modes unresolved)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	107	41	closure captures: || (capture types/modes unresolved)
```

```text
old row 164: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	314	33	closure captures: || (capture types/modes unresolved)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	275	33	closure captures: || (capture types/modes unresolved)
```

```text
old row 165: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	351	64	closure captures: |bucket| (capture types/modes unresolved)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	312	64	closure captures: |bucket| (capture types/modes unresolved)
```

```text
old row 166: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	376	14	closure captures: |(position , bucket)| (capture types/modes unresolved)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	337	14	closure captures: |(position , bucket)| (capture types/modes unresolved)
```

```text
old row 167: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	399	24	closure captures: || (capture types/modes unresolved)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	360	24	closure captures: || (capture types/modes unresolved)
```

```text
old row 168: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	428	30	closure captures: || (capture types/modes unresolved)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	389	30	closure captures: || (capture types/modes unresolved)
```

`ownership_graph/inventory/unknown_macros.tsv`:

```text
old row 112: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	70	33	unexpanded macro vec! (ends at 70:81)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	31	33	unexpanded macro vec! (ends at 31:81)
```

```text
old row 113: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	77	23	unexpanded macro crate :: resume_or_suspend! (ends at 82:10)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	38	23	unexpanded macro crate :: resume_or_suspend! (ends at 43:10)
```

```text
old row 114: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	101	32	unexpanded macro crate :: resume_or_suspend! (ends at 109:18)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	62	32	unexpanded macro crate :: resume_or_suspend! (ends at 70:18)
```

```text
old row 115: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	163	21	unexpanded macro crate :: resume_or_suspend! (ends at 163:120)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	124	21	unexpanded macro crate :: resume_or_suspend! (ends at 124:120)
```

```text
old row 116: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	207	21	unexpanded macro crate :: resume_or_suspend! (ends at 207:116)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	168	21	unexpanded macro crate :: resume_or_suspend! (ends at 168:116)
```

```text
old row 117: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	237	55	unexpanded macro vec! (ends at 237:92)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	198	55	unexpanded macro vec! (ends at 198:92)
```

```text
old row 118: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	247	21	unexpanded macro crate :: resume_or_suspend! (ends at 247:116)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	208	21	unexpanded macro crate :: resume_or_suspend! (ends at 208:116)
```

```text
old row 119: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	281	55	unexpanded macro vec! (ends at 281:92)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	242	55	unexpanded macro vec! (ends at 242:92)
```

```text
old row 120: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	288	21	unexpanded macro crate :: resume_or_suspend! (ends at 288:112)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	249	21	unexpanded macro crate :: resume_or_suspend! (ends at 249:112)
```

```text
old row 121: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	331	18	unexpanded macro vec! (ends at 331:64)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	292	18	unexpanded macro vec! (ends at 292:64)
```

```text
old row 122: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	335	27	unexpanded macro vec! (ends at 335:48)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	296	27	unexpanded macro vec! (ends at 296:48)
```

```text
old row 123: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	362	17	unexpanded macro vec! (ends at 362:28)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	323	17	unexpanded macro vec! (ends at 323:28)
```

```text
old row 124: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	381	37	unexpanded macro vec! (ends at 381:59)
new: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap.rs	342	37	unexpanded macro vec! (ends at 342:59)
```

```text
added: UnknownConstruct	crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs	127	5	unexpanded macro crate :: resume_or_suspend! (ends at 127:81)
```

`ownership_graph/inventory/host_payload_ledger.tsv`:

```text
old row 55: UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	103	14	rum_runtime::runtime::suspension::core::FrameAction.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn PureFrameFinalizer	Unbounded	rum_runtime::runtime::suspension::core::PureFrameFinalizer => rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer (impl at crates/rum-runtime/src/runtime/debug_tools/reap.rs:323:1); rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer.rendered -> rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator ; rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator.prefix -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
new: UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/core.rs	103	14	rum_runtime::runtime::suspension::core::FrameAction.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn PureFrameFinalizer	Unbounded	rum_runtime::runtime::suspension::core::PureFrameFinalizer => rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer (impl at crates/rum-runtime/src/runtime/debug_tools/reap.rs:284:1); rum_runtime::runtime::debug_tools::reap::FinishReapBucketsFinalizer.rendered -> rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator ; rum_runtime::runtime::debug_tools::reap_state::ReapRenderAccumulator.prefix -> rum_core::expr::SharedArgs	TODO.md#standing-engineering-debt	unresolved
```

Retention obligation rows keep Unsplit/Pending and their existing attachment identities:

```text
row 211 before: Unsplit:Reap	Reap	Unsplit	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs	Inherited membership; subclass not re-audited	crates/rum-runtime/src/runtime/debug_tools/reap.rs:94,135,157	rum_runtime::runtime::builtin_adapters::builtin_reap
after: Unsplit:Reap	Reap	Unsplit	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs	Inherited membership; subclass not re-audited	crates/rum-runtime/src/runtime/debug_tools/reap.rs:55,96,118	rum_runtime::runtime::builtin_adapters::builtin_reap
```

```text
row 245 before: Unsplit:Sow	Sow	Unsplit	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs	Inherited membership; subclass not re-audited	crates/rum-runtime/src/runtime/debug_tools/reap.rs:32,40	rum_runtime::runtime::model::session::Session::eval_sow
after: Unsplit:Sow	Sow	Unsplit	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs	Inherited membership; subclass not re-audited	crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs:37,72,127	rum_runtime::runtime::model::session::Session::eval_sow
```

Closed frame inventory additions: FrameRootTrace `reap/sow.rs:SowMatchState` and `reap/sow.rs:SowMatchSuspensionFrame`; PureFrameAbandon `reap/sow.rs:SowMatchSuspensionFrame`; named frame count 560 → 561. Existing Reap render frame identities and abandonment policies remain unchanged.

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-581/tmp cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture`; exit 0. Raw: `.temp/brief-581/fingerprints-replay.log`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.18s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1126 filtered out; finished in 0.17s
```

All 68 fingerprint cells were copied from the executable replay; the other 18 fields of every record are unchanged. The maintenance test writes its fixed legacy review path `target/brief-544/local-contracts.tsv`; the replayed artifact is preserved under this brief. Every changed fingerprint row follows:

| local-contracts.tsv row | ID | Before | After |
| ---: | --- | --- | --- |
| 2 | B01 | `fnv1a64:6154fced8241beda` | `fnv1a64:70b3be864e1d27f8` |
| 3 | B02 | `fnv1a64:dc5a102d3f83973b` | `fnv1a64:9e288efb617c5781` |
| 4 | B03 | `fnv1a64:a818f3bab9228168` | `fnv1a64:2bbc7efdbe2492fa` |
| 5 | B04 | `fnv1a64:412d5d58b686fe02` | `fnv1a64:5423c67c834646f0` |
| 6 | B05 | `fnv1a64:a78d219e27a7651f` | `fnv1a64:2f39889449021b15` |
| 7 | B06 | `fnv1a64:9b62a703830810e6` | `fnv1a64:ac682771911f0274` |
| 8 | B07 | `fnv1a64:e1fcd2701483d1ef` | `fnv1a64:da1a9e31b6f45fc5` |
| 9 | B08 | `fnv1a64:b2d97f1685edf28c` | `fnv1a64:b866e65e5d2ebe1e` |
| 10 | B09 | `fnv1a64:409e7151ff5d82ce` | `fnv1a64:e71769fb7f5a43cc` |
| 11 | B10 | `fnv1a64:ac28b2e188b5f309` | `fnv1a64:4ae584cb878a018b` |
| 12 | B11 | `fnv1a64:02850eb5d07b2402` | `fnv1a64:157b77d99d3a6cf0` |
| 13 | B12 | `fnv1a64:ffaaf14ddb6617ce` | `fnv1a64:a623e9f75b62d8cc` |
| 14 | I01 | `fnv1a64:3f36e7eaeef9b55a` | `fnv1a64:4e95a983bad51e78` |
| 15 | I02 | `fnv1a64:fece9f321fdded1f` | `fnv1a64:867b06284138a315` |
| 16 | I03 | `fnv1a64:9fab5e77b0b14a8e` | `fnv1a64:2cd5639c2907540c` |
| 17 | I04 | `fnv1a64:e182673da3c949db` | `fnv1a64:a1369488b5dd6aa1` |
| 18 | P01 | `fnv1a64:9794e29347c03d06` | `fnv1a64:ae4e36eb37420814` |
| 19 | P02 | `fnv1a64:c3726b7221576379` | `fnv1a64:5d14933a776d609b` |
| 20 | P03 | `fnv1a64:62b898a28708d600` | `fnv1a64:1e1a72cdf9b66642` |
| 21 | P04 | `fnv1a64:d1352e42c42cb33b` | `fnv1a64:9303ad10e6257381` |
| 22 | P05 | `fnv1a64:d641edca7522c20a` | `fnv1a64:9937d43ddc1e1248` |
| 23 | P06 | `fnv1a64:502a161e6795b14e` | `fnv1a64:f6a30ec7e792724c` |
| 24 | P070 | `fnv1a64:8a77b260460b2b4e` | `fnv1a64:30f0ab09c607ec4c` |
| 25 | P080 | `fnv1a64:54dcd2ef50bdf2c9` | `fnv1a64:7818c3ee3e536ccb` |
| 26 | P090 | `fnv1a64:b70d5d02eecefffb` | `fnv1a64:3e0f22bc06a715c1` |
| 27 | P100 | `fnv1a64:b7abb0e101cbaa92` | `fnv1a64:70fc79b2c6eb1e60` |
| 28 | P110 | `fnv1a64:4c320c73a6e6e5e1` | `fnv1a64:8737a7e77b066c33` |
| 29 | P120 | `fnv1a64:5b62939119b01d2c` | `fnv1a64:c93b56bf3120f9be` |
| 30 | P130 | `fnv1a64:85ca2bcd4337abef` | `fnv1a64:7de7f78ee5a839c5` |
| 31 | P140 | `fnv1a64:35198d93e01e4693` | `fnv1a64:381c9c1d1f870fc9` |
| 32 | P150 | `fnv1a64:29dd8505f027a59b` | `fnv1a64:228b7470fe89bae1` |
| 33 | P071 | `fnv1a64:1d3cb6fe31d7810f` | `fnv1a64:96721f61c1537de5` |
| 34 | P081 | `fnv1a64:64387d9567e2af6e` | `fnv1a64:ef5b309a6f6b11ec` |
| 35 | P091 | `fnv1a64:6b904ffc20afffc6` | `fnv1a64:5ef7452263fe8c54` |
| 36 | P101 | `fnv1a64:4434fd40dc4a82e1` | `fnv1a64:7f3a98b4b06a0933` |
| 37 | P111 | `fnv1a64:a52dcc233f49451e` | `fnv1a64:53cfd9c8b4590abc` |
| 38 | P121 | `fnv1a64:d0889ccd2bfa7dc5` | `fnv1a64:151c0ec728e86c27` |
| 39 | P131 | `fnv1a64:ded77fbeecdb1f82` | `fnv1a64:f1cde8e2b99a6870` |
| 40 | P141 | `fnv1a64:603e3e5321cf12a6` | `fnv1a64:c637bad8b05bb7b4` |
| 41 | P151 | `fnv1a64:fab80e674774e16c` | `fnv1a64:a7b3d5b5369eee7e` |
| 42 | B13 | `fnv1a64:29733a8cb3a4cce6` | `fnv1a64:3a78bafac1bbbe74` |
| 43 | R01 | `fnv1a64:c9181a76a6adcfcb` | `fnv1a64:1b2ac220a12e6131` |
| 44 | R02 | `fnv1a64:93d89303e9e5fbc0` | `fnv1a64:fa8a497e1810a682` |
| 45 | R03 | `fnv1a64:e041bd372349fbf7` | `fnv1a64:e5210878fe71e53d` |
| 46 | R040 | `fnv1a64:7d2a5c84dfb3136b` | `fnv1a64:9eb7275245b0ffd1` |
| 47 | R050 | `fnv1a64:a4ce42feed63a16a` | `fnv1a64:89a5c4b53e2f2ea8` |
| 48 | R060 | `fnv1a64:d80231a6b58a9ce3` | `fnv1a64:9b5a39bd2b01a439` |
| 49 | R070 | `fnv1a64:cca645af49846e6e` | `fnv1a64:57c8f8b4510cd0ec` |
| 50 | R080 | `fnv1a64:a886d030734e12db` | `fnv1a64:683afd7b856233a1` |
| 51 | R090 | `fnv1a64:9e1dea4845f76052` | `fnv1a64:64593d65eef666a0` |
| 52 | R100 | `fnv1a64:6d3e980d62f82be8` | `fnv1a64:f0e2235067fa3d7a` |
| 53 | R110 | `fnv1a64:61e59f44065923b1` | `fnv1a64:c42f329df973fda3` |
| 54 | R041 | `fnv1a64:0fac9db6cacc8be6` | `fnv1a64:20b21e24d8e37d74` |
| 55 | R051 | `fnv1a64:452f8b187bfd0c7b` | `fnv1a64:cc3150d193d52241` |
| 56 | R061 | `fnv1a64:458ad580afce55bc` | `fnv1a64:7e4436065256d66e` |
| 57 | R071 | `fnv1a64:a4faf7d02189a3ed` | `fnv1a64:d92dc06f3939307f` |
| 58 | R081 | `fnv1a64:73961a359d777f92` | `fnv1a64:2ce6e3076296f360` |
| 59 | R091 | `fnv1a64:a7099420a54310d3` | `fnv1a64:ef653f211e3c1289` |
| 60 | R101 | `fnv1a64:eaec613dfe4870a3` | `fnv1a64:346c0a27af893179` |
| 61 | R111 | `fnv1a64:9fa106e5570b0594` | `fnv1a64:75e7dc81a3324ef6` |
| 62 | R12 | `fnv1a64:f7937fc4938cc7d4` | `fnv1a64:3e7b22dc6b7585b6` |
| 63 | R13 | `fnv1a64:8f88c659226e5c98` | `fnv1a64:a420650e35bdf68a` |
| 64 | R14 | `fnv1a64:d203bf56de7c6a92` | `fnv1a64:8b548828a39bde60` |
| 65 | R15 | `fnv1a64:215adef3dc72e616` | `fnv1a64:3300a89ef1c1b804` |
| 66 | O010 | `fnv1a64:6f20e5c52b517e1c` | `fnv1a64:59d5cd937e911b4e` |
| 67 | O0200 | `fnv1a64:2fe12f682ed56c93` | `fnv1a64:32e43df16e3e35c9` |
| 68 | O011 | `fnv1a64:1a8aee2ec31c6163` | `fnv1a64:dde2f645389368b9` |
| 69 | O0210 | `fnv1a64:1af360713696e3b4` | `fnv1a64:05e9b62028622c96` |

Certificate replay, hash feature=true: the initial run earned the same two production body rows / three instance marks and rejected stale recorded fingerprint cells. Only then were the freshly generated cells copied from `.temp/brief-546/certificates/`. No admission/status/exclusion changed.

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-581/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`; exit 101. Raw: `.temp/brief-581/certificates-true-replay.log`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.95s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1126 filtered out; finished in 18.64s
```

| promoted.tsv row | ID | Column | Source before | Source after | Changed fingerprint tokens (in occurrence order) |
| ---: | --- | ---: | --- | --- | --- |
| 2 | C01 | 5 | `crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6` | `crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6` | f0fb0fde4ee2d886 → 0803ea849d9eb978; 303baef87db65e85 → 96f190d809c6f5a4; 1af360713696e3b4 → 05e9b62028622c96; 1a8aee2ec31c6163 → dde2f645389368b9; f0fb0fde4ee2d886 → 0803ea849d9eb978; 26b8d71258b6b633 → 463a040a5d268e56; 2fe12f682ed56c93 → 32e43df16e3e35c9; 6f20e5c52b517e1c → 59d5cd937e911b4e |
| 3 | C02 | 5 | `crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6` | `crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6` | 7085a956251740d8 → adfb9da067e2474a; db2315e8dd9de539 → 187e8a84efe1df10; 3f36e7eaeef9b55a → 4e95a983bad51e78; fece9f321fdded1f → 867b06284138a315; e182673da3c949db → a1369488b5dd6aa1; c9181a76a6adcfcb → 1b2ac220a12e6131; 93d89303e9e5fbc0 → fa8a497e1810a682; 29733a8cb3a4cce6 → 3a78bafac1bbbe74; a78d219e27a7651f → 2f39889449021b15 |

Complete exact old/new cells: `.temp/brief-581/certificate-cell-changes-true.json`. Earned instance report:

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

Certificate replay, hash feature=false: the initial run earned the same two production body rows / three instance marks and rejected stale recorded fingerprint cells. Only then were the freshly generated cells copied from `.temp/brief-546/certificates/`. No admission/status/exclusion changed.

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-581/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`; exit 101. Raw: `.temp/brief-581/certificates-false-replay.log`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 42.95s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5d6fedc676d25fda)
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 18.65s
```

| promoted.tsv row | ID | Column | Source before | Source after | Changed fingerprint tokens (in occurrence order) |
| ---: | --- | ---: | --- | --- | --- |
| 2 | C01 | 6 | `crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6` | `crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6` | 3626509a41b6e9e9 → 2d850ccf5dc3fcab; 8a548981c0b37bbb → 434fe62bdfbf766a; 1af360713696e3b4 → 05e9b62028622c96; 1a8aee2ec31c6163 → dde2f645389368b9; 3626509a41b6e9e9 → 2d850ccf5dc3fcab; 846c59bae19148f1 → 18cfd631754a3fd4; 2fe12f682ed56c93 → 32e43df16e3e35c9; 6f20e5c52b517e1c → 59d5cd937e911b4e |
| 3 | C02 | 6 | `crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6` | `crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6` | 8e903f9b9ecc65a7 → 478cd341dd84a5a9; d63af416f56f18da → 7827f070bc028adf; 3f36e7eaeef9b55a → 4e95a983bad51e78; fece9f321fdded1f → 867b06284138a315; e182673da3c949db → a1369488b5dd6aa1; c9181a76a6adcfcb → 1b2ac220a12e6131; 93d89303e9e5fbc0 → fa8a497e1810a682; 29733a8cb3a4cce6 → 3a78bafac1bbbe74; a78d219e27a7651f → 2f39889449021b15 |

Complete exact old/new cells: `.temp/brief-581/certificate-cell-changes-false.json`. Earned instance report:

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

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-581/tmp cargo test -p rum-stdlib --lib ownership_graph -- --nocapture`; exit 0. Raw: `.temp/brief-581/ownership-final.log`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.41s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... HOST PAYLOAD LEDGER: 65 rows; 6 storage edges (3 Declared, 3 Unbounded); 59 PublicInput
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7670 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 61.90s
```

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-581/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture`; exit 0. Raw: `.temp/brief-581/certificates-true-final.log`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1126 filtered out; finished in 18.50s
```

Command: `python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-581/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture`; exit 0. Raw: `.temp/brief-581/certificates-false-final.log`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.62s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5d6fedc676d25fda)
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 18.78s
```

Command: `python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-581/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture`; exit 0. Raw: `.temp/brief-581/workspace-final.log`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4m 06s
     Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
     Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
     Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.93s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.36s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.66s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.34s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.45s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.75s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.77s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 48.41s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.29s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.00s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
test result: ok. 1130 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 27.72s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... HOST PAYLOAD LEDGER: 65 rows; 6 storage edges (3 Declared, 3 Unbounded); 59 PublicInput
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7670 rows
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1074 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1182.59s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.88s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
     Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
     Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s
     Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.99s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.64s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Workspace totals across 46 result groups: 3255 harness-reported passes (including the native skips described below), 0 failures, and 58 ignored tests. Command exit 0; elapsed 1556.38 seconds within the 2,400-second bound.

Native ZeroMQ: **9 loopback-unavailable skips**, reported as skips even though the Rust harness returns early with `ok`. `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` enabled the existing explicit skip path. All earlier-group suites are included in the workspace output above; ignored tests remain ignored.

### 3. P0125 and every measured row

```text
id	path	line	caller	callee	old_line	live_source	step_A_status
P0125	crates/rum-runtime/src/runtime/debug_tools/reap/sow.rs	55	drive_sow_matches	match_expr_owned	38	match self.match_expr_owned(pattern, tag.clone(), SymbolExprMap::default()) {	Repaired: SowMatchSuspensionFrame resumes pending match and completes commit
P0199	crates/rum-runtime/src/runtime/debug_tools/trace.rs	119	finish_structured_trace_capture_expr	filter_structured_trace_expr	174	self.filter_structured_trace_expr(	Unchanged: step B
P0200	crates/rum-runtime/src/runtime/debug_tools/trace.rs	298	filter_structured_trace_expr	filter_structured_trace_items	353	let filtered = self.filter_structured_trace_items(	Unchanged: step B
P0201	crates/rum-runtime/src/runtime/debug_tools/trace.rs	321	filter_structured_trace_detail_expr	filter_structured_trace_detail_items	376	let filtered = self.filter_structured_trace_detail_items(	Unchanged: step B
P0202	crates/rum-runtime/src/runtime/debug_tools/trace.rs	342	filter_structured_trace_detail_items	filter_structured_trace_detail_items	397	let nested = self.filter_structured_trace_detail_items(	Unchanged: step B
P0203	crates/rum-runtime/src/runtime/debug_tools/trace.rs	364	filter_structured_trace_detail_items	trace_detail_item_matches	419	if self.trace_detail_item_matches(&other, pattern, levels, include_heads)? {	Unchanged: step B
P0204	crates/rum-runtime/src/runtime/debug_tools/trace.rs	386	trace_detail_item_matches	trace_detail_underlying_expr_matches	441	if self.trace_detail_underlying_expr_matches(&expr, pattern, levels, include_heads)? {	Unchanged: step B
P0205	crates/rum-runtime/src/runtime/debug_tools/trace.rs	410	trace_detail_underlying_expr_matches	trace_detail_candidate_matches	465	if self.trace_detail_candidate_matches(&patterns, &candidate)? {	Unchanged: step B
P0206	crates/rum-runtime/src/runtime/debug_tools/trace.rs	425	trace_detail_underlying_expr_matches	trace_detail_candidate_matches	480	if self.trace_detail_candidate_matches(&patterns, &candidate)? {	Unchanged: step B
P0208	crates/rum-runtime/src/runtime/debug_tools/trace.rs	453	trace_detail_candidate_matches	match_expr_owned	508	let outcome = self.match_expr_owned(	Unchanged: step B
P0209	crates/rum-runtime/src/runtime/debug_tools/trace.rs	477	filter_structured_trace_items	filter_structured_trace_items	532	let nested = self.filter_structured_trace_items(	Unchanged: step B
P0210	crates/rum-runtime/src/runtime/debug_tools/trace.rs	500	filter_structured_trace_items	filter_structured_trace_leaf	555	self.filter_structured_trace_leaf(other, pattern, levels, include_heads)?	Unchanged: step B
P0211	crates/rum-runtime/src/runtime/debug_tools/trace.rs	535	filter_structured_trace_leaf	match_expr_owned	590	let outcome = self.match_expr_owned(	Unchanged: step B
P0213	crates/rum-runtime/src/runtime/debug_tools/trace.rs	568	filter_structured_trace_leaf	match_expr_owned	623	self.match_expr_owned(pattern, expr.clone(), SymbolExprMap::default())?;	Unchanged: step B
```

Supplied base binaries were never rebuilt. They are the director’s brief-579 candidate-04 one-unit production build; the recorded incoming difference is only the retention census test source, not production. Base hashes:

```text
dd3c9b9b948ea1d49c4a2317dd29836edb7c87d1e0d316bbd7db47d617218871  rum-cli
16cd57d6786be9ecad832576b31f9bea44938dfc67090cd19f408ff0498bb4c2  rum-cli-alloc
```

Method: offline, serial commands with CARGO_BUILD_JOBS=1, RUST_TEST_THREADS=1 and the brief TMPDIR. Fresh one-unit release target directories for each candidate/feature arm; alloc-count uses two arguments `--features rum-cli/alloc-count`. All label result checks precede allocation counts, then two paired CPU-2 user-instruction samples. Each N/2N process uses stdin `/dev/null` and a 150-second bound; slope is (count(2N)-count(N))/N. A label fails only when both candidate samples exceed max(0.05%, its base/base floor); allocation and byte increases fail separately. Raw output/commands precede reduction. No timing suite ran and scripts/core-language-microbench.sh is unchanged, as ruled.

Calibration corrections: repeated-tag 2N=40000 returned an iteration-work-limit error without abort; only that label changed to N=10000 before candidate measurement. Its rejected raw process and incomplete pair are retained under `noise-floor/perf/rejected-bound/` and `paired-results-before-bound-fix.json`; unaffected completed pairs were not repeated. All other labels use N=20000.

Control-source correction discovered on resume: candidate 01/02 and the initial control floors used the exact target statements/setups but an anonymous Do iterator instead of the complete named-iterator workloads supplied in brief-563. All six original sources/diffs are preserved in `perf-controls-before-563/`. The final controls are byte-identical to brief-563, calibrated at 2N, and have two fresh base/base pairs taken before candidate 03 measurement. Those three superseded candidate 01/02 control rows are disclosed below but are not final control evidence. The nine original Sow floors/samples were neither discarded nor repeated.

Original base/base rows (the final three control rows are superseded):

```text
label	N	floor	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.002114163252264767	4157.67325	4157.76115	0.002114163252264767	4157.5912	4157.62245	0.0007516371498983432	False
g7-sow-default-tag	20000	0.035243542823582	16544.7492	16544.8555	0.0006424999177578883	16544.8747	16539.0437	-0.035243542823582	False
g7-sow-tag-hit	20000	0.11838792287157673	19846.3632	19831.48195	-0.07498225166008288	19823.34805	19846.8165	0.11838792287157673	False
g7-sow-tag-decline	20000	0.016369112518543893	17144.1793	17146.98565	0.016369112518543893	17156.0992	17156.22345	0.0007242322310752058	False
g7-sow-predicate-hit	20000	0.006482428767085047	33258.7915	33259.4347	0.0019339247488758815	33264.5383	33266.69465	0.006482428767085047	False
g7-sow-predicate-decline	20000	0.04847258475916272	30624.7337	30609.8891	-0.04847258475916272	30610.10715	30600.52135	-0.03131580021273894	False
g7-sow-pattern-list	20000	0.06418482408836468	41297.6126	41271.1058	-0.06418482408836468	41299.40515	41306.4902	0.01715533183654916	False
g7-sow-nested-scopes	20000	0.06370008896708912	46371.68705	46359.04785	-0.02725628676474301	46351.7406	46322.2145	-0.06370008896708912	False
g7-sow-repeated-tag	10000	0.16681163864522564	50809.7041	50724.9476	-0.16681163864522564	50726.5877	50729.1383	0.005028132416651232	False
dot-three-no-input	20000	0.0007405235929347498	253030.42575	253032.2995	0.0007405235929347498	253029.9683	253030.25985	0.00011522350571446793	False
differences-no-input	20000	0.07222580339736329	9415.7485	9408.9479	-0.07222580339736329	9409.93035	9410.9025	0.01033110728603237	False
take-multispec-small	20000	0.016400676475514242	49199.79985	49191.73075	-0.016400676475514242	49199.5574	49200.28745	0.001483854812089902	False
```

Exact brief-563 control base/base rows:

```text
label	N	floor	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
dot-three-no-input	20000	0.0002129826324526185	254199.1306	254198.5892	-0.0002129826324526185	254199.2383	254199.5747	0.00013233713926918966	False
differences-no-input	20000	0.06969335987637715	10037.6851	10030.6895	-0.06969335987637715	10030.70275	10030.6064	-0.0009605508447529942	False
take-multispec-small	20000	0.03282260255093039	49915.64235	49922.81225	0.014364034323599562	49915.60305	49931.98665	0.03282260255093039	False
```

Final per-label floors and their evidence:

```text
label	N	floor	evidence
g7-sow-no-scope	20000	0.002114163252264767	noise-floor/perf
g7-sow-default-tag	20000	0.035243542823582	noise-floor/perf
g7-sow-tag-hit	20000	0.11838792287157673	noise-floor/perf
g7-sow-tag-decline	20000	0.016369112518543893	noise-floor/perf
g7-sow-predicate-hit	20000	0.006482428767085047	noise-floor/perf
g7-sow-predicate-decline	20000	0.04847258475916272	noise-floor/perf
g7-sow-pattern-list	20000	0.06418482408836468	noise-floor/perf
g7-sow-nested-scopes	20000	0.06370008896708912	noise-floor/perf
g7-sow-repeated-tag	10000	0.16681163864522564	noise-floor/perf
dot-three-no-input	20000	0.0002129826324526185	noise-floor-controls/perf
differences-no-input	20000	0.06969335987637715	noise-floor-controls/perf
take-multispec-small	20000	0.03282260255093039	noise-floor-controls/perf
```

candidate-01: {"candidate": "candidate-01", "labels": 12, "regressions": ["g7-sow-no-scope", "g7-sow-tag-decline"], "allocation_regressions": [], "byte_regressions": [], "worst": 1.347017539412132, "worst_residual": 1.347017539412132}. Raw panels/checks, source manifest and hashes are in `.temp/brief-581/candidate-01`.

```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.002114163252264767	2.0	2.0	168.0	168.0	4157.65525	4187.72025	0.723123929046321	4157.6593	4213.6637	1.347017539412132	True
g7-sow-default-tag	20000	0.035243542823582	21.0	21.0	1464.0	1464.0	16534.27485	16551.0595	0.10151427959357306	16546.122	16551.37495	0.031747318193353635	False
g7-sow-tag-hit	20000	0.11838792287157673	20.0	20.0	1416.0	1416.0	19848.01005	19837.17815	-0.054574236775950524	19824.2122	19836.1007	0.05996959616885178	False
g7-sow-tag-decline	20000	0.016369112518543893	13.0	13.0	848.0	848.0	17141.70655	17173.923	0.18794190593585114	17154.8096	17172.86865	0.10527106054269986	True
g7-sow-predicate-hit	20000	0.006482428767085047	35.0	35.0	2332.0	2332.0	33265.53345	33266.5696	0.00311478546273225	33263.6961	33286.96525	0.06995359123667111	False
g7-sow-predicate-decline	20000	0.04847258475916272	28.0	28.0	1764.0	1764.0	30634.8387	30619.35975	-0.05052727762526743	30602.96405	30619.40725	0.05373074311736392	False
g7-sow-pattern-list	20000	0.06418482408836468	37.0	37.0	2768.0	2768.0	41282.1191	41279.16145	-0.007164482019050844	41298.22295	41280.8335	-0.04210701758536883	False
g7-sow-nested-scopes	20000	0.06370008896708912	49.0	49.0	3552.0	3552.0	46343.35645	46309.74015	-0.07253747370730146	46347.6934	46309.8286	-0.08169726953444201	False
g7-sow-repeated-tag	10000	0.16681163864522564	48.0	48.0	3417.0	3417.0	50729.0663	50775.1929	0.09092735854276945	50731.3029	50769.2325	0.07476567293129843	False
dot-three-no-input	20000	0.0007405235929347498	596.0	596.0	23472.0	23472.0	253029.95535	253034.8436	0.001931885888062368	253028.28035	253035.19865	0.002734200299836509	False
differences-no-input	20000	0.07222580339736329	13.0	13.0	672.0	672.0	9417.9915	9398.6813	-0.20503522433631405	9410.8476	9399.4195	-0.12143539546851656	False
take-multispec-small	20000	0.016400676475514242	109.0	109.0	5824.0	5824.0	49200.49395	49190.4599	-0.020394205818730082	49183.60775	49189.8686	0.01272954605491261	False
```

```json
[
  {
    "binary": "rum-cli",
    "sha256": "547de2d072499f0d8e4833fb2120f9de8a31143858e46b83b6ecf88337e6cc52",
    "base_sha256": "dd3c9b9b948ea1d49c4a2317dd29836edb7c87d1e0d316bbd7db47d617218871",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-01/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "ad9276583c300dc86c8dfcfbb4715e9d4de5626b37f634e43c5b0ab1200a6fab",
    "base_sha256": "16cd57d6786be9ecad832576b31f9bea44938dfc67090cd19f408ff0498bb4c2",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-01/target-alloc"
  }
]
```

Diagnostic-only callgrind attribution, matching statements at N=100/200; it does not replace the hardware gate:

```text
label	base_total	candidate-01_total	base_sow_self	candidate-01_sow_self	candidate-01_continuation_self
g7-sow-no-scope	4143.88	4187.91	75.0	105.0	0
g7-sow-tag-decline	17338.52	17362.71	485.0	501.0	0
```

candidate-02: {"candidate": "candidate-02", "labels": 12, "regressions": ["g7-sow-default-tag", "g7-sow-tag-decline", "g7-sow-predicate-hit"], "allocation_regressions": [], "byte_regressions": [], "worst": 0.21529139291180943, "worst_residual": 0.1331730892000671}. Raw panels/checks, source manifest and hashes are in `.temp/brief-581/candidate-02`.

```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.002114163252264767	2.0	2.0	168.0	168.0	4157.5467	4153.5232	-0.09677582214531188	4157.5977	4166.54865	0.21529139291180943	False
g7-sow-default-tag	20000	0.035243542823582	21.0	21.0	1464.0	1464.0	16545.04555	16558.6289	0.08209919978130475	16537.3501	16559.3734	0.1331730892000671	True
g7-sow-tag-hit	20000	0.11838792287157673	20.0	20.0	1416.0	1416.0	19857.8117	19844.2112	-0.06848941970779565	19858.21035	19843.38915	-0.07463512440838427	False
g7-sow-tag-decline	20000	0.016369112518543893	13.0	13.0	848.0	848.0	17144.28955	17154.37035	0.05879975353075828	17143.25405	17154.3004	0.0644355497957605	True
g7-sow-predicate-hit	20000	0.006482428767085047	35.0	35.0	2332.0	2332.0	33261.492	33293.50525	0.09624718578469249	33270.38775	33289.42935	0.057232876704294	True
g7-sow-predicate-decline	20000	0.04847258475916272	28.0	28.0	1764.0	1764.0	30623.9758	30600.67255	-0.0760947897562092	30594.9027	30600.84845	0.01943379280628932	False
g7-sow-pattern-list	20000	0.06418482408836468	37.0	37.0	2768.0	2768.0	41290.9877	41309.27815	0.04429647005028681	41298.25525	41308.968	0.02593995783877201	False
g7-sow-nested-scopes	20000	0.06370008896708912	49.0	49.0	3552.0	3552.0	46300.1491	46360.80285	0.13100119800693033	46333.93555	46360.42295	0.057166307341649514	False
g7-sow-repeated-tag	10000	0.16681163864522564	48.0	48.0	3417.0	3417.0	50730.476	50753.5844	0.045551317121472756	50728.9966	50792.2314	0.12465217969637177	False
dot-three-no-input	20000	0.0007405235929347498	596.0	596.0	23472.0	23472.0	253031.87165	253034.54795	0.0010576928442196376	253028.02775	253034.3796	0.0025103345492771822	False
differences-no-input	20000	0.07222580339736329	13.0	13.0	672.0	672.0	9410.79245	9398.8105	-0.12732137132618337	9417.9471	9399.8326	-0.19234021817768898	False
take-multispec-small	20000	0.016400676475514242	109.0	109.0	5824.0	5824.0	49198.5807	49190.14495	-0.01714632796306903	49191.8961	49190.67445	-0.002483437510758524	False
```

```json
[
  {
    "binary": "rum-cli",
    "sha256": "2eb37f80e6c6d4d51ee58e7825e5255a174aca88c1388cba3d52886909343aa8",
    "base_sha256": "dd3c9b9b948ea1d49c4a2317dd29836edb7c87d1e0d316bbd7db47d617218871",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-02/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "69ec2d53f9831d13b904c4260aa3581a74e64b9d8e2d318d33c65725b84b7c80",
    "base_sha256": "16cd57d6786be9ecad832576b31f9bea44938dfc67090cd19f408ff0498bb4c2",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-02/target-alloc-resume"
  }
]
```

Diagnostic-only callgrind attribution, matching statements at N=100/200; it does not replace the hardware gate:

```text
label	base_total	candidate-02_total	base_sow_self	candidate-02_sow_self	candidate-02_continuation_self
g7-sow-default-tag	16871.26	16875.25	873.0	893.0	0
g7-sow-tag-decline	17343.18	17336.48	485.0	482.0	0
g7-sow-predicate-hit	33537.0	33586.54	875.0	895.0	0
```

candidate-03: {"candidate": "candidate-03", "labels": 12, "regressions": ["g7-sow-default-tag", "g7-sow-predicate-hit"], "allocation_regressions": [], "byte_regressions": [], "worst": 0.2129368738526649, "worst_residual": 0.1693364195158198}. Raw panels/checks, source manifest and hashes are in `.temp/brief-581/candidate-03`.

```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.002114163252264767	2.0	2.0	168.0	168.0	4157.6876	4166.54085	0.2129368738526649	4157.5651	4153.56635	-0.09618009348788537	False
g7-sow-default-tag	20000	0.035243542823582	21.0	21.0	1464.0	1464.0	16549.7321	16559.8574	0.0611810507796573	16541.27205	16559.9594	0.11297408049097957	True
g7-sow-tag-hit	20000	0.11838792287157673	20.0	20.0	1416.0	1416.0	19845.9092	19845.16445	-0.003752662538625273	19845.4457	19845.66445	0.0011022680130645668	False
g7-sow-tag-decline	20000	0.016369112518543893	13.0	13.0	848.0	848.0	17154.77865	17155.4341	0.003820801266929408	17152.60835	17155.1369	0.014741489739678038	False
g7-sow-predicate-hit	20000	0.006482428767085047	35.0	35.0	2332.0	2332.0	33259.60685	33299.5325	0.1200424592511462	33265.02365	33321.35345	0.1693364195158198	True
g7-sow-predicate-decline	20000	0.04847258475916272	28.0	28.0	1764.0	1764.0	30610.68885	30602.18545	-0.027779185374321447	30616.87115	30601.4385	-0.050405705809686285	False
g7-sow-pattern-list	20000	0.06418482408836468	37.0	37.0	2768.0	2768.0	41274.07625	41310.56855	0.08841457717665424	41288.9631	41310.1433	0.051297485840717805	False
g7-sow-nested-scopes	20000	0.06370008896708912	49.0	49.0	3552.0	3552.0	46335.7119	46362.3088	0.057400434587906446	46340.3186	46362.76985	0.048448631080399984	False
g7-sow-repeated-tag	10000	0.16681163864522564	48.0	48.0	3417.0	3417.0	50728.8689	50799.1717	0.13858538840789691	50718.63	50796.1579	0.15285882130491313	False
dot-three-no-input	20000	0.0002129826324526185	596.0	596.0	23504.0	23504.0	254199.4817	254198.0698	-0.0005554299287169329	254198.81615	254198.75765	-2.3013482464140367e-05	False
differences-no-input	20000	0.06969335987637715	13.0	13.0	704.0	704.0	10051.95925	10035.22665	-0.16646108070921262	10037.75155	10035.28645	-0.024558288653808713	False
take-multispec-small	20000	0.03282260255093039	109.0	109.0	5856.0	5856.0	49915.3825	49922.42225	0.014103367834561453	49915.3752	49922.22415	0.013721122945709041	False
```

```json
[
  {
    "binary": "rum-cli",
    "sha256": "c49b3a1d0a695e7cf5d9fe620749f914d3994f72322d1f3e60473c8c8bf522f9",
    "base_sha256": "dd3c9b9b948ea1d49c4a2317dd29836edb7c87d1e0d316bbd7db47d617218871",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-03/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "01b3a5b1a20928cdd3eaf2e0332dce88155305651698234b2e0cadb4a06f3251",
    "base_sha256": "16cd57d6786be9ecad832576b31f9bea44938dfc67090cd19f408ff0498bb4c2",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-03/target-alloc"
  }
]
```

Diagnostic-only callgrind attribution, matching statements at N=100/200; it does not replace the hardware gate:

```text
label	base_total	candidate-03_total	base_sow_self	candidate-03_sow_self	candidate-03_continuation_self
g7-sow-default-tag	16856.96	16882.5	873.0	894.0	0
g7-sow-predicate-hit	33590.99	33562.46	875.0	896.0	0
```

candidate-04: {"candidate": "candidate-04", "labels": 12, "regressions": ["g7-sow-no-scope", "g7-sow-default-tag", "g7-sow-tag-decline"], "allocation_regressions": [], "byte_regressions": [], "worst": 0.6297094295058425, "worst_residual": 0.6297094295058425}. Raw panels/checks, source manifest and hashes are in `.temp/brief-581/candidate-04`.

```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.002114163252264767	2.0	2.0	168.0	168.0	4157.8047	4183.5274	0.6186606119330307	4157.5763	4183.75695	0.6297094295058425	True
g7-sow-default-tag	20000	0.035243542823582	21.0	21.0	1464.0	1464.0	16546.1048	16560.32915	0.08596796751825231	16541.2665	16561.11455	0.11999111434422005	True
g7-sow-tag-hit	20000	0.11838792287157673	20.0	20.0	1416.0	1416.0	19837.1945	19845.86195	0.04369292240391509	19846.23975	19846.7784	0.002714116158952784	False
g7-sow-tag-decline	20000	0.016369112518543893	13.0	13.0	848.0	848.0	17141.75825	17163.53035	0.12701205840424112	17153.82765	17164.64795	0.06307805010503653	True
g7-sow-predicate-hit	20000	0.006482428767085047	35.0	35.0	2332.0	2332.0	33321.973	33276.6685	-0.13595983647186616	33325.2534	33290.3311	-0.10479230144427865	False
g7-sow-predicate-decline	20000	0.04847258475916272	28.0	28.0	1764.0	1764.0	30635.36505	30588.72455	-0.15224398313478105	30624.6496	30634.3708	0.03174305706994396	False
g7-sow-pattern-list	20000	0.06418482408836468	37.0	37.0	2768.0	2768.0	41306.0086	41301.64115	-0.010573401178248787	41272.722	41303.02015	0.07340962391575445	False
g7-sow-nested-scopes	20000	0.06370008896708912	49.0	49.0	3552.0	3552.0	46358.83895	46351.5275	-0.0157714260443087	46335.5316	46351.8659	0.03525221236480025	False
g7-sow-repeated-tag	10000	0.16681163864522564	48.0	48.0	3417.0	3417.0	50723.0796	50801.8601	0.1553148992948783	50723.0526	50803.2246	0.1580583105520672	False
dot-three-no-input	20000	0.0002129826324526185	596.0	596.0	23504.0	23504.0	254200.0834	254199.1248	-0.0003771045183009747	254198.62465	254199.03905	0.00016302212513252812	False
differences-no-input	20000	0.06969335987637715	13.0	13.0	704.0	704.0	10052.0151	10035.1759	-0.16752063971731124	10052.03325	10035.7211	-0.1622771194076611	False
take-multispec-small	20000	0.03282260255093039	109.0	109.0	5856.0	5856.0	49923.007	49922.5347	-0.0009460567950148047	49939.66625	49922.1196	-0.03513569736762756	False
```

```json
[
  {
    "binary": "rum-cli",
    "sha256": "a799477ef1f1df6bb0213cf52af102f6388971327a83466652bf1ff19c63eece",
    "base_sha256": "dd3c9b9b948ea1d49c4a2317dd29836edb7c87d1e0d316bbd7db47d617218871",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-04/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "bfefaac205045acd2a9848937a41dc69803ae468d242b5868e6dfb8fc57ada62",
    "base_sha256": "16cd57d6786be9ecad832576b31f9bea44938dfc67090cd19f408ff0498bb4c2",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-04/target-alloc"
  }
]
```

Diagnostic-only callgrind attribution, matching statements at N=100/200; it does not replace the hardware gate:

```text
label	base_total	candidate-04_total	base_sow_self	candidate-04_sow_self	candidate-04_continuation_self
g7-sow-no-scope	4161.03	4177.64	75.0	101.0	0
g7-sow-default-tag	16825.31	16882.55	873.0	895.0	0
g7-sow-tag-decline	17316.87	17347.3	485.0	491.0	0
```

candidate-05: {"candidate": "candidate-05", "labels": 12, "regressions": ["g7-sow-default-tag", "g7-sow-tag-decline", "g7-sow-predicate-hit", "g7-sow-repeated-tag"], "allocation_regressions": [], "byte_regressions": [], "worst": 0.5075773842770692, "worst_residual": 0.2644866633192855}. Raw panels/checks, source manifest and hashes are in `.temp/brief-581/candidate-05`.

```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.002114163252264767	2.0	2.0	168.0	168.0	4157.73775	4139.61695	-0.43583316432116437	4157.65175	4178.75505	0.5075773842770692	False
g7-sow-default-tag	20000	0.035243542823582	21.0	21.0	1464.0	1464.0	16549.53515	16574.8043	0.15268797444139004	16549.4074	16575.18635	0.15576962592629418	True
g7-sow-tag-hit	20000	0.11838792287157673	20.0	20.0	1416.0	1416.0	19845.49735	19859.6279	0.0712028010726451	19858.1444	19861.08055	0.014785621158019957	False
g7-sow-tag-decline	20000	0.016369112518543893	13.0	13.0	848.0	848.0	17152.91765	17172.21305	0.11249048350674418	17156.28325	17173.2401	0.09883754979389536	True
g7-sow-predicate-hit	20000	0.006482428767085047	35.0	35.0	2332.0	2332.0	33264.657	33306.07575	0.12451278244054453	33267.8292	33313.8965	0.13847401861737385	True
g7-sow-predicate-decline	20000	0.04847258475916272	28.0	28.0	1764.0	1764.0	30635.83325	30619.58685	-0.05303071036920626	30613.37015	30619.38575	0.0196502376919927	False
g7-sow-pattern-list	20000	0.06418482408836468	37.0	37.0	2768.0	2768.0	41283.26635	41326.3958	0.10447199025955811	41298.87655	41321.8008	0.05550816853878704	False
g7-sow-nested-scopes	20000	0.06370008896708912	49.0	49.0	3552.0	3552.0	46347.29115	46371.75125	0.05277568417287881	46303.47415	46371.29075	0.14646114842333802	False
g7-sow-repeated-tag	10000	0.16681163864522564	48.0	48.0	3417.0	3417.0	50725.1106	50821.4208	0.1898669098219674	50694.9191	50829.0004	0.2644866633192855	True
dot-three-no-input	20000	0.0002129826324526185	596.0	596.0	23504.0	23504.0	254199.8236	254198.61635	-0.0004749216513610932	254198.8928	254198.6798	-8.379265450386697e-05	False
differences-no-input	20000	0.06969335987637715	13.0	13.0	704.0	704.0	10044.9749	10035.44735	-0.09484891794003802	10037.67635	10033.4858	-0.041748207990377484	False
take-multispec-small	20000	0.03282260255093039	109.0	109.0	5856.0	5856.0	49915.25525	49922.44055	0.014394998010147653	49915.55315	49921.2268	0.011366497297848888	False
```

```json
[
  {
    "binary": "rum-cli",
    "sha256": "7ef1ed570ad03b579cb167f87701c179d3df9e02fd17015735f36db1d70f9708",
    "base_sha256": "dd3c9b9b948ea1d49c4a2317dd29836edb7c87d1e0d316bbd7db47d617218871",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-05/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "324f3858c9520748393bd0c467740972e781ffc23d3acb5a0968084b69ace15d",
    "base_sha256": "16cd57d6786be9ecad832576b31f9bea44938dfc67090cd19f408ff0498bb4c2",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-05/target-alloc"
  }
]
```

Diagnostic-only callgrind attribution, matching statements at N=100/200; it does not replace the hardware gate:

```text
label	base_total	candidate-05_total	base_sow_self	candidate-05_sow_self	candidate-05_continuation_self
g7-sow-default-tag	16868.75	16877.54	873.0	898.0	0
g7-sow-tag-decline	17296.11	17353.11	485.0	490.0	0
g7-sow-predicate-hit	33672.56	33608.84	875.0	900.0	0
g7-sow-repeated-tag	51223.47	51363.2	1513.0	1589.0	0
```

candidate-06: {"candidate": "candidate-06", "labels": 12, "regressions": ["g7-sow-default-tag", "g7-sow-predicate-hit"], "allocation_regressions": [], "byte_regressions": [], "worst": 0.23088188821460687, "worst_residual": 0.1772753224664303}. Raw panels/checks, source manifest and hashes are in `.temp/brief-581/candidate-06`.

```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.002114163252264767	2.0	2.0	168.0	168.0	4157.55635	4165.5558	0.19240749436866267	4157.7394	4139.53575	-0.43782566074247464	False
g7-sow-default-tag	20000	0.035243542823582	21.0	21.0	1464.0	1464.0	16546.40905	16575.74175	0.1772753224664303	16549.20775	16575.8417	0.16093791559297355	True
g7-sow-tag-hit	20000	0.11838792287157673	20.0	20.0	1416.0	1416.0	19846.19605	19860.82975	0.07373554087208856	19836.80825	19860.22025	0.1180230191517806	False
g7-sow-tag-decline	20000	0.016369112518543893	13.0	13.0	848.0	848.0	17152.2279	17157.15795	0.02874291333314627	17154.92345	17157.45855	0.014777681797228936	False
g7-sow-predicate-hit	20000	0.006482428767085047	35.0	35.0	2332.0	2332.0	33268.2277	33306.9737	0.11646547675876473	33269.89025	33307.3636	0.11263442625875086	True
g7-sow-predicate-decline	20000	0.04847258475916272	28.0	28.0	1764.0	1764.0	30612.8675	30603.60945	-0.03024234825437233	30606.70245	30604.6301	-0.006770902560926384	False
g7-sow-pattern-list	20000	0.06418482408836468	37.0	37.0	2768.0	2768.0	41306.4349	41295.1283	-0.027372490575316988	41299.6124	41297.71945	-0.004583457059270657	False
g7-sow-nested-scopes	20000	0.06370008896708912	49.0	49.0	3552.0	3552.0	46338.88355	46321.9744	-0.03649019722660185	46352.64095	46321.87735	-0.06636860245607945	False
g7-sow-repeated-tag	10000	0.16681163864522564	48.0	48.0	3417.0	3417.0	50725.59	50842.7062	0.23088188821460687	50845.2755	50890.8789	0.08969053575096364	False
dot-three-no-input	20000	0.0002129826324526185	596.0	596.0	23504.0	23504.0	254199.6145	254197.8341	-0.0007003944531902562	254199.26815	254198.72795	-0.00021251044659553742	False
differences-no-input	20000	0.06969335987637715	13.0	13.0	704.0	704.0	10037.7204	10035.22045	-0.024905555249377098	10045.04095	10033.70265	-0.11287460206920175	False
take-multispec-small	20000	0.03282260255093039	109.0	109.0	5856.0	5856.0	49939.1168	49922.3991	-0.03347616271820453	49915.41765	49921.9552	0.013097255933702812	False
```

```json
[
  {
    "binary": "rum-cli",
    "sha256": "95cd11350b2b38a039f7433f335c3b3ce23edc7cd598e115e4f7b7f0e76527a5",
    "base_sha256": "dd3c9b9b948ea1d49c4a2317dd29836edb7c87d1e0d316bbd7db47d617218871",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-06/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "8a04e1db8761d9cc9ac8c36bfa717ba5e6c0b4045a032ba12f2477f85d54c61e",
    "base_sha256": "16cd57d6786be9ecad832576b31f9bea44938dfc67090cd19f408ff0498bb4c2",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-06/target-alloc"
  }
]
```

Diagnostic-only callgrind attribution, matching statements at N=100/200; it does not replace the hardware gate:

```text
label	base_total	candidate-06_total	base_sow_self	candidate-06_sow_self	candidate-06_continuation_self
g7-sow-default-tag	16841.15	16924.17	873.0	910.0	0
g7-sow-predicate-hit	33551.68	33609.12	875.0	912.0	0
```

candidate-07: {"candidate": "candidate-07", "labels": 12, "regressions": ["g7-sow-default-tag", "g7-sow-predicate-hit", "g7-sow-repeated-tag"], "allocation_regressions": [], "byte_regressions": [], "worst": 0.5058594097860691, "worst_residual": 0.23668240810770236}. Raw panels/checks, source manifest and hashes are in `.temp/brief-581/candidate-07`.

```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.002114163252264767	2.0	2.0	168.0	168.0	4157.66705	4178.699	0.5058594097860691	4157.53495	4152.46965	-0.12183421332393474	False
g7-sow-default-tag	20000	0.035243542823582	21.0	21.0	1464.0	1464.0	16538.80375	16575.5577	0.2222285877236052	16544.6184	16576.03485	0.1898892391498208	True
g7-sow-tag-hit	20000	0.11838792287157673	20.0	20.0	1416.0	1416.0	19858.25175	19861.5771	0.016745431782538844	19855.67	19860.2112	0.022871048924577586	False
g7-sow-tag-decline	20000	0.016369112518543893	13.0	13.0	848.0	848.0	17152.5583	17157.83775	0.030779373593481374	17149.288	17157.88865	0.050151644779661275	False
g7-sow-predicate-hit	20000	0.006482428767085047	35.0	35.0	2332.0	2332.0	33264.7103	33313.9267	0.14795379113825557	33271.2159	33329.2422	0.17440390568954633	True
g7-sow-predicate-decline	20000	0.04847258475916272	28.0	28.0	1764.0	1764.0	30687.26445	30627.56155	-0.19455269496985172	30631.74925	30627.5028	-0.013862904025963019	False
g7-sow-pattern-list	20000	0.06418482408836468	37.0	37.0	2768.0	2768.0	41281.5221	41294.68945	0.03189647408856633	41295.86685	41305.9787	0.024486348807561953	False
g7-sow-nested-scopes	20000	0.06370008896708912	49.0	49.0	3552.0	3552.0	46344.888	46320.92275	-0.051710665478366113	46314.2946	46321.599	0.015771372668171857	False
g7-sow-repeated-tag	10000	0.16681163864522564	48.0	48.0	3417.0	3417.0	50722.8657	50842.9178	0.23668240810770236	50727.3704	50839.3263	0.2207011700334549	True
dot-three-no-input	20000	0.0002129826324526185	596.0	596.0	23504.0	23504.0	254199.3755	254198.56835	-0.0003175263504906489	254199.17615	254199.2472	2.7950523318054366e-05	False
differences-no-input	20000	0.06969335987637715	13.0	13.0	704.0	704.0	10037.82115	10034.46285	-0.03345646380639167	10030.8151	10035.2045	0.043759155724054644	False
take-multispec-small	20000	0.03282260255093039	109.0	109.0	5856.0	5856.0	49939.0088	49922.15125	-0.03375627671649006	49931.3796	49921.97815	-0.018828740714382786	False
```

```json
[
  {
    "binary": "rum-cli",
    "sha256": "1079dec6a63302e02d422568a1a72666f92cf9907c685e89ab3f077db95abcf1",
    "base_sha256": "dd3c9b9b948ea1d49c4a2317dd29836edb7c87d1e0d316bbd7db47d617218871",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-07/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "bead7d11cb7326454ee89d8dc0809acae13a3b8bcc0740c915be2c1d857d5a00",
    "base_sha256": "16cd57d6786be9ecad832576b31f9bea44938dfc67090cd19f408ff0498bb4c2",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-07/target-alloc"
  }
]
```

Diagnostic-only callgrind attribution, matching statements at N=100/200; it does not replace the hardware gate:

```text
label	base_total	candidate-07_total	base_sow_self	candidate-07_sow_self	candidate-07_continuation_self
g7-sow-default-tag	16844.29	16909.33	873.0	910.0	0
g7-sow-predicate-hit	33622.11	33620.42	875.0	912.0	0
g7-sow-repeated-tag	51239.54	51410.13	1513.0	1621.0	0
```

candidate-08: {"candidate": "candidate-08", "labels": 12, "regressions": ["g7-sow-default-tag", "g7-sow-repeated-tag"], "allocation_regressions": [], "byte_regressions": [], "worst": 0.2810840376074042, "worst_residual": 0.2810840376074042}. Raw panels/checks, source manifest and hashes are in `.temp/brief-581/candidate-08`.

```text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
g7-sow-no-scope	20000	0.002114163252264767	2.0	2.0	168.0	168.0	4157.81425	4139.62625	-0.4374413792054388	4157.53735	4152.636	-0.1178907027738263	False
g7-sow-default-tag	20000	0.035243542823582	21.0	21.0	1464.0	1464.0	16534.33585	16576.9986	0.25802518097512994	16547.60875	16576.9531	0.1773328729445467	True
g7-sow-tag-hit	20000	0.11838792287157673	20.0	20.0	1416.0	1416.0	19846.3044	19861.3971	0.0760479114690904	19857.9621	19861.78005	0.019226293114948945	False
g7-sow-tag-decline	20000	0.016369112518543893	13.0	13.0	848.0	848.0	17145.53675	17158.39535	0.07499677722249132	17158.5447	17157.2602	-0.007486066111406231	False
g7-sow-predicate-hit	20000	0.006482428767085047	35.0	35.0	2332.0	2332.0	33317.11025	33311.9815	-0.015393742018776635	33263.6424	33307.6088	0.13217554310891444	False
g7-sow-predicate-decline	20000	0.04847258475916272	28.0	28.0	1764.0	1764.0	30607.5461	30603.75935	-0.012371949020761885	30606.2865	30575.541	-0.1004548526329585	False
g7-sow-pattern-list	20000	0.06418482408836468	37.0	37.0	2768.0	2768.0	41282.4975	41303.1278	0.04997347846991129	41298.3337	41296.6074	-0.004180071797910134	False
g7-sow-nested-scopes	20000	0.06370008896708912	49.0	49.0	3552.0	3552.0	46359.0299	46322.3231	-0.0791793962884424	46352.38215	46323.0575	-0.0632646018172256	False
g7-sow-repeated-tag	10000	0.16681163864522564	48.0	48.0	3417.0	3417.0	50703.7686	50846.2888	0.2810840376074042	50726.1561	50845.26	0.23479780286368435	True
dot-three-no-input	20000	0.0002129826324526185	596.0	596.0	23504.0	23504.0	254199.26965	254198.03175	-0.0004869801560447584	254199.6526	254199.3894	-0.00010354066078965829	False
differences-no-input	20000	0.06969335987637715	13.0	13.0	704.0	704.0	10030.4331	10035.1219	0.04674573822740502	10030.736	10035.71025	0.04959007993032838	False
take-multispec-small	20000	0.03282260255093039	109.0	109.0	5856.0	5856.0	49923.08375	49921.8972	-0.002376756223509968	49939.06595	49921.5498	-0.03507504529126626	False
```

```json
[
  {
    "binary": "rum-cli",
    "sha256": "ddd20fa1bae92eae609ddb4c2c7aea82fbde411798292add0fae1298446e5a38",
    "base_sha256": "dd3c9b9b948ea1d49c4a2317dd29836edb7c87d1e0d316bbd7db47d617218871",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-08/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "6974a86ba1d9a75d6bf50ec952f06a51fbe179c5dbfac2acb34611f60fcbfd18",
    "base_sha256": "16cd57d6786be9ecad832576b31f9bea44938dfc67090cd19f408ff0498bb4c2",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-581/candidate-08/target-alloc"
  }
]
```

Candidate changes: 01 introduced the resumable loop/state/frame. 02 added only `#[inline(always)]` to drive_sow_matches, repairing the new call-boundary cost but leaving three measured residuals. 03 outlined InputRequired frame installation into a cold non-inlined helper; the structural edge/closed-call assertions followed that relocation. 04 extracted the same final commit loop into an inline-always helper so bucket work can be optimized separately; the structural contract requires the helper to follow completed matching. 05 added the zero-scope immediate return after the existing arity/value/tag selection. 06 outlined one nonempty commit helper (inline-never), retaining an empty-commit return in the driver and strengthening the tail-branch contract accordingly. 07 changed only that helper annotation to #[inline], permitting heuristic inlining of its optimized body. 08 changed the infallible commit helper return from Result to Expr, with Ok constructed in the caller. No semantic expectation, root, cursor, scope-cleanup or occurrence-accounting rule changed while tuning.

### 4. Stop-loss, disclosed failures, residuals and boundaries

```json
{
  "compiling_production_candidates": 8,
  "measured_candidates": 8,
  "measured_limit": 8,
  "repaired_lost_sites": 1,
  "required_semantic_cases_passing": 13,
  "required_semantic_cases": 13,
  "consecutive_no_progress": 0,
  "focused_executions": 16,
  "focused_limit": 8,
  "broad_executions": 1,
  "broad_limit": 2,
  "regressing_labels": [
    "g7-sow-default-tag",
    "g7-sow-repeated-tag"
  ],
  "worst_regression_pct": 0.2810840376074042,
  "disclosed_gate_corrections": [
    "early-commit synthetic mutation initially hit cursor guard first; corrected mutation site",
    "two closed frame lists had incorrect insertion order; same authorized entries sorted",
    "candidate 03 cold-helper edit required rustfmt; format applied before release builds",
    "inherited anonymous control repetition loops replaced by exact brief-563 workloads; only three control floors recalibrated before candidate 03",
    "candidate 04 structural expression formatting adjusted by cargo fmt"
  ],
  "noise_floor_correction": "Only repeated-tag N reduced 20000 -> 10000 after 2N hit iteration-work-limit; incomplete pair archived, completed unaffected pairs retained",
  "worst_sample_pct": 0.2810840376074042,
  "focused_extra_reason": "Candidates 04-08 respond to the disclosed instruction failures; semantic oracles and each changed structural contract were rerun. The final selected-source structural recheck passed.",
  "tuning_stopped": true,
  "tuning_stop_reason": "eight measured candidates cap reached",
  "selected_candidate": "candidate-03",
  "selected_performance_ok": false,
  "selected_regressing_labels": [
    "g7-sow-default-tag",
    "g7-sow-predicate-hit"
  ],
  "selected_worst_regression_pct": 0.1693364195158198
}
```

Disclosed non-green/interrupted steps: the intentional seven red cases; the first structural early-commit falsifier hit the cursor check and its synthetic insertion was corrected; the first runtime run found two closed-list insertion-order failures, fixed by sorting the same authorized additions; the first noise-floor repeated-tag 2N exceeded iteration work; candidate 02 alloc build was interrupted by upstream capacity and resumed unchanged in a fresh target (old log/target retained); fmt-03/04 requested wrapping after the helper/contract edits and cargo fmt applied it; the first certificate replay under each feature correctly rejected stale recorded cells before installation. Every rejected candidate/measurement remains in scratch. No failing semantic assertion was weakened.

The selected candidate still has measured performance defects; it is NOT ready for acceptance. Exact residual rows are above. Tuning stopped at the recorded bound and all mandatory verification was still run. Step B still owns the thirteen trace continuation losses and the 96-under-32 held-expression regression control. Existing ReapRenderNode self-reaching debt, Unsplit/Pending retention classifications, generic OptionReentry boundary, and other source-group obligations are unchanged; there is no new retention-coverage or universal stack-safety claim. Director review is required for any recorded performance residual; no production edit-surface expansion was taken. Changes are left uncommitted for the director. No git command or network request was run.

Residual attribution for the selected candidate 03: accepted default-tag/predicate-hit Sow plus bucket self instructions are 304 in the base and 325 in the candidate. The bucket helper is inlined in the base and called separately in the candidate. Changing the parent record_sown_value helper or its attributes is outside the allowed reap.rs edit surface, so it remains untouched; a further experiment there needs a director-scoped follow-up. This is a diagnostic lead, not an unmeasured claim that the next change would pass.

Repository-only boundary deviation: the resume liveness check read /proc to check for orphaned cargo/rustc/perf/valgrind processes; none were active. That read exceeded the brief's repository-only boundary. No external file was modified.

Final independent source/binary/inventory boundary audit:

```json
{
  "source_fence_passed": true,
  "parent_only_declaration_and_eval_sow_relocation": true,
  "bucket_and_occurrence_loop_byte_identical": true,
  "source_files": 11,
  "all_changed_files": 18,
  "selected": "candidate-03",
  "selected_source_and_binary_hashes_passed": true,
  "base_hashes_unchanged": true,
  "host_payloads_self_reaching_storage_unchanged": true,
  "inventory_rows_exactly_match_disclosed_replays": true
}
```
