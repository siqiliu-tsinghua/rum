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

## Execution report — brief 564

### 1. Files, statements, frame roots, and evaluation order

Implemented the bound continuation. The six deferred cases and four bound controls pass, as do the 59 existing continuation/depth cases. Frozen Lost IDs P1908/P1909/P1910 are covered: **69 → 66**, and **G2 3 → 0**, using the brief’s frozen starting census. No global census or retention classification was rewritten.

Changed existing files:

- `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`

- `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs`

- `crates/rum-stdlib/src/lazy.rs`

- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv`

- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv`

- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv`

- `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv`

- `crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv`

- `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv`

- `crates/rum-stdlib/tests/lazy_pull_continuations.rs`

New files:

- `crates/rum-stdlib/src/lazy/table_continuation.rs`

- `crates/rum-stdlib/tests/lazy_pull_continuations/bounds.rs`

This report is appended only to the current brief. All changes remain uncommitted; no git operation or network command was used.

`lazy.rs` adds the table-continuation module and retains the straight-line lower → upper → step evaluations at lines 248, 261, 274. Each keeps its normalized pending expression by cloning the Expr before evaluation; the cold error helper captures only InputRequired. There is no success-path bounds Vec, closure factory, or phase interpreter. The existing `one` closure and default normalization are preserved (233 → 237).

`finish_lazy_table` at lines 290–320 contains the original all-three numeric validation and count statements verbatim (247–256 → 299–308), followed by the original Function / LazyRange / LazyMap / LazyTake construction statements verbatim (258–267 → 310–319). The four vec! rows move 260→312, 262→314, 263→315, 266→318. `invalid_lazy_table_spec` and `lazy_table_count` are unchanged except positions. Other facade items shift positions only; all existing lazy/ machine, descriptor, source, state, realization, continuation, and consumer code is byte-identical to entry.

`LazyTableBoundsFrame` owns four fields:

- `args`: SharedArgs: original held body and iterator specification.

- `iterator`: Expr.

- `bounds`: [Expr; 3]: completed values before pending; normalized raw expressions from pending onward.

- `pending_slot`: BoundSlot: rootless Lower/Upper/Step discriminator, derived trace.

FrameRootTrace traces the original held body/specification, iterator, and all three bound Expr slots; the slot discriminator has no Expr roots. PureFrameAbandon performs no guest work. The frame is immutable and owns no Session borrow, machine, observer, NativeStackDepth, lease, or mutable interior state. On Ok, the cold helper copies the fixed array, replaces the pending slot, and evaluates only later slots. A later suspension creates one replacement frame; the runtime preserves the outer suffix. Err returns before copying/evaluating. Both frozen and working roots can coexist during resume; R05 retention work remains pending.

The plan keeps its body held. Nonnumeric/zero-step validation happens only after every bound finishes; fallback uses the original held arguments. The body-and-bounds case reaches both the bounds and independent realization suspensions without replaying effects. Runtime lists add exactly two trace entries, one pure-abandon entry, and the named-frame count **554 → 555**.

Organization assessment:

- `lazy.rs`: 14,735 characters; cohesive constructor facade remains, cold continuation is a separate 3,192-character module. Broader constructor splitting is outside this narrow brief.

- `tests`: Parent 14,787 characters; new bounds child 4,527 characters. Existing tests unchanged apart from module mount.

- `frame_root_trace.rs`: 30,342 characters, existing split-plan range; only one count assertion changed. A future separation of protocol scanning from protocol assertions is outside this two-list permission.

- `derived_inventory.rs`: 169,102-character pre-existing closed list; exactly three entries added. No inventory module restructuring authorized.

Complete source/inventory diff, produced without git: `.temp/brief-564/final/source-diff.txt`. Source/input snapshot and hashes: `.temp/brief-564/candidate/source/` and `source-sha256.json`; final verification found no drift across 1232 files.

### 2. Red/green verification and every inventory row change

All Cargo/test/build commands were serial and bounded, with CARGO_BUILD_JOBS=1, RUST_TEST_THREADS=1, CARGO_NET_OFFLINE=true and TMPDIR=/home/remi/dev/rum/.temp/brief-564/tmp. Every top-level Cargo invocation appears in the following ledger. Command elapsed_seconds is execution metadata, not a timing benchmark.

```json
[
  {
    "metadata": ".temp/brief-564/incoming/build-default.json",
    "command": [
      "cargo",
      "build",
      "--release",
      "-p",
      "rum-cli"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 0.27995745899534086
  },
  {
    "metadata": ".temp/brief-564/incoming/build-alloc.json",
    "command": [
      "cargo",
      "build",
      "--release",
      "-p",
      "rum-cli",
      "--features",
      "rum-cli/alloc-count",
      "--target-dir",
      ".temp/brief-564/target-alloc"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 0.19238734200189356
  },
  {
    "metadata": ".temp/brief-564/pre/red-lazy-continuations.json",
    "command": [
      "cargo",
      "test",
      "-p",
      "rum-stdlib",
      "--test",
      "lazy_pull_continuations",
      "--",
      "--nocapture"
    ],
    "timeout": 150,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 101,
    "elapsed_seconds": 45.12193864300207
  },
  {
    "metadata": ".temp/brief-564/pre/red-bounds-controls-final.json",
    "command": [
      "cargo",
      "test",
      "-p",
      "rum-stdlib",
      "--test",
      "lazy_pull_continuations",
      "bounds::",
      "--",
      "--nocapture"
    ],
    "timeout": 150,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 101,
    "elapsed_seconds": 7.952210059003846
  },
  {
    "metadata": ".temp/brief-564/implementation/format.json",
    "command": [
      "cargo",
      "fmt",
      "--all"
    ],
    "timeout": 150,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 1.7356246639974415
  },
  {
    "metadata": ".temp/brief-564/implementation/focused-1.json",
    "command": [
      "cargo",
      "test",
      "-p",
      "rum-stdlib",
      "--test",
      "lazy_pull_continuations",
      "--",
      "--nocapture"
    ],
    "timeout": 150,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 40.93365360099415
  },
  {
    "metadata": ".temp/brief-564/inventory/discovery.json",
    "command": [
      "cargo",
      "test",
      "-p",
      "rum-stdlib",
      "--lib",
      "discovery_report_without_inventory_input",
      "--",
      "--nocapture"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 40.977647727995645
  },
  {
    "metadata": ".temp/brief-564/inventory/fingerprints.json",
    "command": [
      "cargo",
      "test",
      "-p",
      "rum-stdlib",
      "--lib",
      "regenerate_local_contract_fingerprints",
      "--",
      "--ignored",
      "--nocapture"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 14.485633329000848
  },
  {
    "metadata": ".temp/brief-564/inventory/final/certificates-default-generated.json",
    "command": [
      "cargo",
      "test",
      "-p",
      "rum-stdlib",
      "local_production_acceptance",
      "--",
      "--nocapture"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 101,
    "elapsed_seconds": 47.53255372700369
  },
  {
    "metadata": ".temp/brief-564/inventory/final/certificates-no-default-generated.json",
    "command": [
      "cargo",
      "test",
      "-p",
      "rum-stdlib",
      "--no-default-features",
      "local_production_acceptance",
      "--",
      "--nocapture"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 101,
    "elapsed_seconds": 91.15676224899653
  },
  {
    "metadata": ".temp/brief-564/inventory/final/certificates-default.json",
    "command": [
      "cargo",
      "test",
      "-p",
      "rum-stdlib",
      "local_production_acceptance"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 32.40164980300324
  },
  {
    "metadata": ".temp/brief-564/inventory/final/certificates-no-default.json",
    "command": [
      "cargo",
      "test",
      "-p",
      "rum-stdlib",
      "--no-default-features",
      "local_production_acceptance"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 31.18686744599836
  },
  {
    "metadata": ".temp/brief-564/final/fmt.json",
    "command": [
      "cargo",
      "fmt",
      "--check"
    ],
    "timeout": 150,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 1.7377380799953244
  },
  {
    "metadata": ".temp/brief-564/final/lazy-continuations.json",
    "command": [
      "cargo",
      "test",
      "-p",
      "rum-stdlib",
      "--test",
      "lazy_pull_continuations",
      "--",
      "--nocapture"
    ],
    "timeout": 150,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 23.561436190997483
  },
  {
    "metadata": ".temp/brief-564/final/lazy.json",
    "command": [
      "cargo",
      "test",
      "-p",
      "rum-stdlib",
      "lazy",
      "--",
      "--nocapture"
    ],
    "timeout": 150,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 0.837468566998723
  },
  {
    "metadata": ".temp/brief-564/final/ownership.json",
    "command": [
      "cargo",
      "test",
      "-p",
      "rum-stdlib",
      "ownership_graph",
      "--",
      "--nocapture"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 59.83681503000116
  },
  {
    "metadata": ".temp/brief-564/final/runtime.json",
    "command": [
      "cargo",
      "test",
      "-p",
      "rum-runtime"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 35.86465899000177
  },
  {
    "metadata": ".temp/brief-564/candidate/build-default.json",
    "command": [
      "cargo",
      "build",
      "--release",
      "-p",
      "rum-cli"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 0.17903214800026035
  },
  {
    "metadata": ".temp/brief-564/candidate/build-alloc.json",
    "command": [
      "cargo",
      "build",
      "--release",
      "-p",
      "rum-cli",
      "--features",
      "rum-cli/alloc-count",
      "--target-dir",
      ".temp/brief-564/target-alloc"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 47.61020205800014
  },
  {
    "metadata": ".temp/brief-564/candidate/build-default-locate.json",
    "command": [
      "cargo",
      "build",
      "--release",
      "-p",
      "rum-cli",
      "--message-format=json"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 0.18006338700070046
  },
  {
    "metadata": ".temp/brief-564/candidate/build-default-explicit.json",
    "command": [
      "cargo",
      "build",
      "--release",
      "-p",
      "rum-cli",
      "--target-dir",
      ".temp/brief-564/target-default"
    ],
    "timeout": 300,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 46.6649300680001
  },
  {
    "metadata": ".temp/brief-564/final/workspace.json",
    "command": [
      "env",
      "RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1",
      "cargo",
      "test",
      "--workspace",
      "--no-fail-fast",
      "--",
      "--nocapture"
    ],
    "timeout": 1800,
    "environment": {
      "CARGO_BUILD_JOBS": "1",
      "RUST_TEST_THREADS": "1",
      "CARGO_NET_OFFLINE": "true",
      "TMPDIR": "/home/remi/dev/rum/.temp/brief-564/tmp"
    },
    "exit_code": 0,
    "elapsed_seconds": 1351.4735149219996
  }
]
```

Red evidence was recorded before any production edit. First full run: 59 existing cases green, all ten new controls/cases red. Two new control assumptions were corrected before the second red run: uncaught Throw contributes its own third diagnostic, and synchronous cancellation from Print is a different checkpoint than cancellation while InputRequired. Original control source is preserved as `pre/bounds-first-controls.rs`. Final cancellation controls compare a resumed prefix with a queued prefix and cancel at the same actual input suspension; they assert exact prompts, unchanged bound log, held body, messages/prints, termination, no pending owner, and a working 1+1. Final pre-fix bounds run: one cancellation control green, nine failures. The six frozen case source strings and inputs are unchanged.

Verbatim output from `.temp/brief-564/pre/red-lazy-continuations.log` (the complete raw output is archived there):

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.46s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 69 tests
test bounds::lazytable_body_and_bounds ... CASE	LazyTable-body-and-bounds	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }, "List[]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-body-and-bounds	"3"	"List[]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'bounds::lazytable_body_and_bounds' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:141:5:
assertion `left == right` failed: must reach every suspension
  left: ["p: "]
 right: ["p: ", "p: "]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test bounds::lazytable_bound_1 ... CASE	LazyTable-bound-1	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "List[1]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bound-1	"1"	"List[1]"	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'bounds::lazytable_bound_1' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:147:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }
FAILED
test bounds::lazytable_bound_2 ... CASE	LazyTable-bound-2	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bound-2	"3"	"List[1, 2]"	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'bounds::lazytable_bound_2' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:147:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }
FAILED
test bounds::lazytable_bound_3 ... CASE	LazyTable-bound-3	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bound-3	"1"	"List[1, 2, 3]"	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'bounds::lazytable_bound_3' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:147:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }
FAILED
test bounds::lazytable_bounds_twice ... CASE	LazyTable-bounds-twice	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bounds-twice	"3"	"List[1, 2]"	"LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'bounds::lazytable_bounds_twice' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:141:5:
assertion `left == right` failed: must reach every suspension
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED
test bounds::lazytable_cancel_first_suspension ... CASE	LazyTable-cancel-at-1	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1]", ["lower: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2, 3]", [])
ROW	LazyTable-cancel-at-1	"TerminatedEvaluation[\"Cancelled\"]"	"List[1]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary", "boundary"]

thread 'bounds::lazytable_cancel_first_suspension' panicked at crates/rum-stdlib/tests/lazy_pull_continuations/bounds.rs:118:5:
assertion `left == right` failed
  left: "List[1, 2, 3]"
 right: "List[1]"
FAILED
test bounds::lazytable_cancel_second_suspension ... CASE	LazyTable-cancel-at-2	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: ["boundary"] }, "List[1]", ["lower: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2, 3]", [])
ROW	LazyTable-cancel-at-2	"1"	"List[1]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2, 3]"	1	2	false	"OK"	"OK"	"None"	"Some(Cancelled)"	[]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary", "boundary"]

thread 'bounds::lazytable_cancel_second_suspension' panicked at crates/rum-stdlib/tests/lazy_pull_continuations/bounds.rs:118:5:
assertion `left == right` failed
  left: "List[1, 2, 3]"
 right: "List[1, 2]"
FAILED
test bounds::lazytable_error_in_step_after_upper ... CASE	LazyTable-error-in-step-after-upper	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }], prints: ["before"] }, "List[1, 2]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])
ROW	LazyTable-error-in-step-after-upper	"3"	"List[1, 2]"	"<error>"	"List[1, 2, 3]"	1	1	false	"OK"	"Other { message: \"uncaught throw: 9 with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }]	["before"]	["before", "after"]

thread 'bounds::lazytable_error_in_step_after_upper' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:147:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("3"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }], prints: ["before"] }
 right: View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }
FAILED
test bounds::lazytable_invalid_step ... CASE	LazyTable-invalid-step	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-invalid-step	"3"	"List[1, 2]"	"LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }]	[]	[]

thread 'bounds::lazytable_invalid_step' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:147:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: [] }
FAILED
test bounds::lazytable_nonnumeric_lower_runs_later_bounds ... CASE	LazyTable-nonnumeric-lower	resumed=(View { status: "OK", value: Some("x"), termination: "None", messages: [], prints: [] }, "List[1]", ["p: "])	queued=(View { status: "OK", value: Some("LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: ["upper", "step"] }, "List[1, 2, 3]", [])
ROW	LazyTable-nonnumeric-lower	"x"	"List[1]"	"LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }]	[]	["upper", "step"]

thread 'bounds::lazytable_nonnumeric_lower_runs_later_bounds' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:147:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("x"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: ["upper", "step"] }
FAILED
test cancel_first_suspension ... CASE	Lazy-cancel-at-1	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1, 2]", [])
ROW	Lazy-cancel-at-1	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2]"	1	1	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary"]
ok
test cancel_second_suspension ... CASE	Lazy-cancel-at-2	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2, 3, 4]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2, 3, 4]", [])
ROW	Lazy-cancel-at-2	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2, 3, 4]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2, 3, 4]"	2	2	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary", "boundary"]	["boundary", "boundary"]
ok
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

failures:

failures:
    bounds::lazytable_body_and_bounds
    bounds::lazytable_bound_1
    bounds::lazytable_bound_2
    bounds::lazytable_bound_3
    bounds::lazytable_bounds_twice
    bounds::lazytable_cancel_first_suspension
    bounds::lazytable_cancel_second_suspension
    bounds::lazytable_error_in_step_after_upper
    bounds::lazytable_invalid_step
    bounds::lazytable_nonnumeric_lower_runs_later_bounds

test result: FAILED. 59 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.52s

error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`
```

Verbatim output from `.temp/brief-564/pre/red-bounds-controls-final.log` (the complete raw output is archived there):

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.78s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 10 tests
test bounds::lazytable_body_and_bounds ... CASE	LazyTable-body-and-bounds	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }, "List[]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-body-and-bounds	"3"	"List[]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'bounds::lazytable_body_and_bounds' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:141:5:
assertion `left == right` failed: must reach every suspension
  left: ["p: "]
 right: ["p: ", "p: "]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED
test bounds::lazytable_bound_1 ... CASE	LazyTable-bound-1	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "List[1]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bound-1	"1"	"List[1]"	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'bounds::lazytable_bound_1' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:147:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }
FAILED
test bounds::lazytable_bound_2 ... CASE	LazyTable-bound-2	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bound-2	"3"	"List[1, 2]"	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'bounds::lazytable_bound_2' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:147:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }
FAILED
test bounds::lazytable_bound_3 ... CASE	LazyTable-bound-3	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bound-3	"1"	"List[1, 2, 3]"	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'bounds::lazytable_bound_3' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:147:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }
FAILED
test bounds::lazytable_bounds_twice ... CASE	LazyTable-bounds-twice	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bounds-twice	"3"	"List[1, 2]"	"LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	2	false	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]

thread 'bounds::lazytable_bounds_twice' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:141:5:
assertion `left == right` failed: must reach every suspension
  left: ["p: "]
 right: ["p: ", "p: "]
FAILED
test bounds::lazytable_cancel_first_suspension ... CASE	LazyTable-cancel-at-1	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1]", ["lower: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1]", ["lower: "])
ROW	LazyTable-cancel-at-1	"TerminatedEvaluation[\"Cancelled\"]"	"List[1]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1]"	1	1	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary"]
ok
test bounds::lazytable_cancel_second_suspension ... CASE	LazyTable-cancel-at-2	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: ["boundary"] }, "List[1]", ["lower: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2]", ["upper: "])
ROW	LazyTable-cancel-at-2	"1"	"List[1]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2]"	1	2	false	"OK"	"OK"	"None"	"Some(Cancelled)"	[]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary", "boundary"]

thread 'bounds::lazytable_cancel_second_suspension' panicked at crates/rum-stdlib/tests/lazy_pull_continuations/bounds.rs:105:5:
assertion `left == right` failed
  left: ["lower: "]
 right: ["lower: ", "upper: "]
FAILED
test bounds::lazytable_error_in_step_after_upper ... CASE	LazyTable-error-in-step-after-upper	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }], prints: ["before"] }, "List[1, 2]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])
ROW	LazyTable-error-in-step-after-upper	"3"	"List[1, 2]"	"<error>"	"List[1, 2, 3]"	1	1	false	"OK"	"Other { message: \"uncaught throw: 9 with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }]	["before"]	["before", "after"]

thread 'bounds::lazytable_error_in_step_after_upper' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:147:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("3"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }], prints: ["before"] }
 right: View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }
FAILED
test bounds::lazytable_invalid_step ... CASE	LazyTable-invalid-step	resumed=(View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-invalid-step	"3"	"List[1, 2]"	"LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }]	[]	[]

thread 'bounds::lazytable_invalid_step' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:147:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("3"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: [] }
FAILED
test bounds::lazytable_nonnumeric_lower_runs_later_bounds ... CASE	LazyTable-nonnumeric-lower	resumed=(View { status: "OK", value: Some("x"), termination: "None", messages: [], prints: [] }, "List[1]", ["p: "])	queued=(View { status: "OK", value: Some("LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: ["upper", "step"] }, "List[1, 2, 3]", [])
ROW	LazyTable-nonnumeric-lower	"x"	"List[1]"	"LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"	"List[1, 2, 3]"	1	1	false	"OK"	"OK"	"None"	"None"	[]	[RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }]	[]	["upper", "step"]

thread 'bounds::lazytable_nonnumeric_lower_runs_later_bounds' panicked at crates/rum-stdlib/tests/lazy_pull_continuations.rs:147:5:
assertion `left == right` failed: value, error, termination, messages, prints
  left: View { status: "OK", value: Some("x"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: ["upper", "step"] }
FAILED

failures:

failures:
    bounds::lazytable_body_and_bounds
    bounds::lazytable_bound_1
    bounds::lazytable_bound_2
    bounds::lazytable_bound_3
    bounds::lazytable_bounds_twice
    bounds::lazytable_cancel_second_suspension
    bounds::lazytable_error_in_step_after_upper
    bounds::lazytable_invalid_step
    bounds::lazytable_nonnumeric_lower_runs_later_bounds

test result: FAILED. 1 passed; 9 failed; 0 ignored; 0 measured; 59 filtered out; finished in 0.08s

error: test failed, to rerun pass `-p rum-stdlib --test lazy_pull_continuations`
```

Implementation attempt 1 passed all 69 cases. The following is the required final post-replay green output, including all prior 563 depth cases on their existing 2 MiB threads, with no added ignores:

Verbatim output from `.temp/brief-564/final/fmt.log` (the complete raw output is archived there):

```text

```

Verbatim output from `.temp/brief-564/final/lazy-continuations.log` (the complete raw output is archived there):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 69 tests
test bounds::lazytable_body_and_bounds ... CASE	LazyTable-body-and-bounds	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-body-and-bounds	"List[1, \"two\", 3]"	"List[1, 2, 3]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test bounds::lazytable_bound_1 ... CASE	LazyTable-bound-1	resumed=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bound-1	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test bounds::lazytable_bound_2 ... CASE	LazyTable-bound-2	resumed=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bound-2	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test bounds::lazytable_bound_3 ... CASE	LazyTable-bound-3	resumed=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bound-3	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test bounds::lazytable_bounds_twice ... CASE	LazyTable-bounds-twice	resumed=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bounds-twice	"LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	"LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test bounds::lazytable_cancel_first_suspension ... CASE	LazyTable-cancel-at-1	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1]", ["lower: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1]", ["lower: "])
ROW	LazyTable-cancel-at-1	"TerminatedEvaluation[\"Cancelled\"]"	"List[1]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1]"	1	1	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary"]
ok
test bounds::lazytable_cancel_second_suspension ... CASE	LazyTable-cancel-at-2	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2]", ["lower: ", "upper: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2]", ["upper: "])
ROW	LazyTable-cancel-at-2	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2]"	2	2	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary", "boundary"]	["boundary", "boundary"]
ok
test bounds::lazytable_error_in_step_after_upper ... CASE	LazyTable-error-in-step-after-upper	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])
ROW	LazyTable-error-in-step-after-upper	"<error>"	"List[1, 2, 3]"	"<error>"	"List[1, 2, 3]"	1	1	true	"Other { message: \"uncaught throw: 9 with tag None\" }"	"Other { message: \"uncaught throw: 9 with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }]	["before", "after"]	["before", "after"]
ok
test bounds::lazytable_invalid_step ... CASE	LazyTable-invalid-step	resumed=(View { status: "OK", value: Some("LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-invalid-step	"LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"	"List[1, 2, 3]"	"LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }]	[RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }]	[]	[]
ok
test bounds::lazytable_nonnumeric_lower_runs_later_bounds ... CASE	LazyTable-nonnumeric-lower	resumed=(View { status: "OK", value: Some("LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: ["upper", "step"] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: ["upper", "step"] }, "List[1, 2, 3]", [])
ROW	LazyTable-nonnumeric-lower	"LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"	"List[1, 2, 3]"	"LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }]	[RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }]	["upper", "step"]	["upper", "step"]
ok
test cancel_first_suspension ... CASE	Lazy-cancel-at-1	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1, 2]", [])
ROW	Lazy-cancel-at-1	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2]"	1	1	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary"]
ok
test cancel_second_suspension ... CASE	Lazy-cancel-at-2	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2, 3, 4]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2, 3, 4]", [])
ROW	Lazy-cancel-at-2	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2, 3, 4]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2, 3, 4]"	2	2	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary", "boundary"]	["boundary", "boundary"]
ok
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

test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.39s
```

Verbatim output from `.temp/brief-564/final/lazy.log` (the complete raw output is archived there):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 46 tests
test lazy::tests::lazy_flat_abandonment_depth ... flat abandonment depth=2000 complete
ok
test lazy::tests::lazy_flat_all_variant_roots_depth ... all source/stage/cursor roots depth=2000 complete
ok
test lazy::tests::lazy_flat_clone_and_working_copy_depth ... flat clone and working copy depth=2000 complete
ok
test lazy::tests::lazy_flat_last_owner_drop_depth ... last-owner flat drop depth=2000 complete
ok
test tests::association_depth_contract::lazy_depth_contract::lazy_cross_module_recursion_falsifier ... ok
test tests::association_depth_contract::lazy_depth_contract::lazy_ownership_is_not_self_reaching ... ok
test tests::association_depth_contract::lazy_depth_contract::lazy_recursive_helper_falsifier ... recursive helper falsifier: [RecursiveEdge { path: "lazy/machine.rs", caller: "drive_pull", callee: "drive_pull", count: 1 }]
ok
test tests::association_depth_contract::lazy_depth_contract::lazy_self_reaching_falsifier - should panic ... 
thread 'tests::association_depth_contract::lazy_depth_contract::lazy_self_reaching_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/lazy_depth_contract.rs:105:5:
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

test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 1041 filtered out; finished in 0.31s

     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-44228b396944f9ba)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-ab1526217bb7e597)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s

     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 41 tests
test bounds::lazytable_body_and_bounds ... CASE	LazyTable-body-and-bounds	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-body-and-bounds	"List[1, \"two\", 3]"	"List[1, 2, 3]"	"List[1, \"two\", 3]"	"List[1, 2, 3]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test bounds::lazytable_bound_1 ... CASE	LazyTable-bound-1	resumed=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bound-1	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test bounds::lazytable_bound_2 ... CASE	LazyTable-bound-2	resumed=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bound-2	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test bounds::lazytable_bound_3 ... CASE	LazyTable-bound-3	resumed=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bound-3	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	"LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test bounds::lazytable_bounds_twice ... CASE	LazyTable-bounds-twice	resumed=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-bounds-twice	"LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	"LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"	"List[1, 2, 3]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test bounds::lazytable_cancel_first_suspension ... CASE	LazyTable-cancel-at-1	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1]", ["lower: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1]", ["lower: "])
ROW	LazyTable-cancel-at-1	"TerminatedEvaluation[\"Cancelled\"]"	"List[1]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1]"	1	1	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary"]	["boundary"]
ok
test bounds::lazytable_cancel_second_suspension ... CASE	LazyTable-cancel-at-2	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2]", ["lower: ", "upper: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2]", ["upper: "])
ROW	LazyTable-cancel-at-2	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2]"	"TerminatedEvaluation[\"Cancelled\"]"	"List[1, 2]"	2	2	true	"OK"	"OK"	"Some(Cancelled)"	"Some(Cancelled)"	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	[RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }]	["boundary", "boundary"]	["boundary", "boundary"]
ok
test bounds::lazytable_error_in_step_after_upper ... CASE	LazyTable-error-in-step-after-upper	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])
ROW	LazyTable-error-in-step-after-upper	"<error>"	"List[1, 2, 3]"	"<error>"	"List[1, 2, 3]"	1	1	true	"Other { message: \"uncaught throw: 9 with tag None\" }"	"Other { message: \"uncaught throw: 9 with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }]	["before", "after"]	["before", "after"]
ok
test bounds::lazytable_invalid_step ... CASE	LazyTable-invalid-step	resumed=(View { status: "OK", value: Some("LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: [] }, "List[1, 2, 3]", [])
ROW	LazyTable-invalid-step	"LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"	"List[1, 2, 3]"	"LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }]	[RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }]	[]	[]
ok
test bounds::lazytable_nonnumeric_lower_runs_later_bounds ... CASE	LazyTable-nonnumeric-lower	resumed=(View { status: "OK", value: Some("LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: ["upper", "step"] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: ["upper", "step"] }, "List[1, 2, 3]", [])
ROW	LazyTable-nonnumeric-lower	"LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"	"List[1, 2, 3]"	"LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }]	[RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }]	["upper", "step"]	["upper", "step"]
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

test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.35s

     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-bdeaa5d842f94483)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

Verbatim test/build result lines from `.temp/brief-564/final/ownership.log` (the complete raw output is archived there):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)
running 20 tests
test tests::ownership_graph::accumulator_coverage::every_self_reaching_arc_chain_participates ... uncovered accumulator ownership paths: 0
ok
test tests::ownership_graph::accumulator_coverage::fresh_names_and_additional_self_edges_are_checked ... ok
test tests::ownership_graph::discovery_report_without_inventory_input ... UnknownHostPayload count: 63
ok
test tests::ownership_graph::fixtures::coverage_is_not_inferred_from_test_like_filenames ... ok
test tests::ownership_graph::fixtures::fresh_names_cross_module_pair_and_recursion_after_comma ... FIXTURE REDISCOVERED fixture::AfterComma	crates/fixture/src/lib.rs	6	fixture::AfterComma.table/std::collections::HashMap[Own]/arg1/tuple/.1/std::boxed::Box[Own]/arg0 -> fixture::AfterComma
ok
test tests::ownership_graph::fixtures::inline_module_path_attribute_and_super_imports_resolve ... ok
test tests::ownership_graph::fixtures::manually_checked_definition_counts ... HAND CROSS-CHECK crates/rum-stdlib/src/associations/basic.rs: 3 definitions: LookupOutcome, LookupPlan, PreparedLookup
ok
test tests::ownership_graph::fixtures::named_generic_public_inputs_are_opaque_but_borrowed_inputs_are_not ... ok
test tests::ownership_graph::fixtures::opaque_constructs_are_located_and_inventory_removal_fails ... DELIBERATE UNKNOWN FAILURE (caught): missing inventory row: UnknownHostPayload	crates/fixture/src/lib.rs	3	31	fixture::Opaque.host/std::boxed::Box[Own]/arg0: opaque owned payload dyn Payload
ok
test tests::ownership_graph::fixtures::owning_ports_and_nonowning_storage_do_not_mix_instantiations ... ok
test tests::ownership_graph::fixtures::rediscovery_and_each_missing_mutual_member ... REDISCOVERED rum_stdlib::associations::basic::LookupOutcome	crates/rum-stdlib/src/associations/basic.rs	302	rum_stdlib::associations::basic::LookupOutcome.List.0/std::vec::Vec[Own]/arg0 -> rum_stdlib::associations::basic::LookupOutcome
ok
test tests::ownership_graph::fixtures::resolution_uses_declarations_not_container_spellings ... ok
test tests::ownership_graph::fixtures::unknown_configuration_cannot_invent_an_unconditional_cycle ... ok
test tests::ownership_graph::fixtures::unmodeled_lexical_modules_and_async_captures_are_explicit_unknowns ... ok
test tests::ownership_graph::host_payload_ledger::ownership::attribution_resolves_alias_scopes_and_excludes_nonowning_paths ... ok
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... HOST PAYLOAD LEDGER: 63 rows; 6 storage edges (3 Declared, 3 Unbounded); 57 PublicInput
ok
test tests::ownership_graph::host_payload_ledger::tests::metadata_duplicates_and_witnesses_are_enforced ... ok
test tests::ownership_graph::host_payload_ledger::tests::missing_stale_and_unrelated_expression_debt_are_rejected ... ok
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
ok
test tests::ownership_graph::source::root_set::syntax_root_discovers_added_recursive_type_and_restores_source ... OLD ROOTS: rum_syntax::ast::ScannerRootSetProbe invisible (no definition or self-reaching row)
ok
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1067 filtered out; finished in 59.67s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-44228b396944f9ba)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-ab1526217bb7e597)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 69 filtered out; finished in 0.00s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-bdeaa5d842f94483)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

Verbatim output from `.temp/brief-564/inventory/fingerprints.log` (the complete raw output is archived there):

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.20s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 1 test
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1086 filtered out; finished in 0.17s
```

The initial certificate-generation runs produced current cells but rejected the stale installed C01 cells, as expected; these are recorded failures, not green runs:

Verbatim output from `.temp/brief-564/inventory/final/certificates-default-generated.log` (the complete raw output is archived there):

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 29.71s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 1 test
test tests::retention_closure::local::production::local_production_acceptance ... body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0

thread 'tests::retention_closure::local::production::local_production_acceptance' panicked at crates/rum-stdlib/src/tests/retention_closure/local/production.rs:98:6:
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED

failures:

failures:
    tests::retention_closure::local::production::local_production_acceptance

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1086 filtered out; finished in 17.68s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```

Verbatim output from `.temp/brief-564/inventory/final/certificates-no-default-generated.log` (the complete raw output is archived there):

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 13s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5d6fedc676d25fda)

running 1 test
test tests::retention_closure::local::production::local_production_acceptance ... body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0

thread 'tests::retention_closure::local::production::local_production_acceptance' panicked at crates/rum-stdlib/src/tests/retention_closure/local/production.rs:98:6:
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
FAILED

failures:

failures:
    tests::retention_closure::local::production::local_production_acceptance

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1080 filtered out; finished in 17.51s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```

After installing the actual four cells, both final-source feature gates passed:

Verbatim output from `.temp/brief-564/inventory/final/certificates-default.log` (the complete raw output is archived there):

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.42s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 1 test
test tests::retention_closure::local::production::local_production_acceptance ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1086 filtered out; finished in 17.82s

     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-44228b396944f9ba)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-ab1526217bb7e597)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s

     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-d206a57361f34ffd)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 69 filtered out; finished in 0.00s

     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-bdeaa5d842f94483)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

Verbatim output from `.temp/brief-564/inventory/final/certificates-no-default.log` (the complete raw output is archived there):

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 13.53s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-5d6fedc676d25fda)

running 1 test
test tests::retention_closure::local::production::local_production_acceptance ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1080 filtered out; finished in 17.46s

     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-6199ab42789c491a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-fb613c76a0ef7535)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 47 filtered out; finished in 0.00s

     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-e7f8ba5487f6d7b9)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 69 filtered out; finished in 0.00s

     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-c6f8c05860b98e46)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s
```

Verbatim output from `.temp/brief-564/final/runtime.log` (the complete raw output is archived there):

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.72s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-95cd5fbb46da4f1c)

running 1097 tests
test runtime::accounting_mode::frozen::option_tests::option_frozen_complement_repeated_resume_and_abandon ... ok
test runtime::accounting_mode::frozen::option_tests::option_loop_frame_resumes_with_another_arc_owner ... ok
test runtime::accounting_mode::frozen::option_tests::option_loop_frame_traces_inputs_and_accumulated_prefix ... ok
test runtime::accounting_mode::frozen::option_tests::option_reentry_domain_replacement_expires_resume ... ok
test runtime::accounting_mode::frozen::option_tests::option_reentry_registration_does_not_mint_certification ... ok
test runtime::accounting_mode::frozen::option_tests::option_working_copy_checkpoint_and_shared_frame_release ... ok
test runtime::accounting_mode::frozen::tests::option_rule_shared_frame_charges_working_result_and_preserves_last_owner ... ok
test runtime::accounting_mode::frozen::tests::retention_shared_frame_keeps_roots_and_lease_until_last_owner_drops ... ok
test runtime::accounting_mode::options::tests::option_flat_late_arm_accounts_slots_and_preflights_freeze ... ok
test runtime::accounting_mode::options::tests::option_handoff_checks_domain_identity_when_generations_collide ... ok
test runtime::accounting_mode::options::tests::option_owner_capacity_and_callback_adoption ... ok
test runtime::accounting_mode::options::tests::option_owner_guest_error_and_unwind_release ... ok
test runtime::accounting_mode::options::tests::option_owner_handoff_and_late_arm_keep_original_visible ... ok
test runtime::accounting_mode::options::tests::option_owner_snapshot_charges_defaults_and_copies ... ok
test runtime::accounting_mode::rule::tests::exits::option_rule_abandon_reset_and_late_arm_frame ... ok
test runtime::accounting_mode::rule::tests::exits::option_rule_frame_refusal_abandons_without_a_transfer ... ok
test runtime::accounting_mode::rule::tests::exits::option_rule_late_arm_after_callback_and_during_input ... ok
test runtime::accounting_mode::rule::tests::exits::option_rule_repeated_nested_input_transfers_once ... ok
test runtime::accounting_mode::rule::tests::option_rule_callback_adoption_refuses_before_growth_and_preserves_errors ... ok
test runtime::accounting_mode::rule::tests::option_rule_clone_and_shell_contract ... ok
test runtime::accounting_mode::rule::tests::option_rule_guest_exits_are_lazy_and_release ... ok
test runtime::accounting_mode::rule::tests::option_rule_late_arm_observes_existing_prefix ... ok
test runtime::accounting_mode::rule::tests::option_rule_name_is_adopted_before_growth ... ok
test runtime::accounting_mode::rule::tests::option_rule_overflow_and_partial_failure_release ... ok
test runtime::accounting_mode::rule::tests::option_rule_unarmed_shared_head_keeps_canonical_rule ... ok
test runtime::accounting_mode::tests::compile::retention_compile_controls_precede_pinned_failures ... ok
test runtime::accounting_mode::tests::exits::retention_bridge_releases_prefix_on_guest_error_and_panic ... ok
test runtime::accounting_mode::tests::exits::retention_frame_factory_is_lazy_and_refusal_abandons_pending_input ... ok
test runtime::accounting_mode::tests::exits::retention_nested_unarmed_bridges_keep_every_outer_prefix ... ok
test runtime::accounting_mode::tests::exits::retention_overflow_stays_invalid_after_release_and_old_ledger_is_isolated ... ok
test runtime::accounting_mode::tests::exits::retention_suspension_checks_actual_continuation_roots ... ok
test runtime::accounting_mode::tests::retention_activation_rejects_live_unarmed_prefix_before_guest_body ... ok
test runtime::accounting_mode::tests::retention_lifetime_normal_early_error_drop_and_unwind ... ok
test runtime::accounting_mode::tests::retention_r_is_separate_from_s_and_both_admission_entries_see_it ... ok
test runtime::accounting_mode::tests::retention_suspension_transfers_once_resumes_twice_and_abandons ... ok
test runtime::accounting_mode::tests::retention_unarmed_operations_do_not_construct_receipts_or_walk_weights ... ok
test runtime::accounting_mode::tests::retention_unarmed_suspended_capacity_is_visible_to_later_arm ... ok
test runtime::accumulator_chain::tests::exclusive_depth_16 ... ok
test runtime::accumulator_chain::tests::exclusive_depth_64 ... ok
test runtime::accumulator_chain::tests::live_weak_does_not_prevent_drain ... ok
test runtime::accumulator_chain::tests::shared_tail_stops_then_drains ... ok
test runtime::accumulator_chain::tests::wrapper_is_transparent_and_extraction_preserves_ownership ... ok
test runtime::builtin_catalog::tests::core_catalog_drives_both_dispatch_and_default_attributes ... ok
test runtime::builtin_catalog::tests::moved_stdlib_system_attributes_remain_kernel_known_without_registration ... ok
test runtime::call_preparation::tests::planner_allows_raw_sequence_splicing_for_held_arguments ... ok
test runtime::call_preparation::tests::planner_keeps_held_arguments_verbatim_under_holdallcomplete ... ok
test runtime::call_preparation::tests::planner_respects_evaluate_wrapper_for_held_arguments ... ok
test runtime::call_preparation::tests::planner_respects_unevaluated_wrapper_for_nonheld_arguments ... ok
test runtime::capabilities::tests::capability_policy_allowlist_is_default_deny ... ok
test runtime::capabilities::tests::capability_policy_builder_can_allow_and_deny_individual_capabilities ... ok
test runtime::capabilities::tests::capability_policy_can_explicitly_allow_diagnostics ... ok
test runtime::capabilities::tests::capability_policy_defaults_to_unrestricted ... ok
test runtime::capabilities::tests::capability_policy_narrowing_preserves_existing_restrictions ... ok
test runtime::capabilities::tests::catalog_capabilities_classify_clock_builtins ... ok
test runtime::capabilities::tests::catalog_capabilities_classify_diagnostic_builtins ... ok
test runtime::capabilities::tests::catalog_capabilities_classify_io_builtins ... ok
test runtime::definitions::application::replace_all::plan::frame_root_trace_tests::rewrite_rule_plan_visits_the_exact_rule_roots ... ok
test runtime::definitions::definition_lhs_preparation::syntax_policy::tests::every_registered_matcher_head_has_an_explicit_lhs_policy ... ok
test runtime::definitions::definition_lhs_preparation::tests::already_canonical_lhs_reuses_its_shared_arguments ... ok
test runtime::definitions::unique_symbols::tests::keeps_small_symbol_sets_on_the_linear_path ... ok
test runtime::definitions::unique_symbols::tests::preserves_first_seen_order_across_the_adaptive_index_threshold ... ok
test runtime::definitions::value_table_assignment::tests::abandoning_prepared_value_table_keeps_old_definitions_and_skips_pending_lhs ... ok
test runtime::eval_call_prepare::abandonment_tests::abandoning_prepared_arguments_and_restores_skips_remaining_arguments ... ok
test runtime::evaluation_cache::tests::append_to_args_become_exclusive_after_definition_take_and_cache_entry_removal ... ok
test runtime::evaluation_cache::tests::append_to_preserves_an_aliased_list_value ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_absent_stale_and_dirty_proofs_recover_by_recomputation ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_direct_update_keeps_args_identity_and_reinserts_proof ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_direct_update_polls_interrupt_before_mutation ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_direct_update_rolls_back_append_and_replace ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_nested_multi_and_bare_part_targets_use_fallback ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_observable_or_constrained_environments_use_fallback ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_recovery_falls_back_when_data_became_unstable ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_update_preserves_an_aliased_value ... ok
test runtime::evaluation_cache::tests::association_stability_index_preserves_distinct_and_duplicate_verdicts ... ok
test runtime::evaluation_cache::tests::association_stability_tail::cache_cap_postorder_insertion_trace_matches_recursive_front ... ok
test runtime::evaluation_cache::tests::association_stability_tail::dirty_and_generation_cache_traces_match_recursive_front ... ok
test runtime::evaluation_cache::tests::association_stability_tail::duplicate_key_cache_trace_matches_recursive_front ... ok
test runtime::evaluation_cache::tests::association_stability_tail::recursive_front_exhaustion_owns_intervening_ordinary_calls ... ok
test runtime::evaluation_cache::tests::association_stability_tail::rule_delayed_cache_trace_matches_recursive_front ... ok
test runtime::evaluation_cache::tests::association_stability_tail::unstable_early_child_cache_trace_matches_recursive_front ... ok
test runtime::evaluation_cache::tests::call_stability_profile_reports_per_source_head_outcomes ... ignored, release-only profiling gate; run with RUM_PROFILE_EVALUATION_CACHE=1
test runtime::evaluation_cache::tests::dirty_pointer_matched_entry_is_evicted_without_proof ... ok
test runtime::evaluation_cache::tests::evaluation_cache_is_invalidated_by_attribute_change ... ok
test runtime::evaluation_cache::tests::evaluation_cache_is_invalidated_by_new_down_value ... ok
test runtime::evaluation_cache::tests::generation_stale_pointer_matched_entry_is_evicted_without_proof ... ok
test runtime::evaluation_cache::tests::path_eviction_removes_only_the_named_exact_call_nodes ... ok
test runtime::evaluation_cache::tests::pointer_mismatched_entry_is_left_untouched ... ok
test runtime::evaluation_cache::tests::proof_reinsertion_uses_current_generation_dirty_snapshot_and_bloom ... ok
test runtime::evaluation_cache::tests::removing_valid_call_stability_entry_preserves_evaluation_outcome ... ok
test runtime::evaluation_cache::tests::repeated_unrelated_writes_do_not_bump_call_stability_generation ... ok
test runtime::evaluation_cache::tests::repeated_writes_to_one_symbol_keep_invalidation_sound ... ok
test runtime::evaluation_cache::tests::shared_args_make_mut_profiles_only_cow_and_preserves_alias_values ... ok
test runtime::evaluation_cache::tests::stability_cache_survives_unrelated_symbol_writes ... ok
test runtime::evaluation_cache::tests::stability_native_stack_poll_is_strided_and_fails_closed ... ok
test runtime::evaluation_cache::tests::valid_call_stability_proof_preflight_is_non_consuming_and_reuses_validity ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::accumulator_is_abandoned_mid_iteration_without_resuming_body ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::appended_chain_abandonment_uses_drain ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::into_vec_preserves_prefix_and_tail_order_when_unique_or_shared ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::shared_tail_survives_abandoned_clone ... ok
test runtime::kernel_data::rewrite_data::suspended_state::tests::abandoning_shared_rewrite_queue_preserves_remaining_binding_order ... ok
test runtime::kernel_data::structure_data::exact_paths::tests::collect_exact_paths_for_position_spec_forced_front_preserves_branch_order ... ok
test runtime::kernel_data::structure_data::exact_paths::tests::exact_expr_at_path_forced_front_matches_each_branched_witness ... ok
test runtime::kernel_data::structure_data::exact_paths::tests::replace_exact_path_forced_front_matches_head_and_child_edges ... ok
test runtime::kernel_data::structure_data::replace::plan::tests::replace_plan_resolves_duplicates_and_prefixes_during_insertion ... ok
test runtime::kernel_data::structure_data::replace::plan::tests::root_target_discards_existing_and_later_descendants ... ok
test runtime::kernel_data::structure_data::replace::plan_application::tests::iterative_replace_plan_rebuild_preserves_unchanged_sibling_storage ... ok
test runtime::kernel_data::structure_data::replace::suspended_state::tests::abandoning_guarded_target_branch_preserves_surviving_chunks_and_updates ... ok
test runtime::lowering::tests::branched_lowering_certifies_front_and_tail ... ok
test runtime::lowering::tests::branched_lowering_matches_original_exact_result ... ok
test runtime::lowering::tests::head_chains_below_bound_preserve_every_element ... ok
test runtime::lowering::tests::lowering_error_restores_front_budget ... ok
test runtime::lowering::tests::public_lowering_paths_reject_manual_trees_above_bound ... ok
test runtime::lowering::tests::tail_preserves_sugar_and_symbol_resolution_order ... ok
test runtime::options::core::rule_tests::option_rule_c02_non_call_immediate_and_delayed ... ok
test runtime::options::reentry_tests::option_reentry_compile_pass_precedes_pinned_compile_fail ... ok
test runtime::pattern_syntax::tests::matcher_pattern_registry_round_trips_system_heads_only ... ok
test runtime::session::guest_loading::tests::abandonment_cleanup_loading_file_drains_nested_worklist ... ok
test runtime::session::guest_loading::tests::cancellation_restores_nested_snapshots_inner_to_outer_at_each_input ... ok
test runtime::session::guest_loading::tests::dependency_exit_restores_package_frame_and_preserves_earlier_definitions ... ok
test runtime::session::guest_loading::tests::guest_environment_restores_handled_error_tail_and_memory_limit_before_child ... ok
test runtime::session::guest_loading::tests::message_scope_and_print_sink_are_not_replayed_across_inputs ... ok
test runtime::session::guest_loading::tests::nested_files_close_once_and_all_owner_queues_empty ... ok
test runtime::session::guest_loading::tests::paused_guest_form_reinstalls_original_deadline_before_child_frames ... ok
test runtime::session::guest_loading::tests::pending_frames_trace_snapshot_definitions_and_previous_form_values ... ok
test runtime::session::session_requests::tests::eval_error_mapper_preserves_non_request_surface_limit_codes ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_alpha_rename_does_not_rewrite_inserted_bound_values ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_sequence_splice_preserves_spliced_item_shared_nodes ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_substitution_preserves_unchanged_sibling_call_shared_nodes ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_substitution_reuses_bound_call_shared_nodes ... ok
test runtime::substitution::lexical_substitution::tests::explicit_function_parameters_shadow_outer_bindings ... ok
test runtime::substitution::lexical_substitution::tests::rule_rhs_substitution_respects_pattern_locals ... ok
test runtime::substitution::lexical_substitution::tests::with_body_treats_binding_names_as_shadowed ... ok
test runtime::suspension::primitive::tests::binding_vec_continuation_round_trips_multiple_symbol_maps ... ok
test runtime::suspension::primitive::tests::option_vec_expr_continuation_round_trips_some_none_and_fallback_payloads ... ok
test runtime::suspension::rewrite_rules::tests::option_rewrite_rule_info_continuation_round_trips_some_none_and_fallbacks ... ok
test runtime::suspension::traversal::tests::option_cases_level_spec_continuation_round_trips_some_none_and_bare_spec ... ok
test runtime::tests::arithmetic::exact_division_and_negative_integer_powers_produce_rationals ... ok
test runtime::tests::arithmetic::exact_integer_arithmetic_evaluates_core_heads ... ok
test runtime::tests::arithmetic::exact_integer_mod_quotient_gcd_lcm_and_powermod_follow_floor_style_semantics ... ok
test runtime::tests::arithmetic::factorial_respects_integer_work_limit ... ok
test runtime::tests::arithmetic::plus_and_times_fold_numeric_parts_without_forcing_symbolic_algebra ... ok
test runtime::tests::arithmetic::power_integer_work_limit_allows_normal_and_trivial_results ... ok
test runtime::tests::arithmetic::power_integer_work_limit_guards_large_denominators ... ok
test runtime::tests::arithmetic::power_memory_projection_refuses_before_large_allocation ... ok
test runtime::tests::arithmetic::power_reads_the_live_integer_work_limit ... ok
test runtime::tests::arithmetic::power_realistic_memory_ceiling_admits_one_million_bit_result ... ok
test runtime::tests::arithmetic::power_respects_integer_work_limit_for_large_positive_exponents ... ok
test runtime::tests::arithmetic::powermod_negative_exponents_use_modular_inverse ... ok
test runtime::tests::arithmetic::rational_atoms_expose_a_rational_head_for_patterns ... ok
test runtime::tests::arithmetic::round_handles_exact_numbers_decimal_literals_and_half_even_ties ... ok
test runtime::tests::arithmetic::scalar_math_builtins_exact_cases ... ok
test runtime::tests::arithmetic::scalar_math_builtins_report_declared_attributes ... ok
test runtime::tests::arithmetic::scalar_math_builtins_thread_over_lists_and_leave_skipped_forms_symbolic ... ok
test runtime::tests::arithmetic::total_evaluates_documented_level_forms_and_leaves_invalid_specs_inert ... ok
test runtime::tests::arithmetic::wide_plus_and_times_group_late_symbolic_duplicates ... ok
test runtime::tests::association_lookup_cache::association_lookup_index_weak_identity_survives_cow_updates ... ok
test runtime::tests::association_lookup_cache::repeated_wide_association_reads_build_one_linear_index ... ok
test runtime::tests::association_part_cursor::below_threshold_part_reads_stay_on_the_linear_scan_path ... ok
test runtime::tests::association_part_cursor::duplicate_association_keys_keep_first_position_and_last_value ... ok
test runtime::tests::association_part_cursor::part_reads_reuse_the_lookup_cache_and_observe_cow_writes ... ok
test runtime::tests::association_part_cursor::wide_association_single_key_and_position_reads_return_exact_values ... ok
test runtime::tests::association_write_index::appended_keys_after_a_wide_read_stay_readable ... ok
test runtime::tests::association_write_index::repeated_replacement_writes_stay_indexed ... ok
test runtime::tests::association_write_index::rewriting_an_appended_key_replaces_it_through_the_index ... ok
test runtime::tests::association_write_index::small_association_writes_bypass_the_lookup_cache ... ok
test runtime::tests::association_write_recovery::association_incremental_writes_preserve_ordinary_symbol_semantics_past_the_cache_size_cap ... ok
test runtime::tests::association_write_recovery::association_incremental_writes_stay_direct_past_the_cache_size_cap ... ok
test runtime::tests::comparison_lowering::comparison_lowering_accepts_a_uniform_chain_at_the_boundary ... ok
test runtime::tests::comparison_lowering::comparison_lowering_bounds_direct_chain_nodes ... ok
test runtime::tests::comparison_lowering::comparison_lowering_keeps_parenthesized_subtrees_independent ... ok
test runtime::tests::comparison_lowering::comparison_lowering_preserves_resolver_call_count_and_order ... ok
test runtime::tests::comparison_lowering::comparison_parser_rejects_a_long_uniform_chain_gracefully ... ok
test runtime::tests::context_and_loader::bare_symbols_search_context_path_before_current_context ... ok
test runtime::tests::context_and_loader::begin_and_end_balance_context_and_context_path ... ok
test runtime::tests::context_and_loader::begin_package_and_end_package_restore_outer_context_and_export_package_context ... ok
test runtime::tests::context_and_loader::begin_package_dependencies_stay_on_private_context_path_when_already_loaded ... ok
test runtime::tests::context_and_loader::begin_package_expands_relative_context_specs_against_the_current_context ... ok
test runtime::tests::context_and_loader::begin_package_loads_present_dependency_and_enters_package ... ok
test runtime::tests::context_and_loader::begin_package_missing_dependency_fails_closed_and_restores_state ... ok
test runtime::tests::context_and_loader::begin_package_second_dependency_failure_restores_state_and_package_list ... ok
test runtime::tests::context_and_loader::begin_package_usage_messages_make_public_symbols_visible_from_private_context ... ok
test runtime::tests::context_and_loader::context_builtin_reports_session_and_symbol_contexts_without_evaluating_symbols ... ok
test runtime::tests::context_and_loader::context_path_and_packages_assignments_update_session_fields ... ok
test runtime::tests::context_and_loader::context_variables_and_builtin_inspection_reflect_session_contexts ... ok
test runtime::tests::context_and_loader::contexts_lists_known_session_contexts ... ok
test runtime::tests::context_and_loader::explicitly_protected_session_configuration_variables_reject_assignments ... ok
test runtime::tests::context_and_loader::file_loading::findfile_resolves_package_names_and_file_wrappers ... ok
test runtime::tests::context_and_loader::file_loading::get_and_needs_syntax_messages_report_the_loaded_file_location ... ok
test runtime::tests::context_and_loader::file_loading::get_cycle_termination_does_not_abort_remaining_file_forms ... ok
test runtime::tests::context_and_loader::file_loading::get_loads_a_script_file_and_returns_the_last_value ... ok
test runtime::tests::context_and_loader::file_loading::get_propagates_exit_without_collapsing_it_to_failed_load ... ok
test runtime::tests::context_and_loader::file_loading::get_rolls_back_loader_side_effects_and_collapses_failed_file_messages ... ok
test runtime::tests::context_and_loader::file_loading::get_treats_resource_termination_as_failed_load_and_rolls_back ... ok
test runtime::tests::context_and_loader::file_loading::loader_builtins_accept_resolved_load_target_specs ... ok
test runtime::tests::context_and_loader::file_loading::missing_findfile_emits_runtime_message ... ok
test runtime::tests::context_and_loader::file_loading::missing_needs_returns_failed_and_emits_messages ... ok
test runtime::tests::context_and_loader::file_loading::needs_accepts_resolved_context_and_file_specs ... ok
test runtime::tests::context_and_loader::file_loading::needs_loads_package_once_and_tracks_loaded_packages ... ok
test runtime::tests::context_and_loader::file_loading::needs_supports_explicit_file_targets ... ok
test runtime::tests::context_and_loader::file_loading::nested_get_prefers_the_current_loader_directory_over_package_search_paths ... ok
test runtime::tests::context_and_loader::path_assignment_remains_writable_without_explicit_protect ... ok
test runtime::tests::context_and_loader::script_forms::eval_script_pins_the_second_form_syntax_error_message ... ok
test runtime::tests::context_and_loader::script_forms::eval_script_preserves_messages_emitted_before_a_later_error ... ok
test runtime::tests::context_and_loader::script_forms::lex_fallback_preserves_normalized_crlf_values_and_file_error_spans ... ok
test runtime::tests::context_and_loader::script_forms::parse_script_forms_matches_the_line_driven_splitter_over_the_corpus ... ok
test runtime::tests::context_and_loader::script_forms::split_script_into_forms_skips_initial_shebang_trivia ... ok
test runtime::tests::context_and_loader::script_forms::split_script_into_forms_stops_hard_syntax_errors_from_poisoning_later_forms ... ok
test runtime::tests::context_and_loader::script_forms::split_script_into_forms_treats_multiline_comments_as_incomplete_trivia ... ok
test runtime::tests::debug_tools::echo_family_prints_and_returns_values_in_the_current_first_pass_subset ... ok
test runtime::tests::debug_tools::on_and_off_gate_matching_messages_at_session_scope ... ok
test runtime::tests::debug_tools::rum_profile_hooks_are_inert_without_profile_env ... ok
test runtime::tests::debug_tools::short_uses_the_current_canonical_text_truncation_subset ... ok
test runtime::tests::debug_tools::trace_detail_form_filter_matches_the_underlying_expression_not_the_association ... ok
test runtime::tests::debug_tools::trace_detail_has_the_same_list_nesting_shape_as_trace ... ok
test runtime::tests::debug_tools::trace_detail_identifies_the_downvalue_that_won_among_overlapping_definitions ... ok
test runtime::tests::debug_tools::trace_detail_preserves_definition_symbol_kind_and_value ... ok
test runtime::tests::debug_tools::trace_detail_returns_the_full_evaluation_pass_association_without_leaking_trace_state ... ok
test runtime::tests::debug_tools::trace_detail_supports_levelspec_and_heads_options ... ok
test runtime::tests::debug_tools::trace_print_emits_trace_lines_and_returns_the_underlying_value ... ok
test runtime::tests::debug_tools::trace_print_uses_the_same_nested_structure_for_deeper_trace_branches ... ok
test runtime::tests::debug_tools::trace_returns_a_first_pass_structured_expression_list_without_leaking_session_trace_state ... ok
test runtime::tests::debug_tools::trace_uses_recorded_depths_to_keep_nested_branches_stable ... ok
test runtime::tests::definition_lhs_preparation::active_definition_lhs_positions_complete_own_value_evaluation_for_both_set_forms ... ok
test runtime::tests::definition_lhs_preparation::active_lhs_and_pattern_test_pattern_operands_still_prepare_once ... ok
test runtime::tests::definition_lhs_preparation::active_nested_calls_dispatch_held_nested_calls_do_not_and_callable_heads_prepare_first ... ok
test runtime::tests::definition_lhs_preparation::adding_hold_attributes_after_an_existing_definition_keeps_old_and_new_identities_live ... ok
test runtime::tests::definition_lhs_preparation::clear_then_reinstall_has_no_stale_prepared_identity_or_matcher ... ok
test runtime::tests::definition_lhs_preparation::definition_lhs_pattern_regions_prepare_constraints_but_defer_binders_and_predicates ... ok
test runtime::tests::definition_lhs_preparation::definition_lhs_preparation_obeys_evaluate_unevaluated_and_sequence_controls ... ok
test runtime::tests::definition_lhs_preparation::held_definition_lhs_positions_keep_raw_symbols_and_source_lookup_reaches_them ... ok
test runtime::tests::definition_lhs_preparation::held_ground_flat_and_orderless_identities_are_reachable_from_both_forms ... ok
test runtime::tests::definition_lhs_preparation::matcher_wrappers_keep_literal_pattern_regions_raw_and_prepare_nested_constraints ... ok
test runtime::tests::definition_lhs_preparation::one_metadata_snapshot_controls_every_argument_even_after_an_earlier_mutation ... ok
test runtime::tests::definition_lhs_preparation::reassigned_then_cleared_lhs_own_value_changes_only_future_definition_identities ... ok
test runtime::tests::definition_store_sealing::definition_store_and_targeted_removal_are_sealed_behind_prepared_tokens ... ok
test runtime::tests::definition_store_sealing::prepared_store_and_removal_call_site_inventory_is_closed ... ok
test runtime::tests::definitions_and_rules::defaultvalues_and_definition_include_explicit_syntax_information ... ok
test runtime::tests::definitions_and_rules::defaultvalues_lists_optional_defaults_and_clearall_removes_them ... ok
test runtime::tests::definitions_and_rules::definition_inspection_builtins_accept_symbol_name_strings ... ok
test runtime::tests::definitions_and_rules::definition_returns_empty_hold_for_symbols_without_definitions ... ok
test runtime::tests::definitions_and_rules::definition_returns_held_first_pass_definition_statements ... ok
test runtime::tests::definitions_and_rules::delayed_self_alias_preserves_downvalues ... ok
test runtime::tests::definitions_and_rules::dynamic_iterator_lhs_values_are_normalized_with_the_existing_cycle_guard ... ok
test runtime::tests::definitions_and_rules::immediate_self_alias_preserves_downvalues ... ok
test runtime::tests::definitions_and_rules::information_returns_first_pass_symbol_summary ... ok
test runtime::tests::definitions_and_rules::information_uses_catalog_and_symbol_name_usage_fallbacks_and_stored_message_tags ... ok
test runtime::tests::definitions_and_rules::iterator_bindings_normalize_set_lhs_for_every_eager_iterator_and_assignment ... ok
test runtime::tests::definitions_and_rules::iterator_lhs_normalization_covers_nested_and_shadowed_iterators ... ok
test runtime::tests::definitions_and_rules::non_iterator_lhs_binding_controls_keep_their_exact_results ... ok
test runtime::tests::definitions_and_rules::ownvalues_lists_immediate_and_delayed_symbol_definitions ... ok
test runtime::tests::definitions_and_rules::real_iterator_definitions_precede_dynamic_values_and_clear_reveals_them_again ... ok
test runtime::tests::definitions_and_rules::self_referential_delayed_ownvalue_returns_symbol_without_message ... ok
test runtime::tests::definitions_and_rules::self_referential_ownvalue_returns_symbol_without_message ... ok
test runtime::tests::definitions_and_rules::set_assigns_simple_symbol_and_lookup_works ... ok
test runtime::tests::definitions_and_rules::setdelayed_stores_rhs_without_immediate_evaluation ... ok
test runtime::tests::definitions_and_rules::valueq_uses_rollback_trial_evaluation_without_side_effects ... ok
test runtime::tests::definitions_and_rules_cases::cases_applies_rule_transformations ... ok
test runtime::tests::definitions_and_rules_cases::cases_can_match_the_root_expression ... ok
test runtime::tests::definitions_and_rules_cases::cases_collects_nested_matches ... ok
test runtime::tests::definitions_and_rules_cases::cases_does_not_treat_heads_as_separate_candidates ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_condition_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_except_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_optional_and_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_preserves_immediate_and_delayed_rule_behavior ... ok
test runtime::tests::definitions_and_rules_cases::cases_rule_rhs_splices_named_blanksequence_matches ... ok
test runtime::tests::definitions_and_rules_cases::cases_supports_exact_range_negative_and_infinity_level_specs ... ok
test runtime::tests::definitions_and_rules_cases::cases_supports_explicit_head_traversal_control ... ok
test runtime::tests::definitions_and_rules_cases::cases_supports_standard_integer_level_specs ... ok
test runtime::tests::definitions_and_rules_cases::cases_traverses_depth_first_with_leaves_before_roots_and_respects_match_limits ... ok
test runtime::tests::definitions_and_rules_cases::deletecases_can_delete_heads_and_association_values ... ok
test runtime::tests::definitions_and_rules_cases::deletecases_deletes_top_level_matches_by_default ... ok
test runtime::tests::definitions_and_rules_cases::deletecases_supports_standard_levelspecs_and_limits ... ok
test runtime::tests::definitions_and_rules_part2::compiled_matcher_matches_interpreted_matcher_on_supported_subset ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_can_substitute_multiple_named_patterns ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_matches_after_argument_evaluation ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_splices_named_blanknullsequence_into_empty_lists ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_splices_named_blanksequence_into_rhs_calls ... ok
test runtime::tests::definitions_and_rules_part2::inspect_definition_catalog_reports_kinds_and_delayedness ... ok
test runtime::tests::definitions_and_rules_part2::interpreted_matcher_owned_can_resume_sequence_conditions_across_input ... ok
test runtime::tests::definitions_and_rules_part2::lowers_named_pattern_assignment ... ok
test runtime::tests::definitions_and_rules_part2::lowers_slots_and_pure_functions ... ok
test runtime::tests::definitions_and_rules_part2::matchq_operator_form_is_inert_and_applies_to_data ... ok
test runtime::tests::definitions_and_rules_part2::matchq_reports_false_for_non_matching_pattern ... ok
test runtime::tests::definitions_and_rules_part2::matchq_reports_true_for_matching_pattern ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_blanknullsequence_in_argument_lists ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_blanksequence_in_argument_lists ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_conditions_on_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_except_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_optional_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_patterntest_on_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_plain_blank_arguments_under_fixed_heads ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_part2::named_slots_fill_from_rules_and_association_arguments ... ok
test runtime::tests::definitions_and_rules_part2::set_supports_immediate_pattern_based_downvalues_and_subvalues ... ok
test runtime::tests::definitions_and_rules_part2::setdelayed_stores_and_applies_simple_downvalue ... ok
test runtime::tests::definitions_and_rules_part2::unmatched_downvalue_leaves_call_in_place ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rhs_substitution_does_not_capture_inserted_values_in_lexical_scopes ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rhs_substitution_keeps_inserted_compound_values_outside_new_locals ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rhs_substitution_preserves_same_name_local_binding_behavior ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rule_uses_current_rhs_value_at_application_time ... ok
test runtime::tests::definitions_and_rules_part3::flat_and_orderless_flat_heads_regroup_for_match_replace_and_replacelist ... ok
test runtime::tests::definitions_and_rules_part3::immediate_rule_captures_current_rhs_value ... ok
test runtime::tests::definitions_and_rules_part3::replace_and_replaceat_exact_rule_index_preserve_first_rule_priority ... ok
test runtime::tests::definitions_and_rules_part3::replace_defaults_to_the_root_level ... ok
test runtime::tests::definitions_and_rules_part3::replace_heads_option_controls_head_traversal ... ok
test runtime::tests::definitions_and_rules_part3::replace_rewrites_association_values_without_touching_keys ... ok
test runtime::tests::definitions_and_rules_part3::replace_supports_levelspecs_and_rule_lists ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_applies_rule_list_in_order ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_applies_single_rule ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_exact_rule_index_preserves_first_rule_priority ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_condition_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_except_and_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_shape_prefilter_stays_conservative_for_head_attributes ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_splices_named_blanksequence_into_rhs_calls ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_traverses_nested_expressions ... ok
test runtime::tests::definitions_and_rules_part3::replacelist_collects_multiple_backtracking_matches_for_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replacelist_supports_operator_form_rule_order_and_max_results ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_delayed_rule_limit_returns_terminated_value ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_limit_returns_terminated_value_for_non_terminating_rewrite_cycle ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_max_iterations_option_controls_effective_limit ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_reaches_a_fixed_point ... ok
test runtime::tests::definitions_and_rules_part3::rewrite_family_operator_forms_are_inert_and_apply_to_data ... ok
test runtime::tests::definitions_and_rules_part3::rewrite_options_report_new_defaults ... ok
test runtime::tests::definitions_and_rules_part4::function_accepts_explicit_attributes_for_application ... ok
test runtime::tests::definitions_and_rules_patterns::alternatives_downvalue_matches_each_branch ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_binds_multiple_named_args ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_falls_back_for_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_falls_back_for_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_handles_named_typed_blank_hit ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_keeps_typed_blank_miss_unevaluated ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_preserves_exact_vs_pattern_precedence ... ok
test runtime::tests::definitions_and_rules_patterns::conditioned_downvalue_matches_only_when_test_passes ... ok
test runtime::tests::definitions_and_rules_patterns::except_downvalue_prefilter_stays_conservative ... ok
test runtime::tests::definitions_and_rules_patterns::except_optional_and_repeated_downvalues_match_expected_shapes ... ok
test runtime::tests::definitions_and_rules_patterns::longest_and_shortest_control_sequence_pattern_greediness ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_condition_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_except_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_optional_and_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_rejects_malformed_repetition_bounds_with_head_specific_messages ... ok
test runtime::tests::definitions_and_rules_patterns::orderless_heads_backtrack_across_exact_and_patternsequence_groups ... ok
test runtime::tests::definitions_and_rules_patterns::orderless_sequence_matching_still_accepts_small_normal_cases ... ok
test runtime::tests::definitions_and_rules_patterns::orderlesspatternsequence_uses_orderless_group_matching_inside_orderless_heads ... ok
test runtime::tests::definitions_and_rules_patterns::outer_conditioned_downvalue_is_stored_and_applied ... ok
test runtime::tests::definitions_and_rules_patterns::patternsequence_flattens_argument_patterns_in_matching_rules_and_definitions ... ok
test runtime::tests::definitions_and_rules_patterns::patterntest_downvalue_matches_only_when_test_passes ... ok
test runtime::tests::definitions_and_rules_patterns::rule_lhs_evaluates_values_without_evaluating_pattern_names ... ok
test runtime::tests::definitions_and_rules_patterns::verbatim_matches_literal_pattern_language_forms ... ok
test runtime::tests::definitions_and_rules_patterns::wide_downvalue_store_index_preserves_specificity_and_alpha_replacement ... ok
test runtime::tests::definitions_and_rules_patterns::wide_ground_downvalue_index_preserves_pattern_priority_fallback_and_invalidation ... ok
test runtime::tests::definitions_and_rules_patterns::wide_ground_downvalue_scan_falls_back_to_live_order_after_matcher_mutation ... ok
test runtime::tests::definitions_and_rules_patterns::wide_pattern_downvalue_store_index_replaces_renamed_condition_pattern ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::inspect_rewrite_rules_can_suspend_and_resume_with_host_input ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::inspect_rewrite_rules_reports_normalized_rules_and_delayed_flags ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::normalize_and_inspect_rewrite_rules_rebuild_rhs_guards ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::normalize_rules_lists_single_rules_rule_lists_and_preserves_delayedness ... ok
test runtime::tests::definitions_and_rules_tags::clear_name_patterns_expand_before_mutation_and_preserve_exact_targets ... ok
test runtime::tests::definitions_and_rules_tags::clearall_and_remove_expand_name_patterns ... ok
test runtime::tests::definitions_and_rules_tags::downvalues_lists_stored_pattern_definitions_in_order ... ok
test runtime::tests::definitions_and_rules_tags::exact_downvalue_dispatch_preserves_ground_and_pattern_semantics ... ok
test runtime::tests::definitions_and_rules_tags::explicit_tagunset_removes_tagged_subvalue_definitions ... ok
test runtime::tests::definitions_and_rules_tags::literal_downvalue_is_prioritized_over_existing_typed_downvalue ... ok
test runtime::tests::definitions_and_rules_tags::redefining_same_downvalue_replaces_the_older_rule_in_place ... ok
test runtime::tests::definitions_and_rules_tags::redefining_same_subvalue_replaces_the_older_rule_in_place ... ok
test runtime::tests::definitions_and_rules_tags::redefining_same_upvalue_replaces_the_older_rule_in_place ... ok
test runtime::tests::definitions_and_rules_tags::string_targets_flow_through_clear_tag_and_unset_normalization ... ok
test runtime::tests::definitions_and_rules_tags::subvalues_list_and_apply_nested_head_definitions ... ok
test runtime::tests::definitions_and_rules_tags::tag_definition_messages_distinguish_missing_tags_from_invalid_positions ... ok
test runtime::tests::definitions_and_rules_tags::tagged_subvalues_attach_to_the_head_symbol_not_to_upvalues ... ok
test runtime::tests::definitions_and_rules_tags::tagset_immediate_captures_rhs_before_later_changes ... ok
test runtime::tests::definitions_and_rules_tags::tagset_preserves_tag_ownership_for_upvalues ... ok
test runtime::tests::definitions_and_rules_tags::typed_subvalues_are_prioritized_over_earlier_generic_subvalues ... ok
test runtime::tests::definitions_and_rules_tags::typed_upvalues_are_prioritized_over_earlier_generic_upvalues ... ok
test runtime::tests::definitions_and_rules_tags::unset_can_remove_symbol_downvalue_and_upvalue_definitions ... ok
test runtime::tests::definitions_and_rules_tags::upset_emits_nosym_when_no_valid_argument_tag_symbol_exists ... ok
test runtime::tests::definitions_and_rules_tags::upset_stores_immediate_upvalues_on_inferred_argument_heads ... ok
test runtime::tests::definitions_and_rules_tags::upsetdelayed_can_store_on_multiple_inferred_tag_symbols ... ok
test runtime::tests::definitions_and_rules_tags::upvalue_candidate_filter_preserves_matching_order_for_valid_tags ... ok
test runtime::tests::definitions_and_rules_tags::upvalue_dispatch_ignores_tags_that_appear_only_below_direct_arguments ... ok
test runtime::tests::definitions_and_rules_tags::upvalues_lists_and_applies_tagged_definitions ... ok
test runtime::tests::definitions_and_rules_tags::wide_subvalue_store_index_preserves_specificity_replacement_and_invalidation ... ok
test runtime::tests::definitions_and_rules_tags::wide_upvalue_store_index_preserves_specificity_replacement_and_invalidation ... ok
test runtime::tests::definitions_rhs_guards::delayed_downvalue_rhs_guard_accepts_or_declines_without_messages ... ok
test runtime::tests::definitions_rhs_guards::downvalues_guard_reconstruction_round_trips_behavior ... ok
test runtime::tests::definitions_rhs_guards::every_non_true_guard_result_declines_to_the_next_candidate ... ok
test runtime::tests::definitions_rhs_guards::explicit_true_guard_is_distinct_from_an_unguarded_downvalue ... ok
test runtime::tests::definitions_rhs_guards::guard_errors_propagate_like_pattern_position_condition_errors ... ok
test runtime::tests::definitions_rhs_guards::guarded_and_unguarded_downvalues_coexist_in_installation_order ... ok
test runtime::tests::definitions_rhs_guards::immediate_downvalue_rhs_evaluates_completely_before_guard_splitting ... ok
test runtime::tests::definitions_rhs_guards::nested_rhs_guards_flatten_in_order_for_both_spellings ... ok
test runtime::tests::definitions_rhs_guards::rhs_guard_side_effects_run_before_the_body_is_selected ... ok
test runtime::tests::definitions_rhs_guards::rhs_guards_and_bodies_share_the_definition_option_scope ... ok
test runtime::tests::definitions_rhs_guards::same_alpha_canonical_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards::unset_lhs_removes_every_rhs_guard_variant ... ok
test runtime::tests::definitions_rhs_guards::wide_exact_dispatch_preserves_guard_variant_order ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::attributes_on_the_guarded_symbol_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::eval_file_rollback_preserves_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::interleaved_guarded_downvalue_definitions_survive_later_bumps ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::listable_guarded_downvalue_recompiles_and_enforces_its_guard ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::matcher_ownership_guard_is_enforced_after_attribute_revision_invalidation ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::remaining_attribute_revision_sources_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::repeated_attribute_bumps_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::unrelated_set_and_clear_attributes_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::valueq_rollback_preserves_every_guarded_lane_and_post_trigger_value ... ok
test runtime::tests::definitions_rhs_guards_default_values::declining_default_guard_leaves_optional_matching_without_a_default ... ok
test runtime::tests::definitions_rhs_guards_default_values::defaultvalue_rhs_guards_accept_decline_and_coexist_in_order ... ok
test runtime::tests::definitions_rhs_guards_default_values::definition_inspection_rebuilds_all_remaining_definition_guard_forms ... ok
test runtime::tests::definitions_rhs_guards_default_values::every_non_true_defaultvalue_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_default_values::immediate_and_delayed_defaultvalue_heads_split_guards_at_the_right_time ... ok
test runtime::tests::definitions_rhs_guards_default_values::nested_defaultvalue_guards_run_left_to_right_and_second_decline_falls_back ... ok
test runtime::tests::definitions_rhs_guards_default_values::same_defaultvalue_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards_default_values::unset_clears_every_defaultvalue_guard_variant_and_table_roundtrip_restores_them ... ok
test runtime::tests::definitions_rhs_guards_own_values::a_later_guarded_ownvalue_replaces_the_single_existing_slot ... ok
test runtime::tests::definitions_rhs_guards_own_values::declining_ownvalue_guards_fall_through_inside_block_and_module ... ok
test runtime::tests::definitions_rhs_guards_own_values::immediate_and_delayed_ownvalue_rhs_guards_accept_or_decline ... ok
test runtime::tests::definitions_rhs_guards_own_values::non_true_ownvalue_guards_decline_and_valueq_reports_false ... ok
test runtime::tests::definitions_rhs_guards_own_values::ownvalue_guards_run_left_to_right_and_decline_before_the_body ... ok
test runtime::tests::definitions_rhs_guards_own_values::ownvalue_rhs_guard_resumes_without_replaying_the_guard ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::declining_guarded_rules_handoff_in_rule_list_order ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::every_non_true_rewrite_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::guarded_exact_rule_preserves_wide_plan_order_and_fallback_index ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::immediate_and_delayed_rewrite_rules_split_bound_guards ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::nested_rewrite_guards_run_left_to_right_and_second_declines ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::replace_list_guard_decline_keeps_later_binding_enumeration ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::replace_repeated_stops_when_the_only_guard_declines ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::rewrite_guard_side_effect_runs_once_per_candidate_tested ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::rewrite_rhs_guards_accept_or_decline_for_every_generic_consumer ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::rule_shaped_data_keeps_its_existing_condition_forms ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::suspending_rewrite_guard_resumes_same_rule_and_position_without_replay ... ok
test runtime::tests::definitions_rhs_guards_sub_values::every_non_true_subvalue_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_sub_values::immediate_and_delayed_subvalue_heads_split_guards_at_the_right_time ... ok
test runtime::tests::definitions_rhs_guards_sub_values::nested_subvalue_guards_run_left_to_right_and_second_decline_falls_back ... ok
test runtime::tests::definitions_rhs_guards_sub_values::same_alpha_canonical_subvalue_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards_sub_values::subvalue_rhs_guards_accept_decline_and_coexist_in_order ... ok
test runtime::tests::definitions_rhs_guards_sub_values::subvalue_rhs_guards_share_the_guarded_body_option_scope ... ok
test runtime::tests::definitions_rhs_guards_sub_values::unset_clears_every_subvalue_guard_variant_and_table_roundtrip_restores_them ... ok
test runtime::tests::definitions_rhs_guards_sub_values::wide_subvalue_identity_index_keeps_guard_variants_distinct ... ok
test runtime::tests::definitions_rhs_guards_up_values::every_non_true_upvalue_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_up_values::immediate_and_delayed_upvalue_heads_split_guards_at_the_right_time ... ok
test runtime::tests::definitions_rhs_guards_up_values::nested_upvalue_guards_run_left_to_right_and_second_decline_falls_back ... ok
test runtime::tests::definitions_rhs_guards_up_values::same_alpha_canonical_upvalue_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards_up_values::tagunset_clears_every_upvalue_guard_variant_and_table_roundtrip_restores_them ... ok
test runtime::tests::definitions_rhs_guards_up_values::upvalue_rhs_guards_accept_decline_and_coexist_in_order ... ok
test runtime::tests::definitions_rhs_guards_up_values::upvalue_rhs_guards_share_the_guarded_body_option_scope ... ok
test runtime::tests::definitions_rhs_guards_up_values::wide_upvalue_identity_index_keeps_guard_variants_distinct ... ok
test runtime::tests::definitions_tagged_preparation::all_tagged_install_entry_points_store_the_prepared_identity_by_owner ... ok
test runtime::tests::definitions_tagged_preparation::tagged_preparation_preserves_patterns_all_holds_and_callable_heads ... ok
test runtime::tests::definitions_tagged_preparation::tagged_snapshot_identity_tracks_attribute_rebind_clear_and_owner_invalidation ... ok
test runtime::tests::definitions_tagged_preparation::upset_infers_tags_only_after_preparation_and_keeps_nosym ... ok
test runtime::tests::definitions_tagged_preparation::upset_multiple_owner_protection_is_preserved_after_canonicalization ... ok
test runtime::tests::definitions_unset_preparation::clear_removes_all_identities_and_reinstall_has_no_stale_unset_target ... ok
test runtime::tests::definitions_unset_preparation::unset_after_attribute_invalidation_addresses_only_the_new_identity ... ok
test runtime::tests::definitions_unset_preparation::unset_after_own_value_rebinding_and_clear_preserves_the_stored_snapshot ... ok
test runtime::tests::definitions_unset_preparation::unset_and_postfix_unset_prepare_active_definition_identities ... ok
test runtime::tests::definitions_unset_preparation::unset_prepares_held_and_pattern_lhs_by_construction ... ok
test runtime::tests::definitions_unset_preparation::unset_snapshot_identity_removes_without_rebinding_but_not_after_rebinding ... ok
test runtime::tests::definitions_value_table_assignment::cross_symbol_downvalues_copy_fails_closed_on_head_mismatch ... ok
test runtime::tests::definitions_value_table_assignment::cross_symbol_downvalues_copy_works_after_target_substitution ... ok
test runtime::tests::definitions_value_table_assignment::downvalue_table_assignment_prepares_active_held_and_pattern_lhs ... ok
test runtime::tests::definitions_value_table_assignment::downvalues_identity_round_trip_restores_definition_behavior ... ok
test runtime::tests::definitions_value_table_assignment::later_mismatched_rule_cannot_partially_replace_a_value_table ... ok
test runtime::tests::definitions_value_table_assignment::malformed_value_table_rhs_leaves_the_existing_table_unchanged ... ok
test runtime::tests::definitions_value_table_assignment::ownvalues_identity_round_trip_restores_the_owned_value ... ok
test runtime::tests::definitions_value_table_assignment::prepared_value_table_identity_is_a_snapshot_across_rebind_clear_and_reinstall ... ok
test runtime::tests::definitions_value_table_assignment::protected_value_table_target_rejects_replacement ... ok
test runtime::tests::definitions_value_table_assignment::protection_is_rechecked_after_value_table_lhs_preparation ... ok
test runtime::tests::definitions_value_table_assignment::subvalue_table_assignment_prepares_active_held_and_pattern_lhs ... ok
test runtime::tests::definitions_value_table_assignment::successful_value_table_assignment_replaces_instead_of_merging ... ok
test runtime::tests::definitions_value_table_assignment::upvalue_table_assignment_prepares_active_held_and_pattern_lhs ... ok
test runtime::tests::definitions_value_table_assignment::upvalues_and_subvalues_identity_round_trips_restore_behavior ... ok
test runtime::tests::definitions_value_table_assignment::value_table_assignment_accepts_immediate_delayed_and_unwrapped_rules ... ok
test runtime::tests::file_names::file_names_depth_controls_nested_provider_enumeration ... ok
test runtime::tests::file_names::file_names_guard_precedes_the_provider_check ... ok
test runtime::tests::file_names::file_names_infinity_stops_at_the_fixed_entry_cap_without_recursing_the_stack ... ok
test runtime::tests::file_names::file_names_malformed_specs_remain_inert ... ok
test runtime::tests::file_names::file_names_matches_immediate_entries_and_sorts_and_deduplicates_paths ... ok
test runtime::tests::frame_root_trace::derived_frame_root_trace_inventory_matches_the_closed_capture_graph ... ok
test runtime::tests::frame_root_trace::manual_frame_root_trace_implementations_match_sorted_allowlist ... ok
test runtime::tests::frame_root_trace::root_visitation::association_entry_form_visits_key_and_value_roots_in_field_order ... ok
test runtime::tests::frame_root_trace::root_visitation::block_symbol_state_map_visits_the_bare_symbol_key ... ok
test runtime::tests::frame_root_trace::root_visitation::container_implementations_visit_keys_values_and_elements ... ok
test runtime::tests::frame_root_trace::root_visitation::enum_shapes_and_generic_arrays_trace_without_special_cases ... ok
test runtime::tests::frame_root_trace::root_visitation::expr_is_one_whole_root_and_bare_symbol_is_an_independent_root ... ok
test runtime::tests::frame_root_trace::root_visitation::file_io_error_is_the_zero_root_negative_calibration_target ... ok
test runtime::tests::frame_root_trace::root_visitation::matcher_binding_table_visits_the_exact_symbol_and_expression_roots ... ok
test runtime::tests::frame_root_trace::root_visitation::nested_derived_fields_are_all_traversed ... ok
test runtime::tests::frame_root_trace::root_visitation::recursive_matcher_arg_visits_every_nested_root_exactly_once ... ok
test runtime::tests::frame_root_trace::root_visitation::shared_expression_types_borrow_their_expression_roots ... ok
test runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold ... ok
test runtime::tests::holding_and_control_io::input_print_and_pause_cover_current_first_pass_io_subset ... ok
test runtime::tests::holding_and_control_io::pause_rejects_unrepresentable_integer_and_rational_durations ... ok
test runtime::tests::holding_and_control_part2::binary_function_syntax_applies_explicit_parameter ... ok
test runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds ... ok
test runtime::tests::holding_and_control_part2::cycle_detection_returns_iteration_limit_for_mutual_downvalues ... ok
test runtime::tests::holding_and_control_part2::cycle_detection_returns_iteration_limit_for_mutual_ownvalues_and_recovers ... ok
test runtime::tests::holding_and_control_part2::dispatch_plan_uses_builtin_metadata_and_head_traits ... ok
test runtime::tests::holding_and_control_part2::evaluation_pass_limit_bounds_fixed_point_revisits ... ok
test runtime::tests::holding_and_control_part2::explicit_parameter_functions_ignore_extra_arguments ... ok
test runtime::tests::holding_and_control_part2::explicit_parameter_functions_respect_shadowing_and_pattern_locals ... ok
test runtime::tests::holding_and_control_part2::fixed_point_evaluation_revisits_expressions_after_side_effects_change_head_meaning ... ok
test runtime::tests::holding_and_control_part2::function_applies_slots_positionally ... ok
test runtime::tests::holding_and_control_part2::head_evaluation_can_activate_special_forms_and_held_builtins ... ok
test runtime::tests::holding_and_control_part2::native_stack_limit_stops_unbounded_definition_recursion_and_session_stays_usable ... ok
test runtime::tests::holding_and_control_part2::pure_function_syntax_uses_slots_and_evaluated_arguments ... ok
test runtime::tests::holding_and_control_part2::recursion_limit_stops_expanding_recursive_downvalues ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_applies_at_every_application_level ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_attribute_round_trips ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_flattens_sequence_like_hold_all ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_holds_outer_arguments_and_evaluate_pierces ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_strips_and_restores_unevaluated ... ok
test runtime::tests::holding_and_control_part2::sum_and_product_malformed_iterators_emit_argform_messages ... ok
test runtime::tests::holding_and_control_part2::sum_and_product_valid_iterators_remain_message_free ... ok
test runtime::tests::holding_and_control_part2::tracing_records_definition_rule_and_evaluation_events ... ok
test runtime::tests::hygiene::kernel_crates_do_not_use_std_fs_outside_tests ... ok
test runtime::tests::hygiene::kernel_crates_do_not_write_stdout_outside_tests_and_bins ... ok
test runtime::tests::iterative_loop_drivers::for_drives_one_hundred_thousand_iterations_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::for_resumes_mid_loop_without_replaying_the_body ... ok
test runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_frozen_prefix_and_unique_tail_values ... ok
test runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_unsuspended_values ... ok
test runtime::tests::iterative_loop_drivers::module_drives_thirty_thousand_initialized_bindings_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::module_resumes_mid_initializer_without_replaying_bindings ... ok
test runtime::tests::iterative_loop_drivers::product_iteration_accumulator_completes_after_multiple_input_suspensions ... ok
test runtime::tests::iterative_loop_drivers::sum_iteration_accumulator_completes_after_multiple_input_suspensions ... ok
test runtime::tests::iterative_loop_drivers::table_iteration_accumulator_completes_after_multiple_input_suspensions ... ok
test runtime::tests::iterative_loop_drivers::unbounded_while_reaches_the_existing_eval_deadline ... ok
test runtime::tests::iterative_loop_drivers::until_drives_one_hundred_thousand_iterations_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::until_resumes_mid_loop_without_replaying_the_body ... ok
test runtime::tests::iterative_loop_drivers::while_drives_one_hundred_thousand_iterations_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::while_resumes_mid_loop_without_replaying_the_body ... ok
test runtime::tests::left_chain_lowering::flat_chain_evaluated_results_remain_unchanged ... ok
test runtime::tests::left_chain_lowering::held_sameq_explicitly_observes_flat_chain_shapes ... ok
test runtime::tests::left_chain_lowering::held_sameq_explicitly_observes_variadic_non_flat_chain_shapes ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_accepts_a_boundary_hand_built_alternating_ast ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_accepts_the_boundary_for_all_operators ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_keeps_direct_canonical_calls_opaque ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_preserves_cross_family_shape_and_resolver_order ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_preserves_message_name_double_resolution_order ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_rejects_a_20_000_link_hand_built_ast_gracefully ... ok
test runtime::tests::left_chain_lowering::left_chain_parser_rejects_20_000_links_gracefully_for_all_operators ... ok
test runtime::tests::left_chain_lowering::left_chain_two_mib_full_round_trip_accepts_and_rejects_without_abort ... ignored, release-only deployment-stack probe; run explicitly with --release
test runtime::tests::left_chain_lowering::left_chain_two_mib_nested_expr_drops_naturally_at_the_boundary ... ignored, release-only deployment-stack probe; run explicitly with --release
test runtime::tests::left_chain_lowering::variadic_non_flat_chain_matching_shapes_are_explicit ... ok
test runtime::tests::lowering_and_context::attributes_reports_builtin_and_user_defined_attribute_sets ... ok
test runtime::tests::lowering_and_context::compound_expression_sugar_flows_through_lowering_and_evaluation ... ok
test runtime::tests::lowering_and_context::evaluates_identity_call ... ok
test runtime::tests::lowering_and_context::long_statement_chain_lowers_to_flat_nary_compound_expression ... ok
test runtime::tests::lowering_and_context::lowering_preserves_tagset_shape_for_runtime_semantic_validation ... ok
test runtime::tests::lowering_and_context::lowering_rejects_invalid_integer_literals_before_runtime ... ok
test runtime::tests::lowering_and_context::lowers_big_integer_into_core_expr ... ok
test runtime::tests::lowering_and_context::lowers_message_name_syntax_into_core_expr ... ok
test runtime::tests::lowering_and_context::lowers_multiline_string_literals_like_ordinary_strings ... ok
test runtime::tests::lowering_and_context::lowers_simple_decimal_literals_to_exact_rationals ... ok
test runtime::tests::lowering_and_context::lowers_tagset_and_unset_syntax_into_core_expr ... ok
test runtime::tests::lowering_and_context::over_cap_statement_chain_line_does_not_swallow_later_lines ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_flat_arithmetic_family_transitions ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_messages_and_context_heads ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_rules_patterns_slots_and_scoping ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_tagged_definition_sugar ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_three_link_left_chains ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_argument_finisher_collapses_equal_changed_prefix ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_bounded_front_handles_exhaustion_locally ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_changed_arg_boundary_keeps_unchanged_child_arcs ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_definition_identity_hold_gate_and_four_arms ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_nested_flat_is_bottom_up_and_fully_flattened ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_rebuild_arms_preserve_root_arc_origins ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_uses_normalized_function_head_attributes ... ok
test runtime::tests::memory_growth_contract::core_builtin_memory_growth_inventory_is_sorted_and_exhaustive ... ok
test runtime::tests::memory_growth_contract::known_unprojected_preflight_builtin_allowlist_is_sorted_and_unique ... ok
test runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_after_input_resume - should panic ... ok
test runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_through_dispatch - should panic ... ok
test runtime::tests::memory_growth_contract::preflight_contract_accepts_recorded_projection ... ok
test runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_after_input_resume - should panic ... ok
test runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_through_dispatch - should panic ... ok
test runtime::tests::memory_growth_contract::preflight_contract_remembers_projection_across_input_resume ... ok
test runtime::tests::messages::builtin_message_registry_populates_listings_and_information ... ok
test runtime::tests::messages::delayed_message_templates_are_evaluated_when_emitted ... ok
test runtime::tests::messages::direct_set_and_setdelayed_calls_share_message_assignment_normalization ... ok
test runtime::tests::messages::inert_message_carriers_list_seeded_tags_and_reject_assignment ... ok
test runtime::tests::messages::inspect_message_catalog_exposes_templates_and_delayed_flags ... ok
test runtime::tests::messages::inspect_message_catalog_merges_builtin_templates_and_user_overrides ... ok
test runtime::tests::messages::inspect_reports_message_definition_presence ... ok
test runtime::tests::messages::invalid_builtin_argument_shapes_emit_runtime_messages ... ok
test runtime::tests::messages::invalid_message_name_form_emits_runtime_message ... ok
test runtime::tests::messages::message_definitions_builtin_lists_immediate_and_delayed_templates ... ok
test runtime::tests::messages::message_family_accepts_symbol_name_strings_and_messagename_targets ... ok
test runtime::tests::messages::message_list_reports_messages_emitted_so_far_in_current_input ... ok
test runtime::tests::messages::message_name_and_message_emit_runtime_messages ... ok
test runtime::tests::messages::message_tags_builtin_lists_global_symbol_and_exact_message_tags ... ok
test runtime::tests::messages::message_templates_can_be_defined_and_formatted ... ok
test runtime::tests::messages::message_templates_leave_unknown_placeholders_unchanged ... ok
test runtime::tests::messages::message_templates_leave_unsupported_format_specs_unchanged ... ok
test runtime::tests::messages::message_templates_support_message_name_and_all_args_placeholders ... ok
test runtime::tests::messages::message_templates_support_richer_format_specifiers ... ok
test runtime::tests::messages::messages_builtin_lists_defined_message_names ... ok
test runtime::tests::messages::push_message_allows_open_user_symbol_families ... ok
test runtime::tests::messages::push_message_allows_test_local_builtins_without_declared_messages ... ok
test runtime::tests::messages::push_message_allows_user_defined_builtin_message_names ... ok
test runtime::tests::messages::push_message_rejects_unregistered_builtin_message_tags - should panic ... ok
test runtime::tests::messages::quiet_keeps_suppressed_messages_out_of_message_list ... ok
test runtime::tests::messages::request_message_list_reports_messages_emitted_so_far_in_current_input ... ok
test runtime::tests::messages::upto_stays_inert_and_selector_diagnostic_stays_exact ... ok
test runtime::tests::messages::usage_message_assignment_holds_symbol_with_existing_own_value ... ok
test runtime::tests::messages_part2::check_and_quiet_accept_message_name_selector_forms ... ok
test runtime::tests::messages_part2::check_can_catch_non_loader_evaluation_errors_via_general_messages ... ok
test runtime::tests::messages_part2::check_can_match_by_message_symbol_only ... ok
test runtime::tests::messages_part2::check_can_match_specific_general_error_selectors ... ok
test runtime::tests::messages_part2::check_can_target_specific_message_selectors ... ok
test runtime::tests::messages_part2::check_leaves_value_unchanged_when_no_messages_are_emitted ... ok
test runtime::tests::messages_part2::check_returns_fallback_when_messages_are_emitted ... ok
test runtime::tests::messages_part2::cycle_termination_emits_pass_limit_message_at_top_level ... ok
test runtime::tests::messages_part2::explicit_parameter_function_arity_mismatch_emits_runtime_message ... ok
test runtime::tests::messages_part2::invalid_builtin_argument_counts_emit_runtime_messages ... ok
test runtime::tests::messages_part2::invalid_message_selector_specs_emit_runtime_messages ... ok
test runtime::tests::messages_part2::quiet_accepts_selector_lists ... ok
test runtime::tests::messages_part2::quiet_can_selectively_suppress_message_streams ... ok
test runtime::tests::messages_part2::quiet_prevents_check_from_observing_suppressed_messages ... ok
test runtime::tests::messages_part2::quiet_suppresses_cycle_termination_messages_without_masking_value ... ok
test runtime::tests::messages_part2::quiet_suppresses_runtime_messages_from_nested_evaluation ... ok
test runtime::tests::messages_part2::unbalanced_context_frames_emit_runtime_messages ... ok
test runtime::tests::messages_part3::beginpackage_accepts_resolved_dependency_lists ... ok
test runtime::tests::messages_part3::beginpackage_can_load_dependencies_via_needs ... ok
test runtime::tests::messages_part3::check_can_catch_iteration_limit_messages_from_replace_repeated ... ok
test runtime::tests::messages_part3::explicit_non_system_heads_do_not_trigger_builtin_dispatch ... ok
test runtime::tests::messages_part3::hold_family_respects_builtin_hold_attributes ... ok
test runtime::tests::messages_part3::path_builtin_reflects_package_search_roots ... ok
test runtime::tests::mutation_operator_forms::append_to_and_add_to_direct_calls_match_their_operator_forms ... ok
test runtime::tests::mutation_operator_forms::append_to_association_key_part_stays_on_generic_path ... ok
test runtime::tests::mutation_operator_forms::append_to_capture_defers_its_value_until_application ... ok
test runtime::tests::mutation_operator_forms::append_to_complex_part_selectors_keep_generic_behavior ... ok
test runtime::tests::mutation_operator_forms::append_to_deep_exact_part_preserves_middle_alias ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_copies_list_shared_at_two_positions ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_keeps_nothing_and_upvalue_behavior ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_non_list_leaf_still_runs_definitions ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_preserves_inner_alias ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_preserves_outer_alias ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_protected_owner_still_refuses ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_rolls_back_multilevel_spine ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_with_monitor_preserves_spine_observations ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_with_tracing_preserves_existing_trace ... ok
test runtime::tests::mutation_operator_forms::append_to_item_that_requires_target_evaluation_stays_on_evaluated_path ... ok
test runtime::tests::mutation_operator_forms::append_to_list_downvalue_still_observes_the_old_value_and_fires ... ok
test runtime::tests::mutation_operator_forms::append_to_list_upvalue_still_observes_the_old_value_and_fires ... ok
test runtime::tests::mutation_operator_forms::append_to_modified_list_attributes_stays_on_evaluated_path ... ok
test runtime::tests::mutation_operator_forms::append_to_non_list_head_still_runs_matching_user_code ... ok
test runtime::tests::mutation_operator_forms::append_to_nothing_keeps_list_normalization_semantics ... ok
test runtime::tests::mutation_operator_forms::append_to_plain_list_preserves_alias_and_returned_value ... ok
test runtime::tests::mutation_operator_forms::append_to_protected_list_target_preserves_value_and_message ... ok
test runtime::tests::mutation_operator_forms::append_to_resumes_item_input_without_losing_or_replaying_the_update ... ok
test runtime::tests::mutation_operator_forms::append_to_rolls_back_if_fast_update_fails_after_definition_take ... ok
test runtime::tests::mutation_operator_forms::append_to_with_monitor_preserves_old_and_new_display_observations ... ok
test runtime::tests::mutation_operator_forms::append_to_with_tracing_preserves_the_existing_trace ... ok
test runtime::tests::mutation_operator_forms::kernel_mutation_heads_advertise_subvalues_hold_all ... ok
test runtime::tests::mutation_operator_forms::kernel_mutation_operator_forms_apply_to_raw_held_targets ... ok
test runtime::tests::mutation_operator_forms::mutation_captures_are_quiet_but_invalid_applications_report_argx ... ok
test runtime::tests::mutation_operator_forms::mutation_operator_form_preserves_direct_protection_failure ... ok
test runtime::tests::namespace_names::names_accepts_current_options_subset ... ok
test runtime::tests::namespace_names::names_rejects_unsupported_resolve_context_alias_values ... ok
test runtime::tests::namespace_names::names_rejects_unsupported_spelling_correction_values ... ok
test runtime::tests::namespace_names::names_supports_current_first_pass_string_patterns ... ok
test runtime::tests::namespace_names::names_without_explicit_context_only_sees_visible_contexts ... ok
test runtime::tests::namespace_names::symbol_name_and_name_q_core_anchors ... ok
test runtime::tests::native_stack_guard::logical_depth_never_refuses_when_the_byte_budget_is_available ... ok
test runtime::tests::native_stack_guard::native_stack_guard_polls_at_the_stride_and_fails_sticky ... ok
test runtime::tests::nothing::nothing_removal_happens_after_sequence_splicing_for_lists_only ... ok
test runtime::tests::nothing::nothing_resolves_as_system_symbol ... ok
test runtime::tests::options_and_protection::argumentsoptions_and_checkarguments_cover_current_first_pass_subset ... ok
test runtime::tests::options_and_protection::block_localizes_options_alongside_other_symbol_state ... ok
test runtime::tests::options_and_protection::codec_and_schema_options_are_discoverable ... ok
test runtime::tests::options_and_protection::delayed_option_rules_are_preserved_and_evaluated_on_lookup ... ok
test runtime::tests::options_and_protection::options_assignment_and_queries_cover_current_first_pass_subset ... ok
test runtime::tests::options_and_protection::options_assignment_unset_and_clearall_interact_with_symbol_state ... ok
test runtime::tests::options_and_protection::optionspattern_and_one_arg_optionvalue_cover_current_first_pass_subset ... ok
test runtime::tests::options_and_protection::optionspattern_can_use_an_explicit_target_symbol ... ok
test runtime::tests::options_and_protection::optionvalue_accepts_lists_of_option_names ... ok
test runtime::tests::options_and_protection::optionvalue_emits_current_optnf_and_nodef_messages ... ok
test runtime::tests::options_and_protection::protect_and_unprotect_expand_name_patterns_without_literal_wildcards ... ok
test runtime::tests::options_and_protection::protect_blocks_direct_options_assignment_and_unset_until_unprotected ... ok
test runtime::tests::options_and_protection::protected_attribute_removal_and_unprotect_remain_escape_hatches ... ok
test runtime::tests::options_and_protection::protected_targets_reject_attribute_updates_and_preserve_attributes ... ok
test runtime::tests::options_and_protection::setoptions_accepts_target_lists_and_returns_per_target_results ... ok
test runtime::tests::options_and_protection::setoptions_and_optionvalue_match_known_system_options_by_name ... ok
test runtime::tests::options_and_protection::setoptions_blocks_protected_system_head_and_unprotect_allows_update ... ok
test runtime::tests::options_and_protection::setoptions_updates_known_options_but_rejects_unknown_ones ... ok
test runtime::tests::options_and_protection::test_create_options_are_discoverable_and_match_consumed_defaults ... ok
test runtime::tests::package_tools::declare_package_declares_names_and_updates_context_path ... ok
test runtime::tests::package_tools::declared_package_stub_autoloads_when_symbol_is_used ... ok
test runtime::tests::package_tools::definition_snapshot_rebuilds_all_rhs_guards_for_roundtrip ... ok
test runtime::tests::package_tools::dump_save_context_strings_skip_module_gensyms_but_keep_dollar_globals ... ok
test runtime::tests::package_tools::dump_save_overwrites_file_and_returns_saved_symbols ... ok
test runtime::tests::package_tools::dump_save_rejects_unsupported_compression_levels_and_preserves_default_bytes ... ok
test runtime::tests::package_tools::dump_save_returns_qualified_symbol_names_without_reevaluating_values ... ok
test runtime::tests::package_tools::once_accepts_option_rules_in_second_position_and_respects_zero_persistence_subset ... ok
test runtime::tests::package_tools::once_caches_per_session_and_per_location_key ... ok
test runtime::tests::package_tools::once_rejects_unimplemented_expiration_policies_without_evaluating_body ... ok
test runtime::tests::package_tools::save_and_dumpsave_accept_names_pattern_objects ... ok
test runtime::tests::package_tools::save_and_dumpsave_honor_current_context_filter_and_attribute_options_subset ... ok
test runtime::tests::package_tools::save_rebuilds_downvalue_rhs_guards_for_roundtrip ... ok
test runtime::tests::package_tools::save_rejects_unknown_options_and_once_reports_unknown_option_names ... ok
test runtime::tests::package_tools::save_writes_textual_state_that_get_can_roundtrip ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_a_root_alias ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_a_shared_child_at_a_second_position ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_a_three_level_middle_alias ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_an_intermediate_list_alias ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_the_selected_child_alias ... ok
test runtime::tests::part_assignment::set_positional_part_direct_path_keeps_spine_identity_and_leaves_proofs_evicted ... ok
test runtime::tests::part_assignment::set_positional_part_direct_path_polls_interrupt_before_cache_eviction_or_mutation ... ok
test runtime::tests::part_assignment::set_positional_part_direct_path_rolls_back_slot_and_definition_after_replacement ... ok
test runtime::tests::part_assignment::set_positional_part_invalid_targets_report_without_protected_part_fallback ... ok
test runtime::tests::part_assignment::set_positional_part_observable_constrained_and_dynamic_scopes_use_generic_path ... ok
test runtime::tests::part_assignment::set_positional_part_rechecks_protection_after_rhs_and_selector_callbacks ... ok
test runtime::tests::part_assignment::set_positional_part_rederives_after_rhs_and_orders_selectors_left_to_right ... ok
test runtime::tests::part_assignment::set_positional_part_refuses_a_target_cleared_by_a_selector ... ok
test runtime::tests::part_assignment::set_positional_part_route_preserves_other_lvalues_and_definition_lhs_forms ... ok
test runtime::tests::part_assignment::set_positional_part_selector_resume_is_single_pass_and_rederives_the_owner ... ok
test runtime::tests::part_assignment::set_positional_part_supports_flat_nested_and_value_semantics ... ok
test runtime::tests::pattern_matching_orderless_optional::flat_optional_omission_collects_every_match_for_builtin_heads ... ok
test runtime::tests::pattern_matching_orderless_optional::flat_orderless_optional_matches_collects_and_omits ... ok
test runtime::tests::pattern_matching_orderless_optional::flat_orderless_optional_with_sequence_neighbors ... ok
test runtime::tests::pattern_matching_orderless_optional::installed_system_down_values_surface_as_a_whole_after_user_mutation ... ok
test runtime::tests::pattern_matching_orderless_optional::one_identity_promotes_singletons_when_remaining_arguments_are_omissible ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_dot_uses_head_default ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_nonmatch_edges_stay_false ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_omission_uses_explicit_defaults ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_present_matches_and_binds_in_documented_order ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_with_other_args_matches_present_and_omitted ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_with_sequence_neighbors_uses_documented_backtracking_order ... ok
test runtime::tests::pattern_matching_orderless_optional::snapshot_after_unset_resurrects_an_installed_system_down_value ... ok
test runtime::tests::reap::nested_reap_scopes_keep_their_own_buckets ... ok
test runtime::tests::reap::reap_buckets_stay_in_first_appearance_order ... ok
test runtime::tests::reap::reap_index_preserves_wide_order_and_appends_to_existing_buckets ... ok
test runtime::tests::reap::reap_patterns_select_which_sows_are_captured ... ok
test runtime::tests::registered_builtin_resolution::registered_builtin_absent_from_static_catalog_resolves_as_system_symbol ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_masks_only_held_argument_positions ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_operator_form_applies_to_multiple_inputs ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_rule_shape_messages_match_replace_all ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_uses_original_and_compound_head_attributes ... ok
test runtime::tests::rule_rhs_materialization::held_delayed_rhs_firing_is_consumer_independent ... ok
test runtime::tests::rule_rhs_materialization::literal_sequence_splicing_is_deferred_until_replacement_result_evaluation ... ok
test runtime::tests::rule_rhs_materialization::literal_sequence_wrapping_a_sequence_binding_keeps_its_literal_head ... ok
test runtime::tests::rule_rhs_materialization::other_rewrite_traversals_share_replacement_sequence_splicing ... ok
test runtime::tests::rule_rhs_materialization::replaceat_immediate_rhs_still_evaluates_at_rule_construction ... ok
test runtime::tests::rule_rhs_materialization::replaceat_inserts_delayed_rhs_raw_before_result_evaluation ... ok
test runtime::tests::rule_rhs_materialization::replacement_produced_sequence_splices_into_held_parent_rebuild ... ok
test runtime::tests::rule_rhs_materialization::replacement_sequence_splice_respects_parent_attributes ... ok
test runtime::tests::rule_rhs_materialization::replacepart_immediate_rhs_still_evaluates_at_rule_construction ... ok
test runtime::tests::rule_rhs_materialization::replacepart_inserts_delayed_rhs_raw_before_result_evaluation ... ok
test runtime::tests::rule_rhs_materialization::sequence_pattern_bindings_keep_structural_splicing ... ok
test runtime::tests::rule_rhs_materialization::top_level_sequence_replacement_has_no_parent_to_splice_into ... ok
test runtime::tests::rule_rhs_materialization::unchanged_literal_sequence_does_not_splice_during_rebuild ... ok
test runtime::tests::scoping_gensym_gc::all_live_modules_run_one_sweep_before_the_next_watermark ... ok
test runtime::tests::scoping_gensym_gc::clear_all_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::scoping_gensym_gc::gensym_gc_threshold_parses_env_in_fresh_processes ... ok
test runtime::tests::scoping_gensym_gc::gensym_gc_watermark_arithmetic_saturates ... ok
test runtime::tests::scoping_gensym_gc::immediate_module_reclaim_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::scoping_gensym_gc::loader_snapshot_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::scoping_gensym_gc::partial_sweep_rearms_from_the_post_sweep_candidate_count ... ok
test runtime::tests::scoping_gensym_gc::reachability::attribute_state_on_dead_candidate_is_cleared_during_reclaim ... ok
test runtime::tests::scoping_gensym_gc::reachability::delayed_own_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::down_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::downvalue_rhs_guard_keeps_its_escaped_module_binding_live ... ok
test runtime::tests::scoping_gensym_gc::reachability::empty_candidate_set_sweep_is_a_noop ... ok
test runtime::tests::scoping_gensym_gc::reachability::global_root_keeps_escaped_candidate_until_root_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::live_candidate_definition_marks_transitive_candidate_dependencies ... ok
test runtime::tests::scoping_gensym_gc::reachability::message_root_escape_keeps_candidate_until_message_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::module_error_path_definition_escape_stays_deferred ... ok
test runtime::tests::scoping_gensym_gc::reachability::module_result_root_retains_only_its_transitive_local_chain ... ok
test runtime::tests::scoping_gensym_gc::reachability::module_throw_payload_root_retains_its_gensym_definition ... ok
test runtime::tests::scoping_gensym_gc::reachability::module_unreferenced_error_paths_reclaim_gensyms_immediately ... ok
test runtime::tests::scoping_gensym_gc::reachability::nested_module_activations_keep_their_candidates_until_each_activation_is_removed ... ok
test runtime::tests::scoping_gensym_gc::reachability::nested_module_error_reclamation_mirrors_success_conservatism ... ok
test runtime::tests::scoping_gensym_gc::reachability::option_root_escape_keeps_candidate_until_options_are_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::outcome_extra_roots_are_empty_for_non_value_errors_and_candidate_is_reclaimed ... ok
test runtime::tests::scoping_gensym_gc::reachability::outcome_extra_roots_keep_success_return_throw_and_cleanup_candidates_live ... ok
test runtime::tests::scoping_gensym_gc::reachability::own_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::recursive_module_sweep_reclaims_dead_retained_gensyms ... ok
test runtime::tests::scoping_gensym_gc::reachability::returned_expr_extra_root_keeps_candidate_definition_live ... ok
test runtime::tests::scoping_gensym_gc::reachability::sub_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::subvalue_rhs_guard_keeps_its_escaped_module_binding_live ... ok
test runtime::tests::scoping_gensym_gc::reachability::suspended_computation_keeps_candidate_live_until_resume_finishes ... ok
test runtime::tests::scoping_gensym_gc::reachability::syntax_information_root_escape_keeps_candidate_until_syntax_info_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::transitive_definition_kind_chain_keeps_all_reachable_candidates_live ... ok
test runtime::tests::scoping_gensym_gc::reachability::unique_generated_symbols_are_not_module_gc_candidates ... ok
test runtime::tests::scoping_gensym_gc::reachability::up_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::upvalue_rhs_guard_keeps_its_escaped_module_binding_live ... ok
test runtime::tests::scoping_gensym_gc::remove_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::block_frame_bare_symbol_is_reclaimed_after_resume_and_next_sweep ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::executing_frame_and_queued_suffix_remain_rooted_during_resume_sweep ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::large_frame_state_trips_memory_limit_and_session_recovers ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::matcher_guard_binding_survives_a_sweep_while_suspended ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::path_a_second_request_keeps_frame_only_candidate_live ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::path_b_resume_sweep_keeps_remaining_frame_candidate_live ... ok
test runtime::tests::scoping_gensym_gc::unreachable_candidates_are_reclaimed_at_the_next_watermark_boundary ... ok
test runtime::tests::scoping_unique::unique_generates_fresh_symbols_for_current_prefix_subset ... ok
test runtime::tests::scoping_unique::unique_registers_generated_symbols_as_known_names ... ok
test runtime::tests::scoping_unique::unique_reports_current_invalid_boundaries ... ok
test runtime::tests::scoping_unique::unique_skips_existing_generated_names ... ok
test runtime::tests::sealed_packages::context_validation_returns_the_call_inert ... ok
test runtime::tests::sealed_packages::declaration_validation ... ok
test runtime::tests::sealed_packages::decoy_assignment_is_distinct ... ok
test runtime::tests::sealed_packages::internal_snapshot_round_trips_registry ... ok
test runtime::tests::sealed_packages::package_install_resumes_at_the_next_form ... ok
test runtime::tests::sealed_packages::package_scope_is_fqn_reachable ... ok
test runtime::tests::sealed_packages::private_state_inside_package ... ok
test runtime::tests::sealed_packages::public_reaches_sealed_private ... ok
test runtime::tests::sealed_packages::re_registration_replaces_in_place ... ok
test runtime::tests::sealed_packages::registration_failure_cleans_sealed_state ... ok
test runtime::tests::sealed_packages::remove_sealed_writes_no_generation ... ok
test runtime::tests::sealed_packages::spelled_private_name_is_ordinary ... ok
test runtime::tests::sealed_snapshot::information_sealed_unit_row ... ok
test runtime::tests::sealed_snapshot::marker_refused_in_ordinary_evaluation ... ok
test runtime::tests::sealed_snapshot::save_refuses_sealed_references ... ok
test runtime::tests::sealed_snapshot::sealed_free_snapshot_bytes_unchanged ... ok
test runtime::tests::sealed_snapshot::snapshot_persists_public_mutation ... ok
test runtime::tests::sealed_snapshot::snapshot_refuses_orphans ... ok
test runtime::tests::sealed_snapshot::snapshot_roundtrip_relinks_leaked_values ... ok
test runtime::tests::sealed_snapshot::snapshot_roundtrip_restores_seal ... ok
test runtime::tests::sealed_visibility::completion_hides_sealed ... ok
test runtime::tests::sealed_visibility::completion_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::contexts_hides_private_context ... ok
test runtime::tests::sealed_visibility::contexts_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::decoy_visibility_is_callers_own ... ok
test runtime::tests::sealed_visibility::definition_hides_sealed_body ... ok
test runtime::tests::sealed_visibility::definition_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::information_hides_sealed ... ok
test runtime::tests::sealed_visibility::information_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::message_catalog_hides_sealed ... ok
test runtime::tests::sealed_visibility::message_catalog_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::nameq_stays_false ... ok
test runtime::tests::sealed_visibility::names_hides_sealed ... ok
test runtime::tests::sealed_visibility::names_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::nested_registration_keeps_registry_consistent ... ok
test runtime::tests::sealed_visibility::save_excludes_sealed ... ok
test runtime::tests::sealed_visibility::wildcard_ops_skip_sealed ... ok
test runtime::tests::session_config::apply_config_updates_host_settings_without_clearing_existing_definitions ... ok
test runtime::tests::session_config::default_native_stack_budget_is_derived_from_minimum_supported_stack ... ok
test runtime::tests::session_config::default_session_configuration_matches_default_session_state ... ok
test runtime::tests::session_config::random_u64_sequences_are_pinned_for_byte_and_integer_seeds ... ok
test runtime::tests::session_config::session_setters_update_limits_capabilities_asserts_and_context_lookup ... ok
test runtime::tests::session_config::session_with_config_normalizes_contexts_paths_and_limits ... ok
test runtime::tests::session_config::trace_message_and_print_accessors_expose_and_take_buffers ... ok
test runtime::tests::session_memory::arithmetic::field_and_session_sums_preserve_overflow_information ... ok
test runtime::tests::session_memory::arithmetic::overflow_saturates_and_invalid_releases_stay_closed ... ok
test runtime::tests::session_memory::arithmetic::synthetic_charge_release_sequences_never_understate_representable_remainders ... ok
test runtime::tests::session_memory::byte_array_proxy_tracks_encoded_string_capacity_and_preserves_invalid_fallback ... ok
test runtime::tests::session_memory::debug_owners::maintained_message_print_trace_capture_and_rollback ... ok
test runtime::tests::session_memory::debug_owners::maintained_reap_monitor_append_replace_resume_and_abandon ... ok
test runtime::tests::session_memory::maintained::maintained_attributes_and_caches_mutate_clear_and_restore ... ok
test runtime::tests::session_memory::maintained::maintained_builtins_and_limits_charge_replace_and_rearm ... ok
test runtime::tests::session_memory::maintained::maintained_contexts_paths_and_loader_rollback ... ok
test runtime::tests::session_memory::maintained::maintained_definitions_store_remove_and_lazy_dispatch ... ok
test runtime::tests::session_memory::maintained::maintained_evaluation_caches_charge_evict_and_keep_capacity ... ok
test runtime::tests::session_memory::maintained::maintained_options_and_syntax_replace_remove_and_rollback ... ok
test runtime::tests::session_memory::maintained::maintained_providers_and_print_sink_take_reinstall ... ok
test runtime::tests::session_memory::maintained::maintained_scopes_registry_sealed_units_and_abandonment ... ok
test runtime::tests::session_memory::maintained::maintained_stack_checkpoint_workloads ... ok
test runtime::tests::session_memory::maintained::maintained_stack_counts_shared_occurrences_and_retained_capacity ... ok
test runtime::tests::session_memory::maintained::maintained_stack_local_arm_survives_suspend_resume_abandon_and_errors ... ok
test runtime::tests::session_memory::maintained::maintained_stack_rearms_from_scratch_and_observation_keeps_walking ... ok
test runtime::tests::session_memory::maintained::maintained_stack_survives_panic_and_rollback ... ok
test runtime::tests::session_memory::maintained::maintained_stubs_generations_and_once_restore_and_resume ... ok
test runtime::tests::session_memory::oracle::byte_array_charges_encoded_capacity_for_valid_invalid_and_nonstring_payloads ... ok
test runtime::tests::session_memory::oracle::container_capacity_and_legacy_length_terms_are_distinct ... ok
test runtime::tests::session_memory::oracle::empty_and_installed_sessions_match_without_sampling_the_peak ... ok
test runtime::tests::session_memory::oracle::exception_registry_and_exception_expression_payloads_are_exact ... ok
test runtime::tests::session_memory::oracle::expressions_charge_each_shared_occurrence_and_spare_slot ... ok
test runtime::tests::session_memory::oracle::numeric_payloads_follow_signed_bytes_not_limb_capacity ... ok
test runtime::tests::session_memory::request_owners::maintained_accounting_route_adapter_replacement ... ok
test runtime::tests::session_memory::request_owners::maintained_requests_queues_nested_resume_and_cleanup ... ok
test runtime::tests::session_memory::session_memory_field_decision_inventory_is_sorted_and_exhaustive ... ok
test runtime::tests::session_memory::session_memory_walk_counts_declared_exception_storage ... ok
test runtime::tests::session_memory::stability_blooms_are_fixed_size_inline_values ... ok
test runtime::tests::session_memory::structures::association_indexes_keep_stale_storage_until_the_owner_prunes_it ... ok
test runtime::tests::session_memory::structures::call_cache_roots_are_additional_occurrences_of_retained_values ... ok
test runtime::tests::session_memory::structures::dispatch_metadata_and_lazy_index_have_exact_separate_charges ... ok
test runtime::tests::session_memory::structures::live_frames_messages_and_traces_preserve_conservative_inline_terms ... ok
test runtime::tests::session_memory::structures::session_memory_layout_baseline ... ok
test runtime::tests::session_memory::structures::suspended_frames_charge_storage_repeated_roots_and_exception_payloads ... ok
test runtime::tests::session_memory::structures::tagged_dispatch_indexes_are_charged_for_both_value_tables ... ok
test runtime::tests::session_requests::api::accepted_cancel_abandons_input_before_conversion_or_continuation ... ok
test runtime::tests::session_requests::api::accepted_cancel_before_request_entry_is_observed ... ok
test runtime::tests::session_requests::api::accepted_script_and_file_stop_before_later_forms_after_cancellation ... ok
test runtime::tests::session_requests::api::cancellation_observer_held_across_session_rebuild_is_inert ... ok
test runtime::tests::session_requests::api::complete_request_returns_transport_friendly_matches ... ok
test runtime::tests::session_requests::api::completed_request_handle_is_inert_for_later_requests ... ok
test runtime::tests::session_requests::api::dropped_accepted_ticket_and_generation_rotation_retire_handles ... ok
test runtime::tests::session_requests::api::eval_expr_request_aggregates_messages_and_classifies_budget_termination ... ok
test runtime::tests::session_requests::api::eval_request_memory_ceiling_returns_classified_termination_and_session_recovers ... ok
test runtime::tests::session_requests::api::eval_request_nested_memory_constraint_catches_before_higher_request_ceiling ... ok
test runtime::tests::session_requests::api::eval_request_print_flood_trips_memory_ceiling_with_classified_termination ... ok
test runtime::tests::session_requests::api::eval_request_returns_transport_friendly_cycle_termination_data ... ok
test runtime::tests::session_requests::api::eval_request_returns_transport_friendly_success_data ... ok
test runtime::tests::session_requests::api::eval_request_surfaces_structured_input_requests ... ok
test runtime::tests::session_requests::api::eval_request_take_pending_input_request_moves_and_clears_the_request ... ok
test runtime::tests::session_requests::api::file_request_maps_completed_syntax_io_and_missing_provider_errors ... ok
test runtime::tests::session_requests::api::file_request_maps_resource_limit_exit_and_cancel_errors ... ok
test runtime::tests::session_requests::api::inspect_request_aggregates_symbol_definitions_and_messages ... ok
test runtime::tests::session_requests::api::is_complete_request_distinguishes_complete_incomplete_and_invalid_input ... ok
test runtime::tests::session_requests::api::ordinary_request_and_lazy_observer_leave_interrupt_poll_unarmed ... ok
test runtime::tests::session_requests::api::parse_request_returns_structured_syntax_diagnostics ... ok
test runtime::tests::session_requests::api::script_requests_report_file_coordinates_for_every_form_shape ... ok
test runtime::tests::session_requests::api::stale_cancel_cannot_overwrite_current_request_cancel ... ok
test runtime::tests::session_requests::batch_boundary::flat_public_file_cancellation_control ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_error - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_file - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_script - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_snapshot - should panic ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_value - should panic ... ok
test runtime::tests::session_requests::batch_boundary::public_file_cancel_nested ... ok
test runtime::tests::session_requests::batch_boundary::public_file_cancel_two_levels ... ok
test runtime::tests::session_requests::batch_boundary::public_file_cleanup_observes_inner_then_outer_state_once ... ok
test runtime::tests::session_requests::batch_boundary::public_file_first_input_answered_then_cancel_deeper_second ... ok
test runtime::tests::session_requests::batch_boundary::public_file_rollback_is_deferred_while_blocked ... ok
test runtime::tests::session_requests::batch_boundary::public_file_snapshot_roots_survive_suspension ... ok
test runtime::tests::session_requests::batch_boundary::public_file_success_and_ordinary_error_control ... ok
test runtime::tests::session_requests::batch_boundary::restore::restore_abandons_nested_cleanup_and_file_rollbacks_once ... ok
test runtime::tests::session_requests::batch_boundary::restore::restore_admission_rejection_preserves_unrelated_request ... ok
test runtime::tests::session_requests::batch_boundary::restore::terminal_restore_retires_identity_and_all_owner_state ... ok
test runtime::tests::session_requests::batch_boundary::script_snapshot_and_guest_cancellation_controls ... ok
test runtime::tests::session_requests::debug_tooling_resume::resume_eval_request_with_input_can_finish_reap_handler_rendering ... ok
test runtime::tests::session_requests::debug_tooling_resume::resume_eval_request_with_input_can_finish_repeated_timing_samples ... ok
test runtime::tests::session_requests::debug_tooling_resume::resume_eval_request_with_input_can_finish_trace_capture ... ok
test runtime::tests::session_requests::input_resume::nested_input_suspension_precedes_the_old_suffix_without_reordering_effects ... ok
test runtime::tests::session_requests::input_resume::queued_input_responses_feed_inputstring_without_prompting_again ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_can_finish_delayed_message_templates ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_can_suspend_multiple_times_without_restarting ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_continues_replace_all_without_restarting_target ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_continues_symbol_evaluation_after_fixed_point_suspend ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_finishes_dialog_subsets ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_finishes_with_bindings_without_reprompting ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_formats_delayed_message_templates_after_resume ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_keeps_monitor_display_and_finishes_the_body ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_preserves_already_evaluated_prefix_arguments ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_preserves_block_scope_until_completion ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_reports_input_parse_errors_as_syntax_diagnostics ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_skips_uninitialized_module_bindings ... ok
test runtime::tests::session_requests::matcher_resume::cases_collection_survives_multiple_matcher_suspensions ... ok
test runtime::tests::session_requests::matcher_resume::defaultvalue_rhs_guard_resumes_during_matching_without_replaying_the_candidate ... ok
test runtime::tests::session_requests::matcher_resume::downvalue_rhs_guard_resumes_at_the_same_candidate_cursor ... ok
test runtime::tests::session_requests::matcher_resume::guarded_replacement_numeric_value_survives_suspension ... ok
test runtime::tests::session_requests::matcher_resume::matcher_binding_numeric_value_survives_suspension ... ok
test runtime::tests::session_requests::matcher_resume::nested_rule_application_numeric_value_survives_suspension ... ok
test runtime::tests::session_requests::matcher_resume::ordered_single_arg_cursor_resumes_without_replaying_the_matched_prefix ... ok
test runtime::tests::session_requests::matcher_resume::position_collection_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::matcher_resume::position_collection_survives_multiple_matcher_suspensions ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_fifth_arg ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_fourth_arg ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_third_arg ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_level_spec ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_match_limit ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_symbol_spec ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_cases ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_downvalue_matching ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_matchq ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_replace_all ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_count_with_suspending_symbol_spec ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_freeq_with_suspending_heads_option ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_multi_sequence_backtracking_conditions ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_nested_matcher_inside_sequence_conditioned_matchq ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_orderless_patternsequence_group_matching ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_position_with_suspending_match_limit ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_position_with_suspending_symbol_spec ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_sequence_conditioned_matchq ... ok
test runtime::tests::session_requests::matcher_resume::sequence_binding_vec_continuation_roundtrip_preserves_multiple_bindings ... ok
test runtime::tests::session_requests::matcher_resume::subvalue_rhs_guard_resumes_at_the_same_candidate_cursor ... ok
test runtime::tests::session_requests::matcher_resume::traversal_count_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::matcher_resume::upvalue_rhs_guard_resumes_at_the_same_candidate_cursor ... ok
test runtime::tests::session_requests::transforms_resume::compiled_rule_vec_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::transforms_resume::deletecases_iterative_tail_match_resume_matches_uninterrupted_association_walk ... ok
test runtime::tests::session_requests::transforms_resume::deletecases_iterative_tail_match_resume_matches_uninterrupted_call_walk ... ok
test runtime::tests::session_requests::transforms_resume::normalized_rule_expr_vec_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::transforms_resume::replace_and_replaceat_indexed_rule_plans_survive_mid_traversal_suspension ... ok
test runtime::tests::session_requests::transforms_resume::replace_iterative_tail_resumes_at_the_same_deep_rule_attempt ... ok
test runtime::tests::session_requests::transforms_resume::replace_levelspec_option_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::transforms_resume::replacepart_resumes_mid_walk_with_the_same_rule_and_partial_result ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_suspending_match_limit ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_suspending_pattern_symbol ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_suspending_target ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_normalize_rules_after_suspending_rule_source ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replace_after_suspending_levelspec ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replace_after_suspending_target ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replaceall_after_suspending_rule_source ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replaceat_after_suspending_delayed_rhs ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replaceat_after_suspending_position_spec ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacelist_after_multiple_backtracking_matches ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacelist_after_suspending_delayed_rhs ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacepart_after_suspending_pattern_position_spec ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacepart_with_suspending_delayed_rhs ... ok
test runtime::tests::session_requests::transforms_resume_patterns::eval_request_reports_prints_on_failure_and_across_input_resume ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_except_matchq ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_patterntest_matchq ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_repeated_patterntest_matchq ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_sequence_patterntest_matchq ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_setattributes_with_suspending_attribute_spec ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_thread_with_suspending_arg_spec ... ok
test runtime::tests::structure_data::replace::replaceat_supports_exact_positions_operator_form_and_rule_lists ... ok
test runtime::tests::structure_data::replace::replacepart_deep_path_shallow_companion_pins_three_level_rebuild ... ok
test runtime::tests::structure_data::replace::replacepart_handles_duplicate_and_overlapping_targets_more_stably ... ok
test runtime::tests::structure_data::replace::replacepart_handles_twenty_thousand_structural_rules_in_one_walk ... ok
test runtime::tests::structure_data::replace::replacepart_rule_lists_preserve_sequential_conflicts_and_missing_positions ... ok
test runtime::tests::structure_data::replace::replacepart_span_preserves_lenient_missing_positions_and_raw_delayed_rhs ... ok
test runtime::tests::structure_data::replace::replacepart_spans_reuse_part_selection_and_fail_closed_on_malformed_shapes ... ok
test runtime::tests::structure_data::replace::replacepart_supports_immediate_delayed_and_head_replacements ... ok
test runtime::tests::structure_data::replace::replacepart_supports_pattern_position_specs_and_rhs_bindings ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_contested_guards_run_last_to_first_and_stop_after_acceptance ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_declining_later_guard_hands_position_to_earlier_rule ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_evaluates_each_rule_position_guard_at_most_once ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_guard_acceptance_preserves_raw_delayed_bodies ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_guard_suspension_resumes_same_rule_position_and_plan ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_pattern_guards_receive_bindings_and_option_scope ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_rhs_guards_accept_and_decline_exact_position_forms ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_unguarded_rules_keep_explicit_later_rule_wins_behavior ... ok
test runtime::tests::structure_data::selectors_part2::direct_empty_span_selects_full_range ... ok
test runtime::tests::structure_data::selectors_part2::direct_single_endpoint_span_selects_prefix_or_suffix_ranges ... ok
test runtime::tests::structure_data::selectors_part2::invalid_span_messages_render_infix_specs ... ok
test runtime::tests::structure_data::selectors_part2::invalid_upto_span_diagnostics_are_not_duplicated_after_normalization ... ok
test runtime::tests::structure_data::selectors_part2::part_and_extract_emit_single_take_for_explicit_invalid_spans ... ok
test runtime::tests::structure_data::selectors_part2::part_and_extract_report_branch_selector_failures ... ok
test runtime::tests::structure_data::selectors_part2::part_and_extract_take_messages_render_span_ranges ... ok
test runtime::tests::structured_exceptions::declining_exception_handler_guard_continues_ancestor_and_handler_scans ... ok
test runtime::tests::structured_exceptions::exception_type_registry_round_trips_through_loader_snapshot ... ok
test runtime::tests::structured_exceptions::suspending_exception_handler_guard_resumes_its_scan_cursor_without_replay ... ok
test runtime::tests::suspension_abandonment::abandonment_evaluates_only_withcleanup_cleanup_and_abandons_nested_input_lifo ... ok
test runtime::tests::suspension_abandonment::abandonment_finishes_module_activation_without_initializers_or_body ... ok
test runtime::tests::suspension_abandonment::abandonment_generations_are_monotonic_and_pure_frames_run_no_continuation ... ok
test runtime::tests::suspension_abandonment::abandonment_pops_monitor_without_a_final_refresh_or_callback ... ok
test runtime::tests::suspension_abandonment::abandonment_pops_option_value_scope_without_running_definition_suffix ... ok
test runtime::tests::suspension_abandonment::abandonment_pops_reap_without_rendering_or_running_body_suffix ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_abort_protect_and_check_abort_checkpoints ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_an_existing_time_deadline_exactly ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_block_initializers_and_bodies_inner_to_outer ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_block_random_prng_state ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_both_loop_iterator_scope_forms_and_iteration_budget ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_check_and_quiet_captures_without_check_fallback ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_constraint_nesting_without_fallbacks ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_test_report_stream_capture_without_a_report ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_trace_and_evaluation_data_captures_without_rendering ... ok
test runtime::tests::suspension_abandonment::abandonment_rolls_back_partial_package_install_without_reporting_failure ... ok
test runtime::tests::suspension_abandonment::checkpoint_rollback_abandons_a_trial_generation_before_restoring_state ... ok
test runtime::tests::suspension_abandonment::checkpoint_rollback_preserves_its_captured_generation ... ok
test runtime::tests::suspension_abandonment::queued_input_abandons_the_older_generation_before_consuming_the_value ... ok
test runtime::tests::syntax_information_debug_catalog::syntax_information_covers_current_cl09_debugging_surface_subset ... ok
test runtime::tests::syntax_information_lifecycle::protect_blocks_direct_syntax_information_assignment_and_unset ... ok
test runtime::tests::syntax_information_lifecycle::syntax_information_defaults_assignment_and_unset_cover_current_first_pass_subset ... ok
test runtime::tests::syntax_information_lifecycle::syntax_information_participates_in_block_and_clearall ... ok
test runtime::tests::syntax_information_lifecycle::unsetting_system_syntax_information_reverts_to_default_rules ... ok
test runtime::tests::thread_and_dispatch::argument_preparation_cow_preserves_dispatch_semantics ... ok
test runtime::tests::thread_and_dispatch::block_localizes_definitions_attributes_and_messages ... ok
test runtime::tests::thread_and_dispatch::downvalue_miss_stability_shortcut_invalidates_on_later_definition ... ok
test runtime::tests::thread_and_dispatch::evaluated_head_function_attributes_control_argument_evaluation ... ok
test runtime::tests::thread_and_dispatch::flat_attribute_flattens_calls_and_matches_flat_patterns ... ok
test runtime::tests::thread_and_dispatch::held_downvalue_matching_normalizes_nested_attribute_heads ... ok
test runtime::tests::thread_and_dispatch::listable_interacts_with_builtin_and_function_attributes ... ok
test runtime::tests::thread_and_dispatch::listable_mismatched_lengths_emit_thread_message_and_leave_call_unevaluated ... ok
test runtime::tests::thread_and_dispatch::listable_threads_user_defined_calls_and_broadcasts_scalars ... ok
test runtime::tests::thread_and_dispatch::module_handles_many_uninitialized_bindings_without_exhausting_the_native_stack ... ok
test runtime::tests::thread_and_dispatch::module_skips_existing_generated_symbol_names ... ok
test runtime::tests::thread_and_dispatch::one_identity_allows_singleton_head_patterns_to_match_atoms ... ok
test runtime::tests::thread_and_dispatch::orderless_attribute_canonicalizes_calls_and_rules ... ok
test runtime::tests::thread_and_dispatch::stable_call_shortcut_preserves_symbol_and_upvalue_evaluation ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_accepts_resolved_head_and_spec_values ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_supports_default_and_custom_thread_heads ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_supports_standard_sequence_spec_subset ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_supports_wide_dense_argument_selection ... ok
test runtime::tests::thread_and_dispatch::thread_evaluates_its_first_argument_before_threading ... ok
test runtime::tests::thread_and_dispatch_part2::thread_builtin_mismatch_emits_message_and_stays_unevaluated ... ok
test runtime::tests::traversal_part2::cases_and_position_report_invalid_match_limit_messages ... ok
test runtime::tests::traversal_part2::count_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::traversal_part2::freeq_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::traversal_part2::level_reports_invalid_levelspec_messages ... ok
test runtime::tests::traversal_part2::memberq_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::traversal_part2::position_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::traversal_part2::traversal_family_reports_invalid_levelspec_messages ... ok
test runtime::tests::valueq_stable_shortcut::value_q_shortcut_populates_only_evaluation_caches ... ok
test runtime::tests::valueq_stable_shortcut::value_q_shortcut_skips_snapshot_and_suppression_even_after_attribute_revision ... ok
test runtime::tests::valueq_stable_shortcut::value_q_stability_shortcut_matches_the_trial_path_over_one_shared_corpus ... ok
test runtime::tests::wide_child_traversals::cases_resumes_mid_walk_between_children ... ok
test runtime::tests::wide_child_traversals::cases_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::count_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::delete_cases_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::position_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::replace_all_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::replace_walks_thirty_thousand_children ... ok
test runtime::tests::wide_definition_dispatch::reap_walks_thirty_thousand_buckets ... ok
test runtime::tests::wide_definition_dispatch::replace_list_walks_thirty_thousand_rules ... ok
test runtime::tests::wide_definition_dispatch::sub_value_dispatch_resumes_mid_walk_with_pending_outcome ... ok
test runtime::tests::wide_definition_dispatch::sub_value_dispatch_walks_thirty_thousand_definitions ... ok
test runtime::tests::wide_definition_dispatch::up_value_dispatch_resumes_mid_walk_with_pending_outcome ... ok
test runtime::tests::wide_definition_dispatch::up_value_dispatch_resumes_to_next_candidate_when_condition_fails ... ok
test runtime::tests::wide_definition_dispatch::up_value_dispatch_walks_thirty_thousand_definitions ... ok

test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 23.35s

   Doc-tests rum_runtime

running 22 tests
test crates/rum-runtime/src/runtime/accounting_mode/options.rs - runtime::accounting_mode::options::RetainedOptions (line 25) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/options.rs - runtime::accounting_mode::options::RetainedOptions (line 31) - compile fail ... ok
test crates/rum-runtime/src/runtime/accounting_mode/options.rs - runtime::accounting_mode::options::RetainedOptions (line 36) - compile fail ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 107) - compile fail ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 120) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 137) - compile fail ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 153) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 165) - compile fail ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 18) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 26) - compile fail ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 36) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 44) - compile fail ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 54) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 64) - compile fail ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 77) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 84) - compile fail ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 96) - compile ... ok
test crates/rum-runtime/src/runtime/native_stack_guard.rs - runtime::native_stack_guard::NativeStackDepth (line 19) - compile fail ... ok
test crates/rum-runtime/src/runtime/native_stack_guard.rs - runtime::native_stack_guard::run_with_native_stack_guard (line 79) - compile fail ... ok
test crates/rum-runtime/src/runtime/options/reentry.rs - runtime::options::reentry::OptionReentry (line 160) - compile ... ok
test crates/rum-runtime/src/runtime/options/reentry.rs - runtime::options::reentry::OptionReentry (line 171) - compile fail ... ok
test crates/rum-runtime/src/runtime/options/reentry.rs - runtime::options::reentry::OptionReentryDeclaration (line 13) ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.70s
```

The workspace command uses --no-fail-fast and the existing RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 opt-in, with --nocapture to expose platform skips. A standalone full stdlib command was not run; its full unit/integration/doc suites are included in this workspace gate. Native ZeroMQ PermissionDenied results are skips, not transport passes.

Verbatim test/build result lines from `.temp/brief-564/final/workspace.log` (the complete raw output is archived there):

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
   Compiling rum-host v0.2.0 (/home/remi/dev/rum/crates/rum-host)
   Compiling rum-terminal v0.2.0 (/home/remi/dev/rum/crates/rum-terminal)
   Compiling rum-repl v0.2.0 (/home/remi/dev/rum/crates/rum-repl)
   Compiling rum-jupyter v0.2.0 (/home/remi/dev/rum/crates/rum-jupyter)
   Compiling rum-cli v0.2.0 (/home/remi/dev/rum/crates/rum-cli)
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2m 54s
     Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
running 15 tests
test eval_tests::already_broken_pipe_returns_with_reusable_session ... ok
test eval_tests::bindings_consume_exactly_their_queued_responses ... ok
test eval_tests::broken_pipe_during_cleanup_still_retires_input ... ok
test eval_tests::cleanup_messages_appear_once_with_original_input_reason ... ok
test eval_tests::cleanup_output_appears_once_and_tail_never_runs ... ok
test eval_tests::eval_helper_abandons ... ! input required: p: 
ok
test eval_tests::file_helper_abandons ... ! input required: p: 
ok
test eval_tests::public_file_nested_get_rolls_back_inner_to_outer ... ok
test eval_tests::refused_admission_preserves_someone_elses_pending_request ... ! cannot start a new evaluation while the session is waiting for host input; call resume_with_input or reset first
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
ok
test eval_tests::unexpected_setup_input_abandons ... ok
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
running 28 tests
test closed_stdout_pipe_exits_successfully_without_stderr ... ok
test eval_exit_renders_accumulated_prints_without_error_rendering ... ok
test eval_exit_uses_requested_status_code_without_error_rendering ... ok
test eval_flag_binds_piped_stdin_as_script_input ... ok
test eval_flag_keeps_non_hold_form_canonical_output_unchanged ... ok
test eval_flag_keeps_to_string_form_arms_unchanged ... ok
test eval_flag_renders_hold_form_transparently_in_canonical_style ... ok
test eval_flag_renders_prints_before_final_value ... ok
test eval_flag_renders_source_aware_syntax_diagnostics ... ok
test eval_flag_treats_replace_repeated_limit_as_recoverable_value ... ok
test file_and_stdin_scripts_render_later_error_at_the_written_character ... ok
test file_exit_renders_accumulated_prints_without_error_rendering ... ok
test file_exit_uses_requested_status_code_without_error_rendering ... ok
test file_flag_renders_source_aware_syntax_diagnostics ... ok
test filter_mode_uses_script_semantics_and_prints_final_result_only ... ok
test format_flag_accepts_prefix_equals_line_width ... ok
test format_flag_prints_canonical_surface_form ... ok
test format_flag_rejects_invalid_line_width ... ok
test format_flag_respects_custom_line_width ... ok
test no_arg_non_tty_stdin_is_filter_program_without_repl_chatter ... ok
test parse_flag_prints_debug_dump ... ok
test positional_dash_reads_stdin_as_program ... ok
test random_default_seed_is_reproducible_across_cli_invocations ... ok
test runtime_messages_go_to_stderr_and_values_go_to_stdout ... ok
test script_args_after_separator_are_bound_as_strings ... ok
test script_args_separator_without_args_binds_empty_list ... ok
test script_input_preserves_quotes_and_backslashes_verbatim ... ok
test unknown_long_flag_is_rejected ... ok
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
running 6 tests
test all_script_modes_deliver_cleanup_once_without_cancel_diagnostic ... ok
test all_script_modes_report_input_and_skip_tail ... ok
test all_script_modes_report_nested_get_input ... ok
test all_script_modes_stop_at_first_of_two_inputs ... ok
test program_modes_consume_stdin_and_arguments_as_binding_data ... ok
test stdin_program_modes_bind_only_arguments ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
     Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
running 9 tests
test evaluation_sigint_after_answer ... ok
test interrupt ... ok
test masked_answer ... ok
test masked_eof ... ok
test masked_interrupt ... ok
test masked_io_error ... ok
test ordinary_answer ... ok
test primary_sigint ... ok
test primary_sigint_after_answer ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
running 7 tests
test expr::tests::displays_global_symbol_without_context_prefix ... ok
test expr::tests::displays_message_name_shorthand ... ok
test expr::tests::displays_nested_call_expression ... ok
test expr::tests::displays_string_atoms_with_escaped_special_characters ... ok
test expr::tests::displays_system_symbol_without_context_prefix ... ok
test expr::tests::normalizes_int_values_and_orders_by_numeric_value ... ok
test expr::tests::normalizes_rational_atoms ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
running 5 tests
test tests::an_existing_frame_root_trace_bound_is_not_duplicated ... ok
test tests::expansion_covers_all_struct_and_enum_field_shapes ... ok
test tests::generic_bounds_are_added_only_for_parameters_used_by_fields ... ok
test tests::pure_frame_abandon_preserves_generics_and_where_clauses ... ok
test tests::unions_are_rejected_with_a_clear_error ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
running 3 tests
test accessor_detaches_the_original_link_and_leaves_payloads_movable ... ok
test next_link_supports_non_clone_payloads_lifetimes_and_const_generics ... ok
test unsupported_arrangements_report_specific_diagnostics ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
running 3 tests
test derives_all_enum_variant_shapes ... ok
test derives_named_tuple_unit_and_generic_structs ... ok
test pure_frame_abandon_discards_structs_and_enums_without_inspecting_fields ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
running 90 tests
test file_io::tests::normalize_path_falls_back_to_current_dir_for_missing_relative_paths ... ok
test file_io::tests::std_file_io_converts_read_and_write_errors ... ok
test file_io::tests::std_file_io_lists_only_immediate_directory_entries ... ok
test file_io::tests::std_file_io_probe_distinguishes_files_directories_and_missing_paths ... ok
test lifecycle_contract_tests::abandoned_poisoned_session_does_not_contaminate_a_fresh_session ... 
ok
test lifecycle_contract_tests::panic_poison_blocks_public_entries_until_reset ... 
ok
test lifecycle_contract_tests::reset_between_requests_isolates_implicit_session_state ... ok
test lifecycle_contract_tests::reset_rebuilds_all_runtime_accumulation_dimensions ... ok
test model::outcome::tests::failure_code_maps_runtime_request_errors_to_host_codes ... ok
test session::tests::panic_is_complete_response_marks_session_poisoned_and_invalid ... 
ok
test tests::capability_profiles::capability_policy_can_allow_clock_without_filesystem ... clock policy AbsoluteTime[] value=895571155823934429/500000000
ok
test tests::capability_profiles::capability_policy_can_allow_reads_without_writes ... ok
test tests::capability_profiles::capability_policy_can_explicitly_allow_diagnostics ... ok
test tests::capability_profiles::clock_policy_allows_composed_datetime_current_readers ... ok
test tests::capability_profiles::default_host_config_installs_entropy_provider_for_random_bytes ... ok
test tests::capability_profiles::default_host_config_installs_entropy_provider_for_random_password ... ok
test tests::capability_profiles::default_host_config_keeps_diagnostic_catalog_unrestricted ... ok
test tests::capability_profiles::default_host_config_keeps_filesystem_catalog_unrestricted ... ok
test tests::capability_profiles::every_host_profile_inherits_the_derived_native_stack_budget ... ok
test tests::capability_profiles::hermetic_profile_pins_all_capabilities_and_providers ... ok
test tests::capability_profiles::hermetic_profile_replays_rendered_results_across_sessions_and_reset ... ok
test tests::capability_profiles::permissive_policy_without_entropy_provider_returns_noentropy_failed ... ok
test tests::capability_profiles::permissive_policy_without_file_provider_returns_noio_failed ... no_provider_get value=$Failed messages=[HostMessage { symbol: "General", tag: "noio", text: "no file provider installed" }]
ok
test tests::capability_profiles::random_policy_allows_entropy_reseed ... ok
test tests::capability_profiles::sandboxed_host_config_allows_pure_datetime_forms ... ok
test tests::capability_profiles::sandboxed_host_config_blocks_clock_builtins_at_dispatch ... sandboxed AbsoluteTime[] value=$Failed messages=[HostMessage { symbol: "General", tag: "sandbox", text: "AbsoluteTime is not permitted in this sandbox." }]
ok
test tests::capability_profiles::sandboxed_host_config_blocks_diagnostic_builtins_at_dispatch ... ok
test tests::capability_profiles::sandboxed_host_config_blocks_entropy_builtins_but_allows_prng_builtins ... ok
test tests::capability_profiles::sandboxed_host_config_blocks_exit_at_dispatch_but_trusted_allows_it ... ok
test tests::capability_profiles::sandboxed_host_config_blocks_filesystem_builtins_at_dispatch ... ok
test tests::capability_profiles::sandboxed_host_config_blocks_interactive_builtins_at_dispatch ... ok
test tests::capability_profiles::sandboxed_host_config_blocks_runtime_constructed_filesystem_calls ... ok
test tests::capability_profiles::sandboxed_host_config_still_allows_pure_work_and_path_string_helpers ... ok
test tests::capability_profiles::sandboxed_host_config_with_file_provider_blocks_internal_loader_calls ... restricted+provider BeginPackage value=$Failed messages=[HostMessage { symbol: "General", tag: "sandbox", text: "Needs is not permitted in this sandbox." }] context="Global`" context_path=List["System`", "Global`"] packages=List[]
ok
test tests::capability_profiles::sandboxed_profile_pins_all_capabilities_and_providers ... ok
test tests::capability_profiles::scripting_profile_pins_all_capabilities_and_providers ... ok
test tests::capability_profiles::trusted_host_file_names_uses_the_std_file_io_provider ... ok
test tests::editor_surface::host_eval_renders_top_level_hold_form_transparently_in_canonical_style ... ok
test tests::editor_surface::host_eval_reports_value_messages_prints_and_structured_outputs ... ok
test tests::editor_surface::host_evaluates_multiline_string_literals_as_ordinary_strings ... ok
test tests::editor_surface::host_lower_dump_returns_canonical_text_and_rendered_syntax_failures ... ok
test tests::editor_surface::host_message_catalog_supports_global_symbol_and_invalid_queries ... ok
test tests::editor_surface::host_parse_inspect_complete_and_is_complete_cover_editor_flows ... ok
test tests::editor_surface::host_print_sink_streams_exclusively_and_clear_restores_buffering ... ok
test tests::editor_surface::host_processes_inline_multiline_csv_data ... ok
test tests::editor_surface::host_queued_input_completes_without_suspension ... ok
test tests::editor_surface::host_suspend_resume_flow_wraps_input_requests ... ok
test tests::eval_contract::host_cancel_handle_interrupts_running_eval_from_another_thread ... host_cancel_handle_interrupts_running_eval_from_another_thread elapsed_ms=7 cancel_latency_ms=7
ok
test tests::eval_contract::host_eval_file_uses_provider_and_missing_provider_is_other ... ok
test tests::eval_contract::host_eval_literal_terminated_evaluation_has_no_termination_reason ... ok
test tests::eval_contract::host_eval_memory_ceiling_reports_structured_termination_and_session_recovers ... ok
test tests::eval_contract::host_eval_ordinary_value_has_no_termination_reason ... ok
test tests::eval_contract::host_eval_recursion_limit_reports_structured_termination_reason ... ok
test tests::eval_contract::host_eval_script_suspends_but_resume_fails_without_running_remaining_forms ... ok
test tests::eval_contract::host_eval_script_uses_last_form_and_classifies_syntax_failures ... ok
test tests::eval_contract::host_eval_with_timeout_interrupts_running_eval ... host_eval_with_timeout_interrupts_running_eval elapsed_ms=200
ok
test tests::eval_contract::host_eval_with_unused_handle_and_generous_timeout_still_succeeds ... host_eval_with_unused_handle_and_generous_timeout_still_succeeds elapsed_ms=0
ok
test tests::eval_contract::host_failure_codes_report_session_reset_requirement ... ok
test tests::eval_contract::host_failures_use_stable_failure_codes ... ok
test tests::eval_contract::host_iteration_accumulator_completes_after_multiple_inputs_without_poisoning ... ok
test tests::option_rule::option_rule_host_cancel_and_reset_dispose_pending_input ... ok
test tests::session_lifecycle::host_boundary_catches_panic_and_poisons_session ... 
ok
test tests::session_lifecycle::host_boundary_normal_eval_does_not_poison_session ... host_boundary_normal_eval_does_not_poison_session first=3 second=7 poisoned=false
ok
test tests::session_lifecycle::host_reset_removes_print_sink_and_restores_buffering ... ok
test tests::session_lifecycle::host_session_and_cancel_handle_are_send ... ok
test tests::session_lifecycle::host_session_introspection_enumerates_user_definitions_and_contexts ... introspection definitions=[HostDefinitionSummary { name: "Global`a", has_own_value: true, down_value_count: 0, up_value_count: 0, sub_value_count: 0 }, HostDefinitionSummary { name: "Global`f", has_own_value: false, down_value_count: 2, up_value_count: 0, sub_value_count: 0 }] contexts=["Global`", "System`"]
ok
test tests::session_lifecycle::host_session_introspection_survives_poisoned_session ... 
ok
test tests::session_lifecycle::host_session_reset_clears_user_state ... host_session_reset_clears_user_state assigned=41 after_reset="x" arithmetic=4 poisoned=false
ok
test tests::session_lifecycle::host_session_reset_recovers_poisoned_session ... 
ok
test tests::session_lifecycle::host_session_reset_restores_original_config_not_mutated_state ... host_session_reset_restores_original_config_not_mutated_state current_context="Original`" context_path=["System`", "Original`"] recursion_limit=77
ok
test tests::session_lifecycle::host_session_snapshot_bytes_roundtrip_and_restore_replaces_user_state ... host_session_snapshot_bytes_roundtrip_and_restore_replaces_user_state definitions=[HostDefinitionSummary { name: "Global`a", has_own_value: true, down_value_count: 0, up_value_count: 0, sub_value_count: 0 }, HostDefinitionSummary { name: "Global`f", has_own_value: false, down_value_count: 1, up_value_count: 0, sub_value_count: 0 }, HostDefinitionSummary { name: "Global`g", has_own_value: false, down_value_count: 1, up_value_count: 0, sub_value_count: 0 }, HostDefinitionSummary { name: "Global`h", has_own_value: false, down_value_count: 1, up_value_count: 0, sub_value_count: 0 }] bytes=145 f7=7 attributes_h=List[Orderless] z_after_restore=z
ok
test tests::session_lifecycle::host_session_snapshot_roundtrip_rederives_lazy_list_identically ... ok
test tests::session_lifecycle::host_session_snapshot_roundtrip_relinks_sealed_values ... ok
test tests::session_lifecycle::parallel_host_sessions_move_to_threads_and_isolate_definitions ... parallel_host_sessions thread=0 result=0
ok
test tests::typed_bridge::host_call_enforces_typed_input_depth_cap ... ok
test tests::typed_bridge::host_call_evaluates_integer_arguments ... ok
test tests::typed_bridge::host_call_normalizes_rational_input ... ok
test tests::typed_bridge::host_call_rejects_empty_head ... ok
test tests::typed_bridge::host_call_rejects_malformed_big_integer ... ok
test tests::typed_bridge::host_call_rejects_output_only_other_argument ... ok
test tests::typed_bridge::host_call_rejects_zero_denominator_rational ... ok
test tests::typed_bridge::host_call_resolves_short_and_fully_qualified_symbols ... ok
test tests::typed_bridge::host_call_resolves_short_system_head_for_string_join ... ok
test tests::typed_bridge::host_call_round_trips_large_integer_and_immediate_rule ... ok
test tests::typed_bridge::host_call_round_trips_typed_association ... ok
test tests::typed_bridge::host_call_treats_string_arguments_as_injection_safe_data ... ok
test tests::typed_bridge::host_eval_exposes_immediate_rule_and_rejects_delayed_rule_data_view ... ok
test tests::typed_bridge::host_eval_exposes_ordered_typed_association_entries ... ok
test tests::typed_bridge::host_eval_exposes_typed_value_variants ... typed_value input="2 + 3" text="5" typed=Integer(5)
ok
test tests::typed_bridge::host_eval_keeps_delayed_association_out_of_typed_data_view ... ok
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
running 7 tests
test every_public_evaluation_entrypoint_rejects_a_pending_continuation ... ok
test public_config_facade_controls_core_builtin_installation ... ok
test public_contract_covers_editor_requests_and_failures ... ok
test public_contract_covers_eval_outputs_without_runtime_types ... ok
test public_contract_preserves_input_suspension_and_resume_boundaries ... ok
test public_contract_rejects_overlapping_eval_and_preserves_block_suspension ... ok
test script_suspension_requires_reset_or_restore_while_inspection_remains_available ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
running 5 tests
test input_restore_failure_retires_owner_preserving_partial_state ... ok
test nested_input_restore_failure_retires_owner_preserving_partial_state ... ok
test nested_public_file_cancellation_keeps_host_usable ... ok
test reset_nested_public_file_runs_no_guest_cleanup ... ok
test throw_restore_failure_control_keeps_partial_state_usable ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
running 4 tests
test all_guest_entries_resume_twice_without_poisoning_or_reopening_provider ... ok
test cancellation_at_either_position_rolls_back_and_old_handle_cannot_cancel_new_input ... ok
test error_after_resumed_load_is_a_failure_value_and_session_remains_usable ... ok
test reset_and_replacement_retire_either_position_without_guest_cleanup ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
running 1 test
test host_payloads_declare_bounded_destruction_at_the_crate_edge ... CASE legacy_impl: /home/remi/.rustup/toolchains/1.85.0-x86_64-unknown-linux-gnu/bin/cargo check --offline --manifest-path /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_impl/Cargo.toml --target-dir /home/remi/dev/rum/target/host-payload-destruction-contract/legacy_impl/target --message-format=json --color=never
error: could not compile `host-payload-legacy-impl` (lib) due to 1 previous error
error: could not compile `host-payload-legacy-new` (lib) due to 1 previous error
error: could not compile `host-payload-legacy-from-arc` (lib) due to 1 previous error
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
error: could not compile `host-payload-legacy-file-io-impl` (lib) due to 1 previous error
error: could not compile `host-payload-legacy-file-io-new` (lib) due to 1 previous error
error: could not compile `host-payload-legacy-file-io-from-arc` (lib) due to 1 previous error
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
error: could not compile `host-payload-legacy-raw-closure-sink` (lib) due to 1 previous error
error: could not compile `host-payload-legacy-boxed-fnmut-sink` (lib) due to 1 previous error
error: could not compile `host-payload-named-sink-without-marker` (lib) due to 1 previous error
error: could not compile `host-payload-adapter-private-field-bypass` (lib) due to 1 previous error
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.06s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.16s
ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.74s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
running 4 tests
test installer_builtins_obey_capabilities_and_survive_reset ... ok
test installer_config_clones_reapply_downstream_builtins ... ok
test installer_order_and_no_core_path_are_preserved ... ok
test installer_snapshot_restore_round_trips_definitions_not_builtin_handlers ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
running 7 tests
test file_and_entropy_providers_from_arc_share_identity_and_state ... ok
test host_config_tracing_and_assert_flags_flow_into_built_sessions ... ok
test host_config_uses_fixed_entropy_for_reseed_random ... ok
test host_config_uses_injected_file_io_for_get_and_needs ... ok
test host_request_string_adapters_and_defaults_cover_public_facade ... ok
test host_session_builder_policy_and_core_builtin_controls_are_applied ... ok
test restore_from_snapshot_source_that_requests_input_returns_synthesized_failure ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
running 16 tests
test cancel_accepted_before_eval_request_survives_request_prologue ... ok
test cancel_handle_interrupts_running_eval_and_session_remains_usable ... ok
test cancel_handle_targets_one_request_and_is_inert_after_completion ... ok
test definitions_and_contexts_start_clean_and_track_user_state ... ok
test eval_with_timeout_returns_normal_value_before_deadline ... ok
test eval_with_timeout_terminates_long_eval_and_session_remains_usable ... ok
test host_session_and_cancel_handle_are_send_across_worker_thread ... ok
test pending_input_request_tracks_suspension_and_clears_after_resume ... ok
test replace_repeated_limit_returns_terminated_success ... ok
test reset_is_idempotent_and_old_cancel_handles_do_not_affect_rebuilt_session ... ok
test resource_limit_returns_terminated_evaluation_and_session_remains_usable ... ok
test restore_from_invalid_bytes_returns_failure_without_panic_or_state_loss ... ok
test snapshot_restore_preserves_package_contexts_and_replaces_prior_state ... ok
test stale_cancel_cannot_erase_a_current_request_cancel ... ok
test structured_host_value_depth_cap_preserves_text_and_uses_other_for_deep_tail ... ok
test terminated_snapshot_restore_fails_while_unlimited_restore_succeeds ... ok
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
running 2 tests
test missing_entropy_provider_fails_random_password_policy_options ... ok
test sandboxed_host_config_blocks_random_password_policy_options ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
running 6 tests
test battered_session_recovers_after_repeated_recoverable_limit_trips ... ignored
test bug_snapshot_restore_500_generated_definitions_overflows_native_stack ... ok
test cancel_interrupts_heavy_rewrite_loop_and_session_recovers ... ok
test cancel_rewrite_loop_stability_10_cycles ... ignored
test eval_with_timeout_interrupts_heavy_rewrite_loop_and_session_recovers ... ok
test snapshot_restore_preserves_20_generated_definitions_and_attributes ... ok
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.29s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
running 13 tests
test accepted_cancellation_survives_timeout_arming_and_the_request_prologue ... ok
test cancellation_during_resumed_work_is_observed_with_timeout_still_armed ... ok
test cancellation_of_expired_suspensions_wins_before_input_conversion ... ok
test cancellation_via_each_evaluation_entrypoint_releases_the_suspended_timeout ... ok
test completion_and_syntax_failure_release_the_owned_timeout ... ok
test expired_input_suspensions_resume_under_the_original_host_deadline ... ok
test immediate_resume_finishes_with_a_generous_host_timeout ... ok
test panics_during_timed_eval_or_resume_poison_until_reset ... 
ok
test rejected_entrypoints_and_inspection_preserve_a_suspended_deadline ... ok
test rejected_timeout_cannot_arm_or_shorten_a_pending_request ... ok
test reset_releases_suspended_timeout_and_invalidates_its_cancel_handle ... ok
test timeout_can_expire_during_a_resumed_segment ... ok
test two_suspensions_keep_the_same_host_deadline ... ok
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.00s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
running 12 tests
test public_cancel_handle_acquired_while_suspended_abandons_before_input_conversion ... cancelLateSentinel404=Success(value=0, termination=None, messages=0, prints=[], trace=0)
ok
test public_cancelled_block_suspension_restores_scope_without_running_continuation ... cancelBlock404=Success(value="outside", termination=None, messages=0, prints=[], trace=0)
ok
test public_cancelled_module_suspension_reclaims_local_without_running_continuation ... Names["Global`cancelModuleLocal404$*"]=Success(value=List[], termination=None, messages=0, prints=[], trace=0)
ok
test public_cancelled_nested_with_cleanup_runs_cleanup_and_abandons_nested_input ... List[cancelCleanupInner404, cancelCleanupOuter404, cancelCleanupAfterInput404, cancelBodyAfterInput404]=Success(value=List[1, 1, 0, 0], termination=None, messages=0, prints=[], trace=0)
ok
test public_cancelled_script_suspension_is_a_reset_free_escape_route ... List[cancelScriptTail404, cancelEscapeAttempt404]=Success(value=List[0, 0], termination=None, messages=0, prints=[], trace=0)
ok
test public_cancelled_suspension_restores_attribute_iterator_and_option_state ... Attributes[cancelAttribute404]=Success(value=List[], termination=None, messages=0, prints=[], trace=0)
ok
test public_cancelled_suspension_restores_prng_and_abort_protect_state ... RandomInteger[1000000]=Success(value=534495, termination=None, messages=0, prints=[], trace=0)
ok
test public_cancelled_suspension_restores_reap_monitor_trace_and_constraint_state ... cancel.matrix.reap.memory=cancelled:before:70413,after:138757,growth:68344;completed:before:70413,after:329503,growth:259090;margin:3336
ok
test public_cancelled_with_suspension_restores_scope_without_running_continuation ... cancelWith404=Success(value="outside", termination=None, messages=0, prints=[], trace=0)
ok
test public_monitor_matrix_restores_capture_package_and_cleanup_owners ... matrix.capture.evaluation_data=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
ok
test public_monitor_matrix_restores_reap_monitor_trace_constraints_random_and_abort_state ... matrix.reap.monitor=Success(value=Null, termination=None, messages=0, prints=[], trace=0)
ok
test public_rollback_matrix_restores_lexical_iterator_attribute_and_option_state ... matrix.block.valueq=Success(value=True, termination=None, messages=0, prints=[], trace=0)
ok
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
running 151 tests
test adapter::render::tests::error_output_covers_memory_constraint_failures ... ok
test adapter::render::tests::error_output_covers_resource_limit_failures ... ok
test adapter::render::tests::error_output_covers_time_constraint_failures ... ok
test adapter::render::tests::json_from_typed_maps_supported_tree_and_escapes_strings ... ok
test adapter::render::tests::json_from_typed_rejects_ambiguous_or_partial_trees ... ok
test adapter::render::tests::render_rule_application_source_covers_replace ... ok
test json_support::tests::parser_handles_nested_json_scalars_and_arrays ... ok
test json_support::tests::parser_reports_malformed_json_without_panicking ... ok
test json_support::tests::parser_round_trips_utf8_and_surrogate_pairs_in_strings ... ok
test signature::tests::hmac_sha256_matches_known_test_vector ... ok
test tests::adapter::complete_request_maps_cursor_prefix_to_session_completion_response ... ok
test tests::adapter::complete_request_replaces_the_full_symbol_when_cursor_is_inside_it ... ok
test tests::adapter::dispatch::control_request_interrupt_updates_lifecycle_state ... ok
test tests::adapter::dispatch::control_request_shutdown_transitions_kernel_to_shutdown ... ok
test tests::adapter::dispatch::dispatch_request_emits_busy_and_idle_status_for_kernel_info ... ok
test tests::adapter::dispatch::dispatch_request_emits_iopub_status_execute_input_and_execute_result ... ok
test tests::adapter::dispatch::dispatch_request_handles_comm_info_without_touching_runtime_session ... ok
test tests::adapter::dispatch::dispatch_request_ignores_comm_messages_without_affecting_session_state ... ok
test tests::adapter::dispatch::dispatch_request_replies_to_debug_request_as_unsupported_without_exiting ... ok
test tests::adapter::dispatch::shell_request_envelope_dispatches_execute_requests ... ok
test tests::adapter::dispatch::shell_request_envelope_dispatches_is_complete_requests ... ok
test tests::adapter::dispatch::shell_request_envelope_dispatches_kernel_info_and_shutdown ... ok
test tests::adapter::execute_depth_capped_typed_value_omits_json_mime ... ok
test tests::adapter::execute_json_representable_value_adds_json_without_changing_expr_mime ... ok
test tests::adapter::execute_literal_cancelled_termination_value_remains_successful ... ok
test tests::adapter::execute_non_representable_value_keeps_only_existing_expr_mime ... ok
test tests::adapter::execute_request_cycle_termination_emits_stderr_trace_and_result ... ok
test tests::adapter::execute_request_maps_to_session_eval_and_tracks_execution_count ... ok
test tests::adapter::execute_request_with_tracing_emits_display_data_separately_from_execute_result ... ok
test tests::adapter::inspect_request_maps_symbol_at_cursor_to_session_inspect_response ... ok
test tests::adapter::is_complete_request_maps_runtime_completeness_to_kernel_shape ... ok
test tests::bootstrap::bootstrap_config_derives_endpoints_and_wire_policy_from_connection_file ... ok
test tests::bootstrap::connection_file_parser_loads_jupyter_connection_shapes ... ok
test tests::bootstrap::connection_file_parser_reports_missing_required_fields ... ok
test tests::bootstrap::connection_file_parser_round_trips_unicode_string_fields ... ok
test tests::bootstrap::connection_file_signature_policy_respects_declared_signature_scheme ... ok
test tests::bootstrap::frontend_style_execute_round_trip_uses_local_socket_backend ... ok
test tests::bootstrap::frontend_style_stdin_round_trip_uses_local_socket_backend ... ok
test tests::bootstrap::local_socket_transport_backend_echoes_heartbeat_payloads ... ok
test tests::bootstrap::local_socket_transport_backend_exposes_frontend_boundary_for_frontend_style_flows ... ok
test tests::bootstrap::mock_transport_backend_exposes_bootstrap_boundary_for_future_real_transports ... ok
test tests::bootstrap::socket_driver_observes_shutdown_promptly_during_running_execute ... ok
test tests::bootstrap::socket_driver_pumps_heartbeat_and_execute_over_local_session ... ok
test tests::bootstrap::socket_driver_resumes_stdin_round_trip_over_local_session ... ok
test tests::bootstrap::socket_session_config_derives_zero_mq_facing_socket_shapes ... ok
test tests::crate_manifest_keeps_jupyter_on_host_facade ... ok
test tests::execute_lifecycle::interrupt::interrupt_after_denied_input_retirement_does_not_retire_again ... ok
test tests::execute_lifecycle::interrupt::interrupt_after_suspension_before_completion_poll_retires_on_worker ... ok
test tests::execute_lifecycle::interrupt::interrupt_at_second_prompt_finishes_original_request ... ok
test tests::execute_lifecycle::interrupt::red_interrupt_while_waiting_finishes_request ... ok
test tests::execute_lifecycle::mismatched_parent_preserves_waiting_execute ... ok
test tests::execute_lifecycle::non_execute_reply_during_input_wait_preserves_busy ... ok
test tests::execute_lifecycle::overlap::immediate_overlap_while_running_is_rejected ... ok
test tests::execute_lifecycle::overlap::immediate_overlap_while_waiting_preserves_count_and_context ... ok
test tests::execute_lifecycle::overlap::overlap_while_running_is_rejected_before_session_move ... ok
test tests::execute_lifecycle::overlap::overlap_with_identical_parent_keeps_original_routing ... ok
test tests::execute_lifecycle::overlap::red_overlap_rejection_preserves_busy_state ... ok
test tests::execute_lifecycle::overlap::red_overlap_reply_keeps_original_routing ... ok
test tests::execute_lifecycle::overlap::wire_overlap_while_running_keeps_original_routing ... ok
test tests::input_abandonment::cleanup_messages_keep_guest_cancel_and_original_input_reason ... ok
test tests::input_abandonment::controls::refused_immediate_admission_keeps_original_request ... ok
test tests::input_abandonment::controls::refused_worker_admission_keeps_original_request ... ok
test tests::input_abandonment::controls::shell_wrapper_is_reply_only_and_caller_services_stdin ... ok
test tests::input_abandonment::controls::worker_nested_get_still_resumes ... ok
test tests::input_abandonment::controls::worker_password_flag_still_resumes ... ok
test tests::input_abandonment::controls::worker_twice_still_resumes ... ok
test tests::input_abandonment::immediate_cleanup_and_prefix_are_delivered_once ... ok
test tests::input_abandonment::immediate_nested_get_abandons_and_rolls_back ... ok
test tests::input_abandonment::immediate_prefix_output_is_delivered_once ... ok
test tests::input_abandonment::immediate_without_stdin_abandons ... ok
test tests::input_abandonment::red_shell_wrapper_without_stdin_abandons ... ok
test tests::input_abandonment::red_sync_execute_abandons ... ok
test tests::input_abandonment::red_worker_without_stdin_abandons ... ok
test tests::input_abandonment::transport::red_event_pump_without_stdin_abandons ... ok
test tests::input_abandonment::transport::red_local_driver_without_stdin_abandons ... ok
test tests::input_abandonment::worker_cleanup_and_prefix_are_delivered_once ... ok
test tests::input_abandonment::worker_nested_get_abandons_and_rolls_back ... ok
test tests::input_abandonment::worker_prefix_output_is_delivered_once ... ok
test tests::input_abandonment::worker_retirement_runs_cleanup_off_transport_thread ... ok
test tests::output_once::cross_family_order_is_grouped_per_segment ... FAMILY stdout=true stderr=true trace=true terminal=true
ok
test tests::output_once::equal_prints_keep_every_occurrence ... FAMILY stdout=true stderr=true trace=true terminal=true
ok
test tests::output_once::failure_after_resume_matches_queued_input ... FAMILY stdout=true stderr=true trace=true terminal=true
ok
test tests::output_once::host_buffers::host_families_append_across_success_exit_and_cancellation ... HOST 42/suspend prints=1->2 messages=1->2 trace=3->6 grouped_append=false
ok
test tests::output_once::host_buffers::synthesized_panic_has_no_buffers_and_preserves_published_prefix ... 
ok
test tests::output_once::interrupt_after_resume_matches_queued_input ... FAMILY stdout=true stderr=true trace=true terminal=true
ok
test tests::output_once::messages_before_and_after_prompt_match_queued_input ... FAMILY stdout=true stderr=true trace=true terminal=true
ok
test tests::output_once::nested_get_outputs_match_queued_input ... FAMILY stdout=true stderr=true trace=true terminal=true
ok
test tests::output_once::one_prompt_outputs_match_queued_input ... FAMILY stdout=true stderr=true trace=true terminal=true
ok
test tests::output_once::red_worker_streams_match_queued_input ... ok
test tests::output_once::trace_events_match_queued_input ... FAMILY stdout=true stderr=true trace=true terminal=true
ok
test tests::output_once::two_prompts_outputs_match_queued_input ... FAMILY stdout=true stderr=true trace=true terminal=true
ok
test tests::pump::control_interrupt_cancels_running_execute_and_deferred_shell_runs_after_idle ... ok
test tests::pump::event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately ... ok
test tests::pump::pump_next_prioritizes_control_messages_before_shell_messages ... ok
test tests::pump::pump_next_prioritizes_stdin_replies_while_kernel_is_waiting_for_input ... ok
test tests::pump::pump_until_idle_stops_after_shutdown_and_leaves_later_shell_work_queued ... ok
test tests::stdin::adapter_can_stage_stdin_requests_for_future_execute_integration ... ok
test tests::stdin::adapter_rejects_stdin_requests_when_execute_disallows_them ... ok
test tests::stdin::execute_request_with_disallowed_stdin_returns_input_error_reply ... ok
test tests::stdin::execute_request_with_inputstring_stages_stdin_and_defers_execute_reply ... ok
test tests::stdin::input_reply_resumes_delayed_message_templates_through_internal_string_driver ... ok
test tests::stdin::input_reply_resumes_execute_and_preserves_shell_routing_ids ... ok
test tests::stdin::input_reply_resumes_execute_without_rerunning_prefix_side_effects ... ok
test tests::stdin::input_reply_resumes_multi_sequence_matcher_backtracking_across_multiple_prompts ... ok
test tests::stdin::input_reply_without_parent_header_resumes_pending_execute ... ok
test tests::transport_coverage::codec_preserves_reply_routing_ids_for_replies_and_stdin_requests ... ok
test tests::transport_coverage::codec_rejects_missing_signature_bad_delimiter_and_session_mismatch ... ok
test tests::transport_coverage::codec_rejects_wrong_channels_for_response_iopub_and_stdin_messages ... ok
test tests::transport_coverage::local_socket_transport_pair_routes_messages_and_heartbeats_in_memory ... ok
test tests::wire::comm_info_request_round_trips_with_empty_comm_registry ... ok
test tests::wire::debug_request_round_trips_as_explicit_unsupported_reply ... ok
test tests::wire::event_pump_ignores_comm_messages_and_keeps_processing_later_execute ... ok
test tests::wire::execute_request_round_trips_allow_stdin_flag ... ok
test tests::wire::history_request_round_trips_with_empty_reply ... ok
test tests::wire::kernel_info_round_trips_on_control_channel ... ok
test tests::wire::mock_socket_transport_routes_reply_and_iopub_messages_by_channel ... ok
test tests::wire::optional_signature_policy_accepts_unsigned_messages_but_still_checks_signed_ones ... ok
test tests::wire::request_headers_round_trip_date_field_through_wire_codec ... ok
test tests::wire::signed_transport_codec_rejects_signature_mismatch ... ok
test tests::wire::signed_transport_codec_round_trips_request_messages ... ok
test tests::wire::transport_codec_accepts_structured_frontend_metadata ... ok
test tests::wire::transport_codec_decodes_escaped_and_utf8_unicode_request_payloads ... ok
test tests::wire::transport_codec_decodes_execute_reply_and_iopub_messages ... ok
test tests::wire::transport_codec_rejects_invalid_channel_for_request_type ... ok
test tests::wire::transport_codec_round_trips_stdin_input_messages ... ok
test tests::wire::transport_encoding_serializes_reply_and_iopub_messages ... ok
test transport::json::encode::tests::encodes_iopub_and_helper_json_values_without_losing_escaping ... ok
test transport::json::encode::tests::encodes_less_common_request_and_response_shapes_as_structured_json ... ok
test transport::json::parse::tests::aggregate_field_helpers_parse_payloads_history_language_and_help_links ... ok
test transport::json::parse::tests::aggregate_field_helpers_reject_malformed_nested_shapes ... ok
test transport::json::parse::tests::field_helpers_accept_supported_shapes_and_report_typed_errors ... ok
test transport::json::parse::tests::iopub_content_parsing_covers_payload_variants_and_errors ... ok
test transport::json::parse::tests::message_header_parsing_defaults_date_and_reports_field_errors ... ok
test transport::json::parse::tests::request_content_parsing_covers_defaults_known_kinds_and_ignored_messages ... ok
test transport::json::parse::tests::response_content_parsing_covers_reply_variants_and_malformed_statuses ... ok
test transport::json::parse::tests::status_parsers_cover_all_known_values_and_error_fields ... ok
test zeromq::tests::fake::zeromq_backend_binds_expected_socket_endpoints ... ok
test zeromq::tests::fake::zeromq_backend_propagates_independently_resolved_bind_endpoints ... ok
test zeromq::tests::fake::zeromq_backend_reports_malformed_shell_frames_as_structured_wire_errors ... ok
test zeromq::tests::fake::zeromq_backend_round_trips_execute_over_fake_router_socket ... ok
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
ok
test zeromq::tests::support::harness_tests::block_on_result_handles_success_inner_error_and_harness_timeout_without_sockets ... 
ok
test zeromq::tests::support::harness_tests::parent_scope_excludes_sentinels_including_late_duplicates_and_stops_at_target_idle ... ok
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
running 48 tests
test commands::tests::execute_history_renders_numbered_entries ... ok
test commands::tests::parse_format_returns_a_structured_command ... ok
test commands::tests::parse_history_reports_invalid_counts ... ok
test commands::tests::trace_step_renderer_degrades_for_missing_and_unknown_fields ... ok
test completeness::tests::certified_incomplete_prefix_corpus_agrees_with_the_host_parser ... ok
test completeness::tests::fast_incomplete_hint_never_hides_lexer_or_parser_errors ... ok
test completeness::tests::incremental_completeness_matches_the_host_matrix ... balanced single line: Complete (NeedsFullCheck)
ok
test completeness::tests::multiline_completeness_work_is_linear_in_appended_source ... linear-cost-counter lines=256 source_bytes=773 fast_examined_bytes=1546 full_check_bytes=773 full_check_calls=1 total_examined_bytes=2319
ok
test terminal::tests::answers_use_the_source_readers_existing_buffer ... ok
test terminal::tests::masked_panic_restores_terminal_and_descriptor ... 
ok
test terminal::tests::masked_prompt_error_restores_terminal_and_descriptor ... ok
test tests::repl_bare_inspect_command_shows_usage_instead_of_evaluating ... ok
test tests::repl_bare_trace_command_shows_usage_instead_of_evaluating ... ok
test tests::repl_complete_command_lists_builtin_and_user_candidates ... ok
test tests::repl_format_command_accepts_suffix_equals_line_width ... ok
test tests::repl_format_command_prints_canonical_surface_form ... ok
test tests::repl_format_command_rejects_invalid_line_width ... ok
test tests::repl_format_command_respects_custom_line_width ... ok
test tests::repl_handles_help_and_quit ... ok
test tests::repl_history_persists_multiline_inputs_as_single_logical_entries ... ok
test tests::repl_history_persists_when_a_history_path_is_configured ... ok
test tests::repl_inspect_command_does_not_fall_through_to_evaluation ... ok
test tests::repl_inspect_shows_definition_summaries ... ok
test tests::repl_inspect_surfaces_attribute_flags ... ok
test tests::repl_inspect_surfaces_message_definition_flags ... ok
test tests::repl_lower_command_prints_canonical_core_expression ... ok
test tests::repl_messages_lists_catalog_entries_for_a_symbol ... ok
test tests::repl_messages_reports_when_a_symbol_has_no_messages ... ok
test tests::repl_parse_command_prints_syntax_debug_output ... ok
test tests::repl_parse_command_renders_source_diagnostic_for_bad_input ... ok
test tests::repl_reads_multiline_input_until_a_form_is_complete ... ok
test tests::repl_renders_hold_form_transparently_in_canonical_style ... ok
test tests::repl_renders_parser_diagnostics_for_regular_input_errors ... ok
test tests::repl_renders_runtime_messages_before_results ... ok
test tests::repl_reports_incomplete_multiline_input_when_input_ends ... ok
test tests::repl_trace_command_filters_with_a_pattern_form ... ok
test tests::repl_trace_command_passes_a_numeric_depth_levelspec ... ok
test tests::repl_trace_command_renders_a_small_tree ... ok
test tests::repl_trace_command_renders_messages_before_the_tree ... ok
test tests::repl_trace_command_rewrites_a_bare_symbol_filter ... ok
test tests::repl_trace_command_truncates_by_default_and_full_is_the_escape_hatch ... ok
test tests::repl_trace_command_uses_the_host_failure_renderer_for_bad_input ... ok
test tests::repl_trace_rules_command_renders_a_small_step ... ok
test tests::repl_trace_rules_composes_with_form_rewrite_and_depth ... ok
test tests::repl_trace_rules_names_the_overlapping_definition_that_fired ... ok
test tests::repl_trace_rules_renders_a_definition_stored_step ... ok
test tests::repl_trace_rules_truncates_and_full_is_the_escape_hatch ... ok
test tests::repl_trace_without_rules_is_byte_for_byte_unchanged ... ok
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
running 10 tests
test cancellation_keeps_prefix_output_and_guest_cleanup_messages_once ... ok
test eof_at_a_later_prompt_cancels_the_original_request ... ok
test permanently_broken_output_cannot_prevent_guest_cleanup ... ok
test read_error_retires_request_before_returning_original_error ... ok
test red_eof_abandons_and_leaves_borrowed_session_usable ... ok
test red_interrupt_abandons_and_leaves_borrowed_session_usable ... ok
test red_password_without_secure_reader_abandons ... ok
test rejected_overlaps_preserve_the_callers_pending_request ... ok
test trace_abandonment_runs_cleanup_once_and_keeps_session_usable ... ok
test write_error_retires_request_including_prompt_flush_and_prefix ... ok
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
running 12 tests
test empty_and_whitespace_answers_are_preserved ... ok
test empty_prompt_uses_input_prompt ... ok
test malformed_input_is_left_to_host_without_reprompt ... ok
test prefix_output_is_flushed_before_each_prompt_and_never_replayed ... ok
test red_answer_is_literal_and_not_saved_in_history ... ok
test red_line_resume_matches_queued ... ok
test red_nested_get_resume_matches_queued ... ok
test red_plain_trace_resume_matches_queued ... ok
test red_trace_resume_matches_queued ... ok
test red_two_inputs_resume_matches_queued ... ok
test trace_form_and_depth_resume_matches_queued ... ok
test trace_full_and_default_limit_survive_input ... ok
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
running 1097 tests
test runtime::accounting_mode::frozen::option_tests::option_frozen_complement_repeated_resume_and_abandon ... ok
test runtime::accounting_mode::frozen::option_tests::option_loop_frame_resumes_with_another_arc_owner ... ok
test runtime::accounting_mode::frozen::option_tests::option_loop_frame_traces_inputs_and_accumulated_prefix ... ok
test runtime::accounting_mode::frozen::option_tests::option_reentry_domain_replacement_expires_resume ... ok
test runtime::accounting_mode::frozen::option_tests::option_reentry_registration_does_not_mint_certification ... plain public registration => $Failed; public installer can assert trust, outside repository certificates => List[17]
ok
test runtime::accounting_mode::frozen::option_tests::option_working_copy_checkpoint_and_shared_frame_release ... ok
test runtime::accounting_mode::frozen::tests::option_rule_shared_frame_charges_working_result_and_preserves_last_owner ... ok
test runtime::accounting_mode::frozen::tests::retention_shared_frame_keeps_roots_and_lease_until_last_owner_drops ... shared frame: root preserved on success/error, complement 272 released at last drop
ok
test runtime::accounting_mode::options::tests::option_flat_late_arm_accounts_slots_and_preflights_freeze ... ok
test runtime::accounting_mode::options::tests::option_handoff_checks_domain_identity_when_generations_collide ... ok
test runtime::accounting_mode::options::tests::option_owner_capacity_and_callback_adoption ... ok
test runtime::accounting_mode::options::tests::option_owner_guest_error_and_unwind_release ... 
ok
test runtime::accounting_mode::options::tests::option_owner_handoff_and_late_arm_keep_original_visible ... ok
test runtime::accounting_mode::options::tests::option_owner_snapshot_charges_defaults_and_copies ... ok
test runtime::accounting_mode::rule::tests::exits::option_rule_abandon_reset_and_late_arm_frame ... ok
test runtime::accounting_mode::rule::tests::exits::option_rule_frame_refusal_abandons_without_a_transfer ... ok
test runtime::accounting_mode::rule::tests::exits::option_rule_late_arm_after_callback_and_during_input ... ok
test runtime::accounting_mode::rule::tests::exits::option_rule_repeated_nested_input_transfers_once ... ok
test runtime::accounting_mode::rule::tests::option_rule_callback_adoption_refuses_before_growth_and_preserves_errors ... ok
test runtime::accounting_mode::rule::tests::option_rule_clone_and_shell_contract ... ok
test runtime::accounting_mode::rule::tests::option_rule_guest_exits_are_lazy_and_release ... 
ok
test runtime::accounting_mode::rule::tests::option_rule_late_arm_observes_existing_prefix ... ok
test runtime::accounting_mode::rule::tests::option_rule_name_is_adopted_before_growth ... ok
test runtime::accounting_mode::rule::tests::option_rule_overflow_and_partial_failure_release ... ok
test runtime::accounting_mode::rule::tests::option_rule_unarmed_shared_head_keeps_canonical_rule ... ok
test runtime::accounting_mode::tests::compile::retention_compile_controls_precede_pinned_failures ... missing-root-traversal/control: success=true
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
error: aborting due to 1 previous error
ok
test runtime::accounting_mode::tests::exits::retention_bridge_releases_prefix_on_guest_error_and_panic ... 
ok
test runtime::accounting_mode::tests::exits::retention_frame_factory_is_lazy_and_refusal_abandons_pending_input ... ok
test runtime::accounting_mode::tests::exits::retention_nested_unarmed_bridges_keep_every_outer_prefix ... nested outer=0, guest entries=1
ok
test runtime::accounting_mode::tests::exits::retention_overflow_stays_invalid_after_release_and_old_ledger_is_isolated ... ok
test runtime::accounting_mode::tests::exits::retention_suspension_checks_actual_continuation_roots ... continuation roots 1: checked before suspension publication, released
ok
test runtime::accounting_mode::tests::retention_activation_rejects_live_unarmed_prefix_before_guest_body ... activation prefix=0: guest-body entries=1, callback=7
ok
test runtime::accounting_mode::tests::retention_lifetime_normal_early_error_drop_and_unwind ... lifetime exit 0: RetentionSnapshot { bytes: Some(0), live: 0, transfers: 0, releases: 1 }
ok
test runtime::accounting_mode::tests::retention_r_is_separate_from_s_and_both_admission_entries_see_it ... R/S separation: S=23906, peak=23906, R=112160, both admissions rejected
ok
test runtime::accounting_mode::tests::retention_suspension_transfers_once_resumes_twice_and_abandons ... suspension first=RetentionSnapshot { bytes: Some(1040), live: 1, transfers: 1, releases: 0 }, second=RetentionSnapshot { bytes: Some(48), live: 1, transfers: 2, releases: 1 }, complete=RetentionSnapshot { bytes: Some(0), live: 0, transfers: 2, releases: 3 }
ok
test runtime::accounting_mode::tests::retention_unarmed_operations_do_not_construct_receipts_or_walk_weights ... ok
test runtime::accounting_mode::tests::retention_unarmed_suspended_capacity_is_visible_to_later_arm ... unarmed suspension: S=31849, R=1024, rejected while pending, released on abandon
ok
test runtime::accumulator_chain::tests::exclusive_depth_16 ... exclusive length=16: maximum payload drop depth=1
ok
test runtime::accumulator_chain::tests::exclusive_depth_64 ... exclusive length=64: maximum payload drop depth=1
ok
test runtime::accumulator_chain::tests::live_weak_does_not_prevent_drain ... live Weak length=16: maximum payload drop depth=1
ok
test runtime::accumulator_chain::tests::shared_tail_stops_then_drains ... shared tail first prefix: 8 nodes/payloads, maximum payload drop depth=1
ok
test runtime::accumulator_chain::tests::wrapper_is_transparent_and_extraction_preserves_ownership ... ok
test runtime::builtin_catalog::tests::core_catalog_drives_both_dispatch_and_default_attributes ... ok
test runtime::builtin_catalog::tests::moved_stdlib_system_attributes_remain_kernel_known_without_registration ... ok
test runtime::call_preparation::tests::planner_allows_raw_sequence_splicing_for_held_arguments ... ok
test runtime::call_preparation::tests::planner_keeps_held_arguments_verbatim_under_holdallcomplete ... ok
test runtime::call_preparation::tests::planner_respects_evaluate_wrapper_for_held_arguments ... ok
test runtime::call_preparation::tests::planner_respects_unevaluated_wrapper_for_nonheld_arguments ... ok
test runtime::capabilities::tests::capability_policy_allowlist_is_default_deny ... ok
test runtime::capabilities::tests::capability_policy_builder_can_allow_and_deny_individual_capabilities ... ok
test runtime::capabilities::tests::capability_policy_can_explicitly_allow_diagnostics ... ok
test runtime::capabilities::tests::capability_policy_defaults_to_unrestricted ... ok
test runtime::capabilities::tests::capability_policy_narrowing_preserves_existing_restrictions ... ok
test runtime::capabilities::tests::catalog_capabilities_classify_clock_builtins ... ok
test runtime::capabilities::tests::catalog_capabilities_classify_diagnostic_builtins ... ok
test runtime::capabilities::tests::catalog_capabilities_classify_io_builtins ... ok
test runtime::definitions::application::replace_all::plan::frame_root_trace_tests::rewrite_rule_plan_visits_the_exact_rule_roots ... ok
test runtime::definitions::definition_lhs_preparation::syntax_policy::tests::every_registered_matcher_head_has_an_explicit_lhs_policy ... ok
test runtime::definitions::definition_lhs_preparation::tests::already_canonical_lhs_reuses_its_shared_arguments ... ok
test runtime::definitions::unique_symbols::tests::keeps_small_symbol_sets_on_the_linear_path ... ok
test runtime::definitions::unique_symbols::tests::preserves_first_seen_order_across_the_adaptive_index_threshold ... ok
test runtime::definitions::value_table_assignment::tests::abandoning_prepared_value_table_keeps_old_definitions_and_skips_pending_lhs ... ok
test runtime::eval_call_prepare::abandonment_tests::abandoning_prepared_arguments_and_restores_skips_remaining_arguments ... ok
test runtime::evaluation_cache::tests::append_to_args_become_exclusive_after_definition_take_and_cache_entry_removal ... ok
test runtime::evaluation_cache::tests::append_to_preserves_an_aliased_list_value ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_absent_stale_and_dirty_proofs_recover_by_recomputation ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_direct_update_keeps_args_identity_and_reinserts_proof ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_direct_update_polls_interrupt_before_mutation ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_direct_update_rolls_back_append_and_replace ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_nested_multi_and_bare_part_targets_use_fallback ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_observable_or_constrained_environments_use_fallback ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_recovery_falls_back_when_data_became_unstable ... ok
test runtime::evaluation_cache::tests::association_lvalue::association_lvalue_update_preserves_an_aliased_value ... ok
test runtime::evaluation_cache::tests::association_stability_index_preserves_distinct_and_duplicate_verdicts ... ok
test runtime::evaluation_cache::tests::association_stability_tail::cache_cap_postorder_insertion_trace_matches_recursive_front ... ok
test runtime::evaluation_cache::tests::association_stability_tail::dirty_and_generation_cache_traces_match_recursive_front ... ok
test runtime::evaluation_cache::tests::association_stability_tail::duplicate_key_cache_trace_matches_recursive_front ... ok
test runtime::evaluation_cache::tests::association_stability_tail::recursive_front_exhaustion_owns_intervening_ordinary_calls ... ok
test runtime::evaluation_cache::tests::association_stability_tail::rule_delayed_cache_trace_matches_recursive_front ... ok
test runtime::evaluation_cache::tests::association_stability_tail::unstable_early_child_cache_trace_matches_recursive_front ... ok
test runtime::evaluation_cache::tests::call_stability_profile_reports_per_source_head_outcomes ... ignored, release-only profiling gate; run with RUM_PROFILE_EVALUATION_CACHE=1
test runtime::evaluation_cache::tests::dirty_pointer_matched_entry_is_evicted_without_proof ... ok
test runtime::evaluation_cache::tests::evaluation_cache_is_invalidated_by_attribute_change ... ok
test runtime::evaluation_cache::tests::evaluation_cache_is_invalidated_by_new_down_value ... ok
test runtime::evaluation_cache::tests::generation_stale_pointer_matched_entry_is_evicted_without_proof ... ok
test runtime::evaluation_cache::tests::path_eviction_removes_only_the_named_exact_call_nodes ... ok
test runtime::evaluation_cache::tests::pointer_mismatched_entry_is_left_untouched ... ok
test runtime::evaluation_cache::tests::proof_reinsertion_uses_current_generation_dirty_snapshot_and_bloom ... ok
test runtime::evaluation_cache::tests::removing_valid_call_stability_entry_preserves_evaluation_outcome ... ok
test runtime::evaluation_cache::tests::repeated_unrelated_writes_do_not_bump_call_stability_generation ... ok
test runtime::evaluation_cache::tests::repeated_writes_to_one_symbol_keep_invalidation_sound ... ok
test runtime::evaluation_cache::tests::shared_args_make_mut_profiles_only_cow_and_preserves_alias_values ... ok
test runtime::evaluation_cache::tests::stability_cache_survives_unrelated_symbol_writes ... ok
test runtime::evaluation_cache::tests::stability_native_stack_poll_is_strided_and_fails_closed ... ok
test runtime::evaluation_cache::tests::valid_call_stability_proof_preflight_is_non_consuming_and_reuses_validity ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::accumulator_is_abandoned_mid_iteration_without_resuming_body ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::appended_chain_abandonment_uses_drain ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::into_vec_preserves_prefix_and_tail_order_when_unique_or_shared ... ok
test runtime::kernel_data::control_data::branching_loops::iteration::tests::shared_tail_survives_abandoned_clone ... ok
test runtime::kernel_data::rewrite_data::suspended_state::tests::abandoning_shared_rewrite_queue_preserves_remaining_binding_order ... ok
test runtime::kernel_data::structure_data::exact_paths::tests::collect_exact_paths_for_position_spec_forced_front_preserves_branch_order ... ok
test runtime::kernel_data::structure_data::exact_paths::tests::exact_expr_at_path_forced_front_matches_each_branched_witness ... ok
test runtime::kernel_data::structure_data::exact_paths::tests::replace_exact_path_forced_front_matches_head_and_child_edges ... ok
test runtime::kernel_data::structure_data::replace::plan::tests::replace_plan_resolves_duplicates_and_prefixes_during_insertion ... ok
test runtime::kernel_data::structure_data::replace::plan::tests::root_target_discards_existing_and_later_descendants ... ok
test runtime::kernel_data::structure_data::replace::plan_application::tests::iterative_replace_plan_rebuild_preserves_unchanged_sibling_storage ... ok
test runtime::kernel_data::structure_data::replace::suspended_state::tests::abandoning_guarded_target_branch_preserves_surviving_chunks_and_updates ... ok
test runtime::lowering::tests::branched_lowering_certifies_front_and_tail ... ok
test runtime::lowering::tests::branched_lowering_matches_original_exact_result ... ok
test runtime::lowering::tests::head_chains_below_bound_preserve_every_element ... ok
test runtime::lowering::tests::lowering_error_restores_front_budget ... ok
test runtime::lowering::tests::public_lowering_paths_reject_manual_trees_above_bound ... ok
test runtime::lowering::tests::tail_preserves_sugar_and_symbol_resolution_order ... ok
test runtime::options::core::rule_tests::option_rule_c02_non_call_immediate_and_delayed ... ok
test runtime::options::reentry_tests::option_reentry_compile_pass_precedes_pinned_compile_fail ... control: success=true
error: aborting due to 1 previous error
ok
test runtime::pattern_syntax::tests::matcher_pattern_registry_round_trips_system_heads_only ... ok
test runtime::session::guest_loading::tests::abandonment_cleanup_loading_file_drains_nested_worklist ... ok
test runtime::session::guest_loading::tests::cancellation_restores_nested_snapshots_inner_to_outer_at_each_input ... ok
test runtime::session::guest_loading::tests::dependency_exit_restores_package_frame_and_preserves_earlier_definitions ... ok
test runtime::session::guest_loading::tests::guest_environment_restores_handled_error_tail_and_memory_limit_before_child ... ok
test runtime::session::guest_loading::tests::message_scope_and_print_sink_are_not_replayed_across_inputs ... ok
test runtime::session::guest_loading::tests::nested_files_close_once_and_all_owner_queues_empty ... ok
test runtime::session::guest_loading::tests::paused_guest_form_reinstalls_original_deadline_before_child_frames ... ok
test runtime::session::guest_loading::tests::pending_frames_trace_snapshot_definitions_and_previous_form_values ... ok
test runtime::session::session_requests::tests::eval_error_mapper_preserves_non_request_surface_limit_codes ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_alpha_rename_does_not_rewrite_inserted_bound_values ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_sequence_splice_preserves_spliced_item_shared_nodes ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_substitution_preserves_unchanged_sibling_call_shared_nodes ... ok
test runtime::substitution::binding_substitution::tests::rule_rhs_substitution_reuses_bound_call_shared_nodes ... ok
test runtime::substitution::lexical_substitution::tests::explicit_function_parameters_shadow_outer_bindings ... ok
test runtime::substitution::lexical_substitution::tests::rule_rhs_substitution_respects_pattern_locals ... ok
test runtime::substitution::lexical_substitution::tests::with_body_treats_binding_names_as_shadowed ... ok
test runtime::suspension::primitive::tests::binding_vec_continuation_round_trips_multiple_symbol_maps ... ok
test runtime::suspension::primitive::tests::option_vec_expr_continuation_round_trips_some_none_and_fallback_payloads ... ok
test runtime::suspension::rewrite_rules::tests::option_rewrite_rule_info_continuation_round_trips_some_none_and_fallbacks ... ok
test runtime::suspension::traversal::tests::option_cases_level_spec_continuation_round_trips_some_none_and_bare_spec ... ok
test runtime::tests::arithmetic::exact_division_and_negative_integer_powers_produce_rationals ... ok
test runtime::tests::arithmetic::exact_integer_arithmetic_evaluates_core_heads ... ok
test runtime::tests::arithmetic::exact_integer_mod_quotient_gcd_lcm_and_powermod_follow_floor_style_semantics ... ok
test runtime::tests::arithmetic::factorial_respects_integer_work_limit ... ok
test runtime::tests::arithmetic::plus_and_times_fold_numeric_parts_without_forcing_symbolic_algebra ... ok
test runtime::tests::arithmetic::power_integer_work_limit_allows_normal_and_trivial_results ... ok
test runtime::tests::arithmetic::power_integer_work_limit_guards_large_denominators ... ok
test runtime::tests::arithmetic::power_memory_projection_refuses_before_large_allocation ... ok
test runtime::tests::arithmetic::power_reads_the_live_integer_work_limit ... ok
test runtime::tests::arithmetic::power_realistic_memory_ceiling_admits_one_million_bit_result ... ok
test runtime::tests::arithmetic::power_respects_integer_work_limit_for_large_positive_exponents ... ok
test runtime::tests::arithmetic::powermod_negative_exponents_use_modular_inverse ... ok
test runtime::tests::arithmetic::rational_atoms_expose_a_rational_head_for_patterns ... ok
test runtime::tests::arithmetic::round_handles_exact_numbers_decimal_literals_and_half_even_ties ... ok
test runtime::tests::arithmetic::scalar_math_builtins_exact_cases ... ok
test runtime::tests::arithmetic::scalar_math_builtins_report_declared_attributes ... ok
test runtime::tests::arithmetic::scalar_math_builtins_thread_over_lists_and_leave_skipped_forms_symbolic ... ok
test runtime::tests::arithmetic::total_evaluates_documented_level_forms_and_leaves_invalid_specs_inert ... ok
test runtime::tests::arithmetic::wide_plus_and_times_group_late_symbolic_duplicates ... ok
test runtime::tests::association_lookup_cache::association_lookup_index_weak_identity_survives_cow_updates ... ok
test runtime::tests::association_lookup_cache::repeated_wide_association_reads_build_one_linear_index ... ok
test runtime::tests::association_part_cursor::below_threshold_part_reads_stay_on_the_linear_scan_path ... ok
test runtime::tests::association_part_cursor::duplicate_association_keys_keep_first_position_and_last_value ... ok
test runtime::tests::association_part_cursor::part_reads_reuse_the_lookup_cache_and_observe_cow_writes ... ok
test runtime::tests::association_part_cursor::wide_association_single_key_and_position_reads_return_exact_values ... ok
test runtime::tests::association_write_index::appended_keys_after_a_wide_read_stay_readable ... ok
test runtime::tests::association_write_index::repeated_replacement_writes_stay_indexed ... ok
test runtime::tests::association_write_index::rewriting_an_appended_key_replaces_it_through_the_index ... ok
test runtime::tests::association_write_index::small_association_writes_bypass_the_lookup_cache ... ok
test runtime::tests::association_write_recovery::association_incremental_writes_preserve_ordinary_symbol_semantics_past_the_cache_size_cap ... ok
test runtime::tests::association_write_recovery::association_incremental_writes_stay_direct_past_the_cache_size_cap ... ok
test runtime::tests::comparison_lowering::comparison_lowering_accepts_a_uniform_chain_at_the_boundary ... ok
test runtime::tests::comparison_lowering::comparison_lowering_bounds_direct_chain_nodes ... ok
test runtime::tests::comparison_lowering::comparison_lowering_keeps_parenthesized_subtrees_independent ... ok
test runtime::tests::comparison_lowering::comparison_lowering_preserves_resolver_call_count_and_order ... ok
test runtime::tests::comparison_lowering::comparison_parser_rejects_a_long_uniform_chain_gracefully ... ok
test runtime::tests::context_and_loader::bare_symbols_search_context_path_before_current_context ... ok
test runtime::tests::context_and_loader::begin_and_end_balance_context_and_context_path ... ok
test runtime::tests::context_and_loader::begin_package_and_end_package_restore_outer_context_and_export_package_context ... ok
test runtime::tests::context_and_loader::begin_package_dependencies_stay_on_private_context_path_when_already_loaded ... ok
test runtime::tests::context_and_loader::begin_package_expands_relative_context_specs_against_the_current_context ... ok
test runtime::tests::context_and_loader::begin_package_loads_present_dependency_and_enters_package ... ok
test runtime::tests::context_and_loader::begin_package_missing_dependency_fails_closed_and_restores_state ... ok
test runtime::tests::context_and_loader::begin_package_second_dependency_failure_restores_state_and_package_list ... ok
test runtime::tests::context_and_loader::begin_package_usage_messages_make_public_symbols_visible_from_private_context ... ok
test runtime::tests::context_and_loader::context_builtin_reports_session_and_symbol_contexts_without_evaluating_symbols ... ok
test runtime::tests::context_and_loader::context_path_and_packages_assignments_update_session_fields ... ok
test runtime::tests::context_and_loader::context_variables_and_builtin_inspection_reflect_session_contexts ... ok
test runtime::tests::context_and_loader::contexts_lists_known_session_contexts ... ok
test runtime::tests::context_and_loader::explicitly_protected_session_configuration_variables_reject_assignments ... ok
test runtime::tests::context_and_loader::file_loading::findfile_resolves_package_names_and_file_wrappers ... ok
test runtime::tests::context_and_loader::file_loading::get_and_needs_syntax_messages_report_the_loaded_file_location ... ok
test runtime::tests::context_and_loader::file_loading::get_cycle_termination_does_not_abort_remaining_file_forms ... ok
test runtime::tests::context_and_loader::file_loading::get_loads_a_script_file_and_returns_the_last_value ... ok
test runtime::tests::context_and_loader::file_loading::get_propagates_exit_without_collapsing_it_to_failed_load ... ok
test runtime::tests::context_and_loader::file_loading::get_rolls_back_loader_side_effects_and_collapses_failed_file_messages ... ok
test runtime::tests::context_and_loader::file_loading::get_treats_resource_termination_as_failed_load_and_rolls_back ... ok
test runtime::tests::context_and_loader::file_loading::loader_builtins_accept_resolved_load_target_specs ... ok
test runtime::tests::context_and_loader::file_loading::missing_findfile_emits_runtime_message ... ok
test runtime::tests::context_and_loader::file_loading::missing_needs_returns_failed_and_emits_messages ... ok
test runtime::tests::context_and_loader::file_loading::needs_accepts_resolved_context_and_file_specs ... ok
test runtime::tests::context_and_loader::file_loading::needs_loads_package_once_and_tracks_loaded_packages ... ok
test runtime::tests::context_and_loader::file_loading::needs_supports_explicit_file_targets ... ok
test runtime::tests::context_and_loader::file_loading::nested_get_prefers_the_current_loader_directory_over_package_search_paths ... ok
test runtime::tests::context_and_loader::path_assignment_remains_writable_without_explicit_protect ... ok
test runtime::tests::context_and_loader::script_forms::eval_script_pins_the_second_form_syntax_error_message ... ok
test runtime::tests::context_and_loader::script_forms::eval_script_preserves_messages_emitted_before_a_later_error ... ok
test runtime::tests::context_and_loader::script_forms::lex_fallback_preserves_normalized_crlf_values_and_file_error_spans ... ok
test runtime::tests::context_and_loader::script_forms::parse_script_forms_matches_the_line_driven_splitter_over_the_corpus ... ok
test runtime::tests::context_and_loader::script_forms::split_script_into_forms_skips_initial_shebang_trivia ... ok
test runtime::tests::context_and_loader::script_forms::split_script_into_forms_stops_hard_syntax_errors_from_poisoning_later_forms ... ok
test runtime::tests::context_and_loader::script_forms::split_script_into_forms_treats_multiline_comments_as_incomplete_trivia ... ok
test runtime::tests::debug_tools::echo_family_prints_and_returns_values_in_the_current_first_pass_subset ... ok
test runtime::tests::debug_tools::on_and_off_gate_matching_messages_at_session_scope ... ok
test runtime::tests::debug_tools::rum_profile_hooks_are_inert_without_profile_env ... ok
test runtime::tests::debug_tools::short_uses_the_current_canonical_text_truncation_subset ... ok
test runtime::tests::debug_tools::trace_detail_form_filter_matches_the_underlying_expression_not_the_association ... ok
test runtime::tests::debug_tools::trace_detail_has_the_same_list_nesting_shape_as_trace ... ok
test runtime::tests::debug_tools::trace_detail_identifies_the_downvalue_that_won_among_overlapping_definitions ... ok
test runtime::tests::debug_tools::trace_detail_preserves_definition_symbol_kind_and_value ... ok
test runtime::tests::debug_tools::trace_detail_returns_the_full_evaluation_pass_association_without_leaking_trace_state ... ok
test runtime::tests::debug_tools::trace_detail_supports_levelspec_and_heads_options ... ok
test runtime::tests::debug_tools::trace_print_emits_trace_lines_and_returns_the_underlying_value ... ok
test runtime::tests::debug_tools::trace_print_uses_the_same_nested_structure_for_deeper_trace_branches ... ok
test runtime::tests::debug_tools::trace_returns_a_first_pass_structured_expression_list_without_leaking_session_trace_state ... ok
test runtime::tests::debug_tools::trace_uses_recorded_depths_to_keep_nested_branches_stable ... ok
test runtime::tests::definition_lhs_preparation::active_definition_lhs_positions_complete_own_value_evaluation_for_both_set_forms ... ok
test runtime::tests::definition_lhs_preparation::active_lhs_and_pattern_test_pattern_operands_still_prepare_once ... ok
test runtime::tests::definition_lhs_preparation::active_nested_calls_dispatch_held_nested_calls_do_not_and_callable_heads_prepare_first ... ok
test runtime::tests::definition_lhs_preparation::adding_hold_attributes_after_an_existing_definition_keeps_old_and_new_identities_live ... ok
test runtime::tests::definition_lhs_preparation::clear_then_reinstall_has_no_stale_prepared_identity_or_matcher ... ok
test runtime::tests::definition_lhs_preparation::definition_lhs_pattern_regions_prepare_constraints_but_defer_binders_and_predicates ... ok
test runtime::tests::definition_lhs_preparation::definition_lhs_preparation_obeys_evaluate_unevaluated_and_sequence_controls ... ok
test runtime::tests::definition_lhs_preparation::held_definition_lhs_positions_keep_raw_symbols_and_source_lookup_reaches_them ... ok
test runtime::tests::definition_lhs_preparation::held_ground_flat_and_orderless_identities_are_reachable_from_both_forms ... ok
test runtime::tests::definition_lhs_preparation::matcher_wrappers_keep_literal_pattern_regions_raw_and_prepare_nested_constraints ... ok
test runtime::tests::definition_lhs_preparation::one_metadata_snapshot_controls_every_argument_even_after_an_earlier_mutation ... ok
test runtime::tests::definition_lhs_preparation::reassigned_then_cleared_lhs_own_value_changes_only_future_definition_identities ... ok
test runtime::tests::definition_store_sealing::definition_store_and_targeted_removal_are_sealed_behind_prepared_tokens ... ok
test runtime::tests::definition_store_sealing::prepared_store_and_removal_call_site_inventory_is_closed ... ok
test runtime::tests::definitions_and_rules::defaultvalues_and_definition_include_explicit_syntax_information ... ok
test runtime::tests::definitions_and_rules::defaultvalues_lists_optional_defaults_and_clearall_removes_them ... ok
test runtime::tests::definitions_and_rules::definition_inspection_builtins_accept_symbol_name_strings ... ok
test runtime::tests::definitions_and_rules::definition_returns_empty_hold_for_symbols_without_definitions ... ok
test runtime::tests::definitions_and_rules::definition_returns_held_first_pass_definition_statements ... ok
test runtime::tests::definitions_and_rules::delayed_self_alias_preserves_downvalues ... ok
test runtime::tests::definitions_and_rules::dynamic_iterator_lhs_values_are_normalized_with_the_existing_cycle_guard ... ok
test runtime::tests::definitions_and_rules::immediate_self_alias_preserves_downvalues ... ok
test runtime::tests::definitions_and_rules::information_returns_first_pass_symbol_summary ... ok
test runtime::tests::definitions_and_rules::information_uses_catalog_and_symbol_name_usage_fallbacks_and_stored_message_tags ... ok
test runtime::tests::definitions_and_rules::iterator_bindings_normalize_set_lhs_for_every_eager_iterator_and_assignment ... ok
test runtime::tests::definitions_and_rules::iterator_lhs_normalization_covers_nested_and_shadowed_iterators ... ok
test runtime::tests::definitions_and_rules::non_iterator_lhs_binding_controls_keep_their_exact_results ... ok
test runtime::tests::definitions_and_rules::ownvalues_lists_immediate_and_delayed_symbol_definitions ... ok
test runtime::tests::definitions_and_rules::real_iterator_definitions_precede_dynamic_values_and_clear_reveals_them_again ... ok
test runtime::tests::definitions_and_rules::self_referential_delayed_ownvalue_returns_symbol_without_message ... ok
test runtime::tests::definitions_and_rules::self_referential_ownvalue_returns_symbol_without_message ... ok
test runtime::tests::definitions_and_rules::set_assigns_simple_symbol_and_lookup_works ... ok
test runtime::tests::definitions_and_rules::setdelayed_stores_rhs_without_immediate_evaluation ... ok
test runtime::tests::definitions_and_rules::valueq_uses_rollback_trial_evaluation_without_side_effects ... ok
test runtime::tests::definitions_and_rules_cases::cases_applies_rule_transformations ... ok
test runtime::tests::definitions_and_rules_cases::cases_can_match_the_root_expression ... ok
test runtime::tests::definitions_and_rules_cases::cases_collects_nested_matches ... ok
test runtime::tests::definitions_and_rules_cases::cases_does_not_treat_heads_as_separate_candidates ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_condition_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_except_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_optional_and_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_honors_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_cases::cases_preserves_immediate_and_delayed_rule_behavior ... ok
test runtime::tests::definitions_and_rules_cases::cases_rule_rhs_splices_named_blanksequence_matches ... ok
test runtime::tests::definitions_and_rules_cases::cases_supports_exact_range_negative_and_infinity_level_specs ... ok
test runtime::tests::definitions_and_rules_cases::cases_supports_explicit_head_traversal_control ... ok
test runtime::tests::definitions_and_rules_cases::cases_supports_standard_integer_level_specs ... ok
test runtime::tests::definitions_and_rules_cases::cases_traverses_depth_first_with_leaves_before_roots_and_respects_match_limits ... ok
test runtime::tests::definitions_and_rules_cases::deletecases_can_delete_heads_and_association_values ... ok
test runtime::tests::definitions_and_rules_cases::deletecases_deletes_top_level_matches_by_default ... ok
test runtime::tests::definitions_and_rules_cases::deletecases_supports_standard_levelspecs_and_limits ... ok
test runtime::tests::definitions_and_rules_part2::compiled_matcher_matches_interpreted_matcher_on_supported_subset ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_can_substitute_multiple_named_patterns ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_matches_after_argument_evaluation ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_splices_named_blanknullsequence_into_empty_lists ... ok
test runtime::tests::definitions_and_rules_part2::downvalue_splices_named_blanksequence_into_rhs_calls ... ok
test runtime::tests::definitions_and_rules_part2::inspect_definition_catalog_reports_kinds_and_delayedness ... ok
test runtime::tests::definitions_and_rules_part2::interpreted_matcher_owned_can_resume_sequence_conditions_across_input ... ok
test runtime::tests::definitions_and_rules_part2::lowers_named_pattern_assignment ... ok
test runtime::tests::definitions_and_rules_part2::lowers_slots_and_pure_functions ... ok
test runtime::tests::definitions_and_rules_part2::matchq_operator_form_is_inert_and_applies_to_data ... ok
test runtime::tests::definitions_and_rules_part2::matchq_reports_false_for_non_matching_pattern ... ok
test runtime::tests::definitions_and_rules_part2::matchq_reports_true_for_matching_pattern ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_blanknullsequence_in_argument_lists ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_blanksequence_in_argument_lists ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_conditions_on_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_except_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_optional_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_patterntest_on_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_plain_blank_arguments_under_fixed_heads ... ok
test runtime::tests::definitions_and_rules_part2::matchq_supports_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_part2::named_slots_fill_from_rules_and_association_arguments ... ok
test runtime::tests::definitions_and_rules_part2::set_supports_immediate_pattern_based_downvalues_and_subvalues ... ok
test runtime::tests::definitions_and_rules_part2::setdelayed_stores_and_applies_simple_downvalue ... ok
test runtime::tests::definitions_and_rules_part2::unmatched_downvalue_leaves_call_in_place ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rhs_substitution_does_not_capture_inserted_values_in_lexical_scopes ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rhs_substitution_keeps_inserted_compound_values_outside_new_locals ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rhs_substitution_preserves_same_name_local_binding_behavior ... ok
test runtime::tests::definitions_and_rules_part3::delayed_rule_uses_current_rhs_value_at_application_time ... ok
test runtime::tests::definitions_and_rules_part3::flat_and_orderless_flat_heads_regroup_for_match_replace_and_replacelist ... ok
test runtime::tests::definitions_and_rules_part3::immediate_rule_captures_current_rhs_value ... ok
test runtime::tests::definitions_and_rules_part3::replace_and_replaceat_exact_rule_index_preserve_first_rule_priority ... ok
test runtime::tests::definitions_and_rules_part3::replace_defaults_to_the_root_level ... ok
test runtime::tests::definitions_and_rules_part3::replace_heads_option_controls_head_traversal ... ok
test runtime::tests::definitions_and_rules_part3::replace_rewrites_association_values_without_touching_keys ... ok
test runtime::tests::definitions_and_rules_part3::replace_supports_levelspecs_and_rule_lists ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_applies_rule_list_in_order ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_applies_single_rule ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_exact_rule_index_preserves_first_rule_priority ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_condition_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_except_and_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_honors_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_shape_prefilter_stays_conservative_for_head_attributes ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_splices_named_blanksequence_into_rhs_calls ... ok
test runtime::tests::definitions_and_rules_part3::replaceall_traverses_nested_expressions ... ok
test runtime::tests::definitions_and_rules_part3::replacelist_collects_multiple_backtracking_matches_for_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_part3::replacelist_supports_operator_form_rule_order_and_max_results ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_delayed_rule_limit_returns_terminated_value ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_limit_returns_terminated_value_for_non_terminating_rewrite_cycle ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_max_iterations_option_controls_effective_limit ... ok
test runtime::tests::definitions_and_rules_part3::replacerepeated_reaches_a_fixed_point ... ok
test runtime::tests::definitions_and_rules_part3::rewrite_family_operator_forms_are_inert_and_apply_to_data ... ok
test runtime::tests::definitions_and_rules_part3::rewrite_options_report_new_defaults ... ok
test runtime::tests::definitions_and_rules_part4::function_accepts_explicit_attributes_for_application ... ok
test runtime::tests::definitions_and_rules_patterns::alternatives_downvalue_matches_each_branch ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_binds_multiple_named_args ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_falls_back_for_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_falls_back_for_sequence_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_handles_named_typed_blank_hit ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_keeps_typed_blank_miss_unevaluated ... ok
test runtime::tests::definitions_and_rules_patterns::compiled_downvalue_matcher_preserves_exact_vs_pattern_precedence ... ok
test runtime::tests::definitions_and_rules_patterns::conditioned_downvalue_matches_only_when_test_passes ... ok
test runtime::tests::definitions_and_rules_patterns::except_downvalue_prefilter_stays_conservative ... ok
test runtime::tests::definitions_and_rules_patterns::except_optional_and_repeated_downvalues_match_expected_shapes ... ok
test runtime::tests::definitions_and_rules_patterns::longest_and_shortest_control_sequence_pattern_greediness ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_alternatives_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_condition_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_except_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_optional_and_repeated_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_honors_patterntest_patterns ... ok
test runtime::tests::definitions_and_rules_patterns::matchq_rejects_malformed_repetition_bounds_with_head_specific_messages ... ok
test runtime::tests::definitions_and_rules_patterns::orderless_heads_backtrack_across_exact_and_patternsequence_groups ... ok
test runtime::tests::definitions_and_rules_patterns::orderless_sequence_matching_still_accepts_small_normal_cases ... ok
test runtime::tests::definitions_and_rules_patterns::orderlesspatternsequence_uses_orderless_group_matching_inside_orderless_heads ... ok
test runtime::tests::definitions_and_rules_patterns::outer_conditioned_downvalue_is_stored_and_applied ... ok
test runtime::tests::definitions_and_rules_patterns::patternsequence_flattens_argument_patterns_in_matching_rules_and_definitions ... ok
test runtime::tests::definitions_and_rules_patterns::patterntest_downvalue_matches_only_when_test_passes ... ok
test runtime::tests::definitions_and_rules_patterns::rule_lhs_evaluates_values_without_evaluating_pattern_names ... ok
test runtime::tests::definitions_and_rules_patterns::verbatim_matches_literal_pattern_language_forms ... ok
test runtime::tests::definitions_and_rules_patterns::wide_downvalue_store_index_preserves_specificity_and_alpha_replacement ... ok
test runtime::tests::definitions_and_rules_patterns::wide_ground_downvalue_index_preserves_pattern_priority_fallback_and_invalidation ... ok
test runtime::tests::definitions_and_rules_patterns::wide_ground_downvalue_scan_falls_back_to_live_order_after_matcher_mutation ... ok
test runtime::tests::definitions_and_rules_patterns::wide_pattern_downvalue_store_index_replaces_renamed_condition_pattern ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::inspect_rewrite_rules_can_suspend_and_resume_with_host_input ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::inspect_rewrite_rules_reports_normalized_rules_and_delayed_flags ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::normalize_and_inspect_rewrite_rules_rebuild_rhs_guards ... ok
test runtime::tests::definitions_and_rules_rewrite_inspect::normalize_rules_lists_single_rules_rule_lists_and_preserves_delayedness ... ok
test runtime::tests::definitions_and_rules_tags::clear_name_patterns_expand_before_mutation_and_preserve_exact_targets ... ok
test runtime::tests::definitions_and_rules_tags::clearall_and_remove_expand_name_patterns ... ok
test runtime::tests::definitions_and_rules_tags::downvalues_lists_stored_pattern_definitions_in_order ... ok
test runtime::tests::definitions_and_rules_tags::exact_downvalue_dispatch_preserves_ground_and_pattern_semantics ... ok
test runtime::tests::definitions_and_rules_tags::explicit_tagunset_removes_tagged_subvalue_definitions ... ok
test runtime::tests::definitions_and_rules_tags::literal_downvalue_is_prioritized_over_existing_typed_downvalue ... ok
test runtime::tests::definitions_and_rules_tags::redefining_same_downvalue_replaces_the_older_rule_in_place ... ok
test runtime::tests::definitions_and_rules_tags::redefining_same_subvalue_replaces_the_older_rule_in_place ... ok
test runtime::tests::definitions_and_rules_tags::redefining_same_upvalue_replaces_the_older_rule_in_place ... ok
test runtime::tests::definitions_and_rules_tags::string_targets_flow_through_clear_tag_and_unset_normalization ... ok
test runtime::tests::definitions_and_rules_tags::subvalues_list_and_apply_nested_head_definitions ... ok
test runtime::tests::definitions_and_rules_tags::tag_definition_messages_distinguish_missing_tags_from_invalid_positions ... ok
test runtime::tests::definitions_and_rules_tags::tagged_subvalues_attach_to_the_head_symbol_not_to_upvalues ... ok
test runtime::tests::definitions_and_rules_tags::tagset_immediate_captures_rhs_before_later_changes ... ok
test runtime::tests::definitions_and_rules_tags::tagset_preserves_tag_ownership_for_upvalues ... ok
test runtime::tests::definitions_and_rules_tags::typed_subvalues_are_prioritized_over_earlier_generic_subvalues ... ok
test runtime::tests::definitions_and_rules_tags::typed_upvalues_are_prioritized_over_earlier_generic_upvalues ... ok
test runtime::tests::definitions_and_rules_tags::unset_can_remove_symbol_downvalue_and_upvalue_definitions ... ok
test runtime::tests::definitions_and_rules_tags::upset_emits_nosym_when_no_valid_argument_tag_symbol_exists ... ok
test runtime::tests::definitions_and_rules_tags::upset_stores_immediate_upvalues_on_inferred_argument_heads ... ok
test runtime::tests::definitions_and_rules_tags::upsetdelayed_can_store_on_multiple_inferred_tag_symbols ... ok
test runtime::tests::definitions_and_rules_tags::upvalue_candidate_filter_preserves_matching_order_for_valid_tags ... ok
test runtime::tests::definitions_and_rules_tags::upvalue_dispatch_ignores_tags_that_appear_only_below_direct_arguments ... ok
test runtime::tests::definitions_and_rules_tags::upvalues_lists_and_applies_tagged_definitions ... ok
test runtime::tests::definitions_and_rules_tags::wide_subvalue_store_index_preserves_specificity_replacement_and_invalidation ... ok
test runtime::tests::definitions_and_rules_tags::wide_upvalue_store_index_preserves_specificity_replacement_and_invalidation ... ok
test runtime::tests::definitions_rhs_guards::delayed_downvalue_rhs_guard_accepts_or_declines_without_messages ... ok
test runtime::tests::definitions_rhs_guards::downvalues_guard_reconstruction_round_trips_behavior ... ok
test runtime::tests::definitions_rhs_guards::every_non_true_guard_result_declines_to_the_next_candidate ... ok
test runtime::tests::definitions_rhs_guards::explicit_true_guard_is_distinct_from_an_unguarded_downvalue ... ok
test runtime::tests::definitions_rhs_guards::guard_errors_propagate_like_pattern_position_condition_errors ... ok
test runtime::tests::definitions_rhs_guards::guarded_and_unguarded_downvalues_coexist_in_installation_order ... ok
test runtime::tests::definitions_rhs_guards::immediate_downvalue_rhs_evaluates_completely_before_guard_splitting ... ok
test runtime::tests::definitions_rhs_guards::nested_rhs_guards_flatten_in_order_for_both_spellings ... ok
test runtime::tests::definitions_rhs_guards::rhs_guard_side_effects_run_before_the_body_is_selected ... ok
test runtime::tests::definitions_rhs_guards::rhs_guards_and_bodies_share_the_definition_option_scope ... ok
test runtime::tests::definitions_rhs_guards::same_alpha_canonical_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards::unset_lhs_removes_every_rhs_guard_variant ... ok
test runtime::tests::definitions_rhs_guards::wide_exact_dispatch_preserves_guard_variant_order ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::attributes_on_the_guarded_symbol_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::eval_file_rollback_preserves_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::interleaved_guarded_downvalue_definitions_survive_later_bumps ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::listable_guarded_downvalue_recompiles_and_enforces_its_guard ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::matcher_ownership_guard_is_enforced_after_attribute_revision_invalidation ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::remaining_attribute_revision_sources_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::repeated_attribute_bumps_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::unrelated_set_and_clear_attributes_preserve_every_guarded_lane ... ok
test runtime::tests::definitions_rhs_guards_attribute_revision::valueq_rollback_preserves_every_guarded_lane_and_post_trigger_value ... ok
test runtime::tests::definitions_rhs_guards_default_values::declining_default_guard_leaves_optional_matching_without_a_default ... ok
test runtime::tests::definitions_rhs_guards_default_values::defaultvalue_rhs_guards_accept_decline_and_coexist_in_order ... ok
test runtime::tests::definitions_rhs_guards_default_values::definition_inspection_rebuilds_all_remaining_definition_guard_forms ... ok
test runtime::tests::definitions_rhs_guards_default_values::every_non_true_defaultvalue_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_default_values::immediate_and_delayed_defaultvalue_heads_split_guards_at_the_right_time ... ok
test runtime::tests::definitions_rhs_guards_default_values::nested_defaultvalue_guards_run_left_to_right_and_second_decline_falls_back ... ok
test runtime::tests::definitions_rhs_guards_default_values::same_defaultvalue_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards_default_values::unset_clears_every_defaultvalue_guard_variant_and_table_roundtrip_restores_them ... ok
test runtime::tests::definitions_rhs_guards_own_values::a_later_guarded_ownvalue_replaces_the_single_existing_slot ... ok
test runtime::tests::definitions_rhs_guards_own_values::declining_ownvalue_guards_fall_through_inside_block_and_module ... ok
test runtime::tests::definitions_rhs_guards_own_values::immediate_and_delayed_ownvalue_rhs_guards_accept_or_decline ... ok
test runtime::tests::definitions_rhs_guards_own_values::non_true_ownvalue_guards_decline_and_valueq_reports_false ... ok
test runtime::tests::definitions_rhs_guards_own_values::ownvalue_guards_run_left_to_right_and_decline_before_the_body ... ok
test runtime::tests::definitions_rhs_guards_own_values::ownvalue_rhs_guard_resumes_without_replaying_the_guard ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::declining_guarded_rules_handoff_in_rule_list_order ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::every_non_true_rewrite_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::guarded_exact_rule_preserves_wide_plan_order_and_fallback_index ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::immediate_and_delayed_rewrite_rules_split_bound_guards ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::nested_rewrite_guards_run_left_to_right_and_second_declines ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::replace_list_guard_decline_keeps_later_binding_enumeration ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::replace_repeated_stops_when_the_only_guard_declines ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::rewrite_guard_side_effect_runs_once_per_candidate_tested ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::rewrite_rhs_guards_accept_or_decline_for_every_generic_consumer ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::rule_shaped_data_keeps_its_existing_condition_forms ... ok
test runtime::tests::definitions_rhs_guards_rewrite_rules::suspending_rewrite_guard_resumes_same_rule_and_position_without_replay ... ok
test runtime::tests::definitions_rhs_guards_sub_values::every_non_true_subvalue_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_sub_values::immediate_and_delayed_subvalue_heads_split_guards_at_the_right_time ... ok
test runtime::tests::definitions_rhs_guards_sub_values::nested_subvalue_guards_run_left_to_right_and_second_decline_falls_back ... ok
test runtime::tests::definitions_rhs_guards_sub_values::same_alpha_canonical_subvalue_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards_sub_values::subvalue_rhs_guards_accept_decline_and_coexist_in_order ... ok
test runtime::tests::definitions_rhs_guards_sub_values::subvalue_rhs_guards_share_the_guarded_body_option_scope ... ok
test runtime::tests::definitions_rhs_guards_sub_values::unset_clears_every_subvalue_guard_variant_and_table_roundtrip_restores_them ... ok
test runtime::tests::definitions_rhs_guards_sub_values::wide_subvalue_identity_index_keeps_guard_variants_distinct ... ok
test runtime::tests::definitions_rhs_guards_up_values::every_non_true_upvalue_guard_result_declines ... ok
test runtime::tests::definitions_rhs_guards_up_values::immediate_and_delayed_upvalue_heads_split_guards_at_the_right_time ... ok
test runtime::tests::definitions_rhs_guards_up_values::nested_upvalue_guards_run_left_to_right_and_second_decline_falls_back ... ok
test runtime::tests::definitions_rhs_guards_up_values::same_alpha_canonical_upvalue_lhs_and_guards_redefine_in_place ... ok
test runtime::tests::definitions_rhs_guards_up_values::tagunset_clears_every_upvalue_guard_variant_and_table_roundtrip_restores_them ... ok
test runtime::tests::definitions_rhs_guards_up_values::upvalue_rhs_guards_accept_decline_and_coexist_in_order ... ok
test runtime::tests::definitions_rhs_guards_up_values::upvalue_rhs_guards_share_the_guarded_body_option_scope ... ok
test runtime::tests::definitions_rhs_guards_up_values::wide_upvalue_identity_index_keeps_guard_variants_distinct ... ok
test runtime::tests::definitions_tagged_preparation::all_tagged_install_entry_points_store_the_prepared_identity_by_owner ... ok
test runtime::tests::definitions_tagged_preparation::tagged_preparation_preserves_patterns_all_holds_and_callable_heads ... ok
test runtime::tests::definitions_tagged_preparation::tagged_snapshot_identity_tracks_attribute_rebind_clear_and_owner_invalidation ... ok
test runtime::tests::definitions_tagged_preparation::upset_infers_tags_only_after_preparation_and_keeps_nosym ... ok
test runtime::tests::definitions_tagged_preparation::upset_multiple_owner_protection_is_preserved_after_canonicalization ... ok
test runtime::tests::definitions_unset_preparation::clear_removes_all_identities_and_reinstall_has_no_stale_unset_target ... ok
test runtime::tests::definitions_unset_preparation::unset_after_attribute_invalidation_addresses_only_the_new_identity ... ok
test runtime::tests::definitions_unset_preparation::unset_after_own_value_rebinding_and_clear_preserves_the_stored_snapshot ... ok
test runtime::tests::definitions_unset_preparation::unset_and_postfix_unset_prepare_active_definition_identities ... ok
test runtime::tests::definitions_unset_preparation::unset_prepares_held_and_pattern_lhs_by_construction ... ok
test runtime::tests::definitions_unset_preparation::unset_snapshot_identity_removes_without_rebinding_but_not_after_rebinding ... ok
test runtime::tests::definitions_value_table_assignment::cross_symbol_downvalues_copy_fails_closed_on_head_mismatch ... ok
test runtime::tests::definitions_value_table_assignment::cross_symbol_downvalues_copy_works_after_target_substitution ... ok
test runtime::tests::definitions_value_table_assignment::downvalue_table_assignment_prepares_active_held_and_pattern_lhs ... ok
test runtime::tests::definitions_value_table_assignment::downvalues_identity_round_trip_restores_definition_behavior ... ok
test runtime::tests::definitions_value_table_assignment::later_mismatched_rule_cannot_partially_replace_a_value_table ... ok
test runtime::tests::definitions_value_table_assignment::malformed_value_table_rhs_leaves_the_existing_table_unchanged ... ok
test runtime::tests::definitions_value_table_assignment::ownvalues_identity_round_trip_restores_the_owned_value ... ok
test runtime::tests::definitions_value_table_assignment::prepared_value_table_identity_is_a_snapshot_across_rebind_clear_and_reinstall ... ok
test runtime::tests::definitions_value_table_assignment::protected_value_table_target_rejects_replacement ... ok
test runtime::tests::definitions_value_table_assignment::protection_is_rechecked_after_value_table_lhs_preparation ... ok
test runtime::tests::definitions_value_table_assignment::subvalue_table_assignment_prepares_active_held_and_pattern_lhs ... ok
test runtime::tests::definitions_value_table_assignment::successful_value_table_assignment_replaces_instead_of_merging ... ok
test runtime::tests::definitions_value_table_assignment::upvalue_table_assignment_prepares_active_held_and_pattern_lhs ... ok
test runtime::tests::definitions_value_table_assignment::upvalues_and_subvalues_identity_round_trips_restore_behavior ... ok
test runtime::tests::definitions_value_table_assignment::value_table_assignment_accepts_immediate_delayed_and_unwrapped_rules ... ok
test runtime::tests::file_names::file_names_depth_controls_nested_provider_enumeration ... ok
test runtime::tests::file_names::file_names_guard_precedes_the_provider_check ... ok
test runtime::tests::file_names::file_names_infinity_stops_at_the_fixed_entry_cap_without_recursing_the_stack ... ok
test runtime::tests::file_names::file_names_malformed_specs_remain_inert ... ok
test runtime::tests::file_names::file_names_matches_immediate_entries_and_sorts_and_deduplicates_paths ... ok
test runtime::tests::frame_root_trace::derived_frame_root_trace_inventory_matches_the_closed_capture_graph ... ok
test runtime::tests::frame_root_trace::manual_frame_root_trace_implementations_match_sorted_allowlist ... ok
test runtime::tests::frame_root_trace::root_visitation::association_entry_form_visits_key_and_value_roots_in_field_order ... ok
test runtime::tests::frame_root_trace::root_visitation::block_symbol_state_map_visits_the_bare_symbol_key ... ok
test runtime::tests::frame_root_trace::root_visitation::container_implementations_visit_keys_values_and_elements ... ok
test runtime::tests::frame_root_trace::root_visitation::enum_shapes_and_generic_arrays_trace_without_special_cases ... ok
test runtime::tests::frame_root_trace::root_visitation::expr_is_one_whole_root_and_bare_symbol_is_an_independent_root ... ok
test runtime::tests::frame_root_trace::root_visitation::file_io_error_is_the_zero_root_negative_calibration_target ... file_io_error_expr_roots=0
ok
test runtime::tests::frame_root_trace::root_visitation::matcher_binding_table_visits_the_exact_symbol_and_expression_roots ... ok
test runtime::tests::frame_root_trace::root_visitation::nested_derived_fields_are_all_traversed ... ok
test runtime::tests::frame_root_trace::root_visitation::recursive_matcher_arg_visits_every_nested_root_exactly_once ... ok
test runtime::tests::frame_root_trace::root_visitation::shared_expression_types_borrow_their_expression_roots ... ok
test runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold ... named_suspension_frames=555
ok
test runtime::tests::holding_and_control_io::input_print_and_pause_cover_current_first_pass_io_subset ... ok
test runtime::tests::holding_and_control_io::pause_rejects_unrepresentable_integer_and_rational_durations ... ok
test runtime::tests::holding_and_control_part2::binary_function_syntax_applies_explicit_parameter ... ok
test runtime::tests::holding_and_control_part2::bounded_recursive_definition_below_native_stack_limit_succeeds ... ok
test runtime::tests::holding_and_control_part2::cycle_detection_returns_iteration_limit_for_mutual_downvalues ... ok
test runtime::tests::holding_and_control_part2::cycle_detection_returns_iteration_limit_for_mutual_ownvalues_and_recovers ... ok
test runtime::tests::holding_and_control_part2::dispatch_plan_uses_builtin_metadata_and_head_traits ... ok
test runtime::tests::holding_and_control_part2::evaluation_pass_limit_bounds_fixed_point_revisits ... ok
test runtime::tests::holding_and_control_part2::explicit_parameter_functions_ignore_extra_arguments ... ok
test runtime::tests::holding_and_control_part2::explicit_parameter_functions_respect_shadowing_and_pattern_locals ... ok
test runtime::tests::holding_and_control_part2::fixed_point_evaluation_revisits_expressions_after_side_effects_change_head_meaning ... ok
test runtime::tests::holding_and_control_part2::function_applies_slots_positionally ... ok
test runtime::tests::holding_and_control_part2::head_evaluation_can_activate_special_forms_and_held_builtins ... ok
test runtime::tests::holding_and_control_part2::native_stack_limit_stops_unbounded_definition_recursion_and_session_stays_usable ... ok
test runtime::tests::holding_and_control_part2::pure_function_syntax_uses_slots_and_evaluated_arguments ... ok
test runtime::tests::holding_and_control_part2::recursion_limit_stops_expanding_recursive_downvalues ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_applies_at_every_application_level ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_attribute_round_trips ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_flattens_sequence_like_hold_all ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_holds_outer_arguments_and_evaluate_pierces ... ok
test runtime::tests::holding_and_control_part2::subvalues_hold_all_strips_and_restores_unevaluated ... ok
test runtime::tests::holding_and_control_part2::sum_and_product_malformed_iterators_emit_argform_messages ... ok
test runtime::tests::holding_and_control_part2::sum_and_product_valid_iterators_remain_message_free ... ok
test runtime::tests::holding_and_control_part2::tracing_records_definition_rule_and_evaluation_events ... ok
test runtime::tests::hygiene::kernel_crates_do_not_use_std_fs_outside_tests ... ok
test runtime::tests::hygiene::kernel_crates_do_not_write_stdout_outside_tests_and_bins ... ok
test runtime::tests::iterative_loop_drivers::for_drives_one_hundred_thousand_iterations_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::for_resumes_mid_loop_without_replaying_the_body ... ok
test runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_frozen_prefix_and_unique_tail_values ... ok
test runtime::tests::iterative_loop_drivers::iteration_accumulator_preserves_unsuspended_values ... ok
test runtime::tests::iterative_loop_drivers::module_drives_thirty_thousand_initialized_bindings_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::module_resumes_mid_initializer_without_replaying_bindings ... ok
test runtime::tests::iterative_loop_drivers::product_iteration_accumulator_completes_after_multiple_input_suspensions ... ok
test runtime::tests::iterative_loop_drivers::sum_iteration_accumulator_completes_after_multiple_input_suspensions ... ok
test runtime::tests::iterative_loop_drivers::table_iteration_accumulator_completes_after_multiple_input_suspensions ... ok
test runtime::tests::iterative_loop_drivers::unbounded_while_reaches_the_existing_eval_deadline ... ok
test runtime::tests::iterative_loop_drivers::until_drives_one_hundred_thousand_iterations_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::until_resumes_mid_loop_without_replaying_the_body ... ok
test runtime::tests::iterative_loop_drivers::while_drives_one_hundred_thousand_iterations_without_native_stack_growth ... ok
test runtime::tests::iterative_loop_drivers::while_resumes_mid_loop_without_replaying_the_body ... ok
test runtime::tests::left_chain_lowering::flat_chain_evaluated_results_remain_unchanged ... ok
test runtime::tests::left_chain_lowering::held_sameq_explicitly_observes_flat_chain_shapes ... ok
test runtime::tests::left_chain_lowering::held_sameq_explicitly_observes_variadic_non_flat_chain_shapes ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_accepts_a_boundary_hand_built_alternating_ast ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_accepts_the_boundary_for_all_operators ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_keeps_direct_canonical_calls_opaque ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_preserves_cross_family_shape_and_resolver_order ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_preserves_message_name_double_resolution_order ... ok
test runtime::tests::left_chain_lowering::left_chain_lowering_rejects_a_20_000_link_hand_built_ast_gracefully ... ok
test runtime::tests::left_chain_lowering::left_chain_parser_rejects_20_000_links_gracefully_for_all_operators ... ok
test runtime::tests::left_chain_lowering::left_chain_two_mib_full_round_trip_accepts_and_rejects_without_abort ... ignored, release-only deployment-stack probe; run explicitly with --release
test runtime::tests::left_chain_lowering::left_chain_two_mib_nested_expr_drops_naturally_at_the_boundary ... ignored, release-only deployment-stack probe; run explicitly with --release
test runtime::tests::left_chain_lowering::variadic_non_flat_chain_matching_shapes_are_explicit ... ok
test runtime::tests::lowering_and_context::attributes_reports_builtin_and_user_defined_attribute_sets ... ok
test runtime::tests::lowering_and_context::compound_expression_sugar_flows_through_lowering_and_evaluation ... ok
test runtime::tests::lowering_and_context::evaluates_identity_call ... ok
test runtime::tests::lowering_and_context::long_statement_chain_lowers_to_flat_nary_compound_expression ... ok
test runtime::tests::lowering_and_context::lowering_preserves_tagset_shape_for_runtime_semantic_validation ... ok
test runtime::tests::lowering_and_context::lowering_rejects_invalid_integer_literals_before_runtime ... ok
test runtime::tests::lowering_and_context::lowers_big_integer_into_core_expr ... ok
test runtime::tests::lowering_and_context::lowers_message_name_syntax_into_core_expr ... ok
test runtime::tests::lowering_and_context::lowers_multiline_string_literals_like_ordinary_strings ... ok
test runtime::tests::lowering_and_context::lowers_simple_decimal_literals_to_exact_rationals ... ok
test runtime::tests::lowering_and_context::lowers_tagset_and_unset_syntax_into_core_expr ... ok
test runtime::tests::lowering_and_context::over_cap_statement_chain_line_does_not_swallow_later_lines ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_flat_arithmetic_family_transitions ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_messages_and_context_heads ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_rules_patterns_slots_and_scoping ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_tagged_definition_sugar ... ok
test runtime::tests::lowering_golden::lowering_golden_forms_cover_three_link_left_chains ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_argument_finisher_collapses_equal_changed_prefix ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_bounded_front_handles_exhaustion_locally ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_changed_arg_boundary_keeps_unchanged_child_arcs ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_corpus_matches_frozen_reference ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_definition_identity_hold_gate_and_four_arms ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_nested_flat_is_bottom_up_and_fully_flattened ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_rebuild_arms_preserve_root_arc_origins ... ok
test runtime::tests::match_normalization_contract::match_normalization_contract_uses_normalized_function_head_attributes ... ok
test runtime::tests::memory_growth_contract::core_builtin_memory_growth_inventory_is_sorted_and_exhaustive ... ok
test runtime::tests::memory_growth_contract::known_unprojected_preflight_builtin_allowlist_is_sorted_and_unique ... ok
test runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_after_input_resume - should panic ... 
ok
test runtime::tests::memory_growth_contract::non_amplifying_contract_is_falsifiable_through_dispatch - should panic ... 
ok
test runtime::tests::memory_growth_contract::preflight_contract_accepts_recorded_projection ... ok
test runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_after_input_resume - should panic ... 
ok
test runtime::tests::memory_growth_contract::preflight_contract_is_falsifiable_through_dispatch - should panic ... 
ok
test runtime::tests::memory_growth_contract::preflight_contract_remembers_projection_across_input_resume ... ok
test runtime::tests::messages::builtin_message_registry_populates_listings_and_information ... ok
test runtime::tests::messages::delayed_message_templates_are_evaluated_when_emitted ... ok
test runtime::tests::messages::direct_set_and_setdelayed_calls_share_message_assignment_normalization ... ok
test runtime::tests::messages::inert_message_carriers_list_seeded_tags_and_reject_assignment ... ok
test runtime::tests::messages::inspect_message_catalog_exposes_templates_and_delayed_flags ... ok
test runtime::tests::messages::inspect_message_catalog_merges_builtin_templates_and_user_overrides ... ok
test runtime::tests::messages::inspect_reports_message_definition_presence ... ok
test runtime::tests::messages::invalid_builtin_argument_shapes_emit_runtime_messages ... ok
test runtime::tests::messages::invalid_message_name_form_emits_runtime_message ... ok
test runtime::tests::messages::message_definitions_builtin_lists_immediate_and_delayed_templates ... ok
test runtime::tests::messages::message_family_accepts_symbol_name_strings_and_messagename_targets ... ok
test runtime::tests::messages::message_list_reports_messages_emitted_so_far_in_current_input ... ok
test runtime::tests::messages::message_name_and_message_emit_runtime_messages ... ok
test runtime::tests::messages::message_tags_builtin_lists_global_symbol_and_exact_message_tags ... ok
test runtime::tests::messages::message_templates_can_be_defined_and_formatted ... ok
test runtime::tests::messages::message_templates_leave_unknown_placeholders_unchanged ... ok
test runtime::tests::messages::message_templates_leave_unsupported_format_specs_unchanged ... ok
test runtime::tests::messages::message_templates_support_message_name_and_all_args_placeholders ... ok
test runtime::tests::messages::message_templates_support_richer_format_specifiers ... ok
test runtime::tests::messages::messages_builtin_lists_defined_message_names ... ok
test runtime::tests::messages::push_message_allows_open_user_symbol_families ... ok
test runtime::tests::messages::push_message_allows_test_local_builtins_without_declared_messages ... ok
test runtime::tests::messages::push_message_allows_user_defined_builtin_message_names ... ok
test runtime::tests::messages::push_message_rejects_unregistered_builtin_message_tags - should panic ... 
ok
test runtime::tests::messages::quiet_keeps_suppressed_messages_out_of_message_list ... ok
test runtime::tests::messages::request_message_list_reports_messages_emitted_so_far_in_current_input ... ok
test runtime::tests::messages::upto_stays_inert_and_selector_diagnostic_stays_exact ... ok
test runtime::tests::messages::usage_message_assignment_holds_symbol_with_existing_own_value ... ok
test runtime::tests::messages_part2::check_and_quiet_accept_message_name_selector_forms ... ok
test runtime::tests::messages_part2::check_can_catch_non_loader_evaluation_errors_via_general_messages ... ok
test runtime::tests::messages_part2::check_can_match_by_message_symbol_only ... ok
test runtime::tests::messages_part2::check_can_match_specific_general_error_selectors ... ok
test runtime::tests::messages_part2::check_can_target_specific_message_selectors ... ok
test runtime::tests::messages_part2::check_leaves_value_unchanged_when_no_messages_are_emitted ... ok
test runtime::tests::messages_part2::check_returns_fallback_when_messages_are_emitted ... ok
test runtime::tests::messages_part2::cycle_termination_emits_pass_limit_message_at_top_level ... ok
test runtime::tests::messages_part2::explicit_parameter_function_arity_mismatch_emits_runtime_message ... ok
test runtime::tests::messages_part2::invalid_builtin_argument_counts_emit_runtime_messages ... ok
test runtime::tests::messages_part2::invalid_message_selector_specs_emit_runtime_messages ... ok
test runtime::tests::messages_part2::quiet_accepts_selector_lists ... ok
test runtime::tests::messages_part2::quiet_can_selectively_suppress_message_streams ... ok
test runtime::tests::messages_part2::quiet_prevents_check_from_observing_suppressed_messages ... ok
test runtime::tests::messages_part2::quiet_suppresses_cycle_termination_messages_without_masking_value ... ok
test runtime::tests::messages_part2::quiet_suppresses_runtime_messages_from_nested_evaluation ... ok
test runtime::tests::messages_part2::unbalanced_context_frames_emit_runtime_messages ... ok
test runtime::tests::messages_part3::beginpackage_accepts_resolved_dependency_lists ... ok
test runtime::tests::messages_part3::beginpackage_can_load_dependencies_via_needs ... ok
test runtime::tests::messages_part3::check_can_catch_iteration_limit_messages_from_replace_repeated ... ok
test runtime::tests::messages_part3::explicit_non_system_heads_do_not_trigger_builtin_dispatch ... ok
test runtime::tests::messages_part3::hold_family_respects_builtin_hold_attributes ... ok
test runtime::tests::messages_part3::path_builtin_reflects_package_search_roots ... ok
test runtime::tests::mutation_operator_forms::append_to_and_add_to_direct_calls_match_their_operator_forms ... ok
test runtime::tests::mutation_operator_forms::append_to_association_key_part_stays_on_generic_path ... ok
test runtime::tests::mutation_operator_forms::append_to_capture_defers_its_value_until_application ... ok
test runtime::tests::mutation_operator_forms::append_to_complex_part_selectors_keep_generic_behavior ... ok
test runtime::tests::mutation_operator_forms::append_to_deep_exact_part_preserves_middle_alias ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_copies_list_shared_at_two_positions ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_keeps_nothing_and_upvalue_behavior ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_non_list_leaf_still_runs_definitions ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_preserves_inner_alias ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_preserves_outer_alias ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_protected_owner_still_refuses ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_rolls_back_multilevel_spine ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_with_monitor_preserves_spine_observations ... ok
test runtime::tests::mutation_operator_forms::append_to_exact_part_with_tracing_preserves_existing_trace ... ok
test runtime::tests::mutation_operator_forms::append_to_item_that_requires_target_evaluation_stays_on_evaluated_path ... ok
test runtime::tests::mutation_operator_forms::append_to_list_downvalue_still_observes_the_old_value_and_fires ... ok
test runtime::tests::mutation_operator_forms::append_to_list_upvalue_still_observes_the_old_value_and_fires ... ok
test runtime::tests::mutation_operator_forms::append_to_modified_list_attributes_stays_on_evaluated_path ... ok
test runtime::tests::mutation_operator_forms::append_to_non_list_head_still_runs_matching_user_code ... ok
test runtime::tests::mutation_operator_forms::append_to_nothing_keeps_list_normalization_semantics ... ok
test runtime::tests::mutation_operator_forms::append_to_plain_list_preserves_alias_and_returned_value ... ok
test runtime::tests::mutation_operator_forms::append_to_protected_list_target_preserves_value_and_message ... ok
test runtime::tests::mutation_operator_forms::append_to_resumes_item_input_without_losing_or_replaying_the_update ... ok
test runtime::tests::mutation_operator_forms::append_to_rolls_back_if_fast_update_fails_after_definition_take ... ok
test runtime::tests::mutation_operator_forms::append_to_with_monitor_preserves_old_and_new_display_observations ... ok
test runtime::tests::mutation_operator_forms::append_to_with_tracing_preserves_the_existing_trace ... ok
test runtime::tests::mutation_operator_forms::kernel_mutation_heads_advertise_subvalues_hold_all ... ok
test runtime::tests::mutation_operator_forms::kernel_mutation_operator_forms_apply_to_raw_held_targets ... ok
test runtime::tests::mutation_operator_forms::mutation_captures_are_quiet_but_invalid_applications_report_argx ... ok
test runtime::tests::mutation_operator_forms::mutation_operator_form_preserves_direct_protection_failure ... ok
test runtime::tests::namespace_names::names_accepts_current_options_subset ... ok
test runtime::tests::namespace_names::names_rejects_unsupported_resolve_context_alias_values ... ok
test runtime::tests::namespace_names::names_rejects_unsupported_spelling_correction_values ... ok
test runtime::tests::namespace_names::names_supports_current_first_pass_string_patterns ... ok
test runtime::tests::namespace_names::names_without_explicit_context_only_sees_visible_contexts ... ok
test runtime::tests::namespace_names::symbol_name_and_name_q_core_anchors ... ok
test runtime::tests::native_stack_guard::logical_depth_never_refuses_when_the_byte_budget_is_available ... ok
test runtime::tests::native_stack_guard::native_stack_guard_polls_at_the_stride_and_fails_sticky ... ok
test runtime::tests::nothing::nothing_removal_happens_after_sequence_splicing_for_lists_only ... ok
test runtime::tests::nothing::nothing_resolves_as_system_symbol ... ok
test runtime::tests::options_and_protection::argumentsoptions_and_checkarguments_cover_current_first_pass_subset ... ok
test runtime::tests::options_and_protection::block_localizes_options_alongside_other_symbol_state ... ok
test runtime::tests::options_and_protection::codec_and_schema_options_are_discoverable ... ok
test runtime::tests::options_and_protection::delayed_option_rules_are_preserved_and_evaluated_on_lookup ... ok
test runtime::tests::options_and_protection::options_assignment_and_queries_cover_current_first_pass_subset ... ok
test runtime::tests::options_and_protection::options_assignment_unset_and_clearall_interact_with_symbol_state ... ok
test runtime::tests::options_and_protection::optionspattern_and_one_arg_optionvalue_cover_current_first_pass_subset ... ok
test runtime::tests::options_and_protection::optionspattern_can_use_an_explicit_target_symbol ... ok
test runtime::tests::options_and_protection::optionvalue_accepts_lists_of_option_names ... ok
test runtime::tests::options_and_protection::optionvalue_emits_current_optnf_and_nodef_messages ... ok
test runtime::tests::options_and_protection::protect_and_unprotect_expand_name_patterns_without_literal_wildcards ... ok
test runtime::tests::options_and_protection::protect_blocks_direct_options_assignment_and_unset_until_unprotected ... ok
test runtime::tests::options_and_protection::protected_attribute_removal_and_unprotect_remain_escape_hatches ... ok
test runtime::tests::options_and_protection::protected_targets_reject_attribute_updates_and_preserve_attributes ... ok
test runtime::tests::options_and_protection::setoptions_accepts_target_lists_and_returns_per_target_results ... ok
test runtime::tests::options_and_protection::setoptions_and_optionvalue_match_known_system_options_by_name ... ok
test runtime::tests::options_and_protection::setoptions_blocks_protected_system_head_and_unprotect_allows_update ... ok
test runtime::tests::options_and_protection::setoptions_updates_known_options_but_rejects_unknown_ones ... ok
test runtime::tests::options_and_protection::test_create_options_are_discoverable_and_match_consumed_defaults ... ok
test runtime::tests::package_tools::declare_package_declares_names_and_updates_context_path ... ok
test runtime::tests::package_tools::declared_package_stub_autoloads_when_symbol_is_used ... ok
test runtime::tests::package_tools::definition_snapshot_rebuilds_all_rhs_guards_for_roundtrip ... ok
test runtime::tests::package_tools::dump_save_context_strings_skip_module_gensyms_but_keep_dollar_globals ... ok
test runtime::tests::package_tools::dump_save_overwrites_file_and_returns_saved_symbols ... ok
test runtime::tests::package_tools::dump_save_rejects_unsupported_compression_levels_and_preserves_default_bytes ... ok
test runtime::tests::package_tools::dump_save_returns_qualified_symbol_names_without_reevaluating_values ... ok
test runtime::tests::package_tools::once_accepts_option_rules_in_second_position_and_respects_zero_persistence_subset ... ok
test runtime::tests::package_tools::once_caches_per_session_and_per_location_key ... ok
test runtime::tests::package_tools::once_rejects_unimplemented_expiration_policies_without_evaluating_body ... ok
test runtime::tests::package_tools::save_and_dumpsave_accept_names_pattern_objects ... ok
test runtime::tests::package_tools::save_and_dumpsave_honor_current_context_filter_and_attribute_options_subset ... ok
test runtime::tests::package_tools::save_rebuilds_downvalue_rhs_guards_for_roundtrip ... ok
test runtime::tests::package_tools::save_rejects_unknown_options_and_once_reports_unknown_option_names ... ok
test runtime::tests::package_tools::save_writes_textual_state_that_get_can_roundtrip ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_a_root_alias ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_a_shared_child_at_a_second_position ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_a_three_level_middle_alias ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_an_intermediate_list_alias ... ok
test runtime::tests::part_assignment::set_positional_part_cow_preserves_the_selected_child_alias ... ok
test runtime::tests::part_assignment::set_positional_part_direct_path_keeps_spine_identity_and_leaves_proofs_evicted ... ok
test runtime::tests::part_assignment::set_positional_part_direct_path_polls_interrupt_before_cache_eviction_or_mutation ... ok
test runtime::tests::part_assignment::set_positional_part_direct_path_rolls_back_slot_and_definition_after_replacement ... ok
test runtime::tests::part_assignment::set_positional_part_invalid_targets_report_without_protected_part_fallback ... ok
test runtime::tests::part_assignment::set_positional_part_observable_constrained_and_dynamic_scopes_use_generic_path ... ok
test runtime::tests::part_assignment::set_positional_part_rechecks_protection_after_rhs_and_selector_callbacks ... ok
test runtime::tests::part_assignment::set_positional_part_rederives_after_rhs_and_orders_selectors_left_to_right ... ok
test runtime::tests::part_assignment::set_positional_part_refuses_a_target_cleared_by_a_selector ... ok
test runtime::tests::part_assignment::set_positional_part_route_preserves_other_lvalues_and_definition_lhs_forms ... ok
test runtime::tests::part_assignment::set_positional_part_selector_resume_is_single_pass_and_rederives_the_owner ... ok
test runtime::tests::part_assignment::set_positional_part_supports_flat_nested_and_value_semantics ... ok
test runtime::tests::pattern_matching_orderless_optional::flat_optional_omission_collects_every_match_for_builtin_heads ... ok
test runtime::tests::pattern_matching_orderless_optional::flat_orderless_optional_matches_collects_and_omits ... ok
test runtime::tests::pattern_matching_orderless_optional::flat_orderless_optional_with_sequence_neighbors ... ok
test runtime::tests::pattern_matching_orderless_optional::installed_system_down_values_surface_as_a_whole_after_user_mutation ... ok
test runtime::tests::pattern_matching_orderless_optional::one_identity_promotes_singletons_when_remaining_arguments_are_omissible ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_dot_uses_head_default ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_nonmatch_edges_stay_false ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_omission_uses_explicit_defaults ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_present_matches_and_binds_in_documented_order ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_with_other_args_matches_present_and_omitted ... ok
test runtime::tests::pattern_matching_orderless_optional::orderless_optional_with_sequence_neighbors_uses_documented_backtracking_order ... ok
test runtime::tests::pattern_matching_orderless_optional::snapshot_after_unset_resurrects_an_installed_system_down_value ... ok
test runtime::tests::reap::nested_reap_scopes_keep_their_own_buckets ... ok
test runtime::tests::reap::reap_buckets_stay_in_first_appearance_order ... ok
test runtime::tests::reap::reap_index_preserves_wide_order_and_appends_to_existing_buckets ... ok
test runtime::tests::reap::reap_patterns_select_which_sows_are_captured ... ok
test runtime::tests::registered_builtin_resolution::registered_builtin_absent_from_static_catalog_resolves_as_system_symbol ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_masks_only_held_argument_positions ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_operator_form_applies_to_multiple_inputs ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_rule_shape_messages_match_replace_all ... ok
test runtime::tests::replace_all_unheld::replace_all_unheld_uses_original_and_compound_head_attributes ... ok
test runtime::tests::rule_rhs_materialization::held_delayed_rhs_firing_is_consumer_independent ... ok
test runtime::tests::rule_rhs_materialization::literal_sequence_splicing_is_deferred_until_replacement_result_evaluation ... ok
test runtime::tests::rule_rhs_materialization::literal_sequence_wrapping_a_sequence_binding_keeps_its_literal_head ... ok
test runtime::tests::rule_rhs_materialization::other_rewrite_traversals_share_replacement_sequence_splicing ... ok
test runtime::tests::rule_rhs_materialization::replaceat_immediate_rhs_still_evaluates_at_rule_construction ... ok
test runtime::tests::rule_rhs_materialization::replaceat_inserts_delayed_rhs_raw_before_result_evaluation ... ok
test runtime::tests::rule_rhs_materialization::replacement_produced_sequence_splices_into_held_parent_rebuild ... ok
test runtime::tests::rule_rhs_materialization::replacement_sequence_splice_respects_parent_attributes ... ok
test runtime::tests::rule_rhs_materialization::replacepart_immediate_rhs_still_evaluates_at_rule_construction ... ok
test runtime::tests::rule_rhs_materialization::replacepart_inserts_delayed_rhs_raw_before_result_evaluation ... ok
test runtime::tests::rule_rhs_materialization::sequence_pattern_bindings_keep_structural_splicing ... ok
test runtime::tests::rule_rhs_materialization::top_level_sequence_replacement_has_no_parent_to_splice_into ... ok
test runtime::tests::rule_rhs_materialization::unchanged_literal_sequence_does_not_splice_during_rebuild ... ok
test runtime::tests::scoping_gensym_gc::all_live_modules_run_one_sweep_before_the_next_watermark ... ok
test runtime::tests::scoping_gensym_gc::clear_all_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::scoping_gensym_gc::gensym_gc_threshold_parses_env_in_fresh_processes ... ok
test runtime::tests::scoping_gensym_gc::gensym_gc_watermark_arithmetic_saturates ... ok
test runtime::tests::scoping_gensym_gc::immediate_module_reclaim_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::scoping_gensym_gc::loader_snapshot_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::scoping_gensym_gc::partial_sweep_rearms_from_the_post_sweep_candidate_count ... ok
test runtime::tests::scoping_gensym_gc::reachability::attribute_state_on_dead_candidate_is_cleared_during_reclaim ... ok
test runtime::tests::scoping_gensym_gc::reachability::delayed_own_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::down_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::downvalue_rhs_guard_keeps_its_escaped_module_binding_live ... ok
test runtime::tests::scoping_gensym_gc::reachability::empty_candidate_set_sweep_is_a_noop ... ok
test runtime::tests::scoping_gensym_gc::reachability::global_root_keeps_escaped_candidate_until_root_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::live_candidate_definition_marks_transitive_candidate_dependencies ... ok
test runtime::tests::scoping_gensym_gc::reachability::message_root_escape_keeps_candidate_until_message_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::module_error_path_definition_escape_stays_deferred ... ok
test runtime::tests::scoping_gensym_gc::reachability::module_result_root_retains_only_its_transitive_local_chain ... ok
test runtime::tests::scoping_gensym_gc::reachability::module_throw_payload_root_retains_its_gensym_definition ... ok
test runtime::tests::scoping_gensym_gc::reachability::module_unreferenced_error_paths_reclaim_gensyms_immediately ... ok
test runtime::tests::scoping_gensym_gc::reachability::nested_module_activations_keep_their_candidates_until_each_activation_is_removed ... ok
test runtime::tests::scoping_gensym_gc::reachability::nested_module_error_reclamation_mirrors_success_conservatism ... ok
test runtime::tests::scoping_gensym_gc::reachability::option_root_escape_keeps_candidate_until_options_are_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::outcome_extra_roots_are_empty_for_non_value_errors_and_candidate_is_reclaimed ... ok
test runtime::tests::scoping_gensym_gc::reachability::outcome_extra_roots_keep_success_return_throw_and_cleanup_candidates_live ... ok
test runtime::tests::scoping_gensym_gc::reachability::own_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::recursive_module_sweep_reclaims_dead_retained_gensyms ... ok
test runtime::tests::scoping_gensym_gc::reachability::returned_expr_extra_root_keeps_candidate_definition_live ... ok
test runtime::tests::scoping_gensym_gc::reachability::sub_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::subvalue_rhs_guard_keeps_its_escaped_module_binding_live ... ok
test runtime::tests::scoping_gensym_gc::reachability::suspended_computation_keeps_candidate_live_until_resume_finishes ... ok
test runtime::tests::scoping_gensym_gc::reachability::syntax_information_root_escape_keeps_candidate_until_syntax_info_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::transitive_definition_kind_chain_keeps_all_reachable_candidates_live ... ok
test runtime::tests::scoping_gensym_gc::reachability::unique_generated_symbols_are_not_module_gc_candidates ... ok
test runtime::tests::scoping_gensym_gc::reachability::up_value_escape_keeps_candidate_until_definition_is_cleared ... ok
test runtime::tests::scoping_gensym_gc::reachability::upvalue_rhs_guard_keeps_its_escaped_module_binding_live ... ok
test runtime::tests::scoping_gensym_gc::remove_candidate_shrink_rearms_the_watermark ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::block_frame_bare_symbol_is_reclaimed_after_resume_and_next_sweep ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::executing_frame_and_queued_suffix_remain_rooted_during_resume_sweep ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::large_frame_state_trips_memory_limit_and_session_recovers ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::matcher_guard_binding_survives_a_sweep_while_suspended ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::path_a_second_request_keeps_frame_only_candidate_live ... ok
test runtime::tests::scoping_gensym_gc::suspension_frames::path_b_resume_sweep_keeps_remaining_frame_candidate_live ... ok
test runtime::tests::scoping_gensym_gc::unreachable_candidates_are_reclaimed_at_the_next_watermark_boundary ... ok
test runtime::tests::scoping_unique::unique_generates_fresh_symbols_for_current_prefix_subset ... ok
test runtime::tests::scoping_unique::unique_registers_generated_symbols_as_known_names ... ok
test runtime::tests::scoping_unique::unique_reports_current_invalid_boundaries ... ok
test runtime::tests::scoping_unique::unique_skips_existing_generated_names ... ok
test runtime::tests::sealed_packages::context_validation_returns_the_call_inert ... ok
test runtime::tests::sealed_packages::declaration_validation ... ok
test runtime::tests::sealed_packages::decoy_assignment_is_distinct ... ok
test runtime::tests::sealed_packages::internal_snapshot_round_trips_registry ... ok
test runtime::tests::sealed_packages::package_install_resumes_at_the_next_form ... ok
test runtime::tests::sealed_packages::package_scope_is_fqn_reachable ... ok
test runtime::tests::sealed_packages::private_state_inside_package ... ok
test runtime::tests::sealed_packages::public_reaches_sealed_private ... ok
test runtime::tests::sealed_packages::re_registration_replaces_in_place ... ok
test runtime::tests::sealed_packages::registration_failure_cleans_sealed_state ... ok
test runtime::tests::sealed_packages::remove_sealed_writes_no_generation ... ok
test runtime::tests::sealed_packages::spelled_private_name_is_ordinary ... ok
test runtime::tests::sealed_snapshot::information_sealed_unit_row ... ok
test runtime::tests::sealed_snapshot::marker_refused_in_ordinary_evaluation ... ok
test runtime::tests::sealed_snapshot::save_refuses_sealed_references ... ok
test runtime::tests::sealed_snapshot::sealed_free_snapshot_bytes_unchanged ... ok
test runtime::tests::sealed_snapshot::snapshot_persists_public_mutation ... ok
test runtime::tests::sealed_snapshot::snapshot_refuses_orphans ... ok
test runtime::tests::sealed_snapshot::snapshot_roundtrip_relinks_leaked_values ... ok
test runtime::tests::sealed_snapshot::snapshot_roundtrip_restores_seal ... ok
test runtime::tests::sealed_visibility::completion_hides_sealed ... ok
test runtime::tests::sealed_visibility::completion_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::contexts_hides_private_context ... ok
test runtime::tests::sealed_visibility::contexts_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::decoy_visibility_is_callers_own ... ok
test runtime::tests::sealed_visibility::definition_hides_sealed_body ... ok
test runtime::tests::sealed_visibility::definition_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::information_hides_sealed ... ok
test runtime::tests::sealed_visibility::information_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::message_catalog_hides_sealed ... ok
test runtime::tests::sealed_visibility::message_catalog_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::nameq_stays_false ... ok
test runtime::tests::sealed_visibility::names_hides_sealed ... ok
test runtime::tests::sealed_visibility::names_reveals_with_diagnostic ... ok
test runtime::tests::sealed_visibility::nested_registration_keeps_registry_consistent ... ok
test runtime::tests::sealed_visibility::save_excludes_sealed ... ok
test runtime::tests::sealed_visibility::wildcard_ops_skip_sealed ... ok
test runtime::tests::session_config::apply_config_updates_host_settings_without_clearing_existing_definitions ... ok
test runtime::tests::session_config::default_native_stack_budget_is_derived_from_minimum_supported_stack ... ok
test runtime::tests::session_config::default_session_configuration_matches_default_session_state ... ok
test runtime::tests::session_config::random_u64_sequences_are_pinned_for_byte_and_integer_seeds ... ok
test runtime::tests::session_config::session_setters_update_limits_capabilities_asserts_and_context_lookup ... ok
test runtime::tests::session_config::session_with_config_normalizes_contexts_paths_and_limits ... ok
test runtime::tests::session_config::trace_message_and_print_accessors_expose_and_take_buffers ... ok
test runtime::tests::session_memory::arithmetic::field_and_session_sums_preserve_overflow_information ... ok
test runtime::tests::session_memory::arithmetic::overflow_saturates_and_invalid_releases_stay_closed ... ok
test runtime::tests::session_memory::arithmetic::synthetic_charge_release_sequences_never_understate_representable_remainders ... ok
test runtime::tests::session_memory::byte_array_proxy_tracks_encoded_string_capacity_and_preserves_invalid_fallback ... ok
test runtime::tests::session_memory::debug_owners::maintained_message_print_trace_capture_and_rollback ... ok
test runtime::tests::session_memory::debug_owners::maintained_reap_monitor_append_replace_resume_and_abandon ... ok
test runtime::tests::session_memory::maintained::maintained_attributes_and_caches_mutate_clear_and_restore ... ok
test runtime::tests::session_memory::maintained::maintained_builtins_and_limits_charge_replace_and_rearm ... ok
test runtime::tests::session_memory::maintained::maintained_contexts_paths_and_loader_rollback ... ok
test runtime::tests::session_memory::maintained::maintained_definitions_store_remove_and_lazy_dispatch ... ok
test runtime::tests::session_memory::maintained::maintained_evaluation_caches_charge_evict_and_keep_capacity ... ok
test runtime::tests::session_memory::maintained::maintained_options_and_syntax_replace_remove_and_rollback ... ok
test runtime::tests::session_memory::maintained::maintained_providers_and_print_sink_take_reinstall ... ok
test runtime::tests::session_memory::maintained::maintained_scopes_registry_sealed_units_and_abandonment ... ok
test runtime::tests::session_memory::maintained::maintained_stack_checkpoint_workloads ... ok
test runtime::tests::session_memory::maintained::maintained_stack_counts_shared_occurrences_and_retained_capacity ... ok
test runtime::tests::session_memory::maintained::maintained_stack_local_arm_survives_suspend_resume_abandon_and_errors ... ok
test runtime::tests::session_memory::maintained::maintained_stack_rearms_from_scratch_and_observation_keeps_walking ... ok
test runtime::tests::session_memory::maintained::maintained_stack_survives_panic_and_rollback ... 
ok
test runtime::tests::session_memory::maintained::maintained_stubs_generations_and_once_restore_and_resume ... ok
test runtime::tests::session_memory::oracle::byte_array_charges_encoded_capacity_for_valid_invalid_and_nonstring_payloads ... ok
test runtime::tests::session_memory::oracle::container_capacity_and_legacy_length_terms_are_distinct ... ok
test runtime::tests::session_memory::oracle::empty_and_installed_sessions_match_without_sampling_the_peak ... ok
test runtime::tests::session_memory::oracle::exception_registry_and_exception_expression_payloads_are_exact ... ok
test runtime::tests::session_memory::oracle::expressions_charge_each_shared_occurrence_and_spare_slot ... ok
test runtime::tests::session_memory::oracle::numeric_payloads_follow_signed_bytes_not_limb_capacity ... ok
test runtime::tests::session_memory::request_owners::maintained_accounting_route_adapter_replacement ... ok
test runtime::tests::session_memory::request_owners::maintained_requests_queues_nested_resume_and_cleanup ... ok
test runtime::tests::session_memory::session_memory_field_decision_inventory_is_sorted_and_exhaustive ... ok
test runtime::tests::session_memory::session_memory_walk_counts_declared_exception_storage ... ok
test runtime::tests::session_memory::stability_blooms_are_fixed_size_inline_values ... ok
test runtime::tests::session_memory::structures::association_indexes_keep_stale_storage_until_the_owner_prunes_it ... ok
test runtime::tests::session_memory::structures::call_cache_roots_are_additional_occurrences_of_retained_values ... ok
test runtime::tests::session_memory::structures::dispatch_metadata_and_lazy_index_have_exact_separate_charges ... ok
test runtime::tests::session_memory::structures::live_frames_messages_and_traces_preserve_conservative_inline_terms ... ok
test runtime::tests::session_memory::structures::session_memory_layout_baseline ... Arc<dyn SuspensionFrame>	16
ok
test runtime::tests::session_memory::structures::suspended_frames_charge_storage_repeated_roots_and_exception_payloads ... ok
test runtime::tests::session_memory::structures::tagged_dispatch_indexes_are_charged_for_both_value_tables ... ok
test runtime::tests::session_requests::api::accepted_cancel_abandons_input_before_conversion_or_continuation ... ok
test runtime::tests::session_requests::api::accepted_cancel_before_request_entry_is_observed ... ok
test runtime::tests::session_requests::api::accepted_script_and_file_stop_before_later_forms_after_cancellation ... ok
test runtime::tests::session_requests::api::cancellation_observer_held_across_session_rebuild_is_inert ... ok
test runtime::tests::session_requests::api::complete_request_returns_transport_friendly_matches ... ok
test runtime::tests::session_requests::api::completed_request_handle_is_inert_for_later_requests ... ok
test runtime::tests::session_requests::api::dropped_accepted_ticket_and_generation_rotation_retire_handles ... ok
test runtime::tests::session_requests::api::eval_expr_request_aggregates_messages_and_classifies_budget_termination ... ok
test runtime::tests::session_requests::api::eval_request_memory_ceiling_returns_classified_termination_and_session_recovers ... ok
test runtime::tests::session_requests::api::eval_request_nested_memory_constraint_catches_before_higher_request_ceiling ... ok
test runtime::tests::session_requests::api::eval_request_print_flood_trips_memory_ceiling_with_classified_termination ... ok
test runtime::tests::session_requests::api::eval_request_returns_transport_friendly_cycle_termination_data ... ok
test runtime::tests::session_requests::api::eval_request_returns_transport_friendly_success_data ... ok
test runtime::tests::session_requests::api::eval_request_surfaces_structured_input_requests ... ok
test runtime::tests::session_requests::api::eval_request_take_pending_input_request_moves_and_clears_the_request ... ok
test runtime::tests::session_requests::api::file_request_maps_completed_syntax_io_and_missing_provider_errors ... ok
test runtime::tests::session_requests::api::file_request_maps_resource_limit_exit_and_cancel_errors ... ok
test runtime::tests::session_requests::api::inspect_request_aggregates_symbol_definitions_and_messages ... ok
test runtime::tests::session_requests::api::is_complete_request_distinguishes_complete_incomplete_and_invalid_input ... ok
test runtime::tests::session_requests::api::ordinary_request_and_lazy_observer_leave_interrupt_poll_unarmed ... ordinary_request_armed=false observer_held_request_armed=false
ok
test runtime::tests::session_requests::api::parse_request_returns_structured_syntax_diagnostics ... ok
test runtime::tests::session_requests::api::script_requests_report_file_coordinates_for_every_form_shape ... ok
test runtime::tests::session_requests::api::stale_cancel_cannot_overwrite_current_request_cancel ... ok
test runtime::tests::session_requests::batch_boundary::flat_public_file_cancellation_control ... ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_error - should panic ... 
ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_file - should panic ... 
ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_script - should panic ... 
ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_snapshot - should panic ... 
ok
test runtime::tests::session_requests::batch_boundary::invariant_falsifier_value - should panic ... 
ok
test runtime::tests::session_requests::batch_boundary::public_file_cancel_nested ... ok
test runtime::tests::session_requests::batch_boundary::public_file_cancel_two_levels ... ok
test runtime::tests::session_requests::batch_boundary::public_file_cleanup_observes_inner_then_outer_state_once ... ok
test runtime::tests::session_requests::batch_boundary::public_file_first_input_answered_then_cancel_deeper_second ... ok
test runtime::tests::session_requests::batch_boundary::public_file_rollback_is_deferred_while_blocked ... ok
test runtime::tests::session_requests::batch_boundary::public_file_snapshot_roots_survive_suspension ... ok
test runtime::tests::session_requests::batch_boundary::public_file_success_and_ordinary_error_control ... ok
test runtime::tests::session_requests::batch_boundary::restore::restore_abandons_nested_cleanup_and_file_rollbacks_once ... ok
test runtime::tests::session_requests::batch_boundary::restore::restore_admission_rejection_preserves_unrelated_request ... ok
test runtime::tests::session_requests::batch_boundary::restore::terminal_restore_retires_identity_and_all_owner_state ... ok
test runtime::tests::session_requests::batch_boundary::script_snapshot_and_guest_cancellation_controls ... ok
test runtime::tests::session_requests::debug_tooling_resume::resume_eval_request_with_input_can_finish_reap_handler_rendering ... ok
test runtime::tests::session_requests::debug_tooling_resume::resume_eval_request_with_input_can_finish_repeated_timing_samples ... ok
test runtime::tests::session_requests::debug_tooling_resume::resume_eval_request_with_input_can_finish_trace_capture ... ok
test runtime::tests::session_requests::input_resume::nested_input_suspension_precedes_the_old_suffix_without_reordering_effects ... ok
test runtime::tests::session_requests::input_resume::queued_input_responses_feed_inputstring_without_prompting_again ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_can_finish_delayed_message_templates ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_can_suspend_multiple_times_without_restarting ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_continues_replace_all_without_restarting_target ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_continues_symbol_evaluation_after_fixed_point_suspend ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_finishes_dialog_subsets ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_finishes_with_bindings_without_reprompting ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_formats_delayed_message_templates_after_resume ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_keeps_monitor_display_and_finishes_the_body ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_preserves_already_evaluated_prefix_arguments ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_preserves_block_scope_until_completion ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_reports_input_parse_errors_as_syntax_diagnostics ... ok
test runtime::tests::session_requests::input_resume::resume_eval_request_with_input_skips_uninitialized_module_bindings ... ok
test runtime::tests::session_requests::matcher_resume::cases_collection_survives_multiple_matcher_suspensions ... ok
test runtime::tests::session_requests::matcher_resume::defaultvalue_rhs_guard_resumes_during_matching_without_replaying_the_candidate ... ok
test runtime::tests::session_requests::matcher_resume::downvalue_rhs_guard_resumes_at_the_same_candidate_cursor ... ok
test runtime::tests::session_requests::matcher_resume::guarded_replacement_numeric_value_survives_suspension ... ok
test runtime::tests::session_requests::matcher_resume::matcher_binding_numeric_value_survives_suspension ... ok
test runtime::tests::session_requests::matcher_resume::nested_rule_application_numeric_value_survives_suspension ... ok
test runtime::tests::session_requests::matcher_resume::ordered_single_arg_cursor_resumes_without_replaying_the_matched_prefix ... ok
test runtime::tests::session_requests::matcher_resume::position_collection_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::matcher_resume::position_collection_survives_multiple_matcher_suspensions ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_fifth_arg ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_fourth_arg ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_heads_option_as_third_arg ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_level_spec ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_match_limit ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_cases_with_suspending_symbol_spec ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_cases ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_downvalue_matching ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_matchq ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_conditioned_replace_all ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_count_with_suspending_symbol_spec ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_freeq_with_suspending_heads_option ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_multi_sequence_backtracking_conditions ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_nested_matcher_inside_sequence_conditioned_matchq ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_orderless_patternsequence_group_matching ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_position_with_suspending_match_limit ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_position_with_suspending_symbol_spec ... ok
test runtime::tests::session_requests::matcher_resume::resume_eval_request_with_input_can_finish_sequence_conditioned_matchq ... ok
test runtime::tests::session_requests::matcher_resume::sequence_binding_vec_continuation_roundtrip_preserves_multiple_bindings ... ok
test runtime::tests::session_requests::matcher_resume::subvalue_rhs_guard_resumes_at_the_same_candidate_cursor ... ok
test runtime::tests::session_requests::matcher_resume::traversal_count_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::matcher_resume::upvalue_rhs_guard_resumes_at_the_same_candidate_cursor ... ok
test runtime::tests::session_requests::transforms_resume::compiled_rule_vec_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::transforms_resume::deletecases_iterative_tail_match_resume_matches_uninterrupted_association_walk ... ok
test runtime::tests::session_requests::transforms_resume::deletecases_iterative_tail_match_resume_matches_uninterrupted_call_walk ... ok
test runtime::tests::session_requests::transforms_resume::normalized_rule_expr_vec_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::transforms_resume::replace_and_replaceat_indexed_rule_plans_survive_mid_traversal_suspension ... ok
test runtime::tests::session_requests::transforms_resume::replace_iterative_tail_resumes_at_the_same_deep_rule_attempt ... ok
test runtime::tests::session_requests::transforms_resume::replace_levelspec_option_continuation_roundtrip_matches_uninterrupted_run ... ok
test runtime::tests::session_requests::transforms_resume::replacepart_resumes_mid_walk_with_the_same_rule_and_partial_result ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_suspending_match_limit ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_suspending_pattern_symbol ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_suspending_target ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_normalize_rules_after_suspending_rule_source ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replace_after_suspending_levelspec ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replace_after_suspending_target ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replaceall_after_suspending_rule_source ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replaceat_after_suspending_delayed_rhs ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replaceat_after_suspending_position_spec ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacelist_after_multiple_backtracking_matches ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacelist_after_suspending_delayed_rhs ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacepart_after_suspending_pattern_position_spec ... ok
test runtime::tests::session_requests::transforms_resume::resume_eval_request_with_input_can_finish_replacepart_with_suspending_delayed_rhs ... ok
test runtime::tests::session_requests::transforms_resume_patterns::eval_request_reports_prints_on_failure_and_across_input_resume ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_except_matchq ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_patterntest_matchq ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_repeated_patterntest_matchq ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_sequence_patterntest_matchq ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_setattributes_with_suspending_attribute_spec ... ok
test runtime::tests::session_requests::transforms_resume_patterns::resume_eval_request_with_input_can_finish_thread_with_suspending_arg_spec ... ok
test runtime::tests::structure_data::replace::replaceat_supports_exact_positions_operator_form_and_rule_lists ... ok
test runtime::tests::structure_data::replace::replacepart_deep_path_shallow_companion_pins_three_level_rebuild ... ok
test runtime::tests::structure_data::replace::replacepart_handles_duplicate_and_overlapping_targets_more_stably ... ok
test runtime::tests::structure_data::replace::replacepart_handles_twenty_thousand_structural_rules_in_one_walk ... ok
test runtime::tests::structure_data::replace::replacepart_rule_lists_preserve_sequential_conflicts_and_missing_positions ... ok
test runtime::tests::structure_data::replace::replacepart_span_preserves_lenient_missing_positions_and_raw_delayed_rhs ... ok
test runtime::tests::structure_data::replace::replacepart_spans_reuse_part_selection_and_fail_closed_on_malformed_shapes ... ok
test runtime::tests::structure_data::replace::replacepart_supports_immediate_delayed_and_head_replacements ... ok
test runtime::tests::structure_data::replace::replacepart_supports_pattern_position_specs_and_rhs_bindings ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_contested_guards_run_last_to_first_and_stop_after_acceptance ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_declining_later_guard_hands_position_to_earlier_rule ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_evaluates_each_rule_position_guard_at_most_once ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_guard_acceptance_preserves_raw_delayed_bodies ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_guard_suspension_resumes_same_rule_position_and_plan ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_pattern_guards_receive_bindings_and_option_scope ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_rhs_guards_accept_and_decline_exact_position_forms ... ok
test runtime::tests::structure_data::replace_part_guards::replacepart_unguarded_rules_keep_explicit_later_rule_wins_behavior ... ok
test runtime::tests::structure_data::selectors_part2::direct_empty_span_selects_full_range ... ok
test runtime::tests::structure_data::selectors_part2::direct_single_endpoint_span_selects_prefix_or_suffix_ranges ... ok
test runtime::tests::structure_data::selectors_part2::invalid_span_messages_render_infix_specs ... ok
test runtime::tests::structure_data::selectors_part2::invalid_upto_span_diagnostics_are_not_duplicated_after_normalization ... ok
test runtime::tests::structure_data::selectors_part2::part_and_extract_emit_single_take_for_explicit_invalid_spans ... ok
test runtime::tests::structure_data::selectors_part2::part_and_extract_report_branch_selector_failures ... ok
test runtime::tests::structure_data::selectors_part2::part_and_extract_take_messages_render_span_ranges ... ok
test runtime::tests::structured_exceptions::declining_exception_handler_guard_continues_ancestor_and_handler_scans ... ok
test runtime::tests::structured_exceptions::exception_type_registry_round_trips_through_loader_snapshot ... ok
test runtime::tests::structured_exceptions::suspending_exception_handler_guard_resumes_its_scan_cursor_without_replay ... ok
test runtime::tests::suspension_abandonment::abandonment_evaluates_only_withcleanup_cleanup_and_abandons_nested_input_lifo ... ok
test runtime::tests::suspension_abandonment::abandonment_finishes_module_activation_without_initializers_or_body ... ok
test runtime::tests::suspension_abandonment::abandonment_generations_are_monotonic_and_pure_frames_run_no_continuation ... ok
test runtime::tests::suspension_abandonment::abandonment_pops_monitor_without_a_final_refresh_or_callback ... ok
test runtime::tests::suspension_abandonment::abandonment_pops_option_value_scope_without_running_definition_suffix ... ok
test runtime::tests::suspension_abandonment::abandonment_pops_reap_without_rendering_or_running_body_suffix ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_abort_protect_and_check_abort_checkpoints ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_an_existing_time_deadline_exactly ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_block_initializers_and_bodies_inner_to_outer ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_block_random_prng_state ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_both_loop_iterator_scope_forms_and_iteration_budget ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_check_and_quiet_captures_without_check_fallback ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_constraint_nesting_without_fallbacks ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_test_report_stream_capture_without_a_report ... ok
test runtime::tests::suspension_abandonment::abandonment_restores_trace_and_evaluation_data_captures_without_rendering ... ok
test runtime::tests::suspension_abandonment::abandonment_rolls_back_partial_package_install_without_reporting_failure ... ok
test runtime::tests::suspension_abandonment::checkpoint_rollback_abandons_a_trial_generation_before_restoring_state ... ok
test runtime::tests::suspension_abandonment::checkpoint_rollback_preserves_its_captured_generation ... ok
test runtime::tests::suspension_abandonment::queued_input_abandons_the_older_generation_before_consuming_the_value ... ok
test runtime::tests::syntax_information_debug_catalog::syntax_information_covers_current_cl09_debugging_surface_subset ... ok
test runtime::tests::syntax_information_lifecycle::protect_blocks_direct_syntax_information_assignment_and_unset ... ok
test runtime::tests::syntax_information_lifecycle::syntax_information_defaults_assignment_and_unset_cover_current_first_pass_subset ... ok
test runtime::tests::syntax_information_lifecycle::syntax_information_participates_in_block_and_clearall ... ok
test runtime::tests::syntax_information_lifecycle::unsetting_system_syntax_information_reverts_to_default_rules ... ok
test runtime::tests::thread_and_dispatch::argument_preparation_cow_preserves_dispatch_semantics ... ok
test runtime::tests::thread_and_dispatch::block_localizes_definitions_attributes_and_messages ... ok
test runtime::tests::thread_and_dispatch::downvalue_miss_stability_shortcut_invalidates_on_later_definition ... ok
test runtime::tests::thread_and_dispatch::evaluated_head_function_attributes_control_argument_evaluation ... ok
test runtime::tests::thread_and_dispatch::flat_attribute_flattens_calls_and_matches_flat_patterns ... ok
test runtime::tests::thread_and_dispatch::held_downvalue_matching_normalizes_nested_attribute_heads ... ok
test runtime::tests::thread_and_dispatch::listable_interacts_with_builtin_and_function_attributes ... ok
test runtime::tests::thread_and_dispatch::listable_mismatched_lengths_emit_thread_message_and_leave_call_unevaluated ... ok
test runtime::tests::thread_and_dispatch::listable_threads_user_defined_calls_and_broadcasts_scalars ... ok
test runtime::tests::thread_and_dispatch::module_handles_many_uninitialized_bindings_without_exhausting_the_native_stack ... ok
test runtime::tests::thread_and_dispatch::module_skips_existing_generated_symbol_names ... ok
test runtime::tests::thread_and_dispatch::one_identity_allows_singleton_head_patterns_to_match_atoms ... ok
test runtime::tests::thread_and_dispatch::orderless_attribute_canonicalizes_calls_and_rules ... ok
test runtime::tests::thread_and_dispatch::stable_call_shortcut_preserves_symbol_and_upvalue_evaluation ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_accepts_resolved_head_and_spec_values ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_supports_default_and_custom_thread_heads ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_supports_standard_sequence_spec_subset ... ok
test runtime::tests::thread_and_dispatch::thread_builtin_supports_wide_dense_argument_selection ... ok
test runtime::tests::thread_and_dispatch::thread_evaluates_its_first_argument_before_threading ... ok
test runtime::tests::thread_and_dispatch_part2::thread_builtin_mismatch_emits_message_and_stays_unevaluated ... ok
test runtime::tests::traversal_part2::cases_and_position_report_invalid_match_limit_messages ... ok
test runtime::tests::traversal_part2::count_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::traversal_part2::freeq_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::traversal_part2::level_reports_invalid_levelspec_messages ... ok
test runtime::tests::traversal_part2::memberq_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::traversal_part2::position_operator_form_preserves_data_and_error_forms ... ok
test runtime::tests::traversal_part2::traversal_family_reports_invalid_levelspec_messages ... ok
test runtime::tests::valueq_stable_shortcut::value_q_shortcut_populates_only_evaluation_caches ... ok
test runtime::tests::valueq_stable_shortcut::value_q_shortcut_skips_snapshot_and_suppression_even_after_attribute_revision ... ok
test runtime::tests::valueq_stable_shortcut::value_q_stability_shortcut_matches_the_trial_path_over_one_shared_corpus ... ok
test runtime::tests::wide_child_traversals::cases_resumes_mid_walk_between_children ... ok
test runtime::tests::wide_child_traversals::cases_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::count_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::delete_cases_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::position_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::replace_all_walks_thirty_thousand_children ... ok
test runtime::tests::wide_child_traversals::replace_walks_thirty_thousand_children ... ok
test runtime::tests::wide_definition_dispatch::reap_walks_thirty_thousand_buckets ... ok
test runtime::tests::wide_definition_dispatch::replace_list_walks_thirty_thousand_rules ... ok
test runtime::tests::wide_definition_dispatch::sub_value_dispatch_resumes_mid_walk_with_pending_outcome ... ok
test runtime::tests::wide_definition_dispatch::sub_value_dispatch_walks_thirty_thousand_definitions ... ok
test runtime::tests::wide_definition_dispatch::up_value_dispatch_resumes_mid_walk_with_pending_outcome ... ok
test runtime::tests::wide_definition_dispatch::up_value_dispatch_resumes_to_next_candidate_when_condition_fails ... ok
test runtime::tests::wide_definition_dispatch::up_value_dispatch_walks_thirty_thousand_definitions ... ok
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 23.02s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
running 1087 tests
test associations::basic::key_take_drop::tests::key_take_drop_forced_front_tail_matches_complete_front_for_both_flags ... ok
test associations::basic::key_take_drop::tests::key_take_drop_forced_tail_preserves_invalid_payload_distinction ... ok
test associations::basic::keys::tests::keys_forced_front_tail_matches_complete_front_exactly ... ok
test associations::basic::keys::tests::keys_forced_tail_preserves_first_error_in_source_order ... ok
test associations::basic::lookup::tests::lookup_outcome_drop_is_bounded_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack drop certificate; run explicitly with --release
test associations::basic::lookup::tests::lookup_planning_forced_front_tail_matches_complete_front_exactly ... ok
test associations::basic::lookup::tests::lookup_query_count_forced_front_tail_matches_complete_front_exactly ... ok
test associations::basic::lookup::tests::lookup_rendering_forced_fronts_agree_owned_borrowed_and_exactly ... ok
test associations::basic::values::tests::values_forced_front_tail_matches_complete_front_exactly ... ok
test associations::basic::values::tests::values_forced_tail_preserves_outer_expression_error_payload ... ok
test associations::key_reshape::traversal::tests::flatten_association_forced_front_tail_matches_complete_front_on_branched_fixture ... ok
test associations::key_reshape::traversal::tests::insert_path_forced_front_tail_matches_complete_front_on_branched_fixture ... ok
test associations::key_reshape::traversal::tests::render_nested_path_forced_front_tail_matches_complete_front_on_branched_fixture ... ok
test associations::missing::target::tests::resume_target_work_given_a_carried_prefix_does_not_replay_it ... ok
test associations::missing::target::tests::target_front_handoff_carries_live_cursors_and_completed_prefixes ... ok
test associations::missing::traversal::counting::tests::zero_front_uses_heap_tails_for_delayed_wrapper_counts ... ok
test associations::missing::traversal::deletion::tests::zero_front_heap_tail_matches_front_for_branched_associations ... ok
test associations::missing::traversal::projection::tests::zero_front_heap_projection_preserves_the_exact_preflight_count ... ok
test catalog::memory_growth_contract::installed_builtin_memory_growth_inventory_is_sorted_and_exhaustive ... ok
test functional_ops::apply_map::traversal::map_indexed::tests::map_indexed_forced_front_tail_matches_complete_front_on_branched_fixture ... ok
test json::encode::depth_tests::to_json_accepts_nesting_at_the_depth_limit ... ok
test json::encode::depth_tests::to_json_refuses_nesting_one_past_the_depth_limit ... ok
test lazy::tests::lazy_flat_abandonment_depth ... flat abandonment depth=2000 complete
ok
test lazy::tests::lazy_flat_all_variant_roots_depth ... all source/stage/cursor roots depth=2000 complete
ok
test lazy::tests::lazy_flat_clone_and_working_copy_depth ... flat clone and working copy depth=2000 complete
ok
test lazy::tests::lazy_flat_last_owner_drop_depth ... last-owner flat drop depth=2000 complete
ok
test polynomials::core::construction::tests::indexed_batch_merge_cancels_duplicates_and_sorts_once ... ok
test polynomials::core::factors::tests::batch_term_product_matches_repeated_binary_product ... ok
test polynomials::core::factors::tests::ordered_factor_merge_matches_concat_and_normalize ... ok
test polynomials::core::factors::tests::ordered_factor_merge_preserves_canonical_ties ... ok
test polynomials::core::intersection::tests::intersection_matches_structural_bases_across_canonical_ties ... ok
test polynomials::core::term_order::tests::sparse_order_avoids_wide_zero_coordinate_probes ... ok
test polynomials::core::term_order::tests::sparse_order_matches_dense_basis_order_exhaustively ... ok
test polynomials::core::variables::tests::wide_variable_collection_indexes_overlaps_and_sorts_once ... ok
test polynomials::factor_collections::tests::indexed_factor_collections_preserve_order_and_multiplicity ... ok
test predicates::collections::matching_tests::augmenting_path_matching_agrees_with_exhaustive_small_graphs ... ok
test predicates::collections::matching_tests::duplicate_free_state_visits_the_exact_expression_roots ... ok
test strings::patterns::matcher::api::tests::reversed_suffix_matching_preserves_repeated_capture_equality ... ok
test strings::patterns::matcher::api::tests::reversible_suffix_pattern_uses_two_match_attempts ... ok
test strings::patterns::matcher::engine::tests::except_filters_with_one_excluded_evaluation ... ok
test strings::patterns::matcher::literal::tests::linear_literal_search_bounds_long_prefix_comparison_work ... ok
test strings::patterns::matcher::literal::tests::literal_matches_preserve_overlap_and_unicode_character_offsets ... ok
test strings::patterns::matcher::reverse::tests::reversal_swaps_sequence_direction_and_boundaries ... ok
test strings::patterns::matcher::reverse::tests::reversed_number_atom_preserves_full_match_language ... ok
test strings::patterns::matcher::split_trim::tests::literal_list_fast_path_is_selected_once_and_bounds_boundary_work ... ok
test strings::patterns::matcher::split_trim::tests::mixed_list_stays_on_the_general_pattern_path ... ok
test strings::patterns::matcher::state_set::tests::indexed_collision_chain_compares_bindings_and_preserves_order ... ok
test strings::patterns::matcher::state_set::tests::wide_distinct_ends_build_one_index_after_the_small_scan_tier ... ok
test strings::patterns::regex::tests::reversed_regex_accepts_exactly_reversed_full_matches ... ok
test strings::scripting::html::tests::unterminated_entity_scan_work_probe ... ok
test strings::similarity::selection::tests::best_ties_selection_uses_one_comparison_pass ... ok
test strings::similarity::selection::tests::partial_nearest_selection_matches_full_sort_with_linear_comparison_work ... ok
test strings::similarity::selection::tests::partial_nearest_selection_preserves_distance_then_source_order ... ok
test strings::similarity::selection::tests::rational_comparison_handles_equal_and_distinct_denominators ... ok
test structure_ops::sequence::delete::tests::delete_planner_starts_at_the_measured_crossover ... ok
test structure_ops::sequence::edit_plan::tests::batched_delete_matches_descending_sequential_reference_with_overlaps_and_head ... ok
test structure_ops::sequence::edit_plan::tests::batched_delete_matches_reference_for_every_nested_path_subset ... ok
test structure_ops::sequence::edit_plan::tests::batched_insert_matches_descending_sequential_reference_with_overlaps ... ok
test structure_ops::sequence::edit_plan::tests::batched_insert_matches_reference_for_every_nested_path_subset ... ok
test structure_ops::sequence::edit_plan::tests::many_deletions_use_one_linear_parent_pass ... ok
test structure_ops::sequence::edit_plan::tests::many_insertions_use_one_linear_parent_pass ... ok
test structure_ops::sequence::insert::tests::insert_planner_starts_at_the_measured_crossover ... ok
test structure_ops::sequence::take_drop::tests::iterative_drop_tail_preserves_fanout_order ... ok
test structure_ops::sequence::take_drop::tests::iterative_take_tail_preserves_fanout_order ... ok
test structure_ops::sequence::take_drop::tests::linear_drop_filter_matches_set_membership_for_small_ranges ... ok
test tests::accounting_mode_route::builtin_adapter_and_catalog_pilot_reenter_guest_evaluation ... callback crossing: AccountingSnapshot { pushes: 3, pops: 3, live: 0, peak: 2, callbacks: 2, walker_fallbacks: 2, suspends: 0, resumes: 0, host_mutations: 0, arms: 1 }
ok
test tests::accounting_mode_route::host_mutation_has_an_explicit_armed_entry ... host crossing: AccountingSnapshot { pushes: 1, pops: 1, live: 0, peak: 1, callbacks: 0, walker_fallbacks: 0, suspends: 0, resumes: 0, host_mutations: 1, arms: 1 }
ok
test tests::accounting_mode_route::nested_memory_constrained_arms_from_unarmed_and_the_walker_still_refuses ... nested limit transition: AccountingSnapshot { pushes: 1, pops: 1, live: 0, peak: 1, callbacks: 1, walker_fallbacks: 1, suspends: 0, resumes: 0, host_mutations: 0, arms: 2 }
ok
test tests::accounting_mode_route::old_registration_keeps_its_shape_and_replaces_the_companion_with_fallback ... legacy registration: AccountingSnapshot { pushes: 1, pops: 1, live: 0, peak: 1, callbacks: 0, walker_fallbacks: 1, suspends: 0, resumes: 0, host_mutations: 0, arms: 1 }
ok
test tests::accounting_mode_route::recursive_evaluation_retains_the_session_shadow_and_balances_stack ... recursive crossing: AccountingSnapshot { pushes: 0, pops: 0, live: 0, peak: 0, callbacks: 0, walker_fallbacks: 1, suspends: 0, resumes: 0, host_mutations: 0, arms: 1 } -> AccountingSnapshot { pushes: 6, pops: 6, live: 0, peak: 4, callbacks: 0, walker_fallbacks: 4, suspends: 0, resumes: 0, host_mutations: 0, arms: 2 }
ok
test tests::accounting_mode_route::suspended_host_entry_and_two_resumes_keep_the_explicit_frame_token ... suspension crossing: AccountingSnapshot { pushes: 1, pops: 1, live: 0, peak: 1, callbacks: 1, walker_fallbacks: 1, suspends: 1, resumes: 0, host_mutations: 0, arms: 1 } -> AccountingSnapshot { pushes: 4, pops: 4, live: 0, peak: 2, callbacks: 1, walker_fallbacks: 3, suspends: 2, resumes: 2, host_mutations: 1, arms: 1 }
ok
test tests::accounting_mode_route::unarmed_evaluation_never_allocates_a_shadow ... ok
test tests::association_depth_contract::lazy_depth_contract::lazy_cross_module_recursion_falsifier ... ok
test tests::association_depth_contract::lazy_depth_contract::lazy_ownership_is_not_self_reaching ... ok
test tests::association_depth_contract::lazy_depth_contract::lazy_recursive_helper_falsifier ... recursive helper falsifier: [RecursiveEdge { path: "lazy/machine.rs", caller: "drive_pull", callee: "drive_pull", count: 1 }]
ok
test tests::association_depth_contract::lazy_depth_contract::lazy_self_reaching_falsifier - should panic ... 
ok
test tests::association_depth_contract::lazy_depth_contract::lazy_source_has_no_recursive_edges ... ok
test tests::association_depth_contract::recursion_inventory::association_recursion_inventory_is_sorted_exhaustive_and_declared ... ok
test tests::association_depth_contract::recursion_inventory::synthetic_unregistered_recursion_is_reported_exactly ... ok
test tests::association_depth_contract::registry::association_depth_registry_is_sorted_and_exhaustive ... ok
test tests::association_depth_contract::registry::real_catalog_misdeclaration_names_association_key_flatten_recursive_edge - should panic ... 
ok
test tests::association_key_reshape_depth::association_key_deflatten_abandons_a_deep_partial_tree_without_growing_the_stack ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_reshape_depth::association_key_deflatten_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_reshape_depth::association_key_deflatten_flatten_round_trip_is_exact_at_physical_height_4097 ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_reshape_depth::association_key_deflatten_three_level_companion_pins_order_overwrite_and_conflicts ... ok
test tests::association_key_reshape_depth::association_key_flatten_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_reshape_depth::association_key_flatten_three_level_companion_pins_exact_paths_and_rule_heads ... ok
test tests::association_key_take_drop_depth::key_drop_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_take_drop_depth::key_take_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_key_take_drop_depth::key_take_drop_three_level_companion_pins_wrappers_sibling_key_order_and_rule_heads ... ok
test tests::association_keys_depth::keys_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_keys_depth::keys_three_level_companion_pins_wrappers_sibling_key_order_and_first_error ... ok
test tests::association_lookup_depth::lookup_hit_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_lookup_depth::lookup_miss_and_resume_drop_complete_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_lookup_depth::lookup_three_level_companion_pins_shape_hits_misses_default_and_wrapper ... ok
test tests::association_values_depth::values_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::association_values_depth::values_three_level_companion_pins_wrappers_sibling_value_order_and_outer_error ... ok
test tests::associations::association_part_cursor::association_key_selector_suspension_resumes_the_original_part_call ... ok
test tests::associations::association_part_cursor::association_part_and_extract_value_message_matrix_is_exact ... ok
test tests::associations::associations::association_and_associationq_cover_first_pass_rule_subset ... ok
test tests::associations::associations::association_canonicalization_preserves_order_and_last_rule_above_index_threshold ... ok
test tests::associations::associations::lookup_keys_values_and_key_cover_first_pass_association_queries ... ok
test tests::associations::associations_aggregate_diagnostics::association_aggregate_diagnostics_cover_invalid_forms ... ok
test tests::associations::associations_basic_diagnostics::association_lookup_keys_values_diagnostics_cover_invalid_forms ... ok
test tests::associations::associations_key_arity_diagnostics::key_function_arity_and_currying_diagnostics_cover_invalid_forms ... ok
test tests::associations::associations_key_diagnostics::key_transform_diagnostics_cover_invalid_forms ... ok
test tests::associations::associations_key_reshape::association_key_deflatten_preserves_wide_branch_order_after_indexing ... ok
test tests::associations::associations_key_reshape::association_key_flatten_and_deflatten_round_trip_well_formed_inputs ... ok
test tests::associations::associations_key_reshape::association_key_flatten_and_deflatten_transform_path_keys ... ok
test tests::associations::associations_key_reshape::key_group_by_groups_association_entries_by_transformed_keys ... ok
test tests::associations::associations_key_reshape::merge_nested_recursively_merges_associations_and_combines_leaf_values ... ok
test tests::associations::associations_key_reshape::merge_nested_suspended_combiner_resumes_without_repeating_calls ... ok
test tests::associations::associations_key_reshape::merge_nested_suspension_freezes_the_iterative_tail_without_replay ... ok
test tests::associations::associations_key_reshape::merge_nested_three_level_companion_pins_branch_order_and_combiner_schedule ... ok
test tests::associations::associations_key_reshape::nested_association_additions_leave_malformed_inputs_inert ... ok
test tests::associations::associations_key_reshape::nested_association_additions_support_operator_forms ... ok
test tests::associations::associations_key_sets::association_thread_and_key_set_ops_cover_first_pass_subset ... ok
test tests::associations::associations_key_sort::custom_key_sort_by_decorates_once_in_input_order ... ok
test tests::associations::associations_key_sort::custom_key_sorts_obey_the_merge_bound_without_growing_the_stack ... ok
test tests::associations::associations_key_sort::custom_key_sorts_preserve_entry_forms_ordering_and_stable_ties ... ok
test tests::associations::associations_key_sort::custom_key_sorts_preserve_the_full_comparator_result_policy ... ok
test tests::associations::associations_key_sort::every_custom_key_sort_by_key_resumes_with_the_uninterrupted_trace ... ok
test tests::associations::associations_key_sort::every_custom_key_sort_comparison_resumes_with_the_uninterrupted_trace ... ok
test tests::associations::associations_key_value_pattern::key_value_pattern_matches_association_and_rule_inputs ... ok
test tests::associations::associations_missing_diagnostics::missing_and_delete_missing_diagnostics_cover_invalid_forms ... ok
test tests::associations::associations_mutation::associateto_mutates_bulk_nested_part_targets ... ok
test tests::associations::associations_mutation::associateto_mutates_deep_key_path_targets ... ok
test tests::associations::associations_mutation::associateto_mutates_mixed_integer_and_key_part_targets ... ok
test tests::associations::associations_mutation::keydropfrom_mutates_deep_key_path_targets ... ok
test tests::associations::associations_mutation::keydropfrom_mutates_mixed_integer_and_key_leaf_targets ... ok
test tests::associations::associations_mutation::keydropfrom_rejects_bulk_nested_part_targets_without_mutation ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_after_attribute_invalidation_addresses_the_new_held_identity ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_after_index_rebind_and_clear_preserves_older_snapshots ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_after_owner_clear_uses_only_the_reinstalled_identity ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_curried_operator_forms_store_the_prepared_identity ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_prepares_every_discovered_shape_for_both_mutation_heads ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_reads_index_and_outer_definition_exactly_once ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_resumes_preparation_and_outer_read_without_replay_for_both_heads ... ok
test tests::associations::associations_mutation_definition_fallback::definition_fallback_uses_canonical_owner_for_head_specific_protection_checks ... ok
test tests::associations::associations_mutation_definition_fallback::symbol_and_part_mutation_fast_paths_remain_separate_from_definition_preparation ... ok
test tests::associations::associations_mutation_diagnostics::association_mutation_diagnostics_cover_invalid_targets_and_arity ... ok
test tests::associations::associations_mutation_operator_forms::association_mutation_heads_advertise_subvalues_hold_all ... ok
test tests::associations::associations_mutation_operator_forms::association_mutation_operator_forms_apply_to_raw_held_targets ... ok
test tests::associations::associations_mutation_operator_forms::association_mutation_operator_forms_report_application_arity ... ok
test tests::associations::associations_mutation_surface::associateto_keydropfrom_and_keyvaluepattern_cover_first_pass_mutation_surface ... ok
test tests::associations::associations_mutation_surface::association_part_mutations_evaluate_symbol_selectors_before_reading_the_owner ... ok
test tests::associations::associations_mutation_surface::keydropfrom_preserves_association_order_at_indexed_width ... ok
test tests::associations::associations_part2::counts_explicit_key_specs_cover_nested_duplicate_and_empty_edges ... ok
test tests::associations::associations_part2::group_by_nested_specs_rekey_delayed_association_values_per_level ... ok
test tests::associations::associations_part2::group_by_rule_specs_consume_delayed_association_values_for_key_and_value ... ok
test tests::associations::associations_part2::group_by_wide_association_nested_and_reducer_phases_preserve_order_and_call_counts ... ok
test tests::associations::associations_part2::group_by_wide_association_preserves_group_and_entry_order ... ok
test tests::associations::associations_set_lvalue::set_association_lvalue_falls_back_or_reports_failed_nested_keys ... ok
test tests::associations::associations_set_lvalue::set_single_association_key_finishes_rhs_and_selector_evaluation_first ... ok
test tests::associations::associations_set_lvalue::set_single_association_key_keeps_canonicalization_and_invalid_fallbacks ... ok
test tests::associations::associations_set_lvalue::set_single_association_key_still_refuses_a_protected_target ... ok
test tests::associations::associations_set_lvalue::set_single_association_keys_preserves_aliases_order_and_rule_heads ... ok
test tests::associations::associations_set_lvalue::set_writes_association_function_and_part_keys ... ok
test tests::associations::join_across::join_across_accepts_empty_sides_for_every_join_type ... ok
test tests::associations::join_across::join_across_duplicate_values_form_an_ordered_cross_product ... ok
test tests::associations::join_across::join_across_failures_emit_one_declared_message_and_stay_inert ... ok
test tests::associations::join_across::join_across_inner_join_is_left_major ... ok
test tests::associations::join_across::join_across_left_right_and_outer_preserve_walk_order_and_bare_rows ... ok
test tests::associations::join_across::join_across_missing_keys_are_unmatched_without_padding ... ok
test tests::associations::join_across::join_across_rule_renamed_keys_keep_both_columns ... ok
test tests::associations::join_across::join_across_supports_multi_key_and_expression_key_specs ... ok
test tests::associations::join_across::join_across_uses_right_row_values_on_column_collisions ... ok
test tests::associations::join_across::join_across_wide_key_index_preserves_duplicate_and_missing_key_semantics ... ok
test tests::associations_missing_surface::missing_and_delete_missing_cover_first_pass_subset ... ok
test tests::builtin_reference_docgen::builtin_reference_is_in_sync ... ok
test tests::bytes::byte_array_to_string_and_predicate_cover_boundaries ... ok
test tests::bytes::byte_catalog_entries_are_pure ... ok
test tests::bytes::crc32_known_vectors_are_unsigned_integers ... ok
test tests::bytes::digest_json_end_to_end_is_pinned ... ok
test tests::bytes::digest_known_answer_vectors_cover_string_and_byte_array_inputs ... ok
test tests::bytes::hex_and_base64_codecs_round_trip_boundaries ... ok
test tests::bytes::malformed_codecs_fail_closed ... ok
test tests::bytes::unit6_codec_byte_datetime_and_schema_heads_are_protected ... ok
test tests::bytes::unknown_digest_output_format_fails_closed ... ok
test tests::bytes::unsupported_argument_types_remain_inert ... ok
test tests::calculus::d_handles_polynomial_and_rational_function_rules ... ok
test tests::calculus::d_keeps_current_variable_boundary_for_pattern_or_non_symbol_heads ... ok
test tests::calculus::d_preserves_abstract_call_fallback_and_variable_free_prepass ... ok
test tests::calculus::d_preserves_nested_chain_rule_factor_order ... ok
test tests::calculus::d_preserves_plus_zero_elision_and_combination ... ok
test tests::calculus::d_preserves_rational_and_abstract_power_rules ... ok
test tests::calculus::d_preserves_times_term_and_factor_order ... ok
test tests::calculus::d_reports_concrete_invalid_variable_and_negative_order_specs ... ok
test tests::calculus::d_supports_indexed_variables_and_higher_order_specs ... ok
test tests::calculus::d_symbolic_order_and_valid_derivatives_stay_message_free ... ok
test tests::calculus::d_uses_abstract_derivative_forms_for_symbol_headed_calls ... ok
test tests::catalog_attributes_match_kernel_vocabulary ... ok
test tests::catalog_capabilities::date_and_time_object_predicates_remain_pure ... ok
test tests::catalog_capabilities::datetime_catalog_capabilities_match_clock_separation_surface ... ok
test tests::catalog_capabilities::random_catalog_capabilities_match_dispatch_surface ... ok
test tests::codecs_matrix::csv_headers_flow_into_json_and_back_to_csv ... ok
test tests::codecs_matrix::grand_tour_json_toml_yaml_json_is_identity ... ok
test tests::codecs_matrix::ini_flows_into_json_and_back_as_strings ... ok
test tests::codecs_matrix::json_toml_json_round_trip_preserves_the_shared_value ... ok
test tests::codecs_matrix::json_yaml_json_round_trip_preserves_the_shared_value ... ok
test tests::codecs_matrix::xml_values_extract_into_the_shared_model ... ok
test tests::codecs_text::codecs_compose_with_json_and_editing_flows ... ok
test tests::codecs_text::csv_codecs_fail_closed_with_messages ... ok
test tests::codecs_text::csv_headers_tsv_ragged_rows_and_round_trips_are_pinned ... ok
test tests::codecs_text::from_csv_parses_core_rows_quoting_newlines_and_unicode ... ok
test tests::codecs_text::from_ini_parses_documented_dialect ... ok
test tests::codecs_text::ini_codecs_fail_closed_with_messages ... ok
test tests::codecs_text::ini_to_ini_output_and_round_trips_are_pinned ... ok
test tests::codecs_text::ini_wide_key_and_section_indexes_preserve_round_trips ... ok
test tests::codecs_text::to_csv_encodes_lists_and_association_rows ... ok
test tests::codecs_text::to_csv_wide_association_rows_follow_header_order ... ok
test tests::codecs_toml::from_toml_depth_limit_accepts_each_descent_at_the_limit ... ok
test tests::codecs_toml::from_toml_depth_limit_rejects_each_descent_one_past ... ok
test tests::codecs_toml::from_toml_depth_limit_rejects_the_far_past_stack_exhaustion_shape ... ok
test tests::codecs_toml::from_toml_golden_corpus_is_byte_stable ... ok
test tests::codecs_toml::from_toml_golden_error_corpus_is_byte_stable ... ok
test tests::codecs_toml::from_toml_maps_datetime_forms ... ok
test tests::codecs_toml::from_toml_multiline_strings_trim_and_continue_lines ... ok
test tests::codecs_toml::from_toml_numbers_are_exact ... ok
test tests::codecs_toml::from_toml_parses_realistic_config ... ok
test tests::codecs_toml::from_toml_rejects_invalid_input_fail_closed ... ok
test tests::codecs_toml::to_toml_golden_corpus_is_byte_stable ... ok
test tests::codecs_toml::to_toml_golden_error_corpus_is_byte_stable ... ok
test tests::codecs_toml::to_toml_selects_tables_quotes_keys_and_preserves_assignment_order ... ok
test tests::codecs_toml::toml_codecs_compose_with_json ... ok
test tests::codecs_xml::from_xml_decodes_unicode_and_numeric_entities ... ok
test tests::codecs_xml::from_xml_depth_limit_accepts_deep_element_nesting ... ok
test tests::codecs_xml::from_xml_depth_limit_rejects_element_nesting_one_past ... ok
test tests::codecs_xml::from_xml_depth_limit_rejects_the_far_past_stack_exhaustion_shape ... ok
test tests::codecs_xml::from_xml_parses_realistic_document_to_symbolic_tree ... ok
test tests::codecs_xml::from_xml_rejects_invalid_xml_fail_closed ... ok
test tests::codecs_xml::to_xml_encodes_compact_escaped_and_pretty_forms ... ok
test tests::codecs_xml::to_xml_rejects_non_xml_trees_fail_closed ... ok
test tests::codecs_xml::xml_round_trips_are_pinned ... ok
test tests::codecs_xml::xml_values_compose_with_json ... ok
test tests::codecs_xml::xml_wide_attribute_index_preserves_round_trip ... ok
test tests::codecs_yaml::from_yaml_depth_limit_accepts_every_collection_descent_at_the_limit ... ok
test tests::codecs_yaml::from_yaml_depth_limit_rejects_every_collection_descent_one_past ... ok
test tests::codecs_yaml::from_yaml_depth_limit_rejects_the_far_past_stack_exhaustion_shape ... ok
test tests::codecs_yaml::from_yaml_golden_corpus_is_byte_stable ... ok
test tests::codecs_yaml::from_yaml_golden_error_corpus_is_byte_stable ... ok
test tests::codecs_yaml::from_yaml_literal_blocks_pin_chomping ... ok
test tests::codecs_yaml::from_yaml_numbers_and_nulls_are_exact ... ok
test tests::codecs_yaml::from_yaml_parses_k8s_flavored_config ... ok
test tests::codecs_yaml::from_yaml_rejects_unsupported_yaml_fail_closed ... ok
test tests::codecs_yaml::from_yaml_uses_norway_safe_typing ... ok
test tests::codecs_yaml::to_yaml_round_trips_adversarial_strings_and_empties ... ok
test tests::codecs_yaml::yaml_codecs_compose_with_json ... ok
test tests::codecs_yaml::yaml_wide_mapping_index_preserves_round_trip ... ok
test tests::combinatorics::binomial_respects_integer_work_limit ... ok
test tests::combinatorics::cheap_combinatorics_builtins_integer_anchors ... ok
test tests::combinatorics::cheap_combinatorics_builtins_report_declared_attributes ... ok
test tests::combinatorics::cheap_combinatorics_builtins_thread_and_passthrough_current_scope ... ok
test tests::combinatorics::fibonacci_respects_integer_work_limit_boundaries ... ok
test tests::combinatorics::fibonacci_wrong_arity_reports_argx_and_stays_inert ... ok
test tests::combinatorics::recurrence_combinatorics_respect_integer_work_limit ... ok
test tests::datetime::absolute_time_reads_exact_timestamp_and_composes_with_date_list ... ok
test tests::datetime::date_list_date_string_absolute_time_and_from_date_list_report_arity_errors ... ok
test tests::datetime::date_list_round_trips_exact_rational_timestamps ... ok
test tests::datetime::date_plus_and_date_difference_use_exact_fixed_duration_arithmetic ... ok
test tests::datetime::date_plus_operator_form_rewrites_to_data_first_call ... ok
test tests::datetime::date_string_renders_utc_and_rejects_non_terminating_fractional_seconds ... ok
test tests::datetime::day_of_week_returns_iso_weekday_symbols ... ok
test tests::datetime::from_date_list_and_date_list_epoch_pins_are_utc ... ok
test tests::datetime::from_date_string_accepts_documented_iso_subset_and_offsets ... ok
test tests::datetime::from_date_string_fails_closed_on_invalid_shapes ... ok
test tests::datetime::leap_year_and_month_length_edges_are_validated ... ok
test tests::debug_tools_evaluation::evaluation_data_captures_result_messages_prints_and_failure_type_without_leaking_streams ... ok
test tests::debug_tools_reports::testcreate_testreport_and_stack_cover_current_first_pass_subset ... ok
test tests::debug_tools_reports::testreport_merges_existing_reports_and_accepts_report_level_options ... ok
test tests::debug_tools_resources::memory_constraint_checks_each_repeated_string_growth_step ... ok
test tests::debug_tools_resources::memory_constraint_keeps_projected_string_repeat_precheck ... ok
test tests::debug_tools_resources::monitor_runs_current_display_and_callback_subset ... ok
test tests::debug_tools_resources::sow_and_reap_collect_by_tag_and_optional_handler ... ok
test tests::debug_tools_resources::time_and_memory_constraints_cover_current_first_pass_resource_subset ... ok
test tests::debug_tools_resources::time_constrained_charges_input_suspension_to_original_deadline ... TimeConstrained Input, expired suspension: Ok(SessionEvalResponse { value: Atom(Symbol(Symbol { id: 1384, identity: None, text: SymbolText { id: 1384, context: "Global`", name: "timedOut511" } })), termination: None, messages: [], prints: [], trace: [] })
ok
test tests::debug_tools_resources::time_constrained_immediate_input_resume_finishes_before_deadline ... TimeConstrained Input, immediate resume: Ok(SessionEvalResponse { value: Atom(Integer(Small(5))), termination: None, messages: [], prints: [], trace: [] })
ok
test tests::debug_tools_resources::time_constrained_preserves_deadline_through_two_input_suspensions ... TimeConstrained Input, expired second suspension: Ok(SessionEvalResponse { value: Atom(Symbol(Symbol { id: 1384, identity: None, text: SymbolText { id: 1384, context: "Global`", name: "timedOut511" } })), termination: None, messages: [], prints: [], trace: [] })
ok
test tests::debug_tools_resources::timing_and_absolute_timing_return_elapsed_and_value ... ok
test tests::definition_lhs_assignment_resume::tagset_resumes_rhs_tag_position_and_lhs_once_in_settled_order ... ok
test tests::definition_lhs_assignment_resume::tagunset_resumes_preparation_once_and_removes_only_the_canonical_identity ... ok
test tests::definition_lhs_assignment_resume::unset_lhs_preparation_resumes_exactly_once_without_replay ... ok
test tests::definition_lhs_assignment_resume::upset_resumes_rhs_inferred_tag_and_lhs_once_without_replay ... ok
test tests::definition_lhs_assignment_resume::value_table_failure_after_suspension_keeps_the_old_table_intact ... ok
test tests::definition_lhs_assignment_resume::value_table_lhs_preparation_resumes_multiple_entries_without_replay ... ok
test tests::delete_missing_depth::delete_missing_recursive_armed_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_depth::delete_missing_recursive_armed_shallow_companion_pins_level_and_delayed_head ... ok
test tests::delete_missing_depth::delete_missing_recursive_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_depth::delete_missing_recursive_shallow_companion_pins_level_interpretation ... ok
test tests::delete_missing_depth::delete_missing_scan_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_depth::delete_missing_scan_shallow_companion_pins_scan_depth ... ok
test tests::delete_missing_target_certificates::delete_missing_target_association_clears_64000_siblings_on_two_mib ... ignored, release-only width probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_list_clears_64000_siblings_on_two_mib ... ignored, release-only width probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_scan_all_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_scan_zero_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::delete_missing_target_certificates::delete_missing_target_shallow_companion_pins_level_and_scan ... ok
test tests::delete_missing_target_semantic_locks::cancelling_target_mode_suspension_discards_frames_without_callbacks ... ok
test tests::delete_missing_target_semantic_locks::delayed_rhs_above_target_gets_one_level_minus_one_cleanup_wrapper ... ok
test tests::delete_missing_target_semantic_locks::target_association_evaluates_only_reached_delayed_rhs_once ... ok
test tests::delete_missing_target_semantic_locks::target_association_parent_resumes_two_different_list_branches_in_order ... ok
test tests::delete_missing_target_semantic_locks::target_list_parent_resumes_first_delayed_sibling_without_replay ... ok
test tests::delete_missing_target_semantic_locks::target_list_parent_resumes_last_delayed_sibling_without_prefix_replay ... ok
test tests::delete_missing_target_semantic_locks::target_list_parent_resumes_middle_delayed_sibling_without_prefix_replay ... ok
test tests::delete_missing_target_semantic_locks::target_mode_mixed_association_parent_pins_exact_scan_zero_and_all_shapes ... ok
test tests::delete_missing_target_semantic_locks::target_mode_mixed_list_parent_pins_exact_scan_zero_and_all_shapes ... ok
test tests::delete_missing_target_semantic_locks::target_mode_propagates_non_input_errors_unchanged ... ok
test tests::exact_reduction_shapes::commonest_accepts_count_limit_form ... ok
test tests::exact_reduction_shapes::commonest_preserves_wide_first_occurrence_order ... ok
test tests::exact_reduction_shapes::exact_aggregation_shapes_reduce_arithmetic_terms ... ok
test tests::exact_reduction_shapes::exact_denominator_and_apply_integer_reducers_keep_certificate_shape ... ok
test tests::exact_reduction_shapes::exact_statistics_cover_median_commonest_and_quantile_contracts ... ok
test tests::exact_reduction_shapes::exact_statistics_report_empty_and_invalid_boundaries ... ok
test tests::exact_reduction_shapes::mean_reports_empty_and_invalid_arity_boundaries ... ok
test tests::exact_reduction_shapes::quantile_accepts_parameter_lists_and_fails_closed ... ok
test tests::exact_reduction_shapes::unicode_letter_identifiers_load_as_ordinary_source_symbols ... ok
test tests::functional_ops::apply_map::apply_and_mapapply_cover_operator_forms_levelspecs_and_association_values ... ok
test tests::functional_ops::apply_map::apply_map_family_reports_current_argument_forms ... ok
test tests::functional_ops::arrays::array_and_outer_cover_current_dense_generation_subset ... ok
test tests::functional_ops::arrays::array_and_outer_respect_resource_limits ... ok
test tests::functional_ops::arrays::array_functional_builtins_report_current_argument_forms ... ok
test tests::functional_ops::arrays::inner_and_dot_dimension_mismatches_report_only_rectangular_same_head_cases ... ok
test tests::functional_ops::arrays::inner_arrayfilter_and_arrayreduce_cover_current_subset ... ok
test tests::functional_ops::callable::callable_functional_builtins_report_current_argument_forms ... ok
test tests::functional_ops::callable::operate_supports_head_chain_updates_and_operator_form ... ok
test tests::functional_ops::incidental_apply_map::map_scan_and_comap_family_cover_current_first_pass_subset ... ok
test tests::functional_ops::incidental_callable::through_threads_arguments_through_current_head_subset ... ok
test tests::functional_ops::iteration::fixed_point_cycle_tracking_covers_value_and_list_results ... ok
test tests::functional_ops::iteration::iteration_builtins_report_current_argument_forms ... ok
test tests::functional_ops::iteration::iteration_lists_preserve_wide_lengths_and_callback_counts ... ok
test tests::functional_ops::iteration::nest_and_fixed_point_families_cover_current_iteration_subset ... ok
test tests::functional_ops::sequence::sequence_functional_builtins_report_current_argument_forms ... ok
test tests::functional_ops::targeted_map::map_at_evaluates_each_target_once_at_scale ... ok
test tests::functional_ops::targeted_map::mapat_mapthread_and_subsetmap_cover_current_first_pass_subset ... ok
test tests::functional_ops::targeted_map::subset_map_validates_wide_disjoint_paths_at_scale ... ok
test tests::functional_ops::targeted_map::targeted_mapping_builtins_report_current_argument_forms ... ok
test tests::functional_ops::targeted_map::targeted_mapping_reports_only_explicit_missing_or_incompatible_paths ... ok
test tests::held_association_exact_path_depth::associate_to_part_target_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::held_association_exact_path_depth::held_association_part_three_level_companion_pins_selectors_order_and_results ... ok
test tests::held_association_exact_path_depth::key_drop_from_part_target_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::held_association_exact_path_depth::shared_exact_path_consumers_keep_exact_shallow_behavior ... ok
test tests::holding_and_control::evaluate_forces_evaluation_inside_held_arguments ... ok
test tests::holding_and_control::holdallcomplete_blocks_upvalue_application ... ok
test tests::holding_and_control::holdallcomplete_prevents_evaluate_override_and_sequence_splicing ... ok
test tests::holding_and_control::holdpattern_preserves_patterns_without_prior_evaluation ... ok
test tests::holding_and_control::if_evaluates_only_the_selected_branch ... ok
test tests::holding_and_control::protect_and_unprotect_round_trip_and_leave_attribute_updates_available ... ok
test tests::holding_and_control::releasehold_releases_supported_hold_wrappers ... ok
test tests::holding_and_control::sequence_splices_in_non_held_and_ordinary_held_calls ... ok
test tests::holding_and_control::sequencehold_prevents_sequence_splicing ... ok
test tests::holding_and_control::setattributes_accepts_resolved_attribute_specs ... ok
test tests::holding_and_control::setattributes_accepts_symbol_lists_and_combined_hold_attributes ... ok
test tests::holding_and_control::setattributes_and_clearattributes_control_argument_evaluation ... ok
test tests::holding_and_control::unevaluated_consumers_still_receive_the_raw_unwrapped_argument ... ok
test tests::holding_and_control::unevaluated_is_restored_only_when_no_call_stage_consumes_it ... ok
test tests::holding_and_control::unevaluated_sequence_splicing_drops_single_position_provenance ... ok
test tests::holding_and_control::unevaluated_temporarily_preserves_direct_function_arguments ... ok
test tests::holding_and_control::user_defined_holdallcomplete_attribute_holds_all_arguments_completely ... ok
test tests::holding_and_control::which_evaluates_conditions_in_order ... ok
test tests::holding_and_control_abort::abortprotect_and_failsafe_resume_through_host_input ... ok
test tests::holding_and_control_abort::abortprotect_assert_and_blockrandom_cover_current_first_pass_subset ... ok
test tests::holding_and_control_cleanup::withcleanup_defers_cleanup_until_suspended_body_finishes ... ok
test tests::holding_and_control_failure_edges::failure_symbolic_property_and_predicate_edges_cover_current_subset ... ok
test tests::holding_and_control_failure_properties::failure_property_accessors_cover_current_subset ... ok
test tests::holding_and_control_part3::block_applies_dynamic_scope_across_called_definitions ... ok
test tests::holding_and_control_part3::block_supports_sequential_initializers_and_nested_shadowing ... ok
test tests::holding_and_control_part3::block_temporarily_rebinds_and_restores_symbol_values ... ok
test tests::holding_and_control_part3::catch_throw_and_return_support_first_pass_nonlocal_control_flow ... ok
test tests::holding_and_control_part3::compound_expression_evaluates_sequentially_and_returns_the_last_value ... ok
test tests::holding_and_control_part3::configurable_replace_repeated_limit_is_respected ... ok
test tests::holding_and_control_part3::module_generates_fresh_lexical_symbols_and_initializes_sequentially ... ok
test tests::holding_and_control_part3::module_global_definition_escape_retains_gensym_definition ... ok
test tests::holding_and_control_part3::module_leaf_definition_gc_reclaims_loop_local_gensyms ... ok
test tests::holding_and_control_part3::module_local_assignment_invalidates_after_prior_read ... ok
test tests::holding_and_control_part3::module_respects_nested_shadowing_and_pattern_locals ... ok
test tests::holding_and_control_part3::return_is_consumed_at_definition_application_boundaries ... ok
test tests::holding_and_control_part3::scoping_builtins_accept_canonical_raw_binding_lists ... ok
test tests::holding_and_control_part3::scoping_builtins_require_raw_binding_lists_instead_of_resolved_variables ... ok
test tests::holding_and_control_part3::staged_with_fails_closed_on_a_malformed_middle_declaration_list ... ok
test tests::holding_and_control_part3::switch_matches_against_held_forms_and_residualizes_when_nothing_matches ... ok
test tests::holding_and_control_part3::throw_through_block_restores_dynamic_scope_before_catch_handles_it ... ok
test tests::holding_and_control_part3::with_evaluates_bindings_and_substitutes_lexically ... ok
test tests::holding_and_control_part3::with_respects_pattern_local_names_but_substitutes_pattern_metadata ... ok
test tests::holding_and_control_part3::with_supports_rule_style_bindings_and_nested_shadowing ... ok
test tests::holding_and_control_part3::with_supports_staged_declaration_lists ... ok
test tests::holding_and_control_part3::with_zero_and_one_argument_forms_keep_exact_arity_diagnostics ... ok
test tests::holding_and_control_part4::break_and_continue_are_handled_by_current_loop_forms ... ok
test tests::holding_and_control_part4::do_supports_count_iterators_and_nested_iterators ... ok
test tests::holding_and_control_part4::return_propagates_through_loop_bodies_to_function_boundaries ... ok
test tests::holding_and_control_part4::while_until_and_for_support_first_pass_loop_control ... ok
test tests::holding_and_control_part5::confirm_info_is_lazy_and_explicit_tags_are_dynamically_caught ... ok
test tests::holding_and_control_part5::do_and_table_respect_resource_limits ... ok
test tests::holding_and_control_part5::iteration_product_scope_resets_between_statements_and_after_errors ... ok
test tests::holding_and_control_part5::iterator_dynamic_values_interact_with_nested_block_shadowing ... ok
test tests::holding_and_control_part5::iterator_lhs_normalization_preserves_lazy_and_container_lvalue_controls ... ok
test tests::holding_and_control_part5::iterator_spec_matrix_covers_counts_lists_ranges_steps_and_rejections ... ok
test tests::holding_and_control_part5::iterator_variables_expose_dynamic_ownvalues_without_leaking_assignments ... ok
test tests::holding_and_control_part5::large_do_count_loop_does_not_recurse_through_the_rust_stack ... ok
test tests::holding_and_control_part5::large_iteration_products_still_refuse ... ok
test tests::holding_and_control_part5::multi_iterator_control_siblings_charge_iteration_product ... ok
test tests::holding_and_control_part5::multi_iterator_table_charges_the_generated_product ... ok
test tests::holding_and_control_part5::multi_iterator_table_precharges_exact_arithmetic_counts ... ok
test tests::holding_and_control_part5::nested_iteration_constructs_share_the_running_product ... ok
test tests::holding_and_control_part5::non_retaining_consumers_do_not_propagate_their_product_into_table ... ok
test tests::holding_and_control_part5::ordinary_nested_iteration_results_are_unchanged ... ok
test tests::holding_and_control_part5::sum_and_product_leave_unsupported_specs_unevaluated_with_argform ... ok
test tests::holding_and_control_part5::sum_and_product_report_declared_attributes ... ok
test tests::holding_and_control_part5::sum_and_product_reuse_concrete_iterator_specs ... ok
test tests::holding_and_control_part5::suspended_nested_loop_retains_and_then_resets_the_running_product ... ok
test tests::holding_and_control_part5::table_memory_constraint_uses_projected_list_size_before_materialization ... ok
test tests::holding_and_control_part6::abort_and_checkabort_cover_current_first_pass_subset ... ok
test tests::holding_and_control_part6::checkabort_validates_entire_option_lists_before_evaluating_body ... ok
test tests::holding_and_control_part6::cleanup_throw::catch_evaluates_symbol_valued_tag_form_before_matching ... ok
test tests::holding_and_control_part6::cleanup_throw::catch_tag_form_keeps_visible_side_effects ... ok
test tests::holding_and_control_part6::cleanup_throw::uncaught_throw_request_messages_follow_nocatch_shape ... ok
test tests::holding_and_control_part6::cleanup_throw::withcleanup_runs_cleanup_on_values_throws_and_aborts ... ok
test tests::holding_and_control_part6::exit_reports_a_request_level_code_and_stops_evaluation ... ok
test tests::holding_and_control_part6::top_level_break_and_continue_report_nofwd_messages ... ok
test tests::holding_and_control_part8::assert_function_none_restores_normal_assert_control ... ok
test tests::holding_and_control_part8::confirm_information_can_suspend_and_resume_inside_enclose ... ok
test tests::holding_and_control_part8::confirm_predicate_and_test_can_suspend_and_resume_inside_enclose ... ok
test tests::holding_and_control_part8::enclose_handler_can_suspend_and_resume_through_host_input ... ok
test tests::holding_and_control_success::missingq_and_success_cover_current_subset ... ok
test tests::incidental_arithmetic::arithmetic_builtins_report_declared_attributes ... ok
test tests::incidental_arithmetic::max_min_handle_exact_extrema_lists_and_registration ... ok
test tests::incidental_arithmetic::numerator_mirrors_denominator_for_exact_numbers_and_registration ... ok
test tests::incidental_arithmetic::power_over_times_distributes_numeric_exponents ... ok
test tests::incidental_arithmetic::power_times_combine_like_bases_and_terms ... ok
test tests::incidental_associations_part2::group_by_nested_specs_recurse_through_lists_and_associations ... ok
test tests::incidental_associations_part2::group_by_rule_specs_transform_group_values ... ok
test tests::incidental_associations_part2::group_by_wide_list_nested_and_reducer_phases_preserve_order_and_call_counts ... ok
test tests::incidental_context_and_loader::path_variables_and_file_name_helpers_cover_current_script_environment_subset ... ok
test tests::incidental_debug_tools_dialogs::dialog_catches_return_and_respects_current_option_subset ... ok
test tests::incidental_debug_tools_dialogs::dialog_uses_current_default_options_from_setoptions ... ok
test tests::incidental_debug_tools_dialogs::dialog_without_body_uses_a_current_single_input_line_subset ... ok
test tests::incidental_debug_tools_resources::byte_count_and_memory_in_use_use_the_current_structural_proxy_subset ... ok
test tests::incidental_debug_tools_resources::max_memory_used_tracks_session_memory_proxy_high_water_mark ... ok
test tests::incidental_debug_tools_resources::memory_in_use_accounts_for_removed_symbol_identity_state ... ok
test tests::incidental_definitions_and_rules_cases::cases_and_deletecases_use_value_only_association_traversal ... ok
test tests::incidental_definitions_and_rules_lifecycle::clear_and_clearall_manage_symbol_lifecycle_state ... ok
test tests::incidental_definitions_and_rules_lifecycle::definition_lhs_evaluates_call_heads_before_storing_rules ... ok
test tests::incidental_definitions_and_rules_lifecycle::definition_lhs_evaluates_non_pattern_constants_before_storing_rules ... ok
test tests::incidental_definitions_and_rules_lifecycle::definition_lhs_evaluates_outer_call_head_before_selecting_definition_owner ... ok
test tests::incidental_definitions_and_rules_lifecycle::protected_symbols_block_definition_and_clear_writes ... ok
test tests::incidental_definitions_and_rules_lifecycle::remove_clears_current_symbol_state_and_declared_stubs ... ok
test tests::incidental_definitions_and_rules_lifecycle::remove_marks_missing_same_evaluation_symbols_stale ... ok
test tests::incidental_definitions_and_rules_lifecycle::remove_marks_same_evaluation_symbols_stale_and_allows_later_reuse ... ok
test tests::incidental_definitions_and_rules_lifecycle::remove_reports_protected_symbols_without_clearing_state ... ok
test tests::incidental_definitions_and_rules_lifecycle::set_list_destructuring_assigns_leaf_targets ... ok
test tests::incidental_definitions_and_rules_part2::default_pattern_syntax_lowers_to_optional_patterns ... ok
test tests::incidental_definitions_and_rules_part2::matcher_pattern_arguments_resolve_ordinary_subexpressions ... ok
test tests::incidental_definitions_and_rules_part2::single_dot_default_pattern_syntax_lowers_to_optional ... ok
test tests::incidental_definitions_and_rules_patterns::arithmetic_defaults_and_promoted_matching_survive_snapshot_roundtrip ... ok
test tests::incidental_definitions_and_rules_patterns::arithmetic_one_identity_optional_idioms_use_installed_defaults ... ok
test tests::incidental_definitions_and_rules_patterns::compiled_downvalue_matcher_falls_back_for_condition_patterns ... ok
test tests::incidental_definitions_and_rules_patterns::install_seeds_mutable_arithmetic_head_defaults ... ok
test tests::incidental_definitions_and_rules_patterns::nested_compound_optional_idiom_remains_unmatched_at_the_power_boundary ... ok
test tests::incidental_definitions_and_rules_patterns::orderless_sequence_backtracking_depth_limit_terminates_gracefully ... ok
test tests::incidental_definitions_updates::apply_to_rechecks_protection_at_store_time ... ok
test tests::incidental_definitions_updates::held_update_builtins_mutate_immediate_symbol_values ... ok
test tests::incidental_definitions_updates::held_update_invalid_selector_diagnostics_use_the_evaluated_offender ... ok
test tests::incidental_definitions_updates::held_update_part_selectors_rederive_owner_state_after_callbacks ... ok
test tests::incidental_definitions_updates::held_update_part_selectors_run_left_to_right_after_binary_rhs_preparation ... ok
test tests::incidental_definitions_updates::held_update_syntax_mutates_module_locals ... ok
test tests::incidental_definitions_updates::held_updates_allow_empty_bulk_part_targets_without_mutation ... ok
test tests::incidental_definitions_updates::held_updates_cover_duplicate_bulk_part_targets ... ok
test tests::incidental_definitions_updates::held_updates_evaluate_rebound_part_selectors_for_all_eleven_heads ... ok
test tests::incidental_definitions_updates::held_updates_mutate_bulk_exact_part_targets ... ok
test tests::incidental_definitions_updates::held_updates_mutate_single_exact_part_targets ... ok
test tests::incidental_definitions_updates::held_updates_reject_nested_part_selector_lists_without_mutation ... ok
test tests::incidental_definitions_updates::held_updates_report_missing_or_invalid_targets_without_writing ... ok
test tests::incidental_definitions_updates::held_updates_respect_protected_symbols ... ok
test tests::incidental_holding_and_control_part2::cycle_detection_allows_same_call_after_state_progress ... ok
test tests::incidental_holding_and_control_part3::module_cross_gensym_definition_retains_transitive_dependency ... ok
test tests::incidental_holding_and_control_part3::module_nested_and_error_paths_remain_conservative ... ok
test tests::incidental_holding_and_control_part3::module_options_and_sow_escapes_retain_gensym_definitions ... ok
test tests::incidental_holding_and_control_part3::module_result_escape_retains_gensym_definition ... ok
test tests::incidental_holding_and_control_part5::confirm_quiet_catches_and_quiets_matching_messages ... ok
test tests::incidental_holding_and_control_part5::table_propagates_return_to_function_boundaries ... ok
test tests::incidental_holding_and_control_part5::table_supports_count_iterators_and_nested_iterators ... ok
test tests::incidental_holding_and_control_part6::failsafe_wrapper_preserves_failure_like_inputs_and_supports_test_forms ... ok
test tests::incidental_holding_and_control_part8::confirm_family_info_can_suspend_and_resume_inside_enclose ... ok
test tests::incidental_holding_and_control_part8::generated_confirmation_failures_expose_current_oracle_metadata_slice ... ok
test tests::incidental_lowering_and_context::applies_at_sugar_through_lowering_and_evaluation ... ok
test tests::incidental_lowering_and_context::functional_operator_sugars_flow_through_lowering_and_first_pass_runtime ... ok
test tests::incidental_nothing::association_values_keep_nothing_but_list_inputs_drop_it ... ok
test tests::incidental_nothing::nothing_is_removed_from_direct_list_elements_only ... ok
test tests::incidental_options_and_protection::setoptions_updates_system_heads_defaults_for_cases_position_and_depth ... ok
test tests::incidental_package_tools::dump_save_restore_preserves_registered_core_builtins ... ok
test tests::incidental_predicates::predicate_q_family_covers_current_exact_and_structural_subsets ... ok
test tests::incidental_predicates::unsame_q_wide_index_detects_late_duplicates ... ok
test tests::incidental_print_sink::print_sink_streams_prints_and_leaves_outcome_prints_empty ... ok
test tests::incidental_print_sink::prints_accumulate_without_sink ... ok
test tests::incidental_scoping_unique::unique_supports_list_and_attribute_forms ... ok
test tests::incidental_session_requests_debug_tooling_resume::resume_eval_request_with_input_can_finish_constraint_limit_and_fallback ... ok
test tests::incidental_session_requests_debug_tooling_resume::resume_eval_request_with_input_can_finish_test_report_input_and_sametest ... ok
test tests::incidental_session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_while_sequence_helpers ... ok
test tests::incidental_session_requests_input_resume::definition_lhs_preparation_resumes_a_callable_head_exactly_once ... ok
test tests::incidental_session_requests_input_resume::definition_lhs_preparation_resumes_multiple_arguments_left_to_right_without_replay ... ok
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_can_finish_select_predicate ... ok
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_finishes_evaluation_data_capture ... ok
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_preserves_do_iterator_scope_until_completion ... ok
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_preserves_logical_sequence_state ... ok
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_preserves_ordering_comparator_state ... ok
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_preserves_predicate_traversal_state ... ok
test tests::incidental_session_requests_input_resume::resume_eval_request_with_input_preserves_prepared_lookup_across_default_and_wrapper ... ok
test tests::incidental_session_requests_input_resume::resumed_set_rhs_normalizes_iterator_lhs_once_and_restores_clean_scope ... ok
test tests::incidental_session_requests_matcher_resume::resume_eval_request_with_input_can_finish_conditioned_cases_over_association_values_only ... ok
test tests::incidental_session_requests_transforms_resume::resume_eval_request_with_input_can_finish_deletecases_after_deleting_an_association_head ... ok
test tests::incidental_session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_associationmap_over_association_entries ... ok
test tests::incidental_session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_keysort_after_suspending_comparer ... ok
test tests::incidental_session_requests_transforms_resume_associations::resume_eval_request_with_input_preserves_associationmap_list_progress ... ok
test tests::incidental_session_requests_transforms_resume_associations::resume_eval_request_with_input_preserves_keyselect_progress ... ok
test tests::incidental_structure_ops_ordering_sets::sort_with_ordering_function_evaluates_comparator ... ok
test tests::incidental_structure_ops_replace::replaceat_supports_sequential_paths_all_and_keyed_positions ... ok
test tests::incidental_structure_ops_replace::replacepart_bare_association_keys_remain_nonselecting_position_patterns ... ok
test tests::incidental_structure_ops_replace::replacepart_rhs_guards_cover_keyed_association_position_forms ... ok
test tests::incidental_structure_ops_replace::replacepart_supports_keyed_association_paths ... ok
test tests::incidental_structure_ops_selectors_part2::extract_preserves_nested_key_and_empty_selector_shapes ... ok
test tests::incidental_structure_ops_selectors_part2::part_and_extract_head_access_preserve_symbol_diagnostics ... ok
test tests::incidental_structure_ops_surface::arithmetic_builtins_thread_over_lists ... ok
test tests::incidental_structure_ops_surface::first_last_rest_and_most_support_defaults_and_empty_calls ... ok
test tests::incidental_structure_ops_surface::flatten_reports_current_invalid_boundaries ... ok
test tests::incidental_structure_ops_surface::fullform_evaluates_its_argument_but_preserves_inner_hold ... ok
test tests::incidental_structure_ops_surface::head_length_and_depth_cover_atoms_calls_and_compound_heads ... ok
test tests::incidental_structure_ops_surface::hold_form_holds_argument_and_fullform_shows_wrapper ... ok
test tests::incidental_thread_and_dispatch::dispatch_stage_guards_preserve_value_and_builtin_semantics ... ok
test tests::installs_identity_builtin ... ok
test tests::integer_digits::integer_digit_builtins_integer_anchors ... ok
test tests::integer_digits::integer_digit_builtins_report_declared_attributes ... ok
test tests::integer_digits::integer_digit_builtins_report_invalid_base_and_negative_length ... ok
test tests::integer_digits::integer_digit_builtins_thread_and_passthrough_current_scope ... ok
test tests::integer_digits::integer_digit_valid_bases_and_defaults_stay_message_free ... ok
test tests::iterative_traversal_depth::apply_certificate_companion_pins_descending_selector ... ok
test tests::iterative_traversal_depth::apply_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::association_assignment_certificate_companion_pins_both_key_selector_spellings ... ok
test tests::iterative_traversal_depth::association_assignment_read_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::association_map_certificate_companion_pins_descending_selector ... ok
test tests::iterative_traversal_depth::association_map_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::cases_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::construction_reaches_physical_height_20001_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::count_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::d_nested_sin_completes_at_physical_height_4097_on_two_mib ... ignored, release-only active-spine safety probe; run explicitly with --release
test tests::iterative_traversal_depth::deletecases_active_spine_returns_in_band_resource_error_on_two_mib ... ignored, release-only active-spine safety probe; run explicitly with --release
test tests::iterative_traversal_depth::deletecases_certificate_companion_pins_descending_selector ... ok
test tests::iterative_traversal_depth::deletecases_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::deletecases_limit_pins_post_order_visit_order ... ok
test tests::iterative_traversal_depth::freeq_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::level_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::lexical_substitution_certificate_companion_pins_three_level_body ... ok
test tests::iterative_traversal_depth::lexical_substitution_iterative_tail_keeps_shadow_local_to_nested_with ... ok
test tests::iterative_traversal_depth::lexical_substitution_keeps_nested_with_shadow_local_to_its_subtree ... ok
test tests::iterative_traversal_depth::map_certificate_companion_pins_descending_selector ... ok
test tests::iterative_traversal_depth::map_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::matchq_completes_at_physical_height_20001_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::memberq_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::merge_nested_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::position_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::replace_active_spine_returns_in_band_resource_error_on_two_mib ... ignored, release-only active-spine safety probe; run explicitly with --release
test tests::iterative_traversal_depth::replace_certificate_companion_pins_descending_selector ... ok
test tests::iterative_traversal_depth::replace_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::replaceall_certificate_companion_pins_matching_rule ... ok
test tests::iterative_traversal_depth::replaceall_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::replacepart_deep_path_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::rule_rhs_substitution_certificate_companion_pins_three_level_body ... ok
test tests::iterative_traversal_depth::rule_rhs_substitution_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::take_multispec_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::tostring_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::iterative_traversal_depth::with_substitution_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::json::from_json_depth_cap_fails_before_native_stack_depth ... ok
test tests::json::from_json_maps_nested_values_to_exact_rum_expressions ... ok
test tests::json::from_json_rejects_invalid_json_fail_closed ... ok
test tests::json::from_json_wide_object_key_index_preserves_all_members ... ok
test tests::json::json_codec_supports_transform_round_trip ... ok
test tests::json::to_json_encodes_supported_subset_compactly_and_pretty ... ok
test tests::json::to_json_rejects_unsupported_values_fail_closed ... ok
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
test tests::logic::comparison::and_or_short_circuit_side_effects_and_compare_exact_values ... ok
test tests::logic::comparison::comparison_operator_forms_apply_single_arguments_on_the_left ... ok
test tests::logic::comparison::equal_and_unequal_leave_mixed_symbolic_chains_intact ... ok
test tests::logic::comparison::unequal_wide_value_index_detects_late_duplicates ... ok
test tests::logic::comparison::wide_numeric_order_chains_reduce_in_one_pass ... ok
test tests::logic::inequality::comparison_parentheses_bound_chain_full_forms ... ok
test tests::logic::inequality::inequality_binary_symbolic_and_attribute_contracts_are_explicit ... ok
test tests::logic::inequality::mixed_comparison_chain_does_not_short_circuit_operand_evaluation ... ok
test tests::logic::inequality::mixed_comparison_chain_evaluates_middle_operand_once ... ok
test tests::logic::inequality::mixed_comparison_chains_have_alternating_inequality_full_forms ... ok
test tests::logic::inequality::parenthesized_and_unparenthesized_equality_results_remain_distinct ... ok
test tests::logic::inequality::uniform_less_chain_evaluates_every_operand_once ... ok
test tests::logic::inequality::uniform_less_chain_keeps_its_nary_head ... ok
test tests::logic::part2::equal_and_unequal_handle_boolean_pairs_without_overreaching ... ok
test tests::logic::part2::equal_reduces_adjacent_mixed_comparable_pairs_only ... ok
test tests::logic::part2::equality_operator_forms_accept_multi_argument_applications ... ok
test tests::logic::part2::order_comparison_operator_forms_report_application_arity ... ok
test tests::logic::part2::unequal_detects_repeated_known_prefix_values_before_unknown_tail ... ok
test tests::logic::session_requests::logic_short_circuit_can_avoid_or_resume_input_requests ... ok
test tests::logic::session_requests::truth_aggregates_short_circuit_and_resume_input_requests ... ok
test tests::logic::truth::and_or_not_respect_current_short_circuit_and_identity_semantics ... ok
test tests::logic::truth::wide_xor_and_equivalent_indexes_preserve_parity_and_uniqueness ... ok
test tests::map_indexed_association_depth::map_indexed_association_completes_at_physical_height_4097_on_two_mib ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::map_indexed_association_depth::map_indexed_association_heads_refuses_in_band_rather_than_terminating ... ignored, release-only deployment-stack probe; run explicitly with --release
test tests::map_indexed_association_depth::map_indexed_association_three_level_companion_pins_positions_and_levelspec ... ok
test tests::merge_native_stack_guard::merge_zero_native_stack_boundary_fails_in_band_and_session_recovers ... ignored, requires rum-runtime/test-native-stack-guard-zero
test tests::number_theory::factorization_builtins_integer_anchors ... ok
test tests::number_theory::factorization_builtins_report_declared_attributes ... ok
test tests::number_theory::factorization_builtins_thread_and_passthrough_like_current_scope ... ok
test tests::number_theory::factorization_respects_integer_work_limit ... ok
test tests::number_theory::large_semiprime_factorization_uses_pollard_rho_path ... ok
test tests::number_theory::next_prime_follows_signed_prime_order ... ok
test tests::number_theory::prime_and_prime_pi_respect_integer_work_limit ... ok
test tests::number_theory::prime_reports_positive_integer_message ... ok
test tests::options_suspension::default_snapshot_survives_guest_options_mutation_and_resume ... ok
test tests::options_suspension::delayed_string_replace_option_resumes_into_consumer ... queued="b"; resumed="b"
ok
test tests::options_suspension::inventory::option_reentry_descriptor_inventory_is_exhaustive ... ok
test tests::options_suspension::inventory::production_option_reentry_implementations_are_exact ... ok
test tests::options_suspension::nested_option_consumers_resume_into_their_own_calls ... ok
test tests::options_suspension::option_consumer_can_suspend_again_in_delayed_replacement ... ok
test tests::options_suspension::option_error_after_resume_propagates_without_running_the_consumer ... ok
test tests::options_suspension::option_loop_preserves_prefix_and_side_effect_order_across_repeated_suspensions ... ok
test tests::options_suspension::option_suspension_cancellation_abandons_remaining_guest_work ... ok
test tests::options_suspension::remaining::aligncolumns_resumes_into_consumer ... consumer=AlignColumns; queued=resumed="a   |b\nlong|c"; default+explicit suspended
ok
test tests::options_suspension::remaining::complement_resumes_into_consumer ... consumer=Complement; queued=resumed=List[]; default+explicit suspended
ok
test tests::options_suspension::remaining::containsall_resumes_into_consumer ... consumer=ContainsAll; queued=resumed=List[True, List[compared]]; default+explicit suspended
ok
test tests::options_suspension::remaining::containsany_resumes_into_consumer ... consumer=ContainsAny; queued=resumed=List[True, List[compared]]; default+explicit suspended
ok
test tests::options_suspension::remaining::containsexactly_resumes_into_consumer ... consumer=ContainsExactly; queued=resumed=List[True, List[compared]]; default+explicit suspended
ok
test tests::options_suspension::remaining::containsnone_resumes_into_consumer ... consumer=ContainsNone; queued=resumed=List[False, List[compared]]; default+explicit suspended
ok
test tests::options_suspension::remaining::containsonly_resumes_into_consumer ... consumer=ContainsOnly; queued=resumed=List[True, List[compared]]; default+explicit suspended
ok
test tests::options_suspension::remaining::disjointq_resumes_into_consumer ... consumer=DisjointQ; queued=resumed=List[False, List[compared]]; default+explicit suspended
ok
test tests::options_suspension::remaining::duplicatefreeq_resumes_into_consumer ... consumer=DuplicateFreeQ; queued=resumed=List[False, List[compared]]; default+explicit suspended
ok
test tests::options_suspension::remaining::htmlunescape_resumes_into_consumer ... consumer=HtmlUnescape; queued=resumed="&"; default+explicit suspended
ok
test tests::options_suspension::remaining::intersectingq_resumes_into_consumer ... consumer=IntersectingQ; queued=resumed=List[True, List[compared]]; default+explicit suspended
ok
test tests::options_suspension::remaining::intersection_resumes_into_consumer ... consumer=Intersection; queued=resumed=List[1]; default+explicit suspended
ok
test tests::options_suspension::remaining::parseargs_resumes_into_consumer ... consumer=ParseArgs; queued=resumed=Association[Rule["Positional", List["tail"]], Rule["Options", Association[]], Rule["Flags", List["a", "b"]]]; default+explicit suspended
ok
test tests::options_suspension::remaining::stringindent_resumes_into_consumer ... consumer=StringIndent; queued=resumed=">a\n>\n>b"; default+explicit suspended
ok
test tests::options_suspension::remaining::stringwrap_resumes_into_consumer ... consumer=StringWrap; queued=resumed="a\nb"; default+explicit suspended
ok
test tests::options_suspension::remaining::subsetq_resumes_into_consumer ... consumer=SubsetQ; queued=resumed=List[True, List[compared]]; default+explicit suspended
ok
test tests::options_suspension::remaining::templateapply_resumes_into_consumer ... consumer=TemplateApply; queued=resumed="Ada"; default+explicit suspended
ok
test tests::options_suspension::remaining::uncompress_resumes_into_consumer ... consumer=Uncompress; queued=resumed=List[a, b]; default+explicit suspended
ok
test tests::options_suspension::remaining::union_resumes_into_consumer ... consumer=Union; queued=resumed=List[1]; default+explicit suspended
ok
test tests::options_suspension::remaining::urldecode_resumes_into_consumer ... consumer=UrlDecode; queued=resumed="a b"; default+explicit suspended
ok
test tests::options_suspension::remaining::urlencode_resumes_into_consumer ... consumer=UrlEncode; queued=resumed="a+b"; default+explicit suspended
ok
test tests::options_suspension::remaining::urlparse_resumes_into_consumer ... consumer=UrlParse; queued=resumed=Association[Rule["Scheme", "https"], Rule["Host", "example.com"], Rule["Port", Null], Rule["Path", "/a"], Rule["Query", Null], Rule["Fragment", Null]]; default+explicit suspended
ok
test tests::options_suspension::string_option_consumers_finish_after_actual_suspension ... consumer=StringPosition; queued=resumed=List[List[1, 1], List[2, 2]]; default+explicit suspended
ok
test tests::ownership_graph::accumulator_coverage::every_self_reaching_arc_chain_participates ... uncovered accumulator ownership paths: 0
ok
test tests::ownership_graph::accumulator_coverage::fresh_names_and_additional_self_edges_are_checked ... ok
test tests::ownership_graph::discovery_report_without_inventory_input ... UnknownHostPayload count: 63
ok
test tests::ownership_graph::fixtures::coverage_is_not_inferred_from_test_like_filenames ... ok
test tests::ownership_graph::fixtures::fresh_names_cross_module_pair_and_recursion_after_comma ... FIXTURE REDISCOVERED fixture::AfterComma	crates/fixture/src/lib.rs	6	fixture::AfterComma.table/std::collections::HashMap[Own]/arg1/tuple/.1/std::boxed::Box[Own]/arg0 -> fixture::AfterComma
ok
test tests::ownership_graph::fixtures::inline_module_path_attribute_and_super_imports_resolve ... ok
test tests::ownership_graph::fixtures::manually_checked_definition_counts ... HAND CROSS-CHECK crates/rum-stdlib/src/associations/basic.rs: 3 definitions: LookupOutcome, LookupPlan, PreparedLookup
ok
test tests::ownership_graph::fixtures::named_generic_public_inputs_are_opaque_but_borrowed_inputs_are_not ... ok
test tests::ownership_graph::fixtures::opaque_constructs_are_located_and_inventory_removal_fails ... DELIBERATE UNKNOWN FAILURE (caught): missing inventory row: UnknownHostPayload	crates/fixture/src/lib.rs	3	31	fixture::Opaque.host/std::boxed::Box[Own]/arg0: opaque owned payload dyn Payload
ok
test tests::ownership_graph::fixtures::owning_ports_and_nonowning_storage_do_not_mix_instantiations ... ok
test tests::ownership_graph::fixtures::rediscovery_and_each_missing_mutual_member ... REDISCOVERED rum_stdlib::associations::basic::LookupOutcome	crates/rum-stdlib/src/associations/basic.rs	302	rum_stdlib::associations::basic::LookupOutcome.List.0/std::vec::Vec[Own]/arg0 -> rum_stdlib::associations::basic::LookupOutcome
ok
test tests::ownership_graph::fixtures::resolution_uses_declarations_not_container_spellings ... ok
test tests::ownership_graph::fixtures::unknown_configuration_cannot_invent_an_unconditional_cycle ... ok
test tests::ownership_graph::fixtures::unmodeled_lexical_modules_and_async_captures_are_explicit_unknowns ... ok
test tests::ownership_graph::host_payload_ledger::ownership::attribution_resolves_alias_scopes_and_excludes_nonowning_paths ... ok
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... HOST PAYLOAD LEDGER: 63 rows; 6 storage edges (3 Declared, 3 Unbounded); 57 PublicInput
ok
test tests::ownership_graph::host_payload_ledger::tests::metadata_duplicates_and_witnesses_are_enforced ... ok
test tests::ownership_graph::host_payload_ledger::tests::missing_stale_and_unrelated_expression_debt_are_rejected ... ok
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
ok
test tests::ownership_graph::source::root_set::syntax_root_discovers_added_recursive_type_and_restores_source ... OLD ROOTS: rum_syntax::ast::ScannerRootSetProbe invisible (no definition or self-reaching row)
ok
test tests::polynomials::coefficient_extracts_exact_monomial_powers_without_requiring_manual_expand ... ok
test tests::polynomials::coefficient_family_fails_closed_on_leaked_laurent_target_powers ... ok
test tests::polynomials::coefficient_family_keeps_current_pattern_bearing_boundary_for_now ... ok
test tests::polynomials::coefficient_family_keeps_non_laurent_residues_and_polynomials ... ok
test tests::polynomials::coefficient_list_builds_rectangular_grids_in_the_requested_variable_order ... ok
test tests::polynomials::expand_distributes_products_and_positive_integer_powers ... ok
test tests::polynomials::expand_preserves_non_polynomial_or_pattern_bearing_regions ... ok
test tests::polynomials::expand_supports_rational_coefficients_and_indexed_indeterminates ... ok
test tests::polynomials::factor_extracts_content_common_monomials_and_linear_factors_over_q ... ok
test tests::polynomials::factor_handles_symmetric_multivariate_cubic_example ... ok
test tests::polynomials::factor_keeps_current_pattern_bearing_boundary_for_now ... ok
test tests::polynomials::polynomial_predicates_cover_current_expression_subsets ... ok
test tests::polynomials::together_combines_rational_function_terms_over_common_polynomial_denominators ... ok
test tests::polynomials::together_keeps_current_pattern_bearing_boundary_for_now ... ok
test tests::polynomials::together_now_cancels_common_polynomial_factors_after_combining ... ok
test tests::polynomials::together_preserves_reduced_canonical_rational_function_forms ... ok
test tests::polynomials::wide_expand_batches_sum_and_product_term_normalization ... ok
test tests::polynomials::wide_factor_collections_preserve_polynomial_shapes ... ok
test tests::polynomials::wide_factor_common_monomial_and_exact_division_preserve_expansion ... ok
test tests::polynomials::wide_monomial_products_normalize_factors_once ... ok
test tests::polynomials::wide_order_preserving_polynomial_transforms_keep_canonical_shapes ... ok
test tests::predicates::collection_predicates_accept_same_test_options ... ok
test tests::predicates::collection_same_test_pairs_run_once_each_without_native_stack_recursion ... ok
test tests::predicates::contains_exactly_custom_same_test_uses_polynomial_matching ... ok
test tests::predicates::list_relation_and_shape_predicates_cover_current_exact_subsets ... ok
test tests::predicates::ordering_predicates_accept_custom_comparators ... ok
test tests::predicates::predicate_operator_forms_are_inert_and_apply_to_data ... ok
test tests::predicates::truth_aggregates_and_coprime_q_cover_current_subsets ... ok
test tests::predicates_part2::negative_power_order_compares_bases_before_exponents ... ok
test tests::predicates_part2::order_places_negative_powers_before_their_bases ... ok
test tests::preflight_memory::projected_preflight_builtins_refuse_before_allocation_and_admit_realistic_inputs ... ok
test tests::random::block_random_does_not_restore_random_password_entropy ... ok
test tests::random::block_random_restores_the_prng_stream ... ok
test tests::random::block_random_restores_the_random_string_stream ... ok
test tests::random::character_class_constants_are_protected ... ok
test tests::random::character_class_constants_have_pinned_values_and_compose ... ok
test tests::random::random_choice_and_sample_cover_current_list_contracts ... ok
test tests::random::random_choice_weighted_forms_use_exact_weights_and_fail_closed ... ok
test tests::random::random_family_heads_report_declared_attributes ... ok
test tests::random::random_integer_and_choice_accept_nested_output_dimensions ... ok
test tests::random::random_integer_default_seed_sequence_is_pinned ... ok
test tests::random::random_integer_respects_large_exact_ranges ... ok
test tests::random::random_password_combined_policy_uses_whole_password_rejection ... ok
test tests::random::random_password_does_not_advance_the_prng_stream ... ok
test tests::random::random_password_exclude_policy_rejects_empty_substrings ... ok
test tests::random::random_password_exclude_policy_resamples_and_can_ignore_case ... ok
test tests::random::random_password_fixed_entropy_uses_rejection_sampling ... ok
test tests::random::random_password_no_adjacent_repeats_policy_resamples ... ok
test tests::random::random_password_policy_cap_exhaustion_fails_closed ... ok
test tests::random::random_password_policy_options_are_discoverable_without_changing_draws ... ok
test tests::random::random_password_policy_rejects_non_positive_length ... ok
test tests::random::random_password_require_policy_rejects_malformed_groups ... ok
test tests::random::random_password_require_policy_resamples_until_groups_are_present ... ok
test tests::random::random_password_respects_custom_alphabet_membership ... ok
test tests::random::random_string_alphabet_duplicates_weight_draws ... ok
test tests::random::random_string_default_seed_sequence_is_pinned ... ok
test tests::random::random_string_draws_from_unicode_code_points ... ok
test tests::random::random_string_invalid_count_or_empty_alphabet_stays_inert ... ok
test tests::random::random_string_seed_random_reproducibility_is_pinned ... ok
test tests::random::random_text_composes_with_character_class_constants ... ok
test tests::random::random_uuid_and_uuid_q_cover_format_boundaries ... ok
test tests::random::reseed_random_uses_entropy_source_in_unrestricted_session ... ok
test tests::random::seed_random_integer_sequence_is_pinned ... ok
test tests::random::seed_random_is_deterministic_only_and_reseed_random_has_exact_arity ... ok
test tests::random::whitespace_character_constant_has_pinned_value_and_composes ... ok
test tests::resumable_traversal_regression::delayed_association_aggregates_evaluate_each_value_once ... ok
test tests::resumable_traversal_regression::large_catenate_list_walk_does_not_recurse_through_the_rust_stack ... ok
test tests::resumable_traversal_regression::large_inner_callback_loop_does_not_recurse_through_the_rust_stack ... ok
test tests::resumable_traversal_regression::large_resumable_fold_loops_do_not_recurse_through_the_rust_stack ... ok
test tests::resumable_traversal_regression::resumable_array_traversals_visit_each_generated_item_once ... ok
test tests::resumable_traversal_regression::resumable_fold_traversals_visit_each_item_once ... ok
test tests::resumable_traversal_regression::resumable_traversals_visit_each_completed_item_once ... ok
test tests::retention_closure::accepted_fixture_exercises_d2_shapes ... ACCEPTED bodies=2 leaves=5 expression_facts=29 loops=1 adoptions=1 transfers=1
ok
test tests::retention_closure::census::tests::conditional_ambiguous_and_missing_roots_are_located ... ok
test tests::retention_closure::census::tests::n_frontier_keeps_unresolved_roots ... EDGE_N_FRONTIER roots=79 definitions=1050 raw=73884 deduplicated=6235 resolved_sites=2730
ok
test tests::retention_closure::census::tests::production_rejection_census_is_complete ... EDGE_ALL roots=7202 definitions=7202 raw=47922 deduplicated=47922 resolved_sites=16231
ok
test tests::retention_closure::compiler_oracle::compiler_oracle_agrees_with_resolution_claims ... ORACLE_BASELINE stable=1.85.0 compile=pass
ok
test tests::retention_closure::compiler_oracle::corpus::census_constructor_resolutions_are_compiler_observed ... A1_CORPUS corpus_0 AGREEMENT site=crates/rum-cli/src/input_retirement.rs:25 target=std::option::Option::Some evidence=Option::<HostTerminationReason>::Some negative=separate-same-name-definition
ok
test tests::retention_closure::compiler_oracle::exact::exact_types_and_constructor_claims_match_rustc ... KNOWN_TYPE_EFFECTS_SEPARATE	crates/rum-runtime/src/runtime/accounting_mode/options/driver.rs:54:28-56:23	std::result::Result::Err	std::result::Result<rum_core::expr::Expr, rum_runtime::runtime::session::session_state::EvalError>
ok
test tests::retention_closure::compiler_oracle::methods::inherent_method_claims_match_rustc ... METHOD_CLAIMS fixture=16
ok
test tests::retention_closure::constructor_tests::enum_constructors_preserve_payloads ... VALID_CONTROL root accepted bodies=1 facts=6
ok
test tests::retention_closure::constructor_tests::partial_moves_keep_sibling_obligations ... SIBLING_CONTROL	RET001	crates/rum-stdlib/src/tests/retention_closure/fixtures/constructors.rs:46:5-46:55	-	fixture::constructors::partial_bad	unleased owned payload
ok
test tests::retention_closure::constructor_tests::trusted_instances_are_individually_exercised ... INSTANCE_EXERCISED fixture::constructors::keep::<u8>
ok
test tests::retention_closure::constructor_tests::variant_shadowing_never_uses_std_intrinsics ... SHADOW_CONTROL some_root	RET002	crates/rum-stdlib/src/tests/retention_closure/fixtures/constructors.rs:95:35-95:48	-	fixture::constructors::local::some_root -> fixture::constructors::local::Some	method dispatch requires autoderef/autoref/trait resolution
ok
test tests::retention_closure::coverage_cannot_shrink ... ok
test tests::retention_closure::extracted_helpers_enter_closure ... ok
test tests::retention_closure::implicit_operations_are_classified ... ok
test tests::retention_closure::local::admission_tests::local_pending_argument_and_wrapped_payload_controls ... ok
test tests::retention_closure::local::admission_tests::local_result_and_operator_edges ... ok
test tests::retention_closure::local::admission_tests::local_retention_is_temporal_not_construction ... ok
test tests::retention_closure::local::contract_tests::local_concrete_contract_controls ... ok
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... ignored, writes a review copy; does not discharge contracts or install marks
test tests::retention_closure::local::option_boundaries::tests::local_option_contract_controls ... OPTION544_CONTRACT: concrete M x R green; stale/Pending/body/schema/raw-container/outside-R/all-R/wrong-reborrow red; promotion=0
ok
test tests::retention_closure::local::option_domain::local_option_domain_reconciliation_and_independent_drift_controls ... OPTION544_DOMAIN: declaration/impl-feature/installer/runtime/keys mutations rejected; outside type excluded; keys=["M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true", "M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true"]
ok
test tests::retention_closure::local::oracle::local_probe_claims_match_stable_rustc ... LOCAL540_TYPES positive=pass functions=7 both_modes=true invariant=true
error: could not compile `rum-runtime` (lib test) due to 84 previous errors; 47 warnings emitted
ok
test tests::retention_closure::local::production::local_production_acceptance ... body	instance	rejections
ok
test tests::retention_closure::local::promotion_tests::production_body_domain_and_build_expiry_and_raw_local ... C01-raw-across-guest: RED RET001	crates/rum-runtime/src/runtime/options/core.rs:1:1-1:1	-	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	raw owned container produced: std::vec::Vec<rum_core::expr::Expr> is live across call at crates/rum-runtime/src/runtime/options/core.rs:100:9-102:10 (callee not proved inert)
ok
test tests::retention_closure::local::promotion_tests::production_inventory_rejects_removed_redirected_stale_and_omitted_evidence ... absent: RED production promotion removed or absent
ok
test tests::retention_closure::local::promotion_tests::production_marks_require_current_local_lint_and_direct_contracts ... C01-O0210-missing: RED RET002	crates/rum-runtime/src/runtime/options/core.rs:99:13-99:92	rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol	missing rule boundary dependency rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>
ok
test tests::retention_closure::local::promotion_tests::production_promotion_cannot_launder_consumers_or_runtime_domain ... D3-predicates-same-test: RED consumer obligation changed or claimed covered: D3-predicates-same-test
ok
test tests::retention_closure::local::reads::oracle::local_option_read_claims_match_stable_rustc ... READ542_SCRATCH stale_input=removed
ok
test tests::retention_closure::local::reads::tests::local_option_borrowed_pattern_controls ... READ542_CONTROL wrong-field red=reject green=clean
ok
test tests::retention_closure::local::reads::tests::local_option_read_contract_controls ... READ542_CONTROL I02-stale red=reject green=clean
ok
test tests::retention_closure::local::reads::tests::local_option_short_circuit_ownership_controls ... READ542_CONTROL rhs-moves-owner red=reject green=clean
ok
test tests::retention_closure::local::rules::tests::local_rule_contract_controls ... RULE543_CONTROL B04-stale: red=reject green=clean
ok
test tests::retention_closure::local::rules::tests::local_rule_macro_definition_arms_hygiene_and_raw_frame_controls ... RULE543_CONTROL macro-original-raw-frame: red=reject green=clean
ok
test tests::retention_closure::local::tests::local_admissions_never_exceed_a1_a2 ... ok
test tests::retention_closure::local::tests::local_contract_falsifiers ... ok
test tests::retention_closure::local::tests::local_inventory_marks_are_earned ... ok
test tests::retention_closure::local::tests::local_missing_mark_falsifier ... ok
test tests::retention_closure::local::tests::local_raw_type_falsifiers_and_closed_list ... ok
test tests::retention_closure::local::tests::local_scope_fingerprint_falsifier ... ok
test tests::retention_closure::local::tests::local_side_door_and_covered_freshness ... ok
test tests::retention_closure::local::tests::local_unknown_type_falsifier ... ok
test tests::retention_closure::local::type_tests::finite_modes_and_opaque_brands ... ok
test tests::retention_closure::method_tests::installed_instances_do_not_cross ... ok
test tests::retention_closure::method_tests::nominal_method_selection_is_exact ... METHOD_FALSIFIER collision	RET002	crates/rum-stdlib/src/tests/retention_closure/fixtures/methods.rs:32:5-32:19	-	fixture::methods::collision	method syntax: method receiver lacks checked nominal/place provenance
ok
test tests::retention_closure::method_tests::receiver_adjustments_fail_closed ... METHOD_FALSIFIER trait_wins	RET002	crates/rum-stdlib/src/tests/retention_closure/fixtures/methods.rs:77:5-77:19	-	fixture::methods::trait_wins	method syntax: earlier trait candidate fixture::methods::First::select at &fixture::methods::Earlier (conditional=false)
ok
test tests::retention_closure::production_obligations_are_exact ... MODULES=706 RECONCILED_FILES=1145 PRODUCTION_FILES=695 FUNCTION_IDENTITIES=7202
ok
test tests::retention_closure::restricted_surface_rejects_six_shapes ... PINNED	bare_vec_loop	RET001	crates/rum-stdlib/src/tests/retention_closure/fixtures/bare_vec_loop.rs:2:22-2:41	std::vec::Vec::new	fixture::bare_vec_loop::root	unleased owned payload
ok
test tests::retention_closure::trusted_items_have_exact_bounds ... ok
test tests::retention_closure::unresolved_edges_are_errors ... ok
test tests::scan_and_keypath_set_native_stack_guard::keypath_set_zero_native_stack_boundary_preserves_both_lvalue_roots ... ignored, requires rum-runtime/test-native-stack-guard-zero
test tests::scan_and_keypath_set_native_stack_guard::scan_zero_native_stack_boundary_covers_each_edge_without_callbacks ... ignored, requires rum-runtime/test-native-stack-guard-zero
test tests::schema::conform_schema_emits_schema_messages_and_fails_closed ... ok
test tests::schema::conform_schema_validates_nested_config_and_fills_defaults ... ok
test tests::schema::conform_schema_wide_association_preserves_value_order ... ok
test tests::schema::list_schema_covers_empty_and_nested_associations ... ok
test tests::schema::optional_defaults_and_extra_keys_follow_schema_contract ... ok
test tests::schema::pattern_leaf_breadth_and_held_schema_semantics ... ok
test tests::schema::schema_errors_are_capped_with_summary ... ok
test tests::schema::schema_errors_returns_exact_nested_invalid_config_errors ... ok
test tests::schema::schema_validation_composes_with_codecs_and_error_data_serializes ... ok
test tests::schema_operator_forms::direct_schema_calls_keep_attributes_and_allow_extra_keys_option ... ok
test tests::schema_operator_forms::direct_schema_value_evaluation_resumes_after_input ... ok
test tests::schema_operator_forms::schema_capture_is_quiet_and_defers_schema_side_effects_until_validation ... ok
test tests::schema_operator_forms::schema_captures_are_inert_and_wrong_application_arity_reports_argx ... ok
test tests::schema_operator_forms::schema_operator_forms_cover_success_failure_and_error_lists ... ok
test tests::schema_operator_forms::schema_operator_value_side_effect_fires_once ... ok
test tests::session_requests_functional_iteration_resume::resume_eval_request_with_input_can_finish_fixedpoint_callback_loop ... ok
test tests::session_requests_functional_iteration_resume::resume_eval_request_with_input_can_finish_fixedpointlist_callback_loop ... ok
test tests::session_requests_functional_iteration_resume::resume_eval_request_with_input_can_finish_nest_callback_loop ... ok
test tests::session_requests_functional_iteration_resume::resume_eval_request_with_input_can_finish_nestlist_callback_loop ... ok
test tests::session_requests_functional_iteration_resume::resume_eval_request_with_input_can_finish_nestwhile_test_and_step ... ok
test tests::session_requests_functional_iteration_resume::resume_eval_request_with_input_can_finish_nestwhilelist_test_and_step ... ok
test tests::session_requests_functional_sequence_resume::every_custom_sort_comparison_resumes_with_the_uninterrupted_trace ... ok
test tests::session_requests_functional_sequence_resume::numeric_suspend_resume_liveness_probes_return_exact_values ... callback_sort=List[List[1, 2, 3], 3]
ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_accumulate_and_blockmap_callbacks ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_custom_equivalence_loops ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_delete_duplicates_by_key_loop ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_fold_callback_loop ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_foldlist_callback_loop ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_foldpair_operator_and_projector ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_foldwhile_step_and_test ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_gatherby_function_list_key_loop ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_sequencefoldlist_callback_loop ... ok
test tests::session_requests_functional_sequence_resume::resume_eval_request_with_input_can_finish_sortby_key_loop ... ok
test tests::session_requests_functional_targeted_resume::resume_eval_request_with_input_can_finish_mapat_callback_loop ... ok
test tests::session_requests_functional_targeted_resume::resume_eval_request_with_input_can_finish_subsetmap_callback_and_final_eval ... ok
test tests::session_requests_functional_tensor_string_resume::resume_eval_request_with_input_can_finish_array_callback_helpers ... ok
test tests::session_requests_functional_tensor_string_resume::resume_eval_request_with_input_can_finish_inner_term_and_reducer_callbacks ... ok
test tests::session_requests_functional_tensor_string_resume::resume_eval_request_with_input_preserves_array_and_outer_progress ... ok
test tests::session_requests_functional_tensor_string_resume::resume_eval_request_with_input_preserves_completed_inner_terms ... ok
test tests::session_requests_predicate_collections_resume::resume_eval_request_with_input_preserves_duplicate_free_pair_progress ... ok
test tests::session_requests_predicate_collections_resume::resume_eval_request_with_input_preserves_same_test_matrix_progress ... ok
test tests::session_requests_set_same_test_resume::custom_same_test_set_operations_preserve_progress_across_input ... ok
test tests::session_requests_transforms_resume_associations::associateto_requires_an_immediate_symbol_value_instead_of_suspending_delayed_values ... ok
test tests::session_requests_transforms_resume_associations::association_part_mutation_keeps_opaque_completion_across_two_suspensions ... ok
test tests::session_requests_transforms_resume_associations::association_part_mutation_resumes_one_selector_without_replay_and_refetches_owner ... ok
test tests::session_requests_transforms_resume_associations::held_part_update_resumes_two_selectors_without_replay_and_refetches_owner ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_catenate_after_suspending_delayed_association_value ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_counts_after_suspending_delayed_association_value ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_countsby_after_multiple_key_suspensions ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_deletemissing_eager_delayed_predicate ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_groupby_after_suspending_delayed_association_value ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_groupby_after_suspending_value_transform ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_groupby_list_key_and_value_loops ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_groupby_nested_and_reducer_group_loops ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_keygroupby_key_and_reducer_loops ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_keyunion_after_suspending_fill_function ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_merge_after_suspending_delayed_association_value ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_nested_deletemissing_predicate ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_can_finish_nested_groupby_after_multiple_key_suspensions ... ok
test tests::session_requests_transforms_resume_associations::resume_eval_request_with_input_reports_catenate_invalid_value_after_suspension ... ok
test tests::stress_corpus::bug_list_pipeline_range_50000_overflows_native_stack ... ok
test tests::stress_corpus::guard_adjacent_user_recursion_above_limit_terminates_and_session_recovers ... ok
test tests::stress_corpus::guard_adjacent_user_recursion_below_limit_succeeds ... ok
test tests::stress_corpus::list_pipeline_processes_large_range_with_exact_result ... ok
test tests::stress_corpus::peano_arithmetic_heavy_multiplication ... ignored
test tests::stress_corpus::peano_arithmetic_multiplies_through_pure_rewriting ... ok
test tests::stress_corpus::reduced_native_stack_limit_returns_stack_limit_and_session_recovers ... ok
test tests::stress_corpus::rule_based_differentiator_handles_generated_polynomial ... ok
test tests::stress_corpus::rule_based_differentiator_handles_heavy_generated_polynomial ... ignored
test tests::stress_corpus::tokenize_with_ini_grammar_handles_generated_100kb_config ... ok
test tests::stress_corpus::tokenize_with_ini_grammar_handles_heavy_generated_config ... ignored
test tests::strings::advanced_patterns::bounded_repetition_and_except_extend_string_patterns ... ok
test tests::strings::advanced_patterns::character_class_pattern_atoms_extend_the_current_string_pattern_subset ... ok
test tests::strings::advanced_patterns::regular_expression_group_nesting_is_capped_and_far_past_shapes_stay_inert ... ok
test tests::strings::advanced_patterns::regular_expression_patterns_extend_the_current_string_pattern_subset ... ok
test tests::strings::advanced_patterns::repeated_anchors_and_shortest_longest_extend_string_patterns ... ok
test tests::strings::advanced_patterns::string_pattern_consumers_reject_malformed_repetition_bounds ... ok
test tests::strings::advanced_patterns::symbolic_blank_patterns_extend_the_current_string_pattern_subset ... ok
test tests::strings::encoding::byte_arrays_transport_and_nearest_extend_cl07 ... ok
test tests::strings::encoding::hash_byte_identity_and_formats_extend_cl07 ... ok
test tests::strings::incidental_runtime::association_aggregation::merge_catenate_counts_groupby_and_positionindex_cover_first_pass_subset ... ok
test tests::strings::incidental_runtime::association_key_ops::key_predicates_take_drop_map_and_sort_cover_first_pass_subset ... ok
test tests::strings::incidental_runtime::association_key_ops::key_sort_by_evaluates_each_key_once_at_scale ... ok
test tests::strings::incidental_runtime::association_key_ops::key_take_and_drop_preserve_order_rules_at_scan_and_indexed_widths ... ok
test tests::strings::incidental_runtime::failure_messages::confirm_failure_wrapping_resumes_delayed_source_failure_message_template ... ok
test tests::strings::incidental_runtime::failure_messages::failure_and_enclose_confirm_family_cover_first_pass_error_flow ... ok
test tests::strings::incidental_runtime::failure_messages::failure_message_template_properties_cover_current_subset ... ok
test tests::strings::incidental_runtime::failure_messages::message_list_reports_last_top_level_messages ... ok
test tests::strings::incidental_runtime::functional_ops::accumulate_blockmap_takewhile_and_fold_family_cover_current_subset ... ok
test tests::strings::incidental_runtime::functional_ops::operator_wrappers_cover_current_curried_subset ... ok
test tests::strings::incidental_runtime::logical_syntax::logical_and_comparison_syntax_lowers_to_existing_builtins ... ok
test tests::strings::incidental_runtime::options::delayed_system_default_options_feed_current_builtin_consumers ... ok
test tests::strings::incidental_runtime::options::setoptions_updates_current_system_default_subset_behavior ... ok
test tests::strings::incidental_runtime::options::system_default_options_cover_current_first_pass_subset ... ok
test tests::strings::incidental_runtime::package_contexts::dump_save_context_strings_roundtrip_public_and_private_contexts ... ok
test tests::strings::incidental_runtime::session_requests::guarded_string_rules_resume_without_replaying_candidates_or_output ... ok
test tests::strings::incidental_runtime::session_requests::resume_eval_request_with_input_can_finish_delayed_string_rules ... ok
test tests::strings::incidental_runtime::session_requests::resume_eval_request_with_input_can_finish_keysortby_after_each_sort_key_suspends ... ok
test tests::strings::incidental_runtime::session_requests::resume_eval_request_with_input_can_finish_keysortby_after_suspending_sort_key ... ok
test tests::strings::incidental_runtime::session_requests::resume_eval_request_with_input_can_finish_replacepart_pattern_bindings_in_delayed_rhs ... ok
test tests::strings::incidental_runtime::session_requests::resume_eval_request_with_input_can_finish_string_apply_list_callbacks ... ok
test tests::strings::incidental_runtime::session_requests::string_apply_resumes_mid_walk_with_accumulated_output ... ok
test tests::strings::incidental_runtime::session_requests::string_replace_and_cases_resume_mid_walk_with_accumulated_output ... ok
test tests::strings::incidental_runtime::structure_ops::delete_duplicates_by_keeps_first_item_for_each_evaluated_key ... ok
test tests::strings::incidental_runtime::syntax_information::syntax_information_includes_current_system_default_subset ... ok
test tests::strings::parts::string_apply_delete_and_count_options_cover_the_next_wave ... ok
test tests::strings::parts::string_apply_walks_fifty_thousand_strings ... ok
test tests::strings::parts::string_builtins_preserve_current_boundary_for_non_string_or_invalid_specs ... ok
test tests::strings::parts::string_take_drop_and_part_support_basic_integer_and_range_specs ... ok
test tests::strings::scripting::html_shell_path_url_and_args_utilities_cover_boundaries ... ok
test tests::strings::scripting::numeric_text_padding_respects_generated_element_limit_boundaries ... ok
test tests::strings::scripting::scripting_pipeline_combines_args_url_and_aligned_output ... ok
test tests::strings::scripting::string_form_is_eager_and_reports_missing_slots ... ok
test tests::strings::scripting::text_shaping_utilities_cover_wrap_indent_dedent_tabs_and_tables ... ok
test tests::strings::search_replace::delayed_string_rules_evaluate_at_application_time ... ok
test tests::strings::search_replace::guarded_string_cases_pin_overlapping_adjacent_and_empty_matches ... ok
test tests::strings::search_replace::guarded_string_rule_lists_try_later_rules_at_the_same_position ... ok
test tests::strings::search_replace::guarded_string_rules_preserve_character_positions_and_candidate_counts ... ok
test tests::strings::search_replace::non_boolean_string_guards_decline_and_delayed_bodies_wait_for_acceptance ... ok
test tests::strings::search_replace::rhs_condition_guards_control_string_replace_for_both_rule_heads ... ok
test tests::strings::search_replace::rhs_guard_whole_line_regression_covers_every_rule_lane ... ok
test tests::strings::search_replace::string_cases_and_position_honor_boolean_overlaps_options ... ok
test tests::strings::search_replace::string_count_matchq_and_freeq_extend_current_pattern_subset ... ok
test tests::strings::search_replace::string_expression_and_alternatives_extend_the_current_string_subset ... ok
test tests::strings::search_replace::string_position_reports_overlapping_one_based_ranges ... ok
test tests::strings::search_replace::string_repeat_and_exact_substring_predicates_cover_first_tier_two_wave ... ok
test tests::strings::search_replace::string_replace_and_cases_walk_fifty_thousand_strings ... ok
test tests::strings::search_replace::string_rule_guards_and_bodies_share_the_enclosing_option_scope ... ok
test tests::strings::search_replace::string_search_and_rewrite_builtins_support_wl_operator_forms ... ok
test tests::strings::search_replace::string_split_does_not_accept_rule_delimiter_specs ... ok
test tests::strings::search_replace::string_split_trim_replace_and_cases_cover_common_text_processing_flows ... ok
test tests::strings::search_replace::string_trim_literal_list_handles_a_wide_input ... ok
test tests::strings::search_replace::string_trim_literal_lists_preserve_exact_boundary_semantics ... ok
test tests::strings::search_replace::string_trim_literal_lists_preserve_unicode_threading_and_general_fallback ... ok
test tests::strings::search_replace::symbolic_string_patterns_bind_named_captures_in_rules ... ok
test tests::strings::search_replace::wide_literal_search_preserves_predicate_overlap_and_collection_semantics ... ok
test tests::strings::similarity::alphabet_distance_alignment_and_diff_extend_cl07 ... ok
test tests::strings::surface::case_conversion_threads_nested_and_mixed_lists_like_listable ... ok
test tests::strings::surface::characters_character_range_and_tostring_cover_basic_text_flows ... ok
test tests::strings::surface::input_like_renderer_uses_part_and_map_family_syntax ... ok
test tests::strings::surface::protected_string_pattern_symbols_reject_definitions_without_changing_matching ... ok
test tests::strings::surface::string_character_classes_and_case_mapping_extend_the_string_subset ... ok
test tests::strings::surface::string_construction_and_measurement_use_code_point_semantics ... ok
test tests::strings::surface::string_formatting_padding_and_riffle_cover_the_next_surface_wave ... ok
test tests::strings::surface::string_generators_respect_resource_limits ... ok
test tests::strings::surface::string_helper_builtins_cover_surface_edges ... ok
test tests::strings::surface::string_pattern_symbols_are_protected ... ok
test tests::strings::surface::syntaxq_textstring_charactercounts_and_removediacritics_extend_cl07 ... ok
test tests::strings::surface::templates_text_words_codes_and_to_expression_extend_cl07 ... ok
test tests::strings::text_odds::decimal_string_formats_exact_numbers_with_fixed_fraction_digits ... ok
test tests::strings::text_odds::decimal_string_invalid_forms_remain_inert ... ok
test tests::strings::text_odds::decimal_string_respects_generated_element_limit_boundaries ... ok
test tests::strings::text_odds::string_lines_splits_lf_and_crlf_with_trailing_newline_rule ... ok
test tests::strings::text_odds::string_pad_right_mirrors_left_padding_subset ... ok
test tests::strings::text_odds::string_padding_respects_generated_element_limit_boundaries ... ok
test tests::strings::text_odds::string_text_odds_report_invalid_arities ... ok
test tests::strings::tokenize::rum_tokens_matches_pre_change_offset_goldens ... ok
test tests::strings::tokenize::rum_tokens_preserves_trivia_and_operator_coverage ... ok
test tests::strings::tokenize::rum_tokens_recovers_malformed_input_with_error_spans ... ok
test tests::strings::tokenize::rum_tokens_reports_multibyte_code_point_positions ... ok
test tests::strings::tokenize::rum_tokens_reports_total_ascending_spans_across_inputs ... ok
test tests::strings::tokenize::tokenize_with_fails_closed_for_invalid_inputs ... ok
test tests::strings::tokenize::tokenize_with_ini_grammar_reports_exact_spans ... ok
test tests::strings::tokenize::tokenize_with_reports_total_ascending_spans_across_inputs ... ok
test tests::strings::tokenize::tokenize_with_skips_zero_width_rules_and_coalesces_text ... ok
test tests::strings::tokenize::tokenize_with_spans_round_trip_through_string_take ... ok
test tests::strings::tokenize::tokenize_with_uses_longest_match_then_earliest_rule ... ok
test tests::structure_ops::collection_stats::by_extrema_take_and_reverse_sort_cover_ties_and_boundaries ... ok
test tests::structure_ops::collection_stats::collection_operator_forms_rewrite_to_data_first_calls ... ok
test tests::structure_ops::collection_stats::collection_stats_report_invalid_arity_and_empty_selection ... ok
test tests::structure_ops::collection_stats::collection_stats_support_normal_expression_heads_where_structure_ops_do ... ok
test tests::structure_ops::collection_stats::custom_reverse_sort_comparator_obeys_the_merge_bound_without_growing_the_stack ... ok
test tests::structure_ops::collection_stats::custom_reverse_sort_covers_boundaries_and_reverses_stable_tagged_ties ... ok
test tests::structure_ops::collection_stats::custom_tally_test_preserves_wide_pair_count_without_growing_the_stack ... ok
test tests::structure_ops::collection_stats::gather_by_groups_in_first_key_order ... ok
test tests::structure_ops::collection_stats::max_by_and_min_by_accept_top_counts ... ok
test tests::structure_ops::collection_stats::maximal_by_and_minimal_by_return_all_extrema_in_input_order ... ok
test tests::structure_ops::collection_stats::maximal_by_key_evaluation_supports_user_downvalues ... ok
test tests::structure_ops::collection_stats::tally_and_riffle_cover_first_occurrence_and_boundaries ... ok
test tests::structure_ops::collection_stats::tally_and_take_largest_by_support_top_counts_idiom ... ok
test tests::structure_ops::dedup::custom_delete_duplicates_test_preserves_wide_pair_count_without_growing_the_stack ... ok
test tests::structure_ops::dedup::delete_duplicates_accepts_custom_same_test ... ok
test tests::structure_ops::dedup::delete_duplicates_keeps_first_structural_occurrence ... ok
test tests::structure_ops::dedup::delete_duplicates_reports_current_invalid_boundaries ... ok
test tests::structure_ops::join::join_concatenates_associations_with_later_keys_winning ... ok
test tests::structure_ops::join::join_concatenates_top_level_same_head_expressions ... ok
test tests::structure_ops::join::join_reports_current_level_argument_boundaries ... ok
test tests::structure_ops::join::join_reports_head_mismatches_and_leaves_atomic_inputs_inert ... ok
test tests::structure_ops::join::join_supports_current_level_argument_subset ... ok
test tests::structure_ops::list_lookup::binary_search_uses_canonical_order_and_not_found_missing ... ok
test tests::structure_ops::list_lookup::duplicates_returns_repeated_values_once_in_first_appearance_order ... ok
test tests::structure_ops::list_lookup::select_positions_returns_position_paths_matching_position ... ok
test tests::structure_ops::list_plumbing::combinatorial_list_ops_symbolically_passthrough_invalid_boundaries ... ok
test tests::structure_ops::list_plumbing::constant_array_covers_scalar_and_multidimensional_integer_shapes ... ok
test tests::structure_ops::list_plumbing::differences_covers_successive_and_ordered_plain_list_subset ... ok
test tests::structure_ops::list_plumbing::list_plumbing_attributes_match_wk_anchors ... ok
test tests::structure_ops::list_plumbing::pad_left_covers_padding_truncation_cyclic_and_normal_heads ... ok
test tests::structure_ops::list_plumbing::pad_right_covers_padding_truncation_cyclic_and_normal_heads ... ok
test tests::structure_ops::list_plumbing::partition_covers_wk_anchored_block_and_offset_forms ... ok
test tests::structure_ops::list_plumbing::partition_operator_form_rewrites_to_data_first_call ... ok
test tests::structure_ops::list_plumbing::permutations_are_distinct_and_canonically_lexicographic ... ok
test tests::structure_ops::list_plumbing::permutations_fail_before_materializing_a_huge_result ... ok
test tests::structure_ops::list_plumbing::subsets_covers_wk_anchored_size_specs_and_ordering ... ok
test tests::structure_ops::list_plumbing::tuples_covers_wk_anchored_repetition_and_cartesian_forms ... ok
test tests::structure_ops::ordering_sets::custom_sort_and_sort_by_cover_boundaries_and_stable_tagged_ties ... ok
test tests::structure_ops::ordering_sets::custom_sort_comparators_obey_the_merge_bound_without_growing_the_stack ... ok
test tests::structure_ops::ordering_sets::every_default_structural_order_consumer_keeps_qualified_symbol_order ... ok
test tests::structure_ops::ordering_sets::intersection_covers_wk_anchored_same_head_subset ... ok
test tests::structure_ops::ordering_sets::ordering_and_set_helpers_report_current_invalid_boundaries ... ok
test tests::structure_ops::ordering_sets::qualified_symbol_name_order_matches_the_allocating_text_order_exactly ... ok
test tests::structure_ops::ordering_sets::qualified_symbol_name_ties_ignore_context_elision_and_generated_identity_stably ... ok
test tests::structure_ops::ordering_sets::separate_canonical_and_numeric_order_consumers_keep_their_base_outputs ... ok
test tests::structure_ops::ordering_sets::set_operation_attributes_match_wk_anchors ... ok
test tests::structure_ops::ordering_sets::set_operations_accept_same_test_option ... ok
test tests::structure_ops::ordering_sets::sort_and_sortby_cover_current_top_level_subset ... ok
test tests::structure_ops::ordering_sets::sort_by_multi_key_evaluation_supports_user_downvalues ... ok
test tests::structure_ops::ordering_sets::sort_by_supports_stable_multi_key_specs_and_tuple_ordering ... ok
test tests::structure_ops::ordering_sets::symbol_sort_order_and_mixed_atom_fallback_match_the_pinned_lists ... ok
test tests::structure_ops::ordering_sets::union_and_complement_cover_current_same_head_subset ... ok
test tests::structure_ops::ordering_sets::wide_custom_same_test_set_operations_preserve_comparison_counts ... ok
test tests::structure_ops::ordering_sets::wide_symbol_sort_matches_the_full_list_and_pinned_base_rendering ... ok
test tests::structure_ops::reordering::reverse_covers_current_positive_level_subset ... ok
test tests::structure_ops::reordering::reverse_reports_current_invalid_boundaries ... ok
test tests::structure_ops::reordering::transpose_covers_current_plain_list_matrix_subset ... ok
test tests::structure_ops::reordering::transpose_reports_current_invalid_boundaries ... ok
test tests::structure_ops::selectors::extract_consumes_exact_paths_and_position_output ... ok
test tests::structure_ops::selectors::invalid_part_and_extract_specs_report_selector_messages ... ok
test tests::structure_ops::selectors::keyed_part_and_extract_cover_association_paths ... ok
test tests::structure_ops::selectors::part_paths::part_supports_exact_indices_nested_paths_and_head_access ... ok
test tests::structure_ops::selectors::wide_batched_key_part_preserves_order_duplicates_and_missing_values ... ok
test tests::structure_ops::sequence::range_constructs_plain_integer_lists_for_current_subset ... ok
test tests::structure_ops::sequence::range_respects_generated_element_limit_before_allocating ... ok
test tests::structure_ops::sequence::sequence_operator_forms_rewrite_to_data_first_calls ... ok
test tests::structure_ops::sequence::take_drop_accept_upto_sequence_specs ... ok
test tests::structure_ops::sequence::take_drop_append_and_prepend_report_current_invalid_boundaries ... ok
test tests::structure_ops::sequence::take_drop_insert_delete_and_take_list_cover_current_structural_subset ... ok
test tests::structure_ops::sequence::take_existing_sequence_specs_are_unchanged ... ok
test tests::structure_ops::sequence::take_multispec_failure_below_first_spec_preserves_message_channel ... ok
test tests::structure_ops::sequence::take_multispec_shallow_companion_pins_three_level_fanout ... ok
test tests::structure_ops::sequence::take_upto_reports_atom_and_malformed_spec_messages ... ok
test tests::structure_ops::surface::append_prepend_symbol_and_construct_cover_current_surface_subset ... ok
test tests::structure_ops::surface::first_last_rest_and_most_emit_expected_messages_for_invalid_inputs ... ok
test tests::structure_ops::surface::flatten_covers_current_same_head_level_subset ... ok
test tests::structure_ops::surface::power_threads_over_lists_without_disturbing_canonicalization ... ok
test tests::structured_exceptions::catch_exceptions_dispatch_survives_body_suspension ... ok
test tests::structured_exceptions::classic_and_exception_channels_do_not_cross_catch ... ok
test tests::structured_exceptions::cleanup_runs_and_enclose_does_not_intercept_exceptions ... ok
test tests::structured_exceptions::exception_dispatch_honors_exact_ancestry_handler_order_and_outer_reraise ... ok
test tests::structured_exceptions::exception_registry_round_trips_and_reregistration_fails_closed ... ok
test tests::structured_exceptions::unregistered_and_uncaught_exceptions_use_fail_closed_in_band_values ... ok
test tests::traversal::count_like_condition_guards_preserve_short_circuit_points ... ok
test tests::traversal::count_memberq_and_freeq_share_standard_traversal_semantics ... ok
test tests::traversal::level_supports_heads_and_apply_head_forms ... ok
test tests::traversal::level_supports_standard_levelspecs_and_depth_first_ordering ... ok
test tests::traversal::position_supports_standard_levelspecs_heads_and_limits ... ok
test tests::traversal::traversal_uses_values_only_association_semantics_and_keyed_positions ... ok
test tests::valueq_effect_suppression::valueq_allows_read_clock_diagnostic_and_random_capabilities ... ok
test tests::valueq_effect_suppression::valueq_refuses_file_write_and_process_then_restores_the_policy ... ok
test tests::valueq_effect_suppression::valueq_restores_prng_on_success_error_suspension_and_nesting ... ok
test tests::valueq_effect_suppression::valueq_restores_the_exact_prior_policy_on_every_outcome ... ok
test tests::valueq_effect_suppression::valueq_suppresses_an_installed_print_sink_and_restores_it ... ok
test toml::encode::depth_tests::to_toml_accepts_nesting_at_the_depth_limit ... ok
test toml::encode::depth_tests::to_toml_refuses_nesting_one_past_the_depth_limit ... ok
test xml::encode::depth_tests::to_xml_accepts_nesting_at_the_depth_limit ... ok
test xml::encode::depth_tests::to_xml_refuses_nesting_one_past_the_depth_limit ... ok
test yaml::encode::depth_tests::to_yaml_accepts_nesting_at_the_depth_limit ... ok
test yaml::encode::depth_tests::to_yaml_refuses_nesting_one_past_the_depth_limit ... ok
test result: ok. 1034 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1090.65s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
running 47 tests
test budgets::red_guest_forms_share_request_deadline ... BUDGET timeout per_form_minimum=100ms forms=8 deadline=400ms single=Null value=TerminatedEvaluation["TimeConstraint"] termination=Some(TimeConstraint)
ok
test budgets::red_guest_load_inherits_memory_constraint ... BUDGET memory value=$Aborted termination=None
ok
test cancellation::green_best_effort_discards_and_next_input ... ok
test cancellation::red_begin_cancel_at_both_inputs ... ok
test cancellation::red_begin_cancel_restores ... ok
test cancellation::red_cancel_cleanup_get_releases_owner ... ok
test cancellation::red_get_cancel_at_both_inputs ... ok
test cancellation::red_get_cancel_restores ... ok
test cancellation::red_needs_cancel_at_both_inputs ... ok
test cancellation::red_needs_cancel_restores ... ok
test cancellation::red_nested_cancel_restores ... ok
test cancellation::red_stub_cancel_at_both_inputs ... ok
test cancellation::red_stub_cancel_restores ... ok
test entries::red_begin_dependency_second ... SOURCE	red_begin_dependency_second	"BeginPackage[\"Main549`\",{\"After549`\",\"Dep549`\"}]"	inputs=["one"]	files=[]
ok
test entries::red_begin_package ... SOURCE	red_begin_package	"BeginPackage[\"Main549`\",{\"Dep549`\",\"After549`\"}]"	inputs=["one", "two", "three", "four", "five", "six"]	files=[]
ok
test entries::red_begin_package_then_q ... SOURCE	red_begin_package_then_q	"BeginPackage[\"Main549`\",{\"Dep549`\",\"After549`\"}];InputString[\"q: \"]"	inputs=["one", "two", "three", "four", "five", "six"]	files=[]
ok
test entries::red_begin_two_dependencies_suspend ... SOURCE	red_begin_two_dependencies_suspend	"BeginPackage[\"Main549`\",{\"Dep549`\",\"Second549`\"}]"	inputs=["one", "two"]	files=[("Second549.rum", "AppendTo[Global`log,5]\nInputString[\"second-dep: \"]\nAppendTo[Global`log,7]\n99")]
ok
test entries::red_get ... SOURCE	red_get	"Get[\"Dep549`\"]"	inputs=["one", "two", "three", "four", "five", "six"]	files=[]
ok
test entries::red_get_then_q ... SOURCE	red_get_then_q	"Get[\"Dep549`\"];InputString[\"q: \"]"	inputs=["one", "two", "three", "four", "five", "six"]	files=[]
ok
test entries::red_needs ... SOURCE	red_needs	"Needs[\"Dep549`\"]"	inputs=["one", "two", "three", "four", "five", "six"]	files=[]
ok
test entries::red_needs_explicit ... SOURCE	red_needs_explicit	"Needs[\"Other549`\",File[\"Dep549.rum\"]];{Dep549`foo,$Packages}"	inputs=["one"]	files=[]
ok
test entries::red_needs_then_q ... SOURCE	red_needs_then_q	"Needs[\"Dep549`\"];InputString[\"q: \"]"	inputs=["one", "two", "three", "four", "five", "six"]	files=[]
ok
test entries::red_nested_resume_then_error ... SOURCE	red_nested_resume_then_error	"Get[\"Outer549`\"]"	inputs=["one"]	files=[("Outer549.rum", "AppendTo[Global`log,0]\nGet[\"Bad549`\"]\nAppendTo[Global`log,5]\n88"), ("Bad549.rum", "Print[\"before\"]\nGlobal`marker=10\nAppendTo[Global`log,1]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nThrow[9]")]
ok
test entries::red_nested_twice_filename ... SOURCE	red_nested_twice_filename	"Get[\"Outer549`\"]"	inputs=["one", "two"]	files=[("Outer549.rum", "AppendTo[Global`log,{1,$InputFileName}]\nGet[\"Inner549`\"]\nAppendTo[Global`log,{5,$InputFileName}]\nInputString[\"outer: \"]\nAppendTo[Global`log,{7,$InputFileName}]\n77"), ("Inner549.rum", "AppendTo[Global`log,{2,$InputFileName}]\nInputString[\"inner: \"]\nAppendTo[Global`log,{4,$InputFileName}]\n44")]
ok
test entries::red_stub ... SOURCE	red_stub	"DeclarePackage[\"Dep549`\",{\"foo\"}];Dep549`foo"	inputs=["one", "two", "three", "four", "five", "six"]	files=[]
ok
test entries::red_stub_then_q ... SOURCE	red_stub_then_q	"DeclarePackage[\"Dep549`\",{\"foo\"}];Dep549`foo;InputString[\"q: \"]"	inputs=["one", "two", "three", "four", "five", "six"]	files=[]
ok
test forms::red_begin_error_after_resume ... SOURCE	red_begin_error_after_resume	"BeginPackage[\"Main549`\",{\"After549`\",\"Bad549`\"}]"	inputs=["one"]	files=[("Bad549.rum", "Print[\"before\"]\nGlobal`marker=10\nAppendTo[Global`log,1]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nThrow[9]")]
ok
test forms::red_get_abort_form ... SOURCE	red_get_abort_form	"Get[\"Abort549`\"]"	inputs=["one"]	files=[("Abort549.rum", "InputString[\"p: \"]\nAbort[]\nAppendTo[Global`log,3]\n77")]
ok
test forms::red_get_cycle_form ... SOURCE	red_get_cycle_form	"Get[\"Cycle549`\"]"	inputs=["one"]	files=[("Cycle549.rum", "InputString[\"p: \"]\nGlobal`f549[]:=Global`g549[]\nGlobal`g549[]:=Global`f549[]\nGlobal`f549[]")]
ok
test forms::red_get_dynamic_context ... SOURCE	red_get_dynamic_context	"Block[{Global`marker=20},Get[\"Dyn549`\"]];Global`marker"	inputs=["one"]	files=[("Dyn549.rum", "AppendTo[Global`log,Global`marker]\nInputString[\"p: \"]\nAppendTo[Global`log,Global`marker]\n77")]
ok
test forms::red_get_error_after_resume ... SOURCE	red_get_error_after_resume	"Get[\"Bad549`\"]"	inputs=["one"]	files=[("Bad549.rum", "Print[\"before\"]\nGlobal`marker=10\nAppendTo[Global`log,1]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nThrow[9]")]
ok
test forms::red_get_exit_form ... SOURCE	red_get_exit_form	"Get[\"Exit549`\"]"	inputs=["one"]	files=[("Exit549.rum", "Global`marker=10\nInputString[\"p: \"]\nExit[7]\nAppendTo[Global`log,3]")]
ok
test forms::red_get_input_expression ... SOURCE	red_get_input_expression	"Get[\"Input549`\"]"	inputs=["2+3"]	files=[("Input549.rum", "AppendTo[Global`log,1]\nInput[\"expr: \"]\nAppendTo[Global`log,3]\n77")]
ok
test forms::red_get_input_syntax_error ... SOURCE	red_get_input_syntax_error	"Get[\"Input549`\"]"	inputs=["]"]	files=[("Input549.rum", "Global`marker=10\nInput[\"expr: \"]\n77")]
ok
test forms::red_get_late_syntax_error ... SOURCE	red_get_late_syntax_error	"Get[\"Syntax549`\"]"	inputs=["one"]	files=[("Syntax549.rum", "Global`marker=10\nInputString[\"p: \"]\n1+*2")]
ok
test forms::red_get_message_checkpoint ... SOURCE	red_get_message_checkpoint	"Message[Global`probe::tag];Get[\"Obs549`\"];$MessageList"	inputs=["one"]	files=[("Obs549.rum", "Print[\"before\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{1,$Line,$InputFileName}]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{3,$Line,$InputFileName}]\n77")]
ok
test forms::red_get_outer_check ... SOURCE	red_get_outer_check	"Check[Get[\"Obs549`\"],999]"	inputs=["one"]	files=[("Obs549.rum", "Print[\"before\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{1,$Line,$InputFileName}]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{3,$Line,$InputFileName}]\n77")]
ok
test forms::red_get_outer_quiet ... SOURCE	red_get_outer_quiet	"Quiet[Get[\"Obs549`\"]]"	inputs=["one"]	files=[("Obs549.rum", "Print[\"before\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{1,$Line,$InputFileName}]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{3,$Line,$InputFileName}]\n77")]
ok
test forms::red_get_return_form ... SOURCE	red_get_return_form	"Get[\"Return549`\"]"	inputs=["one"]	files=[("Return549.rum", "InputString[\"p: \"]\nReturn[8]\nAppendTo[Global`log,3]\n77")]
ok
test forms::red_get_streams_line ... SOURCE	red_get_streams_line	"Get[\"Obs549`\"]"	inputs=["one"]	files=[("Obs549.rum", "Print[\"before\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{1,$Line,$InputFileName}]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nAppendTo[Global`log,{3,$Line,$InputFileName}]\n77")]
ok
test forms::red_get_twice ... SOURCE	red_get_twice	"Get[\"Twice549`\"]"	inputs=["one", "two"]	files=[("Twice549.rum", "AppendTo[Global`log,1]\nInputString[\"p: \"]\nAppendTo[Global`log,3]\nInputString[\"r: \"]\nAppendTo[Global`log,5]\n77")]
ok
test forms::red_lower_later_forms ... SOURCE	red_lower_later_forms	"Get[\"Lower549`\"]"	inputs=["one"]	files=[("Lower549.rum", "InputString[\"p: \"]\nBegin[\"Lower549`\"]\nanswer=42\nEnd[]\nLower549`answer")]
ok
test forms::red_needs_error_after_resume ... SOURCE	red_needs_error_after_resume	"Needs[\"Bad549`\"]"	inputs=["one"]	files=[("Bad549.rum", "Print[\"before\"]\nGlobal`marker=10\nAppendTo[Global`log,1]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nThrow[9]")]
ok
test forms::red_repeat_form_stack ... SOURCE	red_repeat_form_stack	"Get[\"Stack549`\"]"	inputs=["one", "two"]	files=[("Stack549.rum", "Global`step549[]:=(AppendTo[Global`log,1];InputString[\"same: \"];AppendTo[Global`log,3];77)\nGlobal`step549[]\nGlobal`step549[]")]
ok
test forms::red_same_form_message_scope ... SOURCE	red_same_form_message_scope	"Get[\"Scope549`\"]"	inputs=["one"]	files=[("Scope549.rum", "Message[Global`probe::tag]\nInputString[\"p: \"];AppendTo[Global`log,$MessageList];Message[Global`probe::tag];AppendTo[Global`log,$MessageList]\n77")]
ok
test forms::red_stub_error_after_resume ... SOURCE	red_stub_error_after_resume	"DeclarePackage[\"Bad549`\",{\"foo\"}];Bad549`foo"	inputs=["one", "two"]	files=[("Bad549.rum", "Print[\"before\"]\nGlobal`marker=10\nAppendTo[Global`log,1]\nInputString[\"p: \"]\nPrint[\"after\"]\nMessage[Global`probe::tag]\nThrow[9]")]
ok
test green_valueq_and_monitor_standalone_controls ... SOURCE	green_valueq	"ValueQ[InputString[\"a: \"]]"	inputs=["one", "two", "three", "four", "five", "six"]	files=[]
ok
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.08s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
running 69 tests
test bounds::lazytable_body_and_bounds ... CASE	LazyTable-body-and-bounds	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test bounds::lazytable_bound_1 ... CASE	LazyTable-bound-1	resumed=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test bounds::lazytable_bound_2 ... CASE	LazyTable-bound-2	resumed=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test bounds::lazytable_bound_3 ... CASE	LazyTable-bound-3	resumed=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, i], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test bounds::lazytable_bounds_twice ... CASE	LazyTable-bounds-twice	resumed=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("LazyList[LazyTake[LazyMap[Function[i, CompoundExpression[AppendTo[log, 99], i]], LazyRange[1, 1]], 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test bounds::lazytable_cancel_first_suspension ... CASE	LazyTable-cancel-at-1	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1]", ["lower: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1]", ["lower: "])
ok
test bounds::lazytable_cancel_second_suspension ... CASE	LazyTable-cancel-at-2	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2]", ["lower: ", "upper: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2]", ["upper: "])
ok
test bounds::lazytable_error_in_step_after_upper ... CASE	LazyTable-error-in-step-after-upper	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])
ok
test bounds::lazytable_invalid_step ... CASE	LazyTable-invalid-step	resumed=(View { status: "OK", value: Some("LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyTable[i, List[i, CompoundExpression[AppendTo[log, 1], 1], CompoundExpression[AppendTo[log, 2], Input[\"p: \"]], CompoundExpression[AppendTo[log, 3], 0]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: [] }, "List[1, 2, 3]", [])
ok
test bounds::lazytable_nonnumeric_lower_runs_later_bounds ... CASE	LazyTable-nonnumeric-lower	resumed=(View { status: "OK", value: Some("LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: ["upper", "step"] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("LazyTable[CompoundExpression[AppendTo[log, 99], i], List[i, CompoundExpression[AppendTo[log, 1], Input[\"p: \"]], CompoundExpression[AppendTo[log, 2], Print[\"upper\"], 3], CompoundExpression[AppendTo[log, 3], Print[\"step\"], 1]]]"), termination: "None", messages: [RuntimeMessage { symbol: "LazyTable", tag: "argform", text: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step" }], prints: ["upper", "step"] }, "List[1, 2, 3]", [])
ok
test cancel_first_suspension ... CASE	Lazy-cancel-at-1	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "List[1, 2]", [])
ok
test cancel_second_suspension ... CASE	Lazy-cancel-at-2	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2, 3, 4]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary", "boundary"] }, "List[1, 2, 3, 4]", [])
ok
test depth::depth_cancel_deep ... CASE	Depth-cancel-2000	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "1", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "1", [])
ok
test depth::depth_cancel_shallow ... CASE	Depth-cancel-32	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "1", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["boundary"] }, "1", [])
ok
test depth::depth_drop_deep ... CASE	Depth-drop-2000	resumed=(View { status: "OK", value: Some("2000"), termination: "None", messages: [], prints: [] }, "2001", [])	queued=(View { status: "OK", value: Some("2000"), termination: "None", messages: [], prints: [] }, "2001", [])
ok
test depth::depth_drop_shallow ... CASE	Depth-drop-32	resumed=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "33", [])	queued=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "33", [])
ok
test depth::depth_inner_deep ... CASE	Depth-inner-2000	resumed=(View { status: "OK", value: Some("List[2000]"), termination: "None", messages: [], prints: [] }, "2000", ["p: "])	queued=(View { status: "OK", value: Some("List[2000]"), termination: "None", messages: [], prints: [] }, "2000", [])
ok
test depth::depth_inner_shallow ... CASE	Depth-inner-32	resumed=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", ["p: "])	queued=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", [])
ok
test depth::depth_invalid_zero_deep ... CASE	Depth-invalid_zero-2000	resumed=(View { status: "OK", value: Some("Take"), termination: "None", messages: [], prints: [] }, "0", [])	queued=(View { status: "OK", value: Some("Take"), termination: "None", messages: [], prints: [] }, "0", [])
ok
test depth::depth_invalid_zero_shallow ... CASE	Depth-invalid_zero-32	resumed=(View { status: "OK", value: Some("Take"), termination: "None", messages: [], prints: [] }, "0", [])	queued=(View { status: "OK", value: Some("Take"), termination: "None", messages: [], prints: [] }, "0", [])
ok
test depth::depth_no_pull_deep ... CASE	Depth-no_pull-2000	resumed=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "0", [])	queued=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "0", [])
ok
test depth::depth_no_pull_shallow ... CASE	Depth-no_pull-32	resumed=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "0", [])	queued=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "0", [])
ok
test depth::depth_outer_deep ... CASE	Depth-outer-2000	resumed=(View { status: "OK", value: Some("List[2000]"), termination: "None", messages: [], prints: [] }, "2000", ["p: "])	queued=(View { status: "OK", value: Some("List[2000]"), termination: "None", messages: [], prints: [] }, "2000", [])
ok
test depth::depth_outer_shallow ... CASE	Depth-outer-32	resumed=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", ["p: "])	queued=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", [])
ok
test depth::depth_pull_deep ... CASE	Depth-pull-2000	resumed=(View { status: "OK", value: Some("2000"), termination: "None", messages: [], prints: [] }, "2000", [])	queued=(View { status: "OK", value: Some("2000"), termination: "None", messages: [], prints: [] }, "2000", [])
ok
test depth::depth_pull_shallow ... CASE	Depth-pull-32	resumed=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "32", [])	queued=(View { status: "OK", value: Some("32"), termination: "None", messages: [], prints: [] }, "32", [])
ok
test depth::depth_select_deep ... CASE	Depth-select-2000	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "4001", [])	queued=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "4001", [])
ok
test depth::depth_select_shallow ... CASE	Depth-select-32	resumed=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "65", [])	queued=(View { status: "OK", value: Some("2"), termination: "None", messages: [], prints: [] }, "65", [])
ok
test depth::depth_twice_deep ... CASE	Depth-twice-2000	resumed=(View { status: "OK", value: Some("List[2000]"), termination: "None", messages: [], prints: [] }, "2000", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[2000]"), termination: "None", messages: [], prints: [] }, "2000", [])
ok
test depth::depth_twice_shallow ... CASE	Depth-twice-32	resumed=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[32]"), termination: "None", messages: [], prints: [] }, "32", [])
ok
test depth::descriptor_control_deep ... DESCRIPTOR CONTROL depth=2000 stack=2097152 result=0 teardown=complete
ok
test lazy_adapter_unwind_twice ... CASE	Lazy-adapter-unwind-twice	resumed=(View { status: "OK", value: Some("List[List[1], List[2], List[3]]"), termination: "None", messages: [], prints: [] }, "List[List[inner, 1], List[outer, 1], List[inner, 2], List[outer, 2], List[inner, 3], List[outer, 3]]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[List[1], List[2], List[3]]"), termination: "None", messages: [], prints: [] }, "List[List[inner, 1], List[outer, 1], List[inner, 2], List[outer, 2], List[inner, 3], List[outer, 3]]", [])
ok
test lazy_infinite_after_resume ... CASE	Lazy-infinite-after-resume	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationWorkLimit\"]"), termination: "Some(IterationWorkLimit)", messages: [RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }], prints: [] }, "List[1, 2, 3, 4]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationWorkLimit\"]"), termination: "Some(IterationWorkLimit)", messages: [RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }], prints: [] }, "List[1, 2, 3, 4]", [])
ok
test lazy_rerederive ... CASE	Lazy-rerederive	resumed=(View { status: "OK", value: Some("List[List[1, \"two\", 3], List[1, \"two\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 1, 2, 3]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[List[1, \"two\", 3], List[1, \"two\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 1, 2, 3]", [])
ok
test lazy_source_budget_exact ... CASE	Lazy-source-budget-exact	resumed=(View { status: "OK", value: Some("List[3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", ["p: "])	queued=(View { status: "OK", value: Some("List[3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])
ok
test lazydrop_inner ... CASE	LazyDrop-inner	resumed=(View { status: "OK", value: Some("List[3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", ["p: "])	queued=(View { status: "OK", value: Some("List[3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])
ok
test lazydrop_twice ... CASE	LazyDrop-twice	resumed=(View { status: "OK", value: Some("5"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("5"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ok
test lazyfirst_consumer ... CASE	LazyFirst-consumer	resumed=(View { status: "OK", value: Some("List[\"two\"]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[\"two\"]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])
ok
test lazyfirst_empty_default ... CASE	LazyFirst-empty-default	resumed=(View { status: "OK", value: Some("99"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("99"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test lazyfirst_empty_message ... CASE	LazyFirst-empty-message	resumed=(View { status: "OK", value: Some("First[List[]]"), termination: "None", messages: [RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }, RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("First[List[]]"), termination: "None", messages: [RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }, RuntimeMessage { symbol: "First", tag: "nofirst", text: "List[] has zero length and no first element." }], prints: [] }, "List[1, 2, 3]", [])
ok
test lazyfoldlist ... CASE	LazyFoldList	resumed=(View { status: "OK", value: Some("List[0, 1, 3, 6]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[0, 1, 3, 6]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test lazyfoldlist_inner ... CASE	LazyFoldList-inner	resumed=(View { status: "OK", value: Some("List[List[], List[1], List[1, \"two\"], List[1, \"two\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[], List[1], List[1, \"two\"], List[1, \"two\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test lazyfoldlist_twice ... CASE	LazyFoldList-twice	resumed=(View { status: "OK", value: Some("List[0, 1, 3, 6, 10, 15]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[0, 1, 3, 6, 10, 15]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ok
test lazymap ... CASE	LazyMap	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test lazymap_error_in_callback ... CASE	LazyMap-error-in-callback	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: [] }, "List[1, 2]", [])
ok
test lazymap_error_next ... CASE	LazyMap-error-next	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "lazy message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])
ok
test lazymap_inner ... CASE	LazyMap-inner	resumed=(View { status: "OK", value: Some("List[List[1], List[\"two\"], List[3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[1], List[\"two\"], List[3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test lazymap_twice ... CASE	LazyMap-twice	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3, \"four\", 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3, \"four\", 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ok
test lazynestlist ... CASE	LazyNestList	resumed=(View { status: "OK", value: Some("List[1, 2, 3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])
ok
test lazynestlist_lookahead_first ... CASE	LazyNestList-lookahead-first	resumed=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "List[1]", ["p: "])	queued=(View { status: "OK", value: Some("1"), termination: "None", messages: [], prints: [] }, "List[1]", [])
ok
test lazynestlist_twice ... CASE	LazyNestList-twice	resumed=(View { status: "OK", value: Some("List[1, 2, 3, 4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3, 4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ok
test lazypart_consumer ... CASE	LazyPart-consumer	resumed=(View { status: "OK", value: Some("List[\"two\"]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[\"two\"]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])
ok
test lazypermutations_cursor ... CASE	LazyPermutations-cursor	resumed=(View { status: "OK", value: Some("List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]"), termination: "None", messages: [], prints: [] }, "List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]"), termination: "None", messages: [], prints: [] }, "List[List[1, 2, 3], List[1, 3, 2], List[2, 1, 3]]", [])
ok
test lazyselect ... CASE	LazySelect	resumed=(View { status: "OK", value: Some("List[1, 2, 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test lazyselect_inner ... CASE	LazySelect-inner	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test lazyselect_never_after_resume ... CASE	LazySelect-never-after-resume	resumed=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationWorkLimit\"]"), termination: "Some(IterationWorkLimit)", messages: [RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }], prints: [] }, "List[1, 2, 3, 4]", ["p: "])	queued=(View { status: "OK", value: Some("TerminatedEvaluation[\"IterationWorkLimit\"]"), termination: "Some(IterationWorkLimit)", messages: [RuntimeMessage { symbol: "General", tag: "reslim", text: "evaluation exceeded iteration work limit (4)" }], prints: [] }, "List[1, 2, 3, 4]", [])
ok
test lazyselect_reject_resumed_candidate ... CASE	LazySelect-reject-resumed-candidate	resumed=(View { status: "OK", value: Some("List[1, 3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, 3, 4]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4]", [])
ok
test lazyselect_twice ... CASE	LazySelect-twice	resumed=(View { status: "OK", value: Some("List[1, 2, 3, 4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3, 4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ok
test lazysubsets_cursor ... CASE	LazySubsets-cursor	resumed=(View { status: "OK", value: Some("List[List[], List[1], List[2]]"), termination: "None", messages: [], prints: [] }, "List[List[], List[1], List[2]]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[], List[1], List[2]]"), termination: "None", messages: [], prints: [] }, "List[List[], List[1], List[2]]", [])
ok
test lazytable_body ... CASE	LazyTable-body	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test lazytake_consumer ... CASE	LazyTake-consumer	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test lazytuples_cursor ... CASE	LazyTuples-cursor	resumed=(View { status: "OK", value: Some("List[List[1, 3], List[1, 4], List[2, 3]]"), termination: "None", messages: [], prints: [] }, "List[List[1, 3], List[1, 4], List[2, 3]]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[1, 3], List[1, 4], List[2, 3]]"), termination: "None", messages: [], prints: [] }, "List[List[1, 3], List[1, 4], List[2, 3]]", [])
ok
test structural::alternating_take_drop ... CASE	alternating Take/Drop skip and yield	resumed=(View { status: "OK", value: Some("List[4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ok
test structural::finite_positive_drop ... CASE	positive Drop finite exhaustion	resumed=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])
ok
test structural::finite_rejected_select ... CASE	resumed Select rejects last finite item	resumed=(View { status: "OK", value: Some("99"), termination: "None", messages: [], prints: [] }, "List[List[]]", ["p: "])	queued=(View { status: "OK", value: Some("99"), termination: "None", messages: [], prints: [] }, "List[List[]]", [])
ok
test structural::fold_init_above_suspending_adapter ... CASE	Fold init above suspending Map	resumed=(View { status: "OK", value: Some("List[0, 1]"), termination: "None", messages: [], prints: [] }, "List[List[outer, 0], List[inner, 1], List[outer, 1]]", ["p: ", "p: ", "p: "])	queued=(View { status: "OK", value: Some("List[0, 1]"), termination: "None", messages: [], prints: [] }, "List[List[outer, 0], List[inner, 1], List[outer, 1]]", [])
ok
test structural::last_take_child ... CASE	last permitted Take child	resumed=(View { status: "OK", value: Some("List[1]"), termination: "None", messages: [], prints: [] }, "List[1]", ["p: "])	queued=(View { status: "OK", value: Some("List[1]"), termination: "None", messages: [], prints: [] }, "List[1]", [])
ok
test structural::nested_realization ... CASE	callback realizes a suspending lazy list	resumed=(View { status: "OK", value: Some("List[11, 12]"), termination: "None", messages: [], prints: [] }, "List[List[outer, 1], List[inner, 10], List[outer, 2], List[inner, 10]]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[11, 12]"), termination: "None", messages: [], prints: [] }, "List[List[outer, 1], List[inner, 10], List[outer, 2], List[inner, 10]]", [])
ok
test structural::one_callback_two_inputs ... CASE	one callback two inputs	resumed=(View { status: "OK", value: Some("List[2]"), termination: "None", messages: [], prints: [] }, "List[1]", ["first: ", "second: "])	queued=(View { status: "OK", value: Some("List[2]"), termination: "None", messages: [], prints: [] }, "List[1]", [])
ok
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.80s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
running 21 tests
test differences ... CASE	Differences	resumed=(View { status: "OK", value: Some("List[Plus[-1, a], 1, 1]"), termination: "None", messages: [], prints: [] }, "List[2]", ["p: "])	queued=(View { status: "OK", value: Some("List[Plus[-1, a], 1, 1]"), termination: "None", messages: [], prints: [] }, "List[2]", [])
ok
test differences_cancel ... CANCEL	differences	stop=0	log=List[2]	termination=Some(Cancelled)
ok
test differences_error_after_resume ... CASE	differences-error	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[2, 4]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[2, 4]", [])
ok
test differences_order_two ... CASE	Differences-order-two	resumed=(View { status: "OK", value: Some("List[Plus[1, Times[-1, Plus[-1, a]]], 0]"), termination: "None", messages: [], prints: [] }, "List[2]", ["p: "])	queued=(View { status: "OK", value: Some("List[Plus[1, Times[-1, Plus[-1, a]]], 0]"), termination: "None", messages: [], prints: [] }, "List[2]", [])
ok
test differences_twice ... CASE	Differences-twice	resumed=(View { status: "OK", value: Some("List[Plus[-1, a], 1, Plus[-3, b], 1]"), termination: "None", messages: [], prints: [] }, "List[2, 4]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[Plus[-1, a], 1, Plus[-3, b], 1]"), termination: "None", messages: [], prints: [] }, "List[2, 4]", [])
ok
test dot_cancel ... CANCEL	dot	stop=0	log=List[1, 3]	termination=Some(Cancelled)
ok
test dot_chain ... CASE	Dot-chain	resumed=(View { status: "OK", value: Some("List[List[12, 18], List[12, 18]]"), termination: "None", messages: [], prints: [] }, "List[1, 3, 2, 4, 1, 3, 2, 4]", ["p: "])	queued=(View { status: "OK", value: Some("List[List[12, 18], List[12, 18]]"), termination: "None", messages: [], prints: [] }, "List[1, 3, 2, 4, 1, 3, 2, 4]", [])
ok
test dot_error_after_resume ... CASE	dot-error	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 3, 2, 4, 1, 3, 2, 4, 99]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 3, 2, 4, 1, 3, 2, 4, 99]", [])
ok
test dot_twice ... CASE	Dot-twice	resumed=(View { status: "OK", value: Some("List[List[12, 18], List[12, 18]]"), termination: "None", messages: [], prints: [] }, "List[1, 3, 2, 4, 1, 3, 2, 4]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[List[12, 18], List[12, 18]]"), termination: "None", messages: [], prints: [] }, "List[1, 3, 2, 4, 1, 3, 2, 4]", [])
ok
test option_value_cancel ... CANCEL	option_value	stop=0	log=List[1, 2]	termination=Some(Cancelled)
ok
test option_value_error_after_resume ... CASE	option_value-error	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])
ok
test optionvalue_many ... CASE	OptionValue-many	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ok
test optionvalue_twice ... CASE	OptionValue-twice	resumed=(View { status: "OK", value: Some("List[1, \"two\", 3, \"four\", 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[1, \"two\", 3, \"four\", 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ok
test replace_repeated_cancel ... CANCEL	replace_repeated	stop=0	log=List[1, 2]	termination=Some(Cancelled)
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
ok
test scan_error_after_resume ... CASE	scan-error	resumed=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: 9 with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "loop message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[9] returned to top level." }], prints: ["before", "after"] }, "List[1, 2, 3]", [])
ok
test scan_twice ... CASE	Scan-twice	resumed=(View { status: "OK", value: Some("Null"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("Null"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ok
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
running 193 tests
test ast::tests::constructors_preserve_kinds_and_spans_while_equality_ignores_spans ... ok
test ast::tests::display_formats_less_common_nodes_and_precedence_parentheses ... ok
test ast::tests::operator_tokens_cover_all_ast_operator_variants ... ok
test ast::tests::syntax_string_display_escapes_round_trippable_characters ... ok
test document::tests::debug_dump_deep_head_chain_completes_on_small_stack ... ok
test document::tests::debug_dump_matches_alternate_debug_across_front_and_tail ... ok
test document::tests::debug_dump_preserves_every_syntax_variant_and_leaf_layout ... ok
test document::tests::parsed_source_debug_dump_includes_syntax_tokens_and_trivia ... ok
test document::tests::parsed_source_preserves_source_slices_locations_and_trivia ... ok
test document::tests::source_diagnostic_exposes_structured_span_data ... ok
test document::tests::source_diagnostic_renders_line_column_and_snippet ... ok
test document::tests::source_diagnostic_renders_multiline_spans ... ok
test document::tests::source_diagnostic_renders_related_opening_span ... ok
test format_request::tests::parse_format_cli_args_accepts_prefix_and_suffix_width_flags ... ok
test format_request::tests::parse_format_command_accepts_prefix_and_suffix_width_flags ... ok
test format_request::tests::parse_format_command_rejects_invalid_width_with_shared_message ... ok
test format_request::tests::parse_format_command_rejects_unknown_leading_flags ... ok
test formatter::tests::formatter_accounts_for_current_column_when_fitting_nested_groups ... ok
test formatter::tests::formatter_actively_expands_binary_expressions_with_long_inline_comments ... ok
test formatter::tests::formatter_actively_expands_calls_with_escaped_string_literals ... ok
test formatter::tests::formatter_actively_expands_calls_with_long_inline_comments ... ok
test formatter::tests::formatter_actively_expands_long_call_argument_lists ... ok
test formatter::tests::formatter_actively_expands_long_list_literals ... ok
test formatter::tests::formatter_actively_expands_wide_binary_chains_inside_calls ... ok
test formatter::tests::formatter_actively_expands_wide_binary_rhs_inside_calls ... ok
test formatter::tests::formatter_actively_expands_wide_top_level_binary_chains ... ok
test formatter::tests::formatter_actively_expands_wide_top_level_binary_expressions ... ok
test formatter::tests::formatter_actively_expands_wide_top_level_lists ... ok
test formatter::tests::formatter_actively_expands_wide_two_argument_calls ... ok
test formatter::tests::formatter_canonicalizes_core_spacing ... ok
test formatter::tests::formatter_dedents_closing_delimiters_after_multiline_comments ... ok
test formatter::tests::formatter_dedents_nested_closing_delimiters_after_multiline_comments ... ok
test formatter::tests::formatter_expands_long_nested_calls_without_expanding_the_parent ... ok
test formatter::tests::formatter_expands_parent_calls_when_nested_binary_chains_make_the_parent_wide ... ok
test formatter::tests::formatter_expands_wide_parent_calls_around_nested_lists ... ok
test formatter::tests::formatter_handles_association_literals ... ok
test formatter::tests::formatter_handles_deeply_nested_multiline_comments_with_following_inline_comments ... ok
test formatter::tests::formatter_handles_mixed_inline_and_multiline_comments_inside_nested_calls ... ok
test formatter::tests::formatter_handles_mixed_inline_and_multiline_comments_inside_nested_lists ... ok
test formatter::tests::formatter_handles_multiline_comments_at_multiple_nesting_levels ... ok
test formatter::tests::formatter_handles_multiline_comments_in_parenthesized_binary_expressions_with_inline_comments ... ok
test formatter::tests::formatter_handles_nested_lists_followed_by_outer_comment_blocks ... ok
test formatter::tests::formatter_handles_stacked_comments_before_nested_closing_delimiters ... ok
test formatter::tests::formatter_handles_stacked_rhs_comments_inside_parenthesized_calls ... ok
test formatter::tests::formatter_indents_binary_chains_with_mixed_inline_and_multiline_comments_inside_calls ... ok
test formatter::tests::formatter_indents_binary_chains_with_multiline_comments_inside_calls ... ok
test formatter::tests::formatter_indents_binary_chains_with_multiline_comments_inside_parenthesized_calls ... ok
test formatter::tests::formatter_indents_multiline_comments_before_binary_rhs_inside_calls ... ok
test formatter::tests::formatter_indents_multiline_comments_before_nested_outer_arguments ... ok
test formatter::tests::formatter_indents_multiline_comments_inside_nested_calls ... ok
test formatter::tests::formatter_indents_multiline_comments_inside_nested_lists ... ok
test formatter::tests::formatter_indents_multiline_comments_inside_parenthesized_binary_rhs ... ok
test formatter::tests::formatter_indexed_break_points_preserve_every_wide_list_item ... ok
test formatter::tests::formatter_is_idempotent_on_regression_corpus ... ok
test formatter::tests::formatter_keeps_comparison_chains_and_parenthesis_boundaries ... ok
test formatter::tests::formatter_keeps_lhs_inline_comments_while_expanding_binary_chains ... ok
test formatter::tests::formatter_keeps_max_width_binary_expressions_inline ... ok
test formatter::tests::formatter_keeps_max_width_calls_inline ... ok
test formatter::tests::formatter_keeps_multiline_comments_attached_in_place ... ok
test formatter::tests::formatter_keeps_parent_calls_inline_when_nested_binary_chains_fit ... ok
test formatter::tests::formatter_keeps_short_binary_chains_inline ... ok
test formatter::tests::formatter_keeps_stacked_multiline_comments_grouped_with_the_following_arg ... ok
test formatter::tests::formatter_leaves_top_level_multiline_binary_rhs_flush_left ... ok
test formatter::tests::formatter_normalizes_binary_chains_with_middle_multiline_comments ... ok
test formatter::tests::formatter_normalizes_binary_chains_with_mixed_inline_and_multiline_comments ... ok
test formatter::tests::formatter_normalizes_binary_chains_with_trailing_multiline_comments ... ok
test formatter::tests::formatter_normalizes_partially_multiline_binary_chains ... ok
test formatter::tests::formatter_preserves_inline_comments_inside_calls_and_binary_ops ... ok
test formatter::tests::formatter_preserves_multiline_string_literals_verbatim ... ok
test formatter::tests::formatter_preserves_outer_comment_trivia ... ok
test formatter::tests::formatter_preserves_plain_multiline_binary_rhs_inside_calls ... ok
test formatter::tests::formatter_preserves_plain_multiline_call_arguments ... ok
test formatter::tests::formatter_preserves_plain_multiline_list_elements ... ok
test formatter::tests::formatter_preserves_plain_multiline_nested_call_arguments ... ok
test formatter::tests::formatter_respects_custom_line_width_for_binary_chains ... ok
test formatter::tests::formatter_respects_custom_line_width_for_calls ... ok
test formatter::tests::formatter_spaces_apply_sugar ... ok
test formatter::tests::formatter_spaces_compound_expression_sugar ... ok
test formatter::tests::formatter_spaces_functional_operator_sugars ... ok
test formatter::tests::formatter_wraps_right_associative_assignment_chains ... ok
test lexer::tests::comments_do_not_shift_token_spans_for_following_tokens ... ok
test lexer::tests::decodes_named_character_escapes_in_code_symbols ... ok
test lexer::tests::decodes_named_character_escapes_in_strings ... ok
test lexer::tests::decodes_named_character_operator_with_source_span ... ok
test lexer::tests::lexes_bare_dot_without_regressing_decimal_or_repeated_tokens ... ok
test lexer::tests::lexes_context_qualified_symbol ... ok
test lexer::tests::lexes_escaped_multiline_string_literals ... ok
test lexer::tests::lexes_multi_char_operators ... ok
test lexer::tests::lexes_new_infix_operators_before_shared_prefixes ... ok
test lexer::tests::lexes_postfix_operator_tokens_without_splitting_decimal_literals ... ok
test lexer::tests::lexes_raw_multiline_string_literals_only_in_operand_position ... ok
test lexer::tests::lexes_replace_repeated_before_replace_all_prefix ... ok
test lexer::tests::lexes_specific_slash_operators_before_postfix_apply ... ok
test lexer::tests::lexes_string_literals_with_basic_and_unicode_escapes ... ok
test lexer::tests::lexes_unicode_letter_symbols_used_by_package_source ... ok
test lexer::tests::pins_multiline_string_delimiter_greedy_edges ... ok
test lexer::tests::preserves_comments_and_whitespace_as_leading_trivia ... ok
test lexer::tests::preserves_trailing_trivia_on_end_token ... ok
test lexer::tests::reports_unexpected_character_byte_span ... ok
test lexer::tests::reports_unknown_named_character_escapes ... ok
test lexer::tests::reports_unterminated_multiline_string_literals_with_spans ... ok
test lexer::tests::reports_unterminated_nested_comment ... ok
test lexer::tests::skips_comments_and_nested_comments_as_trivia ... ok
test lexer::tests::tokens_carry_byte_spans ... ok
test lexer::tests::treats_initial_shebang_as_script_comment_trivia ... ok
test parser::script::tests::crlf_sources_lose_their_carriage_returns ... ok
test parser::script::tests::hard_errors_poison_through_the_offending_line ... ok
test parser::script::tests::incomplete_tail_reports_the_legacy_shaped_error ... ok
test parser::script::tests::lex_failures_are_reported_for_the_caller_to_fall_back ... ok
test parser::script::tests::parsed_forms_carry_syntax_and_poisoned_forms_do_not ... ok
test parser::script::tests::required_continuations_cross_line_boundaries ... ok
test parser::script::tests::shebang_blank_and_comment_lines_never_start_a_form ... ok
test parser::script::tests::splits_on_new_line_tokens_at_depth_zero ... ok
test parser::script::tests::stops_before_new_line_operators_and_postfixes ... ok
test parser::tests::apply_compound::independent_statement_spines_do_not_accumulate ... ok
test parser::tests::apply_compound::parses_apply_and_composition_operator_precedence ... ok
test parser::tests::apply_compound::parses_apply_level_operators_as_right_associative ... ok
test parser::tests::apply_compound::parses_compound_expression_sugar ... ok
test parser::tests::apply_compound::parses_empty_mid_chain_statement_as_locally_left_leaning ... ok
test parser::tests::apply_compound::parses_function_postfix_as_map_lhs ... ok
test parser::tests::apply_compound::parses_function_postfix_on_set_rhs_before_map ... ok
test parser::tests::apply_compound::parses_long_single_form_statement_chain ... ok
test parser::tests::apply_compound::parses_postfix_apply_as_left_associative ... ok
test parser::tests::apply_compound::parses_postfix_apply_between_set_and_replace_all_precedence ... ok
test parser::tests::apply_compound::parses_right_associative_apply_sugar ... ok
test parser::tests::apply_compound::pins_statement_chain_boundary_with_dedicated_error ... ok
test parser::tests::apply_compound::rejects_empty_statement_spine_without_overflowing ... ok
test parser::tests::basics::parses_association_literals ... ok
test parser::tests::basics::parses_large_integer_literal ... ok
test parser::tests::basics::parses_lists_and_nested_calls ... ok
test parser::tests::basics::parses_multiline_string_literals_as_string_atoms ... ok
test parser::tests::basics::parses_simple_decimal_literal_as_numeric_atom ... ok
test parser::tests::basics::pins_nested_call_parse_depth_boundary ... ok
test parser::tests::basics::pins_power_chain_operator_depth_boundary ... ok
test parser::tests::basics::rejects_deep_postfix_blank_chain_without_panicking ... ok
test parser::tests::basics::rejects_deep_prefix_chain_without_panicking ... ok
test parser::tests::basics::rejects_deep_right_associative_chain_without_panicking ... ok
test parser::tests::basics::rejects_too_deep_nested_call_without_panicking ... ok
test parser::tests::diagnostics::distinguishes_empty_input_from_incomplete_input ... ok
test parser::tests::diagnostics::reports_bare_pure_function_marker_as_hard_error ... ok
test parser::tests::diagnostics::reports_incomplete_association_input ... ok
test parser::tests::diagnostics::reports_incomplete_call_input ... ok
test parser::tests::diagnostics::reports_incomplete_double_bracket_input ... ok
test parser::tests::diagnostics::reports_incomplete_group_input ... ok
test parser::tests::diagnostics::reports_incomplete_rhs_after_binary_operator ... ok
test parser::tests::diagnostics::reports_malformed_call_arguments_without_marking_input_incomplete ... ok
test parser::tests::diagnostics::reports_malformed_list_items_without_marking_input_incomplete ... ok
test parser::tests::diagnostics::reports_missing_closing_tilde_in_infix_application ... ok
test parser::tests::diagnostics::reports_trailing_pattern_tokens_as_hard_errors ... ok
test parser::tests::diagnostics::reports_unexpected_closing_delimiters_with_token_labels ... ok
test parser::tests::diagnostics::reports_unterminated_multiline_string_as_incomplete_input ... ok
test parser::tests::diagnostics::script_form_source_map_preserves_normalized_text_and_raw_file_offsets ... ok
test parser::tests::diagnostics::source_offset_applies_to_syntax_parser_and_lexer_error_spans ... ok
test parser::tests::infix_operators::comparison_chains_do_not_cross_parenthesis_boundaries ... ok
test parser::tests::infix_operators::parses_implicit_multiplication_juxtaposition ... ok
test parser::tests::infix_operators::parses_logical_and_comparison_operator_precedence ... ok
test parser::tests::infix_operators::parses_new_infix_binary_operators ... ok
test parser::tests::infix_operators::parses_prefix_minus_with_power_precedence ... ok
test parser::tests::infix_operators::parses_string_expression_infix_sugar ... ok
test parser::tests::infix_operators::parses_string_join_infix_sugar ... ok
test parser::tests::infix_operators::parses_tilde_infix_application_as_call ... ok
test parser::tests::left_chain_limits::left_chain_homogeneous_boundary_is_exact ... ok
test parser::tests::left_chain_limits::left_chain_implicit_times_boundary_is_exact ... ok
test parser::tests::left_chain_limits::left_chain_independent_spines_have_independent_counts ... ok
test parser::tests::left_chain_limits::left_chain_limit_diagnostic_marks_the_offending_operator ... ok
test parser::tests::left_chain_limits::left_chain_limit_does_not_swallow_later_script_forms ... ok
test parser::tests::left_chain_limits::left_chain_limit_error_is_terminal ... ok
test parser::tests::left_chain_limits::left_chain_mixed_boundary_is_exact ... ok
test parser::tests::parts_spans_comments::attaches_spans_to_root_and_nested_nodes ... ok
test parser::tests::parts_spans_comments::keeps_group_and_postfix_delimiters_inside_node_spans ... ok
test parser::tests::parts_spans_comments::parses_expressions_with_comments_between_tokens ... ok
test parser::tests::parts_spans_comments::parses_part_and_span_sugar_into_canonical_forms ... ok
test parser::tests::patterns::parses_colon_named_pattern_sugar ... ok
test parser::tests::patterns::parses_named_patterns ... ok
test parser::tests::patterns::parses_optional_default_patterns ... ok
test parser::tests::patterns::parses_pattern_alternatives_sugar ... ok
test parser::tests::patterns::parses_pattern_test_sugar ... ok
test parser::tests::rules_slots_assignments::does_not_merge_slot_specs_across_trivia ... ok
test parser::tests::rules_slots_assignments::does_not_merge_slots_across_trivia_into_a_sequence_slot ... ok
test parser::tests::rules_slots_assignments::parses_factorial_derivative_and_repeated_postfix_forms ... ok
test parser::tests::rules_slots_assignments::parses_message_name_before_assignment ... ok
test parser::tests::rules_slots_assignments::parses_message_name_inside_calls ... ok
test parser::tests::rules_slots_assignments::parses_replace_repeated_precedence ... ok
test parser::tests::rules_slots_assignments::parses_rule_and_replace_all_precedence ... ok
test parser::tests::rules_slots_assignments::parses_rule_delayed_rhs_before_assignment_rhs ... ok
test parser::tests::rules_slots_assignments::parses_rule_expression_as_pure_function_body ... ok
test parser::tests::rules_slots_assignments::parses_rule_rhs_before_assignment_rhs ... ok
test parser::tests::rules_slots_assignments::parses_slots_and_pure_functions ... ok
test parser::tests::rules_slots_assignments::parses_tagset_before_assignment_rhs ... ok
test parser::tests::rules_slots_assignments::parses_unset_postfix ... ok
test trivia::tests::parsed_comments_attach_to_following_tokens_and_end_token_trivia ... ok
test trivia::tests::syntax_token_kind_labels_cover_atoms_punctuation_and_operators ... ok
test trivia::tests::trivia_constructors_labels_and_lexed_source_accessors_preserve_ranges ... ok
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.55s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
running 5 tests
test tests::host_eval_response_keeps_canonical_contract_bytes ... ok
test tests::host_eval_response_uses_hold_transparent_canonical_contract_text ... ok
test tests::host_failure_renders_messages_before_syntax_diagnostic ... ok
test tests::host_inspect_renders_summary_and_definition_lines ... ok
test tests::host_message_catalog_renders_immediate_and_delayed_templates ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
running 7 tests
test crates/rum-derive/src/lib.rs - derive_accumulator_chain_node (line 24) - compile fail ... error: AccumulatorChainNode requires exactly one #[accumulator_link] field
error: aborting due to 2 previous errors
ok
test crates/rum-derive/src/lib.rs - derive_accumulator_chain_node (line 32) - compile fail ... error: AccumulatorChainNode requires exactly one #[accumulator_link] field
error: aborting due to 2 previous errors
ok
test crates/rum-derive/src/lib.rs - derive_accumulator_chain_node (line 45) - compile fail ... error: the marked link must have type AccumulatorChain<Self> (or AccumulatorChain<NodeType>)
error: aborting due to 1 previous error
ok
test crates/rum-derive/src/lib.rs - derive_accumulator_chain_node (line 56) - compile fail ... error[E0277]: the trait bound `Node: AccumulatorChainNode` is not satisfied
error: aborting due to 1 previous error
ok
test crates/rum-derive/src/lib.rs - derive_accumulator_chain_node (line 63) - compile fail ... error[E0277]: the trait bound `Node: BoundedDestruction` is not satisfied
error: aborting due to 1 previous error
ok
test crates/rum-derive/src/lib.rs - derive_frame_root_trace (line 102) - compile fail ... error: FrameRootTrace cannot be derived for unions; use a struct or enum with traceable fields
error: aborting due to 1 previous error
ok
test crates/rum-derive/src/lib.rs - derive_frame_root_trace (line 86) - compile fail ... error[E0277]: the trait bound `MissingTraversal: FrameRootTrace` is not satisfied
error: aborting due to 1 previous error
ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
   Doc-tests rum_host
running 1 test
test crates/rum-host/src/lib.rs - (line 12) ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.06s
   Doc-tests rum_jupyter
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
running 22 tests
test crates/rum-runtime/src/runtime/accounting_mode/options.rs - runtime::accounting_mode::options::RetainedOptions (line 25) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/options.rs - runtime::accounting_mode::options::RetainedOptions (line 31) - compile fail ... error[E0616]: field `values` of struct `RetainedOptions` is private
error: aborting due to 1 previous error
ok
test crates/rum-runtime/src/runtime/accounting_mode/options.rs - runtime::accounting_mode::options::RetainedOptions (line 36) - compile fail ... error: lifetime may not live long enough
error: aborting due to 2 previous errors
ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 107) - compile fail ... error[E0521]: borrowed data escapes outside of closure
error: aborting due to 1 previous error
ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 120) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 137) - compile fail ... error[E0382]: use of moved value: `cx`
error: aborting due to 1 previous error
ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 153) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 165) - compile fail ... error[E0616]: field `error` of struct `CallbackFailure` is private
error: aborting due to 1 previous error
ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 18) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 26) - compile fail ... error[E0277]: the trait bound `Untraceable: FrameRootTrace` is not satisfied
error: aborting due to 1 previous error
ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 36) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 44) - compile fail ... error[E0624]: associated function `new` is private
error: aborting due to 1 previous error
ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 54) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 64) - compile fail ... error[E0382]: use of moved value: `values`
error: aborting due to 1 previous error
ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 77) - compile ... ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 84) - compile fail ... error[E0616]: field `values` of struct `RetainedValues` is private
error: aborting due to 1 previous error
ok
test crates/rum-runtime/src/runtime/accounting_mode/retained.rs - runtime::accounting_mode::retained::RetainedValues (line 96) - compile ... ok
test crates/rum-runtime/src/runtime/native_stack_guard.rs - runtime::native_stack_guard::NativeStackDepth (line 19) - compile fail ... error[E0405]: cannot find trait `FrameRootVisitor` in this scope
error: aborting due to 1 previous error
ok
test crates/rum-runtime/src/runtime/native_stack_guard.rs - runtime::native_stack_guard::run_with_native_stack_guard (line 79) - compile fail ... error: lifetime may not live long enough
error: aborting due to 1 previous error
ok
test crates/rum-runtime/src/runtime/options/reentry.rs - runtime::options::reentry::OptionReentry (line 160) - compile ... ok
test crates/rum-runtime/src/runtime/options/reentry.rs - runtime::options::reentry::OptionReentry (line 171) - compile fail ... error[E0061]: this method takes 4 arguments but 3 arguments were supplied
error: aborting due to 1 previous error
ok
test crates/rum-runtime/src/runtime/options/reentry.rs - runtime::options::reentry::OptionReentryDeclaration (line 13) ... ok
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.85s
   Doc-tests rum_stdlib
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Workspace exit code: **0**. Native ZeroMQ platform skips recorded: **9**.

Ownership discovery was replayed without inventory input. Existing identity/reason/status/disposition/exclusion fields are unchanged; the only additions are the two named derives and one shared capture macro, all UnknownConstruct. No UnknownHostPayload, unknown-storage, self-reaching addition/removal, or row in untouched code occurred. Full discovery output: `.temp/brief-564/inventory/discovery.log`. Exact existing-row movements and new rows:

```json
{
  "host_payloads.tsv": {
    "before_count": 63,
    "after_count": 63,
    "changed": [],
    "added": []
  },
  "unknown_attributes.tsv": {
    "before_count": 1443,
    "after_count": 1445,
    "changed": [
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t16\t1\tunresolved attribute/expansion derive (Clone , Copy , FrameRootTrace)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t17\t1\tunresolved attribute/expansion derive (Clone , Copy , FrameRootTrace)",
        "reason": "position"
      }
    ],
    "added": [
      "UnknownConstruct\tcrates/rum-stdlib/src/lazy/table_continuation.rs\t10\t1\tunresolved attribute/expansion derive (Clone , Copy , FrameRootTrace)",
      "UnknownConstruct\tcrates/rum-stdlib/src/lazy/table_continuation.rs\t17\t1\tunresolved attribute/expansion derive (FrameRootTrace , PureFrameAbandon)"
    ]
  },
  "unknown_captures.tsv": {
    "before_count": 2938,
    "after_count": 2938,
    "changed": [
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t151\t25\tclosure captures: || (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t152\t25\tclosure captures: || (capture types/modes unresolved)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t155\t25\tclosure captures: || (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t156\t25\tclosure captures: || (capture types/modes unresolved)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t233\t15\tclosure captures: || (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t237\t15\tclosure captures: || (capture types/modes unresolved)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t305\t30\tclosure captures: |items| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t357\t30\tclosure captures: |items| (capture types/modes unresolved)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t305\t55\tclosure captures: |item| (capture types/modes unresolved)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t357\t55\tclosure captures: |item| (capture types/modes unresolved)",
        "reason": "position"
      }
    ],
    "added": []
  },
  "unknown_macros.tsv": {
    "before_count": 3113,
    "after_count": 3114,
    "changed": [
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t53\t9\tunexpanded macro vec! (ends at 53:69)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t54\t9\tunexpanded macro vec! (ends at 54:69)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t76\t9\tunexpanded macro vec! (ends at 76:65)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t77\t9\tunexpanded macro vec! (ends at 77:65)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t99\t9\tunexpanded macro vec! (ends at 99:51)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t100\t9\tunexpanded macro vec! (ends at 100:51)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t115\t9\tunexpanded macro vec! (ends at 115:45)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t116\t9\tunexpanded macro vec! (ends at 116:45)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t134\t9\tunexpanded macro vec! (ends at 134:27)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t135\t9\tunexpanded macro vec! (ends at 135:27)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t167\t9\tunexpanded macro vec! (ends at 167:26)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t168\t9\tunexpanded macro vec! (ends at 168:26)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t190\t9\tunexpanded macro vec! (ends at 190:52)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t191\t9\tunexpanded macro vec! (ends at 191:52)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t209\t9\tunexpanded macro vec! (ends at 209:27)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t210\t9\tunexpanded macro vec! (ends at 210:27)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t260\t9\tunexpanded macro vec! (ends at 260:45)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t312\t9\tunexpanded macro vec! (ends at 312:45)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t262\t54\tunexpanded macro vec! (ends at 262:70)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t314\t54\tunexpanded macro vec! (ends at 314:70)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t263\t50\tunexpanded macro vec! (ends at 263:71)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t315\t50\tunexpanded macro vec! (ends at 315:71)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t266\t9\tunexpanded macro vec! (ends at 266:54)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t318\t9\tunexpanded macro vec! (ends at 318:54)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t292\t9\tunexpanded macro vec! (ends at 292:69)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t344\t9\tunexpanded macro vec! (ends at 344:69)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t318\t45\tunexpanded macro vec! (ends at 318:70)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t370\t45\tunexpanded macro vec! (ends at 370:70)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t332\t17\tunexpanded macro format! (ends at 332:75)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t384\t17\tunexpanded macro format! (ends at 384:75)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t339\t9\tunexpanded macro vec! (ends at 339:28)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t391\t9\tunexpanded macro vec! (ends at 391:28)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t344\t5\tunexpanded macro matches! (ends at 347:6)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t396\t5\tunexpanded macro matches! (ends at 399:6)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t354\t9\tunexpanded macro vec! (ends at 354:69)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t406\t9\tunexpanded macro vec! (ends at 406:69)",
        "reason": "position"
      },
      {
        "before": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t460\t41\tunexpanded macro vec! (ends at 460:57)",
        "after": "UnknownConstruct\tcrates/rum-stdlib/src/lazy.rs\t512\t41\tunexpanded macro vec! (ends at 512:57)",
        "reason": "position"
      }
    ],
    "added": [
      "UnknownConstruct\tcrates/rum-stdlib/src/lazy/table_continuation.rs\t37\t9\tunexpanded macro rum_runtime :: resume_or_suspend! (ends at 42:11)"
    ]
  },
  "unknown_storage.tsv": {
    "before_count": 88,
    "after_count": 88,
    "changed": [],
    "added": []
  },
  "self_reaching.tsv": {
    "before_count": 59,
    "after_count": 59,
    "changed": [],
    "removed": [],
    "added": []
  }
}
```

Evidence-only obligation movements (N:LazyTable retains lazy::eval_lazy_table and N/Pending; the adjacent LazyTuples evidence also follows its unchanged statements):

```json
[
  {
    "line": 135,
    "before": "N:LazyTable\tLazyTable\tN\tPending\tcrates/rum-stdlib/src/catalog/memory_growth_contract.rs:308\timin and imax retained while later bounds evaluate\tcrates/rum-stdlib/src/lazy.rs:244; crates/rum-stdlib/src/lazy.rs:245; crates/rum-stdlib/src/lazy.rs:246\trum_stdlib::lazy::eval_lazy_table",
    "after": "N:LazyTable\tLazyTable\tN\tPending\tcrates/rum-stdlib/src/catalog/memory_growth_contract.rs:308\timin and imax retained while later bounds evaluate\tcrates/rum-stdlib/src/lazy.rs:248; crates/rum-stdlib/src/lazy.rs:261; crates/rum-stdlib/src/lazy.rs:274\trum_stdlib::lazy::eval_lazy_table"
  },
  {
    "line": 136,
    "before": "Unsplit:LazyTuples\tLazyTuples\tUnsplit\tPending\tcrates/rum-stdlib/src/catalog/memory_growth_contract.rs\tInherited membership; subclass not re-audited\tcrates/rum-stdlib/src/lazy.rs:302,318\trum_stdlib::lazy::eval_lazy_tuples",
    "after": "Unsplit:LazyTuples\tLazyTuples\tUnsplit\tPending\tcrates/rum-stdlib/src/catalog/memory_growth_contract.rs\tInherited membership; subclass not re-audited\tcrates/rum-stdlib/src/lazy.rs:354,370\trum_stdlib::lazy::eval_lazy_tuples"
  }
]
```

All 68 contract fingerprint row changes, with the first 18 fields unchanged:

```json
[
  {
    "line": 2,
    "id": "B01",
    "status": "Pending",
    "old": "fnv1a64:ff4d13fbdd49b224",
    "new": "fnv1a64:97e9f9cbd1620a22"
  },
  {
    "line": 3,
    "id": "B02",
    "status": "Pending",
    "old": "fnv1a64:09aaf83c63fa02c5",
    "new": "fnv1a64:f25d84ddfa27f86b"
  },
  {
    "line": 4,
    "id": "B03",
    "status": "Discharged",
    "old": "fnv1a64:d62a4c998c417a8a",
    "new": "fnv1a64:b13fcc2d666ada44"
  },
  {
    "line": 5,
    "id": "B04",
    "status": "Discharged",
    "old": "fnv1a64:b725f55383f1289c",
    "new": "fnv1a64:8a14ac32dc071b8a"
  },
  {
    "line": 6,
    "id": "B05",
    "status": "Discharged",
    "old": "fnv1a64:08fd0f06c2c64541",
    "new": "fnv1a64:aee489e122b84e87"
  },
  {
    "line": 7,
    "id": "B06",
    "status": "Pending",
    "old": "fnv1a64:d745b1beb84f6e78",
    "new": "fnv1a64:07039fcfe2b95ff6"
  },
  {
    "line": 8,
    "id": "B07",
    "status": "Pending",
    "old": "fnv1a64:4cd32d30af47c671",
    "new": "fnv1a64:39cc414d8bf99f57"
  },
  {
    "line": 9,
    "id": "B08",
    "status": "Pending",
    "old": "fnv1a64:c95286f55c3f3b56",
    "new": "fnv1a64:fd013cff43ff3340"
  },
  {
    "line": 10,
    "id": "B09",
    "status": "Discharged",
    "old": "fnv1a64:93e069292e5311b0",
    "new": "fnv1a64:b5fec5daa0d638be"
  },
  {
    "line": 11,
    "id": "B10",
    "status": "Discharged",
    "old": "fnv1a64:ddaea9c138ec6bbb",
    "new": "fnv1a64:4cf2bbb55f56e3f5"
  },
  {
    "line": 12,
    "id": "B11",
    "status": "Discharged",
    "old": "fnv1a64:787da6b09de54e9c",
    "new": "fnv1a64:4b6c5d8ff5fb418a"
  },
  {
    "line": 13,
    "id": "B12",
    "status": "Discharged",
    "old": "fnv1a64:52ece9250a5ba6b0",
    "new": "fnv1a64:750b45d67cdecdbe"
  },
  {
    "line": 14,
    "id": "I01",
    "status": "Discharged",
    "old": "fnv1a64:dd2efef94a01a8a4",
    "new": "fnv1a64:75cbe4c93e1a00a2"
  },
  {
    "line": 15,
    "id": "I02",
    "status": "Discharged",
    "old": "fnv1a64:603e8c9abafccd41",
    "new": "fnv1a64:062607751aeed687"
  },
  {
    "line": 16,
    "id": "I03",
    "status": "Pending",
    "old": "fnv1a64:07ba01b6e32fbcf0",
    "new": "fnv1a64:59143a5476c7d47e"
  },
  {
    "line": 17,
    "id": "I04",
    "status": "Discharged",
    "old": "fnv1a64:1788333629fa2765",
    "new": "fnv1a64:da5a41827274130b"
  },
  {
    "line": 18,
    "id": "P01",
    "status": "Discharged",
    "old": "fnv1a64:98ef09c638fc8f98",
    "new": "fnv1a64:3369404b9f3b7716"
  },
  {
    "line": 19,
    "id": "P02",
    "status": "Discharged",
    "old": "fnv1a64:dbede63b3d7fef0b",
    "new": "fnv1a64:5fe372b4f609f6e5"
  },
  {
    "line": 20,
    "id": "P03",
    "status": "Discharged",
    "old": "fnv1a64:eefae8708aed9ad2",
    "new": "fnv1a64:b3ded0155ee8427c"
  },
  {
    "line": 21,
    "id": "P04",
    "status": "Discharged",
    "old": "fnv1a64:fe861651e8a31ec5",
    "new": "fnv1a64:e738a2f37ed1146b"
  },
  {
    "line": 22,
    "id": "P05",
    "status": "Discharged",
    "old": "fnv1a64:202089359b09fef4",
    "new": "fnv1a64:ccebb694ec228852"
  },
  {
    "line": 23,
    "id": "P06",
    "status": "Discharged",
    "old": "fnv1a64:a36c0df5968b4030",
    "new": "fnv1a64:c58a6aa7090e673e"
  },
  {
    "line": 24,
    "id": "P070",
    "status": "Discharged",
    "old": "fnv1a64:ddb9aa377500ba30",
    "new": "fnv1a64:ffd806e8e783e13e"
  },
  {
    "line": 25,
    "id": "P080",
    "status": "Discharged",
    "old": "fnv1a64:70e67da9b254a8fb",
    "new": "fnv1a64:8cce6312611ae3b5"
  },
  {
    "line": 26,
    "id": "P090",
    "status": "Discharged",
    "old": "fnv1a64:0b680c9e9280e805",
    "new": "fnv1a64:7b6eb69c6907482b"
  },
  {
    "line": 27,
    "id": "P100",
    "status": "Discharged",
    "old": "fnv1a64:7c8d6b299c8ac3cc",
    "new": "fnv1a64:ef62faf4b9013a5a"
  },
  {
    "line": 28,
    "id": "P110",
    "status": "Discharged",
    "old": "fnv1a64:252aab34b9e3d083",
    "new": "fnv1a64:d30470bcd0ce4bad"
  },
  {
    "line": 29,
    "id": "P120",
    "status": "Discharged",
    "old": "fnv1a64:d5d637edfec08f76",
    "new": "fnv1a64:9a08e0e47055b060"
  },
  {
    "line": 30,
    "id": "P130",
    "status": "Discharged",
    "old": "fnv1a64:f0a0868dddfba071",
    "new": "fnv1a64:dd999aaabaad7957"
  },
  {
    "line": 31,
    "id": "P140",
    "status": "Discharged",
    "old": "fnv1a64:165dfb8abcb1604d",
    "new": "fnv1a64:dee98ac6e2eb37a3"
  },
  {
    "line": 32,
    "id": "P150",
    "status": "Discharged",
    "old": "fnv1a64:5d232b4a0b5fb8a5",
    "new": "fnv1a64:7d11720bcc651fcb"
  },
  {
    "line": 33,
    "id": "P071",
    "status": "Discharged",
    "old": "fnv1a64:864cf41609b7fc11",
    "new": "fnv1a64:7a4df1ba22772877"
  },
  {
    "line": 34,
    "id": "P081",
    "status": "Discharged",
    "old": "fnv1a64:3b4afa7058e6fad0",
    "new": "fnv1a64:a287d2689d2d54de"
  },
  {
    "line": 35,
    "id": "P091",
    "status": "Discharged",
    "old": "fnv1a64:d3032b9d1cc0abd8",
    "new": "fnv1a64:b764102a340ab8d6"
  },
  {
    "line": 36,
    "id": "P101",
    "status": "Discharged",
    "old": "fnv1a64:1d2d9c01ef476d83",
    "new": "fnv1a64:cb07618a0631e8ad"
  },
  {
    "line": 37,
    "id": "P111",
    "status": "Discharged",
    "old": "fnv1a64:a424933be000de20",
    "new": "fnv1a64:0b1968b1cb139f8e"
  },
  {
    "line": 38,
    "id": "P121",
    "status": "Discharged",
    "old": "fnv1a64:8c394a54a49da6af",
    "new": "fnv1a64:b04a386cc58a6eb9"
  },
  {
    "line": 39,
    "id": "P131",
    "status": "Discharged",
    "old": "fnv1a64:54d017b9ba454a1c",
    "new": "fnv1a64:27bece99125b3d0a"
  },
  {
    "line": 40,
    "id": "P141",
    "status": "Discharged",
    "old": "fnv1a64:d4b428721f2adfb8",
    "new": "fnv1a64:871d1f48d02641b6"
  },
  {
    "line": 41,
    "id": "P151",
    "status": "Discharged",
    "old": "fnv1a64:8d0d86c0c6602036",
    "new": "fnv1a64:d084e309d7d4c1a0"
  },
  {
    "line": 42,
    "id": "B13",
    "status": "Discharged",
    "old": "fnv1a64:65564547e8ec2a78",
    "new": "fnv1a64:9514335913561bf6"
  },
  {
    "line": 43,
    "id": "R01",
    "status": "Discharged",
    "old": "fnv1a64:fa8793f8ee188cb5",
    "new": "fnv1a64:d3de88dc4286233b"
  },
  {
    "line": 44,
    "id": "R02",
    "status": "Discharged",
    "old": "fnv1a64:afbe9a2fef4b9412",
    "new": "fnv1a64:76d15a1c313c463c"
  },
  {
    "line": 45,
    "id": "R03",
    "status": "Discharged",
    "old": "fnv1a64:4cae9ea4180d5da9",
    "new": "fnv1a64:e951b548e2508a5f"
  },
  {
    "line": 46,
    "id": "R040",
    "status": "Discharged",
    "old": "fnv1a64:a9402dc3dae829d5",
    "new": "fnv1a64:4bf6431487ac33db"
  },
  {
    "line": 47,
    "id": "R050",
    "status": "Discharged",
    "old": "fnv1a64:ae3aac9c23ca60d4",
    "new": "fnv1a64:d898e4a1ffd0c6b2"
  },
  {
    "line": 48,
    "id": "R060",
    "status": "Discharged",
    "old": "fnv1a64:7437107f7dce2dfd",
    "new": "fnv1a64:23ce20eee59f4e33"
  },
  {
    "line": 49,
    "id": "R070",
    "status": "Discharged",
    "old": "fnv1a64:a3b8c28a3a88b9d0",
    "new": "fnv1a64:0af59a827ecf13de"
  },
  {
    "line": 50,
    "id": "R080",
    "status": "Discharged",
    "old": "fnv1a64:de8c9c28f97ef065",
    "new": "fnv1a64:a15eaa7541f8dc0b"
  },
  {
    "line": 51,
    "id": "R090",
    "status": "Discharged",
    "old": "fnv1a64:368d048b002c5e0c",
    "new": "fnv1a64:91ccad07d88f1c1a"
  },
  {
    "line": 52,
    "id": "R100",
    "status": "Discharged",
    "old": "fnv1a64:9b4ff0ec3617250a",
    "new": "fnv1a64:76657080104084c4"
  },
  {
    "line": 53,
    "id": "R110",
    "status": "Discharged",
    "old": "fnv1a64:7110253e2f73cbf3",
    "new": "fnv1a64:93d3559dc91e49fd"
  },
  {
    "line": 54,
    "id": "R041",
    "status": "Discharged",
    "old": "fnv1a64:4b8fa8720013e978",
    "new": "fnv1a64:7b4d96832a7ddaf6"
  },
  {
    "line": 55,
    "id": "R051",
    "status": "Discharged",
    "old": "fnv1a64:998a3ab41faef485",
    "new": "fnv1a64:0990e4b1f63554ab"
  },
  {
    "line": 56,
    "id": "R061",
    "status": "Discharged",
    "old": "fnv1a64:2615df92ac4e4ca6",
    "new": "fnv1a64:9e0078bd7348a0b0"
  },
  {
    "line": 57,
    "id": "R071",
    "status": "Discharged",
    "old": "fnv1a64:9e6cdaff06079667",
    "new": "fnv1a64:be51a4c2d6515321"
  },
  {
    "line": 58,
    "id": "R081",
    "status": "Discharged",
    "old": "fnv1a64:3877d47e383698cc",
    "new": "fnv1a64:ab4d644954ad0f5a"
  },
  {
    "line": 59,
    "id": "R091",
    "status": "Discharged",
    "old": "fnv1a64:4a9d232db188cd0d",
    "new": "fnv1a64:a3869e5b62575fe3"
  },
  {
    "line": 60,
    "id": "R101",
    "status": "Discharged",
    "old": "fnv1a64:0000358aab68e43d",
    "new": "fnv1a64:9031b300d23998f3"
  },
  {
    "line": 61,
    "id": "R111",
    "status": "Discharged",
    "old": "fnv1a64:d5c65b000a147f0e",
    "new": "fnv1a64:6ed14120080d9ea8"
  },
  {
    "line": 62,
    "id": "R12",
    "status": "Discharged",
    "old": "fnv1a64:4a56dd6daece3cce",
    "new": "fnv1a64:b88f4711a19256e8"
  },
  {
    "line": 63,
    "id": "R13",
    "status": "Discharged",
    "old": "fnv1a64:d8e1b601268ea05a",
    "new": "fnv1a64:27c281b4fae9b534"
  },
  {
    "line": 64,
    "id": "R14",
    "status": "Discharged",
    "old": "fnv1a64:96e5799f793b83cc",
    "new": "fnv1a64:09bb096a95b1fa5a"
  },
  {
    "line": 65,
    "id": "R15",
    "status": "Discharged",
    "old": "fnv1a64:1f294612e9668208",
    "new": "fnv1a64:290db54f65ab5ea6"
  },
  {
    "line": 66,
    "id": "O010",
    "status": "Discharged",
    "old": "fnv1a64:76dd19d5372d8386",
    "new": "fnv1a64:310c78b63a6bd890"
  },
  {
    "line": 67,
    "id": "O0200",
    "status": "Discharged",
    "old": "fnv1a64:11259d5f0b68864d",
    "new": "fnv1a64:d9b12c9b31a25da3"
  },
  {
    "line": 68,
    "id": "O011",
    "status": "Discharged",
    "old": "fnv1a64:b6bfcd078b5ff27d",
    "new": "fnv1a64:6656dd76f33112b3"
  },
  {
    "line": 69,
    "id": "O0210",
    "status": "Discharged",
    "old": "fnv1a64:cfa1122a4bcbe3ae",
    "new": "fnv1a64:bb226e5bbda7a3c8"
  }
]
```

All four actual certificate-cell changes; only fingerprint text differs, with all preconditions, strengths, exclusions, owners, instances and statuses unchanged:

```json
[
  {
    "row": "C01",
    "line": 2,
    "feature_hash": "true",
    "column": 5,
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:4c3bb00fc206576b\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:aa197d00ffff920b;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:cfa1122a4bcbe3ae;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:b6bfcd078b5ff27d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:4c3bb00fc206576b\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:814ec67beb7232dd;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:11259d5f0b68864d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:76dd19d5372d8386;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:111f333865605c91\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:ff9e10ecb6061f55;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:bb226e5bbda7a3c8;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:6656dd76f33112b3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:111f333865605c91\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:a7d91fa29343c40f;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:d9b12c9b31a25da3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:310c78b63a6bd890;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "non_fingerprint_text_identical": true
  },
  {
    "row": "C01",
    "line": 2,
    "feature_hash": "false",
    "column": 6,
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:0a735865da4dfa51\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:a3a0d68586e8a973;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:cfa1122a4bcbe3ae;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:b6bfcd078b5ff27d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:0a735865da4dfa51\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:0d5c4f04b7ffc561;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:11259d5f0b68864d;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:76dd19d5372d8386;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:560fe11c6ec0b786\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:253ca0c58e4549a4;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>\": \"O0210@fnv1a64:bb226e5bbda7a3c8;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow>\": \"O011@fnv1a64:6656dd76f33112b3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"} || instance=Instance { body: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol\", key: \"rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\", mode: \"rum_runtime::runtime::accounting_mode::counter::Unarmed\", reentry: \"rum_stdlib::option_consumers::OptionConsumer\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:560fe11c6ec0b786\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:ba2daf537b1bc2e2;direct-covered={};direct-contracts={\"rum_runtime::runtime::accounting_mode::options::driver::drive::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>\": \"O0200@fnv1a64:d9b12c9b31a25da3;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B02 and I03 typed replacements: checked snapshot/prefix, callback adoption, retained borrowed consumer handoff, frozen complement and copied working occurrence;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::options::snapshot::<rum_runtime::runtime::accounting_mode::counter::Unarmed>\": \"O010@fnv1a64:310c78b63a6bd890;precondition=entry/resume TypeId membership and generation; private OptionState constructor; concrete fieldless Copy and FrameRootTrace descriptor;guarantee=B01 typed replacement: private checked default snapshot; one copied occurrence, actual capacity and Arc control charged;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "non_fingerprint_text_identical": true
  },
  {
    "row": "C02",
    "line": 3,
    "feature_hash": "true",
    "column": 5,
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:4f5f2173b3727b69\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:f14372f76ba45159;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:dd2efef94a01a8a4;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:603e8c9abafccd41;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:1788333629fa2765;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:fa8793f8ee188cb5;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:afbe9a2fef4b9412;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:65564547e8ec2a78;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:08fd0f06c2c64541;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=true;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:4ca7ae41382d017f\", domain_fingerprint: \"fnv1a64:f1b1b5ec1f9ab7c9\" };mark=fnv1a64:77962e00a4a30170;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:75cbe4c93e1a00a2;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:062607751aeed687;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:da5a41827274130b;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:d3de88dc4286233b;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:76d15a1c313c463c;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:9514335913561bf6;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:aee489e122b84e87;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "non_fingerprint_text_identical": true
  },
  {
    "row": "C02",
    "line": 3,
    "feature_hash": "false",
    "column": 6,
    "before": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:892ccb7ae02ee93f\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:a7f011ceccc88c23;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:dd2efef94a01a8a4;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:603e8c9abafccd41;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:1788333629fa2765;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:fa8793f8ee188cb5;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:afbe9a2fef4b9412;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:65564547e8ec2a78;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:08fd0f06c2c64541;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "after": "instance=Instance { body: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", key: \"rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr\", mode: \"runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow\", reentry: \"none (nongeneric C02)\", build: \"target=x86_64-linux;pointer=64;debug=true;hash=false;compiler=rustc 1.85.0 (4d91de4e4 2025-02-17);binary: rustc;commit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c46688;commit-date: 2025-02-17;host: x86_64-unknown-linux-gnu;release: 1.85.0;LLVM version: 19.1.7;build-inputs=fnv1a64:0bb558ca76e00284;rustflags=None\", body_fingerprint: \"fnv1a64:f371f7a9c1ee0fd8\", domain_fingerprint: \"fnv1a64:ed48860e2b2c615f\" };mark=fnv1a64:8bdecfea2e7ba68f;direct-covered={};direct-contracts={\"<rum_core::expr::SharedArgs as std::ops::Deref>::deref\": \"I01@fnv1a64:75cbe4c93e1a00a2;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=SharedArgs deref borrows the same Vec with pointer identity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<rum_core::expr::SharedExpr as std::ops::Deref>::deref\": \"I02@fnv1a64:062607751aeed687;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=borrows same Expr with pointer identity; no owner or capacity receipt;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"<std::vec::Vec<rum_core::expr::Expr> as std::ops::Index<usize>>::index\": \"I04@fnv1a64:da5a41827274130b;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec<Expr> indexing by usize borrows one element; bounds panic unwinds;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::clone_result\": \"R01@fnv1a64:d3de88dc4286233b;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::accounting_mode::rule::evaluate_rule\": \"R02@fnv1a64:76d15a1c313c463c;precondition=exact private borrowed Session/name inputs or terminal resume outcome; finite internal Unarmed/AccountingShadow modes; complete direct rule records;guarantee=clone/adopt or activation bridge; callback result charged before growth; lazy real InputRequired only; outgoing Expr/Err is caller-owned unleased W;strength=Runtime-enforced;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"rum_runtime::runtime::expr_runtime_helpers::is_system_symbol_name\": \"B13@fnv1a64:9514335913561bf6;precondition=shared immutable input; exact declaration and compiler selection checked;guarantee=None=false; System matching name=true; different name or context=false; scalar result;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\", \"std::vec::Vec<rum_core::expr::Expr>::len\": \"B05@fnv1a64:aee489e122b84e87;precondition=checked exact receiver and arguments; no inferred receipt;guarantee=Vec length reads initialized length even with spare capacity;strength=Test-checked;exclusions=allocator-rss;stack-bounds;guest-host-private;no-implicit-receipt\"}",
    "non_fingerprint_text_identical": true
  }
]
```

The existing replay harness intrinsically emitted `target/brief-544/local-contracts.tsv` and `.temp/brief-546/certificates/`; those prescribed outputs were copied into this brief’s evidence before installation. All new scratch scripts, measurements, snapshots, and logs are under `.temp/brief-564/`.

### 3. All 54 allocation/instruction rows against both references

Exactly one candidate was measured: **108 result checks, 216 allocation processes, 432 instruction processes = 648 metric processes**; all succeeded, with no metric retry/timeout. Allocation collection preceded both instruction samples. Each isolated process used stdin /dev/null; instructions used taskset -c 2 perf stat -e cpu_core/instructions/u, at N and 2N. Slopes are (count(2N)-count(N))/N. The 30 original labels use 20,000/40,000; 24 depth labels use 1,000/2,000. The 162 .rum workload sources are byte-identical to 563; all 18 inherited files also match 559 exactly. No timing suite ran, as this brief directs. Raw stdout/stderr/perf CSVs were saved before parsing.

**Release artifact correction, before any measurement:** the initial default Cargo builds returned success/fresh but left target/release/rum-cli at its September 30 hash. Those logs and stale binaries were preserved; none was measured. The accepted 563 candidate-3 binaries were already preserved before this brief. Comparing their archived source with this brief’s immutable incoming source found only three test files and two certificate inventories different, with release production source identical. Those accepted binaries are the true incoming reference under incoming-accepted/. The candidate default was recompiled in an explicit local target directory; the new frame symbol is present and its hash changed. No old source was reconstructed. The initial task-local default preservation copied a stale artifact; pre-edit provenance for the correct default reference relies on the pre-existing immutable 563 archive. This procedure departure is disclosed rather than hidden.

Binary hashes, `incoming`:

```json
{
  "rum-cli": "3fd7e2c57d740406de983e0716a9f47bd93dd9ac4f8d568da2c795e938341c73",
  "rum-cli-alloc": "197c1604a0638f6db49fb893fd0a1f84e45a1ba96f47e3f70cf7bccb99400034"
}
```

Binary hashes, `incoming-accepted`:

```json
{
  "rum-cli": "431d2c42c6b73e21d879b4877eb7d40cedec419ab82ee6c1cf94b8060fae8302",
  "rum-cli-alloc": "197c1604a0638f6db49fb893fd0a1f84e45a1ba96f47e3f70cf7bccb99400034"
}
```

Binary hashes, `candidate`:

```json
{
  "rum-cli": "75483887d622bf73959de3414e247d4f2669c9ca8f725feefedecc7a610b8ca9",
  "rum-cli-alloc": "de938e06c0ad0a6843ca9d03df5be8c733d614db5a0f0ea6cddbfcaaaa604b43"
}
```

Binary hashes, `pre-g2`:

```json
{
  "rum-cli": "c08b67e8f53c9651c8d139e8c39bdb392320c82376b0c4eec7f03c26397c80cc",
  "rum-cli-alloc": "4285dd8a3ccded49ae769b166b04282057cc2a6a0ceb67c1c9d81202b561ac03"
}
```

Source equivalence evidence:

```json
{
  "changed": [
    "crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs",
    "crates/rum-runtime/src/runtime/tests/frame_root_trace.rs",
    "crates/rum-stdlib/src/tests/association_depth_contract/lazy_depth_contract.rs",
    "crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv",
    "crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv"
  ],
  "added": []
}
```

Incoming delta: **0 allocation/byte/peak regressions; 5 labels above +0.05% instructions in both samples**. Worst repeatable incremental delta: **+0.666866623%**, full LazyTable construction. Other incremental flagged labels: short construction, LazyTable realization, table-small, and depth-64 select-restart. These are reported and not tuned under the one-candidate rule. This is not a zero-cost result.

Cumulative pre-G2 delta: **0 allocation/byte/peak regressions; 26 paired instruction regressions**, worst repeatable **+2.510866502%**. The prior pre-G2 Fold-256 first instruction pair is unavailable because its original 2N run timed out. It is shown as None below; the second paired delta is reported. No extra pre-G2 baseline measurement was authorized or performed; existing allocation samples 1/2 and instruction samples 2/3 were replayed.

```json
{
  "incoming-accepted": {
    "labels": 54,
    "allocation_regressions": 0,
    "paired_instruction_regressions": 5,
    "worst_repeatable_instruction_pct": 0.6668666232894527,
    "missing_instruction_pairs": [],
    "instruction_regression_labels": [
      "lazy-table-construct-short",
      "lazy-table-construct-full",
      "lazy-table-realize-small",
      "table-small",
      "lazy-depth-select-restart-64"
    ],
    "peak_growth_labels": []
  },
  "pre-g2": {
    "labels": 54,
    "allocation_regressions": 0,
    "paired_instruction_regressions": 26,
    "worst_repeatable_instruction_pct": 2.510866502466591,
    "missing_instruction_pairs": [
      "lazy-depth-fold-through-source-256"
    ],
    "instruction_regression_labels": [
      "lazy-range-take-small",
      "lazy-range-normal-small",
      "lazy-map-realize-small",
      "lazy-select-realize-small",
      "lazy-select-scan-small",
      "lazy-nestlist-realize-small",
      "lazy-foldlist-realize-small",
      "lazy-drop-realize-small",
      "lazy-first-small",
      "lazy-part-small",
      "lazy-transparent-adapters-small",
      "lazy-table-construct-short",
      "lazy-table-construct-full",
      "lazy-table-realize-small",
      "lazy-permutations-take-small",
      "lazy-subsets-take-small",
      "lazy-tuples-take-small",
      "take-multispec-small",
      "lazy-depth-map-build-8",
      "lazy-depth-drop-all-one-8",
      "lazy-depth-take-map-8",
      "lazy-depth-fold-through-source-8",
      "lazy-depth-select-restart-64",
      "lazy-depth-drop-all-one-64",
      "lazy-depth-take-map-64",
      "lazy-depth-fold-through-source-64"
    ],
    "peak_growth_labels": []
  }
}
```

All 54 rows against `incoming-accepted` (counts/bytes/instructions are per iteration; peak columns are whole-process peaks at N/2N):

```tsv
label	N	base_count	candidate_count	delta_count	base_bytes_total	candidate_bytes_total	delta_bytes_total	base_peak_1N	candidate_peak_1N	delta_peak_1N	base_peak_2N	candidate_peak_2N	delta_peak_2N	base_instructions_1	candidate_instructions_1	delta_instructions_pct_1	base_instructions_2	candidate_instructions_2	delta_instructions_pct_2	allocation_regression	instruction_regression
lazy-range-take-small	20000	141.0	141.0	0.0	2714.0	2714.0	0.0	11428045	11428045	0	12068045	12068045	0	42750.89	42741.01615	-0.023096244312093006	42752.3252	42753.99435	0.003904232090756743	False	False
lazy-range-normal-small	20000	157.0	157.0	0.0	3388.0	3388.0	0.0	11428491	11428491	0	12068491	12068491	0	54843.9474	54897.8294	0.09824602814787475	54941.01105	54793.137	-0.2691505801839411	False	False
lazy-map-realize-small	20000	333.0	333.0	0.0	13740.0	13740.0	0.0	11429646	11429646	0	12069646	12069646	0	179697.2637	179604.54935	-0.05159474779471429	179646.53795	179654.5371	0.004452715922753647	False	False
lazy-select-realize-small	20000	236.0	236.0	0.0	6834.0	6834.0	0.0	9594202	9594202	0	10234202	10234202	0	115390.82475	115382.09915	-0.007561779733278318	115388.61645	115663.44325	0.2381749677353051	False	False
lazy-select-scan-small	20000	1471.0	1471.0	0.0	62793.0	62793.0	0.0	9595818	9595818	0	10235818	10235818	0	800731.0614	800771.2985	0.0050250454790345955	800779.54695	800672.70225	-0.013342586034692516	False	False
lazy-nestlist-realize-small	20000	197.0	197.0	0.0	11316.0	11316.0	0.0	11429099	11429099	0	12069099	12069099	0	146457.46525	146404.09095	-0.03644355028873392	146502.0365	146356.18505	-0.09955591982503398	False	False
lazy-foldlist-realize-small	20000	216.0	216.0	0.0	7604.0	7604.0	0.0	11429094	11429094	0	12069094	12069094	0	114624.3687	114594.27725	-0.026252227463741118	114620.2629	114775.9732	0.13584884213346804	False	False
lazy-drop-realize-small	20000	351.0	351.0	0.0	14204.0	14204.0	0.0	9595465	9595465	0	10235465	10235465	0	192383.92145	192415.70325	0.01651998761666995	192480.955	192594.21135	0.05884028890028592	False	False
lazy-first-small	20000	64.0	64.0	0.0	2868.0	2868.0	0.0	9595621	9595621	0	10235621	10235621	0	41632.8534	41732.15985	0.23852905071357622	41713.2995	41635.1337	-0.1873881973781577	False	False
lazy-part-small	20000	152.0	152.0	0.0	6182.0	6182.0	0.0	9596439	9596439	0	10236439	10236439	0	88967.4294	88971.60775	0.004696494018285868	88968.3296	89089.3995	0.13608201991015534	False	False
lazy-transparent-adapters-small	20000	319.0	319.0	0.0	12876.0	12876.0	0.0	949239	949239	0	1589239	1589239	0	183091.92275	182977.0545	-0.0627380215766471	182983.346	183002.23785	0.010324354873270103	False	False
lazy-table-construct-short	20000	63.0	63.0	0.0	1537.0	1537.0	0.0	942784	942784	0	1582784	1582784	0	23583.4638	23725.5327	0.602408964199741	23582.79945	23742.3673	0.6766281091365567	False	True
lazy-table-construct-full	20000	64.0	64.0	0.0	1449.0	1449.0	0.0	942664	942664	0	1582664	1582664	0	24086.0998	24264.1473	0.7392126640611174	24055.71735	24216.1369	0.6668666232894527	False	True
lazy-table-realize-small	20000	251.0	251.0	0.0	9190.0	9190.0	0.0	10053648	10053648	0	10693648	10693648	0	117743.3447	117822.7992	0.06748109644960643	117791.89845	117855.00135	0.053571511139871575	False	True
lazy-permutations-take-small	20000	29.0	29.0	0.0	1729.0	1729.0	0.0	9593774	9593774	0	10233774	10233774	0	28670.1053	28639.88705	-0.10539985704202115	28638.28185	28671.83315	0.11715542215742225	False	False
lazy-subsets-take-small	20000	29.0	29.0	0.0	1417.0	1417.0	0.0	8446869	8446869	0	9086869	9086869	0	25882.4586	25885.81845	0.012981185643612392	25827.38385	25885.49285	0.22498988026617983	False	False
lazy-tuples-take-small	20000	29.0	29.0	0.0	1361.0	1361.0	0.0	8905896	8905896	0	9545896	9545896	0	27066.9292	27064.80095	-0.007862916344414383	27059.83025	27065.10135	0.019479427443935826	False	False
scan-no-input	20000	33.0	33.0	0.0	2536.0	2536.0	0.0	942414	942414	0	1582414	1582414	0	29993.8476	30001.32785	0.024939281214453324	29994.133	30001.0133	0.022938819401763233	False	False
optionvalue-three-no-input	20000	9.0	9.0	0.0	1128.0	1128.0	0.0	9134068	9134068	0	9774068	9774068	0	9738.6665	9737.649	-0.010448042347477315	9738.1646	9737.84975	-0.0032331554551956465	False	False
replacerepeated-no-input	20000	61.0	61.0	0.0	7691.0	7691.0	0.0	948049	948049	0	1588049	1588049	0	40117.96275	40139.49535	0.053673214001870306	40117.78905	40106.62375	-0.027831294456648337	False	False
dot-three-no-input	20000	596.0	596.0	0.0	23504.0	23504.0	0.0	8678236	8678236	0	9318236	9318236	0	256343.88955	256428.685	0.03307878730749447	256344.47775	256428.23725	0.032674587233239016	False	False
differences-no-input	20000	13.0	13.0	0.0	704.0	704.0	0.0	9591399	9591399	0	10231399	10231399	0	9985.45205	9985.89755	0.004461490554152903	9985.5482	9987.3045	0.017588418430558583	False	False
do-count-null	20000	0.0	0.0	0.0	0.0	0.0	0.0	299668	299668	0	299668	299668	0	497.0971	497.0822	-0.002997402318383191	497.1081	497.0417	-0.013357255695489734	False	False
do-iterator-null	20000	0.0	0.0	0.0	32.0	32.0	0.0	940146	940146	0	1580146	1580146	0	1136.86535	1136.912	0.00410338832121937	1136.87905	1137.04335	0.014451845163288013	False	False
table-iterator-null	20000	0.0	0.0	0.0	32.0	32.0	0.0	940025	940025	0	1580025	1580025	0	348.94645	349.0406	0.02698121731856684	349.1334	348.96965	-0.04690184324960267	False	False
replaceall-simple	20000	5.0	5.0	0.0	1352.0	1352.0	0.0	943388	943388	0	1583388	1583388	0	11403.2974	11403.1985	-0.0008672929989406164	11403.36075	11403.15925	-0.0017670229366273382	False	False
table-small	20000	22.0	22.0	0.0	977.0	977.0	0.0	9132887	9132887	0	9772887	9772887	0	14741.2322	14757.3288	0.10919439963776867	14741.34185	14817.3684	0.5157369713938209	False	True
table-iterator-plus	20000	2.0	2.0	0.0	72.0	72.0	0.0	940209	940209	0	1580209	1580209	0	565.34045	565.20635	-0.023720220267275938	565.29155	565.75045	0.08117934895719436	False	False
table-iterator-pair-list	20000	2.0	2.0	0.0	136.0	136.0	0.0	8574075	8574075	0	11294075	11294075	0	1726.747	1726.8663	0.006908944969929998	1726.6289	1726.81	0.010488646402251334	False	False
take-multispec-small	20000	109.0	109.0	0.0	5856.0	5856.0	0.0	9594023	9594023	0	10234023	10234023	0	51115.23485	51091.7672	-0.04591126318574945	51115.803	51091.26685	-0.048001104472528144	False	False
lazy-depth-map-build-8	1000	16.001	16.001	0.0	1879.864	1879.864	0.0	762116	762116	0	1220100	1220100	0	19551.962	19567.29	0.07839622437892935	19576.477	19541.237	-0.1800119602725192	False	False
lazy-depth-map-first-8	1000	188.0	188.0	0.0	11176.0	11176.0	0.0	338970	338970	0	370970	370970	0	131683.015	131692.565	0.007252264082802107	131677.917	132035.183	0.2713180828946449	False	False
lazy-depth-map-eight-8	1000	1464.001	1464.001	0.0	82776.864	82776.864	0.0	990724	990724	0	1678084	1678084	0	959617.696	959586.822	-0.0032173229118859226	959582.361	959618.98	0.0038161393423052914	False	False
lazy-depth-select-restart-8	1000	327.0	327.0	0.0	16980.0	16980.0	0.0	340857	340857	0	372857	372857	0	179638.432	179654.681	0.009045391801243419	179649.404	180143.042	0.27477853475093017	False	False
lazy-depth-drop-all-one-8	1000	170.0	170.0	0.0	3128.0	3128.0	0.0	337461	337461	0	369461	369461	0	49458.683	49465.799	0.014387766855827877	49465.557	49459.489	-0.01226712154479559	False	False
lazy-depth-take-map-8	1000	220.0	220.0	0.0	12144.0	12144.0	0.0	341707	341707	0	373707	373707	0	142260.682	142264.781	0.00288133020478476	142254.957	142261.711	0.004747813462846473	False	False
lazy-depth-fold-through-source-8	1000	743.001	743.001	0.0	58440.864	58440.864	0.0	1219923	1219923	0	2136659	2136659	0	602177.55	602181.041	0.0005797293505693801	602178.575	602190.73	0.0020185042285847388	False	False
lazy-depth-nest-map-8	1000	192.0	192.0	0.0	11964.0	11964.0	0.0	339282	339282	0	371282	371282	0	142307.961	142464.66	0.11011260290629821	142485.876	142468.473	-0.012213842163544264	False	False
lazy-depth-map-build-64	1000	16.001	16.001	0.0	5911.864	5911.864	0.0	770631	770631	0	1228615	1228615	0	39821.744	39784.327	-0.09396122882011637	39796.486	39804.652	0.020519399627416668	False	False
lazy-depth-map-first-64	1000	1364.0	1364.0	0.0	85320.0	85320.0	0.0	351517	351517	0	383517	383517	0	956939.046	954183.462	-0.2879581527703623	954255.338	954248.016	-0.0007672998733609937	False	False
lazy-depth-map-eight-64	1000	10872.001	10872.001	0.0	647704.864	647704.864	0.0	999239	999239	0	1686599	1686599	0	7417215.988	7417714.967	0.0067273084781049874	7417293.73	7417906.476	0.008261045366464437	False	False
lazy-depth-select-restart-64	1000	1671.0	1671.0	0.0	99412.0	99412.0	0.0	353404	353404	0	385404	385404	0	976291.918	981448.861	0.5282173195251261	976199.617	978997.18	0.2865769409536645	False	True
lazy-depth-drop-all-one-64	1000	1178.0	1178.0	0.0	17520.0	17520.0	0.0	350008	350008	0	382008	382008	0	348579.126	348591.863	0.003653976687068017	348580.907	348578.482	-0.0006956778042854772	False	False
lazy-depth-take-map-64	1000	1620.0	1620.0	0.0	93064.0	93064.0	0.0	366798	366798	0	398798	398798	0	1031900.315	1031868.74	-0.003059888590106752	1031961.574	1031995.683	0.003305258728558691	False	False
lazy-depth-fold-through-source-64	1000	43334.001	43334.001	0.0	3402649.864	3402649.864	0.0	4767601	4767601	0	9215025	9215025	0	34526487.225	34528045.58	0.0045135057900269615	34524944.183	34526133.156	0.0034438086089272346	False	False
lazy-depth-nest-map-64	1000	1368.0	1368.0	0.0	86108.0	86108.0	0.0	351829	351829	0	383829	383829	0	968277.031	966867.643	-0.14555627727163012	967102.721	966861.147	-0.024979145932935154	False	False
lazy-depth-map-build-256	1000	16.001	16.001	0.0	19735.864	19735.864	0.0	799818	799818	0	1257802	1257802	0	108703.146	108661.154	-0.03862997672579027	108681.255	108665.594	-0.014410028665945429	False	False
lazy-depth-map-first-256	1000	5396.0	5396.0	0.0	339528.0	339528.0	0.0	394528	394528	0	426528	426528	0	3754100.131	3748412.066	-0.1515160704699925	3754242.594	3754095.941	-0.003906327210556615	False	False
lazy-depth-map-eight-256	1000	43128.001	43128.001	0.0	2584600.864	2584600.864	0.0	1028426	1028426	0	1715786	1715786	0	29530508.3	29529087.438	-0.0048115053949082665	29531634.321	29531779.982	0.0004932371788779477	False	False
lazy-depth-select-restart-256	1000	6279.0	6279.0	0.0	382036.0	382036.0	0.0	396415	396415	0	428415	428415	0	3656052.049	3667315.906	0.30808798258439296	3667058.813	3667316.795	0.007035120328180433	False	False
lazy-depth-drop-all-one-256	1000	4634.0	4634.0	0.0	66864.0	66864.0	0.0	393019	393019	0	425019	425019	0	2301996.178	2302007.353	0.0004854482430038942	2302004.039	2302014.95	0.0004739783169505074	False	False
lazy-depth-take-map-256	1000	6420.0	6420.0	0.0	370504.0	370504.0	0.0	452817	452817	0	484817	484817	0	4082488.789	4082707.736	0.005363076576969661	4082846.754	4082697.512	-0.0036553417013251988	False	False
lazy-depth-fold-through-source-256	1000	689320.001	689320.001	0.0	54008857.864	54008857.864	0.0	17104759	17104759	0	33840183	33840183	0	538606853.468	538580683.739	-0.004858781285754787	538619162.525	538526723.06	-0.017162305285733925	False	False
lazy-depth-nest-map-256	1000	5400.0	5400.0	0.0	340316.0	340316.0	0.0	394840	394840	0	426840	426840	0	3765046.388	3765021.111	-0.0006713595901541858	3776306.469	3765038.137	-0.29839559083730816	False	False
```

All 54 rows against `pre-g2` (counts/bytes/instructions are per iteration; peak columns are whole-process peaks at N/2N):

```tsv
label	N	base_count	candidate_count	delta_count	base_bytes_total	candidate_bytes_total	delta_bytes_total	base_peak_1N	candidate_peak_1N	delta_peak_1N	base_peak_2N	candidate_peak_2N	delta_peak_2N	base_instructions_1	candidate_instructions_1	delta_instructions_pct_1	base_instructions_2	candidate_instructions_2	delta_instructions_pct_2	allocation_regression	instruction_regression
lazy-range-take-small	20000	141.0	141.0	0.0	2714.0	2714.0	0.0	11428045	11428045	0	12068045	12068045	0	41864.1059	42741.01615	2.0946589713265684	41802.7956	42753.99435	2.2754429132007647	False	True
lazy-range-normal-small	20000	157.0	157.0	0.0	3444.0	3388.0	-56.0	11428491	11428491	0	12068491	12068491	0	53655.83865	54897.8294	2.3147355092175026	53724.3123	54793.137	1.9894618548705	False	True
lazy-map-realize-small	20000	333.0	333.0	0.0	13780.0	13740.0	-40.0	11429646	11429646	0	12069646	12069646	0	178317.2125	179604.54935	0.7219363918668176	178328.74175	179654.5371	0.7434557867618707	False	True
lazy-select-realize-small	20000	236.0	236.0	0.0	6874.0	6834.0	-40.0	9594202	9594202	0	10234202	10234202	0	114231.77065	115382.09915	1.007012754380332	114258.16825	115663.44325	1.2299120680153264	False	True
lazy-select-scan-small	20000	1471.0	1471.0	0.0	62849.0	62793.0	-56.0	9595818	9595818	0	10235818	10235818	0	799020.4463	800771.2985	0.2191248306733229	799008.6503	800672.70225	0.2082645725268728	False	True
lazy-nestlist-realize-small	20000	197.0	197.0	0.0	11372.0	11316.0	-56.0	11429099	11429099	0	12069099	12069099	0	145161.1358	146404.09095	0.8562589037003354	145148.7688	146356.18505	0.8318473935274628	False	True
lazy-foldlist-realize-small	20000	216.0	216.0	0.0	7644.0	7604.0	-40.0	11429094	11429094	0	12069094	12069094	0	113099.0303	114594.27725	1.3220687622464977	113087.51155	114775.9732	1.4930575683005154	False	True
lazy-drop-realize-small	20000	352.0	351.0	-1.0	14300.0	14204.0	-96.0	9595465	9595465	0	10235465	10235465	0	191299.7439	192415.70325	0.5833564265425029	191378.41095	192594.21135	0.6352860774445634	False	True
lazy-first-small	20000	64.0	64.0	0.0	2924.0	2868.0	-56.0	9595677	9595621	-56	10235677	10235621	-56	41422.51415	41732.15985	0.7475299516555234	41374.18445	41635.1337	0.6307054833077252	False	True
lazy-part-small	20000	152.0	152.0	0.0	6222.0	6182.0	-40.0	9596551	9596439	-112	10236551	10236439	-112	88375.37145	88971.60775	0.6746634160822884	88375.4313	89089.3995	0.8078808663194659	False	True
lazy-transparent-adapters-small	20000	322.0	319.0	-3.0	13084.0	12876.0	-208.0	949519	949239	-280	1589519	1589239	-280	182400.50915	182977.0545	0.3160875771053284	182151.29215	183002.23785	0.46716424020711855	False	True
lazy-table-construct-short	20000	63.0	63.0	0.0	1537.0	1537.0	0.0	942784	942784	0	1582784	1582784	0	23582.30335	23725.5327	0.6073594588036801	23582.9431	23742.3673	0.6760148609271877	False	True
lazy-table-construct-full	20000	64.0	64.0	0.0	1449.0	1449.0	0.0	942664	942664	0	1582664	1582664	0	24056.0753	24264.1473	0.8649457461583632	24012.74625	24216.1369	0.8470111993125418	False	True
lazy-table-realize-small	20000	251.0	251.0	0.0	9230.0	9190.0	-40.0	10053760	10053648	-112	10693760	10693648	-112	116678.26715	117822.7992	0.980929934902619	116627.2691	117855.00135	1.052697417571613	False	True
lazy-permutations-take-small	20000	29.0	29.0	0.0	1729.0	1729.0	0.0	9593774	9593774	0	10233774	10233774	0	28226.02715	28639.88705	1.4662350383234823	28227.9792	28671.83315	1.5723900986861938	False	True
lazy-subsets-take-small	20000	29.0	29.0	0.0	1417.0	1417.0	0.0	8446869	8446869	0	9086869	9086869	0	25473.90965	25885.81845	1.6169830452389844	25537.91965	25885.49285	1.3610082761772668	False	True
lazy-tuples-take-small	20000	29.0	29.0	0.0	1361.0	1361.0	0.0	8905896	8905896	0	9545896	9545896	0	26638.6928	27064.80095	1.5995835576436468	26679.1129	27065.10135	1.4467814257797196	False	True
scan-no-input	20000	33.0	33.0	0.0	2536.0	2536.0	0.0	942414	942414	0	1582414	1582414	0	30010.90485	30001.32785	-0.031911733577727475	30007.9624	30001.0133	-0.023157520352001804	False	False
optionvalue-three-no-input	20000	9.0	9.0	0.0	1128.0	1128.0	0.0	9134068	9134068	0	9774068	9774068	0	9738.86165	9737.649	-0.012451660610679482	9738.567	9737.84975	-0.007365046623386551	False	False
replacerepeated-no-input	20000	61.0	61.0	0.0	7691.0	7691.0	0.0	948049	948049	0	1588049	1588049	0	40117.82965	40139.49535	0.05400516475844164	40117.78085	40106.62375	-0.027810860330790277	False	False
dot-three-no-input	20000	596.0	596.0	0.0	23504.0	23504.0	0.0	8678236	8678236	0	9318236	9318236	0	256453.65765	256428.685	-0.009737685252320105	256452.9535	256428.23725	-0.009637732637768348	False	False
differences-no-input	20000	13.0	13.0	0.0	704.0	704.0	0.0	9591399	9591399	0	10231399	10231399	0	9989.7326	9985.89755	-0.03838991646282208	9990.58515	9987.3045	-0.03283741593455014	False	False
do-count-null	20000	0.0	0.0	0.0	0.0	0.0	0.0	299668	299668	0	299668	299668	0	496.95	497.0822	0.02660227387061287	496.8492	497.0417	0.03874415013649379	False	False
do-iterator-null	20000	0.0	0.0	0.0	32.0	32.0	0.0	940146	940146	0	1580146	1580146	0	1137.1392	1136.912	-0.01997996375465716	1136.89205	1137.04335	0.013308211628348943	False	False
table-iterator-null	20000	0.0	0.0	0.0	32.0	32.0	0.0	940025	940025	0	1580025	1580025	0	348.8655	349.0406	0.05019126282190811	349.0439	348.96965	-0.02127239582184748	False	False
replaceall-simple	20000	5.0	5.0	0.0	1352.0	1352.0	0.0	943388	943388	0	1583388	1583388	0	11403.388	11403.1985	-0.0016617868303758776	11447.41965	11403.15925	-0.3866408444281988	False	False
table-small	20000	22.0	22.0	0.0	977.0	977.0	0.0	9132887	9132887	0	9772887	9772887	0	14756.31495	14757.3288	0.006870617789300759	14756.4478	14817.3684	0.412840548251725	False	False
table-iterator-plus	20000	2.0	2.0	0.0	72.0	72.0	0.0	940209	940209	0	1580209	1580209	0	565.33085	565.20635	-0.022022502398377775	565.0963	565.75045	0.11575903080589534	False	False
table-iterator-pair-list	20000	2.0	2.0	0.0	136.0	136.0	0.0	8574075	8574075	0	11294075	11294075	0	1726.62565	1726.8663	0.013937589772283765	1726.76935	1726.81	0.00235410710758277	False	False
take-multispec-small	20000	109.0	109.0	0.0	5856.0	5856.0	0.0	9594023	9594023	0	10234023	10234023	0	50781.6916	51091.7672	0.6106051024105774	50788.7304	51091.26685	0.5956763392533926	False	True
lazy-depth-map-build-8	1000	22.001	16.001	-6.0	2255.864	1879.864	-376.0	762116	762116	0	1220100	1220100	0	19450.58	19567.29	0.6000335208512952	19468.087	19541.237	0.37574313285122507	False	True
lazy-depth-map-first-8	1000	194.0	188.0	-6.0	11552.0	11176.0	-376.0	339418	338970	-448	371418	370970	-448	131667.504	131692.565	0.01903354984233996	131482.213	132035.183	0.4205663925051173	False	False
lazy-depth-map-eight-8	1000	1470.001	1464.001	-6.0	83152.864	82776.864	-376.0	990724	990724	0	1678084	1678084	0	961539.721	959586.822	-0.203101229969882	961693.762	959618.98	-0.21574248289654152	False	False
lazy-depth-select-restart-8	1000	333.0	327.0	-6.0	17356.0	16980.0	-376.0	341305	340857	-448	373305	372857	-448	179761.101	179654.681	-0.05920079450335569	180210.556	180143.042	-0.037463954109340225	False	False
lazy-depth-drop-all-one-8	1000	176.0	170.0	-6.0	3504.0	3128.0	-376.0	337909	337461	-448	369909	369461	-448	48246.677	49465.799	2.526851745665293	48248.045	49459.489	2.510866502466591	False	True
lazy-depth-take-map-8	1000	234.0	220.0	-14.0	12968.0	12144.0	-824.0	342603	341707	-896	374603	373707	-896	141428.03	142264.781	0.5916443861941678	141434.614	142261.711	0.5847910752597052	False	True
lazy-depth-fold-through-source-8	1000	749.001	743.001	-6.0	58816.864	58440.864	-376.0	1219923	1219923	0	2136659	2136659	0	601486.522	602181.041	0.115467092710686	601496.901	602190.73	0.11535038648520235	False	True
lazy-depth-nest-map-8	1000	198.0	192.0	-6.0	12340.0	11964.0	-376.0	339730	339282	-448	371730	371282	-448	142705.741	142464.66	-0.1689357402937297	142208.357	142468.473	0.18291189455201806	False	False
lazy-depth-map-build-64	1000	78.001	16.001	-62.0	9423.864	5911.864	-3512.0	770631	770631	0	1228615	1228615	0	41618.401	39784.327	-4.406882426838077	41632.136	39804.652	-4.389599419064149	False	False
lazy-depth-map-first-64	1000	1426.0	1364.0	-62.0	88832.0	85320.0	-3512.0	355101	351517	-3584	387101	383517	-3584	956256.734	954183.462	-0.21681123136539915	956257.944	954248.016	-0.21018680290305625	False	False
lazy-depth-map-eight-64	1000	10934.001	10872.001	-62.0	651216.864	647704.864	-3512.0	999239	999239	0	1686599	1686599	0	7417732.4	7417714.967	-0.000235017914640423	7417548.236	7417906.476	0.004829628181735046	False	False
lazy-depth-select-restart-64	1000	1733.0	1671.0	-62.0	102924.0	99412.0	-3512.0	356988	353404	-3584	388988	385404	-3584	980952.971	981448.861	0.05055186279669943	978361.115	978997.18	0.0650133156610666	False	True
lazy-depth-drop-all-one-64	1000	1240.0	1178.0	-62.0	21032.0	17520.0	-3512.0	353592	350008	-3584	385592	382008	-3584	345180.254	348591.863	0.988355782367556	345163.555	348578.482	0.9893648823961154	False	True
lazy-depth-take-map-64	1000	1746.0	1620.0	-126.0	100160.0	93064.0	-7096.0	373966	366798	-7168	405966	398798	-7168	1031343.8	1031868.74	0.050898643110075525	1031229.672	1031995.683	0.07428131877880162	False	True
lazy-depth-fold-through-source-64	1000	43396.001	43334.001	-62.0	3406161.864	3402649.864	-3512.0	4771185	4767601	-3584	9218609	9215025	-3584	34505574.24	34528045.58	0.06512379664718182	34505637.202	34526133.156	0.05939885671439882	False	True
lazy-depth-nest-map-64	1000	1430.0	1368.0	-62.0	89620.0	86108.0	-3512.0	355413	351829	-3584	387413	383829	-3584	968789.243	966867.643	-0.1983506747091357	968792.609	966861.147	-0.19936795368347138	False	False
lazy-depth-map-build-256	1000	270.001	16.001	-253.99999999999997	33999.864	19735.864	-14264.0	799818	799818	0	1257802	1257802	0	117673.149	108661.154	-7.658497351847027	117656.186	108665.594	-7.641410371741952	False	False
lazy-depth-map-first-256	1000	5650.0	5396.0	-254.0	353792.0	339528.0	-14264.0	408864	394528	-14336	440864	426528	-14336	3764210.665	3748412.066	-0.4197054948836154	3775667.651	3754095.941	-0.571334979504523	False	False
lazy-depth-map-eight-256	1000	43382.001	43128.001	-254.0	2598864.864	2584600.864	-14264.0	1028426	1028426	0	1715786	1715786	0	29550094.567	29529087.438	-0.07108988755474277	29549588.953	29531779.982	-0.06026808368917447	False	False
lazy-depth-select-restart-256	1000	6533.0	6279.0	-254.0	396300.0	382036.0	-14264.0	410751	396415	-14336	442751	428415	-14336	3677620.435	3667315.906	-0.28019555530884865	3699799.133	3667316.795	-0.8779486894376798	False	False
lazy-depth-drop-all-one-256	1000	4888.0	4634.0	-254.0	81128.0	66864.0	-14264.0	407355	393019	-14336	439355	425019	-14336	2339174.934	2302007.353	-1.588918402799533	2339176.337	2302014.95	-1.5886526557317704	False	False
lazy-depth-take-map-256	1000	6930.0	6420.0	-510.0	399104.0	370504.0	-28600.0	481489	452817	-28672	513489	484817	-28672	4087163.559	4082707.736	-0.10901993364538232	4086623.004	4082697.512	-0.09605711111980719	False	False
lazy-depth-fold-through-source-256	1000	689574.001	689320.001	-254.0	54023121.864	54008857.864	-14264.0	17119095	17104759	-14336	33854519	33840183	-14336	None	538580683.739	None	538246355.447	538526723.06	0.05208908711831217	False	False
lazy-depth-nest-map-256	1000	5654.0	5400.0	-254.0	354580.0	340316.0	-14264.0	409176	394840	-14336	441176	426840	-14336	3775398.438	3765021.111	-0.27486706821591156	3775396.795	3765038.137	-0.27437269676444664	False	False
```

### 4. Lost counts, stop-loss, and open issues

Frozen Lost count **69 → 66**, G2 **3 → 0**, by repairing P1908/P1909/P1910. The six deferred cases and the bound controls prove value/effect/prompt/lifecycle behavior; the full census was not rerun or reclassified.

```json
{
  "red_attempts": 2,
  "implementation_attempts": 1,
  "maintenance_cargo_calls": 6,
  "final_focused_cargo_calls_including_fmt": 5,
  "no_progress_implementation_streak": 0,
  "measured_candidates": 1,
  "metric_processes": 648,
  "result_checks": 108,
  "timing_suites": 0
}
```

No compile-error implementation attempt and no nonprogressing implementation streak occurred. Red evidence used 2 focused attempts, implementation 1, maintenance 6 (including the two stale-certificate generations), and final focused verification 5 including fmt, all below eight per milestone. The workspace is a separate broad gate. Release artifact correction adds two default-build diagnostic/correction commands, with no second measured candidate. All Cargo commands and exits are in the ledger above.

Remaining issues for the director: the five incremental and 26 cumulative instruction regressions; the historical missing pre-G2 instruction pair; and the adjacent ordinary Print-triggered cancellation checkpoint behavior recorded by the first red control. No source edit beyond the fence was made to address them. The guide’s Lazy Sequences paragraph remains accurate and unchanged. No lease, promotion, unsafe, dependency, public API, or other suspension group changed.
