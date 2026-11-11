# Brief 725 — G15-C2: install the prepared-stage completion (P0492)

## Goal

Install the accepted family2 repair: after a real prepared-argument carrier
is installed and normalized, publish the exact owned
`call.stage.prepare_arguments` completion as `None` before the later call
dispatch can Return or Throw, so the resumed path equals the queued path
whole-record (rum-vs-rum). Preserve the ordinary queued publication
(`eval_core/call.rs:277-279`), every real producer Return/Throw/InputRequired,
exact diagnostic ownership, every guest effect, the installed fixed-point,
definition-resume and P0248 repairs, and the disabled path. The accepted
design and its copied receipts are `.temp/brief-724-c1/` (report, plan,
inventory, both drafts, `cost-comparison2.json`,
`falsifier-calibration2.json`, `source-candidate2/`, `diffs/`).

## Context pointers

- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`, the testing and organization
  policy documents.
- `.temp/brief-724-c1/REPORT.md`, `PLAN.md`, `INVENTORY.md`,
  `DRAFT-stage-A-shape-and-falsifiers.md` and the continuation ruling appended
  to brief 724 (G0, G1, G2 and the family2 acceptance).
- `crates/rum-runtime/src/runtime/eval_core/call.rs` — the two prepared
  suspension helpers at `:26`/`:50`, `install_prepared_call_arguments`,
  `resume_prepared_call_dispatch_frame`, `PreparedDispatchFrame`, the ordinary
  and unreachable dispatchers.
- `crates/rum-runtime/src/runtime/profiling/logical/queue_api.rs` — generic
  boundary insertion and the new completion entry.
- `crates/rum-runtime/src/runtime/suspension/core/{storage.rs,
  profile_queue.rs,profile_queue/arena.rs}` — FrameStorage wrappers, the
  `Kind` enum, consumer matching, and the new `profile_queue/prepared.rs`.
- `crates/rum-runtime/src/runtime/profiling/tests/fixedpoint_pass.rs` — the
  permanent-regression pattern this brief's new test follows (queued vs
  resumed whole rows, `test_support::enabled()`, existing accessors only).
- Immutable evidence: `.temp/g15-c2/c9/` (the accepted P0492 witnesses and
  panels) and `.temp/brief-724/`.

## Scope

- Files you MAY modify:
  1. `crates/rum-runtime/src/runtime/eval_core/call.rs`.
  2. `crates/rum-runtime/src/runtime/profiling/logical/queue_api.rs`.
  3. `crates/rum-runtime/src/runtime/suspension/core/storage.rs`.
  4. `crates/rum-runtime/src/runtime/suspension/core/profile_queue.rs`.
  5. `crates/rum-runtime/src/runtime/suspension/core/profile_queue/arena.rs`.
  6. New `crates/rum-runtime/src/runtime/suspension/core/profile_queue/prepared.rs`.
  7. New `crates/rum-runtime/src/runtime/profiling/tests/prepared_stage.rs`
     and one `mod prepared_stage;` line in
     `crates/rum-runtime/src/runtime/profiling/tests/mod.rs`.
  8. The standing derived inventories and their canonical replay outputs:
     `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`
     and the named-frame count in
     `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` (exactly the
     new frame type), plus the canonical regeneration of the retention
     fingerprints, certificate cells, ownership/census/typed-slot/LP01
     inventories through their own replay commands — never hand-edited.
- Files/areas you MUST NOT touch:
  - `Expr`/`Symbol` representation, any public type shape; the public
    `TypedFrame` trait; the driver's dispatch order or polling; `Cargo.lock`.
  - The ordinary queued publication body, the unreachable P0496 preparation
    branch body, the fixed-point publication body (`profile_queue.rs:368`)
    and the definition publication body (`:608`).
  - The C9 campaign harness (`c2_probe.rs`, `c2_native.rs`, the oracle
    fixture edits) from the design copies; it must never enter the live tree.
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`; the
    `.temp/g15-c2/c9`, `c8`, `c7`, `.temp/brief-623`, `.temp/brief-724` and
    `.temp/brief-724-c1` scenes.
- The standing maintenance duty applies in full: after the last edit and
  before the first acceptance suite, replay all seven inventory families
  canonically (retention contract fingerprints; ownership discovery and
  reconciliation; both certificate builds and `merge_promoted`; the rejection
  census; LP01 producer sites, bindings and losses; typed-slot receipts; the
  frame-root closed lists and named-frame count). Disclose every changed row;
  the new frame adds exactly one genuinely new derived type. STOP on a
  changed existing identity/status/disposition/exclusion, a disappearing
  row, a new `UnknownHostPayload`, or a row outside touched code. The census
  pin is updated once, at the final source, with a per-class table; a
  changed count is a STOP with a kind/file/callee/reason comparison.

