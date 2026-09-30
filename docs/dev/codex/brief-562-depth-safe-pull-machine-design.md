# brief-562 — design addendum: a depth-safe, resumable lazy pull machine

`Scope: NONE — no production change.` Design-only, revising brief-561's design,
with declared scratch exceptions (see Scope). **Do not run `git` in any form**,
including before you have read this brief.

## Goal

brief-561 designed how a suspended lazy pull resumes (read its appended report
first; it stays the base of this work). Its design keeps the recursive
`PullStepper` — a `Box`-chained tree walked by native recursion — and adds four
more recursive walks over it: the cold path reconstruction, the resumed driver,
the working copy made on resume, and root tracing of the frame.

**The director's adjacent-boundary probe found that the recursive stepper
already aborts the process.** Build a chain with a loop and realize it:

```
x = LazyRange[1];
Do[x = LazyMap[Identity, x], {n}];
y = Take[x, 0];
```

With the release `rum-cli`, `ulimit -s 2048` (the ruled 2 MiB embedder stack):
`n = 2000` survives, `n = 5000` ends in `thread 'main' has overflowed its stack`
(exit 134). With 8 MiB: 5 000 survives, 10 000 aborts. `First[x]` aborts the
same way. `Take[x, 0]` pulls nothing, so construction or destruction of the
stepper tree overflows before any `next`. The descriptor `Expr` itself is not
the problem: building `x` at `n = 20000` and replacing it (`x = 0`) survives on
2 MiB. `Rest[x]`, which builds a descriptor without realizing, also survives.
`lazy.rs` never enters the native stack guard, and the ownership graph's
accumulator gate covers `Option<Arc<_>>` chains only; the stepper appears
solely as a row of `crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv`
(`rum_stdlib::lazy::PullStepper`, line 41). The probe files are in
`.temp/director/gate-561/` (`thr.rum`, `which.rum`, `exprdrop.rum`).

This is a guest-reachable fail-open that predates the suspension branch. G2
rewrites exactly this machine, so **G2 now includes making it depth-safe**:
no walk of the machine — construction from the descriptor, the pull,
destruction, the resume working copy, the path or cursor handling, root
tracing, abandonment — may recurse natively in proportion to the descriptor's
depth. Revise the design so that holds, then redraft the implementation briefs.

## The director's preferred direction — evaluate it, do not assume it

Every descriptor is, as far as the director can see, a **unary chain**: one
source at the bottom (`LazyRange`, `LazyPermutations`, `LazySubsets`,
`LazyTuples`, `LazyNestList`), and adapters with exactly one lazy child
(`LazyMap`, `LazySelect`, `LazyTake`, `LazyDrop`, `LazyFoldList`); `LazyTable`
lowers to `LazyTake[LazyMap[…, LazyRange[…]], n]`. If so, the machine can be
**linearized**: one source plus a `Vec` of stages ordered outermost to
innermost, driven by an explicit cursor — descend to request a value, ascend
with a value (or with exhaustion) through the stages, restart the descent from
a stage that rejects (`Select` false, `Drop` skipping). Then:

- construction, destruction, cloning and root tracing are iterations over a
  `Vec`, with no recursion to guard and no self-reaching type;
- the pull is a loop, not native recursion;
- **the suspension state is the cursor itself** — which stage is waiting, in
  which direction, holding which pending value (`Select`'s candidate,
  `NestList`'s yielded value) — which is the "pending-pull program counter"
  brief-548's draft asked for, and it replaces brief-561's waiter record and
  its cold path reconstruction.

Decide on correctness and enforceability, per `CLAUDE.md`'s standard: a
representation in which deep recursion cannot be written beats one in which
every walk must remember a guard. Compare it honestly against the alternative
— keep the recursive tree, put the native stack guard
(`kernel_api::run_with_native_stack_guard`, `NativeStackDepth`;
`crates/rum-runtime/src/runtime/native_stack_guard.rs`) on every recursive walk,
fail closed with `StackLimitExceeded`, and give the `Box` chain an iterative
`Drop` — and choose. If the unary-chain premise is false anywhere, show where
with `path:line` and design for what is actually there.

## What the revised design must settle

1. **The premise.** Confirm or refute the unary chain from the descriptor
   constructors, `stepper_from_descriptor` (`lazy.rs:685`) and every helper
   that walks a descriptor (`:629`, `:649`, `:670`, `:797`, and any other).
   List every native recursion proportional to descriptor depth, today and in
   brief-561's design, with its walk (construct, pull, drop, clone, trace,
   abandon) and its evidence.
