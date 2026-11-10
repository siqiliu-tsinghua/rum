# Brief 721 — G15-C2 definition-resume repair, implementation

**Status:** implementation brief. It authorizes a live source change on the
exact scope below, one gated step, and no git. It is the implementation of
brief 720 continuation 2, accepted in the director review of 2026-11-10.
**Worker:** cdx (gpt-6-astra), reasoning effort max.

## Goal

Install the definition-resume repair on the live tree so that a Definition
record opened inside a resumed down-, sub- or up-value RHS is published at
its logical completion — after its RHS recovery is installed and before the
carried suffix is joined — and the two declining-stage `false_guard`
records are sampled before their consumer. On the final tree the 21
minimized witnesses and the 470 frozen campaign records must be whole-record
equal to the queued arm (`native_same=true`, 0 DIFF), all ten counters
included, while the disabled request caller
`resume_eval_request_from_checkpoint` stays at exactly 81,588 instructions
/ 30 calls in three serial Callgrind draws, and executes no new
instruction anywhere else on the default path.

## Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`, and brief 720 whole,
  including both director reviews:
  `docs/dev/codex/brief-720-g15c2-definition-resume-boundaries-design.md`.
- Accepted route and evidence: `.temp/brief-720/c2/` —
  `route-design.md`, `draft-publication-route-addendum.md`,
  `diffs/publication-final.patch`, `diffs/sampling-final.patch`,
  `diffs/index.json`, `cost-*-comparison.json`, `*-records.json`,
  `new-edge-reach.json`, `reports`, `logs/`, and `source-sample/` (the
  accepted candidate tree) vs `source-base/` (its base).
- The frozen scene: `.temp/g15-c2/c7/` (25,251 entries; never overwrite).
  Frozen plans and witnesses: `.temp/brief-720/fixtures/`.
- Reference records: `.temp/brief-720/{base,publish,sample,probe}-{min,all}-records.json`,
  `{base,sample}-fresh2-records.json`, `indexed-native-{base,publish,sample}.json`,
  `mutant-*-records.json`.
- The scratch native fixture that emits `C2_NATIVE` rows lives only in the
  copied arms (`.temp/brief-720/source-base/crates/rum-stdlib/tests/suspension_oracle/fixtures/logical_profile/c2_native.rs`
  plus its `descriptors.rs`/`mod.rs`/`plans.rs` wiring). It is evidence
  scaffolding, not an installed test.

## Scope

Production files, exactly four, installed in two stages:

1. `crates/rum-runtime/src/runtime/profiling/logical/completion.rs` —
   extract the charged one-record constructor
   `pub(in crate::runtime) fn take_deferred_definition_boundary(epoch, control) -> Option<Arc<ProfileBoundary>>`;
   the existing Session publisher drains through it with its loop body
   unchanged.
2. `crates/rum-runtime/src/runtime/suspension/core/profile_queue.rs` —
   `fn publish_deferred_definitions_before_join(&mut self) -> bool`, called
   from `append` after the arena/epoch identity and validity checks and
   before the root locks.
3. `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` — mechanism B:
   the phase set gains `CallStageSubValues` and `CallStageUpValues`, and the
   call uses the renamed queue API.
4. `crates/rum-runtime/src/runtime/profiling/logical/queue_api.rs` — rename
   `logical_insert_down_values_before_consumer` to
   `logical_insert_definition_stage_before_consumer`, doc updated.

Permanent tests, exactly two files:

5. `crates/rum-runtime/src/runtime/profiling/tests/retained_pause.rs` — the
   three-family foreign-deferred regression
   `brief720_queue_join_preserves_foreign_deferred_definition` from the
   accepted source.
6. `crates/rum-runtime/src/runtime/profiling/tests/default_fallback.rs` —
   the 18-cell lifecycle regression
   `brief720_definition_resume_lifecycle_probe` from the accepted source,
   with its "scratch-only / not installed" header rewritten for the live
   tree.

Generated inventories, canonical regeneration only, never hand-edited:

- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv`,
  `unknown_captures.tsv`, `unknown_macros.tsv`, `unknown_storage.tsv`
- `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv`,
  `promoted.tsv`

Everything else is a STOP. Not authorized: the `c2_probe` module, the
`c2_native` fixture wiring, public API or dependency changes, `unsafe`, any
new flag, any wrapper/attribute/const change in the driver, duplicated
resume bodies, inventory hand edits, test weakening, skip additions.

## The accepted artifacts (identity, not homage)

The four installed production files must hash exactly to the accepted
copies in `.temp/brief-720/c2/` (`diffs/index.json`):

- `completion.rs` → `e4652c8e39cbda8bd6d330d1a31322e465a815232cc0619495ef103167cb0214`
- `profile_queue.rs` → `90f8da7caf175a79454922b6771f972ba3983103fe383ce852294177eeac9c19`
- `mod.rs` (after stage S) → `205acce85107f7ed9c05c82dd5d5ebe4704086af9c6509477cde3a9b26c20e2a`
- `queue_api.rs` (after stage S) → `09c85979f3ec9d909883b8306878fcef9fb168b4c8fc984a42c5fe737e185324`

If the copy differs from the accepted sha, STOP and report the diff; a
"better" variant is a new design round, not this brief.

## Invariants (the shape)

- The disabled path executes nothing new: the publication lives only in
  the profiled append edge (`append_profiled` stays `#[cold]
  #[inline(never)]`; guest/guest keeps its direct append); no flag load,
  allocation or check on the ordinary path; the profiled pause symbol
  stays untouched.
