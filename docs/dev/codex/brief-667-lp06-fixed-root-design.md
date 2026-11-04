# Brief 667 — LP06 design: the fixed debug root at the two execution-scope sites

**Design for the live tree: `Scope: NONE — no live source, workload, manifest
or version-control change.` Disposable build probes only, under this brief's
root.**
Brief 666 stopped at one hard bar: the pinned rebound-carrier's execution
scope adds a fixed 16 B to `evaluate_at_root`'s debug frame (2528 -> 2544 in
all 88 supplied shape/height points), and the two permitted attachment
attempts made it worse (2560, 2576). I reproduced all three rows on
independent builds. This round answers one question with measured evidence:
**what is the minimal debug-frame cost of the execution scope at the two
attachment sites under the pinned carrier and the standing hard lines, and is
a 2528 B form possible?** It returns either the 2528 B form with the full row
set, or the measured exhaustion proof that no form reaches it, with the
minimum-cost form and every family's exclusion.

## Goal

When this round is done, the design record states, with measurement and not
argument, one of two things:

- **A 2528 B form exists.** The exact form is in a probe tree; its
  `evaluate_at_root` debug CFA peak is 2528 B at every one of the 88 points,
  in two independent builds, with the checkpoint body still 4,606 live
  instructions / 25,861 B / 4,693 own Ir in both draws, zero real decoded
  byte differences; the root and abandonment check sequences still 223 -> 231
  and 1262 -> 1270 (+8 each, check-only); 29/29 own and incoming pairs exact;
  `drive_input_request` 88 own Ir/call on all 121 resume labels in both
  draws; admission 63 Ir/admission on both routes and draws; and the whole
  falsifier set green (lp06 36/36, transitions 11/11, entries 23/23, combined
  4/4, lp664 6/6, lp665 4/4; the literal 39 red + 1 green host-root control
  on HEAD, 40/40 on the form; the cap and the three stale/foreign falsifiers
  red on HEAD and green here, identical formatted bodies). The round drafts
  the 668 implementation brief for it.
- **No 2528 B form exists under the pinned constraints.** The record is an
  exhaustion proof: every form family is enumerated, each with its measured
  frame, its measured disabled-path cost at every site its guard runs, and
  the constraint that excludes it; the minimum-cost form is named with its
  full row set; and the round drafts the 668 implementation brief for the
  minimum. "Impossible" without a measured exclusion of every family is not a
  result.

The carrier's semantics are **not** reopened. The pushed value, the push/pop
conditions, the two attachment sites, the checkpoint body, the root/abandon
check sequences, the `ContextGuard` niche, the optional epoch/storage, the
resume hook and the `resume_profile_configured` code shape stay exactly as
ruled at 665 and as implemented at 666.

## Context pointers

- `.temp/director/brief-666-gate.md` — the ruling, the reproduced rows, the
  constraints. Read it first.
- `.temp/brief-666/report.json` — the 666 STOP: the 88 captures, the two
  failed attachment forms, the workspace and inventory receipts.
- `.temp/brief-666/attempts/{final,attached,internal}/` — the three measured
  source trees, binaries, symbols and CFA records. The `final` tree is the
  pinned form.
- `.temp/director/gate-666/fde.py` — my independent FDE reader (symbol
  address from `nm`, CFA peak from `readelf --debug-dump=frames`).
  `.temp/director/gate-666/head/` is a clean HEAD tree; `head-target/` and
  `live-target/` hold the two independent debug lib-test builds that
  reproduce **2528** (HEAD) and **2544** (pinned). Rebuild your own; do not
  reuse these targets.
- `.temp/brief-665/probe/rebound-carrier-sem` — the measured semantic arm:
  the whole tree, production plus literal tests plus replayed inventories.
  The 666 live tree equals this tree plus the required
  `resume_profile_configured` comment (the only difference; verified).
- `.temp/brief-665/d-instruments/` — `frame_cfi.py` and the 22-shape x
  four-height matrix in `physical-frame-differences.json`; the 665 focus,
  entry and d-instruments for the instruction rows.
- `.temp/brief-665/rebound-carrier-from-live.patch`,
  `source-proof-rebound-carrier.json`, `semantic-proof-rebound-carrier.json`,
  `cap-stale-foreign-proof.json` — the pinned surface and the exact records.
- Live sites: `profiling/logical/context.rs` (`ExecutionScope`,
  `ContextGuard`, `enter_front_with_control`), `resolution.rs`
  (`evaluate_at_root`), `suspension/core/abandon.rs`
  (`drain_abandonment_worklist`), `suspension/core.rs` (the third
  `NativeStackEntry::enter` site), `native_stack_guard.rs` (the entry guard),
  `profiling/timing.rs` (the adopted tokens).

## Scope

- **NONE for the live tree.** No file in the repository is edited, no derived
  inventory is copied, no workload, manifest or version-control command runs.
- Probe trees live under `.temp/brief-667/probe/`; copy HEAD
  (`.temp/director/gate-666/head/`) and the pinned arm
  (`.temp/brief-665/probe/rebound-carrier-sem`) into it and edit the copies.
  A form may change files anywhere in a probe tree — `native_stack_guard.rs`
  included — but every variant tree must be disclosed file by file, and the
  implementation scope of 668 is the director's to rule, not yours.
- `.temp/brief-665/` and `.temp/brief-666/` are read-only; never execute in
  place.

