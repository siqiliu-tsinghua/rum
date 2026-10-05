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
