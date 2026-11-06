# Brief 678 — LP08 residual design continuation: a zero-disabled-cost completion form, the parser boundary and reset-resume

**Design continuation for the live tree: `Scope: NONE — no live source,
workload, manifest, inventory or version-control change.` Disposable
build probes only, under this brief's root. This round continues brief
677's design session; its report and draft are the starting point, not
evidence to re-derive from zero.**
Brief 677 found the measured direction — the decoded-result metadata
boundary, the owner-checked definition handoff and the post-install
prepared completion — and closed the nine native calibration rows and
the four source-attributed mirrors on the valid fixtures. It is not a
retaining form: the strongest partial form (ordered2) pays +37
instructions/call in the fixed-point resume driver and +16 bytes per
recursive level in Function and Module, turns the existing
parser-consumer control red, leaves all seven reset-resume probes red,
and covers 41/50 adopted keys with three mispair and seven Throw
wrong-outcome mutants open. The director's gate accepted the design
record and set the cost bar; this round walks the unwalked families to
a retaining form.

This round answers one question with measured evidence: **what exact
form completes the paused outer phases and the four resumed mirrors at
the standing cost bar, with the parser-boundary and reset-resume
controls green and every selected site/route/mutant row closed — or
what measured exclusion of every remaining family blocks it?**

## Goal

When this round is done, the design record states, with measurement and
not argument:

1. **A retaining form at the cost bar.** At least one measured form in
   the production-like layout (a new private `profiling/logical/
   completion.rs` and small helpers, not the appended over-35k
   `logical/mod.rs` prototype) that reproduces the base rows exactly:
   fixed-point self-instructions at base, recursive Function stack
   18,304 B/level and Module 18,896 B/level with residual 0, in two
   independent CGU=1 builds and two draws. The disabled path may pay
   only the standing cold-load gate (one relaxed load and an early
   return in an outlined helper); all mirrored work stays behind the
   enabled branch, and no off-by-default state enters a hot frame. A
   gate-only control is measured beside the form so the delta is
   attributed, not asserted. The statically selected mirrored-work
   family (`Ordinary` empty hook, `Profiled` body, one shared semantic
   body — no duplicated bodies) is the first family to walk; a different
   family must be named and measured, not implied. The form keeps the
   valid-fixture coverage: 20 valid cases SAME and 67/67 synchronous
   mirror comparisons, with the corrected nine native vectors
   reproduced.
2. **The parser boundary green.** The existing
   `logical_after_consumer_sees_the_input_parser_error` control
   (`crates/rum-runtime/src/runtime/profiling/tests/queue_boundaries.rs:190`)
   stays intact and green under the form: diagnose the first structural
   fault, state the exact terminal-parent restoration rule with
   `path:line`, and measure it. No global fault suppression, no changed
   owner identity, no changed pair check, no changed guest completion
   order.
3. **Reset-resume green.** All seven failing reset-resume probes pass
   under the form, with the exact metadata retirement rule and an actual
   host-reset probe: old-epoch markers cannot publish into the new
   epoch, no queued frame is discarded, no suffix effects, exactly-once
   retirement.
4. **The definition handoff closed.** All eight free definition finish
   sites keep their signatures and token-first interfaces; a direct call
   with no enclosing phase has an explicit measured flush; the three
   mispair mutants execute and turn the exact counter assertion red with
   semantic equality disabled.
5. **The outcome subsets closed.** The seven Throw wrong-outcome mutants
   turn the exact per-outcome counter assertion red (the current
   aggregate assertion cannot distinguish them); the assertion is
   designed and measured, not relabelled from the aggregate.
6. **The route plan.** All 50 adopted keys exercised individually at
   their pause/error/outcome routes; the nine unobserved keys covered by
   a calibrated fixture (replace the invalid While plan, whose numeric
   `Less` that minimal session does not support); the full per-site/
   route mutant matrix designed, with no exclusion assumed. A director
   ruling is requested for any row the form cannot cover.
7. **The plan.** A step-by-step design with `path:line` evidence per
   step, the invariants each step preserves, the gate tests and mutants,
   the call-site/ripple inventory, risks and open questions — plus the
   updated drafted implementation brief in `TEMPLATE.md` shape.

If a family has no form under the cost bar, the record is a measured
exclusion of that family with its exact rows; an unmeasured family is
OPEN, and "impossible" is not a result. The promotion remains blocked
until the implementation stage proves all of it.

