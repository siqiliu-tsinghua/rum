# Brief 809 — G15-CM4 D.2: the joint measurement of the two placements and the conditional landing (implementation)

Implementation brief. Scope: the two live source paths named below — edited only after the
predeclared selection rule returns a qualifier, and only in the measured hunk — plus, only then,
the source-derived inventory outputs the standing producers regenerate, and `.temp/brief-809/`.
No render, no install, no re-freeze, no epoch, no acceptance, no tag, no network, no new
dependency, no new `unsafe` in `rum-core`/`rum-runtime`, no git command. All scratch under
`.temp/brief-809/`.

## Why this brief

Brief 808 Amendment 3 sealed DONE (gate c203, ruling 855). The closed mechanism inventory (gate
c202, ruling 854) left exactly two repair targets with exact bills: **eval_core**
(+8,005,829..+8,107,809; row 1116 carries +7,970,633, of which +12,320,486 is repeated
flag-address LEA over 1,046,892 visits, offset by −4,468,781 of moves and +118,928 other) and
**pattern_matching** (+6,273,782 exact in all nine pairings; `BindingSnapshot::capture` outlined
at 426,421 eager visits). The placements, invariants, dispositions, joint design and drafts are
sealed in the 808 record. This brief measures the two placements separately and jointly on the
armed and real roles at one path, prices them by the 805 subtraction on the deduplicated
nonpublication union, and lands only an individually verified source form. Acceptance of the
class form belongs to a later decision.

## Goal

1. Measure **E-only**, **M-only** and **E+M** on both roles (three whole Callgrind draws per arm
   per variant = 18 pricing profiles) against the frozen **J0** (the three 807a ELFs and the nine
   807f whole draws), and run the **B2** four-cell diagnostic matrix (four builds, 12 diagnostic
   profiles) to close the matcher's cause question.
2. Publish every cell, including null, adverse and unseparated results. The definitive package
   price is `median(armed J0→J1 savings) − median(real J0→J1 savings)` on the deduplicated
   nonpublication union, with the closure-corrected membership and `additive_claimed=false`.
3. Land exactly the qualifying hunk(s) by the predeclared selection rule, verify the semantic,
   publication, standing and timing gates on the formatted live sources, and seal the receipt
   that brief 810 consumes. No qualifying form means no landing and no 810.

## Context and pins

- **The 808 record, read-only** (`.temp/brief-808/`): `report.json` `0f00aead…`; `placements.json`
  `300373af…`; `invariants.json` `13fb3030…`; `j0-seed-currencies.json` `d9f3d62b…`;
  `dispositions.json` `bde2a794…`; `decisions.json` `41f3669c…`; `joint-design.md` `449004e4…`;
  `draft-809.md` `e2cc35c0…`; `draft-810.md` `cc0e430c…`; `live-exit.json` `7aed5c5a…`. The
  stop-3 mechanism record (`.temp/director/808-stop-3/`): `seal.json` `a6dc561a…`,
  `mechanisms.tsv`, `machine-verdicts.txt`, `components` tables and the two ELF disassemblies
  (`base.dis` `bb4c9a45…`, `new.dis` `d372ac1d…`).
- **J0, frozen**: the three 807a ELFs at `.temp/brief-807a/bins/{base,new,control}`
  `524c2bc2…` / `2fa652ea…` / `c166a2ff…` with their release receipts and `SYMBOLS` records,
  built from the 806 roots below at one path, one codegen unit, offline; the nine 807f whole
  draws (`.temp/brief-807f/captures/whole-*`) with their raw hashes; the 93 family nine-pair
  medians and per-draw totals (`family-pairings.tsv`); `new-publication-catalogue.tsv`; the
  reconciliation and per-site prices; the four falsifiers; the driven ledger
  `a2c48c90fab83d49ffceb07e8c74992268fea353cf5e03574f8cbe63435d3895` (26 lines, 1,741 B).
- **805**: `.temp/brief-805/evidence/joint-analysis.json` `f250efdf…` — the definitive
  nonpublication union, shared caller 1186, the publication subtraction, and the 13,556,424
  differential as history only.
- **804**: `.temp/brief-804/evidence/price-ledger.tsv` `d757d2dc…` — the refusals and
  withdrawals (1383-always closure +376,710; 1086-always +45,220; 1199-inline +18,347;
  1618/1263/1014-always refused on the publication choice).
- **The 806 role roots**: baseline `cb9205a6…`/5,842, real `dda9146e…`/5,851, disabled
  `30cd48fc…`/5,851, armed `df0db63c…`/5,851 at `.temp/brief-806/completion/images/`; the 807a
  Stage B environment, flags and argv.
