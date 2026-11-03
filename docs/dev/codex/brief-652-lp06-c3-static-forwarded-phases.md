# Brief 652 — LP06 C3: static forwarded phases and the timing adapters

## Goal

The four forwarded suspend starts still pass their phase as a runtime
`&'static str`, so the single shared
`runtime_path_timing_start_adopted_forwarded_slow` in the A1 tree reserves 40
bytes in the release builds and saves the phase registers across its inner
call; the pre-A1 base, with one constant caller, reserved 24. Turn the phase
into a compile-time marker (`P::NAME`) so each of the four sites gets its own
cold body at the base 24-byte shape, and give the two timing recorders the
token-first disabled path with their lexical rechecks at the head of the
existing cold bodies. Every semantic path stays byte-identical. LP06 stays
unpassed until brief-653.

## Context pointers

- Read `AGENTS.md`, then `.temp/director/brief-649-design-verdict.md`
  (Q-CODEGEN, Q-INCOMING, Q-CALIBRATION) and the design report's F4, F6 and
  "Draft implementation brief C3" sections (`.temp/brief-649/report-design.json`).
- Source at HEAD: `crates/rum-runtime/src/runtime/profiling/timing.rs:128`
  (adopted start), `:188` (forwarded start wrapper), `:201` (its cold body),
  `:214` (boundary recorder), `:234` (its cold body);
  `.../profiling/control_timing.rs:101` (forwarded recorder), `:122` (its
  cold body); `.../profiling/mod.rs:10` (`mod control_timing;`) and `:39`
  (the config import the re-export sits beside); call sites
  `.../definitions/definitions.rs:43` and
  `.../kernel_data/control_data/branching_loops.rs:163,190,372`.
- B evidence, read-only:
  `.temp/brief-648/instruments/lp06-prologues/{base,candidate}/cli/runtime-tests.objdump`.
- Instruments and specs are supplied in `.temp/brief-652/instruments/`; do
  not edit them. The entry arm (HEAD `a4c6a9a6`) is director-built and pinned
  under `.temp/brief-652/{bin,objdump}/candidate/` (attempt 1); consume it
  read-only and do not rebuild it.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/profiling/timing.rs` — the forwarded start
  wrapper and its slow body (generic phase), and the boundary recorder's
  wrapper/slow gate placement. The adopted start's `#[inline]` and gate-first
  body are unchanged.
- `crates/rum-runtime/src/runtime/profiling/control_timing.rs` — the
  `ForwardedTimingPhase` trait and its four marker declarations, and the
  forwarded recorder's wrapper/slow gate placement.
- `crates/rum-runtime/src/runtime/profiling/mod.rs` — one re-export block
  for the trait and the four markers.
- `crates/rum-runtime/src/runtime/definitions/definitions.rs` — the single
  forwarded start call at 43.
- `crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs`
  — the three forwarded start calls at 163, 190 and 372.
- Replay-generated inventories (by replay only).

Everything else is MUST NOT, in particular: `nonlocal.rs`, every semantic
evaluator, classifier, matcher and witness, `open_phase`/`complete_phase`,
the `PhaseKey` tables, equality/hash, callable parsing and projection, macro
expansion, every frame, and any widening of `control_timing`'s module
visibility.

## Deliverable

1. **The trait and the four markers** in `control_timing.rs`:

   ```rust
   pub(in crate::runtime) trait ForwardedTimingPhase {
       const NAME: &'static str;
   }
   ```

   with four `pub(in crate::runtime)` zero-sized markers whose `NAME` is the
   existing literal exactly: `SetRhsSuspend = "set.rhs.suspend"`,
   `WhileTestSuspend = "while.test.suspend"`,
   `WhileBodySuspend = "while.body.suspend"`, and
   `CompoundItemSuspend = "compound.item.suspend"`.

