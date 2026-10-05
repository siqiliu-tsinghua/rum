# brief-572 — G4 step B: a flat, resumable rule left-hand-side walk

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

`Rule` and `RuleDelayed` prepare their left-hand side by walking it: ordinary
subexpressions are evaluated, pattern constructs stay held
(`crates/rum-runtime/src/runtime/definitions/definitions.rs:80-235`,
`evaluate_rule_lhs` / `evaluate_pattern_rule_lhs`, and the classifier
`definition_lhs_preparation/syntax_policy.rs:111`). A subexpression that asks
for input resumes past the walk and past the rule's completion — `Rule-pattern`
resumes to `"two"` where the queued path returns
`Rule[h[Pattern[x, Blank[]], "two", 3], 9]`. The walk and its classifier are
also guest-capable recursions with no guard. Replace them with brief-570's
design: an explicit work stack that is the suspension state, and completion
frames for both rule heads. Ten live sites, P0342, P0344, P0350-P0357: lost
continuations **56 -> 46**.

brief-570's report and `.temp/brief-570/draft-B-rule-lhs-continuations.md`
are the raw material; **this brief governs** where they differ (performance
gate, depth calibration).

## Context pointers

- brief-570 and its report; `.temp/brief-570/design.md`, `recursion.md`,
  `inventory-plan.md` (section B), `red-tests.md`, `B-rule-cases.json`,
  `probe.rs`, `post-fix-depth.md`, `post-fix-depth.rs`.
- The accepted pattern: briefs 563 and 568 (flat walk, immutable Session-rooted
  frame, cold working copy on an actual resume, structural depth contract with
  falsifiers — `tests/association_depth_contract/{lazy,schema}_depth_contract.rs`);
  566 and 569 for one-unit measurement.
- `docs/guide/patterns-and-rules.md`, `docs/dev/codex/TEMPLATE.md`.

## Scope

**Objective:** the flat rule walk and classifier, the rule completion frames,
the owning tests, the structural and private depth checks, one narrow guide
clarification, and the standing and performance gates.

**Edit surface:** `definitions/definitions.rs` (the rule entry completions;
removal of the old walk); `definitions/mod.rs` (module declaration); new
`definitions/rule_lhs.rs` and `definitions/rule_lhs/{state,continuation,tests}.rs`;
`definition_lhs_preparation/syntax_policy.rs` (make
`definition_lhs_contains_pattern_syntax` iterative, nothing else); new
`crates/rum-stdlib/tests/rule_pattern_continuations.rs` and children;
`crates/rum-stdlib/src/tests/association_depth_contract/{mod,recursion_inventory}.rs`
and a new `rule_pattern_depth_contract.rs` (scanner extensions only as far as
the falsifiers below need; the lazy and schema checks stay green); rum-runtime's
frame lists (exactly the seven new derive entries and two frames, named-frame
count **556 -> 558**); `docs/guide/patterns-and-rules.md` (one clarification:
rule left-hand-side work resumes after input; binder, test and delayed
right-hand-side positions stay held — no new semantics).

**Inventory exceptions, authorized by name** (`inventory-plan.md` §B): remove
`unknown_captures.tsv` rows 403 and 404 (the closures at `definitions.rs:187`
and `:217`) and `unknown_macros.tsv` rows 288-290 (the `vec!` at `:180`,
`:198`, `:207`) **only if the construct is actually gone**; relocate rows 283
(`:87`) and 284 (`:111`) keeping `UnknownConstruct`. `N:Rule`,
`N:RuleDelayed` and the shifted `N:NormalizeRules` keep class and status; only
evidence positions move.

**Standing duty — always in scope** (`TEMPLATE.md`). **STOP** on any other
row change or disappearance, any new `UnknownHostPayload` or `self_reaching`
row, any new row in untouched code, or any retention reclassification.

**Fence:** no change to `Expr`, the matcher (compiler, engine, normalizer,
codecs), the held-pattern resolver (step C), the request lifecycle, manifests,
the catalog, other docs. No `NativeStackDepth` in a frame, no recursive
fallback, no depth cap.

**Disclosure duty:** every changed function and moved item, every state field
and root, every inventory row change, every measured label, and any adjacent
defect.

## Deliverable

brief-570 design.md §3 for the rule walk: Classify / Structural / EvaluateOnly
visits; `Pattern`'s inner part, `Condition` and `PatternTest` first operands
and wrapper arguments descend structurally even when they contain no pattern
syntax; `Blank` constraints evaluate; binder, test and delayed right-hand-side
positions stay held; reconstructed ordinary calls are not applied; malformed
arities and the `OptionsPattern` / `Verbatim` fallbacks are preserved; Set's
left-hand-side semantics are not imported. Capture only on `InputRequired` on
the error edge; a cold working copy on an actual `Ok` resume; the answer
applied once and the cursor advanced once; the immediate right-hand side
evaluated once after the completed left-hand side (through the existing
`EvalRuleSuspensionFrame`), the delayed one held; a second suspension freezes
the new state; abandonment runs nothing pending.

## Tests

Red first, recorded before any production edit: the 27 rule witnesses of
`B-rule-cases.json` through the rum-vs-rum oracle (value, error, termination,
messages, prints, `log`, actual suspension, exact prompts, no owner after,
`1 + 1`), the twice variants, one child with two inputs, left-hand side then
right-hand side, a nested pattern, held positions, an ordinary error after a
resume, cancellation at the first and second prompt, answer-free abandonment;
and a control that a structural child with no pattern syntax is rebuilt, not
evaluated whole.

**Structural depth contract** (`rule_pattern_depth_contract.rs`), each check
with a falsifier seen to fire: no recursive edge in the new modules, per file
and combined, and none in the shared classifier; outbound calls audited;
falsifiers for direct, cross-module and callback-hidden recursion, an
unscanned helper, a missing file or function, a self-reaching state type and a
missing root.

**Private depth tests** (`post-fix-depth.md`): a ruled 2 MiB thread that owns
construction and teardown, the walker driven directly on a Rust-built
left-hand side at **n = 2 000**, ordinary, one and two suspensions, an error
and abandonment; and 20 000 scalar work records through the state's copy,
trace, drop and abandonment. **Run the construction-and-teardown control
first**; if it does not pass at 2 000 in the test profile, STOP and report the
measured ceiling — never lower n silently. Run no probe that aborts a process.

## rum / Rust pitfalls (read twice)

1. A structural child with no pattern syntax must not be reclassified and
   evaluated whole.
2. Held binder, test and delayed right-hand-side positions are never reached
   by evaluation; fixtures that place prompts there must keep failing to reach
   them.
3. Loops for the classifier and the driver; a native guard returning
   `StackLimitExceeded` does not meet this contract.
4. One change per measured candidate; stop-loss keyed on progress (failing
   tests, remaining sites, regressing labels, worst regression); two
   consecutive compiling candidates that move none of them stop the work.
