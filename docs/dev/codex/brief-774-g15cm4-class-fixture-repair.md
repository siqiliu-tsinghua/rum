# Brief 774 — G15-CM4: the class-fixture repair — P2299's type_name identity, the contract positive assertion, and the profiled class-lease run

**Director-dispatched, one implementation stage, one gate.** This continues ruling 782.
Brief 773's inventory is accepted: P2299 misses by the generic `type_name` spelling, C5 by the
flag-off arm, P0492 by the token-None arm, and `positive_receipt` pins no contract event. This
brief repairs exactly that: P2299's recipe frame becomes the exact `type_name` suffix, the
positive assertion becomes `contract_hit` (which proves the row's own contract event), and the
class-lease module is run once under the campaign's observation environment. Every seam, the
class table, `provenance.tsv`, the q25 files and `kind_matches` stay byte-identical.

## Goal

When this stage is done, in the carried scene under `.temp/brief-774/scene/`: the class-lease
module is green — all 97 positive rows and all 97 withheld rows green, the digest KAT green, the
importer clause green, and one new falsifier test green — with `P2299` passing on the exact
`type_name` suffix, `C5.default.profiled.finish` passing on a committed `EvalError::InputRequired`
Return receipt from `suspend_profiled_cm740_driven`, and `P0492` passing on a committed
allocation of `ProfiledPreparedDispatchFrame`, both reached by running that one test with
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1`. The positive predicate accepts only the row's contract
event. The updated generator copy reproduces the two generated files byte-for-byte. The
flag-off workspace suite is green with exactly the two named switch-required positives skipped.
Q25 stays open.

## Context pointers

- Read `AGENTS.md`; rulings `.temp/director/ruling-782.md`, `ruling-781.md`, `ruling-780.md`;
  brief 773's report `.temp/brief-773/report.json` and logs under `.temp/brief-773/logs/`;
  the drafted source `.temp/brief-773/draft-brief-774.md`.
- The carried scene walk `12a687e2edc347de64575511695534894430fc2599ea09f9afe3d433b1909184`,
  5,816 files; class table `8c8b22321d5434a4a3a38365eb533c27a1bfc36353ccbb5b72fa0d156e6cfcb7`
  (4,803,954 B); scene `provenance.tsv`
  `965eca10f966966e791f0974aa2dcba21c85fdfbe50c244ca246e1264985f10f`; fixture before this
  stage `3738e89432a37928e5b5a08d96dcd2c821a1be3a55a2fa254e50231c56993621` (41,447 B).
- The flag-off control, already measured, never rerun: `.temp/brief-772/logs/class-lease-fixtures.log`
  — `test result: FAILED. 193 passed; 3 failed; 0 ignored; 0 measured; 592 filtered out;
  finished in 22.05s`, `FIXTURE POSITIVE_MISS` on `C5.default.profiled.finish`, `P0492`,
  `P2299`; `.temp/brief-772/logs/invocation.log` `INVOCATION EXIT code=101 elapsed_s=75.979`.
- The contract records stay where they are and are read-only: `.temp/brief-756/frame-authorization.tsv`
  (rows 2/41/84), `.temp/brief-756/row-verdicts.log` (lines 1/40/83), `.temp/brief-756/class.md`.
- The campaign observation environment: `.temp/brief-743/design/breadth.py:73-80` sets
  `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` and `RUM_CM740_PROFILE=on`; briefs 705/739 carry the
  required-switch suite convention (the direct suite with disclosed skips, then the switch
  suites separately). That is why the class-lease evidence run carries the flag and the
  workspace gate does not.
- Key scene files (read; edit only what Scope allows):
  - `crates/rum-runtime/src/cm740_probe/class_lease.rs` — `kind_matches` 117-118; recipe frame
    uses at 139/149/265/390/431/437. Do not edit.
  - `crates/rum-runtime/src/cm740_probe/class_rows.rs` — `Recipe` 1-13; P2299 at 999-1008,
    frame line 1001; P0492 at 483-492; P0508 at 495-498 (`frame: "No"`).
  - `crates/rum-stdlib/tests/suspension_oracle/class_lease_fixtures.rs` — `positive_receipt`
    12-28, `run_case` 30-69 (asserts 63-65), digest KAT 79-85.
  - `crates/rum-runtime/src/runtime/definitions/application/down_values/default_fallback.rs`
    — `suspend` 125-135, `suspend_profiled_cm740_driven` 237-264. Do not edit.
  - `crates/rum-runtime/src/runtime/eval_core/call.rs` — token gate 36-52,
    `suspend_profiled_prepared_stage` 138-152, struct 1226. Do not edit.
  - `crates/rum-stdlib/src/schema/continuation.rs` — struct 16, `Vec<usize>` impl 38,
    construction 33, consumer hit 106. Do not edit.
- The generator to copy, never modify in place: `.temp/brief-766/generator/class_forms.py`
  (`FRAME_TOKEN` 38, `frame_token` 70-72, recipe assignment 317, source needles 133/145-146/
  211/280-281, `write_rows` near 1681-1682, `write_fixture` 1164+, `cargo_fmt_files` 1449-1478)
  and its companions under `.temp/brief-766/generator/`.

## Scope

- Files you MAY modify, and only inside the carried scene under `.temp/brief-774/scene/`, plus
  scratch under `.temp/brief-774/`:
  - `crates/rum-runtime/src/cm740_probe/class_rows.rs`: the P2299 `frame` string only
    (`"SchemaValidationSuspensionFrame"` → `"SchemaValidationSuspensionFrame<alloc::vec::Vec<usize>>"`).
  - `crates/rum-stdlib/tests/suspension_oracle/class_lease_fixtures.rs`: rename the old
    predicate to `legacy_positive_receipt` (kept, called only by the falsifier), add `kind_of`
    and `contract_hit`, use `contract_hit` in `run_case`, and add one falsifier test. The
    withheld branch keeps the old predicate.
  - `.temp/brief-774/generator/`: a copy of `.temp/brief-766/generator/` carrying the
    receipt-frame / emission-token split and the updated `write_fixture` template. Do not
    modify `.temp/brief-766/`.
- Files you MUST NOT touch: `class_lease.rs` (`kind_matches`, `note_allocation`,
  `note_return_kind`, `digest`); every seam, including `default_fallback.rs`, `call.rs`,
  `continuation.rs`, `down_values.rs`, `core.rs`; `q25_ownership.rs`, `q25_p2379_proof.rs`,
  the class table, `provenance.tsv`, overlays, the 756/757/759 tables; the recipes of C5,
  P0492, P0260 and every row other than P2299; `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  `docs/**`, `.git`, `Cargo.lock`.
- **Standing inventories.** This stage edits `.rs` files, so the retention fingerprints and the
  ownership graph cover the edit. Replay them after the edits and before the first acceptance
  suite. Quote the replay verdict lines in the report. Do not hand-edit an inventory. A suite
  red whose only cause is a stale fingerprint, certificate, or position cell is an ordering
  artifact: the replay is the fix, and it is not a regression. Fingerprints of the two edited
  files may move. STOP if a row committed before this stage changes identity, status,
  disposition, or exclusion, or disappears; if a new `UnknownHostPayload` appears; or if a new
  `UnknownConstruct` points at code this stage did not touch. This stage adds no frame type, so
  `frame_root_trace` closed lists and the named-frame count stay. LP01 statement text and
  typed-slot insertion sites stay because no producer body changes; if one of those moves,
  STOP.
- **Environment rule.** The class-lease evidence command sets
  `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` in the process environment. The inventory replays and
  the workspace gate do not. Do not set `RUM_PROFILE_LEXICAL_DEFER`, `RUM_CM740_PROFILE` or
  any other `RUM_*` variable.

## Deliverable

1. **Scene.** Copy `.temp/brief-772/scene` to `.temp/brief-774/scene` excluding `target`
   (brief 772 measured `SCENE COPY rc=0 elapsed_s=1.750`). Recompute the walk and the four pins
   above. STOP on any mismatch. Do not rebuild the scene from the live tree.
2. **P2299 recipe identity.** In `class_rows.rs`, P2299's `frame` field becomes exactly
   `SchemaValidationSuspensionFrame<alloc::vec::Vec<usize>>`. No other recipe field changes.
   `kind_matches` already accepts that suffix against the observed kind (`.temp/brief-773/logs/generic.log`
   `IDENTITY P2299 … matches_type_name_suffix=true`); the 756 short spelling does not match
   (`matches_756_short=false`). Do not write the short spelling; do not edit the 756 row.
3. **Generator copy, split and plan check.** Copy `.temp/brief-766/generator/` to
   `.temp/brief-774/generator/`. `frame_token` stays the emission token (bare identifier, so
   `impl_spans`' `for {frame}`, `source_edge`'s `{frame} {`/`{frame}<` and the seam planning
   are unchanged). Add `receipt_frame(row, frame_object)`: for `P2299` return
   `SchemaValidationSuspensionFrame<alloc::vec::Vec<usize>>`; for every other row return
   `frame_token(frame_object)`. The recipe emitter writes `receipt_frame`; the seam emitters
   keep `frame_token`. The updated `write_fixture` must carry the same `contract_hit`,
   `kind_of`, renamed `legacy_positive_receipt` and falsifier test as the scene file.
   **Plan check, log-only:** on a scratch tree under `.temp/brief-774/` (declare exactly what
   `COPY` points at), compute the located rows as the 766 emission did, render
   `write_rows(located)` and `write_fixture(located)`, canonicalize with the generator's own
   `cargo_fmt_files`, and require the result to equal the two scene files byte-for-byte. Also
   require the emission tokens (`frame_token(frame_object)` for all 97 rows) to equal
   `.temp/brief-766/evidence/class-forms.json`'s `frame`, and the 27 seam files' digests to be
   unchanged from the carried scene. If the plan cannot be recomputed, STOP and report — no
   hand-edit-only path, no weaker check. Do not run a 27-file re-emit; if any command rewrites
   a seam, restore it from the pre-edit copy and STOP.
4. **The positive predicate.** In `class_lease_fixtures.rs`:

```rust
fn kind_of(line: &str) -> &str { /* the kind= field, "" when absent */ }

fn contract_hit(group: &str, frame: &str, lines: &[&str]) -> bool {
    fn receipt(line: &str) -> bool {
        line.starts_with("CLASS_RECEIPT ")
            && line.contains(" commit=true ")
            && line.contains(" kind=")
            && line.contains(" issuer=")
            && line.contains(" owner=")
            && line.contains(" occurrence=")
            && line.contains(" slot=")
            && (!line.contains(" issuer=Return ") || line.contains(" kind=EvalError::InputRequired "))
            && (line.contains(" issuer=Return ") || !line.contains(" generation=0 "))
    }
    let consumed = lines.iter().any(|line| line.starts_with("CLASS_CONSUME "));
    if !consumed {
        return false;
    }
    match group {
        "synchronous" => lines.iter().any(|line| {
            receipt(line)
                && line.contains(" issuer=Return ")
                && line.contains(" kind=EvalError::InputRequired ")
        }),
        "cursor" => {
            lines.iter().any(|line| {
                line.starts_with("CLASS_CURSOR ") && line.contains(" accept=true")
            }) && lines.iter().any(|line| receipt(line) && kind_matches(kind_of(line), frame))
        }
        "queued-success" => {
            lines.iter().any(|line| {
                line.starts_with("CLASS_QUEUED_PERMIT ")
                    && line.contains(" paired=true")
                    && !line.contains(" withheld=true")
            }) && lines.iter().any(|line| receipt(line) && kind_matches(kind_of(line), frame))
        }
        "typed" | "raw" => lines.iter().any(|line| {
            receipt(line)
                && !line.contains(" issuer=Return ")
                && kind_matches(kind_of(line), frame)
        }),
        _ => false,
    }
}
```

   `kind_matches` is copied verbatim from `class_lease.rs:117-118` as a local helper (the
   fixture must not reach into the runtime's private predicate). `run_case` reads `group` and
   `frame` from the `CLASS_INSTALL` line whose `commit=` equals this case's commit flag (the
   install line is emitted by `install` from the recipe, before both halves). Positive mode
   asserts `contract_hit(group, frame, &lines)` and still asserts a `CLASS_CONSUME` line. The
   withheld branch keeps `legacy_positive_receipt`: `hits == 0`, no line for which it is true,
   no `CLASS_CONSUME`; a withheld `CLASS_CURSOR accept=true` must not by itself fail withheld
   mode. Do not make `RECIPES` public (`Recipe` is `pub(super)`).
5. **Falsifier test** `class_lease_contract_predicate_falsifiers`, no guest run, no seam edit.
   Assert `legacy_positive_receipt` accepts and `contract_hit` rejects each bad line, and
   `contract_hit` accepts each good line:
   - Wrong kind: a committed `...PreparedDispatchFrame` receipt plus a consume, group `typed`,
     frame `ProfiledPreparedDispatchFrame`: legacy true, `contract_hit` false.
   - Missing consume: the P0492 profiled-kind receipt with no `CLASS_CONSUME`: false.
   - Withheld: the same receipt with `commit=false`: both predicates false.
   - Neighbour C5: a committed `DirectDefaultFallbackFrame<false>` receipt plus a consume,
     group `synchronous`: legacy true, `contract_hit` false.
   - Neighbour P0289: a committed `...EvaluateDefinitionRhsSuspensionFrame` receipt plus a
     consume, group `synchronous`: legacy true, `contract_hit` false; the same lines plus the
     committed Return `EvalError::InputRequired` receipt and its consume: true.
   - P2299 identity: kind `rum_stdlib::schema::continuation::SchemaValidationSuspensionFrame<alloc::vec::Vec<usize>>`,
     group `raw`, frame `SchemaValidationSuspensionFrame<alloc::vec::Vec<usize>>`, plus a
     consume: true. Kind `path::SchemaValidationSuspensionFrame<Other>` with that frame:
     false. The Vec kind against frame `SchemaValidationSuspensionFrame`: false.
6. **One flag-on evidence run.** From the carried scene, one attempt:

```
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 \
  python3 scripts/run-bounded.py --timeout 900 -- \
  cargo test --locked --offline -j8 --no-fail-fast -p rum-stdlib --test suspension_oracle class_lease -- --nocapture --test-threads=8
```

   `CARGO_BUILD_JOBS=8`, `RUST_TEST_THREADS=8`. The 900 s bound covers the rebuild (the 772
   from-scratch build was 52.83 s, the 196 tests 22.05 s) with room. Acceptance:
   `test result: ok. 197 passed; 0 failed; 0 ignored; …; 592 filtered out`. On exit 137 with
   `oom_kill` risen, rerun that one command once at `-j1`. A red that is not the named
   falsifier, or a second functional failure after an oom retry, is a STOP. No second tuning
   pass. Do not rerun the flag-off command.
7. **Receipt diff, log-only Python.** Compare the new log with `.temp/brief-772/logs/class-lease-fixtures.log`
   per row. A synchronous row must still have a committed `EvalError::InputRequired`
   `issuer=Return` receipt. A typed, raw, cursor, or queued-success row that passed on a
   `kind_matches` frame receipt in the 772 log must still have that contract receipt, except
   P2299, whose new contract kind is the type_name suffix, and P0492, whose new contract kind
   is `ProfiledPreparedDispatchFrame`. C5's new contract receipt is the Return line, not
   `DirectDefaultFallbackFrame<true>` and not `<false>`. Neighbour-frame kinds on the ten extra
   rows may change. Losing a contract receipt, or a new positive miss, is a STOP. Withheld
   rows stay at hits 0 with no consume and no committed receipt. `FIXTURE DIGEST_KAT PASS`
   stays.
8. **Certificate.** Do not hand-edit a chain cell. Run the existing addition-chain check
   log-only against the carried scene (copy the 768 certificate phase under
   `.temp/brief-774/` if a path must change; logs only). Expected movement is exactly two
   addition finals: `class_rows.rs` leaves `53e192ab0682ccef6bb69bfbb8ab4a757f8e804226fbaefc2cdf24a628d9e576`
   and `class_lease_fixtures.rs` leaves `3738e89432a37928e5b5a08d96dcd2c821a1be3a55a2fa254e50231c56993621`.
   These stay: `class_lease.rs` `8f5ba488be854120e2e540542def29cd2db72805e14356e6748468b99ab6399b`,
   `q25_ownership.rs` `43416be62901a79c797f057b81a7364b668a91099188098250dbeaaaa68f2052`,
   `q25_p2379_proof.rs` `ee789084c85f6794e5267d8b817ec91b089a015705395373e8d6c9daad653bea`.
   `CHAIN SUMMARY files=152 closed=152 stops=0` stays, with those two finals updated. If any
   other path's final moves, STOP. Do not rewrite the class table or `provenance.tsv`; if the
   tool cannot run without rewriting them, STOP.
9. **Flag-off workspace gate.** After the replays and the flag-on run:

```
python3 scripts/run-bounded.py --timeout 2400 -- \
  cargo test --workspace --locked --offline --no-fail-fast -- \
  --skip class_lease_positive_c5_default_profiled_finish --skip class_lease_positive_p0492
```

   `CARGO_BUILD_JOBS=8`, `RUST_TEST_THREADS=8`, no `RUM_PROFILE_*` variable. One attempt. The
   two named tests are this stage's switch-required positives (brief 739's convention); every
   other test must be green, including P2299's positive, the other 194 class tests and the
   falsifier. A red anywhere else is a STOP, not a new skip; report its name, its output and
   whether it reproduces at `--test-threads=1`. A timeout is a STOP for a larger carve. Do not
   drop this gate and do not export the profile variable to make it pass.
