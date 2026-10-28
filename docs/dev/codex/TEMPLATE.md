# Codex Task Brief Template (rum)

Claude owns direction, diagnosis, and target selection; Codex executes one
brief exactly and self-verifies; Claude runs the authoritative gate afterward
and commits. Copy this file to `docs/dev/codex/brief-NNN-<slug>.md`, fill every
section, then dispatch via `scripts/dispatch-brief.sh NNN`.

The dispatcher runs `cdx exec -s workspace-write` with stdin from
`/dev/null` and logs to `.temp/logs/cdx_brief_NNN.log`. `workspace-write` (default
since 2026-07-17, least privilege) confines writes to the repo + tmp while
still allowing `cargo` and the release `rum-cli`. A brief that truly needs
process attach (the macOS `sample` profiler) is dispatched with
`CDX_SANDBOX=danger-full-access`, stated to the user. The brief's **Scope**
and **Hard rules** are the inner fence on top: "touch nothing outside this
repo, no network, only the listed commands."

Three brief shapes use this template:

- **Design-only brief** — `Scope: NONE — no source change`. Codex surveys the
  code and returns a step-by-step *plan*: files/functions per step, how the
  invariants stay intact, the gate tests per step, a call-site/ripple
  inventory, and risks + open questions — every claim carrying `path:line`
  evidence. It must also **draft the follow-up implementation briefs
  themselves**, in this template's shape, not merely describe the steps.
  Codex writes no code in this shape.
- **Diagnostic brief** — profile/measure and report; no source change. The
  first brief in any performance lane is almost always this: Claude cannot
  prescribe a precise change before the profile exists.
- **Implementation brief** — a precise, pre-diagnosed mechanical change with a
  test gate, plus a measured A/B gate when a hot path is touched.

**Plan-first is the default (user ruling 2026-07-29).** Any non-trivial or
cross-cutting task opens with a design-only brief; Claude then reviews the
plan, settles the open questions, and **rewrites** Codex's drafts into his own
implementation briefs before dispatching them one at a time. Codex's drafts
are raw material, never dispatched as-is. Trivial mechanical jobs may still go
straight to an implementation brief.

**How Claude writes an implementation brief (user ruling 2026-10-26: spend
director tokens up front rather than worker hours on a wrong route).**

- **Give the target shape as code**: types, signatures and the call
  structure. The worker fills in the bodies. A departure from the given
  shape is a STOP.
- **Put a "must not change" list beside every cost bar.** It names what is
  captured and when, the order of execution, and the observable rows. An
  edit that changes *whether or when* something happens is a STOP, never a
  cost fix.
- **Name the tempting shortcuts as forbidden.** These include duplicated
  bodies, flattening a representation, silent-drop helpers, lazy init, and
  inline attributes keyed on `debug_assertions`.
- **Ship the falsifier.** Claude writes it and calibrates it red and green
  on both arms. The worker inserts it and must reproduce both results.
- **Stage it.** The shape and its proofs form one stop. The measurement
  campaign follows only after it.
- **Hand over the instruments** with their standing exclusions and bounds
  listed. An exclusion ruled once belongs to the instrument and is
  restated in every brief that reuses it.
- **State the concurrency plan** (pitfall 9).

---

## Goal

One paragraph. What must be true when you are done. For a diagnostic brief,
the deliverable is evidence, not a code change; for a design-only brief it is
a reviewable plan plus the drafted follow-up briefs, still not a code change.

## Context pointers

- Read `AGENTS.md` (operational notes, the Testing Policy hard lines) and the
  relevant `docs/` entries named below before touching anything.
- Key files for this task: <list with one-line roles>.
- Acceptance is mechanical: the named tests and the named measured gates
  decide, not prose.

## Scope

- Files you MAY modify: <explicit list, or "NONE — diagnostic only">.
- Files/areas you MUST NOT touch:
  - `crates/rum-core/src/expr.rs` `Expr`/`Symbol` representation and any public
    type/serde shape, unless this brief's Goal is exactly that;
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, anything under
    `docs/dev/codex/` except appending to your own report — unless the brief
    explicitly lists a doc file as in-scope;
  - `.git`, git config, `Cargo.lock` (unless the Goal is a dependency change).
