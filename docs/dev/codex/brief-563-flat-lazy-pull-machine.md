# brief-563 — G2 step A: a flat, depth-safe, resumable lazy pull machine

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

Replace the lazy pull tree with the flat machine brief-562 designed, and in the
same change make a suspended pull resume into its realization and its consumer.
Two defects close together:

1. **Lost continuations (group G2, eleven sites).** A lazy callback that asks for
   input propagates `InputRequired` with a plain `?` from stack-owned stepper
   state, so the resumed request completes with the callback's value instead of
   the realized list. Frozen Lost IDs P1898, P1899, P1901, P1902, P1904, P1906,
   P1907, P1913 (machine and driver) and P1483, P2353, P2355 (the `Normal`,
   `Take`, `First` consumers; `Part` inherits `First` through its symbolic seam).
   Lost continuations **80 -> 69**, G2 **14 -> 3**; the remaining three are
   `LazyTable`'s eager bounds, which are step B and stay unchanged here.
2. **A guest-reachable process abort.** The `Box`-chained `PullStepper` is built,
   pulled and dropped by native recursion proportional to the descriptor's depth;
   `x = LazyRange[1]; Do[x = LazyMap[Identity, x], {n}]; Take[x, 0]` aborts the
   process at n ~2 170 on the ruled 2 MiB stack (release; 944 bytes per
   constructor activation), and at n 300-600 in the debug profile.

After this brief no walk of the machine — construction, pull, clone, root
tracing, drop, abandonment — recurses natively in proportion to depth, and that
property is **enforced by tests that fail if recursion is reintroduced**, not
only demonstrated at one depth.

**This brief governs.** brief-562's draft A
(`.temp/brief-562/draft-A-pull-machine.md`) is the raw material; where this brief
differs from it — the cancellation checkpoint, the depth tests, the structural
depth checks, the timing suite, the bounds — follow this brief.

## Context pointers

- The design: brief-561 and brief-562 with their appended reports;
  `.temp/brief-562/design.md` (types, pseudocode, frame and root inventory, the
  apply-once table), `inventory-plan.md`, `ownership-existing.json`,
  `red-results.md`, `probe.rs`, `cases.json`, `depth-brackets.json`,
  `performance-gate.md`, `performance-panel.json`; `.temp/brief-561/probe.rs`,
  `cases.json`, `red-results.md`.
- Source: `crates/rum-stdlib/src/lazy.rs` (`:27` `PullContext`, `:57`
  `PullStepper`, `:105` `next`, `:566` `realize`, `:685`
  `stepper_from_descriptor`); consumers `associations/basic.rs:217`,
  `structure_ops/sequence/take_drop.rs:79`, `structure_ops/surface.rs:99`.
- Protocol, read-only: `crates/rum-runtime/src/runtime/suspension/core.rs`
  (`:81` immutable `resume(self: Arc<Self>)`, `:156` the driver clones the queued
  `Arc`, `:552`, `:764-829`), `suspension/primitive.rs:36` (the `Option<Expr>`
  continuation codec), `native_stack_guard.rs` (its `compile_fail` doctests
  forbid storing a `NativeStackDepth` in a frame).
- How a loop frame carries its state and what the ordinary path cost: briefs 559
  and 560; `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:60-105`.
- The permanent oracle harness shape: `crates/rum-stdlib/tests/loop_continuations.rs`,
  `guest_loading_resume.rs`.
- The recursion scanner: `crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs`
  (`recursive_edges_from_source`, `:196`); the ownership inventories:
  `crates/rum-stdlib/src/tests/ownership_graph/` (`inventory/self_reaching.tsv:41`).
- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`,
  `docs/guide/evaluation.md` §Lazy Sequences, `docs/dev/lazy-sequences-design.md`,
  `docs/dev/codex/TEMPLATE.md` (standing duty).

## Scope

**Objective:** the flat machine and every walk of it; capture on `InputRequired`
and resume into the realization and its consumer; the one new cancellation
checkpoint; the structural and empirical depth checks; the owning tests and the
two scoped documentation updates; the inventory and performance gates.

**Edit surface:** `crates/rum-stdlib/src/lazy.rs` and new children under
`crates/rum-stdlib/src/lazy/` as design.md §8 lays out (`state`, `descriptor`,
`source`, `machine`, `realization`, `continuation`); the lazy realization
branches of `associations/basic.rs`, `structure_ops/sequence/take_drop.rs` and
`structure_ops/surface.rs`, with their completion frames in new children beside
them; new `crates/rum-stdlib/tests/lazy_pull_continuations.rs` (and support
children if it outgrows the guideline limit); private unit tests under the new
lazy module; a new lazy depth-contract test module under
`crates/rum-stdlib/src/tests/`, and the visibility of
`recursive_edges_from_source` widened to that module (no other change to the
association inventory); the Lazy Sequences section of `docs/guide/evaluation.md`
and §§2 and 5 of `docs/dev/lazy-sequences-design.md`. New files stay within the
guideline limits; split only at a named responsibility boundary.

**Inventory exceptions, authorized by name (`inventory-plan.md`):**

- remove the `self_reaching.tsv` row `rum_stdlib::lazy::PullStepper`
  (`lazy.rs:57`, witness `…PullStepper.Drop.inner/std::boxed::Box[Own]/arg0 ->
  rum_stdlib::lazy::PullStepper`); **no lazy type may appear in
  `self_reaching.tsv` afterwards**;
- re-identify the `unknown_attributes.tsv` budget derive at `lazy.rs:12:1` from
  `(Clone , Copy)` to `(Clone , Copy , FrameRootTrace)`;
- relocate exactly these ten `UnknownConstruct` rows, keeping their status and
  construct: closures `:133:38`, `:152:30`, `:658:15` -> `lazy/source.rs`;
  closures `:725:36`, `:730:18` -> `lazy/descriptor.rs`; `vec!` `:163:74` ->
  `lazy/source.rs`; `vec!` `:171:67`, `:179:68`, `:216:21` -> `lazy/machine.rs`;
  `vec!` `:732:21` -> `lazy/descriptor.rs`;
- add the new rows inventory-plan.md names for step A (nine frame and state
  derives, up to three consumer capture macros), each with its actual file and
  line from discovery.

The other 24 lazy rows and every consumer row keep their identity and reason;
only positions may move.

**Standing maintenance duty — always in scope.** Every source change expires the
retention contract fingerprints. Recompute the 68 fingerprints in
`crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv`
with the existing replay; run `local_production_acceptance` in the default and
`--no-default-features` builds on the final source and copy the actual C01/C02
cells from `.temp/brief-546/certificates/` into `inventory/promoted.tsv`.
Reconcile every inventory under `crates/rum-stdlib/src/tests/ownership_graph/inventory/`
against what discovery prints — never hand-invent a row — and disclose every
changed row. Fingerprints, hashes and positions may move; new `UnknownConstruct`
rows in code this brief added or changed are expected and stay Unknown. The
named exceptions above are the only other departures. **STOP** on any other
existing row's identity, status, disposition or exclusion change or
disappearance, any new `UnknownHostPayload`, any new `self_reaching` row, any
new row in untouched code, or any retention reclassification.

**Fence:** `LazyTable`'s eager bound evaluation (`lazy.rs:437-439`) stays as it
is (step B); its body realizes through the new machine, which is in scope. No
change to rum-runtime, rum-core, the host crates, the `Part` seam, any public
API, trait or macro protocol, the source-budget charging, or any other
suspension group. No edits to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`,
earlier briefs, the lockfile, or unrelated docs and tests.

**Disclosure duty:** every moved item and statement, every changed inventory
row, every new frame field and its roots, the frozen/working copy overlap, the
checkpoint, every measured candidate and label, and any adjacent defect you
meet. `First` and `Take` are N/Pending in the census (brief-561's "Unsettled"
note was wrong); surface any classification you think must change, do not
change it.

## Deliverable

1. **The flat machine** (design.md): one `PullSource` plus a `Vec<PullStage>`,
   outermost first, driven by an indexed Down/Up cursor. No stage, source or
   machine owns another machine node. Construction from the descriptor is an
   iterative walk that validates the whole descriptor before `Take[_, 0]`
   returns. Preserve every semantic in design.md's apply-once table: `Take`
   checks and decrements **before** descending and ascends transparently;
   `Drop` decrements a skip only after `Some`, then requests its child again;
   `Select` rejection restarts at its own child; `FoldList` emits its init
   without pulling; `NestList` yields its saved old value after computing the
   lookahead; sources check exhaustion before charging; only sources charge.
   No fusion, no memo; every realization builds fresh state from the inert
   descriptor.