- The enabled insertion is the queue's own shape: `Context::capture`, a
  charged `Kind::Boundary` cell built by the shared constructor,
  `deferred=false`, inserted at the nested frontier after the RHS recovery
  and before the root join.
- The owner lookup matches root and epoch in the existing registry, drains
  only that control and generation, manufactures nothing when ownership is
  missing, and leaves foreign records deferred. No registry lock is held
  across record construction or insertion.
- The shared constructor has exactly two callers (the Session publisher and
  the queue join). The driver, the mode trait, storage dispatch, the three
  typed definition frontiers and Return recovery stay byte-identical.
- Guest results, evaluation order and count, effects, traces, requests,
  cancellation/abandonment, carriers and public APIs are unchanged.
- Mechanism B is byte-identical to the accepted sampling source.

## Verification (you run it; quote every verdict line)

**1. Base red on a copy.** From a byte copy of the live tree with the
frozen fixture wiring, reproduce: 21 minimized = 8 DIFF / 13 SAME; 470
frozen = 147 valid DIFF, 468 valid, 2 invalid retained; the three Return
witnesses `[1,0,1,0,0,1,0,0,0,0]`. The deliberately wrong terminal-open
field must be rejected by the same whole-record comparator. A base
mismatch is STOP.

**2. Stage P — publication and Return (files 1, 2, 5, 6).** Install, then
prove on a copy of the installed tree:

- 21 minimized: exactly 2 DIFF, `C7.loss.P0312.false_guard` and
  `C7.loss.P0323.false_guard`; 470 frozen: 0 valid DIFF.
- the three Return witnesses → `[1,1,0,1,0,0,0,0,0,0]`; the 13 original
  controls unchanged; the 9 fresh and 1 indexed witnesses language-exact.
- omit-publication → exactly 147 changed and DIFF ids; before-recovery →
  125 including the three Return witnesses at the bad vector; both with
  the actual new-edge hit quoted. The reach proof: all 147 bound ids have
  at least one queue-join publication at the typed frontier and a nested
  pause in the raw log.
- the foreign-deferred regression green for all three families; the 18
  lifecycle cells green; the existing profile-queue cap, generation and
  order tests green.
- cost: caller 81,588 / 30 in three serial draws, all 60 required rows
  point-equal at identical calls, no deterministic equal-call increase in
  any `rum_runtime` row; the forced same-source rebuild control; the
  indexed cohort 86,950 / 32.

**3. Stage S — sampling (files 3, 4).** Install, then prove:

- 21 minimized → 0 DIFF; 470 → 0; the two omission mutants → exactly one
  difference each (`P0312.false_guard`, `P0323.false_guard`); the 13
  controls, 9 fresh (base 8 → candidate 0) and the indexed witness (base 1
  → 0) retained.
- the installed sampling files hash-match the accepted shas above.
- cost: caller 81,588 / 30 in three draws on the final candidate, all 60
  rows, indexed 86,950 / 32.

**4. Combined acceptance on the final installed tree.**

- `cargo fmt --check` clean; `cargo check --all-targets` rc=0; no new
  warnings (default lib 0; lib-test only the three standing
  `test_builtins` names).
- focused `profiling::` suite at 8 and 1 threads: base 219/2/2 → candidate
  221/2/2, with `lp655_root_context_falsifier` and
  `lp656_public_root_context_falsifier` the only failures (the standing
  root reds; assertions unchanged).
- `cargo test -p rum-runtime --lib` at 8 and 1: 1418/0/5 with the two
  standing `--skip` names; stdlib `1111/0/53`; the six standing
  switch-gated host targets at 8 and 1 under
  `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`.
- `cargo test --workspace` once at 8 threads with the seven standing
  skips: expected 4,217/0/61 (the accepted 4,215 plus the two new tests).
- the seven standing replays after the final source edit and before the
  first acceptance suite: 68 contract fingerprints regenerated
  (canonical), ownership 61 / 79 / 8,828, both certificate cells GREEN,
  rejection census 1,220 (14 / 1,168 / 38), LP01 five modes 88,795
  records with zero semantic differences, typed slots 10/0/1, frame
  roots 15/0. Quote every
  verdict; the six TSVs are generated, never hand-edited; a stale
  fingerprint/certificate/position cell is an ordering artifact, an
  identity or disposition change is a STOP.