- **The live preimages**: `crates/rum-runtime/src/runtime/profiling/timing.rs` `2e3d38e2…`
  (the existing `#[inline]` declaration at :152-158, the closure at :156, the flag load at :957)
  and `crates/rum-runtime/src/runtime/pattern_matching/core.rs` `53fe924b…` (the ordinary caller
  at :42, the cold driven caller at :63, `fn capture` at :1085). The real/armed image copies of
  both files are the placement's role preimages as recorded in `placements.json`.
- **Functional**: the canonical program `dc6af977…`, the fixed functional program `c70feffd…`,
  the 120 ordered labels, the archived normalizer, the 14 disabled flags and the off-state env.
- **Entry census**: 5,899 paths; against `.temp/brief-808/live-exit.json` `7aed5c5a…` exactly
  `TODO.md` and `docs/dev/codex/brief-808-g15cm4-residual2-inventory.md` modified and this brief
  added, with no other addition, removal or change.

## Scope

The only production edits are the two substitutions below — separately or together, exactly as
measured. Everything else is scratch under `.temp/brief-809/`. The live tree is edited only in
Stage D, only after the selection rule returns a qualifier.

**E — the evaluator capture mode.** In the existing declaration

```rust
#[inline]
pub(in crate::runtime) fn runtime_path_timing_start_adopted(
    session: &crate::runtime::Session,
    phase: &'static str,
) -> Option<RuntimePathTimingToken> {
    run_configured_timing(|| runtime_path_timing_start_adopted_slow(session, phase))
}
```

replace only the closure capture at `crates/rum-runtime/src/runtime/profiling/timing.rs:156`:

```rust
    run_configured_timing(move || runtime_path_timing_start_adopted_slow(session, phase))
```

The declaration, the attribute, the types, the atomic load at :957 and every operation stay.
Planning band 0..8,107,809 differential translated Ir; the four-row J0 seed for a full
base-shaped recovery is 8,034,374..8,090,378 (b0 = 28,031).

**M — the matcher inline hint.** At `crates/rum-runtime/src/runtime/pattern_matching/core.rs:1085`
(live and armed) / :538 (real), insert exactly one line before `fn capture`:

```rust
impl BindingSnapshot {
    #[inline]
    fn capture(bindings: &SymbolExprMap, matcher: &MatcherNode) -> Self {
```

No other change. Planning band −376,710..+6,822,736; a base-shaped recovery is about
5,724,828..6,822,736; the J0 seed is exactly 6,273,782 (b0 = 0).

## Stage A (200 s) — authenticate and preregister

Re-authenticate every pin above by hash and content; read the 808 record and the stop-3 mechanism
tables; bind the three J0 ELFs to the nine whole draws and the 93 family medians; bind the entry
census (5,899; two modified, one added). Freeze before any output: the estimator (the union rule,
the changed-edge closure and its expansion audit, the 3×3 per-role pairings, the all-81 price
combination table, the b0+b1 uncertainty, the same-code controls and the seed bands), the three
pricing variants and their build order, the B2 matrix and its order, the capture order, the 120
labels, the off flags, the CPU, the driver, the source transformations, and the selection rule.
Stop on a stale pin, an unbound draw or executable, or a changed preimage.

## Stage B (1,800 s) — the builds

**Pricing builds (six):** E-only, M-only, E+M, each on real and armed. Start every variant from a
copy of its authenticated role root at one absolute, disposable staging path and one external
target path under `.temp/brief-809/`. Offline Rust 1.85.0, one release codegen unit
(`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`), private offline `CARGO_HOME`, the exact 807a argv
(`cargo build --offline --locked -j 8 --release -p rum-cli --bin rum-cli`), a bounded process
group, `TMPDIR`/`TMP`/`TEMP` inside the brief. Save clean parent/postimage manifests,
environment, feature graph, version, ELF and section hashes, the symbol map, argv/exit and the
output per variant. Require the 120 ordered labels, iteration counts and normalized result bytes
to equal the same role's 807a receipt and the fixed functional run, across arms; compare
normalized stderr and exit. A mismatch blocks the affected cell from pricing and landing; preserve
it exactly.

