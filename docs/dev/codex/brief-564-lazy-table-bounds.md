# brief-564 — G2 step B: LazyTable's bounds resume into the plan

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

`LazyTable[body, {i, lower, upper, step}]` evaluates its three bounds eagerly,
in order, with plain `?` (`eval_lazy_table`, `crates/rum-stdlib/src/lazy.rs`,
the three evaluations now near `:244-246`). If a bound asks for input, the
resumed request completes with that bound's value instead of the lazy plan:
`LazyTable-bound-1/2/3` resume to the scalars `1`, `3`, `1` where the queued
path returns `LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]`.
Frozen Lost IDs P1908, P1909, P1910. Lost continuations **69 -> 66**, and with
them group G2 is closed (**3 -> 0**).

After this brief: a suspension in a bound resumes into the remaining bounds,
then the unchanged all-three validation and plan construction; a second
suspension in a later bound carries only the remaining work; the body is never
evaluated to build the plan.

brief-562's draft B (`.temp/brief-562/draft-B-table-bounds.md`) is the raw
material; **this brief governs** where they differ (depth, performance gate,
inventory permissions).

## Context pointers

- brief-563 and its appended report — the accepted flat machine (`lazy/`), its
  consumer completion frames, and how a frame is captured on the error edge and
  resumed from a cold copy of the immutable, Session-rooted frame
  (`lazy/continuation.rs`); the oracle harness in
  `crates/rum-stdlib/tests/lazy_pull_continuations.rs` and its children.
- brief-562's report and `.temp/brief-562/design.md` §§3, 5, 9;
  `.temp/brief-561/cases.json`, `red-results.md` (the six B cases' pre-fix
  values).
- `eval_lazy_table`, `invalid_lazy_table_spec` and `lazy_table_count` in
  `crates/rum-stdlib/src/lazy.rs`.
- How a loop frame carries its own state: `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:60-105`.
- `docs/dev/codex/TEMPLATE.md` — the standing duty, which now names
  rum-runtime's frame lists.

## Scope

**Objective:** the bounds continuation, a private finishing helper shared by
the ordinary and resumed paths, the owning tests, and the standing and
performance gates.

**Edit surface:** `eval_lazy_table` and a new private `finish_lazy_table`
helper in `crates/rum-stdlib/src/lazy.rs` (the existing validation and the four
plan `vec!` statements may move into that helper inside `lazy.rs`); a new
`crates/rum-stdlib/src/lazy/table_continuation.rs`; additions to
`crates/rum-stdlib/tests/lazy_pull_continuations.rs` and its children; and the
rum-runtime frame lists the standing duty names — `derived_inventory.rs` gets
exactly the new types' derive entries and the named-frame count in
`frame_root_trace.rs` rises by exactly the number of new frames (554 today).

**Named inventory additions:** the `FrameRootTrace, PureFrameAbandon` derive on
`lazy::table_continuation::LazyTableBoundsFrame`, the derive on its slot type,
and up to three capture-macro sites for the lower, upper and step calls — all
`UnknownConstruct`, staying Unknown. The existing rows (the `one` closure, the
moved validation and plan constructs) keep their identity and reason; only
positions move. `N:LazyTable` keeps its owner `lazy::eval_lazy_table` and its
N/Pending status; only its evidence positions follow. No removal or
re-identification is authorized in this brief.

**Standing maintenance duty — always in scope** (`TEMPLATE.md`): replay the 68
contract fingerprints, run `local_production_acceptance` in the default and
`--no-default-features` builds on the final source and install the C01/C02
cells, reconcile every ownership inventory against discovery, add the new
frame types to rum-runtime's lists. **STOP** on any other existing row change
or disappearance, any new `UnknownHostPayload` or `self_reaching` row, any new
row in untouched code, or any retention reclassification.

**Fence:** brief-563's machine, source, state, descriptor, realization,
continuation and consumer code stays as it is, and its tests stay green. No
change to rum-runtime outside the two frame lists, rum-core, the host crates,
the `Part` seam, the catalog's iterator forms, any public API, trait, macro
protocol or dependency, or any other suspension group. No edits to
`AGENTS.md`, `TODO.md`, root `PROGRESS.md`, earlier briefs, the lockfile, or
unrelated docs and tests. The guide's Lazy Sequences paragraph needs no change
for this; if you find it does, STOP and name the sentence.

**Disclosure duty:** every slot and root, every moved statement and row, every
test result, every measured label, and any adjacent defect.

## Deliverable

1. **Ordinary path unchanged in shape.** Keep the straight-line lower -> upper
   -> step evaluations; capture only on `InputRequired`, in a cold helper on
   the error edge. Normalize default bounds once. No bounds `Vec`, closure
   factory or phase interpreter on the success path.
2. **The frame.** One immutable `LazyTableBoundsFrame` owns the original held
   arguments, the iterator, the three slots (evaluated values before the
   pending one, normalized expressions from it on) and which slot is pending.
   It derives `FrameRootTrace` and `PureFrameAbandon`; it owns no machine, no
   `Session` borrow, no observer, no `NativeStackDepth`.
3. **Resume.** On `Ok`, work on a cold copy: put the answer in the pending slot,
   evaluate only the later slots, then run the shared finish — all-three
   validation, count, inert plan. A suspension in a later bound installs a
   replacement frame carrying only the remaining work; the runtime keeps the
   outer suffix, so no consumer frame is appended twice. `Err` and abandonment
   evaluate no further bound.
