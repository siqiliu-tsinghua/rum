# brief-570 — design: rule and held-pattern continuations (group G4)

`Scope: NONE — no production change.` Design-only, with one declared scratch
test (see Scope). **Do not run `git` in any form**, including before you have
read this brief.

## Goal

Group G4: rule left-hand sides and held patterns are normalized by evaluating
their ordinary subexpressions while keeping pattern constructs held, and a
subexpression that asks for input resumes **past** that work. brief-548
recorded sixteen sites (`.temp/brief-548/draft-G4.md` has the list):

- `definitions/definitions.rs:85, 108` — `eval_rule` / `eval_rule_delayed` →
  `evaluate_rule_lhs`; `:177, 187, 195, 204, 217, 222, 227, 229` — the
  recursive `evaluate_pattern_rule_lhs` and its `evaluate` calls;
- `kernel_data/rewrite_data/cases/match_q.rs:50, 68, 71, 83` —
  `resolve_matcher_pattern_expr_owned`, the recursive
  `resolve_match_pattern_subexprs`, `normalize_cases_spec`;
- `pattern_matching/core.rs:18` (`match_expr` → `match_expr_owned`) and
  `pattern_matching_interpreted.rs:320` (`match_expr_interpreted` →
  `…_owned`) — **both callers carry `#[allow(dead_code)]`**.

The director re-ran brief-548's probes on `f127e786`
(`.temp/director/brief548-probe-rerun-f127e786.txt`): 51 still diverge, among
them every G4 witness (`Rule-pattern`, `RuleDelayed-pattern`,
`MatchQ-pattern`, `Cases-pattern`, `Rule-twice`, `MatchQ-pattern-twice`, …).
Lost continuations today: 59; G4 accounts for sixteen.

## What the director's adjacent-boundary probes found

Release `rum-cli`, one codegen unit, source identical to HEAD
(`.temp/brief-569/candidate-09/rum-cli`), ruled 2 MiB stack, `v` a nested
list built incrementally by `Do[v = {v}, {n}]` (probe files in
`.temp/director/gate-g4/`):

| Program | n = 1 500 | n = 6 000 |
| --- | --- | --- |
| `Rule[v, 1]` | passes | passes |
| `MatchQ[1, v]` | `False` | **process abort (exit 134)** |
| `Cases[{1, 2}, v]` | `0` | **process abort (exit 134)** |
| `ReplaceAll[v, 1 -> 2]` | passes | fails closed (`StackLimitExceeded`) |

and, from the G3 probes, `MatchQ[v, w]` on two deep literals aborts between
800 and 1 200 levels. `v` here contains **no pattern syntax**, so
`resolve_match_pattern_subexprs` returns at its first `contains_pattern_syntax`
test; the abort is somewhere else in the MatchQ / Cases pipeline —
`contains_pattern_syntax`, `normalize_match_expr`, matcher compilation, or the
match itself are all candidates. **Do not run any probe that aborts a
process** — Codex's upstream content filter cut brief-567's first run at
exactly that step. Locate the recursion by reading the source; the director
runs any abort reproduction.

## What the design must settle

1. **The inventory.** Every function on the pipelines `Rule`/`RuleDelayed`
   left-hand side, `MatchQ`, `Cases` (and any other consumer of
   `resolve_matcher_pattern_expr_owned` / `evaluate_rule_lhs`) that recurses in
   proportion to the pattern's or value's depth, with `path:line`, whether it
   can run guest code, whether it is guarded, and which brief-548 site it owns.
   Name the most likely site of each abort above, with reasoning from source.
2. **The two dead-code sites.** Are `match_expr` and `match_expr_interpreted`
   reachable from any production path, any test, or any public or
   `#[doc(hidden)]` API? If not, the right repair is removal, not a
   continuation — say so with evidence, and say what removing them does to the
   census and the inventories.
3. **Ownership of depth.** Which recursive walkers G4 rewrites
   (`evaluate_pattern_rule_lhs`, `resolve_match_pattern_subexprs`, …) and
   therefore must make depth-safe — the G2/G3 precedent: a repair that rewrites
   a walker makes it resumable **and** depth-safe in one design, enforced by
   the recursion scanner as `lazy_depth_contract.rs` and
   `schema_depth_contract.rs` do — and which belong to the matcher proper
   (compile, normalize, match), whose depth defect the director has recorded
   for the hardening line. Draw the boundary from evidence, and say what a G4
   depth test can and cannot prove given the matcher's own limit.
