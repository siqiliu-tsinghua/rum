# Brief 650 — LP06 C1: the builtin recorder's disabled fast path

## Goal

Remove the disabled-path cost of the adopted builtin recorder measured by
brief-648: the outlined
`record_profile_builtin_timing_adopted_boundary` call (19 instructions per
incoming call with profiling off), the unobserved outer availability layer
`Option<Option<AdoptedBuiltinTimingToken>>`, and the live `&Result` staging it
forces in `try_dispatch_builtin_call`. When you are done the disabled CLI
dispatch has no recorder call edge and both CLI `try_dispatch_builtin_call`
entry reservations are back at the base 856 bytes, while the A2 builtin
lifecycle still records exactly as today on the enabled path. LP06 stays
unpassed overall; that verdict belongs to brief-653.

## Context pointers

- Read `AGENTS.md` first, then `docs/dev/codex/brief-647-lp06-a2-builtin-catch.md`
  (the A2 contract and its §"Checked and unchecked dispatch" distinction) and
  `docs/dev/codex/brief-648-lp06-b-disabled-path.md` §1-§3.
- The design: `.temp/director/brief-649-design-verdict.md` and the B verdict
  `.temp/director/brief-648-b-verdict.md`. Read-only B evidence:
  `.temp/brief-648/instruments/` (never modify or execute a prior campaign
  script in place).
- Source: `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:180-245`
  (`.map` start and `finish_builtin_dispatch`),
  `crates/rum-runtime/src/runtime/profiling/control_builtin.rs:58-95` (the
  recorder wrapper and its cold body),
  `.../profiling/config.rs:4-24` (the two gates). Tests:
  `.../profiling/tests/control.rs:222` (`lp06_a2_builtin_no_handler`), `:670`
  (`lp06_a2_builtin_ok`) and `.../tests/queue_boundaries.rs:275` for the
  mid-test gate flip idiom.
- Acceptance is mechanical: the named tests and the compiled shape check
  decide, not prose.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` — only the start
  statement at 184-185 and the `finish_builtin_dispatch` signature/body at
  222-245.
- `crates/rum-runtime/src/runtime/profiling/control_builtin.rs` — only the
  wrapper at 58-71 and the first statement of its cold body at 75-79.
- `crates/rum-runtime/src/runtime/profiling/tests/control.rs` — add the new
  test named in Deliverable 3.
- Replay-generated inventories listed under Verification (by replay only).

Everything else is MUST NOT, in particular: `run_builtin_handler`, the
capability branches, `ResumeCallDispatchStage` frames, the `resume_typed_or_suspend!`
macro, the freeze/finish closures, `validate_builtin_memory_growth_slow`,
`builtin_timing_start_adopted` and its slow body, `BuiltinRecord`, the legacy
`record_profile_builtin_timing`, and every file outside the list above.

## Deliverable

1. **Start layer erased.** In `try_dispatch_builtin_call` replace

   ```rust
   let profile_builtin_start =
       builtin_symbol.map(|symbol| self.builtin_timing_start_adopted(&symbol, args.len()));
   ```

   with

   ```rust
   let profile_builtin_start =
       builtin_symbol.and_then(|symbol| self.builtin_timing_start_adopted(&symbol, args.len()));
   ```

   and change `finish_builtin_dispatch`'s parameter to
   `Option<crate::runtime::profiling::control_builtin::AdoptedBuiltinTimingToken>`,
   passing it to the recorder without `.flatten()`.

2. **Token-first always-inline wrapper.** In `control_builtin.rs`:

   ```rust
   #[inline(always)]
   pub(in crate::runtime) fn record_profile_builtin_timing_adopted_boundary(
       &mut self,
       token: Option<AdoptedBuiltinTimingToken>,
       result: &Result<Expr, EvalError>,
   ) {
       let Some(token) = token else {
           return;
       };
       self.record_profile_builtin_timing_adopted_boundary_slow(token, result);
   }
   ```

   and move the existing `if !lexical_profile_enabled() { return; }` check to
   the first statement of the `#[cold] #[inline(never)]` slow body. Keep every
   remaining statement of that body byte-identical. Moving the recheck keeps
   the present-token-but-now-disabled early return, the Box drop and the
   Storage release; deleting it is a STOP.

3. **Gate-transition test.** Add `lp06_c1_builtin_gate_transition` in
   `profiling/tests/control.rs`, modelled on `lp06_a2_incoming_policy_cases`
   (`test_support::enabled()`, `BuiltinEnabled::new()`,
   `super::epoch::reset_for_lease(enabled.lease())`, a `System`Symbol, and
   `builtin_timing_start_adopted`). Two halves:
   - flag stays on: `record_profile_builtin_timing_adopted_boundary(Some(token), &Ok(...))`
     must yield the Builtin row `(calls, open, ok, abandoned) == (1, 0, 1, 0)`;
   - flag flipped to `2` in `LEXICAL_PROFILE_ACTIVE` between start and the
     same recorder call must yield `(1, 0, 0, 1)` — the token is dropped, not
     silently finished or double-closed — then restore the flag.
   End both halves with
   `Session::logical_profile_test_assert_quiescent()` and
   `assert_eq!(test_support::active(), 0)`.
   Eprintln one `LP06 C1 NATIVE ...` verdict line per half.

