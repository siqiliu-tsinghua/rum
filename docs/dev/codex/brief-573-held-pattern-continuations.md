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