4. **The design** for the G4-owned walkers: representation (an explicit work
   stack, or another shape — choose and justify against the guarded
   alternative), what one frame owns (the partly rebuilt expression, the
   cursor, held positions, pending subexpression), apply-once on resume, a
   second suspension, abandonment, and how `eval_rule`, `eval_rule_delayed`,
   `MatchQ`, `Cases` and any other consumer complete. Follow brief-563/568's
   protocol: immutable Session-rooted frames, a cold working copy on an actual
   resume, capture only on `InputRequired` on the error edge,
   `FrameRootTrace`/`PureFrameAbandon`, no `Session` borrow, observer or
   `NativeStackDepth` in a frame, rum-runtime's frame lists.
5. **The ordinary path.** This is the evaluator's pattern pipeline and is hot.
   Name the labels (existing `matchq-*`, `cases-*`, `replace*` and rule
   definition labels, plus new ones with pattern-bearing subexpressions to
   evaluate) and the plan; **both arms with one codegen unit**
   (`TEMPLATE.md` pitfall 8).
6. **Inventory, census and layout consequences**, listed so the director can
   authorize them by name.

## What I want back

1. The recursion inventory and the dead-code finding, with evidence.
2. The design, frame inventory, and the depth boundary.
3. Red tests, run in the scratch test and recorded: every brief-548 G4
   witness through the rum-vs-rum oracle (value, `log`, messages, prints,
   suspension asserted, exact prompts, no owner after, `1 + 1`), twice
   variants, an ordinary error after a resume, cancellation at a suspended
   subexpression, a nested pattern (`h[g[x_, (input)], y_]`), `Condition` and
   `PatternTest` positions that must stay held. Depth cases only at depths the
   current tree survives, as controls; write the post-fix deep cases unrun.
4. Drafted implementation brief(s) in `TEMPLATE.md` shape, with red tests,
   predicted lost-continuation count, the named inventory rows, the structural
   depth checks with falsifiers, the one-unit performance gate, the standing
   duty, and serial runs (`env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`).
5. Open questions for the director.

## Context pointers

- brief-548's report, `.temp/brief-548/draft-G4.md`, `probes.json`,
  `lost.tsv`, `probe.rs`.
- briefs 563, 568 and their reports (the accepted flat-walk pattern and its
  structural depth checks); briefs 566 and 569 (ordinary-path tuning, one
  codegen unit).
- `crates/rum-runtime/src/runtime/definitions/definitions.rs`,
  `kernel_data/rewrite_data/cases/match_q.rs`, `pattern_matching/`,
  `pattern_matching_interpreted.rs`, `suspension/core.rs`,
  `native_stack_guard.rs`.
