# Brief 716 — the byte-delta witnesses: ownership anchors for the raw windows

**Type:** implementation. **Scope:** the seven witness rows B01/B02/B05-B09
and their test-only plumbing, plus standing replays. Product accounting,
destruction, leases, charged fields and every public shape are out.

## Goal

Brief 715 proved the defect and the repair. A witness that asserts the delta
of the process-global `PROCESS_BYTES` counter across a window which no
drained lease makes quiescent cannot distinguish its own refund from another
test's; the same retirement observed on its **owning** `Budget` can. Convert
exactly the seven raw-lease rows to the owning Budget, keep the nine global
ones untouched, and make the repaired collector carry a permanent calibrated
foreign refund so a regression to the process anchor reds the first draw.
The test-name set and every existing semantic assertion stay unchanged.

## Context pointers

- Brief 715: `docs/dev/codex/brief-715-byte-delta-witness-isolation.md`
  (its ruling at the end is binding), the report
  `.temp/brief-715/report.json`, the inventory
  `.temp/brief-715/class-inventory.md` (rows B01-B16), the calibration table
  `.temp/brief-715/calibration-table.md`, and the measured artifacts
  `.temp/brief-715/evidence/` (`injection.diff`, `missing-boundary-refund.diff`,
  `runs.jsonl`, the per-run JSON receipts).
- The counter and its owners:
  `crates/rum-runtime/src/runtime/profiling/logical/budget.rs` — `Budget`
  `:15`, `Storage` `:20`, claims `:45-57`, `Budget::drop` `:60-66`,
  `Storage::resize` `:68-83`, `Storage::drop` `:86-91`, `process_bytes`
  `:10-13`, and the retained rows `:97-112` (B03), `:115-160` (B04),
  `:162-186` (B05).
- The epoch: `logical/epoch.rs` — `Epoch.budget`, `RECORD_PATH_STORAGE_BYTES`
  `:14-15`, `for_test` `:125-146`, `storage`/`storage_admitted` `:149-199`,
  `reset_unleased` `:1102-1150`, `reset_for_lease` `:1170-1187`.
- The witnesses: `suspension/core/profile_queue/tests.rs` (`:238/:254` B01;
  `:409-499` B02, samples `:451`/`:480`, assertion `:492`);
  `profiling/tests/brief713_cleanup.rs` (`:27/35/39` B06, `:86/93/104` B07,
  `:144-174` B08); `profiling/tests/lease_release.rs` (`:1-32` B09).
- The mounts: `logical/mod.rs:1466` and `:1548` are the `#[cfg(test)]
  #[path = "../tests/…"]` shape to follow for the new helper module.
- `AGENTS.md`, `docs/dev/codex/TEMPLATE.md`, `docs/testing-strategy.md`,
  `docs/code-organization-guidelines.md`.

## Scope

MAY:

- `crates/rum-runtime/src/runtime/profiling/logical/budget.rs` — the cfg(test)
  probe and snapshot, and B05's observations only.
- `crates/rum-runtime/src/runtime/profiling/logical/epoch.rs` — cfg(test)
  delegating wrappers only; no new field, no changed executable body.
- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` — the cfg(test)
  re-export and the new helper's `#[cfg(test)] #[path]` mount only.
- `crates/rum-runtime/src/runtime/profiling/tests/budget_witness.rs` (new) —
  the test-only foreign-refund helper; keep it under 10k characters.
- `crates/rum-runtime/src/runtime/suspension/core/profile_queue/tests.rs` —
  B01/B02 observer replacement and the permanent injection.
- `crates/rum-runtime/src/runtime/profiling/tests/brief713_cleanup.rs` —
  B06/B07/B08 observations only.
- `crates/rum-runtime/src/runtime/profiling/tests/lease_release.rs` — B09
  observation only; B10/B11 unchanged.
- Standing generated fingerprints, positions and certificates **only by
  replay**, every changed row disclosed.

Anything else is a STOP. In particular: no production accounting, drop,
admission, reset or lease body; no charged field in `Budget`, `Storage`,
`Epoch`, `RecordPath` or `ProfileBoundary`; no `Cargo.toml`/`Cargo.lock`
edit; no host, stdlib, doc or brief edit; no new test name, no renamed test,
no skip and no filter.

