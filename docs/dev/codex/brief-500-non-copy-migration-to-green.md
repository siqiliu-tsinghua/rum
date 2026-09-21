# brief-500 — finish the non-`Copy` migration to a green workspace

## Goal

Take the brief-499 stage-1 tree from "rum-runtime compiles" to **the whole
workspace green**, with `Symbol` no longer `Copy` and **nothing else changed**,
and deliver the convergence ledger that brief-499 could only start. This is a
mechanical ownership migration: no representation change, no behavior change,
no measurement. The representation swap and the cost measurement are
brief-501's and must not be started here.

## Where the tree is

You are on branch `spike/brief-499-symbol-non-copy`, at commit `ca0bba4a`,
which is a checkpoint that **does not compile**. `main` is clean and green at
`3024c33c` and must stay that way — it is the base arm of a later A/B. Do not
switch, merge, rebase or commit anything.

Already done, by brief-499 and the director:

- `Copy` removed from `Symbol`'s derive (`crates/rum-core/src/expr.rs:35`).
- `rum-core` (lib), `rum-syntax` (lib) and `rum-runtime` (lib and lib-test)
  compile.
- 74 Rust files carry compiler-forced ownership repairs.

Known still broken, from a director `cargo check --workspace --all-targets`:

- `rum-stdlib` (lib): 2 errors — one E0507 "cannot move out of an `Arc`", one
  E0382 "borrow of moved value: `target`".
- `rum-core` test targets: a run of E0382 "use of moved value" in what look
  like symbol-text fixtures (`same_a`, `global_z`, `name_backtick`,
  `unicode_latin`, …). These were invisible until `rum-runtime` compiled,
  because `rum-core` and `rum-syntax` both declare `rum-runtime` as a
  **dev-dependency** — the dependency order inverts for test targets. That is
  brief-499's finding; keep it in mind rather than rediscovering it.
- `rum-host`, `rum-terminal`, `rum-repl`, `rum-cli`, `rum-jupyter`: never
  reached, never measured. `rum-derive` is also a workspace member.
- `cargo fmt --check` fails on brief-499's own edits. Fixing that is in scope.

## The stop-loss, corrected — this is the part brief-499 got wrong

brief-499 said "if the same crate fails to repair twice for the same reason,
STOP". That rule was mis-keyed and it fired on a repair that was **converging
monotonically**: 304 distinct diagnostics, then 80, then 5. A non-`Copy`
migration produces only one reason by construction — every error is an
ownership error — so "the same reason twice" was guaranteed. The worker's stop
was correct; the rule was not.

**Re-keyed on progress, not on reason.** After each repair pass, record the
distinct-diagnostic count for the crate you are on. Continue while it falls.
**Stop and report** if either of these happens:

- a pass does not reduce the distinct count for that crate, or
- a pass produces an error class that is **not** a non-`Copy` ownership error
  (E0204, E0277 `Copy` bound, E0382, E0507, E0508, E0599 on `.copied()`),
  which would mean the migration has started changing semantics.

The count sequence per crate is a deliverable, not bookkeeping. Report it even
where it converged in one pass.

## Scope

**Files you MAY modify** — a disclosure duty, not a list:

- **Any `.rs` file under `crates/` that the compiler forces you to change**,
  plus formatting-only fixes from `cargo fmt` on files this migration already
  touched. Every file must appear in your report, tagged with the error kind
  that forced it.
- **A change that is not compiler-forced or fmt-forced is a stop.** Write it
  in the report instead of making it.

**MUST NOT touch**: `crates/rum-core/src/expr.rs`'s representation — the
`Symbol`/`SymbolText` fields, the interner, `PartialEq`/`Hash`/`Ord`,
`interned_id`, serde — beyond what a `Copy` removal mechanically forces;
`scripts/**`; `docs/**` except appending your report to this file;
`AGENTS.md`; `TODO.md`; `PROGRESS.md`; `Cargo.toml`; `Cargo.lock`; `.git`.