- `docs/dev/codex/TEMPLATE.md`, `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, except: append your report to this file;
  scratch under `.temp/brief-570/`; **one** scratch integration test,
  `crates/rum-stdlib/tests/zz_brief570_probe.rs`, **deleted before the report
  is appended**, its final content copied to `.temp/brief-570/probe.rs`.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- No network; nothing outside this repository; all scratch and `TMPDIR` under
  `.temp/brief-570/`.
- **Run no probe that aborts a process**, in-process or through the CLI.
- Commands, serial, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief570_probe`
  (150 s); `.temp/brief-569/candidate-09/rum-cli` on passing programs (60 s);
  `valgrind --tool=callgrind` on it for attribution (150 s). File reads,
  `rg`/`grep`, `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch, the deleted test.
2. The recursion inventory, the abort-site reasoning, the dead-code finding.
3. The design, frame inventory, and the depth boundary.
4. The red tests with their pre-fix values.
5. The drafted implementation brief(s).
6. Open questions for the director.

---

## Report — brief-570

### 1. What changed

**Design only. No production code, inventory, dependency or public behavior changed.** Work products are under [.temp/brief-570/](../../../.temp/brief-570/). The one scratch integration test, `crates/rum-stdlib/tests/zz_brief570_probe.rs`, was copied to [probe.rs](../../../.temp/brief-570/probe.rs) and **deleted before this report was appended**. No git command, network request, CLI/performance run or process-aborting probe was run. Cargo runs were serial, offline, single-job/single-test-thread, bounded at 150 seconds, with TMPDIR/CARGO_TARGET_DIR inside the scratch root.

Artifacts: [recursion/dead-code audit](../../../.temp/brief-570/recursion.md), [208-row function inventory](../../../.temp/brief-570/recursion-inventory.tsv), [frame/algorithm design](../../../.temp/brief-570/design.md), [inventory authorization plan](../../../.temp/brief-570/inventory-plan.md), [ordinary-path gate plan](../../../.temp/brief-570/performance-plan.md), [full red evidence](../../../.temp/brief-570/red-tests.md), [unrun depth cases](../../../.temp/brief-570/post-fix-depth.md), and three implementation drafts below. SHA-256 comparison of 1,748 pre-existing files under crates/docs/scripts found no changed existing file before this append. This checks the allowed edit scope; it makes no version-control claim about unrelated prior changes.

### 2. Recursion inventory, abort reasoning and obsolete adapters

The full TSV records identified depth-recursive functions on the reviewed G4/consumer/matcher paths with path:line, guest-code capability, guard, owner and frozen brief-548 ID. It distinguishes adjacent width recursion and implicit operations. The scratch AST audit read 362 runtime/core files, 4,436 functions, 95,539 calls/references and 1,988 macros, with no macro parse failures. The resolved source graph reached 840 functions and found 50 recursive components / 188 members; twenty manual codec/comparator/Expr/derived-ownership entries supplement them. This is source-audit evidence, not typechecked whole-program call resolution: guest evaluator reentry is an explicit boundary, and macro/operator/derive edges require manual review. Qualified Vec/Option codec calls were checked to remove false same-name cycles.

Runtime paths below are relative to `crates/rum-runtime/src/runtime/`.

| G4 boundary | Source and current defect | Frozen IDs |
| --- | --- | --- |
| Rule callers | definitions/definitions.rs:85,108; LHS completion is lost | P0342, P0344 |
| Rule walk | evaluate_rule_lhs :163 and evaluate_pattern_rule_lhs :171; pending work :177,187,195,204,217,222,227,229; guest-capable unguarded recursion | P0350–P0357 |
| Rule classifier | definitions/definition_lhs_preparation/syntax_policy.rs:111; pure unguarded recursion | Supports rule sites |
| Held resolver | kernel_data/rewrite_data/cases/match_q.rs:46-75; normalization :50, head :68, args :71; guest-capable unguarded recursion | P0774, P0776, P0777 |
| Held classifier/strip | match_q.rs:177 and substitution/function_and_subst_helpers.rs:23; pure unguarded recursion | Supports held sites |
| Obsolete adapters | match_q.rs:83, pattern_matching/core.rs:18, pattern_matching_interpreted.rs:320 | P0779, P1019, P1285 — retire |

Matcher depth remains separate. `matcher_and_symbols.rs:3,82,93,116,148,292,449` recursively compiles nested calls/patterns; :388 validates repetition bounds; :530,545 recurse over compiled argument groups. `pattern_matching/core.rs:605,642` recursively scans binding names before matching (:40). Nested matching follows :34 -> :504 -> :256 -> :320 -> sequence/entry.rs:4 -> sequence.rs:266 -> core.rs:34, plus the condition/test/alternative/key-value/sequence branches enumerated individually in the TSV. These paths can run guest code and lack a blanket native guard; only some orderless choice paths use logical MAX_MATCH_DEPTH=512. Matcher codecs and derived Clone/root-trace/drop of MatcherNode/MatcherArg and companion types are also recursive.

`resolution/match_normalization.rs:69,104` already has a 16-level recursive front and explicit-stack fallback :129. Its Orderless comparator (`expr_runtime_helpers.rs:113,155,176,236`) and shared Expr equality/Hash/Debug/Display/last-owner drop are distinct unguarded boundaries. Cases/Count/Position traversal is explicit-stack; DeleteCases and ReplacePart application/exact-path traversal have bounded fronts. Association decoding, ReplacePart planning, option/spec parsers and RHS scans are catalogued separately. The generic evaluator's recursion/native-byte checks at eval_core/mod.rs:49-58 do not guard a helper that never reenters evaluate.

The director's deep-literal observations were **not reproduced**:

- `Rule[v,1]` passing at n=6,000 bypasses the structural rule walker: raw v is classified as non-pattern and evaluated once; the returned literal is not revisited.
- `MatchQ[1,v]` sees a shallow symbol during preflight/resolution. After bounded normalization, **unguarded literal-call matcher compilation is the leading abort candidate**. Binding-name capture and compiled-tree teardown are additional candidates even though scalar 1 rejects a Call matcher immediately. Source does not identify a unique faulting instruction.
- `Cases[{1,2},v]` takes the symbol branch at definitions/definition_catalog/rewrite_rules.rs:149, bypassing the held resolver entirely, then shares those compiler/capture/drop risks.
- `MatchQ[v,w]` additionally takes the unguarded nested matching cycle with several Rust activations per level, plausibly explaining the earlier 800–1,200 boundary. This is inference, not measured attribution.
- `ReplaceAll[v,1->2]` has a shallow rule and explicit traversal (definitions/application/replace_all.rs:176); its reported StackLimitExceeded is consistent with generic evaluation/native-budget enforcement. No exact throwing edge was instrumented, and it proves nothing about G4 depth.

The two allow(dead_code) matcher wrappers have **one test consumer**, rather than being wholly uncalled: tests/definitions_and_rules_part2.rs:47,50 invokes them and :58-60 compares bindings. They have no production or public/doc(hidden) path: the compiled wrapper is pub(in crate::runtime), the interpreted wrapper pub(super) in the cfg(test) module at runtime/mod.rs:48-49. Migrate the test to owned outcomes `.matched`/`.bindings`, then delete both wrappers; preserve the owned interpreted suspension test :66-120. `normalize_cases_spec` has no caller in crates/scripts; production Cases already resolves/compiles in separate framed phases. Delete it too. [caller-search.txt](../../../.temp/brief-570/caller-search.txt) records the uses and unrelated AST-visitor names.

### 3. Design, frame inventory and depth boundary

Choose **two separate explicit work-stack drivers**, plus iterative classifiers and unary HoldPattern stripping. A native guard cannot reconstruct suspended caller work and NativeStackDepth cannot be stored in a frame. A bounded recursive front would add a second capture/control scheme; start with flat stacks and measure the cost.

Each work item retains original head/shared arguments, resolved head, rebuilt prefix and next-child cursor; rule state also retains original wrapper policy. Rule visits distinguish Classify, Structural and EvaluateOnly. Pattern inner, Condition/PatternTest first operand and wrapper arguments descend structurally even when the child contains no pattern syntax; Blank constraints evaluate normally. Binder/test/delayed-RHS positions remain held. Ordinary rule calls are rebuilt without applying them. The held resolver instead resolves head/args and applies a prepared call once. Preserve current OptionsPattern/Verbatim/nested HoldPattern distinctions; do not reuse Set-LHS semantics.

| Proposed FrameRootTrace types | Owned work |
| --- | --- |
| RuleLhsState, RuleLhsWork, RuleLhsPolicy, RuleLhsDestination, RuleLhsAwait | Flat parent stack, original holding policy, cursor/destination and pending ordinary expression. |
| RulePatternSuspensionFrame | Immutable state/await, cold working copy on actual successful resume. |
| RuleLhsCompletionFrame | Raw RHS and immediate/delayed flag; immediate RHS uses existing EvalRuleSuspensionFrame, delayed RHS remains held. |
| MatcherPatternState, MatcherPatternWork, MatcherPatternDestination, MatcherPatternAwait | Flat parent stack; distinguish ordinary Evaluate from submitted PreparedCall. |
| MatcherPatternSuspensionFrame | Complete resolver and normalize its root exactly once, then existing consumer suffix. |

Twelve root-traced types; the three named frames also derive PureFrameAbandon. Add exactly those types to runtime/tests/frame_root_trace/derived_inventory.rs; frame_root_trace.rs:483 predicts named count 556 -> 558 -> 559. Existing frames retain identity.

Capture only on InputRequired's error edge, moving live state into an immutable Session-rooted frame. On actual Ok resume make the cold working copy, insert the answer once, advance the cursor once and continue; Err forwards without copying. A second suspension freezes updated state and the generic driver appends the untouched outer suffix once. No Session borrow, observer, token, closure state, mutable Arc interior or Arc::try_unwrap. Pure abandon executes no sibling, pending call, held guard, RHS or normalization; no new cancellation-checkpoint policy.

When submitting a matcher prepared call, pop its work record and **move** args into the existing call evaluator. Its lower frames own submitted args if it suspends; G4 keeps only the parent stack/destination. The resumed answer is the completed call value, never a request to invoke it again. Working-copy overlap is real and does not discharge a retention obligation.

Existing frames already complete MatchQ, Cases, DeleteCases, Count/MemberQ/FreeQ/Position, both Catch paths and guarded/unguarded ReplacePart. Exact edges/frames are in recursion.md. Only the two live Rule entry callers lack LHS completion; rewrite-rule consumers inherit that failure. Reap is a passing control, not a direct resolver consumer.

G4 owns its driver/classifier/strip/rebuild/completion code and flat state copy/trace/drop/abandon graph. It does **not** repair matcher compile/match/codecs, compiled-tree ownership, generic evaluator or shared Expr depth. Structural gates scan new modules individually/combined and extract the exact shared helpers, assert expected functions/files and audit outbound calls. Falsifiers cover direct/mutual/callback-hidden recursion, an unscanned helper, self-reaching state and missing root sentinels. Preserve lazy/schema checks; scalar root sentinels avoid adding Expr recursion to the state witness.

Only n=16 end-to-end controls ran. Proposed n=2,000 private walker cases and 20,000 scalar-only work records are [written unrun](../../../.temp/brief-570/post-fix-depth.md), with [draft Rust builders](../../../.temp/brief-570/post-fix-depth.rs). The matcher fixture collapses prepared levels to scalars to avoid deep compilation; AST build/check/teardown are iterative in a ruled 2 MiB thread. Director calibration precedes enabling them. An inherited limit/abort is blocked evidence, not green or permission to lower n silently. Known deep-literal .rum files remain director-only hardening probes.

Layout: definitions/rule_lhs.rs and cases/pattern_resolution.rs, each with state/continuation/tests children; drivers target <10k chars and state/continuation <6k where practical. Current definitions.rs (25,061 chars) becomes smaller; core.rs (24,397) and interpreted.rs (32,285) only lose wrappers, without unrelated splitting. Tests belong under stdlib/tests/rule_pattern_continuations and association_depth_contract; no public API or runtime-root file.

[Inventory-plan.md](../../../.temp/brief-570/inventory-plan.md) names forecasted changes: remove rule map closures unknown_captures.tsv:403/404 and reconstruction macros unknown_macros.tsv:288-290; relocate existing RHS/delayed-construction rows :283/284 and matcher registry closure unknown_captures.tsv:852. Other old rows retain identity/status with replayed positions. New derives/macros/closures stay UnknownConstruct. N:Rule/RuleDelayed and N:Count/DeleteCases/FreeQ/MatchQ/MemberQ stay N/Pending; only moved source evidence changes. Each implementation draft carries TEMPLATE's replay duty for 68 contract fingerprints, promoted certificates in both feature builds, ownership discovery and frame closed lists. No production inventory replay was run today.

### 4. Red tests and pre-fix values

Two scratch runs compiled and exited 101 due to red equality assertions; the second is final. Exact commands/exit statuses and verbatim counts:

```text
Commands (serial, no production tests):
TMPDIR=/home/remi/dev/rum/.temp/brief-570/tmp python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-570/target RUST_TEST_NOCAPTURE=1 cargo test -p rum-stdlib --test zz_brief570_probe > .temp/brief-570/probe-run-01.txt 2>&1
exit: 101
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 03s
test result: FAILED. 14 passed; 48 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
TMPDIR=/home/remi/dev/rum/.temp/brief-570/tmp python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-570/target RUST_TEST_NOCAPTURE=1 cargo test -p rum-stdlib --test zz_brief570_probe > .temp/brief-570/probe-run-02.txt 2>&1
exit: 101
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.35s
test ast_inventory ... AST files=362 functions=4436 calls=95539 macros=1988 frame_records=721 macro_parse_failures=0
test result: FAILED. 15 passed; 50 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.48s
```

Final breakdown: **56 queued/resumed comparisons, 47 divergent and nine equal controls**; three first-input cancellations pass; three second-input cancellation tests fail because input two is never reached; answer-free abandon, n=16 depth and AST audit pass. Total **50 failed / 15 passed**. The harness asserts actual suspension, exact pending identity and prompt sequence, compares structured value/error/termination/messages/prints and log, and verifies no pending owner plus successful 1+1 on each fresh session before equality assertions.

All fourteen original G4 witnesses retain brief-548's exact source and responses ([provenance check](../../../.temp/brief-570/fixture-provenance.tsv)). Values/logs below are decoded directly from final raw rows; prompts are actual/expected.

| Witness | Resumed value; log | Queued value; log | Prompts |
| --- | --- | --- | --- |
| `Rule-plain` | `"two"`; `List[1]` | `Rule["two", 9]`; `List[1, 3]` | 1/1 |
| `Rule-pattern` | `"two"`; `List[2]` | `Rule[h[Pattern[x, Blank[]], "two", 3], 9]`; `List[2, 3]` | 1/1 |
| `RuleDelayed-plain` | `"two"`; `List[1]` | `RuleDelayed["two", CompoundExpression[AppendTo[log, 3], 9]]`; `List[1]` | 1/1 |
| `RuleDelayed-pattern` | `"two"`; `List[2]` | `RuleDelayed[h[Pattern[x, Blank[]], "two", 3], 9]`; `List[2, 3]` | 1/1 |
| `Rule-Blank` | `Integer`; `List[2]` | `Rule[Pattern[x, Blank[Integer]], 9]`; `List[2]` | 1/1 |
| `Rule-PatternTest` | `Integer`; `List[2]` | `Rule[PatternTest[Pattern[x, Blank[Integer]], IntegerQ], 9]`; `List[2]` | 1/1 |
| `Rule-Condition` | `Integer`; `List[2]` | `Rule[Condition[Pattern[x, Blank[Integer]], True], 9]`; `List[2]` | 1/1 |
| `Rule-wrapper` | `Integer`; `List[2]` | `Rule[Alternatives[Pattern[x, Blank[]], Blank[Integer]], 9]`; `List[2]` | 1/1 |
| `Rule-head` | `h`; `List[2]` | `Rule[h[Pattern[x, Blank[]]], 9]`; `List[2]` | 1/1 |
| `Rule-twice` | `"two"`; `List[2]` | `Rule[h[Pattern[x, Blank[]], "two", "four"], 9]`; `List[2, 4]` | 1/2 |
| `MatchQ-pattern-args` | `False`; `List[2]` | `True`; `List[2, 3]` | 1/1 |
| `MatchQ-pattern-head` | `False`; `List[2]` | `True`; `List[2]` | 1/1 |
| `MatchQ-normalize-head` | `False`; `List[1]` | `True`; `List[1]` | 1/1 |
| `MatchQ-pattern-twice` | `False`; `List[2]` | `True`; `List[2, 4]` | 1/2 |

Added witnesses cover direct resolver families, rewrite-rule consumers, distinct two-prompt Rule/RuleDelayed/MatchQ/Cases, nested h[g[x_,input],y_], two inputs inside one child, LHS then RHS, held Condition/PatternTest/HoldPattern/delayed RHS, and a nested prepared call which itself suspends. Selected observations:

- Cases-pattern: resumed List[] / log List[2], queued List[h[1,2,3]] / log List[2,3]. FreeQ returns False on both sides but skips log entry 3 after resume; bool-only assertions would miss it.
- Nested prepared-call MatchQ: False versus True with identical List[1,2] logs; only parent completion is lost.
- Guarded/unguarded ReplacePart resolver: resumed List[List[1,2],9], queued List[List[1,9],List[3,9]], both log List[2]. HoldPattern makes these fixtures exercise resolver work rather than only RuleDelayed preparation.
- Error after resume in Rule/MatchQ/Cases: resumed 2/False/List[] and log List[1], one probe message, print prefix; queued uncaught Throw["g4 boom"], log List[1,2], two probe messages then Throw::nocatch, prints prefix then suffix. Tests compare full structured errors and ordered outputs.
- Rule-one-child-two-inputs reaches both prompts but loses parent and final sibling: resumed 3 / List[1,2,3], queued Rule[h[Pattern[x,Blank[]],3,4],9] / List[1,2,3,4].
- First-input cancellation and answer-free abandonment return TerminatedEvaluation["Cancelled"], Some(Cancelled), prefix-only effects, no owner and successful subsequent 1+1. Second-input cancellation cases fail on prompt reachability; this is not evidence of a separate cancellation defect.

Depth controls ran on a 2 MiB thread at **n=16 only**: Head[Rule[v,1]] -> Rule; MatchQ[1,v] -> False; Length[Cases[{1,2},v]] -> 0; MatchQ[v,v] -> True. The thread owned session creation through teardown.

Every recorded value/log/error/message/print is in [red-results.tsv](../../../.temp/brief-570/red-results.tsv); exact prompts in [prompts.tsv](../../../.temp/brief-570/prompts.tsv); source/answers/setup in [cases.json](../../../.temp/brief-570/cases.json). Full raw output is probe-run-02.txt. The first exploratory run included unsuitable noncatalog SequenceCases/SequenceReplace fixtures; those were removed from the final suite and replaced by actual resolver consumers. Its raw log is retained for disclosure, not contract evidence. No assertion was weakened to claim green.

### 5. Drafted implementation briefs and performance gate

Complete TEMPLATE-shaped raw material for the director's rewrite, not dispatch authorization:

| Draft | Result and scope | Predicted Lost |
| --- | --- | --- |
| [A — retire obsolete wrappers](../../../.temp/brief-570/draft-A-retire-obsolete-wrappers.md) | Migrate parity test to owned outcomes, remove three adapters, replay positions/fingerprints. | 59 -> 56 |
| [B — rule-LHS continuations](../../../.temp/brief-570/draft-B-rule-lhs-continuations.md) | Flat rule walk/classifier and caller completion; 27 red comparisons, lifecycle/depth/structural gates; seven trace types/two frames. | 56 -> 46 |
| [C — held-pattern continuations](../../../.temp/brief-570/draft-C-held-pattern-continuations.md) | Flat held walk/classifier/strip and preserved consumer suffixes; 29 added comparisons (20 red/nine controls); five trace types/one frame. | 46 -> 43 |

The 59 baseline comes from the director's brief. These are predicted updates to frozen IDs, not a fresh lost-site census. Preserve brief-548 snapshots. Completing A/B/C removes G4's sixteen entries through **thirteen live repairs and three retirements**. Other Lost groups and matcher hardening remain separate.

[Performance-plan.md](../../../.temp/brief-570/performance-plan.md) requires **CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 on both arms**, identical toolchain/features/source paths, serial runs, profiles off, exact statements and explicit scratch outputs. Run the full core-language-microbench.sh A/B and compare every label's rum_seconds, including same-tier neighbors. Display matchq-integer-*, matcher-single-*, cases/count/position/deletecases, replaceall/replace/replacepart, rule-rhs and downvalue labels. Add identical no-input rule plain/delayed/pattern-child/Blank/wrapper and held active-child/head/prepared-call labels, plus depth-8/depth-32 inline patterns to expose repeated scans. Hiding the whole pattern behind a symbol would bypass the target walk.

Fine instruction slopes use (count(2N)-count(N))/N with TEMPLATE's hardware event when authorized; wall timings do not replace a required instruction gate. No extra Set/Print/sink inside a timed statement. Retain raw counts/env/binary identity, bound best-of-two wall A/B and two instruction samples, cap candidates/focused/broad runs, and stop after two compiling attempts move none of the semantic/depth/performance counters. Inherited measured command bounds and the remaining full-suite calibration decision are explicit. No performance measurement or cost claim was made today.

### 6. Open questions for the director

1. Accept the serial A/B/C split and named retirement/inventory exceptions when rewriting the briefs? Recommendation: migrate the parity test and retire all three adapters; do not retain a borrowed mutable-bindings compatibility path.
2. Accept G4 ownership of both walkers, both classifiers and unary strip, with compiler/matcher/codec/Expr hardening separate? Source supports this split; exact abort-IP attribution remains unmeasured. Enable proposed n=2,000 private cases only after safe calibration of construction, callback and teardown using the specified shallow matcher collapse.
3. Supply the immutable one-unit base/export per stage, measured parent bound for the full micro-suite, and authorized instruction-counter command/noise rule. Historical candidate-09 must not silently substitute for a later changed base. Missing decisive hardware evidence must be reported.
4. Confirm narrow guide clarifications and shared scanner additions (callback/outbound-helper falsifiers) in the issued scope. Preserve current rule-versus-held policies, including OptionsPattern and nested HoldPattern differences; any intentional harmonization is a separate semantic decision.

No implementation or broad matcher-hardening work was started. Design deliverables are complete; the director owns rewrite, calibration and the authoritative gate.
