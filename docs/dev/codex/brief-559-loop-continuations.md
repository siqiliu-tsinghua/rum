# brief-559 — loops that resume past their own remaining work: Scan, OptionValue, Differences, Dot, ReplaceRepeated

Implementation brief. **Do not run `git` in any form — not `status`, not
`diff`, not `log` — at any point, including before you have read this brief.**
The director owns git.

## Goal

Step 4 of the suspension-continuation branch: the first of the brief-548 repair
groups. brief-548 found 86 sites where a builtin suspends for input in the
middle of its work and resumes **past** that work. This brief takes the three
groups that share one mechanism — **an outer loop whose inner operation resumes
correctly, but whose own cursor, completed prefix and final wrapping are not
carried by any frame**:

| Group | Site | Builtin |
| --- | --- | --- |
| G1 | `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:452` (`evaluate_scan_parts`, an identity frame at `:464-479`) | `Scan` |
| G6 | `crates/rum-runtime/src/runtime/options/builtins.rs:265` (`resolve_option_value_query` → `resolve_option_value_scalar`) | `OptionValue[f, {a, b, c}]` |
| G8 | `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:149` (`resume_replace_repeated_loop` → `apply_replace_all_with_plan`) | `ReplaceRepeated` |
| G8 | `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:50` (`eval_dot` → `compute_inner`) | `Dot` with three or more operands |
| G8 | `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:27` and `:80` (`eval_differences` → `successive_differences` → `difference_expr`) | `Differences` |

When you are done, each of these builtins, suspended for input at any point in
its loop and resumed, produces exactly what it produces when the answers are
queued up front — same value, same side effects, same messages — and brief-548's
confirmed count of lost continuations falls from 86 to 80.

The director re-ran brief-548's 195 probes on `6614e59b`: all 91 divergences
still diverge and all 104 controls still agree (`.temp/director/g-reprobe/`).

## The oracle

The resumed path equals the queued-input path (`Session::queue_input_response`)
in value, the `log` of side effects, and messages. A test proves nothing unless
it asserts that the suspension actually happened. Suspend in the **middle** of
the loop, and suspend **twice**. brief-550 built a permanent harness of exactly
this shape in `crates/rum-stdlib/tests/guest_loading_resume.rs`; reuse its
shape. If a shared helper would serve these and the later groups, you may
extract one under `crates/rum-stdlib/tests/` and disclose it; do not change
what brief-550's tests assert.

## Red tests first

The witnesses and their pre-fix outcomes are in `.temp/brief-548/probes.json`
and `.temp/brief-548/lost.tsv`; the drafts are `.temp/brief-548/draft-G1.md`,
`draft-G6.md` and `draft-G8.md`. At minimum, as permanent tests, each red
before the change:

- `Scan` and `Scan-twice` — resumed `"two"` with `log {1, 2}` against queued
  `Null` with `log {1, 2, 3}`;
- `OptionValue-many` and `OptionValue-twice` — resumed `"two"` against queued
  `{1, "two", 3}`;
- `ReplaceRepeated-predicate` and `ReplaceRepeated-twice` — resumed
  `{h[1], h[2], h[3]}` against queued `TerminatedEvaluation["IterationLimit"]`
  (the iteration limit must still terminate after a resume);
- `Dot-chain` and `Dot-twice` — resumed `{{4, 6}, {4, 6}}` against queued
  `{{12, 18}, {12, 18}}`;
- `Differences`, `Differences-twice`, `Differences-order-two` — resumed `1`
  against the queued difference lists.

Add for each: an ordinary error raised after a resume still behaves as it does
queued; cancellation while suspended in the middle runs no remaining iteration
and leaves the session usable (`1+1` is `2`). Save the failing output under
`.temp/brief-559/red/` before any production change.

## Design constraints

- **Frames carry the loop.** Each driver's frame holds what the loop needs to
  continue — the operator or rules, the remaining parts or operands, the next
  index or iteration count, the completed prefix, and the final wrapping — and
  its `resume` consumes the resumed value and continues the loop. Initial
  execution and resumption share the same step function.
- **Reuse correct inner frames.** `apply_replace_all_with_plan`, `compute_inner`
  and `difference_expr` already resume their own phase correctly where brief-548
  said so; the caller's frame is what is missing.
- **Nothing on the success path.** Freeze or move state into a frame only when
  the call actually returns `InputRequired`; no clone, allocation, lock or
  counter on the ordinary path.
- Every new frame derives `FrameRootTrace`; a missing traversal stays a compile
  error. Keep existing custom abandonment; never run a skipped iteration on
  abandonment.
- **No leases.** The retention line will convert Scan (region R01) and the
  callback-history loops later, on top of the state these frames carry — name
  that state clearly.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- brief-548's report (the mechanism, §4 root causes, §5 enforcement); the drafts
  and evidence above.
- `crates/rum-runtime/src/runtime/suspension/core.rs` — `insert_suspension_frame`
  (`:515`), `drive_resuming_frames` (`:709`), `resume_or_suspend!` (`:784`);
  `eval_dispatch/mod.rs:466` — the generic dispatch frame whose `complete` is
  what currently swallows the remaining work.
- Existing correct loop frames to model on: `Do`, `Table`, `NestList`, `Select`
  resume correctly (brief-548's controls) — read how they carry their state.

## Scope

- **Objective:** the five builtins' continuation frames and step functions; the
  permanent tests; microbench labels for them.
- **Standing duty — always in scope** (`docs/dev/codex/TEMPLATE.md`, Scope):
  replay the retention fingerprints and certificates and reconcile ownership
  inventory positions; only fingerprints, hashes and positions may change, and
  any identity, status, disposition or exclusion change is a STOP.
- **Fence:** only these five sites and their direct callers. No other brief-548
  group (G2-G5, G7), no enforcement work (E1, E2), no evaluator redesign, no
  `Expr` or `Symbol` change, no leases, no overlap-policy change, no new
  dependency, no `unsafe`. Do not loosen an existing test or helper. Do not
  edit `AGENTS.md`, `PROGRESS.md`, `TODO.md`, earlier briefs or unrelated docs.
- **Disclosure duty:** every changed file with line ranges and why; every new
  frame and root field; any consumer or site you find that brief-548 did not
  list — report it with evidence before widening the objective.

## rum / Rust pitfalls (read twice)

1. **A callee frame does not complete its caller.** Carry the cursor, the
   completed prefix, the pending operation and the final wrapping together.
2. **Suspend in the middle and twice**, and require the corrected path to reach
   both prompts. Never replay a completed side effect.
3. **`ReplaceRepeated`'s limit is part of the loop**: an iteration consumed
   before the suspension is still consumed after it.
4. **Stop-loss is keyed on progress**: sites repaired, equality tests passing,
   lifecycle checks passing, compile errors falling. Stop when two consecutive
   compiling attempts move none.

## Verification (MANDATORY — you run it; iterate to green)

Every command through `python3 scripts/run-bounded.py --timeout <seconds> --`,
bounds from measured runs, cargo as `env CARGO_BUILD_JOBS=4
CARGO_NET_OFFLINE=true cargo …`.

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Plus the retention replay in the `--no-default-features` build as the template
describes. Paste commands, bounds, exit statuses and counts verbatim; list
native ZeroMQ skips.

**Performance gate** — these are loop drivers, so show the ordinary path costs
nothing. Before editing, build and preserve base binaries under
`.temp/brief-559/base/` into their own target directory, and check that base and
candidate differ and that the base lacks the change. Then, allocation counts
first (`rum-cli/alloc-count`, `RUM_PROFILE_ALLOC=1`); per-iteration
`cpu_core/instructions:u` with `taskset -c 2`, `rum-cli` fed `< /dev/null`,
`(count(2N) - count(N)) / N`, one label per run, at most two samples per
(label, N, build), N at least 20 000 where the iteration limit allows. Labels:
new scratch labels for `Scan`, `OptionValue` with three names, `ReplaceRepeated`,
three-operand `Dot` and `Differences` — each evaluating without any input
request — plus the existing `do-count-null`, `do-iterator-null`,
`table-iterator-null` and `replaceall-simple`. Timing is secondary; if you run
it, check first that no CPU-heavy process competes. A regression on any label is
a defect to fix or report.

## Checkpointing — protect the scene

Scratch under `.temp/brief-559/`. After each milestone write
`.temp/brief-559/PROGRESS.md`: done and verified with evidence, the exact next
action, the tree's state, the counters.

## Hard rules

- Do NOT `git commit`, branch, push or touch git config. Do not run `git`.
- Do NOT modify files outside the objective, the standing duty and the fence;
  STOP and report instead.
- Touch nothing outside this repository, user configuration included. No
  network. Commands: file reads and edits within scope, `rg`/`grep`, `python3`,
  bounded `cargo`, the release `rum-cli`, bounded `taskset`/`perf`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- **No new `unsafe`.**
- You MUST paste real verbatim output and say plainly what did not run.

## Report format (your final message, appended to this file)

1. What changed — per file with line ranges; frames, fields and roots.
2. Verification — every command's verbatim result; red before and green after
   for every permanent test.
3. Allocation and per-iteration instruction rows per label.
4. Lost count before and after, progress counters, disclosures, anything
   unresolved.


---

## Execution report — STOP at ownership-inventory fence

This is an incomplete implementation handoff, not an accepted gate. The six scoped sites compile and the 21 permanent focused tests pass. Execution stopped before writing any ownership or retention inventory because discovery requires additions beyond position/hash changes. No git command, network operation, external file edit, new dependency, lease, unsafe block, or existing-test weakening occurred.

The binding Scope instruction says: **“only fingerprints, hashes and positions may change, and any identity, status, disposition or exclusion change is a STOP.”** The mandatory new frames add five derive discovery occurrences; the implementation also adds six closure and five macro discovery occurrences. These are sixteen additional inventory records after ignoring every source-position stamp, not positional refreshes. Host payload identities and storage rows do not change. The inventory files remain byte-for-byte as received. Director resolution of this scope conflict is required before continuing; no permission was inferred.

### 1. What changed

- [crates/rum-runtime/src/runtime/options/builtins.rs](../../../crates/rum-runtime/src/runtime/options/builtins.rs) lines **262–304, 510–539**: Many queries share an indexed scalar driver; the frame carries target, explicit options, all query names, next index and completed values. A resumed scalar is appended once before the remaining names run.
- [crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs](../../../crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs) lines **149–176, 444–472**: Add the missing post-rewrite-pass continuation and share the existing evaluation/fixed-point/limit steps. Preserve current expression, compiled rules, consumed iteration and effective limit.
- [crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs](../../../crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs) lines **446–495**: Replace the identity Scan frame with a driver carrying operator and the owned remaining-parts iterator. The iterator is the next-part cursor; resume discards the completed callback value and returns Null only after the suffix finishes.
- [crates/rum-stdlib/src/functional_ops/arrays/mod.rs](../../../crates/rum-stdlib/src/functional_ops/arrays/mod.rs) lines **3–5, 36–99**: Dot shares an operand driver. Its frame retains original head/arguments for remaining contractions and invalid-call reconstruction, plus the next operand index. The inner frame supplies the completed current tensor as the resume outcome.
- [crates/rum-stdlib/src/structure_ops/list_plumbing.rs](../../../crates/rum-stdlib/src/structure_ops/list_plumbing.rs) lines **3–5, 28–75, 114–169**: Differences has separate row and order continuations: pair cursor, input row and completed difference prefix; remaining order and final List/element-limit work. The unchanged difference_expr owns the inner evaluation.
- [crates/rum-stdlib/tests/loop_continuations.rs](../../../crates/rum-stdlib/tests/loop_continuations.rs) lines **1–222**: New 21-test integration harness following brief-550’s queued-versus-resumed shape; no shared extraction and no edit to guest_loading_resume or any existing test. Eleven required witnesses, five ordinary-error cases, five cancellation cases each covering both prompt positions. Typed values/status, termination, messages, prints and effect log compared; every requested suspension and fresh 1+1=2 asserted.
- This brief: this appended report only; the original brief prefix is preserved byte-for-byte.
- Scratch: all evidence, base binaries/source snapshots, checkpoints and the Python capture helper are under `.temp/brief-559/`. [file-audit.json](../../../.temp/brief-559/file-audit.json) records the five changed production files and one new test; [review.patch](../../../.temp/brief-559/review.patch) is a Python-generated comparison against the saved initial bytes. No git was used to obtain it.

All six frames derive `FrameRootTrace` and `PureFrameAbandon`. Existing custom abandonment is untouched. Frame construction and storage capture occur only in the InputRequired branch; no default-path clone/allocation claim is accepted without the still-pending performance gate.

| Frame | Root-bearing fields | Cursor / scalar state | Resume work |
| --- | --- | --- | --- |
| `EvaluateScanPartsSuspensionFrame` (replaced identity frame) | `operator: Expr`, `remaining_parts: IntoIter<Expr>` (iterator’s remaining slice is traced) | Iterator owns next-part cursor | Discard callback result; continue suffix; finish Null |
| `OptionValuesSuspensionFrame` | `target: Symbol`, `explicit: Option<SymbolOptions>`, `items: Vec<(Expr, Symbol)>`, `values: Vec<Expr>` | `next_index: usize` | Append resumed scalar once, finish names, wrap List |
| `ResumeReplaceRepeatedAfterReplaceSuspensionFrame` | `current: Expr`, `rules: RewriteRulePlan` | `iteration`, `effective_limit` | Evaluate completed rewrite pass, compare fixed point, advance/check saved iteration limit |
| `DotSuspensionFrame` | `head: Expr`, `args: SharedArgs` | `next_index: usize` | Use resumed contraction as current tensor; finish remaining operands |
| `DifferencesRowSuspensionFrame` | `items: Vec<Expr>`, `differences: Vec<Expr>` | `next_pair: usize` | Append resumed difference once; finish row; encode Vec continuation |
| `DifferencesOrderSuspensionFrame` | None; completed row arrives as resume outcome | `remaining_order: usize` | Decode completed row; finish higher orders, generated-element check and List wrapping |

File-size assessment: modified implementation sizes are Scan 15,477 chars, options 18,842, repeated rewrite 17,251, Differences/list plumbing 14,025 and arrays 6,023. The warning-range files retain existing responsibility boundaries for this fenced repair. A future split can move Scan driving out of normalization, OptionValue driving out of option parsing, repeated-loop frames out of rewrite entry normalization, and Differences driving out of the broader list-plumbing module. No file crosses 20k; no unrelated split or TODO edit was made.

### 2. Verification

Bounds derive from saved brief-550 measurements: isolated release 35.01s → 300s; alloc rebuild 1.99s → 30s; integration 8.41s → 80/90s; ownership discovery 5.40s → 80s including the newly rebuilt test crate. Source reads were under 0.3s and bounded at 30s. Every build/test command uses the bounded runner and `env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true`. Raw output was saved before reduction.

The newly authored error tests initially guessed an error spelling and omitted `Throw::nocatch` in assertions that were unreachable because equality failed first. Before any production change, the final red tests were corrected to assert the observed structured `Other` / `uncaught throw: 9` error and the exact `probe::tag`, `probe::tag`, `Throw::nocatch` sequence. Queued-error preconditions now run before comparison. No existing test or equality, prompt, error, or cancellation requirement was loosened.

**`default-build.txt` — exit 0, bound 300s, measured 34.212195s**

```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo build --release --target-dir .temp/brief-559/base/target -p rum-cli
```

Verbatim result lines (complete [raw output](../../../.temp/brief-559/base/default-build.txt)):

```text
    Finished `release` profile [optimized] target(s) in 34.13s
```

**`alloc-build.txt` — exit 0, bound 30s, measured 1.206638s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo build --release --target-dir .temp/brief-559/base/target -p rum-cli --features alloc-count
```

Verbatim result lines (complete [raw output](../../../.temp/brief-559/base/alloc-build.txt)):

```text
    Finished `release` profile [optimized] target(s) in 1.10s
```

**`loops-01.txt` — exit 101, bound 80s, measured 7.124798s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```

Verbatim result lines (complete [raw output](../../../.temp/brief-559/red/loops-01.txt)):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.89s
test result: FAILED. 2 passed; 19 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
error: test failed, to rerun pass `-p rum-stdlib --test loop_continuations`
```

**`loops-final.txt` — exit 101, bound 80s, measured 6.815308s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```

Verbatim result lines (complete [raw output](../../../.temp/brief-559/red/loops-final.txt)):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.62s
test result: FAILED. 2 passed; 19 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
error: test failed, to rerun pass `-p rum-stdlib --test loop_continuations`
```

**`loops-green-01.txt` — exit 101, bound 90s, measured 2.253031s**

```text
python3 scripts/run-bounded.py --timeout 90 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```

Verbatim result lines (complete [raw output](../../../.temp/brief-559/raw/loops-green-01.txt)):

```text
error[E0382]: use of moved value: `current`
error[E0382]: use of moved value: `rules`
error[E0382]: borrow of moved value: `explicit`
error[E0382]: use of moved value: `values`
error: could not compile `rum-runtime` (lib) due to 4 previous errors
```

**`loops-green-02.txt` — exit 101, bound 90s, measured 5.163689s**

```text
python3 scripts/run-bounded.py --timeout 90 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```

Verbatim result lines (complete [raw output](../../../.temp/brief-559/raw/loops-green-02.txt)):

```text
error[E0405]: cannot find trait `FrameAbandon` in this scope
error[E0412]: cannot find type `AbandonmentContext` in this scope
error[E0412]: cannot find type `FrameAbandonAction` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameAbandonAction`
error[E0405]: cannot find trait `FrameRootTrace` in this scope
error[E0405]: cannot find trait `FrameRootVisitor` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameRootTrace`
error[E0405]: cannot find trait `FrameAbandon` in this scope
error[E0412]: cannot find type `AbandonmentContext` in this scope
error[E0412]: cannot find type `FrameAbandonAction` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameAbandonAction`
error[E0405]: cannot find trait `FrameRootTrace` in this scope
error[E0405]: cannot find trait `FrameRootVisitor` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameRootTrace`
error[E0405]: cannot find trait `FrameAbandon` in this scope
error[E0412]: cannot find type `AbandonmentContext` in this scope
error[E0412]: cannot find type `FrameAbandonAction` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameAbandonAction`
error[E0277]: the trait bound `DotSuspensionFrame: FrameAbandon` is not satisfied
error[E0277]: the trait bound `DifferencesOrderSuspensionFrame: FrameAbandon` is not satisfied
error[E0277]: the trait bound `DifferencesOrderSuspensionFrame: rum_runtime::kernel_api::FrameRootTrace` is not satisfied
error[E0277]: the trait bound `DifferencesRowSuspensionFrame: FrameAbandon` is not satisfied
error[E0277]: the trait bound `DifferencesRowSuspensionFrame: rum_runtime::kernel_api::FrameRootTrace` is not satisfied
error: could not compile `rum-stdlib` (lib) due to 23 previous errors
```

**`loops-green-03.txt` — exit 0, bound 90s, measured 11.581996s**

```text
python3 scripts/run-bounded.py --timeout 90 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```

Verbatim result lines (complete [raw output](../../../.temp/brief-559/raw/loops-green-03.txt)):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.34s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

**`fmt-check-01.txt` — exit 1, bound 30s, measured 1.807871s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo fmt --check
```

Verbatim result lines (complete [raw output](../../../.temp/brief-559/raw/fmt-check-01.txt)):

```text
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/options/builtins.rs:278:
Diff in /home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/list_plumbing.rs:129:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:3:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:10:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:33:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:41:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:48:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:63:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:83:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:91:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:99:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:106:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:113:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:119:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:125:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:131:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:137:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:143:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:155:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:167:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:172:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:177:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:182:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:187:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:192:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:197:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:202:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:210:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:217:
```

**`ownership-discovery.txt` — exit 0, bound 80s, measured 28.401708s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```

Verbatim result lines (complete [raw output](../../../.temp/brief-559/raw/ownership-discovery.txt)):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 22.99s
UnknownHostPayload count: 63
UnknownConstruct count: 7562
Self-reaching type count: 60
Production files: 630; test-only files: 406; nominal definitions: 1601; aliases: 40; owning edges: 3644; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.28s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

The final pre-change red output and first compiled green output below cover every permanent test, including the two cancellation controls that already passed before repair. Red: 2 passed / 19 failed. Green: 21 passed / 0 failed. The run requires every expected prompt, including both prompts in all twice cases; cancellation logs are asserted at both stops.

**Final red, before any production edit:**

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.62s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-bdeaa5d842f94483)

running 21 tests
test differences ... CASE	Differences	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "List[2]", ["p: "])	queued=(View { status: "OK", value: Some("List[Plus[-1, a], 1, 1]"), termination: "None", messages: [], prints: [] }, "List[2]", [])

thread 'differences' panicked at crates/rum-stdlib/tests/loop_continuations.rs:68:5:
assertion `left == right` failed: value, error, termination, messages and prints
  left: View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[Plus[-1, a], 1, 1]"), termination: "None", messages: [], prints: [] }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test differences_cancel ... CANCEL	differences	stop=0	log=List[2]	termination=Some(Cancelled)

thread 'differences_cancel' panicked at crates/rum-stdlib/tests/loop_continuations.rs:86:13:
must suspend at input 1
FAILED
test differences_error_after_resume ... CASE	differences-error	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }], prints: ["before"] }, "List[2]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[2, 4]", [])

thread 'differences_error_after_resume' panicked at crates/rum-stdlib/tests/loop_continuations.rs:68:5:
assertion `left == right` failed: value, error, termination, messages and prints
  left: View { status: "OK", value: Some("1"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }], prints: ["before"] }
 right: View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }
FAILED
test differences_order_two ... CASE	Differences-order-two	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "List[2]", ["p: "])	queued=(View { status: "OK", value: Some("List[Plus[1, Times[-1, Plus[-1, a]]], 0]"), termination: "None", messages: [], prints: [] }, "List[2]", [])

thread 'differences_order_two' panicked at crates/rum-stdlib/tests/loop_continuations.rs:68:5:
assertion `left == right` failed: value, error, termination, messages and prints
  left: View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[Plus[1, Times[-1, Plus[-1, a]]], 0]"), termination: "None", messages: [], prints: [] }
FAILED
test differences_twice ... CASE	Differences-twice	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "List[2]", ["p: "])	queued=(View { status: "OK", value: Some("List[Plus[-1, a], 1, Plus[-3, b], 1]"), termination: "None", messages: [], prints: [] }, "List[2, 4]", [])

thread 'differences_twice' panicked at crates/rum-stdlib/tests/loop_continuations.rs:66:5:
assertion `left == right` failed: must reach every suspension, in order
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED
test dot_cancel ... CANCEL	dot	stop=0	log=List[1, 3]	termination=Some(Cancelled)
CANCEL	dot	stop=1	log=List[1, 3, 2, 4]	termination=Some(Cancelled)
ok
test dot_chain ... CASE	Dot-chain	resumed=(View { status: "OK", value: Some("List[List[4, 6], List[4, 6]]"), termination: "None", messages: [], prints: [] }, "List[1, 3, 2, 4, 1, 3, 2, 4]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[12, 18], List[12, 18]]"), termination: "None", messages: [], prints: [] }, "List[1, 3, 2, 4, 1, 3, 2, 4]", [])

thread 'dot_chain' panicked at crates/rum-stdlib/tests/loop_continuations.rs:68:5:
assertion `left == right` failed: value, error, termination, messages and prints
  left: View { status: "OK", value: Some("List[List[4, 6], List[4, 6]]"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[List[12, 18], List[12, 18]]"), termination: "None", messages: [], prints: [] }
FAILED
test dot_error_after_resume ... CASE	dot-error	resumed=(View { status: "OK", value: Some("List[List[4, 6], List[4, 6]]"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }], prints: ["before"] }, "List[1, 3, 2, 4, 1, 3, 2, 4]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 3, 2, 4, 1, 3, 2, 4, 99]", [])

thread 'dot_error_after_resume' panicked at crates/rum-stdlib/tests/loop_continuations.rs:68:5:
assertion `left == right` failed: value, error, termination, messages and prints
  left: View { status: "OK", value: Some("List[List[4, 6], List[4, 6]]"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }], prints: ["before"] }
 right: View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }
FAILED
test dot_twice ... CASE	Dot-twice	resumed=(View { status: "OK", value: Some("List[List[4, 6], List[4, 6]]"), termination: "None", messages: [], prints: [] }, "List[1, 3, 2, 4, 1, 3, 2, 4]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[List[12, 18], List[12, 18]]"), termination: "None", messages: [], prints: [] }, "List[1, 3, 2, 4, 1, 3, 2, 4]", [])

thread 'dot_twice' panicked at crates/rum-stdlib/tests/loop_continuations.rs:68:5:
assertion `left == right` failed: value, error, termination, messages and prints
  left: View { status: "OK", value: Some("List[List[4, 6], List[4, 6]]"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[List[12, 18], List[12, 18]]"), termination: "None", messages: [], prints: [] }
FAILED
test option_value_cancel ... CANCEL	option_value	stop=0	log=List[1, 2]	termination=Some(Cancelled)

thread 'option_value_cancel' panicked at crates/rum-stdlib/tests/loop_continuations.rs:86:13:
must suspend at input 1
FAILED
test option_value_error_after_resume ... CASE	option_value-error	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }], prints: ["before"] }, "List[1, 2]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])

thread 'option_value_error_after_resume' panicked at crates/rum-stdlib/tests/loop_continuations.rs:68:5:
assertion `left == right` failed: value, error, termination, messages and prints
  left: View { status: "OK", value: Some("\"two\""), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }], prints: ["before"] }
 right: View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }
FAILED
test optionvalue_many ... CASE	OptionValue-many	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])

thread 'optionvalue_many' panicked at crates/rum-stdlib/tests/loop_continuations.rs:68:5:
assertion `left == right` failed: value, error, termination, messages and prints
  left: View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }
FAILED
test optionvalue_twice ... CASE	OptionValue-twice	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3, \"four\", 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])

thread 'optionvalue_twice' panicked at crates/rum-stdlib/tests/loop_continuations.rs:66:5:
assertion `left == right` failed: must reach every suspension, in order
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED
test replace_repeated_cancel ... CANCEL	replace_repeated	stop=0	log=List[1, 2]	termination=Some(Cancelled)
CANCEL	replace_repeated	stop=1	log=List[1, 2, 3, 4]	termination=Some(Cancelled)
ok
test replace_repeated_error_after_resume ... CASE	replace_repeated-error	resumed=(View { status: "OK", value: Some("List[h[1], h[2], h[3]]"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }], prints: ["before"] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3, 99]", [])

thread 'replace_repeated_error_after_resume' panicked at crates/rum-stdlib/tests/loop_continuations.rs:68:5:
assertion `left == right` failed: value, error, termination, messages and prints
  left: View { status: "OK", value: Some("List[h[1], h[2], h[3]]"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }], prints: ["before"] }
 right: View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }
FAILED
test replacerepeated_predicate ... CASE	ReplaceRepeated-predicate	resumed=(View { status: "OK", value: Some("List[h[1], h[2], h[3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationLimit\"]"), termination: "Some(IterationLimit)", messages: [RuntimeMessage { symbol: "General", tag: "iterlim", text: "ReplaceRepeated exceeded iteration limit (1)" }], prints: [] }, "List[1, 2, 3]", [])

thread 'replacerepeated_predicate' panicked at crates/rum-stdlib/tests/loop_continuations.rs:68:5:
assertion `left == right` failed: value, error, termination, messages and prints
  left: View { status: "OK", value: Some("List[h[1], h[2], h[3]]"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("TerminatedEvaluation[\"IterationLimit\"]"), termination: "Some(IterationLimit)", messages: [RuntimeMessage { symbol: "General", tag: "iterlim", text: "ReplaceRepeated exceeded iteration limit (1)" }], prints: [] }
FAILED
test replacerepeated_twice ... CASE	ReplaceRepeated-twice	resumed=(View { status: "OK", value: Some("List[h[1], h[2], h[3], h[4], h[5]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationLimit\"]"), termination: "Some(IterationLimit)", messages: [RuntimeMessage { symbol: "General", tag: "iterlim", text: "ReplaceRepeated exceeded iteration limit (1)" }], prints: [] }, "List[1, 2, 3, 4, 5]", [])

thread 'replacerepeated_twice' panicked at crates/rum-stdlib/tests/loop_continuations.rs:68:5:
assertion `left == right` failed: value, error, termination, messages and prints
  left: View { status: "OK", value: Some("List[h[1], h[2], h[3], h[4], h[5]]"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("TerminatedEvaluation[\"IterationLimit\"]"), termination: "Some(IterationLimit)", messages: [RuntimeMessage { symbol: "General", tag: "iterlim", text: "ReplaceRepeated exceeded iteration limit (1)" }], prints: [] }
FAILED
test scan ... CASE	Scan	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("Null"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])

thread 'scan' panicked at crates/rum-stdlib/tests/loop_continuations.rs:68:5:
assertion `left == right` failed: value, error, termination, messages and prints
  left: View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("Null"), termination: "None", messages: [], prints: [] }
FAILED
test scan_cancel ... CANCEL	scan	stop=0	log=List[1, 2]	termination=Some(Cancelled)

thread 'scan_cancel' panicked at crates/rum-stdlib/tests/loop_continuations.rs:86:13:
must suspend at input 1
FAILED
test scan_error_after_resume ... CASE	scan-error	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }], prints: ["before"] }, "List[1, 2]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])

thread 'scan_error_after_resume' panicked at crates/rum-stdlib/tests/loop_continuations.rs:68:5:
assertion `left == right` failed: value, error, termination, messages and prints
  left: View { status: "OK", value: Some("2"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }], prints: ["before"] }
 right: View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }
FAILED
test scan_twice ... CASE	Scan-twice	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("Null"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])

thread 'scan_twice' panicked at crates/rum-stdlib/tests/loop_continuations.rs:66:5:
assertion `left == right` failed: must reach every suspension, in order
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED

failures:

failures:
    differences
    differences_cancel
    differences_error_after_resume
    differences_order_two
    differences_twice
    dot_chain
    dot_error_after_resume
    dot_twice
    option_value_cancel
    option_value_error_after_resume
    optionvalue_many
    optionvalue_twice
    replace_repeated_error_after_resume
    replacerepeated_predicate
    replacerepeated_twice
    scan
    scan_cancel
    scan_error_after_resume
    scan_twice

test result: FAILED. 2 passed; 19 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

error: test failed, to rerun pass `-p rum-stdlib --test loop_continuations`
```

**Green, first compiling implementation:**

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.34s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-bdeaa5d842f94483)

running 21 tests
test differences ... CASE	Differences	resumed=(View { status: "OK", value: Some("List[Plus[-1, a], 1, 1]"), termination: "None", messages: [], prints: [] }, "List[2]", ["p: "])	queued=(View { status: "OK", value: Some("List[Plus[-1, a], 1, 1]"), termination: "None", messages: [], prints: [] }, "List[2]", [])
ok
test differences_cancel ... CANCEL	differences	stop=0	log=List[2]	termination=Some(Cancelled)
CANCEL	differences	stop=1	log=List[2, 4]	termination=Some(Cancelled)
ok
test differences_error_after_resume ... CASE	differences-error	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[2, 4]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[2, 4]", [])
ok
test differences_order_two ... CASE	Differences-order-two	resumed=(View { status: "OK", value: Some("List[Plus[1, Times[-1, Plus[-1, a]]], 0]"), termination: "None", messages: [], prints: [] }, "List[2]", ["p: "])	queued=(View { status: "OK", value: Some("List[Plus[1, Times[-1, Plus[-1, a]]], 0]"), termination: "None", messages: [], prints: [] }, "List[2]", [])
ok
test differences_twice ... CASE	Differences-twice	resumed=(View { status: "OK", value: Some("List[Plus[-1, a], 1, Plus[-3, b], 1]"), termination: "None", messages: [], prints: [] }, "List[2, 4]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[Plus[-1, a], 1, Plus[-3, b], 1]"), termination: "None", messages: [], prints: [] }, "List[2, 4]", [])
ok
test dot_cancel ... CANCEL	dot	stop=0	log=List[1, 3]	termination=Some(Cancelled)
CANCEL	dot	stop=1	log=List[1, 3, 2, 4]	termination=Some(Cancelled)
ok
test dot_chain ... CASE	Dot-chain	resumed=(View { status: "OK", value: Some("List[List[12, 18], List[12, 18]]"), termination: "None", messages: [], prints: [] }, "List[1, 3, 2, 4, 1, 3, 2, 4]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[12, 18], List[12, 18]]"), termination: "None", messages: [], prints: [] }, "List[1, 3, 2, 4, 1, 3, 2, 4]", [])
ok
test dot_error_after_resume ... CASE	dot-error	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 3, 2, 4, 1, 3, 2, 4, 99]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 3, 2, 4, 1, 3, 2, 4, 99]", [])
ok
test dot_twice ... CASE	Dot-twice	resumed=(View { status: "OK", value: Some("List[List[12, 18], List[12, 18]]"), termination: "None", messages: [], prints: [] }, "List[1, 3, 2, 4, 1, 3, 2, 4]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[List[12, 18], List[12, 18]]"), termination: "None", messages: [], prints: [] }, "List[1, 3, 2, 4, 1, 3, 2, 4]", [])
ok
test option_value_cancel ... CANCEL	option_value	stop=0	log=List[1, 2]	termination=Some(Cancelled)
CANCEL	option_value	stop=1	log=List[1, 2, 3, 4]	termination=Some(Cancelled)
ok
test option_value_error_after_resume ... CASE	option_value-error	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])
ok
test optionvalue_many ... CASE	OptionValue-many	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test optionvalue_twice ... CASE	OptionValue-twice	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3, \"four\", 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3, \"four\", 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ok
test replace_repeated_cancel ... CANCEL	replace_repeated	stop=0	log=List[1, 2]	termination=Some(Cancelled)
CANCEL	replace_repeated	stop=1	log=List[1, 2, 3, 4]	termination=Some(Cancelled)
ok
test replace_repeated_error_after_resume ... CASE	replace_repeated-error	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3, 99]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3, 99]", [])
ok
test replacerepeated_predicate ... CASE	ReplaceRepeated-predicate	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationLimit\"]"), termination: "Some(IterationLimit)", messages: [RuntimeMessage { symbol: "General", tag: "iterlim", text: "ReplaceRepeated exceeded iteration limit (1)" }], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationLimit\"]"), termination: "Some(IterationLimit)", messages: [RuntimeMessage { symbol: "General", tag: "iterlim", text: "ReplaceRepeated exceeded iteration limit (1)" }], prints: [] }, "List[1, 2, 3]", [])
ok
test replacerepeated_twice ... CASE	ReplaceRepeated-twice	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationLimit\"]"), termination: "Some(IterationLimit)", messages: [RuntimeMessage { symbol: "General", tag: "iterlim", text: "ReplaceRepeated exceeded iteration limit (1)" }], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationLimit\"]"), termination: "Some(IterationLimit)", messages: [RuntimeMessage { symbol: "General", tag: "iterlim", text: "ReplaceRepeated exceeded iteration limit (1)" }], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ok
test scan ... CASE	Scan	resumed=(View { status: "OK", value: Some("Null"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("Null"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test scan_cancel ... CANCEL	scan	stop=0	log=List[1, 2]	termination=Some(Cancelled)
CANCEL	scan	stop=1	log=List[1, 2, 3, 4]	termination=Some(Cancelled)
ok
test scan_error_after_resume ... CASE	scan-error	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])
ok
test scan_twice ... CASE	Scan-twice	resumed=(View { status: "OK", value: Some("Null"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("Null"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

```

**Standing-duty discovery evidence (no inventory edits):**

| Inventory | Existing rows | Discovered rows | Added after ignoring positions | Removed |
| --- | ---: | ---: | ---: | ---: |
| `host_payloads.tsv` | 63 | 63 | 0 | 0 |
| `unknown_attributes.tsv` | 1429 | 1434 | 5 | 0 |
| `unknown_captures.tsv` | 2932 | 2938 | 6 | 0 |
| `unknown_macros.tsv` | 3097 | 3102 | 5 | 0 |
| `unknown_storage.tsv` | 88 | 88 | 0 | 0 |

[ownership-identity-delta.json](../../../.temp/brief-559/ownership-identity-delta.json) contains verbatim discovery rows and the normalized multiplicity comparison. [ownership-discovery.txt](../../../.temp/brief-559/raw/ownership-discovery.txt) is the complete replay output. The finder itself passed; it is a report-only test and does **not** establish that the unchanged inventory matches. New `derive` attributes are reported as unresolved by `source_items.rs:527–546`; all closure sites are reported by `:443–452` and macros by `:515–524`. These are discovery classifications, not Rust compilation or root-traversal failures.

**Did not run / not green:**

- `cargo fmt --check` ran and failed: formatting changes are needed only in `options/builtins.rs`, `structure_ops/list_plumbing.rs` and the new test. Formatting was not applied after the stop trigger.
- `cargo test -p rum-runtime`, `cargo test -p rum-stdlib` (full suite), and `cargo test --workspace` did not run.
- Retention contract-fingerprint refresh, both `local_production_acceptance` certificate replays, and `--no-default-features` retention replay did not run. Their source-dependent fingerprints are now stale.
- Native ZeroMQ tests did not run; there are **no observed skips to list** for this execution.
- No exact-inventory acceptance is claimed. No historical 195-probe whole-census replay was run.

### 3. Allocation and instruction rows

Only the required pre-edit base builds completed. They were compiled from the initial source into `.temp/brief-559/base/target/`; preserved binaries:

```text
.temp/brief-559/base/rum-cli	sha256:f40c5b1f206d9a0f5771ccd71990227b13aa2d470a91eb6a49ec7596cfbfab5c
.temp/brief-559/base/rum-cli-alloc	sha256:046be5b3e9b64e9a0ecb2fac12d5f5e829c18433b7f846097c73de19138fc716
```

The saved source bytes and hashes prove that the base was built before the loop changes. No candidate release was built, so base/candidate binary inequality was not checked. No allocation, instruction or timing claim is made.

| Required label | Allocation row | `(count(2N) - count(N)) / N`, cpu_core/instructions:u |
| --- | --- | --- |
| Scan, no input | NOT RUN | NOT RUN |
| OptionValue, three names, no input | NOT RUN | NOT RUN |
| ReplaceRepeated, no input | NOT RUN | NOT RUN |
| Dot, three operands, no input | NOT RUN | NOT RUN |
| Differences, no input | NOT RUN | NOT RUN |
| do-count-null | NOT RUN | NOT RUN |
| do-iterator-null | NOT RUN | NOT RUN |
| table-iterator-null | NOT RUN | NOT RUN |
| replaceall-simple | NOT RUN | NOT RUN |

Scratch benchmark labels and the full timing A/B remain unimplemented/unrun because execution stopped at the ownership fence. No perf samples were consumed.

### 4. Counters, disclosures and exact next action

- Six scoped sites repaired in code; 11/11 minimum equality witnesses pass. Five ordinary-error tests and five cancellation tests (ten suspension positions) pass, with fresh-session usability checks.
- For brief-548’s frozen confirmed set, these six scoped repairs account for **86 → 80** lost continuations. This is scoped test evidence, not a refreshed census or a claim that the unfinished gate accepted the candidate.
- Two red runs; three implementation compile/test runs (two compile failures, then one compiling green run). The compiling stop-loss counter is **0** consecutive non-improving attempts. Runtime’s four move errors were repaired, then stdlib’s 23 derive-import diagnostics were repaired.
- No additional consumer/site outside the six named obligations was repaired or added to the objective. No helper extraction from brief-550. No existing custom abandonment changed. No inventory identity, status, disposition or exclusion was modified.
- The stop is the explicit Scope maintenance fence: mandatory frame additions produce new discovery rows, so a position/hash-only refresh cannot reconcile ownership. The code compiles and focused tests pass, but formatting, broad gates, retention replay and all candidate/performance work remain incomplete.

Exact next action after director resolution: reconcile the permitted discovery additions (or choose an approved alternative), format only the three named files, refresh contract fingerprints and replay both existing certificate cells without changing their identities/status/strengths/exclusions, run all four mandatory gates and no-default retention, then build a separate candidate release and finish the bounded allocation-first instruction/timing A/B. Keep the preserved base unchanged. The current patch and complete scene are recorded in [.temp/brief-559/PROGRESS.md](../../../.temp/brief-559/PROGRESS.md).

---

## Resume — the director's ruling on the ownership rows

You stopped correctly on the rule as written; the rule was too strict, and it
is corrected in `docs/dev/codex/TEMPLATE.md` (Scope, standing duty). **The
sixteen new `UnknownConstruct` rows you found — five derives on the new frames,
six closures, five macro calls, all in code this brief added or changed — are
expected.** Add them, disclosing each row's file and line; unknowns stay
unknown. The rule that still stops you: an existing row's identity, status,
disposition or exclusion changing or disappearing, any new
`UnknownHostPayload` row (discovery still reports 63), or a new row pointing
at code this brief did not touch.

Continue from `.temp/brief-559/PROGRESS.md` and do not redo the repairs: format
the changed files, reconcile the ownership inventories with those additions,
replay the retention fingerprints and certificates, run every gate in
Verification, and finish the performance gate against the base you preserved.
Append your report covering both runs.

## Second resume — the host stopped the run, not the work

The resumed run was killed by the director's harness when the machine ran
critically low on memory; nothing in the run caused it. The working tree is as
that run left it and is also preserved, without moving HEAD, on
`refs/wip/brief-559-interrupted` and in `.temp/brief-559/wip-backup/`.

Read `.temp/brief-559/PROGRESS.md` and continue from its last line: runtime is
green (1116), focused 21/21, fmt clean; what remains is the final ownership
discovery and the contract fingerprint and certificate replay (the frame
changes after your last replay expired them again), `rum-stdlib`, the
workspace, the `--no-default-features` retention gate, and the performance
gate. **Run everything serially from here on** (user instruction): every cargo
command with `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`, and never two cargo
commands at once. Serial runs are slower — derive each timeout from a measured
serial run of the same shape, not from the parallel timings. Append one report
covering all three runs.

---

## Three-run execution report — STOP at serial CLI test-harness scope fence

This report covers the initial repair, the ownership-ruling resume interrupted by the host, and the serial second resume. The candidate is **not an accepted gate**. Six scoped sites are repaired; the 21 permanent tests pass; runtime, formatting, full stdlib, final ownership discovery, all 68 contract fingerprints, and both certificate builds have successful evidence. The serial workspace command fails in three existing CLI tests before running the other workspace crates. Full no-default retention and the performance gate remain incomplete.

The stop is the brief's hard rule: **“Do NOT modify files outside the objective, the standing duty and the fence; STOP and report instead.”** The needed repair is to the output-capture assertions in `crates/rum-cli/src/eval_tests.rs`, outside the five builtin sites and their direct callers. No change to that file, no test weakening, and no override of the mandatory single-thread environment was made.

### 1. What changed across all three runs

All 13 crate-file hashes at the end of this run still match the saved second-run audit. This third run changed no production or permanent test source and introduced no further inventory identities. It appended this report and wrote scratch evidence under `.temp/brief-559/serial/`, plus checkpoint and command-ledger entries.

| File | Final changed line ranges | Why |
| --- | --- | --- |
| [crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs](../../../crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs) | 149, 153–176, 444–472 | Carry the completed rewrite pass into evaluation/fixed-point comparison, preserving consumed iterations and effective limit. |
| [crates/rum-runtime/src/runtime/options/builtins.rs](../../../crates/rum-runtime/src/runtime/options/builtins.rs) | 262–297, 299–300, 302–316, 522–550 | Share the indexed scalar driver; freeze query names and completed values only on suspension; append the resumed scalar once. |
| [crates/rum-runtime/src/runtime/tests/frame_root_trace.rs](../../../crates/rum-runtime/src/runtime/tests/frame_root_trace.rs) | 483 | Reconcile the exact registered-frame count, 545 to 550; retain every assertion and all 33 stateful identities. |
| [crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs](../../../crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs) | 343, 432, 721, 932–933, 1159, 1184, 1362, 1476–1477 | Add the five new frame identities to both the derived-root and pure-abandonment data arrays; no checker weakening. |
| [crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs](../../../crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs) | 451–459, 461–471, 478–482, 487, 490–492 | Replace the identity Scan continuation with an operator and remaining-parts driver, finishing with Null. |
| [crates/rum-stdlib/src/functional_ops/arrays/mod.rs](../../../crates/rum-stdlib/src/functional_ops/arrays/mod.rs) | 3–5, after 40 (deletion), 48–57, 60–72, 79–99 | Continue the operand chain from the completed contraction; retain head/arguments for suffix work and invalid-call reconstruction. |
| [crates/rum-stdlib/src/structure_ops/list_plumbing.rs](../../../crates/rum-stdlib/src/structure_ops/list_plumbing.rs) | 3–5, 28–45, 53–75, 119–145, 148–187 | Carry both Differences row/prefix state and remaining order; freeze row storage only on suspension. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv) | 436–441, 600–602, 1021, 1033, 1330–1332 | Five authorized new derives and nine existing source-position updates; unknown classifications unchanged. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv) | 888–898, 1176–1180, 2050, 2072, 2691–2702 | Six authorized new closures and 24 existing source-position updates; unknown classifications unchanged. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv) | 743–748, 1118–1121, 2170–2171, 2203, 2787–2795 | Five authorized new macro sites and 17 existing source-position updates; unknown classifications unchanged. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv) | 2–69 | Refresh all 68 source fingerprints by regeneration; preserve all other 18 fields, 62 Discharged and six Pending. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv) | 2–3 | Replay the C01/C02 default/no-default certificate cells; preserve identities, statuses, preconditions, strengths and exclusions. |
| [crates/rum-stdlib/tests/loop_continuations.rs](../../../crates/rum-stdlib/tests/loop_continuations.rs) | 1–373 | Add 21 permanent queued-versus-resumed tests: 11 witnesses, five later errors, five cancellation cases covering both prompts. |

The brief itself has appended reports only; the prefix saved at the beginning of this third run is preserved byte-for-byte. The final crate patch and precise ranges are in [final-review.patch](../../../.temp/brief-559/resume/final-review.patch) and [final-file-audit.json](../../../.temp/brief-559/resume/final-file-audit.json), made from saved source bytes without git. [initial-audit.json](../../../.temp/brief-559/serial/initial-audit.json) proves the third-run starting state.

All six frames derive `FrameRootTrace` and `PureFrameAbandon`. No existing custom abandonment changed. Cold-state capture is confined to InputRequired; the success-path performance claim is still **unmeasured**, not accepted by inspection.

| Frame | Final root-bearing fields | Cursor/scalar state and resume work |
| --- | --- | --- |
| `EvaluateScanPartsSuspensionFrame` (existing identity frame replaced) | `operator: Expr`, `remaining_parts: IntoIter<Expr>` | Iterator is the cursor; discard completed callback result, run suffix, return Null. |
| `OptionValuesSuspensionFrame` | `target: Symbol`, `explicit: Option<SymbolOptions>`, `items: Arc<[(Expr, Symbol)]>`, `values: SharedArgs` | `next_index`; reconstruct prefix after suspension, append resumed scalar exactly once, finish names, wrap List. |
| `ResumeReplaceRepeatedAfterReplaceSuspensionFrame` | `current: Expr`, `rules: RewriteRulePlan` | `iteration`, `effective_limit`; evaluate completed rewrite, check fixed point and preserved iteration limit. |
| `DotSuspensionFrame` | `head: Expr`, `args: SharedArgs` | `next_index`; resumed inner contraction supplies current tensor; finish operands. |
| `DifferencesRowSuspensionFrame` | `items: SharedArgs`, `differences: SharedArgs` | `next_pair`; append resumed difference, finish row, encode continuation value. |
| `DifferencesOrderSuspensionFrame` | None; row arrives as outcome | `remaining_order`; decode row, finish orders, element-limit check, List wrapping. |

The second run replaced the new OptionValue/Differences frames' direct Vec fields with frozen Arc/SharedArgs storage after the existing frame-ownership gate rejected them. Reconstruction happens in resumed helpers. The checker and its assertions were not loosened; the final runtime gate passes. Root inventory totals are 965 derived types and 550 frames (517 pure, 33 stateful). Scan already existed; the other five are new.

