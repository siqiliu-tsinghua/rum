# Brief 776 — G15-CM4 implementation: ownership inventory reconciliation and the deferred 774 acceptance

**Implementation brief, staged (A–D), one gate.** It continues rulings 783 and 784. Brief 775
delivered the complete zero-unknown divergence inventory and the reconciliation design; ruling
784 decided the two open questions: a fourth disposition `Local` for the two `dyn Fn` storage
rows, and the carried 152-chain as the differ's path set. This brief implements the
reconciliation in the carried scene `.temp/brief-774/scene`, adopts the inventories after the
differ passes, and then runs brief 774's deferred acceptance. A front-half stop does not skip
later evidence by shrinking it.

## Goal

When this brief is done: in the carried scene, `production_inventory_is_exact` prints
`EXACT INVENTORY self-reaching types: 61 rows`, `EXACT INVENTORY UnknownHostPayload: 92
rows`, `EXACT INVENTORY UnknownConstruct: 10134 rows` and passes, and
`production_host_payload_ledger_is_complete_and_attributed` passes with the 92-entry ledger
(79 PublicInput / 8 Unbounded / 3 Declared / 2 Local); every adopted row is one the differ
admitted; and brief 774's deferred acceptance (flag-on class-lease module, receipt diff,
flag-off workspace with the two named skips, `cargo fmt --check`) has been run with its
evidence. Q25's thirteen rows stay named stops; a green ledger does not close them.

## Context pointers

- Read `AGENTS.md`; rulings `.temp/director/ruling-783.md` and `.temp/director/ruling-784.md`;
  brief 775's `INVENTORY.md`, `RECONCILIATION.md`, `brief-776-draft.md` and report under
  `.temp/brief-775/`.
