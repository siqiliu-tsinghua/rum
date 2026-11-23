# Brief 775 — G15-CM4 design-only: the carried scene's ownership-graph divergence, zero-unknown inventory and reconciliation decision

**Director-dispatched, design-only (`Scope: NONE`), one stage, one gate.** This continues
ruling 783. Brief 774 staged the class-fixture repair in `.temp/brief-774/scene` and proved it
mechanically (plan check, certificate), but its standing ownership replay stopped correctly on
24 missing host-payload rows — 11 identity-preserving position pairs plus 13 new emission facts
— and never reached the `UnknownConstruct` partition or the host-payload ledger. This brief
produces the complete zero-unknown inventory of that divergence and decides the reconciliation.
It writes nothing outside `.temp/brief-775/`.

## Goal

When this brief is done, in `.temp/brief-775/` only: the complete, evidence-cited, zero-unknown
inventory of the carried scene's ownership-graph divergence — every row of all four fact tables
(`self_reaching.tsv`, `host_payloads.tsv`, the four `unknown_*.tsv` partitions,
`host_payload_ledger.tsv`) classified as unchanged, identity-preserving move (position or
impl-anchor), or new-and-attributed to the declared emission delta; the per-row ledger
disposition decision for the new host payloads; the fail-closed reconciliation design with four
watched falsifiers; the blast radius; and a drafted implementation brief. The carried scene and
the live tree stay byte-identical; no acceptance suite runs.

## Context pointers

- Read `AGENTS.md`; rulings `.temp/director/ruling-783.md`, `ruling-782.md`; brief 774 with its
  Gate c133; brief 767's disposition and 768/769's evidence (the 60 pinned paths, the five
  additions, `source_schema` cells).
- **The red:** `.temp/brief-774/logs/ownership.log` (round 1: one self-reaching cell, replayed),
  `.temp/brief-774/logs/ownership-2.log` — `EXACT INVENTORY self-reaching types: 61 rows`, then
  `UnknownHostPayload inventory mismatch:` with 24 `missing inventory row:` lines and 11 `stale
  inventory row:` lines; the 13 unpaired rows name `q25_ownership.rs:376`, ten `lib.rs` sites,
  two `options/core.rs` readers and `suspension/core.rs:647`.
- **The check:** in the carried scene, `crates/rum-stdlib/src/tests/ownership_graph/mod.rs`
  (`production_inventory_is_exact`, the three partitions in order), `evidence.rs` (the
  `compare` mismatch format and `sort_rows`), `source.rs` (`production()` scans
  `crates/{rum-core,rum-runtime,rum-stdlib}/src`), `graph.rs`, `host_payload_ledger.rs` plus
  `host_payload_ledger/tests.rs` (`production_host_payload_ledger_is_complete_and_attributed`,
  the header and the nine columns), and the seven tables under
  `crates/rum-stdlib/src/tests/ownership_graph/inventory/` (committed sizes: self_reaching 61,
  host_payloads 79, unknown_attributes 2,207, unknown_captures 3,235, unknown_macros 3,245,
  unknown_storage 179, host_payload_ledger 79 entries + header).
- **The declared emission delta:** `.temp/brief-768/evidence/` and `.temp/brief-769/` — the 60
  pinned paths, the five additions (`class_lease.rs`, `class_rows.rs`, `class_lease_fixtures.rs`,
  `q25_ownership.rs`, `q25_p2379_proof.rs`), the 27 seams and the module insert. The live tree
  has no `crates/rum-runtime/src/cm740_probe/` at all; every new row must trace to one of these.

## Scope

- You MAY write only under `.temp/brief-775/`: `logs/`, `scratch/`, analysis scripts, review
  copies, the report. The scratch copy is `.temp/brief-775/scratch/` — copy
  `.temp/brief-774/scene` excluding `target`.
- You MUST NOT touch `.temp/brief-774/scene/**` (byte-verify the walk at entry and exit),
  `.temp/brief-766/`, `.temp/brief-768/`, `.temp/brief-769/`, every accepted scene, the live
  tree, `.git`, `Cargo.lock`, any `AGENTS.md`/`docs/**`/`TODO.md`. No live build, no live edit,
  no acceptance suite, no receipt.
- The scratch copy may run cargo directly inside the dispatch scope (`CARGO_BUILD_JOBS=8`,
  `RUST_TEST_THREADS=8`, no `RUM_*` variable, `--no-fail-fast` as a cargo flag before `--`).
  At most three scratch test invocations; one scratch copy; no second build tree.

## Deliverable

1. **Entry re-authentication.** Recompute and record: the carried-scene walk `12a687e2…`/5,816;
   the class table `8c8b2232…`; `provenance.tsv` `965eca10…`; the fixture `3738e894…`; the two
   replayed files' current digests (`local-contracts.tsv`, `self_reaching.tsv`);
   `class_rows.rs` `6da1dd03…` and `class_lease_fixtures.rs` `475d0edb…`; and the carried
   scene's seven inventory tables' digests and row counts. Any mismatch is a STOP.