10. **Formatting.** `cargo fmt --check` in the carried scene on the two edited files (or the
    workspace if the tool requires it), record the exit.

## Must not change

- `digest(b"abc")` stays `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`.
- `kind_matches` stays exact or `::`-suffix; a bare recipe name must not match an instantiation.
- The 27 seam files stay byte-identical to the carried scene; overlays stay
  `73297529dff14b93ab70129ce81b6dd9f17cd00171028e46902e5a8cef89b739`; the class table and
  `provenance.tsv` pins above stay.
- Q25's thirteen rows stay named stops: P0214, P0216, P1070, P2348, P2354, P2379, P2381,
  P2385, P2387, P2409, P2411, P2416, P2418. A green fixture does not close them. 744/745/746
  stay held. CM5 stays unauthorized.
- Withheld mode stays red for a committed receipt and for a consume.
- No new `unsafe`. No default-path cost. The profile flag stays the existing `AtomicU8` cold
  load (`config.rs:569-572`).

## Forbidden

- Base-name matching. It would authorize `SchemaValidationSuspensionFrame`'s default `HashMap`
  index (`continuation.rs:16`) and both `DirectDefaultFallbackFrame<false>` and `<true>`.
- Retargeting C5 at `DirectDefaultFallbackFrame<false>` or `<true>`, or P0492 at
  `PreparedDispatchFrame`, or P0260 at `TryApplyDownValuesFromSymbolSuspensionFrame<LinearDownValueScan>`.