2. **The representation and the pull loop**, in enough detail to implement:
   the stage and source types, the cursor, and pseudocode for one pull
   covering `Take` (checks before descending; decrements before yielding),
   `Drop` (skips after a `Some`), `Select` (rejection restarts the descent),
   `FoldList` (emits its init without pulling), `NestList` (eager lookahead),
   `Map`. Exhaustion must propagate exactly as today. Each semantic in
   brief-561 §"Semantics to preserve" and its apply-once table must still
   hold; say which rows of that table change and why.
3. **Suspension and resume on the new shape**: what the frame carries, how the
   one pending callback result is applied, how the drive continues, a second
   suspension, and abandonment. Keep brief-561's settled decisions unless the
   new shape changes them — immutable Session-rooted frames and a cold working
   copy on an actual resume, the cancellation observer re-acquired rather than
   stored (`NativeStackDepth` tokens cannot be stored in a frame at all, by a
   `compile_fail` doctest), the `Option<Expr>` continuation codec, the
   `Normal`/`Take`/`First` completion frames, the separate `LazyTable` bounds
   frame — and say for each whether it stands.
4. **The ordinary path.** Show the ordinary pull loop against today's
   recursion: what each pull executes, what it allocates, where the stage
   dispatch sits. A pull that never suspends must not allocate or clone, and
   the capture must stay on the error edge (brief-560's lesson: state carried
   in a hot loop's error arm changed its codegen). Name the measurement plan;
   brief-561's 30-label panel is the starting point, with depth-varied labels
   added if the representation changes per-stage cost. The implementation will
   measure; a regression there is fixed, not bought back by returning to
   recursion — the recursion is the fail-open.
5. **Budget and cancellation on a deep chain.** Source pulls are what is
   charged today. Say whether a chain thousands of stages deep needs any
   additional poll (a rejection loop at a deep stage returns to the source and
   is charged; ascending through the stages is not), and why.
6. **Inventory consequences.** Linearizing removes the `self_reaching.tsv` row
   for `PullStepper` and may change other rows' identities; the standing duty
   STOPs on any of that unless the implementation brief authorizes it by name.
   **List every ownership-graph and retention-contract row the change would
   add, remove or re-identify**, so the director can authorize them exactly.
7. **Module layout.** brief-561 kept the hot bodies in place and put new cold
   code in `lazy/continuation.rs`. The representation change rewrites the hot
   bodies; propose the layout (`docs/code-organization-guidelines.md` limits),
   and name each existing item whose identity moves.

## What I want back

1. The premise check and the recursion inventory (point 1), with evidence.
2. The revised design (points 2-7), with the frame and stage inventory (fields
   and roots), and the table of what changed from brief-561's design.
3. **Depth tests**, run in the scratch test and recorded, alongside brief-561's
   39 oracle cases (keep them; re-run them on the unchanged tree to confirm the
   pre-fix values still hold). Each depth case must **force every stage to
   run** and assert the exact result — for example
   `n` nested `LazyMap[Function[x, x + 1], …]` over `LazyRange[0]`, where
   `First` must return exactly `n` — and must run on a thread with a 2 MiB
   stack (`std::thread::Builder::stack_size`). Cover construction and
   destruction without a pull (`Take[x, 0]`), a pull, a deep chain with a
   suspension at the innermost callback and at the outermost, resumed through
   the oracle; a deep `Select` and `Drop` chain whose rejections restart; and
   cancellation of a deep suspended chain. **A native stack overflow aborts the
   whole test binary**, so the pre-fix evidence for the deep cases is the
   release `rum-cli` under `ulimit -s 2048`, recorded with exit status; keep a
   shallow companion of every deep case in the scratch test, which is the
   falsifier the test binary can run today. Also measure the current per-level
   cost of each overflowing walk (release, e.g. by bisecting `n` under
   `ulimit -s`, or `/proc` `VmStk` at several heights) and state the pre-fix
   ceiling on 2 MiB.
4. Redrafted implementation briefs in `TEMPLATE.md` shape, replacing
   brief-561's drafts A and B where the design changes them — each with its red
   tests (depth and oracle), its predicted lost-continuation count (80 -> 69 ->
   66 today, adjust if the steps change), the named inventory rows it
   authorizes, the performance gate, and the standing inventory duty; serial
   runs (`env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1`). Scope as objective,
   fence and disclosure duty.
5. Open questions for the director.

## Context pointers

- brief-561 and its appended report; `.temp/brief-561/design.md`,
  `draft-A-pull-machine.md`, `draft-B-table-bounds.md`, `performance-gate.md`,
  `performance-panel.json`, `probe.rs`, `cases.json`, `red-results.md`.
- `.temp/director/gate-561/` — the director's depth probes.
- `crates/rum-stdlib/src/lazy.rs`; `docs/dev/lazy-sequences-design.md`;
  `docs/guide/evaluation.md` §Lazy Sequences.
- The native stack guard: `crates/rum-runtime/src/runtime/native_stack_guard.rs`
  and its uses in `crates/rum-stdlib/src/associations/` and
  `functional_ops/apply_map/traversal.rs`; the association depth registry
  `crates/rum-stdlib/src/tests/association_depth_contract/registry.rs`.
- Ownership graph inventories: `crates/rum-stdlib/src/tests/ownership_graph/`
  (`mod.rs:40`, `accumulator_coverage.rs:33`, `inventory/`).
- `docs/dev/codex/TEMPLATE.md` (standing duty), `AGENTS.md`,
  `docs/code-organization-guidelines.md`.

## Scope

- Files you MAY modify: **NONE**, with these exceptions:
  - append your report to **this** file;
  - write scratch under `.temp/brief-562/`;
  - create **one** scratch integration test,
    `crates/rum-stdlib/tests/zz_brief562_probe.rs`, and **delete it before
    appending the report**, copying its final content to
    `.temp/brief-562/probe.rs`; say in the report that you did;
  - build the release `rum-cli` from the unchanged tree into the default
    target directory for the depth measurements.
- Everything else is read-only.

## Hard rules

- **No `git`, in any form.**
- Do NOT modify any file outside the exceptions; STOP and report instead.
- Touch nothing outside this repository. No network. All scratch, including
  `.rum` probes and `TMPDIR`, stays under `.temp/brief-562/`.
- Commands, serial only, each through `python3 scripts/run-bounded.py`:
  `env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true cargo test -p rum-stdlib --test zz_brief562_probe`
  (bound 80 s; brief-561's runs took 7 s);
  `env CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true cargo build --release -p rum-cli`
  (bound 300 s; brief-560's release builds took 89-108 s);
  `target/release/rum-cli <file>.rum < /dev/null` under `bash -c 'ulimit -s …'`
  (bound 60 s each). File reads, `rg`/`grep`, `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No production code.** No new `unsafe`.

## Report format (your final message, appended to this file)

1. What changed — "design only", scratch artifacts, deletion of the scratch
   test, the release build.
2. The premise check and recursion inventory.
3. The revised design, the frame and stage inventory, and what changed from
   brief-561.
4. The depth measurements and the tests with their pre-fix values.
5. The redrafted implementation briefs.
6. Open questions for the director.

---

## Execution report — brief 562 (2026-09-30)

### 1. What changed

**Design only; no production change.** Revised the pull-machine design and both
implementation drafts under `.temp/brief-562/`, and appended this report. No git,
network, production edit, unsafe, dependency or vendor-engine comparison was used.
All commands after the initial brief read ran serially through the bounded runner;
all scratch and TMPDIR stayed under `.temp/brief-562/`.

The one authorized integration test,
`crates/rum-stdlib/tests/zz_brief562_probe.rs`, was copied to
[probe.rs](../../../.temp/brief-562/probe.rs) and **deleted before this report was
appended**. The release `rum-cli` was built from the unchanged tree into the default
target directory. No production correctness/performance gate is claimed.

Artifacts:

- [Revised design, pseudocode, frame/stage/root inventory and 561 change table](../../../.temp/brief-562/design.md).
- [Exact inventory authorization and item moves](../../../.temp/brief-562/inventory-plan.md),
  [all 36 existing lazy ownership rows](../../../.temp/brief-562/ownership-existing.json),
  [all 97 existing rows in the four touched source files](../../../.temp/brief-562/ownership-touched-baseline.json).
- [Tests, every pre-fix value and stack measurements](../../../.temp/brief-562/red-results.md),
  [full equality records](../../../.temp/brief-562/red-results.tsv),
  [original cases/inputs](../../../.temp/brief-562/cases.json),
  [final raw test log](../../../.temp/brief-562/probe-run-02.txt).
- [CLI sources/outputs/exit ledger](../../../.temp/brief-562/depth-runs.jsonl),
  [depth brackets](../../../.temp/brief-562/depth-brackets.json),
  [release stack bytes/self-call evidence](../../../.temp/brief-562/release-stack-evidence.json),
  [binary/source identity](../../../.temp/brief-562/release-metadata.json).
- [Draft A](../../../.temp/brief-562/draft-A-pull-machine.md),
  [draft B](../../../.temp/brief-562/draft-B-table-bounds.md),
  [complete future performance gate](../../../.temp/brief-562/performance-gate.md),
  [fixed 54-label manifest](../../../.temp/brief-562/performance-panel.json).

A Python hash audit recorded **1,733** baseline source/doc/script/root files.
After deleting the scratch test, all 1,733 were unchanged, with no new source file.
The [cleanup record](../../../.temp/brief-562/cleanup-verification.json) proves that
state; the final audit checks that only this brief changed, by append. Generated
Cargo artifacts are from the two authorized tests and one release build.

### 2. Premise check and recursion inventory

**The unary-chain premise holds.** In `crates/rum-stdlib/src/lazy.rs`, descriptor
construction and decoding show:

| Kind | Constructor / decoder evidence | Lazy children |
| --- | --- | ---: |
| Range, Permutations, Subsets, Tuples, NestList | constructors :331/:312/:387/:489/:296; source decoder :690-745 | 0 |
| Map, Select | constructors :273/:364; decoder :746/:755 | 1, args[1] |
| Take, Drop | constructors :463/:224; decoder :764/:773 | 1, args[0] |
| FoldList | constructor :250; decoder :782 | 1, args[2] |
| Table | :406-460 constructs Take(Map(Range)) | one lowered chain |

Tuple factors, callback functions and accumulator values are ordinary Expr data,
not additional lazy input ports. There is no recognized multi-child descriptor.

Only `stepper_from_descriptor` (:685, recursive calls :752/:761/:769/:778/:790)
walks the descriptor spine. The other requested helpers are not recursive descriptor
walks: `lazy_table_count` :629 does arithmetic; `advance_subset_cursor` :649 and
`advance_tuple_cursor` :670 loop over indices; `wrap_descriptor` :797 makes one
wrapper. `is_lazy_list` :536 and `lazy_list_descriptor` :591 inspect one head/wrapper;
`lazy_drop` :543/Rest make a descriptor, not a machine. The full helper/call-site
inventory with evidence is in the design.

| Native-depth walk | Today | Added/retained by 561 |
| --- | --- | --- |
| construction | recursive decoder above, unguarded | retained |
| pull | next :105; child calls :167/:175/:189/:193/:199/:211 | retained plus resumed helper |
| destruction / error unwind | Box children :82/:86/:89/:93/:100; emitted recursive drop glue | frozen and working trees also recursively drop |
| callback path handling | no resumable machine | cold recursive path reconstruction and path following |
| resume / second suspension | missing continuation | recursive application/unwind/retry |
| working copy | no suspended machine copy | recursive Box-tree clone |
| root tracing | no lazy frame | derived recursive Box tracing |
| abandonment / frame retirement | loses machine during ordinary unwind | drops recursive frame-owned trees |

`ownership_graph/inventory/self_reaching.tsv:41` records PullStepper, but the
accumulator gate at `ownership_graph/accumulator_coverage.rs:18-21` only selects
Option<Arc<_>> edges. It does not cover Box chains. `lazy.rs` uses no native guard.
Some Take tail calls happen to be optimized into a loop in this release; that does
not protect the other arms or give a Rust-level depth guarantee.

### 3. Revised design, fields/roots and changes from 561

Choose **one PullSource plus Vec<PullStage>, outermost first, driven by an indexed
Down/Up cursor**. Neither a stage nor a source owns another machine node. Construct,
clone, trace, drop and abandon iterate over flat storage. No waiter/path expansion
or recursive resumed interpreter remains. The [design](../../../.temp/brief-562/design.md)
gives complete types, roots, pseudocode, error edges and the ten-row apply-once table.

The source variants keep existing cursor fields: Range current/step; Permutations
current/exhausted; Subsets elements/k/combo; Tuples factors/digits/exhausted;
NestList function/current. Stages hold Map function, Select predicate, Take/Drop
remaining, or FoldList function/accumulator/emitted_init. Only Expr-valued fields
are roots; scalar counters/indices are not. A pending cursor holds Map/Fold stage
index, Select index plus candidate, or Nest's saved old yield.

One `LazyRealizeSuspensionFrame` owns the flat machine, output prefix, output budget,
saved source count/ceiling and pending cursor. It keeps immutable Session-rooted
ownership and a **cold working copy only on actual Ok resume**. First suspension
moves live state. Err completion copies nothing. Reacquire the observer for the same
request; never reset its saved source budget/deadline, use try_unwrap, mutate rooted
frames, or store NativeStackDepth. The native guard's compile_fail doctests explicitly
forbid frame-stored/escaping tokens (`native_stack_guard.rs:19/:79`).

Down checks Take before entering its child and decrements **before the child**, as
current :188 requires; Up forwards without a second check/decrement, even at zero.
Drop consumes one skip only after Some and requests its child again. Select false
restarts at its own child, leaving outer ancestors pending. Fold init turns Up
without touching its child. Nest computes lookahead before yielding its saved old
current. None traverses every waiting Drop to clear remaining, with no callback.
The resumed callback answer is applied once, then that cursor enters the same loop;
a second suspension freezes a fresh cursor and leaves the existing consumer suffix
in the runtime queue. No callback is replayed.

Normal/Take/First completion frames and the `Option<Expr>` continuation codec stand;
Part keeps the existing First/Drop symbolic seam. The separate LazyTable bounds
frame stands: original held args, iterator, normalized/evaluated three slots, pending
slot; lower->upper->step, then all-three validation and inert plan construction.
No body evaluation during construction; pure abandonment executes no lazy callback.

| 561 decision | 562 disposition |
| --- | --- |
| single machine frame; separate consumer finishers | retained |
| immutable rooted frame; cold resume copy | retained, now flat |
| observer reacquisition; Option codec; separate bounds frame | retained |
| eager lookahead, init, source-only charge, rerederivation, exhaustion/errors | retained |
| callback apply-once rows | semantics retained; scalar cursor replaces leaf path |
| MapInput/SelectInput/FoldInput and transparent/skip waiter rows | same effects handled by ordinary Up dispatch; records removed |
| preserve the recursive hot bodies / no phase dispatch | superseded by the explicit iterative loop; cost must be measured |
| recursive cold path, copy, trace, resume and drop | eliminated from machine shape |

The guarded-tree alternative preserves more current dispatch code, but requires
guards on every fallible recursive entry plus iterative drop. It cannot simply
"guard the derives": Clone/FrameRootTrace are infallible, tracing cannot omit roots,
and trace/drop/abandon have no Session/error return for StackLimitExceeded. It would
still need explicit copy/trace/teardown machinery and more proof obligations. The
flat unary representation is the stronger enforceable choice.

Ordinary pulls add **no control/continuation allocation or clone**: cursor/value are
local, and pending state is boxed only on callback InputRequired. Existing source
values, callback Expr construction and prefix growth still perform their original
value allocations; a literal ban on those would contradict existing semantics.
Stage dispatch, bounds checks and functional polls are measured costs. The future
panel retains all 30 original labels and adds 24 depth labels at 8/64/256, for 54
labels per candidate, allocations first, paired instruction slopes, then complete
monotonic AbsoluteTiming microbench A/B. Recursion is not an allowed regression fix.

Source charges remain unchanged. **Additional cancellation polls are proposed** at
construction/resume entry, after each successful guest callback, and at most every
256 transitions in long stage/construction/cold-copy walks. A shallow specialization
omits the per-stage countdown. This is functional cancellation work, not diagnostics
or a new source charge. Source polls alone miss long ascent and Fold-init turns.
The new cancellation red below makes the proposed callback checkpoint explicit;
its exact timing is a director decision within the cooperative contract.

Layout: keep constructor/catalog owners and public wrappers in lazy.rs; add
`lazy/{state,descriptor,source,machine,realization,continuation}.rs`, with new consumer
completion children and B's `table_continuation.rs`. New files target 2-10k chars;
old lazy.rs shrinks from 23,887 to about 13-16k with a documented warning-range
assessment. The explicit [item/row map](../../../.temp/brief-562/inventory-plan.md)
names every old identity moving. The realization driver does not import capture
code, avoiding a drive/resume module cycle.

Inventory authorization requests are precise: remove the PullStepper self-reaching
row; re-identify the budget derive; relocate ten existing closure/macro rows, with
all exact old strings saved. The other 24 lazy rows and old consumer rows retain
identity/reason apart from positions. Nine A derives, three possible A capture
macros, two B derives and three possible B capture macros are named additions;
actual new UnknownConstruct sites must still be disclosed from discovery. No new
host payload or recursive machine is authorized. Refresh all **68** local contract
fingerprints and **C01/C02** default/no-default certificates by replay. Current
N:First, N:Take, N:LazyTable, Unsplit:LazyTuples and Unsplit:Normal keep their live
Pending classes/owners; First/Take are already N, contrary to 561's older census
note. No lease or retention promotion is proposed. Prefix/candidate/accumulator and
frozen/working overlap remain the later R02/R05 accounting work.

### 4. Depth measurements and pre-fix tests

The final scratch run is **7 passed, 41 failed, 9 ignored**, exit 101. All original
39 cases are retained unchanged and all 39 complete value/log/message/print/error/
termination/prompt records match 561 exactly: **2 passing controls, 37 red**.
[Comparison proof](../../../.temp/brief-562/oracle-comparison.json).

The nine new shallow companions run on explicit **2 MiB threads**, n=32:

| Case | Resumed value / log | Queued value / log | Prompts | Result |
| --- | --- | --- | --- | --- |
| zero-output construct/drop | List[] / 0 | List[] / 0 | 0/0 | pass |
| zero-output, malformed deepest leaf | Take / 0 | Take / 0 | 0/0 | pass |
| incrementing Map pull | 32 / 32 | 32 / 32 | 0/0 | pass |
| innermost callback | 1 / 1 | List[32] / 32 | 1/1 | red |
| outermost callback | 32 / 32 | List[32] / 32 | 1/1 | red |
| inner then outer callback | 1 / 1 | List[32] / 32 | 1/2 | red |
| Select inner/outer rejection | 2 / 65 | 2 / 65 | 0/0 | pass |
| 32 Drop[1] stages | 32 / 33 | 32 / 33 | 0/0 | pass |
| cancel at suspended inner callback | Cancelled / 1 | Cancelled / 32 | 1/1 | red |

Incrementing Maps require result n and n callback effects. Select requires 2n+1
predicate calls across two rejection/restart points. Drop requires result n and
n+1 source calls. The malformed deepest leaf makes skipping construction for an
output budget of zero observable. Cancellation uses identical Print sinks: both
paths emit one boundary print and General::cancel, but queued cancellation currently
allows outer callbacks before it is observed. No assertion was weakened to accept
that discrepancy; the proposed checkpoint intentionally tightens cooperative timing.
Every completed path checks no pending owner and a fresh `1+1 == 2` request.

All nine 10,000-stage counterparts are saved as ignored pre-fix Rust tests because
stack overflow aborts the binary. Their isolated **release CLI** counterparts on
2 MiB all print `built`, then abort **exit 134**. The suspension/cancel witnesses
abort during construction before any prompt; this does **not** claim that deep
resume/copy/trace/abandon ran pre-fix. Those exact deep Rust tests must be enabled
and pass after A. Descriptor replacement and Rest at depth **20,000** both pass on
2 MiB; the descriptor itself is not the observed boundary.

**50** bounded serial CLI runs (cap 100) established these brackets:

| Probe | Stack KiB | Last observed pass | First observed abort |
| --- | ---: | ---: | ---: |
| malformed leaf, constructor isolated | 2,048 | 2,156 | 2,187 |
| valid Take[_,0], construction/drop | 2,048 | 2,156 | 2,187 |
| incrementing Map First | 2,048 | 2,156 | 2,187 |
| malformed leaf | 4,096 | 4,375 | 4,406 |
| malformed leaf | 8,192 | 8,812 | 8,843 |

The malformed leaf cannot complete a PullStepper chain, so it isolates the
constructor. The current public 2 MiB ceiling is approximately **2.17k stages**, not
a certified exact maximum between these endpoints. Increasing stack by 6 MiB moves
the boundary consistently with about **941-950 native bytes per level**.

A Python ELF/prologue read of the actual release binary independently records
**944 bytes per recursive constructor activation**, **656 for next**, and **48 for
normal PullStepper drop** (saved registers + local subtraction + return address).
Self-call addresses/raw bytes and binary hash are archived. Next/drop independently
measured *failure ceilings* are masked by construction; the report does not invent
those thresholds. Clone/trace/path/resume are proposed 561 code, absent in this
binary, so no empirical activation cost exists for them. A adds direct private
walk tests to isolate all flat walks after implementation.

Verbatim verification lines:

```text
# bounded 300 s, offline serial cargo build --release -p rum-cli; exit 0
    Finished `release` profile [optimized] target(s) in 6.05s

# bounded 80 s, offline serial zz_brief562_probe -- --nocapture; first run, exit 101
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.69s
running 55 tests
test result: FAILED. 6 passed; 41 failed; 8 ignored; 0 measured; 0 filtered out; finished in 0.27s

# same command, final run after adding malformed-leaf pair; exit 101
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.92s
running 57 tests
test result: FAILED. 7 passed; 41 failed; 9 ignored; 0 measured; 0 filtered out; finished in 0.28s
```

Exact commands/environments are in run metadata and red-results.md. Every child
uses CARGO_BUILD_JOBS=1, RUST_TEST_THREADS=1, CARGO_NET_OFFLINE=true and local TMPDIR.
The CLI children use `ulimit -s`, stdin `/dev/null`, and a 60-second bound. Process
durations are bookkeeping, not benchmark timings. No formatter/workspace/retention/
instruction/allocation-count/microbenchmark gate ran in this design-only scope.

### 5. Redrafted implementation briefs

Both drafts follow TEMPLATE.md's complete Goal, Context, Scope, Deliverable,
pitfalls, Verification, Checkpointing, Hard rules and Report structure. They are
raw material for director rewrite, **not approved implementation dispatches**.

| Draft | Unit of change | Recorded incoming tests | Predicted Lost count |
| --- | --- | --- | --- |
| [A — flat machine and consumers](../../../.temp/brief-562/draft-A-pull-machine.md) | depth-safe representation and every walk, pending cursor, realization/three finishers, explicit cancellation checkpoints | 33 old cases + 9 shallow companions = 35 red/7 controls; 9 deep native-abort obligations; plus specified structural/private walk controls | global 80->69; G2 14->3 |
| [B — eager bounds](../../../.temp/brief-562/draft-B-table-bounds.md) | ordered lower/upper/step continuation and same inert plan | six old bounds/composition cases remain red; retain all 51 enabled A cases plus its extra controls | global 69->66; G2 3->0 |

A must enable the nine deep tests once **all** walks are iterative; exact output is
required, not tolerated StackLimit or ignored tests. B retains them enabled. Both
require serial bounded correctness and both-feature inventory replay, preserved
incoming/candidate binaries, all 54 allocation/instruction rows per candidate and
complete microbench A/B. The measurement cap is 1,728 isolated metric processes per
step (base plus at most three candidates), plus at most eight full timing-suite
invocations with their internal runs disclosed. Two compiling non-improving attempts
trigger stop-loss. The 30 old workload statements/setup/shapes are unchanged.

A's Scope explicitly names the self-reaching removal, derive change and ten
construct relocations. B authorizes no further old-row removal. Both retain the
standing duty and STOP on any other old identity/status/disposition/exclusion
change, unexpected disappearance, new host payload or row in untouched code.
No inventory maintenance was silently performed in this design scope.

### 6. Open questions for the director

1. Ratify the **callback-completion cancellation checkpoint**, plus 256-transition
   long-chain polling, before dispatch. It resolves the new queued log32 versus
   suspended log1 witness without changing source charges, but is an explicit
   cooperative-timing tightening and measured cost. A includes scoped guide wording;
   if choosing a different checkpoint contract, rewrite the oracle/brief explicitly.
2. Ratify the proposed **+0.05% per paired instruction sample** noise rule, or supply
   the required stricter decision. Supply missing same-shape timeout calibrations
   for broad suites/feature builds when rewriting. No performance success is claimed
   by a flat-type argument or by these native-stack measurements.
3. Authorize the named **inventory/item move map** in the director-authored A. The
   immutable-frame/cold-copy, observer, codec and consumer/bounds decisions from
   561 remain settled; they do not need reopening. No blanket relocation exception
   or retention-class change is requested.

The design, executed pre-fix evidence, scope audit and replacement drafts are
complete. Production repair, enabled deep post-fix tests and measured gates remain
future implementation work.
