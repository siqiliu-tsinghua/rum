# Brief 793 — install the rendered inventories and run the complete gate

## Goal

Produce the four authenticated candidate trees and the evidence that all six named retention
remnants pass on the armed tree, while the complete flag-off workspace exposes exactly the ten
accepted reds and the same 71 ignored tests. The producer reviews, source authentication, cold
builds, F48, the focused gate, formatting, the P0543 lane and the flag-on controls all belong to
this dispatch. The director's live render commit (R4) consumes the handoff material; no live,
git or network operation belongs here.

## Context pointers

- Read `AGENTS.md`; brief 790 including Amendment 1; ruling 799 (`.temp/director/ruling-799.md`);
  brief 791 and its checked report (the four postimages); brief 792 and its calibrated tools;
  `.temp/brief-790/emission-plan.md` (every table; the overlay set is 165 rows under ruling 799).
- Historical runners and command vectors: `.temp/director/impl-789/{stage_c_writers.py,
  lane_build.py, run_f48_tests.py, run_14_tests.py, run_stage_d.py, gate/}`; the build/cleanup
  conventions at `.temp/brief-789/scripts/run_cargo.py:114–125,152–185`; the decisions at
  `.temp/director/impl-789-decisions.md:62–106,124–141,163–186` (F53, F54, F55, F58 and the final
  source scan).
- The evidence: `.temp/brief-790/{panics.log,six-reds.tsv,six-case-minima.tsv,receipts.tsv,
  ignored-set.tsv,stage-d-reds.tsv,class-absences.tsv,lp01-producers.tsv,typed-slot-reports.tsv,
  standing-inputs.tsv,carried-artifacts.tsv,cell-adoption.tsv,overlay_digest.py}`; the 762 merge
  controls at `.temp/brief-790/merge-controls.log`; the 791 manifest at `.temp/brief-791/manifest.tsv`;
  the 792 tools at `.temp/brief-792/tools/`.
- The pinned inputs: `.temp/brief-784/images/{baseline,armed,disabled,real}` (5,842 + 5,851 × 3),
  baseline `dd7c6539…`, armed `d7640402…`, disabled `74d9c345…`, real `8c45350b…`, debris 0.
- The scene's `promoted.tsv` already carries the rendered anchors (`core.rs:214:5-230:6`,
  `396:5-410:6`); the live tree still carries `106:5-122:6`/`224:5-238:6` and is not this
  dispatch's business.

## Scope

- Create fresh working copies under `.temp/brief-793/images/{baseline,armed,disabled,real}` from
  the 784 inputs. Write all attempt logs, review archives, temporary files, isolated targets, the
  P0543 lane, binaries, receipts and the final manifest under `.temp/brief-793/`.
- For the stable probe runner use its existing exclusive lock and `ACTIVE` workspace under
  `.temp/probe/` (`scripts/probe-build.py:45–54`); preserve a pre-existing probe session; use
  attempt-owned output/cache locations where the protocol allows; never resume a 789 attempt or
  use a primary binary from its cache.
- Install on **both arms** the four 791 postimages (`frame_root_trace.rs`,
  `frame_root_trace/derived_inventory.rs`, `retention_closure/census/tests.rs`,
  `crates/rum-stdlib/Cargo.toml`) and, after the replays, the generated
  `crates/rum-stdlib/src/tests/retention_closure/inventory/{local-contracts,promoted}.tsv`. All
  four copies receive the identical emitted
  `crates/rum-stdlib/tests/suspension_oracle/mutations/source_auth/overlays.tsv` (165 rows) and
  `crates/rum-stdlib/tests/suspension_oracle/inventory/obligations/mutants.tsv` (151 rows).
- Baseline and real keep the three Rust preimages and the manifest preimage `6dcfaa9f…`; real is
  the zero-overlay receipt control; baseline keeps its F54 preimages and cannot compile `--lib`.
  Existing images, historical inputs, the live WIP, and all production Rust outside the four
  declared postimages stay untouched. No network, no git, no unsafe, no benchmark changes.