5. **Both performance arms use one codegen unit** (`TEMPLATE.md` pitfall 8).

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-572/tmp …`. Bounds: focused 150 s;
`cargo test -p rum-runtime`, `ownership_graph`, `local_production_acceptance`
(either feature set) 300 s; a one-unit release build 900 s (observed ~100 s);
a metric process 150 s; `cargo test --workspace --no-fail-fast` 2 400 s.

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

**Performance gate.** Base: `.temp/brief-572/base/rum-cli` and
`rum-cli-alloc`, the director's one-unit build of `bb06a06a` (hashes in
`sha256.txt`); do not rebuild them. Candidates: the working tree with
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and `--features
rum-cli/alloc-count` (pass `--features` and its value as **two** arguments),
into a fresh target directory each; check the hash changed. Panel
(`performance-plan.md`): new no-input rule labels — plain, delayed, pattern
child, `Blank`, wrapper, inline-pattern depth 8 and 32 (the pattern written
inline, not behind a symbol, so the walk runs) — plus the neighbours
`matchq-integer-*`, `cases-*`, `replaceall-simple`, a downvalue definition
label, and the controls `dot-three-no-input`, `differences-no-input`,
`take-multispec-small` (`.temp/brief-563/perf/`). Scratch workloads, each
result checked first. Allocation counts, then two paired samples of `taskset -c
2 perf stat -x, -e cpu_core/instructions/u`, one label per process, stdin
`/dev/null`, `(count(2N) - count(N)) / N`, raw output saved before parsing. A
label regresses when **both** samples exceed +0.05%. No timing suite. At most
eight measured candidates; report every residual with its attribution.

## Checkpointing — protect the scene

Everything under `.temp/brief-572/`; `PROGRESS.md` after the red baseline,
each compiled attempt and each gate: evidence, compile state, remaining sites,
failing tests, performance counters, exact next action.

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

1. Changed functions, state fields and roots, holding policy, file sizes, the
   guide line.
2. Verbatim red and green output, structural checks and falsifiers, depth
   tests, the broad suites; every inventory row change.
3. Ten sites repaired (56 -> 46 predicted); performance rows for base and
   every candidate, both samples.
4. Stop-loss counters, residuals with attribution, open questions.

---

## Execution report

The flat Rule/RuleDelayed LHS continuation repair is implemented. The required
semantic, depth and standing checks are recorded below. **The performance gate
fails on four final labels; this is not an accepted performance result.** No
`git` command was run or external network access used. No unsafe code was
added; scratch stayed under `.temp/brief-572/`. The native tests' denied loopback
attempts are disclosed below. Changes are left uncommitted for the director's gate.

### 1. Changed functions, state, roots and holding policy

`crates/rum-runtime/src/runtime/definitions/definitions.rs`:

- `Session::eval_rule` (79) and `eval_rule_delayed` (93) capture the new
  `RuleLhsCompletionFrame` only when their LHS returns `InputRequired`.
- New `finish_rule_lhs` (110) holds the delayed RHS, or evaluates the immediate
  RHS once using the existing `EvalRuleSuspensionFrame`. The old immediate RHS
  capture and delayed rule construction moved here. `resume_rule_with_rhs`
  retains its behavior and body at 176.
- The old `evaluate_rule_lhs` and recursive `evaluate_pattern_rule_lhs` were
  removed from this file. Their classification/walk responsibility moves to
  `rule_lhs.rs`; no recursive fallback remains. Policy helper imports move
  with that responsibility. Other definitions only shift source positions.

`definitions/rule_lhs.rs`: `evaluate_rule_lhs` (30) keeps the direct evaluator
path for a root with no pattern syntax; `drive_rule_lhs` (40) loops over local
`RuleLhsVisit::{Classify,Structural,EvaluateOnly}` and `RuleLhsStep` values.
`suspend_rule_lhs` (105) captures on the input-required error edge only.
`rule_lhs_inert_pattern` (122) shares fully held HoldPattern calls and inert
empty blank/named-empty-blank leaves. `rule_lhs_policy` (141) chooses the old
arity/holding policy, `rule_lhs_child` (158) chooses the next visit, and
`accept_rule_lhs_value` (168) iteratively delivers one value, advances one cursor,
and rebuilds completed parents. Only this last helper gained `inline(always)`
in candidate 04.

`definitions/rule_lhs/state.rs`: `RuleLhsState::destination` (41) derives the
pending slot from the last work record; `pending_expr` (52) borrows its original
expression for capture on suspension. A root evaluator suspension bypasses this
walk and is completed by the outer rule frame.

`definitions/rule_lhs/continuation.rs`: both new frame `resume` methods
(11, 47) propagate an error without copying state or running pending work.
`resume_rule_pattern` (24) calls the cold `rule_lhs_working_copy` (36) only on
an actual successful resume, delivers the answer once, and returns to the flat
driver. Another input request freezes the updated state in a new frame.
Both frames derive pure abandonment; abandonment performs no remaining guest work.

`definition_lhs_preparation/syntax_policy.rs` changes only
`definition_lhs_contains_pattern_syntax` (111): its borrowed iterator loop
keeps sibling iterators only across outstanding branches, checks a callable
head before arguments, and handles unary tails without a scan allocation.
The syntax-plan, blank-head and wrapper-head functions retain their bodies.
`definitions/mod.rs` adds only the module declaration.

Every retained field/root:

| Type | Fields and tracing |
| --- | --- |
| `RuleLhsState` | `stack: Vec<RuleLhsWork>`; traces each work record. |
| `RuleLhsWork` | `original_head: Expr`, `original_args: SharedArgs`, `resolved_head: Option<Expr>`, `prefix: Vec<Expr>` are traced roots. Original arguments retain binders, held tests, the pending child and unvisited siblings. `next: usize` and `policy: RuleLhsPolicy` are scalar control. |
| `RuleLhsPolicy` | `NamedInner`, `BlankArguments`, `FirstOperand`, `WrapperArguments`, `OrdinaryCall`; no expression payload. |
| `RuleLhsDestination` | `Root`, `Head`, `Argument(usize)`; no expression payload. |
| `RuleLhsAwait` | `destination` is scalar control; `pending_expr: Expr` is traced. |
| `RulePatternSuspensionFrame` | `state` and `pending` are traced transitively. |
| `RuleLhsCompletionFrame` | `rhs: Expr` is traced; `delayed: bool` is scalar control. |

The local visit/step enums are not retained in suspension frames. No frame
stores a Session borrow, cancellation observer, native stack guard or recursive
work link. Runtime's closed derive list gains exactly the seven types above;
the pure-abandon list gains exactly the two frames. The named-frame assertion
in `suspension_protocol_repository_invariants_hold` changes 556 to 558.

Holding remains positional: Pattern's binder is copied raw while its inner
operand is structural; Blank constraints use EvaluateOnly; Condition and
PatternTest descend structurally into operand one and copy operand two raw;
wrapper operands are structural even without nested pattern syntax. Ordinary
heads/children classify, and rebuilt ordinary calls are never applied.
Malformed arities and OptionsPattern/Verbatim retain the ordinary fallback.
HoldPattern stays entirely held. No Set LHS preparation policy was imported.

The stdlib integration root and its `rule`, `support`, `lifecycle`, and `policy`
children add the 27 required rum-vs-rum witnesses, four cancellations,
answer-free abandonment, the non-applying structural-child controls, held
binder and construction control. The helper compares value, error, termination,
messages, prints and `log`, checks real suspension/exact prompts, retired owner,
and subsequent `1+1`. Private runtime tests own the 2 MiB/2,000 depth walk,
root sentinels, and 20,000 scalar-state copy/trace/drop/abandonment checks.
Their six diagnostic writes use stderr, following the final hygiene failure
and repair described below; assertions and production code were unchanged.

The opt-in scanner extension in `association_depth_contract/recursion_inventory.rs`
adds `flat_walk_call_graph`, `FlatCalls::{resolve,visit_expr_call,
visit_expr_method_call,visit_expr_path,visit_macro}` and
`flat_walk_recursive_edges`. Existing association/lazy/schema scanning stays
unchanged. `rule_pattern_depth_contract.rs` specifies the four audited source
files and exact function sets, resolves unique function/method names and
function-item/alias/closure callbacks, checks per-file and combined cycles,
and audits every outbound name/macro. Its self-reaching and root-field checks
have separate falsifiers. This is an explicit source boundary, not a claim of
arbitrary Rust trait/type or macro-expansion proof. Evaluator reentry, shared
Expr operations and existing completion remain named outbound boundaries.
The association-depth module gains one declaration.

Relocated/removed rule items and unchanged functions whose declaration positions
shifted in the two production files (old → new; `removed` means the recursive
helper was replaced, not retained as a fallback):

| File | Item | Before line | After line/location |
| --- | --- | --- | --- |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | eval_normalize_rules | 12 | 11 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | eval_set | 22 | 21 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | eval_set_delayed | 67 | 66 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | eval_rule | 80 | 79 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | eval_rule_delayed | 100 | 93 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | resume_normalize_rules | 115 | 132 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | resume_normalize_rules_after_rule_exprs | 122 | 139 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | resume_set_with_rhs | 142 | 159 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | resume_rule_with_rhs | 159 | 176 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | evaluate_rule_lhs | 163 | rule_lhs.rs:30 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | evaluate_pattern_rule_lhs | 171 | removed |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | finish_definition_assignment | 238 | 180 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | finish_prepared_definition_assignment | 405 | 347 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | try_set_symbol_own_value_lvalue | 463 | 405 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | try_set_list_destructuring_lvalue | 505 | 447 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | store_list_destructuring_assignments | 528 | 470 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | is_context_path_symbol_expr | 547 | 489 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | is_packages_symbol_expr | 551 | 493 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | is_path_symbol_expr | 555 | 497 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | normalize_context_list | 559 | 501 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | list_expr_args | 570 | 512 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | list_destructuring_shapes_match | 579 | 521 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | runtime_profile_outcome_for_optional_result | 595 | 537 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | runtime_profile_outcome_for_bool_result | 610 | 552 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | runtime_profile_outcome_for_expr_result | 625 | 567 |
| crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/syntax_policy.rs | definition_lhs_blank_head | 122 | 140 |
| crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/syntax_policy.rs | definition_lhs_pattern_wrapper_head | 133 | 151 |
| crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/syntax_policy.rs | every_registered_matcher_head_has_an_explicit_lhs_policy | 151 | 169 |

Complete new test/scanner item names and line positions:

`crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs`:

```text
DEPTH@6, on_small_stack@8, shallow@18, spine@24, bottom@31, drain@44, rule_lhs_construction_and_teardown_control@56, rule_lhs_deep_classifier_true_and_false@66, prompt@80, direct_walk@87, rule_lhs_deep_ordinary@140, rule_lhs_deep_one_suspension@144, rule_lhs_deep_two_suspensions@154, rule_lhs_deep_error_after_resume@164, rule_lhs_deep_abandon_at_first_and_second@174, Roots@186, visit_expr@188, visit_symbol@191, roots@195, root_fixture@200, assert_roots@217, rule_lhs_roots_cover_every_field_and_destination@221, rule_lhs_missing_root_falsifier@250, rule_lhs_scalar_state_copy_trace_drop_and_abandon_20000@260, rule_lhs_error_resume_propagates_without_running_work@292
```

`crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs`:

```text
OWNER@7, FILES@8, sources@40, assert_files_and_functions@53, OUTBOUND@73, assert_outbound@115, structural_rule_pattern_depth_contract@132, direct_cross_module_and_callback_recursion_falsifiers@154, unscanned_helper_falsifier@181, missing_file_falsifier@186, missing_function_falsifier@193, assert_no_self_reaching@201, rule_lhs_ownership_is_not_self_reaching@210, self_reaching_state_falsifier@221, assert_root_fields@227, rule_lhs_source_root_fields_are_complete@284, missing_root_field_falsifier@289
```

`crates/rum-stdlib/tests/rule_pattern_continuations/lifecycle.rs`:

```text
cancel_case@3, cancel_rule_first@47, cancel_rule_second@51, cancel_delayed_first@56, cancel_delayed_second@60, abandon_without_answer@64
```

`crates/rum-stdlib/tests/rule_pattern_continuations/policy.rs`:

```text
structural_plain_children_are_rebuilt_without_applying_the_call@4, structural_plain_child_resumes_without_applying_the_call@20, binder_is_held@31, construction_and_teardown_control_2000_on_2mib@42
```

`crates/rum-stdlib/tests/rule_pattern_continuations/rule.rs`:

```text
rule_plain@4, rule_pattern@15, ruledelayed_plain@26, ruledelayed_pattern@37, rule_blank@48, rule_patterntest@59, rule_condition@70, rule_wrapper@81, rule_head@92, rule_twice@103, replacepart_pattern@114, normalizerules_rule_pattern@125, replaceall_rule_pattern@136, replacerepeated_rule_pattern@147, replace_rule_pattern@158, replacelist_rule_pattern@169, replaceat_rule_pattern@180, rule_distinct_twice@191, ruledelayed_distinct_twice@202, rule_nested_pattern@213, rule_lhs_then_rhs@224, rule_one_child_two_inputs@235, rule_error_after_resume@246, rule_condition_held@257, rule_patterntest_held@268, rule_holdpattern_held@279, ruledelayed_rhs_held@290
```

`crates/rum-stdlib/tests/rule_pattern_continuations/support.rs`:

```text
session@6, View@16, fmt@24, view@34, finish@52, run@66, record@102, compare@137
```

`crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs` additions:

```text
flat_walk_call_graph@869, FlatCalls@898, resolve@904, visit_expr_call@927, visit_expr_method_call@945, visit_expr_path@949, visit_macro@964, flat_walk_recursive_edges@970
```

All changed source/data file sizes (characters, before → after):

| File | Before | After | Lines |
| --- | --- | --- | --- |
| crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/syntax_policy.rs | 6903 | 7544 | 211 |
| crates/rum-runtime/src/runtime/definitions/definitions.rs | 25061 | 22502 | 579 |
| crates/rum-runtime/src/runtime/definitions/mod.rs | 792 | 806 | 27 |
| crates/rum-runtime/src/runtime/definitions/rule_lhs/continuation.rs | 0 | 1793 | 57 |
| crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs | 0 | 1758 | 60 |
| crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs | 0 | 9695 | 306 |
| crates/rum-runtime/src/runtime/definitions/rule_lhs.rs | 0 | 7317 | 204 |
| crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs | 169908 | 170719 | 1526 |
| crates/rum-runtime/src/runtime/tests/frame_root_trace.rs | 30342 | 30342 | 739 |
| crates/rum-stdlib/src/tests/association_depth_contract/mod.rs | 379 | 412 | 20 |
| crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs | 28288 | 32108 | 981 |
| crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs | 0 | 10022 | 309 |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv | 246920 | 248115 | 1460 |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv | 420378 | 420100 | 2944 |
| crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv | 395803 | 397500 | 3130 |
| crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv | 91646 | 91646 | 69 |
| crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv | 111755 | 111757 | 318 |
| crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv | 18161 | 18161 | 3 |
| crates/rum-stdlib/tests/rule_pattern_continuations/lifecycle.rs | 0 | 2790 | 86 |
| crates/rum-stdlib/tests/rule_pattern_continuations/policy.rs | 0 | 2495 | 76 |
| crates/rum-stdlib/tests/rule_pattern_continuations/rule.rs | 0 | 7502 | 298 |
| crates/rum-stdlib/tests/rule_pattern_continuations/support.rs | 0 | 4911 | 159 |
| crates/rum-stdlib/tests/rule_pattern_continuations.rs | 0 | 171 | 7 |
| docs/guide/patterns-and-rules.md | 6705 | 7006 | 216 |

The production walk/state/continuation files remain below 10k characters;
the private test file remains below 10k. The structural test is 10,022 and
keeps one cohesive source-boundary contract. `definitions.rs` remains in the
split-plan range after extracting the walk; assignment planning/storage is a
possible later boundary, outside this brief. The existing 30k frame protocol
test can later split scanning from assertions, and the 32k recursion inventory
can split its declared edge table from scanner mechanics. The 170k closed
derive inventory is existing architecture debt: this brief makes only the
mandated seven derive/two pure-frame additions, with no unrelated list rewrite.
TSV sizes are inventories, not executable module growth.

Guide clarification: `docs/guide/patterns-and-rules.md:144` adds only:

> When `Rule` or `RuleDelayed` prepares its left-hand side, ordinary
> subexpressions can request input. Resuming continues that preparation and
> then completes the rule. Pattern binders, the test operands of `Condition` and `PatternTest`,
> and a delayed right-hand side stay held during this preparation.

### 2. Verification and every inventory row change

All Cargo/CLI/profile processes used the brief's serial bounded wrapper,
`CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true`, and
`TMPDIR=<repo>/.temp/brief-572/tmp`. The full commands, bounds, elapsed durations
and exits are in each adjacent `logs/<label>.json`. Final tests run in
`.temp/brief-572/verification-source` with `CARGO_TARGET_DIR` under this brief,
so the existing tests' hard-coded target/brief-544 and .temp/brief-546 replay
paths also remain under brief-572. Formatting runs in the main tree. Source
and inventory equality between the export and main tree is checked separately.

The 27 mandatory witnesses were red before any production edit. All 27 failed,
as did two second-prompt cancellations and two added resume fixtures; five
controls already passed. The construction-and-teardown control at n=2,000 on
2 MiB was the first depth run and passed before production changes. Its
private counterpart also ran separately before the deep driver suite.
No depth was lowered and no process-aborting probe was run.

Verbatim red/first-green witness and aggregate output:

`logs/red.txt`:

```text
test rule_pattern_continuations::rule::rule_pattern ... CASE	Rule-pattern	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[2]", ["p: "])	queued=(View { status: "OK", value: Some("Rule[h[Pattern[x, Blank[]], \"two\", 3], 9]"), termination: "None", messages: [], prints: [] }, "List[2, 3]", [])
PROMPTS	Rule-pattern	["p: "]	["p: "]
test result: FAILED. 5 passed; 31 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
```

`logs/attempt-01b-continuations.txt`:

```text
test rule_pattern_continuations::rule::rule_pattern ... CASE	Rule-pattern	resumed=(View { status: "OK", value: Some("Rule[h[Pattern[x, Blank[]], \"two\", 3], 9]"), termination: "None", messages: [], prints: [] }, "List[2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("Rule[h[Pattern[x, Blank[]], \"two\", 3], 9]"), termination: "None", messages: [], prints: [] }, "List[2, 3]", [])
PROMPTS	Rule-pattern	["p: "]	["p: "]
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
```

`logs/final-continuations.txt`:

```text
test rule_pattern_continuations::rule::rule_pattern ... CASE	Rule-pattern	resumed=(View { status: "OK", value: Some("Rule[h[Pattern[x, Blank[]], \"two\", 3], 9]"), termination: "None", messages: [], prints: [] }, "List[2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("Rule[h[Pattern[x, Blank[]], \"two\", 3], 9]"), termination: "None", messages: [], prints: [] }, "List[2, 3]", [])
PROMPTS	Rule-pattern	["p: "]	["p: "]
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
```

Verbatim construction/private-depth and structural-falsifier evidence:

`logs/construction-control.txt`:

```text