## Deliverable

1. The cfg(test) observer plumbing in the exact shape below, with no field
   added anywhere and no non-test code path touching it.
2. The seven replacements, each keeping every existing assertion at its
   site: B01, B02 (with the permanent injection), B05, B06, B07, B08, B09.
3. The calibration matrix M1-M6 of the ruling, each mutant in a scratch copy,
   each run at 8 and at 1 thread, every predicted red traced to its named
   assertion, every restoration proven by an identical binary sha.
4. B07's exact expectation from its own receipt — or a STOP if the measured
   refund differs from `RECORD_PATH_STORAGE_BYTES + size_of::<ProfileBoundary>() + 16`.
5. Standing replays before the first acceptance suite, then the acceptance
   runs, the warning matrix, the release-erasure certificate and a
   `rum-report/1` report.

## Target shape and invariants

One observer, no second ledger. In `budget.rs`, cfg(test) only:

```rust
#[cfg(test)]
pub(in crate::runtime) struct BudgetProbe(std::sync::Weak<Budget>);

#[cfg(test)]
impl BudgetProbe {
    pub(in crate::runtime) fn bytes(&self) -> Option<usize> {
        self.0.upgrade().map(|budget| budget.bytes.load(Ordering::Relaxed))
    }
}

#[cfg(test)]
impl Budget {
    pub(super) fn test_bytes(&self) -> usize {
        self.bytes.load(Ordering::Relaxed)
    }
    pub(super) fn test_probe(self: &Arc<Self>) -> BudgetProbe {
        BudgetProbe(Arc::downgrade(self))
    }
}
```

`Epoch` gains cfg(test) `test_budget_bytes(&self) -> usize` and
`test_budget_probe(&self) -> BudgetProbe` delegating to `self.budget`; the
probe type is re-exported under cfg(test) from `logical/mod.rs`. Every
declaration and body above carries the attribute itself, not a comment. A
departure from this ownership shape is a STOP, not a local redesign.

The replacements, each with its exact expectation:

| ID | Site (715 numbering) | Required observation |
|---|---|---|
| B01 | `profile_queue/tests.rs:238,254` | `before = epoch.test_budget_bytes()`; after the failed allocation `assert_eq!(epoch.test_budget_bytes(), before)`. Boundary identity, row and empty-arena assertions unchanged. |
| B02 | `profile_queue/tests.rs:409-499` | Sample `epoch.test_budget_bytes()` before the real retirement and after the catch-unwind; `checked_sub(...).expect("owned charges unexpectedly grew in the refund window")`; `assert_eq!(refunded, RECORD_PATH_STORAGE_BYTES + boundary_bytes)` — 248 + 48 = 296. All caught-policy, no-nested-admission, disposal, row and fault assertions unchanged. |
| B05 | `budget.rs:162-186` | `initial`/`parked` from `epoch.test_budget_bytes()`; keep `size_of::<QueueArena>() + 2*size_of::<usize>()` exactly; keep `Arc::strong_count(&epoch) == 1`; capture the weak probe before `drop(epoch)` and `assert_eq!(probe.bytes(), None)` after it. |
| B06 | `brief713_cleanup.rs:27,35,39` | Capture the old Budget's weak probe before `drop(old)`; before the joined cleanup `assert!(probe.bytes().expect(...) >= 32 * 1024)`; after the join `assert_eq!(probe.bytes(), None)`. Rows and faults unchanged. |
| B07 | `brief713_cleanup.rs:86,93,104` | Sample the retained **old** epoch's Budget across the boundary drop; `assert_eq!(refunded, RECORD_PATH_STORAGE_BYTES + size_of::<ProfileBoundary>() + 16)`; old calls/abandoned/open, both faults, rows and alias assertions unchanged. |
| B08 | `brief713_cleanup.rs:144,146,150,160,169-174` | All four samples on the same retained **old** Budget; handoff `== RECORD_PATH_STORAGE_BYTES`, drop `== size_of::<ProfileBoundary>() + 16`, and their sum as its own third assertion. Both windows stay separate; never sample the replacement epoch. |
| B09 | `lease_release.rs:22,25-30` | Keep the growth assertion at `:22`. Capture the current epoch's weak probe after the Call and drop the temporary `Arc<Epoch>`, then `drop(enabled)`, then `let _lease = test_support::lease();` and **there** `assert_eq!(probe.bytes(), None)` — the handoff reset inside `lease()` is what retires and drops the old epoch (`epoch.rs:1103-1150`). |