## Deliverable

An authenticated four-role release manifest (before/after hashes and file counts), full replay
transcripts, the four arm/build reviews and their two complete promotion postimages, the sealed
165-row overlay table and 151-row mutants table with the measured SHA-256, the rebuilt P0543
lane, six cold primary build receipts, the twelve source-auth receipts, both disabled-restore
records, the complete acceptance report, and per-role snapshots of the four postimages and the
exact generated role bytes for R4.

## Verification

1. **Entry and compiler preflight.** Re-hash the 784 inputs and the fresh copies with
   `.temp/brief-784/tools/common.py:77–105`; all four rows of `scene-pins.tsv` must match. Verify
   the four 791 postimages (preimage and postimage SHA-256 and byte counts) and the 792
   calibration. Before editing any Rust, run on the candidate armed checkout:
   `cargo test --offline --locked -j8 -p rum-stdlib --lib --no-default-features --no-run`.
   Under Q-build this must **succeed** — the manifest postimage is the fix (the director watched
   the pre-fix `error[E0432]: unresolved import sha2` at `ownership_graph/reconcile.rs:10:5` and
   the post-fix success with the identical command). Archive the compiler log. A failure is a
   STOP; the `--features sha2/compress` fallback is **not** authorized. Use the identical
   toolchain, target, debug profile and flags for the two certificate rounds; never import a live
   or historical cell.
2. **Substitutions.** Install the four 791 postimages on each arm. Require each full postimage
   hash and exact preimage; the pure/stateful lists 514/54 with 568 names; the
   derived-FrameRootTrace array byte-identical and the 568-count assertion intact; the only census
   literal edit at `census/tests.rs:188` (`233` → `405`); the keyed census pair
   `405:58–405:68` (armed) vs `233:58–233:68` (live), RET002/`options/core.rs`/callee `-`/the full
   method-syntax reason. The manifest is the fifth overlay path with identical arm bytes. Record
   the changed-file manifest before any test suite.
