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
