# brief-573 — G4 step C: a flat, resumable held-pattern resolver

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

Held matcher patterns are resolved by evaluating their ordinary
subexpressions while pattern constructs stay held
(`resolve_match_pattern_subexprs`,
`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:46-75`,
with the classifier at `:177-197` and `strip_hold_pattern` in
`substitution/function_and_subst_helpers.rs:23`). A subexpression that asks
for input resumes past the resolution — `MatchQ-pattern-args` resumes to
`False` where the queued path returns `True`, `Cases-pattern` to `List[]`
where it returns `List[h[1,2,3]]` — and the resolver, classifier and strip are
guest-capable or pure recursions with no guard. Replace them with brief-570's
design: an explicit work stack that is the suspension state, root
normalization exactly once on completion, then the existing consumer frames.
Sites P0774, P0776, P0777: lost continuations **46 -> 43**, and with them
group G4 is closed (thirteen repairs, three retirements).

brief-570's report and `.temp/brief-570/draft-C-held-pattern-continuations.md`
are the raw material; **this brief governs** where they differ.

## Context pointers

- brief-570 and its report; `.temp/brief-570/design.md`, `recursion.md`,
  `inventory-plan.md` (section C), `red-tests.md`, `C-matcher-cases.json`,
  `probe.rs`, `post-fix-depth.md`, `post-fix-depth.rs`.
- brief-572 and its appended report: the accepted rule walk
  (`definitions/rule_lhs.rs`, `rule_lhs/`), the shared test harness
  (`crates/rum-stdlib/tests/rule_pattern_continuations.rs`) and the structural
  contract (`tests/association_depth_contract/rule_pattern_depth_contract.rs`)
  this step extends.
- `eval_call_prepare.rs:484, 498`;
  `definitions/definition_catalog/rewrite_rules.rs:142-167`; the existing
  consumer frames in `recursion.md`; `suspension/core.rs`.

## Scope

**Objective:** the flat held resolver, classifier and strip, the owning tests,
the extended structural contract, the private depth tests, one guide
clarification, and the standing and measurement gates.

**Edit surface:** `cases/match_q.rs` (delegate to the new driver; keep MatchQ's
validation, fast paths and existing frames); `cases.rs` (module declaration);
new `cases/pattern_resolution.rs` and `cases/pattern_resolution/{state,continuation,tests}.rs`;
`substitution/function_and_subst_helpers.rs` (`strip_hold_pattern` iterative,
nothing else); `rule_pattern_continuations.rs` and children (add the matcher
fixtures; brief-572's stay green); `association_depth_contract/{mod,recursion_inventory,rule_pattern_depth_contract}.rs`;
rum-runtime's frame lists (exactly five new derive entries and one frame,
named-frame count **558 -> 559**); `docs/guide/patterns-and-rules.md` (one
clarification: a held pattern's ordinary subexpressions and prepared calls
resume in order; the holding policy is unchanged).

**Inventory exception, authorized by name:** relocate `unknown_captures.tsv`
row 852 (the `matches_pattern_head` closure at `match_q.rs:197:22`) into
`pattern_resolution.rs`, keeping its `UnknownConstruct` identity and text.
`N:Count`, `N:DeleteCases`, `N:FreeQ`, `N:MatchQ`, `N:MemberQ` keep class and
status; only evidence positions move.

**Standing duty — always in scope** (`TEMPLATE.md`). **STOP** on any other
row change or disappearance, any new `UnknownHostPayload` or `self_reaching`
row, any new row in untouched code, or any retention reclassification.

**Fence:** no change to the matcher (`MatcherNode`, compile, match, codecs),
the match normalizer or comparator, repetition validation, consumer
traversals, request policy, the generic evaluator, `Expr`, manifests, the
catalog or other docs. **Do not add frames to callers that already own their
continuation** — return the complete resolver result instead; an extra outer
frame duplicates suffixes or guest calls. No `NativeStackDepth` in a frame, no
recursive fallback, no depth cap.

**Disclosure duty:** every changed function, state field and root, every
inventory row change, every measured label, any adjacent defect.

## Deliverable

brief-570 design.md §3 for the held resolver: the root unary `HoldPattern`
stripped once at entry; recognized pattern calls opaque; head then selected
arguments left to right; once a call is prepared, its work item is popped and
its arguments **moved** into `resume_call_from_prepared_args` — if that call
suspends, only the parent state and the prepared-call destination are frozen,
and on resume the completed value is inserted once (never applied again); the
root normalized exactly once, including a pattern-free root that suspended in
its ordinary evaluation; then the existing MatchQ / Cases / Count / MemberQ /
FreeQ / Position / DeleteCases / Catch / ReplacePart suffix. Cases' symbol and
rule candidate branch is unchanged. Capture only on `InputRequired`, on the
error edge; a cold working copy on an actual `Ok` resume; a second suspension
freezes the new state; abandonment runs nothing pending.

## Tests

Red first, recorded before any production edit: the 29 comparisons of
`C-matcher-cases.json` (20 red, nine controls) through the shared oracle, plus
MatchQ and Cases distinct twice, nested patterns, a nested prepared call that
itself suspends, a root-only suspended prepared call (a passing control), an
ordinary error after a resume, cancellation at the first and second prompt,
answer-free abandonment for MatchQ and Cases. Every brief-572 case stays
green.

**Structural contract** (extend `rule_pattern_depth_contract.rs`): both owned
module trees scanned per file and combined, the shared classifier and strip
extracted and checked, outbound calls audited; falsifiers for direct, mutual
and callback-hidden recursion, a helper escape, an injected self-reaching
row, a missing root, both await variants.

**Private depth tests** (`post-fix-depth.md`): a ruled 2 MiB thread that owns
construction and teardown, the held driver at **n = 2 000** (ordinary, one and
two suspensions, an error, abandonment, a prepared call), with a shallow
collapsing head so the compiled result stays shallow; 20 000 scalar work
records through the state's copy, trace, drop and abandonment. **Run the
construction-and-teardown control first**; if it fails at 2 000, STOP and
report the measured ceiling. Run no probe that aborts a process, and none of
the director's deep-literal programs.

## rum / Rust pitfalls (read twice)

1. A prepared call can run guest code; it is not a constructor.
2. The detector reads syntax, not symbol values: a symbol whose value is a
   deep list is not resolved again. Preserve that; do not use it to claim the
   walker is depth-safe.
3. The matcher's compiler, binding scan, match recursion, codecs and `Expr`'s
   shared operations are separate depth risks owned by the hardening line —
   never fold them in.