## How to repair, and the one repair that is a stop

1. **Prefer a borrow.** If the error can be resolved by taking `&Symbol`,
   iterating with `.iter()` instead of `.copied()`, or matching by reference,
   do that.
2. **A `Symbol` clone is acceptable** where a borrow does not fit. On today's
   representation it is a 24-byte `memcpy` and costs nothing; what it costs
   after brief-501 swaps in `Arc<SymbolText>` is exactly what brief-501
   measures, so an honest clone here is not a defect — a hidden one is.
3. **Cloning an `Expr`, `SharedExpr`, `SharedArgs` or `Vec<Expr>` to resolve a
   `Symbol` move is a STOP.** That is a hot-path regression wearing a compile
   fix's clothes. Report the site; do not do it.
4. **Do not add a clone to a path that runs when profiling is OFF if a borrow
   would do.** The diagnostics hard line (`AGENTS.md`) is unchanged by this
   migration.
5. **E0204 is expected and is itself evidence.** When a containing type must
   drop its own `derive(Copy)`, do it — and record the type. The list of types
   that lose `Copy` is the transitive closure `Symbol`'s `Copy`-ness was
   holding up, and it is a required deliverable.
6. **No semantic change.** Not an ordering, not a message, not a public
   signature beyond the `Copy` bound itself. If a repair seems to require one,
   STOP and report.

## Deliverable

- A green workspace on this branch, left **uncommitted**.
- **The convergence ledger**: per crate, in the order you reached it, the
  distinct-diagnostic count at first observation and after each repair pass.
  Include the crates brief-499 already measured (`rum-core` 1, `rum-syntax` 0,
  `rum-runtime` 222 lib / 304 distinct all-targets → 80 → 5) so the table is
  whole, marked as inherited rather than re-measured.
- **The `Copy`-loss closure**: every type that had to drop `derive(Copy)`,
  with `path:line`.
- The file inventory, each entry tagged with the error kind that forced it.
- Anything you refused under rule 3 or rule 6.

## Verification (MANDATORY — you run it, and paste it verbatim)

```
cargo fmt --check
cargo test --workspace
```

Both must be green. This migration is semantically neutral, so **a failing
test is a finding, not a thing to fix by changing the test** — report it with
the verbatim failure and stop. Bound anything long with
`scripts/run-bounded.py --timeout SECONDS -- <command>`, sized from a measured
run of the same shape; a full `cargo test --workspace` on this host is the
long one.

## Hard rules