- the timing lane: `scripts/core-language-microbench.sh` base vs candidate
  with profiling off, four passes, a layout control, tier-median quotients
  on the target labels and same-tier neighbors, reported in full. The
  instruction lane decides a regression; a timing-only movement is
  recorded, not a STOP.
- scope: the diff touches only the six authorized files plus the six
  generated inventories; the live tree is otherwise unchanged; report
  `scripts/report.py check 721` without `--against-git`.

## Checkpointing and the stop rule

Work under `.temp/brief-721/`; preserve the whole 720 scene (`.temp/brief-720/`,
`.temp/g15-c2/c7/`) byte-for-byte; never overwrite a consumed reference.
Save raw logs, receipts, records and captures before parsing. Two
consecutive executed bounded families with no new evidence stop the round;
source reviews and failed launches do not count. If any stage does not
reproduce the accepted table, STOP with the raw files and do not adjust a
witness, filter or expectation to pass. Stop for my gate with the live
tree left uncommitted; do not commit, branch or push.

## Hard rules

No git in any form, no network, no `unsafe`, no dependency or public-carrier
change, no foreign-engine comparisons, no inventory hand edits, no weakened
assertion, no new skip, no default-path cost. One cargo process at a time,
Rust 1.85.0, offline and locked, eight jobs; diagnostics and Callgrind
serial and alone; every command bounded; no competing load. Preserve every
C7 and brief-720 artifact.

## Report format

`rum-report/1` under `.temp/brief-721/`, checked with
`scripts/report.py check 721` (no `--against-git`). Report the base-red,
Stage-P, Stage-S and combined gates separately, with every quoted verdict,
the full 21/470 tables, the four mutants, the reach proof, the three-draw
cost tables (caller row and all 60 rows), the indexed cohort, the forced
rebuild receipt, the seven replay verdicts, the focused/runtime/stdlib/
workspace/host results, the microbench rows, the scope diff and every raw
artifact path. The director owns the authoritative gate and git.

## Director gate (2026-11-10) — implementation accepted: the definition-resume repair is installed, and the STOP was the sandbox boundary

**What I reproduced myself.** The four installed production files hash
exactly to the accepted copies (`completion.rs` `e4652c8e…`,
`profile_queue.rs` `90f8da7c…`, `mod.rs` `205acce8…`, `queue_api.rs`
`09c85979…`), and the diff is exactly those four plus the two
permanent-test files (pure appends; the bodies are byte-identical to the
accepted scratch tests, the lifecycle header rewritten as authorized)
and the six generated inventories. My own tree walk against the
worker's entry manifest (2,711 files, whose 12 pre-change hashes equal
`HEAD`) finds exactly those 12 changed, 0 missing, 0 extra; the 720
scene (86,758 files) and the C7 scene (25,251 files) are byte-,
size- and mtime-identical with no link change.

From the 24 raw Callgrind captures I recomputed every cost row: the
explicit caller `resume_eval_request_from_checkpoint` is
**81,588 instructions / 30 calls in all three draws** on base, the
forced same-source rebuild, publication and sampling, and
**86,950 / 32** on the four indexed arms; all 60 required rows are
point-equal across arms and my parse matches the submitted tables on
every cell (0 mismatches); my sweep of every `rum_runtime` row outside
the 60 finds no deterministic equal-call difference in any arm. The
two new symbols do not appear in the disabled captures at all. From
the raw record JSONs I recomputed the matrix: base 21 minimized
**8 DIFF / 13 SAME**, stage P exactly the two declining-stage
`false_guard` rows, stage S 0; base 470 frozen **147 valid DIFF** /
468 valid / 2 invalid retained, stage P 0 and stage S 0. The
P and S records are byte-identical to the continuation-2 accepted
references, and so is the base. Mutants: omit-publication 147,
before-recovery 125, omit-sub 1, omit-up 1, and in each the changed-id
set equals its DIFF set exactly; the three Return witnesses read
`[1,0,1,0,0,1,0,0,0,0]` under misplacement and
`[1,1,0,1,0,0,0,0,0,0]` repaired, and the wrong-terminal-open negative
control is rejected by the same whole-record comparator. The reach
file is exactly the 147 base-DIFF ids, each with a queue-join
publication, its typed frontier and a nested pause in the raw probe
log, and the full-table artifact is a faithful projection of the raw
records (491 rows × 3 arms, 0 mismatches). The shared constructor has
exactly two call sites. `report.py check 721` verifies 5,221
citations, and the seven sealed deliverables hash-match the final
receipt.