Weak absence is only ever asserted after a synchronous owning drop or a
join; no concurrent weak count stands in for a destructor-completion signal.
No observer keeps a strong `Arc<Budget>` or `Arc<Epoch>` across its own
window. Every old owner is sampled through its original Budget, never the
current epoch or the current generation.

B02's permanent control: a second thread temporarily admits the current
generation, claims exactly 4096 bytes in a **separate** Epoch's Budget,
restores its unregistered state, and waits; the parent takes its first
sample, releases the worker so the real `Storage` drop lands inside the
window, and joins; the foreign Epoch outlives both samples. Assert the
worker's own foreign-Budget delta is 4096, that it is unregistered after the
drop, and that its Budget reads 488 after the join. The helper's interface
is test-only:

```rust
pub(in crate::runtime) struct ForeignRefund { /* channels, join, foreign Epoch */ }
impl ForeignRefund {
    pub(in crate::runtime) fn charged(lease: &TestLease, bytes: usize) -> Self;
    pub(in crate::runtime) fn refund_and_join(&mut self);
}
```

RAII must release and join on an earlier assertion's unwind; no sleep for
synchronisation; no `B715_ANCHOR`-style selector and no process equality in
the live candidate; do not copy 715's `lease_drained()` bootstrap into the
live collector — its raw lease acquisition stays byte-identical.

## Falsifier matrix (mandatory)

Scratch copies only; each mutant at 8 threads and at 1; each predicted red
quoted from its log; each restored candidate re-run green with its
pre-mutant binary sha:

| # | Mutant | Predicted observer | Predicted tuple |
|---|---|---|---|
| M1 | boundary whole-Storage refund withheld through the observer | B02 owned equality | 248 ≠ 296 |
| M2 | the expired arena's Storage withheld through the observer | B05 `probe.bytes() == None` | `Some(…)` after `drop(epoch)` |
| M3 | `release_thread_registration` skipped in `TestLease::drop` | B09 probe after the handoff reset | `Some(header)` |
| M4 | `PROCESS_BYTES.fetch_sub` removed from `Storage::drop`, local kept; focused run of B04 alone (`--exact`) | B04 `PROCESS_BYTES == before` at `budget.rs:159` | `before + own ≠ before` |
| M5 | the 32KiB old-epoch charge removed (test-side) | B06's lower bound | `probe < 32KiB` |
| M6 | the old boundary's refund withheld at B07's site | B07 exact expectation | 248 ≠ 296 |

A mutant that reds a different assertion than predicted is a STOP with both
lines quoted — do not re-predict after the fact. A mutant that does not red
is a STOP; do not weaken the expectation to make it red. M4's run is focused
because a process-global leak poisons later drains in the same process; say
so in the report.

## Cost bar and must-not-change list

- Zero default-path cost: every new item is `#[cfg(test)]` and every new call
  is inside a test. No new `unsafe`, counter, ledger, mutex, allocation,
  atomic or branch in production; no new field in any charged type.
- `RECORD_PATH_STORAGE_BYTES` stays the type-derived expression (248 here);
  `Storage` and `Option<Storage>` stay 16; boundary charge 48; collector
  total 296; `logical/mod.rs:53`'s compile-time size equality untouched.
- The nine retained rows B03/B04/B10-B16 keep their process-global
  assertions exactly; changing one is a STOP.
- The test-name set stays 1413 with 5 ignored and 2 filtered; no test added,
  removed, renamed or filtered, and no assertion deleted, relaxed, given a
  tolerance or a saturating difference.
- No environment override, no serialisation of tests, no extra drain or reset
  to make a window quiet.