2. **The generic forwarded start** in `timing.rs`: the wrapper and its
   single cold body take `P: crate::runtime::profiling::ForwardedTimingPhase`
   in place of `phase: &'static str`, and the body's two uses become
   `P::NAME` (the inner `runtime_path_timing_start_adopted_slow(self, ...)`
   and `complete_profile_runtime_path_boundary_slow(...)`). Keep the
   wrapper's `#[inline]` and its gate-first body and the cold body's
   `#[cold] #[inline(never)]`. One source body, monomorphized per marker.

3. **One re-export block** in `profiling/mod.rs`, of the trait and the four
   markers, `pub(in crate::runtime)`, beside the existing `use config::{...}`.
   Do not change `mod control_timing;`.

4. **The four call sites** name the marker through that re-export, fully
   qualified so no producer import changes:
   `self.runtime_path_timing_start_adopted_forwarded::<crate::runtime::profiling::WhileTestSuspend>(&result)`
   and the same for `SetRhsSuspend`, `WhileBodySuspend`,
   `CompoundItemSuspend` at their existing statements. No other call-site
   edit.

5. **The token-first recorders.** The boundary recorder (`timing.rs:214`)
   and the forwarded recorder (`control_timing.rs:101`) test the token
   first and pass a bare token to their existing cold bodies; the existing
   `if !lexical_profile_enabled() { return; }` becomes the first statement
   of each cold body. Keep every other statement byte-identical. Keep
   `#[inline]` on both wrappers — do **not** use `#[inline(always)]`: B's
   release objdump already has both wrappers absent, and at `-O0` the
   always-inliner folds their 56-byte `Option` parameter into the caller's
   frame (measured at the C2 gate: +96 bytes per Catch frame).

6. **Compiled shape check**, after acceptance is green, exactly as in
   Verification.

## Must not change

- Each phase's NAME literal and key; every start/completion pairing; the
  boundary inserted after the whole remainder; the legacy record after the
  logical completion; the pause before the consumer; the second-pause
  lifecycle; every A1 and A2 oracle map; the Catch and builtin paths.
- Every evaluate, matcher, classifier and witness; Return/Break/Continue raw
  outcomes; status projection and value separation; Compound item order.
- No new `PhaseKey`, no runtime phase dispatch, no const-index conversion
  tied to `PhaseKey` order, no widened module visibility, no moved timing
  function, no new field on any frame, no duplicated semantic body.

## Cost bars and falsifier

Director calibration (base = the immutable pre-A1 export; entry = HEAD
`a4c6a9a6`, pinned):

| symbol | base cli / debug / stack-release | entry cli / debug / stack-release | candidate bar |
|---|---|---|---|
| `runtime_path_timing_start_adopted_forwarded_slow` | 24 / 88 / 24, 1 body | 40 / 88 / 40, 1 body | **4 bodies, each <= 24 / 88 / 24** |
| `resume_while_loop` | 808 / 3528 / 808 | 776 / 3528 / 776 | <= 776 / 3528 / 776 |
| `resume_compound_expression_from_index` | 776 / 1304 / 776 | 696 / 1304 / 680 | <= 696 / 1304 / 680 |
| `resume_catch_after_body` | 1016 / 792 / 1016 | 1000 / 792 / 1000 | <= 1000 / 792 / 1000 |

- Release arms additionally: the boundary, forwarded-recorder and
  adopted-start wrapper symbols are **absent**; their cold bodies are
  present. The debug arm keeps the wrappers outlined (no absence check
  there).
- Four forwarded bodies is the target ordinal count; any other count is a
  STOP.