2. **Suspension and resume.** Capture only a callback's `InputRequired` into a
   cold `AwaitCursor` (stage index and phase; `Select`'s candidate or
   `NestList`'s saved yield) on the error edge; no pending check, allocation or
   clone on a successful pull. One `LazyRealizeSuspensionFrame` owns the
   machine, the realized prefix, the output budget, the saved source count and
   ceiling, and the cursor; the first suspension moves live state into it. On
   an actual `Ok` resume, make a cold working copy (the queued `Arc` stays
   Session-rooted while it runs; never `try_unwrap`, never a `Mutex`/`RefCell`
   in a frame), apply the one answer, and continue the same loop at the cursor.
   A second suspension freezes a fresh cursor; the consumer suffix is not
   duplicated. Re-acquire the cancellation observer on resume; never store it,
   a `Session` borrow or a `NativeStackDepth` in a frame.
3. **The consumers.** `Normal`, `Take` and `First` completion frames hold the
   original evaluated call and decode the machine's result through the existing
   `Option<Expr>` codec before returning a value, default, residual or message.
   `Part` keeps its seam.
4. **The cancellation checkpoint — director ruling.** After each guest callback
   returns `Ok`, and before its value is applied, observe cancellation and stop
   with the same cancellation outcome a source pull produces; it charges no
   source work. That is the **only** new checkpoint. Do **not** add the polls
   brief-562 proposed at construction or resume entry or every 256 stage
   transitions, nor a shallow specialization: between two source polls the
   machine does O(depth) callback-free work, and the depth is bounded by the
   charged, cancellable work that built the descriptor. The checkpoint makes
   every lazy callback a cancellation boundary, as every source pull already
   is; it is what makes the queued and resumed paths agree when the request is
   cancelled at a callback (today queued log 32, resumed log 1).
5. **Roots and abandonment.** Every new type derives `FrameRootTrace`; frames
   derive `PureFrameAbandon`, which runs no callback. Every root is traced,
   including the pending `Select` candidate and `NestList` yield.
6. **Documentation.** `docs/guide/evaluation.md` §Lazy Sequences states that a
   realization observes cancellation at each source pull and after each
   callback, and that a suspended realization resumes into its work.
   `docs/dev/lazy-sequences-design.md` §§2 and 5 record the flat machine, and
   distinguish its finite descriptor state from the growing realized prefix
   (the old "realized elements are never retained" wording is wrong for a
   realizer's output).

## Tests

**Red first, recorded before any production edit.**

- **The oracle suite** (`lazy_pull_continuations.rs`): promote the 33 step-A
  cases from brief-561 unchanged (all except `LazyTable-bound-1/2/3`,
  `LazyTable-bounds-twice`, `LazyTable-invalid-step`,
  `LazyTable-body-and-bounds`), with the harness's full equality — `Expr`
  values, ordered messages, prints, error and termination, the final `log`, the
  exact prompt count and order, no pending owner and a working `1 + 1` after.
  Identical print sinks on both paths. Pre-fix: 31 red, 2 controls.
- **Structural cases** brief-561 named but did not run, recorded pre-fix: the
  last permitted `LazyTake` child suspends with its count already zero; an
  alternating `LazyTake`/`LazyDrop` chain with both skipping and yielding
  phases; one callback with two inputs before it returns; `FoldList` init above
  another suspending adapter; finite exhaustion through a positive `Drop` and
  through a resumed, rejected `Select`; a callback that realizes a second lazy
  list which suspends (nested realization).
- **Depth cases, in-process.** brief-562's nine (`no_pull`, `invalid_zero`,
  `pull`, `inner`, `outer`, `twice`, `select`, `drop`, `cancel`) at **n = 2 000**
  on a `std::thread::Builder::stack_size(2 * 1024 * 1024)` thread that also owns
  the Session's teardown, each forcing every stage and asserting the exact
  result (n incrementing Maps return n with n callbacks; `inner`, `outer`,
  `twice` return `List[n]` with every prompt; `select` 2 with log 2n+1; `drop`
  n with log n+1; `cancel` `Cancelled` with log 1 and no further callback).
  Keep the n = 32 companions too. **Why 2 000, and not brief-562's 10 000**
  (director-measured, debug `rum-cli` on a 2 MiB stack): the pre-fix lazy
  ceiling is 300-600, but the descriptor `Expr`'s own build and drop — recorded
  standing debt, outside this brief — already aborts between 3 000 and 4 000;
  a test at 10 000 would abort before the machine is reached. If your own
  control (build and drop the descriptor without realizing it, on the same
  thread) does not pass at 2 000, **STOP** and report the measured ceiling;
  do not lower n silently. Pre-fix, each deep test aborts the test binary:
  record that by running each one **alone** (`-- --exact <name>`), never in a
  shared binary.
- **Depth cases, release.** Through the release `rum-cli` under
  `bash -c 'ulimit -s 2048; ulimit -c 0; …'`, stdin `/dev/null`, the same nine
  shapes at n = 20 000 (the descriptor control survives there; pre-fix the
  machine aborts at ~2 170): pre-fix exit 134, post-fix exit 0 with the exact
  printed values. Save the `.rum` sources and outputs.
- **Structural depth checks — the enforcement.** A depth alone cannot falsify a
  cheap recursive walk (a 48-byte recursive drop would need ~40 000 levels,
  beyond the `Expr` debt's own ceiling), so the property is enforced by
  structure, in the new lazy depth-contract test module:
  (a) `recursive_edges_from_source` over every file of `lazy.rs` and `lazy/`
  reports **zero** recursive edges; (b) no row of
  `ownership_graph/inventory/self_reaching.tsv` names a `rum_stdlib::lazy::`
  type. Ship a falsifier for each (`TEMPLATE.md`, "a check nobody has watched
  fail"): (a) on a synthetic lazy-shaped source with one recursive helper
  reports exactly that edge; (b) on a synthetic row naming a lazy type fails.
  Both checks must fail on the incoming tree (`stepper_from_descriptor` and
  `next` recurse; `PullStepper` is self-reaching) — record it.
- **Private walk tests** under the new lazy module, on a 2 MiB thread at the
  same n: clone and working copy, root visits of every variant (pending
  candidate and saved yield included), last-owner drop and abandonment, each
  isolated from descriptor construction.

Post-fix every case runs and passes; an ignored test, a caught
`StackLimitExceeded` or a lowered n is not a pass. No existing assertion is
weakened.

## rum / Rust pitfalls (read twice)

1. Up carries a boundary (the next parent is `boundary - 1`). Retrying a
   `Select`/`Drop` child must not re-decrement an outer `Take`; exhaustion still
   clears every waiting `Drop`; `FoldList` init turns upward without entering
   its child.
2. A resumed `NestList` answer becomes the next current; the output is the saved
   old value, and the source charge is already paid. A `Map` child or
   combinatorial cursor has already advanced; never recompute it. A false
   predicate result is discarded before the restart.
3. Keep the capture inside the callback's error edge. brief-560 showed that
   state carried in a hot loop's error arm changes its codegen; "cold" is not
   evidence. A regression is fixed in the flat loop, never by restoring
   recursion — the recursion is the abort.
4. No borrowed stage reference may survive a callback evaluation or a `Vec`
   mutation.
5. Root tracing is not retention coverage; the prefix, candidate, current value,
   accumulator and `Vec` capacity remain the retention line's R02/R05 work. No
   leases.

## Verification (MANDATORY — run to green or evidence)

All commands run **serially**, one at a time, through
`python3 scripts/run-bounded.py --timeout <s> -- env CARGO_BUILD_JOBS=1
RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=<repo>/.temp/brief-563/tmp …`.
Bounds, from measured serial history (briefs 559-562): a focused integration
or unit-test target 150 s (observed 7-15 s plus a rebuild);
`ownership_graph`, `local_production_acceptance` (either feature set) and
`cargo test -p rum-runtime` 120 s each (observed 5-45 s);
`cargo test -p rum-stdlib` and `cargo test --workspace` 1 500 s each (observed
990-1 047 s and 651-1 012 s); a release `rum-cli` build 300 s incremental,
1 200 s for the first build into a fresh target directory; a release CLI depth
probe 60 s. A timeout is a recorded failure, not a licence to retry unbounded.

After focused green and the final inventory replay, archive verbatim output of:

```text
cargo fmt --check
cargo test -p rum-stdlib --test lazy_pull_continuations
cargo test -p rum-stdlib --test loop_continuations
cargo test -p rum-stdlib --test guest_loading_resume
cargo test -p rum-stdlib lazy
cargo test -p rum-stdlib ownership_graph
cargo test -p rum-stdlib local_production_acceptance
cargo test -p rum-stdlib --no-default-features local_production_acceptance
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Report native ZeroMQ and any other platform skips as skips.

**Performance gate.** Execute `.temp/brief-562/performance-gate.md` with its
fixed 54-label manifest (`performance-panel.json`: brief-561's 30 labels at
N = 20 000/40 000 with their exact setup, plus the 24 depth labels at
N = 1 000/2 000). Before any source edit, build and preserve the incoming
default and `alloc-count` release binaries with source hashes. For the base and
every measured candidate, in this order: allocation counts, then two paired
samples of `taskset -c 2 perf stat -e cpu_core/instructions:u`, one label per
process, stdin `/dev/null`, per-iteration `(count(2N) - count(N)) / N`, **all 54
labels**, profiling variables off except for the allocation pass. Check each
workload's printed result first; a label whose result is wrong measured
nothing. **Director ruling on the decision rule:** no label may show a
new per-call allocation or byte slope, and no label may exceed the base by more
than **+0.05% in both paired instruction samples**; a breach is a regression to
fix or report, including on a neighbour label. The full
`scripts/core-language-microbench.sh` timing suite runs **twice only** — base
and the final candidate — with `--rum-cli`, `--output-dir` (in scratch),
`--timeout` and `--build-timeout` passed explicitly; every `timings.tsv` row is
reported, and **timing informs, it does not decide** (on this host timing has
read +145% where instructions read +0.04%).

**Caps and stop-loss:** at most eight focused Cargo attempts per milestone,
three measured candidates, 1 728 metric processes, the two timing-suite runs,
and 100 release depth probes. Stop after two consecutive compiling attempts
that reduce none of: failing oracle and structural cases, remaining Lost
sites, regressing labels, the worst regression. Falling compile-error counts
are progress before the tree compiles. At a cap, report — never relax an
assertion or call a regression free.

## Checkpointing — protect the scene

Keep everything under `.temp/brief-563/`: the immutable pre-fix evidence,
preserved binaries and hashes, every candidate's source and binaries, raw perf
and allocation output (saved before parsing), the depth probes and their
outputs. After each milestone update `.temp/brief-563/PROGRESS.md` with what is
done and verified, the exact next action, whether the tree compiles and which
files are half-edited, and the counters above, so the director can resume
without rediscovery.

## Hard rules

- **No `git`, in any form.** The director owns status, diffs and commits.
- No network; touch nothing outside this repository. All scratch, `.rum`
  probes, `TMPDIR`, binaries and logs stay in `.temp/brief-563/`.
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the release `rum-cli`, `taskset`/`perf`, the microbench script.
- Scope is the objective, the fence and the disclosure duty. STOP and report
  before editing past the fence.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's documented behaviour
  and rum-vs-rum comparisons only.
- No new `unsafe`, dependency, depth cap, lease, mutable frame payload,
  rootless payload, source-budget redefinition, public API, or other
  suspension group.
- No allocation, clone, path record or pending check on a successful pull.
  Recursive construction, pull, clone, trace, drop or abandonment is never an
  acceptable fallback.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Files and items changed or moved; the source, stage, cursor and frame types
   with their roots; the checkpoint; the documentation changes and file sizes.
2. Verbatim pre-fix and post-fix output: the oracle suite, the structural
   cases, the in-process and release depth cases (exact values, prompts, logs,
   exit statuses), the structural depth checks and their falsifiers, the broad
   suites.
3. The performance gate: all 54 allocation and instruction rows for base and
   every candidate, the two timing-suite runs, preserved hashes, and every
   regression.
4. Lost continuations before and after (80 -> 69 predicted), every standing-duty
   row change including the named exceptions, the stop-loss counters, and open
   questions.

---