3. **Standing replays** (F58 order, serially, one checkout at a time; keep each generated file
   outside the transient in-tree review directory before cleanup):
   - **F13 per edited role** (armed, disabled):
     `cargo test --offline -p rum-stdlib --lib tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints -- --ignored --exact --nocapture`.
     Compare the 68 rows and require the first 18 cells unchanged; the fingerprints themselves
     change wholesale because the manifest and the three Rust files are in the F13 dependency
     context — that is the expected replay, not drift. Atomically install this checkout's
     `target/brief-544/local-contracts.tsv` at the declared path, set its mtime to the adoption
     time (F55), re-read. Then replay
     `tests::retention_closure::local::contract_tests::local_concrete_contract_controls -- --exact --nocapture`
     and require its 1/0 line and every recorded falsifier. 400 s bounds
     (`f13_refresh.py:37–66`). Record the actual resulting role SHA-256 (it supersedes the design's
     predicted `a444b850…`/`7c9c615a…`).
   - **Ownership replay** (armed is the authored role): the four
     `tests::ownership_graph::reconcile::*` maintenance selectors
     (`regenerate_self_reaching_inventory`, `regenerate_host_payload_inventory`,
     `regenerate_unknown_construct_inventory`, `regenerate_host_payload_ledger_review`) with
     `--ignored --exact --nocapture`; archive the seven review copies in `target/brief-775/`;
     compare the seven digests with `standing-inputs.tsv`, 61 self-reaching, 92 hosts, 10,135
     constructs and 92 ledger data rows; recompute all 152 chain pins against the final armed
     files and compare the chain on armed/disabled/real. The seven tables and the chain are one
     armed-authored inventory installed as common bytes on all roles: on disabled and real the
     installed tables and `emission_chain.tsv` must be byte-identical to armed's
     (`unknown_macros.tsv` `843215…`, chain `0e508bb4…`). A disabled regeneration is a
     diagnostic, never installed and never a bar: it must differ from the armed review only in
     `unknown_macros.tsv`, where exactly the two `cfg!` rows at `lib.rs:27:32` and `lib.rs:27:51`
     are absent (3,725 macros / 10,133 constructs against armed's 3,727/10,135), the declared
     input-arm difference (`ENABLED = !cfg!(disabled) && !cfg!(live)` vs `false`). The ignored
     legacy adoption reads a different chain reference and is outside F48.
   - **Class and producers**: `oracle::obligations::tests::obligations_discovery_campaign` in the
     `suspension_oracle` integration target with `--ignored --exact --nocapture`; capture every
     20-field record; require the 2,422 class digests on both arms, the exact eight absent keys
     (`class-absences.tsv`) and 20,625 class rows; the eight LP01 producer paths and all 875
     bindings (`lp01-producers.tsv`); recompute the 875 active cell-19 values from the eight
     sources; the frozen `fixtures/logical_profile/fingerprints.tsv` (796 rows,
     `e9203ee2…`); one `oracle::fixtures::logical_profile::logical_counter_port` run on its
     recorded profiled-child protocol; all negative controls.
   - **Typed slots**: `cargo test --offline -p rum-stdlib --test typed_slot_census -- --exact --nocapture`
     with `source_baseline_matches_earned_receipt`; archive the fresh `.temp/typed-slot-census/`
     output; require the production tuple `461/50/611/100/615/375/240`, the plain/twin splits
     `535/336/199/48` and `80/39/41/34`, and the 10/0/1 outer verdict on both arms; compare each
     fresh raw report to its role under `typed-slot-reports.tsv` and attribute source digests and
     moved positions (the archived arms already differ in five raw files).
4. **Whole promotion inventory, last adoption.** On each edited checkout run
   `cargo test --offline --locked -p rum-stdlib --lib tests::retention_closure::local::production::local_production_acceptance -- --exact --nocapture`
   and the same command with `--no-default-features` inserted before `--`, starting from an empty
   attempt-owned certificate output directory. Archive this invocation's
   `promoted-hash-<bool>.tsv`, `acceptance-hash-<bool>.tsv` and the four
   `LeaseProbe538-{probe,start,drive,resume}-hash-<bool>.tsv` files. The discovery pass may exit at
   the active-cell comparison at `production.rs:99` after the writes at 80–89; require three
   instances and four probes with zero rejections, 2 body rows, 3 production marks and 4 probe
   marks. A missing review, compile error, earlier assertion, changed metadata or wrong role/run
   time is a STOP. Run the 792 strict merge on both fresh reviews for that checkout: column 4 from
   the `hash=true` review, column 5 from the `hash=false` review; require `UNRECORDED` in both
   inactive input cells and byte-equal metadata in 0,1,2,3,6,7,8; install both rows atomically
   with F55 mtime; rerun the identical selector in both builds and require 1/0 per build and
   byte-identical re-emission; archive the second set separately. Name each changed whole cell and
   every changed stamp by key. No manual inventory mutation or cross-role reuse.
5. **Final tables.** Run the 792 seal against the new copies; cross-check
   `overlay_digest.py --emissions <archive>` with a fresh SHA of the files on disk. Require the
   exact re-emission, **165** sorted overlay rows including the five new paths and the refreshed
   local-contracts row, the 151 mutant schema cells rebound, additions 9 unchanged
   (`243dc0cf…`), the allowance 4/4, the source bijection 3/3/3/3, the complete decoded
   baseline/disabled/armed byte closure, and the distinct real control. Archive the actual numeric
   SHA-256 (Q-seal: this is its only source) and the final four role pins. F16 removes only
   attempt-created debris.
6. **Retained tests and release gates.** Run all six selectors of `six-reds.tsv` on the final armed
   tree and the corresponding targeted checks on disabled: `--lib` on `rum-runtime` for red 1
   (`runtime::tests::frame_root_trace::suspension_protocol_repository_invariants_hold`) and
   `runtime::tests::frame_root_trace::derived_frame_root_trace_inventory_matches_the_closed_capture_graph`;
   `--lib` on `rum-stdlib` for
   `tests::retention_closure::census::tests::production_rejection_census_is_complete`,
   `tests::retention_closure::local::production::local_production_acceptance`,
   `tests::retention_closure::local::promotion_tests::production_inventory_rejects_removed_redirected_stale_and_omitted_evidence`,
   `tests::retention_closure::local::promotion_tests::production_promotion_cannot_launder_consumers_or_runtime_domain`
   and `tests::retention_closure::production_obligations_are_exact`. The census writes its dump via
   `RUM_CENSUS_DUMP` before asserting and calls `local::verify_promotions` at `census/tests.rs:418`;
   require 1,229 keyed rows, the `14/1,177/38` kind split, the exact one moved position, and the
   promotion mutation controls reaching their specific errors in both feature builds. Run the
   complete `--test typed_slot_census` target on both arms expecting 10/0/1. Baseline keeps its
   F54 preimages; real is the zero-overlay receipt control.
   - Rebuild a fresh P0543 lane with exactly nine changed manifests and otherwise identical input
     (`lane_build.py:29–99`), continuation slots and release `codegen-units=1`; capture the four
     lane env values and the tree/manifest/binary evidence.
   - Rebuild the two product configurations × real/disabled/armed with the 792 gate adaptation
     (`gate/stage_c789.py`): six cold primary binaries and the disabled restore per build for the
     swapped-arm negative; the stable `ACTIVE` tree and binary proof at every execution; validate
     all twelve named receipt files, each named negative, all 23 exact prefix minima, the six
     cases, **165** positive overlay rows and 9 additions. Enforce F48's two active selectors
     (`run_f48_tests.py:17–20`) at 2/0 plus the chain, every selector of `run_14_tests.py:20–35` at
     14/0 with the fresh lane, then `cargo fmt --all -- --check` on armed.
7. **Stage D.** On final armed run the complete flag-off
   `cargo test --offline --workspace --no-fail-fast` with the final `RUM_CM740_BASELINE` and the
   fresh lane environment, in a 20G scope. Strip inherited `RUM_PROFILE_*` and `RUM_CM740_*` first;
   set a dedicated `CARGO_TARGET_DIR`, `TMPDIR`, offline mode, eight build jobs and eight test
   threads. Compare names from raw harness lines, including ignored reasons where printed, to
   `stage-d-reds.tsv` and `ignored-set.tsv`. Require the six named remnants **green**, exactly the
   ten accepted failures — `lp658_host_resume_reset`, `lp03_adopted_phases_are_exact_in_completed_and_resumed_runs`,
   `lp658_host_eval_reset`, `lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs`,
   `lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs`,
   `runtime::profiling::logical::control_tests::lp655_root_context_falsifier`,
   `runtime::profiling::logical::control_tests::lp656_public_root_context_falsifier`,
   `class_lease_fixtures::class_lease_positive_c5_default_profiled_finish`,
   `class_lease_fixtures::class_lease_positive_p0492`, `q25_p2379_proof::q25_p2379_ownership_proof`
   — exactly the same 71 ignores, and **4,553 passed** for the same 70-result-line population. A
   complete run with zero OOM events; the raw cargo exit code is kept (the flag-off workspace is
   expected nonzero). Run the flag-on LP03/04/05 and class_lease selectors with
   `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` and require `3+1+1` / 0 and `197 passed; 0 failed`. Repeat
   fmt, clean only attempt-owned debris, and recompute all four trees against the sealed pins.
8. **R4 integration handoff.** Give the director the exact four postimages, the five rendered
   production inputs in `scene-membership.tsv`, the two role-specific generated tables, all seals
   and the measured overlay SHA. In the director's final integrated render checkout the rendered
   anchors must already be present; that checkout repeats its own F13, ownership and both
   production reviews before its first suite. No version-control or live-checkout operation in
   this dispatch.

## Checkpointing and STOP rules

Run every long command inside the enclosing dispatch scope (the dispatcher caps the worker at
22 GiB) and bound it with `scripts/run-bounded.py --timeout <seconds> --`; do **not** create a
nested `systemd-run`/`memory-scope.sh` scope — the sandbox denies the user-scope bus (gate c151's
STOP: `runs/armed/preflight/raw.log`, `Failed to connect to user scope bus`). The 792 adapter now
inherits any enclosing cgroup whose `memory.max` is at least 20 GiB and records
`# inherited-scope: unit=… max=<actual> peak=… oom=0 oom_kill=0 rc=…`; a run with no such scope
is a STOP. Keep the recorded peak and OOM counters in the report. The 789 bounds: 400 s per local
producer, 1,800 s for the lane, 12,600 s for all six-case legs with bounded children, 1,500 s for
F48, 3,000 s for the 14 tests, 5,400 s for the workspace, 900/1,200 s for the flag-on lanes,
300 s for fmt. Run heavy commands serially.
Keep the replay logs and the generated files before cleanup, twice for the two R3 rounds.
Checkpoint pins, commands, completed gates, tree state and the exact next action after each phase.

STOP on input drift, a new source difference, a no-default import failure, missing or mixed
certificate output, an unexpected rejection, an inventory discrepancy outside a ruled
substitution, a failed negative control, an unaccounted receipt row, an altered
class/chain/allowance, a wrong 14/F48/flag-on bar, an extra or missing red or ignore, a time
limit, OOM or final debris. A first-pass stale fingerprint, certificate or position cell alone is
an ordering artifact: restore the replay order, keep the first evidence, and report the fresh
replay. An unexpected semantic, compiler or evidence failure is reported at its producing step.
Every assertion and all inventory checks stay intact.

## Hard rules, disclosure and report

Only the named candidate files and work directories are writable. Keep all historical gate
scripts and archives read-only. No network, unsafe, benchmark changes, external engines or
version-control commands. Quote the actual per-role replay verdicts, the F55 adoptions, the R3
discovery/recheck logs, F48/typed-slot/LP/class results, the positive and negative source-auth
verdicts, every Stage D exception, the scope peak/OOM, fmt, and the final tree hashes. Disclose
any incomplete gate, ordering artifact and any director-authorized change. Write
`.temp/brief-793/report.json` in `rum-report/1`, run `python3 scripts/report.py check 793` and
return its summary.

## Budget

9,600 s, dispatched with `BRIEF_MEMORY_MAX=22G` (the acceptance and Stage D run inside the
dispatch's 22 GiB scope, inherited and recorded by the adapter — no nested scope; Stage D's
carried peak is 18.48 GB). A shortfall is topped up explicitly per the standing 2026-11-19 ruling.

## Amendment 1 (gate c151) — inherit the dispatch scope; resume at the preflight

The first dispatch correctly STOPped before Cargo started: the 792 scope binding demanded a
cgroup whose `memory.max` equals exactly 20 GiB and otherwise asked `scripts/memory-scope.sh` for a
nested `systemd-run --user` unit, which the sandbox denies (`runs/armed/preflight/raw.log`). The
director amended `.temp/brief-792/tools/memory.py` (a scratch tool, disclosed): it now inherits any
enclosing cgroup with `memory.max >= 20 GiB`, records the actual max together with peak/OOM
counters, and refuses anything smaller; the verifier accepts any recorded max of at least 20 GiB
with `oom=0 oom_kill=0 rc=<raw rc>`. Its unit fixtures still pass (the 16 GiB line is refused; a
22 GiB line is accepted; an OOM or a mismatched rc is refused), and all six 792 calibration
drivers were rerun green.

Resume here, keeping the STOP evidence untouched: the four fresh trees are already verified and
pinned (`preflight-stop-trees.tsv`: baseline/disabled/real at input, armed `928086…` with only
`crates/rum-stdlib/Cargo.toml` changed), the start checkpoint is `start-ns`, and the first
preflight attempt stays as evidence. Continue at `PROGRESS.md`'s documented command with a fresh
`--out .temp/brief-793/runs/armed/preflight-02`; do not re-copy the trees or re-install the
manifest. From then on every state inherits the dispatch scope exactly as above. The bars, bounds,
STOP rules and budgets of the brief are otherwise unchanged.

## Amendment 2 (gate c151) — the ownership inventory is armed-authored; the disabled difference is the declared arm switch

The resumed dispatch correctly STOPped at the disabled ownership comparison. On armed all seven
review digests equal the installed tables and `standing-inputs.tsv` (61 self-reaching, 92 hosts,
92 ledger rows, 3,727 macros, 10,135 constructs; `unknown_macros.tsv` `843215…`), both F13 replays
passed with their 68 records and 18 identity cells, the plain no-default preflight passed in the
inherited 22 GiB scope (`Finished in 42.80s`, peak 3,038,380,032 B, oom 0), and all 152 chain pins
and the three installed chain copies verify (`0e508bb4…`). On disabled six of the seven digests
match and `unknown_macros.tsv` has 3,725 rows (10,133 constructs) against the pinned 3,727: the
two absent rows are `UnknownConstruct` at `crates/rum-runtime/src/lib.rs:27:32` and `27:51`, the
`cfg!(disabled)`/`cfg!(live)` invocations present in the armed `ENABLED` expression and absent
from the disabled literal `false`.

Ruling: the seven tables and the chain are a single **armed-authored** inventory installed as
common bytes on all roles — exactly what the historical writer did (one armed output per
inventory, installed on four roles). The replay bar is therefore the armed one; on disabled and
real the installed tables and `emission_chain.tsv` must be byte-identical to armed's, and the
disabled regeneration stays a diagnostic that is never installed and never becomes a bar: it may
differ from the armed review only in `unknown_macros.tsv`, by exactly those two declared rows
(3,725/10,133). No review is installed from the disabled run and no inventory is hand-edited. The
question `disabled-ownership` is answered this way (option 1, with the diagnostic framing).

Resume from the ownership checkpoint: keep both arms' producers and all review copies as
evidence, keep the installed tables and chain untouched, record the disabled diagnostic exactly
as above, and continue at the class discovery and LP01 controls on both arms, the typed slots,
the four R3 discoveries and their green rechecks, the promotion adoption, the 165-row seal, the
retained selectors, the P0543 lane, the six cold primaries and both restores, the twelve receipts,
F48, the fourteen selectors, both fmt runs, Stage D and both flag-on controls. All other bars,
bounds, STOP rules and budgets are unchanged.

## Amendment 3 (gate c152) — the nine loopback EPERM reds are the dispatch sandbox's network denial; the host replay is the authoritative Stage D

The third dispatch stopped at Stage D exactly as the bar demands: the complete flag-off workspace
returned 4,544 passed, 19 failed and 71 ignored across 70 result lines (raw exit 101), with all
ten accepted reds present, all six named remnants green, the 71 ignores byte-equal and the
source-auth control green. The nine extra failures are the native ZeroMQ tests, each failing at
`native ZeroMQ test cannot bind loopback: Operation not permitted (os error 1)` — the dispatch
sandbox's network denial — and each is `ok` in the historical population. The same nine pass on
the host; c135 already classified brief 776's stage-D reds as the fence's loopback, and the 789
host driver records "the 9 zeromq loopback names pass on the host (they are sandbox-only)" with
the 789 host Stage D green for all nine.

Ruling: the nine loopback reds are declared environmental for the dispatch sandbox, by name. The
dispatch-side Stage D population bar is: 70 result lines, 4,544 passed, 19 failed, 71 ignored,
raw exit 101; exactly the ten accepted reds; exactly those nine extras, each with the EPERM
cause; zero missing reds; zero other extras; zero extra, missing or differently-reasoned
ignores; six remnants and the source-auth control green. The authoritative Stage D bar
(4,553/10/71) is the director's host replay of the same armed bytes with the same command, lane
environment, target isolation and thread/job counts:

```
WORKSPACE PASS passed=4553 failed=10 ignored=71 results=70 raw_exit=101
FLAG ON PASS leg=flag-on-lp passed=3+1+1 failed=0
FLAG ON PASS leg=flag-on-class-lease passed=197 failed=0
FMT PASS stage=final
FINAL TREE PASS tree=c04bbf8245a8b41c6b069f36df0c2da3785f25e7565a6938e2dff29d77a14afb files=5851 debris=0
STAGE D HOST PASS workspace=true flag_on_lp=true flag_on_class_lease=true fmt=true zero_oom=true
```

(from `.temp/director/gate-793/host-stage-d.out`; its 71 ignored lines are byte-identical to
`stage-d/ignored-verbatim.txt`; the workspace scope line is `max=21474836480
peak=17456148480 oom=0 oom_kill=0 rc=101`). The worker's report keeps its own 4,544/19/71 line
and quotes both verdicts; no assertion is weakened and no red is reclassified.

Resume at the Stage D tail only: do not rerun the workspace. Reuse the calibrated `LP`/`LEASE`/
`FMT` jobs and `flag_on()`/`shared.tree` in a tail script under `scripts/`, with the lane
environment, `RUM_CM740_BASELINE` and `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`, and run: the two
flag-on legs (`3+1+1` passed / 0 failed and `197 passed; 0 failed`), the final `cargo fmt --all
-- --check`, the four tree pins against `seal/sealed-trees.tsv` (debris 0), then
`scripts/handoff.py` for the R4 handoff. Keep every existing log, run record and the STOP
evidence; update `PROGRESS.md` and `report.json`, run `python3 scripts/report.py check 793`, and
return the summary. All other bars, bounds, STOP rules and budgets are unchanged.

## Director gate (c154) — R4: the live render commit

Ruling 806. The sealed four-role release landed in the working tree: 190 crates paths (165
overlay rows, 9 additions, `additions.tsv`, 15 carried/regenerated/adopted files; 10 adds and
180 modifications, 0 live-only), after measuring that everything else in `crates/` was
byte-equal and archiving the 23 inherited WIP entries to `.temp/director/r4/pre-install/wip/`.
Live's crates subtree now equals the sealed armed subtree (`a7de3a76…`, 4,700 files) and the
recomputed delta is 0. Live's own replays: F13 `5d27e91b…` (68 records/18 cells/68 fingerprints)
with the F55 adoption stamped; the seven ownership tables byte-exact (61/92/92/10,135) and the
152-member chain `0e508bb4…`; both promotion configurations zero-rejection, the canonical merge
left `promoted.tsv` at `51b693ec…`, the second fresh emission byte-identical; the live-scene
seal re-emitted `57e9acbd…` with mutants `e9940d5d…`, additions `243dc0cf…` and the same
`source_schema` cell, and the independent reconstruction agreed. The first live suite reproduced
the authoritative Stage D: `FMT PASS`, `WORKSPACE PASS passed=4553 failed=10 ignored=71
results=70 raw_exit=101` (the ten accepted reds and 71 ignores exact by name; peak
17,388,060,672 B, oom 0), flag-on legs `3+1+1/0` and `197/0`, `SUITE PASS`. The release is
committed with TODO.md; no dependency, no `unsafe`, no tag.