File-size assessment: options 19,277 chars, repeated rewrite 17,251, Scan 15,477, arrays 6,023, list plumbing 14,441, new integration tests 11,586. Keep these coherent loop repairs within the fence; possible future splits are OptionValue driving from parsing, Scan driving from normalization, repeated-loop frames from entry normalization, and Differences from other list plumbing. `frame_root_trace.rs` is a pre-existing 30,342-char test module changed only at its count. `derived_inventory.rs` is a pre-existing large data inventory (167,818 chars) changed only by ten data entries; a future organization change can partition its exact lists by crate while retaining set reconciliation. No unrelated module split or TODO edit was made.

#### Every ownership addition and moved record

The director authorized exactly sixteen new UnknownConstruct occurrences. Unknowns remain unknown. Host payloads are still 63, storage rows 88, self-reaching types 60; no existing identity disappeared and no status/disposition/exclusion changed. The following are verbatim discovery rows:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	445	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	523	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
UnknownConstruct	crates/rum-stdlib/src/functional_ops/arrays/mod.rs	81	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	55	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	150	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	461	40	closure captures: |replaced| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	538	40	closure captures: |value| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs	490	40	closure captures: |_| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/functional_ops/arrays/mod.rs	95	40	closure captures: |current| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	67	40	closure captures: |row| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	164	40	closure captures: |value| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	154	9	unexpanded macro crate :: resume_or_suspend! (ends at 166:10)
UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	286	28	unexpanded macro crate :: resume_or_suspend! (ends at 297:22)
UnknownConstruct	crates/rum-stdlib/src/functional_ops/arrays/mod.rs	62	22	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 71:10)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	38	19	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 45:10)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	133	24	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 142:18)
```

All fifty existing records whose source positions moved are disclosed below as exact before/after pairs. No other record was rewritten semantically. Full normalized multiplicity and position evidence is in [final-ownership-reconciliation.json](../../../.temp/brief-559/serial/final-ownership-reconciliation.json).

```text
unknown_attributes.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	221	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	244	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
unknown_attributes.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	265	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	288	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
unknown_attributes.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	306	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	329	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
unknown_attributes.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	351	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	374	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
unknown_attributes.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	384	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	407	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
unknown_attributes.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	413	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	455	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
unknown_attributes.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	447	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	489	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
unknown_attributes.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs	464	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs	477	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
unknown_attributes.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	189	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	294	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	205	34	closure captures: |symbol| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	228	34	closure captures: |symbol| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	217	17	closure captures: |limit| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	240	17	closure captures: |limit| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	218	14	closure captures: |limit| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	241	14	closure captures: |limit| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	274	40	closure captures: |payload| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	297	40	closure captures: |payload| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	330	18	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	353	18	closure captures: || (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	343	26	closure captures: |value| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	366	26	closure captures: |value| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	369	18	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	392	18	closure captures: || (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	374	22	closure captures: |limit| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	397	22	closure captures: |limit| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	376	26	closure captures: |value| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	399	26	closure captures: |value| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	407	26	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	430	26	closure captures: || (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	289	26	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	331	26	closure captures: || (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	307	22	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	349	22	closure captures: || (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	433	30	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	475	30	closure captures: || (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	467	30	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	509	30	closure captures: || (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	121	29	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	226	29	closure captures: || (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	132	14	closure captures: |items| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	237	14	closure captures: |items| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	133	25	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	238	25	closure captures: || (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	176	14	closure captures: |dim| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	281	14	closure captures: |dim| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	226	18	closure captures: |_| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	331	18	closure captures: |_| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	277	22	closure captures: |index| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	382	22	closure captures: |index| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	298	18	closure captures: |items| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	403	18	closure captures: |items| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	310	26	closure captures: |item| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	415	26	closure captures: |item| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	317	26	closure captures: |item| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	422	26	closure captures: |item| (capture types/modes unresolved)
unknown_captures.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	346	14	closure captures: |_| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	451	14	closure captures: |_| (capture types/modes unresolved)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	155	20	unexpanded macro crate :: resume_or_suspend! (ends at 164:10)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	178	20	unexpanded macro crate :: resume_or_suspend! (ends at 187:10)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	256	13	unexpanded macro vec! (ends at 260:14)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	279	13	unexpanded macro vec! (ends at 283:14)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	276	17	unexpanded macro panic! (ends at 276:81)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	299	17	unexpanded macro panic! (ends at 299:81)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	278	13	unexpanded macro assert! (ends at 281:14)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	301	13	unexpanded macro assert! (ends at 304:14)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	294	13	unexpanded macro debug_assert! (ends at 294:49)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	317	13	unexpanded macro debug_assert! (ends at 317:49)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	298	25	unexpanded macro format! (ends at 298:78)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	340	25	unexpanded macro format! (ends at 340:78)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	315	17	unexpanded macro format! (ends at 315:89)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	357	17	unexpanded macro format! (ends at 357:89)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	397	20	unexpanded macro vec! (ends at 401:19)
AFTER	UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	439	20	unexpanded macro vec! (ends at 443:19)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs	452	68	unexpanded macro vec! (ends at 452:78)
AFTER	UnknownConstruct	crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs	460	68	unexpanded macro vec! (ends at 460:78)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs	453	9	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 458:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs	462	20	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 470:14)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	92	9	unexpanded macro vec! (ends at 98:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	197	9	unexpanded macro vec! (ends at 203:10)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	142	9	unexpanded macro format! (ends at 145:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	247	9	unexpanded macro format! (ends at 250:10)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	154	9	unexpanded macro format! (ends at 157:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	259	9	unexpanded macro format! (ends at 262:10)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	166	9	unexpanded macro format! (ends at 169:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	271	9	unexpanded macro format! (ends at 274:10)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	206	21	unexpanded macro vec! (ends at 206:30)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	311	21	unexpanded macro vec! (ends at 311:30)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	264	21	unexpanded macro vec! (ends at 264:60)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	369	21	unexpanded macro vec! (ends at 369:60)
unknown_macros.tsv
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	282	10	unexpanded macro vec! (ends at 282:42)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	387	10	unexpanded macro vec! (ends at 387:42)
```

Retention contract rows changed only in their fingerprint field. All 68 changed row IDs and TSV lines follow; the replay on this third run regenerated them and found zero further differences. Old/new fingerprints are recorded individually in [contract-reconciliation.json](../../../.temp/brief-559/resume/contract-reconciliation.json).

```text
line 2	B01	Pending	fnv1a64:7d431998c96c7783 -> fnv1a64:9b9b39b0fc5a90f6
line 3	B02	Pending	fnv1a64:9d32f1a14a05066a -> fnv1a64:eb9407372bfc8db3
line 4	B03	Discharged	fnv1a64:e1ea69e8e30bdd11 -> fnv1a64:f0681df4dd546fac
line 5	B04	Discharged	fnv1a64:706b96e09cf0caeb -> fnv1a64:d39f3b872237fa8e
line 6	B05	Discharged	fnv1a64:009d5f648d0d370e -> fnv1a64:d92d6fdfe68460f7
line 7	B06	Pending	fnv1a64:8cd92818c46b32bf -> fnv1a64:9b486b6d442c4542
line 8	B07	Pending	fnv1a64:02b0cf27cf048e1e -> fnv1a64:2b69ceebbd2ec3a7
line 9	B08	Pending	fnv1a64:5a4e4eea3c158f05 -> fnv1a64:2277ee2559205ba0
line 10	B09	Discharged	fnv1a64:dfa1d86fbda4a947 -> fnv1a64:c8cb5c6d9d8ec1da
line 11	B10	Discharged	fnv1a64:ec2d1c0a27d51630 -> fnv1a64:32610982f50c5ac1
line 12	B11	Discharged	fnv1a64:31c3483db6e4f0eb -> fnv1a64:94f6ece43c2c208e
line 13	B12	Discharged	fnv1a64:9eae586b99ad3e47 -> fnv1a64:87d7dc69799756da
line 14	I01	Discharged	fnv1a64:5b25049636246e03 -> fnv1a64:797d24ae69128776
line 15	I02	Discharged	fnv1a64:57dedcf88543bf0e -> fnv1a64:306eed73debae8f7
line 16	I03	Pending	fnv1a64:1f2c12307c908007 -> fnv1a64:6ae51703c4f8091a
line 17	I04	Discharged	fnv1a64:454ccf6712e2d58a -> fnv1a64:a7de9df1cab446d3
line 18	P01	Discharged	fnv1a64:ecf5aff84df2295f -> fnv1a64:34976f8fb0dc7ae2
line 19	P02	Discharged	fnv1a64:b66888fe746154e0 -> fnv1a64:aa3cf53d7745ef51
line 20	P03	Discharged	fnv1a64:28b5467191f6fd89 -> fnv1a64:18f26a672b2f11b4
line 21	P04	Discharged	fnv1a64:920e0fb6ceae226a -> fnv1a64:e06f254cb0a5a9b3
line 22	P05	Discharged	fnv1a64:07176294f5465eb3 -> fnv1a64:944385f4a8561c86
line 23	P06	Discharged	fnv1a64:ef2d7d3c25dcd7c7 -> fnv1a64:d857013a05c6f05a
line 24	P070	Discharged	fnv1a64:297b197e045251c7 -> fnv1a64:12a49d7be43c6a5a
line 25	P080	Discharged	fnv1a64:cd5cc62c27c9a4f0 -> fnv1a64:e64455b814cb9a01
line 26	P090	Discharged	fnv1a64:c07a287879f5332a -> fnv1a64:160f6f43fc20f3f3
line 27	P100	Discharged	fnv1a64:b39c656a59a5d33b -> fnv1a64:2c05ed441298c23e
line 28	P110	Discharged	fnv1a64:298cd8521191b568 -> fnv1a64:20bb0d6ab5896c29
line 29	P120	Discharged	fnv1a64:068450821bf2b825 -> fnv1a64:fa306e7538996340
line 30	P130	Discharged	fnv1a64:a67e2884fdb8681e -> fnv1a64:cf372848ebe29da7
line 31	P140	Discharged	fnv1a64:09bb1a479b129d42 -> fnv1a64:19a1c172830caebb
line 32	P150	Discharged	fnv1a64:f8d8f5be48e5634a -> fnv1a64:a0abf62d619cec13
line 33	P071	Discharged	fnv1a64:335864e0b33c9cbe -> fnv1a64:7aa212fe6619d1c7
line 34	P081	Discharged	fnv1a64:816afd84d2df3ce7 -> fnv1a64:bd652f37ac72dafa
line 35	P091	Discharged	fnv1a64:f4aa7d3b3e56531f -> fnv1a64:546373a346877f22
line 36	P101	Discharged	fnv1a64:218fc91f46f55268 -> fnv1a64:18bdfe37eaed0929
line 37	P111	Discharged	fnv1a64:4b8995033f473d97 -> fnv1a64:5b80b026459f93ca
line 38	P121	Discharged	fnv1a64:abc274e9efc5887c -> fnv1a64:b24717e19fc9443d
line 39	P131	Discharged	fnv1a64:0e15b946d344ec6b -> fnv1a64:71495ded588c1c0e
line 40	P141	Discharged	fnv1a64:e1822c9b74dc127f -> fnv1a64:375131455eb3d782
line 41	P151	Discharged	fnv1a64:6a85533293eece65 -> fnv1a64:3a5eb7de231da100
line 42	B13	Discharged	fnv1a64:1ae9bba1f507eebf -> fnv1a64:2958fef674c90142
line 43	R01	Discharged	fnv1a64:4d5b32468598ad7a -> fnv1a64:891c0e27a74f0123
line 44	R02	Discharged	fnv1a64:2c19fdd521686149 -> fnv1a64:a30bb913c87598f4
line 45	R03	Discharged	fnv1a64:69268a2a615957a6 -> fnv1a64:fecc75a41a617fbf
line 46	R040	Discharged	fnv1a64:619d128dc6494a9a -> fnv1a64:df83db7a4778db43
line 47	R050	Discharged	fnv1a64:5124ce28cf30f193 -> fnv1a64:26e0a8e5220e2be6
line 48	R060	Discharged	fnv1a64:28642cb9293a1192 -> fnv1a64:dd61e9df8bfaf66b
line 49	R070	Discharged	fnv1a64:e9d8c59eb480fbe7 -> fnv1a64:25d2f7518e1499fa
line 50	R080	Discharged	fnv1a64:0c513859e2679e8a -> fnv1a64:6ee306e49a390fd3
line 51	R090	Discharged	fnv1a64:69f9ac5f7ad0eefb -> fnv1a64:6c392a884156837e
line 52	R100	Discharged	fnv1a64:a7100e3b8ce18791 -> fnv1a64:b58dc247872a1a2c
line 53	R110	Discharged	fnv1a64:da78e86e7942b9b8 -> fnv1a64:ff879e6952834399
line 54	R041	Discharged	fnv1a64:01231ecc0c2fadbf -> fnv1a64:0f9262208bf0c042
line 55	R051	Discharged	fnv1a64:4e9c568e07233faa -> fnv1a64:a4319d59894f0073
line 56	R061	Discharged	fnv1a64:a05cd6ef28c545f5 -> fnv1a64:482d2734f22d6230
line 57	R071	Discharged	fnv1a64:4324489625adc4e4 -> fnv1a64:13402060306ca8f5
line 58	R081	Discharged	fnv1a64:6f86cebef551a83b -> fnv1a64:e7f05698ae44973e
line 59	R091	Discharged	fnv1a64:f68cf67a3d30e282 -> fnv1a64:f4ac6a113041817b
line 60	R101	Discharged	fnv1a64:132dc485a725d552 -> fnv1a64:4ac135d5271d95ab
line 61	R111	Discharged	fnv1a64:71be30473cb6548d -> fnv1a64:bdd58a05aa4d0558
line 62	R12	Discharged	fnv1a64:367c3bf364a95ccd -> fnv1a64:b42eb7a498caab18
line 63	R13	Discharged	fnv1a64:4661dd148a845941 -> fnv1a64:848fb80a388addfc
line 64	R14	Discharged	fnv1a64:cdf473e03656933b -> fnv1a64:465dfbb9ef49823e
line 65	R15	Discharged	fnv1a64:d6211f1157a2c8ef -> fnv1a64:6fb2e98209ebd512
line 66	O010	Discharged	fnv1a64:8657a904eb6cc4d5 -> fnv1a64:727c325480eaa290
line 67	O0200	Discharged	fnv1a64:0482bc1be9c9c342 -> fnv1a64:14696346d1c3d4bb
line 68	O011	Discharged	fnv1a64:6aece94136cbd612 -> fnv1a64:1feaa667998cbaeb
line 69	O0210	Discharged	fnv1a64:d1f0af645d8b8b2d -> fnv1a64:e270bc39c7c94e78
```

Certificate changes are exactly C01 (TSV line 2) and C02 (line 3), default column 5 and no-default column 6. Both newly earned active cells match on the third-run replay. The four-cell identity/content proof is [certificate-replay.json](../../../.temp/brief-559/serial/certificate-replay.json); prior old/new cell hashes and non-fingerprint equality are [certificate-reconciliation.json](../../../.temp/brief-559/resume/certificate-reconciliation.json). Neither production body, its recorded instances, nor any admission/exclusion changed.

### 2. Verification and the scope stop

**Run 1:** saved immutable base binaries/source; authored red tests before production changes (19 failed, two cancellation controls already passed); repaired six sites; 21/21 green; stopped at the original inventory fence. **Run 2:** applied the director-authorized sixteen additions and source positions, formatted, refreshed fingerprints/certificates, fixed only the new frame storage and frozen frame-type inventories required by existing gates; final runtime 1,116 passed, focused 21 passed, fmt exit 0. Its broad stdlib run was interrupted and has no completed result. **Run 3:** serial discovery, fingerprint comparison and both certificate gates green; full stdlib 1,093 passed/53 ignored; workspace fails in CLI as detailed below.

All third-run cargo commands used `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true`; no two cargo commands were started together. TMPDIR was confined to `.temp/brief-559/tmp`. No git, network request, new dependency, lease, unsafe code or unrelated source edit occurred.

**Timeout disclosure:** the saved full-suite measurements were parallel. The first full serial stdlib run used an explicit 1,800s bootstrap cap and measured 1,046.980532s. This bootstrap cap was **not** derived from a prior full serial run and therefore does not fulfill the literal prior-serial-measurement requirement; this is disclosed rather than relabeled as measured evidence. The subsequent workspace bound was 2,200s (over twice that measured serial suite duration). The isolated discovery/fingerprint/default-certificate bounds were 80/24/120s; their saved earlier tests each executed a single case, but earlier compilation used four jobs. No-default certificate bound 130s used the newly measured serial default certificate 32.430571s. No parallel full-suite time was used as the new serial full-suite bound.

#### CLI failure evidence

`eval_tests.rs:114–137` starts the current libtest executable with `--exact ... --nocapture` and inherits `RUST_TEST_THREADS=1`. Serialized libtest emits `test NAME ... ` before the guest's first output on the same line. The assertions at lines 183–190 and 411–414 require bare `cleanup` or `prefix` lines; the assertion at 227–231 filters lines starting with `inner:`/`outer:`. All three child invocations first passed `assert_child_passed`, so their internal cleanup/reuse assertions completed. The first two failures directly show the output joined to the harness prefix. The third failure's missing first-line match is consistent with the same mechanism; its full child stdout is not printed by that assertion, so this last attribution is an inference from the identical helper and filter.

The CLI test file is byte-identical to the pre-brief snapshot (SHA256 `5209182d30654efb9d80c218039374ac7482d93da21474d5ba7cd13dad4a1c56`), recorded in [cli-scope-evidence.json](../../../.temp/brief-559/serial/cli-scope-evidence.json). The preserved base test executable was not built/run; no base CLI-test pass/fail claim is made. The fixture programs do not call the five repaired builtin families. No test or thread-setting workaround was attempted.

Verbatim failed workspace output (including successful child summaries, which are not additional workspace cases):

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-jupyter v0.2.0 (/home/remi/dev/rum/crates/rum-jupyter)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
warning: function `register_test_if_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:320:27
    |
320 | pub(in crate::runtime) fn register_test_if_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: `#[warn(dead_code)]` on by default

warning: function `register_test_return_builtin` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:330:27
    |
330 | pub(in crate::runtime) fn register_test_return_builtin(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `register_test_vm_template_builtins` is never used
   --> crates/rum-runtime/src/runtime/test_builtins.rs:358:27
    |
358 | pub(in crate::runtime) fn register_test_vm_template_builtins(session: &mut Session) {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `rum-runtime` (lib test) generated 3 warnings
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3m 05s
     Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)

running 15 tests
test eval_tests::already_broken_pipe_returns_with_reusable_session ... ok
test eval_tests::bindings_consume_exactly_their_queued_responses ... ok
test eval_tests::broken_pipe_during_cleanup_still_retires_input ... ok
test eval_tests::cleanup_messages_appear_once_with_original_input_reason ... 
thread 'eval_tests::cleanup_messages_appear_once_with_original_input_reason' panicked at crates/rum-cli/src/eval_tests.rs:411:9:
assertion `left == right` failed: script: 
running 1 test
test eval_tests::cleanup_messages_appear_once_with_original_input_reason ... cleanup
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s


  left: 0
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test eval_tests::cleanup_output_appears_once_and_tail_never_runs ... 
thread 'eval_tests::cleanup_output_appears_once_and_tail_never_runs' panicked at crates/rum-cli/src/eval_tests.rs:188:9:
assertion `left == right` failed: eval: 
running 1 test
test eval_tests::cleanup_output_appears_once_and_tail_never_runs ... prefix
cleanup
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s


  left: 0
 right: 1
FAILED
test eval_tests::eval_helper_abandons ... ! input required: p: 
ok
test eval_tests::file_helper_abandons ... ! input required: p: 
ok
test eval_tests::public_file_nested_get_rolls_back_inner_to_outer ... 
thread 'eval_tests::public_file_nested_get_rolls_back_inner_to_outer' panicked at crates/rum-cli/src/eval_tests.rs:231:5:
assertion `left == right` failed
  left: ["outer:2"]
 right: ["inner:3", "outer:2"]
FAILED
test eval_tests::refused_admission_preserves_someone_elses_pending_request ... ! cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first
! cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first
! cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first
! cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first
ok
test eval_tests::retirement_does_not_consume_a_queued_future_answer ... ! input required: p: 
ok
test eval_tests::script_helper_abandons ... ! input required: p: 
ok
test eval_tests::script_keeps_earlier_forms_after_nested_get_retirement ... ! input required: nested: 
ok
test eval_tests::shared_request_helper_abandons ... ! input required: p: 
ok
test eval_tests::terminal_stdin_does_not_make_scripts_interactive ... ! input required: p: 
! input required: p: 
ok
test eval_tests::unexpected_setup_input_abandons ... ok

failures:

failures:
    eval_tests::cleanup_messages_appear_once_with_original_input_reason
    eval_tests::cleanup_output_appears_once_and_tail_never_runs
    eval_tests::public_file_nested_get_rolls_back_inner_to_outer

test result: FAILED. 12 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

error: test failed, to rerun pass `-p rum-cli --bin rum-cli`
```

Repairing the output harness is outside scope. Stopping here leaves the CLI untouched and preserves the serial instruction. Director resolution is needed before any such repair or thread-policy exception.

#### Red before / green after, every permanent test

| Permanent test | Final pre-edit red run | Latest full stdlib run |
| --- | --- | --- |
| `differences` | FAILED | ok |
| `differences_cancel` | FAILED | ok |
| `differences_error_after_resume` | FAILED | ok |
| `differences_order_two` | FAILED | ok |
| `differences_twice` | FAILED | ok |
| `dot_cancel` | ok (cancellation control) | ok |
| `dot_chain` | FAILED | ok |
| `dot_error_after_resume` | FAILED | ok |
| `dot_twice` | FAILED | ok |
| `option_value_cancel` | FAILED | ok |
| `option_value_error_after_resume` | FAILED | ok |
| `optionvalue_many` | FAILED | ok |
| `optionvalue_twice` | FAILED | ok |
| `replace_repeated_cancel` | ok (cancellation control) | ok |
| `replace_repeated_error_after_resume` | FAILED | ok |
| `replacerepeated_predicate` | FAILED | ok |
| `replacerepeated_twice` | FAILED | ok |
| `scan` | FAILED | ok |
| `scan_cancel` | FAILED | ok |
| `scan_error_after_resume` | FAILED | ok |
| `scan_twice` | FAILED | ok |

The initial report above already preserves the full red and first-green witness output. Complete current-source focused output is [loops-final.txt](../../../.temp/brief-559/resume/loops-final.txt); full pre-edit output is [red/loops-final.txt](../../../.temp/brief-559/red/loops-final.txt). Final stdlib integration output, verbatim:

```text
running 21 tests
test differences ... ok
test differences_cancel ... ok
test differences_error_after_resume ... ok
test differences_order_two ... ok
test differences_twice ... ok
test dot_cancel ... ok
test dot_chain ... ok
test dot_error_after_resume ... ok
test dot_twice ... ok
test option_value_cancel ... ok
test option_value_error_after_resume ... ok
test optionvalue_many ... ok
test optionvalue_twice ... ok
test replace_repeated_cancel ... ok
test replace_repeated_error_after_resume ... ok
test replacerepeated_predicate ... ok
test replacerepeated_twice ... ok
test scan ... ok
test scan_cancel ... ok
test scan_error_after_resume ... ok
test scan_twice ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

#### Every captured build, test and maintenance command across the three runs

The ledger below preserves the actual commands, numeric bounds, exit statuses and measured durations from [commands.jsonl](../../../.temp/brief-559/commands.jsonl). Each excerpt is copied verbatim from its linked raw output. Successful fmt commands are silent. Earlier commands retain their historical four-job invocation; this does not describe the third-run policy. The host-killed second-run stdlib command has only a partial log and no terminal result, so it is not invented as a completed ledger entry. Read-only inspections and report/checkpoint edits were bounded at 30s and are not test gates.

**1. [.temp/brief-559/base/default-build.txt](../../../.temp/brief-559/base/default-build.txt) — bound 300s; exit 0; elapsed 34.212195s**

```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo build --release --target-dir .temp/brief-559/base/target -p rum-cli
```

```text
    Finished `release` profile [optimized] target(s) in 34.13s
```

**2. [.temp/brief-559/base/alloc-build.txt](../../../.temp/brief-559/base/alloc-build.txt) — bound 30s; exit 0; elapsed 1.206638s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo build --release --target-dir .temp/brief-559/base/target -p rum-cli --features alloc-count
```

```text
    Finished `release` profile [optimized] target(s) in 1.10s
```

**3. [.temp/brief-559/red/loops-01.txt](../../../.temp/brief-559/red/loops-01.txt) — bound 80s; exit 101; elapsed 7.124798s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.89s
test result: FAILED. 2 passed; 19 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
error: test failed, to rerun pass `-p rum-stdlib --test loop_continuations`
```

**4. [.temp/brief-559/red/loops-final.txt](../../../.temp/brief-559/red/loops-final.txt) — bound 80s; exit 101; elapsed 6.815308s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.62s
test result: FAILED. 2 passed; 19 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
error: test failed, to rerun pass `-p rum-stdlib --test loop_continuations`
```

**5. [.temp/brief-559/raw/loops-green-01.txt](../../../.temp/brief-559/raw/loops-green-01.txt) — bound 90s; exit 101; elapsed 2.253031s**

```text
python3 scripts/run-bounded.py --timeout 90 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```

```text
error[E0382]: use of moved value: `current`
error[E0382]: use of moved value: `rules`
error[E0382]: borrow of moved value: `explicit`
error[E0382]: use of moved value: `values`
error: could not compile `rum-runtime` (lib) due to 4 previous errors
```

**6. [.temp/brief-559/raw/loops-green-02.txt](../../../.temp/brief-559/raw/loops-green-02.txt) — bound 90s; exit 101; elapsed 5.163689s**

```text
python3 scripts/run-bounded.py --timeout 90 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```

```text
error[E0405]: cannot find trait `FrameAbandon` in this scope
error[E0412]: cannot find type `AbandonmentContext` in this scope
error[E0412]: cannot find type `FrameAbandonAction` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameAbandonAction`
error[E0405]: cannot find trait `FrameRootTrace` in this scope
error[E0405]: cannot find trait `FrameRootVisitor` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameRootTrace`
error[E0405]: cannot find trait `FrameAbandon` in this scope
error[E0412]: cannot find type `AbandonmentContext` in this scope
error[E0412]: cannot find type `FrameAbandonAction` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameAbandonAction`
error[E0405]: cannot find trait `FrameRootTrace` in this scope
error[E0405]: cannot find trait `FrameRootVisitor` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameRootTrace`
error[E0405]: cannot find trait `FrameAbandon` in this scope
error[E0412]: cannot find type `AbandonmentContext` in this scope
error[E0412]: cannot find type `FrameAbandonAction` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameAbandonAction`
error[E0277]: the trait bound `DotSuspensionFrame: FrameAbandon` is not satisfied
error[E0277]: the trait bound `DifferencesOrderSuspensionFrame: FrameAbandon` is not satisfied
error[E0277]: the trait bound `DifferencesOrderSuspensionFrame: rum_runtime::kernel_api::FrameRootTrace` is not satisfied
error[E0277]: the trait bound `DifferencesRowSuspensionFrame: FrameAbandon` is not satisfied
error[E0277]: the trait bound `DifferencesRowSuspensionFrame: rum_runtime::kernel_api::FrameRootTrace` is not satisfied
error: could not compile `rum-stdlib` (lib) due to 23 previous errors
```

**7. [.temp/brief-559/raw/loops-green-03.txt](../../../.temp/brief-559/raw/loops-green-03.txt) — bound 90s; exit 0; elapsed 11.581996s**

```text
python3 scripts/run-bounded.py --timeout 90 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.34s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```

**8. [.temp/brief-559/raw/fmt-check-01.txt](../../../.temp/brief-559/raw/fmt-check-01.txt) — bound 30s; exit 1; elapsed 1.807871s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo fmt --check
```

```text
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/options/builtins.rs:278:
Diff in /home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/list_plumbing.rs:129:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:3:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:10:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:33:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:41:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:48:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:63:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:83:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:91:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:99:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:106:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:113:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:119:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:125:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:131:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:137:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:143:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:155:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:167:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:172:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:177:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:182:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:187:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:192:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:197:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:202:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:210:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:217:
```

**9. [.temp/brief-559/raw/ownership-discovery.txt](../../../.temp/brief-559/raw/ownership-discovery.txt) — bound 80s; exit 0; elapsed 28.401708s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 22.99s
UnknownHostPayload count: 63
UnknownConstruct count: 7562
Self-reaching type count: 60
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.28s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

**10. [.temp/brief-559/resume/fmt-apply.txt](../../../.temp/brief-559/resume/fmt-apply.txt) — bound 30s; exit 0; elapsed 1.697579s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo fmt --all
```

No stdout/stderr was emitted.

**11. [.temp/brief-559/resume/ownership-discovery.txt](../../../.temp/brief-559/resume/ownership-discovery.txt) — bound 80s; exit 0; elapsed 23.377317s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.01s
UnknownHostPayload count: 63
UnknownConstruct count: 7562
Self-reaching type count: 60
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.27s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

**12. [.temp/brief-559/resume/regenerate-fingerprints.txt](../../../.temp/brief-559/resume/regenerate-fingerprints.txt) — bound 80s; exit 0; elapsed 0.242661s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --lib tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --exact --ignored --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.12s
```

**13. [.temp/brief-559/resume/reconcile.txt](../../../.temp/brief-559/resume/reconcile.txt) — bound 30s; exit 0; elapsed 0.146960s**

```text
python3 scripts/run-bounded.py --timeout 30 -- python3 .temp/brief-559/reconcile.py
```

```text
host_payloads.tsv: 63 -> 63; added=0; positions=0; removed=0
unknown_attributes.tsv: 1429 -> 1434; added=5; positions=9; removed=0
unknown_captures.tsv: 2932 -> 2938; added=6; positions=24; removed=0
unknown_macros.tsv: 3097 -> 3102; added=5; positions=17; removed=0
unknown_storage.tsv: 88 -> 88; added=0; positions=0; removed=0
Contract fingerprints: 68 refreshed; first 18 fields identical; 62 Discharged, 6 Pending unchanged.
```

**14. [.temp/brief-559/resume/cert-default-generate.txt](../../../.temp/brief-559/resume/cert-default-generate.txt) — bound 180s; exit 101; elapsed 30.988131s**

```text
python3 scripts/run-bounded.py --timeout 180 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.83s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 17.08s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```

**15. [.temp/brief-559/resume/cert-nodefault-generate.txt](../../../.temp/brief-559/resume/cert-nodefault-generate.txt) — bound 180s; exit 101; elapsed 40.346712s**

```text
python3 scripts/run-bounded.py --timeout 180 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --no-default-features --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 23.01s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 17.17s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```

**16. [.temp/brief-559/resume/runtime.txt](../../../.temp/brief-559/resume/runtime.txt) — bound 100s; exit 101; elapsed 14.884440s**

```text
python3 scripts/run-bounded.py --timeout 100 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-runtime
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.54s
test result: FAILED. 1092 passed; 2 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.22s
error: test failed, to rerun pass `-p rum-runtime --lib`
```

**17. [.temp/brief-559/resume/fmt-check.txt](../../../.temp/brief-559/resume/fmt-check.txt) — bound 30s; exit 0; elapsed 1.599503s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo fmt --check
```

No stdout/stderr was emitted.

**18. [.temp/brief-559/resume/runtime-green.txt](../../../.temp/brief-559/resume/runtime-green.txt) — bound 100s; exit 101; elapsed 9.175221s**

```text
python3 scripts/run-bounded.py --timeout 100 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-runtime
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.91s
test result: FAILED. 1093 passed; 1 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.19s
error: test failed, to rerun pass `-p rum-runtime --lib`
```

**19. [.temp/brief-559/resume/fmt-apply-02.txt](../../../.temp/brief-559/resume/fmt-apply-02.txt) — bound 30s; exit 0; elapsed 1.548234s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo fmt --all
```

No stdout/stderr was emitted.

**20. [.temp/brief-559/resume/loops-green.txt](../../../.temp/brief-559/resume/loops-green.txt) — bound 90s; exit 101; elapsed 1.747638s**

```text
python3 scripts/run-bounded.py --timeout 90 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```

```text
error[E0308]: mismatched types
error[E0308]: mismatched types
error: could not compile `rum-runtime` (lib) due to 2 previous errors
```

**21. [.temp/brief-559/resume/runtime-final.txt](../../../.temp/brief-559/resume/runtime-final.txt) — bound 100s; exit 101; elapsed 2.556136s**

```text
python3 scripts/run-bounded.py --timeout 100 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-runtime
```

```text
error[E0308]: mismatched types
error[E0308]: mismatched types
error: could not compile `rum-runtime` (lib) due to 2 previous errors
error: could not compile `rum-runtime` (lib test) due to 2 previous errors
```

**22. [.temp/brief-559/resume/loops-final.txt](../../../.temp/brief-559/resume/loops-final.txt) — bound 90s; exit 0; elapsed 12.193048s**

```text
python3 scripts/run-bounded.py --timeout 90 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.95s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

**23. [.temp/brief-559/resume/fmt-final.txt](../../../.temp/brief-559/resume/fmt-final.txt) — bound 30s; exit 0; elapsed 1.748870s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo fmt --check
```

No stdout/stderr was emitted.

**24. [.temp/brief-559/resume/runtime-pass.txt](../../../.temp/brief-559/resume/runtime-pass.txt) — bound 100s; exit 0; elapsed 15.253260s**

```text
python3 scripts/run-bounded.py --timeout 100 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-runtime
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.76s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.06s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.50s
```

**25. [.temp/brief-559/resume/ownership-discovery-final.txt](../../../.temp/brief-559/resume/ownership-discovery-final.txt) — bound 80s; exit 0; elapsed 26.047116s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 20.64s
UnknownHostPayload count: 63
UnknownConstruct count: 7562
Self-reaching type count: 60
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.29s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

**26. [.temp/brief-559/resume/regenerate-fingerprints-final.txt](../../../.temp/brief-559/resume/regenerate-fingerprints-final.txt) — bound 80s; exit 0; elapsed 0.248791s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --lib tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --exact --ignored --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.12s
```

**27. [.temp/brief-559/resume/reconcile-final.txt](../../../.temp/brief-559/resume/reconcile-final.txt) — bound 30s; exit 0; elapsed 0.149028s**

```text
python3 scripts/run-bounded.py --timeout 30 -- python3 .temp/brief-559/reconcile.py
```

```text
host_payloads.tsv: 63 -> 63; added=0; positions=0; removed=0
unknown_attributes.tsv: 1429 -> 1434; added=5; positions=9; removed=0
unknown_captures.tsv: 2932 -> 2938; added=6; positions=24; removed=0
unknown_macros.tsv: 3097 -> 3102; added=5; positions=17; removed=0
unknown_storage.tsv: 88 -> 88; added=0; positions=0; removed=0
Contract fingerprints: 68 refreshed; first 18 fields identical; 62 Discharged, 6 Pending unchanged.
```

**28. [.temp/brief-559/resume/cert-default-final-generate.txt](../../../.temp/brief-559/resume/cert-default-final-generate.txt) — bound 180s; exit 101; elapsed 30.556782s**

```text
python3 scripts/run-bounded.py --timeout 180 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.44s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 17.00s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```

**29. [.temp/brief-559/resume/cert-nodefault-final-generate.txt](../../../.temp/brief-559/resume/cert-nodefault-final-generate.txt) — bound 180s; exit 101; elapsed 36.942160s**

```text
python3 scripts/run-bounded.py --timeout 180 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --no-default-features --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.57s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 17.24s
error: test failed, to rerun pass `-p rum-stdlib --lib`
```

**30. [.temp/brief-559/resume/install-certificates-final.txt](../../../.temp/brief-559/resume/install-certificates-final.txt) — bound 30s; exit 0; elapsed 0.059199s**

```text
python3 scripts/run-bounded.py --timeout 30 -- python3 .temp/brief-559/install-certificates.py
```

```text
Replayed four certificate cells; C01/C02 identities/status/source/preconditions/strength/exclusions unchanged.
```

**31. [.temp/brief-559/resume/scope-audit.txt](../../../.temp/brief-559/resume/scope-audit.txt) — bound 30s; exit 0; elapsed 0.290637s**

```text
python3 scripts/run-bounded.py --timeout 30 -- python3 .temp/brief-559/audit-final.py
```

```text
Scope audit: 12 original crate files changed; one new integration test; no unrelated Rust/config/dependency change.
```

**32. [.temp/brief-559/serial/ownership-discovery.txt](../../../.temp/brief-559/serial/ownership-discovery.txt) — bound 80s; exit 0; elapsed 5.513441s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --lib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
test tests::ownership_graph::discovery_report_without_inventory_input ... UnknownHostPayload count: 63
UnknownConstruct count: 7562
Self-reaching type count: 60
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.37s
```

**33. [.temp/brief-559/serial/regenerate-fingerprints.txt](../../../.temp/brief-559/serial/regenerate-fingerprints.txt) — bound 24s; exit 0; elapsed 0.200545s**

```text
python3 scripts/run-bounded.py --timeout 24 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --lib tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --exact --ignored --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.11s
```

**34. [.temp/brief-559/serial/reconcile.txt](../../../.temp/brief-559/serial/reconcile.txt) — bound 24s; exit 0; elapsed 0.142067s**

```text
python3 scripts/run-bounded.py --timeout 24 -- python3 .temp/brief-559/serial/reconcile.py
```

```text
host_payloads.tsv: 63 -> 63; added=0; positions=0; removed=0
unknown_attributes.tsv: 1429 -> 1434; added=5; positions=9; removed=0
unknown_captures.tsv: 2932 -> 2938; added=6; positions=24; removed=0
unknown_macros.tsv: 3097 -> 3102; added=5; positions=17; removed=0
unknown_storage.tsv: 88 -> 88; added=0; positions=0; removed=0
Contract fingerprints: 0 refreshed; 68 regenerated; first 18 fields identical; 62 Discharged, 6 Pending unchanged.
```

**35. [.temp/brief-559/serial/cert-default.txt](../../../.temp/brief-559/serial/cert-default.txt) — bound 120s; exit 0; elapsed 32.430571s**

```text
python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.07s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 17.20s
```

**36. [.temp/brief-559/serial/cert-nodefault.txt](../../../.temp/brief-559/serial/cert-nodefault.txt) — bound 130s; exit 0; elapsed 32.177274s**

```text
python3 scripts/run-bounded.py --timeout 130 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --no-default-features --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.09s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 16.96s
```

**37. [.temp/brief-559/serial/stdlib.txt](../../../.temp/brief-559/serial/stdlib.txt) — bound 1800s; exit 0; elapsed 1046.980532s**

```text
python3 scripts/run-bounded.py --timeout 1800 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1045.23s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**38. [.temp/brief-559/serial/workspace.txt](../../../.temp/brief-559/serial/workspace.txt) — bound 2200s; exit 101; elapsed 185.480404s**

```text
python3 scripts/run-bounded.py --timeout 2200 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture
```

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3m 05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
test result: FAILED. 12 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
error: test failed, to rerun pass `-p rum-cli --bin rum-cli`
```

**Native ZeroMQ:** no native test was reached in the third-run workspace command: cargo stopped at CLI. `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` was set, but there are **zero observed native skips to list**, not evidence that native tests passed. No native run was recorded in either earlier brief-559 run.

**Not completed:** workspace is red; no full `cargo test -p rum-stdlib --no-default-features ...retention_closure...` run; no candidate release/alloc build; no allocation, instruction or timing samples; no historical whole-census 195-probe replay. The no-default certificate acceptance test did run and pass, but is not the full no-default retention gate. `cargo fmt --check` and standalone `cargo test -p rum-runtime` last passed in run 2 on source whose hashes are unchanged in run 3; they were not rerun after the CLI scope stop.

### 3. Allocation and instruction rows

The original base remains unchanged in its own target directory. The base binaries still hash to:

```text
.temp/brief-559/base/rum-cli	sha256:f40c5b1f206d9a0f5771ccd71990227b13aa2d470a91eb6a49ec7596cfbfab5c
.temp/brief-559/base/rum-cli-alloc	sha256:046be5b3e9b64e9a0ecb2fac12d5f5e829c18433b7f846097c73de19138fc716
```

Saved base source lacks the five new frame types; candidate source contains them. Scan's base identity frame is documented in the saved patch. No candidate binary was built, so base/candidate binary inequality remains unchecked.

| Required label (N=20,000; 2N=40,000 prepared) | Allocation counts | `(count(2N)-count(N))/N`, `cpu_core/instructions:u` |
| --- | --- | --- |
| scan-no-input | NOT RUN | NOT RUN |
| optionvalue-three-no-input | NOT RUN | NOT RUN |
| replacerepeated-no-input | NOT RUN | NOT RUN |
| dot-three-no-input | NOT RUN | NOT RUN |
| differences-no-input | NOT RUN | NOT RUN |
| do-count-null | NOT RUN | NOT RUN |
| do-iterator-null | NOT RUN | NOT RUN |
| table-iterator-null | NOT RUN | NOT RUN |
| replaceall-simple | NOT RUN | NOT RUN |

The second run prepared [labels.json](../../../.temp/brief-559/perf/labels.json), eighteen `.rum` sources and scratch runners. None was run. No perf sample budget was consumed. Candidate build and allocation-first instruction/timing A/B remain pending; there is no “zero ordinary-path cost” acceptance claim and no measured regression/improvement claim. Before reuse, the old timing runner's four-job environment and uncalibrated bounds must be updated to the director's serial policy.

### 4. Counters, disclosures and exact next action

- Scoped sites repaired: **6/6**; minimum equality witnesses **11/11**; later-error tests **5/5**; cancellation tests **5/5**, exercising **10** suspended positions, with reusable sessions. Full stdlib confirms all 21 on current source.
- Frozen brief-548 lost count: **86 → 80** supported by these six scoped repairs, not a new 195-probe census or a completed acceptance/performance gate.
- Third-run completed maintenance: exact discovery, 68 fingerprint comparisons, four certificate cells in two passing build replays. Full stdlib: **1,093 passed, 53 ignored**. Prior current-source runtime: **1,116 passed, 3 ignored**; formatting exit 0. Workspace: **12 CLI tests passed, 3 failed**; later crates not reached.
- Stop-loss was 0 at the last green stdlib milestone; the one subsequent compiling workspace attempt did not add a green gate. Stop is the explicit scope fence before a second non-improving attempt, not exhaustion or an inferred inventory prohibition.
- No additional lost-continuation site, consumer or builtin was repaired or added to the objective. The disclosed newly encountered issue is the existing CLI test helper's line framing under mandated serial libtest output. No helper extraction, source redesign, leases, dependencies, unsafe code, custom-abandonment change or existing-test weakening.

Exact next action after a director ruling: address only the approved serial CLI test-harness invocation/assertion issue, then replay source-dependent fingerprints/certificates if any source changes; run the serial workspace gate and full no-default retention gate; build and verify a separate candidate against the preserved base; finish allocation-first N/2N instructions and the required timing A/B without a competing CPU workload. The measured serial stdlib duration is 1,046.980532s and the full workspace compilation observed here was 185s, so retain adequate bounds. Keep the original base immutable. Current state and this stop are saved in [.temp/brief-559/PROGRESS.md](../../../.temp/brief-559/PROGRESS.md).

## Third resume — the director authorizes one test-harness fix

You stopped correctly. The three failures are a latent defect in brief-555's
`rum-cli` test harness, exposed only by serial runs: `child()`
(`crates/rum-cli/src/eval_tests.rs:115-140`) re-runs the test binary with
`--nocapture` and inherits `RUST_TEST_THREADS=1`, and libtest's pretty
formatter, when single-threaded, writes `test <name> ... ` **before** the test
runs, so the child's first output line is joined onto it and the parent's
exact-line assertions miss it. The director's gate ran brief-555 with the
default thread count and never saw it.

**Authorized, and only this, outside the original fence:** in `child()`, pass
`--quiet` (the terse formatter writes nothing before a test runs) and remove
`RUST_TEST_THREADS` from the child's environment, so the child's output framing
no longer depends on how the parent was run. Change no assertion. Then prove it
both ways: `cargo test -p rum-cli` once with `RUST_TEST_THREADS=1` and once
without it, both green, raw output saved.

Then continue from `.temp/brief-559/PROGRESS.md`: replay the fingerprints and
certificates this edit expires, and finish the serial workspace gate, the
`--no-default-features` retention gate and the performance gate, still with
`env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1` for every cargo command except the
one default-thread `rum-cli` check above. Append one report covering all four
runs.

---

## Four-run execution report — correctness green; performance gate fails

This report covers all four executions. The six scoped continuation sites are repaired, all 21 permanent tests pass, and formatting, runtime, stdlib, the full serial workspace, both certificate builds and the full no-default-features retention gate have passing evidence. **The candidate is not an accepted performance gate:** OptionValue, ReplaceRepeated and Differences have repeatable ordinary-path instruction increases. Allocation counts and peaks are unchanged. Two scoped performance experiments were measured and rejected because they regressed other required labels; their source changes were fully reverted. The final source is byte-identical to the version that passed the broad gates.

No git command, network operation, user-configuration edit, new dependency, lease, unsafe block, existing-assertion weakening or unrelated retained source edit occurred. The only retained fourth-run Rust source edit is the director-authorized CLI child invocation.

### 1. What changed, frames and roots

| File | Final changed lines | Reason |
| --- | --- | --- |
| [crates/rum-cli/src/eval_tests.rs](../../../crates/rum-cli/src/eval_tests.rs) | 116–122 | In child(), add --quiet and remove RUST_TEST_THREADS from the child environment. Preserve every assertion. Both parent thread configurations pass. |
| [crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs](../../../crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs) | 149, 153–176, 444–472 | Carry the completed rewrite pass into evaluation/fixed-point comparison, preserving consumed iteration count and effective limit. |
| [crates/rum-runtime/src/runtime/options/builtins.rs](../../../crates/rum-runtime/src/runtime/options/builtins.rs) | 262–297, 299–300, 302–316, 522–550 | Share an indexed scalar driver; freeze names and completed values only on suspension; append the resumed scalar once and finish the List. |
| [crates/rum-runtime/src/runtime/tests/frame_root_trace.rs](../../../crates/rum-runtime/src/runtime/tests/frame_root_trace.rs) | 483 | Reconcile the exact frame count, 545 to 550; all 33 stateful identities and checker assertions remain. |
| [crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs](../../../crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs) | 343, 432, 721, 932–933, 1159, 1184, 1362, 1476–1477 | Add the five new frame identities to both exact derived-root and pure-abandonment data arrays. |
| [crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs](../../../crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs) | 451–459, 461–471, 478–482, 487, 490–492 | Replace Scan’s identity continuation with operator and remaining-parts state; continue the suffix and return Null. |
| [crates/rum-stdlib/src/functional_ops/arrays/mod.rs](../../../crates/rum-stdlib/src/functional_ops/arrays/mod.rs) | 3–5, after 40 (deletion), 48–57, 60–72, 79–99 | Continue Dot’s operand chain from the completed inner contraction, preserving head/arguments for suffix work and invalid-call reconstruction. |
| [crates/rum-stdlib/src/structure_ops/list_plumbing.rs](../../../crates/rum-stdlib/src/structure_ops/list_plumbing.rs) | 3–5, 28–45, 53–75, 119–145, 148–187 | Carry Differences row/prefix cursor and remaining order in separate frames; preserve final element-limit check and List wrapping. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv) | 436–441, 600–602, 1021, 1033, 1330–1332 | Five authorized derive additions and nine existing source-position updates; unknown classifications unchanged. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv) | 888–898, 1176–1180, 2050, 2072, 2691–2702 | Six authorized closure additions and 24 existing source-position updates; unknown classifications unchanged. |
| [crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv](../../../crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv) | 743–748, 1118–1121, 2170–2171, 2203, 2787–2795 | Five authorized macro additions and 17 existing source-position updates; unknown classifications unchanged. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv) | 2–69 | Regenerate all 68 fingerprint fields; the other 18 fields, 62 Discharged and six Pending, remain unchanged. |
| [crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv](../../../crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv) | 2–3 | Re-earn C01/C02 default/no-default certificate cells; identities, statuses, preconditions, strengths and exclusions remain unchanged. |
| [crates/rum-stdlib/tests/loop_continuations.rs](../../../crates/rum-stdlib/tests/loop_continuations.rs) | 1–373 | Add 21 permanent tests: 11 equality witnesses, five later errors and five cancellation cases covering ten suspended positions; every expected prompt and reusable session asserted. |

This brief has appended reports only. Scratch, raw logs, Python capture/reduction scripts, source snapshots, binaries, exact file/range audit and comparison patch are under `.temp/brief-559/`. See [final file audit](../../../.temp/brief-559/fourth/final-file-audit.json) and [final patch made from saved bytes](../../../.temp/brief-559/fourth/final-review.patch). No git was used.

| Frame | Root-bearing fields | Cursor and resume work |
| --- | --- | --- |
| EvaluateScanPartsSuspensionFrame (replaces the identity Scan frame) | operator: Expr; remaining_parts: IntoIter<Expr> | Iterator is the next-part cursor; discard completed callback result, finish suffix, return Null. |
| OptionValuesSuspensionFrame | target: Symbol; explicit: Option<SymbolOptions>; items: Arc<[(Expr, Symbol)]>; values: SharedArgs | next_index; reconstruct prefix only after suspension, append resumed scalar once, finish names and List wrapping. |
| ResumeReplaceRepeatedAfterReplaceSuspensionFrame | current: Expr; rules: RewriteRulePlan | iteration and effective_limit; evaluate completed rewrite, compare fixed point and enforce preserved limit. |
| DotSuspensionFrame | head: Expr; args: SharedArgs | next_index; resumed inner contraction is current tensor; finish remaining operands. |
| DifferencesRowSuspensionFrame | items: SharedArgs; differences: SharedArgs | next_pair; append resumed difference once, finish row and encode continuation value. |
| DifferencesOrderSuspensionFrame | None; completed row arrives as outcome | remaining_order; decode row, finish orders, check generated-element limit and wrap List. |

All six frames derive FrameRootTrace and PureFrameAbandon; existing custom abandonment is untouched. Scan is a replaced existing frame; the other five are new. Root inventories remain 965 derived types and 550 frames (517 pure, 33 stateful). Storage capture occurs only on InputRequired; the measured instruction regressions below mean that this is not a zero ordinary-path cost acceptance claim.

File-size assessment: implementation files remain under 20k chars (options 19,277; repeated rewrite 17,251; Scan 15,477; list plumbing 14,441; arrays 6,023). These coherent, fenced repairs retain their existing boundaries; future splits can separate OptionValue driving from parsing, Scan driving from normalization, repeated-loop frames from entry normalization, and Differences from broader list plumbing. CLI tests are 14,747 chars and the new integration tests 11,586. The pre-existing 30,342-char frame test changed only its count, and the pre-existing 167,818-char derived inventory changed only ten data entries; a future split can partition exact inventory lists by crate. No unrelated split or TODO edit was made.

**Ownership records:** all sixteen authorized additions are below, copied verbatim from discovery. The fifty existing source-position moves were fully disclosed in the preceding three-run report and remain identical; [the full before/after row proof](../../../.temp/brief-559/fourth/ownership-reconciliation.json) includes every move. No existing identity disappeared or changed classification. Host payloads remain 63, unknown constructs 7,562 and self-reaching types 60. The CLI test-only edit adds no discovery row.

```text
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	445	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	523	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
UnknownConstruct	crates/rum-stdlib/src/functional_ops/arrays/mod.rs	81	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	55	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	150	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	461	40	closure captures: |replaced| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	538	40	closure captures: |value| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs	490	40	closure captures: |_| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/functional_ops/arrays/mod.rs	95	40	closure captures: |current| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	67	40	closure captures: |row| (capture types/modes unresolved)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	164	40	closure captures: |value| (capture types/modes unresolved)
UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs	154	9	unexpanded macro crate :: resume_or_suspend! (ends at 166:10)
UnknownConstruct	crates/rum-runtime/src/runtime/options/builtins.rs	286	28	unexpanded macro crate :: resume_or_suspend! (ends at 297:22)
UnknownConstruct	crates/rum-stdlib/src/functional_ops/arrays/mod.rs	62	22	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 71:10)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	38	19	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 45:10)
UnknownConstruct	crates/rum-stdlib/src/structure_ops/list_plumbing.rs	133	24	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 142:18)
```
**Contract rows:** the following 68 rows changed only their fingerprint column during run 4; final regeneration found zero further differences. Earlier run-2 fingerprint transitions are recorded above and in [.temp/brief-559/resume/contract-reconciliation.json](../../../.temp/brief-559/resume/contract-reconciliation.json).

```text
line	id	status	previous	final
2	B01	Pending	fnv1a64:9b9b39b0fc5a90f6	fnv1a64:b60825c0d4ad8bb0
3	B02	Pending	fnv1a64:eb9407372bfc8db3	fnv1a64:f5bcf9719ba2a659
4	B03	Discharged	fnv1a64:f0681df4dd546fac	fnv1a64:10c54100af192a32
5	B04	Discharged	fnv1a64:d39f3b872237fa8e	fnv1a64:68790df1ee24c998
6	B05	Discharged	fnv1a64:d92d6fdfe68460f7	fnv1a64:60cf814c9e09266d
7	B06	Pending	fnv1a64:9b486b6d442c4542	fnv1a64:502664c9643cddac
8	B07	Pending	fnv1a64:2b69ceebbd2ec3a7	fnv1a64:accea6345595dcdd
9	B08	Pending	fnv1a64:2277ee2559205ba0	fnv1a64:c52accb1ffe9f126
10	B09	Discharged	fnv1a64:c8cb5c6d9d8ec1da	fnv1a64:3caa0f57160242d4
11	B10	Discharged	fnv1a64:32610982f50c5ac1	fnv1a64:16bd817dac78d233
12	B11	Discharged	fnv1a64:94f6ece43c2c208e	fnv1a64:29d0bf4f0818ef98
13	B12	Discharged	fnv1a64:87d7dc69799756da	fnv1a64:fbb68f52f20ad7d4
14	I01	Discharged	fnv1a64:797d24ae69128776	fnv1a64:93ea10be41658230
15	I02	Discharged	fnv1a64:306eed73debae8f7	fnv1a64:b810fee0963fae6d
16	I03	Pending	fnv1a64:6ae51703c4f8091a	fnv1a64:1477771138785614
17	I04	Discharged	fnv1a64:a7de9df1cab446d3	fnv1a64:de717acf3e818479
18	P01	Discharged	fnv1a64:34976f8fb0dc7ae2	fnv1a64:543932c211c8b64c
19	P02	Discharged	fnv1a64:aa3cf53d7745ef51	fnv1a64:26509d0eb1f48903
20	P03	Discharged	fnv1a64:18f26a672b2f11b4	fnv1a64:db55c84b16a504ca
21	P04	Discharged	fnv1a64:e06f254cb0a5a9b3	fnv1a64:ea981787204bc259
22	P05	Discharged	fnv1a64:944385f4a8561c86	fnv1a64:f8e5e9953ea90780
23	P06	Discharged	fnv1a64:d857013a05c6f05a	fnv1a64:4c35b4237e3a7154
24	P070	Discharged	fnv1a64:12a49d7be43c6a5a	fnv1a64:868350655cafeb54
25	P080	Discharged	fnv1a64:e64455b814cb9a01	fnv1a64:e716469d350f3373
26	P090	Discharged	fnv1a64:160f6f43fc20f3f3	fnv1a64:5f8713163d3c2199
27	P100	Discharged	fnv1a64:2c05ed441298c23e	fnv1a64:33a6591bc0a94fc8
28	P110	Discharged	fnv1a64:20bb0d6ab5896c29	fnv1a64:e40035d81e844a0b
29	P120	Discharged	fnv1a64:fa306e7538996340	fnv1a64:b831a042d5e6b246
30	P130	Discharged	fnv1a64:cf372848ebe29da7	fnv1a64:509bff918449b6dd
31	P140	Discharged	fnv1a64:19a1c172830caebb	fnv1a64:ad6e233c098427d1
32	P150	Discharged	fnv1a64:a0abf62d619cec13	fnv1a64:028ef385cd365ab9
33	P071	Discharged	fnv1a64:7aa212fe6619d1c7	fnv1a64:95fe7ac5cb05a2fd
34	P081	Discharged	fnv1a64:bd652f37ac72dafa	fnv1a64:c2ed9de989bb35f4
35	P091	Discharged	fnv1a64:546373a346877f22	fnv1a64:51550d4fc342448c
36	P101	Discharged	fnv1a64:18bdfe37eaed0929	fnv1a64:dc0326a553e7e70b
37	P111	Discharged	fnv1a64:5b80b026459f93ca	fnv1a64:bc05703a2ddb1d04
38	P121	Discharged	fnv1a64:b24717e19fc9443d	fnv1a64:cd1e18f2aeb116af
39	P131	Discharged	fnv1a64:71495ded588c1c0e	fnv1a64:062330582478eb18
40	P141	Discharged	fnv1a64:375131455eb3d782	fnv1a64:ac035c0150b6e2ec
41	P151	Discharged	fnv1a64:3a5eb7de231da100	fnv1a64:d04795ad74cdc206
42	B13	Discharged	fnv1a64:2958fef674c90142	fnv1a64:de36f85294d999ac
43	R01	Discharged	fnv1a64:891c0e27a74f0123	fnv1a64:72be7a3f134dc549
44	R02	Discharged	fnv1a64:a30bb913c87598f4	fnv1a64:ed2fc76368f2ff0a
45	R03	Discharged	fnv1a64:fecc75a41a617fbf	fnv1a64:76f9a30d6acc5f65
46	R040	Discharged	fnv1a64:df83db7a4778db43	fnv1a64:1975f06c44768769
47	R050	Discharged	fnv1a64:26e0a8e5220e2be6	fnv1a64:2f062694b1a79a60
48	R060	Discharged	fnv1a64:dd61e9df8bfaf66b	fnv1a64:77f5ae7226b7f601
49	R070	Discharged	fnv1a64:25d2f7518e1499fa	fnv1a64:2b5b66036b5cf4f4
50	R080	Discharged	fnv1a64:6ee306e49a390fd3	fnv1a64:a575e3c20e064d79
51	R090	Discharged	fnv1a64:6c392a884156837e	fnv1a64:37e2c5b8fd68c708
52	R100	Discharged	fnv1a64:b58dc247872a1a2c	fnv1a64:d5eae55358eed4b2
53	R110	Discharged	fnv1a64:ff879e6952834399	fnv1a64:3c3829018af8b53b
54	R041	Discharged	fnv1a64:0f9262208bf0c042	fnv1a64:c4705b7cac0158ac
55	R051	Discharged	fnv1a64:a4319d59894f0073	fnv1a64:eda9412bca6a2e19
56	R061	Discharged	fnv1a64:482d2734f22d6230	fnv1a64:2a0b1773c7ea3936
57	R071	Discharged	fnv1a64:13402060306ca8f5	fnv1a64:6ff9a7a982bb1857
58	R081	Discharged	fnv1a64:e7f05698ae44973e	fnv1a64:ef90c2705c5524c8
59	R091	Discharged	fnv1a64:f4ac6a113041817b	fnv1a64:57221ca13cda9491
60	R101	Discharged	fnv1a64:4ac135d5271d95ab	fnv1a64:01b520b0f5aea341
61	R111	Discharged	fnv1a64:bdd58a05aa4d0558	fnv1a64:f002ab64924cb82e
62	R12	Discharged	fnv1a64:b42eb7a498caab18	fnv1a64:0412c4569639a2ee
63	R13	Discharged	fnv1a64:848fb80a388addfc	fnv1a64:5704808f7388b402
64	R14	Discharged	fnv1a64:465dfbb9ef49823e	fnv1a64:4dfe67919d5a0fc8
65	R15	Discharged	fnv1a64:6fb2e98209ebd512	fnv1a64:4937bf2b4cb21cbc
66	O010	Discharged	fnv1a64:727c325480eaa290	fnv1a64:aac3291ab3077816
67	O0200	Discharged	fnv1a64:14696346d1c3d4bb	fnv1a64:a835c510583b4dd1
68	O011	Discharged	fnv1a64:1feaa667998cbaeb	fnv1a64:ba7e6afa3449ba81
69	O0210	Discharged	fnv1a64:e270bc39c7c94e78	fnv1a64:c93ff2d9ca20354e
```
**Certificate rows:** C01 is TSV line 2 and C02 line 3; columns 5 and 6 are default and no-default. Every non-fingerprint component and all other columns were checked unchanged. Final passing replays reproduce all four inventory cells exactly. SHA-256 identifies each complete serialized cell below; full earned TSVs are saved in fourth/.

| Row | TSV line / column | Feature hash | Before cell SHA-256 | After cell SHA-256 |
| --- | --- | --- | --- | --- |
| C01 | 2 / 5 | true | 74ad96653405e82d3fa579858a21098ce4f6f2b010fb8f4c4bbc837ac6150197 | b1dba4b4ec444b2c256198256ce8acc524f0da122d808b9dcb6f6f7c261b801a |
| C01 | 2 / 6 | false | e2dc9eb3d7d6a6273f51d7ddb7f2367bf55ebe9b1fe0ad75020f5ce9886605b1 | e07cb5f71d9e8bf2de3698fae2b5d50ec26fcafc313b2d4da7f5d588501b7583 |
| C02 | 3 / 5 | true | 90b8aab5d4244dd5fad40eb833b5db9b1074f8d79d45ee1f25b500c58b65da94 | 5fe6e16d3ae3284d206448232fdc3342f90c0a7797dd354f3ad88746ca391d99 |
| C02 | 3 / 6 | false | 1b472c16efb54dfd28032ccaba75589760fd89cf4aaf4f6edbfc8f89cae2f630 | b57f28a9d803497905de408a71fd6286ced7e98aab9b0d66005fcdccc239d898 |

### 2. Verification across all four runs

1. Run 1 saved immutable base binaries and source, recorded 19 failing and two passing controls before production edits, repaired six sites, and passed all 21 focused tests; it stopped at the original ownership fence.
2. Run 2 applied the authorized sixteen UnknownConstruct additions, corrected only the new frame storage and exact frame inventories required by existing gates, and passed runtime (1116) and focused tests (21); the host interrupted its broad test run.
3. Run 3 ran serial discovery, fingerprints and certificate replays, then passed full stdlib (1093); workspace stopped at three CLI output-framing failures.
4. Run 4 changed exactly the director-authorized child invocation, passed CLI tests both serially and with default threads, refreshed fingerprints and certificate cells, and completed the remaining gates recorded below.

Every fourth-run cargo command uses CARGO_BUILD_JOBS=1, CARGO_NET_OFFLINE=true and repo-local TMPDIR; RUST_TEST_THREADS=1 except the single explicitly authorized default-thread CLI check. No two cargo commands ran together. All commands used the bounded runner directly or through the capture/benchmark wrappers. Raw output was retained before reduction. Compile-fail diagnostics inside passing host, retention and rustdoc tests are deliberate rejection controls.

Timeout provenance: full workspace 2,200s used measured serial stdlib 1,046.98s plus the observed serial workspace compilation 185.48s; no-default retention used measured default stdlib 905.38s within this run, with the same 2,200s bound. The first serial CLI 240s bound used the 185.48s serial workspace compilation; the default-thread CLI 110s bound used the measured serial CLI 37.16s. Certificate bounds 120/130s came from the preceding serial 32.43/32.18s replays. The first isolated serial release used 400s from more than twice the measured serial workspace compilation; **this is a cross-profile bootstrap bound, not a prior same-profile serial release measurement**. It measured 108.35s; subsequent release bounds used that and the measured 87.91s rebuild. Alloc-feature rebuilds measured 2.32s. Microbench bounds used brief-550’s sequential full-suite 1.81s and newly measured same-shape panels (about 2.9s). The third-run initial serial-stdlib bootstrap caveat recorded above still applies.

**Final correctness:** workspace exit 0 with 2,877 reported passed / 0 failed / 58 ignored; nine native early-return skips are included in that reported pass total. Runtime is 1,094 unit + 22 rustdoc = 1,116 passed, three ignored. Stdlib is 1,025 unit + 47 guest-loading + 21 loop = 1,093 passed, 53 ignored. No-default retention: 49 passed, one ignored, 1,022 filtered. CLI: 58 passed in each required parent-thread configuration. Final focused tests: 21 passed; final fmt emitted no output and exited 0. Both certificate builds pass.

**Every permanent test, red before production edits and final green after:**

| Test | Pre-edit result | Final result |
| --- | --- | --- |
| differences | FAILED | ok |
| differences_cancel | FAILED | ok |
| differences_error_after_resume | FAILED | ok |
| differences_order_two | FAILED | ok |
| differences_twice | FAILED | ok |
| dot_cancel | ok (control) | ok |
| dot_chain | FAILED | ok |
| dot_error_after_resume | FAILED | ok |
| dot_twice | FAILED | ok |
| option_value_cancel | FAILED | ok |
| option_value_error_after_resume | FAILED | ok |
| optionvalue_many | FAILED | ok |
| optionvalue_twice | FAILED | ok |
| replace_repeated_cancel | ok (control) | ok |
| replace_repeated_error_after_resume | FAILED | ok |
| replacerepeated_predicate | FAILED | ok |
| replacerepeated_twice | FAILED | ok |
| scan | FAILED | ok |
| scan_cancel | FAILED | ok |
| scan_error_after_resume | FAILED | ok |
| scan_twice | FAILED | ok |

The full final pre-edit witness output and first green output remain verbatim in the initial report above. Final full witness output: [.temp/brief-559/fourth/final-loops.txt](../../../.temp/brief-559/fourth/final-loops.txt). No guest-loading helper was extracted or weakened.

**Native ZeroMQ skips, verbatim:**

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
**Every captured build, test and maintenance command:** the ledger preserves earlier four-job invocations as historical evidence; those are not the policy of runs 3/4. Each excerpt below is copied from its linked raw output. Routine bounded read-only inspections and scratch/report edits are not verification gates. The host-interrupted second-run broad stdlib command has no terminal status and is not invented as a completed run. Nested benchmark executions and every raw counter are separately rendered in [the benchmark ledger](../../../.temp/brief-559/fourth/benchmark-ledger.md).

**1. [.temp/brief-559/base/default-build.txt](../../../.temp/brief-559/base/default-build.txt) — bound 300.0s; exit 0; elapsed 34.21219451801153s**

```text
python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo build --release --target-dir .temp/brief-559/base/target -p rum-cli
```
```text
    Finished `release` profile [optimized] target(s) in 34.13s
```
**2. [.temp/brief-559/base/alloc-build.txt](../../../.temp/brief-559/base/alloc-build.txt) — bound 30.0s; exit 0; elapsed 1.206638256990118s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo build --release --target-dir .temp/brief-559/base/target -p rum-cli --features alloc-count
```
```text
    Finished `release` profile [optimized] target(s) in 1.10s
```
**3. [.temp/brief-559/red/loops-01.txt](../../../.temp/brief-559/red/loops-01.txt) — bound 80.0s; exit 101; elapsed 7.124798158009071s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.89s
test result: FAILED. 2 passed; 19 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
error: test failed, to rerun pass `-p rum-stdlib --test loop_continuations`
```
**4. [.temp/brief-559/red/loops-final.txt](../../../.temp/brief-559/red/loops-final.txt) — bound 80.0s; exit 101; elapsed 6.815307882992784s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.62s
test result: FAILED. 2 passed; 19 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
error: test failed, to rerun pass `-p rum-stdlib --test loop_continuations`
```
**5. [.temp/brief-559/raw/loops-green-01.txt](../../../.temp/brief-559/raw/loops-green-01.txt) — bound 90.0s; exit 101; elapsed 2.2530312520102598s**

```text
python3 scripts/run-bounded.py --timeout 90 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```
```text
error[E0382]: use of moved value: `current`
error[E0382]: use of moved value: `rules`
error[E0382]: borrow of moved value: `explicit`
error[E0382]: use of moved value: `values`
error: could not compile `rum-runtime` (lib) due to 4 previous errors
```
**6. [.temp/brief-559/raw/loops-green-02.txt](../../../.temp/brief-559/raw/loops-green-02.txt) — bound 90.0s; exit 101; elapsed 5.163689056993462s**

```text
python3 scripts/run-bounded.py --timeout 90 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```
```text
error[E0405]: cannot find trait `FrameAbandon` in this scope
error[E0412]: cannot find type `AbandonmentContext` in this scope
error[E0412]: cannot find type `FrameAbandonAction` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameAbandonAction`
error[E0405]: cannot find trait `FrameRootTrace` in this scope
error[E0405]: cannot find trait `FrameRootVisitor` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameRootTrace`
error[E0405]: cannot find trait `FrameAbandon` in this scope
error[E0412]: cannot find type `AbandonmentContext` in this scope
error[E0412]: cannot find type `FrameAbandonAction` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameAbandonAction`
error[E0405]: cannot find trait `FrameRootTrace` in this scope
error[E0405]: cannot find trait `FrameRootVisitor` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameRootTrace`
error[E0405]: cannot find trait `FrameAbandon` in this scope
error[E0412]: cannot find type `AbandonmentContext` in this scope
error[E0412]: cannot find type `FrameAbandonAction` in this scope
error[E0433]: failed to resolve: use of undeclared type `FrameAbandonAction`
error[E0277]: the trait bound `DotSuspensionFrame: FrameAbandon` is not satisfied
error[E0277]: the trait bound `DifferencesOrderSuspensionFrame: FrameAbandon` is not satisfied
error[E0277]: the trait bound `DifferencesOrderSuspensionFrame: rum_runtime::kernel_api::FrameRootTrace` is not satisfied
error[E0277]: the trait bound `DifferencesRowSuspensionFrame: FrameAbandon` is not satisfied
error[E0277]: the trait bound `DifferencesRowSuspensionFrame: rum_runtime::kernel_api::FrameRootTrace` is not satisfied
error: could not compile `rum-stdlib` (lib) due to 23 previous errors
```
**7. [.temp/brief-559/raw/loops-green-03.txt](../../../.temp/brief-559/raw/loops-green-03.txt) — bound 90.0s; exit 0; elapsed 11.58199571998557s**

```text
python3 scripts/run-bounded.py --timeout 90 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.34s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```
**8. [.temp/brief-559/raw/fmt-check-01.txt](../../../.temp/brief-559/raw/fmt-check-01.txt) — bound 30.0s; exit 1; elapsed 1.8078709569817875s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo fmt --check
```
```text
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/options/builtins.rs:278:
Diff in /home/remi/dev/rum/crates/rum-stdlib/src/structure_ops/list_plumbing.rs:129:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:3:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:10:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:33:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:41:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:48:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:63:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:83:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:91:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:99:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:106:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:113:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:119:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:125:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:131:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:137:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:143:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:155:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:167:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:172:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:177:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:182:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:187:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:192:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:197:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:202:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:210:
Diff in /home/remi/dev/rum/crates/rum-stdlib/tests/loop_continuations.rs:217:
```
**9. [.temp/brief-559/raw/ownership-discovery.txt](../../../.temp/brief-559/raw/ownership-discovery.txt) — bound 80.0s; exit 0; elapsed 28.40170797999599s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 22.99s
UnknownHostPayload count: 63
UnknownConstruct count: 7562
Self-reaching type count: 60
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.28s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```
**10. [.temp/brief-559/resume/fmt-apply.txt](../../../.temp/brief-559/resume/fmt-apply.txt) — bound 30.0s; exit 0; elapsed 1.6975786330003757s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo fmt --all
```
No stdout/stderr was emitted.

**11. [.temp/brief-559/resume/ownership-discovery.txt](../../../.temp/brief-559/resume/ownership-discovery.txt) — bound 80.0s; exit 0; elapsed 23.37731687197811s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.01s
UnknownHostPayload count: 63
UnknownConstruct count: 7562
Self-reaching type count: 60
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.27s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```
**12. [.temp/brief-559/resume/regenerate-fingerprints.txt](../../../.temp/brief-559/resume/regenerate-fingerprints.txt) — bound 80.0s; exit 0; elapsed 0.24266052400344051s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --lib tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --exact --ignored --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.12s
```
**13. [.temp/brief-559/resume/reconcile.txt](../../../.temp/brief-559/resume/reconcile.txt) — bound 30.0s; exit 0; elapsed 0.14695972501067445s**

```text
python3 scripts/run-bounded.py --timeout 30 -- python3 .temp/brief-559/reconcile.py
```
```text
host_payloads.tsv: 63 -> 63; added=0; positions=0; removed=0
unknown_attributes.tsv: 1429 -> 1434; added=5; positions=9; removed=0
unknown_captures.tsv: 2932 -> 2938; added=6; positions=24; removed=0
unknown_macros.tsv: 3097 -> 3102; added=5; positions=17; removed=0
unknown_storage.tsv: 88 -> 88; added=0; positions=0; removed=0
Contract fingerprints: 68 refreshed; first 18 fields identical; 62 Discharged, 6 Pending unchanged.
```
**14. [.temp/brief-559/resume/cert-default-generate.txt](../../../.temp/brief-559/resume/cert-default-generate.txt) — bound 180.0s; exit 101; elapsed 30.988130510988412s**

```text
python3 scripts/run-bounded.py --timeout 180 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.83s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 17.08s
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
error: test failed, to rerun pass `-p rum-stdlib --lib`
```
**15. [.temp/brief-559/resume/cert-nodefault-generate.txt](../../../.temp/brief-559/resume/cert-nodefault-generate.txt) — bound 180.0s; exit 101; elapsed 40.34671167001943s**

```text
python3 scripts/run-bounded.py --timeout 180 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --no-default-features --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 23.01s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 17.17s
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
error: test failed, to rerun pass `-p rum-stdlib --lib`
```
**16. [.temp/brief-559/resume/runtime.txt](../../../.temp/brief-559/resume/runtime.txt) — bound 100.0s; exit 101; elapsed 14.884440253983485s**

```text
python3 scripts/run-bounded.py --timeout 100 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-runtime
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 10.54s
test result: FAILED. 1092 passed; 2 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.22s
error: test failed, to rerun pass `-p rum-runtime --lib`
```
**17. [.temp/brief-559/resume/fmt-check.txt](../../../.temp/brief-559/resume/fmt-check.txt) — bound 30.0s; exit 0; elapsed 1.599503386998549s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo fmt --check
```
No stdout/stderr was emitted.

**18. [.temp/brief-559/resume/runtime-green.txt](../../../.temp/brief-559/resume/runtime-green.txt) — bound 100.0s; exit 101; elapsed 9.175220600998728s**

```text
python3 scripts/run-bounded.py --timeout 100 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-runtime
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.91s
test result: FAILED. 1093 passed; 1 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.19s
error: test failed, to rerun pass `-p rum-runtime --lib`
```
**19. [.temp/brief-559/resume/fmt-apply-02.txt](../../../.temp/brief-559/resume/fmt-apply-02.txt) — bound 30.0s; exit 0; elapsed 1.5482341310125776s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo fmt --all
```
No stdout/stderr was emitted.

**20. [.temp/brief-559/resume/loops-green.txt](../../../.temp/brief-559/resume/loops-green.txt) — bound 90.0s; exit 101; elapsed 1.7476375599799212s**

```text
python3 scripts/run-bounded.py --timeout 90 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```
```text
error[E0308]: mismatched types
error[E0308]: mismatched types
error: could not compile `rum-runtime` (lib) due to 2 previous errors
```
**21. [.temp/brief-559/resume/runtime-final.txt](../../../.temp/brief-559/resume/runtime-final.txt) — bound 100.0s; exit 101; elapsed 2.5561358660052065s**

```text
python3 scripts/run-bounded.py --timeout 100 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-runtime
```
```text
error[E0308]: mismatched types
error[E0308]: mismatched types
error: could not compile `rum-runtime` (lib) due to 2 previous errors
error: could not compile `rum-runtime` (lib test) due to 2 previous errors
```
**22. [.temp/brief-559/resume/loops-final.txt](../../../.temp/brief-559/resume/loops-final.txt) — bound 90.0s; exit 0; elapsed 12.193048207001993s**

```text
python3 scripts/run-bounded.py --timeout 90 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 11.95s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```
**23. [.temp/brief-559/resume/fmt-final.txt](../../../.temp/brief-559/resume/fmt-final.txt) — bound 30.0s; exit 0; elapsed 1.748869603005005s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo fmt --check
```
No stdout/stderr was emitted.

**24. [.temp/brief-559/resume/runtime-pass.txt](../../../.temp/brief-559/resume/runtime-pass.txt) — bound 100.0s; exit 0; elapsed 15.253260474011768s**

```text
python3 scripts/run-bounded.py --timeout 100 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-runtime
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.76s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 4.06s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.50s
```
**25. [.temp/brief-559/resume/ownership-discovery-final.txt](../../../.temp/brief-559/resume/ownership-discovery-final.txt) — bound 80.0s; exit 0; elapsed 26.04711617997964s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 20.64s
UnknownHostPayload count: 63
UnknownConstruct count: 7562
Self-reaching type count: 60
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.29s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```
**26. [.temp/brief-559/resume/regenerate-fingerprints-final.txt](../../../.temp/brief-559/resume/regenerate-fingerprints-final.txt) — bound 80.0s; exit 0; elapsed 0.2487913949880749s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --lib tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --exact --ignored --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.12s
```
**27. [.temp/brief-559/resume/reconcile-final.txt](../../../.temp/brief-559/resume/reconcile-final.txt) — bound 30.0s; exit 0; elapsed 0.14902800699928775s**

```text
python3 scripts/run-bounded.py --timeout 30 -- python3 .temp/brief-559/reconcile.py
```
```text
host_payloads.tsv: 63 -> 63; added=0; positions=0; removed=0
unknown_attributes.tsv: 1429 -> 1434; added=5; positions=9; removed=0
unknown_captures.tsv: 2932 -> 2938; added=6; positions=24; removed=0
unknown_macros.tsv: 3097 -> 3102; added=5; positions=17; removed=0
unknown_storage.tsv: 88 -> 88; added=0; positions=0; removed=0
Contract fingerprints: 68 refreshed; first 18 fields identical; 62 Discharged, 6 Pending unchanged.
```
**28. [.temp/brief-559/resume/cert-default-final-generate.txt](../../../.temp/brief-559/resume/cert-default-final-generate.txt) — bound 180.0s; exit 101; elapsed 30.556782308005495s**

```text
python3 scripts/run-bounded.py --timeout 180 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.44s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 17.00s
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
error: test failed, to rerun pass `-p rum-stdlib --lib`
```
**29. [.temp/brief-559/resume/cert-nodefault-final-generate.txt](../../../.temp/brief-559/resume/cert-nodefault-final-generate.txt) — bound 180.0s; exit 101; elapsed 36.94216039200546s**

```text
python3 scripts/run-bounded.py --timeout 180 -- env CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --no-default-features --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.57s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 17.24s
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
error: test failed, to rerun pass `-p rum-stdlib --lib`
```
**30. [.temp/brief-559/resume/install-certificates-final.txt](../../../.temp/brief-559/resume/install-certificates-final.txt) — bound 30.0s; exit 0; elapsed 0.05919916299171746s**

```text
python3 scripts/run-bounded.py --timeout 30 -- python3 .temp/brief-559/install-certificates.py
```
```text
Replayed four certificate cells; C01/C02 identities/status/source/preconditions/strength/exclusions unchanged.
```
**31. [.temp/brief-559/resume/scope-audit.txt](../../../.temp/brief-559/resume/scope-audit.txt) — bound 30.0s; exit 0; elapsed 0.2906373940058984s**

```text
python3 scripts/run-bounded.py --timeout 30 -- python3 .temp/brief-559/audit-final.py
```
```text
Scope audit: 12 original crate files changed; one new integration test; no unrelated Rust/config/dependency change.
```
**32. [.temp/brief-559/serial/ownership-discovery.txt](../../../.temp/brief-559/serial/ownership-discovery.txt) — bound 80.0s; exit 0; elapsed 5.513440890994389s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --lib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
test tests::ownership_graph::discovery_report_without_inventory_input ... UnknownHostPayload count: 63
UnknownConstruct count: 7562
Self-reaching type count: 60
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.37s
```
**33. [.temp/brief-559/serial/regenerate-fingerprints.txt](../../../.temp/brief-559/serial/regenerate-fingerprints.txt) — bound 24.0s; exit 0; elapsed 0.20054453201009892s**

```text
python3 scripts/run-bounded.py --timeout 24 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --lib tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --exact --ignored --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.11s
```
**34. [.temp/brief-559/serial/reconcile.txt](../../../.temp/brief-559/serial/reconcile.txt) — bound 24.0s; exit 0; elapsed 0.14206662998185493s**

```text
python3 scripts/run-bounded.py --timeout 24 -- python3 .temp/brief-559/serial/reconcile.py
```
```text
host_payloads.tsv: 63 -> 63; added=0; positions=0; removed=0
unknown_attributes.tsv: 1429 -> 1434; added=5; positions=9; removed=0
unknown_captures.tsv: 2932 -> 2938; added=6; positions=24; removed=0
unknown_macros.tsv: 3097 -> 3102; added=5; positions=17; removed=0
unknown_storage.tsv: 88 -> 88; added=0; positions=0; removed=0
Contract fingerprints: 0 refreshed; 68 regenerated; first 18 fields identical; 62 Discharged, 6 Pending unchanged.
```
**35. [.temp/brief-559/serial/cert-default.txt](../../../.temp/brief-559/serial/cert-default.txt) — bound 120.0s; exit 0; elapsed 32.43057072200463s**

```text
python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.07s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 17.20s
```
**36. [.temp/brief-559/serial/cert-nodefault.txt](../../../.temp/brief-559/serial/cert-nodefault.txt) — bound 130.0s; exit 0; elapsed 32.17727358700358s**

```text
python3 scripts/run-bounded.py --timeout 130 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --no-default-features --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.09s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 16.96s
```
**37. [.temp/brief-559/serial/stdlib.txt](../../../.temp/brief-559/serial/stdlib.txt) — bound 1800.0s; exit 0; elapsed 1046.980531830981s**

```text
python3 scripts/run-bounded.py --timeout 1800 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1045.23s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
**38. [.temp/brief-559/serial/workspace.txt](../../../.temp/brief-559/serial/workspace.txt) — bound 2200.0s; exit 101; elapsed 185.48040424397914s**

```text
python3 scripts/run-bounded.py --timeout 2200 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3m 05s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s
test result: FAILED. 12 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
error: test failed, to rerun pass `-p rum-cli --bin rum-cli`
```
**39. [.temp/brief-559/fourth/cli-serial.txt](../../../.temp/brief-559/fourth/cli-serial.txt) — bound 240.0s; exit 0; elapsed 37.16123390299617s**

```text
python3 scripts/run-bounded.py --timeout 240 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-cli
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 35.93s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
```
**40. [.temp/brief-559/fourth/cli-default-threads.txt](../../../.temp/brief-559/fourth/cli-default-threads.txt) — bound 110.0s; exit 0; elapsed 0.41530501598026603s**

```text
python3 scripts/run-bounded.py --timeout 110 -- env -u RUST_TEST_THREADS CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-cli
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
```
**41. [.temp/brief-559/fourth/fmt-check.txt](../../../.temp/brief-559/fourth/fmt-check.txt) — bound 30.0s; exit 0; elapsed 1.870673027005978s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo fmt --check
```
No stdout/stderr was emitted.

**42. [.temp/brief-559/fourth/ownership-discovery.txt](../../../.temp/brief-559/fourth/ownership-discovery.txt) — bound 80.0s; exit 0; elapsed 5.479600405000383s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --lib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
test tests::ownership_graph::discovery_report_without_inventory_input ... UnknownHostPayload count: 63
UnknownConstruct count: 7562
Self-reaching type count: 60
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.36s
```
**43. [.temp/brief-559/fourth/regenerate-fingerprints.txt](../../../.temp/brief-559/fourth/regenerate-fingerprints.txt) — bound 24.0s; exit 0; elapsed 0.2647349509934429s**

```text
python3 scripts/run-bounded.py --timeout 24 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --lib tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --exact --ignored --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.14s
```
**44. [.temp/brief-559/fourth/reconcile.txt](../../../.temp/brief-559/fourth/reconcile.txt) — bound 24.0s; exit 0; elapsed 0.16520874699926935s**

```text
python3 scripts/run-bounded.py --timeout 24 -- python3 .temp/brief-559/fourth/reconcile.py
```
```text
host_payloads.tsv: 63 -> 63; added=0; positions=0; removed=0
unknown_attributes.tsv: 1429 -> 1434; added=5; positions=9; removed=0
unknown_captures.tsv: 2932 -> 2938; added=6; positions=24; removed=0
unknown_macros.tsv: 3097 -> 3102; added=5; positions=17; removed=0
unknown_storage.tsv: 88 -> 88; added=0; positions=0; removed=0
Contract fingerprints: 68 refreshed; 68 regenerated; first 18 fields identical; 62 Discharged, 6 Pending unchanged.
```
**45. [.temp/brief-559/fourth/cert-default-generate.txt](../../../.temp/brief-559/fourth/cert-default-generate.txt) — bound 120.0s; exit 101; elapsed 31.034172851999756s**

```text
python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.04s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 16.88s
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
error: test failed, to rerun pass `-p rum-stdlib --lib`
```
**46. [.temp/brief-559/fourth/cert-nodefault-generate.txt](../../../.temp/brief-559/fourth/cert-nodefault-generate.txt) — bound 130.0s; exit 101; elapsed 32.2428607030015s**

```text
python3 scripts/run-bounded.py --timeout 130 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --no-default-features --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.05s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 17.04s
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
error: test failed, to rerun pass `-p rum-stdlib --lib`
```
**47. [.temp/brief-559/fourth/install-certificates.txt](../../../.temp/brief-559/fourth/install-certificates.txt) — bound 24.0s; exit 0; elapsed 0.06452120101312175s**

```text
python3 scripts/run-bounded.py --timeout 24 -- python3 .temp/brief-559/fourth/install-certificates.py
```
```text
Replayed four certificate cells; C01/C02 identities/status/source/preconditions/strength/exclusions unchanged.
```
**48. [.temp/brief-559/fourth/workspace.txt](../../../.temp/brief-559/fourth/workspace.txt) — bound 2200.0s; exit 0; elapsed 1011.8540624229936s**

```text
python3 scripts/run-bounded.py --timeout 2200 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace -- --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 22.95s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.25s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.19s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.18s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.11s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.72s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.73s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 44.06s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.81s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 19.01s
test tests::ownership_graph::discovery_report_without_inventory_input ... UnknownHostPayload count: 63
UnknownConstruct count: 7562
Self-reaching type count: 60
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7562 rows
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1025 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 905.38s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.79s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.33s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.31s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.19s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
**49. [.temp/brief-559/fourth/retention-nodefault.txt](../../../.temp/brief-559/fourth/retention-nodefault.txt) — bound 2200.0s; exit 0; elapsed 896.8441565249814s**

```text
python3 scripts/run-bounded.py --timeout 2200 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --no-default-features --lib tests::retention_closure -- --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.71s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 49 passed; 0 failed; 1 ignored; 0 measured; 1022 filtered out; finished in 881.97s
```
**50. [.temp/brief-559/fourth/scope-audit.txt](../../../.temp/brief-559/fourth/scope-audit.txt) — bound 24.0s; exit 0; elapsed 0.16441969899460673s**

```text
python3 scripts/run-bounded.py --timeout 24 -- python3 .temp/brief-559/fourth/audit.py
```
```text
Scope audit: 13 original crate files changed; one new integration test; no unrelated Rust/config/dependency change.
Fourth run changed exactly CLI child invocation and two retention inventories; assertions and brief prefix preserved.
```
**51. [.temp/brief-559/fourth/candidate-default-build.txt](../../../.temp/brief-559/fourth/candidate-default-build.txt) — bound 400.0s; exit 0; elapsed 108.35064043701277s**

```text
python3 scripts/run-bounded.py --timeout 400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo build --release --target-dir .temp/brief-559/candidate/target -p rum-cli
```
```text
    Finished `release` profile [optimized] target(s) in 1m 48s
```
**52. [.temp/brief-559/fourth/candidate-alloc-build.txt](../../../.temp/brief-559/fourth/candidate-alloc-build.txt) — bound 240.0s; exit 0; elapsed 2.321019573020749s**

```text
python3 scripts/run-bounded.py --timeout 240 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo build --release --target-dir .temp/brief-559/candidate/target -p rum-cli --features alloc-count
```
```text
    Finished `release` profile [optimized] target(s) in 2.23s
```
**53. [.temp/brief-559/fourth/alloc-base.txt](../../../.temp/brief-559/fourth/alloc-base.txt) — bound 200.0s; exit 0; elapsed 2.8758903830021154s**

```text
python3 scripts/run-bounded.py --timeout 200 -- python3 .temp/brief-559/fourth/perf-panel.py alloc base 1
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=base	sample=1	exit=0
scan-no-input	N=40000	build=base	sample=1	exit=0
optionvalue-three-no-input	N=20000	build=base	sample=1	exit=0
optionvalue-three-no-input	N=40000	build=base	sample=1	exit=0
replacerepeated-no-input	N=20000	build=base	sample=1	exit=0
replacerepeated-no-input	N=40000	build=base	sample=1	exit=0
dot-three-no-input	N=20000	build=base	sample=1	exit=0
dot-three-no-input	N=40000	build=base	sample=1	exit=0
differences-no-input	N=20000	build=base	sample=1	exit=0
differences-no-input	N=40000	build=base	sample=1	exit=0
do-count-null	N=20000	build=base	sample=1	exit=0
do-count-null	N=40000	build=base	sample=1	exit=0
do-iterator-null	N=20000	build=base	sample=1	exit=0
do-iterator-null	N=40000	build=base	sample=1	exit=0
table-iterator-null	N=20000	build=base	sample=1	exit=0
table-iterator-null	N=40000	build=base	sample=1	exit=0
replaceall-simple	N=20000	build=base	sample=1	exit=0
replaceall-simple	N=40000	build=base	sample=1	exit=0
```
**54. [.temp/brief-559/fourth/alloc-candidate.txt](../../../.temp/brief-559/fourth/alloc-candidate.txt) — bound 20.0s; exit 0; elapsed 2.875351091992343s**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-559/fourth/perf-panel.py alloc candidate 1
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=candidate	sample=1	exit=0
scan-no-input	N=40000	build=candidate	sample=1	exit=0
optionvalue-three-no-input	N=20000	build=candidate	sample=1	exit=0
optionvalue-three-no-input	N=40000	build=candidate	sample=1	exit=0
replacerepeated-no-input	N=20000	build=candidate	sample=1	exit=0
replacerepeated-no-input	N=40000	build=candidate	sample=1	exit=0
dot-three-no-input	N=20000	build=candidate	sample=1	exit=0
dot-three-no-input	N=40000	build=candidate	sample=1	exit=0
differences-no-input	N=20000	build=candidate	sample=1	exit=0
differences-no-input	N=40000	build=candidate	sample=1	exit=0
do-count-null	N=20000	build=candidate	sample=1	exit=0
do-count-null	N=40000	build=candidate	sample=1	exit=0
do-iterator-null	N=20000	build=candidate	sample=1	exit=0
do-iterator-null	N=40000	build=candidate	sample=1	exit=0
table-iterator-null	N=20000	build=candidate	sample=1	exit=0
table-iterator-null	N=40000	build=candidate	sample=1	exit=0
replaceall-simple	N=20000	build=candidate	sample=1	exit=0
replaceall-simple	N=40000	build=candidate	sample=1	exit=0
```
**55. [.temp/brief-559/fourth/instructions-base-1.txt](../../../.temp/brief-559/fourth/instructions-base-1.txt) — bound 40.0s; exit 0; elapsed 2.921211287000915s**

```text
python3 scripts/run-bounded.py --timeout 40 -- python3 .temp/brief-559/fourth/perf-panel.py instructions base 1
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=base	sample=1	exit=0
scan-no-input	N=40000	build=base	sample=1	exit=0
optionvalue-three-no-input	N=20000	build=base	sample=1	exit=0
optionvalue-three-no-input	N=40000	build=base	sample=1	exit=0
replacerepeated-no-input	N=20000	build=base	sample=1	exit=0
replacerepeated-no-input	N=40000	build=base	sample=1	exit=0
dot-three-no-input	N=20000	build=base	sample=1	exit=0
dot-three-no-input	N=40000	build=base	sample=1	exit=0
differences-no-input	N=20000	build=base	sample=1	exit=0
differences-no-input	N=40000	build=base	sample=1	exit=0
do-count-null	N=20000	build=base	sample=1	exit=0
do-count-null	N=40000	build=base	sample=1	exit=0
do-iterator-null	N=20000	build=base	sample=1	exit=0
do-iterator-null	N=40000	build=base	sample=1	exit=0
table-iterator-null	N=20000	build=base	sample=1	exit=0
table-iterator-null	N=40000	build=base	sample=1	exit=0
replaceall-simple	N=20000	build=base	sample=1	exit=0
replaceall-simple	N=40000	build=base	sample=1	exit=0
```
**56. [.temp/brief-559/fourth/instructions-candidate-1.txt](../../../.temp/brief-559/fourth/instructions-candidate-1.txt) — bound 40.0s; exit 0; elapsed 2.9228999399929307s**

```text
python3 scripts/run-bounded.py --timeout 40 -- python3 .temp/brief-559/fourth/perf-panel.py instructions candidate 1
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=candidate	sample=1	exit=0
scan-no-input	N=40000	build=candidate	sample=1	exit=0
optionvalue-three-no-input	N=20000	build=candidate	sample=1	exit=0
optionvalue-three-no-input	N=40000	build=candidate	sample=1	exit=0
replacerepeated-no-input	N=20000	build=candidate	sample=1	exit=0
replacerepeated-no-input	N=40000	build=candidate	sample=1	exit=0
dot-three-no-input	N=20000	build=candidate	sample=1	exit=0
dot-three-no-input	N=40000	build=candidate	sample=1	exit=0
differences-no-input	N=20000	build=candidate	sample=1	exit=0
differences-no-input	N=40000	build=candidate	sample=1	exit=0
do-count-null	N=20000	build=candidate	sample=1	exit=0
do-count-null	N=40000	build=candidate	sample=1	exit=0
do-iterator-null	N=20000	build=candidate	sample=1	exit=0
do-iterator-null	N=40000	build=candidate	sample=1	exit=0
table-iterator-null	N=20000	build=candidate	sample=1	exit=0
table-iterator-null	N=40000	build=candidate	sample=1	exit=0
replaceall-simple	N=20000	build=candidate	sample=1	exit=0
replaceall-simple	N=40000	build=candidate	sample=1	exit=0
```
**57. [.temp/brief-559/fourth/instructions-base-2.txt](../../../.temp/brief-559/fourth/instructions-base-2.txt) — bound 40.0s; exit 0; elapsed 2.875528206001036s**

```text
python3 scripts/run-bounded.py --timeout 40 -- python3 .temp/brief-559/fourth/perf-panel.py instructions base 2
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=base	sample=2	exit=0
scan-no-input	N=40000	build=base	sample=2	exit=0
optionvalue-three-no-input	N=20000	build=base	sample=2	exit=0
optionvalue-three-no-input	N=40000	build=base	sample=2	exit=0
replacerepeated-no-input	N=20000	build=base	sample=2	exit=0
replacerepeated-no-input	N=40000	build=base	sample=2	exit=0
dot-three-no-input	N=20000	build=base	sample=2	exit=0
dot-three-no-input	N=40000	build=base	sample=2	exit=0
differences-no-input	N=20000	build=base	sample=2	exit=0
differences-no-input	N=40000	build=base	sample=2	exit=0
do-count-null	N=20000	build=base	sample=2	exit=0
do-count-null	N=40000	build=base	sample=2	exit=0
do-iterator-null	N=20000	build=base	sample=2	exit=0
do-iterator-null	N=40000	build=base	sample=2	exit=0
table-iterator-null	N=20000	build=base	sample=2	exit=0
table-iterator-null	N=40000	build=base	sample=2	exit=0
replaceall-simple	N=20000	build=base	sample=2	exit=0
replaceall-simple	N=40000	build=base	sample=2	exit=0
```
**58. [.temp/brief-559/fourth/instructions-candidate-2.txt](../../../.temp/brief-559/fourth/instructions-candidate-2.txt) — bound 40.0s; exit 0; elapsed 2.9719649410108104s**

```text
python3 scripts/run-bounded.py --timeout 40 -- python3 .temp/brief-559/fourth/perf-panel.py instructions candidate 2
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=candidate	sample=2	exit=0
scan-no-input	N=40000	build=candidate	sample=2	exit=0
optionvalue-three-no-input	N=20000	build=candidate	sample=2	exit=0
optionvalue-three-no-input	N=40000	build=candidate	sample=2	exit=0
replacerepeated-no-input	N=20000	build=candidate	sample=2	exit=0
replacerepeated-no-input	N=40000	build=candidate	sample=2	exit=0
dot-three-no-input	N=20000	build=candidate	sample=2	exit=0
dot-three-no-input	N=40000	build=candidate	sample=2	exit=0
differences-no-input	N=20000	build=candidate	sample=2	exit=0
differences-no-input	N=40000	build=candidate	sample=2	exit=0
do-count-null	N=20000	build=candidate	sample=2	exit=0
do-count-null	N=40000	build=candidate	sample=2	exit=0
do-iterator-null	N=20000	build=candidate	sample=2	exit=0
do-iterator-null	N=40000	build=candidate	sample=2	exit=0
table-iterator-null	N=20000	build=candidate	sample=2	exit=0
table-iterator-null	N=40000	build=candidate	sample=2	exit=0
replaceall-simple	N=20000	build=candidate	sample=2	exit=0
replaceall-simple	N=40000	build=candidate	sample=2	exit=0
```
**59. [.temp/brief-559/fourth/inline-loops.txt](../../../.temp/brief-559/fourth/inline-loops.txt) — bound 80.0s; exit 0; elapsed 17.12255647900747s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.91s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```
**60. [.temp/brief-559/fourth/inline-default-build.txt](../../../.temp/brief-559/fourth/inline-default-build.txt) — bound 240.0s; exit 0; elapsed 87.91308905699407s**

```text
python3 scripts/run-bounded.py --timeout 240 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo build --release --target-dir .temp/brief-559/candidate/target -p rum-cli
```
```text
    Finished `release` profile [optimized] target(s) in 1m 27s
```
**61. [.temp/brief-559/fourth/inline-alloc-build.txt](../../../.temp/brief-559/fourth/inline-alloc-build.txt) — bound 20.0s; exit 0; elapsed 2.3199244209972676s**

```text
python3 scripts/run-bounded.py --timeout 20 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo build --release --target-dir .temp/brief-559/candidate/target -p rum-cli --features alloc-count
```
```text
    Finished `release` profile [optimized] target(s) in 2.25s
```
**62. [.temp/brief-559/fourth/alloc-inline.txt](../../../.temp/brief-559/fourth/alloc-inline.txt) — bound 20.0s; exit 0; elapsed 2.9271197720081545s**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-559/fourth/perf-panel.py alloc candidate-inline 1
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=candidate-inline	sample=1	exit=0
scan-no-input	N=40000	build=candidate-inline	sample=1	exit=0
optionvalue-three-no-input	N=20000	build=candidate-inline	sample=1	exit=0
optionvalue-three-no-input	N=40000	build=candidate-inline	sample=1	exit=0
replacerepeated-no-input	N=20000	build=candidate-inline	sample=1	exit=0
replacerepeated-no-input	N=40000	build=candidate-inline	sample=1	exit=0
dot-three-no-input	N=20000	build=candidate-inline	sample=1	exit=0
dot-three-no-input	N=40000	build=candidate-inline	sample=1	exit=0
differences-no-input	N=20000	build=candidate-inline	sample=1	exit=0
differences-no-input	N=40000	build=candidate-inline	sample=1	exit=0
do-count-null	N=20000	build=candidate-inline	sample=1	exit=0
do-count-null	N=40000	build=candidate-inline	sample=1	exit=0
do-iterator-null	N=20000	build=candidate-inline	sample=1	exit=0
do-iterator-null	N=40000	build=candidate-inline	sample=1	exit=0
table-iterator-null	N=20000	build=candidate-inline	sample=1	exit=0
table-iterator-null	N=40000	build=candidate-inline	sample=1	exit=0
replaceall-simple	N=20000	build=candidate-inline	sample=1	exit=0
replaceall-simple	N=40000	build=candidate-inline	sample=1	exit=0
```
**63. [.temp/brief-559/fourth/instructions-inline-1.txt](../../../.temp/brief-559/fourth/instructions-inline-1.txt) — bound 20.0s; exit 0; elapsed 3.023626897018403s**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-559/fourth/perf-panel.py instructions candidate-inline 1
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=candidate-inline	sample=1	exit=0
scan-no-input	N=40000	build=candidate-inline	sample=1	exit=0
optionvalue-three-no-input	N=20000	build=candidate-inline	sample=1	exit=0
optionvalue-three-no-input	N=40000	build=candidate-inline	sample=1	exit=0
replacerepeated-no-input	N=20000	build=candidate-inline	sample=1	exit=0
replacerepeated-no-input	N=40000	build=candidate-inline	sample=1	exit=0
dot-three-no-input	N=20000	build=candidate-inline	sample=1	exit=0
dot-three-no-input	N=40000	build=candidate-inline	sample=1	exit=0
differences-no-input	N=20000	build=candidate-inline	sample=1	exit=0
differences-no-input	N=40000	build=candidate-inline	sample=1	exit=0
do-count-null	N=20000	build=candidate-inline	sample=1	exit=0
do-count-null	N=40000	build=candidate-inline	sample=1	exit=0
do-iterator-null	N=20000	build=candidate-inline	sample=1	exit=0
do-iterator-null	N=40000	build=candidate-inline	sample=1	exit=0
table-iterator-null	N=20000	build=candidate-inline	sample=1	exit=0
table-iterator-null	N=40000	build=candidate-inline	sample=1	exit=0
replaceall-simple	N=20000	build=candidate-inline	sample=1	exit=0
replaceall-simple	N=40000	build=candidate-inline	sample=1	exit=0
```
**64. [.temp/brief-559/fourth/instructions-inline-2.txt](../../../.temp/brief-559/fourth/instructions-inline-2.txt) — bound 20.0s; exit 0; elapsed 2.9230357479827944s**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-559/fourth/perf-panel.py instructions candidate-inline 2
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=candidate-inline	sample=2	exit=0
scan-no-input	N=40000	build=candidate-inline	sample=2	exit=0
optionvalue-three-no-input	N=20000	build=candidate-inline	sample=2	exit=0
optionvalue-three-no-input	N=40000	build=candidate-inline	sample=2	exit=0
replacerepeated-no-input	N=20000	build=candidate-inline	sample=2	exit=0
replacerepeated-no-input	N=40000	build=candidate-inline	sample=2	exit=0
dot-three-no-input	N=20000	build=candidate-inline	sample=2	exit=0
dot-three-no-input	N=40000	build=candidate-inline	sample=2	exit=0
differences-no-input	N=20000	build=candidate-inline	sample=2	exit=0
differences-no-input	N=40000	build=candidate-inline	sample=2	exit=0
do-count-null	N=20000	build=candidate-inline	sample=2	exit=0
do-count-null	N=40000	build=candidate-inline	sample=2	exit=0
do-iterator-null	N=20000	build=candidate-inline	sample=2	exit=0
do-iterator-null	N=40000	build=candidate-inline	sample=2	exit=0
table-iterator-null	N=20000	build=candidate-inline	sample=2	exit=0
table-iterator-null	N=40000	build=candidate-inline	sample=2	exit=0
replaceall-simple	N=20000	build=candidate-inline	sample=2	exit=0
replaceall-simple	N=40000	build=candidate-inline	sample=2	exit=0
```
**65. [.temp/brief-559/fourth/cold-loops.txt](../../../.temp/brief-559/fourth/cold-loops.txt) — bound 80.0s; exit 0; elapsed 16.225860025995644s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.97s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```
**66. [.temp/brief-559/fourth/perf-fmt-check.txt](../../../.temp/brief-559/fourth/perf-fmt-check.txt) — bound 30.0s; exit 1; elapsed 1.8700956930115353s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo fmt --check
```
```text
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:155:
Diff in /home/remi/dev/rum/crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:162:
```
**67. [.temp/brief-559/fourth/cold-default-build.txt](../../../.temp/brief-559/fourth/cold-default-build.txt) — bound 180.0s; exit 0; elapsed 88.16959619001136s**

```text
python3 scripts/run-bounded.py --timeout 180 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo build --release --target-dir .temp/brief-559/candidate/target -p rum-cli
```
```text
    Finished `release` profile [optimized] target(s) in 1m 28s
```
**68. [.temp/brief-559/fourth/cold-alloc-build.txt](../../../.temp/brief-559/fourth/cold-alloc-build.txt) — bound 20.0s; exit 0; elapsed 2.3209038139902987s**

```text
python3 scripts/run-bounded.py --timeout 20 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo build --release --target-dir .temp/brief-559/candidate/target -p rum-cli --features alloc-count
```
```text
    Finished `release` profile [optimized] target(s) in 2.24s
```
**69. [.temp/brief-559/fourth/alloc-cold.txt](../../../.temp/brief-559/fourth/alloc-cold.txt) — bound 20.0s; exit 0; elapsed 2.9252826960000675s**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-559/fourth/perf-panel.py alloc candidate-cold 1
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=candidate-cold	sample=1	exit=0
scan-no-input	N=40000	build=candidate-cold	sample=1	exit=0
optionvalue-three-no-input	N=20000	build=candidate-cold	sample=1	exit=0
optionvalue-three-no-input	N=40000	build=candidate-cold	sample=1	exit=0
replacerepeated-no-input	N=20000	build=candidate-cold	sample=1	exit=0
replacerepeated-no-input	N=40000	build=candidate-cold	sample=1	exit=0
dot-three-no-input	N=20000	build=candidate-cold	sample=1	exit=0
dot-three-no-input	N=40000	build=candidate-cold	sample=1	exit=0
differences-no-input	N=20000	build=candidate-cold	sample=1	exit=0
differences-no-input	N=40000	build=candidate-cold	sample=1	exit=0
do-count-null	N=20000	build=candidate-cold	sample=1	exit=0
do-count-null	N=40000	build=candidate-cold	sample=1	exit=0
do-iterator-null	N=20000	build=candidate-cold	sample=1	exit=0
do-iterator-null	N=40000	build=candidate-cold	sample=1	exit=0
table-iterator-null	N=20000	build=candidate-cold	sample=1	exit=0
table-iterator-null	N=40000	build=candidate-cold	sample=1	exit=0
replaceall-simple	N=20000	build=candidate-cold	sample=1	exit=0
replaceall-simple	N=40000	build=candidate-cold	sample=1	exit=0
```
**70. [.temp/brief-559/fourth/instructions-cold-1.txt](../../../.temp/brief-559/fourth/instructions-cold-1.txt) — bound 20.0s; exit 0; elapsed 2.9223971929750405s**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-559/fourth/perf-panel.py instructions candidate-cold 1
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=candidate-cold	sample=1	exit=0
scan-no-input	N=40000	build=candidate-cold	sample=1	exit=0
optionvalue-three-no-input	N=20000	build=candidate-cold	sample=1	exit=0
optionvalue-three-no-input	N=40000	build=candidate-cold	sample=1	exit=0
replacerepeated-no-input	N=20000	build=candidate-cold	sample=1	exit=0
replacerepeated-no-input	N=40000	build=candidate-cold	sample=1	exit=0
dot-three-no-input	N=20000	build=candidate-cold	sample=1	exit=0
dot-three-no-input	N=40000	build=candidate-cold	sample=1	exit=0
differences-no-input	N=20000	build=candidate-cold	sample=1	exit=0
differences-no-input	N=40000	build=candidate-cold	sample=1	exit=0
do-count-null	N=20000	build=candidate-cold	sample=1	exit=0
do-count-null	N=40000	build=candidate-cold	sample=1	exit=0
do-iterator-null	N=20000	build=candidate-cold	sample=1	exit=0
do-iterator-null	N=40000	build=candidate-cold	sample=1	exit=0
table-iterator-null	N=20000	build=candidate-cold	sample=1	exit=0
table-iterator-null	N=40000	build=candidate-cold	sample=1	exit=0
replaceall-simple	N=20000	build=candidate-cold	sample=1	exit=0
replaceall-simple	N=40000	build=candidate-cold	sample=1	exit=0
```
**71. [.temp/brief-559/fourth/instructions-cold-2.txt](../../../.temp/brief-559/fourth/instructions-cold-2.txt) — bound 20.0s; exit 0; elapsed 2.9708847900037654s**

```text
python3 scripts/run-bounded.py --timeout 20 -- python3 .temp/brief-559/fourth/perf-panel.py instructions candidate-cold 2
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=candidate-cold	sample=2	exit=0
scan-no-input	N=40000	build=candidate-cold	sample=2	exit=0
optionvalue-three-no-input	N=20000	build=candidate-cold	sample=2	exit=0
optionvalue-three-no-input	N=40000	build=candidate-cold	sample=2	exit=0
replacerepeated-no-input	N=20000	build=candidate-cold	sample=2	exit=0
replacerepeated-no-input	N=40000	build=candidate-cold	sample=2	exit=0
dot-three-no-input	N=20000	build=candidate-cold	sample=2	exit=0
dot-three-no-input	N=40000	build=candidate-cold	sample=2	exit=0
differences-no-input	N=20000	build=candidate-cold	sample=2	exit=0
differences-no-input	N=40000	build=candidate-cold	sample=2	exit=0
do-count-null	N=20000	build=candidate-cold	sample=2	exit=0
do-count-null	N=40000	build=candidate-cold	sample=2	exit=0
do-iterator-null	N=20000	build=candidate-cold	sample=2	exit=0
do-iterator-null	N=40000	build=candidate-cold	sample=2	exit=0
table-iterator-null	N=20000	build=candidate-cold	sample=2	exit=0
table-iterator-null	N=40000	build=candidate-cold	sample=2	exit=0
replaceall-simple	N=20000	build=candidate-cold	sample=2	exit=0
replaceall-simple	N=40000	build=candidate-cold	sample=2	exit=0
```
**72. [.temp/brief-559/fourth/restored-scope-audit.txt](../../../.temp/brief-559/fourth/restored-scope-audit.txt) — bound 24.0s; exit 0; elapsed 0.16454199599684216s**

```text
python3 scripts/run-bounded.py --timeout 24 -- python3 .temp/brief-559/fourth/audit.py
```
```text
Scope audit: 13 original crate files changed; one new integration test; no unrelated Rust/config/dependency change.
Fourth run changed exactly CLI child invocation and two retention inventories; assertions and brief prefix preserved.
```
**73. [.temp/brief-559/fourth/timing-base-1.txt](../../../.temp/brief-559/fourth/timing-base-1.txt) — bound 14.0s; exit 0; elapsed 1.6703780730022117s**

```text
python3 scripts/run-bounded.py --timeout 14 -- python3 .temp/brief-559/fourth/timing-run.py base 1
```
```text
python3 scripts/run-bounded.py --timeout 14 -- taskset -c 2 scripts/core-language-microbench.sh --output-dir /home/remi/dev/rum/.temp/brief-559/fourth/perf/timing/base/1 --timeout 10 --build-timeout 300 --scale 1 --reps 7 --rum-cli /home/remi/dev/rum/.temp/brief-559/base/rum-cli < /dev/null
core microbench output: /home/remi/dev/rum/.temp/brief-559/fourth/perf/timing/base/1
```
**74. [.temp/brief-559/fourth/timing-candidate-1.txt](../../../.temp/brief-559/fourth/timing-candidate-1.txt) — bound 14.0s; exit 0; elapsed 1.568512441997882s**

```text
python3 scripts/run-bounded.py --timeout 14 -- python3 .temp/brief-559/fourth/timing-run.py candidate 1
```
```text
python3 scripts/run-bounded.py --timeout 14 -- taskset -c 2 scripts/core-language-microbench.sh --output-dir /home/remi/dev/rum/.temp/brief-559/fourth/perf/timing/candidate/1 --timeout 10 --build-timeout 300 --scale 1 --reps 7 --rum-cli /home/remi/dev/rum/.temp/brief-559/candidate/rum-cli < /dev/null
core microbench output: /home/remi/dev/rum/.temp/brief-559/fourth/perf/timing/candidate/1
```
**75. [.temp/brief-559/fourth/timing-candidate-2.txt](../../../.temp/brief-559/fourth/timing-candidate-2.txt) — bound 14.0s; exit 0; elapsed 1.570156253001187s**

```text
python3 scripts/run-bounded.py --timeout 14 -- python3 .temp/brief-559/fourth/timing-run.py candidate 2
```
```text
python3 scripts/run-bounded.py --timeout 14 -- taskset -c 2 scripts/core-language-microbench.sh --output-dir /home/remi/dev/rum/.temp/brief-559/fourth/perf/timing/candidate/2 --timeout 10 --build-timeout 300 --scale 1 --reps 7 --rum-cli /home/remi/dev/rum/.temp/brief-559/candidate/rum-cli < /dev/null
core microbench output: /home/remi/dev/rum/.temp/brief-559/fourth/perf/timing/candidate/2
```
**76. [.temp/brief-559/fourth/timing-base-2.txt](../../../.temp/brief-559/fourth/timing-base-2.txt) — bound 14.0s; exit 0; elapsed 1.5700311709952075s**

```text
python3 scripts/run-bounded.py --timeout 14 -- python3 .temp/brief-559/fourth/timing-run.py base 2
```
```text
python3 scripts/run-bounded.py --timeout 14 -- taskset -c 2 scripts/core-language-microbench.sh --output-dir /home/remi/dev/rum/.temp/brief-559/fourth/perf/timing/base/2 --timeout 10 --build-timeout 300 --scale 1 --reps 7 --rum-cli /home/remi/dev/rum/.temp/brief-559/base/rum-cli < /dev/null
core microbench output: /home/remi/dev/rum/.temp/brief-559/fourth/perf/timing/base/2
```
**77. [.temp/brief-559/fourth/scratch-timing-base.txt](../../../.temp/brief-559/fourth/scratch-timing-base.txt) — bound 40.0s; exit 0; elapsed 2.321660462010186s**

```text
python3 scripts/run-bounded.py --timeout 40 -- python3 .temp/brief-559/fourth/perf-panel.py scratch-timing base 1
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=base	sample=1	exit=0
optionvalue-three-no-input	N=20000	build=base	sample=1	exit=0
replacerepeated-no-input	N=20000	build=base	sample=1	exit=0
dot-three-no-input	N=20000	build=base	sample=1	exit=0
differences-no-input	N=20000	build=base	sample=1	exit=0
do-count-null	N=20000	build=base	sample=1	exit=0
do-iterator-null	N=20000	build=base	sample=1	exit=0
table-iterator-null	N=20000	build=base	sample=1	exit=0
replaceall-simple	N=20000	build=base	sample=1	exit=0
```
**78. [.temp/brief-559/fourth/scratch-timing-candidate.txt](../../../.temp/brief-559/fourth/scratch-timing-candidate.txt) — bound 40.0s; exit 0; elapsed 2.3247420150146354s**

```text
python3 scripts/run-bounded.py --timeout 40 -- python3 .temp/brief-559/fourth/perf-panel.py scratch-timing candidate 1
```
Panel rows below are a reduction of the saved JSON records; the individual commands, bounds, statuses and literal stdout/stderr/perf rows are copied in the benchmark ledger.

```text
scan-no-input	N=20000	build=candidate	sample=1	exit=0
optionvalue-three-no-input	N=20000	build=candidate	sample=1	exit=0
replacerepeated-no-input	N=20000	build=candidate	sample=1	exit=0
dot-three-no-input	N=20000	build=candidate	sample=1	exit=0
differences-no-input	N=20000	build=candidate	sample=1	exit=0
do-count-null	N=20000	build=candidate	sample=1	exit=0
do-iterator-null	N=20000	build=candidate	sample=1	exit=0
table-iterator-null	N=20000	build=candidate	sample=1	exit=0
replaceall-simple	N=20000	build=candidate	sample=1	exit=0
```
**79. [.temp/brief-559/fourth/perf-reduction.txt](../../../.temp/brief-559/fourth/perf-reduction.txt) — bound 24.0s; exit 0; elapsed 0.11425142601365224s**

```text
python3 scripts/run-bounded.py --timeout 24 -- python3 .temp/brief-559/fourth/reduce-perf.py
```
Verbatim allocation, instruction and timing rows from this successful reduction are pasted in section 3.

**80. [.temp/brief-559/fourth/final-loops.txt](../../../.temp/brief-559/fourth/final-loops.txt) — bound 80.0s; exit 0; elapsed 17.372343945025932s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --test loop_continuations -- --nocapture --test-threads=1
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.14s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```
**81. [.temp/brief-559/fourth/final-fmt.txt](../../../.temp/brief-559/fourth/final-fmt.txt) — bound 30.0s; exit 0; elapsed 1.6677135610079858s**

```text
python3 scripts/run-bounded.py --timeout 30 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo fmt --check
```
No stdout/stderr was emitted.

**82. [.temp/brief-559/fourth/final-ownership-discovery.txt](../../../.temp/brief-559/fourth/final-ownership-discovery.txt) — bound 80.0s; exit 0; elapsed 19.64340223700856s**

```text
python3 scripts/run-bounded.py --timeout 80 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --lib tests::ownership_graph::discovery_report_without_inventory_input -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.33s
test tests::ownership_graph::discovery_report_without_inventory_input ... UnknownHostPayload count: 63
UnknownConstruct count: 7562
Self-reaching type count: 60
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 5.23s
```
**83. [.temp/brief-559/fourth/final-regenerate-fingerprints.txt](../../../.temp/brief-559/fourth/final-regenerate-fingerprints.txt) — bound 24.0s; exit 0; elapsed 0.21511846501380205s**

```text
python3 scripts/run-bounded.py --timeout 24 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --lib tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --exact --ignored --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 0.12s
```
**84. [.temp/brief-559/fourth/final-cert-default.txt](../../../.temp/brief-559/fourth/final-cert-default.txt) — bound 120.0s; exit 0; elapsed 17.030421290983213s**

```text
python3 scripts/run-bounded.py --timeout 120 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 16.88s
```
**85. [.temp/brief-559/fourth/final-cert-nodefault.txt](../../../.temp/brief-559/fourth/final-cert-nodefault.txt) — bound 130.0s; exit 0; elapsed 31.03739890299039s**

```text
python3 scripts/run-bounded.py --timeout 130 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-559/tmp cargo test -p rum-stdlib --no-default-features --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture
```
```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.93s
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 17.00s
```
### 3. Allocation, instruction and timing evidence

Final executable paths are `.temp/brief-559/base/rum-cli` and `.temp/brief-559/candidate/rum-cli` (and the corresponding `rum-cli-alloc` files). Their separate binaries were saved before the rejected experiments. **Do not use candidate/target/release/rum-cli as the final measured executable:** that scratch target last built the rejected cold variant. Both final saved binaries and both original bases remain unchanged, and their hashes differ as follows. Saved base source lacks the five new frame types and Scan remaining-parts state.

```text
rum-cli
base sha256:f40c5b1f206d9a0f5771ccd71990227b13aa2d470a91eb6a49ec7596cfbfab5c
candidate sha256:6656e7f00b50eab546079550d261a3669f7e17a749a87722c46b2fea1b2a66bb
```
```text
rum-cli-alloc
base sha256:046be5b3e9b64e9a0ecb2fac12d5f5e829c18433b7f846097c73de19138fc716
candidate sha256:68ece5f5e8744bdd8866becb7973d21486ce758eed3b8530d7f8dbcc6a2e8910
```
All instruction runs pin CPU 2, use only cpu_core/instructions/u, and close stdin with /dev/null. N=20,000 and 2N=40,000 for all nine labels. Each individual label/N/build has two instruction samples, never more; per-iteration cost is (count(2N) - count(N)) / N. Counters report 100% event running time. Alloc-count builds and RUM_PROFILE_ALLOC=1 ran first; profiling was disabled for default-binary instructions and all timings.

**Final allocation rows, verbatim:** counts and peaks are identical for every label at both N and 2N. bytes_total is five bytes greater at both sizes, consistent with the five-character-longer candidate path; its per-iteration difference is zero.

```text
label	N	base_N	base_2N	candidate_N	candidate_2N	base_per_iteration	candidate_per_iteration	delta_N	delta_2N	base_bytes_N	candidate_bytes_N	base_peak_N	candidate_peak_N
scan-no-input	20000	661661	1321661	661661	1321661	33.0	33.0	0	0	51204971	51204976	942414	942414
optionvalue-three-no-input	20000	181794	361794	181794	361794	9.0	9.0	0	0	30460125	30460130	9134068	9134068
replacerepeated-no-input	20000	1221717	2441717	1221717	2441717	61.0	61.0	0	0	154311068	154311073	948049	948049
dot-three-no-input	20000	11921740	23841740	11921740	23841740	596.0	596.0	0	0	477979704	477979709	8678236	8678236
differences-no-input	20000	261670	521670	261670	521670	13.0	13.0	0	0	21969680	21969685	9591399	9591399
do-count-null	20000	1588	1588	1588	1588	0.0	0.0	0	0	478931	478936	299668	299668
do-iterator-null	20000	1612	1612	1612	1612	0.0	0.0	0	0	1119811	1119816	940146	940146
table-iterator-null	20000	1614	1614	1614	1614	0.0	0.0	0	0	1119731	1119736	940025	940025
replaceall-simple	20000	101746	201746	101746	201746	5.0	5.0	0	0	27530285	27530290	943388	943388
```
**Final instruction rows, both samples, verbatim:**

```text
label	sample	N	base_N	base_2N	candidate_N	candidate_2N	base_per_iteration	candidate_per_iteration	delta_pct
scan-no-input	1	20000	605154680	1206811073	603869550	1204151083	30082.8197	30014.0766	-0.228512
optionvalue-three-no-input	1	20000	201078978	396376501	201869365	397916401	9764.8762	9802.3518	0.383780
replacerepeated-no-input	1	20000	808716955	1613831005	811875409	1620150993	40255.7025	40413.7792	0.392682
dot-three-no-input	1	20000	5137813246	10268851555	5136460542	10266123276	256551.9155	256483.1367	-0.026809
differences-no-input	1	20000	203807811	401931290	207465769	409265047	9906.1740	10089.9639	1.855307
do-count-null	1	20000	13473308	23433172	13454862	23392119	497.9932	496.8628	-0.226981
do-iterator-null	1	20000	26288479	49047598	26268631	49010767	1137.9560	1137.1068	-0.074621
table-iterator-null	1	20000	10511430	17492511	10511852	17492098	349.0541	349.0123	-0.011961
replaceall-simple	1	20000	232676689	461743818	231679427	459745539	11453.3564	11403.3056	-0.436997
scan-no-input	2	20000	605167545	1206753594	603853232	1204181839	30079.3024	30016.4303	-0.209021
optionvalue-three-no-input	2	20000	201104970	396401026	201851896	397902561	9764.8028	9802.5333	0.386392
replacerepeated-no-input	2	20000	808714055	1613830552	812095032	1620592033	40255.8248	40424.8501	0.419878
dot-three-no-input	2	20000	5137821001	10268871142	5136457918	10266147363	256552.5070	256484.4722	-0.026519
differences-no-input	2	20000	203807223	401942644	207463045	409290278	9906.7710	10091.3617	1.863277
do-count-null	2	20000	13472746	23435838	13453953	23391918	498.1546	496.8983	-0.252201
do-iterator-null	2	20000	26289345	49049527	26266363	49010819	1138.0091	1137.2228	-0.069094
table-iterator-null	2	20000	10510267	17494168	10514304	17492184	349.1950	348.8940	-0.086213
replaceall-simple	2	20000	232679115	461745288	231680414	459744874	11453.3087	11403.2230	-0.437303
```
The three repeated increases are OptionValue **+0.383780% / +0.386392%**, ReplaceRepeated **+0.392682% / +0.419878%**, and Differences **+1.855307% / +1.863277%**. These are reported defects under the brief’s fix-or-report rule; no zero-cost or accepted performance claim is made. Other required instruction labels improve in both samples.

**Rejected experiments:** four inline(always) attributes on newly shared helpers removed OptionValue’s increase and reduced ReplaceRepeated’s, but worsened Differences to about +2.19%. A second variant placed repeated-pass and Differences error handling in cold helpers and computed the Differences resume cursor only on error; it reduced Differences to about +1.10% but introduced a repeatable Dot increase of +0.114% and increased OptionValue again. Both pass the 21 focused semantic tests and keep allocation counts/peaks equal, but neither satisfies the neighbor rule. All experiment source edits were reverted. Distinct binaries, source snapshots and both samples remain saved; no third sample was taken for any existing binary/N/label. These observations do not establish the final regressions’ exact machine-code cause.

Experiment candidate-inline, instruction rows, verbatim:

```text
label	sample	N	base_N	base_2N	candidate_N	candidate_2N	base_per_iteration	candidate_per_iteration	delta_pct
scan-no-input	1	20000	605154680	1206811073	603852498	1204153855	30082.8197	30015.0678	-0.225218
optionvalue-three-no-input	1	20000	201078978	396376501	201094986	396392391	9764.8762	9764.8702	-0.000060
replacerepeated-no-input	1	20000	808716955	1613831005	810417378	1616790519	40255.7025	40318.6571	0.156387
dot-three-no-input	1	20000	5137813246	10268851555	5136456655	10266141727	256551.9155	256484.2536	-0.026374
differences-no-input	1	20000	203807811	401931290	208144911	410616665	9906.1740	10123.5877	2.194730
do-count-null	1	20000	13473308	23433172	13453466	23394765	497.9932	497.0650	-0.186398
do-iterator-null	1	20000	26288479	49047598	26269765	49011736	1137.9560	1137.0985	-0.075346
table-iterator-null	1	20000	10511430	17492511	10514337	17492278	349.0541	348.8970	-0.044979
replaceall-simple	1	20000	232676689	461743818	232121607	460625315	11453.3564	11425.1854	-0.245963
scan-no-input	2	20000	605167545	1206753594	603868717	1204211576	30079.3024	30017.1430	-0.206652
optionvalue-three-no-input	2	20000	201104970	396401026	201093001	396383687	9764.8028	9764.5343	-0.002750
replacerepeated-no-input	2	20000	808714055	1613830552	810194937	1616790656	40255.8248	40329.7859	0.183728
dot-three-no-input	2	20000	5137821001	10268871142	5136458658	10266131626	256552.5070	256483.6484	-0.026840
differences-no-input	2	20000	203807223	401942644	208148191	410617798	9906.7710	10123.4803	2.187487
do-count-null	2	20000	13472746	23435838	13452935	23392161	498.1546	496.9613	-0.239544
do-iterator-null	2	20000	26289345	49049527	26268235	49011991	1138.0091	1137.1878	-0.072170
table-iterator-null	2	20000	10510267	17494168	10513219	17494727	349.1950	349.0754	-0.034265
replaceall-simple	2	20000	232679115	461745288	231678195	459746648	11453.3087	11403.4227	-0.435560
```
Experiment candidate-cold, instruction rows, verbatim:

```text
label	sample	N	base_N	base_2N	candidate_N	candidate_2N	base_per_iteration	candidate_per_iteration	delta_pct
scan-no-input	1	20000	605154680	1206811073	603855267	1204153512	30082.8197	30014.9123	-0.225735
optionvalue-three-no-input	1	20000	201078978	396376501	201264980	396714172	9764.8762	9772.4596	0.077660
replacerepeated-no-input	1	20000	808716955	1613831005	810175303	1617190785	40255.7025	40350.7741	0.236169
dot-three-no-input	1	20000	5137813246	10268851555	5143675764	10280579560	256551.9155	256845.1898	0.114314
differences-no-input	1	20000	203807811	401931290	205967246	406257996	9906.1740	10014.5375	1.093899
do-count-null	1	20000	13473308	23433172	13453519	23396306	497.9932	497.1393	-0.171458
do-iterator-null	1	20000	26288479	49047598	26269978	49010224	1137.9560	1137.0123	-0.082925
table-iterator-null	1	20000	10511430	17492511	10512664	17496439	349.0541	349.1888	0.038590
replaceall-simple	1	20000	232676689	461743818	231678603	459745484	11453.3564	11403.3440	-0.436662
scan-no-input	2	20000	605167545	1206753594	603856495	1204212757	30079.3024	30017.8131	-0.204424
optionvalue-three-no-input	2	20000	201104970	396401026	201266629	396704550	9764.8028	9771.8960	0.072641
replacerepeated-no-input	2	20000	808714055	1613830552	810175232	1616749766	40255.8248	40328.7267	0.181096
dot-three-no-input	2	20000	5137821001	10268871142	5143679185	10280585913	256552.5070	256845.3364	0.114140
differences-no-input	2	20000	203807223	401942644	205972475	406292373	9906.7710	10015.9949	1.102517
do-count-null	2	20000	13472746	23435838	13454351	23394259	498.1546	496.9954	-0.232699
do-iterator-null	2	20000	26289345	49049527	26268574	49008181	1138.0091	1136.9804	-0.090399
table-iterator-null	2	20000	10510267	17494168	10511851	17493952	349.1950	349.1051	-0.025774
replaceall-simple	2	20000	232679115	461745288	231680075	460626676	11453.3087	11447.3301	-0.052200
```
**Timing:** five one-second /proc CPU samples before instruction/timing phases found no competing process at or above 20% of one CPU; CPU 2 was allowed. No cargo/test workload overlapped measurements. Timings use in-kernel AbsoluteTiming (monotonic Instant), never civil-clock differences. The stock suite ran twice per build in A/B/B/A order, seven in-kernel repetitions, with best-of-two per-label comparison. Scratch labels ran once per label/build with three in-kernel repetitions. These timing samples do not override the repeated instruction defects.

Scratch timing rows, verbatim:

```text
label	N	base_seconds	candidate_seconds	delta_pct
scan-no-input	20000	0.038573608	0.038511552	-0.1609
optionvalue-three-no-input	20000	0.012404508	0.012371574	-0.2655
replacerepeated-no-input	20000	0.053433540	0.054453530	1.9089
dot-three-no-input	20000	0.322943249	0.322744168	-0.0616
differences-no-input	20000	0.012109251	0.012795431	5.6666
do-count-null	20000	0.000467448	0.000477798	2.2142
do-iterator-null	20000	0.001025451	0.001052938	2.6805
table-iterator-null	20000	0.000372701	0.000366921	-1.5508
replaceall-simple	20000	0.014124792	0.014415036	2.0549
```
Full stock-suite comparison, all 120 labels, verbatim (56 positive deltas are disclosed, with no post hoc pruning or extra samples):

```text
label	iterations	base_seconds	candidate_seconds	delta_pct
association-part-key-read-width4000	40	0.001252220	0.001165950	-6.8894
association-part-key-read-width32000	40	0.009660700	0.008916570	-7.7027
association-part-positional-read-width4000	40	0.000030871	0.000030833	-0.1231
association-part-positional-read-width32000	40	0.000030758	0.000031151	1.2777
loop-empty	5000	0.000258069	0.000263530	2.1161
do-count-null	10000	0.000235058	0.000237345	0.9730
do-iterator-null	1000	0.000057032	0.000056489	-0.9521
table-count-null	5000	0.000088761	0.000084042	-5.3165
table-iterator-null	1000	0.000020134	0.000019456	-3.3674
table-iterator-symbol	1000	0.000006629	0.000006321	-4.6463
literal-integer	2000	0.000090243	0.000088685	-1.7264
literal-string	2000	0.000098128	0.000098169	0.0418
symbol-ownvalue-read	2000	0.000216748	0.000223209	2.9809
symbol-ownvalue-guarded-read	1000	0.000184728	0.000184854	0.0682
set-ownvalue	1000	0.000265380	0.000267841	0.9273
list-literal-small	1000	0.000059606	0.000060390	1.3153
head-list	2000	0.000925176	0.000927091	0.2070
length-list	2000	0.000929072	0.000927152	-0.2067
part-list	2000	0.001092810	0.001096920	0.3761
sameq-integer	2000	0.000630204	0.000640224	1.5900
sameq-list	1000	0.000622045	0.000621144	-0.1448
if-true	2000	0.000728605	0.000740807	1.6747
module-local	500	0.000733384	0.000734816	0.1953
module-chain-escape-small	96	0.000046979	0.000049233	4.7979
module-chain-escape-large	192	0.000093527	0.000093039	-0.5218
block-local	500	0.000470808	0.000461570	-1.9622
block-symbol-only-repeat	1000	0.000305406	0.000306335	0.3042
block-init-null-repeat	1000	0.000345815	0.000348362	0.7365
block-init-read-repeat	1000	0.000399940	0.000404961	1.2554
with-local	500	0.000478329	0.000480963	0.5507
with-deep-body-small	500	0.001742750	0.001727220	-0.8911
table-small	200	0.000187228	0.000188214	0.5266
table-iterator-plus	1000	0.000030764	0.000030906	0.4616
table-iterator-pair-list	1000	0.000073373	0.000072863	-0.6951
do-small	200	0.000888091	0.000897911	1.1057
d-nested-sin-small	200	0.005262030	0.005315660	1.0192
downvalue-exact-hit	2000	0.000791562	0.000795145	0.4526
downvalue-exact-miss	2000	0.000565580	0.000564660	-0.1627
downvalue-pattern-hit	1000	0.000951161	0.000961127	1.0478
downvalue-pattern-miss	1000	0.000292919	0.000289496	-1.1686
downvalue-exact-miss-repeat	5000	0.001287150	0.001280340	-0.5291
downvalue-pattern-miss-repeat	5000	0.001319550	0.001307240	-0.9329
downvalue-guarded-hit	1000	0.001898930	0.001917470	0.9763
downvalue-guarded-decline-fallback	1000	0.001423020	0.001428080	0.3556
upvalue-guarded-hit	1000	0.002349390	0.002346070	-0.1413
subvalue-unguarded-hit	1000	0.001392920	0.001408930	1.1494
subvalue-unguarded-miss	1000	0.000773352	0.000792480	2.4734
subvalue-guarded-hit	1000	0.002463200	0.002488320	1.0198
default-unguarded-hit	1000	0.001292480	0.001288480	-0.3095
default-unguarded-miss	1000	0.001091360	0.001072400	-1.7373
default-guarded-hit	1000	0.001637220	0.001653710	1.0072
matchq-integer-success	1000	0.000312278	0.000308934	-1.0708
matchq-integer-fail	1000	0.000317136	0.000314884	-0.7101
matcher-single-tiny	1000	0.000669555	0.000692048	3.3594
matcher-single-wide-small	800	0.000038298	0.000038104	-0.5066
matcher-single-wide-large	1600	0.000075393	0.000075489	0.1273
replaceall-simple	500	0.000352505	0.000360537	2.2785
rule-rhs-deep-small	500	0.002987200	0.002974480	-0.4258
rewrite-guarded-hit	500	0.001772170	0.001775840	0.2071
rewrite-guarded-decline-fallback	500	0.002526500	0.002522240	-0.1686
cases-wide-no-match	1600	0.000163142	0.000159499	-2.2330
count-wide-no-match	1600	0.000154231	0.000149077	-3.3417
position-wide-no-match	1600	0.000184306	0.000183923	-0.2078
deletecases-wide-no-match	1600	0.000171076	0.000166082	-2.9192
deletecases-wide-association-no-match	1600	0.000431029	0.000423967	-1.6384
deletecases-leaf-delete-small	500	0.002179180	0.002175640	-0.1624
replace-wide-no-match	1600	0.000084191	0.000084562	0.4407
replace-leaf-rebuild-small	500	0.002690370	0.002639110	-1.9053
take-multispec-small	500	0.001712950	0.001682790	-1.7607
replacepart-deep-path-small	500	0.001336160	0.001303820	-2.4204
replacepart-wide-rule-list	1600	0.000291415	0.000288638	-0.9529
replacepart-guarded-hit	500	0.000538234	0.000534935	-0.6129
replacepart-guarded-decline-fallback	500	0.000599282	0.000598013	-0.2118
string-repeat-small	500	0.000188897	0.000187112	-0.9450
character-range-small	200	0.000130804	0.000132515	1.3081
stringtrim-literal-list-4000	100	0.000442640	0.000425849	-3.7934
stringtrim-literal-list-32000	100	0.003210790	0.003164930	-1.4283
stringtrim-single-literal-4000	100	0.000416595	0.000406886	-2.3306
stringtrim-single-literal-32000	100	0.003217690	0.003149290	-2.1257
stringtrim-whitespace-4000	100	0.000416261	0.000396250	-4.8073
stringtrim-whitespace-32000	100	0.003220100	0.003135370	-2.6313
stringreplace-single-literal-unguarded	200	0.000341471	0.000353162	3.4237
stringreplace-pattern-unguarded	500	0.001172460	0.001170260	-0.1876
stringreplace-guarded-hit	500	0.000689451	0.000687604	-0.2679
stringreplace-guarded-decline-fallback	500	0.000829712	0.000820933	-1.0581
appendto-list-incremental-small	250	0.000404116	0.000404202	0.0213
appendto-list-incremental-large	500	0.001350160	0.001338140	-0.8903
appendto-part-outer1000-small	100	0.001763970	0.001766510	0.1440
appendto-part-outer1000-large	200	0.003568340	0.003566130	-0.0619
association-set-key-incremental-small	100	0.001490560	0.001516510	1.7410
association-set-key-incremental-large	200	0.005546170	0.005605830	1.0757
association-small-literal-repeat	500	0.000718193	0.000719127	0.1300
merge-nested-small	200	0.001164420	0.001153250	-0.9593
set-part-fixed100-len100	100	0.000130715	0.000130679	-0.0275
set-part-fixed100-len800	100	0.000478020	0.000478329	0.0646
set-part-fill-small	200	0.000373343	0.000375094	0.4690
set-part-fill-large	400	0.001144120	0.001144680	0.0489
upvalue-candidates-tiny	1000	0.001945530	0.001928140	-0.8938
upvalue-candidates-wide-small	1600	0.000155090	0.000152234	-1.8415
upvalue-candidates-wide-large	3200	0.000310400	0.000299609	-3.4765
sort-callback-small	100	0.000165900	0.000165996	0.0579
sort-callback-large	200	0.000379837	0.000382013	0.5729
sortby-callback-small	100	0.000196829	0.000197172	0.1743
sortby-callback-large	200	0.000443333	0.000442498	-0.1883
reversesort-callback-small	100	0.000157219	0.000158318	0.6990
reversesort-callback-large	200	0.000354705	0.000353518	-0.3346
keysort-callback-small	50	0.000083202	0.000083715	0.6166
keysort-callback-large	100	0.000183569	0.000184837	0.6907
keysortby-callback-small	25	0.000045043	0.000045212	0.3752
keysortby-callback-large	50	0.000096945	0.000098022	1.1109
sort-default-wide	1600	0.000016429	0.000016143	-1.7408
sort-symbol-default-width4000	4000	0.000287480	0.000272994	-5.0390
sort-symbol-default-width16000	16000	0.001186700	0.001168590	-1.5261
sortby-default-wide	1600	0.000505176	0.000494431	-2.1270
reversesort-default-wide	1600	0.000016388	0.000016735	2.1174
keysort-default-wide	1600	0.000371094	0.000368803	-0.6174
keysortby-default-wide	1600	0.000874862	0.000862603	-1.4012
valueq-undefined-symbol-definitions-2000	20	0.000007861	0.000007929	0.8650
valueq-undefined-symbol-definitions-8000	20	0.000007837	0.000007917	1.0208
module-retained-gensym-gc-batch	40	0.000061000	0.000060707	-0.4803
```
Required stock-neighbor timing rows, copied verbatim from each timings.tsv:

[.temp/brief-559/fourth/perf/timing/base/1/timings.tsv](../../../.temp/brief-559/fourth/perf/timing/base/1/timings.tsv):

```text
label	iterations	rum_seconds
do-count-null	10000	0.000235058
do-iterator-null	1000	5.7086e-05
table-iterator-null	1000	2.0508e-05
replaceall-simple	500	0.000353179
```
[.temp/brief-559/fourth/perf/timing/base/2/timings.tsv](../../../.temp/brief-559/fourth/perf/timing/base/2/timings.tsv):

```text
label	iterations	rum_seconds
do-count-null	10000	0.000241001
do-iterator-null	1000	5.7032e-05
table-iterator-null	1000	2.0134e-05
replaceall-simple	500	0.000352505
```
[.temp/brief-559/fourth/perf/timing/candidate/1/timings.tsv](../../../.temp/brief-559/fourth/perf/timing/candidate/1/timings.tsv):

```text
label	iterations	rum_seconds
do-count-null	10000	0.000240842
do-iterator-null	1000	5.6489e-05
table-iterator-null	1000	1.9662e-05
replaceall-simple	500	0.000360537
```
[.temp/brief-559/fourth/perf/timing/candidate/2/timings.tsv](../../../.temp/brief-559/fourth/perf/timing/candidate/2/timings.tsv):

```text
label	iterations	rum_seconds
do-count-null	10000	0.000237345
do-iterator-null	1000	5.7403e-05
table-iterator-null	1000	1.9456e-05
replaceall-simple	500	0.000365893
```
The required stock neighbors include do-count-null +0.9730% and replaceall-simple +2.2785% in this best-of-two timing panel. All other positive rows are retained above. No timing non-regression acceptance is claimed.

Final sample ledger: 72 allocation executions, 144 instruction executions, 18 scratch-timing executions and four stock-suite executions = **238**, including both rejected binary identities. The original 130-run cap was explicitly expanded by 54 for each correction attempt before running it. Each instruction (label, N, binary identity) remains capped at two; no immutable base sample was rerun. [.temp/brief-559/fourth/performance-sample-proof.json](../../../.temp/brief-559/fourth/performance-sample-proof.json) verifies counts.

### 4. Counters, disclosures and remaining work

- Six of six scoped sites repaired; eleven of eleven equality witnesses pass. Five later-error tests and five cancellation tests pass, covering ten suspended positions and reusable sessions.
- Frozen brief-548 lost count: **86 → 80**, supported by the scoped tests. The historical 195-probe census was not rerun.
- All mandatory correctness commands have successful final-source evidence. The final focused, fmt, discovery, 68-fingerprint and both certificate confirmations pass after restoring source. Full workspace and full no-default retention passed before the experiments. The restored final source is byte-identical to those passing versions; hashes and the replay confirmations establish that identity.
- The candidate remains **not accepted** because three primary instruction regressions and positive timing rows remain. No regressing experiment is retained. The explicit fix-or-report rule is used here; there is no claim that the progress-based stop-loss fired, since each experiment improved some target costs. No further samples or speculative tuning were run after the bounded experiments.
- No additional lost-continuation site or consumer was repaired or added. The only outside-fence source change is the director’s explicitly authorized CLI helper invocation; assertions are unchanged. No new dependency, lease, unsafe, custom-abandonment change, helper extraction or admission/status/disposition/exclusion change is retained.
- Rejected experimental source was not promoted into ownership or retention inventories and was not claimed to pass broad gates. Its binaries and logs are preserved solely as rejected diagnostic evidence.

The next director action is a performance diagnosis/ruling for the three reproducible instruction increases (and the disclosed timing rows), using the preserved base and final candidate. The implementation, all current correctness evidence and the complete performance findings are concrete and reviewable; no further authorization is requested or inferred here. The checkpoint is [.temp/brief-559/PROGRESS.md](../../../.temp/brief-559/PROGRESS.md).

