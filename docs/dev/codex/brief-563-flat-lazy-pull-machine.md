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

## Execution report — candidate 3 retained; gates not green


The scoped flat machine and resumable consumers are implemented. All 59 new integration cases pass, the full standard-library suite passes, and the machine no longer aborts in any of the nine release shapes at depth 20,000. This is **not an accepted performance or workspace gate**. Candidate 3 still regresses on 24 paired instruction labels; two runtime inventories require an edit outside the authorized fence; four release CLI input shapes exit 1 instead of the required 0. Tuning stopped at the three-candidate cap. The director owns the resulting tree and all version-control actions; no git command was run.


No runtime/core/host/adapter source, public API, trait or macro protocol, dependency, lockfile, Part seam, root guide/roadmap, or earlier brief was edited. The proposed runtime inventory patch is saved but **unapplied**; the requested scope decision has not arrived. All task-owned scratch is under `.temp/brief-563/`. Existing required test harnesses write their fixed review/fixture locations, including `target/brief-544/` and `.temp/brief-546/certificates/`; those harnesses were not rewritten.


### 1. Files, moved statements, state and roots


The complete source-only before/after patch is [final/source-diff.txt](../../../.temp/brief-563/final/source-diff.txt). The final source audit verified that all 1,231 captured candidate-3 source/input hashes still match after broad testing. [final/unchanged-lazy-items.json](../../../.temp/brief-563/final/unchanged-lazy-items.json) records 20 byte-identical retained lazy functions, including the whole LazyTable constructor and its three eager bound evaluations. Both source cursor helpers moved byte-for-byte.


| Incoming item or statements | Final responsibility |

| --- | --- |

| lazy::PullStepper | Removed; split into state::{PullMachine, PullSource, PullStage}; no machine node owns another. |

| PullStepper::next source arms | source::pull_source; Range/Permutations/Subsets/Tuples cursor order retained; Nest adds only error capture and the successful-callback cancellation checkpoint. |

| PullStepper::next adapter arms | machine::drive_pull; indexed descent/ascent and exhausted; no recursive child calls. |

| stepper_from_descriptor | descriptor::machine_from_descriptor; borrowed outer-first iterative validation, including the complete tail before Take[_,0] returns. |

| PullContext and new/charge_source_pull | state::PullContext; scalar work limit/count move into SavedPullLimits; observer stays live-only; pending cursor is populated only on InputRequired. |

| advance_subset_cursor / advance_tuple_cursor | source module, unchanged function bodies. |

| realize output-driving loop | realization::drive_items; lazy::realize keeps the entry, validates/builds and moves state to cold capture only on error. |

| PullBudget and reached | Remain in lazy.rs; only the budget derive gains FrameRootTrace. |

| Normal / Take / First lazy branches | Same owners and original fallback constructs; add error-edge calls to cold completion children. First still uses resolve_first_like; Part is unchanged. |

| New realization and consumer captures | Existing resume_or_suspend! protocol; machine insert arm and three consumer captures. All actual new macro rows are disclosed below. |

| Private walk tests | New lazy/tests/mod.rs responsibility; moved from the initial lazy/tests.rs location to satisfy the existing tests-directory hygiene convention. |

| Depth enforcement | New association_depth_contract/lazy_depth_contract.rs; parent mounts the sibling; existing recursion scanner changes visibility only. |



Candidate 1 used normal Vec growth and rebuilt a value-bearing Down/Up cursor at every transition. Candidate 2 reserves one stage slot initially and, only when it fills, counts the remaining inert adapter tail and reserves its exact capacity. Thus empty and single-stage construction have no unconditional extra walk. The validating walk still checks every node; there is no fusion, memoization, depth cap, or elision. Candidate 3 keeps the indexed cursor at entry but runs cooperating descent/ascent loops, carrying one value during ascent. Positive Drop and rejected Select restart at their own child; outer Take allowances stay consumed. Exhaustion clears every waiting Drop.


A first suspension moves the machine and prefix. The frozen prefix uses existing `SharedArgs`, accepting the live Vec only on the cold suspension edge. This follows the repository rule against a directly owned Vec frame field. The immutable frame remains Session-rooted during resume. Only an actual Ok outcome creates the working machine/prefix/cursor copy; Err forwards without copying. Frozen and working owners overlap until the runtime retires the old queued Arc. A second suspension freezes fresh working state without duplicating the consumer suffix. Copy and consumer completion work stays in cold named helpers; no Arc::try_unwrap, interior-mutability payload, or ownership transfer shortcut was added.


| Type / field or variant | Traced roots / scalar state |

| --- | --- |

| PullMachine | source plus every stage; Vec order is outermost first. |

| PullSource::Range | current/step Rational scalars; no Expr roots. |

| PullSource::Permutations | every current Expr; exhausted is scalar. |

| PullSource::Subsets | every elements Expr; k/combo are scalar cursor state. |

| PullSource::Tuples | every inner_lists Expr; digits/exhausted are scalar. |

| PullSource::NestList | function and current Expr. |

| PullStage::Map / Select | function / predicate Expr. |

| PullStage::Take / Drop | remaining scalar; no Expr root. |

| PullStage::FoldList | function and accumulator Expr; emitted_init scalar. |

| AwaitCursor::Map / Fold | stage index, no Expr root. |

| AwaitCursor::Select | stage index and pending candidate Expr. |

| AwaitCursor::Nest | saved yield_value Expr. |

| SavedPullLimits / PullBudget | saved source count/ceiling and output mode/limit; scalar, rootless by construction. |

| LazyRealizeSuspensionFrame.machine | all source and stage roots above. |

| LazyRealizeSuspensionFrame.items | every realized prefix Expr through immutable SharedArgs. |

| LazyRealizeSuspensionFrame.budget / limits / cursor | scalar budget/count/ceiling plus pending candidate or saved yield when present. |

| NormalLazyCompletionFrame / TakeLazyCompletionFrame | original evaluated head Expr and all argument Exprs in SharedArgs. |

| FirstLazyCompletionFrame | all original evaluated argument Exprs in SharedArgs; canonical First head is reconstructed by the existing consumer logic. |

| Live DriveCursor / PullContext | Not frozen fields and do not gain trace derives; local carried Expr, indices, live observer and error-only pending slot. |



All nine new stored state/frame types derive FrameRootTrace; the existing budget gains that derive too. The four frames derive PureFrameAbandon and run no callback on abandonment. Root tracing is **not** retention coverage: prefix/candidate/current/accumulator occurrences, Vec capacity and frozen/working overlap remain R02/R05 work, with no new lease or LocalCovered classification.


The one new checkpoint is after a successful guest callback and before applying its answer, including a resumed answer. It consumes no source work. There is no constructor poll, resume-entry poll, periodic stage poll or shallow specialization. Sources still check exhaustion before charging; only sources charge. FoldList init emits without a child pull; NestList applies lookahead and yields the saved old value. The 33 promoted cases retain complete Expr/message/print/error/termination/log/prompt equality and identical print sinks, pending-owner checks, and post-run 1+1 checks.


Documentation edits are restricted to evaluation.md’s Lazy Sequences section and lazy-sequences-design.md §§2/5: callback/source cancellation boundaries, resumable realization/consumer completion, flat walks, retained output prefix and copy overlap.


| File | Characters | Assessment |

| --- | --- | --- |

| crates/rum-stdlib/src/associations/basic.rs | 9921 | Preferred range |

| crates/rum-stdlib/src/associations/basic/lazy_completion.rs | 1551 | Preferred range |

| crates/rum-stdlib/src/lazy.rs | 13442 | Warning range; named responsibility retained |

| crates/rum-stdlib/src/lazy/continuation.rs | 5367 | Preferred range |

| crates/rum-stdlib/src/lazy/descriptor.rs | 6341 | Preferred range |

| crates/rum-stdlib/src/lazy/machine.rs | 5553 | Preferred range |

| crates/rum-stdlib/src/lazy/realization.rs | 761 | Preferred range |

| crates/rum-stdlib/src/lazy/source.rs | 3610 | Preferred range |

| crates/rum-stdlib/src/lazy/state.rs | 3564 | Preferred range |

| crates/rum-stdlib/src/lazy/tests/mod.rs | 7516 | Preferred range |

| crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs | 21138 | Existing split-plan range; broader split outside fence |

| crates/rum-stdlib/src/structure_ops/sequence/take_drop/lazy_completion.rs | 1541 | Preferred range |

| crates/rum-stdlib/src/structure_ops/surface.rs | 14797 | Warning range; named responsibility retained |

| crates/rum-stdlib/src/structure_ops/surface/lazy_completion.rs | 1684 | Preferred range |

| crates/rum-stdlib/src/tests/association_depth_contract/lazy_depth_contract.rs | 2509 | Preferred range |

| crates/rum-stdlib/src/tests/association_depth_contract/mod.rs | 352 | Preferred range |

| crates/rum-stdlib/src/tests/association_depth_contract/recursion_inventory.rs | 28288 | Existing split-plan range; broader split outside fence |

| crates/rum-stdlib/tests/lazy_pull_continuations.rs | 14728 | Warning range; named responsibility retained |

| crates/rum-stdlib/tests/lazy_pull_continuations/depth.rs | 7709 | Preferred range |

| crates/rum-stdlib/tests/lazy_pull_continuations/structural.rs | 2135 | Preferred range |

| docs/guide/evaluation.md | 18926 | Warning range; named responsibility retained |

| docs/dev/lazy-sequences-design.md | 9535 | Preferred range |



All new production files are below 10k characters. The 14.7k integration root keeps the shared oracle and 33 promoted cases together; structural and depth responsibilities are separate children. Existing take_drop.rs and the shared recursion scanner remain in split-plan range; only the scoped lazy branch / visibility was changed.


### 2. Verbatim verification evidence


Commands, bounds, exit codes and raw output are archived for every attempt. The complete Cargo ledger is [final/cargo-command-ledger.json](../../../.temp/brief-563/final/cargo-command-ledger.json). All Cargo commands set CARGO_BUILD_JOBS=1, RUST_TEST_THREADS=1, CARGO_NET_OFFLINE=true and TMPDIR under this scratch directory; all builds/tests/measurements ran serially through the bounded runner.


#### Red evidence before production edits


<details>
<summary>pre/descriptor-control.log</summary>


[pre/descriptor-control.log](../../../.temp/brief-563/pre/descriptor-control.log)


```text
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling proc-macro2 v1.0.106
   Compiling syn v2.0.117
   Compiling version_check v0.9.5
   Compiling generic-array v0.14.7
   Compiling autocfg v1.5.0
   Compiling typenum v1.20.1
   Compiling num-traits v0.2.19
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling serde_derive v1.0.228
   Compiling thiserror v2.0.18
   Compiling cfg-if v1.0.4
   Compiling digest v0.10.7
   Compiling num-integer v0.1.46
   Compiling thiserror-impl v2.0.18
   Compiling num-bigint v0.4.6
   Compiling cpufeatures v0.2.17
   Compiling crc32fast v1.5.0
   Compiling rum-core v0.2.0 (/home/remi/dev/rum/crates/rum-core)
   Compiling rum-syntax v0.2.0 (/home/remi/dev/rum/crates/rum-syntax)
   Compiling rum-derive v0.2.0 (/home/remi/dev/rum/crates/rum-derive)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling sha1 v0.10.6
   Compiling sha2 v0.10.9
   Compiling md-5 v0.10.6
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 05s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 1 test
test depth::descriptor_control_deep ... DESCRIPTOR CONTROL depth=2000 stack=2097152 result=0 teardown=complete
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 58 filtered out; finished in 1.40s
```

</details>


<details>
<summary>pre/oracle.log</summary>


[pre/oracle.log](../../../.temp/brief-563/pre/oracle.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 33 tests
test cancel_first_suspension ... CASE	Lazy-cancel-at-1	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1, 2]", [])
ROW	Lazy-cancel-at-1	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2]"	1	1	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary"]
ok
test cancel_second_suspension ... CASE	Lazy-cancel-at-2	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: ["boundary"] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2, 3, 4]", [])
ROW	Lazy-cancel-at-2	"2"	"List[1, 2]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2, 3, 4]"	1	2	false	"OK"	"OK"	"None"	"Some(Cancelled)"	[]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary", "boundary"]

thread 'cancel_second_suspension' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:162:5:
assertion `left == right` failed: must reach cancellation boundary
  left: ["p: "]
 right: ["p: ", "p: "]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test lazy_adapter_unwind_twice ... CASE	Lazy-adapter-unwind-twice	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "List[List[inner, 1], List[outer, 1], List[inner, 2]]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[1], List[2], List[3]]"), termination: "None", messages: [], prints: [] }, "List[List[inner, 1], List[outer, 1], List[inner, 2], List[outer, 2], List[inner, 3], List[outer, 3]]", [])
ROW	Lazy-adapter-unwind-twice	"2"	"List[List[inner, 1], List[outer, 1], List[inner, 2]]"	"List[List[1], List[2], List[3]]"	"List[List[inner, 1], List[outer, 1], List[inner, 2], List[outer, 2], List[inner, 3], List[outer, 3]]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazy_adapter_unwind_twice' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:77:5:
assertion `left == right` failed: must reach every suspension
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED
test lazy_infinite_after_resume ... CASE	Lazy-infinite-after-resume	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationWorkLimit\"]"), termination: "Some(IterationWorkLimit)", messages: [RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }], prints: [] }, "List[1, 2, 3, 4]", [])
ROW	Lazy-infinite-after-resume	"2"	"List[1, 2]"	"TerminatedEvaluation[\"IterationWorkLimit\"]"	"List[1, 2, 3, 4]"	1	1	false	"OK"	"OK"	"None"	"Some(IterationWorkLimit)"	[]	[RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }]	[]	[]

thread 'lazy_infinite_after_resume' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("TerminatedEvaluation[\"IterationWorkLimit\"]"), termination: "Some(IterationWorkLimit)", messages: [RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }], prints: [] }
FAILED
test lazy_rerederive ... CASE	Lazy-rerederive	resumed=(View { status: "OK", value: Some("List[\"two\", \"two\"]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 1, 2]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[List[1, \"two\", 3], List[1, \"two\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 1, 2, 3]", [])
ROW	Lazy-rerederive	"List[\"two\", \"two\"]"	"List[1, 2, 1, 2]"	"List[List[1, \"two\", 3], List[1, \"two\", 3]]"	"List[1, 2, 3, 1, 2, 3]"	2	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazy_rerederive' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("List[\"two\", \"two\"]"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[List[1, \"two\", 3], List[1, \"two\", 3]]"), termination: "None", messages: [], prints: [] }
FAILED
test lazy_source_budget_exact ... CASE	Lazy-source-budget-exact	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])
ROW	Lazy-source-budget-exact	"2"	"List[1, 2]"	"List[3, 4]"	"List[1, 2, 3, 4]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazy_source_budget_exact' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[3, 4]"), termination: "None", messages: [], prints: [] }
FAILED
test lazydrop_inner ... CASE	LazyDrop-inner	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])
ROW	LazyDrop-inner	"\"two\""	"List[1, 2]"	"List[3, 4]"	"List[1, 2, 3, 4]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazydrop_inner' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[3, 4]"), termination: "None", messages: [], prints: [] }
FAILED
test lazydrop_twice ... CASE	LazyDrop-twice	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("5"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	LazyDrop-twice	"\"two\""	"List[1, 2]"	"5"	"List[1, 2, 3, 4, 5]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazydrop_twice' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:77:5:
assertion `left == right` failed: must reach every suspension
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED
test lazyfirst_consumer ... CASE	LazyFirst-consumer	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[\"two\"]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])
ROW	LazyFirst-consumer	"\"two\""	"List[1, 2]"	"List[\"two\"]"	"List[1, 2]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazyfirst_consumer' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[\"two\"]"), termination: "None", messages: [], prints: [] }
FAILED
test lazyfirst_empty_default ... CASE	LazyFirst-empty-default	resumed=(View { status: "OK", value: Some("False"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("99"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyFirst-empty-default	"False"	"List[1, 2]"	"99"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazyfirst_empty_default' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("False"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("99"), termination: "None", messages: [], prints: [] }
FAILED
test lazyfirst_empty_message ... CASE	LazyFirst-empty-message	resumed=(View { status: "OK", value: Some("False"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("First[List[]]"), termination: "None", messages: [RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }, RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyFirst-empty-message	"False"	"List[1, 2]"	"First[List[]]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }, RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }]	[]	[]

thread 'lazyfirst_empty_message' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("False"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("First[List[]]"), termination: "None", messages: [RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }, RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }], prints: [] }
FAILED
test lazyfoldlist ... CASE	LazyFoldList	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[0, 1, 3, 6]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyFoldList	"3"	"List[1, 2]"	"List[0, 1, 3, 6]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazyfoldlist' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[0, 1, 3, 6]"), termination: "None", messages: [], prints: [] }
FAILED
test lazyfoldlist_inner ... CASE	LazyFoldList-inner	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[], List[1], List[1, \"two\"], List[1, \"two\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyFoldList-inner	"\"two\""	"List[1, 2]"	"List[List[], List[1], List[1, \"two\"], List[1, \"two\", 3]]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazyfoldlist_inner' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[List[], List[1], List[1, \"two\"], List[1, \"two\", 3]]"), termination: "None", messages: [], prints: [] }
FAILED
test lazyfoldlist_twice ... CASE	LazyFoldList-twice	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[0, 1, 3, 6, 10, 15]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	LazyFoldList-twice	"3"	"List[1, 2]"	"List[0, 1, 3, 6, 10, 15]"	"List[1, 2, 3, 4, 5]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazyfoldlist_twice' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:77:5:
assertion `left == right` failed: must reach every suspension
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED
test lazymap ... CASE	LazyMap	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyMap	"\"two\""	"List[1, 2]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazymap' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }
FAILED
test lazymap_error_in_callback ... CASE	LazyMap-error-in-callback	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: [] }, "List[1, 2]", [])
ROW	LazyMap-error-in-callback	"<error>"	"List[1, 2]"	"<error>"	"List[1, 2]"	1	1	true	"Other { message: \"uncaught throw: 9 with tag None\" }"	"Other { message: \"uncaught throw: 9 with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }]	[]	[]
ok
test lazymap_error_next ... CASE	LazyMap-error-next	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }], prints: ["before"] }, "List[1, 2]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])
ROW	LazyMap-error-next	"2"	"List[1, 2]"	"<error>"	"List[1, 2, 3]"	1	1	false	"OK"	"Other { message: \"uncaught throw: 9 with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }]	["before"]	["before", "after"]

thread 'lazymap_error_next' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("2"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }], prints: ["before"] }
 right: View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }
FAILED
test lazymap_inner ... CASE	LazyMap-inner	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[1], List[\"two\"], List[3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyMap-inner	"\"two\""	"List[1, 2]"	"List[List[1], List[\"two\"], List[3]]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazymap_inner' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[List[1], List[\"two\"], List[3]]"), termination: "None", messages: [], prints: [] }
FAILED
test lazymap_twice ... CASE	LazyMap-twice	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3, \"four\", 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	LazyMap-twice	"\"two\""	"List[1, 2]"	"List[1, \"two\", 3, \"four\", 5]"	"List[1, 2, 3, 4, 5]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazymap_twice' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:77:5:
assertion `left == right` failed: must reach every suspension
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED
test lazynestlist ... CASE	LazyNestList	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])
ROW	LazyNestList	"3"	"List[1, 2]"	"List[1, 2, 3, 4]"	"List[1, 2, 3, 4]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazynestlist' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[1, 2, 3, 4]"), termination: "None", messages: [], prints: [] }
FAILED
test lazynestlist_lookahead_first ... CASE	LazyNestList-lookahead-first	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "List[1]", ["p: "])	queued=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "List[1]", [])
ROW	LazyNestList-lookahead-first	"2"	"List[1]"	"1"	"List[1]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazynestlist_lookahead_first' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }
FAILED
test lazynestlist_twice ... CASE	LazyNestList-twice	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3, 4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	LazyNestList-twice	"3"	"List[1, 2]"	"List[1, 2, 3, 4, 5]"	"List[1, 2, 3, 4, 5]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazynestlist_twice' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:77:5:
assertion `left == right` failed: must reach every suspension
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED
test lazypart_consumer ... CASE	LazyPart-consumer	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[\"two\"]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])
ROW	LazyPart-consumer	"\"two\""	"List[1, 2]"	"List[\"two\"]"	"List[1, 2]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazypart_consumer' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[\"two\"]"), termination: "None", messages: [], prints: [] }
FAILED
test lazypermutations_cursor ... CASE	LazyPermutations-cursor	resumed=(View { status: "OK", value: Some("List[1, 3, 2]"), termination: "None", messages: [], prints: [] }, "List[List[1, 2, 3], List[1, 3, 2]]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]"), termination: "None", messages: [], prints: [] }, "List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]", [])
ROW	LazyPermutations-cursor	"List[1, 3, 2]"	"List[List[1, 2, 3], List[1, 3, 2]]"	"List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]"	"List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazypermutations_cursor' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("List[1, 3, 2]"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]"), termination: "None", messages: [], prints: [] }
FAILED
test lazyselect ... CASE	LazySelect	resumed=(View { status: "OK", value: Some("True"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazySelect	"True"	"List[1, 2]"	"List[1, 2, 3]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazyselect' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("True"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[1, 2, 3]"), termination: "None", messages: [], prints: [] }
FAILED
test lazyselect_inner ... CASE	LazySelect-inner	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazySelect-inner	"\"two\""	"List[1, 2]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazyselect_inner' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }
FAILED
test lazyselect_never_after_resume ... CASE	LazySelect-never-after-resume	resumed=(View { status: "OK", value: Some("False"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationWorkLimit\"]"), termination: "Some(IterationWorkLimit)", messages: [RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }], prints: [] }, "List[1, 2, 3, 4]", [])
ROW	LazySelect-never-after-resume	"False"	"List[1, 2]"	"TerminatedEvaluation[\"IterationWorkLimit\"]"	"List[1, 2, 3, 4]"	1	1	false	"OK"	"OK"	"None"	"Some(IterationWorkLimit)"	[]	[RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }]	[]	[]

thread 'lazyselect_never_after_resume' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("False"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("TerminatedEvaluation[\"IterationWorkLimit\"]"), termination: "Some(IterationWorkLimit)", messages: [RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }], prints: [] }
FAILED
test lazyselect_reject_resumed_candidate ... CASE	LazySelect-reject-resumed-candidate	resumed=(View { status: "OK", value: Some("False"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, 3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])
ROW	LazySelect-reject-resumed-candidate	"False"	"List[1, 2]"	"List[1, 3, 4]"	"List[1, 2, 3, 4]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazyselect_reject_resumed_candidate' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("False"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[1, 3, 4]"), termination: "None", messages: [], prints: [] }
FAILED
test lazyselect_twice ... CASE	LazySelect-twice	resumed=(View { status: "OK", value: Some("True"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3, 4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	LazySelect-twice	"True"	"List[1, 2]"	"List[1, 2, 3, 4, 5]"	"List[1, 2, 3, 4, 5]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazyselect_twice' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:77:5:
assertion `left == right` failed: must reach every suspension
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED
test lazysubsets_cursor ... CASE	LazySubsets-cursor	resumed=(View { status: "OK", value: Some("List[1]"), termination: "None", messages: [], prints: [] }, "List[List[], List[1]]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[], List[1], List[2]]"), termination: "None", messages: [], prints: [] }, "List[List[], List[1], List[2]]", [])
ROW	LazySubsets-cursor	"List[1]"	"List[List[], List[1]]"	"List[List[], List[1], List[2]]"	"List[List[], List[1], List[2]]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazysubsets_cursor' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("List[1]"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[List[], List[1], List[2]]"), termination: "None", messages: [], prints: [] }
FAILED
test lazytable_body ... CASE	LazyTable-body	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-body	"\"two\""	"List[1, 2]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazytable_body' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }
FAILED
test lazytake_consumer ... CASE	LazyTake-consumer	resumed=(View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTake-consumer	"\"two\""	"List[1, 2]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazytake_consumer' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("\"two\""), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }
FAILED
test lazytuples_cursor ... CASE	LazyTuples-cursor	resumed=(View { status: "OK", value: Some("List[1, 4]"), termination: "None", messages: [], prints: [] }, "List[List[1, 3], List[1, 4]]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[1, 3], List[1, 4], List[2, 3]]"), termination: "None", messages: [], prints: [] }, "List[List[1, 3], List[1, 4], List[2, 3]]", [])
ROW	LazyTuples-cursor	"List[1, 4]"	"List[List[1, 3], List[1, 4]]"	"List[List[1, 3], List[1, 4], List[2, 3]]"	"List[List[1, 3], List[1, 4], List[2, 3]]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazytuples_cursor' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("List[1, 4]"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[List[1, 3], List[1, 4], List[2, 3]]"), termination: "None", messages: [], prints: [] }
FAILED