4. One change per measured candidate; stop-loss keyed on progress.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-573/tmp …`. Bounds: focused 150 s; runtime,
ownership and retention 300 s; a one-unit release build 900 s; a metric
process 150 s; `cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test rule_pattern_continuations
cargo test -p rum-stdlib --lib rule_pattern_depth_contract
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Measure, do not tune.** G4's ordinary-path cost (step B's residuals and
whatever this step adds) is erased as a whole by the next brief, against the
pre-G4 base. Here, build the final candidate with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and `--features
rum-cli/alloc-count` (two arguments), and measure it against **both**
`.temp/brief-573/base/` (the director's one-unit build of `d2e0641c`, after
step B) and `.temp/brief-572/base/` (pre-G4): allocation counts, then two
paired samples of `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, one
label per process, stdin `/dev/null`, `(count(2N) - count(N)) / N`, each
result checked first. Panel: brief-572's labels (`.temp/brief-572/perf/`) plus
new no-input held-pattern labels from `performance-plan.md` — an active child,
an active head, a prepared call, inline-pattern depth 8 and 32 — and the
neighbours `matchq-integer-*`, `cases-*`, `replaceall-simple`. One measured
candidate; no timing suite. Report every row and every regression.

## Checkpointing — protect the scene

Everything under `.temp/brief-573/`; `PROGRESS.md` after the red baseline,
each compiled attempt and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository.
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the release `rum-cli` on passing programs, `taskset`/`perf`,
  `valgrind --tool=callgrind`. **No probe that aborts a process.**
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed functions, state fields and roots, holding and completion, file
   sizes, the guide line.
2. Verbatim red and green output, structural checks and falsifiers, depth
   tests, the broad suites; every inventory row change.
3. Three sites repaired (46 -> 43 predicted; G4 closed); measured rows against
   both bases, both samples.
4. Stop-loss counters, residuals, open questions.

---

## brief-573 execution report

### 1. Changed functions, state and holding/completion contract

Production candidate 01 is the sole implementation/measurement candidate. The flat resolver, classifier and unary strip compile and all target tests pass on the first implementation attempt; no tuning or source-level performance change has been made.

Changed production functions:
- cases/match_q.rs: resolve_matcher_pattern_expr_owned now delegates. resolve_match_pattern_subexprs is replaced by the flat driver; contains_pattern_syntax and matches_pattern_head move to the new owner. Validation, fast paths and the three consumer frames remain unchanged apart from positions/import.
- cases/pattern_resolution.rs: resolve_held_matcher_pattern strips root unary HoldPattern; drive_matcher_pattern visits raw head then arguments, keeps pattern calls opaque, moves a popped work item's prefix into resume_call_from_prepared_args, and normalizes the completed root once. suspend_matcher_pattern captures only an InputRequired error. accept_matcher_pattern_value inserts each result, advances the cursor, and pops a completed call. contains_pattern_syntax is an iterative borrowed syntax scan; matches_pattern_head retains its OptionsPattern exclusion and original closure.
- pattern_resolution/state.rs: MatcherPatternState::destination distinguishes Root/Head/Argument from the last parent record.
- pattern_resolution/continuation.rs: MatcherPatternSuspensionFrame::resume forwards Err untouched and calls cold resume_matcher_pattern only on Ok; matcher_pattern_working_copy makes the cold clone. Successful evaluation and PreparedCall answers share acceptance. A second suspension freezes the changed live state; the generic runtime retains the outer consumer suffix.
- substitution/function_and_subst_helpers.rs: strip_hold_pattern uses a while loop; no other helper changed.

State/root disclosure:
MatcherPatternState.stack is Vec<MatcherPatternWork>. Each work record contains original_head: Expr, original_args: SharedArgs, resolved_head: Option<Expr>, prefix: Vec<Expr>, and next: usize. The first four trace original, held, pending, unvisited and completed values; the cursor is scalar. MatcherPatternDestination is scalar Root/Head/Argument(usize). MatcherPatternAwait is Evaluate { destination, pending_expr: Expr } or PreparedCall { destination }; the latter holds no already-submitted call. MatcherPatternSuspensionFrame owns state and pending; PureFrameAbandon runs none of the remaining evaluation or normalization. MatcherPatternStep is local Visit(Expr), PreparedCall { head: Expr, args: Vec<Expr> }, Done(Expr), never captured. No frame stores a session borrow, observer, NativeStackDepth or recursive state link. No new unsafe, recursive fallback, depth cap, instrumentation or caller frame.

Source contract: both owned production module trees plus extracted definition_lhs_contains_pattern_syntax and strip_hold_pattern, eight files in total. Exact function sets, per-file scans, per-tree scans, combined scan and explicit outbound calls are checked. The scanner now conservatively retains all owned candidates for an ambiguous method name (the two state destination methods), with a falsifier. Direct, cross-file mutual, function-item callback, callback alias, closure, Self method, helper escape, missing file/function/root, both await variants and both injected self-reaching state rows are covered. This is a bounded source audit, not arbitrary Rust resolution.

Guide: one paragraph after the existing rule preparation clarification: held matcher ordinary subexpressions and prepared calls resume in order after input; recognized pattern constructs remain held.

Depth scope: construction/teardown control first; all AST construction and teardown on the ruled 2MiB thread. n=16 controls and n=2000 driver tests use a shallow collapsing head whose invocation count is checked, including queued versus suspended paths, one/two inputs, error, abandonment at either prompt and apply-once prepared-call input. Raw roots stay alive until frames retire and are drained iteratively. Twenty thousand scalar work records exercise ordinary/cold copies, trace, drop and both await variants' pure abandonment. A separate ordinary root suspension verifies final normalization. This proves owned driver/state behavior only; raw repetition validation, matcher compilation/binding scan/matching/codecs, comparison, generic evaluation and deep last-owner Expr operations remain adjacent hardening boundaries. No abort or director-only program ran.

Inventory: the authorized closure identity is at supplied-baseline unknown_captures.tsv row 850 (match_q.rs:188:22), not historical row 852 (match_q.rs:197:22). Authorization is by the exact named closure, and its UnknownConstruct text is retained. All other moved entries are source-position-only. No row removal, new UnknownHostPayload/self-reaching row, untouched-code addition or retention reclassification.

File-size assessment: driver 7,115 chars, state 1,183, continuation 1,401; private tests 12,753 (cohesive depth/root lifecycle suite, below the general 15k test preference but above the brief's preferred 10k). Structural contract is 15,259 (slightly inside test warning range; combines the two required owner contracts and their falsifiers). Existing recursion_inventory 32,612 and frame_root_trace 30,342 remain split-plan files; this task only changes the shared conservative method edge collector and exact count respectively. Their future boundaries are the standalone flat-walk scanner and frame-discovery/fixture checks. derived_inventory 171,460 remains existing closed-list data debt; exactly five trace entries and one PureFrameAbandon entry added, not an unrelated repartition. A future split can separate the FrameRootTrace and PureFrameAbandon lists. No new runtime-root file; the module sits under cases/.

Additional changed test functions: `lifecycle::cancel_case` gains MatchQ/Cases branches; `abandon_matcher_without_answer`, `cancel_matchq_first`, `cancel_matchq_second`, `cancel_cases_first`, `cancel_cases_second`, `abandon_matchq_without_answer`, and `abandon_cases_without_answer` are new. `recursion_inventory::FlatCalls::visit_expr_method_call` now retains all owned same-name candidates. The structural contract generalizes `sources`, `assert_files_and_functions`, `assert_outbound`, `structural_rule_pattern_depth_contract`, `assert_no_self_reaching`, `rule_lhs_ownership_is_not_self_reaching`, `assert_root_fields` and the existing falsifiers to both owners; it adds `root_fields`, `matcher_self_reaching_state_falsifier`, `matcher_missing_await_root_falsifier`, `matcher_missing_prepared_await_falsifier`, and `ambiguous_method_recursion_cannot_escape_combined_scan`. The new file function inventory below includes every new driver, state, continuation, integration and private-test function. Existing Rule fixtures/support are untouched.

`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs`: `resolve_held_matcher_pattern`, `drive_matcher_pattern`, `suspend_matcher_pattern`, `accept_matcher_pattern_value`, `contains_pattern_syntax`, `matches_pattern_head`.

`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/continuation.rs`: `resume`, `resume_matcher_pattern`, `matcher_pattern_working_copy`.

`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/state.rs`: `destination`.

`crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs`: `on_small_stack`, `shallow`, `spine`, `drain`, `matcher_pattern_construction_and_teardown_control`, `matcher_pattern_deep_classifier_and_strip`, `prompt`, `direct_walk`, `matcher_pattern_deep_ordinary`, `matcher_pattern_deep_one_suspension`, `matcher_pattern_deep_two_suspensions`, `matcher_pattern_deep_error_after_resume`, `matcher_pattern_deep_abandon_first_and_second`, `matcher_pattern_deep_prepared_call_suspension`, `matcher_pattern_ordinary_root_resume_normalizes`, `visit_expr`, `visit_symbol`, `roots`, `root_fixture`, `assert_roots`, `matcher_pattern_roots_cover_every_field_and_await_variant`, `matcher_pattern_missing_root_falsifier`, `matcher_pattern_scalar_state_copy_trace_drop_abandon_20000`, `matcher_pattern_error_resume_runs_no_work`.

`crates/rum-stdlib/tests/rule_pattern_continuations/matcher.rs`: `matchq_pattern_args`, `matchq_pattern_head`, `matchq_pattern_twice`, `matchq_normalize_head`, `cases_pattern`, `deletecases_pattern`, `position_pattern`, `count_pattern`, `memberq_pattern`, `freeq_pattern`, `catch_pattern`, `reap_pattern`, `matchq_distinct_twice`, `cases_distinct_twice`, `matchq_nested_pattern`, `cases_nested_pattern`, `matchq_prepared_call_suspends`, `matchq_error_after_resume`, `cases_error_after_resume`, `matchq_condition_held`, `matchq_patterntest_held`, `control_cases`, `control_cases_target`, `control_cases_spec`, `control_cases_rule`, `control_matchq_normalize_twice`, `replacepart_resolver`, `replacepart_guarded_resolver`, `matchq_nested_prepared_call`.

Final file sizes (characters; sizes exclude this appended report):

| File | Before | After |
| --- | ---: | ---: |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases.rs` | 53 | 77 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs` | 8948 | 7168 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs` | 0 | 7115 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/continuation.rs` | 0 | 1401 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/state.rs` | 0 | 1183 |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/tests.rs` | 0 | 12753 |
| `crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs` | 5608 | 5579 |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` | 30342 | 30342 |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | 170719 | 171460 |
| `crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs` | 32108 | 32612 |
| `crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs` | 10022 | 15259 |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | 248115 | 249096 |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 420100 | 420111 |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 397500 | 398282 |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 91646 | 91646 |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv` | 111757 | 111812 |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 18161 | 18161 |
| `crates/rum-stdlib/tests/rule_pattern_continuations.rs` | 171 | 188 |
| `crates/rum-stdlib/tests/rule_pattern_continuations/lifecycle.rs` | 2790 | 4505 |
| `crates/rum-stdlib/tests/rule_pattern_continuations/matcher.rs` | 0 | 8300 |
| `docs/guide/patterns-and-rules.md` | 7006 | 7184 |

Guide clarification: `docs/guide/patterns-and-rules.md:149`. New derive entries are the five named types above; `MatcherPatternSuspensionFrame` is the sole new PureFrameAbandon/named frame, **558 → 559**.

### 2. Red/green, structural/depth, standing and broad gates

All Cargo and metric processes ran serially through `scripts/run-bounded.py`, with `CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true` and `TMPDIR=/home/remi/dev/rum/.temp/brief-573/tmp`. The broad/replay commands use the byte-identical `verification-source` export to confine generated artifacts. Each exact argv, cwd, exit and elapsed duration is recorded in `.temp/brief-573/logs/*.json`; full unfiltered output is in the corresponding `.txt`. No git, network, unsafe change, abort probe, or director-only literal program was used.

Red baseline ran before every production edit. It reproduced 20 red held comparisons and nine equal controls, plus two second-prompt cancellation failures. All 36 brief-572 tests stayed green. There are 58 total oracle rows: the required 56 original comparisons plus two existing policy comparisons.

```text
test result: FAILED. 49 passed; 22 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s
```

Verbatim 29 red matcher rows (fields: ROW, label, resumed value/log, queued value/log, actual/expected prompt count, equality, statuses, terminations, ordered messages, prints):

```tsv
ROW	Cases-distinct-twice	"List[]"	"List[2]"	"List[h[1, 2, 4, 5]]"	"List[2, 4, 5]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Cases-error-after-resume	"List[]"	"List[1]"	"<error>"	"List[1, 2]"	1	1	false	"OK"	"Other { message: \"uncaught throw: \\\"g4 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g4 boom\"] returned to top level." }]	["prefix"]	["prefix", "suffix"]
ROW	Cases-nested-pattern	"List[]"	"List[2]"	"List[h[g[1, 2], 7, 3]]"	"List[2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Cases-pattern	"List[]"	"List[2]"	"List[h[1, 2, 3]]"	"List[2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Catch-pattern	"<error>"	"List[2]"	"99"	"List[2, 3]"	1	1	false	"Other { message: \"uncaught throw: 99 with tag h[1, 2, 3]\" }"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[99, h[1, 2, 3]] returned to top level." }]	[]	[]	[]
ROW	control-Cases	"List[1, 2, 3]"	"List[1, 2, 3]"	"List[1, 2, 3]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	control-Cases-rule	"List[1, \"two\", 3]"	"List[1, 2, 3]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	control-Cases-spec	"List[2]"	"List[2]"	"List[2]"	"List[2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	control-Cases-target	"List[2]"	"List[1]"	"List[2]"	"List[1]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	control-MatchQ-normalize-twice	"True"	"List[2, 4]"	"True"	"List[2, 4]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Count-pattern	"0"	"List[2]"	"1"	"List[2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	DeleteCases-pattern	"List[h[1, 2, 3], h[4, 5, 6]]"	"List[2]"	"List[h[4, 5, 6]]"	"List[2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	FreeQ-pattern	"False"	"List[2]"	"False"	"List[2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-Condition-held	"False"	"List[2]"	"False"	"List[2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-distinct-twice	"False"	"List[2]"	"True"	"List[2, 4, 5]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-error-after-resume	"False"	"List[1]"	"<error>"	"List[1, 2]"	1	1	false	"OK"	"Other { message: \"uncaught throw: \\\"g4 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g4 boom\"] returned to top level." }]	["prefix"]	["prefix", "suffix"]
ROW	MatchQ-nested-pattern	"False"	"List[2]"	"True"	"List[2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-nested-prepared-call	"False"	"List[1, 2]"	"True"	"List[1, 2]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-normalize-head	"False"	"List[1]"	"True"	"List[1]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-pattern-args	"False"	"List[2]"	"True"	"List[2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-pattern-head	"False"	"List[2]"	"True"	"List[2]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-pattern-twice	"False"	"List[2]"	"True"	"List[2, 4]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-PatternTest-held	"False"	"List[2]"	"False"	"List[2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-prepared-call-suspends	"True"	"List[1, 2]"	"True"	"List[1, 2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MemberQ-pattern	"False"	"List[2]"	"True"	"List[2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Position-pattern	"List[List[1, 2]]"	"List[2]"	"List[List[1]]"	"List[2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Reap-pattern	"List[99, List[List[99]]]"	"List[2, 3]"	"List[99, List[List[99]]]"	"List[2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	ReplacePart-guarded-resolver	"List[List[1, 2], 9]"	"List[2]"	"List[List[1, 9], List[3, 9]]"	"List[2]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	ReplacePart-resolver	"List[List[1, 2], 9]"	"List[2]"	"List[List[1, 9], List[3, 9]]"	"List[2]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
```

Verbatim 29 green matcher rows and exact prompts:

```tsv
ROW	Cases-distinct-twice	"List[h[1, 2, 4, 5]]"	"List[2, 4, 5]"	"List[h[1, 2, 4, 5]]"	"List[2, 4, 5]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Cases-error-after-resume	"<error>"	"List[1, 2]"	"<error>"	"List[1, 2]"	1	1	true	"Other { message: \"uncaught throw: \\\"g4 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g4 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g4 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g4 boom\"] returned to top level." }]	["prefix", "suffix"]	["prefix", "suffix"]
ROW	Cases-nested-pattern	"List[h[g[1, 2], 7, 3]]"	"List[2, 3]"	"List[h[g[1, 2], 7, 3]]"	"List[2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Cases-pattern	"List[h[1, 2, 3]]"	"List[2, 3]"	"List[h[1, 2, 3]]"	"List[2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Catch-pattern	"99"	"List[2, 3]"	"99"	"List[2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	control-Cases	"List[1, 2, 3]"	"List[1, 2, 3]"	"List[1, 2, 3]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	control-Cases-rule	"List[1, \"two\", 3]"	"List[1, 2, 3]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	control-Cases-spec	"List[2]"	"List[2]"	"List[2]"	"List[2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	control-Cases-target	"List[2]"	"List[1]"	"List[2]"	"List[1]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	control-MatchQ-normalize-twice	"True"	"List[2, 4]"	"True"	"List[2, 4]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Count-pattern	"1"	"List[2, 3]"	"1"	"List[2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	DeleteCases-pattern	"List[h[4, 5, 6]]"	"List[2, 3]"	"List[h[4, 5, 6]]"	"List[2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	FreeQ-pattern	"False"	"List[2, 3]"	"False"	"List[2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-Condition-held	"False"	"List[2]"	"False"	"List[2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-distinct-twice	"True"	"List[2, 4, 5]"	"True"	"List[2, 4, 5]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-error-after-resume	"<error>"	"List[1, 2]"	"<error>"	"List[1, 2]"	1	1	true	"Other { message: \"uncaught throw: \\\"g4 boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"g4 boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g4 boom\"] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"g4 boom\"] returned to top level." }]	["prefix", "suffix"]	["prefix", "suffix"]
ROW	MatchQ-nested-pattern	"True"	"List[2, 3]"	"True"	"List[2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-nested-prepared-call	"True"	"List[1, 2]"	"True"	"List[1, 2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-normalize-head	"True"	"List[1]"	"True"	"List[1]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-pattern-args	"True"	"List[2, 3]"	"True"	"List[2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-pattern-head	"True"	"List[2]"	"True"	"List[2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-pattern-twice	"True"	"List[2, 4]"	"True"	"List[2, 4]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-PatternTest-held	"False"	"List[2]"	"False"	"List[2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MatchQ-prepared-call-suspends	"True"	"List[1, 2]"	"True"	"List[1, 2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	MemberQ-pattern	"True"	"List[2, 3]"	"True"	"List[2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Position-pattern	"List[List[1]]"	"List[2, 3]"	"List[List[1]]"	"List[2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	Reap-pattern	"List[99, List[List[99]]]"	"List[2, 3]"	"List[99, List[List[99]]]"	"List[2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	ReplacePart-guarded-resolver	"List[List[1, 9], List[3, 9]]"	"List[2]"	"List[List[1, 9], List[3, 9]]"	"List[2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ROW	ReplacePart-resolver	"List[List[1, 9], List[3, 9]]"	"List[2]"	"List[List[1, 9], List[3, 9]]"	"List[2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
```

```tsv
PROMPTS	Cases-distinct-twice	["first: ", "second: "]	["first: ", "second: "]
PROMPTS	Cases-error-after-resume	["first: "]	["first: "]
PROMPTS	Cases-nested-pattern	["first: "]	["first: "]
PROMPTS	Cases-pattern	["first: "]	["first: "]
PROMPTS	Catch-pattern	["first: "]	["first: "]
PROMPTS	control-Cases	["p: "]	["p: "]
PROMPTS	control-Cases-rule	["p: "]	["p: "]
PROMPTS	control-Cases-spec	["p: "]	["p: "]
PROMPTS	control-Cases-target	["p: "]	["p: "]
PROMPTS	control-MatchQ-normalize-twice	["p: ", "p: "]	["p: ", "p: "]
PROMPTS	Count-pattern	["first: "]	["first: "]
PROMPTS	DeleteCases-pattern	["first: "]	["first: "]
PROMPTS	FreeQ-pattern	["first: "]	["first: "]
PROMPTS	MatchQ-Condition-held	["first: "]	["first: "]
PROMPTS	MatchQ-distinct-twice	["first: ", "second: "]	["first: ", "second: "]
PROMPTS	MatchQ-error-after-resume	["first: "]	["first: "]
PROMPTS	MatchQ-nested-pattern	["first: "]	["first: "]
PROMPTS	MatchQ-nested-prepared-call	["first: "]	["first: "]
PROMPTS	MatchQ-normalize-head	["p: "]	["p: "]
PROMPTS	MatchQ-pattern-args	["p: "]	["p: "]
PROMPTS	MatchQ-pattern-head	["p: "]	["p: "]
PROMPTS	MatchQ-pattern-twice	["p: ", "p: "]	["p: ", "p: "]
PROMPTS	MatchQ-PatternTest-held	["first: "]	["first: "]
PROMPTS	MatchQ-prepared-call-suspends	["first: "]	["first: "]
PROMPTS	MemberQ-pattern	["first: "]	["first: "]
PROMPTS	Position-pattern	["first: "]	["first: "]
PROMPTS	Reap-pattern	["first: "]	["first: "]
PROMPTS	ReplacePart-guarded-resolver	["first: "]	["first: "]
PROMPTS	ReplacePart-resolver	["first: "]	["first: "]
```

Final integration result, including every brief-572 case and all MatchQ/Cases lifecycle cases:

```text
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s
```

Verbatim lifecycle records from the final integration run:

```text
test rule_pattern_continuations::lifecycle::abandon_cases_without_answer ... ABANDON_WITHOUT_ANSWER	Cases	(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["prefix"] }, "List[1]", ["first: "])
test rule_pattern_continuations::lifecycle::abandon_matchq_without_answer ... ABANDON_WITHOUT_ANSWER	MatchQ	(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["prefix"] }, "List[1]", ["first: "])
test rule_pattern_continuations::lifecycle::abandon_without_answer ... ABANDON_WITHOUT_ANSWER	(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["prefix"] }, "List[1]", ["first: "])
test rule_pattern_continuations::lifecycle::cancel_cases_first ... CANCEL	Cases	stop=0	(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["first"] }, "List[1]", [])
test rule_pattern_continuations::lifecycle::cancel_cases_second ... CANCEL	Cases	stop=1	(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["first", "second"] }, "List[1, 2, 3]", [])
test rule_pattern_continuations::lifecycle::cancel_delayed_first ... CANCEL	RuleDelayed	stop=0	(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["first"] }, "List[1]", [])
test rule_pattern_continuations::lifecycle::cancel_delayed_second ... CANCEL	RuleDelayed	stop=1	(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["first", "second"] }, "List[1, 2, 3]", [])
test rule_pattern_continuations::lifecycle::cancel_matchq_first ... CANCEL	MatchQ	stop=0	(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["first"] }, "List[1]", [])
test rule_pattern_continuations::lifecycle::cancel_matchq_second ... CANCEL	MatchQ	stop=1	(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["first", "second"] }, "List[1, 2, 3]", [])
test rule_pattern_continuations::lifecycle::cancel_rule_first ... CANCEL	Rule	stop=0	(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["first"] }, "List[1]", [])
test rule_pattern_continuations::lifecycle::cancel_rule_second ... CANCEL	Rule	stop=1	(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["first", "second"] }, "List[1, 2, 3]", [])
```

Construction-and-teardown control ran first on the ruled 2 MiB thread, before any deep driver case:

```text
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_construction_and_teardown_control ... HELD_CONSTRUCTION_TEARDOWN n=2000 stack=2MiB passed
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 0.01s
```

Verbatim private depth/root evidence (n=16 controls and n=2000, no lower substituted depth):

```text
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_construction_and_teardown_control ... HELD_CONSTRUCTION_TEARDOWN n=2000 stack=2MiB passed
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_deep_abandon_first_and_second ... HELD_DEEP n=16 queued=false prompts=2 error=false abandon=Some(0) prepared=false stack=2MiB passed
HELD_DEEP n=2000 queued=false prompts=2 error=false abandon=Some(0) prepared=false stack=2MiB passed
HELD_DEEP n=16 queued=false prompts=2 error=false abandon=Some(1) prepared=false stack=2MiB passed
HELD_DEEP n=2000 queued=false prompts=2 error=false abandon=Some(1) prepared=false stack=2MiB passed
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_deep_classifier_and_strip ... HELD_CLASSIFIER_AND_STRIP n=2000 true/false, unary strip, OptionsPattern excluded stack=2MiB passed
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_deep_error_after_resume ... HELD_DEEP n=16 queued=true prompts=1 error=true abandon=None prepared=false stack=2MiB passed
HELD_DEEP n=16 queued=false prompts=1 error=true abandon=None prepared=false stack=2MiB passed
HELD_DEEP n=2000 queued=true prompts=1 error=true abandon=None prepared=false stack=2MiB passed
HELD_DEEP n=2000 queued=false prompts=1 error=true abandon=None prepared=false stack=2MiB passed
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_deep_one_suspension ... HELD_DEEP n=16 queued=true prompts=1 error=false abandon=None prepared=false stack=2MiB passed
HELD_DEEP n=16 queued=false prompts=1 error=false abandon=None prepared=false stack=2MiB passed
HELD_DEEP n=2000 queued=true prompts=1 error=false abandon=None prepared=false stack=2MiB passed
HELD_DEEP n=2000 queued=false prompts=1 error=false abandon=None prepared=false stack=2MiB passed
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_deep_ordinary ... HELD_DEEP n=16 queued=true prompts=0 error=false abandon=None prepared=false stack=2MiB passed
HELD_DEEP n=16 queued=false prompts=0 error=false abandon=None prepared=false stack=2MiB passed
HELD_DEEP n=2000 queued=true prompts=0 error=false abandon=None prepared=false stack=2MiB passed
HELD_DEEP n=2000 queued=false prompts=0 error=false abandon=None prepared=false stack=2MiB passed
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_deep_prepared_call_suspension ... HELD_DEEP n=16 queued=true prompts=1 error=false abandon=None prepared=true stack=2MiB passed
HELD_DEEP n=16 queued=false prompts=1 error=false abandon=None prepared=true stack=2MiB passed
HELD_DEEP n=2000 queued=true prompts=1 error=false abandon=None prepared=true stack=2MiB passed
HELD_DEEP n=2000 queued=false prompts=1 error=false abandon=None prepared=true stack=2MiB passed
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_deep_two_suspensions ... HELD_DEEP n=16 queued=true prompts=2 error=false abandon=None prepared=false stack=2MiB passed
HELD_DEEP n=16 queued=false prompts=2 error=false abandon=None prepared=false stack=2MiB passed
HELD_DEEP n=2000 queued=true prompts=2 error=false abandon=None prepared=false stack=2MiB passed
HELD_DEEP n=2000 queued=false prompts=2 error=false abandon=None prepared=false stack=2MiB passed
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_roots_cover_every_field_and_await_variant ... HELD_ROOTS raw head, held/pending/unvisited args, resolved head, prefix, Evaluate/PreparedCall and all destinations passed
test runtime::kernel_data::rewrite_data::cases::pattern_resolution::tests::matcher_pattern_scalar_state_copy_trace_drop_abandon_20000 ... HELD_SCALAR_STATE records=20000 ordinary/cold copy, trace, drop, both await variants pure abandon stack=2MiB passed
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1108 filtered out; finished in 0.59s
```

Verbatim structural contract/falsifier output:

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 58.97s
     Running unittests src/lib.rs (.temp/brief-573/test-target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 13 tests
test tests::association_depth_contract::rule_pattern_depth_contract::ambiguous_method_recursion_cannot_escape_combined_scan ... ok
test tests::association_depth_contract::rule_pattern_depth_contract::direct_cross_module_and_callback_recursion_falsifiers ... G4_RECURSION_FALSIFIERS direct, cross-module, function-item, callback alias, closure, Self method fired
ok
test tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_await_root_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
assertion `left == right` failed: required G4 root fields
  left: {"MatcherPatternAwait::Evaluate": {"destination"}, "MatcherPatternAwait::PreparedCall": {"destination"}, "MatcherPatternState": {"stack"}, "MatcherPatternSuspensionFrame": {"pending", "state"}, "MatcherPatternWork": {"original_args", "original_head", "prefix", "resolved_head"}, "RuleLhsAwait": {"destination", "pending_expr"}, "RuleLhsCompletionFrame": {"rhs"}, "RuleLhsState": {"stack"}, "RuleLhsWork": {"original_args", "original_head", "policy", "prefix", "resolved_head"}, "RulePatternSuspensionFrame": {"pending", "state"}}
 right: {"MatcherPatternAwait::Evaluate": {"destination", "pending_expr"}, "MatcherPatternAwait::PreparedCall": {"destination"}, "MatcherPatternState": {"stack"}, "MatcherPatternSuspensionFrame": {"pending", "state"}, "MatcherPatternWork": {"original_args", "original_head", "prefix", "resolved_head"}, "RuleLhsAwait": {"destination", "pending_expr"}, "RuleLhsCompletionFrame": {"rhs"}, "RuleLhsState": {"stack"}, "RuleLhsWork": {"original_args", "original_head", "policy", "prefix", "resolved_head"}, "RulePatternSuspensionFrame": {"pending", "state"}}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
ok
test tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_missing_prepared_await_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
assertion `left == right` failed: required G4 root fields
  left: {"MatcherPatternAwait::Evaluate": {"destination", "pending_expr"}, "MatcherPatternState": {"stack"}, "MatcherPatternSuspensionFrame": {"pending", "state"}, "MatcherPatternWork": {"original_args", "original_head", "prefix", "resolved_head"}, "RuleLhsAwait": {"destination", "pending_expr"}, "RuleLhsCompletionFrame": {"rhs"}, "RuleLhsState": {"stack"}, "RuleLhsWork": {"original_args", "original_head", "policy", "prefix", "resolved_head"}, "RulePatternSuspensionFrame": {"pending", "state"}}
 right: {"MatcherPatternAwait::Evaluate": {"destination", "pending_expr"}, "MatcherPatternAwait::PreparedCall": {"destination"}, "MatcherPatternState": {"stack"}, "MatcherPatternSuspensionFrame": {"pending", "state"}, "MatcherPatternWork": {"original_args", "original_head", "prefix", "resolved_head"}, "RuleLhsAwait": {"destination", "pending_expr"}, "RuleLhsCompletionFrame": {"rhs"}, "RuleLhsState": {"stack"}, "RuleLhsWork": {"original_args", "original_head", "policy", "prefix", "resolved_head"}, "RulePatternSuspensionFrame": {"pending", "state"}}
ok
test tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::matcher_self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
self-reaching G4 state
ok
test tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:84:5:
assertion `left == right` failed: required G4 files
  left: {"definitions/definition_lhs_preparation/syntax_policy.rs", "definitions/rule_lhs.rs", "definitions/rule_lhs/continuation.rs", "kernel_data/rewrite_data/cases/pattern_resolution.rs", "kernel_data/rewrite_data/cases/pattern_resolution/continuation.rs", "kernel_data/rewrite_data/cases/pattern_resolution/state.rs", "substitution/function_and_subst_helpers.rs"}
 right: {"definitions/definition_lhs_preparation/syntax_policy.rs", "definitions/rule_lhs.rs", "definitions/rule_lhs/continuation.rs", "definitions/rule_lhs/state.rs", "kernel_data/rewrite_data/cases/pattern_resolution.rs", "kernel_data/rewrite_data/cases/pattern_resolution/continuation.rs", "kernel_data/rewrite_data/cases/pattern_resolution/state.rs", "substitution/function_and_subst_helpers.rs"}
ok
test tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:93:9:
assertion `left == right` failed: required G4 functions in definitions/definition_lhs_preparation/syntax_policy.rs
  left: {}
 right: {"definition_lhs_contains_pattern_syntax"}
ok
test tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:363:5:
assertion `left == right` failed: required G4 root fields
  left: {"MatcherPatternAwait::Evaluate": {"destination", "pending_expr"}, "MatcherPatternAwait::PreparedCall": {"destination"}, "MatcherPatternState": {"stack"}, "MatcherPatternSuspensionFrame": {"pending", "state"}, "MatcherPatternWork": {"original_args", "original_head", "prefix", "resolved_head"}, "RuleLhsAwait": {"destination"}, "RuleLhsCompletionFrame": {"rhs"}, "RuleLhsState": {"stack"}, "RuleLhsWork": {"original_args", "original_head", "policy", "prefix", "resolved_head"}, "RulePatternSuspensionFrame": {"pending", "state"}}
 right: {"MatcherPatternAwait::Evaluate": {"destination", "pending_expr"}, "MatcherPatternAwait::PreparedCall": {"destination"}, "MatcherPatternState": {"stack"}, "MatcherPatternSuspensionFrame": {"pending", "state"}, "MatcherPatternWork": {"original_args", "original_head", "prefix", "resolved_head"}, "RuleLhsAwait": {"destination", "pending_expr"}, "RuleLhsCompletionFrame": {"rhs"}, "RuleLhsState": {"stack"}, "RuleLhsWork": {"original_args", "original_head", "policy", "prefix", "resolved_head"}, "RulePatternSuspensionFrame": {"pending", "state"}}
ok
test tests::association_depth_contract::rule_pattern_depth_contract::rule_lhs_ownership_is_not_self_reaching ... ok
test tests::association_depth_contract::rule_pattern_depth_contract::rule_lhs_source_root_fields_are_complete ... ok
test tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:249:5:
self-reaching G4 state
ok
test tests::association_depth_contract::rule_pattern_depth_contract::structural_rule_pattern_depth_contract ... G4_SOURCE_CONTRACT 8 files, exact function sets, per-file/combined recursion and outbound audit passed
ok
test tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:162:5:
unaudited G4 outbound calls: {"drive -> moved_to_unscanned_helper"}
ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.37s
```

The fingerprint replay generated all 68 cells; both certificate replays intentionally failed on their stale installed C01 cell after earning zero-rejection marks. Exact emitted cells were installed, then both gates passed. No precondition, strength, disposition, exclusion or promotion changed.

`fingerprints-replay`:

```text
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/.temp/brief-573/verification-source/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 0.27s
```

`certificates-true-replay`:

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 18.55s
```

`certificates-false-replay`:

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1103 filtered out; finished in 20.86s
```

Required final gates, with actual result lines. `cargo fmt --check` exited 0 with no output. No required gate was omitted. The workspace command retains its actual exit; a loopback-only failure is followed by the supported focused skip replay below.

`cargo test -p rum-runtime -- --nocapture` — exit 0:

```text
test result: ok. 1118 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 31.44s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.98s
```

`cargo test -p rum-stdlib --lib ownership_graph -- --nocapture` — exit 0:

```text
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7632 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1090 filtered out; finished in 69.56s
```

`cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture` — exit 0:

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out; finished in 19.56s
```

`cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture` — exit 0:

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1103 filtered out; finished in 18.05s
```

`cargo test --workspace --no-fail-fast -- --nocapture` — exit 101:

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 303.85s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.37s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.06s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: FAILED. 142 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 1118 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 25.92s
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7632 rows
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1057 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1453.37s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.07s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.55s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.58s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.65s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Workspace harness totals (before the focused native skip replay):

```json
{
  "suites": 44,
  "passed": 3104,
  "failed": 9,
  "ignored": 58,
  "only_failed_target": "rum-jupyter --lib",
  "failures": "native ZeroMQ loopback PermissionDenied"
}
```

The workspace exit is 101 solely for nine existing native ZeroMQ loopback PermissionDenied cases, as in brief-572. This checkout requires `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`; it does not skip automatically without that flag. No adapter code or test contract was changed. The focused supported replay follows, with all nine early returns reported as **skips**, not successful native socket coverage:

```text
python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-573/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-573/target-verification env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter --lib zeromq::tests::native -- --nocapture
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s
```

Every ownership inventory row change follows; old row numbers refer to the captured supplied baseline. There are no removals.

`host_payloads.tsv`: 63 → 63; 0 existing moves, 0 additions, 0 removals.

`unknown_attributes.tsv`: 1460 → 1465; 3 existing moves, 5 additions, 0 removals.

Baseline row 394, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	191	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	144	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
```

Baseline row 395, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	212	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	165	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
```

Baseline row 396, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	233	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	186	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
```

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/continuation.rs	4	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/state.rs	3	1	unresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/state.rs	8	1	unresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/state.rs	17	1	unresolved attribute/expansion derive (Clone , Copy , rum_derive :: FrameRootTrace)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution/state.rs	24	1	unresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace)
```

`unknown_captures.tsv`: 2944 → 2944; 8 existing moves, 0 additions, 0 removals.

Baseline row 850, authorized matches_pattern_head relocation (historical row 852; supplied baseline row 850):

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	188	22	closure captures: |pattern_head| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs	176	22	closure captures: |pattern_head| (capture types/modes unresolved)
```

Baseline row 851, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	205	27	closure captures: || (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	158	27	closure captures: || (capture types/modes unresolved)
```

Baseline row 852, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	226	30	closure captures: || (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	179	30	closure captures: || (capture types/modes unresolved)
```

Baseline row 853, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	245	18	closure captures: || (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	198	18	closure captures: || (capture types/modes unresolved)
```

Baseline row 854, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	248	26	closure captures: |value| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	201	26	closure captures: |value| (capture types/modes unresolved)
```

Baseline row 1720, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs	40	17	closure captures: |symbol| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs	39	17	closure captures: |symbol| (capture types/modes unresolved)
```

Baseline row 1721, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs	41	22	closure captures: |symbol| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs	40	22	closure captures: |symbol| (capture types/modes unresolved)
```

Baseline row 1722, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs	139	14	closure captures: |arg| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs	138	14	closure captures: |arg| (capture types/modes unresolved)
```

`unknown_macros.tsv`: 3130 → 3135; 7 existing moves, 5 additions, 0 removals.

Baseline row 670, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	16	17	unexpanded macro format! (ends at 19:18)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	17	17	unexpanded macro format! (ends at 20:18)
```

Baseline row 671, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	25	21	unexpanded macro crate :: resume_or_suspend! (ends at 33:10)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	26	21	unexpanded macro crate :: resume_or_suspend! (ends at 34:10)
```

Baseline row 672, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	84	23	unexpanded macro crate :: resume_or_suspend! (ends at 92:10)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	60	23	unexpanded macro crate :: resume_or_suspend! (ends at 68:10)
```

Baseline row 673, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	107	9	unexpanded macro crate :: resume_or_suspend! (ends at 114:10)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	83	9	unexpanded macro crate :: resume_or_suspend! (ends at 90:10)
```

Baseline row 674, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	156	5	unexpanded macro matches! (ends at 159:6)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs	132	5	unexpanded macro matches! (ends at 135:6)
```

Baseline row 1732, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs	42	13	unexpanded macro matches! (ends at 52:14)
UnknownConstruct	crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs	41	13	unexpanded macro matches! (ends at 51:14)
```

Baseline row 1733, source position only:

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs	127	21	unexpanded macro vec! (ends at 127:41)
UnknownConstruct	crates/rum-runtime/src/runtime/substitution/function_and_subst_helpers.rs	126	21	unexpanded macro vec! (ends at 126:41)
```

```tsv
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs	103	9	unexpanded macro crate :: resume_or_suspend! (ends at 103:97)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs	113	8	unexpanded macro matches! (ends at 113:62)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs	121	44	unexpanded macro unreachable! (ends at 121:58)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs	124	13	unexpanded macro debug_assert_eq! (ends at 124:47)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs	155	24	unexpanded macro matches! (ends at 155:66)
```

`unknown_storage.tsv`: 88 → 88; 0 existing moves, 0 additions, 0 removals.

`self_reaching.tsv`: all 59 rows byte-identical. All 63 `host_payloads.tsv` rows and the host-payload ledger are unchanged.

All five allowed obligation evidence updates (every other cell, including N/Pending, unchanged):

`obligations.tsv:57`:

```tsv
N:Count	Count	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:124	evaluated target/value retained while pattern resolves; resolved_args matcher pattern vector	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:68; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:83; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57	rum_runtime::runtime::builtin_adapters::builtin_count
N:Count	Count	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:124	evaluated target/value retained while pattern resolves; resolved_args matcher pattern vector	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:75; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57	rum_runtime::runtime::builtin_adapters::builtin_count
```

`obligations.tsv:66`:

```tsv
N:DeleteCases	DeleteCases	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:146	evaluated target/value retained while pattern resolves; resolved_args matcher pattern vector	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:68; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:83; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57	rum_stdlib::rewrite_builtins::eval_delete_cases
N:DeleteCases	DeleteCases	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:146	evaluated target/value retained while pattern resolves; resolved_args matcher pattern vector	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:75; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57	rum_stdlib::rewrite_builtins::eval_delete_cases
```

`obligations.tsv:103`:

```tsv
N:FreeQ	FreeQ	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:221	evaluated target/value retained while pattern resolves; resolved_args matcher pattern vector	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:68; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:83; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57	rum_runtime::runtime::builtin_adapters::builtin_free_q
N:FreeQ	FreeQ	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:221	evaluated target/value retained while pattern resolves; resolved_args matcher pattern vector	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:75; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57	rum_runtime::runtime::builtin_adapters::builtin_free_q
```

`obligations.tsv:147`:

```tsv
N:MatchQ	MatchQ	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:336	evaluated target/value retained while pattern resolves; resolved_args matcher pattern vector	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:68; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:83; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57	rum_stdlib::rewrite_builtins::eval_match_q
N:MatchQ	MatchQ	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:336	evaluated target/value retained while pattern resolves; resolved_args matcher pattern vector	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:75; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57	rum_stdlib::rewrite_builtins::eval_match_q
```

`obligations.tsv:151`:

```tsv
N:MemberQ	MemberQ	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:346	evaluated target/value retained while pattern resolves; resolved_args matcher pattern vector	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:68; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:83; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57	rum_runtime::runtime::builtin_adapters::builtin_member_q
N:MemberQ	MemberQ	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:346	evaluated target/value retained while pattern resolves; resolved_args matcher pattern vector	crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:111; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:151; crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:191; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/pattern_resolution.rs:75; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/match_q.rs:59; crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:57	rum_runtime::runtime::builtin_adapters::builtin_member_q
```

Every replayed local-contract fingerprint row (row, ID, before, after; all other 18 cells unchanged):

```tsv
2	B01	fnv1a64:a2e887771d42718c	fnv1a64:5d209f30a83ba1ad
3	B02	fnv1a64:33c32b325169d331	fnv1a64:bd41ac4e29a8824c
4	B03	fnv1a64:7c607132cbccd402	fnv1a64:522ce940b328c6bf
5	B04	fnv1a64:e2c91a8f837e8c54	fnv1a64:6141da8898bb6b65
6	B05	fnv1a64:3e9df268e815524d	fnv1a64:28012223d2897000
7	B06	fnv1a64:cdee82559a270f40	fnv1a64:363cafb7a6dbd599
8	B07	fnv1a64:e2184b776a53f9fd	fnv1a64:7bdd1108d5826950
9	B08	fnv1a64:0da564fe0d77ad4e	fnv1a64:fb6abb37c0721f83
10	B09	fnv1a64:d1ee84622bb746e8	fnv1a64:01bc77e85d5e3f51
11	B10	fnv1a64:5aa51024431ebe87	fnv1a64:752d340634b949ae
12	B11	fnv1a64:a420cbec9d72b254	fnv1a64:22998be5b2af9165
13	B12	fnv1a64:90fb045e07bfdbe8	fnv1a64:c0c8f7e43966d451
14	I01	fnv1a64:80ca727489fa680c	fnv1a64:3b028a2e14f3982d
15	I02	fnv1a64:95df6ffce04bda4d	fnv1a64:7f429fb7cabff800
16	I03	fnv1a64:2ced2cfb7b194428	fnv1a64:1e83afabd5d5d611
17	I04	fnv1a64:a3938da00da764d1	fnv1a64:3828b1bd8e543eec
18	P01	fnv1a64:76a496323f8f47e0	fnv1a64:6b7ea1b48f0870b9
19	P02	fnv1a64:e2dd098a99caf357	fnv1a64:cda9983f19af249e
20	P03	fnv1a64:44d204247c75727a	fnv1a64:a8bd953d47a5dba7
21	P04	fnv1a64:289e4947d612ef31	fnv1a64:b21cca63ae519e4c
22	P05	fnv1a64:2bf40d57dc88cb9c	fnv1a64:d0a68d92d0e3aa9d
23	P06	fnv1a64:e17a292e93ef7568	fnv1a64:11481cb4c5966dd1
24	P070	fnv1a64:1bc7c5707264ef68	fnv1a64:4b95b8f6a40be7d1
25	P080	fnv1a64:4a84e8da1e858dc7	fnv1a64:2eba9eacd903446e
26	P090	fnv1a64:1a5d03f7c6191771	fnv1a64:aa18c5bd3b4adb0c
27	P100	fnv1a64:2f519bce401cdd04	fnv1a64:499fd706895239f5
28	P110	fnv1a64:1654baba27e8a97f	fnv1a64:86002ce92ae49c56
29	P120	fnv1a64:e78ae38eb8dee5ee	fnv1a64:d4817badde7572a3
30	P130	fnv1a64:85e5a4d49907d3fd	fnv1a64:1faa6a6604364350
31	P140	fnv1a64:5ab9d272fad50bc9	fnv1a64:05197670c2d3aad4
32	P150	fnv1a64:4dc37d0d327b2111	fnv1a64:17608d39642fdaac
33	P071	fnv1a64:3be1fe8fc79c0d9d	fnv1a64:3b8903138e9d12f0
34	P081	fnv1a64:db3c9965a11cac88	fnv1a64:71e97c33842551f1
35	P091	fnv1a64:95eb4cc5bb7d4f20	fnv1a64:4c49e40d000bb479
36	P101	fnv1a64:0e57ab875d4c467f	fnv1a64:7e031db660483956
37	P111	fnv1a64:ac039d501e906658	fnv1a64:14f5c00de502eca1
38	P121	fnv1a64:014331899024c71b	fnv1a64:d2bcd50a6a5993da
39	P131	fnv1a64:80733cf5b9d2add4	fnv1a64:feebfceecf0f8ce5
40	P141	fnv1a64:00742b43a919a280	fnv1a64:c41117589f8d0759
41	P151	fnv1a64:d5f019d399ce58ae	fnv1a64:80a5cb73a44c04e3
42	B13	fnv1a64:5bff15decac3cb40	fnv1a64:c44d4340d7789199
43	R01	fnv1a64:95e44740e2763fa1	fnv1a64:9c039952e08cb99c
44	R02	fnv1a64:dc21fcb4cce432ba	fnv1a64:2804020d8afe0a67
45	R03	fnv1a64:7ecdefef3fc5b5c5	fnv1a64:e3777e76d52f80c8
46	R040	fnv1a64:586f9178b09d48c1	fnv1a64:5005d0c58ae7a63c
47	R050	fnv1a64:f6fc32e7b731f7fc	fnv1a64:aeb742162492017d
48	R060	fnv1a64:718d318985930f39	fnv1a64:def7a1c0e3a87224
49	R070	fnv1a64:43aa617f82be6b88	fnv1a64:da57444d65c710f1
50	R080	fnv1a64:6a97f692dd2c2dd1	fnv1a64:ff2d1ab05dd907ec
51	R090	fnv1a64:02b1d701abd37144	fnv1a64:710d43cf80452bb5
52	R100	fnv1a64:4186158575a27e82	fnv1a64:17528d935cfe713f
53	R110	fnv1a64:7da095b9733c75ef	fnv1a64:b82c7ce889193aa6
54	R041	fnv1a64:42387908e1eb8a40	fnv1a64:aa86a66aeea05099
55	R051	fnv1a64:a87f320d534723f1	fnv1a64:383af3d2c878e78c
56	R061	fnv1a64:ae33c99a6d01eb5e	fnv1a64:879e75a2c3ef56f3
57	R071	fnv1a64:02c776a7f0637ac3	fnv1a64:d829cb5bb27c9312
58	R081	fnv1a64:eb3c0522dbc8b204	fnv1a64:058a405b24fe0ef5
59	R091	fnv1a64:39b074f4eeda1689	fnv1a64:2650ff71d1f19614
60	R101	fnv1a64:26078f7bf8bcf879	fnv1a64:b0e59df6799d56e4
61	R111	fnv1a64:fbe743c38519f1b6	fnv1a64:8fd6669132fe323b
62	R12	fnv1a64:b6b9d5aaf7bb2076	fnv1a64:de955d010aa0a27b
63	R13	fnv1a64:1dfc8ace3a7189d2	fnv1a64:e2cb8e6bed3a73ef
64	R14	fnv1a64:49a9aa441ccd9d04	fnv1a64:63f7e57c6602f9f5
65	R15	fnv1a64:93f186f8e099d6d0	fnv1a64:f44a9ffd524c2cc9
66	O010	fnv1a64:4614cfac36b82d3e	fnv1a64:25a8482731c4fe53
67	O0200	fnv1a64:55817447498c31c9	fnv1a64:ffe11845118ad0d4
68	O011	fnv1a64:b415ee119324d3b9	fnv1a64:21805e48f13a36a4
69	O0210	fnv1a64:c6cebb2f2694b156	fnv1a64:ac032fbda06107db
```

Every promoted-certificate cell change is listed by row, feature and column below. Full before/after cell bytes are saved in `certificate-cell-changes-{true,false}.json`; installed cells equal the emitted replay cells. Only fingerprint tokens differ; this is no retention reclassification.

```tsv
2	C01	true	column=5	before_sha256=1b75d6931ce8827c23c6df70f748d102d6f4a3e838dffbb76227ea060e77a9b7	after_sha256=83a981d662e7bb1d6cb89b2c62fe11b77084a11a9db82e9749dd82214db96469
3	C02	true	column=5	before_sha256=46c4b7afdf97500c8b0b99f7f3eb0e22605f3948ad45fa2b0dd7343925500451	after_sha256=90e50f2d675679d8543e9ad96cd9eb3846eb4ef1fb6c913646716a5a9dada4c8
2	C01	false	column=6	before_sha256=fef8342c9c2da4252164f24ab7b1cb190e94a773314d58e0c5d0d0bf3e17e3ad	after_sha256=12d1af1e223f275fa094502df71cef3baf5f28f833224df8f477c5711231ecc5
3	C02	false	column=6	before_sha256=c0a3629d49b5ac5d859a90a979576039f6a2b368dae813a8ff3f6d4f9e9b90e3	after_sha256=789c28e5bcfde569dbad6a53252ae435704eea26d7e7729e695ae315893b3b71
```

### 3. Repaired sites and one-candidate measurements

P0774 (root completion/normalization), P0776 (head continuation), and P0777 (argument continuation) are repaired. The frozen-census arithmetic is **46 → 43 predicted**, subject to the director’s fresh census. G4 closes with thirteen live repairs and three obsolete-adapter retirements across A/B/C; this does not certify global matcher depth safety.

One candidate was built with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and `--features rum-cli/alloc-count` as two arguments, each bounded at 900 seconds. Both director-supplied base binary digests were checked before and after. Base `post-B` is `.temp/brief-573/base/` (director’s d2e0641c); base `pre-G4` is `.temp/brief-572/base/`.

`candidate-01-build-default`:

```text
    Finished `release` profile [optimized] target(s) in 1m 48s
```

`candidate-01-build-alloc`:

```text
    Finished `release` profile [optimized] target(s) in 1m 48s
```

```json
[
  {
    "binary": "rum-cli",
    "sha256": "51fafd069ec8acf1d901a563bbf2c3d4e3a5724d34473690e88c4ebb6acc9bc0",
    "base_sha256": "fcf18d24c323c3e389615a9657a35c6f843c1d5d58c3db7d66861738fae0b137",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-573/candidate-01/target-default"
  },
  {
    "binary": "rum-cli-alloc",
    "sha256": "6c2a610af49637aec7dd6a248f46f8cc66e0daed8597b1be2b77bf12840d42c2",
    "base_sha256": "dbc0dc6dcc3be5d5382882b56cf702766135313968f724c1f81129ca733e5cd9",
    "codegen_units": 1,
    "target": "/home/remi/dev/rum/.temp/brief-573/candidate-01/target-alloc"
  }
]
```

Panel: all 17 brief-572 labels plus six held matcher labels (active child/head, Cases active child, prepared call, and 8/32 inline outer h wrappers around the active leaf). The Cases child alias intentionally uses the same statement as the existing `cases-pattern-child` neighbor. Statements and N/2N/check programs are in `perf/labels.json` and `perf/*.rum`, with source hashes recorded. All 69 label/arm result checks passed before measurement.

Allocation counts ran first, then two paired instruction samples. Each sample measures all three arms on the same files, sharing that sample’s candidate counts for the two base comparisons; sample 1 order is pre-G4/post-B/candidate and sample 2 reverses it. Every metric is one label per process, stdin `/dev/null`, instructions via `taskset -c 2 perf stat -x, -e cpu_core/instructions/u`. Slopes are `(count(2N) - count(N)) / N`. All 414 metric processes exited successfully and returned Null; RUM_PROFILE variables were cleared for instruction runs, with RUM_PROFILE_ALLOC enabled only for allocation runs. No CPU-heavy workload ran concurrently and no timing suite ran, as this brief specifies. Raw counters/stdout/stderr/commands are preserved under `perf/panels/` and `perf/paired-results.json`.

Every measured label, both samples, allocations and instruction deltas follows verbatim from the reducers. No noise threshold is assumed: every positive sample is marked, including tiny or mixed-sign deltas; increased count or bytes is also marked. These regressions are reported defects, not an accepted performance gate.

Against `post-B`:

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	positive_samples	allocation_regression	regression
g4-rule-plain	20000	0.0	0.0	32.0	32.0	998.87575	999.12015	0.024467507595415938	999.0867	999.03315	-0.005359895192280639	1	False	True
g4-ruledelayed-plain	20000	0.0	0.0	32.0	32.0	999.03515	999.0532	0.0018067432361990043	998.7538	999.0976	0.0344228978152783	1,2	False	True
g4-rule-pattern-child	20000	17.0	17.0	1816.0	1816.0	21421.08605	21439.1115	0.08414816110595691	21420.87345	21440.34485	0.09089918786668871	1,2	False	True
g4-ruledelayed-pattern-child	20000	17.0	17.0	1816.0	1816.0	21124.64955	21140.58485	0.07543462419239422	21128.2853	21143.85785	0.07370475066426074	1,2	False	True
g4-rule-blank-constraint	20000	18.0	18.0	1816.0	1816.0	18252.6	18253.8751	0.006985854070129527	18252.88175	18251.85125	-0.00564568386578479	1	False	True
g4-rule-wrapper	20000	29.0	29.0	2296.0	2296.0	24227.0026	24231.6763	0.019291284510769557	24227.07625	24226.894	-0.0007522575077412164	1	False	True
g4-rule-skew-depth-8	1000	69.0	69.0	9048.0	9048.0	62609.354	62610.957	0.0025603202997492858	62599.301	62609.846	0.016845236019480403	1,2	False	True
g4-rule-skew-depth-32	1000	217.0	217.0	36312.0	36312.0	212343.508	212348.368	0.002288744330236625	212345.074	212349.534	0.002100354821510386	1,2	False	True
matchq-integer-success	20000	1.0	1.0	160.0	160.0	5144.29075	5286.1969	2.7585172941478886	5144.1705	5286.4096	2.765054152073687	1,2	False	True
matchq-integer-fail	20000	2.0	2.0	161.0	161.0	5266.16395	5416.34915	2.8518899416338783	5266.2278	5416.20095	2.8478287627436316	1,2	False	True
cases-wide-no-match	1000	33.001	33.001	263279.864	263279.864	2412470.903	2412677.332	0.008556745689380918	2412445.419	2412556.542	0.0046062389277068405	1,2	False	True
cases-pattern-child	20000	35.0	36.0	5744.0	6160.0	39503.0584	40482.5838	2.4796191476657903	39501.835	40384.82815	2.235321852769623	1,2	True	True
downvalue-definition	20000	29.0	29.0	3172.0	3172.0	24119.88775	24209.7225	0.37245094558948644	24149.8927	24149.99725	0.0004329211781506359	1,2	False	True
replaceall-simple	20000	5.0	5.0	1352.0	1352.0	11433.4594	11396.9624	-0.31921222372993663	11446.66895	11370.8256	-0.6625800949716432	none	False	False
dot-three-no-input	20000	596.0	596.0	23504.0	23504.0	254197.1608	254203.98165	0.0026832911817376015	254200.2128	254204.59365	0.0017233856540510573	1,2	False	True
differences-no-input	20000	13.0	13.0	704.0	704.0	10037.83245	10033.774	-0.04043153758758189	10052.0698	10034.1966	-0.17780616684536232	none	False	False
take-multispec-small	20000	109.0	109.0	5856.0	5856.0	49966.50635	49949.4016	-0.0342324313815201	49966.6605	49949.1167	-0.03511101167147368	none	False	False
g4-matchq-pattern-child	20000	16.0	17.0	2428.0	2844.0	22083.55745	22969.20245	4.010427223988766	22083.47635	22969.14255	4.010537951376492	1,2	True	True
g4-matchq-pattern-head	20000	16.0	18.0	2364.0	2844.0	21173.6637	22064.7807	4.208610340779129	21173.3792	22064.774	4.209978915410928	1,2	True	True
g4-cases-pattern-child	20000	35.0	36.0	5744.0	6160.0	39415.8028	40382.4035	2.4523176780253397	39479.58995	40387.0382	2.2985250129225365	1,2	True	True
g4-matchq-prepared-call	20000	8.0	9.0	488.0	904.0	15268.34135	16019.32785	4.918585999519842	15268.4737	16019.27135	4.917306501959007	1,2	True	True
g4-matchq-skew-depth-8	1000	60.0	63.0	7312.0	10224.0	52480.477	55771.503	6.2709529107366935	52480.607	55775.182	6.277699874927123	1,2	True	True
g4-matchq-skew-depth-32	1000	222.0	227.0	30384.0	43280.0	204433.694	203199.08	-0.6039190389036331	204432.391	203201.29	-0.60220447159961	none	True	True
```

Against `pre-G4`:

```tsv
label	N	base_count	candidate_count	base_bytes_total	candidate_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	positive_samples	allocation_regression	regression
g4-rule-plain	20000	0.0	0.0	32.0	32.0	999.0304	999.12015	0.008983710605803275	998.9844	999.03315	0.0048799560833812095	1,2	False	True
g4-ruledelayed-plain	20000	0.0	0.0	32.0	32.0	999.02625	999.0532	0.0026976268141076787	999.0416	999.0976	0.005605372188699498	1,2	False	True
g4-rule-pattern-child	20000	25.0	17.0	1400.0	1816.0	22079.848	21439.1115	-2.9019062993549727	22076.228	21440.34485	-2.8803976385820884	none	True	True
g4-ruledelayed-pattern-child	20000	25.0	17.0	1400.0	1816.0	21785.593	21140.58485	-2.9607096304424707	21785.52215	21143.85785	-2.945370304103545	none	True	True
g4-rule-blank-constraint	20000	16.0	18.0	1112.0	1816.0	17184.57615	18253.8751	6.2224342379256115	17185.5421	18251.85125	6.204687310969392	1,2	True	True
g4-rule-wrapper	20000	47.0	29.0	2488.0	2296.0	26310.373	24231.6763	-7.9006736240493485	26310.35215	24226.894	-7.918777134269561	none	False	False
g4-rule-skew-depth-8	1000	73.0	69.0	3256.0	9048.0	59731.198	62610.957	4.821197458654702	59718.538	62609.846	4.841558579347671	1,2	True	True
g4-rule-skew-depth-32	1000	217.0	217.0	9016.0	36312.0	217261.915	212348.368	-2.261577690687311	217268.336	212349.534	-2.2639295216952404	none	True	True
matchq-integer-success	20000	1.0	1.0	160.0	160.0	5144.2989	5286.1969	2.7583544960810835	5144.147	5286.4096	2.765523613535925	1,2	False	True
matchq-integer-fail	20000	2.0	2.0	161.0	161.0	5266.09745	5416.34915	2.8531887498587816	5266.3304	5416.20095	2.8458250549566744	1,2	False	True
cases-wide-no-match	1000	33.001	33.001	263279.864	263279.864	2412418.716	2412677.332	0.010720195390812037	2412443.473	2412556.542	0.004686907745821145	1,2	False	True
cases-pattern-child	20000	35.0	36.0	5744.0	6160.0	39489.75505	40482.5838	2.514142588990298	39456.6238	40384.82815	2.352467749660825	1,2	True	True
downvalue-definition	20000	29.0	29.0	3172.0	3172.0	24122.0	24209.7225	0.363661802503934	24122.1001	24149.99725	0.11564975638254094	1,2	False	True
replaceall-simple	20000	5.0	5.0	1352.0	1352.0	11407.72345	11396.9624	-0.09433126641932299	11407.6313	11370.8256	-0.32264103767097874	none	False	False
dot-three-no-input	20000	596.0	596.0	23504.0	23504.0	254199.87085	254203.98165	0.00161715267055218	254196.99075	254204.59365	0.0029909480743883776	1,2	False	True
differences-no-input	20000	13.0	13.0	704.0	704.0	10045.1549	10033.774	-0.1132974066930581	10052.074	10034.1966	-0.17784787497586718	none	False	False
take-multispec-small	20000	109.0	109.0	5856.0	5856.0	49950.74895	49949.4016	-0.002697356953251795	49949.9575	49949.1167	-0.001683284715503497	none	False	False
g4-matchq-pattern-child	20000	16.0	17.0	2428.0	2844.0	22083.58795	22969.20245	4.010283573507811	22083.5278	22969.14255	4.010295628581595	1,2	True	True
g4-matchq-pattern-head	20000	16.0	18.0	2364.0	2844.0	21173.80865	22064.7807	4.207896957640633	21173.64115	22064.774	4.208689680187594	1,2	True	True
g4-cases-pattern-child	20000	35.0	36.0	5744.0	6160.0	39428.9816	40382.4035	2.418073866762005	39481.60905	40387.0382	2.2932934391132864	1,2	True	True
g4-matchq-prepared-call	20000	8.0	9.0	488.0	904.0	15268.4557	16019.32785	4.9178002330648285	15268.5196	16019.27135	4.916991101088808	1,2	True	True
g4-matchq-skew-depth-8	1000	60.0	63.0	7312.0	10224.0	52481.636	55771.503	6.268606032022328	52480.645	55775.182	6.27762292174574	1,2	True	True
g4-matchq-skew-depth-32	1000	222.0	227.0	30384.0	43280.0	204427.293	203199.08	-0.600806762138173	204428.71	203201.29	-0.6004146873499261	none	True	True
```

Measured residual highlights: against post-B, MatchQ integer success is +2.7585%/+2.7651%, integer failure +2.8519%/+2.8478%, active child +4.0104%/+4.0105%, active head +4.2086%/+4.2100%, prepared call +4.9186%/+4.9173%, and held depth 8 +6.2710%/+6.2777%. Depth 32 instructions fall about 0.60%, while allocations rise 222 → 227 and bytes 30,384 → 43,280. Against pre-G4, the prior Rule blank-constraint and depth-8 costs remain +6.2224%/+6.2047% and +4.8212%/+4.8416%. The full tables retain all smaller and mixed-sign neighbor deltas as well. No attribution or tuning was performed.

### 4. Stop-loss counters, residuals and open questions

```json
{
  "production_candidates": 1,
  "measured_candidates": 1,
  "remaining_target_sites": 0,
  "predicted_global_lost": 43,
  "semantic_target_failures": 0,
  "depth_target_failures": 0,
  "structural_failures": 0,
  "no_progress_streak": 0,
  "performance": {
    "post-B": {
      "labels": 23,
      "regressions": 20,
      "instruction_any_positive": 19,
      "instruction_both_positive": 16,
      "allocation_regressions": 7,
      "worst_positive_delta": 6.277699874927123,
      "worst_both_sample_delta": 6.2709529107366935
    },
    "pre-G4": {
      "labels": 23,
      "regressions": 19,
      "instruction_any_positive": 15,
      "instruction_both_positive": 15,
      "allocation_regressions": 12,
      "worst_positive_delta": 6.27762292174574,
      "worst_both_sample_delta": 6.268606032022328
    }
  },
  "tuning": "none; expressly deferred to next brief"
}
```

The first implementation repaired all three target sites and all semantic/depth failures. One production candidate and one measured candidate; zero compiler-error repair iterations and zero no-progress streak. One workspace run. No tuning candidate, attribution rerun, best-sample selection or timing substitution. All targeted semantic, structural, ownership and retention gates passed. The broad workspace run completed with only the nine existing loopback-denied native ZeroMQ failures; the supported focused replay verified nine skips. The original workspace failure is not relabeled green. Expected red-baseline and stale-certificate failures are retained above.

Ordinary-path instruction/allocation regressions against both bases remain for the next brief’s combined G4 cost work. The provided per-label tables, including neighbors and mixed-sign positives, are the residual record; performance is not green. The director must reconcile the predicted global lost-site count with the fresh census. Separate matcher/compiler/Expr depth boundaries remain unchanged and are not claimed repaired. No new adjacent semantic defect was observed. The existing native transport skip-flag requirement is an adjacent environment limitation, recorded with its real failed and skipped output.

Scene: all changes are uncommitted. `.temp/brief-573/PROGRESS.md` contains checkpoint history; `final-source.patch`, `final-source-audit.json`, exact logs, inventory row/cell disclosures and measurement files contain the reviewable evidence. No git command was run.
