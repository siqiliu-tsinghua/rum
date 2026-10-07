# brief-595 — G11 step B: output a suspended capture owns, and the four G11 entries

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

brief-594 removed G11's three kernel panics. Step B repairs what remains of
brief-593's diagnosis and attaches the four owed oracle entries
(`EvaluationData.body`, `TestReport.input`, `TestReport.expectedOutput`,
`TestReport.sameTest`):

1. **`EvaluationData[expr, props]`** — `EvalEvaluationDataSuspensionFrame`
   (`runtime/debug_tools/evaluation_data.rs:30-36, 207-214`) omits the
   already-prepared selector, so the resumed path skips
   `project_evaluation_data_properties` (`:43-51`) and returns the whole
   capture association. Carry the selector; eager and resumed paths share one
   completion helper (finish the capture, then project once).
2. **Output a capture owns leaks at a pause.** Both pause paths in
   `session/session_requests.rs` (`:234-247`, `:352-367`) package every print,
   message and trace event after the *request* checkpoint (`:493-495`) — they
   cannot see that a still-running capture owns them. The capture then consumes
   them, so they vanish from the final cumulative output (`EvaluationData`,
   `TestReport` input), or a scope that suppresses messages has already leaked
   them (**`Quiet` surfaces a message at the pause** — `.temp/brief-593/quiet-message.txt`).

**The contract (director ruling).** Output inside an unsettled capture is
*provisional*: at a pause it stays in the kernel buffer, available to that
capture and nested observers, and is absent from the host response; the
capture settles it exactly once on completion (its existing consume, filter or
release policy) or rolls it back on abandonment. Output outside every active
capture is public at the next response, in order, per channel. Responses stay
cumulative; nothing already exposed disappears and nothing is exposed twice.
**The rule covers three channels — prints, messages and the raw trace array**
(`EvaluationData` enables and truncates tracing; ruling: trace joins the same
boundary). It lives in **one place**: a declarative output boundary that
suspension frames report (default: none) and one pause-response builder that
clips each channel at the earliest active boundary.

brief-593's report (§2, §3) and `.temp/brief-593/draft-B-capture-output-and-descriptors.md`
are the raw material; **this brief governs** where they differ.

## Rulings carried into this brief

- Failed `TestReport` cases keep their current copy-and-release policy
  (`test_report/evaluation.rs:252-273`) — tested and documented, not changed.
- `PrintSink` stays exclusive streaming (`docs/guide/embedding.md:337-348`);
  captures do not intercept streamed lines. Queued and resumed sink
  transcripts must be identical.
- `Check`, `Quiet` and `ConfirmQuiet` declare **messages** provisional while
  their body is unsettled (prints pass through); `EvaluationData` and
  `TestReport`'s input capture declare all three channels. `Reap`, the timing
  family, the constraints and `Monitor` own no buffered output.
- Q1 (one combined print/message stream) stays out of scope: per-channel order
  only, no response or serde shape change.

## Context pointers

- brief-593 and its report; `.temp/brief-593/{probe.rs,probe-test.log,quiet-message.txt}`;
  brief-594 and its report.
- `runtime/debug_tools/{evaluation_data.rs,properties.rs,test_report/evaluation.rs}`;
  `runtime/session/session_requests.rs:217-511`;
  `runtime/kernel_data/control_data/check_quiet.rs`,
  `confirmation/builtins.rs:436-472`, `confirmation/builtins/resume_frames.rs:91-120`;
  `runtime/suspension/core.rs:393-459, 713-717, 764-829`.
- The oracle: `support/{drive.rs:35-42,observe.rs:60-75,lifecycle.rs}` check
  print prefixes and final messages only — they need per-pause message and
  trace expectations.
- `docs/guide/{evaluation,embedding}.md`, `docs/testing-strategy.md`.

## Scope

**Objective:** the selector, the output boundary and pause builder, the
abandonment rollback through the same boundary, the strengthened oracle, the
four entries, and the standing and instruction gates.

**Edit surface:** `debug_tools/evaluation_data.rs`,
`debug_tools/test_report/evaluation.rs`; `runtime/suspension.rs`,
`suspension/core.rs` (capture-related abandonment only) and a new
`suspension/output.rs`; `session/session_requests.rs` (a distinct pause
builder; terminal behaviour unchanged); `control_data/check_quiet.rs`,
`confirmation/builtins/resume_frames.rs` (boundary declarations);
`runtime/tests/suspension_abandonment.rs` and one focused capture-output test
module; the oracle's `descriptors.rs`, `support/{drive,observe,lifecycle}.rs`
(observation gains the raw trace array and per-pause message and trace
expectations), `fixtures/` and `mutations/` (message- and trace-prefix
falsifiers); `inventory/{effects,owed}.tsv` — attach exactly the four G11
entries and delete exactly their four owed rows **only after all their gates
pass in both builds**, plus supplemental descriptors on existing entries;
`crates/rum-host/tests/` (one capture-output lifecycle module);
`docs/guide/{evaluation,embedding}.md` and `docs/testing-strategy.md` (state
the contract).