**The STOP is an environment boundary, not a product signal.** The
worker's single workspace run read 4,208/9/61: exactly the nine native
ZeroMQ tests panicking at
`native ZeroMQ test cannot bind loopback: Operation not permitted`
(`rum-jupyter` lib, 143 passed beside them). That is the standing
worker-sandbox boundary every implementation round has hit; the nine
tests bind loopback on the host. My own host run under the same seven
standing skips is **4,217 passed / 0 failed / 61 ignored over 63
binaries**, exit 0, peak 6.80 GB in a 14G scope — the brief's table
exactly. The worker correctly added no skip, no filter and no rerun.

**My host acceptance, same tree.** `cargo fmt --check` clean;
`cargo check --workspace --all-targets` rc=0 with only the three
standing `test_builtins` lib-test warnings and none in the default
lib; focused `profiling::` 8 and 1 threads **221/2/2**, the two
failures exactly `lp655_root_context_falsifier` and
`lp656_public_root_context_falsifier`, assertions untouched; runtime
lib 8 and 1 threads **1418/0/5** with the two standing skips; stdlib
**1111/0/53**; typed slots **10/0/1**; frame roots **15/0**; the six
switch-gated host targets at 8 and 1 threads under
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1`, 13/0/0 each. The seven standing
replays, run through the canonical generators at this gate, are green
and rewrite nothing: ownership 61 / 79 / 8,828 with zero replay rounds;
census 1,220 = 14 / 1,168 / 38 with all 1,220 rows paired and none
retired, new, base-only or candidate-only; 68 contract fingerprints
with 0 changed; both certificate builds GREEN with 0 promoted rows
merged; the inventories run 19 passed / 0 failed at the same frontier;
typed slots 10/0/1, frame roots 15/0 and the LP01 five-mode parent
(88,795 records asserted, 0 semantic differences) green in the same
runs. The six installed TSVs hash exactly as before the replays.

**The timing lane, run at the gate.** The worker's STOP ended the round
before the timing lane, so I ran the brief-637/703 protocol myself on
this host: base, candidate and layout-control release `rum-cli`
binaries at one path with one codegen unit, four interleaved passes per
arm, 120 labels, `AbsoluteTiming` in-kernel, profiling off. The four
tier medians (candidate / layout control) are 0.9959 / 0.9945 under
50 µs, 0.9996 / 1.0000 at 50–200 µs, 1.0006 / 0.9994 at 200 µs–1 ms
and 0.9993 / 1.0006 over 1 ms; the layout envelope is median 0.29%,
p90 1.28%, max 13.04% |movement|. The 703 reducer flags the two named
calibration labels (−1.3%, −1.1%) and three separated labels —
`symbol-ownvalue-guarded-read` +2.3%, `table-iterator-null` +4.9%,
`table-count-null` +11.6% — none outside the envelope. I ran the
703-style instruction probes on those five exact benchmark bodies
(n = 50/100, four serial draws per cell, three arms, profiling off):
the per-iteration executed-instruction distributions overlap and no
arm is separated in any label — `table-count-null` 2,004,724 /
2,004,740 / 2,004,758 (medians base/final/layout out of ~2.0 M per
iteration), `table-iterator-null` 429,761 / 429,750 / 429,799,
`symbol-ownvalue-guarded-read` 3,541 / 3,535 / 3,527 (within-arm
spreads 70–155), `downvalue-guarded-decline-fallback` 20,796 / 20,785
/ 20,791, `stringreplace-single-literal-unguarded` 37,112 / 37,059 /
37,105. The instruction lane decides, and it reads identical; the
timing movements are recorded as noise, not a regression. (The lane's
idle guard was adapted for this host: the director harness process is
excluded by name and every other process must stay under 0.5 s of CPU
per second, so a real build or test still trips it by an order of
magnitude.)

**Recorded, not gated.** (1) The worker's workspace red is the
sandbox's loopback denial; no skip was added and the opt-in was not
used. (2) The certificate-emission runs red on the stale promoted
cells and the canonical merge settles them — an ordering artifact of a
derived cell moving with the source, as in every source-change round.
(3) The typed-slot compiler-control and certificate-emission discovery
attempts that fail by design are preserved as raw evidence. (4) The
timing lane is non-decisive by the brief's own rule; the instruction
lane decides and reads identical, and the per-label probes settle every
flagged label without separation.

**Ruled.** The implementation is accepted: the installed tree is
byte-identical to the accepted route, the disabled caller and the
whole 60-row table are exactly where continuation 2 left them, and the
21/470 matrix, the mutants, the reach proof and the permanent
regressions all reproduce. The continuation-2 route stands as the
repair; the two dead ends stay dead. Next: continuation 8 of brief 623
re-settles P0267/P0312/P0323 and carries the 270 LocalTail rows, the
45 reopened rows, the 22 strict rows and the five 719 edges to the
gate.