- **Every source change carries a standing maintenance duty — always in scope,
  whatever the fence says.** The retention contract fingerprints hash every
  `.rs`, `.py` and `.toml` under `crates/`
  (`crates/rum-stdlib/src/tests/retention_closure/local/contracts.rs`), so any
  edit anywhere expires `inventory/local-contracts.tsv` and the certificate
  cells in `inventory/promoted.tsv`; and the ownership-graph inventories under
  `crates/rum-stdlib/src/tests/ownership_graph/inventory/` record source
  positions and unknown constructs exactly. Refresh them by replaying, never by
  hand: recompute the contract fingerprints, run `local_production_acceptance`
  in both the default and `--no-default-features` builds and copy the cells it
  writes to `.temp/brief-546/certificates/`, and reconcile the ownership rows
  against the discovery the tests print. Disclose every changed row.
  Fingerprints, hashes and positions may move. **New `UnknownConstruct` rows
  for code this brief added or changed are expected** — a derive on a new
  frame, a new closure, a new macro call — and are added with each row's file
  and line disclosed; unknowns stay unknown. **STOP instead** if a row
  committed at HEAD would change its identity, status, disposition or
  exclusion, or would disappear (a row this brief added earlier is not
  "existing"), if a new `UnknownHostPayload` row appears (that needs a
  disposition decision), or if a new row points at code the brief did not
  touch. **A new suspension frame or `FrameRootTrace`/`PureFrameAbandon`
  type, in any crate, also enters rum-runtime's closed lists** —
  `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`
  (one entry per derive) and the named-frame count asserted in
  `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` — and that edit
  is in scope wherever the fence puts rum-runtime; add exactly the new types,
  nothing else.
  - **Three more standing inventories pin source sites the same way**
    (added 2026-10-28, after brief-629).
    - *LP01.* `cargo test -p rum-stdlib --test suspension_oracle --
      logical_counter_port` checks `logical-profile-required.tsv` (producer
      site and hash, columns 17 and 19), `logical-profile-bindings.tsv`
      (column 3) and the `producers` table in `logical-profile-losses.json`,
      all under `crates/rum-stdlib/tests/suspension_oracle/inventory/`. A
      producer may move; its statement text may not.
    - *Typed-slot receipts.* `cargo test -p rum-stdlib --test
      typed_slot_census` checks `typed-slot-stages.tsv`, whose receipts name
      their insertion sites.
    - *The rejection census.* Its counts are asserted with reasons in
      `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs`. If
      they move, STOP with a comparison keyed by kind, file, callee and
      reason; the director rules on it and updates the file.
  - **A brief never forbids this replay.** Either the worker replays, or the
    brief says the director replays and that the run continues past a
    failing standing inventory. A brief that forbids the edit and also stops
    on the failing test can only stop (brief-629 A5).
  - **Any stage that changes source runs the whole workspace before it
    reports.** A later measurement-only stage does not stand in for that run
    (brief-629: the ownership rows went stale at A3 and surfaced at the
    final gate).
- Do NOT add unconditional cost to the default evaluation path. Any profiling
  counter or instrumentation MUST be behind an env-gated cold path (the
  existing `RUM_PROFILE_*` `AtomicU8` pattern); never a `Mutex`/allocation/
  branch paid on every call when the flag is off. A default-path regression
  voids the change.

## Deliverable

Bullet list of concrete artifacts (for diagnostic: the tables/numbers to
produce; for implementation: code + tests + the before/after per-label rows).

## rum / Rust pitfalls (read twice)

1. **Timing numbers come from the microbench scripts, not ad-hoc wall
   clocks.** `scripts/core-language-microbench.sh` produces per-label
   `timings.tsv`; an A/B is two runs — base build and candidate build —
   compared per label. Profiling env vars are OFF for any timing number.