**Standing duty — always in scope** (`TEMPLATE.md`): fingerprints, both
certificates, ownership reconciliation, every row disclosed; a new frame or
derive type enters rum-runtime's frame lists. **STOP** on any existing row
change or disappearance, any new `UnknownHostPayload` or `self_reaching` row,
any new row in untouched code.

**Fence:** `push_print` and message emission unchanged — no per-emission
counter, ID, allocation or lock; raw buffers never split, cleared or reindexed
at a pause; `Expr`, response shapes, adapters, the lockfile unchanged; no other
owed entry; `scripts/` unchanged. No edits to `AGENTS.md`, `TODO.md`, root
`PROGRESS.md` or earlier briefs.

**Disclosure duty:** every changed function and type, every inventory row,
every measured label, any adjacent defect.

## Deliverable

The boundary query reads frame metadata only — it evaluates nothing and
mutates nothing — and runs only when packaging `InputRequired` (initial and
repeated pauses through the same helper) and on abandonment (inner to outer,
existing restoration helpers keep tracing, handled-error and dynamic-state
work; output rollback happens once through the common rule; never a `Check`
fallback or confirmation info callback). A pause is not a terminal failure:
capture buffers and handled-error bookkeeping survive it. Nested captures:
an inner completion cannot expose output an outer capture still owns.

## Tests

Red first, recorded before any production edit: `EvaluationData` scalar and
list selectors and captured print prefixes; `TestReport` input print prefix;
the `Quiet` message passing the old checker and failing the strengthened one.
Then each of the four entries with `Middle`, `Twice`, `Error`,
`AlreadyResumed`, `Nested` and its declared `Decline`, through `Checks::ALL`,
in both builds, lifecycle at the first and second pause. `TestReport` pauses
twice inside one input capture **and** in two distinct cases (and again for
expected output and `SameTest`, whose output belongs to the surrounding
request), with `OutputLog`, `ActualMessages` and aggregates compared — plus
wrong output, false `SameTest`, a handled failure, a terminal error, an empty
report. `EvaluationData`: no selector, `All`, `Properties`, scalar, list,
missing and invalid selectors, a selector whose preparation pauses; output
before, between and after two pauses inside, and outside before and after;
effects run once. `Check`/`Quiet`/`ConfirmQuiet`: messages before, between and
after two pauses, matching and non-matching, nested, fallback and info paths,
cancellation. A `PrintSink` control. Abandonment keeps the outside prefix and
discards the provisional capture. Falsifiers for the new message- and
trace-prefix checks, each watched to fail.

## rum / Rust pitfalls (read twice)

1. Never fix a prefix failure by relaxing the oracle or re-appending leaked
   output; keep the raw capture data and change visibility.
2. Cumulative responses repeat a prefix by design; never concatenate whole
   response arrays.
3. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: reds turned green, entries attached,
   falsifiers watched, regressing labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-595/tmp …`. Bounds: focused 300 s; the oracle 900 s
per build; runtime, ownership and retention 300 s; a one-unit release build
900 s; a metric process 150 s; `cargo test --workspace --no-fail-fast` 3 000 s.

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-host
scripts/check-suspension-oracle.sh 900
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

The oracle's `CATALOG` lines must show `owed` reduced by exactly four in both
builds. Report native ZeroMQ skips as skips.

**Instruction gate.** **Before any production edit**, build the base from the
repository tree with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` into
`.temp/brief-595/base/target-default` and `target-alloc` (`--features
rum-cli/alloc-count`, two arguments), binaries copied with hashes; candidates
the same way in fresh directories. Panel (scratch workloads, each result
checked first): ordinary arithmetic; no-output `Check`, `Quiet`,
`EvaluationData`, `TestReport`; buffered `Print` and `Message`; captured
`Print` and `Message` inside `EvaluationData` and `Check`; the controls
`dot-three-no-input`, `differences-no-input`, `take-multispec-small`
(`.temp/brief-563/perf/`). **Noise floor first** (base against itself, two
pairs); a label regresses when both candidate samples exceed `max(+0.05%,
floor)`. Allocation counts (timing-valued results allocate a varying amount —
compare against base run-to-run spread), then two paired samples of `taskset -c
2 perf stat -x, -e cpu_core/instructions/u`, one label per process, stdin
`/dev/null`, `(count(2N) - count(N)) / N`, raw output first. No timing suite.
At most eight measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-595/`; `PROGRESS.md` after the red capture, each
compiling candidate and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the oracle gate script, the release `rum-cli` on passing programs,
  `taskset`/`perf`, `valgrind --tool=callgrind`. No probe that aborts a
  process.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed files, functions and types; the boundary and where it lives.
2. Verbatim red and green output, the oracle in both builds, ownership,
   certificates and the broad suites; every inventory row change.
3. The four entries attached (owed before and after); the noise floors and
   every measured row.
4. Stop-loss counters, residuals, open questions.

---