4. **Semantics.** Lower and upper stay arbitrary expressions until all three are
   evaluated; a nonnumeric or zero-step result still yields the original held
   call with `LazyTable::argform`, after the later bounds' side effects have
   run. The body stays held while the plan is built; realizing the plan later
   uses brief-563's machine and suspends independently.

## Tests

Red first, recorded on the incoming tree before any production edit:

- the **six** deferred cases, unchanged from brief-561: `LazyTable-bound-1`,
  `-bound-2`, `-bound-3`, `LazyTable-bounds-twice`, `LazyTable-invalid-step`,
  `LazyTable-body-and-bounds` — full oracle equality (value, ordered messages,
  prints, error and termination, `log`, exact prompts, no pending owner, a
  working `1 + 1` after). `bound-3`'s log already matches; only the value
  shows the defect, which is why the value is compared;
- **bound-specific controls:** an ordinary error in `step` after `upper`
  resumes; cancellation at the first and at the second bound suspension (the
  held body never runs, log unchanged); a nonnumeric `lower` whose later bounds
  still run their effects before the `argform` fallback.

Every brief-563 case stays enabled and green, including the 2 000-stage depth
cases on their 2 MiB threads and the structural depth checks.

## rum / Rust pitfalls (read twice)

1. Parsing a bound as a number before all three are evaluated skips later side
   effects. Held arguments and evaluated slots are different things.
2. Building the plan is not realizing the body. A bound's answer is never the
   public result.
3. A second suspension carries its remaining work only; do not rebuild from the
   original held call.
4. Extracting a shared finish helper and adding a cold capture can change
   `eval_lazy_table`'s codegen — measure it.
5. Bound roots are later retention (R05) work; no lease, no promotion.

## Verification (MANDATORY — run to green or evidence)

All commands **serial**, through `python3 scripts/run-bounded.py --timeout <s>
-- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-564/tmp …`. Bounds from measured serial history: a
focused test target 150 s; `ownership_graph`, `local_production_acceptance`
(either feature set) and `cargo test -p rum-runtime` 300 s each (observed
17-45 s plus a rebuild); `cargo test -p rum-stdlib` and `cargo test
--workspace` 1 800 s each (observed 990-1 047 s); a release `rum-cli` build
300 s incremental, 1 200 s into a fresh target directory. A timeout is a
recorded failure.

After focused green and the final replay, archive verbatim output of
`cargo fmt --check`, `cargo test -p rum-stdlib --test lazy_pull_continuations`,
`cargo test -p rum-stdlib lazy`, `cargo test -p rum-stdlib ownership_graph`,
`local_production_acceptance` in both feature sets, `cargo test -p
rum-runtime`, and `cargo test --workspace --no-fail-fast` (without
`--no-fail-fast` cargo stops at the first failing target and the rest never
runs). Report native ZeroMQ skips as skips.

**Performance — measure, do not tune.** G2's ordinary-path cost will be erased
as a whole by the next brief, against the pre-G2 binary brief-563 preserved
(`.temp/brief-563/base/`). Here, measure what B adds: preserve the incoming
default and `alloc-count` release binaries before any edit, build the final
candidate, and run `.temp/brief-562/performance-gate.md`'s 54-label panel on
both — allocation counts, then two paired samples of `taskset -c 2 perf stat
-e cpu_core/instructions/u` (the `/u` spelling; `:u` is rejected by this
host's perf), one label per process, stdin `/dev/null`, per-iteration
`(count(2N) - count(N)) / N`, each workload's result checked first. Report
every label's delta against the incoming binary **and** against
`.temp/brief-563/base/rum-cli`. One measured candidate; no timing suite. A
label beyond +0.05% in both samples is reported, not tuned here.

**Stop-loss:** at most eight focused Cargo attempts per milestone; stop after
two consecutive compiling attempts that reduce neither the failing cases nor
the remaining Lost sites. Falling compile-error counts are progress before the
tree compiles.

## Checkpointing — protect the scene

Everything under `.temp/brief-564/`: immutable pre-fix evidence, preserved
binaries and hashes, raw perf output saved before parsing. After each
milestone update `.temp/brief-564/PROGRESS.md` with what is done and verified,
the exact next action, whether the tree compiles, half-edited files, and the
counters above.

## Hard rules

- **No `git`, in any form.**
- No network; touch nothing outside this repository; all scratch in
  `.temp/brief-564/`.
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the release `rum-cli`, `taskset`/`perf`.
- Scope is the objective, the fence and the disclosure duty; STOP and report
  before editing past the fence.
- **Testing Policy is a hard line** (`AGENTS.md`).
- No new `unsafe`, dependency, lease, mutable or rootless frame payload, public
  API, or other suspension group. Never store a `Session` borrow, the
  cancellation observer or a `NativeStackDepth` in a frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Files and statements changed or moved; the frame, its slots and roots; the
   evaluation order.
2. Verbatim red and green output for the six cases, the controls, brief-563's
   suite, and the broad suites; every inventory row change.
3. The 54-label allocation and instruction rows against the incoming binary and
   against the pre-G2 base.
4. Lost continuations before and after (69 -> 66 predicted), the stop-loss
   counters, and open questions.

---