```

`logs/construction-control-02.txt`:

```text
test runtime::definitions::rule_lhs::tests::rule_lhs_construction_and_teardown_control ... RULE_CONSTRUCTION_TEARDOWN n=2000 stack=2MiB passed
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1107 filtered out; finished in 0.01s
```

`logs/hygiene-repair-private.txt`:

```text
running 12 tests
test runtime::definitions::rule_lhs::tests::rule_lhs_construction_and_teardown_control ... RULE_CONSTRUCTION_TEARDOWN n=2000 stack=2MiB passed
ok
test runtime::definitions::rule_lhs::tests::rule_lhs_deep_abandon_at_first_and_second ... RULE_DEEP n=2000 abandon_at=0 stack=2MiB passed
RULE_DEEP n=2000 abandon_at=1 stack=2MiB passed
ok
test runtime::definitions::rule_lhs::tests::rule_lhs_deep_classifier_true_and_false ... RULE_CLASSIFIER n=2000 true/false stack=2MiB passed
ok
test runtime::definitions::rule_lhs::tests::rule_lhs_deep_error_after_resume ... RULE_DEEP n=2000 prompts=1 error=true stack=2MiB passed
ok
test runtime::definitions::rule_lhs::tests::rule_lhs_deep_one_suspension ... RULE_DEEP n=2000 prompts=1 error=false stack=2MiB passed
ok
test runtime::definitions::rule_lhs::tests::rule_lhs_deep_ordinary ... RULE_DEEP n=2000 prompts=0 error=false stack=2MiB passed
ok
test runtime::definitions::rule_lhs::tests::rule_lhs_deep_two_suspensions ... RULE_DEEP n=2000 prompts=2 error=false stack=2MiB passed
ok
test runtime::definitions::rule_lhs::tests::rule_lhs_error_resume_propagates_without_running_work ... ok
test runtime::definitions::rule_lhs::tests::rule_lhs_missing_root_falsifier - should panic ... 
thread 'runtime::definitions::rule_lhs::tests::rule_lhs_missing_root_falsifier' panicked at crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs:218:5:
assertion `left == right` failed: all rule LHS roots must be traced
  left: [Atom(Integer(Small(1))), Atom(Integer(Small(2))), Atom(Integer(Small(3))), Atom(Integer(Small(4))), Atom(Integer(Small(6)))]
 right: [Atom(Integer(Small(1))), Atom(Integer(Small(2))), Atom(Integer(Small(3))), Atom(Integer(Small(4))), Atom(Integer(Small(5))), Atom(Integer(Small(6)))]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
ok
test runtime::definitions::rule_lhs::tests::rule_lhs_roots_cover_every_field_and_destination ... RULE_ROOTS raw head, held/pending/unvisited args, resolved head, prefix, pending expression, RHS passed
ok
test runtime::definitions::rule_lhs::tests::rule_lhs_scalar_state_copy_trace_drop_and_abandon_20000 ... RULE_SCALAR_STATE records=20000 ordinary/cold copy, trace, drop, pure abandon stack=2MiB passed
ok
test runtime::tests::definitions_and_rules_patterns::rule_lhs_evaluates_values_without_evaluating_pattern_names ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 0.82s
```

`logs/final-structural.txt`:

```text
running 9 tests
test tests::association_depth_contract::rule_pattern_depth_contract::direct_cross_module_and_callback_recursion_falsifiers ... RULE_RECURSION_FALSIFIERS direct, cross-module, function-item, callback alias, closure, Self method fired
ok
test tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_file_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:54:5:
assertion `left == right` failed: required rule-LHS files
  left: {"definition_lhs_preparation/syntax_policy.rs", "rule_lhs.rs", "rule_lhs/continuation.rs"}
 right: {"definition_lhs_preparation/syntax_policy.rs", "rule_lhs.rs", "rule_lhs/continuation.rs", "rule_lhs/state.rs"}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
ok
test tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_function_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:63:9:
assertion `left == right` failed: required rule-LHS functions in definition_lhs_preparation/syntax_policy.rs
  left: {}
 right: {"definition_lhs_contains_pattern_syntax"}
