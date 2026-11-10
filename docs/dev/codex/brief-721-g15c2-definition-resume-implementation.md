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