2. **A probe must be the label's EXACT statement — nothing added.** Do not
   wrap the timed expression in an assignment, a sink `Set`, a `Print`, or any
   extra builtin to "use" the result; that measures a different thing. If the
   loop might be elided, raise the iteration count; do not add a sink.
3. **Default-path cost is unconditional cost.** A counter that does an atomic
   load per call when disabled still costs; keep the disabled path a single
   relaxed load that returns early. No `Mutex`, no allocation, no per-call
   heap touch when the feature is off.
4. **Evaluator flows are suspension-safe.** If you touch an `eval_*` /
   `resume_*` path, preserve the suspend/resume continuation shape; do not
   collapse a `suspend_with_frame*` into an eager call.
5. **Hot paths avoid owned-tree clones.** `Expr`/`SharedExpr`/`SharedArgs` are
   `Arc`-shared; prefer borrowed traversal and `Arc` clone over deep clone on
   any path that runs per evaluation step.
6. **One variable per measurement.** Change one thing, rerun the SAME label
   plus its same-tier neighbors. An optimization that helps one row and
   regresses a neighbor in the same tier is rejected.
7. **Stop-loss is real, and it is keyed on progress.** Report the brief's
   progress counters after each attempt, and STOP when two consecutive
   attempts move none of them — do not keep tuning. **Reducing compiler errors
   is progress**: a compile error you are actively fixing is not a wall, and
   the stop-loss is measured only once the tree compiles. Keying it on the
   failure's reason instead fires on every converging mechanical repair.
8. **An instruction gate builds both arms with one codegen unit.** Per-iteration
   instruction slopes (`taskset -c 2 perf stat -x, -e cpu_core/instructions/u`,
   `(count(2N) - count(N)) / N`, stdin `/dev/null`) decide; timing does not. But
   the default release profile has 16 codegen units, and adding code anywhere in
   a crate repartitions them and moves inlining in code the change never
   touched: brief-565's ordinary `Take` read +0.6% from that alone and 0.00%
   with one unit, while untouched controls sat within ±0.02%. Build base and
   candidate with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` (from the same
   source paths' exports, each into its own target directory) whenever the
   gate's threshold is finer than about 1%.
9. **Concurrency is planned per stage, from measured resource profiles**
   (user ruling 2026-10-26; it supersedes the 2026-09-30 all-serial cargo
   rule).
   - Run one cargo command at a time. The dispatcher already runs you inside
     a memory-capped scope (`scripts/memory-scope.sh`: `MemoryMax` 16G, no
     swap, `OOMPolicy=continue`). A command that reaches the cap is killed on
     its own with exit 137, and you keep running. `oom_kill` then rises in
     your scope's `memory.events`, at
     `/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)/memory.events`. Do not
     call `systemd-run` yourself: the agy and grk sandboxes cannot reach the
     user bus.
   - Builds use `CARGO_BUILD_JOBS=8`, which peaked at about 9 GiB.
   - Tests use `RUST_TEST_THREADS=1`, unless the brief states a measured
     profile.
     - rum-stdlib's tests at the default thread count peaked at 14.6 GB and
       were OOM-killed.
     - At `17576575`, the logical profiler's tests fail under parallel
       threads, 13–16 of 30 per run. That is an open defect (`TODO.md`), not
       a property to rely on.
   - On an `oom-kill` (exit 137 with `oom_kill` risen), rerun that command
     once with one job.
   - `perf stat`, timing, Callgrind and heap captures run serially and alone,
     with nothing else running during them.
   - A stage the brief does not name runs serially.
10. **Your own self-check is a report row, not a bar.** If a check stricter
    than the brief's bar fails, report it. It is never a reason to edit
    production. After any production edit, the focused set also includes
    the repository invariant tests (`frame_root_trace`, the inventories).
11. **A measurement pipeline is re-entry safe.**
    - Never rerun a reference stage (panels, a calibration, base receipts)
      once a later stage has consumed it.
    - A resumed block skips completed stages, on the evidence of their
      receipts.
    - The report lists every attempt of every stage with its result,
      including the passes.

## Verification (MANDATORY — you run it; iterate to green / to evidence)

Diagnostic brief: produce the named artifacts and paste the key rows VERBATIM.
No source change means no test gate; the evidence IS the deliverable.

Implementation brief, run exactly these and paste results verbatim:

```
cargo fmt --check
cargo test --workspace        # or the narrower -p suite the brief names
```

Then, if the brief names a performance gate:

```
bash scripts/core-language-microbench.sh --output-dir <base-dir>   # on BASE
bash scripts/core-language-microbench.sh --output-dir <new-dir>    # on the change
# paste the per-label rum_seconds rows for the target label AND its same-tier neighbors
```

Loop: edit → test → fix → rerun, until green; then rerun the gate and paste
the before/after rows. Never claim a result without the verbatim lines.

## Checkpointing — protect the scene

All scratch output — raw logs, probe files, tables, the PROGRESS file — goes
under `.temp/brief-NNN/` at the repository root. `.temp/` is git-ignored and,
unlike `/tmp`, survives a reboot; unlike `target/`, it survives `cargo clean`.


A run can be cut short by quota, a dropped connection, or an upstream filter.
After each milestone, write `.temp/brief-NNN/PROGRESS.md` stating:

- what is **done and verified**, with the evidence already saved;
- the **exact next action**, specific enough to resume without re-deriving it;
- the **tree's state** — does it compile, which tests are green, which files
  are half-edited and in what way;
- anything the director needs in order to take over.

Prefer to leave the tree compiling at each checkpoint. An interrupted run that
leaves a readable checkpoint costs one dispatch; one that leaves unexplained
edits costs the director a reconstruction.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all changes in
  the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and write
  that in the report instead.
- Full machine access, but touch NOTHING outside this repo (except an archive
  path the brief explicitly grants), no network. The only commands you run are
  file edits within Scope, `cargo`, the release `rum-cli`, `sample` / the
  `scripts/sample-*.py` reducers, the microbench scripts, and `python3` for
  parsing.
- **Testing Policy is a hard line** (`AGENTS.md`): tests verify rum's own
  documented behavior; no black-box comparisons against other vendors'
  engines, no oracle harnesses, no compatibility matrices, no third-party
  engine outputs committed — do not reintroduce any of these.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe` (the
  only `unsafe` in the repo is the off-by-default alloc-count `GlobalAlloc` in
  `rum-cli`). If you are convinced `unsafe` is unavoidable, STOP and report.