- Recorded, not gated, in every arm: `try_apply_function`,
  `eval_control_data_builtin` and `try_dispatch_builtin_call` prologues
  (F6's executed neighbour comparison and F7 belong to brief-653).
- The disabled path gains no call edge and no classification; the adopted
  start still checks the gate before any slow call.
- A FAIL is a STOP with the values; do not retune a spec, an instrument or
  the source shape.

Falsifier: the supplied reducer is calibrated on the retained base and
entry objdumps — the entry arm prints one 40-byte body against the 24 bar
(red) and the base prints one 24-byte body (green); the candidate must print
four bodies at or below the bar in each kind.

## Verification (MANDATORY — run in this order)

First the standing source set from `.temp/brief-645/inventory/commands.tsv`,
unchanged and with the stage root `.temp/brief-652/`:

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
python3 -I .temp/brief-652/instruments/stage_build.py .temp/brief-652 base cli
python3 -I .temp/brief-652/instruments/stage_build.py .temp/brief-652 candidate cli --attempt 2 --target-suffix c3
python3 -I .temp/brief-652/instruments/stage_build.py .temp/brief-652 base stack-debug
python3 -I .temp/brief-652/instruments/stage_build.py .temp/brief-652 candidate stack-debug --attempt 2 --target-suffix c3
python3 -I .temp/brief-652/instruments/stage_build.py .temp/brief-652 base stack-release
python3 -I .temp/brief-652/instruments/stage_build.py .temp/brief-652 candidate stack-release --attempt 2 --target-suffix c3
python3 -I .temp/brief-652/instruments/stage_shape.py --objdump .temp/brief-652/objdump/base/cli.objdump --spec .temp/brief-652/instruments/c3-base-cli.json --out .temp/brief-652/shape/base-cli.json
python3 -I .temp/brief-652/instruments/stage_shape.py --objdump .temp/brief-652/objdump/candidate/cli-attempt2.objdump --spec .temp/brief-652/instruments/c3-cand-cli.json --out .temp/brief-652/shape/cand-cli.json
python3 -I .temp/brief-652/instruments/stage_shape.py --objdump .temp/brief-652/objdump/base/stack-debug.objdump --spec .temp/brief-652/instruments/c3-base-stack-debug.json --out .temp/brief-652/shape/base-stack-debug.json
python3 -I .temp/brief-652/instruments/stage_shape.py --objdump .temp/brief-652/objdump/candidate/stack-debug-attempt2.objdump --spec .temp/brief-652/instruments/c3-cand-stack-debug.json --out .temp/brief-652/shape/cand-stack-debug.json
python3 -I .temp/brief-652/instruments/stage_shape.py --objdump .temp/brief-652/objdump/base/stack-release.objdump --spec .temp/brief-652/instruments/c3-base-stack-release.json --out .temp/brief-652/shape/base-stack-release.json
python3 -I .temp/brief-652/instruments/stage_shape.py --objdump .temp/brief-652/objdump/candidate/stack-release-attempt2.objdump --spec .temp/brief-652/instruments/c3-cand-stack-release.json --out .temp/brief-652/shape/cand-stack-release.json
```

Paste every `STAGE-SHAPE` line verbatim. A FAIL is a STOP with the values;
do not retune the shape. Do not run Callgrind, perf or timing; the runtime
slope proof is brief-653's.

Concurrency: one cargo command at a time, `CARGO_BUILD_JOBS=8`,
`RUST_TEST_THREADS=8`, `CARGO_NET_OFFLINE=true`, no nested memory scope; the
dispatcher's 16G scope is the only scope. Reproduce a substantive red once
at `--test-threads=1`. No competing CPU work during a shape build.

## Checkpointing — protect the scene

After each milestone write `.temp/brief-652/PROGRESS.md` with what is done
and verified, the exact next action, the tree state, changed files,
generated inventory rows, every replay verdict and every outstanding cost
bar. After the tree compiles, two attempts that move none of these counters
stop with evidence. Preserve failed runs; restore any mutated source byte
exactly.

## Hard rules

- No version-control command, network, `unsafe`, public API/feature/serde
  change, semantic frame/carrier/continuation change, unrelated source edit
  or prior-evidence edit.
- No duplicated semantic body, second global switch, lazy capture,
  debug-only inline policy, `unreachable!`/`expect` on a state the types
  allow, or a new field on `Call`/`PhaseEntry`/`ProfileBoundary`/`ProfileQueue`.
- All scratch under `.temp/brief-652/`; `.temp/brief-648/`, `.temp/brief-649/`,
  `.temp/brief-650/`, `.temp/brief-651/` and the director's entry arm are
  read-only.
- Only rum-native tests and rum-vs-rum A/B; no external-engine comparison.
- A new `UnknownHostPayload`, an existing inventory identity/status/
  disposition change, or a rejection-count movement stops the run with the
  comparison.

## Report format

Write a `rum-report/1` report at `.temp/brief-652/report.json`: changed
files and line ranges, every replay verdict verbatim, all runs with
attempts, both arms' symbolic verdict lines, the six `STAGE-SHAPE` lines
with receipts, the four instantiations' prologue values, the recorded F6/F7
prologues, the must-not-change proofs, the size assessment of the touched
files, and any unpassed cost question. Run
`python3 -I scripts/report.py check 652` and return its summary and the full
compliance verdict lines.

## Director gate — 2026-11-04: GREEN

| Step | Verdict | Detail |
|---|---|---|
| scope | PASS | 22,822 citations verified against Git; the 12 changed files are exactly the scope set (five source, seven replayed inventories) |
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

- **Shape, recomputed by me on the pinned candidate attempt-2 objdumps and
  the fresh base arm**, against the six supplied specs: forwarded slow
  bodies four ordinals each 24 / 56 / 24 bytes (cli / stack-debug /
  stack-release); `resume_while_loop` 744 / 3528 / 744;
  `resume_compound_expression_from_index` 648 / 1304 / 664;
  `resume_catch_after_body` 1000 / 792 / 1000. Base arm: one body at
  24 / 88 / 24; While 808 / 3528 / 808; Compound 776 / 1304 / 776; Catch
  1016 / 792 / 1016. Every bar met; no spec, instrument or source shape was
  retuned.
- **The literals are static and per marker.** In the candidate CLI objdump
  the four bodies sit at `0xe2ce0`, `0xe2d30`, `0xe2d80`, `0xe2dd0`, each
  loading its own `.rodata` literal ("set.rhs.suspend" 15 B,
  "compound.item.suspend" 21 B, "while.body.suspend" and
  "while.test.suspend" 18 B) and each called from exactly one site; no
  phase register is saved across a shared body any more.
- **Release wrappers absent, cold bodies present.** Both release arms lack
  the boundary, forwarded-recorder and adopted-start wrapper symbols and
  keep their cold bodies (boundary 2, forwarded 1, adopted 1 hits); the
  debug arm keeps the wrapper outlined and the four forwarded cold bodies
  at 56 B.
- **The recorder reorder earns its place.** My full-shape probe before
  dispatch (generic phases plus token-first recorders) matched these
  candidate values exactly; the reorder lowers While 776 -> 744 and
  Compound 696/680 -> 648/664 rather than costing a frame.
- **No semantic movement.** The diff touches only the forwarded wrapper and
  its one cold body, the two recorder wrappers' gate placement (the lexical
  recheck moved to the head of each existing cold body), one trait with
  four zero-sized markers, one re-export block and four call-site
  turbofish arguments. No `PhaseKey`, no new field on any frame, no moved
  phase literal, no widened visibility; the adopted start's gate-first body
  is byte-identical.
- **Inventories, column-audited against HEAD**: the ownership inventories
  move only line/end-line columns (52/28/4/2 rows); `local-contracts.tsv`
  moves only the fingerprint column across 68 rows; `promoted.tsv` only its
  two certificate cells; `logical-profile-required.tsv` only the
  producer_fingerprint column across 140 rows of the edited producer's
  body. Row sets and keys are unchanged, and the gate's replay steps are
  green.
- **Recorded, not gated**: F6's neighbours (`try_apply_function`
  1096/2808/1096; `eval_control_data_builtin` 1240/120/984) and F7
  (`try_dispatch_builtin_call` [872, 856] in CLI at both the pinned entry
  arm and the candidate) are unchanged by C3 and remain brief-653's. LP06
  stays unpassed until the D campaign.
