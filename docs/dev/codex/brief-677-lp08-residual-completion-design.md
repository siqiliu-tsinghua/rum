# Brief 677 — LP08 residual design: completing paused outer phases and the resumed synchronous attributions

**Design for the live tree: `Scope: NONE — no live source, workload,
manifest, inventory or version-control change.` Disposable build probes
only, under this brief's root.**
Brief 676 gated its design round as STOP: the complete non-encoding
counter promotion has 91,424 in-set loss cells and no retaining
promotion, the publication forms are excluded or unmeasured, and the
nine native queued/resumed differences come from a deliberate deferral.
The director has resolved the two blocking questions: the residual
completion is authorized (this brief), and the epoch-wide synchronous
collector gets its own design round (brief 678). This round designs the
residual completion; it does not implement it.

The director ruling of 2026-11-06 supersedes brief-637's
"no adopted phase crosses a pause": the target contract in
`docs/dev/profiling.md` governs the final line. `InputRequired` leaves
the record open and publishes no terminal call or error; a pause that
publishes `Outcome::Error` is a measured deferral, not a semantic
decision. The adopted explicit-boundary form is already live on some
routes; this round decides how far it extends and in what exact shape.

This round answers one question with measured evidence: **what exact
form completes the open outer-phase records across a suspension on the
resumed path so that the resumed logical vector equals the queued one,
and what exact form restores the four resumed synchronous
attributions — at what disabled-path cost and coverage, with the
adopted keys, guest carriers and token-first interfaces unchanged?**

## Goal

When this round is done, the design record states, with measurement and
not argument:

1. **Current state, re-derived.** For every site that still closes a
   record as `Outcome::Error` on `InputRequired` (`complete_phase`'s
   pause branch and `complete_definition_rule`, and any other current
   path), the exact site, key, enclosing logical operation, and how it
   closes. For every site that already inserts an explicit boundary
   (`call.pause()` plus `logical_insert_boundary`, the LP02 test span,
   the queue-boundary tests), the exact site and what makes it work.
   The nine native rows are the calibration:
   `call.stage.function`, `call.stage.ordinary_builtin`,
   `evaluate.once_unstable`, `evaluate.total`, `evaluate_call.dispatch`,
   `evaluate_call.resume_after_head`, `evaluate_call.total`,
   `evaluate_inner_unstable.call`, `evaluate_inner_unstable.total`, with
   the queued/resumed vectors reproduced before any form is judged
   (`.temp/brief-676/forms/native-parity-blocker.json`). The four
   synchronous groups are the other target:
   `evaluate.fixed_point_equal`, `evaluate.stability_check`,
   `evaluate.trace_update`, `prepare_arguments.append_arg`, with the
   exact missing observation named per group.
2. **The form.** For each open family, the exact carrier and lifecycle:
   where the boundary is created, what it is attached to (queue entry,
   frontier, frame), where the resumed completion observes the actual
   outcome, how exactly-once is guaranteed, and what happens on a second
   pause, a suspending cleanup, cancellation, reset, Drop and an invalid
   epoch. The contract is `docs/dev/profiling.md`: `InputRequired`
   closes an executing interval and leaves a record open, publishing no
   terminal call or error; completion publishes the specified outcome
   exactly once; cancellation and reset retire leases exactly once and
   never fabricate `ok`. The form must preserve guest carriers, pushed
   values, sampling boundaries, every adopted `PhaseKey` identity/name/
   policy, the token-first start/finish interfaces and the public API.
   A form that merely relabels the existing `Outcome::Error` rows,
   substitutes telemetry, or leaves a record open forever is rejected
   with the measured red row named.
   The four synchronous groups carry their own form: what observation
   the queued path makes and the resumed path misses (source
   attribution, the synchronous mirror), and the exact point that
   reproduces it.
3. **The cost.** Each form carries its measured disabled-path cost
   (instruction slope and stack at one source path, CGU=1, two
   independent builds) and its enabled coverage rows. A hot recursive
   frame states its per-level stack bytes; an enabled-only cold helper
   does not excuse a default-path slot. No form is closed on source
   intent or on another label's savings.
4. **The mutants.** For each adopted phase/site/route the form covers:
   completion suppression, duplicate completion, wrong
   `Some`/`None`/`Return`/`Throw`, missing resumed mirror or source
   restoration, token mispair, and omitted reset/Drop cleanup must each
   turn the exact counter assertion red with semantic equality disabled.
   A real candidate SAME and zero structural faults come first; a base
   loss is never mutant credit.