ok
test tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::missing_root_field_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:281:5:
assertion `left == right` failed: required rule-LHS root fields
  left: {"RuleLhsAwait": {"destination"}, "RuleLhsCompletionFrame": {"rhs"}, "RuleLhsState": {"stack"}, "RuleLhsWork": {"original_args", "original_head", "policy", "prefix", "resolved_head"}, "RulePatternSuspensionFrame": {"pending", "state"}}
 right: {"RuleLhsAwait": {"destination", "pending_expr"}, "RuleLhsCompletionFrame": {"rhs"}, "RuleLhsState": {"stack"}, "RuleLhsWork": {"original_args", "original_head", "policy", "prefix", "resolved_head"}, "RulePatternSuspensionFrame": {"pending", "state"}}
ok
test tests::association_depth_contract::rule_pattern_depth_contract::rule_lhs_ownership_is_not_self_reaching ... ok
test tests::association_depth_contract::rule_pattern_depth_contract::rule_lhs_source_root_fields_are_complete ... ok
test tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::self_reaching_state_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:202:5:
self-reaching rule-LHS state
ok
test tests::association_depth_contract::rule_pattern_depth_contract::structural_rule_pattern_depth_contract ... RULE_SOURCE_CONTRACT 4 files, exact function sets, per-file/combined recursion and outbound audit passed
ok
test tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier - should panic ... 
thread 'tests::association_depth_contract::rule_pattern_depth_contract::unscanned_helper_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/rule_pattern_depth_contract.rs:125:5:
unaudited rule-LHS outbound calls: {"drive -> moved_to_unscanned_helper"}
ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.15s
```

`logs/adjacent-depth-contracts.txt` (unchanged lazy/schema/association neighbors included):

```text
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 1078 filtered out; finished in 1.12s
```

The scalar tests trace distinct values for raw head, original held/pending/
unvisited args, resolved head, prefix, pending expression and completion RHS,
including all three destination tags. The direct deep tests own construction,
inspection and iterative teardown on their 2 MiB thread. They keep the raw
source alive through frame retirement; they do not exercise generic last-owner
recursive Expr teardown, deep parsing, formatting, equality or matcher compilation.
Those are separate, fenced depth boundaries.

Two initial compile corrections were required: missing trace-trait imports
(22 derive-related errors) and using the private test EvalOutcome's `.value`.
Neither counted as a compiling candidate or a weakened test. The source audit's
self-reaching namespace was corrected to the actual discovery namespace
`rum_runtime::runtime::definitions::rule_lhs::` before candidate 02 qualification.
Candidates 02–04 each passed 36 integration, 11 private, 28 adjacent structural,
and one ownership-discovery test before measurement.

The first final runtime gate exposed the stdout hygiene failure in the new
private `tests.rs`; six diagnostic writes changed to stderr. The final measured
production code is unchanged: the sole post-measurement Rust difference from
candidate 04 is this cfg(test) module, absent from both release dependency files.
All remaining 1,232 measured source/manifest entries match; the export matches
the final tree. Fingerprints and certificates were replayed again after that fix.
The first stale-certificate runs are expected maintenance reds, not green tests.
No retention class, status, instance body or promoted identity was changed.

`logs/fingerprint-replay.txt` (exit 0):

```text
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/.temp/brief-572/verification-source/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1105 filtered out; finished in 0.19s
```

`logs/fingerprint-replay-after-hygiene.txt` (exit 0):

```text
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/.temp/brief-572/verification-source/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1105 filtered out; finished in 0.31s
```

`logs/certificates-true-replay.txt` (exit 101):

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1105 filtered out; finished in 17.58s
```

`logs/certificates-false-replay.txt` (exit 101):

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1099 filtered out; finished in 18.02s
```

`logs/certificates-true-replay-after-hygiene.txt` (exit 101):

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1105 filtered out; finished in 17.38s
```

`logs/certificates-false-replay-after-hygiene.txt` (exit 101):

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1099 filtered out; finished in 17.45s
```

`logs/hygiene-repair-check.txt` (exit 0):

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1106 filtered out; finished in 0.46s
```

Runtime compile-fail doctests emit expected compiler diagnostics and pass; three
existing unused test-helper warnings were also emitted.

The workspace run also encountered nine native ZeroMQ loopback PermissionDenied
failures. The existing helper requires `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`; it does
not automatically skip without that flag in this checkout. The full no-fail-fast
run was allowed to finish, then those nine native tests were rerun with the
supported skip flag. They are skips, not successful native socket coverage.
The original workspace exit remains disclosed; no test contract was edited.

Mandatory final commands and verbatim result lines (the initial failed runtime
run and its repaired run are both disclosed):

`cargo fmt --check` — `logs/final-fmt.txt`, exit 0, bound 150 s:

```text
(no output; exit 0)
```

`cargo fmt --check` — `logs/final-fmt-after-hygiene.txt`, exit 0, bound 150 s:

```text
(no output; exit 0)
```

`cargo test -p rum-stdlib --test rule_pattern_continuations -- --nocapture` — `logs/final-continuations.txt`, exit 0, bound 150 s:

```text
     Running tests/rule_pattern_continuations.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rule_pattern_continuations-7084ce392f17d58c)
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
```

`cargo test -p rum-stdlib --lib rule_pattern_depth_contract -- --nocapture` — `logs/final-structural.txt`, exit 0, bound 150 s:

```text
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1097 filtered out; finished in 0.15s
```

`cargo test -p rum-runtime -- --nocapture` — `logs/final-runtime.txt`, exit 101, bound 300 s:

```text
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_runtime-95cd5fbb46da4f1c)
stdout writes are forbidden in rum-core, rum-runtime, and rum-stdlib outside /tests/ and /bin/ paths; /bin/ is excluded because codegen tools own their stdout. Violations:
/home/remi/dev/rum/.temp/brief-572/verification-source/crates/rum-runtime/src/runtime/definitions/rule_lhs/tests.rs
test result: FAILED. 1104 passed; 1 failed; 3 ignored; 0 measured; 0 filtered out; finished in 24.50s
```

`cargo test -p rum-runtime -- --nocapture` — `logs/final-runtime-after-hygiene.txt`, exit 0, bound 300 s:

```text
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_runtime-95cd5fbb46da4f1c)
test result: ok. 1105 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 23.95s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.97s
```

`cargo test -p rum-stdlib --lib ownership_graph -- --nocapture` — `logs/final-ownership.txt`, exit 0, bound 300 s:

```text
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1086 filtered out; finished in 59.27s
```

`cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture` — `logs/final-acceptance-default.txt`, exit 0, bound 300 s:

```text
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_stdlib-3a556b9db7fd59ac)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1105 filtered out; finished in 17.75s
```

`cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture` — `logs/final-acceptance-no-default.txt`, exit 0, bound 300 s:

```text
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_stdlib-5d6fedc676d25fda)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1099 filtered out; finished in 17.67s
```

`cargo test --workspace --no-fail-fast -- --nocapture` — `logs/final-workspace.txt`, exit 101, bound 2400 s:

```text
     Running unittests src/main.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
     Running tests/cli.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
     Running tests/input_requests.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
     Running tests/terminal.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/accumulator_chain.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/derive.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s
     Running tests/adapter_contract.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
     Running tests/batch_boundary_ownership.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/guest_loading_resume.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/host_payload_destruction_contract.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 291.30s
     Running tests/installer.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/lifecycle_buildout.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/lifecycle_contract.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
     Running tests/random_password_policy.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/stress.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.32s
     Running tests/suspended_timeout_contract.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.05s
     Running tests/suspension_abandonment_contract.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: FAILED. 142 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
     Running unittests src/bin/rum-jupyter-kernel.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
     Running tests/input_abandonment.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
     Running tests/input_continuation.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1105 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 24.06s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 1053 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1431.55s
     Running unittests src/bin/generate-builtin-reference.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/guest_loading_resume.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s
     Running tests/lazy_pull_continuations.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.68s
     Running tests/loop_continuations.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
     Running tests/rule_pattern_continuations.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
     Running tests/schema_continuations.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.12s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.56s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.90s
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

`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test -p rum-jupyter --lib zeromq::tests::native -- --nocapture` — `logs/final-native-skips.txt`, exit 0, bound 150 s:

```text
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-572/target-verification/debug/deps/rum_jupyter-dd24b36d3edaa47a)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 142 filtered out; finished in 0.00s
```

Workspace harness totals across 44 result lines: 3052 passed, 9 failed, 58 ignored. The nine failures are all loopback-denied native ZeroMQ tests. The focused flag-enabled rerun reports nine `ok` results in the harness, but all nine return early and are **skips**. No native socket case executed. Verbatim skip output:

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
```

Standing ownership inventories were reconciled from discovery, preserving all
existing identities except the five explicitly authorized removals. Counts:
attributes 1,453 → 1,460; captures 2,946 → 2,944; macros 3,117 → 3,130;
storage 88 unchanged; host payloads 63 unchanged; self-reaching rows 59
unchanged. The 23 additions are UnknownConstruct (seven derives, sixteen
macros). Eighteen rows move (including the two authorized relocations).
No new unknown points into untouched code. Full exact before/after rows are
in `.temp/brief-572/candidate-04-ownership-row-changes.json`. Intermediate
discovery/row snapshots for candidates 01–03 are preserved in their corresponding
`candidate-0N-ownership-row-changes.json` files; the tables below show the final
baseline-to-candidate delta.

Every original-row change/removal follows; row numbers refer to the saved
baseline. All retained rows remain UnknownConstruct, with the same construct
identity; source positions/macro end positions alone move. `D/` abbreviates
`crates/rum-runtime/src/runtime/definitions/`.