## Context pointers

- `.temp/brief-677/report-design.json` (sections `answer`, `cost`,
  `coverage`, `plan`), `drafts/`, `costs/`, `forms/`, `source-map/`,
  `calibration/corrected-native-vectors.json` — the state this round
  continues. Read-only; copy before adapting a script.
- `.temp/director/brief-677-gate.md` — the director's rulings on the
  cost bar and the eight open questions.
- `docs/dev/profiling.md` — the contract.
- Brief-637's "Ruling: no adopted phase crosses a pause"
  (`docs/dev/codex/brief-637-lp03-evaluator-dispatch-arguments.md:1248`)
  — the superseded deferral.
- Live sites: `crates/rum-runtime/src/runtime/profiling/logical/mod.rs`
  (`complete_phase`, `finish_input_required`, `ProfileBoundary`,
  `Call::pause`), `logical/{queue_api,epoch,context,segments,record,
  incoming}.rs`, `profiling/{timing,control_builtin,control_timing}.rs`,
  `suspension/typed/adapter.rs`, `suspension/core/{driver,profile_mode,
  profile_queue}.rs`, `resolution.rs`, `eval_call_prepare.rs`,
  `eval_core/call.rs`, and the queue-boundary tests.
- `.temp/brief-621/MANIFEST-DESIGN.md` — the promotion model.
- `docs/dev/codex/TEMPLATE.md` and `AGENTS.md`.

## Scope

Files you MAY write: anything under `.temp/brief-678/` — probe trees,
builds, binaries, receipts, the draft implementation brief and the
report.

MUST NOT touch:

- the live repository tree — no source, test, workload, manifest,
  inventory, docs or `.git` change; you run no version-control command;
- `.temp/brief-677/` or any retained root, except to read and copy.
  Never execute a file in place;
- the public API, features, serde, manifests, dependencies, `unsafe`.

Do not run the workspace acceptance suites, the semantics matrix, the
census or inventory replays, or any source gate. This round designs and
probes; it repairs nothing.

## Hard bars

- **The cost bar (director ruling, supersedes the draft's open
  question).** The disabled path pays only the standing cold-load gate;
  the recursive Function and Module stack rows equal base exactly
  (18,304 / 18,896 B per level, residual 0) in both builds and draws;
  the disabled instruction delta is attributed to a measured gate-only
  control. Off-by-default state in a hot frame is a violation even at
  zero CPU. The measured +37/call and +16 B/level are rejected.
- The instruments are calibrated before any form is judged: the
  corrected nine native vectors, the four mirrors and the boundary
  controls reproduce the current tree. A probe that cannot reproduce the
  finding cannot judge a form.
- Every count, key, route and `path:line` is re-derived from the current
  source; a brief-677 number is a claim, never evidence.
- The parser control, its assertions and the guest completion order stay
  as they are; a form that needs a changed assertion is rejected.
- A form that changes a guest carrier, a pushed value, an adopted key, a
  sampling boundary or a token-first interface is rejected unless the
  director names the exception first.
- No live source edit, no version-control command, no network, no new
  `unsafe`, no manifest or workload change.

## Verification

One cargo command at a time; jobs 8, offline/locked; the worker's cargo
runs plain inside the dispatcher's 16G scope (never nested). Build both
arms at one path with one release codegen unit; two independent builds
and draws for every cost row; serial Callgrind, CPU-pinned, release,
every `RUM_PROFILE_*` cleared except the declared modes; four stack
heights twice each with the residual printed. A timeout is not data.
Persist raw output before parsing; derive any bound from data.

## Checkpointing

All scratch under `.temp/brief-678/`; keep `PROGRESS.md` with the exact
next action and the receipts already saved. Leave every tree compiling
at each checkpoint.

## Deliverable

- `.temp/brief-678/report-design.json` in `rum-report/1`, with the
  measured cost rows, the parser and reset controls, the definition and
  outcome mutant results, the route plan and every open question.
- `.temp/brief-678/drafts/implementation-lp08-residual-2.md` — the
  updated implementation brief in `TEMPLATE.md` shape, plus the probe
  trees, binaries and receipts.
- Run `python3 -I scripts/report.py check 678` and return its summary and
  all compliance verdict lines.

## Report format

Every run and verdict line quoted; the drafted implementation brief
under the draft directory; the report's `summary` states the one answer
(measured) and the draft's bar list.