- Editing the `<false>` arm so it calls `note_input_return`.
- Editing 756/757/759 rows, weakening the withheld predicate or the falsifier, or hand-editing
  an inventory, a chain cell, the class table or `provenance.tsv`.
- A second flag-off cargo run of the class-lease module, or a second flag-on run to tune the
  predicate; a hand-edit-only path instead of the generator plan check.
- Setting any `RUM_*` variable outside the single flag-on class-lease command.

## Why the flag, and why it is not yet sufficient for C5

`runtime_path_timing_start_adopted` returns `None` immediately when `lexical_profile_enabled()`
is false (`timing.rs:156-158`). `suspend_prepared_stage_with_boundary` then constructs
`PreparedDispatchFrame` (`call.rs:48-52`). `ProfiledPreparedDispatchFrame` is constructed only
in `suspend_profiled_prepared_stage` (`call.rs:152`). The 772 log has that name twice, both
`CLASS_INSTALL` echoes, and zero `kind=` fields.

`suspend` calls `suspend_frame::<false>` before any `driven()` check when the flag is false
(`default_fallback.rs:125-135`). The only `note_input_return("C5.default.profiled.finish")` is
in `suspend_profiled_cm740_driven` (`default_fallback.rs:258-264`), reached only when the flag
is on, `direct_default_fallback_profiled()` is true (`logical/mod.rs:1237-1244`, phase
`DownValuesDirectDefaultLookup`), and `driven()` is true. `InputRequired` does not complete
that phase (`timing.rs:700-703`). The phase is opened at `down_values.rs:69` and `:160`.