- Do NOT `git commit`, branch, switch, merge, push, stash, `git checkout`, or
  touch git config or the index. Leave everything in the working tree; the
  director commits. (`git checkout` restores from the index, which on your own
  uncommitted output is a delete — this has cost this project real work twice.)
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`. Not to
  dodge a move, not anywhere.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no oracle harnesses, no third-party engine comparisons.
- No network. Touch nothing outside this repo. Commands: file edits in Scope,
  `cargo`, `scripts/run-bounded.py`, `python3` for parsing.
- No measurement in this brief. No `perf`, no microbench, no release build for
  timing. If you find yourself curious about the cost, that is brief-501.
- Paste real verbatim output. If a command did not reach green, say so — never
  fake or interpolate a result.

## Report format (your final message, and appended to this file)

1. **The convergence ledger** — the per-crate table, written before anything
   else, so a truncated run still delivers it.
2. **The `Copy`-loss closure** — the types that dropped `derive(Copy)`.
3. **What changed** — per file, line ranges, one-line why, tagged with the
   error kind that forced it.
4. **Verification** — both commands, verbatim, with test counts.
5. **Refusals** — sites where rule 3 or rule 6 stopped you; empty if none.
6. **Stop-loss / open questions** — where you stopped and why; empty if none.

## Execution report (2026-09-21)

### 1. The convergence ledger

| Crate / first reach | Inherited brief-499 counts | Brief-500 distinct diagnostics |
|---|---|---|
| `rum-core` | Lib: **1 → 0**; test targets previously blocked | **0** on first workspace check; **0** after the stdlib repair |
| `rum-syntax` | Lib: **0**; test targets previously blocked | **0** on first workspace check; **0** after the stdlib repair |
| `rum-runtime` | Lib: **222**; all targets: **304 → 80 → 5**; director repaired the final five before this run | **0** on first workspace check; **0** after the stdlib repair |
| `rum-stdlib` — first workspace check | Not reached by brief-499; brief-500 described 2 remaining library errors | **29 → 0**, one repair pass: E0382=28, E0507=1 initially; lib=2, lib-test=29, 31 emitted / 29 distinct |
| `rum-derive` — first workspace check | Not separately measured | **0**; no ownership repair |
| `rum-host` — check after repair | Not reached | **0**; no ownership repair |
| `rum-terminal` — check after repair | Not reached | **0**; no ownership repair |
| `rum-jupyter` — check after repair | Not reached | **0**; no ownership repair |
| `rum-repl` — check after repair | Not reached | **0**; no ownership repair |
| `rum-cli` — check after repair | Not reached | **0**; no ownership repair |

Inherited numbers are transcribed from brief-499, not re-measured. Both new observations used `cargo check --workspace --all-targets --message-format=json`, offline and bounded to 120 seconds. Diagnostics were deduplicated by package, error code, message, and primary source spans across targets. Independent targets in the first check were observed together; downstream order follows Cargo's check messages. The successful check exited 0 in 5.02s.

The named `same_a`, `global_z`, `name_backtick`, and Unicode fixtures belong to **rum-stdlib**, at `src/tests/structure_ops/ordering_sets.rs`, rather than rum-core as the brief suggested. No core or syntax test ownership repair was needed. Compilation converged; the mandatory workspace test later failed, so the workspace is **not green**.

### 2. The Copy-loss closure

**35 types total: Symbol plus 34 containing types.** All losses were inherited; this brief removed no additional Copy derives. Locations below are current type declarations (their derive is immediately above). Symbol is the initial removal; the other 34 are the inherited E0204 closure.

| Type | Current path:line |
|---|---|
| `Symbol` | `crates/rum-core/src/expr.rs:36` |
| `CallPreparationMetadata` | `crates/rum-runtime/src/runtime/call_preparation.rs:8` |
| `CallStabilityHeadShape` | `crates/rum-runtime/src/runtime/evaluation_cache/mod.rs:22` |
| `CallStabilitySourceHeadKey` | `crates/rum-runtime/src/runtime/evaluation_cache/mod.rs:30` |
| `InvalidationSiteKey` | `crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs:64` |
| `SymbolTimingFrame` | `crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs:8` |
| `PresenceRootShape` | `crates/rum-runtime/src/runtime/profiling/mod.rs:115` |
| `PresenceShapeKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:125` |
| `PresenceContextShapeKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:137` |
| `PresenceArgSite` | `crates/rum-runtime/src/runtime/profiling/mod.rs:144` |
| `PresenceArgSiteKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:150` |
| `PresenceBindingKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:167` |
| `PresenceBindingNameKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:173` |
| `PresenceSourceKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:179` |
| `PresenceSourceShapeKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:186` |
| `PresenceSourceTargetShapeKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:194` |
| `PresenceSourceFingerprintKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:209` |
| `PresenceSourceFingerprintGroupKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:220` |
| `PresenceReuseKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:242` |
| `PresenceReuseGroupKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:251` |
| `PresenceFingerprintKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:270` |
| `PresenceFingerprintGroupKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:278` |
| `PresenceExactFingerprintKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:297` |
| `ExpressionRebuildKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:305` |
| `ExpressionRebuildGroupKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:313` |
| `RuleRhsAlphaRenameKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:334` |
| `RuleRhsTimingKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:346` |
| `DefinitionRuleTimingKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:360` |
| `DefinitionDispatchKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:375` |
| `BuiltinTimingKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:396` |
| `RuntimePathTimingKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:412` |
| `BuiltinTimingFrame` | `crates/rum-runtime/src/runtime/profiling/mod.rs:453` |
| `RuntimePathTimingFrame` | `crates/rum-runtime/src/runtime/profiling/mod.rs:459` |
| `ExpressionProvenanceValue` | `crates/rum-runtime/src/runtime/profiling/mod.rs:511` |
| `PresenceProvenanceKey` | `crates/rum-runtime/src/runtime/profiling/mod.rs:531` |

### 3. What changed

**18 Rust files changed in this brief: 3 compiler-forced repairs and 15 formatting-only repairs**, plus this appended report. All remain uncommitted. The new ownership repairs add exactly two Symbol clones and no Expr, SharedExpr, SharedArgs, or Vec<Expr> clone. No public signature, representation, ordering, message, assertion, or expected result was changed by this brief.

Runtime paths below are relative to `crates/rum-runtime/src/runtime/`; stdlib paths are explicit. Ranges refer to the final working tree.

| File | Changed lines | Forced by | Why |
|---|---|---|---|
| `crates/rum-stdlib/src/associations/mutation/resume_frames.rs` | 24 | E0507 | Clone only the Symbol extracted from the Arc-owned continuation; the callee requires an owned Symbol. |
| `crates/rum-stdlib/src/associations/mutation.rs` | 570 | E0382 | Clone the Symbol placed in the held Part expression while retaining the target for validation and mutation. |
| `crates/rum-stdlib/src/tests/structure_ops/ordering_sets.rs` | 9–10, 26–27, 33–34, 40–41, 48–49, 56–57, 73–74, 76, 93–94 | E0382 | Borrow symbols in the private sorting helper, expected vectors, and nested fixture iteration; preserve every assertion and ordering case. |
| `definitions/definition_tags.rs` | 285–287 | FMT (inherited E0507) | Wrap the inherited continuation call. |
| `evaluation_cache/profile/source.rs` | 174–176 | FMT (inherited E0507) | Wrap the inherited symbol-shape match arm. |
| `evaluation_cache/stability.rs` | 107–112, 157–162, 202–207 | FMT (inherited E0382/E0507) | Wrap three inherited borrowed profiling calls. |
| `evaluation_cache/tests/association_stability_tail.rs` | 207–210 | FMT (inherited E0382) | Wrap the inherited symbol fixture construction. |
| `evaluation_cache/tests.rs` | 99–103, 112–116 | FMT (inherited E0382) | Wrap two inherited definition-entry assignments. |
| `kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs` | 149–150 | FMT (inherited E0507) | Wrap the inherited scope-restoration call. |
| `kernel_data/control_data/branching_loops/iteration/driver.rs` | 545–546 | FMT (inherited E0382/E0507) | Wrap the inherited syntax-information insertion. |
| `messages/message_catalog.rs` | 89–92 | FMT (inherited E0382) | Wrap the inherited symbol-count entry expression. |
| `profiling/presence.rs` | 370–372 | FMT (inherited E0507) | Wrap the inherited call-head shape match arm. |
| `scoping/lexical/block.rs` | 257–258, 472–473 | FMT (inherited E0382/E0507) | Wrap two inherited syntax-information insertions. |
| `tests/association_write_index.rs` | 23–27, 38–42 | FMT (inherited E0382) | Wrap two inherited definition-entry fixture assignments. |
| `tests/part_assignment.rs` | 75–78, 378–379 | FMT (inherited E0382) | Wrap the inherited Part fixture and dynamic-binding call. |
| `tests/scoping_gensym_gc/reachability.rs` | 450–454, 459–463 | FMT (inherited E0382) | Wrap two inherited candidate-count assertions. |
| `tests/sealed_snapshot.rs` | 18–19 | FMT (inherited E0507) | Wrap the inherited indexed Symbol clone. |
| `tests/sealed_visibility.rs` | 26–27 | FMT (inherited E0507) | Wrap the inherited indexed Symbol clone. |
| `docs/dev/codex/brief-500-non-copy-migration-to-green.md` | 157–EOF | Required report | Append this ledger, Copy-loss closure, inventory, and verbatim verification. |

Inherited repairs are documented in [brief-499's 74-file inventory](brief-499-symbol-interner-a1-spike.md#2-what-changed). The director's already-present final E0507 repairs are at `profiling/report_presence_shapes.rs:12,82,118` and `session/session_api.rs:392,404`; they were not modified here. The current 18-file inventory above describes this brief's delta, not those earlier committed edits.

Organization assessment: splitting is outside the compiler/fmt-only scope. The 23,063-character stdlib `associations/mutation.rs` retains a future boundary between held-target/selector preparation and association mutation. The 20,423-character iteration `driver.rs` retains the existing iteration-driving versus state capture/restore boundary. Other touched warning-range modules remain in their current domains (profiling, cache stability, messages, and Block scoping); larger test files remain grouped by their ordering, cache-proof, Part-assignment, and gensym-reachability scenarios. No split or unrelated cleanup was made.

### 4. Verification

`cargo fmt --check`: **exit 0**, no stdout or stderr. Actual command:

```text
TERM=dumb NO_COLOR=1 CARGO_TERM_COLOR=never CARGO_NET_OFFLINE=true scripts/run-bounded.py --timeout 120 -- cargo fmt --check
```

Verbatim output is empty:

```text
```

The earlier formatting check exited 1 and identified exactly the 15 inherited files listed above; `cargo fmt` repaired those files before this final successful check.

`cargo test --workspace`: **exit 101**. Across the **20 executed test groups: 2,265 passed, 1 failed, 57 ignored**. The remaining test groups and doctests were not reached. This is not a green workspace.

The 900-second bound is based on the prior same-shape workspace run in `target/brief-498/logs/workspace.log`: 24.83s compilation plus 72.15s across test summaries (96.98s recorded work, approximately 9.3x headroom). This is timeout sizing only; no benchmark, perf run, release build, or representation-cost measurement was performed.

Cargo's quiet setting reduces output volume without selecting or skipping tests. Temporary test files were routed to the existing repository-local `target/tmp`; the repository's native ZeroMQ sandbox-skip setting was enabled. The counts are harness results and do not certify loopback operations.

Actual command:

```text
TERM=dumb NO_COLOR=1 CARGO_TERM_COLOR=never CARGO_TERM_QUIET=true CARGO_NET_OFFLINE=true TMPDIR="$PWD/target/tmp" RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