- The carried scene is `.temp/brief-774/scene`; its walk must be
  `18dca765e844738dbd64d49dd9fc488e6ea3b2723416df0e7c734ed051a33e07` / 5,816 at entry
  (`12a687e2…` is brief 774's pre-edit entry — do not use it as the pin).
- Inventory and ledger code: `crates/rum-stdlib/src/tests/ownership_graph/mod.rs` (the
  exactness test), `evidence.rs`, `host_payload_ledger.rs` (`check` at line 87, `check_entry`
  at line 115, `HEADER`, `PUBLIC_INPUT`), `host_payload_ledger/ownership.rs`,
  `host_payload_ledger/adapters.rs`.
- Maintenance pattern: `regenerate_local_contract_fingerprints` in
  `crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs:8` (`#[ignore]`,
  writes a review copy under `target/`, never installs).
- The carried chain: `.temp/brief-768/logs/certificate.log` (152 `CHAIN PASS … final=` lines)
  with `.temp/brief-774/logs/certificate.log`'s three carried exceptions (`CHAIN UPDATED
  final` for `class_rows.rs` `6da1dd03…` and `class_lease_fixtures.rs` `475d0edb…`;
  `CHAIN INHERITED` for `guest.rs` `1a2c5ff5…`).
- The 775 logs: `.temp/brief-775/logs/ownership-construct.log`,
  `ownership-ledger.log`, `inventory-classes.log` (the gone rows, the admitted rewrite
  spellings, the per-file chain/live/scene hashes).
- 774's acceptance commands and the two skips are in brief 774, items 6–10.

## Scope

- You MAY modify, in the carried scene only:
  - `crates/rum-stdlib/src/tests/ownership_graph/` — one new `reconcile.rs` (the four
    `#[ignore]` commands, the differ, the four falsifier tests) plus its one `mod reconcile;`
    declaration line in `mod.rs`; `host_payload_ledger.rs` only as Section **The `Local`
    change** states (including `pub(super)` visibility for `Entry`, `Disposition` and
    `check_entry` if the differ needs it);
  - from stage C, after the differ passes: the seven files under
    `crates/rum-stdlib/src/tests/ownership_graph/inventory/`, and the new
    `inventory/emission_chain.tsv`.
- You MUST NOT touch: discovery (`graph.rs`, `source.rs`, the exactness test body), the
  `PublicInput` / `Declared` / `Unbounded` predicates and witnesses, the 27 seam files' bytes
  or any other source file, `.temp/brief-768/`, `.temp/brief-769/`, the class table
  `8c8b2232…`, `provenance.tsv` `965eca10…`, the live tree, `Cargo.lock`, `AGENTS.md`,
  `docs/**`, `TODO.md`. No live build, no live edit.
- Stage A and stage B write only under `.temp/brief-776/` and the review copies under the
  scene's `target/brief-775/`. Review copies are never auto-installed; only stage C's explicit
  byte-exact install writes the inventory paths.

## The `Local` change (ruling 784 D1 — exact, nothing else)

In `host_payload_ledger.rs`:

```rust
const LOCAL: &str = "probe-local closure stored under rum_runtime::cm740_probe; not a by-value input; not expression-reachable";
```

- `enum Disposition` gains `Local`; `entries()` parses the cell `"Local"`.
- `check_entry` gains, after the PublicInput bi-conditional, exactly:

```rust
let local = row
    .construct
    .split_once(": opaque owned payload ")
    .is_some_and(|(trail, ty)| {
        trail.starts_with("rum_runtime::cm740_probe::") && ty.starts_with("dyn Fn")
    });
if local != (entry.disposition == Disposition::Local) {
    return Err("disposition disagrees with discovered storage/input site".into());
}
```

- The witness match gains `Disposition::Local => LOCAL.into()`; the `owed_by` /
  `external_implementation_coverage` rule stays "only Unbounded carries debt", so both cells
  are `-`.
- Nothing else changes: a `dyn Fn` payload outside the probe root still fails (`Local` is
  fail-closed); a probe-rooted `public input … dyn Fn` row still takes PublicInput (the prefix
  check runs first).

## The chain table (ruling 784 D2)

`crates/rum-stdlib/src/tests/ownership_graph/inventory/emission_chain.tsv`: exactly 152
tab-separated rows `path\tsha256`, sorted by path, unique, digests 64-char lowercase hex. It
is the 768 `CHAIN PASS … final=` set with the 774 carried exceptions (`class_rows.rs` →
`6da1dd03…`, `class_lease_fixtures.rs` → `475d0edb…`, `guest.rs` → `1a2c5ff5…`). Derivation
log: parse the two certificate logs, write the table, then hash every row against the scene
and print `CHAIN TABLE rows=152 scene_match=152`; any mismatch is a STOP. The table is read by
the adoption command only — never by discovery, never by the exactness tests.

## The differ (three-sided, fail-closed)

```rust
fn differ_rows(
    committed: &[String],
    discovered: &[String],
    review: &[String],
    path_set: &BTreeSet<String>,
    admitted_rewrites: &BTreeSet<(String, String, String, usize)>, // (file, old construct, new construct, count)
    admitted_deletions: &BTreeSet<(String, String)>,               // (file, construct)
    ledger_ok: impl Fn(&str) -> Result<(), String>,
) -> Result<(), String>;
```

1. `review` must equal `discovered` as multisets (byte-identical row strings after the
   maintenance command wrote them); a row in one and not the other is "fabricated" or
   "missing from the review copy".
2. `committed` vs `discovered` pair by the identity key `(kind, file, construct)` with `@\d+`
   → `@ANCHOR` and `ends at N:M` → `ends at POS`, as multisets. A pair whose only deltas are
   line, column, `@N` and the ends-at span is accepted; every other unpaired committed row
   must consume an `admitted_rewrites` entry (same file, same old construct, same new
   construct) or an `admitted_deletions` entry; every other unpaired discovered row is **new**
   and is accepted only when `path_set` contains its file and, for `UnknownHostPayload`,
   `ledger_ok(row)` is `Ok`.
3. Every admission must be consumed exactly once and exactly `count` times; an unconsumed
   admission, or an occurrence beyond it, is an error.
4. Discovery never calls `differ_rows`; the `#[ignore]` commands never install.

## Adopted sets (ruling 784 D3/D4)

- Admissions: the two derive-list rewrite pairs — `(rum_derive :: FrameRootTrace ,
  rum_derive :: PureFrameAbandon)` → `(rum_derive :: FrameRootTrace)` in
  `runtime/debug_tools/trace/resume_frames.rs` ×1 and
  `runtime/kernel_data/structure_data/replace/guarded/resume_frames.rs` ×6, and the
  `rum_runtime :: kernel_api :: …` spelling in
  `strings/patterns/search/guarded/replace/resume_frames.rs` ×6 and
  `strings/patterns/search/rules/resume_frames.rs` ×8 — plus the one deletion
  `(strings/patterns/search/rules/cases.rs, unexpanded macro rum_runtime :: resume_or_suspend!
  (ends at 515:18))`. Exact spellings are in `.temp/brief-775/logs/inventory-classes.log`
  (`GONE class=…` and `GONE FILE …` lines).
- Ledger: 92 entries. Fact columns come only from the review copy. The 11 moved entries are
  rekeyed on the discovered five-column row (their identity is unchanged; disposition
  `PublicInput`, witness `PUBLIC_INPUT`, `-`/`-`). The 11 new public-input rows take the same
  four cells. The two storage rows take `Local` and the `LOCAL` witness. The four `Unbounded`
  entries with unchanged fact keys (`typed/action.rs:20`, `typed/action.rs:22`,
  `typed/chain.rs:9`, `typed/entry.rs:11`) take the `verified witness:` text from
  `.temp/brief-775/logs/ownership-ledger.log`. The other 64 entries are unchanged.
- The four `#[ignore]` commands write `target/brief-775/self_reaching.tsv`,
  `host_payloads.tsv`, `unknown_attributes.tsv`, `unknown_captures.tsv`,
  `unknown_macros.tsv`, `unknown_storage.tsv` (one file per construct partition, split by
  construct prefix: `unresolved attribute` / `closure captures` / `unexpanded macro` /
  otherwise storage, sorted with `evidence::sort_rows`), and
  `host_payload_ledger_review.tsv` (discovered host rows with every cell the ruling names
  filled; it chooses no disposition the ruling does not name and never writes the inventory
  path).

## Falsifiers (ordinary hermetic tests; no `graph::discover`, no I/O)

(a) **Unattributable file.** A synthetic new row whose file is not in the synthetic path set
    is rejected by `differ_rows`; the same row with its file in the path set and
    `ledger_ok = Ok` (host case) is accepted.
(b) **Mutated identity.** The `resume_frames.rs:51:1` derive pair fails with empty admissions,
    passes with its exact admission, and fails again when the new construct is mutated with no
    fresh admission.
(c) **Fabricated row.** A review list differing from the discovered list fails; equal
    multisets plus the admitted rewrites and deletion pass.
(d) **Missing or wrong disposition, four arms.** A host row absent from the candidate ledger
    fails `ledger_ok` (check 1); each of the two `dyn Fn` rows is rejected as `PublicInput`,
    `Declared` and `Unbounded` and accepted as `Local` with the exact `LOCAL` witness; a wrong
    Local witness fails `check_entry`; a non-probe `dyn Fn` row claimed `Local` fails.

Stage B watches both arms of each falsifier (the red arm must print the expected error).

## Stages

**Stage A — re-authentication and classification replay (900 s).** Verify the scene walk
`18dca765…`/5,816, the seven table digests/counts, the class table, `provenance.tsv`, and the
two finals; recompute the multiset pairing from the 775 logs with your own script under
`.temp/brief-776/` and require constructs 1,241 pairs / 1,290 new / 22 gone and ledger 11/13/0;
require the 22 gone rows to be the named ones and the two `dyn Fn` rows to be the only
undispositioned. Any mismatch is a STOP. No cargo.

**Stage B — commands, chain table, differ, `Local`, falsifiers (1,800 s).** Implement the
sections above; derive and verify the chain table; run:

```
cargo test --locked --offline -j8 --no-fail-fast -p rum-stdlib --lib reconcile -- --nocapture --test-threads=8
```

(the falsifier tests carry names containing `reconcile`; `--no-fail-fast` is a cargo flag,
before `--`). Do not install inventories.

**Stage C — adoption (1,200 s).** Run the four `#[ignore]` commands (exact filters, one cargo
invocation at a time). Build the 92-entry candidate ledger from the review copy with only the
ruling's cells. Then run the adoption command (an `#[ignore]` test): it re-discovers, reads the
review copies and the candidate ledger, calls `differ_rows` per family with the 152-path set,
and re-hashes the chain table; it must print the differ verdicts and `CHAIN TABLE rows=152
scene_match=152` before any install. If every check passes, install the six construct tables
and `host_payloads.tsv` byte-exactly from the review copies (sha256 logged before/after; the
bytes may not be hand-edited) and write the candidate ledger to
`inventory/host_payload_ledger.tsv`. Then:

```
cargo test --locked --offline -j8 --no-fail-fast -p rum-stdlib --lib production_inventory_is_exact -- --nocapture --test-threads=8
cargo test --locked --offline -j8 --no-fail-fast -p rum-stdlib --lib production_host_payload_ledger_is_complete_and_attributed -- --nocapture --test-threads=8
```

Require `EXACT INVENTORY self-reaching types: 61 rows`, `EXACT INVENTORY UnknownHostPayload:
92 rows`, `EXACT INVENTORY UnknownConstruct: 10134 rows` and the ledger test `test result:
ok`. A STOP if the differ rejects anything.

**Stage D — the deferred 774 acceptance (4,200 s), after C is green.** One flag-on attempt,
bound 900 s:

```
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 \
  python3 scripts/run-bounded.py --timeout 900 -- \
  cargo test --locked --offline -j8 --no-fail-fast -p rum-stdlib --test suspension_oracle class_lease -- --nocapture --test-threads=8
```

Expected `test result: ok. 197 passed; 0 failed; 0 ignored`. Then the receipt diff against
`.temp/brief-772/logs/class-lease-fixtures.log` (brief 774 item 7; 600 s; every contract
receipt present, the named exceptions only). Then one flag-off workspace attempt, bound
2,400 s, no `RUM_PROFILE_*`:

```
python3 scripts/run-bounded.py --timeout 2400 -- \
  cargo test --workspace --locked --offline --no-fail-fast -- \
  --skip class_lease_positive_c5_default_profiled_finish --skip class_lease_positive_p0492
```

Every other test green, including `class_lease_positive_p2299` and the two ownership tests;
any other red is a STOP. Then `cargo fmt --check` (300 s); `cargo fmt` is permitted once in
stage B or C only if its diff touches stage-B/C files only.

## Environment

No network, no new dependency, no new `unsafe`, no `RUM_*` except the single flag-on command;
`.temp` only, never `/tmp`; never read `~/.config/ccds/api-key`; no `git`. One cargo command
at a time, `CARGO_BUILD_JOBS=8`, `RUST_TEST_THREADS=8`; workers run cargo directly inside the
dispatch scope (no nested `memory-scope.sh`). On exit 137 with `oom_kill` risen, rerun that one
command once at `-j1`.

## Verification

The `rum-report/1` must quote verbatim: the stage-A pins and the replayed counts; `CHAIN TABLE
rows=152 scene_match=152`; the falsifier verdicts for both arms of each; the three `EXACT
INVENTORY` lines; the ledger test `test result: ok`; the flag-on `197 passed; 0 failed` and its
`FIXTURE DIGEST_KAT`; the receipt-diff verdict; the flag-off workspace result; `cargo fmt
--check`. `scripts/report.py check 776` must print no PROBLEM.

## Checkpointing and stops

Write `.temp/brief-776/PROGRESS.md` after each stage; logs under `.temp/brief-776/logs/`.
STOP if: the scene walk has moved; a row cannot be paired or attributed; an admission is
unconsumed or over-consumed; the chain table does not verify 152/152; a falsifier's red arm is
accepted or its green arm rejected; the install would need a discovery or evidence change; the
flag-on module is not the expected 197/0; a contract receipt is lost; the workspace reds on
any test other than the two named skips; `cargo fmt --check` fails. Never cut the inventory,
the differ or the workspace gate to fit the budget: stop with the captured lines and counts.

## Hard rules

- Do not `git commit`, branch, or push; do not run `git` at all.
- Do not modify discovery to make an inventory match; do not hand-edit a fact column.
- Do not disposition `Bound.live` or `Operations.constructing` by anything other than the
  exact `Local` change above; do not invent a witness.
- Do not auto-accept the 21 derive-list rewrites or the one deletion: they go through
  `admitted_rewrites` / `admitted_deletions`, consumed exactly.
- Q25's thirteen rows stay named stops; 744/745/746 stay held; CM5 stays unauthorized.

## Budget

**Focused 8,100 s + workspace 0 + epoch 0**, carved from the stage reserve after brief 775
(30,362.36678847017 → **22,262.36678847017 s**). Component bounds: stage A 900 s; stage B
1,800 s; stage C 1,200 s; stage D 4,200 s (flag-on 900 s, receipt diff 600 s, workspace
2,400 s, `cargo fmt --check` 300 s). Unspent returns. A projected overrun stops for a larger
carve, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-776/report.json`, stage = the furthest stage completed (`A` to
`D`), via `scripts/report.py`; `check` must print no PROBLEM. The summary names the stage, the
adopted counts, the `Local` rows, the differ verdicts, and the acceptance totals.

## Gate c135 (2026-11-23)

Brief 776 ran on `grk` (grok-4.7, effort xhigh), unit
`brief-776-grk-20261123-190804-1256163`, session `262410e3-467b-468b-89b2-e9414085236e`,
19:08:04–20:22:57 CST, `status=success turns=138 cost_usd=10.03322 refused=0`, rc 0,
`duration_ms=4474340`, peak 10,306,478,080 B (`oom=0 oom_kill=0`); report STOP
(`check: OK, 105 citations verified`; 11 changes, 2 added / 9 modified, 1 disclosure,
2 deviations). Accepted through stage C; stage D's STOP accepted as the correct terminal
state for the scene.

- **Stage A** re-authenticated the carried scene (walk `18dca765…`/5,816; the seven tables;
  class table `8c8b2232…`; `provenance.tsv` `965eca10…`; finals `6da1dd03…`/`475d0edb…`) and
  reproduced my own c134 re-pairing (`pairs=1241 new=1290 gone=22`, `11/13/0`,
  `only_dyn_fn=true count=2`, `WITNESS rows=4`, `problems=0 ok=true`).
- **Stage B** implements exactly ruling 784 D1 (read in the diff: `LOCAL`, `Local`, the
  bi-conditional, the witness arm, visibility, no other change) and the 1,090-line
  `reconcile.rs` (three-sided fail-closed differ, review commands that never install, the
  adoption command, four hermetic falsifiers). Falsifiers: `4 passed; 0 failed; 5 ignored`
  with every red and green arm printed. The chain table, derived independently here from the
  768 certificate plus the three carried overrides, is 152 rows, sorted, unique, equal to the
  staged file, and `CHAIN TABLE rows=152 scene_match=152`.
- **Stage C** adopted: six `DIFFER VERDICT … ok`, `LEDGER COUNTS rows=92 PublicInput=79
  Unbounded=8 Declared=3 Local=2`, eight `INSTALL … equal=true`, then `EXACT INVENTORY
  self-reaching types: 61 rows`, `UnknownHostPayload: 92 rows`, `UnknownConstruct: 10134
  rows`, and the ledger test green. My independent ledger audit: 68 pairs / 11 gone / 24 new
  (11 rekeys + 11 public-input + 2 Local), exactly four cell changes (the four `Unbounded`
  witnesses equal to the 775 `verified witness:` lines), 64 unchanged.
- **Stage D**: flag-on `test result: ok. 197 passed; 0 failed` under
  `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` with `FIXTURE DIGEST_KAT PASS` present; receipt diff
  `rows=97 problems=1` where the one problem is the checker's `startswith` line rule under
  parallel capture (**F3**; no receipt lost); fingerprint replay `changed=68 problems=0`
  (my column check: only column 19 changed); workspace exit 101, 4,488 passed / 85 failed /
  71 ignored — 9 environmental loopback refusals and the rest the scene's own baseline gap
  (`unreconciled source file: crates/rum-runtime/src/cm740_probe/class_rows.rs`, source
  identity preimages, recursion/call-site inventories), **F5** the hygiene red from a
  `#[cfg(test)]` module with `println!` in `lib.rs`, and `Q25_SELECTOR … Err(NotPresent)`,
  none in the 776-edited set; `cargo fmt --check` red only on `q25_ownership.rs` and
  `q25_p2379_proof.rs`, byte-unchanged by this brief.
- **F4**: the ledger test's printed composition lumps the 2 `Local` rows into "10 Unbounded"
  (message only; the checks are green).
- **Scope**: the scene's changed set is exactly the 12 expected paths; the live tree is
  untouched (no live file newer than the dispatch; the 23 inherited entries unchanged); all
  writes under `.temp/brief-776/` and the scene's `target/brief-775/`; 3 git invocations at
  teardown, all denied.
- **Budget**: 4,493 s charged of 8,100; 3,607 s returns; reserve 22,262.36678847017 →
  25,869.36678847017 s.
- The deferred 774 acceptance's workspace/fmt gates move to the constructed baseline; F3/F4/F5
  are its scope. Q25's thirteen rows stay named stops; 744/745/746 stay held.