## The question (facts, constraints, duties — not a prescribed form)

**The bar.** `evaluate_at_root`'s debug CFA peak must be **2528 B, exactly**,
at all 88 points, in the uninstrumented build and in the supplied instrumented
capture. Equality, not a bound: 2543 would fail as surely as 2544.

**The fixed arithmetic.** 2528 = 158 x 16. The three measured forms are one,
two and three 16-byte steps above it: pinned 2544 (the zero-sized Drop
marker), 2560 (the scope holding the entry), 2576 (the entry created inside
the scope). The instruction rows of the pinned form are otherwise accepted;
only the frame byte is at issue.

**The constraints.** Unwind safety (the unwind falsifiers); the push/pop
conditions and the pushed value; the two sites; the checkpoint body; the
root/abandon +8 check sequences; the pinned representation; no new `unsafe`;
no `debug_assertions`-keyed attribute or debug-specific policy; and **no
default-path cost anywhere**. The last one is a discriminator, not a
formality: `NativeStackEntry` has a third production site
(`suspension/core.rs:636`) besides the two attachment sites, so any check,
flag or call that rides that guard's construction or drop is paid there on
the default path — a violation even at zero measured CPU, and off-by-default
state in a hot frame is the rule's own wording.

**The calibration duty.** Before enumerating forms, establish by direct
measurement what the added 16 B is: the pinned zero-sized Drop local; a
non-Drop call with no local; a one-byte non-Drop local. Two independent
builds per shape, and the two instruments (your FDE reader and the supplied
`frame_cfi.py`) must agree on HEAD 2528 and pinned 2544. An instrument that
cannot reproduce those two rows is not usable for this question.

**The enumeration duty.** Every family gets a measured verdict — at minimum:
the pinned marker; the marker merged with the existing entry in each
direction; an attachment to a guard already present at the site with no new
local; and any other form you can construct inside the constraints. The
template's two-attempt stop-loss applies to tuning one family, not to
enumerating families: the round ends when the family space is exhausted, not
when two attempts stop moving.

## Deliverable

- `.temp/brief-667/report-design.json` (rum-report/1, stage `design`) with:
  - the calibration table (HEAD 2528, pinned 2544, the three forms, two
    builds each, both instruments);
  - one row per form: identity, probe-tree diff reference, 88-point frame
    row, checkpoint row (4,606 / 25,861 B / 4,693 own Ir, both draws, zero
    real byte differences), root 223 -> 231, abandonment 1262 -> 1270, the
    disabled-path cost at every site its guard runs (own Ir/call), and the
    exclusion reason or the selection;
  - for the selected form: 29/29 own and incoming pairs, `drive_input_request`
    88 on all 121 labels both draws with incoming slopes, admission 63 both
    routes and draws, `cargo fmt --check`, and the falsifier set (the named
    suites and the 39 + 1 split on HEAD, 40/40 on the form);
  - the exhaustion proof (if no 2528 form) or the exact form diff;
  - the drafted 668 implementation brief in `TEMPLATE.md` shape, with the
    exact bars.
- Every claim cites a log line the checker resolves; run
  `python3 -I scripts/report.py check 667` and return its summary.

## Hard bars

- The instrument calibration above, twice, before any form is believed.
- All 88 points per form — never one point, never a shape average.
- The checkpoint row and the root/abandon rows per form; the selected form's
  full row set as above. A form that moves a checkpoint byte or an own-PC
  count is excluded regardless of its frame.
- A form that adds a default-path cost at any site is excluded; its measured
  cost is still recorded.
- A form that changes any semantic row is excluded; the row is quoted.
- No accepted reference is redrawn: the 665 instruments and the 666 captures
  are reused, not regenerated.
- Probe builds: one cargo command at a time, jobs 8, inside the dispatcher's
  memory scope; timing and Callgrind runs serial and alone on CPU 2, release
  `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, every `RUM_PROFILE_*` cleared.
- No source change reaches the live tree; no git command; no network.

## Verification

Design-only: no workspace acceptance run is required. The named rows above
are the gate. The selected form's falsifiers run in its probe tree (or a copy
of it), with the assertion bodies byte-identical to the 665 semantic arm.

## Checkpointing and concurrency

All scratch under `.temp/brief-667/`; keep `PROGRESS.md` with the exact next
action, the compiled source state and the receipts already saved. Leave each
probe tree compiling at a checkpoint. One cargo command at a time; builds at
`CARGO_BUILD_JOBS=8`; tests at `RUST_TEST_THREADS=8 --no-fail-fast`; on an
exit 137 with `oom_kill` risen, rerun that one command once with one job.

## Hard rules

- No version-control command, network, new `unsafe`, public API/feature/serde
  change, workload change, inventory weakening or hand-edited derived cell.
- No live-tree edit of any kind; no execution in a read-only prior root.
- No change to the pinned carrier representation, the pushed value, the
  push/pop conditions, the two attachment sites, the checkpoint body or the
  root/abandon check sequences.
- No external-engine comparison; rum-native A/B only.
- The director's gate owns acceptance; the 668 scope is not yours to widen.

## Report format

`.temp/brief-667/report-design.json` in `rum-report/1`, every run and attempt
listed, every verdict line quoted, every claim cited, every FLAGGED row
settled. Run `python3 -I scripts/report.py check 667` and return the summary
plus every compliance verdict line.