**Diagnostic builds (four), the B2 matrix** from `joint-design.md` §B2: one fixed armed source
manifest with the instrumentation constant at `rum-runtime/src/lib.rs:27` selected from the two
sealed preimages (axis I), and the number of static users of the existing capture body varied at
the discretionary inliner (axis U). Save one-CGU input bitcode, target layout, CPU/features, the
optimization pipeline and link inputs with the release recipe; apply the call-site
`alwaysinline` attribute to the one cold direct call (U1) or the identity transform (U2), then the
identical mandatory-inline pass; never copy the body in Rust, take its address, remove the cold
route or change the capture declaration. Bind by demangled caller/callee and instruction kind.
Verify the diagonal controls (the real one-user and armed two-user stock decisions), the equal
ordinary output and edges, and the full driven ledger in the armed cell. All four builds are
scratch-only; B2 carries no price and can never authorize a landing. If matching pre-inline input
or a faithful backend continuation is unavailable, record the exact missing pass/cell and every
attempted observation; never simulate U1 by dropping the snapshot; continue independent pricing.

## Stage C (2,500 s) — measure and decide

Eighteen pricing profiles: three whole canonical Callgrind draws per arm per variant, the
canonical 807f workload, driver, markers, CPUs, the 14 off flags and off-state env; serial and in
a preregistered balanced order across the saved binaries. Twelve diagnostic profiles for B2 (three
per cell). Preserve every raw, resolved body/edge/PC table and receipt, including rejected
attempts; include the v2.5 quiet qualification where the 807f driver requires it. A missing draw,
CPU/ELF mismatch or invalid boundary leaves the affected cell unpriced.

For each cell: register the helper/inline owner, every caller (including contexts with zero
executed calls) and the observable child-edge counts; bind fresh publication PCs on each binary
(verify only — the 38 sites are user-owned, never repriced); form the union of every row whose
incoming count from the measured set changes on any tested pair/arm, including new or re-billed
owners, and declare the membership by counts before reading its savings; count each row once;
keep the per-row raw/publication/net arithmetic with a zero-remainder reconciliation. Audit E's
two evaluator rows, its two parent rows and every library-wide use of the start wrapper; audit M's
1380/1381/1383 union, all 19 outgoing witnesses and any SpecFromIter, iterator, allocation or
drop change.

Estimators: for each role, all nine J0 new/base pairings and all nine J1 new/base pairings on the
same union; print all nine role savings and the differential of medians; tabulate all 81
combinations; compute the b0+b1 combined uncertainty, the same-code signed bands and the
per-currency bands; the definitive package price uses the 805 subtraction. A saving confined to
the real role has the wrong sign for a repair. Book a positive differential only when it exceeds
its own combined uncertainty, its 81-price interval separates from zero, and its magnitude
separates from the same-code departures on the same rows. Print every cell's outcome, including
zero, adverse, unqualified and unseparated. `additive_claimed=false`.

Apply the isolation falsifiers and negative controls from `placements.json` on the singles and on
the combined binary: for E a per-boundary PC/flag/destination and address-work table in both
contexts with all relaxed loads intact and a lower armed excess beyond its band; for M positive
eager traversal on the 426,421 events, the rollback/freeze/suspension cases, zero ordinary calls
to a standalone copy, the 19 external witnesses, and rejection of the historical 1383-always
row-local claim. Check every must-preserve invariant in `invariants.json` in full: the AtomicU8
order/initialization/lease boundary, no default-path state, the 120 labels and raw-stream
comparison, the 38 gates by owner/order/visits (366,888/366,916/366,929; 37×4 + one reused ×3),
zero ordinary slow-path self, the four falsifiers, the positive undriven resume, the forced and
typed paths, P0256 and the 12 routes, the ordered 26-line/1,741-byte transcript `a2c48c90…`, the
two mid-request callback edges twice each and the closed scopes.

## Stage D (1,500 s) — verify and land the measured form

Apply the preregistered selection rule: use the measured joint form if both isolations and the
combined union pass; else use only the qualifying single; if both singles pass but the joint
fails, prefer the larger separated single differential with no regressions, and on indistinguishable
prices prefer the narrower declaration footprint (M); if neither qualifies, land nothing and end
the sequence without 810. Run the repository microbench A/B for the chosen form on the
authenticated same-role base and candidate, alone, bounded
(`scripts/core-language-microbench.sh --output-dir …`), with per-label rows for the target and its
same-tier neighbors; a same-tier regression stops that landing.

Apply only the qualifying hunk(s); run `cargo fmt --check`; rebind the live and staged sources and
the identical release at one path; verify the semantic source identity and the same-path release
identity. Then run every source-covered standing producer on the final formatted sources before
any acceptance selector (there is none in 809): F13 fingerprints/contracts and falsifiers; both
promotion configurations; the certificates; ownership discovery and source positions; the
152-member chain; class discovery/source authentication; typed-slot receipts; the rejection census;
LP01 (1,115 rows, statement identity, columns 17/19, binding 3, the loss record). Quote every
actual producer line and pair every cell. A stale-cell-only red is an ordering artifact — replay
its producer; never hand-edit an inventory. A changed normative identity, an unexplained delta or
an unsupported census/oracle expectation stops for a ruling.

