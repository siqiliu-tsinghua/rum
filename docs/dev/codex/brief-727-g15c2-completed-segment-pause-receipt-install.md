# Brief 727 — G15-C2: install the completed-segment pause receipt

Stage: **implementation**. Accepted design: brief 726 (rule (b),
candidate2). `Scope:` the two segment files, a new permanent runtime
test module, one stdlib rum-vs-rum regression, one doc paragraph, and
the canonically regenerated standing inventories. Nothing else.

## Goal

Install one enabled-only ownership rule in the logical profiler. An
exact segment generation whose ordered explicit finish succeeded while
its epoch was valid is promoted to `ExplicitlyFinished`; a later pause
of that same receipt is inert. Every other state keeps its existing
check: a live displaced segment still faults, an ordinary
end/Drop/`Ended` state is never a completion certificate, a missing
TLS slot is never evidence, a foreign owner, stale epoch, reused slot,
cancellation and cap behave exactly as today.

When you are done: the three frozen two-pause program failures
(`direct.two`, `replace.two`, `unheld.two`) are valid and whole-record
equal to their queued arms; the seven controls and every queued arm
are byte-identical to the frozen red records; real and disabled
instrumentation records are byte-identical; the prepared boundary
still samples `None` immediately; prepared16 and prepared56 are exact;
all five modes are SAME; the disabled instruction rows of every
touched function are unchanged; the permanent runtime falsifier is
red on the pre-fix source for its named assertion and green on the
installed source at 8 and 1 threads; the class-wide rum-vs-rum
regression is red on the pre-fix source and green on the install; all
seven standing inventory families replay; and the full acceptance
suite is green.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/dev/codex/TEMPLATE.md`;
  `docs/dev/profiling.md` is the canonical logical-profiling contract
  (the design brief's `docs/profiling.md` pointer is stale).
- Accepted design evidence (read-only; never write): `.temp/brief-726/report.json`,
  `OWNERSHIP-RULE.md`, `segments.rs.candidate2.patch`,
  `stack.rs.candidate2.patch`, `class-inventory.json`,
  `trace-proof-final.json`, `disabled-cost-final.json`, `frozen/`.
  `.temp/g15-c2/c11/` is frozen evidence: read it, never write it.
- Live source: `crates/rum-runtime/src/runtime/profiling/logical/segments.rs`
  (`pause_admitted` around :548, `end_admitted` around :621,
  `finish_active_admitted` around :704, `install` around :430);
  `segments/stack.rs` (`push` around :78);
  `crates/rum-runtime/src/runtime/eval_core/call.rs:519-531`;
  `suspension/core/profile_queue/prepared.rs:40-80`;
  `suspension/core/driver.rs:155-156`.
- Design gate: `docs/dev/codex/brief-726-g15c2-prepared-consumer-context-retirement-design.md`,
  section "Director gate (2026-11-11)".

## Scope

Files you MAY modify, and no others:

1. `crates/rum-runtime/src/runtime/profiling/logical/segments.rs`.
2. `crates/rum-runtime/src/runtime/profiling/logical/segments/stack.rs`.
3. New `crates/rum-runtime/src/runtime/profiling/logical/segments/completion_tests.rs`
   with a `#[cfg(test)] mod completion_tests;` declaration in
   `segments.rs`. The module name may differ; it must be a permanent
   test module, not a scratch name. Test-only `eprintln!` PASS lines
   are allowed in it.
4. `crates/rum-stdlib/tests/suspension_oracle.rs` for one new
   `#[test]` (or a module declaration plus one new file under
   `crates/rum-stdlib/tests/suspension_oracle/`) providing the
   class-wide rum-vs-rum regression. Do not edit
   `fixtures/logical_profile/plans.tsv`, the LP01 inventories, or any
   frozen expectation.
5. `docs/dev/profiling.md` — one paragraph in "Lifecycle and epochs".
6. The canonically regenerated standing inventory files that move.

Files and areas you MUST NOT touch: any other source or test file
(including `profiling/logical/mod.rs` — the installed shape carries no
`B726_EPOCH` print), `crates/rum-core/src/expr.rs` and public
representation/serde shapes, `Cargo.toml`, `Cargo.lock`, `AGENTS.md`,
`PROGRESS.md`, `TODO.md`, `CLAUDE.md`, `docs/dev/codex/`, the
historical scenes under `.temp/g15-c2/`, `.git`, and any
`docs/` file other than the one paragraph above.

## Accepted source shape (departure is a STOP)

