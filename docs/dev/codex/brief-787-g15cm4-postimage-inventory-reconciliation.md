# Brief 787 — G15-CM4 Stage D repair: the postimage inventory reconciliation — the population/aliases identity and the typed-slot census — design-only

Base: `4a663307` (the c144 gate; brief 786 dispatched from it). You are the design worker
for this brief, not the project director. `CLAUDE.md` belongs to the director and is not
your instruction file. Follow `AGENTS.md` and this brief. Do not commit, branch, or push.

## Why this brief exists

Brief 786 (`docs/dev/codex/brief-786-g15cm4-proof-closure-apply.md`, STOP accepted at
ruling 795) completed its stage A green — the re-derived `proof_controls.rs` (`5835c100…`)
is installed on the armed and disabled images and both proof tests pass there — and stopped
in stage B under its own rule: the `obligations_discovery_campaign` capture is not
`population.tsv`'s schema. The measured capture is 29,861 data lines over 29,828 raw
`g.source_rows` (36 lines of a column count other than 20, 15 unsorted lines from multiline
raw strings in column 17 of `crates/rum-jupyter/src/zeromq_fake_tests.rs`, every 20-column
row with empty policies and aliases), while the committed file is `239f1a9c…`, 2,431 lines,
2,430 data rows, last committed by brief 738 (`85fdf7cb`). The campaign is a diagnostic
dump plus reconcile (`IDENTITY DELTA new=8976 missing=8 ...`), not the table's generator;
the plan's premise (`.temp/brief-783/plan.md:205-216`) is refuted. Separately, brief 784
stage A's `cases.rs` else-branch rewrite is measured to move the typed-slot census
(`G15-C1 CENSUS production frames=461 finalizers=50 implementations=611 typed=100 macros=615
result=375 insert=240`) against the plan's Q4 forecast `(461, 50, 611, 614, 374, 240)`, and
`source_baseline_matches_earned_receipt` panics at `reporting.rs:331` ("recorded decoder use
must be rediscovered").

This brief is the complete zero-unknown inventory and design of both reconciliations, so
the continuation can certify the scene without inventing inventory semantics. **Design
only: `Scope: NONE — no source change.`** You write no source; you plan with `path:line`
evidence and you draft the continuation implementation brief 788.

## The carried state (re-measure it yourself; any drift is a stop)

The 784 images now carry the re-derived proof file. Under the worker's own `tree_hash`
(`.temp/brief-786/tools/common.py`, same algorithm as 784/785): baseline `3d0b6829…`/5,842,
armed `f6969139d0c0c94099944d87af2fb19341b6f1681b8bc4d62e1b1a42ee48aec4`/5,851, disabled
`0d9d1c5f43ee94486865d1888fabb7442e1e0d96c4d17ff1eeff6aec79526867`/5,851, real
`57e3533e9be0d52bfb5296f54fdb384c53d1256b794880ade61fcaa3c10ec1ce`/5,851; debris 0.
`proof_controls.rs` is `5835c1004180a675cf5ec123c35fa0f9a3e219999cf0230abdb1dea26c4622eb`
on armed and disabled and `a3092419e6d4e87a3363c2804fed303a74a92abef24afb6dc5b0998716c15d35`
on baseline, real and live. The live tree carries the 23 inherited porcelain entries
(record `.temp/director/gate-c88/porcelain.txt`); do not run git in any form.

Carried measurements to re-derive, not to trust: the armed and disabled population captures
(`04e1a42c…` / `c1a4778a…`, `.temp/brief-786/logs/stage-b-population-{armed,disabled}.log`);
`IDENTITY DELTA new=8976 missing=8 ambiguous_dispatch=3336 ambiguous_occurrence=53
stale_tail=211 stale_destructor=2893 unknown=913 port_blocked=241`; the census line above
(`stage-b-typed-slot-all.log:23`); the `reporting.rs:331` failure; the 13 SourceIdentity
tests of the plan's §3 and the six hardcoded pin sites `receipts.rs:116,167,187`,
`g15a.rs:67`, `g15c1.rs:67`, `s4.rs:144`.

## Scope: NONE — no source change

Read-only. Your writes go to `.temp/brief-787/` only. Do not edit the images, live source,
`.temp/brief-784/`, `.temp/brief-785/` or `.temp/brief-786/`. Running cargo is allowed; use
one `CARGO_TARGET_DIR` per tree under `.temp/brief-787/`, a measured `pwd` and the tree hash
in every log, never a shared dir across trees. The images are read-only for you: run tests
with an external target dir.

## Stage A — entry and instruments (300 s)

Re-measure the four pins and the proof shas. Re-run on armed the population campaign
(`oracle::obligations::tests::obligations_discovery_campaign -- --ignored --exact
--nocapture`) and the typed-slot target (`-p rum-stdlib --test typed_slot_census` full
target), each with its own target dir, and quote verbatim: the capture's line count, bad
widths, empty columns, the committed-file comparison, the `IDENTITY DELTA` line; the
`G15-C1 CENSUS` line; the `reporting.rs:331` panic. A result that does not reproduce a
carried fact is a stop. Calibrate the two instruments before believing them: a negative
result most of all.

## Stage B — the population and aliases identity (900 s)

A complete inventory, zero unknown. For each table — `population.tsv`, `aliases.tsv`,
`provenance.tsv`, `logical-profile-required.tsv`, `typed-slot-stages.tsv`, and any other
file under `crates/rum-stdlib/tests/suspension_oracle/inventory/` that a stage-B/C writer
touches:

1. Who writes it (a generator, a hand-curated commit, a certificate) and who reads it; its
   header and row semantics (which columns are discovered, which curated, which sealed);
2. Its state on the carried images: every consumer test, its verdict on armed and on
   disabled, and the exact assertion that fires;
3. For the 13 `SourceIdentity` tests of the plan's §3: each one's measured failure on the
   carried images (run it), its true owning file, and whether the plan's attribution holds
   — the plan's attribution is not trusted (rulings 793/795);
4. The campaign's real role: a diagnostic dump plus `reconcile`; the `Kind` composition of
   the identity delta; the 8 `MissingAcceptedKey` rows and the mechanism that moved their
   `EdgeKey::key()`; the empty policies/aliases columns' consequence;
5. `aliases.tsv`: its 2,204 rows, the six sealed rows the worker named (P0027, P2106,
   P2107, P2380, P2382, P2384) and what "sealed" means at their site;
6. The inventory seal at `mutations/source_auth/mod.rs:131-139`: quote it, name the test and
   the gate it serves, find its design record and intent, and establish (or refute) whether
   any inventory overlay has ever existed — the 147-row table has zero rows under
   `inventory/`. State the pipeline's intended inventory-update mechanism (a new baseline at
   the re-freeze, an overlay, a certificate) from the code and the record, not from taste.