| Inventory | Baseline row | Old site | New site | Construct | Reason |
| --- | --- | --- | --- | --- | --- |
| unknown_captures.tsv | 405 | D/definitions.rs:496:78 | D/definitions.rs:438:78 | closure captures: \|\| (capture types/modes unresolved) → closure captures: \|\| (capture types/modes unresolved) | source position only |
| unknown_captures.tsv | 406 | D/definitions.rs:563:35 | D/definitions.rs:505:35 | closure captures: \|existing\| (capture types/modes unresolved) → closure captures: \|existing\| (capture types/modes unresolved) | source position only |
| unknown_captures.tsv | 407 | D/definitions.rs:592:18 | D/definitions.rs:534:18 | closure captures: \|(lhs_item , rhs_item)\| (capture types/modes unresolved) → closure captures: \|(lhs_item , rhs_item)\| (capture types/modes unresolved) | source position only |
| unknown_captures.tsv | 403 | D/definitions.rs:187:26 | removed | closure captures: \|arg\| (capture types/modes unresolved) | explicit brief-572 removal; recursive construct replaced by flat prefix walk |
| unknown_captures.tsv | 404 | D/definitions.rs:217:26 | removed | closure captures: \|arg\| (capture types/modes unresolved) | explicit brief-572 removal; recursive construct replaced by flat prefix walk |
| unknown_macros.tsv | 261 | D/definition_lhs_preparation/syntax_policy.rs:123:5 | D/definition_lhs_preparation/syntax_policy.rs:141:5 | unexpanded macro matches! (ends at 130:6) → unexpanded macro matches! (ends at 148:6) | source position only |
| unknown_macros.tsv | 262 | D/definition_lhs_preparation/syntax_policy.rs:134:5 | D/definition_lhs_preparation/syntax_policy.rs:152:5 | unexpanded macro matches! (ends at 143:6) → unexpanded macro matches! (ends at 161:6) | source position only |
| unknown_macros.tsv | 282 | D/definitions.rs:42:30 | D/definitions.rs:41:30 | unexpanded macro crate :: resume_or_suspend! (ends at 50:14) → unexpanded macro crate :: resume_or_suspend! (ends at 49:14) | source position only |
| unknown_macros.tsv | 283 | D/definitions.rs:87:19 | D/definitions.rs:120:19 | unexpanded macro crate :: resume_or_suspend! (ends at 95:10) → unexpanded macro crate :: resume_or_suspend! (ends at 128:10) | authorized relocation |
| unknown_macros.tsv | 284 | D/definitions.rs:111:13 | D/definitions.rs:117:62 | unexpanded macro vec! (ends at 111:39) → unexpanded macro vec! (ends at 117:76) | authorized relocation |
| unknown_macros.tsv | 285 | D/definitions.rs:143:12 | D/definitions.rs:160:12 | unexpanded macro matches! (ends at 143:166) → unexpanded macro matches! (ends at 160:166) | source position only |
| unknown_macros.tsv | 286 | D/definitions.rs:151:13 | D/definitions.rs:168:13 | unexpanded macro debug_assert! (ends at 151:62) → unexpanded macro debug_assert! (ends at 168:62) | source position only |
| unknown_macros.tsv | 287 | D/definitions.rs:160:44 | D/definitions.rs:177:44 | unexpanded macro vec! (ends at 160:58) → unexpanded macro vec! (ends at 177:58) | source position only |
| unknown_macros.tsv | 291 | D/definitions.rs:416:16 | D/definitions.rs:358:16 | unexpanded macro matches! (ends at 421:14) → unexpanded macro matches! (ends at 363:14) | source position only |
| unknown_macros.tsv | 292 | D/definitions.rs:443:52 | D/definitions.rs:385:52 | unexpanded macro vec! (ends at 443:66) → unexpanded macro vec! (ends at 385:66) | source position only |
| unknown_macros.tsv | 293 | D/definitions.rs:458:52 | D/definitions.rs:400:52 | unexpanded macro vec! (ends at 458:66) → unexpanded macro vec! (ends at 400:66) | source position only |
| unknown_macros.tsv | 294 | D/definitions.rs:519:17 | D/definitions.rs:461:17 | unexpanded macro format! (ends at 519:73) → unexpanded macro format! (ends at 461:73) | source position only |
| unknown_macros.tsv | 295 | D/definitions.rs:548:5 | D/definitions.rs:490:5 | unexpanded macro matches! (ends at 548:126) → unexpanded macro matches! (ends at 490:126) | source position only |
| unknown_macros.tsv | 296 | D/definitions.rs:552:5 | D/definitions.rs:494:5 | unexpanded macro matches! (ends at 552:123) → unexpanded macro matches! (ends at 494:123) | source position only |
| unknown_macros.tsv | 297 | D/definitions.rs:556:5 | D/definitions.rs:498:5 | unexpanded macro matches! (ends at 556:119) → unexpanded macro matches! (ends at 498:119) | source position only |
| unknown_macros.tsv | 288 | D/definitions.rs:180:21 | removed | unexpanded macro vec! (ends at 180:51) | explicit brief-572 removal; recursive construct replaced by flat prefix walk |
| unknown_macros.tsv | 289 | D/definitions.rs:198:21 | removed | unexpanded macro vec! (ends at 198:51) | explicit brief-572 removal; recursive construct replaced by flat prefix walk |
| unknown_macros.tsv | 290 | D/definitions.rs:207:21 | removed | unexpanded macro vec! (ends at 207:51) | explicit brief-572 removal; recursive construct replaced by flat prefix walk |

Every added row (all UnknownConstruct, full source positions):

| Inventory | Source position | Unknown construct |
| --- | --- | --- |
| unknown_attributes.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs/continuation.rs:4:1 | unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) |
| unknown_attributes.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs/continuation.rs:40:1 | unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon) |
| unknown_attributes.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:3:1 | unresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace) |
| unknown_attributes.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:8:1 | unresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace) |
| unknown_attributes.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:18:1 | unresolved attribute/expansion derive (Clone , Copy , rum_derive :: FrameRootTrace) |
| unknown_attributes.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:27:1 | unresolved attribute/expansion derive (Clone , Copy , rum_derive :: FrameRootTrace) |
| unknown_attributes.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:34:1 | unresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/definition_lhs_preparation/syntax_policy.rs:124:20 | unexpanded macro matches! (ends at 124:62) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/definitions.rs:85:9 | unexpanded macro crate :: resume_or_suspend! (ends at 90:10) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/definitions.rs:102:9 | unexpanded macro crate :: resume_or_suspend! (ends at 107:10) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:50:16 | unexpanded macro matches! (ends at 50:58) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:51:20 | unexpanded macro matches! (ends at 51:49) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:52:21 | unexpanded macro matches! (ends at 52:59) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:67:17 | unexpanded macro unreachable! (ends at 67:59) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:86:16 | unexpanded macro matches! (ends at 86:59) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:90:16 | unexpanded macro matches! (ends at 90:61) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:112:9 | unexpanded macro crate :: resume_or_suspend! (ends at 115:11) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:174:12 | unexpanded macro matches! (ends at 174:59) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:182:41 | unexpanded macro unreachable! (ends at 182:55) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:185:17 | unexpanded macro debug_assert_eq! (ends at 185:51) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:188:20 | unexpanded macro matches! (ends at 188:70) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:45:12 | unexpanded macro matches! (ends at 45:62) |
| unknown_macros.tsv | crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:57:41 | unexpanded macro unreachable! (ends at 57:89) |

Retention `obligations.tsv` changes exactly three evidence cells; all other
fields, including the old payload descriptions, N class and Pending status,
are preserved:

| Row | ID | Old evidence | New evidence |
| --- | --- | --- | --- |
| 177 | N:NormalizeRules | crates/rum-runtime/src/runtime/definitions/definitions.rs:129; crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:200; crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:248 | crates/rum-runtime/src/runtime/definitions/definitions.rs:146; crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:200; crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:248 |
| 227 | N:Rule | crates/rum-runtime/src/runtime/definitions/definitions.rs:85; crates/rum-runtime/src/runtime/definitions/definitions.rs:108; crates/rum-runtime/src/runtime/definitions/definitions.rs:184; crates/rum-runtime/src/runtime/definitions/definitions.rs:222; crates/rum-runtime/src/runtime/definitions/definitions.rs:226 | crates/rum-runtime/src/runtime/definitions/definitions.rs:84; crates/rum-runtime/src/runtime/definitions/definitions.rs:101; crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:111; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:11; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:13 |
| 228 | N:RuleDelayed | crates/rum-runtime/src/runtime/definitions/definitions.rs:85; crates/rum-runtime/src/runtime/definitions/definitions.rs:108; crates/rum-runtime/src/runtime/definitions/definitions.rs:184; crates/rum-runtime/src/runtime/definitions/definitions.rs:222; crates/rum-runtime/src/runtime/definitions/definitions.rs:226 | crates/rum-runtime/src/runtime/definitions/definitions.rs:84; crates/rum-runtime/src/runtime/definitions/definitions.rs:101; crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:111; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:11; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:13 |

`local-contracts.tsv` changes only its last (19th) fingerprint column on the
following 68 rows. Values are the saved pre-brief baseline and final replay;
all other columns are byte-identical. The baseline-to-final cells are in `baseline-to-final-fingerprint-row-changes.json`.
The intermediate pre-hygiene replay is preserved as `before-hygiene-fix-fingerprint-row-changes.json`.