The rule is (b) only, exactly as fixed in
`.temp/brief-726/OWNERSHIP-RULE.md`; the candidate2 patches are
authoritative for body ordering and the installed shape excludes the
test-only observer print and the scratch module name.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SegmentState {
    Live,
    Ended,
    ExplicitlyFinished,
}
struct SegmentCompletion {
    epoch: Arc<Epoch>,
    state: Cell<SegmentState>,
    _storage: Storage,
}
// Active: epoch: Arc<Epoch> is replaced by completion: Rc<SegmentCompletion>.
// All active.epoch reads become active.completion.epoch.
// Segment: pub(in crate::runtime) epoch: Arc<Epoch> is replaced by
// completion: Rc<SegmentCompletion>; record, segment, index and the
// !Send marker are unchanged.
// Stack::push gains its last parameter, completion: Rc<SegmentCompletion>.
```

In `install`, before the active-slot insertion and after the existing
head/thread checks: claim
`size_of::<SegmentCompletion>() + 2 * size_of::<usize>()` through the
existing `storage_admitted` path (a claim failure returns `None`
through the existing epoch invalidation), construct the `Rc` with
`SegmentState::Live`, clone it into `Stack::push`, and return it
inside the `Segment`. One fresh receipt per install, even at a reused
index or for the same record. No receipt is created by `finish_active`
or by any reader.

At the beginning of `pause_admitted`, before the unchanged top check:

```rust
if self.completion.state.get() == SegmentState::ExplicitlyFinished {
    return;
}
```

Otherwise the original pause body runs unchanged with
`self.completion.epoch` in place of `self.epoch`.

Inside `end_admitted`, after the exact generation/epoch/record/order
validation, the successful slot take, park and
`active.path.interval(...)`:

```rust
self.completion.state.set(SegmentState::Ended);
```

`finish_active_admitted` keeps its original active-length, path and
active-thread checks. Replace only its temporary-token construction:

```rust
let completion = ACTIVE.with_borrow(|stack| {
    let active = stack.frame(id.index)?;
    (active.completion.epoch.id == epoch.id
        && active.path.record == record
        && active.segment == id.segment)
        .then(|| Rc::clone(&active.completion))
});
let Some(completion) = completion else { return; };
let token = Segment {
    completion,
    record,
    segment: id.segment,
    index: id.index,
    _native_thread: PhantomData,
};
token.end_admitted(/* the same _access argument */);
if epoch.is_valid() && token.completion.state.get() == SegmentState::Ended {
    token.completion.state.set(SegmentState::ExplicitlyFinished);
}
```

Terminal states are one-way: `Live` (executing or parked), `Ended`
(ordinary end/Drop; never a pause exemption), `ExplicitlyFinished`
(ordered explicit finish; repeated pause and Drop inert). A faulted
epoch stays invalid and never certifies parity; no state reopens an
operation. The receipt owns `Epoch` and its `Storage` claim, holds no
`RecordPath`/`Context`/guard backreference, and is released on the
last `Active`/`Segment` Drop. No unbounded retired-generation
registry, no second admission, no fabricated guard, no missing-top
suppression, no phase-specific exemption.

Omission falsifier: delete only the
`ExplicitlyFinished` assignment line (the condition may remain). It
must restore the frozen red direct.two whole record and its
instrumentation fault while raw ARM/HIT markers identify the exact
record and generation.

The accepted design already proved the rule in a byte copy. This
round re-proves it on the exact installed shape (the observer print
and scratch module name removed) before the live install, then runs
the acceptance on the live tree.

## Stage A — copied proof on the exact installed shape (no live edit)

1. Freeze the entry (file hashes of the live source tree; the design
   hashed 2,301 files) under `.temp/brief-727/`. Build only through
   `scripts/probe-build.py` with the scene environment
   (`CARGO_HOME=.temp/g15-c2/cargo-home`, scene `TMPDIR`,
   `RUSTUP_TOOLCHAIN=1.85.0`), release CGU=1, offline and locked,
   `-j8`, one Cargo process at a time; one `--force` sha256
   spot-check.
2. Apply the exact shape and the permanent tests to a byte copy.
   Reproduce the frozen red panel from the design data: 3 DIFF /
   7 SAME, semantic SAME 10/10, real == disabled, the three invalid
   epochs and the exact `C11_PAUSE_MISMATCH` markers; all 120
   original differences stay present and unbound.
3. Candidate: 10 valid whole-record SAME, zero faults, the seven
   controls and all ten queued arms byte-identical, prepared16 and
   prepared56 exact, five modes (disabled, lexical, timing, cache,
   all) 50 SAME, repeated/nested/Return/Throw/Abort programs, and
   the malformed-answer probe retained and flagged exactly as the
   design left it.
4. Omission: delete only the promotion line; raw ARM/HIT markers; the
   ten full records must be byte-identical to the frozen red records
   (compare the record files, not a summary); restoring the line
   returns valid equality.
5. Permanent runtime falsifier (`completion_tests.rs`): the six
   outcomes (`None`, `Some`, `Error`, `Return`, `Throw`, `Cancelled`),
   a repeated pause of the still-live `ContextGuard`, active-slot
   reuse for a live record, a Drop of the old guard that does not
   disturb the replacement, zero faults, and epoch quiescence.
   **Assert `epoch.is_valid()` and `faults == 0` immediately after the
   repeated pause, before any reuse open**, so the pre-fix red lands
   on that named assertion and not on a helper `unwrap`. Watch it red
   on the pre-fix copy at 8 and 1 threads, then green on the candidate
   at 8 and 1. Keep the missing-slot (`Ended` still faults), displaced
   live (still faults), cross-thread resume, reset/cap and final-Drop
   receipt-storage tests.
6. Class-wide rum-vs-rum regression (the stdlib test family): the
   three two-pause programs, all with setup
   `log={};c11f[83]:=Input["finish"];` and answers 83 then 89 —
   `c11f[Input["rule"]]`, `ReplaceAll[c11f[1],1:>Input["rule"]]`,
   `ReplaceAllUnheld[c11f[1],1:>Input["rule"]]` — run queued and
   resumed and compare the complete native ten-field rows
   (`Session::logical_profile_test_rows`), not a summary. Assert the
   required key families are non-empty and that the queued arm is
   non-trivial, so an empty comparison cannot pass. Watch it red on
   the pre-fix copy at 8 and 1 and green on the candidate at 8 and 1.
   A `direct.prep` control stays identical.
7. Layout receipts on both arms: `size_of::<Segment>()`,
   `Option<Segment>`, `ContextGuard`, `Option<ContextGuard>`, `Call`,
   and the `Active`/`ActiveSlot` pointer shape. Base must equal
   candidate (the design measured 32/32/120/120/8).
8. Stop and report the copied evidence before the live install if any
   of Stage A is red for an unexpected reason. A compile error you are
   fixing is not a wall; the standing stop-loss is two consecutive
   concrete attempts gaining none of [proved rule, closed class row,
   calibrated falsifier, candidate record].

## Stage B — live install and standing replay

Install the exact accepted bytes (scope items 1-5). Then replay the
standing inventories, as the **last edit step and before the first
acceptance suite**, in this order:

1. Retention fingerprints:
   `regenerate_local_contract_fingerprints -- --ignored --nocapture`;
   verify only fingerprint cells move.
2. Ownership discovery and reconciliation:
   `production_inventory_is_exact` with `--nocapture`; reconcile rows
   with the canonical `own_replay` logic from
   `scripts/director-gate.py` (do not invoke git or the full director
   gate). New `UnknownConstruct` rows in code this brief added or
   changed are expected and stay unknown, each with file and line.
3. Certificates: `local_production_acceptance` in the default and
   `--no-default-features` builds; merge the freshly emitted
   `.temp/brief-546/certificates/promoted-hash-{true,false}.tsv`
   through canonical `merge_promoted`, then rerun both. Never write a
   cell by hand.
4. Rejection census: `production_rejection_census_is_complete` with
   `RUM_CENSUS_DUMP=<new-scene>/frontier.tsv`; pair by
   kind/file/callee/reason; also run
   `exact_types_and_constructor_claims_match_rustc`.
5. LP01: `cargo test -p rum-stdlib --test suspension_oracle --
   logical_counter_port`; refresh producer sites/hashes, binding
   positions and the loss producer table only by canonical replay,
   preserving statement text and every expected loss/identity.
6. Typed slots: `cargo test -p rum-stdlib --test typed_slot_census`.
7. Runtime frame roots and the named-frame count: their exact checks;
   this shape adds no frame or derive, so no new row/count is
   expected.

Then the five-mode LP01 parent (`logical_counter_port`) at 8 and 1
threads. STOP on any existing identity/status/disposition/exclusion
change, a vanished row, a new `UnknownHostPayload`, or a new row
pointing at code this brief did not touch. A red whose only cause is
a stale fingerprint/certificate/position is an ordering artifact:
replay it, keep the attempted log, report it as such. Never weaken a
test or edit an inventory to pass.

## Stage C — acceptance and disabled cost

After the last source edit and the replays:

- `cargo fmt --check`.
- `cargo check --all-targets` rc=0, no new warnings (the three
  standing lib-test warnings are the floor).
- The new runtime falsifier and the stdlib regression at 8 and 1;
  focused profiling tests at 8 and 1.
- `cargo test -p rum-runtime -j8` at 8 and 1 threads with the standing
  ignored set; the new tests are the only expected delta.
- the stdlib default and `--no-default-features` suites at 8 and 1 as
  the standing command; report counts.
- One `cargo test --workspace -j8 --no-fail-fast` at
  `RUST_TEST_THREADS=8`, with the standing ignored set (61) unchanged.
  Reproduce any genuine red at `--test-threads=1` before calling it a
  defect; report pre-existing reds separately.
- Disabled instruction gate on fresh base and candidate artifacts, two
  draws per arm, profiling off, no exclusions:
  `install`, `Stack::push`, `Segment::pause_admitted`,
  `Segment::end`, `Segment::end_admitted`,
  `finish_active_admitted`, `Segment::drop`, `ancestry_admitted`,
  `owner_admitted`, `executing_epoch_admitted`, `enter_admitted`,
  `running_admitted`, plus the generated `Vec<ActiveSlot>` Drop. The
  design measured eleven zero-entry rows, `running_admitted` at 40
  calls inlined into `Epoch::rows` (5,680 Ir in both arms and both
  draws; quote the containing function, never a fabricated standalone
  count, and prove the changed active branch is never entered), and
  the TLS destructor at 1 call / 24 Ir in both draws. Candidate1's
  24 -> 23 row is the known failed representation and must not
  reappear.
- Layout equality base vs candidate for the carriers above; a hot
  guest frame must not gain state.
- Microbench: if the diff changes any code executed with profiling off
  or any retained carrier layout, run the required rum-vs-rum
  `scripts/core-language-microbench.sh` A/B serially with profiling
  off and stop on a real regression; otherwise record NA with the
  instruction and layout receipts. The design's zero-entry rows do not
  by themselves authorize skipping a gate the diff actually requires.
- Artifact and scope audit: every change inside the permitted files
  and the regenerated inventories; no git, network, unsafe or
  historical-scene write; all failed attempts and build receipts
  retained; `python3 -I scripts/report.py check` on the report.

## Must not change

- The exact prepared boundary sequence: `prepared.rs` still completes
  with `Outcome::None` immediately after install/normalize, before
  later dispatch; no delayed sampling, fabricated result, reopened
  operation, borrowed epoch, replaced guard or blanket fault
  suppression.
- Every existing generation, owner/control, LIFO, active-slot,
  stale-epoch, cancellation, cap, source-stack and execution-control
  check and restore.
- Full-record comparisons, including open, abandoned, cancelled,
  Return and Throw fields. An invalid epoch never certifies parity.
- The disabled path: no new load, branch, allocation, lock or helper
  call. The enabled path adds exactly one charged receipt per
  installed segment; disclose it. Neither `Segment` nor `Active` gains
  a pointer-sized field.
- The 120 original differences stay unbound discovery evidence, not
  C1 losses. The malformed-answer probe stays flagged and
  unrelabelled. No assertion, witness or exclusion is weakened.

## Stop points and stop-loss

Stop on: a departure from the accepted shape; a requirement to modify
a forbidden file; an invalid epoch used as parity; a changed outcome
boundary; an uncharged diagnostic owner; a new disabled-cost row; an
inventory identity/status/disposition/exclusion change; a new
`UnknownHostPayload`; a genuine workspace red that reproduces at one
thread (report it, do not paper over it). The stop-loss is two
consecutive concrete attempts gaining none of [proved rule, closed
class row, calibrated falsifier, candidate record]; fixing compiler or
harness errors is recorded and is not a wall.

## Hard rules

- All scratch under `.temp/brief-727/`, never `/tmp`. Never build into
  the live `./target`.
- Build only through `scripts/probe-build.py`; Rust 1.85.0, offline
  and locked, `-j8`, one Cargo process at a time, bounded commands.
- Tests at `RUST_TEST_THREADS=8 --no-fail-fast`; timing, perf and
  Callgrind runs serial and alone.
- No git command in any form; no network; no `unsafe`; no writes to
  the frozen C7-C11/brief-623 scenes.
- Leave the live edits uncommitted for the director's gate. Do not
  edit `TODO.md`, `CLAUDE.md` or any codex brief.

## Report format

`rum-report/1` at `.temp/brief-727/report.json` (or one file per
stage, never overwriting): `summary`, `sections` (stage A proof, live
install, standing replays, acceptance, disabled cost, layout,
microbench/NA, attempts), `runs`, `measurements`, `changes`,
`disclosures`, `checks` (PASS/FLAGGED/FAIL with verbatim evidence
lines), `questions`, `stop` if the stop-loss fires. Every cited line
must appear verbatim in its log. Run
`python3 -I scripts/report.py check .temp/brief-727/report.json`
before finishing; the final message is the output of
`python3 scripts/report.py summary 727`.