4. **Compiled shape check** (after acceptance is green) exactly as below.

## Must not change

- Handler invocation count and order; capability refusal; `ValueQ`'s
  Cancelled path; missing-spec/missing-handler early returns.
- The boundary is recorded before a `freeze()` frame is constructed, and
  before optional mapping/validation: InputRequired still pauses the Call and
  inserts the boundary; every other outcome still calls `Expr::finish_logical`;
  the legacy recorder still fires exactly once with `result.is_ok()`; Storage
  is still dropped.
- Start capture semantics: the same symbol, the same
  `arg_count.min(u16::MAX)` arity, the same gates in `builtin_timing_start_adopted`.
- No new field, no new outlined adapter, no duplicated body, no
  debug-only inline policy, no flattened guest representation, no semantic
  frame change.

## Cost bars and falsifier

Targets on the rebuilt CLI (`rum-cli`, release, CGU=1):

- `try_dispatch_builtin_call` specialization ordinals 1 and 2: entry
  reservation **856 bytes each** (base values), from the B candidate's
  888/872.
- The symbol
  `...::record_profile_builtin_timing_adopted_boundary` is **absent** from the
  emitted binary (fully inlined). The `..._boundary_slow` symbol remains, and
  its only callers are `try_dispatch_builtin_call` bodies.
- Base arm (pre-A1 export): 856/856 with both adopted-recorder symbols absent.

The instrument is supplied and calibrated by me:

- `.temp/brief-650/instruments/stage_shape.py` — static prologue reservation,
  symbol presence/absence and call-edge reducer; it reproduces the B campaign
  numbers exactly. My calibration runs on the retained B objdumps printed:

  ```
  STAGE-SHAPE PASS prologue-max ...try_dispatch_builtin_call$ values=[856, 856] max=856 ordinals=2
  STAGE-SHAPE PASS absent ...record_profile_builtin_timing_adopted_boundary$ hits=0
  STAGE-SHAPE PASS absent ...record_profile_builtin_timing_adopted_boundary_slow$ hits=0
  [base arm]
  STAGE-SHAPE FAIL prologue-max ...try_dispatch_builtin_call$ values=[888, 872] max=856 ordinals=2
  STAGE-SHAPE FAIL absent ...record_profile_builtin_timing_adopted_boundary$ hits=1
  STAGE-SHAPE PASS present ...record_profile_builtin_timing_adopted_boundary_slow$ hits=1
  STAGE-SHAPE FAIL callers ...record_profile_builtin_timing_adopted_boundary_slow$ -> 1 callers, 1 unallowed, 0 forbidden
  [B candidate arm]
  ```

  A repaired tree must print `PASS` on every line of
  `.temp/brief-650/instruments/c1-candidate.json`; the fresh base arm must
  print `PASS` on `c1-base.json`. A FAIL is a STOP: report the values, do not
  retune the shape.

Runtime call-slope proof (the call edge is gone when profiling is off) is
brief-653's; do not run Callgrind, perf or timing here.

## Verification (MANDATORY — run in this order)

