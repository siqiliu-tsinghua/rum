# Brief 651 — LP06 C2: the Catch incoming carrier

## Goal

Remove the duplicated frozen outcome from `IncomingPathTimingToken`, whose
64-byte extent (against the base legacy token's 56) reserves +16 bytes in the
Catch `resume_catch_after_body` frame. The frozen outcome stays in the
existing `Policy::Incoming` record and is read back at terminal completion
through the existing `Epoch::inspect`; the enabled lifecycle records exactly
as today. When you are done the carrier and its Option measure 56 bytes and
the Catch reservations are at the base 1016 (CLI and stack-release) / 792
(stack-debug), with no new disabled work. LP06 stays unpassed overall until
brief-653.

## Context pointers

- Read `AGENTS.md`, `docs/dev/codex/brief-647-lp06-a2-builtin-catch.md` (the
  A2 Catch contract), and `.temp/director/brief-649-design-verdict.md` plus
  the design report's F3/C2 sections
  (`.temp/brief-649/report-design.json`).
- Source: `crates/rum-runtime/src/runtime/profiling/control_timing.rs:9-99`
  (token, start, recorder), `.../profiling/logical/mod.rs:9-17` (module
  declarations) and `:583-608` (`Call::finish`/`pause`),
  `.../profiling/logical/epoch.rs:217-228` (`inspect`), and the unchanged
  Catch body `.../kernel_data/control_data/nonlocal.rs:111-151`.
- Tests: `.../profiling/tests/control.rs:260-307`
  (`lp06_a2_incoming_policy_cases`) and the A2 Catch cases in the same file.
- B evidence is read-only under `.temp/brief-648/instruments/`; never modify
  or execute a prior campaign script in place.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/profiling/control_timing.rs` — the
  `IncomingPathTimingToken` definition, the incoming start's return, and the
  incoming recorder wrapper and its cold gate ONLY.
- `crates/rum-runtime/src/runtime/profiling/logical/incoming.rs` — NEW, the
  small `impl Call { fn finish_incoming }` module.
- `crates/rum-runtime/src/runtime/profiling/logical/mod.rs` — exactly one
  `mod incoming;` declaration line.
- `crates/rum-runtime/src/runtime/profiling/tests/control.rs` — the two new
  tests named below.
- Replay-generated inventories (by replay only).

Everything else is MUST NOT, in particular: `nonlocal.rs` (every semantic
Catch path stays byte-identical), `record.rs`, `epoch.rs`, `Call`/`CallInner`
/`PhaseEntry`/queue layouts, the legacy timing functions, the Catch
classifier, and every semantic frame.

## Deliverable

1. **Carrier keeps only the legacy token.**

   ```rust
   pub(in crate::runtime) struct IncomingPathTimingToken {
       legacy: RuntimePathTimingToken,
   }
   ```

   The start keeps its gates, its frozen classifier, its `runtime_path_timing_start`
   call and its `open_phase(..., Policy::Incoming(frozen))` exactly as now,
   and returns `Some(IncomingPathTimingToken { legacy })`. The `frozen`
   value is still computed and installed in the policy; only its duplicate
   copy in the token disappears.

2. **Finish reads the existing record.** New
   `profiling/logical/incoming.rs`:

   ```rust
   use super::{Call, Policy};

   impl Call {
       pub(in crate::runtime) fn finish_incoming(self) {
           let policy = self
               .inner
               .epoch
               .inspect(self.inner.record)
               .map(|(_, _, policy)| policy);
           match policy {
               Some(Policy::Incoming(frozen)) => self.finish(frozen),
               Some(_) => {
                   self.inner.epoch.fault();
                   drop(self);
               }
               None => drop(self),
           }
       }
   }
   ```

   and `mod incoming;` in `logical/mod.rs`. A live record whose policy is not
   `Incoming` is an invariant violation: fault the epoch and let `Call`'s drop
   tear the record down. A retired/closed record returns `None`; the existing
   `Call` drop remains authoritative. Never `call.pause()` to obtain the
   policy, never add an accessor, cache or field, and never derive `Cancelled`
   from the legacy Error bucket.

3. **Recorder fast path.** In the cold recorder's terminal branch replace
   `call.finish(token.frozen)` with `call.finish_incoming()`. The pause branch
   (`InputRequired`) is unchanged: `call.pause()`, then insert the boundary
   after the complete remainder. Apply the C1 pattern to the incoming
   recorder: `#[inline(always)]` on the small wrapper, token test first, and
   the existing `if !lexical_profile_enabled() { return; }` as the first
   statement of the `#[cold] #[inline(never)]` body. Keep every other
   statement of that body byte-identical.

4. **Tests** in `profiling/tests/control.rs`, each ending with
   `Session::logical_profile_test_assert_quiescent()`:
   - `lp06_c2_incoming_wrong_live_policy_faults`: start a phase with
     `Policy::Expr` through `logical_phase_start`, call `finish_incoming`,
     and assert the epoch fault counter is 1 and no record stays open.
   - `lp06_c2_incoming_retired_record_drops`: start an Incoming phase, retire
     the epoch (`Retire::DiagnosticCap`), call `finish_incoming`, and assert
     no panic, no new fault, no double settlement and no open record.
   Eprintln one `LP06 C2 NATIVE ...` verdict line per test.

5. **Compiled shape check**, after acceptance is green, exactly as below.

## Must not change

- The frozen outcome is still captured at the existing start statement and
  installed as `Policy::Incoming(frozen)` before form resolution; the record
  stays open through form, matcher, handler and bypass.
- Pause semantics: `InputRequired` takes the phase with the non-InputRequired
  sentinel, calls `call.pause()`, and inserts after the whole remainder's
  frontier. A synchronous terminal completion never pauses.
- The legacy classifier remains at its current line and is passed once to
  `record_profile_runtime_path_timing`; its Error bucket still includes
  Cancelled.
- All five Catch forms, the total/body actual outcomes, the second handler
  pause, and the `Call`/`PhaseEntry`/queue representations.

## Cost bars and falsifier

- `IncomingPathTimingToken` and its Option both measure **56 bytes** (B: 64).
- `resume_catch_after_body` entry reservation: **≤ 1016** CLI and
  stack-release, **≤ 792** stack-debug (B: 1032 / 1032 / 808). A larger value
  is a STOP; report it, do not retune.
- The incoming recorder wrapper symbol is absent (inlined); its cold body
  remains. No new disabled call or lookup.
- The frozen outcome is still stored once: the existing
  `lp06_a2_incoming_policy_cases` five outcomes, including Cancelled, stay
  green; the incoming_ok / incoming_error / premature_finish mutants killed
  by their A2 assertions must be reproduced on this shape from the A2
  receipts (`docs/dev/codex/brief-647-lp06-a2-builtin-catch.md` and
  `.temp/brief-647/attempt2/logs/mutant-incoming_ok-calibration.log`,
  `.temp/brief-647/attempt2/logs/mutant-premature_finish-calibration.log`),
  restored byte-identically, and followed by green.
- A fifth calibration must kill the Cancelled/Error conflation: mutating the
  start's frozen classifier to report `Outcome::Error` for Cancelled must turn
  the Cancelled cell of `lp06_a2_incoming_policy_cases` red; restore
  byte-identically and pass. This pins that the cached-policy read-back keeps
  the logical outcome, not the legacy bucket.
- The enabled-only cost of one `Epoch::inspect` per terminal Incoming
  completion (its lock and its `Owner` Arc clone) is accepted; disclose it in
  the report. Do not add an epoch accessor, a cached policy or a `Session`
  field to avoid it.
- Instruments are supplied in `.temp/brief-651/instruments/`:
  `stage_build.py`, `stage_shape.py`, the six `c2-{base,cand}-{cli,stack-debug,stack-release}.json`
  specs and `c2-incoming-probes.json` (the size probe appended to
  `control_timing.rs` in the candidate stack-debug build only). They pin
  attempt-suffixed artifacts and build both arms at one common absolute path
  (`<root>/b/active`), release CGU=1, offline, jobs 8, under
  `run-bounded 880`. Do not edit them.

## Verification (MANDATORY — run in this order)

First the standing source set from `.temp/brief-645/inventory/commands.tsv`,
unchanged and with the stage root `.temp/brief-651/`:

```
SOURCE-FMT, SOURCE-CHECK, SOURCE-FINGERPRINTS, SOURCE-OWNERSHIP,
SOURCE-CERTIFICATES, SOURCE-CERTIFICATES-NO-DEFAULT, SOURCE-REJECTIONS,
SOURCE-RUSTC-CLAIMS, SOURCE-TYPED-SLOTS, SOURCE-FRAME-ROOTS, SOURCE-PROFILE,
SOURCE-RUNTIME, SOURCE-RUNTIME-NO-DEFAULT, SOURCE-LOGICAL-PORT,
SOURCE-ORACLE, SOURCE-ORACLE-NO-DEFAULT, SOURCE-WORKSPACE,
SOURCE-WORKSPACE-NO-DEFAULT, SOURCE-HOST-LP03, SOURCE-HOST-LP03-NO-DEFAULT,
SOURCE-HOST-LP04, SOURCE-HOST-LP04-NO-DEFAULT, SOURCE-HOST-LP05,
SOURCE-HOST-LP05-NO-DEFAULT
```

Replay the derived inventories as the last step of the edit stage, before the
first acceptance suite; quote every replay verdict. The workspace keeps
exactly the three standing switch-only skips, each run separately with
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1` in both feature modes; in sandbox
workspace commands keep `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` and quote every
native skip line. No new exclusion.

Then the compiled shape phase (after every source check is green):

```
python3 -I .temp/brief-651/instruments/stage_build.py .temp/brief-651 base cli
python3 -I .temp/brief-651/instruments/stage_build.py .temp/brief-651 candidate cli
python3 -I .temp/brief-651/instruments/stage_build.py .temp/brief-651 base stack-debug
python3 -I .temp/brief-651/instruments/stage_build.py .temp/brief-651 candidate stack-debug --candidate-probes .temp/brief-651/instruments/c2-incoming-probes.json
python3 -I .temp/brief-651/instruments/stage_build.py .temp/brief-651 base stack-release
python3 -I .temp/brief-651/instruments/stage_build.py .temp/brief-651 candidate stack-release
python3 -I .temp/brief-651/instruments/stage_shape.py --objdump .temp/brief-651/objdump/base/cli.objdump --spec .temp/brief-651/instruments/c2-base-cli.json --out .temp/brief-651/shape/base-cli.json
python3 -I .temp/brief-651/instruments/stage_shape.py --objdump .temp/brief-651/objdump/candidate/cli.objdump --spec .temp/brief-651/instruments/c2-cand-cli.json --out .temp/brief-651/shape/cand-cli.json
python3 -I .temp/brief-651/instruments/stage_shape.py --objdump .temp/brief-651/objdump/base/stack-debug.objdump --spec .temp/brief-651/instruments/c2-base-stack-debug.json --out .temp/brief-651/shape/base-stack-debug.json
python3 -I .temp/brief-651/instruments/stage_shape.py --objdump .temp/brief-651/objdump/candidate/stack-debug.objdump --spec .temp/brief-651/instruments/c2-cand-stack-debug.json --out .temp/brief-651/shape/cand-stack-debug.json
python3 -I .temp/brief-651/instruments/stage_shape.py --objdump .temp/brief-651/objdump/base/stack-release.objdump --spec .temp/brief-651/instruments/c2-base-stack-release.json --out .temp/brief-651/shape/base-stack-release.json
python3 -I .temp/brief-651/instruments/stage_shape.py --objdump .temp/brief-651/objdump/candidate/stack-release.objdump --spec .temp/brief-651/instruments/c2-cand-stack-release.json --out .temp/brief-651/shape/cand-stack-release.json
.temp/brief-651/bin/candidate/stack-debug brief651_incoming_layouts --nocapture --test-threads=1
```

Paste every `STAGE-SHAPE` and `LP06_LAYOUT` line verbatim. A FAIL is a STOP
with the values; do not retune the shape. Do not run Callgrind, perf or
timing; the runtime slope proof is brief-653's.

Concurrency: one cargo command at a time, `CARGO_BUILD_JOBS=8`,
`RUST_TEST_THREADS=8`, `CARGO_NET_OFFLINE=true`, no nested memory scope; the
dispatcher's 16G scope is the only scope. Reproduce a substantive red once at
`--test-threads=1`. No competing CPU work during a shape build.

## Checkpointing — protect the scene

After each milestone write `.temp/brief-651/PROGRESS.md` with what is done and
verified, the exact next action, the tree state, changed files, generated
inventory rows, every replay verdict and every outstanding cost bar. After
the tree compiles, two attempts that move none of these counters stop with
evidence. Preserve failed runs; restore any mutated source byte exactly.

## Hard rules

- No version-control command, network, `unsafe`, public API/feature/serde
  change, semantic frame/carrier/continuation change, unrelated source edit or
  prior-evidence edit.
- No duplicated semantic body, second global switch, lazy capture, debug-only
  inline policy, `unreachable!`/`expect` on a state the types allow, or a
  new field on `Call`/`PhaseEntry`/`ProfileBoundary`/`ProfileQueue`.
- All scratch under `.temp/brief-651/`; `.temp/brief-648/`, `.temp/brief-649/`
  and `.temp/brief-650/` are read-only.
- Only rum-native tests and rum-vs-rum A/B; no external-engine comparison.
- A new `UnknownHostPayload`, an existing inventory identity/status/disposition
  change, or a rejection-count movement stops the run with the comparison.

## Report format

Write a `rum-report/1` report at `.temp/brief-651/report.json`: changed files
and line ranges, every replay verdict verbatim, all runs with attempts, both
new tests' verdict lines, the six shape runs' `STAGE-SHAPE` lines and the
`LP06_LAYOUT` lines with receipts, the must-not-change proofs, the size
assessment of the touched files, and any unpassed cost question. Run
`python3 -I scripts/report.py check 651` and return its summary and the full
compliance verdict lines.

## Amendment 1 — 2026-11-04: the incoming wrapper is `#[inline]`; the debug arm expects it outlined

The run stopped correctly at the debug cost bar: candidate stack-debug
`resume_catch_after_body` reserves 888 against 792. The director reproduced
it (888 on a plain build of the submitted tree), then ran a controlled probe
with exactly one line changed — `#[inline(always)]` -> `#[inline]` on
`record_profile_runtime_path_timing_adopted_incoming` — and measured:

| arm | CLI release | stack-debug | stack-release |
|---|---|---|---|
| base (pre-A1) | 1016 | 792 | 1016 |
| submitted C2 | 1000, wrapper absent | 888 | 1000 |
| probe C2 `#[inline]` | 1000, wrapper absent | **792**, wrapper present | (this amendment) |

At `-O0` LLVM's always-inliner folds the wrapper's parameter and body
allocas into the Catch frame: a 56-byte Option plus `self`/outcome/result
slots is +96. With `#[inline]`, B's own release objdump already shows this
wrapper absent — the release inliner inlines it anyway — so `inline(always)`
bought nothing in release and cost 96 debug bytes per Catch frame.

The working tree already holds the completed C2 source. Execute this
amendment on it; do not re-apply the earlier deliverables, and make no
source edit other than the one below.

1. In `crates/rum-runtime/src/runtime/profiling/control_timing.rs`, change
   the attribute on `record_profile_runtime_path_timing_adopted_incoming`
   from `#[inline(always)]` to `#[inline]`. Every other byte of the three
   source files stays as it is now.

2. Candidate stack-debug is checked with the new supplied spec
   `.temp/brief-651/instruments/c2-cand-stack-debug-outlined.json`:
   prologue-max 792 ordinals 1; the wrapper and its slow body both
   **present** (outlined in the debug profile is the intended shape). The
   base stack-debug spec is unchanged (`absent` for both: the base has no
   adopted path). All other specs are unchanged.

3. Amended bars: CLI release 1000 <= 1016, wrapper absent, slow present;
   stack-release 1000 <= 1016, wrapper absent, slow present; stack-debug
   792, wrapper present, slow present; `IncomingPathTimingToken` and its
   Option 56 bytes; the five outcomes, the two C2 tests and the five mutant
   calibrations on the amended bytes.

4. Full re-acceptance on the amended source, in the same order:
   inventory replay (quote every verdict), the 24-command SOURCE catalogue,
   the two C2 tests, the five mutant calibrations (red, byte-identical
   restore, green), then the shape phase. Rebuild all six arms with fresh
   artifacts and target dirs:

   ```
   python3 -I .temp/brief-651/instruments/stage_build.py .temp/brief-651 base cli --attempt 2 --target-suffix amend1
   python3 -I .temp/brief-651/instruments/stage_build.py .temp/brief-651 candidate cli --attempt 2 --target-suffix amend1
   python3 -I .temp/brief-651/instruments/stage_build.py .temp/brief-651 base stack-debug --attempt 2 --target-suffix amend1
   python3 -I .temp/brief-651/instruments/stage_build.py .temp/brief-651 candidate stack-debug --attempt 2 --target-suffix amend1 --candidate-probes .temp/brief-651/instruments/c2-incoming-probes.json
   python3 -I .temp/brief-651/instruments/stage_build.py .temp/brief-651 base stack-release --attempt 2 --target-suffix amend1
   python3 -I .temp/brief-651/instruments/stage_build.py .temp/brief-651 candidate stack-release --attempt 2 --target-suffix amend1
   python3 -I .temp/brief-651/instruments/stage_shape.py --objdump .temp/brief-651/objdump/base/cli-attempt2.objdump --spec .temp/brief-651/instruments/c2-base-cli.json --out .temp/brief-651/shape/base-cli-amend1.json
   python3 -I .temp/brief-651/instruments/stage_shape.py --objdump .temp/brief-651/objdump/candidate/cli-attempt2.objdump --spec .temp/brief-651/instruments/c2-cand-cli.json --out .temp/brief-651/shape/cand-cli-amend1.json
   python3 -I .temp/brief-651/instruments/stage_shape.py --objdump .temp/brief-651/objdump/base/stack-debug-attempt2.objdump --spec .temp/brief-651/instruments/c2-base-stack-debug.json --out .temp/brief-651/shape/base-stack-debug-amend1.json
   python3 -I .temp/brief-651/instruments/stage_shape.py --objdump .temp/brief-651/objdump/candidate/stack-debug-attempt2.objdump --spec .temp/brief-651/instruments/c2-cand-stack-debug-outlined.json --out .temp/brief-651/shape/cand-stack-debug-amend1.json
   python3 -I .temp/brief-651/instruments/stage_shape.py --objdump .temp/brief-651/objdump/base/stack-release-attempt2.objdump --spec .temp/brief-651/instruments/c2-base-stack-release.json --out .temp/brief-651/shape/base-stack-release-amend1.json
   python3 -I .temp/brief-651/instruments/stage_shape.py --objdump .temp/brief-651/objdump/candidate/stack-release-attempt2.objdump --spec .temp/brief-651/instruments/c2-cand-stack-release.json --out .temp/brief-651/shape/cand-stack-release-amend1.json
   .temp/brief-651/bin/candidate/stack-debug-attempt2 brief651_incoming_layouts --nocapture --test-threads=1
   ```

   The base arms are rebuilt rather than reused so the amendment's receipts
   form one campaign. A FAIL is a STOP with the values; do not retune.
   Never edit an instrument; the new spec file is supplied.

5. Report: copy the stopped record to `.temp/brief-651/report-stop.json`,
   then write the amendment's final `rum-report/1` at
   `.temp/brief-651/report.json` with the same fields plus this amendment's
   edit, bars, and every verdict line. Run
   `python3 -I scripts/report.py check 651` and return its summary and the
   full compliance lines. Everything else in the brief, including Hard
   rules and reporting, is unchanged.

## Director gate — 2026-11-04: GREEN (amendment 1)

`python3 -I scripts/director-gate.py run 651` (`.temp/director/gate-651`):

| Step | Verdict | Detail |
|---|---|---|
| scope | PASS | 23,554 citations verified against Git; the 10 changed files are exactly the scope set |
| fmt | PASS | formatted |
| check | PASS | 3 warnings, 0 new against HEAD, 0 gone |
| ownership | PASS | 61 self-reaching types, 79 UnknownHostPayload, 8018 UnknownConstruct; 0 replay rounds |
| census | PASS | 1067 {RET001: 10, RET002: 1022, RET003: 35}; paired 1067, retired 0, new 0 |
| oracle | PASS | 1 passed |
| standing | PASS | LP01 1/0; typed-slot 10/0 |
| fingerprints | PASS | 68 contracts, 0 fingerprints changed, copied in |
| certificates | PASS | both builds pass; 0 promoted rows merged; 1 round |
| inventories | PASS | 19 passed, 0 failed; frontier 1067 |
| workspace | PASS | 62 binaries, 4095 passed, 0 failed, 61 ignored; peak 7.0 GB of 14, oom_kill 0 |

Beyond the script, reproduced by the director:

- **First run stopped correctly.** Candidate stack-debug reserved 888
  against the 792 bar; the worker changed nothing after the failure and
  preserved `report-stop.json` and every failed run.
- **Attribution.** The +96 was `#[inline(always)]` on the incoming
  recorder wrapper: LLVM's always-inliner runs at `-O0` and folds the
  wrapper's 56-byte Option parameter and body allocas into the Catch frame.
  A controlled probe with that one attribute changed to `#[inline]`
  measured stack-debug 792 (wrapper outlined) and CLI 1000 (wrapper
  absent); B's release objdump already had this wrapper absent with
  `#[inline]`, so the attribute bought nothing in release. The debug
  spec's `absent` clause was the brief's error; amendment 1 supplied
  `c2-cand-stack-debug-outlined.json` (792, wrapper present).
- **Shape, reproduced on the pinned attempt-2 objdumps**: CLI 1016/1000,
  stack-debug 792/792, stack-release 1016/1000 (base/candidate); release
  wrapper absent and cold body present in both release arms; debug wrapper
  and cold body present. Layout: `IncomingPathTimingToken` 56 and
  `Option<IncomingPathTimingToken>` 56. All six binary and objdump
  sha256 receipts verify.
- **Inventories**: `local-contracts.tsv` changes only the fingerprint
  column across all 68 rows; the ownership inventories shift only
  `logical/mod.rs` line numbers by +1 (the `mod incoming;` declaration);
  `promoted.tsv` changes only its certificate cells. Identities, statuses
  and dispositions are untouched.
- **C2 bars, as ruled**: carrier and Option 56; CLI and stack-release
  1000 <= 1016 with the wrapper gone; stack-debug 792 with the wrapper
  outlined; the five Incoming outcomes (including Cancelled), the two new
  tests and the five mutant calibrations green on the final bytes. The
  enabled terminal `Epoch::inspect` lock and temporary `Owner` Arc clone
  remain disclosed and accepted. LP06 stays unpassed until brief-653.