5. **The plan.** A step-by-step design with `path:line` evidence per
   step, the invariants each step preserves, the gate tests and
   mutants, the call-site/ripple inventory, risks and open questions —
   plus the drafted implementation brief in `TEMPLATE.md` shape.

If a site family has no zero-cost form under the pinned constraints, the
record is a measured exhaustion proof with the minimum-cost form named;
"impossible" without a measured exclusion of every family is not a
result. A family left unmeasured is recorded OPEN, not closed.

## Context pointers

- `.temp/brief-676/report-design.json`, `forms/native-parity-blocker.json`,
  `forms/thread-exclusions.json`, `forms/epoch-exclusions.json`,
  `inventory-current/loss-groups.tsv`,
  `inventory-current/loss-disposition.tsv`,
  `inventory-current/completion-sites.tsv`,
  `inventory-current/path-sites.tsv` and `drafts/` — the state this
  round starts from. Read-only; copy before adapting a script.
- `docs/dev/profiling.md` — the contract: lifecycle and epochs,
  logical operations and keys, execution telemetry, the owner table.
- Brief-637's "Ruling: no adopted phase crosses a pause"
  (`docs/dev/codex/brief-637-lp03-evaluator-dispatch-arguments.md:1248`)
  and the runs that led to it — the deferral this round supersedes.
- Live sites: `crates/rum-runtime/src/runtime/profiling/logical/mod.rs`
  (`ProfileBoundary`, `Call::pause`, `finish_input_required`,
  `complete_phase`, `complete_definition_rule`, the LP02 test span,
  `logical_profile_test_rows`), `logical/queue_api.rs`,
  `logical/epoch.rs`, `logical/context.rs`, `logical/segments.rs`,
  `logical/record.rs`, `logical/incoming.rs`, the
  `profiling/timing.rs`, `profiling/control_builtin.rs`,
  `profiling/control_timing.rs` boundary users, the resume-frame and
  suspension paths they depend on, and the queue-boundary tests under
  `profiling/tests/`.
- `.temp/brief-621/MANIFEST-DESIGN.md` — the promotion model and the
  rule that an owner promotion is atomic and retains its regression.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md` — the standing duties and
  the testing policy.

## Scope

Files you MAY write: anything under `.temp/brief-677/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-676/`, `.temp/brief-675/`, or any retained root, except
  to read and copy. Never execute a file in place;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs and
probes; it repairs nothing.

## Hard bars

- The instruments are calibrated before any form is judged: the nine
  native rows, the queued/resumed vectors and the boundary probes must
  reproduce the current tree's rows. A probe that cannot reproduce the
  finding cannot judge a form.
- Every count, key, route and `path:line` is re-derived from the
  current source; a brief-676 number is a claim, never evidence.
- A form that changes a guest carrier, a pushed value, an adopted key,
  a sampling boundary, a completion order or a token-first interface is
  rejected unless the design names the exception, the contract clause
  it serves and the measured red row.
- A disabled-path cost claim needs the one-source-path, CGU=1,
  two-independent-build instruction/stack measurement; savings elsewhere
  do not substitute.
- The contract in `docs/dev/profiling.md` is authoritative; the 637
  pause-as-Error ruling is superseded for the sites it still governs
  (director ruling 2026-11-06, restated). A design that keeps the
  terminal Error and calls the remaining parity impossible is not an
  answer unless it is a measured exclusion of every family.
- No live source edit, no version-control command, no network, no new
  `unsafe`, no manifest or workload change.

## Verification

One cargo command at a time; jobs 8, offline/locked; the worker's cargo
runs plain inside the dispatcher's 16G scope (never nested). Build both
arms at one path with one release codegen unit for every comparison;
serial Callgrind, CPU-pinned, release, every `RUM_PROFILE_*` cleared
except the declared modes. A timeout is not data. Persist raw output
before parsing; derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-677/`; keep `PROGRESS.md` with the exact
next action and the receipts already saved. Leave every tree compiling
at each checkpoint.

## Deliverable

- `.temp/brief-677/report-design.json` in `rum-report/1`, with the
  measured rows, the site/key state, the form families and their costs,
  the synchronous-group evidence and every open question for the
  director.
- `.temp/brief-677/drafts/implementation-lp08-residual.md` — the drafted
  implementation brief in `TEMPLATE.md` shape, plus the probe trees,
  binaries and receipts that support it.
- Run `python3 -I scripts/report.py check 677` and return its summary and
  all compliance verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one answer
(measured) and the draft's bar list.