## Stage C — the typed-slot census (1,200 s)

A complete inventory, zero unknown:

1. The exact `cases.rs` else-branch diff brief 784 stage A applied
   (`session.insert_suspension_frame(frame)` -> the `resume_or_suspend!` form) and every
   macro/decoder/result use it adds or removes;
2. The measured census tuple on armed and on disabled (frames, finalizers, implementations,
   typed, macros, result, insert), against the committed pins at the six sites and against
   the plan's Q4 forecast; each pin site's semantics (is it a preimage record, a postimage
   expectation, or a both-arms invariant?) with `path:line` and the surrounding assertion;
3. The removed decoder use: which use, where it was recorded, which receipt owns it, why
   `reporting.rs:331` fires, and the receipt's regeneration path (if any exists);
4. Nested0431 and the `#callN` ordinals: re-derive the removed row and the shifted ordinals
   with a calibrated instrument (not by memory), and their consumers
   (`typed-slot-stages.tsv`, the census tests, the G15-A/G15-C1/S4 sites);
5. The eight typed-slot tests red on the carried image: each cause, separated into
   pre-existing and `cases.rs`-caused;
6. The two candidate paths, each with its complete consumer list and forecast: (a) update
   the six pins and whatever inventory must move, with the seal question answered; (b)
   revert the `cases.rs` else branch to a form that preserves the census tuple, the decoder
   use and the ordinals, with the render/class-form consequences stated. Cost neither down
   for its size; state which is semantically correct and why.

## Stage D — dispositions, falsifiers, order (800 s)

Exactly one disposition per family: repair (exact sites, scope, falsifier), inventory
regeneration (generator, review copy, adoption, falsifier, order — and only where the seal
permits), named stop (only with a standing authority). Every disposition carries a strength
argument against the current state and a calibrated falsifier (red and green, named
variant). Produce: the corrected brief 784 stage C writer list, file by file (the
`population.tsv` entry is void as written — replace it with the measured truth); the
deterministic order with the invalidation rule and no cycle; and the stops. No sealed gate,
inventory or certificate may be weakened without a named authority and its own designed
step and falsifier; a proposal that changes `source_auth/mod.rs` must say exactly what it
changes, why the seal exists, and what falsifier proves the change does not open the
class.

## Stage E — the drafted continuation brief 788 (300 s)

Draft brief 788 in this template's shape: the chosen dispositions as exact steps; then
brief 784's stages B–D with the corrected writer list (the four per-tree target dirs; the
host-payload generators by full path; the `--nocapture` campaign on armed and disabled
only; F13 on four trees; both-mode `local_production_acceptance`; the writers; the four
rebuilt images; the twelve `rows=9` receipts; the six-case gate; `cargo fmt --check`; the
default-mode workspace with the nine skips and exactly the ten red names; the 71 ignored;
the flag-on `5 passed; 0 failed`; the class-lease fixtures; the F16 cleanup; the final
pins). State the run book, the stop list and the forecast.

## Checkpointing and stops

Write `.temp/brief-787/PROGRESS.md` and a stage log after each stage. Stop, with the exact
evidence, on: any pin drift; a capture or census that does not reproduce the carried facts;
a consumer or a pin site you cannot state semantically; a generator claim you cannot verify
by running the generator; an unattributable failure; a disposition weaker than the current
state; a proposed seal change without a named authority and its falsifier; a required
change outside the drafted 788's scope; any live edit; any need for network, a new
dependency or `unsafe`. A stop is a correct terminal state.

## Hard rules

No git in any form. No network. No new dependency. No `unsafe`. No committed-file edit. No
sealed-artifact edit. No hand edit of a derived file. No CM5. No 744/745/746. Q25's
thirteen rows stay open. `.temp` only; never `/tmp`; never read `~/.config/ccds/api-key`.
One `CARGO_TARGET_DIR` per tree, always under `.temp/brief-787/`.

## Budget

Focused **3,500 s** (A 300 / B 900 / C 1,200 / D 800 / E 300) + workspace **0** + epoch
**0**, carved after brief 786 (charged 840 s + 2,858 s of its 7,500 s authorization and
returned 3,802 s; reserve 3,870.83478847017 s) -> reserve after the carve
**370.83478847017 s**. Unspent returns; a projected overrun stops for a director ruling,
never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-787/report.json`, stage `A-E / c146`; `summary`, `changes`
(none expected outside `.temp/brief-787/`), `disclosures`, `deviations`, `questions`, checks
and measurements with every verdict line quoted, and the final message
`scripts/report.py summary`. A DONE report has no FAIL row. The director gates from raw
evidence.