| Row | ID | Before | Final |
| --- | --- | --- | --- |
| 2 | B01 | fnv1a64:ff9ea020a988a76d | fnv1a64:a2e887771d42718c |
| 3 | B02 | fnv1a64:7497ba847b39a2c0 | fnv1a64:33c32b325169d331 |
| 4 | B03 | fnv1a64:b76672075001201b | fnv1a64:7c607132cbccd402 |
| 5 | B04 | fnv1a64:4ec723bd1a837c55 | fnv1a64:e2c91a8f837e8c54 |
| 6 | B05 | fnv1a64:adacc7ac87d52324 | fnv1a64:3e9df268e815524d |
| 7 | B06 | fnv1a64:11989e314e6feef9 | fnv1a64:cdee82559a270f40 |
| 8 | B07 | fnv1a64:d03ba3af5502ed74 | fnv1a64:e2184b776a53f9fd |
| 9 | B08 | fnv1a64:b9d185c4aeca6d9f | fnv1a64:0da564fe0d77ad4e |
| 10 | B09 | fnv1a64:e2aa2ad4844a3ec1 | fnv1a64:d1ee84622bb746e8 |
| 11 | B10 | fnv1a64:e4e96c42afa58f4e | fnv1a64:5aa51024431ebe87 |
| 12 | B11 | fnv1a64:101ed51a3477a255 | fnv1a64:a420cbec9d72b254 |
| 13 | B12 | fnv1a64:a1b6aad06052d3c1 | fnv1a64:90fb045e07bfdbe8 |
| 14 | I01 | fnv1a64:dd808b1e16409ded | fnv1a64:80ca727489fa680c |
| 15 | I02 | fnv1a64:04ee4540800bab24 | fnv1a64:95df6ffce04bda4d |
| 16 | I03 | fnv1a64:a8d5b66e0b3f7581 | fnv1a64:2ced2cfb7b194428 |
| 17 | I04 | fnv1a64:0a1f0daa2a2a44e0 | fnv1a64:a3938da00da764d1 |
| 18 | P01 | fnv1a64:ded327f82350a699 | fnv1a64:76a496323f8f47e0 |
| 19 | P02 | fnv1a64:f1ebcd416b79e27e | fnv1a64:e2dd098a99caf357 |
| 20 | P03 | fnv1a64:e8cef9fddfdfef33 | fnv1a64:44d204247c75727a |
| 21 | P04 | fnv1a64:6972d899ffe2bec0 | fnv1a64:289e4947d612ef31 |
| 22 | P05 | fnv1a64:bcf0c78eb328a4dd | fnv1a64:2bf40d57dc88cb9c |
| 23 | P06 | fnv1a64:f235cfa0ec826d41 | fnv1a64:e17a292e93ef7568 |
| 24 | P070 | fnv1a64:2c836be2caf7e741 | fnv1a64:1bc7c5707264ef68 |
| 25 | P080 | fnv1a64:96b1250127232c0e | fnv1a64:4a84e8da1e858dc7 |
| 26 | P090 | fnv1a64:66a37abee41e3680 | fnv1a64:1a5d03f7c6191771 |
| 27 | P100 | fnv1a64:f8ffc1a215bd94a5 | fnv1a64:2f519bce401cdd04 |
| 28 | P110 | fnv1a64:ece51adaf4570d86 | fnv1a64:1654baba27e8a97f |
| 29 | P120 | fnv1a64:19d6cea2f9a72c3f | fnv1a64:e78ae38eb8dee5ee |
| 30 | P130 | fnv1a64:7408fd0c83b6c774 | fnv1a64:85e5a4d49907d3fd |
| 31 | P140 | fnv1a64:dafdbe42842b87d8 | fnv1a64:5ab9d272fad50bc9 |
| 32 | P150 | fnv1a64:fbc48a61c1591ca0 | fnv1a64:4dc37d0d327b2111 |
| 33 | P071 | fnv1a64:9270e5591e22a594 | fnv1a64:3be1fe8fc79c0d9d |
| 34 | P081 | fnv1a64:9c50e49a75057c61 | fnv1a64:db3c9965a11cac88 |
| 35 | P091 | fnv1a64:14993c2b079e6959 | fnv1a64:95eb4cc5bb7d4f20 |
| 36 | P101 | fnv1a64:e4e80ba829baaa86 | fnv1a64:0e57ab875d4c467f |
| 37 | P111 | fnv1a64:e3a6015a69d15451 | fnv1a64:ac039d501e906658 |
| 38 | P121 | fnv1a64:7872c5796d800b6a | fnv1a64:014331899024c71b |
| 39 | P131 | fnv1a64:ec71462350d79dd5 | fnv1a64:80733cf5b9d2add4 |
| 40 | P141 | fnv1a64:c5da97082f96d6b9 | fnv1a64:00742b43a919a280 |
| 41 | P151 | fnv1a64:ba06efc6e7e7cd7f | fnv1a64:d5f019d399ce58ae |
| 42 | B13 | fnv1a64:9fa931ba7f0caaf9 | fnv1a64:5bff15decac3cb40 |
| 43 | R01 | fnv1a64:7e8c6fc1c14ffb90 | fnv1a64:95e44740e2763fa1 |
| 44 | R02 | fnv1a64:2de067c3f97163f3 | fnv1a64:dc21fcb4cce432ba |
| 45 | R03 | fnv1a64:3c8a2ad1d72d01fc | fnv1a64:7ecdefef3fc5b5c5 |
| 46 | R040 | fnv1a64:35e4ddf97ff3a8b0 | fnv1a64:586f9178b09d48c1 |
| 47 | R050 | fnv1a64:0553fbf1b5477c3d | fnv1a64:f6fc32e7b731f7fc |
| 48 | R060 | fnv1a64:d12edcb889df7b68 | fnv1a64:718d318985930f39 |
| 49 | R070 | fnv1a64:04beacb456a73b61 | fnv1a64:43aa617f82be6b88 |
| 50 | R080 | fnv1a64:d123769cf9af0de0 | fnv1a64:6a97f692dd2c2dd1 |
| 51 | R090 | fnv1a64:e2df5a74e9d3e465 | fnv1a64:02b1d701abd37144 |
| 52 | R100 | fnv1a64:7c8c1659f9d6ca9b | fnv1a64:4186158575a27e82 |
| 53 | R110 | fnv1a64:5381ef2365ca07d6 | fnv1a64:7da095b9733c75ef |
| 54 | R041 | fnv1a64:85e294e4963469f9 | fnv1a64:42387908e1eb8a40 |
| 55 | R051 | fnv1a64:f4c5a8d4714c4300 | fnv1a64:a87f320d534723f1 |
| 56 | R061 | fnv1a64:11f0b45c0432f60f | fnv1a64:ae33c99a6d01eb5e |
| 57 | R071 | fnv1a64:89a2f4536c3589b2 | fnv1a64:02c776a7f0637ac3 |
| 58 | R081 | fnv1a64:b4ea2af6b16969a5 | fnv1a64:eb3c0522dbc8b204 |
| 59 | R091 | fnv1a64:c2b0e8f6b49b4b18 | fnv1a64:39b074f4eeda1689 |
| 60 | R101 | fnv1a64:1b35e82d52327728 | fnv1a64:26078f7bf8bcf879 |
| 61 | R111 | fnv1a64:c0a03d7415f87987 | fnv1a64:fbe743c38519f1b6 |
| 62 | R12 | fnv1a64:4c499230cf2c3ac7 | fnv1a64:b6b9d5aaf7bb2076 |
| 63 | R13 | fnv1a64:4fd04bade1b5310b | fnv1a64:1dfc8ace3a7189d2 |
| 64 | R14 | fnv1a64:1357d017f26e54a5 | fnv1a64:49a9aa441ccd9d04 |
| 65 | R15 | fnv1a64:17ed367394120969 | fnv1a64:93f186f8e099d6d0 |
| 66 | O010 | fnv1a64:5ac25c56cc508f6f | fnv1a64:4614cfac36b82d3e |
| 67 | O0200 | fnv1a64:d5c56016d2e2add8 | fnv1a64:55817447498c31c9 |
| 68 | O011 | fnv1a64:13b7994097713fe8 | fnv1a64:b415ee119324d3b9 |
| 69 | O0210 | fnv1a64:b0b875b1d4c6eda7 | fnv1a64:c6cebb2f2694b156 |

`promoted.tsv` changes only C01/C02 rows 2/3, default column 5 and no-default
column 6. Exact replay cells are in `baseline-to-final-certificate-cell-changes-{true,false}.json`;
the following table enumerates every changed fingerprint token in cell order
(including repeated tokens). Normalizing those tokens leaves the entire old
and new cell equal. No other cell changes. Intermediate replay cells are kept
in `before-hygiene-fix-certificate-cell-changes-*.json`.

| ID | Row | Column | hash feature | Changed tokens, in order |
| --- | --- | --- | --- | --- |
| C01 | 2 | 5 | true | 8ae78081242f6dcb → 71e8921f8d3c9ad6; 7e8ef6bea4db5a6d → f7bd8abba72b94a3; b0b875b1d4c6eda7 → c6cebb2f2694b156; 13b7994097713fe8 → b415ee119324d3b9; 8ae78081242f6dcb → 71e8921f8d3c9ad6; 7ee5c81eaf3f1487 → d6bb154f4c7ab891; d5c56016d2e2add8 → 55817447498c31c9; 5ac25c56cc508f6f → 4614cfac36b82d3e |
| C02 | 3 | 5 | true | 032eb69a5be27e09 → 55ed3783a90b5da8; e05b69ac9761cb26 → c7e9cebe6553d74d; dd808b1e16409ded → 80ca727489fa680c; 04ee4540800bab24 → 95df6ffce04bda4d; 0a1f0daa2a2a44e0 → a3938da00da764d1; 7e8c6fc1c14ffb90 → 95e44740e2763fa1; 2de067c3f97163f3 → dc21fcb4cce432ba; 9fa931ba7f0caaf9 → 5bff15decac3cb40; adacc7ac87d52324 → 3e9df268e815524d |
| C01 | 2 | 6 | false | b410c4288df684ce → ec6777785fd7b82e; 58226dce6b775b6e → 59d354669c0a9571; b0b875b1d4c6eda7 → c6cebb2f2694b156; 13b7994097713fe8 → b415ee119324d3b9; b410c4288df684ce → ec6777785fd7b82e; 160394f5a090ec34 → fae13f3a6b34c4f7; d5c56016d2e2add8 → 55817447498c31c9; 5ac25c56cc508f6f → 4614cfac36b82d3e |
| C02 | 3 | 6 | false | d9c1814a1c2b5f10 → 8133eb66176655b0; feb5779efe523626 → 10c08e0cfe54e84f; dd808b1e16409ded → 80ca727489fa680c; 04ee4540800bab24 → 95df6ffce04bda4d; 0a1f0daa2a2a44e0 → a3938da00da764d1; 7e8c6fc1c14ffb90 → 95e44740e2763fa1; 2de067c3f97163f3 → dc21fcb4cce432ba; 9fa931ba7f0caaf9 → 5bff15decac3cb40; adacc7ac87d52324 → 3e9df268e815524d |

### 3. Repaired sites and every measured row

The ten targeted sites P0342, P0344, P0350–P0357 are replaced by the flat driver
and rule completion frames. Remaining target lost sites: zero; global lost
continuations **56 → 46 predicted**. No fresh repository-wide census was run.
The held-pattern resolver (step C), matcher and generic Expr depth remain
outside this patch.

The director's one-unit base was used unchanged, never rebuilt:

| Build | Binary | SHA-256 |
| --- | --- | --- |
| base | rum-cli | 88d29a74f1726412e82e89717e18ca42cc81f3a727ea088b14d356bc63de43a1 |
| base | rum-cli-alloc | e6e4f9161ed454cb946015ad1e0bd8f012e5e7e23fd678245e8f4d81f6564628 |
| candidate-01 | rum-cli | 61930fcae4c3518584874691f5f027b15dd9abafd720b5dba4e4cbb78c70b529 |
| candidate-01 | rum-cli-alloc | 48d1f9157420869bdabfbf1b285a4781e9d0ee731110a74dfc9058fb9fce3d9e |
| candidate-02 | rum-cli | 7fdd14e4a29b2f664a08cc1506ea3d6b8aa51933e6934c06f3909bd7e1612c72 |
| candidate-02 | rum-cli-alloc | a921c1d024904fc37eab148bbbd9bcd59aa396d491ead5d0e89eaa13ff9e03bc |
| candidate-03 | rum-cli | 67cf216746acce484d73218448ec05b09f3dabf8652496e2b11bef7f2a915af0 |
| candidate-03 | rum-cli-alloc | 40a5f0c9e9c6d6e72a8e9c571f7028f579eae629abbe1f407ba90f53f2ffe3db |
| candidate-04 | rum-cli | 41c2bab9aac188146b1835206bd14cb153280021421cd6fc67868c17d765bd03 |
| candidate-04 | rum-cli-alloc | f0468107dc6fb29114d96cac15acbb332107a2585a1a4e072931721219202b8c |

Every candidate has fresh default and alloc-count release target directories,
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`; alloc-count used separate `--features`
and `rum-cli/alloc-count` arguments. Both hashes changed from base and the
preceding candidate. Candidate changes, one at a time: 01 is the initial flat
walk/classifier; 02 changes classifier traversal storage; 03 shares inert
pattern leaves; 04 inlines value acceptance. New audit-list entries track those
changes without changing guest behavior.

The frozen 17-label panel and exact statements/expected results are in
`.temp/brief-572/perf/labels.json` and its `*-check.rum` and `*-<N>.rum`/`*-<2N>.rum` files, with source hashes. The pattern is inline at depth 8/32;
the depth counts enclosing unary calls above the bottom patterned call. Every
base/candidate result was checked before measurement. Allocation measurement
precedes instruction measurement. Each candidate has 68 allocation processes
and 136 hardware-counter processes: stdin `/dev/null`, one label per process,
`taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, two paired samples,
`(count(2N)-count(N))/N`. Profile flags are off for instructions; allocation
measurement alone sets its allocation flag. Raw stdout/stderr is saved before
reduction in `perf/panels/candidate-0N/`. No timing suite or concurrent CPU-heavy
workload was run. The result checks and all metric processes exited zero.

Tables below are verbatim slope TSVs. Counts and bytes are per-iteration
allocation slopes; instruction columns contain both separately paired base
and candidate slopes; delta columns are percentages. Regression means **both**
deltas exceed +0.05%; a single positive sample does not meet that regression rule.

`perf/candidate-01-slopes.tsv`:

```tsv
label	N	base_count	candidate-01_count	base_bytes_total	candidate-01_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.0	0.0	32.0	32.0	999.13025	998.8372	-0.02933051021125621	998.9338	999.0895	0.015586618452601009	False
g4-ruledelayed-plain	20000	0.0	0.0	32.0	32.0	999.0646	998.9255	-0.013923023596273953	999.0905	999.0544	-0.0036132862838766933	False
g4-rule-pattern-child	20000	25.0	29.0	1400.0	2360.0	22079.02295	24797.82005	12.31393756035748	22079.7879	24799.42675	12.31732325653363	True
g4-ruledelayed-pattern-child	20000	25.0	29.0	1400.0	2360.0	21786.9168	24497.3118	12.440470695697513	21788.5038	24496.4993	12.428551886155681	True
g4-rule-blank-constraint	20000	16.0	18.0	1112.0	1816.0	17184.00865	18276.01175	6.354763444558675	17184.7147	18274.51065	6.34165867181955	True
g4-rule-wrapper	20000	47.0	49.0	2488.0	3256.0	26308.1746	29983.6818	13.970970072549239	26312.61275	29983.4364	13.950813949481322	True
g4-rule-skew-depth-8	1000	73.0	97.0	3256.0	10104.0	59722.234	70267.303	17.656856238833928	59728.486	70274.356	17.656349099489987	True
g4-rule-skew-depth-32	1000	217.0	293.0	9016.0	38904.0	217273.923	249773.48	14.957872786233995	217276.822	249760.681	14.950448327157527	True
matchq-integer-success	20000	1.0	1.0	160.0	160.0	5144.15485	5144.24185	0.0016912399128266031	5144.3945	5144.30215	-0.0017951578169195237	False
matchq-integer-fail	20000	2.0	2.0	161.0	161.0	5266.30585	5266.43185	0.002392568976983256	5266.21135	5266.3684	0.0029822198457818416	False
cases-wide-no-match	1000	33.001	33.001	263279.864	263279.864	2412531.484	2412427.215	-0.004321974684762253	2412395.408	2412432.886	0.001553559581313202	False
cases-pattern-child	20000	35.0	35.0	5744.0	5744.0	39556.5643	39587.3881	0.07792334988001581	39463.2676	39835.67395	0.9436784449141644	True
downvalue-definition	20000	29.0	29.0	3172.0	3172.0	24121.99475	24120.06775	-0.007988559901350722	24122.1417	24149.86235	0.11491786403028392	False
replaceall-simple	20000	5.0	5.0	1352.0	1352.0	11407.72165	11422.8415	0.13254048848572175	11407.5884	11370.94425	-0.3212260884167262	False
dot-three-no-input	20000	596.0	596.0	23504.0	23504.0	254197.40745	254195.75155	-0.0006514228514831188	254201.0215	254196.036	-0.0019612431022442145	False
differences-no-input	20000	13.0	13.0	704.0	704.0	10045.1197	10035.28685	-0.09788683752567362	10030.85555	10035.13675	0.042680307563580655	False
take-multispec-small	20000	109.0	109.0	5856.0	5856.0	49950.2514	49949.08715	-0.002330819099749526	49966.5554	49949.61735	-0.03389877461914548	False
```

`perf/candidate-02-slopes.tsv`:

```tsv
label	N	base_count	candidate-02_count	base_bytes_total	candidate-02_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.0	0.0	32.0	32.0	998.9495	998.96025	0.001076130475063941	998.95905	999.1475	0.01886463714404485	False
g4-ruledelayed-plain	20000	0.0	0.0	32.0	32.0	999.06455	999.10625	0.004173904478954604	999.053	998.83135	-0.022186010151603597	False
g4-rule-pattern-child	20000	25.0	27.0	1400.0	2296.0	22076.98125	24456.5496	10.778504194272465	22077.0562	24454.94615	10.770865139166519	True
g4-ruledelayed-pattern-child	20000	25.0	27.0	1400.0	2296.0	21786.9667	24154.44935	10.866508782978034	21787.01875	24153.14815	10.86027155505156	True
g4-rule-blank-constraint	20000	16.0	18.0	1112.0	1816.0	17184.6667	18278.0802	6.3627274190892535	17185.0292	18276.1299	6.3491349784846385	True
g4-rule-wrapper	20000	47.0	49.0	2488.0	3256.0	26314.87585	29997.32255	13.993783291970207	26308.2003	29995.3791	14.015321298887917	True
g4-rule-skew-depth-8	1000	73.0	79.0	3256.0	9528.0	59722.493	65742.98	10.080769736119333	59720.18	65740.191	10.080363120137958	True
g4-rule-skew-depth-32	1000	217.0	227.0	9016.0	36792.0	217276.621	217154.249	-0.05632083168304325	217273.055	217131.641	-0.06508584324916855	False
matchq-integer-success	20000	1.0	1.0	160.0	160.0	5144.3429	5144.3964	0.001039977331207531	5144.40925	5144.22225	-0.0036350140689145327	False
matchq-integer-fail	20000	2.0	2.0	161.0	161.0	5266.2267	5274.24845	0.15232443373545124	5266.2655	5274.31965	0.15293854819891095	True
cases-wide-no-match	1000	33.001	33.001	263279.864	263279.864	2412430.41	2412485.204	0.0022713194035572926	2412444.951	2412560.897	0.004806161481618609	False
cases-pattern-child	20000	35.0	35.0	5744.0	5744.0	39477.9854	39548.8196	0.1794270890023819	39464.71865	39498.2	0.0848386892021935	True
downvalue-definition	20000	29.0	29.0	3172.0	3172.0	24165.9654	24318.1396	0.6297046175527399	24121.9048	24149.9359	0.11620599713171309	True
replaceall-simple	20000	5.0	5.0	1352.0	1352.0	11407.741	11422.96255	0.13343176357176745	11394.5937	11396.94005	0.020591782925971636	False
dot-three-no-input	20000	596.0	596.0	23504.0	23504.0	254201.0371	254204.0191	0.0011730872674808879	254197.3779	254203.5819	0.0024406231296625336	False
differences-no-input	20000	13.0	13.0	704.0	704.0	10030.8473	10033.2205	0.023659018316424252	10037.7141	10033.1321	-0.045647843267404564	False
take-multispec-small	20000	109.0	109.0	5856.0	5856.0	49950.27915	49948.997	-0.0025668525217792393	49966.1198	49949.1906	-0.03388135814380577	False
```

`perf/candidate-03-slopes.tsv`:

```tsv
label	N	base_count	candidate-03_count	base_bytes_total	candidate-03_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.0	0.0	32.0	32.0	999.0943	998.9897	-0.010469482210040404	998.96435	998.9827	0.0018369023879527546	False
g4-ruledelayed-plain	20000	0.0	0.0	32.0	32.0	998.89265	999.16745	0.027510463711988287	998.98605	999.14505	0.015916138168292626	False
g4-rule-pattern-child	20000	25.0	17.0	1400.0	1816.0	22077.03615	21617.05925	-2.0835083879681027	22077.10645	21618.52115	-2.0771983911868075	False
g4-ruledelayed-pattern-child	20000	25.0	17.0	1400.0	1816.0	21789.22645	21318.57215	-2.1600321658045707	21787.18525	21316.68615	-2.159522189769769	False
g4-rule-blank-constraint	20000	16.0	18.0	1112.0	1816.0	17183.778	18278.44765	6.370366574800945	17184.7518	18279.1516	6.3684353008811145	True
g4-rule-wrapper	20000	47.0	29.0	2488.0	2296.0	26310.1936	24465.51915	-7.011253805445195	26310.37195	24462.99805	-7.021466300479274	False
g4-rule-skew-depth-8	1000	73.0	69.0	3256.0	9048.0	59719.02	63167.665	5.774784984750259	59725.747	63165.733	5.75963662706469	True
g4-rule-skew-depth-32	1000	217.0	217.0	9016.0	36312.0	217270.625	214216.098	-1.4058628496143921	217264.284	214206.088	-1.407592607351893	False
matchq-integer-success	20000	1.0	1.0	160.0	160.0	5144.20685	5144.06945	-0.002670965690265703	5144.26825	5144.32305	0.0010652632665530959	False
matchq-integer-fail	20000	2.0	2.0	161.0	161.0	5266.2732	5274.21205	0.1507489205079704	5266.2562	5274.2787	0.15233782207557667	True
cases-wide-no-match	1000	33.001	33.001	263279.864	263279.864	2412445.472	2412436.049	-0.00039059950200925897	2412388.705	2412434.825	0.0019117980408545066	False
cases-pattern-child	20000	35.0	35.0	5744.0	5744.0	39418.1567	39665.967	0.6286704421163192	39488.1126	39535.0011	0.11874079795852754	True
downvalue-definition	20000	29.0	29.0	3172.0	3172.0	24122.0114	24150.0668	0.11630622146212577	24121.877	24119.98935	-0.007825468971589533	False
replaceall-simple	20000	5.0	5.0	1352.0	1352.0	11394.446	11396.9561	0.02202915350162371	11433.64015	11448.8449	0.13298258297906873	False
dot-three-no-input	20000	596.0	596.0	23504.0	23504.0	254200.91635	254204.42695	0.001381033573899515	254197.46655	254205.10185	0.003003688472436039	False
differences-no-input	20000	13.0	13.0	704.0	704.0	10037.3684	10033.39215	-0.0396144670748555	10052.04045	10033.27855	-0.18664767708927998	False
take-multispec-small	20000	109.0	109.0	5856.0	5856.0	49942.23645	49949.50915	0.014562223314285028	49942.08815	49949.6222	0.01508557266842736	False
```

`perf/candidate-04-slopes.tsv`:

```tsv
label	N	base_count	candidate-04_count	base_bytes_total	candidate-04_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
g4-rule-plain	20000	0.0	0.0	32.0	32.0	999.0214	999.24605	0.022487005783866643	998.83045	999.10265	0.0272518724274029	False
g4-ruledelayed-plain	20000	0.0	0.0	32.0	32.0	998.95125	999.0392	0.008804233439829723	999.03705	999.0967	0.005970749533257269	False
g4-rule-pattern-child	20000	25.0	17.0	1400.0	1816.0	22078.4242	21438.97185	-2.8962771265170217	22077.01755	21434.46125	-2.9105213081646486	False
g4-ruledelayed-pattern-child	20000	25.0	17.0	1400.0	1816.0	21786.95435	21142.4037	-2.958424751094235	21787.06785	21142.5035	-2.958472220482855	False
g4-rule-blank-constraint	20000	16.0	18.0	1112.0	1816.0	17186.0134	18253.128	6.209203816866582	17184.53255	18252.7111	6.215930208703879	True
g4-rule-wrapper	20000	47.0	29.0	2488.0	2296.0	26310.26725	24226.66505	-7.919350192081387	26308.02665	24226.79595	-7.911010307570898	False
g4-rule-skew-depth-8	1000	73.0	69.0	3256.0	9048.0	59727.783	62605.068	4.817330989834323	59721.041	62614.03	4.8441704155826715	True
g4-rule-skew-depth-32	1000	217.0	217.0	9016.0	36312.0	217266.018	212357.402	-2.2592654135171752	217275.71	212347.107	-2.268363546021779	False
matchq-integer-success	20000	1.0	1.0	160.0	160.0	5144.18745	5144.19205	8.942131375366813e-05	5144.3616	5144.3271	-0.00067063714960236	False
matchq-integer-fail	20000	2.0	2.0	161.0	161.0	5266.15555	5274.0904	0.15067633161727212	5266.3552	5274.29045	0.15067821479266552	True
cases-wide-no-match	1000	33.001	33.001	263279.864	263279.864	2412469.451	2412408.173	-0.0025400528895658248	2412444.072	2412438.085	-0.0002481715563718012	False
cases-pattern-child	20000	35.0	35.0	5744.0	5744.0	39467.69655	39496.31255	0.0725048647410853	39464.8592	39498.4093	0.08501259267130745	True
downvalue-definition	20000	29.0	29.0	3172.0	3172.0	24091.9912	24142.08245	0.2079166042531222	24198.0798	24150.0064	-0.1986661768096254	False
replaceall-simple	20000	5.0	5.0	1352.0	1352.0	11394.6278	11423.0928	0.24981070465504018	11433.6304	11383.82465	-0.43560748649003056	False
dot-three-no-input	20000	596.0	596.0	23504.0	23504.0	254200.82475	254204.40785	0.0014095548287462734	254197.2271	254203.77165	0.0025745953544431543	False
differences-no-input	20000	13.0	13.0	704.0	704.0	10030.7498	10035.3195	0.045556913402422516	10051.9555	10035.35905	-0.16510667998879125	False
take-multispec-small	20000	109.0	109.0	5856.0	5856.0	49957.86555	49949.17855	-0.017388653226813044	49950.09995	49949.99295	-0.00021421378557651138	False
```

### 4. Stop-loss, residual attribution and open questions

| Candidate | Remaining target sites | Failing semantic tests | Regressing labels | Worst both-sample delta | No-progress streak |
| --- | --- | --- | --- | --- | --- |
| candidate-01 | 0 | 0 | 7 | 17.656349099% | 0 |
| candidate-02 | 0 | 0 | 8 | 13.993783292% | 0 |
| candidate-03 | 0 | 0 | 4 | 6.368435301% | 0 |
| candidate-04 | 0 | 0 | 4 | 6.209203817% | 0 |

The separate post-measurement test-output repair reduced broad runtime failures
from one to zero without changing the measured production candidate.

Four of the allowed eight candidates were measured. No-progress streak is
zero: every step improved at least the worst both-sample regression, and
candidate 03 reduced the number of regressing labels. The cap and two-attempt
stop-loss were **not** exhausted. Tuning ends with the scoped evidence below;
this is not a claim that the performance gate passes or that four more
candidates were attempted.

Final candidate 04 still fails four labels:

| Label | Sample 1 | Sample 2 | Allocation count / bytes, base → final |
| --- | ---: | ---: | --- |
| `g4-rule-blank-constraint` | +6.209204% | +6.215930% | 16 → 18 / 1,112 → 1,816 |
| `g4-rule-skew-depth-8` | +4.817331% | +4.844170% | 73 → 69 / 3,256 → 9,048 |
| `matchq-integer-fail` | +0.150676% | +0.150678% | 2 → 2 / 161 → 161 |
| `cases-pattern-child` | +0.072505% | +0.085013% | 35 → 35 / 5,744 → 5,744 |

One Callgrind attribution pass ran on candidate 03's same four residuals,
N=10/20, base/candidate, sixteen bounded passing processes. It was used to
choose candidate 04's single inlining change, not to replace the hardware
gate. Every reduced self-cost sum equals the raw profile total. Raw files and
function details: `.temp/brief-572/attribution/candidate-03/`. No second
attribution pass was run on candidate 04.

```tsv
label	base_total	candidate-03_total	base_rule_self	candidate-03_rule_self	candidate-03_continuation_self
g4-rule-blank-constraint	17297.3	18340.5	1227.0	1955.0	0
g4-rule-skew-depth-8	59522.2	63472.6	11116.0	13672.0	0
matchq-integer-fail	5235.0	5078.3	0.0	0.0	0
cases-pattern-child	39677.6	39917.3	1046.0	1046.0	0
```

For Blank, candidate-03 driver self cost is 562 and value acceptance 234
instructions/iteration, versus 278 in the removed recursive helper;
completion contributes 96 and work-Vec drop 58. Together with classifier and
allocation/vector-growth differences, the broader definitions self-cost rises
1,227 → 1,955 (+728). At depth 8, driver self is 4,725 and acceptance 3,456;
the old recursive helper was 4,483 and classifier cost falls 5,867 → 4,531.
Definitions self-cost rises 11,116 → 13,672 (+2,556), with vector growth,
reallocation and atom-clone costs outside that subtotal. Both ordinary labels
show zero cold-continuation self cost. Candidate 04's inline delivery improves
the hardware deltas but leaves the stack/transition cost positive.

The two neighbor profiles do not execute the new walk. Cases' definitions
subtotal stays 1,046; the profile adds 221.9 in `intern_symbol_text` and 100.4
in `Symbol::system`, offset by other interning/hash changes. MatchQ's small
Callgrind slope actually decreases (5,235 → 5,078.3), with differences in
unchanged interning, hash maps and dispatch; it does not reproduce the hardware
positive. These neighbors remain unresolved failures. They are not dismissed
as noise, and this evidence does not justify editing the fenced matcher/Expr
or claiming the walk caused every unchanged-code instruction difference.

The remaining design question is how to remove short-rule work-stack and
transition overhead while retaining the prescribed immutable suspension state,
and how to explain the two small neighbor deltas with a separately scoped
investigation. Deep held-pattern resolution, matcher compilation, parsing and
last-owner Expr teardown are known adjacent boundaries, not fixed or newly
certified here. No unsafe shortcut, recursive fallback, depth cap, default-path
diagnostic instrumentation or out-of-scope repair was added. The director has
an uncommitted, semantically verified repair with explicit performance defects;
it should not be treated as performance-accepted.