Complete verbatim stdout/stderr:

```text
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


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 28 tests
............................
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s


running 7 tests
.......
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 5 tests
.....
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
...
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
...
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 89 tests
....................................................................................... 87/89
..
test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s


running 7 tests
.......
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 1 test
test host_payloads_declare_bounded_destruction_at_the_crate_edge has been running for over 60 seconds
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 67.52s


running 4 tests
....
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 7 tests
.......
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s


running 15 tests
...............
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s


running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 6 tests
ii....
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.21s


running 12 tests
............
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s


running 106 tests
....................................................................................... 87/106
...................
test result: ok. 106 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 45 tests
.............................................
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s


running 999 tests
.....................i................................................................. 87/999
....................................................................................... 174/999
....................................................................................... 261/999
....................................................................................... 348/999
....................................................................................... 435/999
.................................ii.................................................... 522/999
....................................................................................... 609/999
....................................................................................... 696/999
....................................................................................... 783/999
....................................................................................... 870/999
....................................................................................... 957/999
..........................................
test result: ok. 996 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 5.48s


running 984 tests
i....................................iiii.iiiii.i...................................... 87/984
....................................................................................... 174/984
...........................................................i.i.i.iiii.................. 261/984
.......................i.i............................................................. 348/984
....................................................................................... 435/984
..............................................i.i.iiiiii.i.ii....iiiiii.i.ii.iiii...... 522/984
..........................................................ii.i......................... 609/984
...........................................................................ii.......... 696/984
......................................................i...i.i.......................... 783/984
....................................................................................... 870/984
....................................................................................... 957/984
..................... 978/984
tests::ownership_graph::production_inventory_is_exact --- FAILED
.....
failures:

---- tests::ownership_graph::production_inventory_is_exact stdout ----
EXACT INVENTORY self-reaching types: 60 rows
EXACT INVENTORY UnknownHostPayload: 58 rows

thread 'tests::ownership_graph::production_inventory_is_exact' panicked at crates/rum-stdlib/src/tests/ownership_graph/mod.rs:73:37:
UnknownConstruct inventory mismatch:
missing inventory row: UnknownConstruct	crates/rum-core/src/expr.rs	35	1	unresolved attribute/expansion derive (Clone , Debug)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	7	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , FrameRootTrace)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	163	9	unexpanded macro crate :: resume_or_suspend! (ends at 166:10)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	201	32	closure captures: |symbol| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	204	26	unexpanded macro crate :: resume_or_suspend! (ends at 209:14)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	216	22	unexpanded macro crate :: resume_or_suspend! (ends at 226:10)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	280	34	closure captures: |total , arg| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	295	18	closure captures: |symbol| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	296	29	closure captures: || (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	300	13	unexpanded macro format! (ends at 300:64)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	360	8	unexpanded macro cfg! (ends at 360:63)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	374	22	closure captures: |value| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	387	5	unexpanded macro assert! (ends at 392:6)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	411	5	unexpanded macro panic! (ends at 415:6)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	418	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	47	34	closure captures: |symbol| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	48	49	closure captures: |spec| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	55	59	closure captures: |symbol| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	57	45	closure captures: |definition| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	59	42	closure captures: |definition| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/mod.rs	21	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/mod.rs	29	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	63	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	202	43	closure captures: |stats| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	241	38	closure captures: |stack| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	266	51	closure captures: |stack| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	283	43	closure captures: |stats| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	325	43	closure captures: |stats| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	348	43	closure captures: |stats| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	7	1	unresolved attribute/expansion derive (Clone , Debug)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	308	21	unexpanded macro unreachable! (ends at 308:86)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	442	13	unexpanded macro unreachable! (ends at 442:89)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	461	30	closure captures: || (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	464	23	closure captures: |value| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	465	21	closure captures: |value| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	473	27	closure captures: || (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	476	23	closure captures: |value| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	477	21	closure captures: |value| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs	555	21	closure captures: |key , _| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs	169	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs	178	40	closure captures: |payload| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs	187	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs	206	14	closure captures: || (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	130	17	unexpanded macro crate :: resume_or_suspend! (ends at 144:18)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	196	22	closure captures: |item| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	198	22	closure captures: |items| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	199	23	unexpanded macro vec! (ends at 201:14)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	244	68	closure captures: |symbol| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	263	18	closure captures: |key| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	299	45	unexpanded macro vec! (ends at 299:67)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	301	52	unexpanded macro vec! (ends at 301:74)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	303	45	unexpanded macro vec! (ends at 303:75)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	315	21	closure captures: |(key , _)| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	319	21	closure captures: |(key , _)| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	319	50	closure captures: |symbol| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	320	18	closure captures: |(key , def)| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	332	18	closure captures: |(key , source)| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	343	18	closure captures: |source| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	343	27	unexpanded macro vec! (ends at 343:85)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	351	22	closure captures: |key| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	356	22	closure captures: |key| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	408	38	closure captures: |symbol| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	449	18	closure captures: |def| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	449	24	unexpanded macro vec! (ends at 449:73)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	465	19	closure captures: || (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	465	22	unexpanded macro vec! (ends at 465:57)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	114	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	124	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	136	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	143	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	149	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	166	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	172	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	178	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	185	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	193	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	208	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	219	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	241	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	250	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	269	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	277	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	296	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	304	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	312	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	333	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	345	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	359	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	374	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	395	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	411	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	452	1	unresolved attribute/expansion derive (Clone , Debug)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	458	1	unresolved attribute/expansion derive (Clone , Debug)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	510	1	unresolved attribute/expansion derive (Clone , Debug)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	530	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , Hash)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/presence.rs	384	32	closure captures: |stack| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/presence.rs	392	35	closure captures: |stack| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/presence.rs	400	42	closure captures: |stack| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	267	25	closure captures: |key , _| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	313	25	unexpanded macro crate :: resume_or_suspend! (ends at 313:113)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	314	25	unexpanded macro crate :: resume_or_suspend! (ends at 314:94)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	392	17	unexpanded macro crate :: resume_or_suspend! (ends at 392:109)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	393	17	unexpanded macro crate :: resume_or_suspend! (ends at 393:85)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	428	26	closure captures: |definition| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	432	26	closure captures: |definition| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	487	25	closure captures: |key , _| (capture types/modes unresolved)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	507	17	unexpanded macro crate :: resume_or_suspend! (ends at 507:97)
missing inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	508	17	unexpanded macro crate :: resume_or_suspend! (ends at 508:79)
stale inventory row: UnknownConstruct	crates/rum-core/src/expr.rs	35	1	unresolved attribute/expansion derive (Clone , Copy , Debug)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/call_preparation.rs	7	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , FrameRootTrace)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	166	9	unexpanded macro crate :: resume_or_suspend! (ends at 169:10)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	204	32	closure captures: |symbol| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	207	26	unexpanded macro crate :: resume_or_suspend! (ends at 212:14)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	219	22	unexpanded macro crate :: resume_or_suspend! (ends at 229:10)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	283	34	closure captures: |total , arg| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	298	18	closure captures: |symbol| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	299	29	closure captures: || (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	303	13	unexpanded macro format! (ends at 303:64)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	363	8	unexpanded macro cfg! (ends at 363:63)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	377	22	closure captures: |value| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	390	5	unexpanded macro assert! (ends at 395:6)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	414	5	unexpanded macro panic! (ends at 418:6)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	421	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	48	23	closure captures: |symbol| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	49	49	closure captures: |spec| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	58	23	closure captures: |symbol| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	60	45	closure captures: |definition| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/eval_dispatch/mod.rs	62	42	closure captures: |definition| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/mod.rs	21	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/mod.rs	29	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/mod.rs	63	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	200	43	closure captures: |stats| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	239	38	closure captures: |stack| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	264	51	closure captures: |stack| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	281	43	closure captures: |stats| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	323	43	closure captures: |stats| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	346	43	closure captures: |stats| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/profile/source.rs	7	1	unresolved attribute/expansion derive (Clone , Copy , Debug)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	293	21	unexpanded macro unreachable! (ends at 293:86)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	427	13	unexpanded macro unreachable! (ends at 427:89)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	446	30	closure captures: || (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	449	23	closure captures: |value| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	450	21	closure captures: |value| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	458	27	closure captures: || (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	461	23	closure captures: |value| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/evaluation_cache/stability.rs	462	21	closure captures: |value| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs	554	21	closure captures: |key , _| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs	168	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs	177	40	closure captures: |payload| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs	186	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver/resume_frames.rs	205	14	closure captures: || (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	127	17	unexpanded macro crate :: resume_or_suspend! (ends at 141:18)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	193	22	closure captures: |item| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	195	22	closure captures: |items| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	196	23	unexpanded macro vec! (ends at 198:14)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	241	68	closure captures: |symbol| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	260	18	closure captures: |key| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	296	45	unexpanded macro vec! (ends at 296:67)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	298	52	unexpanded macro vec! (ends at 298:74)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	300	45	unexpanded macro vec! (ends at 300:75)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	312	21	closure captures: |(key , _)| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	316	21	closure captures: |(key , _)| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	316	50	closure captures: |symbol| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	317	18	closure captures: |(key , def)| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	329	18	closure captures: |(key , source)| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	340	18	closure captures: |source| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	340	27	unexpanded macro vec! (ends at 340:85)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	348	22	closure captures: |key| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	353	22	closure captures: |key| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	405	38	closure captures: |symbol| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	446	18	closure captures: |def| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	446	24	unexpanded macro vec! (ends at 446:73)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	462	19	closure captures: || (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/messages/message_catalog.rs	462	22	unexpanded macro vec! (ends at 462:57)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	114	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	124	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	136	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	143	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	149	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	166	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	172	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	178	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	185	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	193	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	208	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	219	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	241	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	250	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	269	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	277	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	296	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	304	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	312	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	333	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	345	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	359	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	374	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	395	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	411	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	452	1	unresolved attribute/expansion derive (Clone , Copy , Debug)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	458	1	unresolved attribute/expansion derive (Clone , Copy , Debug)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	510	1	unresolved attribute/expansion derive (Clone , Copy , Debug)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/mod.rs	530	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq , Hash)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/presence.rs	382	32	closure captures: |stack| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/presence.rs	390	35	closure captures: |stack| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/profiling/presence.rs	398	42	closure captures: |stack| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	266	25	closure captures: |key , _| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	312	25	unexpanded macro crate :: resume_or_suspend! (ends at 312:113)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	313	25	unexpanded macro crate :: resume_or_suspend! (ends at 313:94)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	391	17	unexpanded macro crate :: resume_or_suspend! (ends at 391:109)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	392	17	unexpanded macro crate :: resume_or_suspend! (ends at 392:85)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	427	26	closure captures: |definition| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	431	26	closure captures: |definition| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	485	25	closure captures: |key , _| (capture types/modes unresolved)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	505	17	unexpanded macro crate :: resume_or_suspend! (ends at 505:97)
stale inventory row: UnknownConstruct	crates/rum-runtime/src/runtime/scoping/lexical/block.rs	506	17	unexpanded macro crate :: resume_or_suspend! (ends at 506:79)


failures:
    tests::ownership_graph::production_inventory_is_exact

test result: FAILED. 931 passed; 1 failed; 52 ignored; 0 measured; 0 filtered out; finished in 20.23s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```

