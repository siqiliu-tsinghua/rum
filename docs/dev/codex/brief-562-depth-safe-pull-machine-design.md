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
