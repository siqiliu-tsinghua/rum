# Brief 729 — G15-C2: carry the original Input source through queued request packaging

Stage: **implementation**, one shape, staged. The design is accepted in
`.temp/brief-728/`; this brief installs it with the director's rulings
(brief 728's gate, R1-R8) already applied.

## Goal

When the guest `Input[...]` answer text fails to parse or lower, the
ordinary (queued) request and the resumed request must publish the same
`SessionRequestError::Syntax(StructuredSourceDiagnostic)` built from the
original Input answer, never `Other` on one side and `Syntax` on the
other. The diagnostic must be the exact one the parser produces for that
answer: same message, rendered text, spans, lines and columns. The
repair is a single error-owned representation; it adds no session state,
no lookup, and no default-path work. The accepted brief-727
completed-segment rule and logical-profile epoch validity stay untouched.

## Context pointers

- `.temp/brief-728/selected-design.md`, `producer-conversion-inventory.md`
  (+ `.json`), `proposed-shape.patch`, `frozen/`, `logs/`: the accepted
  design, the class inventory, the proven shape and its raw records. Read
  them; **never write** inside `.temp/brief-728/`.
- Live source: `crates/rum-runtime/src/runtime/session/session_state.rs:496-508`
  (`EvalError`), `builtins_input.rs:304-318` (`resume_input_after_text`),
  `session/request_errors.rs:28-55` (the one mapper),
  `session/session_requests.rs:160-180,270-290,370-455` (ordinary,
  resumed, script and file tails), `session_helpers.rs:588-615` (loader
  message rendering), `suspension/error.rs:270-410` (legacy error wire).
- `.temp/brief-727/class-records.json`: the original two-pause record
  `B726.class.error` (R1).
- `scripts/probe-build.py` (the only build path), `scripts/run-bounded.py`,
  `scripts/report.py`, `docs/dev/codex/TEMPLATE.md`.

## Scope

Files you MAY modify (exact; anything else is a STOP):

1. `crates/rum-runtime/src/runtime/session/session_state.rs` — the
   `EvalError::Syntax` shape, the explicit `From`, the private helper.
2. `crates/rum-runtime/src/runtime/builtins_input.rs` — the two attach
   exits in `resume_input_after_text`.
3. `crates/rum-runtime/src/runtime/session/request_errors.rs` — the one
   mapper.
4. `crates/rum-runtime/src/runtime/session/session_requests.rs` —
   pattern shapes only (the queues and tails are unchanged).
5. `crates/rum-runtime/src/runtime/session/session_helpers.rs` — the
   loader match shape only; emitted messages are unchanged.
6. `crates/rum-runtime/src/runtime/suspension/error.rs` — the legacy
   error wire, optional second payload.
7. `crates/rum-runtime/src/runtime/profiling/tests/queue_boundaries.rs`
8. `crates/rum-runtime/src/runtime/profiling/tests/retained_pause.rs`
9. `crates/rum-runtime/src/runtime/tests/comparison_lowering.rs`
10. `crates/rum-runtime/src/runtime/tests/left_chain_lowering.rs`
11. `crates/rum-runtime/src/runtime/tests/context_and_loader/script_forms.rs`
12. `crates/rum-runtime/src/runtime/tests/definitions_rhs_guards_attribute_revision.rs`
13. `crates/rum-runtime/src/runtime/tests/session_requests/input_resume.rs`
    — the permanent runtime A/B (extend this module; do not add a new
    test module).
14. `crates/rum-host/tests/input_syntax_contract.rs` — NEW permanent host
    contract test.
15. `crates/rum-runtime/src/runtime/model/requests.rs` — rustdoc on the
    host error variant only (no behavior).
16. `docs/host-embedding-and-hardening.md` — one paragraph stating that a
    malformed `Input` answer is a `Syntax` failure on both the queued and
    resumed paths.

Standing generated cells are **always in scope, only through canonical
replay** (never by hand): `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv`,
`crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv`,
`crates/rum-stdlib/src/tests/ownership_graph/inventory/*.tsv`,
`crates/rum-stdlib/tests/suspension_oracle/inventory/logical-profile-required.tsv`,
`logical-profile-bindings.tsv`, `logical-profile-losses.json`,
`typed-slot-stages.tsv`. The rejection census counts are asserted in
`crates/rum-stdlib/src/tests/retention_closure/census/tests.rs`; a
changed count or reason is a director disposition, never a repin.

**MUST NOT touch:** anything under `crates/rum-core/`, `crates/rum-syntax/`,
the parser, the profiling epoch or completed-segment machinery
(`profiling/logical/**`, `suspension/segments*`), `Cargo.toml`,
`Cargo.lock`, `TODO.md`, `AGENTS.md`, `CHANGELOG.md`, `docs/dev/codex/**`,
every frozen `.temp/g15-c2/**` and `.temp/brief-728/**` scene, and the
C2 ledger. No git. No network. No `unsafe`. No live edit before Stage A's
copied proof is complete.

## Target shape (a departure is a STOP)

`crates/rum-runtime/src/runtime/session/session_state.rs`:

```rust
    #[error("{error}")]
    Syntax {
        error: SyntaxError,
        input_source: Option<String>,
    },
```

and, beside the enum (the `#[from]` derive is replaced by this impl):

```rust
impl From<SyntaxError> for EvalError {
    fn from(error: SyntaxError) -> Self {
        Self::Syntax {
            error,
            input_source: None,
        }
    }
}

impl EvalError {
    pub(in crate::runtime) fn with_input_source(self, source: String) -> Self {
        match self {
            Self::Syntax {
                error,
                input_source: None,
            } => Self::Syntax {
                error,
                input_source: Some(source),
            },
            error => error,
        }
    }
}
```

No `#[source]` on the field: `#[error(transparent)]` forwards
`Error::source()` to `SyntaxError::source()`, which is `None`; a plain
field with `#[error("{error}")]` preserves `Error::source()` and
`Display` exactly.

`builtins_input.rs`, inside `resume_input_after_text`, exactly two error
exits and nothing on the success path:

```rust
        let syntax = match parse_expr(&source) {
            Ok(syntax) => syntax,
            Err(error) => return Err(EvalError::from(error).with_input_source(source)),
        };
        let expr = match self.lower_syntax_expr(&syntax) {
            Ok(expr) => expr,
            Err(error) => return Err(error.with_input_source(source)),
        };
```

The result of `self.evaluate(expr)` is never wrapped: a nested `Input`
or `ToExpression` owns its own source.

`request_errors.rs`, the one existing mapper keeps one classification
surface:

```rust
        EvalError::Syntax {
            error,
            input_source,
        } => input_source
            .as_deref()
            .or(source)
            .map(|source| {
                SessionRequestError::Syntax(
                    rum_syntax::SourceDiagnostic::new(
                        rum_syntax::SourceText::new(source),
                        error.clone(),
                    )
                    .structured(),
                )
            })
            .unwrap_or_else(|| SessionRequestError::Other {
                message: error.to_string(),
            }),
```

`suspension/error.rs`, the legacy wire: the bare one-payload encoding is
byte-identical; a present source appends exactly one string. Decoding an
absent second payload yields `None`, a present empty string yields
`Some("")`. The consumed `debug_assert!(args.next().is_none())` contract
must still hold.

Every other `EvalError::Syntax(..)` site migrates its pattern shape only
(`Syntax { error, .. }` or `Syntax { error: SyntaxError::X { .. }, input_source: None }`).
**No assertion is weakened, deleted or relabelled.** The two direct
lowerer assertions require `input_source: None` and keep their
span/limit conditions.

## Must not change (beside every cost bar)

- `Display` text of any `EvalError`; `std::error::Error::source()` for
  `EvalError::Syntax` stays `None`; no `#[source]`.
- Ordinary and resumed completion/publication order, the existing
  resumed-input clone, message checkpoints, request retirement, file
  reread order, cancellation and timeout precedence, abandonment and
  reset behavior.
- No session field, thread-local, request-id map, retired-request
  registry, second classifier, or Syntax-to-Other normalization.
- No default-path initialization, clear, lookup, allocation, lock,
  diagnostic counter or branch. No padding instructions to make a count
  equal. No `#[inline]`/`#[cold]` tuning of the cold path.
- No clone of the guest tree and no moved completion boundary.

## Stage A — copied proof on the FINAL shape, before any live edit

New scene `.temp/brief-729/` only. Copy the brief-728 harness scripts
(`freeze.py`, `build.py`, `run.py`, `panel.py`, `parity.py`, `cost*.py`,
`audit.py`) into the new scene and repoint their root; read the frozen
inputs from `.temp/brief-728/frozen/`, never write there.

1. Freeze the live source hashes without git; assemble `source-base`
   (live tree + the six brief-727 harness files named in
   `.temp/brief-728/base-harness-files.json`) and `source-candidate` (the
   target shape above, no `#[source]`). Also assemble
   `source-instrumented`: the candidate plus an env-gated,
   error-path-only print (`B729_SOURCE_ARM` / `B729_SOURCE_HIT`) and an
   `B729_OMIT_INPUT_SOURCE` switch that makes `with_input_source`
   return the error with `None` — the own source-provenance omission.
2. Build every arm through `scripts/probe-build.py` with a scene adapter
   (Rust 1.85.0, offline, locked, `-j 8`, release, one stable active
   path, one Cargo process, **unique per-invocation `LOGS` root** so
   `--force` can never overwrite a prior raw Cargo log). One `--force`
   sha256 spot check with a saved prior receipt.
3. Reproduce the red and the controls first, from raw records:
   `Input["answer"]` answered `(` → queued `Other { message:
   "incomplete input: expected expression at bytes 1..1" }` against
   resumed `Syntax(StructuredSourceDiagnostic { … })`; native ten-field
   rows SAME; valid epochs, zero faults. Controls both SAME:
   `Input["answer"]` answered `83`, and `InputString["answer"]` answered
   `"("` (a string value, not an error).
4. Watch the named parity assertion **red on base, green on candidate,
   red on the omission** (the omission runs the instrumented binary with
   `B729_OMIT_INPUT_SOURCE=1` and must print a real ARM/HIT line). The
   red/green/red receipts are the proof that the attach is what closes
   the class.
5. Whole-observation equality on the minimal case, the two controls and
   the nine class cases (empty answer, non-ASCII, multiline, unclosed
   delimiter, 129-delimiter parse depth, nested answer, `Input` in the
   prompt argument, multiple inputs, prints/messages/trace): candidate
   queued == candidate resumed == base resumed, byte for byte; real ==
   disabled whole-record; native rows equal to base.
6. The original two-pause record `B726.class.error` follows R1: the
   frozen record stays byte-identical, and the candidate changes exactly
   `semantic_queued` and `semantic_same`. Any other field moving is a
   STOP.
7. Confirm 118 active epoch summaries `valid=true faults=0`, the 60
   P1566/P1568 cells SAME with 0 new keys, and the 55/55 + 60/60 matrix
   cells re-derived from the frozen raw logs.
8. Disabled cost, two draws per arm at N=10 and N=20, one stable path,
   release CGU=1, profiling off, serial and alone. Bar (R2): no changed
   call-site row executes **more** instructions per call than base; the
   changed function's call count is identical; every other measured named
   row is exact; the capture helper has zero entries on the disabled
   success path. A deterministic equal-call increase on any named rum row
   is a STOP; a decrease is recorded, never padded. The core-language
   microbench is NA for this shape (no label evaluates `Input`); state
   that reason explicitly.

## Stage B — install, format, then standing replay as the last edit

1. Apply the final shape to the live tree; run `cargo fmt` on the touched
   files until `cargo fmt --check` is clean. Formatting is the last
   source edit.
2. Before the first acceptance suite, replay every standing inventory on
   the installed tree, through the canonical generators and never by
   hand: contract fingerprints (`regenerate_local_contract_fingerprints`
   and `fingerprint_diff`), `production_inventory_is_exact` via the pure
   `director-gate.py` functions, `local_production_acceptance` in both
   the default and `--no-default-features` builds (copy the certificate
   cells the tests write), the rejection census with
   `RUM_CENSUS_DUMP` on both base and candidate, LP01 (`logical_counter_port`
   at 8 and 1 threads plus the five-mode parents), typed-slot and
   frame-root. Every replay verdict line is quoted verbatim.
3. Stop conditions: an existing identity/status/disposition/exclusion
   change, a vanished row, a new `UnknownHostPayload`, a new row outside
   touched code, or a census count/reason move. A stale
   fingerprint/certificate/position cell is an ordering artifact: replay
   fixes it and it is never reported as a regression.
4. After the last replay edit, re-run the copied proof records on the
   installed tree and confirm they equal the Stage-A candidate receipts.

## Stage C — acceptance

`cargo fmt --check`; `cargo check --all-targets --offline --locked -j8`
with no new warnings; then, at 8 and again at 1 test threads:

- focused: `-p rum-runtime -- input_syntax` (the new/updated tests) and
  the migrated test modules;
- `-p rum-runtime --lib` full suite;
- `-p rum-stdlib` default and `--no-default-features`;
- `-p rum-host`;
- the seven standing skips exactly as listed in brief 728's draft
  (`lp03`, `lp04`, `lp05`, `lp658_host_eval_reset`,
  `lp658_host_resume_reset`, `lp655`, `lp656`); the first five run
  separately with `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`; the last two stay
  the accepted pre-existing root-context reds; no new skip and no broader
  exclusion;
- one `cargo test --workspace` at 8 threads, `--no-fail-fast`.

The no-default `wide_symbol_sort_matches_the_full_list_and_pinned_base_rendering`
failure stays the separate pre-existing obligation; reproduce it on
untouched base at one thread before classifying it. Any new genuine red
is reproduced at `--test-threads=1` before it is called a defect.

## Permanent tests

- `input_resume.rs`: `queued_and_resumed_malformed_input_report_the_same_syntax_diagnostic`.
  For each answer in `["(", "1 ! 2", "", "\"é\" + )", "f[", 129 nested delimiters]`:
  the queued arm queues the answer and evaluates `Input["answer"]`; the
  resumed arm suspends and resumes with the same answer. Assert both
  are `SessionRequestError::Syntax`; assert the queued and resumed
  `SessionRequestFailure` are equal whole (error, messages, prints,
  trace); assert the diagnostic equals an **independently** constructed
  `rum_syntax::SourceDiagnostic::new(rum_syntax::SourceText::new(answer),
  rum_syntax::parse_expr(answer).unwrap_err()).structured()`; assert the
  pending request is cleared and a fresh request succeeds. Do not weaken
  the existing resumed `Syntax` test.
- A legacy-wire unit test: an `EvalError::Syntax` with `Some(source)`
  round-trips through `ContinuationValue`; a bare one round-trips to
  `None`; the bare encoding bytes are exactly the old payload.
- `crates/rum-host/tests/input_syntax_contract.rs`: a malformed answer
  through `HostSession` maps to `HostFailureCode::Syntax` with the same
  rendered text on the queued and resumed paths, and a fresh request
  succeeds afterwards.

## Hard rules

- No git, no network, no `unsafe`, no edit outside Scope, no write into
  any frozen scene, no `.temp` file outside `.temp/brief-729/` (the
  sanctioned shared probe cache and the mandated `CARGO_HOME` metadata
  are the only exceptions, and both are disclosed).
- Never weaken an assertion, a comparator, a classifier or an inventory;
  never normalize Syntax into Other; never fabricate a source.
- Stop-loss: stop after two measured attempts with no progress, on an
  unexplained calibration failure, an invalid-epoch parity certificate, a
  frozen-scene write, or a public-API/scope departure, and say which.
- Keep every attempt, including failed ones, with its raw log and a
  unique `LOGS` root.

## Report format

`rum-report/1` in `.temp/brief-729/report.json`: the installed shape and
its exact diff, the copied red/green/omission receipts, the cost table
with both draws, the replay verdict lines, the acceptance counts, every
changed inventory row with file/line, the disclosures, and the open
questions. Run `python3 -I scripts/report.py check .temp/brief-729/report.json`
and quote its verdict. No source change is authorized outside Scope.

## Director gate (2026-11-11) — Stage A accepted, the ownership stop ruled, and continuation 1 completes the round

Run: log `.temp/logs/cdx_brief_729.log`, 27,953 lines, sha256
`3cbd68dc7ac54193d527bd67c37d7071cfaad2cb823454bff1f5dfad995f6f3e`,
memory-scope peak 3,074,510,848 B, oom 0, oom_kill 0, rc=0; report
`.temp/brief-729/report.json`, 59 runs, `report.py check` 289 citations
(my own run: OK).

**My gate, reproduced myself.** The live delta is exactly the 17
authorized files (16 modified + 1 added) and nothing else; the 16
source/test/doc files are byte-identical to `source-candidate`, and the
17th, `local-contracts.tsv`, is byte-identical to what the canonical
generator emits on the installed tree (my own
`regenerate_local_contract_fingerprints --ignored` run).
`proposed-shape.patch` applied to `source-base` reproduces
`source-candidate` 16/16; `source-base` is `source-entry` plus exactly
the six brief-727 harness files, and `source-entry` equals HEAD. No
file under the frozen brief-726/727/728 and `g15-c2` scenes (other than
the mandated `cargo-home`) is newer than the dispatch, and the log
contains no git or network command.

From the raw records: base == frozen on all four; omission == base;
disabled == candidate; on both red instances the candidate's queued
observation is byte-equal to its resumed one **and to the base resumed
diagnostic**, with the delta against frozen exactly
`{semantic_queued, semantic_same}`; both controls byte-exact; all nine
extended class cases are base-DIFF, candidate-SAME, disabled ==
candidate, with phase and native rows equal to base; my parse of every
log finds 118 epoch summaries, all `valid=true faults=0
context_leases=0`; the 55/55 and 60/60 frozen matrix cells re-derive
with `raw_real == raw_disabled`.

**Cost, re-parsed by me from the eight raw Callgrind captures.**
`resume_input_after_text` reads 160.0/2 calls on base and 158.0/2 on
candidate in both draws and both N — 80.0 -> 79.0 Ir/call at identical
calls; `eval_expr_request_from_checkpoint` 103.0,
`resume_eval_request_from_checkpoint` 1449.0 and the `EvalError` drop
29.0 are exact on every arm and draw; `with_input_source` has zero
entries in all eight captures; my sweep over every captured rum row
finds only the standing `intern_symbol_text` / `Symbol::system` /
`intern_text` rows, whose within-arm draw spreads exceed the arm
difference and whose signs alternate. The forced rebuild is a fresh
compile (`cache_hit=false`) sha-equal to base `72e1bb87…`.

**Host:** `cargo check --workspace --all-targets --offline --locked
-j8` rc=0 with only the three standing `test_builtins` warnings;
`cargo fmt --check` rc=0; every touched pre-existing test module and
the pre-existing resumed-`Syntax` test are green on my run.

**F1 — the STOP, ruled.** The canonical ownership discovery reports
exactly 2 REMOVED, 2 NEW and 83 MOVEs (`new_UnknownHostPayload=0`). The
removed rows are the `error (transparent)` and `from` attributes on
`EvalError::Syntax` (`session_state.rs:501:5`, `:502:12`); the new rows
are `error ("{error}")` on the same variant and the `|source|` capture
in the legacy-wire branch (`suspension/error.rs:419`), both inside
touched files. My own multiset comparison of all seven inventory files
against the generated set finds no other identity change after position
normalization. These four rows are the approved R3 shape's own text —
the attributes R3 deletes and the payload R3 adds — so the vanished row
*is* the repair, not collateral. **Ruled:** the canonical replay may
install exactly this delta; a second discovery must read zero actions;
any other identity, status, disposition or exclusion change remains a
STOP.

**F2 — a red found by the first execution.** Neither permanent test had
been run. On first run both abort with SIGABRT `stack overflow` at 8
**and** at 1 threads. Reproduced by me; gdb locates the recursion in
`rum_syntax::parser::front` (`parse_primary -> parse_function_expr ->
parse_binary_expr -> ... -> parse_postfix_expr -> parse_primary`,
`front.rs:560`) on the 129-parenthesis answer at the 128-delimiter
guard, inside libtest's 2 MiB test thread — the test's own independent
`parse_expr` construction overflows too. `RUST_MIN_STACK=3M` passes;
2M aborts; both binaries. The parser is untouched and release is
unaffected (the release extended panel records the whole observation),
so this is a test-shape defect, not a product regression. It also
exposes a pre-existing gap:
`MAX_DELIMITER_NESTING_DEPTH`'s rationale claims the cap stays below a
2 MiB debug thread's stack, and the parenthesised path at 129 does not.
That gap is outside this line (`rum-syntax` is MUST NOT touch) and
becomes a recorded open obligation. **Ruled:** both tests keep all six
answers and every assertion, and run their bodies on a spawned thread
with an explicit measured stack (the repo's own convention, at least 2x
the measured 3 MiB requirement); no answer dropped, no assertion
weakened, no `RUST_MIN_STACK` dependence.

**Continuation 1 (this brief, re-dispatched) completes the round.**

1. Fix the two permanent tests' stack shape; retain the pre-fix abort as
   a receipt and quote both fixed tests green at 8 and at 1 threads.
2. Install the ruled ownership delta through the canonical replay;
   re-run discovery to zero actions; quote the full action table and
   every verdict line. The prepared `replays.py ownership` mode's
   blanket `REMOVED` refusal is superseded for exactly the two ruled
   rows; any other `REMOVED` or `MOVE-FILE` remains a STOP.
3. Run the remaining Stage-B replays: certificates (default and
   `--no-default-features`), census (both arms plus the compiler
   oracle, identities exact), LP01 (`logical_counter_port` at 8 and 1
   threads plus the five-mode parents), typed-slot and frame-root.
4. Re-run the copied proof on the installed tree and confirm the
   records equal the Stage-A candidate receipts.
5. Stage C acceptance: `fmt --check`; `check --all-targets`; the
   focused tests (the exact new runtime test names — the `input_syntax`
   filter alone does not match the queued/resumed parity test — and
   `-p rum-host --test input_syntax_contract`); the migrated modules;
   `-p rum-runtime --lib`; `-p rum-stdlib` default and no-default;
   `-p rum-host`; the seven standing skips exactly as listed (the first
   five with `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`, the last two the
   accepted pre-existing root reds); one `cargo test --workspace` at 8
   threads with `--no-fail-fast`; any new red reproduced at
   `--test-threads=1` before it is called a defect.
6. Report and stop.

The production shape, the Stage-A receipts and the cost gate stand and
are not re-run. No production edit is authorized by this continuation;
the only source change is the two tests' thread shape. Any further
inventory movement is a STOP.

## Director gate (2026-11-11, continuation 1)

**Run facts.** Log `.temp/logs/cdx_brief_729.log`, 16,111 lines,
1,504,286 B, sha256 `c6031649…`; memory-scope peak 4,908,994,560 B, oom
0, oom_kill 0, rc=0; report `.temp/brief-729/report.json`, 96 runs,
`report.py check` 538 citations; no executed git or network command.

**My gate.** The continuation's own delta is exactly 7 files: the two
test bodies (`crates/rum-host/tests/input_syntax_contract.rs`,
`crates/rum-runtime/src/runtime/tests/session_requests/input_resume.rs`)
and five canonical inventories; production changes 0; the Stage-A 16
source/test/doc files are byte-identical to the proven candidate. Both
test bodies run on an explicit 8 MiB `std::thread::Builder` stack with
all six answers and every assertion intact; the pre-fix aborts are
retained at 8 and 1 threads; my own runs of both fixed tests are 1/0 at
8 and at 1. Ownership: the action table is exactly 83 MOVE / 2 REMOVED
/ 2 NEW; my own multiset check of the three TSVs against HEAD confirms
only position moves and the two ruled identity changes, 0 new host
payload, second discovery 0 actions, verify `61/79/8,852`.
Fingerprints: 68 rows, fingerprint cell only, keys equal. Promoted:
exactly 2 rows. Census: 1,220 rows in both arms and the two frontier
dumps byte-identical; compiler oracle green. LP01: producer identity
`b41ac6a5dbf0e9a4`, required 1,115 / bindings 875 / G13 240; the
five-mode child observations are byte-identical at 8 and 1 (only the
libtest elapsed-time line differs); loss positions PASS. Frame-root, my
own run on the live tree: 15/0, 568 named frames. `fmt --check` clean;
`report.py check` OK (538).

**The STOP is well taken, and it is three deltas, not two.** I
reproduced the typed-slot red myself at 1 thread: 1 passed / 9 failed /
1 ignored with the same names. Mechanism, read from the census: the
`#callN` anchor counts AST call expressions per caller; the approved R3
shape replaced the tuple-variant constructor `Self::Syntax(..)` (an
`ExprCall`, ordinal 2) with a struct-variant literal (not a call), and
the new `input_source` closure adds one `string_from_expr` call
(ordinal 3). `syntax_error_from_expr` therefore moves 3 → 2, and every
recorded ordinal from 4 onward realigns on the existing rows. My own
check over all 755 active `Whole`/`Nested`/`Transform`/`M` anchors finds
exactly one vanished live anchor (`Nested0387`) and no removed identity
reappearing. The worker stopped at the emitter's first panic, so it did
not reach the next failure: the exact test-support `X` set is 48 active,
and the new permanent round-trip test's four legacy-wire calls make it
52.

**Ruled — three exact deltas, nothing else.**

- **T1.** In
  `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv`
  row `Nested0387` changes only its identity cell:
  `…from_continuation_expr#call3:syntax_error_from_expr:` becomes the
  same text with `#call2`. Same function, same decoder (`D119`), same
  statement (`suspension/error.rs:413:24-46`), same population: this is
  a position-only MOVE of an earned receipt, not a new receipt.
- **T2.** In `crates/rum-stdlib/src/tests/ownership_graph/root_set.rs`
  the `EvalError` row changes only its field string, `"Syntax.0"` →
  `"Syntax.error"` (target `rum_syntax::error::SyntaxError` unchanged).
  The `SessionRequestError` `"Syntax.0"` row and all six edge assertions
  stay byte-identical. This file is added to scope for this one
  identifier only.
- **T3.** Four new rows join the `X` block of `typed-slot-stages.tsv`,
  in the existing hash-id row shape, with `X` + FNV-1a-64 of the
  identity as the id (verified against all eight existing hash ids):
  `Xa0a2bf2cebe978d6` — `…input_syntax_legacy_wire_preserves_optional_source_and_bare_payload#call5:from_continuation_expr:`;
  `X191e8771f0ce8c6a` — `…#call9:from_continuation_expr:`;
  `Xd0fd530e2fae9485` — `…#method10:into_continuation_expr:error`;
  `X6c4d31b0c973b5f4` — `…#method17:into_continuation_expr:bare`;
  file `crates/rum-runtime/src/runtime/suspension/error.rs`, name the
  decoder, population `test-support`, code `R`, status `legacy-test`,
  obligation `-`. The new test is the approved round-trip proof; the
  receipts are its canonical consequence, not a weakening.

**Calibration (mine).** In a byte copy
`.temp/director/probe-729-rebind/live` carrying exactly T1–T3, the
typed-slot family reads 10 passed / 0 failed / 1 ignored at 8 threads;
the live tree without them reads 1/9/1 at 1 thread.

The parser-stack question is answered by the recorded obligation:
`rum-syntax` stays untouched, the tests' explicit stack is the fix, and
`TODO.md` owns a future parser-depth brief.

**Continuation 2 completes the round.**

1. Install exactly T1–T3; no other file changes.
2. `typed_slot_census` at 8 and 1 threads: quote 10/0/1 both.
3. Frame-root and LP01 parents at 8 and 1; quote every verdict line.
4. Re-run the installed copied proof; confirm the records equal the
   Stage-A candidate receipts.
5. The two fixed permanent tests at 8 and 1 (both binaries): quote 1/0.
6. Stage C acceptance exactly as written above, including the
   untouched-base no-default wide-symbol reproduction and one workspace
   run at 8; any new red reproduced at `--test-threads=1` before it is
   called a defect.
7. Report and stop. Any further inventory movement is a STOP.

## Director gate (2026-11-11, continuation 2)

Run facts: log `.temp/logs/cdx_brief_729.log`, 23,923 lines, 1,928,455 B,
sha256 `02d15a79…`, memory-scope peak 14,439,993,344 B, oom 0, oom_kill
0, rc=0; report `.temp/brief-729/report.json`, 153 runs, 17 PASS /
2 FLAGGED / 1 FAIL, `report.py check` 1,195 citations; no executed
git/network command.

**Scope and install.** My own manifest walk: 2,320 files at entry and at
exit, with the entry→final delta exactly four paths —
`root_set.rs` (T2, one identifier), `typed-slot-stages.tsv` (T1's
one-line `#call3`→`#call2` rebind plus T3's four `X` rows),
`local-contracts.tsv` (68 fingerprint cells) and `promoted.tsv` (2
rows) — 0 added, 0 removed; production changes 0 and the permanent test
bodies unchanged. The live tree equals `final-hashes.json` on all
2,320 files, and the acceptance copy `source-continuation2` equals it
except three `__pycache__` build artifacts. My continuation-1 probe
(the continuation-1 tree with my calibrated T1–T3) differs from the
live tree only in the two derived tables and the two ruling docs, so
the installed T1–T3 text is byte-identical to the calibrated ruling. I
recomputed all four T3 ids as `X` + FNV-1a-64 of the identity: all four
match, and the same rule reproduces the eight pre-existing hash ids.
`SessionRequestError` keeps `"Syntax.0"` and the six edge assertions
are intact; only the `EvalError` field string changed.

**The derived refresh, ruled.** The contract fingerprint's rot guard
hashes every `.rs`/`.py`/`.toml` under `crates/` by design
(`contracts.rs:88-120`), so any source edit under `crates/` — T2
included — expires all 68 contract cells and the promoted certificate
values that embed them. I checked the two files column by column: only
column 18 differs in the 68 `local-contracts` rows, and only columns 4/5
in the 2 `promoted` rows; no row was added, removed or reordered and
every id is unchanged. My own run of
`regenerate_local_contract_fingerprints` on the live tree emits a file
byte-identical to the installed `local-contracts.tsv` (`cmp` clean),
and `local_production_acceptance` is green on both the default and
`--no-default-features` arms with both files byte-unchanged before and
after. Ruling: this is the canonical replay consequence of T2, not
inventory movement; no identity, status, disposition, exclusion, count
or reason changed, and the "no other file changes" line is amended for
exactly these two derived tables.

**My reproductions.** `cargo fmt --check` clean; `check --all-targets`
rc=0 with zero warnings; ownership `production_inventory_is_exact` green
at `61/79/8,852` with the worker's discovery reading zero actions;
`typed_slot_census` **10/0/1** at 8 threads (67.82 s) and at 1
(243.81 s); `frame_root_trace` **15/0** at 8 and 1 with 568 named
suspension frames and 59 named pure finalizers; the LP01 five-mode
parents green at 8 (164.85 s) and 1 (164.97 s) — `required=1115
bound=875 executed=875 encoding-owned=240`, counter-losses 22,772 in
`all`, every mode line `ok`; the two fixed permanent tests 1/0 at 8 and
at 1 (runtime malformed-input and `-p rum-host --test
input_syntax_contract`); the five switch-gated host tests 5/0 at 8 and
at 1 under `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`; the two accepted
root-context reds exactly 0/2 at 8 and 1; the untouched-base no-default
wide-symbol failure reproduced at `ordering_sets.rs:271` (0/1, exit
101) with that test file unchanged by the round. The installed copied
proof reads byte-equal to the Stage-A candidate records on the tree I
verified byte-identical to the live tree.

**The STOP.** The worker's workspace run (4,225 passed / 9 failed / 61
ignored at 8 threads) fails only the nine native ZeroMQ loopback tests,
each EPERM at the bind before evaluation; the helper is unchanged from
entry, the failures reproduce at 1 thread, and no flag, skip, assertion
or repin was added. That is the standing worker-sandbox boundary, not a
product signal. I ran the prescribed workspace myself on the host:
**4,234 passed / 0 failed / 61 ignored** over 65 summaries, rc=0, with
the seven standing skips — on the tree where the nine loopback tests
pass.

**Rulings.** Q1 the STOP is accepted as the environment boundary;
Q2 the derived-cell refresh is authorized as T2's canonical replay
consequence; Q3 the round is complete and the install is accepted. C2
stays Open at 314 Residual + 1 Lost; no ledger, settlement, port table,
`CHANGELOG` or S10 item is touched. Next: continuation 13 of brief 623
on this installed source.