2. **The scratch observation.** In the scratch copy only: install the 24 observed host-payload
   rows (the exact `missing inventory row:` payloads from `ownership-2.log`, paired with their
   stale counterparts) into `host_payloads.tsv`, rebuild, and run
   `cargo test --locked --offline -j8 --no-fail-fast -p rum-stdlib --lib production_inventory_is_exact -- --nocapture --test-threads=8`
   so the test reaches and reports the **complete** `UnknownConstruct` partition mismatch
   (all four files, every missing and stale row). Then run
   `cargo test … --lib production_host_payload_ledger_is_complete_and_attributed -- --nocapture --test-threads=8`
   in the same scratch copy and capture its complete missing/unattributed set. If a further
   layer appears (a fourth puzzle beyond the three partitions and the ledger), iterate once and
   record it. Quote every verdict and mismatch line; the carried scene must be byte-identical
   afterwards (the exit walk equals the entry walk).
3. **The complete divergence inventory.** For each of the five table families: every missing
   row, every stale row, and the identity-keyed pairing — classify each pair as a position
   move, an impl-anchor move (e.g. `@20`→`@25`), or a construct change (a STOP unless it is a
   declared emission edit); classify every unpaired new row, attributing it to a declared
   emission artifact (the 60 pinned paths, the 27 seams, or one of the five additions), with
   its file, construct and the exact source evidence. Every committed row that disappears
   must be explained by a declared edit or the inventory is STOP. End with the zero-unknown
   statement: counts per family, per class, and none unexplained.
4. **The ledger decision.** For each new host-payload row: the disposition
   (`PublicInput` / `Declared` / `Unbounded`), the witness text when the class has one, the
   `owed_by` and `external_implementation_coverage` cells, and the reason (these are the probe
   seam's own opaque inputs: `impl FnOnce() -> R`, `dyn Fn(…) -> …` in a `Box`, generic
   continuation parameters, etc.). For each moved row: whether its ledger entry must be
   rekeyed, and how identity-keyed matching avoids a stale-entry illusion. State the ledger's
   exactness rule as the tests enforce it.
5. **The reconciliation design.** An explicit `#[ignore]` maintenance command per table family
   (the pattern of `regenerate_local_contract_fingerprints`: writes a review copy under
   `target/brief-775/`-style scratch, never automatic admission; discovery still never reads
   the inventory), and a fail-closed differ: identity-keyed pairing; position/impl-anchor-only
   moves accepted; a new row accepted only when its file is in the declared emission path set
   **and** a ledger disposition exists; a disappeared row, a mutated identity, or an
   unattributable file is a STOP. Specify four falsifiers and how each is watched failing:
   (a) an unattributable new row, (b) a mutated identity, (c) a fabricated row absent from
   discovery, (d) a missing ledger disposition. Specify the adoption ceremony (review copies →
   differ → director ruling → install) and the `include_str!` rebuild consequence.
6. **Blast radius.** The seven fact tables and the ledger (sizes, which tests read them, the
   rebuild cost); the certificate/chain — verify whether any of these files is in the 152-chain
   and say so; the 768/769 evidence tables stay untouched; the class table and `provenance.tsv`
   stay untouched; the live tree untouched; the scene carries the reconciled delta until the
   install. Also name the deferred 774 acceptance (flag-on module, the receipt diff, the
   workspace gate with its two skips, `cargo fmt --check`).
7. **The drafted implementation brief** in `TEMPLATE.md` shape: stage A the complete inventory
   with the front-half zero-unknown stop; stage B the maintenance command + differ + four
   falsifiers; stage C the adoption and a green `production_inventory_is_exact` +
   `production_host_payload_ledger_is_complete_and_attributed`; stage D the deferred 774
   acceptance; its own budget and stop lines.

## Environment

No network, no new dependency, no new `unsafe`, no `RUM_*` variable beyond what the commands
state; `.temp` only, never `/tmp`; never read `~/.config/ccds/api-key`; no `git`. The scratch
copy's cargo runs directly in the dispatch scope (no nested `memory-scope.sh`).

## Verification

`rum-report/1` must quote, verbatim: the entry pins and the seven table digests/counts; the
scratch exit walk equality; the construct-partition's complete `missing inventory row:` /
`stale inventory row:` lines; the ledger's complete mismatch lines; the per-family inventory
counts and the zero-unknown statement; the four falsifier designs; the chain-membership answer
for the seven tables; and `scripts/report.py check 775` must print no PROBLEM.

## Checkpointing and stops

Write `PROGRESS.md` notes as you go. STOP if: the carried scene's bytes change; a live write or
live build happens; a scratch observation needs a change to discovery code or to any non-inventory
source; any row cannot be paired or attributed to a declared emission artifact; any ledger entry
cannot be dispositioned; the 768/769 evidence would need to change; a fourth layer would need
more than one extra iteration; or the projected work exceeds the remaining focused budget. Never
cut the inventory to fit the budget: stop with the captured lines and counts.

## Budget

**focused 3,600 s + workspace 0 + epoch 0**, carved from the reserve after brief 774
(32,148.36678847017 → **28,548.36678847017 s**). Component bounds: entry pins and scene copy
300 s; scratch builds and the three test invocations 1,800 s; inventory analysis and the ledger
decision 1,200 s; drafting and slack 300 s. Unspent returns. A projected overrun stops for a
larger carve, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-775/report.json`, stage `""`, via `scripts/report.py`; `check`
must print no PROBLEM. The summary names the per-family divergence counts, the 13 new rows'
dispositions, the reconciliation decision (the maintenance command, the differ, the four
falsifiers), the blast radius and the drafted implementation brief's budget.