failures:

failures:
    cancel_second_suspension
    lazy_adapter_unwind_twice
    lazy_infinite_after_resume
    lazy_rerederive
    lazy_source_budget_exact
    lazydrop_inner
    lazydrop_twice
    lazyfirst_consumer
    lazyfirst_empty_default
    lazyfirst_empty_message
    lazyfoldlist
    lazyfoldlist_inner
    lazyfoldlist_twice
    lazymap
    lazymap_error_next
    lazymap_inner
    lazymap_twice
    lazynestlist
    lazynestlist_lookahead_first
    lazynestlist_twice
    lazypart_consumer
    lazypermutations_cursor
    lazyselect
    lazyselect_inner
    lazyselect_never_after_resume
    lazyselect_reject_resumed_candidate
    lazyselect_twice
    lazysubsets_cursor
    lazytable_body
    lazytake_consumer
    lazytuples_cursor

test result: FAILED. 2 passed; 31 failed; 0 ignored; 0 measured; 26 filtered out; finished in 0.28s

error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`
```

</details>


<details>
<summary>pre/structural.log</summary>


[pre/structural.log](../../../.temp/brief-563/pre/structural.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 7 tests
test structural::alternating_take_drop ... CASE	alternating Take/Drop skip and yield	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	alternating Take/Drop skip and yield	"2"	"List[1, 2]"	"List[4, 5]"	"List[1, 2, 3, 4, 5]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'structural::alternating_take_drop' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:77:5:
assertion `left == right` failed: must reach every suspension
  left: ["p: "]
 right: ["p: ", "p: "]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test structural::finite_positive_drop ... CASE	positive Drop finite exhaustion	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])
ROW	positive Drop finite exhaustion	"2"	"List[1, 2]"	"List[]"	"List[1, 2]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'structural::finite_positive_drop' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }
FAILED
test structural::finite_rejected_select ... CASE	resumed Select rejects last finite item	resumed=(View { status: "OK", value: Some("False"), termination: "None", messages: [], prints: [] }, "List[List[]]", ["p: "])	queued=(View { status: "OK", value: Some("99"), termination: "None", messages: [], prints: [] }, "List[List[]]", [])
ROW	resumed Select rejects last finite item	"False"	"List[List[]]"	"99"	"List[List[]]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'structural::finite_rejected_select' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("False"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("99"), termination: "None", messages: [], prints: [] }
FAILED
test structural::fold_init_above_suspending_adapter ... CASE	Fold init above suspending Map	resumed=(View { status: "OK", value: Some("0"), termination: "None", messages: [], prints: [] }, "List[List[outer, 0]]", ["p: "])	queued=(View { status: "OK", value: Some("List[0, 1]"), termination: "None", messages: [], prints: [] }, "List[List[outer, 0], List[inner, 1], List[outer, 1]]", [])
ROW	Fold init above suspending Map	"0"	"List[List[outer, 0]]"	"List[0, 1]"	"List[List[outer, 0], List[inner, 1], List[outer, 1]]"	1	3	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'structural::fold_init_above_suspending_adapter' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:77:5:
assertion `left == right` failed: must reach every suspension
  left: ["p: "]
 right: ["p: ", "p: ", "p: "]
FAILED
test structural::last_take_child ... CASE	last permitted Take child	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "List[1]", ["p: "])	queued=(View { status: "OK", value: Some("List[1]"), termination: "None", messages: [], prints: [] }, "List[1]", [])
ROW	last permitted Take child	"1"	"List[1]"	"List[1]"	"List[1]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'structural::last_take_child' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:79:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[1]"), termination: "None", messages: [], prints: [] }
FAILED
test structural::nested_realization ... CASE	callback realizes a suspending lazy list	resumed=(View { status: "OK", value: Some("11"), termination: "None", messages: [], prints: [] }, "List[List[outer, 1], List[inner, 10]]", ["p: "])	queued=(View { status: "OK", value: Some("List[11, 12]"), termination: "None", messages: [], prints: [] }, "List[List[outer, 1], List[inner, 10], List[outer, 2], List[inner, 10]]", [])
ROW	callback realizes a suspending lazy list	"11"	"List[List[outer, 1], List[inner, 10]]"	"List[11, 12]"	"List[List[outer, 1], List[inner, 10], List[outer, 2], List[inner, 10]]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'structural::nested_realization' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:77:5:
assertion `left == right` failed: must reach every suspension
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED
test structural::one_callback_two_inputs ... CASE	one callback two inputs	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "List[1]", ["first: ", "second: "])	queued=(View { status: "OK", value: Some("List[2]"), termination: "None", messages: [], prints: [] }, "List[1]", [])
ROW	one callback two inputs	"2"	"List[1]"	"List[2]"	"List[1]"	2	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'structural::one_callback_two_inputs' panicked at crates/rum-stdlib/tests/lazy_pull_continuations/structural.rs:18:5:
assertion `left == right` failed
  left: View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[2]"), termination: "None", messages: [], prints: [] }
FAILED

failures:

failures:
    structural::alternating_take_drop
    structural::finite_positive_drop
    structural::finite_rejected_select
    structural::fold_init_above_suspending_adapter
    structural::last_take_child
    structural::nested_realization
    structural::one_callback_two_inputs

test result: FAILED. 0 passed; 7 failed; 0 ignored; 0 measured; 52 filtered out; finished in 0.06s

error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`
```

</details>


<details>
<summary>pre/depth-shallow.log</summary>


[pre/depth-shallow.log](../../../.temp/brief-563/pre/depth-shallow.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 9 tests
test depth::depth_cancel_shallow ... CASE	Depth-cancel-32	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "1", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "32", [])
ROW	Depth-cancel-32	"TerminatedEvaluation[\"Cancelled\"]"	"1"	"TerminatedEvaluation[\"Cancelled\"]"	"32"	1	1	false	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary"]

thread 'lazy-pull-2mib' panicked at crates/rum-stdlib/tests/lazy_pull_continuations/depth.rs:78:9:
assertion `left == right` failed
  left: "32"
 right: "1"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'depth::depth_cancel_shallow' panicked at crates/rum-stdlib/tests/lazy_pull_continuations/depth.rs:8:63:
called `Result::unwrap()` on an `Err` value: Any { .. }
FAILED
test depth::depth_drop_shallow ... CASE	Depth-drop-32	resumed=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "33", [])	queued=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "33", [])
ROW	Depth-drop-32	"32"	"33"	"32"	"33"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_inner_shallow ... CASE	Depth-inner-32	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "1", ["p: "])	queued=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", [])
ROW	Depth-inner-32	"1"	"1"	"List[32]"	"32"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazy-pull-2mib' panicked at crates/rum-stdlib/tests/lazy_pull_continuations/depth.rs:41:9:
assertion `left == right` failed: complete depth result
  left: View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }

thread 'depth::depth_inner_shallow' panicked at crates/rum-stdlib/tests/lazy_pull_continuations/depth.rs:8:63:
called `Result::unwrap()` on an `Err` value: Any { .. }
FAILED
test depth::depth_invalid_zero_shallow ... CASE	Depth-invalid_zero-32	resumed=(View { status: "OK", value: Some("Take"), termination: "None", messages: [], prints: [] }, "0", [])	queued=(View { status: "OK", value: Some("Take"), termination: "None", messages: [], prints: [] }, "0", [])
ROW	Depth-invalid_zero-32	"Take"	"0"	"Take"	"0"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_no_pull_shallow ... CASE	Depth-no_pull-32	resumed=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "0", [])	queued=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "0", [])
ROW	Depth-no_pull-32	"List[]"	"0"	"List[]"	"0"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_outer_shallow ... CASE	Depth-outer-32	resumed=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "32", ["p: "])	queued=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", [])
ROW	Depth-outer-32	"32"	"32"	"List[32]"	"32"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazy-pull-2mib' panicked at crates/rum-stdlib/tests/lazy_pull_continuations/depth.rs:41:9:
assertion `left == right` failed: complete depth result
  left: View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }

thread 'depth::depth_outer_shallow' panicked at crates/rum-stdlib/tests/lazy_pull_continuations/depth.rs:8:63:
called `Result::unwrap()` on an `Err` value: Any { .. }
FAILED
test depth::depth_pull_shallow ... CASE	Depth-pull-32	resumed=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "32", [])	queued=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "32", [])
ROW	Depth-pull-32	"32"	"32"	"32"	"32"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_select_shallow ... CASE	Depth-select-32	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "65", [])	queued=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "65", [])
ROW	Depth-select-32	"2"	"65"	"2"	"65"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_twice_shallow ... CASE	Depth-twice-32	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "1", ["p: "])	queued=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", [])
ROW	Depth-twice-32	"1"	"1"	"List[32]"	"32"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'lazy-pull-2mib' panicked at crates/rum-stdlib/tests/lazy_pull_continuations/depth.rs:40:9:
assertion `left == right` failed: all suspensions reached
  left: ["p: "]
 right: ["p: ", "p: "]

thread 'depth::depth_twice_shallow' panicked at crates/rum-stdlib/tests/lazy_pull_continuations/depth.rs:8:63:
called `Result::unwrap()` on an `Err` value: Any { .. }
FAILED

failures:

failures:
    depth::depth_cancel_shallow
    depth::depth_inner_shallow
    depth::depth_outer_shallow
    depth::depth_twice_shallow

test result: FAILED. 5 passed; 4 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.15s

error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`
```

</details>


<details>
<summary>pre/depth-contract.log</summary>


[pre/depth-contract.log](../../../.temp/brief-563/pre/depth-contract.log)


```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 47.07s
     Running unittests src/lib.rs (.temp/brief-563/target-tests/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 4 tests
test tests::association_depth_contract::lazy_depth_contract::lazy_ownership_is_not_self_reaching ... 
thread 'tests::association_depth_contract::lazy_depth_contract::lazy_ownership_is_not_self_reaching' panicked at crates/rum-stdlib/src/tests/association_depth_contract/../lazy_depth_contract.rs:46:5:
self-reaching lazy ownership: ["rum_stdlib::lazy::PullStepper\tcrates/rum-stdlib/src/lazy.rs\t57\trum_stdlib::lazy::PullStepper.Drop.inner/std::boxed::Box[Own]/arg0 -> rum_stdlib::lazy::PullStepper"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test tests::association_depth_contract::lazy_depth_contract::lazy_recursive_helper_falsifier ... recursive helper falsifier: [RecursiveEdge { path: "lazy/machine.rs", caller: "drive_pull", callee: "drive_pull", count: 1 }]
ok
test tests::association_depth_contract::lazy_depth_contract::lazy_self_reaching_falsifier ... self-reaching falsifier rejected: rum_stdlib::lazy::state::PullMachine	lazy/state.rs	1	recursive owned child
ok
test tests::association_depth_contract::lazy_depth_contract::lazy_source_has_no_recursive_edges ... 
thread 'tests::association_depth_contract::lazy_depth_contract::lazy_source_has_no_recursive_edges' panicked at crates/rum-stdlib/src/tests/association_depth_contract/../lazy_depth_contract.rs:28:5:
lazy recursion: [RecursiveEdge { path: "/home/remi/dev/rum/crates/rum-stdlib/src/lazy.rs", caller: "stepper_from_descriptor", callee: "stepper_from_descriptor", count: 5 }]
FAILED

failures:

failures:
    tests::association_depth_contract::lazy_depth_contract::lazy_ownership_is_not_self_reaching
    tests::association_depth_contract::lazy_depth_contract::lazy_source_has_no_recursive_edges

test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 1078 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```

</details>


Each deep test ran alone at n=2,000 on the 2 MiB thread, which owns Session teardown. All nine incoming runs aborted; all had Cargo exit 101 with SIGABRT in the child. The immutable sources and outputs remain under `pre/` and `pre/test-source/`.