First the standing source set, from `$PWD`, with the stage scratch root
`.temp/brief-650/` (the exact commands and bounds of
`.temp/brief-645/inventory/commands.tsv`, unchanged):

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
first acceptance suite; quote every replay verdict. The workspace has exactly
the three standing switch-only skips
(`lp03_adopted_phases_are_exact_in_completed_and_resumed_runs`,
`lp04_adopted_definition_rows_are_exact_in_completed_and_resumed_runs`,
`lp05_adopted_assignment_rows_are_exact_in_completed_and_resumed_runs`), each
run separately with `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` in both feature
modes. No new exclusion. In sandbox workspace commands only, keep
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` and quote all native skip lines.

Then the compiled shape phase (after every source check is green):

```
python3 -I .temp/brief-650/instruments/stage_build.py .temp/brief-650 base cli
python3 -I .temp/brief-650/instruments/stage_build.py .temp/brief-650 candidate cli
python3 -I .temp/brief-650/instruments/stage_shape.py --objdump .temp/brief-650/objdump/base/cli.objdump --spec .temp/brief-650/instruments/c1-base.json --out .temp/brief-650/shape/base.json
python3 -I .temp/brief-650/instruments/stage_shape.py --objdump .temp/brief-650/objdump/candidate/cli.objdump --spec .temp/brief-650/instruments/c1-candidate.json --out .temp/brief-650/shape/candidate.json
```

`stage_build.py` builds each arm from one common absolute source path
(`.temp/brief-650/b/active`), release CGU=1, offline, jobs 8, under
`run-bounded 880`; it hashes the candidate export against the live tree and
pins binary and objdump sha256 receipts. Do not edit those scripts; if one
fails, STOP with its log.

Concurrency: one cargo command at a time, `CARGO_BUILD_JOBS=8`,
`RUST_TEST_THREADS=8`, `CARGO_NET_OFFLINE=true`, no nested memory scope.
Reproduce a substantive red once at `--test-threads=1`. No competing CPU work
during the shape build.

## Checkpointing — protect the scene

After each milestone write `.temp/brief-650/PROGRESS.md` with what is done and
verified, the exact next action, the tree state, changed files, generated
inventory rows, every replay verdict and any outstanding cost bar. After the
tree compiles, two attempts that move none of these counters stop with
evidence. Preserve failed runs and restore any mutated source byte exactly.

## Hard rules

- No version-control command, network, `unsafe`, public API/feature/serde
  change, semantic frame/carrier/continuation change, unrelated source edit or
  prior-evidence edit.
- No duplicated semantic body, no second global dispatch switch, no
  lazy capture, no debug-only inline attribute, no `unreachable!`/`expect` on
  a state the types allow.
- All scratch under `.temp/brief-650/`; `.temp/brief-648/` and
  `.temp/brief-649/` are read-only.
- Only rum-native tests and rum-vs-rum A/B; no external-engine comparison.
- A new `UnknownHostPayload`, an existing inventory identity/status/disposition
  change, or a rejection-count movement stops the run with the comparison.

## Report format

Write a `rum-report/1` report at `.temp/brief-650/report.json`: the diff's
changed files and line ranges, every replay verdict copied verbatim, all runs
with attempts, the two test halves' verdict lines, both shape runs with their
`STAGE-SHAPE` lines and receipts, the must-not-change proofs, the size
assessment of the touched files, and any unpassed cost question. Run
`python3 -I scripts/report.py check 650` and return its summary and the full
compliance verdict lines.

## Director gate — 2026-11-04: GREEN (bar re-specified on new evidence)

`python3 -I scripts/director-gate.py run 650` (`.temp/director/gate-650`):

| Step | Verdict | Detail |
|---|---|---|
| scope | PASS | 11,688 citations verified against Git; 8 files, exactly the scope set |
| fmt | PASS | formatted |
| check | PASS | 3 warnings, 0 new against HEAD, 0 gone |
| ownership | PASS | 61 self-reaching types, 79 UnknownHostPayload, 8017 UnknownConstruct; 0 replay rounds |
| census | PASS | 1067 {RET001: 10, RET002: 1022, RET003: 35}; paired 1067, retired 0, new 0 |
| oracle | PASS | 1 passed |
| standing | PASS | LP01 1/0; typed-slot 10/0 |
| fingerprints | PASS | 68 contracts, 0 fingerprints changed, copied in |
| certificates | PASS | both builds pass; 0 promoted rows merged; 1 round |
| inventories | PASS | 19 passed, 0 failed; frontier 1067 |
| workspace | PASS | 62 binaries, 4093 passed, 0 failed, 61 ignored; peak 6.5 GB of 14, oom_kill 0, kernel OOM lines 0 |

Beyond the script, reproduced by the director:

- **Worker artifacts.** I re-ran the calibrated reducer on the worker's
  pinned original candidate objdump: `[872, 856]`, wrapper absent, cold body
  present, its only callers the dispatch bodies — the report's exact values.
- **Clean rebuild.** The same C1 source at the same absolute path
  (`builds/candidate/cli-attempt4`, sha256 `655089cb...`) reproduces the
  shape `[872, 856]`; the byte hash differs from the worker's build. An early
  probe build had overwritten the pinned candidate path; the rebuild restored
  it, and `.temp/brief-650/EVIDENCE-RECONCILIATION.md` records the loss. The
  builder now pins per attempt.
- **Frame attribution probes** (same path, CGU=1, `-p rum-cli` release):
  A1-only (commit 6864588b) and A1 + A2's non-dispatch files both reserve
  `[856, 856]`; A2 `[888, 872]`; C1 `[872, 856]`; C1 with the
  `Option<Option<...>>` layer restored `[888, 872]`; C1 with the entire
  builtin diagnostic removed `[872, 856]`; C1 with the token unboxed
  `[904, 888]`; C1 with only the finish call removed `[872, 872]`.
  Therefore the single-Option form is worth 16 bytes in both
  specializations, the recorder inlining is frame-neutral, and the remaining
  +16 in the enlarged-spec specialization is the restructured dispatch's
  layout cost, present with no diagnostic at all. Open finding F7; no source
  shape tested restores 856 while keeping the adopted boundary.
- **Bar.** C1's acceptance is: no ordinal grows; the ordinary specialization
  returns to base; the recorder wrapper and its call edge are gone; the
  adopted diagnostic adds 0 bytes over a diagnostic-free build of the same
  tree; the enabled lifecycle and gate transition are unchanged. "Both
  ordinals at 856" was not achievable inside C1's causal scope.

- **No-default workspace** at the host: 62 binaries, 4093 passed, 0 failed,
  61 ignored (`.temp/director/gate-650-nodefault.log`).
- **Switch-gated host tests** lp03/lp04/lp05, `1 passed` each, with
  `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` in the default and the
  `--no-default-features` builds (`.temp/director/gate-650-switch/`).
- **Native ZeroMQ**: the gate workspace ran without
  `RUM_ALLOW_NATIVE_ZEROMQ_SKIP`; its nine native tests passed on the host.
