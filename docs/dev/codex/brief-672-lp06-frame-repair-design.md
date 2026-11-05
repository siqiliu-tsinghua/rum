# Brief 672 — LP06 design: the `try_apply_function` frame (the E1 adapter's slot)

**Design for the live tree: `Scope: NONE — no live source, workload,
manifest or version-control change.` Disposable build probes only, under
this brief's root.**
D2 gated LP06 open: base `654f2ce5` against the committed line
`19b61027` shows six ordinary stack rows growing, and the physical-frame
attribution puts the whole per-level growth on `try_apply_function` —
the release prologue reservation 1,048 -> 1,192 B (+144, 13.7%) and the
debug per-level total 2,816 -> 2,928 B (+112). The 653 D campaign's same
six rows on `029c06ff` were
0/-32, and the 668 source stage measured surface vs surface+A1 at 0. The
suspect is the E1 by-value result adapter that returned the eight F6
rows to base (`runtime_path_timing_start` plus
`finish_profile_runtime_path_timing_adopted_boundary(token, produced)`),
which holds a larger temporary across the recursive `evaluate` call.
This round answers one question with measured evidence: **what is the
minimal form of that adapter's locals that returns
`try_apply_function`'s frame to the base reservation while keeping the
eight F6 rows exact, F7 632/600 and the While/enabled evidence green —
and if no zero-growth form exists under the pinned constraints, what is
the measured exhaustion proof and the minimum-cost form?**

## Goal

When this round is done, the design record states, with measurement and
not argument, one of two things:

- **A zero-growth form exists.** The exact form is in a probe tree; the
  release `try_apply_function` prologue reservation is 1,048 B
  (`sub $0x418`) and the debug reservation 2,816 B/level, at every
  measured height, in two independent builds; the six ordinary rows
  (`function-body`, `status-body`, `message-body` in debug and release)
  are back to the base slopes with zero residual, and no other ordinary
  row grows; the eight `try_apply_function` F6 rows and the 21
  `eval_control_data_builtin'2` rows return their fresh base pairs
  exactly in both draws; F7 is 632/600 under 856; the three While owner
  slopes and the enabled controls are unchanged; the lease falsifier is
  green. The round drafts the implementation brief for it.
- **No zero-growth form exists under the pinned constraints.** The
  record is an exhaustion proof: every form family is enumerated, each
  with its measured frame, its measured F6/F7/While effect and the
  constraint that excludes it; the minimum-cost form is named with its
  full row set; and the round drafts the implementation brief for the
  minimum. "Impossible" without a measured exclusion of every family is
  not a result.

The carrier semantics are **not** reopened: the pushed value, the
push/pop conditions, the two attachment sites, the checkpoint body, the
root/abandon check sequences, the `ContextGuard` niche, the optional
epoch/storage, the resume hook, the lease drain and the phase/epoch
behaviour stay exactly as implemented at `19b61027`. The design changes
only where the adapter's locals live and how they are passed — never the
branches, the observations, the public API, features or serde.

## Context pointers

- `.temp/director/brief-671-gate.md` — my ruling, the six rows and my
  reproductions. Read it first.
- `.temp/brief-671/report.json` — the D2 record: the stack panel, the
  `try_apply_function` totals in
  `instruments/safe-frame-attribution/comparison.json`, the six-row
  slopes in `instruments/stack-comparison.json`, F6 in
  `panels/candidate-retained29.json`, the While rows and the enabled
  controls.
- `.temp/brief-668/d-instruments/stack-comparison.json` — surface vs
  surface+A1 at 0 (the A1 repair did not move these rows).
- `.temp/brief-653/instruments/stack-comparison.json` — `029c06ff` at
  0/-32 on the same six rows.
- `.temp/director/gate-671/repro-stack.py` and the objdump prologues —
  my independent measurements; `.temp/brief-671/instruments/bin/`
  holds the pinned binaries (base and candidate, `cli`, `stack-debug`,
  `stack-release`) with their receipts. Rebuild your own; do not reuse
  their targets.
- Live sites: `crates/rum-runtime/src/runtime/scoping/callable.rs` (the
  E1 adapter in `try_apply_function`),
  `crates/rum-runtime/src/runtime/scoping/callable/status_application.rs`
  (the borrowed finish), `crates/rum-runtime/src/runtime/profiling/timing.rs`
  (the boundary helpers), `profiling/logical/` (the scope and phase
  code), and `suspension/core/` (the lease drain and the frame storage).
- `.temp/brief-671/src/{base,candidate}` — the two exports, read-only,
  with their hash manifests; the 668/669 briefs and the 667 design for
  the carrier and the standing hard lines.

## Scope

Files you MAY write: anything under `.temp/brief-672/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-671/`, `.temp/brief-668/`, `.temp/brief-653/` or any
  other retained root, except to read and copy. Never execute a file in
  place;
- the two 671 exports, except to copy;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs and
probes; it repairs nothing.

## Hard bars

- The design's probe instrument is calibrated first: the unmodified
  candidate reproduces the six pinned slopes exactly (base and
  candidate), the release prologue reservation on the ordinary build is
  the measured 1,192 B, and the stack probe's zero residual is
  reproduced. A probe that cannot reproduce the finding cannot judge a
  form.
- A zero-growth claim needs the six rows at the base slopes and the
  eight F6 pairs exact on the *same* probe tree, plus the 21 builtin
  rows exact, F7 632/600, the three While slopes and the enabled
  controls unchanged. Savings elsewhere do not substitute.
- A form that removes or weakens the F6 fix's instruction effect, the
  disabled-path guards, the lease drain or any carrier semantic is
  rejected, with the measured red row named.
- The frame is measured with at least two independent instruments (the
  stack probe and the static prologue reservation, or the physical-frame
  CFA), at every height, in two independent builds.
- No live source edit, no version-control command, no network, no new
  `unsafe`, no manifest or workload change.

## Verification

One cargo command at a time; jobs 8, test threads 8, offline/locked; the
worker's cargo runs plain inside the dispatcher's 16G scope (never
nested). Build both arms at one path with one release codegen unit for
every comparison; serial Callgrind, CPU-pinned, release, every
`RUM_PROFILE_*` cleared except the declared modes. A timeout is not data.
Persist raw output before parsing; derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-672/`; keep `PROGRESS.md` with the exact
next action and the receipts already saved. Leave every tree compiling
at each checkpoint.

## Deliverable

- `.temp/brief-672/report-design.json` in `rum-report/1`, with the
  measured rows, the form families, the exclusion or the zero-growth
  form, and every open question for the director.
- `.temp/brief-672/drafts/` — the drafted implementation brief in
  `TEMPLATE.md` shape, plus the probe trees, binaries and receipts that
  support it.
- Run `python3 -I scripts/report.py check 672` and return its summary and
  all compliance verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one answer
(measured) and the draft's bar list.