- One cargo command at a time, `-j8`, `--offline --locked`, tests at
  `RUST_TEST_THREADS=8`; a red is reproduced at `--test-threads=1` before it
  is called a defect; every long or child-spawning command goes through
  `scripts/run-bounded.py`.
- Bound the calibration: at most 18 bounded runs in total (the 12-row
  reference matrix reusing 715's completed receipts where the shape is
  identical, plus the six mutants), each logged and parsed once. No campaign.

## Verification (mandatory, ordered)

1. Implement the plumbing and the seven replacements; `cargo fmt --check`;
   `cargo check -p rum-runtime --all-targets --offline --locked`.
2. Replay the standing inventories covering the touched files **before the
   first acceptance suite** (retention fingerprints, ownership
   discovery/positions, LP01 producer positions/hashes and losses, typed-slot
   receipts, both certificates). Disclose every changed row; never hand-edit
   or weaken an inventory. A stale derived-cell-only red is an ordering
   artifact, not a regression.
3. The full runtime lib suite at 8 threads and once at 1: `1413 passed;
   0 failed; 5 ignored; 2 filtered`, exit 0, both times. Then two more
   8-thread draws for 3 total, each the same tuple.
4. The M1-M6 matrix, each at 8 and 1, each prediction traced; every
   restoration green with the candidate sha.
5. B07's receipt-or-STOP; B06's present-lower-bound evidence; the weak
   probes' happens-before edges named in the report.
6. The four-build warning matrix: default lib 0 warnings, lib-test the 3
   standing `test_builtins` baselines, and the same pair with
   `--features test-builtin-memory-growth,test-continuation-slots` — 0/0/3/3.
7. `cargo check --workspace --all-targets --offline --locked`. The
   authoritative `cargo test --workspace` belongs to the director's gate.
8. A fresh `cargo build -p rum-runtime --release --lib --offline --locked`
   in a private target, then extract it and the accepted artifact
   `target/release/deps/librum_runtime-c71407e195689fc9.rlib` (sha256
   `d9f2339d05f2babf3c37f6da750e6ec72702febf2d42785a96f600c493642627`)
   and compare all 16 object members: they must be byte-identical. rmeta
   differences are only the known non-source classes (mimalloc
   `__DATE__`/`__TIME__`, build-script `OUT_DIR` hashes), at most seven runs
   of at most 16 bytes. Print the four-column table: the new identifiers
   (`BudgetProbe`, `test_bytes`, `test_probe`, `test_budget_bytes`,
   `test_budget_probe`, `ForeignRefund`) at zero object-code occurrences and
   zero symbols, their rmeta occurrences listed separately. Any object-member
   difference or any non-zero symbol is a scope violation and a STOP.
9. Report and stop for the director's gate. Do not restart brief 714's draw
   campaign; that is the director's next step.

## Checkpointing

Save every log before parsing it; never overwrite a completed attempt. Keep
the scratch tree, its private target, every mutant diff with its sha, the
candidate and restored binary shas, the completed-stage ledger and the
remaining run budget under `.temp/brief-716/`. Re-entry consumes completed
receipts instead of rerunning them. Leave the live candidate free of every
mutant.

## Hard rules

`AGENTS.md` and `TEMPLATE.md` apply; the testing policy is rum-only (this
brief compares rum against itself). No git of any kind, no network, no
third-party engine, no new `unsafe`, no edit outside Scope, no production
shape or public-contract change. Do not weaken an assertion, an inventory or
a test; no new skip or filter. An unexpected product over-release is a STOP,
not a witness adjustment. All scratch under `.temp/brief-716/`, never
`/tmp`. Report every failed attempt and settle every artifact explicitly.
No commit: the director owns the gate.

## Report format

`rum-report/1` at `.temp/brief-716/report.json`, with the exact
command/log/exit/count table, every verdict line copied, every FLAGGED row
settled, and explicit scope, replay, cost and falsifier evidence. Run
`python3 -I scripts/report.py check 716` without any git option; the final
message is exactly `python3 -I scripts/report.py summary 716` output.