## The accepted shape (reproduce it; a departure is a STOP)

The mechanical source is the accepted `.temp/brief-724-c1/source-candidate2`
diff for the six production files. Its call structure:

```rust
// call.rs: both prepared suspension helpers select static mode from the
// existing timing token. Some(token) -> cold helper records InputRequired and
// suspends the actual witness with ProfiledPreparedDispatchFrame; None ->
// the existing ordinary PreparedDispatchFrame. Nothing else changes.

struct ProfiledPreparedDispatchFrame { prepared: PreparedDispatchFrame }

impl Session {
    fn resume_prepared_call_dispatch_frame<const PROFILED: bool>(...) {
        let prepared = outcome?;                    // real errors propagate
        let restore = self.install_prepared_call_arguments(...)  // install+normalize
            .map(SuspendedUnevaluatedRestorePlan::freeze);       // freeze restoration
        if PROFILED {
            self.logical_complete_prepared_arguments();          // exact owner, None
        }
        self.resume_suspended_call_with_dispatch_plan(..., stage_index + 1, restore)
    }
}
// Ordinary frame resume delegates with ::<false>; the enabled wrapper with ::<true>.

enum Kind { Guest, PreparedGuest(Arc<ProfileBoundary>), /* existing variants */ }
// queue_api: the existing generic insertion binds this preparation key to the
// just-inserted witnessed consumer cell; prepared.rs consumes the exact
// inserted_consumer receipt, validates epoch/root/generational cell/Guest kind,
// installs the boundary Arc in that cell, and the completion reads only the
// resuming queue's first generational cell (exact epoch, request-control Arc,
// logical generation, record, OptionalExpr policy) and publishes actual None
// once. No phase/active-stack search, no guest carrier conversion.
```

The generic post-consumer boundary remains installed: it observes genuine
producer errors when `outcome?` skips the success hook and no-ops after the
successful completion. Repeated producer pauses retain the same record and
complete it exactly once. The dead P0496 branch is unchanged (source
invariant; never exercised with a synthetic plan, carrier or result).

## Deliverable

**Stage A — copied proof and permanent-test calibration.** On a byte copy of
the live tree plus exactly the six-file shape, with the C9 campaign harness
used copy-only:

- Reproduce the accepted matrix: all 16 C9 records resumed==queued
  whole-record with their language; exactly the nine witnesses differ from the
  run-1 defect (the omission mutant reproduces run 1 exactly); the seven
  controls, fixed-point 90/12/2, definition 470 (468 valid), P0248 36 against
  a fresh current-base run and native 60 exact; `extension4` changes only the
  two named B722 records, phase-confined to `call.stage.prepare_arguments`
  with queued arm and language byte-equal; 84 controls and the P1470 20 /
  P1471 14 omissions exact to C9.
- Reproduce the five falsifiers from the accepted calibration (omit, before,
  after, pause, wrong-owner) with armed/hit/caught, the actual event order
  (`installed < publication < dispatch`), the wrong-owner record identity and
  the two-real-pause/one-completion trace. The omission must omit actual
  publication work.
- Reproduce the 12 lifecycle routes and the moved/stale generation
  diagnostics; the extra queued-counter self-check stays red and unchanged on
  base and candidate. No faults, open records or charged bytes.
- Compile the permanent regression test in
  `profiling/tests/prepared_stage.rs` from the live tree and calibrate it
  **base-red / candidate-green / omission-red at 8 and 1 threads** using
  existing accessors only and no probe seams. It must exercise a real paused
  preparation (`Input`) whose later consumer Returns/Throws, compare the
  `Runtime { phase: "call.stage.prepare_arguments", source: None }` row
  resumed against queued whole-row plus language, and cover the repeated-pause
  completion-once case. If the test-builtin form cannot be calibrated
  base-red, use the design's real Accumulate recipe through the stdlib
  suspension-oracle harness and disclose the substitution.
- Source audit: ordinary and unreachable dispatch bodies, both publication
  bodies, the driver and the public trait byte-identical; no new `unsafe`.

**Stage B — live install and gates.** Install the accepted shape and the
permanent test; then replay the standing inventories as the last edit step
before the first acceptance suite, and run:

- `cargo fmt --check`; focused preparation/logical-profile/typed-slot/
  frame-root/retention/ownership tests; `cargo test -p rum-runtime -j8`;
  relevant rum-stdlib default and `--no-default-features` suites; one
  `cargo test --workspace -j8 --no-fail-fast` at 8 threads with the standing
  skips. Report exact counts and the delta against base; reproduce a genuine
  red at `--test-threads=1` before calling it a defect. The two standing
  root-context reds stay a separate obligation.