The test binary compiles rum-runtime with `test-builtin-memory-growth` and
`test-continuation-slots` (`rum-stdlib/Cargo.toml` dev-dependency). `lexical_profile_enabled`
then calls `admit()` (`config.rs:4-12`). `guest.rs:1959` holds `logical_profile_test_acquire`
across `begin` at 1973, and `lease()` registers that thread (`lease.rs:38`), so `admit()`
succeeds on the evaluating thread while the lease is live (`lease.rs:248-253`). Set the
variable in the process environment before cargo. `logical_profile_test_acquire` does not
store `LEXICAL_PROFILE_ENABLED` (`logical/mod.rs:637-640`). `logical_profile_reset` resets
counters only.

Flag-on is necessary for both rows. It is sufficient for P0492 if the existing token gate runs,
because both call sites pass `runtime_path_timing_start_adopted` (`call.rs:400` and `:527`).
It is not proven sufficient for C5: the phase must be open and `driven()` must select
`suspend_profiled_cm740_driven`. STOP if the flag-on log has no committed Return receipt for
C5 or no `kind=` of `ProfiledPreparedDispatchFrame` for P0492.

## Verification

Replay standing inventories first. Then the one flag-on class-lease command. Then the workspace
command, flag off. Then `cargo fmt --check`. Paste `test result:` lines, `FIXTURE DIGEST_KAT
PASS`, the three new contract receipt lines (C5 Return; P0492 profiled; P2299 Vec suffix), the
receipt-diff verdict, the two moved chain finals and the plan-check lines (`PLAN RECIPE rows=97
equal=true`, `PLAN EMIT tokens=97 equal=true`, `PLAN FIXTURE bytes_equal=true`, `PLAN ROWS
bytes_equal=true`, `PLAN SEAMS unchanged=27`).