## Director gate — 2026-11-09: accepted; the release-identity bar is corrected, not the code

**The STOP is well taken and the bar is mine — the third instance of one
class.** I required a fresh release archive whose 16 object members are
byte-identical to the accepted C28 artifact `d9f2339d…`; the fresh archive
`94172a36…` differed in 14 of 16 members and `lib.rmeta` grew 18,810,704 →
18,811,104 B. I compared the two archives myself:

- Section sets are identical in every member (41,250 sections per side, the
  per-member counts equal).
- After normalizing rustc's `.llvm.<u64>` suffix in section names, **every
  section of every member hashes identically**, except `.strtab`,
  `.symtab` and `.shstrtab`, which carry the names.
- Symbol tables are identical in entry count, value, size, type, bind,
  visibility and index in every member, after the same normalization.
- The suffix moved exactly once per crate, `3721675394416604558` →
  `899173758429542867`, because the crate metadata moved by the 400 bytes
  the `#[cfg(test)]` items record. It is the only difference in the
  archive.

So the object code is provably unchanged, and a raw byte-identity bar
cannot hold across any source change — exactly what brief 714's R2 found
for the rlib sha and Q39 for the fixed release-archive SHA. **Ruling R5:**
release identity on this line is asserted on the section-name-normalized,
content-hashed members plus the normalized symbol tables; a raw rlib or
object byte-identity bar is retired. The certificate is
`.temp/director/gate-716-erasure.json`: 16/16 members identical after
normalization, fresh rlib sha256 `b8b6d951…`, and the four-column erasure
table stands unchanged (the new identifiers occur zero times in object code
and symbols; `BudgetProbe` appears once, in `lib.rmeta`).

**One director edit, disclosed.** `tests/budget_witness.rs` asserted the
foreign Budget's post-refund value against the literal `488` — a value
taken from the observation, which this brief's own pitfalls forbid. It now
captures `baseline` from the foreign epoch at creation and asserts
`remaining == baseline` (`baseline=488` is printed). The meaning is
unchanged: after the drop and join the foreign Budget must read exactly its
own retained charge.

**The derived cells moved because the source moved, and I replayed them.**
My edit invalidated the 68 `local-contracts.tsv` fingerprints and the 2
`promoted.tsv` certificate rows; the first workspace run I made therefore
carried 17 `stale contract fingerprint` reds in `retention_closure`. I
re-ran the canonical generator on the live tree (`regenerate_local_contract_fingerprints
-- --ignored`), the two certificate emissions and the canonical
`merge_promoted` helper, installed their outputs, and re-verified: default
and no-default certificates 1/0 each, census 1/0, ownership exact 1/0,
`-p rum-stdlib --lib` 1111/0/53. This is a replay, not a hand edit.

**Accepted readings.** The test-name set is byte-identical to the 715
reference (1,420 names, `diff` empty). M1-M6 each reddened exactly its
predicted assertion at 8 and at 1 thread — `248 != 296` at
`profile_queue/tests.rs:497`; `Some(120) != None` at `budget.rs:188`;
`Some(4232) != None` at `tests/lease_release.rs:32`; `7104 != 0` at
`budget.rs:159` (focused); `present=1024 minimum=32768` at
`brief713_cleanup.rs:32`; `248 != 296` at `brief713_cleanup.rs:115` — and
every restoration returned to the candidate binary sha `a32e56e6…`. The
warning matrix is 0/0/3/3. The repaired collector under the permanent
4096-byte foreign refund prints `refunded=296 expected=296`, which is the
instrument this whole repair exists to leave behind.

**My gate, on the final tree:** `cargo fmt --check` clean; full runtime lib
`1413 passed; 0 failed; 5 ignored; 2 filtered out` at 8 threads and once at
1; `cargo test --workspace --offline --locked -j 8 --no-fail-fast` with the
seven standing skips `4,212 passed / 0 failed / 61 ignored` over 63
binaries (the accepted 713 total); the six switch-gated host targets
13/13 at 8 and at 1 under `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`;
`-p rum-stdlib --lib` 1111/0/53.

Brief 716 is accepted. Brief 714 restarts from zero as stage 32.