- **Hot-path changes require the microbench A/B gate**, not just the "no
  default-path cost" rule: eval/dispatch/matcher, `Expr`/`SharedExpr`/
  `SharedArgs` construction or `Drop`, and candidate-value paths all qualify.
  A regression on any hot label is a defect to fix or report, not to ship.
- **Probe budget — keep gate runs bounded.** Best-of-2 for a wall A/B; run any
  slow defining mode ONCE; cap the total probe-run count and state the cap;
  set the FULL env up front so you never re-run on polluted data; if a number
  looks off, REPORT it — do not silently re-measure everything.
- **Timeout discipline — derive bounds from data, never guess.** Use
  `scripts/run-bounded.py --timeout SECONDS -- <command>` for anything that
  could run long or spawn children; size the bound from a prior measured run
  of the same shape, and remember a baseline build can be slower than an
  optimized one.
- Minimal diff: no drive-by reformatting, renames, or comment changes outside
  the task.
- You MUST paste the real verbatim verification output. If a run did not reach
  green / did not produce the evidence, say so explicitly — never fake it.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why (or
   "diagnostic only, no source change").
2. Verification — each command run, with the exact verbatim output lines
   (test counts, per-label rows, or attribution tables). **Quote every
   verdict line your own instruments print** (PASS, STOP, flagged), and
   settle each flagged row in the text. A report that says "no STOP" while
   its own closure file marks a row STOP hides the row from its reader, even
   when the row turns out not to be one (brief-629 A5b).
3. The finding / verdict — for diagnostic, the mechanism the numbers point to;
   for implementation, the before/after rows and any neighbor checked.
4. Stop-loss / open questions — where you stopped and why, anything unsure
   (empty if none).