- The disabled-instruction gate on the accepted protocol: release `rum-cli`
  for base and candidate at one stable absolute path, one codegen unit,
  offline locked Rust 1.85.0, all `RUM_PROFILE_*` switches off; fresh
  Callgrind captures on the three panels (arguments, fixedpoint, definition)
  with two draws per arm plus one forced same-source rebuild; the required
  caller and neighbor rows exact at unchanged call counts; no new helper
  symbol in any disabled capture; no explicit exclusion. The three
  `rum-core` interner functions (`Symbol::system`, `intern_symbol_text`,
  `intern_text`) are the recorded hash-seed class: report them with every
  draw, the same-binary spread and the arm ranges; no exclusion, no
  universal no-cost claim. A row that separates beyond the same-binary
  spread is a STOP.
- The rum-vs-rum microbench A/B (profiling off, serial and alone, four
  interleaved passes, unchanged labels, same-tier neighbors, tier medians and
  the layout envelope; labels that separate settle by instruction probes).
- Stop for the director gate with the full receipts.

## rum / Rust pitfalls

- The completion sequence is exactly `outcome?` → install/normalize → freeze
  restoration → complete → dispatch. An edit that changes *whether or when*
  anything publishes is a STOP, never a cost fix.
- No new disabled-path flag, `Mutex`, allocation, `Arc` clone or diagnostic
  lookup in the ordinary path; the enabled-only work rides the static
  separation. No duplicated evaluator body, no silent-drop helper, no lazy
  initialization, no `inline` keyed on debug assertions.
- Selection is by exact queue ownership only — request-control `Arc`, logical
  generation, epoch, record. Never a phase-name or active-stack scan.
- Preserve the suspend/resume continuation shape, native-stack polling, all
  argument normalization/restoration and real producer events. Real producer
  Return/Throw and InputRequired keep their existing paths.
- One Cargo command at a time at `-j8`; tests at `RUST_TEST_THREADS=8`; builds
  finish before their binaries launch; Callgrind/timing runs serial and alone
  after the idle check. Persist raw output before parsing; derive bounds from
  saved receipts; never rerun a consumed reference stage.
- Stop-loss is keyed on progress: two consecutive concrete families with no
  change to the frozen counters stop and report.

## Verification

The mechanical acceptance is the named tests and measured gates, not prose.
Every claim carries a raw log path, a source/fixture/binary hash and whole
records; failed and invalid attempts are retained with their reasons; no
receipt's meaning changes. C2 and S10 cannot close on this brief.

## Report

`REPORT.md` plus the `rum-report/1` envelope at the brief's `.temp/` root:
scope diff in full, complete ripple, the whole-record matrix and language,
falsifier calibration lines, inventory replay verdicts and changed rows, the
disabled and timing receipts, test counts and deltas, every invalid/failed
attempt, and the exact residual obligations.

## Director ruling (2026-11-11) — run 1: the installed shape stands, the typed-slot STOP is well taken, and continuation 1 integrates the one new receipt

Run 1 (log `.temp/logs/cdx_brief_725.log`, 37,829 lines, sha256
`8e59d545…`, memory-scope peak 4.90 GB, oom 0, rc=0) installed the accepted
family2 shape and its four permanent tests into the live tree, reproduced
stage A on byte copies, replayed six of the seven standing inventory
families green, and stopped at the typed-slot family before any acceptance
suite, disabled capture or timing run. No git ran; every artifact is
retained.