### 5. Refusals

None under repair rule 3 or rule 6. The out-of-scope TSV inventory refresh was not attempted; the mandatory test-failure stop is recorded below.

### 6. Stop-loss / open questions

Stopped at the mandatory test failure, `tests::ownership_graph::production_inventory_is_exact`, at `crates/rum-stdlib/src/tests/ownership_graph/mod.rs:73`. The exact UnknownConstruct inventory reports **108 missing rows and 108 stale rows**, including the deliberate Copy removals and line/column changes from ownership and formatting repairs. The self-reaching-types inventory (60 rows) and UnknownHostPayload inventory (58 rows) matched.

The relevant baselines are `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv`, `unknown_captures.tsv`, and `unknown_macros.tsv`. Refreshing these TSV fixtures is outside this brief's .rs/fmt/report scope, and the Verification section explicitly requires stopping on a failing test. No fixture, scanner, assertion, or expected result was changed to bypass the failure. No test was rerun or skipped after the failure.

No diagnostic-count stop-loss fired: the only ownership repair pass reduced stdlib's distinct count from 29 to 0, with no error outside the allowed ownership classes. The remaining decision is whether a subsequent brief should authorize reviewing and refreshing the exact source inventory, followed by another full workspace test.

Checkout observation: the requested branch was already clean at `7f15a2d06cbd32868ab8059a3779b017c701b84c`, rather than the brief's stated `ca0bba4a`. The branch and its HEAD remain unchanged; `main` remains `3024c33cea578d1d69fe87663231ecc7e50350ee`. No commit, branch switch, merge, rebase, stash, index/config write, network operation, new unsafe, or file edit outside Scope was performed.