[pre/depth-no_pull.log](../../../.temp/brief-563/pre/depth-no_pull.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 1 test
test depth::depth_no_pull_deep ... 
thread 'lazy-pull-2mib' has overflowed its stack
fatal runtime error: stack overflow
error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`

Caused by:
  process didn't exit successfully: `/home/remi/dev/rum/.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd --exact 'depth::depth_no_pull_deep' --nocapture` (signal: 6, SIGABRT: process abort signal)
```

[pre/depth-invalid_zero.log](../../../.temp/brief-563/pre/depth-invalid_zero.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 1 test
test depth::depth_invalid_zero_deep ... 
thread 'lazy-pull-2mib' has overflowed its stack
fatal runtime error: stack overflow
error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`

Caused by:
  process didn't exit successfully: `/home/remi/dev/rum/.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd --exact 'depth::depth_invalid_zero_deep' --nocapture` (signal: 6, SIGABRT: process abort signal)
```

[pre/depth-pull.log](../../../.temp/brief-563/pre/depth-pull.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 1 test
test depth::depth_pull_deep ... 
thread 'lazy-pull-2mib' has overflowed its stack
fatal runtime error: stack overflow
error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`

Caused by:
  process didn't exit successfully: `/home/remi/dev/rum/.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd --exact 'depth::depth_pull_deep' --nocapture` (signal: 6, SIGABRT: process abort signal)
```

[pre/depth-inner.log](../../../.temp/brief-563/pre/depth-inner.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 1 test
test depth::depth_inner_deep ... 
thread 'lazy-pull-2mib' has overflowed its stack
fatal runtime error: stack overflow
error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`

Caused by:
  process didn't exit successfully: `/home/remi/dev/rum/.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd --exact 'depth::depth_inner_deep' --nocapture` (signal: 6, SIGABRT: process abort signal)
```

[pre/depth-outer.log](../../../.temp/brief-563/pre/depth-outer.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 1 test
test depth::depth_outer_deep ... 
thread 'lazy-pull-2mib' has overflowed its stack
fatal runtime error: stack overflow
error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`

Caused by:
  process didn't exit successfully: `/home/remi/dev/rum/.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd --exact 'depth::depth_outer_deep' --nocapture` (signal: 6, SIGABRT: process abort signal)
```

[pre/depth-twice.log](../../../.temp/brief-563/pre/depth-twice.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 1 test
test depth::depth_twice_deep ... 
thread 'lazy-pull-2mib' has overflowed its stack
fatal runtime error: stack overflow
error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`

Caused by:
  process didn't exit successfully: `/home/remi/dev/rum/.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd --exact 'depth::depth_twice_deep' --nocapture` (signal: 6, SIGABRT: process abort signal)
```

[pre/depth-select.log](../../../.temp/brief-563/pre/depth-select.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 1 test
test depth::depth_select_deep ... 
thread 'lazy-pull-2mib' has overflowed its stack
fatal runtime error: stack overflow
error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`

Caused by:
  process didn't exit successfully: `/home/remi/dev/rum/.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd --exact 'depth::depth_select_deep' --nocapture` (signal: 6, SIGABRT: process abort signal)
```

[pre/depth-drop.log](../../../.temp/brief-563/pre/depth-drop.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 1 test
test depth::depth_drop_deep ... 
thread 'lazy-pull-2mib' has overflowed its stack
fatal runtime error: stack overflow
error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`

Caused by:
  process didn't exit successfully: `/home/remi/dev/rum/.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd --exact 'depth::depth_drop_deep' --nocapture` (signal: 6, SIGABRT: process abort signal)
```

[pre/depth-cancel.log](../../../.temp/brief-563/pre/depth-cancel.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 1 test
test depth::depth_cancel_deep ... 
thread 'lazy-pull-2mib' has overflowed its stack
fatal runtime error: stack overflow
error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`

Caused by:
  process didn't exit successfully: `/home/remi/dev/rum/.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd --exact 'depth::depth_cancel_deep' --nocapture` (signal: 6, SIGABRT: process abort signal)
```

The recursion scanner’s incoming red reports five self-calls in stepper_from_descriptor. Its same-file direct-call analysis does **not** resolve the inner.next receiver, so no claim is made that it separately reported that recursion. The ownership check independently rejected PullStepper. Both falsifiers were watched reject their synthetic defect. The initial temporary ../ module mount failed discovery resolution and was replaced by a conventional sibling module; no unresolved-storage row was admitted.


#### Post-fix values and final candidate checks


The first green implementation emitted the detailed actual value/log/prompt records below. The final candidate reruns the unchanged assertions over every one of the same 59 cases and is independently green in the final focused and full standard-library commands. No new lazy case is ignored, lowered in depth, or allowed to pass via StackLimitExceeded.


<details>
<summary>Detailed post-fix observations: implementation/attempt-2.log</summary>


[implementation/attempt-2.log](../../../.temp/brief-563/implementation/attempt-2.log)


```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.20s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 49 tests
test cancel_first_suspension ... CASE	Lazy-cancel-at-1	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1, 2]", [])
ROW	Lazy-cancel-at-1	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2]"	1	1	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary"]
ok
test cancel_second_suspension ... CASE	Lazy-cancel-at-2	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2, 3, 4]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2, 3, 4]", [])
ROW	Lazy-cancel-at-2	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2, 3, 4]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2, 3, 4]"	2	2	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary", "boundary"]	["boundary", "boundary"]
ok
test depth::depth_cancel_shallow ... CASE	Depth-cancel-32	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "1", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "1", [])
ROW	Depth-cancel-32	"TerminatedEvaluation[\"Cancelled\"]"	"1"	"TerminatedEvaluation[\"Cancelled\"]"	"1"	1	1	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary"]
ok
test depth::depth_drop_shallow ... CASE	Depth-drop-32	resumed=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "33", [])	queued=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "33", [])
ROW	Depth-drop-32	"32"	"33"	"32"	"33"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_inner_shallow ... CASE	Depth-inner-32	resumed=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", ["p: "])	queued=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", [])
ROW	Depth-inner-32	"List[32]"	"32"	"List[32]"	"32"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_invalid_zero_shallow ... CASE	Depth-invalid_zero-32	resumed=(View { status: "OK", value: Some("Take"), termination: "None", messages: [], prints: [] }, "0", [])	queued=(View { status: "OK", value: Some("Take"), termination: "None", messages: [], prints: [] }, "0", [])
ROW	Depth-invalid_zero-32	"Take"	"0"	"Take"	"0"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_no_pull_shallow ... CASE	Depth-no_pull-32	resumed=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "0", [])	queued=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "0", [])
ROW	Depth-no_pull-32	"List[]"	"0"	"List[]"	"0"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_outer_shallow ... CASE	Depth-outer-32	resumed=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", ["p: "])	queued=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", [])
ROW	Depth-outer-32	"List[32]"	"32"	"List[32]"	"32"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_pull_shallow ... CASE	Depth-pull-32	resumed=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "32", [])	queued=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "32", [])
ROW	Depth-pull-32	"32"	"32"	"32"	"32"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_select_shallow ... CASE	Depth-select-32	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "65", [])	queued=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "65", [])
ROW	Depth-select-32	"2"	"65"	"2"	"65"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_twice_shallow ... CASE	Depth-twice-32	resumed=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", [])
ROW	Depth-twice-32	"List[32]"	"32"	"List[32]"	"32"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazy_adapter_unwind_twice ... CASE	Lazy-adapter-unwind-twice	resumed=(View { status: "OK", value: Some("List[List[1], List[2], List[3]]"), termination: "None", messages: [], prints: [] }, "List[List[inner, 1], List[outer, 1], List[inner, 2], List[outer, 2], List[inner, 3], List[outer, 3]]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[List[1], List[2], List[3]]"), termination: "None", messages: [], prints: [] }, "List[List[inner, 1], List[outer, 1], List[inner, 2], List[outer, 2], List[inner, 3], List[outer, 3]]", [])
ROW	Lazy-adapter-unwind-twice	"List[List[1], List[2], List[3]]"	"List[List[inner, 1], List[outer, 1], List[inner, 2], List[outer, 2], List[inner, 3], List[outer, 3]]"	"List[List[1], List[2], List[3]]"	"List[List[inner, 1], List[outer, 1], List[inner, 2], List[outer, 2], List[inner, 3], List[outer, 3]]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazy_infinite_after_resume ... CASE	Lazy-infinite-after-resume	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationWorkLimit\"]"), termination: "Some(IterationWorkLimit)", messages: [RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }], prints: [] }, "List[1, 2, 3, 4]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationWorkLimit\"]"), termination: "Some(IterationWorkLimit)", messages: [RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }], prints: [] }, "List[1, 2, 3, 4]", [])
ROW	Lazy-infinite-after-resume	"TerminatedEvaluation[\"IterationWorkLimit\"]"	"List[1, 2, 3, 4]"	"TerminatedEvaluation[\"IterationWorkLimit\"]"	"List[1, 2, 3, 4]"	1	1	true	"OK"	"OK"	"Some(IterationWorkLimit)"	"Some(IterationWorkLimit)"	[RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }]	[RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }]	[]	[]
ok
test lazy_rerederive ... CASE	Lazy-rerederive	resumed=(View { status: "OK", value: Some("List[List[1, \"two\", 3], List[1, \"two\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 1, 2, 3]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[List[1, \"two\", 3], List[1, \"two\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 1, 2, 3]", [])
ROW	Lazy-rerederive	"List[List[1, \"two\", 3], List[1, \"two\", 3]]"	"List[1, 2, 3, 1, 2, 3]"	"List[List[1, \"two\", 3], List[1, \"two\", 3]]"	"List[1, 2, 3, 1, 2, 3]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazy_source_budget_exact ... CASE	Lazy-source-budget-exact	resumed=(View { status: "OK", value: Some("List[3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", ["p: "])	queued=(View { status: "OK", value: Some("List[3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])
ROW	Lazy-source-budget-exact	"List[3, 4]"	"List[1, 2, 3, 4]"	"List[3, 4]"	"List[1, 2, 3, 4]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazydrop_inner ... CASE	LazyDrop-inner	resumed=(View { status: "OK", value: Some("List[3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", ["p: "])	queued=(View { status: "OK", value: Some("List[3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])
ROW	LazyDrop-inner	"List[3, 4]"	"List[1, 2, 3, 4]"	"List[3, 4]"	"List[1, 2, 3, 4]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazydrop_twice ... CASE	LazyDrop-twice	resumed=(View { status: "OK", value: Some("5"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("5"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	LazyDrop-twice	"5"	"List[1, 2, 3, 4, 5]"	"5"	"List[1, 2, 3, 4, 5]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazyfirst_consumer ... CASE	LazyFirst-consumer	resumed=(View { status: "OK", value: Some("List[\"two\"]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[\"two\"]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])
ROW	LazyFirst-consumer	"List[\"two\"]"	"List[1, 2]"	"List[\"two\"]"	"List[1, 2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazyfirst_empty_default ... CASE	LazyFirst-empty-default	resumed=(View { status: "OK", value: Some("99"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("99"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyFirst-empty-default	"99"	"List[1, 2, 3]"	"99"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazyfirst_empty_message ... CASE	LazyFirst-empty-message	resumed=(View { status: "OK", value: Some("First[List[]]"), termination: "None", messages: [RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }, RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("First[List[]]"), termination: "None", messages: [RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }, RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyFirst-empty-message	"First[List[]]"	"List[1, 2, 3]"	"First[List[]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }, RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }]	[RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }, RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }]	[]	[]
ok
test lazyfoldlist ... CASE	LazyFoldList	resumed=(View { status: "OK", value: Some("List[0, 1, 3, 6]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[0, 1, 3, 6]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyFoldList	"List[0, 1, 3, 6]"	"List[1, 2, 3]"	"List[0, 1, 3, 6]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazyfoldlist_inner ... CASE	LazyFoldList-inner	resumed=(View { status: "OK", value: Some("List[List[], List[1], List[1, \"two\"], List[1, \"two\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[], List[1], List[1, \"two\"], List[1, \"two\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyFoldList-inner	"List[List[], List[1], List[1, \"two\"], List[1, \"two\", 3]]"	"List[1, 2, 3]"	"List[List[], List[1], List[1, \"two\"], List[1, \"two\", 3]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazyfoldlist_twice ... CASE	LazyFoldList-twice	resumed=(View { status: "OK", value: Some("List[0, 1, 3, 6, 10, 15]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[0, 1, 3, 6, 10, 15]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	LazyFoldList-twice	"List[0, 1, 3, 6, 10, 15]"	"List[1, 2, 3, 4, 5]"	"List[0, 1, 3, 6, 10, 15]"	"List[1, 2, 3, 4, 5]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazymap ... CASE	LazyMap	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyMap	"List[1, \"two\", 3]"	"List[1, 2, 3]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazymap_error_in_callback ... CASE	LazyMap-error-in-callback	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: [] }, "List[1, 2]", [])
ROW	LazyMap-error-in-callback	"<error>"	"List[1, 2]"	"<error>"	"List[1, 2]"	1	1	true	"Other { message: \"uncaught throw: 9 with tag None\" }"	"Other { message: \"uncaught throw: 9 with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }]	[]	[]
ok
test lazymap_error_next ... CASE	LazyMap-error-next	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])
ROW	LazyMap-error-next	"<error>"	"List[1, 2, 3]"	"<error>"	"List[1, 2, 3]"	1	1	true	"Other { message: \"uncaught throw: 9 with tag None\" }"	"Other { message: \"uncaught throw: 9 with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }]	["before", "after"]	["before", "after"]
ok
test lazymap_inner ... CASE	LazyMap-inner	resumed=(View { status: "OK", value: Some("List[List[1], List[\"two\"], List[3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[1], List[\"two\"], List[3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyMap-inner	"List[List[1], List[\"two\"], List[3]]"	"List[1, 2, 3]"	"List[List[1], List[\"two\"], List[3]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazymap_twice ... CASE	LazyMap-twice	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3, \"four\", 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3, \"four\", 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	LazyMap-twice	"List[1, \"two\", 3, \"four\", 5]"	"List[1, 2, 3, 4, 5]"	"List[1, \"two\", 3, \"four\", 5]"	"List[1, 2, 3, 4, 5]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazynestlist ... CASE	LazyNestList	resumed=(View { status: "OK", value: Some("List[1, 2, 3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])
ROW	LazyNestList	"List[1, 2, 3, 4]"	"List[1, 2, 3, 4]"	"List[1, 2, 3, 4]"	"List[1, 2, 3, 4]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazynestlist_lookahead_first ... CASE	LazyNestList-lookahead-first	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "List[1]", ["p: "])	queued=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "List[1]", [])
ROW	LazyNestList-lookahead-first	"1"	"List[1]"	"1"	"List[1]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazynestlist_twice ... CASE	LazyNestList-twice	resumed=(View { status: "OK", value: Some("List[1, 2, 3, 4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3, 4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	LazyNestList-twice	"List[1, 2, 3, 4, 5]"	"List[1, 2, 3, 4, 5]"	"List[1, 2, 3, 4, 5]"	"List[1, 2, 3, 4, 5]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazypart_consumer ... CASE	LazyPart-consumer	resumed=(View { status: "OK", value: Some("List[\"two\"]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[\"two\"]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])
ROW	LazyPart-consumer	"List[\"two\"]"	"List[1, 2]"	"List[\"two\"]"	"List[1, 2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazypermutations_cursor ... CASE	LazyPermutations-cursor	resumed=(View { status: "OK", value: Some("List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]"), termination: "None", messages: [], prints: [] }, "List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]"), termination: "None", messages: [], prints: [] }, "List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]", [])
ROW	LazyPermutations-cursor	"List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]"	"List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]"	"List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]"	"List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazyselect ... CASE	LazySelect	resumed=(View { status: "OK", value: Some("List[1, 2, 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazySelect	"List[1, 2, 3]"	"List[1, 2, 3]"	"List[1, 2, 3]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazyselect_inner ... CASE	LazySelect-inner	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazySelect-inner	"List[1, \"two\", 3]"	"List[1, 2, 3]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazyselect_never_after_resume ... CASE	LazySelect-never-after-resume	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationWorkLimit\"]"), termination: "Some(IterationWorkLimit)", messages: [RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }], prints: [] }, "List[1, 2, 3, 4]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationWorkLimit\"]"), termination: "Some(IterationWorkLimit)", messages: [RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }], prints: [] }, "List[1, 2, 3, 4]", [])
ROW	LazySelect-never-after-resume	"TerminatedEvaluation[\"IterationWorkLimit\"]"	"List[1, 2, 3, 4]"	"TerminatedEvaluation[\"IterationWorkLimit\"]"	"List[1, 2, 3, 4]"	1	1	true	"OK"	"OK"	"Some(IterationWorkLimit)"	"Some(IterationWorkLimit)"	[RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }]	[RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }]	[]	[]
ok
test lazyselect_reject_resumed_candidate ... CASE	LazySelect-reject-resumed-candidate	resumed=(View { status: "OK", value: Some("List[1, 3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, 3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])
ROW	LazySelect-reject-resumed-candidate	"List[1, 3, 4]"	"List[1, 2, 3, 4]"	"List[1, 3, 4]"	"List[1, 2, 3, 4]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazyselect_twice ... CASE	LazySelect-twice	resumed=(View { status: "OK", value: Some("List[1, 2, 3, 4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3, 4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	LazySelect-twice	"List[1, 2, 3, 4, 5]"	"List[1, 2, 3, 4, 5]"	"List[1, 2, 3, 4, 5]"	"List[1, 2, 3, 4, 5]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazysubsets_cursor ... CASE	LazySubsets-cursor	resumed=(View { status: "OK", value: Some("List[List[], List[1], List[2]]"), termination: "None", messages: [], prints: [] }, "List[List[], List[1], List[2]]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[], List[1], List[2]]"), termination: "None", messages: [], prints: [] }, "List[List[], List[1], List[2]]", [])
ROW	LazySubsets-cursor	"List[List[], List[1], List[2]]"	"List[List[], List[1], List[2]]"	"List[List[], List[1], List[2]]"	"List[List[], List[1], List[2]]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazytable_body ... CASE	LazyTable-body	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-body	"List[1, \"two\", 3]"	"List[1, 2, 3]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazytake_consumer ... CASE	LazyTake-consumer	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTake-consumer	"List[1, \"two\", 3]"	"List[1, 2, 3]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test lazytuples_cursor ... CASE	LazyTuples-cursor	resumed=(View { status: "OK", value: Some("List[List[1, 3], List[1, 4], List[2, 3]]"), termination: "None", messages: [], prints: [] }, "List[List[1, 3], List[1, 4], List[2, 3]]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[1, 3], List[1, 4], List[2, 3]]"), termination: "None", messages: [], prints: [] }, "List[List[1, 3], List[1, 4], List[2, 3]]", [])
ROW	LazyTuples-cursor	"List[List[1, 3], List[1, 4], List[2, 3]]"	"List[List[1, 3], List[1, 4], List[2, 3]]"	"List[List[1, 3], List[1, 4], List[2, 3]]"	"List[List[1, 3], List[1, 4], List[2, 3]]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test structural::alternating_take_drop ... CASE	alternating Take/Drop skip and yield	resumed=(View { status: "OK", value: Some("List[4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	alternating Take/Drop skip and yield	"List[4, 5]"	"List[1, 2, 3, 4, 5]"	"List[4, 5]"	"List[1, 2, 3, 4, 5]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test structural::finite_positive_drop ... CASE	positive Drop finite exhaustion	resumed=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])
ROW	positive Drop finite exhaustion	"List[]"	"List[1, 2]"	"List[]"	"List[1, 2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test structural::finite_rejected_select ... CASE	resumed Select rejects last finite item	resumed=(View { status: "OK", value: Some("99"), termination: "None", messages: [], prints: [] }, "List[List[]]", ["p: "])	queued=(View { status: "OK", value: Some("99"), termination: "None", messages: [], prints: [] }, "List[List[]]", [])
ROW	resumed Select rejects last finite item	"99"	"List[List[]]"	"99"	"List[List[]]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test structural::fold_init_above_suspending_adapter ... CASE	Fold init above suspending Map	resumed=(View { status: "OK", value: Some("List[0, 1]"), termination: "None", messages: [], prints: [] }, "List[List[outer, 0], List[inner, 1], List[outer, 1]]", ["p: ", "p: ", "p: "])	queued=(View { status: "OK", value: Some("List[0, 1]"), termination: "None", messages: [], prints: [] }, "List[List[outer, 0], List[inner, 1], List[outer, 1]]", [])
ROW	Fold init above suspending Map	"List[0, 1]"	"List[List[outer, 0], List[inner, 1], List[outer, 1]]"	"List[0, 1]"	"List[List[outer, 0], List[inner, 1], List[outer, 1]]"	3	3	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test structural::last_take_child ... CASE	last permitted Take child	resumed=(View { status: "OK", value: Some("List[1]"), termination: "None", messages: [], prints: [] }, "List[1]", ["p: "])	queued=(View { status: "OK", value: Some("List[1]"), termination: "None", messages: [], prints: [] }, "List[1]", [])
ROW	last permitted Take child	"List[1]"	"List[1]"	"List[1]"	"List[1]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test structural::nested_realization ... CASE	callback realizes a suspending lazy list	resumed=(View { status: "OK", value: Some("List[11, 12]"), termination: "None", messages: [], prints: [] }, "List[List[outer, 1], List[inner, 10], List[outer, 2], List[inner, 10]]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[11, 12]"), termination: "None", messages: [], prints: [] }, "List[List[outer, 1], List[inner, 10], List[outer, 2], List[inner, 10]]", [])
ROW	callback realizes a suspending lazy list	"List[11, 12]"	"List[List[outer, 1], List[inner, 10], List[outer, 2], List[inner, 10]]"	"List[11, 12]"	"List[List[outer, 1], List[inner, 10], List[outer, 2], List[inner, 10]]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test structural::one_callback_two_inputs ... CASE	one callback two inputs	resumed=(View { status: "OK", value: Some("List[2]"), termination: "None", messages: [], prints: [] }, "List[1]", ["first: ", "second: "])	queued=(View { status: "OK", value: Some("List[2]"), termination: "None", messages: [], prints: [] }, "List[1]", [])
ROW	one callback two inputs	"List[2]"	"List[1]"	"List[2]"	"List[1]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok

test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.58s
```

</details>


<details>
<summary>Detailed post-fix observations: implementation/attempt-3-depth.log</summary>


[implementation/attempt-3-depth.log](../../../.temp/brief-563/implementation/attempt-3-depth.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 19 tests
test depth::depth_cancel_deep ... CASE	Depth-cancel-2000	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "1", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "1", [])
ROW	Depth-cancel-2000	"TerminatedEvaluation[\"Cancelled\"]"	"1"	"TerminatedEvaluation[\"Cancelled\"]"	"1"	1	1	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary"]
ok
test depth::depth_cancel_shallow ... CASE	Depth-cancel-32	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "1", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "1", [])
ROW	Depth-cancel-32	"TerminatedEvaluation[\"Cancelled\"]"	"1"	"TerminatedEvaluation[\"Cancelled\"]"	"1"	1	1	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary"]
ok
test depth::depth_drop_deep ... CASE	Depth-drop-2000	resumed=(View { status: "OK", value: Some("2000"), termination: "None", messages: [], prints: [] }, "2001", [])	queued=(View { status: "OK", value: Some("2000"), termination: "None", messages: [], prints: [] }, "2001", [])
ROW	Depth-drop-2000	"2000"	"2001"	"2000"	"2001"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_drop_shallow ... CASE	Depth-drop-32	resumed=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "33", [])	queued=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "33", [])
ROW	Depth-drop-32	"32"	"33"	"32"	"33"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_inner_deep ... CASE	Depth-inner-2000	resumed=(View { status: "OK", value: Some("List[2000]"), termination: "None", messages: [], prints: [] }, "2000", ["p: "])	queued=(View { status: "OK", value: Some("List[2000]"), termination: "None", messages: [], prints: [] }, "2000", [])
ROW	Depth-inner-2000	"List[2000]"	"2000"	"List[2000]"	"2000"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_inner_shallow ... CASE	Depth-inner-32	resumed=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", ["p: "])	queued=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", [])
ROW	Depth-inner-32	"List[32]"	"32"	"List[32]"	"32"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_invalid_zero_deep ... CASE	Depth-invalid_zero-2000	resumed=(View { status: "OK", value: Some("Take"), termination: "None", messages: [], prints: [] }, "0", [])	queued=(View { status: "OK", value: Some("Take"), termination: "None", messages: [], prints: [] }, "0", [])
ROW	Depth-invalid_zero-2000	"Take"	"0"	"Take"	"0"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_invalid_zero_shallow ... CASE	Depth-invalid_zero-32	resumed=(View { status: "OK", value: Some("Take"), termination: "None", messages: [], prints: [] }, "0", [])	queued=(View { status: "OK", value: Some("Take"), termination: "None", messages: [], prints: [] }, "0", [])
ROW	Depth-invalid_zero-32	"Take"	"0"	"Take"	"0"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_no_pull_deep ... CASE	Depth-no_pull-2000	resumed=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "0", [])	queued=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "0", [])
ROW	Depth-no_pull-2000	"List[]"	"0"	"List[]"	"0"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_no_pull_shallow ... CASE	Depth-no_pull-32	resumed=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "0", [])	queued=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "0", [])
ROW	Depth-no_pull-32	"List[]"	"0"	"List[]"	"0"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_outer_deep ... CASE	Depth-outer-2000	resumed=(View { status: "OK", value: Some("List[2000]"), termination: "None", messages: [], prints: [] }, "2000", ["p: "])	queued=(View { status: "OK", value: Some("List[2000]"), termination: "None", messages: [], prints: [] }, "2000", [])
ROW	Depth-outer-2000	"List[2000]"	"2000"	"List[2000]"	"2000"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_outer_shallow ... CASE	Depth-outer-32	resumed=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", ["p: "])	queued=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", [])
ROW	Depth-outer-32	"List[32]"	"32"	"List[32]"	"32"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_pull_deep ... CASE	Depth-pull-2000	resumed=(View { status: "OK", value: Some("2000"), termination: "None", messages: [], prints: [] }, "2000", [])	queued=(View { status: "OK", value: Some("2000"), termination: "None", messages: [], prints: [] }, "2000", [])
ROW	Depth-pull-2000	"2000"	"2000"	"2000"	"2000"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_pull_shallow ... CASE	Depth-pull-32	resumed=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "32", [])	queued=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "32", [])
ROW	Depth-pull-32	"32"	"32"	"32"	"32"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_select_deep ... CASE	Depth-select-2000	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "4001", [])	queued=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "4001", [])
ROW	Depth-select-2000	"2"	"4001"	"2"	"4001"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_select_shallow ... CASE	Depth-select-32	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "65", [])	queued=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "65", [])
ROW	Depth-select-32	"2"	"65"	"2"	"65"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_twice_deep ... CASE	Depth-twice-2000	resumed=(View { status: "OK", value: Some("List[2000]"), termination: "None", messages: [], prints: [] }, "2000", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[2000]"), termination: "None", messages: [], prints: [] }, "2000", [])
ROW	Depth-twice-2000	"List[2000]"	"2000"	"List[2000]"	"2000"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::depth_twice_shallow ... CASE	Depth-twice-32	resumed=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", [])
ROW	Depth-twice-32	"List[32]"	"32"	"List[32]"	"32"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test depth::descriptor_control_deep ... DESCRIPTOR CONTROL depth=2000 stack=2097152 result=0 teardown=complete
ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 40 filtered out; finished in 23.36s
```

</details>


<details>
<summary>Detailed post-fix observations: implementation/attempt-5-private.log</summary>


[implementation/attempt-5-private.log](../../../.temp/brief-563/implementation/attempt-5-private.log)


```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 39.67s
     Running unittests src/lib.rs (.temp/brief-563/target-tests/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 4 tests
test lazy::tests::lazy_flat_abandonment_depth ... flat abandonment depth=2000 complete
ok
test lazy::tests::lazy_flat_all_variant_roots_depth ... all source/stage/cursor roots depth=2000 complete
ok
test lazy::tests::lazy_flat_clone_and_working_copy_depth ... flat clone and working copy depth=2000 complete
ok
test lazy::tests::lazy_flat_last_owner_drop_depth ... last-owner flat drop depth=2000 complete
ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1082 filtered out; finished in 0.05s
```

</details>


<details>
<summary>Detailed post-fix observations: implementation/attempt-6-lazy.log</summary>


[implementation/attempt-6-lazy.log](../../../.temp/brief-563/implementation/attempt-6-lazy.log)


```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.36s
     Running unittests src/lib.rs (.temp/brief-563/target-tests/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 45 tests
test lazy::tests::lazy_flat_abandonment_depth ... flat abandonment depth=2000 complete
ok
test lazy::tests::lazy_flat_all_variant_roots_depth ... all source/stage/cursor roots depth=2000 complete
ok
test lazy::tests::lazy_flat_clone_and_working_copy_depth ... flat clone and working copy depth=2000 complete
ok
test lazy::tests::lazy_flat_last_owner_drop_depth ... last-owner flat drop depth=2000 complete
ok
test tests::association_depth_contract::lazy_depth_contract::lazy_ownership_is_not_self_reaching ... ok
test tests::association_depth_contract::lazy_depth_contract::lazy_recursive_helper_falsifier ... recursive helper falsifier: [RecursiveEdge { path: "lazy/machine.rs", caller: "drive_pull", callee: "drive_pull", count: 1 }]
ok
test tests::association_depth_contract::lazy_depth_contract::lazy_self_reaching_falsifier - should panic ... 
thread 'tests::association_depth_contract::lazy_depth_contract::lazy_self_reaching_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/lazy_depth_contract.rs:76:5:
self-reaching lazy ownership: ["rum_stdlib::lazy::state::PullMachine\tlazy/state.rs\t1\trecursive owned child"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
ok
test tests::association_depth_contract::lazy_depth_contract::lazy_source_has_no_recursive_edges ... ok
test tests::holding_and_control_part5::confirm_info_is_lazy_and_explicit_tags_are_dynamically_caught ... ok
test tests::holding_and_control_part5::iterator_lhs_normalization_preserves_lazy_and_container_lvalue_controls ... ok
test tests::lazy::bounded_realizers_pull_lazy_ranges ... ok
test tests::lazy::finite_lazy_transformer_compositions_realize_to_lists ... ok
test tests::lazy::huge_lazy_combinatorial_spaces_fail_closed_at_iteration_ceiling ... ok
test tests::lazy::lazy_combinatorial_builders_reject_invalid_inputs_without_partial_values ... ok
test tests::lazy::lazy_drop_and_rest_keep_the_tail_lazy ... ok
test tests::lazy::lazy_fold_list_emits_init_then_running_accumulators ... ok
test tests::lazy::lazy_list_is_opaque_structural_and_inert ... ok
test tests::lazy::lazy_nest_and_fold_nested_errors_propagate ... ok
test tests::lazy::lazy_nest_list_builds_an_opaque_doubling_stream ... ok
test tests::lazy::lazy_permutations_large_take_is_lazy_cheap_and_opaque ... ok
test tests::lazy::lazy_permutations_matches_eager_canonical_order_and_edges ... ok
test tests::lazy::lazy_pull_cancellation_does_not_stale_the_same_value_for_a_fresh_request ... ok
test tests::lazy::lazy_range_builds_canonical_infinite_descriptors ... ok
test tests::lazy::lazy_range_rejects_invalid_arguments_without_partial_values ... ok
test tests::lazy::lazy_select_filters_under_bounded_realization ... ok
test tests::lazy::lazy_subsets_large_take_is_lazy_and_opaque ... ok
test tests::lazy::lazy_subsets_matches_eager_order_and_edges ... ok
test tests::lazy::lazy_table_holds_its_body_and_binds_the_iterator_lexically ... ok
test tests::lazy::lazy_table_realizes_arithmetic_iterator_forms ... ok
test tests::lazy::lazy_table_rejects_invalid_specs_without_partial_values ... ok
test tests::lazy::lazy_table_result_is_opaque_and_bounded_take_is_lazy ... ok
test tests::lazy::lazy_transformers_build_canonical_opaque_descriptors ... ok
test tests::lazy::lazy_transformers_reject_invalid_inputs_without_partial_values ... ok
test tests::lazy::lazy_tuples_matches_eager_order_and_edges ... ok
test tests::lazy::nested_lazy_map_and_select_eval_errors_propagate ... ok
test tests::lazy::never_matching_lazy_select_fails_closed_at_source_pull_ceiling ... ok
test tests::lazy::new_lazy_builders_reject_invalid_arity_and_fold_input ... ok
test tests::lazy::normal_on_infinite_lazy_nest_list_fails_closed ... ok
test tests::lazy::normal_on_infinite_lazy_range_fails_closed_at_iteration_ceiling ... ok
test tests::lazy::other_lazy_part_specs_follow_the_existing_part_path ... ok
test tests::lazy::positive_integer_part_redispatches_through_lazy_realizers ... ok
test tests::lazy::snapshot_roundtrip_rederives_composed_lazy_descriptor ... ok
test tests::lazy::snapshot_roundtrip_rederives_lazy_combinatorial_descriptors ... ok
test tests::lazy::snapshot_roundtrip_rederives_lazy_nest_and_fold_descriptors ... ok
test tests::lazy::snapshot_roundtrip_rederives_lazy_table_descriptor ... ok

test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 1041 filtered out; finished in 0.16s
```

</details>


[final2/lazy-continuations.log](../../../.temp/brief-563/final2/lazy-continuations.log)


```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (.temp/brief-563/target-tests/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 59 tests
test cancel_first_suspension ... ok
test cancel_second_suspension ... ok
test depth::depth_cancel_deep ... ok
test depth::depth_cancel_shallow ... ok
test depth::depth_drop_deep ... ok
test depth::depth_drop_shallow ... ok
test depth::depth_inner_deep ... ok
test depth::depth_inner_shallow ... ok
test depth::depth_invalid_zero_deep ... ok
test depth::depth_invalid_zero_shallow ... ok
test depth::depth_no_pull_deep ... ok
test depth::depth_no_pull_shallow ... ok
test depth::depth_outer_deep ... ok
test depth::depth_outer_shallow ... ok
test depth::depth_pull_deep ... ok
test depth::depth_pull_shallow ... ok
test depth::depth_select_deep ... ok
test depth::depth_select_shallow ... ok
test depth::depth_twice_deep ... ok
test depth::depth_twice_shallow ... ok
test depth::descriptor_control_deep ... ok
test lazy_adapter_unwind_twice ... ok
test lazy_infinite_after_resume ... ok
test lazy_rerederive ... ok
test lazy_source_budget_exact ... ok
test lazydrop_inner ... ok
test lazydrop_twice ... ok
test lazyfirst_consumer ... ok
test lazyfirst_empty_default ... ok
test lazyfirst_empty_message ... ok
test lazyfoldlist ... ok
test lazyfoldlist_inner ... ok
test lazyfoldlist_twice ... ok
test lazymap ... ok
test lazymap_error_in_callback ... ok
test lazymap_error_next ... ok
test lazymap_inner ... ok
test lazymap_twice ... ok
test lazynestlist ... ok
test lazynestlist_lookahead_first ... ok
test lazynestlist_twice ... ok
test lazypart_consumer ... ok
test lazypermutations_cursor ... ok
test lazyselect ... ok
test lazyselect_inner ... ok
test lazyselect_never_after_resume ... ok
test lazyselect_reject_resumed_candidate ... ok
test lazyselect_twice ... ok
test lazysubsets_cursor ... ok
test lazytable_body ... ok
test lazytake_consumer ... ok
test lazytuples_cursor ... ok
test structural::alternating_take_drop ... ok
test structural::finite_positive_drop ... ok
test structural::finite_rejected_select ... ok
test structural::fold_init_above_suspending_adapter ... ok
test structural::last_take_child ... ok
test structural::nested_realization ... ok
test structural::one_callback_two_inputs ... ok

test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.34s
```

| Depth shape | Exact successful in-process contract at n=32 and n=2,000 |

| --- | --- |

| no_pull | List[], log 0; whole descriptor validated |

| invalid_zero | Take head, log 0; invalid tail remains residual |

| pull | n, log n |

| inner / outer | List[n], log n, exactly one p: prompt |

| twice | List[n], log n, exactly two p: prompts |

| select | 2, log 2n+1 |

| drop | n, log n+1 |

| cancel | Cancelled, log 1, one prompt on resumed path, no later callback |

| descriptor control | construction and Session teardown complete without realization |



Private tests isolate flat state from descriptor construction at depth 2,000 on 2 MiB: clone/working copy, every source/stage/await-cursor root, last-owner drop and abandonment. The final `lazy` run passes all four plus both structural checks and both falsifiers.


#### Release depth, n=20,000, 2 MiB stack, stdin /dev/null


Sources are [depth/*.rum](../../../.temp/brief-563/depth). Each process has a 60s bound. Incoming descriptor control exits 0; all nine incoming realization shapes exit 134. Final descriptor/no_pull/invalid_zero/pull/select/drop exit 0 with the exact values below. Final inner/outer/twice/cancel exit 1 at InputRequired. They no longer abort, but **fail the required exit-0 postcondition**. The pre-fix n=32 input control also exits 1: CLI --file retires InputRequired rather than driving host resume. The proposed release Rust harness substitution was not performed without a ruling.


`descriptor_control` incoming/final:

base: exit 0; [base/release-depth-descriptor_control.log](../../../.temp/brief-563/base/release-depth-descriptor_control.log)

```text
built
after
Null
```

candidate-3: exit 0; [candidate-3/release-depth-descriptor_control.log](../../../.temp/brief-563/candidate-3/release-depth-descriptor_control.log)

```text
built
after
Null
```

`no_pull` incoming/final:

base: exit 134; [base/release-depth-no_pull.log](../../../.temp/brief-563/base/release-depth-no_pull.log)

```text
built

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
depth: line 1:    11 Aborted                    (core dumped) "$@"
```

candidate-3: exit 0; [candidate-3/release-depth-no_pull.log](../../../.temp/brief-563/candidate-3/release-depth-no_pull.log)

```text
built
{}
0
after
Null
```

`invalid_zero` incoming/final:

base: exit 134; [base/release-depth-invalid_zero.log](../../../.temp/brief-563/base/release-depth-invalid_zero.log)

```text
built

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
depth: line 1:    15 Aborted                    (core dumped) "$@"
```

candidate-3: exit 0; [candidate-3/release-depth-invalid_zero.log](../../../.temp/brief-563/candidate-3/release-depth-invalid_zero.log)

```text
built
Take
after
Null
```

`pull` incoming/final:

base: exit 134; [base/release-depth-pull.log](../../../.temp/brief-563/base/release-depth-pull.log)

```text
built

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
depth: line 1:    19 Aborted                    (core dumped) "$@"
```

candidate-3: exit 0; [candidate-3/release-depth-pull.log](../../../.temp/brief-563/candidate-3/release-depth-pull.log)

```text
built
20000
20000
after
Null
```

`inner` incoming/final:

base: exit 134; [base/release-depth-inner.log](../../../.temp/brief-563/base/release-depth-inner.log)

```text
built

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
depth: line 1:    23 Aborted                    (core dumped) "$@"
```

candidate-3: exit 1; [candidate-3/release-depth-inner.log](../../../.temp/brief-563/candidate-3/release-depth-inner.log)

```text
built
! input required: p: 
error: input required: p:
```

`outer` incoming/final:

base: exit 134; [base/release-depth-outer.log](../../../.temp/brief-563/base/release-depth-outer.log)

```text
built

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
depth: line 1:    27 Aborted                    (core dumped) "$@"
```

candidate-3: exit 1; [candidate-3/release-depth-outer.log](../../../.temp/brief-563/candidate-3/release-depth-outer.log)

```text
built
! input required: p: 
error: input required: p:
```

`twice` incoming/final:

base: exit 134; [base/release-depth-twice.log](../../../.temp/brief-563/base/release-depth-twice.log)

```text
built

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
depth: line 1:    31 Aborted                    (core dumped) "$@"
```

candidate-3: exit 1; [candidate-3/release-depth-twice.log](../../../.temp/brief-563/candidate-3/release-depth-twice.log)

```text
built
! input required: p: 
error: input required: p:
```

`select` incoming/final:

base: exit 134; [base/release-depth-select.log](../../../.temp/brief-563/base/release-depth-select.log)

```text
built

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
depth: line 1:    35 Aborted                    (core dumped) "$@"
```

candidate-3: exit 0; [candidate-3/release-depth-select.log](../../../.temp/brief-563/candidate-3/release-depth-select.log)

```text
built
2
40001
after
Null
```

`drop` incoming/final:

base: exit 134; [base/release-depth-drop.log](../../../.temp/brief-563/base/release-depth-drop.log)

```text
built

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
depth: line 1:    39 Aborted                    (core dumped) "$@"
```

candidate-3: exit 0; [candidate-3/release-depth-drop.log](../../../.temp/brief-563/candidate-3/release-depth-drop.log)

```text
built
20000
20001
after
Null
```

`cancel` incoming/final:

base: exit 134; [base/release-depth-cancel.log](../../../.temp/brief-563/base/release-depth-cancel.log)

```text
built

thread 'main' has overflowed its stack
fatal runtime error: stack overflow
depth: line 1:    43 Aborted                    (core dumped) "$@"
```

candidate-3: exit 1; [candidate-3/release-depth-cancel.log](../../../.temp/brief-563/candidate-3/release-depth-cancel.log)

```text
built
boundary
! input required: p: 
error: input required: p:
```

[pre/cli-input-control.log](../../../.temp/brief-563/pre/cli-input-control.log)


```text
built
! input required: p: 
error: input required: p:
```

#### Mandatory commands on the final source


| Command | Bound (s) | Exit | Raw log |

| --- | --- | --- | --- |

| cargo fmt --check | 150 | 0 | [final2/fmt.log](../../../.temp/brief-563/final2/fmt.log) |

| cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations | 150 | 0 | [final2/lazy-continuations.log](../../../.temp/brief-563/final2/lazy-continuations.log) |

| cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test loop_continuations | 150 | 0 | [final2/loop-continuations.log](../../../.temp/brief-563/final2/loop-continuations.log) |

| cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test guest_loading_resume | 150 | 0 | [final2/guest-loading-resume.log](../../../.temp/brief-563/final2/guest-loading-resume.log) |

| cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests lazy | 150 | 0 | [final2/lazy.log](../../../.temp/brief-563/final2/lazy.log) |

| cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests ownership_graph | 120 | 0 | [final2/ownership.log](../../../.temp/brief-563/final2/ownership.log) |

| cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests local_production_acceptance | 120 | 0 | [inventory/final/certificates-default.log](../../../.temp/brief-563/inventory/final/certificates-default.log) |

| cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --no-default-features local_production_acceptance | 120 | 0 | [inventory/final/certificates-no-default.log](../../../.temp/brief-563/inventory/final/certificates-no-default.log) |

| cargo test -p rum-runtime --target-dir .temp/brief-563/target-tests | 120 | 101 | [final2/runtime.log](../../../.temp/brief-563/final2/runtime.log) |

| cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests | 1500 | 0 | [final2/stdlib.log](../../../.temp/brief-563/final2/stdlib.log) |

| cargo test --workspace --target-dir .temp/brief-563/target-tests -- --nocapture | 1500 | 101 | [final2/workspace.log](../../../.temp/brief-563/final2/workspace.log) |

| env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --target-dir .temp/brief-563/target-tests -- --nocapture | 1500 | 101 | [final2/workspace-platform.log](../../../.temp/brief-563/final2/workspace-platform.log) |



[final2/fmt.log](../../../.temp/brief-563/final2/fmt.log)


```text
(No stdout/stderr.)
```

[final2/lazy-continuations.log](../../../.temp/brief-563/final2/lazy-continuations.log)


```text
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.34s
```

[final2/loop-continuations.log](../../../.temp/brief-563/final2/loop-continuations.log)


```text
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
```

[final2/guest-loading-resume.log](../../../.temp/brief-563/final2/guest-loading-resume.log)


```text
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s
```

[final2/lazy.log](../../../.temp/brief-563/final2/lazy.log)


```text
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 1041 filtered out; finished in 0.26s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.27s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

[final2/ownership.log](../../../.temp/brief-563/final2/ownership.log)


```text
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1066 filtered out; finished in 59.16s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 59 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

[inventory/final/certificates-default.log](../../../.temp/brief-563/inventory/final/certificates-default.log)


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1085 filtered out; finished in 17.45s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 59 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

[inventory/final/certificates-no-default.log](../../../.temp/brief-563/inventory/final/certificates-no-default.log)


```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1079 filtered out; finished in 17.66s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 59 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

[final2/runtime.log](../../../.temp/brief-563/final2/runtime.log)


```text
test result: FAILED. 1092 passed; 2 failed; 3 ignored; 0 measured; 0 filtered out; finished in 23.09s
error: test failed, to rerun pass `-p rum-runtime --lib`
```

[final2/stdlib.log](../../../.temp/brief-563/final2/stdlib.log)


```text
test result: ok. 1033 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1089.75s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.58s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

[final2/workspace.log](../../../.temp/brief-563/final2/workspace.log)


```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.78s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.94s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: FAILED. 142 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
error: test failed, to rerun pass `-p rum-jupyter --lib`
```

[final2/workspace-platform.log](../../../.temp/brief-563/final2/workspace-platform.log)


```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.94s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.02s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: FAILED. 1092 passed; 2 failed; 3 ignored; 0 measured; 0 filtered out; finished in 23.10s
error: test failed, to rerun pass `-p rum-runtime --lib`
```

The full standard-library command passed 1,033 unit tests, with 53 pre-existing ignores, plus 47 guest-loading, 59 lazy and 21 loop integration tests. Runtime passed 1,092 and failed two, with three pre-existing ignores. The failures are exactly the missing derived and pure-abandon inventory entries; [protocol/runtime-inventory-failures.json](../../../.temp/brief-563/protocol/runtime-inventory-failures.json) lists the discovered differences. The first runtime attempt also exposed direct insertion and test stdout; those scoped defects were fixed before candidate 3 was measured. Consumer copies were moved out of trait methods and the frozen prefix changed to SharedArgs to honor the existing frame rules.


Both feature-set certificate generators first rejected stale installed C01 evidence, after emitting actual new cells. Copying only those cells made both gates pass. These failures and the earlier compile errors remain in the command ledger; they are not omitted from the record.


Workspace first stopped in rum-jupyter because native ZeroMQ could not bind loopback. Contrary to the guide’s automatic-skip wording, the existing harness requires RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 for PermissionDenied. The retry used that documented switch, made no adapter edit, and recorded **nine native skips**. Cargo reports those early returns as ok; they are not counted here as exercised native transport coverage. The retry then stopped at the same two runtime inventory failures. Workspace therefore did not finish the packages/doc tests scheduled after runtime; full stdlib was separately completed.


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

All ordinary ignored tests reported by the final suites (deduplicated verbatim):


```text
test associations::basic::lookup::tests::lookup_outcome_drop_is_bounded_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack drop certificate; run explicitly with --release
test tests::association_key_reshape_depth::association_key_deflatten_abandons_a_deep_partial_tree_without_growing_the_stack ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_reshape_depth::association_key_deflatten_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_reshape_depth::association_key_deflatten_flatten_round_trip_is_exact_at_physical_height_4097 ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_reshape_depth::association_key_flatten_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_take_drop_depth::key_drop_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_take_drop_depth::key_take_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_keys_depth::keys_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_lookup_depth::lookup_hit_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_lookup_depth::lookup_miss_and_resume_drop_complete_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_values_depth::values_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_depth::delete_missing_recursive_armed_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_depth::delete_missing_recursive_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_depth::delete_missing_scan_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_association_clears_64000_siblings_on_two_mib ... ignored, release-only width probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_list_clears_64000_siblings_on_two_mib ... ignored, release-only width probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_scan_all_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_scan_zero_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::held_association_exact_path_depth::associate_to_part_target_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::held_association_exact_path_depth::key_drop_from_part_target_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::apply_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::association_assignment_read_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::association_map_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::cases_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::construction_reaches_physical_height_20001_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::count_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::d_nested_sin_completes_at_physical_height_4097_on_two_mib ... ignored, release-only active-spine safety probe; run explicitly with --release
test tests::iterative_traversal_depth::deletecases_active_spine_returns_in_band_resource_error_on_two_mib ... ignored, release-only active-spine safety probe; run explicitly with --release
test tests::iterative_traversal_depth::deletecases_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::freeq_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::level_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::map_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::matchq_completes_at_physical_height_20001_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::memberq_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::merge_nested_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::position_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::replace_active_spine_returns_in_band_resource_error_on_two_mib ... ignored, release-only active-spine safety probe; run explicitly with --release
test tests::iterative_traversal_depth::replace_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::replaceall_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::replacepart_deep_path_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::rule_rhs_substitution_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::take_multispec_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::tostring_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::with_substitution_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::map_indexed_association_depth::map_indexed_association_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::map_indexed_association_depth::map_indexed_association_heads_refuses_in_band_rather_than_terminating ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::merge_native_stack_guard::merge_zero_native_stack_boundary_fails_in_band_and_session_recovers ... ignored, requires rum-runtime/test-native-stack-guard-zero
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... ignored, writes a review copy; does not discharge contracts or install marks
test tests::scan_and_keypath_set_native_stack_guard::keypath_set_zero_native_stack_boundary_preserves_both_lvalue_roots ... ignored, requires rum-runtime/test-native-stack-guard-zero
test tests::scan_and_keypath_set_native_stack_guard::scan_zero_native_stack_boundary_covers_each_edge_without_callbacks ... ignored, requires rum-runtime/test-native-stack-guard-zero
test tests::stress_corpus::peano_arithmetic_heavy_multiplication ... ignored
test tests::stress_corpus::rule_based_differentiator_handles_heavy_generated_polynomial ... ignored
test tests::stress_corpus::tokenize_with_ini_grammar_handles_heavy_generated_config ... ignored
test runtime::evaluation_cache::tests::call_stability_profile_reports_per_source_head_outcomes ... ignored, release-only profiling gate; run with RUM_PROFILE_EVALUATION_CACHE=1
test runtime::tests::left_chain_lowering::left_chain_two_mib_full_round_trip_accepts_and_rejects_without_abort ... ignored, release-only deployment-stack probe; run explicitly with --release
test runtime::tests::left_chain_lowering::left_chain_two_mib_nested_expr_drops_naturally_at_the_boundary ... ignored, release-only deployment-stack probe; run explicitly with --release
test battered_session_recovers_after_repeated_recoverable_limit_trips ... ignored
test cancel_rewrite_loop_stability_10_cycles ... ignored
```

The source fence prevents installing the 10 FrameRootTrace and 4 PureFrameAbandon entries in `rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`. The exact additive patch is [proposed-runtime-inventory.patch](../../../.temp/brief-563/proposed-runtime-inventory.patch). It changes no assertion or runtime implementation and remains unapplied pending the director’s scope decision.


### 3. All performance labels, samples and preserved inputs


One immutable incoming base and all three distinct candidates were preserved. All 54 separately unwrapped workload result checks pass for each build. The nine inherited brief-559 .rum programs are reused byte-for-byte; all generated programs use .rum. Sources, raw allocation/perf output, stdout/stderr, per-process commands and results are under [perf/](../../../.temp/brief-563/perf). The label/source manifest is [perf/labels.json](../../../.temp/brief-563/perf/labels.json). The metric slope is (count(2N)-count(N))/N; no wall-clock difference is used for a decision.


Controlled Cargo environment and feature/build flags are in per-command metadata; profiling flags are cleared by run.py/metric-panel.py and only the allocation pass enables RUM_PROFILE_ALLOC=1. The preserved environment.json files are empty dictionaries, not a full host-environment inventory; no historical environment has been invented afterward. Normal instructions use taskset -c 2 and the accepted cpu_core/instructions/u event spelling.


| Build | Source/input manifest SHA256 | Default binary SHA256 | alloc-count binary SHA256 |

| --- | --- | --- | --- |

| base | 1514cb6c51367fb30d73caa51064702a7ed5f0cdfdb6051d498604af507475ea | c08b67e8f53c9651c8d139e8c39bdb392320c82376b0c4eec7f03c26397c80cc | 4285dd8a3ccded49ae769b166b04282057cc2a6a0ceb67c1c9d81202b561ac03 |

| candidate-1 | 0b532be2113d701013ec90604650ffa8373cea4e893498652f4f65163885dd3b | d882fa260aeef58960ab381e8d768a8a22c21fa39fe4600e3f19a54a21627d54 | 3a0d912cbd42c13a0ad85d31cfe7c4f1623a0ae02ed6129ac4287a1cc7009082 |

| candidate-2 | 8f5bb9ce6fe2e4342510dc64ed3532321d19844439715d84d68e0cc542f94ee5 | e7576249f2769ef2cd86775cc9f7fa01f43629082660c5d6c68028241e599e64 | 3178dab23115dce32e7bfa7820edf76b210a74087d0ce2106d3ddd16b4f313a9 |

| candidate-3 | b24ff50700d3bcc5ba406de020570f2105e9b5de3a5b5f173282f3d7813847e8 | 431d2c42c6b73e21d879b4877eb7d40cedec419ab82ee6c1cf94b8060fae8302 | 197c1604a0638f6db49fb893fd0a1f84e45a1ba96f47e3f70cf7bccb99400034 |



Procedure failures are retained: the literal cpu_core/instructions:u spelling was rejected by perf in 108 invocations, with no measured CLI run; the accepted /u spelling was then used. The initial 60s allocation and first valid instruction attempts timed out for depth-fold-through-source-256 at N=2,000. Its N=1,000 instruction process had taken 36.304979166s; the revised 150s metric bound exceeds twice the extrapolated 2N duration. The allocation cell succeeded on its second permitted attempt. That allocation retry occurred after the first valid instruction pass, a procedure-order departure. No timeout is counted as a measurement. The baseline has 54 allocation slopes, 53 complete instruction pairs and one second-sample-only Fold-256 slope. The missing first pair is shown as missing, never as a pass.


The two valid base instruction passes are saved as samples 2 and 3 (sample 1 was the syntax rejection), paired with each candidate’s samples 1 and 2. All three candidates have complete 108-process allocation passes and 216-process instruction passes. All raw results predate parsing. [perf/bounds.json](../../../.temp/brief-563/perf/bounds.json) records the bound calibration.


#### Incoming base: every label


| Label | N | alloc/call | bytes/call | peak N | peak 2N | instructions/call sample 1 | instructions/call sample 2 |

| --- | --- | --- | --- | --- | --- | --- | --- |

| lazy-range-take-small | 20000 | 141 | 2714 | 11428045 | 12068045 | 41864.1059 | 41802.7956 |

| lazy-range-normal-small | 20000 | 157 | 3444 | 11428491 | 12068491 | 53655.8386 | 53724.3123 |

| lazy-map-realize-small | 20000 | 333 | 13780 | 11429646 | 12069646 | 178317.212 | 178328.742 |

| lazy-select-realize-small | 20000 | 236 | 6874 | 9594202 | 10234202 | 114231.771 | 114258.168 |

| lazy-select-scan-small | 20000 | 1471 | 62849 | 9595818 | 10235818 | 799020.446 | 799008.65 |

| lazy-nestlist-realize-small | 20000 | 197 | 11372 | 11429099 | 12069099 | 145161.136 | 145148.769 |

| lazy-foldlist-realize-small | 20000 | 216 | 7644 | 11429094 | 12069094 | 113099.03 | 113087.512 |

| lazy-drop-realize-small | 20000 | 352 | 14300 | 9595465 | 10235465 | 191299.744 | 191378.411 |

| lazy-first-small | 20000 | 64 | 2924 | 9595677 | 10235677 | 41422.5142 | 41374.1845 |

| lazy-part-small | 20000 | 152 | 6222 | 9596551 | 10236551 | 88375.3715 | 88375.4313 |

| lazy-transparent-adapters-small | 20000 | 322 | 13084 | 949519 | 1589519 | 182400.509 | 182151.292 |

| lazy-table-construct-short | 20000 | 63 | 1537 | 942784 | 1582784 | 23582.3033 | 23582.9431 |

| lazy-table-construct-full | 20000 | 64 | 1449 | 942664 | 1582664 | 24056.0753 | 24012.7463 |

| lazy-table-realize-small | 20000 | 251 | 9230 | 10053760 | 10693760 | 116678.267 | 116627.269 |

| lazy-permutations-take-small | 20000 | 29 | 1729 | 9593774 | 10233774 | 28226.0272 | 28227.9792 |

| lazy-subsets-take-small | 20000 | 29 | 1417 | 8446869 | 9086869 | 25473.9097 | 25537.9196 |

| lazy-tuples-take-small | 20000 | 29 | 1361 | 8905896 | 9545896 | 26638.6928 | 26679.1129 |

| scan-no-input | 20000 | 33 | 2536 | 942414 | 1582414 | 30010.9048 | 30007.9624 |

| optionvalue-three-no-input | 20000 | 9 | 1128 | 9134068 | 9774068 | 9738.86165 | 9738.567 |

| replacerepeated-no-input | 20000 | 61 | 7691 | 948049 | 1588049 | 40117.8296 | 40117.7809 |

| dot-three-no-input | 20000 | 596 | 23504 | 8678236 | 9318236 | 256453.658 | 256452.954 |

| differences-no-input | 20000 | 13 | 704 | 9591399 | 10231399 | 9989.7326 | 9990.58515 |

| do-count-null | 20000 | 0 | 0 | 299668 | 299668 | 496.95 | 496.8492 |

| do-iterator-null | 20000 | 0 | 32 | 940146 | 1580146 | 1137.1392 | 1136.89205 |

| table-iterator-null | 20000 | 0 | 32 | 940025 | 1580025 | 348.8655 | 349.0439 |

| replaceall-simple | 20000 | 5 | 1352 | 943388 | 1583388 | 11403.388 | 11447.4196 |

| table-small | 20000 | 22 | 977 | 9132887 | 9772887 | 14756.3149 | 14756.4478 |

| table-iterator-plus | 20000 | 2 | 72 | 940209 | 1580209 | 565.33085 | 565.0963 |

| table-iterator-pair-list | 20000 | 2 | 136 | 8574075 | 11294075 | 1726.62565 | 1726.76935 |

| take-multispec-small | 20000 | 109 | 5856 | 9594023 | 10234023 | 50781.6916 | 50788.7304 |

| lazy-depth-map-build-8 | 1000 | 22.001 | 2255.864 | 762116 | 1220100 | 19450.58 | 19468.087 |

| lazy-depth-map-first-8 | 1000 | 194 | 11552 | 339418 | 371418 | 131667.504 | 131482.213 |

| lazy-depth-map-eight-8 | 1000 | 1470.001 | 83152.864 | 990724 | 1678084 | 961539.721 | 961693.762 |

| lazy-depth-select-restart-8 | 1000 | 333 | 17356 | 341305 | 373305 | 179761.101 | 180210.556 |

| lazy-depth-drop-all-one-8 | 1000 | 176 | 3504 | 337909 | 369909 | 48246.677 | 48248.045 |

| lazy-depth-take-map-8 | 1000 | 234 | 12968 | 342603 | 374603 | 141428.03 | 141434.614 |

| lazy-depth-fold-through-source-8 | 1000 | 749.001 | 58816.864 | 1219923 | 2136659 | 601486.522 | 601496.901 |

| lazy-depth-nest-map-8 | 1000 | 198 | 12340 | 339730 | 371730 | 142705.741 | 142208.357 |

| lazy-depth-map-build-64 | 1000 | 78.001 | 9423.864 | 770631 | 1228615 | 41618.401 | 41632.136 |

| lazy-depth-map-first-64 | 1000 | 1426 | 88832 | 355101 | 387101 | 956256.734 | 956257.944 |

| lazy-depth-map-eight-64 | 1000 | 10934.001 | 651216.864 | 999239 | 1686599 | 7417732.4 | 7417548.24 |

| lazy-depth-select-restart-64 | 1000 | 1733 | 102924 | 356988 | 388988 | 980952.971 | 978361.115 |

| lazy-depth-drop-all-one-64 | 1000 | 1240 | 21032 | 353592 | 385592 | 345180.254 | 345163.555 |

| lazy-depth-take-map-64 | 1000 | 1746 | 100160 | 373966 | 405966 | 1031343.8 | 1031229.67 |

| lazy-depth-fold-through-source-64 | 1000 | 43396.001 | 3406161.86 | 4771185 | 9218609 | 34505574.2 | 34505637.2 |

| lazy-depth-nest-map-64 | 1000 | 1430 | 89620 | 355413 | 387413 | 968789.243 | 968792.609 |

| lazy-depth-map-build-256 | 1000 | 270.001 | 33999.864 | 799818 | 1257802 | 117673.149 | 117656.186 |

| lazy-depth-map-first-256 | 1000 | 5650 | 353792 | 408864 | 440864 | 3764210.67 | 3775667.65 |

| lazy-depth-map-eight-256 | 1000 | 43382.001 | 2598864.86 | 1028426 | 1715786 | 29550094.6 | 29549589 |

| lazy-depth-select-restart-256 | 1000 | 6533 | 396300 | 410751 | 442751 | 3677620.44 | 3699799.13 |

| lazy-depth-drop-all-one-256 | 1000 | 4888 | 81128 | 407355 | 439355 | 2339174.93 | 2339176.34 |

| lazy-depth-take-map-256 | 1000 | 6930 | 399104 | 481489 | 513489 | 4087163.56 | 4086623 |

| lazy-depth-fold-through-source-256 | 1000 | 689574.001 | 54023121.9 | 17119095 | 33854519 | missing | 538246355 |

| lazy-depth-nest-map-256 | 1000 | 5654 | 354580 | 409176 | 441176 | 3775398.44 | 3775396.79 |



#### candidate-1 — all 54 allocation and instruction rows


Allocation/byte regressions: **26**. Paired instruction regressions: **24**. Worst repeatable instruction increase (minimum of the pair): **41.413832840%**. Full precision: [candidate-1/metrics-comparison.tsv](../../../.temp/brief-563/candidate-1/metrics-comparison.tsv).


| Label | alloc/call | delta alloc | bytes/call | delta bytes | peak N | peak 2N | Allocation gate |

| --- | --- | --- | --- | --- | --- | --- | --- |

| lazy-range-take-small | 141 | 0 | 2714 | 0 | 11428045 | 12068045 | clear |

| lazy-range-normal-small | 157 | 0 | 3604 | 160 | 11428491 | 12068491 | REGRESSION |

| lazy-map-realize-small | 332 | -1 | 13812 | 32 | 11429646 | 12069646 | REGRESSION |

| lazy-select-realize-small | 235 | -1 | 6906 | 32 | 9594202 | 10234202 | REGRESSION |

| lazy-select-scan-small | 1471 | 0 | 63009 | 160 | 9595818 | 10235818 | REGRESSION |

| lazy-nestlist-realize-small | 197 | 0 | 11532 | 160 | 11429099 | 12069099 | REGRESSION |

| lazy-foldlist-realize-small | 215 | -1 | 7676 | 32 | 11429094 | 12069094 | REGRESSION |

| lazy-drop-realize-small | 350 | -2 | 14204 | -96 | 9595465 | 10235465 | clear |

| lazy-first-small | 64 | 0 | 3084 | 160 | 9595837 | 10235837 | REGRESSION |

| lazy-part-small | 151 | -1 | 6254 | 32 | 9596583 | 10236583 | REGRESSION |

| lazy-transparent-adapters-small | 319 | -3 | 13308 | 224 | 949455 | 1589455 | REGRESSION |

| lazy-table-construct-short | 63 | 0 | 1537 | 0 | 942784 | 1582784 | clear |

| lazy-table-construct-full | 64 | 0 | 1449 | 0 | 942664 | 1582664 | clear |

| lazy-table-realize-small | 250 | -1 | 9262 | 32 | 10053792 | 10693792 | REGRESSION |

| lazy-permutations-take-small | 29 | 0 | 1729 | 0 | 9593774 | 10233774 | clear |

| lazy-subsets-take-small | 29 | 0 | 1417 | 0 | 8446869 | 9086869 | clear |

| lazy-tuples-take-small | 29 | 0 | 1361 | 0 | 8905896 | 9545896 | clear |

| scan-no-input | 33 | 0 | 2536 | 0 | 942414 | 1582414 | clear |

| optionvalue-three-no-input | 9 | 0 | 1128 | 0 | 9134068 | 9774068 | clear |

| replacerepeated-no-input | 61 | 0 | 7691 | 0 | 948049 | 1588049 | clear |

| dot-three-no-input | 596 | 0 | 23504 | 0 | 8678236 | 9318236 | clear |

| differences-no-input | 13 | 0 | 704 | 0 | 9591399 | 10231399 | clear |

| do-count-null | 0 | 0 | 0 | 0 | 299668 | 299668 | clear |

| do-iterator-null | 0 | 0 | 32 | 0 | 940146 | 1580146 | clear |

| table-iterator-null | 0 | 0 | 32 | 0 | 940025 | 1580025 | clear |

| replaceall-simple | 5 | 0 | 1352 | 0 | 943388 | 1583388 | clear |

| table-small | 22 | 0 | 977 | 0 | 9132887 | 9772887 | clear |

| table-iterator-plus | 2 | 0 | 72 | 0 | 940209 | 1580209 | clear |

| table-iterator-pair-list | 2 | 0 | 136 | 0 | 8574075 | 11294075 | clear |

| take-multispec-small | 109 | 0 | 5856 | 0 | 9594023 | 10234023 | clear |

| lazy-depth-map-build-8 | 16.001 | -6 | 2095.864 | -160 | 762116 | 1220100 | clear |

| lazy-depth-map-first-8 | 188 | -6 | 11392 | -160 | 338970 | 370970 | clear |

| lazy-depth-map-eight-8 | 1464.001 | -6 | 82992.864 | -160 | 990724 | 1678084 | clear |

| lazy-depth-select-restart-8 | 327 | -6 | 17196 | -160 | 340857 | 372857 | clear |

| lazy-depth-drop-all-one-8 | 170 | -6 | 3344 | -160 | 337461 | 369461 | clear |

| lazy-depth-take-map-8 | 221 | -13 | 12936 | -32 | 341707 | 373707 | clear |

| lazy-depth-fold-through-source-8 | 743.001 | -6 | 58656.864 | -160 | 1219923 | 2136659 | clear |

| lazy-depth-nest-map-8 | 192 | -6 | 12180 | -160 | 339282 | 371282 | clear |

| lazy-depth-map-build-64 | 19.001 | -59 | 10159.864 | 736 | 770631 | 1228615 | REGRESSION |

| lazy-depth-map-first-64 | 1367 | -59 | 89568 | 736 | 351517 | 383517 | REGRESSION |

| lazy-depth-map-eight-64 | 10875.001 | -59 | 651952.864 | 736 | 999239 | 1686599 | REGRESSION |

| lazy-depth-select-restart-64 | 1674 | -59 | 103660 | 736 | 353404 | 385404 | REGRESSION |

| lazy-depth-drop-all-one-64 | 1181 | -59 | 21768 | 736 | 350008 | 382008 | REGRESSION |

| lazy-depth-take-map-64 | 1624 | -122 | 101920 | 1760 | 366798 | 398798 | REGRESSION |

| lazy-depth-fold-through-source-64 | 43337.001 | -59 | 3406897.86 | 736 | 4767601 | 9215025 | REGRESSION |

| lazy-depth-nest-map-64 | 1371 | -59 | 90356 | 736 | 351829 | 383829 | REGRESSION |

| lazy-depth-map-build-256 | 21.001 | -249 | 37807.864 | 3808 | 799818 | 1257802 | REGRESSION |

| lazy-depth-map-first-256 | 5401 | -249 | 357600 | 3808 | 394528 | 426528 | REGRESSION |

| lazy-depth-map-eight-256 | 43133.001 | -249 | 2602672.86 | 3808 | 1028426 | 1715786 | REGRESSION |

| lazy-depth-select-restart-256 | 6284 | -249 | 400108 | 3808 | 396415 | 428415 | REGRESSION |

| lazy-depth-drop-all-one-256 | 4639 | -249 | 84936 | 3808 | 393019 | 425019 | REGRESSION |

| lazy-depth-take-map-256 | 6426 | -504 | 407008 | 7904 | 452817 | 484817 | REGRESSION |

| lazy-depth-fold-through-source-256 | 689325.001 | -249 | 54026929.9 | 3808 | 17104759 | 33840183 | REGRESSION |

| lazy-depth-nest-map-256 | 5405 | -249 | 358388 | 3808 | 394840 | 426840 | REGRESSION |



| Label | instructions/call 1 | delta 1 % | instructions/call 2 | delta 2 % | Paired instruction gate |

| --- | --- | --- | --- | --- | --- |

| lazy-range-take-small | 42703.863 | 2.00591206 | 42687.158 | 2.11555815 | REGRESSION |

| lazy-range-normal-small | 54960.946 | 2.43236791 | 54982.545 | 2.34201732 | REGRESSION |

| lazy-map-realize-small | 179458.395 | 0.639973413 | 179683.895 | 0.759918528 | REGRESSION |

| lazy-select-realize-small | 115151.751 | 0.805363031 | 115150.002 | 0.780542926 | REGRESSION |

| lazy-select-scan-small | 801056.479 | 0.254816126 | 800971.533 | 0.245664806 | REGRESSION |

| lazy-nestlist-realize-small | 146496.101 | 0.919644017 | 146513.486 | 0.940219791 | REGRESSION |

| lazy-foldlist-realize-small | 114396.167 | 1.14690351 | 114509.92 | 1.2577945 | REGRESSION |

| lazy-drop-realize-small | 192265.679 | 0.504932668 | 192210.552 | 0.434814458 | REGRESSION |

| lazy-first-small | 41646.1841 | 0.539971811 | 41598.231 | 0.54151291 | REGRESSION |

| lazy-part-small | 88627.6613 | 0.285475292 | 88611.2934 | 0.266886562 | REGRESSION |

| lazy-transparent-adapters-small | 183046.865 | 0.354360935 | 183055.982 | 0.496669329 | REGRESSION |

| lazy-table-construct-short | 23689.9186 | 0.456339012 | 23652.3463 | 0.294294057 | REGRESSION |

| lazy-table-construct-full | 24098.4725 | 0.176243421 | 24099.4721 | 0.361165687 | REGRESSION |

| lazy-table-realize-small | 117487.95 | 0.693945128 | 117487.401 | 0.737504965 | REGRESSION |

| lazy-permutations-take-small | 28623.0117 | 1.40644855 | 28623.1613 | 1.39996614 | REGRESSION |

| lazy-subsets-take-small | 25866.4267 | 1.54085908 | 25866.6609 | 1.28726695 | REGRESSION |

| lazy-tuples-take-small | 27048.0752 | 1.53679594 | 27047.9248 | 1.38239941 | REGRESSION |

| scan-no-input | 29989.9748 | -0.0697414827 | 29989.9857 | -0.0599062667 | clear |

| optionvalue-three-no-input | 9737.65935 | -0.0123453854 | 9739.4502 | 0.0090690961 | clear |

| replacerepeated-no-input | 40131.8178 | 0.0348676639 | 40131.8127 | 0.0349767602 | clear |

| dot-three-no-input | 256452.683 | -0.000380107661 | 256452.067 | -0.000345521464 | clear |

| differences-no-input | 9987.64455 | -0.0209019609 | 9985.69545 | -0.0489430792 | clear |

| do-count-null | 497.07265 | 0.0246805514 | 497.04865 | 0.0401429649 | clear |

| do-iterator-null | 1136.92195 | -0.0191049609 | 1137.18075 | 0.0253937918 | clear |

| table-iterator-null | 349.44515 | 0.166152858 | 349.2207 | 0.0506526543 | REGRESSION |

| replaceall-simple | 11403.5235 | 0.00118868182 | 11403.343 | -0.385035679 | clear |

| table-small | 14741.1827 | -0.10254796 | 14741.093 | -0.104054514 | clear |

| table-iterator-plus | 565.3063 | -0.00434258983 | 565.14735 | 0.00903385848 | clear |

| table-iterator-pair-list | 1727.1001 | 0.027478452 | 1726.63865 | -0.00756904794 | clear |

| take-multispec-small | 51109.3328 | 0.645195423 | 51109.2574 | 0.631098666 | REGRESSION |

| lazy-depth-map-build-8 | 18836.392 | -3.15768476 | 18834.634 | -3.25380198 | clear |

| lazy-depth-map-first-8 | 131160.577 | -0.3850054 | 131168.037 | -0.238949431 | clear |

| lazy-depth-map-eight-8 | 960462.536 | -0.112027093 | 960455.846 | -0.128722474 | clear |

| lazy-depth-select-restart-8 | 179178.415 | -0.324144655 | 179154.228 | -0.586163221 | clear |

| lazy-depth-drop-all-one-8 | 49218.949 | 2.01521029 | 49221.069 | 2.01671177 | REGRESSION |

| lazy-depth-take-map-8 | 141169.278 | -0.18295666 | 141192.727 | -0.171023905 | clear |

| lazy-depth-fold-through-source-8 | 601475.498 | -0.00183279252 | 601468.726 | -0.00468414716 | clear |

| lazy-depth-nest-map-8 | 141932.786 | -0.54164254 | 141929.837 | -0.195853469 | clear |

| lazy-depth-map-build-64 | 34988.5 | -15.9302156 | 34976.324 | -15.987198 | clear |

| lazy-depth-map-first-64 | 949611.944 | -0.694875107 | 951080.12 | -0.541467293 | clear |

| lazy-depth-map-eight-64 | 7421950.04 | 0.0568588859 | 7422504.13 | 0.0668131685 | REGRESSION |

| lazy-depth-select-restart-64 | 973576.614 | -0.751958271 | 973813.187 | -0.464851672 | clear |

| lazy-depth-drop-all-one-64 | 404263.882 | 17.1167462 | 404260.836 | 17.1215298 | REGRESSION |

| lazy-depth-take-map-64 | 1021901.82 | -0.915502377 | 1021895.01 | -0.90519748 | clear |

| lazy-depth-fold-through-source-64 | 34559557.4 | 0.156447682 | 34560109.4 | 0.157864692 | REGRESSION |

| lazy-depth-nest-map-64 | 963905.084 | -0.504150829 | 962257.809 | -0.674530332 | clear |

| lazy-depth-map-build-256 | 86384.045 | -26.5898417 | 86360.577 | -26.5992041 | clear |

| lazy-depth-map-first-256 | 3738330.32 | -0.687537184 | 3738495.59 | -0.984516341 | clear |

| lazy-depth-map-eight-256 | 29559626.4 | 0.0322565262 | 29560036.5 | 0.0353558658 | clear |

| lazy-depth-select-restart-256 | 3653947.5 | -0.643702536 | 3653252.43 | -1.2580873 | clear |

| lazy-depth-drop-all-one-256 | 3307916.93 | 41.4138328 | 3307923.2 | 41.4140159 | REGRESSION |

| lazy-depth-take-map-256 | 4038490.53 | -1.19087542 | 4037938.41 | -1.19131608 | clear |

| lazy-depth-fold-through-source-256 | 539287052 | missing | 539215304 | 0.180019508 | MISSING BASE PAIR |

| lazy-depth-nest-map-256 | 3749404 | -0.688521581 | 3749402.95 | -0.688506332 | clear |



#### candidate-2 — all 54 allocation and instruction rows


Allocation/byte regressions: **0**. Paired instruction regressions: **34**. Worst repeatable instruction increase (minimum of the pair): **43.467416148%**. Full precision: [candidate-2/metrics-comparison.tsv](../../../.temp/brief-563/candidate-2/metrics-comparison.tsv).


| Label | alloc/call | delta alloc | bytes/call | delta bytes | peak N | peak 2N | Allocation gate |

| --- | --- | --- | --- | --- | --- | --- | --- |

| lazy-range-take-small | 141 | 0 | 2714 | 0 | 11428045 | 12068045 | clear |

| lazy-range-normal-small | 157 | 0 | 3388 | -56 | 11428491 | 12068491 | clear |

| lazy-map-realize-small | 333 | 0 | 13740 | -40 | 11429646 | 12069646 | clear |

| lazy-select-realize-small | 236 | 0 | 6834 | -40 | 9594202 | 10234202 | clear |

| lazy-select-scan-small | 1471 | 0 | 62793 | -56 | 9595818 | 10235818 | clear |

| lazy-nestlist-realize-small | 197 | 0 | 11316 | -56 | 11429099 | 12069099 | clear |

| lazy-foldlist-realize-small | 216 | 0 | 7604 | -40 | 11429094 | 12069094 | clear |

| lazy-drop-realize-small | 351 | -1 | 14204 | -96 | 9595465 | 10235465 | clear |

| lazy-first-small | 64 | 0 | 2868 | -56 | 9595621 | 10235621 | clear |

| lazy-part-small | 152 | 0 | 6182 | -40 | 9596439 | 10236439 | clear |

| lazy-transparent-adapters-small | 319 | -3 | 12876 | -208 | 949239 | 1589239 | clear |

| lazy-table-construct-short | 63 | 0 | 1537 | 0 | 942784 | 1582784 | clear |

| lazy-table-construct-full | 64 | 0 | 1449 | 0 | 942664 | 1582664 | clear |

| lazy-table-realize-small | 251 | 0 | 9190 | -40 | 10053648 | 10693648 | clear |

| lazy-permutations-take-small | 29 | 0 | 1729 | 0 | 9593774 | 10233774 | clear |

| lazy-subsets-take-small | 29 | 0 | 1417 | 0 | 8446869 | 9086869 | clear |

| lazy-tuples-take-small | 29 | 0 | 1361 | 0 | 8905896 | 9545896 | clear |

| scan-no-input | 33 | 0 | 2536 | 0 | 942414 | 1582414 | clear |

| optionvalue-three-no-input | 9 | 0 | 1128 | 0 | 9134068 | 9774068 | clear |

| replacerepeated-no-input | 61 | 0 | 7691 | 0 | 948049 | 1588049 | clear |

| dot-three-no-input | 596 | 0 | 23504 | 0 | 8678236 | 9318236 | clear |

| differences-no-input | 13 | 0 | 704 | 0 | 9591399 | 10231399 | clear |

| do-count-null | 0 | 0 | 0 | 0 | 299668 | 299668 | clear |

| do-iterator-null | 0 | 0 | 32 | 0 | 940146 | 1580146 | clear |

| table-iterator-null | 0 | 0 | 32 | 0 | 940025 | 1580025 | clear |

| replaceall-simple | 5 | 0 | 1352 | 0 | 943388 | 1583388 | clear |

| table-small | 22 | 0 | 977 | 0 | 9132887 | 9772887 | clear |

| table-iterator-plus | 2 | 0 | 72 | 0 | 940209 | 1580209 | clear |

| table-iterator-pair-list | 2 | 0 | 136 | 0 | 8574075 | 11294075 | clear |

| take-multispec-small | 109 | 0 | 5856 | 0 | 9594023 | 10234023 | clear |

| lazy-depth-map-build-8 | 16.001 | -6 | 1879.864 | -376 | 762116 | 1220100 | clear |

| lazy-depth-map-first-8 | 188 | -6 | 11176 | -376 | 338970 | 370970 | clear |

| lazy-depth-map-eight-8 | 1464.001 | -6 | 82776.864 | -376 | 990724 | 1678084 | clear |

| lazy-depth-select-restart-8 | 327 | -6 | 16980 | -376 | 340857 | 372857 | clear |

| lazy-depth-drop-all-one-8 | 170 | -6 | 3128 | -376 | 337461 | 369461 | clear |

| lazy-depth-take-map-8 | 220 | -14 | 12144 | -824 | 341707 | 373707 | clear |

| lazy-depth-fold-through-source-8 | 743.001 | -6 | 58440.864 | -376 | 1219923 | 2136659 | clear |

| lazy-depth-nest-map-8 | 192 | -6 | 11964 | -376 | 339282 | 371282 | clear |

| lazy-depth-map-build-64 | 16.001 | -62 | 5911.864 | -3512 | 770631 | 1228615 | clear |

| lazy-depth-map-first-64 | 1364 | -62 | 85320 | -3512 | 351517 | 383517 | clear |

| lazy-depth-map-eight-64 | 10872.001 | -62 | 647704.864 | -3512 | 999239 | 1686599 | clear |

| lazy-depth-select-restart-64 | 1671 | -62 | 99412 | -3512 | 353404 | 385404 | clear |

| lazy-depth-drop-all-one-64 | 1178 | -62 | 17520 | -3512 | 350008 | 382008 | clear |

| lazy-depth-take-map-64 | 1620 | -126 | 93064 | -7096 | 366798 | 398798 | clear |

| lazy-depth-fold-through-source-64 | 43334.001 | -62 | 3402649.86 | -3512 | 4767601 | 9215025 | clear |

| lazy-depth-nest-map-64 | 1368 | -62 | 86108 | -3512 | 351829 | 383829 | clear |

| lazy-depth-map-build-256 | 16.001 | -254 | 19735.864 | -14264 | 799818 | 1257802 | clear |

| lazy-depth-map-first-256 | 5396 | -254 | 339528 | -14264 | 394528 | 426528 | clear |

| lazy-depth-map-eight-256 | 43128.001 | -254 | 2584600.86 | -14264 | 1028426 | 1715786 | clear |

| lazy-depth-select-restart-256 | 6279 | -254 | 382036 | -14264 | 396415 | 428415 | clear |

| lazy-depth-drop-all-one-256 | 4634 | -254 | 66864 | -14264 | 393019 | 425019 | clear |

| lazy-depth-take-map-256 | 6420 | -510 | 370504 | -28600 | 452817 | 484817 | clear |

| lazy-depth-fold-through-source-256 | 689320.001 | -254 | 54008857.9 | -14264 | 17104759 | 33840183 | clear |

| lazy-depth-nest-map-256 | 5400 | -254 | 340316 | -14264 | 394840 | 426840 | clear |



| Label | instructions/call 1 | delta 1 % | instructions/call 2 | delta 2 % | Paired instruction gate |

| --- | --- | --- | --- | --- | --- |

| lazy-range-take-small | 42698.601 | 1.99334282 | 42694.2625 | 2.13255343 | REGRESSION |

| lazy-range-normal-small | 54993.1876 | 2.49245745 | 55029.5842 | 2.42957396 | REGRESSION |

| lazy-map-realize-small | 180214.84 | 1.06418619 | 180278.158 | 1.09315875 | REGRESSION |

| lazy-select-realize-small | 115726.81 | 1.30877675 | 115758.936 | 1.31348837 | REGRESSION |

| lazy-select-scan-small | 801405.191 | 0.298458533 | 801498.097 | 0.311566933 | REGRESSION |

| lazy-nestlist-realize-small | 146750.306 | 1.09476282 | 146683.726 | 1.05750601 | REGRESSION |

| lazy-foldlist-realize-small | 115095.202 | 1.76497694 | 115013.569 | 1.70315694 | REGRESSION |

| lazy-drop-realize-small | 193029.702 | 0.904317808 | 193003.217 | 0.849001511 | REGRESSION |

| lazy-first-small | 41648.8568 | 0.54642422 | 41650.2018 | 0.667124449 | REGRESSION |

| lazy-part-small | 89078.374 | 0.795473432 | 89383.2781 | 1.14041514 | REGRESSION |

| lazy-transparent-adapters-small | 183794.571 | 0.76428597 | 183553.405 | 0.769751855 | REGRESSION |

| lazy-table-construct-short | 23626.6043 | 0.187856756 | 23558.7272 | -0.102683749 | clear |

| lazy-table-construct-full | 24065.7742 | 0.0403180896 | 24097.7548 | 0.354014069 | clear |

| lazy-table-realize-small | 117964.484 | 1.1023618 | 117985.558 | 1.16464066 | REGRESSION |

| lazy-permutations-take-small | 28620.1785 | 1.39641101 | 28619.8891 | 1.38837374 | REGRESSION |

| lazy-subsets-take-small | 25909.7834 | 1.7110595 | 25909.9343 | 1.45671478 | REGRESSION |

| lazy-tuples-take-small | 27003.536 | 1.36959855 | 27093.3796 | 1.55277558 | REGRESSION |

| scan-no-input | 29996.8586 | -0.046803987 | 29996.988 | -0.0365714601 | clear |

| optionvalue-three-no-input | 9747.3977 | 0.0876493609 | 9747.25215 | 0.0891830389 | REGRESSION |

| replacerepeated-no-input | 40125.769 | 0.0197900785 | 40126.0096 | 0.0205113539 | clear |

| dot-three-no-input | 256835.733 | 0.148984305 | 256836.652 | 0.149617676 | REGRESSION |

| differences-no-input | 9987.20335 | -0.0253184955 | 9986.15905 | -0.0443027103 | clear |

| do-count-null | 496.8989 | -0.0102827246 | 497.00505 | 0.0313676665 | clear |

| do-iterator-null | 1137.0533 | -0.00755404439 | 1137.05695 | 0.0145044554 | clear |

| table-iterator-null | 349.00635 | 0.0403737257 | 348.9652 | -0.0225473071 | clear |

| replaceall-simple | 11403.2386 | -0.00130969849 | 11403.282 | -0.38556855 | clear |

| table-small | 14741.1945 | -0.102467656 | 14725.3654 | -0.21063572 | clear |

| table-iterator-plus | 565.23765 | -0.0164859215 | 565.13045 | 0.00604321777 | clear |

| table-iterator-pair-list | 1726.64475 | 0.00110620388 | 1726.6539 | -0.00668589583 | clear |

| take-multispec-small | 51109.9964 | 0.646502292 | 51109.6045 | 0.631782184 | REGRESSION |

| lazy-depth-map-build-8 | 19558.835 | 0.55656438 | 19571.863 | 0.533056997 | REGRESSION |

| lazy-depth-map-first-8 | 132026.862 | 0.272928391 | 132029.01 | 0.415871461 | REGRESSION |

| lazy-depth-map-eight-8 | 962219.553 | 0.0707024354 | 962156.136 | 0.0480791306 | clear |

| lazy-depth-select-restart-8 | 180549.714 | 0.438700584 | 180566.554 | 0.197545587 | REGRESSION |

| lazy-depth-drop-all-one-8 | 50649.127 | 4.97951393 | 50647.886 | 4.97396527 | REGRESSION |

| lazy-depth-take-map-8 | 142825.316 | 0.987983782 | 142818.219 | 0.978264769 | REGRESSION |

| lazy-depth-fold-through-source-8 | 603490.434 | 0.333159917 | 603486.573 | 0.330786742 | REGRESSION |

| lazy-depth-nest-map-8 | 142805.567 | 0.0699523364 | 142816.753 | 0.427820146 | REGRESSION |

| lazy-depth-map-build-64 | 39817.829 | -4.32638438 | 39800.376 | -4.39987033 | clear |

| lazy-depth-map-first-64 | 956930.886 | 0.0704990591 | 959622.477 | 0.351843665 | REGRESSION |

| lazy-depth-map-eight-64 | 7438896.62 | 0.28531923 | 7438911.34 | 0.288007632 | REGRESSION |

| lazy-depth-select-restart-64 | 979843.64 | -0.113087073 | 979928.301 | 0.160184821 | clear |

| lazy-depth-drop-all-one-64 | 415455.614 | 20.3590325 | 415449.453 | 20.3630705 | REGRESSION |

| lazy-depth-take-map-64 | 1036486.3 | 0.498620925 | 1036445.91 | 0.505826504 | REGRESSION |

| lazy-depth-fold-through-source-64 | 34604612.2 | 0.287020153 | 34605739.1 | 0.29010301 | REGRESSION |

| lazy-depth-nest-map-64 | 969565.641 | 0.0801410633 | 972381.13 | 0.370411682 | REGRESSION |

| lazy-depth-map-build-256 | 108664.868 | -7.65534115 | 108667.949 | -7.63940878 | clear |

| lazy-depth-map-first-256 | 3764842.68 | 0.0167901867 | 3759221.05 | -0.43559443 | clear |

| lazy-depth-map-eight-256 | 29614687.5 | 0.218587893 | 29617627 | 0.230250516 | REGRESSION |

| lazy-depth-select-restart-256 | 3681792.74 | 0.113451186 | 3681937.41 | -0.482775452 | clear |

| lazy-depth-drop-all-one-256 | 3355953.84 | 43.4674161 | 3355958.56 | 43.467532 | REGRESSION |

| lazy-depth-take-map-256 | 4100759.98 | 0.332661608 | 4100646.09 | 0.343146015 | REGRESSION |

| lazy-depth-fold-through-source-256 | 539758490 | missing | 539766070 | 0.282345446 | MISSING BASE PAIR |

| lazy-depth-nest-map-256 | 3770163.36 | -0.138662901 | 3775804.56 | 0.0108006661 | clear |



#### candidate-3 — all 54 allocation and instruction rows


Allocation/byte regressions: **0**. Paired instruction regressions: **24**. Worst repeatable instruction increase (minimum of the pair): **2.515150614%**. Full precision: [candidate-3/metrics-comparison.tsv](../../../.temp/brief-563/candidate-3/metrics-comparison.tsv).


| Label | alloc/call | delta alloc | bytes/call | delta bytes | peak N | peak 2N | Allocation gate |

| --- | --- | --- | --- | --- | --- | --- | --- |

| lazy-range-take-small | 141 | 0 | 2714 | 0 | 11428045 | 12068045 | clear |

| lazy-range-normal-small | 157 | 0 | 3388 | -56 | 11428491 | 12068491 | clear |

| lazy-map-realize-small | 333 | 0 | 13740 | -40 | 11429646 | 12069646 | clear |

| lazy-select-realize-small | 236 | 0 | 6834 | -40 | 9594202 | 10234202 | clear |

| lazy-select-scan-small | 1471 | 0 | 62793 | -56 | 9595818 | 10235818 | clear |

| lazy-nestlist-realize-small | 197 | 0 | 11316 | -56 | 11429099 | 12069099 | clear |

| lazy-foldlist-realize-small | 216 | 0 | 7604 | -40 | 11429094 | 12069094 | clear |

| lazy-drop-realize-small | 351 | -1 | 14204 | -96 | 9595465 | 10235465 | clear |

| lazy-first-small | 64 | 0 | 2868 | -56 | 9595621 | 10235621 | clear |

| lazy-part-small | 152 | 0 | 6182 | -40 | 9596439 | 10236439 | clear |

| lazy-transparent-adapters-small | 319 | -3 | 12876 | -208 | 949239 | 1589239 | clear |

| lazy-table-construct-short | 63 | 0 | 1537 | 0 | 942784 | 1582784 | clear |

| lazy-table-construct-full | 64 | 0 | 1449 | 0 | 942664 | 1582664 | clear |

| lazy-table-realize-small | 251 | 0 | 9190 | -40 | 10053648 | 10693648 | clear |

| lazy-permutations-take-small | 29 | 0 | 1729 | 0 | 9593774 | 10233774 | clear |

| lazy-subsets-take-small | 29 | 0 | 1417 | 0 | 8446869 | 9086869 | clear |

| lazy-tuples-take-small | 29 | 0 | 1361 | 0 | 8905896 | 9545896 | clear |

| scan-no-input | 33 | 0 | 2536 | 0 | 942414 | 1582414 | clear |

| optionvalue-three-no-input | 9 | 0 | 1128 | 0 | 9134068 | 9774068 | clear |

| replacerepeated-no-input | 61 | 0 | 7691 | 0 | 948049 | 1588049 | clear |

| dot-three-no-input | 596 | 0 | 23504 | 0 | 8678236 | 9318236 | clear |

| differences-no-input | 13 | 0 | 704 | 0 | 9591399 | 10231399 | clear |

| do-count-null | 0 | 0 | 0 | 0 | 299668 | 299668 | clear |

| do-iterator-null | 0 | 0 | 32 | 0 | 940146 | 1580146 | clear |

| table-iterator-null | 0 | 0 | 32 | 0 | 940025 | 1580025 | clear |

| replaceall-simple | 5 | 0 | 1352 | 0 | 943388 | 1583388 | clear |

| table-small | 22 | 0 | 977 | 0 | 9132887 | 9772887 | clear |

| table-iterator-plus | 2 | 0 | 72 | 0 | 940209 | 1580209 | clear |

| table-iterator-pair-list | 2 | 0 | 136 | 0 | 8574075 | 11294075 | clear |

| take-multispec-small | 109 | 0 | 5856 | 0 | 9594023 | 10234023 | clear |

| lazy-depth-map-build-8 | 16.001 | -6 | 1879.864 | -376 | 762116 | 1220100 | clear |

| lazy-depth-map-first-8 | 188 | -6 | 11176 | -376 | 338970 | 370970 | clear |

| lazy-depth-map-eight-8 | 1464.001 | -6 | 82776.864 | -376 | 990724 | 1678084 | clear |

| lazy-depth-select-restart-8 | 327 | -6 | 16980 | -376 | 340857 | 372857 | clear |

| lazy-depth-drop-all-one-8 | 170 | -6 | 3128 | -376 | 337461 | 369461 | clear |

| lazy-depth-take-map-8 | 220 | -14 | 12144 | -824 | 341707 | 373707 | clear |

| lazy-depth-fold-through-source-8 | 743.001 | -6 | 58440.864 | -376 | 1219923 | 2136659 | clear |

| lazy-depth-nest-map-8 | 192 | -6 | 11964 | -376 | 339282 | 371282 | clear |

| lazy-depth-map-build-64 | 16.001 | -62 | 5911.864 | -3512 | 770631 | 1228615 | clear |

| lazy-depth-map-first-64 | 1364 | -62 | 85320 | -3512 | 351517 | 383517 | clear |

| lazy-depth-map-eight-64 | 10872.001 | -62 | 647704.864 | -3512 | 999239 | 1686599 | clear |

| lazy-depth-select-restart-64 | 1671 | -62 | 99412 | -3512 | 353404 | 385404 | clear |

| lazy-depth-drop-all-one-64 | 1178 | -62 | 17520 | -3512 | 350008 | 382008 | clear |

| lazy-depth-take-map-64 | 1620 | -126 | 93064 | -7096 | 366798 | 398798 | clear |

| lazy-depth-fold-through-source-64 | 43334.001 | -62 | 3402649.86 | -3512 | 4767601 | 9215025 | clear |

| lazy-depth-nest-map-64 | 1368 | -62 | 86108 | -3512 | 351829 | 383829 | clear |

| lazy-depth-map-build-256 | 16.001 | -254 | 19735.864 | -14264 | 799818 | 1257802 | clear |

| lazy-depth-map-first-256 | 5396 | -254 | 339528 | -14264 | 394528 | 426528 | clear |

| lazy-depth-map-eight-256 | 43128.001 | -254 | 2584600.86 | -14264 | 1028426 | 1715786 | clear |

| lazy-depth-select-restart-256 | 6279 | -254 | 382036 | -14264 | 396415 | 428415 | clear |

| lazy-depth-drop-all-one-256 | 4634 | -254 | 66864 | -14264 | 393019 | 425019 | clear |

| lazy-depth-take-map-256 | 6420 | -510 | 370504 | -28600 | 452817 | 484817 | clear |

| lazy-depth-fold-through-source-256 | 689320.001 | -254 | 54008857.9 | -14264 | 17104759 | 33840183 | clear |

| lazy-depth-nest-map-256 | 5400 | -254 | 340316 | -14264 | 394840 | 426840 | clear |



| Label | instructions/call 1 | delta 1 % | instructions/call 2 | delta 2 % | Paired instruction gate |

| --- | --- | --- | --- | --- | --- |

| lazy-range-take-small | 42800.4957 | 2.23673665 | 42859.1643 | 2.52702896 | REGRESSION |

| lazy-range-normal-small | 54713.1347 | 1.97051454 | 54735.2087 | 1.88163674 | REGRESSION |

| lazy-map-realize-small | 179649.552 | 0.747173972 | 179649.717 | 0.740752913 | REGRESSION |

| lazy-select-realize-small | 115389.396 | 1.01340082 | 115392.153 | 0.992475652 | REGRESSION |

| lazy-select-scan-small | 800733.659 | 0.2144141 | 800712.715 | 0.213272391 | REGRESSION |

| lazy-nestlist-realize-small | 146390.007 | 0.846556927 | 146411.129 | 0.869700694 | REGRESSION |

| lazy-foldlist-realize-small | 114654.552 | 1.37536237 | 114618.56 | 1.35386178 | REGRESSION |

| lazy-drop-realize-small | 192455.676 | 0.604251985 | 192480.094 | 0.575656755 | REGRESSION |

| lazy-first-small | 41727.4267 | 0.736103436 | 41795.4474 | 1.01817838 | REGRESSION |

| lazy-part-small | 88954.3696 | 0.655157755 | 89063.7894 | 0.778902111 | REGRESSION |

| lazy-transparent-adapters-small | 182817.333 | 0.228521319 | 183006.55 | 0.469531393 | REGRESSION |

| lazy-table-construct-short | 23631.0171 | 0.206569304 | 23558.213 | -0.104864138 | clear |

| lazy-table-construct-full | 24088.2371 | 0.133695125 | 24032.8952 | 0.0839091864 | REGRESSION |

| lazy-table-realize-small | 117694.331 | 0.870825369 | 117774.421 | 0.983604957 | REGRESSION |

| lazy-permutations-take-small | 28638.4814 | 1.46125524 | 28638.0761 | 1.4528029 | REGRESSION |

| lazy-subsets-take-small | 25882.5459 | 1.60413657 | 25882.1067 | 1.34774878 | REGRESSION |

| lazy-tuples-take-small | 27013.1219 | 1.40558361 | 27067.0164 | 1.45395951 | REGRESSION |

| scan-no-input | 29993.9894 | -0.0563643452 | 29994.0056 | -0.0465103222 | clear |

| optionvalue-three-no-input | 9738.86755 | 6.05820291e-05 | 9738.63525 | 0.000700821794 | clear |

| replacerepeated-no-input | 40118.0254 | 0.000487937662 | 40117.8469 | 0.000164640213 | clear |

| dot-three-no-input | 256344.199 | -0.0426817855 | 256343.979 | -0.0424929986 | clear |

| differences-no-input | 9987.5579 | -0.0217693515 | 9986.38595 | -0.0420315721 | clear |

| do-count-null | 497.2196 | 0.0542509307 | 496.99335 | 0.0290128272 | clear |

| do-iterator-null | 1137.1218 | -0.00153015567 | 1137.11985 | 0.0200370827 | clear |

| table-iterator-null | 349.07585 | 0.0602954434 | 349.1366 | 0.0265582639 | clear |

| replaceall-simple | 11403.3972 | 8.06777775e-05 | 11403.3667 | -0.384828646 | clear |

| table-small | 14741.2221 | -0.102280617 | 14789.4528 | 0.223665278 | clear |

| table-iterator-plus | 565.189 | -0.0250915017 | 565.2462 | 0.0265264522 | clear |

| table-iterator-pair-list | 1726.7266 | 0.00584666398 | 1726.69775 | -0.00414647156 | clear |

| take-multispec-small | 51115.2087 | 0.656766424 | 51115.5425 | 0.643473754 | REGRESSION |

| lazy-depth-map-build-8 | 19562.562 | 0.575725762 | 19558.884 | 0.466388916 | REGRESSION |

| lazy-depth-map-first-8 | 132038.406 | 0.28169593 | 131689.855 | 0.157924023 | REGRESSION |

| lazy-depth-map-eight-8 | 959580.749 | -0.203732821 | 959581.081 | -0.219683342 | clear |

| lazy-depth-select-restart-8 | 179671.547 | -0.049818342 | 179622.723 | -0.326192324 | clear |

| lazy-depth-drop-all-one-8 | 49464.478 | 2.52411373 | 49461.556 | 2.51515061 | REGRESSION |

| lazy-depth-take-map-8 | 142251.971 | 0.582586776 | 142256.795 | 0.581315264 | REGRESSION |

| lazy-depth-fold-through-source-8 | 602197.664 | 0.118230746 | 602179.92 | 0.113553203 | REGRESSION |

| lazy-depth-nest-map-8 | 142476.173 | -0.16086809 | 142478.916 | 0.190255345 | clear |

| lazy-depth-map-build-64 | 39800.145 | -4.3688752 | 39788.104 | -4.42934756 | clear |

| lazy-depth-map-first-64 | 954250.653 | -0.209784771 | 954245.312 | -0.210469572 | clear |

| lazy-depth-map-eight-64 | 7417042.31 | -0.00930330137 | 7417531.74 | -0.000222405025 | clear |

| lazy-depth-select-restart-64 | 976111.193 | -0.493579014 | 976564.926 | -0.183591618 | clear |

| lazy-depth-drop-all-one-64 | 348581.257 | 0.985283185 | 348580.473 | 0.98994171 | REGRESSION |

| lazy-depth-take-map-64 | 1031779.95 | 0.0422890989 | 1031835.05 | 0.0587046723 | clear |

| lazy-depth-fold-through-source-64 | 34527231.3 | 0.0627639403 | 34527161.2 | 0.0623781873 | REGRESSION |

| lazy-depth-nest-map-64 | 966869.313 | -0.198178295 | 965461.015 | -0.343891352 | clear |

| lazy-depth-map-build-256 | 108670.778 | -7.65031877 | 108664.309 | -7.64250254 | clear |

| lazy-depth-map-first-256 | 3759637.63 | -0.121487249 | 3754024.6 | -0.573224526 | clear |

| lazy-depth-map-eight-256 | 29530349 | -0.06682071 | 29530973 | -0.0629991064 | clear |

| lazy-depth-select-restart-256 | 3666118.58 | -0.312752586 | 3656134.72 | -1.18018329 | clear |

| lazy-depth-drop-all-one-256 | 2301994.84 | -1.58945351 | 2302012.49 | -1.58875765 | clear |

| lazy-depth-take-map-256 | 4082588.63 | -0.111934082 | 4083116.89 | -0.0857948481 | clear |

| lazy-depth-fold-through-source-256 | 538536608 | missing | 538568716 | 0.059890836 | MISSING BASE PAIR |

| lazy-depth-nest-map-256 | 3770686.82 | -0.124797901 | 3765043.34 | -0.274234804 | clear |



Candidate 1’s geometric stage growth produced repeated byte-slope regressions, including deep stages despite fewer allocation calls. Candidate 2 removed all allocation/byte regressions but worsened the deep Drop instruction cost. Candidate 3 removed most of that transition cost: worst repeatable regression fell from +43.467416148% to +2.515150614%, and paired regressing labels fell from 34 to 24. The remaining shallow/source and neighbor regressions are still defects under the +0.05% rule, including take-multispec-small and lazy-table-construct-full. No fourth tuning candidate, assembly-guided retry, or selected-label remeasurement was performed. Peak values for both N sizes are exposed in every allocation table; they do not establish a retention bound.


#### Required complete timing A/B — exactly two runs


Both runs passed and emitted 161 matching label/iteration rows. [core-language-microbench.sh](../../../.temp/brief-563/core-language-microbench.sh) is the scratch copy: original suite unchanged, repository root anchored, 17 exact lazy statements plus 24 exact depth statements appended with setup outside repetition. [perf/timing-harness.json](../../../.temp/brief-563/perf/timing-harness.json) records original/scratch hashes and the 900s inner bound calibration. Each run used one CLI process, seven internal repetitions, scale 1, explicit preserved --rum-cli, --output-dir, --timeout 900 and --build-timeout 300, pinned to CPU 2. The outer runner bound is 960s. Only in-kernel AbsoluteTiming rows below are compared; process elapsed fields are bounds/provenance only. Timing informs and does not override instruction failures.


| Label | Iterations | Base seconds | Candidate 3 seconds | Delta % (informational) |

| --- | --- | --- | --- | --- |

| association-part-key-read-width4000 | 40 | 0.00146258 | 0.00110387 | -24.5258379 |

| association-part-key-read-width32000 | 40 | 0.00863645 | 0.00768755 | -10.9871533 |

| association-part-positional-read-width4000 | 40 | 3.076e-05 | 3.1073e-05 | 1.01755527 |

| association-part-positional-read-width32000 | 40 | 3.1016e-05 | 3.1481e-05 | 1.49922621 |

| loop-empty | 5000 | 0.000257856 | 0.000257479 | -0.146205634 |

| do-count-null | 10000 | 0.000240995 | 0.000236064 | -2.04610054 |

| do-iterator-null | 1000 | 5.6623e-05 | 5.8443e-05 | 3.21424156 |

| table-count-null | 5000 | 8.3048e-05 | 8.7829e-05 | 5.75691167 |

| table-iterator-null | 1000 | 1.9334e-05 | 2.024e-05 | 4.68604531 |

| table-iterator-symbol | 1000 | 5.973e-06 | 6.9e-06 | 15.5198393 |

| literal-integer | 2000 | 9.4616e-05 | 9.3154e-05 | -1.5451932 |

| literal-string | 2000 | 9.4466e-05 | 9.1235e-05 | -3.4202782 |

| symbol-ownvalue-read | 2000 | 0.000215271 | 0.000234125 | 8.75826284 |

| symbol-ownvalue-guarded-read | 1000 | 0.00017983 | 0.000180948 | 0.621698271 |

| set-ownvalue | 1000 | 0.000264817 | 0.000273725 | 3.36383238 |

| list-literal-small | 1000 | 5.9222e-05 | 6.076e-05 | 2.59700787 |

| head-list | 2000 | 0.000917356 | 0.000947722 | 3.3101653 |

| length-list | 2000 | 0.000924839 | 0.00094903 | 2.61569852 |

| part-list | 2000 | 0.00107511 | 0.00111095 | 3.33361237 |

| sameq-integer | 2000 | 0.000627628 | 0.000658238 | 4.8770928 |

| sameq-list | 1000 | 0.000620864 | 0.000632145 | 1.81698407 |

| if-true | 2000 | 0.000714466 | 0.000746076 | 4.42428331 |

| module-local | 500 | 0.000993502 | 0.000748014 | -24.7093614 |

| module-chain-escape-small | 96 | 6.0445e-05 | 4.8193e-05 | -20.2696666 |

| module-chain-escape-large | 192 | 0.000123287 | 9.9433e-05 | -19.3483498 |

| block-local | 500 | 0.000553792 | 0.000469787 | -15.1690526 |

| block-symbol-only-repeat | 1000 | 0.000337922 | 0.00030862 | -8.67123182 |

| block-init-null-repeat | 1000 | 0.00036268 | 0.00034977 | -3.55961178 |

| block-init-read-repeat | 1000 | 0.000401131 | 0.00040032 | -0.202178341 |

| with-local | 500 | 0.000479945 | 0.000488625 | 1.80854056 |

| with-deep-body-small | 500 | 0.00174378 | 0.00175444 | 0.611315648 |

| table-small | 200 | 0.000187694 | 0.000189346 | 0.880155999 |

| table-iterator-plus | 1000 | 3.0962e-05 | 3.3121e-05 | 6.97306376 |

| table-iterator-pair-list | 1000 | 7.4927e-05 | 7.5207e-05 | 0.373697065 |

| do-small | 200 | 0.000894805 | 0.000910364 | 1.7388146 |

| d-nested-sin-small | 200 | 0.00530864 | 0.00542341 | 2.16194732 |

| downvalue-exact-hit | 2000 | 0.000780591 | 0.000804982 | 3.12468373 |

| downvalue-exact-miss | 2000 | 0.000558094 | 0.000580877 | 4.08228721 |

| downvalue-pattern-hit | 1000 | 0.000961465 | 0.000969324 | 0.817398449 |

| downvalue-pattern-miss | 1000 | 0.00028946 | 0.000301201 | 4.05617356 |

| downvalue-exact-miss-repeat | 5000 | 0.00124637 | 0.00130728 | 4.88699182 |

| downvalue-pattern-miss-repeat | 5000 | 0.00129377 | 0.00134256 | 3.77114943 |

| downvalue-guarded-hit | 1000 | 0.00187968 | 0.00191841 | 2.0604571 |

| downvalue-guarded-decline-fallback | 1000 | 0.00140868 | 0.0014355 | 1.90391004 |

| upvalue-guarded-hit | 1000 | 0.00232592 | 0.00235498 | 1.24939809 |

| subvalue-unguarded-hit | 1000 | 0.00139646 | 0.00140423 | 0.556406915 |

| subvalue-unguarded-miss | 1000 | 0.000771152 | 0.000785315 | 1.8366029 |

| subvalue-guarded-hit | 1000 | 0.00243983 | 0.00244753 | 0.315595759 |

| default-unguarded-hit | 1000 | 0.00129156 | 0.00134687 | 4.28241816 |

| default-unguarded-miss | 1000 | 0.00106808 | 0.00109151 | 2.19365591 |

| default-guarded-hit | 1000 | 0.00163149 | 0.00166304 | 1.9338151 |

| matchq-integer-success | 1000 | 0.000310058 | 0.000323315 | 4.27565165 |

| matchq-integer-fail | 1000 | 0.000314411 | 0.000326773 | 3.93179628 |

| matcher-single-tiny | 1000 | 0.000682123 | 0.000708827 | 3.91483647 |

| matcher-single-wide-small | 800 | 3.9055e-05 | 3.8379e-05 | -1.73089233 |

| matcher-single-wide-large | 1600 | 7.7515e-05 | 7.5113e-05 | -3.09875508 |

| replaceall-simple | 500 | 0.000356093 | 0.000358209 | 0.594226789 |

| rule-rhs-deep-small | 500 | 0.0029869 | 0.00304787 | 2.04124678 |

| rewrite-guarded-hit | 500 | 0.00177323 | 0.00179568 | 1.26605122 |

| rewrite-guarded-decline-fallback | 500 | 0.00250022 | 0.00253828 | 1.52226604 |

| cases-wide-no-match | 1600 | 0.000156602 | 0.000156503 | -0.0632175834 |

| count-wide-no-match | 1600 | 0.000147382 | 0.000152565 | 3.51671167 |

| position-wide-no-match | 1600 | 0.000180661 | 0.000187571 | 3.82484321 |

| deletecases-wide-no-match | 1600 | 0.000167709 | 0.000171068 | 2.00287403 |

| deletecases-wide-association-no-match | 1600 | 0.000428917 | 0.000433356 | 1.03493217 |

| deletecases-leaf-delete-small | 500 | 0.00217513 | 0.00220425 | 1.33877056 |

| replace-wide-no-match | 1600 | 8.4703e-05 | 8.5795e-05 | 1.28921054 |

| replace-leaf-rebuild-small | 500 | 0.00264526 | 0.00267376 | 1.07739882 |

| take-multispec-small | 500 | 0.00169147 | 0.00173458 | 2.54867068 |

| replacepart-deep-path-small | 500 | 0.00130778 | 0.00134275 | 2.67399716 |

| replacepart-wide-rule-list | 1600 | 0.000282896 | 0.000293704 | 3.82048527 |

| replacepart-guarded-hit | 500 | 0.000533128 | 0.000543127 | 1.87553458 |

| replacepart-guarded-decline-fallback | 500 | 0.000588864 | 0.000606759 | 3.03890202 |

| string-repeat-small | 500 | 0.000182951 | 0.000191618 | 4.73733404 |

| character-range-small | 200 | 0.000130532 | 0.000135513 | 3.81592253 |

| stringtrim-literal-list-4000 | 100 | 0.000456433 | 0.000426659 | -6.52319179 |

| stringtrim-literal-list-32000 | 100 | 0.00336881 | 0.00317303 | -5.8115477 |

| stringtrim-single-literal-4000 | 100 | 0.000406982 | 0.000408203 | 0.300013268 |

| stringtrim-single-literal-32000 | 100 | 0.00315368 | 0.00314868 | -0.158544938 |

| stringtrim-whitespace-4000 | 100 | 0.000395386 | 0.000397534 | 0.54326658 |

| stringtrim-whitespace-32000 | 100 | 0.00314299 | 0.00314061 | -0.0757240717 |

| stringreplace-single-literal-unguarded | 200 | 0.000351822 | 0.000353327 | 0.427773135 |

| stringreplace-pattern-unguarded | 500 | 0.00116257 | 0.00117878 | 1.39432464 |

| stringreplace-guarded-hit | 500 | 0.000691359 | 0.000694393 | 0.438845809 |

| stringreplace-guarded-decline-fallback | 500 | 0.000813411 | 0.0008234 | 1.22803847 |

| appendto-list-incremental-small | 250 | 0.00057857 | 0.00045922 | -20.628446 |

| appendto-list-incremental-large | 500 | 0.00179543 | 0.00150638 | -16.0992074 |

| appendto-part-outer1000-small | 100 | 0.00178395 | 0.00178194 | -0.112671319 |

| appendto-part-outer1000-large | 200 | 0.00359565 | 0.00358589 | -0.2714391 |

| association-set-key-incremental-small | 100 | 0.00150634 | 0.00150099 | -0.3551655 |

| association-set-key-incremental-large | 200 | 0.00561672 | 0.00558425 | -0.578095401 |

| association-small-literal-repeat | 500 | 0.000712425 | 0.000749719 | 5.23479665 |

| merge-nested-small | 200 | 0.00115652 | 0.00118308 | 2.29654481 |

| set-part-fixed100-len100 | 100 | 0.000129519 | 0.000131039 | 1.17357299 |

| set-part-fixed100-len800 | 100 | 0.000478749 | 0.000481057 | 0.48208978 |

| set-part-fill-small | 200 | 0.000372778 | 0.000375298 | 0.676005558 |

| set-part-fill-large | 400 | 0.00114069 | 0.00114615 | 0.478657655 |

| upvalue-candidates-tiny | 1000 | 0.0019413 | 0.00199694 | 2.86612064 |

| upvalue-candidates-wide-small | 1600 | 0.000151393 | 0.000155477 | 2.69761482 |

| upvalue-candidates-wide-large | 3200 | 0.000298585 | 0.000309376 | 3.61404625 |

| sort-callback-small | 100 | 0.000168214 | 0.000167059 | -0.68662537 |

| sort-callback-large | 200 | 0.000384184 | 0.000381866 | -0.603356725 |

| sortby-callback-small | 100 | 0.000198156 | 0.000196296 | -0.938654394 |

| sortby-callback-large | 200 | 0.000441591 | 0.000441164 | -0.0966958113 |

| reversesort-callback-small | 100 | 0.000158059 | 0.00015874 | 0.430851771 |

| reversesort-callback-large | 200 | 0.00035419 | 0.000356942 | 0.776984105 |

| keysort-callback-small | 50 | 8.2538e-05 | 8.3453e-05 | 1.10858029 |

| keysort-callback-large | 100 | 0.000183163 | 0.00018307 | -0.0507744468 |

| keysortby-callback-small | 25 | 4.471e-05 | 4.4957e-05 | 0.552449117 |

| keysortby-callback-large | 50 | 9.7235e-05 | 9.7296e-05 | 0.062734612 |

| sort-default-wide | 1600 | 1.6379e-05 | 1.6155e-05 | -1.36760486 |

| sort-symbol-default-width4000 | 4000 | 0.000265495 | 0.000289609 | 9.08265692 |

| sort-symbol-default-width16000 | 16000 | 0.00116895 | 0.00120447 | 3.03862441 |

| sortby-default-wide | 1600 | 0.000495313 | 0.000498316 | 0.6062833 |

| reversesort-default-wide | 1600 | 1.6664e-05 | 1.6737e-05 | 0.438070091 |

| keysort-default-wide | 1600 | 0.000369909 | 0.000372301 | 0.646645526 |

| keysortby-default-wide | 1600 | 0.000872098 | 0.000869013 | -0.353744648 |

| valueq-undefined-symbol-definitions-2000 | 20 | 7.757e-06 | 8.104e-06 | 4.47337888 |

| valueq-undefined-symbol-definitions-8000 | 20 | 7.761e-06 | 8.09e-06 | 4.23914444 |

| module-retained-gensym-gc-batch | 40 | 6.3657e-05 | 6.509e-05 | 2.25112713 |

| lazy-range-take-small | 20000 | 0.0507097 | 0.0517975 | 2.14515172 |

| lazy-range-normal-small | 20000 | 0.0674194 | 0.0692127 | 2.65991688 |

| lazy-map-realize-small | 20000 | 0.234026 | 0.237085 | 1.30711972 |

| lazy-select-realize-small | 20000 | 0.14829 | 0.151143 | 1.92393283 |

| lazy-select-scan-small | 20000 | 1.05697 | 1.06733 | 0.980160269 |

| lazy-nestlist-realize-small | 20000 | 0.193037 | 0.197453 | 2.28764434 |

| lazy-foldlist-realize-small | 20000 | 0.150874 | 0.154044 | 2.10109098 |

| lazy-drop-realize-small | 20000 | 0.254021 | 0.255171 | 0.452718476 |

| lazy-first-small | 20000 | 0.059378 | 0.0592108 | -0.281585773 |

| lazy-part-small | 20000 | 0.126207 | 0.127489 | 1.01579152 |

| lazy-transparent-adapters-small | 20000 | 0.246266 | 0.247225 | 0.389416322 |

| lazy-table-construct-short | 20000 | 0.030907 | 0.030085 | -2.65959168 |

| lazy-table-construct-full | 20000 | 0.0311578 | 0.0302675 | -2.85739044 |

| lazy-table-realize-small | 20000 | 0.155103 | 0.158769 | 2.36359065 |

| lazy-permutations-take-small | 20000 | 0.0377525 | 0.0382956 | 1.43858023 |

| lazy-subsets-take-small | 20000 | 0.0346725 | 0.0356196 | 2.73155959 |

| lazy-tuples-take-small | 20000 | 0.0348272 | 0.0359421 | 3.20123352 |

| lazy-depth-map-build-8 | 1000 | 0.00129528 | 0.00132453 | 2.258199 |

| lazy-depth-map-first-8 | 1000 | 0.00913801 | 0.00914051 | 0.0273582541 |

| lazy-depth-map-eight-8 | 1000 | 0.0650175 | 0.0645158 | -0.771638405 |

| lazy-depth-select-restart-8 | 1000 | 0.0129112 | 0.0127848 | -0.978994981 |

| lazy-depth-drop-all-one-8 | 1000 | 0.00287546 | 0.00293918 | 2.21599327 |

| lazy-depth-take-map-8 | 1000 | 0.00999607 | 0.00997043 | -0.256500805 |

| lazy-depth-fold-through-source-8 | 1000 | 0.0404923 | 0.0402506 | -0.596903609 |

| lazy-depth-nest-map-8 | 1000 | 0.0100081 | 0.0100637 | 0.555550004 |

| lazy-depth-map-build-64 | 1000 | 0.00325135 | 0.00338098 | 3.98695926 |

| lazy-depth-map-first-64 | 1000 | 0.0650682 | 0.0647735 | -0.452909409 |

| lazy-depth-map-eight-64 | 1000 | 0.504042 | 0.499567 | -0.88782284 |

| lazy-depth-select-restart-64 | 1000 | 0.0697234 | 0.0682298 | -2.14217895 |

| lazy-depth-drop-all-one-64 | 1000 | 0.0191465 | 0.0174378 | -8.92434649 |

| lazy-depth-take-map-64 | 1000 | 0.0706461 | 0.0692451 | -1.98312433 |

| lazy-depth-fold-through-source-64 | 1000 | 2.23661 | 2.20757 | -1.29839355 |

| lazy-depth-nest-map-64 | 1000 | 0.0659957 | 0.0654419 | -0.839145581 |

| lazy-depth-map-build-256 | 1000 | 0.00966684 | 0.0107465 | 11.1686963 |

| lazy-depth-map-first-256 | 1000 | 0.258272 | 0.256537 | -0.671772395 |

| lazy-depth-map-eight-256 | 1000 | 1.98926 | 1.96642 | -1.14816565 |

| lazy-depth-select-restart-256 | 1000 | 0.264566 | 0.256493 | -3.0514125 |

| lazy-depth-drop-all-one-256 | 1000 | 0.163888 | 0.0950025 | -42.0320585 |

| lazy-depth-take-map-256 | 1000 | 0.281436 | 0.272538 | -3.16164243 |

| lazy-depth-fold-through-source-256 | 1000 | 35.573 | 35.1966 | -1.05810587 |

| lazy-depth-nest-map-256 | 1000 | 0.259548 | 0.256366 | -1.22597747 |



### 4. Continuations, every standing-duty row, caps and open decisions


The eleven scoped Lost sites are implemented: P1898, P1899, P1901, P1902, P1904, P1906, P1907, P1913 and consumer P1483, P2353, P2355. This supports the brief’s predicted **80 -> 69**, G2 **14 -> 3** outcome; it is not a regenerated global census or a retention reclassification. LazyTable’s three eager-bound sites remain unchanged. First and Take remain N/Pending in the live retention census.


| Inventory | Incoming rows | Final rows | Existing changed | Added | Removed |

| --- | --- | --- | --- | --- | --- |

| host_payloads.tsv | 63 | 63 | 0 | 0 | 0 |

| unknown_attributes.tsv | 1434 | 1443 | 5 | 9 | 0 |

| unknown_captures.tsv | 2938 | 2938 | 29 | 0 | 0 |

| unknown_macros.tsv | 3108 | 3113 | 61 | 5 | 0 |

| unknown_storage.tsv | 88 | 88 | 0 | 0 | 0 |

| self_reaching.tsv | 60 | 59 | 1 | 0 | 1 |



All discovery rows were reconciled from actual emitted output. Host payloads and unknown storage remain byte-identical; all other untouched inventory files are unchanged. There are no new self-reaching types, no new UnknownHostPayload, no untouched-code row, and no status/disposition/exclusion change. The budget derive and PullStepper removal are the named exceptions. Five closures and five vec! rows relocate exactly as authorized. Every actual changed/added/removed ownership row follows; before/after text preserves discovery status and reason.


#### unknown_attributes.tsv


```tsv
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	290	1	unresolved attribute/expansion derive (Clone , Debug , FrameRootTrace)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	295	1	unresolved attribute/expansion derive (Clone , Debug , FrameRootTrace)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	296	1	unresolved attribute/expansion derive (Clone , Debug , FrameRootTrace)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	301	1	unresolved attribute/expansion derive (Clone , Debug , FrameRootTrace)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	303	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default , PartialEq , Eq , FrameRootTrace)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	308	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default , PartialEq , Eq , FrameRootTrace)
authorized budget derive
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	12	1	unresolved attribute/expansion derive (Clone , Copy)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	16	1	unresolved attribute/expansion derive (Clone , Copy , FrameRootTrace)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	4	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	6	1	unresolved attribute/expansion derive (Clone , Copy , Debug , PartialEq , Eq)
ADDED	UnknownConstruct	crates/rum-stdlib/src/associations/basic/lazy_completion.rs	11	1	unresolved attribute/expansion derive (FrameRootTrace , PureFrameAbandon)
ADDED	UnknownConstruct	crates/rum-stdlib/src/lazy/continuation.rs	19	1	unresolved attribute/expansion derive (FrameRootTrace , PureFrameAbandon)
ADDED	UnknownConstruct	crates/rum-stdlib/src/lazy/state.rs	6	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
ADDED	UnknownConstruct	crates/rum-stdlib/src/lazy/state.rs	13	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
ADDED	UnknownConstruct	crates/rum-stdlib/src/lazy/state.rs	39	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
ADDED	UnknownConstruct	crates/rum-stdlib/src/lazy/state.rs	60	1	unresolved attribute/expansion derive (Clone , Copy , FrameRootTrace)
ADDED	UnknownConstruct	crates/rum-stdlib/src/lazy/state.rs	67	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
ADDED	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop/lazy_completion.rs	11	1	unresolved attribute/expansion derive (FrameRootTrace , PureFrameAbandon)
ADDED	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface/lazy_completion.rs	11	1	unresolved attribute/expansion derive (FrameRootTrace , PureFrameAbandon)
```

#### unknown_captures.tsv


```tsv
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	196	28	closure captures: |(entry_key , _)| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	198	28	closure captures: |(entry_key , _)| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	218	33	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	223	47	closure captures: || (capture types/modes unresolved)
authorized relocation
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	133	38	closure captures: |index| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy/source.rs	35	34	closure captures: |index| (capture types/modes unresolved)
authorized relocation
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	152	30	closure captures: |(list , digit)| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy/source.rs	54	26	closure captures: |(list , digit)| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	344	25	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	151	25	closure captures: || (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	348	25	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	155	25	closure captures: || (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	426	15	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	233	15	closure captures: || (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	498	30	closure captures: |items| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	305	30	closure captures: |items| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	498	55	closure captures: |item| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	305	55	closure captures: |item| (capture types/modes unresolved)
authorized relocation
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	658	15	closure captures: |index| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy/source.rs	87	15	closure captures: |index| (capture types/modes unresolved)
authorized relocation
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	725	36	closure captures: |list| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy/descriptor.rs	55	22	closure captures: |list| (capture types/modes unresolved)
authorized relocation
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	730	18	closure captures: |list| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy/descriptor.rs	50	40	closure captures: |list| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	80	25	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	85	32	closure captures: || (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	222	14	closure captures: |expr| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	227	14	closure captures: |expr| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	263	65	closure captures: |limit| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	268	65	closure captures: |limit| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	428	22	closure captures: |arg| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	433	22	closure captures: |arg| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	507	59	closure captures: |index| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	512	59	closure captures: |index| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	514	32	closure captures: |index| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	519	32	closure captures: |index| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	519	33	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	524	33	closure captures: || (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	519	52	closure captures: |index| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	524	52	closure captures: |index| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	528	30	closure captures: |index| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	533	30	closure captures: |index| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	162	29	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	168	29	closure captures: || (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	262	9	closure captures: |total , item| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	268	9	closure captures: |total , item| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	283	13	closure captures: |total , arg| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	289	13	closure captures: |total , arg| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	346	14	closure captures: |symbol| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	352	14	closure captures: |symbol| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	347	25	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	353	25	closure captures: || (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	383	18	closure captures: |entry| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	389	18	closure captures: |entry| (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	384	22	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	390	22	closure captures: || (capture types/modes unresolved)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	385	29	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	391	29	closure captures: || (capture types/modes unresolved)
```

#### unknown_macros.tsv


```tsv
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	63	24	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 73:18)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	65	24	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 75:18)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	100	24	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 105:18)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	102	24	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 107:18)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	164	33	unexpanded macro vec! (ends at 164:50)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	166	33	unexpanded macro vec! (ends at 166:50)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	249	33	unexpanded macro vec! (ends at 249:54)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	254	33	unexpanded macro vec! (ends at 254:54)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	253	5	unexpanded macro format! (ends at 253:93)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	258	5	unexpanded macro format! (ends at 258:93)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	263	5	unexpanded macro format! (ends at 263:95)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	268	5	unexpanded macro format! (ends at 268:95)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	267	5	unexpanded macro format! (ends at 267:86)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	272	5	unexpanded macro format! (ends at 272:86)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	287	12	unexpanded macro matches! (ends at 287:102)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	292	12	unexpanded macro matches! (ends at 292:102)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	311	40	unexpanded macro vec! (ends at 311:51)
AFTER	UnknownConstruct	crates/rum-stdlib/src/associations/basic.rs	316	40	unexpanded macro vec! (ends at 316:51)
authorized relocation
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	163	74	unexpanded macro vec! (ends at 163:93)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy/source.rs	65	78	unexpanded macro vec! (ends at 65:97)
authorized relocation
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	171	67	unexpanded macro vec! (ends at 171:78)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy/machine.rs	67	61	unexpanded macro vec! (ends at 67:72)
authorized relocation
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	179	68	unexpanded macro vec! (ends at 179:87)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy/machine.rs	78	62	unexpanded macro vec! (ends at 78:81)
authorized relocation
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	216	21	unexpanded macro vec! (ends at 216:53)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy/machine.rs	106	61	unexpanded macro vec! (ends at 106:93)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	246	9	unexpanded macro vec! (ends at 246:69)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	53	9	unexpanded macro vec! (ends at 53:69)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	269	9	unexpanded macro vec! (ends at 269:65)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	76	9	unexpanded macro vec! (ends at 76:65)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	292	9	unexpanded macro vec! (ends at 292:51)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	99	9	unexpanded macro vec! (ends at 99:51)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	308	9	unexpanded macro vec! (ends at 308:45)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	115	9	unexpanded macro vec! (ends at 115:45)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	327	9	unexpanded macro vec! (ends at 327:27)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	134	9	unexpanded macro vec! (ends at 134:27)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	360	9	unexpanded macro vec! (ends at 360:26)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	167	9	unexpanded macro vec! (ends at 167:26)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	383	9	unexpanded macro vec! (ends at 383:52)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	190	9	unexpanded macro vec! (ends at 190:52)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	402	9	unexpanded macro vec! (ends at 402:27)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	209	9	unexpanded macro vec! (ends at 209:27)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	453	9	unexpanded macro vec! (ends at 453:45)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	260	9	unexpanded macro vec! (ends at 260:45)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	455	54	unexpanded macro vec! (ends at 455:70)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	262	54	unexpanded macro vec! (ends at 262:70)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	456	50	unexpanded macro vec! (ends at 456:71)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	263	50	unexpanded macro vec! (ends at 263:71)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	459	9	unexpanded macro vec! (ends at 459:54)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	266	9	unexpanded macro vec! (ends at 266:54)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	485	9	unexpanded macro vec! (ends at 485:69)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	292	9	unexpanded macro vec! (ends at 292:69)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	511	45	unexpanded macro vec! (ends at 511:70)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	318	45	unexpanded macro vec! (ends at 318:70)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	525	17	unexpanded macro format! (ends at 525:75)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	332	17	unexpanded macro format! (ends at 332:75)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	532	9	unexpanded macro vec! (ends at 532:28)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	339	9	unexpanded macro vec! (ends at 339:28)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	537	5	unexpanded macro matches! (ends at 540:6)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	344	5	unexpanded macro matches! (ends at 347:6)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	547	9	unexpanded macro vec! (ends at 547:69)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	354	9	unexpanded macro vec! (ends at 354:69)
authorized relocation
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	732	21	unexpanded macro vec! (ends at 732:47)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy/descriptor.rs	57	25	unexpanded macro vec! (ends at 57:51)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	798	41	unexpanded macro vec! (ends at 798:57)
AFTER	UnknownConstruct	crates/rum-stdlib/src/lazy.rs	460	41	unexpanded macro vec! (ends at 460:57)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	48	17	unexpanded macro matches! (ends at 48:49)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	50	17	unexpanded macro matches! (ends at 50:49)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	107	17	unexpanded macro matches! (ends at 107:49)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	112	17	unexpanded macro matches! (ends at 112:49)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	139	76	unexpanded macro vec! (ends at 139:96)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	144	76	unexpanded macro vec! (ends at 144:96)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	140	14	unexpanded macro matches! (ends at 140:46)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	145	14	unexpanded macro matches! (ends at 145:46)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	147	13	unexpanded macro format! (ends at 150:14)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	152	13	unexpanded macro format! (ends at 155:14)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	164	9	unexpanded macro format! (ends at 167:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	169	9	unexpanded macro format! (ends at 172:10)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	183	9	unexpanded macro format! (ends at 186:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	188	9	unexpanded macro format! (ends at 191:10)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	195	9	unexpanded macro format! (ends at 199:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	200	9	unexpanded macro format! (ends at 204:10)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	251	17	unexpanded macro format! (ends at 254:18)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	256	17	unexpanded macro format! (ends at 259:18)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	279	17	unexpanded macro format! (ends at 279:94)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	284	17	unexpanded macro format! (ends at 284:94)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	342	21	unexpanded macro vec! (ends at 347:7)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	347	21	unexpanded macro vec! (ends at 352:7)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	399	5	unexpanded macro debug_assert! (ends at 399:68)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	404	5	unexpanded macro debug_assert! (ends at 404:68)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	400	5	unexpanded macro debug_assert! (ends at 400:47)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	405	5	unexpanded macro debug_assert! (ends at 405:47)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	413	5	unexpanded macro debug_assert! (ends at 413:42)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	418	5	unexpanded macro debug_assert! (ends at 418:42)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	452	20	unexpanded macro vec! (ends at 455:7)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	457	20	unexpanded macro vec! (ends at 460:7)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	501	5	unexpanded macro debug_assert_eq! (ends at 501:41)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	506	5	unexpanded macro debug_assert_eq! (ends at 506:41)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	534	18	unexpanded macro vec! (ends at 534:73)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs	539	18	unexpanded macro vec! (ends at 539:73)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	14	13	unexpanded macro vec! (ends at 14:45)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	16	13	unexpanded macro vec! (ends at 16:45)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	97	8	unexpanded macro matches! (ends at 97:35)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	99	8	unexpanded macro matches! (ends at 99:35)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	325	13	unexpanded macro format! (ends at 328:14)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	331	13	unexpanded macro format! (ends at 334:14)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	337	13	unexpanded macro format! (ends at 340:14)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	343	13	unexpanded macro format! (ends at 346:14)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	417	9	unexpanded macro format! (ends at 420:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	423	9	unexpanded macro format! (ends at 426:10)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	433	9	unexpanded macro format! (ends at 437:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	439	9	unexpanded macro format! (ends at 443:10)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	439	37	unexpanded macro vec! (ends at 439:55)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	445	37	unexpanded macro vec! (ends at 445:55)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	451	9	unexpanded macro format! (ends at 454:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	457	9	unexpanded macro format! (ends at 460:10)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	456	37	unexpanded macro vec! (ends at 456:55)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	462	37	unexpanded macro vec! (ends at 462:55)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	460	48	unexpanded macro vec! (ends at 460:84)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	466	48	unexpanded macro vec! (ends at 466:84)
position
BEFORE	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	463	9	unexpanded macro vec! (ends at 463:64)
AFTER	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface.rs	469	9	unexpanded macro vec! (ends at 469:64)
ADDED	UnknownConstruct	crates/rum-stdlib/src/associations/basic/lazy_completion.rs	47	5	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 52:6)
ADDED	UnknownConstruct	crates/rum-stdlib/src/lazy/continuation.rs	42	9	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 48:11)
ADDED	UnknownConstruct	crates/rum-stdlib/src/lazy/machine.rs	119	25	unexpanded macro unreachable! (ends at 119:83)
ADDED	UnknownConstruct	crates/rum-stdlib/src/structure_ops/sequence/take_drop/lazy_completion.rs	47	5	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 52:6)
ADDED	UnknownConstruct	crates/rum-stdlib/src/structure_ops/surface/lazy_completion.rs	51	5	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 56:6)
```

#### self_reaching.tsv


```tsv
position
BEFORE	rum_stdlib::associations::basic::LookupOutcome	crates/rum-stdlib/src/associations/basic.rs	297	rum_stdlib::associations::basic::LookupOutcome.List.0/std::vec::Vec[Own]/arg0 -> rum_stdlib::associations::basic::LookupOutcome
AFTER	rum_stdlib::associations::basic::LookupOutcome	crates/rum-stdlib/src/associations/basic.rs	302	rum_stdlib::associations::basic::LookupOutcome.List.0/std::vec::Vec[Own]/arg0 -> rum_stdlib::associations::basic::LookupOutcome
REMOVED	rum_stdlib::lazy::PullStepper	crates/rum-stdlib/src/lazy.rs	57	rum_stdlib::lazy::PullStepper.Drop.inner/std::boxed::Box[Own]/arg0 -> rum_stdlib::lazy::PullStepper
```

#### Retention fingerprints — all 68


| ID | Status (unchanged) | Before fingerprint | After fingerprint |

| --- | --- | --- | --- |

| B01 | Pending | fnv1a64:e847dab3db2c8f18 | fnv1a64:f41266250be4dbce |

| B02 | Pending | fnv1a64:02fa770079172d65 | fnv1a64:c856afdf4de15b6b |

| B03 | Discharged | fnv1a64:536c48b12439253e | fnv1a64:53913e4672f8a004 |

| B04 | Discharged | fnv1a64:a2cd08dbaa4fefc0 | fnv1a64:db662617fd47eae6 |

| B05 | Discharged | fnv1a64:b16630a2c6210a79 | fnv1a64:0c857ad3ff015b97 |

| B06 | Pending | fnv1a64:8a4fe47bf0331664 | fnv1a64:b230b407776132b2 |

| B07 | Pending | fnv1a64:03fba078e8e7d549 | fnv1a64:ac0bd4a0a104fb07 |

| B08 | Pending | fnv1a64:869b403a5487ba12 | fnv1a64:7f9794c3cc82b110 |

| B09 | Discharged | fnv1a64:e9bbad261133b22c | fnv1a64:a386232773893b2a |

| B10 | Discharged | fnv1a64:9f268f41407d0a93 | fnv1a64:53947051c9e32cc1 |

| B11 | Discharged | fnv1a64:6424ba38c44415c0 | fnv1a64:9cbdd775173c10e6 |

| B12 | Discharged | fnv1a64:a8c82d21ed3c472c | fnv1a64:6292a3234f91d02a |

| I01 | Discharged | fnv1a64:c629c5b147e48598 | fnv1a64:d1f45122789cd24e |

| I02 | Discharged | fnv1a64:08a7ae36be579279 | fnv1a64:63c6f867f737e397 |

| I03 | Pending | fnv1a64:8afdec4d12de58ec | fnv1a64:6b92d68d8b83426a |

| I04 | Discharged | fnv1a64:a2dea75aa7dffd05 | fnv1a64:407a10927e28c30b |

| P01 | Discharged | fnv1a64:0c9b34058a180884 | fnv1a64:a8ccc749aca9b752 |

| P02 | Discharged | fnv1a64:b025b60c238f8583 | fnv1a64:533039502a64cf91 |

| P03 | Discharged | fnv1a64:5535fa242f311d76 | fnv1a64:ced214f9f4b39b2c |

| P04 | Discharged | fnv1a64:f7d59515fdc04965 | fnv1a64:bd31cdf4d28a776b |

| P05 | Discharged | fnv1a64:a5356c0e6ed10808 | fnv1a64:d01fd8c50823351e |

| P06 | Discharged | fnv1a64:f94751f2796be0ac | fnv1a64:b311c7f3dbc169aa |

| P070 | Discharged | fnv1a64:3394ee3457e15aac | fnv1a64:ed5f6435ba36e3aa |

| P080 | Discharged | fnv1a64:4a30ce5d7f170553 | fnv1a64:dc1eac50d2e7b601 |

| P090 | Discharged | fnv1a64:e003e2e49ef49125 | fnv1a64:7cae23c6af24cbab |

| P100 | Discharged | fnv1a64:250c79c75ce71d90 | fnv1a64:7046635d6a42e696 |

| P110 | Discharged | fnv1a64:e5dd6284de96406b | fnv1a64:ea2a37d1fd524ae9 |

| P120 | Discharged | fnv1a64:595714ec677f32b2 | fnv1a64:a474d5f9365d65b0 |

| P130 | Discharged | fnv1a64:a7c8f9d6179baf49 | fnv1a64:4fd92dfdcfb8d507 |

| P140 | Discharged | fnv1a64:72a6c1dbe6d9bf1d | fnv1a64:ae07783beb92d0f3 |

| P150 | Discharged | fnv1a64:95fe9cd2f5233fc5 | fnv1a64:ad5d10540abaae4b |

| P071 | Discharged | fnv1a64:3152c9c6b67879e9 | fnv1a64:303553daf0ec0527 |

| P081 | Discharged | fnv1a64:f6221b0eb106564c | fnv1a64:dc1899ffaf676d4a |

| P091 | Discharged | fnv1a64:f95896cb8b412944 | fnv1a64:3c4bfaee043f5b92 |

| P101 | Discharged | fnv1a64:dde0535213f9dd6b | fnv1a64:e22d289f32b5e7e9 |

| P111 | Discharged | fnv1a64:8c0340849314457c | fnv1a64:5247ede67aee215a |

| P121 | Discharged | fnv1a64:c681b38e370ecaaf | fnv1a64:53405225cdd07a45 |

| P131 | Discharged | fnv1a64:40772b41e0a41140 | fnv1a64:7910487e339c0c66 |

| P141 | Discharged | fnv1a64:eeca56f9f23c4b24 | fnv1a64:9cae93c2d773e0f2 |

| P151 | Discharged | fnv1a64:8dffe1befab74ef2 | fnv1a64:75c66ed13bee3c70 |

| B13 | Discharged | fnv1a64:1860780520cfd264 | fnv1a64:40414790a7fdeeb2 |

| R01 | Discharged | fnv1a64:b40b2659c202a2b5 | fnv1a64:66e64b15456d22db |

| R02 | Discharged | fnv1a64:2a66171755ddab36 | fnv1a64:f1e99079697abe6c |

| R03 | Discharged | fnv1a64:08842d6cdd77f811 | fnv1a64:fc39154afe19853f |

| R040 | Discharged | fnv1a64:bdec1f4cce3e7fd5 | fnv1a64:59b27b7565537cfb |

| R050 | Discharged | fnv1a64:e77180fee7d55fe8 | fnv1a64:79cb05bd575c1e7e |

| R060 | Discharged | fnv1a64:208b8d6b2c66d0ad | fnv1a64:a2a530f5d974a563 |

| R070 | Discharged | fnv1a64:5e8fe32892a8154c | fnv1a64:4486621991092c4a |

| R080 | Discharged | fnv1a64:69e3104d7764c605 | fnv1a64:077e79854dad8c0b |

| R090 | Discharged | fnv1a64:9c60bdc7a5f8f650 | fnv1a64:3990d5582e3de3d6 |

| R100 | Discharged | fnv1a64:1891ed03ce0ecfbe | fnv1a64:18b6e2991cce4a84 |

| R110 | Discharged | fnv1a64:16749042075192fb | fnv1a64:c737879f77b37359 |

| R041 | Discharged | fnv1a64:fe99db2f37f79164 | fnv1a64:267aaababf25adb2 |

| R051 | Discharged | fnv1a64:6e2610fa2c229da5 | fnv1a64:0ad051dc3c52d82b |

| R061 | Discharged | fnv1a64:f74e2ce0ab92d502 | fnv1a64:146d3f3ff02c8660 |

| R071 | Discharged | fnv1a64:7eb33be3bee52837 | fnv1a64:ca07f290c7f0427d |

| R081 | Discharged | fnv1a64:e0f6e31bf892f290 | fnv1a64:2c30ccb205eebb96 |

| R091 | Discharged | fnv1a64:442d70b2f05b025d | fnv1a64:abdbbf5a872573b3 |

| R101 | Discharged | fnv1a64:fd71aee6201a856d | fnv1a64:42457c834e0f23a3 |

| R111 | Discharged | fnv1a64:a771e9ef13341c1a | fnv1a64:23294d551033c028 |

| R12 | Discharged | fnv1a64:017f8b2339b1185a | fnv1a64:8e741b8483e224e8 |

| R13 | Discharged | fnv1a64:31240b9a02357eae | fnv1a64:663b0a3e8e3426d4 |

| R14 | Discharged | fnv1a64:3f64883d3997dd90 | fnv1a64:8a9e71d346f3a696 |

| R15 | Discharged | fnv1a64:06f4d76f31366454 | fnv1a64:e1995c5912796bc2 |

| O010 | Discharged | fnv1a64:11302f9751a8afe2 | fnv1a64:59a6e5433fa88440 |

| O0200 | Discharged | fnv1a64:6d6e63b03590e51d | fnv1a64:a8cf1a103a49f6f3 |

| O011 | Discharged | fnv1a64:631449f339f8952d | fnv1a64:e52ded7de70669e3 |

| O0210 | Discharged | fnv1a64:75acb080596c5c3a | fnv1a64:affdbfdbcda4db48 |



All first 18 contract fields are identical; 62 Discharged and 6 Pending remain unchanged. The existing maintenance replay generated the review file before installation. No new contract or coverage claim was added.


#### C01/C02 certificate cells — actual default/no-default replay


C01, column 5, hash feature true:

```tsv
BEFORE	instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:f37f19d5e4aebc9d", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:4f0c0637b6a9b610;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:75acb080596c5c3a;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:631449f339f8952d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:f37f19d5e4aebc9d", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:0d839ce1a03714de;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:6d6e63b03590e51d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:11302f9751a8afe2;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
AFTER	instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:4c3bb00fc206576b", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:aa197d00ffff920b;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:affdbfdbcda4db48;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:e52ded7de70669e3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:4c3bb00fc206576b", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:814ec67beb7232dd;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:a8cf1a103a49f6f3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:59a6e5433fa88440;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
```

C01, column 6, hash feature false:

```tsv
BEFORE	instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:fb03941ca2fb81e7", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:d1154d15793c3300;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:75acb080596c5c3a;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:631449f339f8952d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:fb03941ca2fb81e7", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:016fe9fb8e58deb6;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:6d6e63b03590e51d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:11302f9751a8afe2;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
AFTER	instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:0a735865da4dfa51", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:a3a0d68586e8a973;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>": "O0210@fnv1a64:affdbfdbcda4db48;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>": "O011@fnv1a64:e52ded7de70669e3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"} || instance=Instance { body: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol", key: "rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>", mode: "rum_runtime::runtime::accounting_mode::counter::Unarmed", reentry: "rum_stdlib::option_consumers::OptionConsumer", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:0a735865da4dfa51", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:0d5c4f04b7ffc561;direct-covered={};direct-contracts={"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>": "O0200@fnv1a64:a8cf1a103a49f6f3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>": "O010@fnv1a64:59a6e5433fa88440;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
```

C02, column 5, hash feature true:

```tsv
BEFORE	instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:9aeb80ab86ca0b13", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:eb7b59ad8f3c97ff;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:c629c5b147e48598;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:08a7ae36be579279;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:a2dea75aa7dffd05;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:b40b2659c202a2b5;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:2a66171755ddab36;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:1860780520cfd264;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:b16630a2c6210a79;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
AFTER	instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:4f5f2173b3727b69", domain_fingerprint: "fnv1a64:f1b1b5ec1f9ab7c9" };mark=fnv1a64:f14372f76ba45159;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:d1f45122789cd24e;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:63c6f867f737e397;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:407a10927e28c30b;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:66e64b15456d22db;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:f1e99079697abe6c;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:40414790a7fdeeb2;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:0c857ad3ff015b97;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
```

C02, column 6, hash feature false:

```tsv
BEFORE	instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:c7a03be7357f262d", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:7a8ba7190be57c2e;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:c629c5b147e48598;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:08a7ae36be579279;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:a2dea75aa7dffd05;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:b40b2659c202a2b5;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:2a66171755ddab36;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:1860780520cfd264;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:b16630a2c6210a79;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
AFTER	instance=Instance { body: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", key: "rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr", mode: "runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow", reentry: "none (nongeneric C02)", build: "target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None", body_fingerprint: "fnv1a64:892ccb7ae02ee93f", domain_fingerprint: "fnv1a64:ed48860e2b2c615f" };mark=fnv1a64:a7f011ceccc88c23;direct-covered={};direct-contracts={"<rum_core::expr::SharedArgs as std::ops::Deref>::deref": "I01@fnv1a64:d1f45122789cd24e;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<rum_core::expr::SharedExpr as std::ops::Deref>::deref": "I02@fnv1a64:63c6f867f737e397;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index": "I04@fnv1a64:407a10927e28c30b;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::clone_result": "R01@fnv1a64:66e64b15456d22db;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::accounting_mode::rule::evaluate_rule": "R02@fnv1a64:f1e99079697abe6c;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name": "B13@fnv1a64:40414790a7fdeeb2;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt", "std::vec::Vec<rum_core::expr::Expr>::len": "B05@fnv1a64:0c857ad3ff015b97;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt"}
```

Only fnv1a64 fingerprint cells changed; identities, status, preconditions, source, strength, exclusions and all non-fingerprint text are identical. Actual emitted certificate files are archived under [inventory/final/](../../../.temp/brief-563/inventory/final).


#### Authorized obligation evidence positions


```tsv
BEFORE	N:First	First	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:210	lazy stepper current/accumulator and realized items	crates/rum-stdlib/src/structure_ops/surface.rs:99; crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:57; crates/rum-stdlib/src/lazy.rs:162; crates/rum-stdlib/src/lazy.rs:578	rum_stdlib::structure_ops::surface::eval_first
AFTER	N:First	First	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:210	lazy stepper current/accumulator and realized items	crates/rum-stdlib/src/structure_ops/surface.rs:101; crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:59; crates/rum-stdlib/src/lazy/source.rs:64; crates/rum-stdlib/src/lazy/realization.rs:25	rum_stdlib::structure_ops::surface::eval_first
```

```tsv
BEFORE	N:LazyTable	LazyTable	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:308	imin and imax retained while later bounds evaluate	crates/rum-stdlib/src/lazy.rs:437; crates/rum-stdlib/src/lazy.rs:438; crates/rum-stdlib/src/lazy.rs:439	rum_stdlib::lazy::eval_lazy_table
AFTER	N:LazyTable	LazyTable	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:308	imin and imax retained while later bounds evaluate	crates/rum-stdlib/src/lazy.rs:244; crates/rum-stdlib/src/lazy.rs:245; crates/rum-stdlib/src/lazy.rs:246	rum_stdlib::lazy::eval_lazy_table
```

```tsv
BEFORE	Unsplit:LazyTuples	LazyTuples	Unsplit	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs	Inherited membership; subclass not re-audited	crates/rum-stdlib/src/lazy.rs:495,511	rum_stdlib::lazy::eval_lazy_tuples
AFTER	Unsplit:LazyTuples	LazyTuples	Unsplit	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs	Inherited membership; subclass not re-audited	crates/rum-stdlib/src/lazy.rs:302,318	rum_stdlib::lazy::eval_lazy_tuples
```

```tsv
BEFORE	Unsplit:Normal	Normal	Unsplit	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs	Inherited membership; subclass not re-audited	crates/rum-stdlib/src/associations/basic.rs:270,278	rum_stdlib::associations::basic::eval_normal
AFTER	Unsplit:Normal	Normal	Unsplit	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs	Inherited membership; subclass not re-audited	crates/rum-stdlib/src/associations/basic.rs:275,283	rum_stdlib::associations::basic::eval_normal
```

```tsv
BEFORE	N:Take	Take	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:585	lazy stepper current/accumulator and realized items	crates/rum-stdlib/src/structure_ops/surface.rs:99; crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:57; crates/rum-stdlib/src/lazy.rs:162; crates/rum-stdlib/src/lazy.rs:578	rum_stdlib::structure_ops::sequence::take_drop::eval_take
AFTER	N:Take	Take	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:585	lazy stepper current/accumulator and realized items	crates/rum-stdlib/src/structure_ops/surface.rs:101; crates/rum-stdlib/src/structure_ops/sequence/take_drop.rs:59; crates/rum-stdlib/src/lazy/source.rs:64; crates/rum-stdlib/src/lazy/realization.rs:25	rum_stdlib::structure_ops::sequence::take_drop::eval_take
```

Only evidence positions changed in these five rows. All classes, reasons, owners and Pending statuses are unchanged.


#### Attempt ledger and stop-loss


| Artifact | Command | Bound s | Exit |

| --- | --- | --- | --- |

| base/build-alloc.json | cargo build --release -p rum-cli --features rum-cli/alloc-count --target-dir .temp/brief-563/target-alloc | 1200 | 0 |

| base/build-default.json | cargo build --release -p rum-cli --target-dir .temp/brief-563/target-default | 1200 | 0 |

| candidate-1/build-alloc.json | cargo build --release -p rum-cli --features rum-cli/alloc-count --target-dir .temp/brief-563/target-alloc | 300 | 0 |

| candidate-1/build-default.json | cargo build --release -p rum-cli --target-dir .temp/brief-563/target-default | 300 | 0 |

| candidate-2/build-alloc.json | cargo build --release -p rum-cli --features rum-cli/alloc-count --target-dir .temp/brief-563/target-alloc | 300 | 0 |

| candidate-2/build-default.json | cargo build --release -p rum-cli --target-dir .temp/brief-563/target-default | 300 | 0 |

| candidate-2/fmt.json | cargo fmt --all | 150 | 0 |

| candidate-2/focused-1.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --nocapture | 150 | 0 |

| candidate-3/build-alloc.json | cargo build --release -p rum-cli --features rum-cli/alloc-count --target-dir .temp/brief-563/target-alloc | 300 | 0 |

| candidate-3/build-default.json | cargo build --release -p rum-cli --target-dir .temp/brief-563/target-default | 300 | 0 |

| candidate-3/fmt.json | cargo fmt --all | 150 | 0 |

| candidate-3/focused-1.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations | 150 | 0 |

| final/certificates-default.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests local_production_acceptance | 120 | 0 |

| final/certificates-no-default.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --no-default-features local_production_acceptance | 120 | 0 |

| final/fmt.json | cargo fmt --check | 150 | 0 |

| final/guest-loading-resume.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test guest_loading_resume | 150 | 0 |

| final/lazy-continuations.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations | 150 | 0 |

| final/lazy.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests lazy | 150 | 0 |

| final/loop-continuations.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test loop_continuations | 150 | 0 |

| final/ownership.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests ownership_graph | 120 | 0 |

| final/runtime.json | cargo test -p rum-runtime --target-dir .temp/brief-563/target-tests | 120 | 101 |

| final2/fmt.json | cargo fmt --check | 150 | 0 |

| final2/guest-loading-resume.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test guest_loading_resume | 150 | 0 |

| final2/lazy-continuations.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations | 150 | 0 |

| final2/lazy.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests lazy | 150 | 0 |

| final2/loop-continuations.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test loop_continuations | 150 | 0 |

| final2/ownership.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests ownership_graph | 120 | 0 |

| final2/runtime.json | cargo test -p rum-runtime --target-dir .temp/brief-563/target-tests | 120 | 101 |

| final2/stdlib.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests | 1500 | 0 |

| final2/workspace-platform.json | env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --target-dir .temp/brief-563/target-tests -- --nocapture | 1500 | 101 |

| final2/workspace.json | cargo test --workspace --target-dir .temp/brief-563/target-tests -- --nocapture | 1500 | 101 |

| implementation/attempt-1.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --skip _deep --nocapture | 150 | 101 |

| implementation/attempt-2.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --skip _deep --nocapture | 150 | 0 |

| implementation/attempt-3-depth.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations depth:: -- --nocapture | 150 | 0 |

| implementation/attempt-4-private.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --lib lazy_flat -- --nocapture | 150 | 101 |

| implementation/attempt-5-private.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --lib lazy_flat -- --nocapture | 150 | 0 |

| implementation/attempt-6-lazy.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --lib lazy -- --nocapture | 150 | 0 |

| implementation/fmt-before.json | cargo fmt --all -- --check | 150 | 1 |

| implementation/fmt.json | cargo fmt --all | 150 | 0 |

| inventory/certificates-default.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests local_production_acceptance -- --nocapture | 120 | 101 |

| inventory/certificates-no-default.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --no-default-features local_production_acceptance -- --nocapture | 120 | 101 |

| inventory/discovery-2.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --lib discovery_report_without_inventory_input -- --nocapture | 120 | 0 |

| inventory/discovery.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --lib discovery_report_without_inventory_input -- --nocapture | 120 | 0 |

| inventory/final-discovery.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --lib discovery_report_without_inventory_input -- --nocapture | 120 | 0 |

| inventory/final/certificates-default-generated.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests local_production_acceptance -- --nocapture | 120 | 101 |

| inventory/final/certificates-default.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests local_production_acceptance | 120 | 0 |

| inventory/final/certificates-no-default-generated.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --no-default-features local_production_acceptance -- --nocapture | 120 | 101 |

| inventory/final/certificates-no-default.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --no-default-features local_production_acceptance | 120 | 0 |

| inventory/final/discovery.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --lib discovery_report_without_inventory_input -- --nocapture | 120 | 0 |

| inventory/final/fingerprints.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --lib regenerate_local_contract_fingerprints -- --ignored --nocapture | 120 | 0 |

| inventory/fingerprints.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --lib regenerate_local_contract_fingerprints -- --ignored --nocapture | 120 | 0 |

| pre/depth-cancel.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --exact depth::depth_cancel_deep --nocapture | 150 | 101 |

| pre/depth-contract.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --lib lazy_depth_contract -- --nocapture | 150 | 101 |

| pre/depth-drop.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --exact depth::depth_drop_deep --nocapture | 150 | 101 |

| pre/depth-inner.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --exact depth::depth_inner_deep --nocapture | 150 | 101 |

| pre/depth-invalid_zero.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --exact depth::depth_invalid_zero_deep --nocapture | 150 | 101 |

| pre/depth-no_pull.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --exact depth::depth_no_pull_deep --nocapture | 150 | 101 |

| pre/depth-outer.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --exact depth::depth_outer_deep --nocapture | 150 | 101 |

| pre/depth-pull.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --exact depth::depth_pull_deep --nocapture | 150 | 101 |

| pre/depth-select.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --exact depth::depth_select_deep --nocapture | 150 | 101 |

| pre/depth-shallow.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations _shallow -- --nocapture | 150 | 101 |

| pre/depth-twice.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --exact depth::depth_twice_deep --nocapture | 150 | 101 |

| pre/descriptor-control.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --exact depth::descriptor_control_deep --nocapture | 150 | 0 |

| pre/oracle.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations -- --skip depth:: --skip structural:: --nocapture | 150 | 101 |

| pre/structural.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations structural:: -- --nocapture | 150 | 101 |

| protocol/fmt.json | cargo fmt --all | 150 | 0 |

| protocol/focused-1.json | cargo test -p rum-stdlib --target-dir .temp/brief-563/target-tests --test lazy_pull_continuations | 150 | 0 |

| protocol/runtime.json | cargo test -p rum-runtime --target-dir .temp/brief-563/target-tests | 120 | 101 |



There are 68 archived Cargo invocations including formatting, builds, maintenance, expected stale-certificate failures, mandatory final checks and the documented platform retry. The pre-fix phase used 14 focused Cargo invocations, including the nine required isolated abort probes; this is above eight if the complete red phase is counted as one milestone. It is disclosed as a procedure-limit problem, not hidden by reporting only compilation rounds. Initial implementation used six focused test attempts (two compiler-error fixes) plus two formatting commands; candidate 2 had one focused integration attempt; candidate 3 had one before and one after protocol fixes. Final command groups and repeated maintenance are fully listed above.


| Budget | Used | Limit / qualification |

| --- | --- | --- |

| Measured candidates | 3 | 3; tuning stopped |

| Metric process attempts | 1405 | 1728; 433 base attempts + 3 x 324; includes 108 perf syntax failures and two timeouts |

| Timing suites | 2 | 2; one base, one final, one CLI process each |

| Release CLI depth/control probes | 21 | 100; ten incoming, one shallow CLI input control, ten final |

| Separate workload-result checks | 216 | 54 per preserved build; separate from metrics |

| Instruction pair coverage | 53/54 complete baseline pairs | Fold-256 first pair remains missing; no gate pass inferred |



No two measured compiling candidates failed to improve all stop-loss measures: candidate 2 removed 26 allocation/byte regressions; candidate 3 reduced instruction regressions and the worst regression. The numerical gates nevertheless fail.


Open decisions / nonpassing evidence:

1. Candidate 3 has 24 repeatable instruction regressions and one incompletely paired baseline label. The cap prevents further tuning in this brief.
2. Runtime and workspace remain red on the closed FrameRootTrace/PureFrameAbandon inventories. The exact 14-entry additive patch is prepared, not applied, because rum-runtime is fenced.
3. Four release CLI input/cancellation shapes exit 1. The n=32 incoming CLI control proves the driver cannot satisfy the specified resume workflow with stdin /dev/null. No substitute release harness was silently accepted.
4. Descriptor Expr native-depth debt remains outside scope. The n=2,000 debug and n=20,000 release controls passed; no higher descriptor ceiling or arbitrary Expr walk safety is claimed.
5. Native ZeroMQ needed an explicit environment skip opt-in; the nine platform skips and original failures are both preserved. Workspace packages/doc tests after the failing runtime target did not run in that command.
6. The initial red-phase Cargo count, incomplete baseline pair, allocation retry order, empty full-environment artifact and fixed-location outputs of required existing harnesses are disclosed above. No assertion, inventory classification, scope fence or performance threshold was relaxed to manufacture green.


Final handoff state: source compiles; functional lazy tests and full stdlib are green; ownership and both certificate gates are green; runtime/workspace and performance are not green. Nothing is half-edited. No further implementation, measurement or version-control action was taken after the final candidate. [PROGRESS.md](../../../.temp/brief-563/PROGRESS.md) identifies the exact outstanding decisions and resume point.