**My gate, from raw evidence.** Live tree: the entry manifest holds 2,716
files with 0 missing and exactly 16 modified plus 2 added — the six-file
repair, the two frame-root files and eight canonical inventory outputs; my
own walk equals `final-live-hashes.json` on all 2,718 files. Every installed
file equals its `formatting.after` hash, the six production files in the
install copy equal the accepted `.temp/brief-724-c1/source-candidate2`
copies byte-for-byte, the difference is formatter-only, and `cargo fmt
--check` is clean. The four permanent tests are green on the live source at
8 and 1 threads (my runs); base is red 0/4 at both with the exact
resumed/queued row difference (`[3,2,1,0,2,0,1,0,0,0]` against
`[3,3,0,0,3,0,0,0,0,0]`), and the omission arm is red 0/4 at both. All seven
C2 family record sets (arguments 16, all 90, min 12, control 470 with 468
valid, fresh 2, nested 1, extensions 4 with the two B722 duplicates) and the
fixed-point (104/104/104), native (60), required (1) and P0248 (36) record
sets are byte-identical to the accepted continuation-1 records after
symbol-id normalization; the arguments family has all 16 resumed == queued,
exactly the nine witnesses change against run 1, and the omission mutant
restores run 1's 16 records whole-record. The five falsifiers' raw order:
installed < publication < dispatch in the instrumented arm; the before
mutant prints INSTALLED after publication and the after mutant after
dispatch (terminal records equal, caught by the order); omit 9/16; pause
16/16 with two real pauses, one retained record and one completion; the
wrong owner closes `RecordId(200)` and leaves the expected `RecordId(230)`
open under the same epoch, control and generation. Inventories: one new
type in both frame-root lists with count 568; 68 retention fingerprint
cells only; exactly the two promoted certificate cells; rejection census
1,220 = 14/1,168/38 with zero added or removed; ownership 74 MOVE + 8 NEW,
no removal, status or disposition change, every new row in touched code;
LP01 three producer position moves and three loss positions, identities and
dispositions unchanged. Protected scenes: 213,605 files, none newer than
the run start, and the full hash replay reports zero changed. Source: no
`unsafe`, no `Cargo.lock`, no docs; the ordinary and unreachable dispatch
bodies, both publication bodies, the driver and the public trait are
outside the diff. The stage-A binaries and records carry their build
receipts; no consumed reference stage was rerun.

**The STOP is well taken and reproducible.** On the live tree at 8 and at 1
thread the census prints `production frames=461 finalizers=50
implementations=611 typed=100`, test-only unchanged, `PARTITION
implementations=627 macros=546`, and fails the unchanged pin `production
610/535 and test-only 16/11; combined 626/546` inside `receipts.rs`
(`typed_slot_census.rs:113`). The discovered delta is exactly one identity,
the approved `ProfiledPreparedDispatchFrame` (production, TypedFrame,
`PreparedCallArguments -> Expr`); all 626 existing identities are preserved
and the rejection census is unchanged. The typed-slot table is an
earned-receipt inventory, not a derived output, so this brief's "canonical
regeneration, never hand-edited" clause cannot cover a new staged receipt;
the worker did not edit it, did not weaken an assertion, and did not mark
the new type covered.

**G15C2-TYPED is authorized as a narrow integration.** Add exactly one F
receipt for the new frame under its own stage identity (`G15-C2`), status
`typed`, population `production`, anchored to `F0275P`; add the stage row
and a stage transition check in the same shape as `g15a.rs` and
`g15c1.rs` — exactly one owned F receipt for the stage, status typed,
production population, kind `TypedFrame`, `PreparedCallArguments -> Expr`,
and no insertion claim, because the frame is constructed by the cold
helper and has no typed macro site — and register that check in the
baseline transition sequence. Update every exact guard that pins the
affected totals, and let a fresh discovery find any I have not named:
`receipts.rs` (production cohort 610 -> 611, the F row count 629 -> 630
and its message), `g15a.rs` and `g15c1.rs` (typed 99 -> 100 and their
printed totals) and `typed_slot_census.rs` (the two forced-red markers
626 -> 627 and 625 -> 626). No existing receipt row, stage, status,
identity, anchor, pin or assertion may change; the discovery must read
existing=626 preserved=626 missing=0 new=1, and no type may be left
silently uncovered. The installed `profile_queue/prepared.rs` header still
says "Copy-only candidate" and must state the live duty.

**Then finish stage B as briefed**, on the live tree: after the last edit,
replay all seven inventory families canonically again; `cargo fmt --check`;
the focused, runtime, stdlib and workspace suites at the briefed thread
counts with exact counts and deltas; the disabled Callgrind gate on the
three panels (two draws per arm plus one forced same-source rebuild, the
required rows and call counts exact, no new helper symbol in any disabled
capture, no exclusion; the three `rum-core` interner rows reported with
every draw, the same-binary spread and the arm ranges, a separation beyond
the same-binary spread a STOP); and the rum-vs-rum microbench A/B. Stop for
my gate with the full receipts.

Recorded, not gated: the root-replay reducer's first attempt read a
libtest-prefixed count line and was replaced by the saved-line reducer
(count 568, verify 15/0); the extra queued-counter self-check is red on
base and candidate alike; the three retained invalid recipes and the
discovery/emission ordering reds are the established artifacts. The
acceptance, disabled and timing gates are unrun; C2 stays Open at 328
residuals + 1 Lost, S10 stays unopened, and the two root-context reds stay
separate obligations.

Next: brief 725 continuation 1, forked from session
`01a1c7dd-9c67-77d1-bc06-560621b37145`, completes the typed integration,
replays the seven families, and runs the remainder of stage B; then my
gate. The install stays uncommitted until stage B is green.