No acceptance, no render, no install. A no-landing outcome is a complete negative report.

## Stage E (300 s) — seal

The `rum-report/1` report at `.temp/brief-809/report.json` (`"brief": 809`, `"stage": "809"`):
pins, builds, the 30 capture receipts, unions, arithmetic, falsifiers and controls, decisions, the
landed hunk manifest or the no-landing record, every producer line, the charge. Run
`python3 -I scripts/report.py check .temp/brief-809/report.json`, detach the summary, seal,
rehash the seal, and audit the exit: zero live changes beyond a declared landing, zero
`.git-attempts`, zero unexplained.

## Deliverable

The frozen preregistration; the six pricing and four diagnostic build receipts (manifests,
environment, symbol maps, functional equality); all 30 whole-draw receipts with raw hashes; the
per-cell union tables with raw/publication/net rows and the zero-remainder reconciliation; the
nine role savings and all 81 prices with the b0+b1 bars and the same-code controls; the falsifier
and negative-control results; the selected form or the no-landing record; for a landing the fmt
state, the source identity, the producer verdict lines, the microbench rows and the receipt 810
consumes.

## rum / Rust pitfalls

A smaller named evaluator symbol can do more work per entry; profile counts are translated
instructions, not time. A cold residual matcher symbol can coexist with a successful ordinary
inline. A per-family median or a real-only speedup cannot authorize a repair. Do not turn the two
split driven streams into an order proof; keep the libc leaf outside source proposals and row 1056
held. `#[inline(always)]` costs the debug frame and the historical forced hint re-billed — forcing
the edge is not a candidate. A check nobody has watched fail is not known to work: every falsifier
ships with its firing control.

## Verification

Read back the frozen preregistration, every ELF/source/functional receipt, the union membership
and ownership, every falsifier with its firing control, the microbench rows, every producer line
and the final source hashes. Emit the four-stage decision (structural, semantic, differential,
landing) with evidence.

## Checkpointing

Append `CLOCK stage=<A|B|C|D|E> t=<monotonic_ns>` to `.temp/brief-809/PROGRESS.md` at every stage
entry and exit. Each stage's box is measured from its own CLOCK entry; the charge is continuous
from dispatch; a stage reaching its deadline stops there with the exact missing cell and its
elapsed time, never kept open. Save each immutable cell and attempt before replacing a staging
tree; at each checkpoint record the exact next cell and any live dirt attributable to this brief.

## Hard rules

No git command, tag, network, new dependency, new `unsafe` in `rum-core`/`rum-runtime`, or work
outside `.temp/brief-809/`. No source change outside the two named sites and, after a qualified
landing, the standing producers' source-derived inventory outputs. No duplicated bodies,
representation flattening, silent-drop helpers, lazy init, `cfg(debug_assertions)` keys,
default-path state, or any change to whether or when anything runs. No forced inline, no
publication relocation, no gate removal, no fastpath. The B2 products never cross into the live
tree. Bounded process groups for children; serial measurements; no competing load. Preserve every
sealed input; the 807/807a–807f and 808 records stay read-only. Q25's thirteen STOPs (P0214,
P0216, P1070, P2348, P2354, P2379, P2381, P2385, P2387, P2409, P2411, P2416, P2418) remain
stopped; 744/745/746 held; CM5 unauthorized; the class campaign's C/C10 and the restart require a
separately qualified epoch.

## Charge

6,300 s of the 10,060.155 s pool: **A 200 / B 1,800 / C 2,500 / D 1,500 / E 300**. The
3,760.155 s remainder stays unallocated for 810; no top-up. Thirty whole draws at roughly 49 CPU-s
each are about 1,470 CPU-s before interpretation; the 807a receipts give 52.76–81.40 s per clean
arm build. A charge over a stage box cannot consume another box.

## Report format

Write `.temp/brief-809/report.json` as `rum-report/1` with the pins, builds, captures, unions,
arithmetic, falsifiers, decisions, the landing or no-landing record and the charge; check it with
`python3 -I scripts/report.py check .temp/brief-809/report.json`; detach the summary; seal and
rehash; audit the exit. Return the class-form funding/redesign and the fastpath to the user; no
acceptance.
