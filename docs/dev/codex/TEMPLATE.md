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
   (test counts, per-label rows, or attribution tables).
3. The finding / verdict — for diagnostic, the mechanism the numbers point to;
   for implementation, the before/after rows and any neighbor checked.
4. Stop-loss / open questions — where you stopped and why, anything unsure
   (empty if none).