## Checkpointing

Write `.temp/brief-774/PROGRESS.md` after the pin check, after the edits and the plan check,
after the flag-on run, and after the workspace run. Logs under `.temp/brief-774/logs/`.

## Hard rules

- Do not git commit, branch, push, or run git. The director owns git.
- Scope is the list above. A file outside it is a STOP, not a quiet edit.
- No network. No new dependency. Scratch under `.temp/brief-774/` only, never `/tmp`.
- No new `unsafe`.
- Q25 stays open. 744/745/746 stay held. CM5 stays unauthorized.
- Do not weaken the fixture, the withheld predicate or an inventory.

## Stop lines

- An entry pin mismatches the 772 walk or the four hashes above.
- The plan check cannot be recomputed, or one of its five equality lines is not `equal=true`.
- The flag-on module is not green, or lacks C5's committed `EvalError::InputRequired`
  `issuer=Return` receipt, or lacks a `kind=` field containing `ProfiledPreparedDispatchFrame`
  for P0492.
- Any 772 contract-event pass loses that receipt, or a new positive or withheld failure
  appears, or the digest KAT fails.
- A seam file, the class table, `provenance.tsv`, a q25 file, or `class_lease.rs` changes bytes.
- The chain moves a final other than the two named additions, or does not close 152/152.
- A new `UnknownConstruct` points at code this stage did not touch, or a committed inventory
  row changes identity.
- The flag-off workspace gate reds on any test outside the two named skips (including P2299's
  positive), or a timeout occurs.
- The projected command time exceeds the remaining focused budget. Stop for a carve. Do not cut
  the workspace gate or the replay.
- Q25 would be reported closed.

## Budget

**focused 6,000 s + workspace 0 s + epoch 0**, carved from the reserve after brief 773
(34,556.36678847017 → **28,556.36678847017 s**). Component bounds: the flag-on class-lease
module 900 s; the generator plan check 900 s; the standing replays 900 s; the certificate and
the receipt diff 600 s; the flag-off workspace 2,400 s; formatting and slack 300 s. Unspent
returns. A projected overrun stops for a larger carve, never a scope cut.

## Report format

`rum-report/1` at `.temp/brief-774/report.json`, stage `""`, via `scripts/report.py`;
`check` must print no PROBLEM. The summary names the P2299 frame string, the environment
variable, the predicate change, the test counts, the two moved chain finals and the two skipped
tests.
