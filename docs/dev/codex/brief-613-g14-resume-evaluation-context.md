# brief-613 — G14: restore evaluation context and completion checks on resume

**Do not run `git` in any form**, including before you have read this brief.
This run forks the brief-612 design session. Its report (appended to
`brief-612-g14-resume-evaluation-stack-design.md`) is the design this brief
implements, with the director's rulings below. Its draft
(`.temp/brief-612/draft-G14.md`) is raw material; **this brief governs**.

## Goal

The queued-input path and the resumed path must reach the same logical
evaluation stack, depth and epoch at every continuation level. Each
successful evaluation level must also run its resource epilogue before its
result leaves that level.

The repair removes three G14 defects:
- false cycles after one pause;
- depth that grows by one per resume;
- `TimeConstrained`/`MemoryConstrained` bypasses that occur only at
  completion.

It does this through a private, queue-owned context and an inline
evaluation-return event. Frame implementations do not change, and the
ordinary path takes on no new work. G13's remaining 73 steps do not change
for it.

The repair is one atomic brief. A stack-only intermediate repair would leave
known constraint bypasses at a gate.

## Director rulings on brief-612

1. **The model is accepted as designed.**
   - `begin_input_suspension` saves the full snapshot. The drop-two
     heuristic and its rationale are deleted.
   - Every queue cell carries a mandatory `{stack_len, depth}` context,
     captured at insertion.
   - The driver restores the context, through accounted truncation, before
     it invokes any frame or event.
   - Records move unchanged through widening and splicing.
   - The global epoch is never rewound, and depth is never derived from the
     tracked length.
   - No frame type, `Session` field, `Expr`, slot or result carrier gains a
     field.
2. **Return boundaries (open question 2): approved as designed.**
   - An internal `evaluation_return => frame` form of the typed macro is
     backed by a crate-private
     `SlotWitness<Expr>::suspend_evaluation_return`, constrained to
     `TypedFrame<Input = Expr, Output = Expr>`.
   - It is used at exactly the two fixed-point sites (`resolution.rs:367`,
     `:434`). It inserts one inline `EvaluationReturn` event, then the
     fixed-point frame.
   - The existing macro forms expand byte-identically.
   - **Why the macro form, not S5's witnessed cold helper:** these two sites
     are the evaluator's hottest loop. S4 measured that passing the request
     and session into a non-inlined helper costs +160 to +288 B per level in
     release. The new arm must keep the existing lazy cold closure's
     captured environment.
   - A closed source-boundary test pins the two callers and the
     marker-before-consumer order.
3. **Native polling (open question 1): approved as designed.**
   - Save and restore the displaced base and threshold at the outer resume
     entry, and rebase to the live segment.
   - On each resumed frame entry, and each deferred `Evaluate` entry, poll
     actual headroom on the cold path and set the next threshold to the
     restored depth + 4.
   - This is physical-safety equivalence. It is not identical addresses or
     cadence, and the report must not claim it is.
4. **Cost (open question 3).**
   - The ordinary path tolerates zero regression. Allocations and bytes
     must be exactly equal on all 107 labels, and instruction slopes must
     sit within the noise bar.
   - Resume-path deltas are attributed per function and per cell, and I
     rule on them.
   - No percentage is pre-approved. A terminated base run is not a speed
     result.
5. **Resource witnesses (open question 4): both approved,** as test-only
   native builtins registered by the test, never production builtins.
   - The memory witness is the primary deterministic red test.
   - The time witness must sleep at least 1.5× its limit, so the queued arm
     reaches 99 deterministically.
   - Both witnesses assert one actual pause in the resumed arm.

## Context pointers

- The brief-612 report: three traces, the model, the route inventory, the
  accounting, the epilogue, re-suspension and the coverage table. Also
  `.temp/brief-612/{probe.rs,matrix.tsv,epilogue.txt,memory-epilogue.txt}`.
- Owners, relative to `crates/rum-runtime/src/runtime/`:
  - `suspension/core.rs:492-828`, `suspension/core/{queue,abandon,fault}.rs`
    and `suspension/typed/{entry,witness,resume_token,chain}.rs`;
  - `resolution.rs:360-445`;
  - `eval_core/mod.rs:15-96`, read-only;
  - `metadata.rs:17`, the poll stride;
  - `debug_tools/resources/{maintained,memory}.rs` and
    `debug_tools/resources/occurrence/suspension.rs`;
  - `accounting_mode/counter.rs`;
  - `session/guest_loading/scope.rs`.
- The E1 oracle (`crates/rum-stdlib/tests/suspension_oracle/`). The
  retention analyzer's typed-macro adapter
  (`crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs`)
  models the macro's arms exactly and has a drift falsifier. A new arm
  **will** trip it.
- `TEMPLATE.md`, `AGENTS.md`, and `docs/guide/{evaluation,resource-limits-and-sandbox}.md`.

## Scope

A listed file or class is permission to do this brief's owned work there.
It is not permission to tidy, retype frames, or change evaluator, matcher or
builtin semantics. Disclose every changed file.

**Source.** Paths are relative to `crates/rum-runtime/src/runtime/`:

1. `suspension/core.rs`, `suspension/core/{queue,abandon,fault}.rs`, and a
   new sibling `suspension/core/evaluation_context.rs` for the new
   responsibility.
2. `suspension/typed/{entry,witness}.rs`: the private envelope traversal,
   the private helper and the internal macro arm only. Public types, the
   existing arms, witnesses, tokens and frame contracts are unchanged.
   `resume_token.rs` and `chain.rs` change only where the route inventory
   requires a captured context. Disclose each change.
3. `resolution.rs`: the two `InputRequired` construction branches only.
4. `debug_tools/resources/{maintained,memory}.rs` and
   `debug_tools/resources/occurrence/suspension.rs`: cold truncation, queue
   storage and the independent model only.
5. `session/guest_loading/scope.rs`: retire its top-two comment only. A
   behavioural change there is a STOP with its counterexample.

**Tests.**

- **Runtime.** A new `runtime/tests/resume_evaluation_context.rs` and its
  module declaration; `tests/session_memory_maintained.rs`; and
  `suspension/typed/tests/*`. Test-only observers add nothing to release
  evaluation.
- **Oracle.** A new
  `crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_stack.rs`,
  its registration in `fixtures/mod.rs`, and a mutation helper beside it if
  needed. Register descriptors only against real existing roles.
- **Compile controls.** `crates/rum-stdlib/tests/typed_slot_compile_controls.rs`
  gains positive and negative pairs for the helper's
  `Input = Expr, Output = Expr` constraint.
- **The typed-macro adapter.**
  `crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs`, and
  `retention_closure/compiler_oracle/corpus.rs` if it pins the macro text,
  model the new arm exactly as they model the other two. That means a
  checked expansion list and the existing drift falsifier, which must still
  fail on a drifted arm. Watch it fail once against the new arm before
  updating it.

**Pins that follow code.** A test or inventory row that names a touched
file's path, line, function or count may be updated only to follow the same
construct, disclosed as old → new. This covers:
- `runtime/tests/frame_root_trace{.rs,/derived_inventory.rs}`;
- ownership inventories, by discovery replay only;
- retention inventories (`obligations`, `local-contracts`, `promoted`), by
  replay only;
- census pins, if the macro-use facts move;
- G4/G7 contract tests (`association_depth_contract/`), for a new callee on
  an audited path, each with a one-line audit.

**STOP conditions.** A new `UnknownHostPayload`, an unpaired removal, or a
changed identity, status, disposition or exclusion is a STOP. Do not
hand-write fingerprints or certificate cells.

**Read-only:** `eval_core/`, every frame implementation, `Expr`/`Symbol`,
public continuation shapes, `Cargo.toml`, `Cargo.lock`, the toolchain,
`AGENTS.md`, `TODO.md`, `PROGRESS.md` and other briefs.

**Disclosure duty.** If a needed change falls outside every list and class,
make no edit there. STOP and report the file, the line and why.

Scratch goes under `.temp/brief-613/`. The compile controls may write
`.temp/typed-slot-compile-controls/`, and the certificate replay may write
`.temp/brief-546/certificates/`.

## Deliverable

Follow the draft's deliverable items 1-9 (`.temp/brief-612/draft-G14.md`,
§Deliverable), with these rulings applied:

1. **Red first.** Port all 34 brief-612 seeds through `Fixture` and
   `accept(Checks::ALL)` with correct queued outcomes, before any
   production edit.
   - Record the full red ledger, in which every descriptor is attempted.
   - Brief-612 predicts 25 unequal pairs. Rerun and record the actual
     number rather than citing the prediction.
   - Then add the remaining dimensions and invariants, each with its own
     red baseline.
2. **Implement the model** as ruled above.
   - The two return events are inserted at the two sites.
   - At an event, the driver restores the parent context, then runs the
     resource epilogue on a successful `Expr` only.
   - Errors propagate without the epilogue. A wrong successful slot is a
     structural driver fault.
   - Then apply the native policy, and accounting through an accounted
     truncation that is checked by `assert_maintained_memory_accounting`
     against its independent occurrence walker.
3. **Class mutation falsifiers.** Each mutation must turn its assigned
   descriptors red, while the no-input control stays green:
   - restore drop-two;
   - omit context restoration;
   - derive depth from length;
   - lose a record in widen/append;
   - rewind the epoch;
   - skip the return event's check.

   Final production contains none of the mutants.
4. **Long runs.** 1 100 and 2 000 changing-expression resumes at the
   default recursion limit, plus 100 resumes at limit 32. Both arms
   complete with equal results. Never raise the limit to pass.

## Verification (MANDATORY)

**Run rules.**
- Commands are serial, offline, with one job and one test thread.
- Run each command through `python3 scripts/run-bounded.py`. Bounds: 900 s
  for focused and build commands, 1 500 s for each oracle build, and
  3 600 s for the workspace.
- Your tool timeout must exceed every bound you pass.
- Raw output goes to `.temp/brief-613/` before you parse it.

**Correctness.**
- Run the draft's final list, plus `scripts/check-suspension-oracle.sh 1500`.
- Both oracle builds must stay warning-free with fault-occurrence delta 0.
- Report every count.
- Native ZeroMQ skips are reported as skips.

**Performance base.** It is supplied, not rebuilt from git. `.temp/brief-613/`
holds:
- `base/` — the S5 one-codegen-unit binaries `rum-cli`, `rum-cli-alloc`,
  `resume` and `resume-alloc`. The director proved them fresh against S5
  HEAD, and their build commands are alongside.
- `base-source/` — the S5 source export (`21a68525`).
- `stack-base/` — the S5 stack-probe test binaries, with `probe.rs`.
- `base-hashes.txt` — verify against it first.

Build the candidates with the same commands and the same cwd, into separate
target directories. The resume harness is `.temp/brief-611/resume-harness/`;
copy it under `.temp/brief-613/` before building. The label panel is
`.temp/brief-611/r3/perf/labels.json` and its `.rum` files. Preflight the
result and the actual resume count of every N/2N run.

**Performance order.**
1. **Allocations.** Exact equality on all 107 ordinary labels. Every resume
   delta is attributed to context cells, return-event capacity and the full
   snapshot. No `Arc` per event.
2. **Instructions.** Two interleaved pairs per arm, `taskset -c 2 perf stat
   -x, -e cpu_core/instructions/u`, stdin `/dev/null`, slope
   `(I(2N) - I(N)) / N`.
   - A label is flagged when its delta exceeds
     `max(0.05% of base, base range, candidate range)`.
   - A flagged ordinary label gets five runs per arm. A flag that persists
     gets six callgrind processes per arm, with every deterministic delta
     attributed by address.
   - Resume rows: report the delta per actual resume and its attribution.
3. **`memcpy`.** Ordinary counts are not above base, and the three leaves
   show no deterministic growth.
4. **Stack.** Use the S5 probe (`stack-base/probe.rs`, appended to
   `runtime/tests/holding_and_control_part2.rs` in a scratch tree copy, never
   in the repository). Measure peel, plus, sub, up and guarded, in debug and
   release, at four heights × three samples each.
   - The bar: each pair is at or below S5, or its growth is attributed per
     slot to the named closure environment.
   - Report the full table and the changed resident prologues.
5. **Timing** comes last and is an indication only:
   `scripts/core-language-microbench.sh` base/candidate, profiling off.

**Stop-loss.** At most six compiled semantic repair attempts. After each
one, record the failing G14 descriptor and invariant IDs and any regression
in the existing suite.
- STOP after two consecutive compiled attempts that reduce none of those
  counts.
- Before the tree compiles, falling compile errors count as progress.
- Once correctness is green, two consecutive attempts that reduce neither
  the flagged ordinary rows nor the unattributed resume deltas also trigger
  a STOP.

A stack or performance residual is not a failing gate. It goes to the report
with its attribution, and I rule on it.

## Checkpointing

Write `.temp/brief-613/PROGRESS.md` at each of these points:
- after the red baseline;
- after each compiled attempt;
- at correctness closure;
- after allocations, instructions and stack.

Each checkpoint records the next action, the failure-count history, the
edits and the evidence paths. Leave the source uncommitted.

## Hard rules

- **No `git`, in any form.** No network, and no scratch outside
  `.temp/brief-613/`, except the two pre-authorized paths above.
- **Stay inside Scope.** STOP with evidence rather than inferring an
  exception.
- **The Testing Policy is a hard line:** rum against rum only.
- **No new `unsafe`, and no aborting probes.**
- **No default-path cost:** no field, branch, store, counter or allocation.
- **Do not weaken tests to pass.** Do not raise the recursion limit, remove
  a failing label, or refresh a ledger by hand.

## Report format

Append `## brief-613 execution report` to this file:
1. Files and lines changed and why, with every inventory row disclosed
   old → new.
2. The red baseline, the failure-count progression, the exact final test
   output, and both fault-occurrence deltas.
3. Before and after contexts for the three traced witnesses; the route
   controls; the resource, fault and lifecycle outcomes; and 2 000 resumes.
4. Allocation and instruction tables and their attribution; the stack table
   and slot attribution; copies; timing.
5. G13 transparency, the limits of the native policy, open costs, and the
   stop-loss ledger.

## brief-613 execution report

**STOP: ownership construct identity changed. This is an incomplete execution,
not a green gate.** No inventory was refreshed. No further implementation or
gate execution was performed after identifying the STOP condition. The source
remains uncommitted; no git command, network operation, sub-agent, unsafe code,
or aborting probe was used.

### 1. STOP evidence and changed files

Fresh discovery (`.temp/brief-613/ownership-discovery.txt`) reports 79
UnknownHostPayload rows, 7,709 UnknownConstruct rows and 59 self-reaching rows.
After ignoring source coordinates and the implementation-block line embedded
in function names, exactly one host-payload identity is removed and one added.
Both point to `crates/rum-runtime/src/runtime/suspension/core/queue.rs:6:12`:

```text
old construct:
rum_runtime::runtime::suspension::core::queue::FrameQueue.Legacy.0/std::collections::VecDeque[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame

new discovered construct:
rum_runtime::runtime::suspension::core::queue::FrameQueue.Legacy.0/std::collections::VecDeque[Own]/arg0/rum_runtime::runtime::suspension::core::evaluation_context::Queued<arg0> via payload/std::sync::Arc[LastStrongHolder]/arg0/alias(rum_runtime::runtime::suspension::core::DynSuspensionFrame): opaque owned payload dyn SuspensionFrame
```

The old row is `ownership_graph/inventory/host_payloads.tsv:57`, with its
disposition row at `host_payload_ledger.tsv:53`. The source envelope is
`suspension/core/evaluation_context.rs:10`. This is a change in the recorded
construct's ownership path, beyond a file/line/count pin. The brief says:
“A new `UnknownHostPayload`, an unpaired removal, or a changed identity,
status, disposition or exclusion is a STOP.” The total remaining 79 does not
authorize replacing that identity. The exact old/new rows and normalization
are in `.temp/brief-613/ownership-stop.json`. No disposition, witness,
fingerprint, certificate cell or exclusion was changed to accommodate it.
The 16 existing ownership/retention inventory files are byte-identical to the
initial snapshot (`inventories-unchanged.json`). Other inventory differences
have not been replayed or fully classified after this STOP.

There are 22 changed source/test files, listed below with hunk coordinates.
`owned-changes.json` contains their initial/candidate hashes and sizes;
`owned-changes.patch` contains all old/new hunks. The initial snapshot is in
`.temp/brief-613/initial-source/`. This brief additionally receives this report.

| File | Candidate changed lines | Reason |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs` | 205-222 | Cold accounted truncation removes each occurrence while retaining vector capacity. |
| `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs` | 11-19, 27-33 | Independent queue-storage calculation includes both context words and zero-allocation return events. |
| `crates/rum-runtime/src/runtime/resolution.rs` | 376-382, 440-446 | Select evaluation_return only at the two InputRequired fixed-point construction branches. |
| `crates/rum-runtime/src/runtime/session/guest_loading/scope.rs` | 178-184 | Remove the obsolete drop-two comment; behavior unchanged. |
| `crates/rum-runtime/src/runtime/suspension/core/abandon.rs` | 38-50 | Restore each queued context before abandonment; do not derive depth from stack length. |
| `crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs` | 1-128 | New private context/envelope, restore, native poll and return-epilogue responsibility; cfg(test) observations. |
| `crates/rum-runtime/src/runtime/suspension/core/queue.rs` | 1-10, 15-24, 28-99, 101-128, 146-153, 156-170, 179-190, 204-210 | Mandatory context in every Legacy/Mixed cell; preserve it through widening and append; inline return event. |
| `crates/rum-runtime/src/runtime/suspension/core.rs` | 13-25, 169-190, 196-202, 492-500, 521-532, 556-567, 610-617, 636-643, 664-671, 701-745, 764-790, 811-829 | Full snapshot, context capture at insertion, driver restoration, event handling and displaced native-state preservation. |
| `crates/rum-runtime/src/runtime/suspension/typed/entry.rs` | 9-21, 23-29, 34-40, 43-49, 53-59, 61-67, 73-79 | Inline EvaluationReturn traversal, no retained roots or allocation, discard during abandonment. |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs` | 59-76, 91-99 | Exact cell-layout/count pins for the larger queue cells; adapter allocation unchanged. |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/evaluation_context.rs` | 1-256 | Route controls for populated widening, chain, tokens, nested append, both Evaluate arms, Finalize and fault/error behavior. |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs` | 383-388 | Declare the new route-control test module. |
| `crates/rum-runtime/src/runtime/suspension/typed/witness.rs` | 28-71 | Crate-private Expr-to-Expr helper and new internal macro arm; original arms byte-identical. |
| `crates/rum-runtime/src/runtime/tests/mod.rs` | 192-196 | Declare the new runtime test module. |
| `crates/rum-runtime/src/runtime/tests/resume_evaluation_context.rs` | 1-420 | Snapshot/epoch/depth/native/accounting/lifecycle invariants, three traces and closed caller boundary. |
| `crates/rum-runtime/src/runtime/tests/session_memory_structures.rs` | 249-258 | Count/layout pins follow the same saved-stack and queue-storage constructs under the explicit pins class. |
| `crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs` | 371-388, 513-552 | New-arm four-view positive control and drift mutations for the new and existing identifier arms. |
| `crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs` | 54-80, 134-145, 152-161, 178-195, 205-211 | Exact checked expansion for the third arm, including its resource-event/helper operations. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_stack.rs` | 1-477 | 34 original seeds plus 37 dimensions, complete attempted ledger, strict acceptance and mutation panel. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_stack_support.rs` | 1-131 | Test-only resource/widening witnesses and raw paired-observation support using existing InputString registration. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs` | 16-22 | Register the G14 fixture module. |
| `crates/rum-stdlib/tests/typed_slot_compile_controls.rs` | 230-261, 327-332 | Two new positive/negative pairs for helper Input=Expr and Output=Expr; expected pairs 32 to 34. |

Full old/new hunks and byte hashes are in `owned-changes.patch` and `owned-changes.json`.

Count/layout pin disclosures, also in `pin-disclosures.md`:

- `runtime/tests/session_memory_structures.rs:252`: saved roots 1 → 3;
  one entry shell/root occurrence → three of each. Legacy queue cell
  `Arc<dyn SuspensionFrame>` → `(usize, usize, Arc<dyn SuspensionFrame>)`.
  This uses the brief's explicit count-pin class, rather than its named
  runtime-test file list. No assertion was removed.
- `suspension/typed/tests/accounting.rs`: both queue-cell layouts gain two
  `usize` fields. The measured insertion charge is 136 → 200 bytes for four
  cells; the adapter allocation remains five `usize` words.
- `typed_slot_compile_controls.rs`: expected paired controls 32 → 34.
  The two new pairs constrain the helper's input and output independently.
- No inventory row has an applied old → new update. The proposed ownership
  replacement above is explicitly unapplied and is the STOP evidence.

`suspension/core.rs` grows from 29,879 to 31,587 bytes, still in the split-plan
range; context/restore/return/native-policy logic is isolated in the new
4,647-byte `core/evaluation_context.rs`. The compile-control harness grows
from 22,610 to 23,890 bytes; its existing paired-build responsibility stays
together. No unrelated module split was attempted.

### 2. Red baseline, progression and verification actually run

All 34 original descriptors were attempted through `Fixture` and
`accept(Checks::ALL)` before production edits: **25 failed, 9 passed**, with
25 unequal raw pairs (`red-seeds.txt`). The first 32 added dimensions had
25 failures/7 passes (`red-dimensions.txt`). All six initial runtime
invariants failed (`red-invariants-2.txt`). Their first compilation exposed
test-import mistakes, corrected before recording that six-failure baseline.

The expanded final population was also replayed against unchanged S5
production in an isolated copy. `red-final-population.txt` records **34 seeds:
25 failures; 37 dimensions: 27 failures**, hence 52/71 rejected descriptors.
Every descriptor was attempted, even after an earlier rejection. The strict
ledger is accompanied by raw queued/resumed observations and actual pauses.
The added widening and post-child-return epoch controls pass S5 but reject
their targeted metadata/epoch mutations.

Only **one compiled semantic repair attempt** was made. It changed the
original 66 raw pairs from 50 unequal to zero unequal and all six initial
invariants from failing to passing. One strict fixture expectation initially
used `HoldForm` instead of the observed/documented queued `Hold`; correcting
that test typo made the 34-seed strict ledger pass. The first existing runtime
suite had two failures caused by the now-larger snapshot/cell layout; the
disclosed exact count pins were corrected. Subsequent changes added controls
or corrected test harnesses, not another production semantic repair.

Latest focused default-feature G14 run (`attempt-1-oracle-epoch.txt`), exact
summary output:

```text
continuation-slot fault-occurrence delta=0; observations=666
G14 TOTAL attempted=37 failed=0 passed=37
continuation-slot fault-occurrence delta=0; observations=790
continuation-slot fault-occurrence delta=0; observations=1402
G14 TOTAL attempted=34 failed=0 passed=34
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 133 filtered out; finished in 1.89s
```

These three tests are the dimensions, mutation-panel control run, and seeds.
All 71 strict descriptors pass. They are not the full two-feature oracle gate.

The full runtime run in `runtime-validation.txt` reported:

```text
test result: ok. 1184 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 29.91s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.46s
```

The second line is rustdoc. The three ignored tests are the release-only
evaluation-cache profile and two release-only left-chain deployment/drop
probes. The three dead-code warnings in read-only `test_builtins.rs` were
already present in the red runtime build. The runtime run includes 11 new
context/lifecycle tests and three new typed-route controls. Two test-only
assertions were subsequently strengthened (recursive closed-caller scan and
exact first-segment return-level comparison); those latest assertions have
not been rerun. There was no subsequent production semantic change.

The required macro-adapter red was witnessed (`macro-adapter-red.txt`):

```text
test result: FAILED. 1 passed; 5 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 87.85s
```

After adding the checked arm, a second run caught rustfmt's braces around the
new lazy closure expression (`macro-adapter-validation.txt`):

```text
test result: FAILED. 1 passed; 6 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 101.40s
```

The expected token expansion was corrected to include those braces and
compiled during ownership discovery, but the adapter tests have not been
rerun. No green adapter result is claimed. The two pre-existing macro arms
are byte-identical to the initial snapshot (`macro-old-arms.json`).

Fresh ownership discovery itself completed normally; it emits facts and does
not certify inventory equality:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 5.71s
```

Final required verification status at STOP:

| Required command/group | Evidence/status |
| --- | --- |
| `cargo fmt --check` | Not run; owned Rust files were formatted directly. |
| Runtime context filter | Initial six pass; expanded controls pass in the full runtime run; latest two strengthened assertions unverified. |
| Separate maintained-memory, typed-test and root-trace filters | Not run as separate final commands; existing tests were included in the runtime run. |
| `cargo test -p rum-runtime` | 1,184 unit tests and 22 rustdoc tests pass, with the later test-only caveat above. |
| Compile controls, both feature configurations | Added, not run. |
| Census, both feature configurations | Not run. |
| `scripts/check-suspension-oracle.sh 1500` | Not run. Full default-feature and no-default-feature fault deltas are both **unmeasured**. |
| Full ownership-graph suite | Discovery only; identity STOP prevents replay/closure. |
| Local production acceptance, both configurations | Not run. |
| Complete production-rejection census | Not run. |
| Workspace `--no-fail-fast` | Not run. No native ZeroMQ run or skip was observed. |

Observed commands were serial, offline, one build job/test thread, bounded
through `scripts/run-bounded.py`; raw logs and targets are under brief-613.
An identical verification copy was prepared at `validation-source/`, with
`validation-manifest.json`, because canonical oracle/census tools hard-code
historical scratch paths. Running them there would keep their unchanged
behavior inside the permitted scratch tree. Those gates had not started
when discovery triggered the STOP.

### 3. Context, routes, resources, lifecycle and mutations

The before states below come from the accepted brief-612 traces and the S5
red replay. Candidate observations are in
`attempt-1-expanded-invariants-2.txt`; the trace test later passed in the full
runtime run. Let T be Table, F the definition call, C its input compound, and
I InputString. Entries retain their original epochs.

| Witness | S5 resumed context | Candidate context and result |
| --- | --- | --- |
| False cycle, `Table[f[1],{3}]` | Snapshot `[T,F]`, installed depth 2; F survives the Table handoff and the second call is rejected. | Full `[T,F,C,I] / 4`, epoch 255. Return levels `[T,F,C]/3`, `[T,F]/2`, `[T]/1`; every next input is again the full four-entry context. Three pauses, 50 observed events, result `List[7,7,7]`. |
| Changing-expression depth leak | First snapshot `[F]/1`; old F roots accumulate. S5 stops after 1,021 pauses at the default limit. | Every input has `[F,C,I]/5`; return levels `[F,C]/4`, `[F]/3`, `[]/2`, `[]/1`. Entry/global epochs advance 256, 257, … with the assignment. All 1,100 pauses complete at fixed depth 5, with 23,105 events. |
| Passing direct Table | Snapshot `[]/0`, incorrectly loses T though the value happens to pass. | `[T,I]/2` at each input, `[T]/1` at Table handoff, `[]/0` at completion. Three-pause trace has 23 events; the strict 1,100-pause direct control also passes. |

The final strict population additionally verifies **2,000 actual resumed
pauses** at the unchanged default recursion limit and 100 at limit 32, with
equal queued results. S5 reaches only 1,021 of the 2,000 planned pauses. No
terminated baseline is treated as a performance result.

Route controls cover Legacy and Mixed, populated widening, witnessed insert,
fresh token, FrameChain, both deferred Evaluate arms, nested append, Finalize,
ordinary errors and structural wrong-slot failure. Existing chain/token
production files are unchanged: their common insertion endpoints capture the
context. Context is restored before frame dispatch and abandonment. The
return event restores its parent level, checks successful Expr results only,
forwards errors without the success epilogue and reports a structural driver
fault for a wrong successful slot.

The deterministic memory witness grows retained memory beyond its 5,000,000
byte limit; the time witness sleeps 150 ms against 100 ms. Both now produce
99, matching queued evaluation, and both assert one actual resumed pause.
Their S5 resumed result is 77. Test-only registration uses existing builtin
roles; no production builtin was added.

Independent maintained-memory checks cover per-occurrence truncation, shared
roots, retained vector capacity, arming/rearming, a second pause, completion,
error/fault cleanup and cancellation. Shadow counts remain balanced. Existing
guest-file prefix-restore tests pass in the runtime run; that production file
has only the obsolete comment removed. These are semantic accounting checks,
not the unperformed performance allocation panel.

All seven class-mutant runs rejected their assigned strict descriptors and
kept the separate genuinely input-free control green:

| Mutation | Assigned rejecting descriptors |
| --- | --- |
| Restore drop-two | `g14.stack-pass`, `g14.false-cycle-table` |
| Omit restoration | `g14.false-cycle-table`, `g14.stack-pass` |
| Depth from tracked length | `g14.untracked-depth-pressure` |
| Lose widening record | `g14.legacy-widen` |
| Lose append record | `g14.nested-append-depth-pressure` |
| Rewind global epoch | `g14.epoch-after-child-return` |
| Skip return success check | `g14.memory-epilogue`, `g14.time-epilogue` |

`mutation-summary.json` gives exact commands, expected failing outputs and
restoration hashes. All seven production-restoration hashes were checked again
at STOP and match. Initial harness weaknesses are preserved, not hidden:
the first “no-input” control used a sentinel that itself requested input;
the first widening/epoch assignments did not exercise the intended boundary.
The control was made actually input-free, and new boundary-specific fixtures
were added without removing the old strict descriptors. The initial mutant
logs remain under `mutants/*-initial.*`.

### 4. Performance, stack, copies and timing

All six supplied binaries match `base-hashes.txt` (`hash-verification.json`).
No candidate performance binaries or measurements were produced before the
ownership STOP. Consequently no ordinary-path zero-regression claim is made.

| Required evidence | Result at STOP |
| --- | --- |
| 107 ordinary allocation/byte N/2N rows | Unmeasured. |
| Resume allocation/capacity attribution | Unmeasured performance panel; structural unit accounting only. |
| Interleaved instruction slopes, flagged-row follow-ups and address attribution | Unmeasured. |
| Ordinary memcpy counts and three deterministic leaves | Unmeasured. |
| Debug/release S5 probe, five families × four heights × three samples | Unmeasured; no candidate stack probe built. |
| Changed resident prologues and closure-slot attribution | Unmeasured. |
| Final monotonic microbenchmark A/B | Not run. |

The source adds two context words per queue cell, inline return events and the
full saved snapshot, with no Arc per event. The unit-accounting observation
of +64 bytes for four cells is not a substitute for the mandatory allocation
panel. Instruction, native-stack and timing costs remain open for the director.

### 5. G13, native-policy limits and stop-loss ledger

G13's remaining S6–S54/F1–F23/R frame work is unchanged. No production frame
implementation, public continuation shape, Session field, Expr/slot/result
carrier, manifest, lockfile, toolchain, eval_core implementation, or unrelated
roadmap document changed. No extra ordinary-path counter, branch, record
capture or field was introduced by source intent; the missing performance
evidence prevents claiming zero generated-code cost.

Native resume saves/restores the displaced base and threshold, rebases to the
live segment, polls before each resumed callable frame and deferred Evaluate,
and sets the next threshold to restored logical depth + 4. A zero-budget
control returns StackLimit through cleanup without an overflowing probe.
This is the approved **physical-safety equivalence**, not identical native
addresses or polling cadence. Native resident-stack measurements remain open.

The semantic stop-loss history is one compiled attempt: 50/66 unequal
descriptors plus six failing initial invariants → zero unequal pairs/zero
initial invariant failures. The strict expectation typo and two layout pins
were separately corrected. The final expanded population is 71/71 passing;
no second production semantic attempt occurred. Mutation builds are negative
controls, not repair attempts. No consecutive non-improving-attempt or budget
threshold was reached. The actual stop is the explicit ownership-identity
boundary above. Final correctness closure, inventory replay and all required
performance gates remain outstanding pending a director ruling on that path.

## Director amendment — redispatch 1: the queue-envelope transition, then finish the step

The STOP was correct. The rule named a changed identity, and you neither
widened it nor wrote the row by hand. The report is also honest about what
did not run.

### Accepted as reported (I re-verify all of it at my gate)

- The red ledgers: 25 of 34 seeds and 27 of 37 dimensions on S5, every
  descriptor attempted, plus the six red invariants.
- One compiled semantic attempt, leading to 71/71 strict descriptors.
- The seven class mutants, each rejecting its assigned descriptors while the
  input-free control stays green.
- 2 000 resumes at the default limit, and 100 at limit 32.
- The memory and time witnesses now give 99 on both paths.
- The design as written. I read the diff:
  - the `Queued<P>` envelope on every cell, preserved through widening and
    append;
  - the full snapshot;
  - the return event inserted before the fixed-point frame;
  - context restored before dispatch and before abandonment;
  - native base and threshold saved and restored;
  - the third macro arm, whose `pub(crate)` helper keeps it unusable outside
    the crate.

### A. Ruling: the queue-envelope transition is approved

```text
old  FrameQueue.Legacy.0/VecDeque[Own]/arg0/Arc[LastStrongHolder]/arg0/alias(DynSuspensionFrame)
new  FrameQueue.Legacy.0/VecDeque[Own]/arg0/Queued<arg0> via payload/Arc[LastStrongHolder]/arg0/alias(DynSuspensionFrame)
```

Several things are unchanged:
- the terminal payload, `dyn SuspensionFrame`;
- the owner, the `VecDeque` cell;
- the `Arc[LastStrongHolder]` edge.

The added hop is a plain struct field: no `Drop`, no sharing and no second
owner. This is the same kind of transition as S2's approved
`SuspendedEvaluation.frames` → `FrameQueue.Legacy.0`.

Carry the ledger row's four cells over unchanged: disposition `Unbounded`,
the witness, `owed_by` `TODO.md#standing-engineering-debt`, and external
coverage `unresolved`. Apply the transition by discovery replay, so the
construct cell takes the discovered text. Disclose it old → new in both
`host_payloads.tsv` and `host_payload_ledger.tsv`.

This approval covers exactly this one transition. Every other STOP condition
stands. If the replay shows any other new, removed or changed identity,
status, disposition or exclusion, including a `FrameQueue.Mixed` or `Entry`
row, it is still a STOP.

### B. Pin rulings

- The `session_memory_structures.rs` pins are accepted under the
  pins-that-follow-code class. Saved roots 1 → 3 is the full snapshot the
  design rules.
- The tuple `(usize, usize, Arc<…>)` stands in for the private `Queued<…>`
  in two tests. This is accepted, because a size mismatch fails loudly.
  Disclose it as a stand-in.

### C. Run the canonical gates in the repository

Two unchanged canonical tools write to fixed scratch paths:
`scripts/check-suspension-oracle.sh` writes to `.temp/suspension-oracle/`,
and `typed_slot_census` writes to `.temp/brief-611/r3/census/`. Both paths
are authorized, for those tools only.

Run every gate in the repository tree, not in `validation-source/`, which
you may delete. My gate runs them in the repository too. A copy only doubles
the build.

### D. Attribute the widening; do not redesign it

`push_evaluation_return` widens a `Legacy` owner to `Mixed`. Return events
are inserted at the fixed-point sites, so nearly every real suspension now
widens. The S2 legacy-only cells survive only for owners that never cross a
return site.

This is a resume-path cost, and it is acceptable in principle, since
`Legacy` goes away at R. The resume attribution must name it. For each
workload, report:
- how many owners widen;
- the widening move (cells × cell size);
- the cell sizes, `Queued<Entry>` against `Queued<Arc<…>>`.

I rule on the numbers.

### E. Then finish the step, on the final source

1. **What did not run, run first:**
   - the two strengthened runtime assertions;
   - the macro adapter, both views, watched green after its red.
2. **Every correctness gate:**
   - fmt, the context, maintained-memory, typed and root-trace filters, and
     `cargo test -p rum-runtime`;
   - the 48-level test;
   - the compile controls, the census and local production, each in both
     builds;
   - the full ownership suite and the production rejection census;
   - `scripts/check-suspension-oracle.sh 1500`: both builds warning-free,
     both fault-occurrence deltas reported;
   - the workspace with `--no-fail-fast`, reporting native ZeroMQ skips as
     skips.

   Replay the inventories, certificates and fingerprints, and disclose every
   changed row.
3. **Every performance section, in the brief's order**, against the frozen
   S5 bases in `.temp/brief-613/` (hashes already verified, so verify them
   again only if a file changed):
   - allocations;
   - instructions;
   - `memcpy` and the three leaves;
   - the stack table, with attribution per slot;
   - timing, last.

The stop-loss continues from your ledger: one compiled semantic attempt is
used.

### Process

- This run forks your session, so your scratch and state are where you left
  them in `.temp/brief-613/`.
- Your tool timeout must exceed every `--timeout` you pass.
- Checkpoint as before. Then append `## Execution report — redispatch 1`
  after this section.

## Execution report — redispatch 1

**STOP: additional fixed scratch paths conflict with the Hard rules.** The
approved queue-envelope transition has been replayed and its ownership gate
passes. Execution is still incomplete. This redispatch made no production or
test-source repair; all 22 previously changed Rust files retain their prior
hashes (`.temp/brief-613/redispatch-source-check.json`). The semantic attempt
count remains **1 / 6**. No git, network, unsafe, sub-agent or aborting probe
was used.

### 1. New STOP evidence

The required `production_rejection_census_is_complete` calls `report::write`,
which unconditionally writes under `source::workspace().join("target/brief-535")`
at `crates/rum-stdlib/src/tests/retention_closure/census/report.rs:65`.
`source::workspace()` is the repository root, derived from
`CARGO_MANIFEST_DIR` (`retention_closure/source.rs:14`). It does not consult
`CARGO_TARGET_DIR` or `TMPDIR`. The mandatory workspace gate includes further
fixed scratch writers:

| Fixed output path | Writer and source line | Required gate that reaches it |
| --- | --- | --- |
| `target/brief-535/` | `retention_closure/census/report.rs:65`; compiler-oracle `mod.rs:30`, `corpus.rs:15`, `exact.rs:31`, `methods.rs:11` | Production rejection census and workspace compiler-oracle tests |
| `target/brief-540/oracle/` | `retention_closure/local/oracle/mod.rs:143` | Workspace `local_probe_claims_match_stable_rustc` |
| `.temp/brief-542/falsifiers/` | `retention_closure/local/reads/tests.rs:23` | Workspace local read-contract falsifiers |
| `.temp/brief-543/oracle-reads/` | `retention_closure/local/reads/oracle.rs:53` | Workspace local-read compiler-oracle tests |

The table's source paths are relative to
`crates/rum-stdlib/src/tests/`. These are existing real directories, not
symlinks into brief-613. Exact resolved paths are recorded in
`.temp/brief-613/redispatch-scratch-stop.json`.

The Hard rule prohibits scratch outside brief-613 except the named
exceptions. Amendment C adds only `.temp/suspension-oracle/` and
`.temp/brief-611/r3/census/`, and requires repository execution instead of a
verification copy. The four paths above are not among those exceptions.
Their writers are outside the editable source scope. I did not run those
writers, alter the tools, redirect paths with filesystem tricks, or infer a
further exception from the general gate requirement. The director needs to
authorize those canonical-tool output paths or explicitly authorize a
different routing change before those gates can run under the current rules.

### 2. Approved discovery replay and all changed files

Fresh discovery is in `redispatch-ownership-discovery.txt`. The replay script
is `.temp/brief-613/replay-ownership.py`; its raw output is
`redispatch-ownership-replay.txt`. It checks the host population, permitted
hop, complete self-reaching edge witnesses, existing construct pairings and
unchanged ledger cells before writing inventories.

The approved transition was applied at both
`ownership_graph/inventory/host_payloads.tsv:57` and
`ownership_graph/inventory/host_payload_ledger.tsv:53`:

```text
old: FrameQueue.Legacy.0/VecDeque[Own]/arg0/Arc[LastStrongHolder]/arg0/alias(DynSuspensionFrame)
new: FrameQueue.Legacy.0/VecDeque[Own]/arg0/Queued<arg0> via payload/Arc[LastStrongHolder]/arg0/alias(DynSuspensionFrame)
```

The new construct is copied from discovery. The four ledger cells remain
byte-identical: `Unbounded`, its existing AccountedCompoundFrame expression
witness, `TODO.md#standing-engineering-debt`, and `unresolved`. There is no
other host-payload identity transition, and no `FrameQueue.Mixed` or `Entry`
row change. All 59 self-reaching identities and complete edge witnesses are
unchanged. No existing UnknownConstruct row is removed; coordinate-only
changes are paired after normalizing embedded source coordinates. Fifteen
new derive/macro/closure sites in the owned code remain UnknownConstruct;
none is reclassified or waived.

Only these six repository inventory files changed in this redispatch,
followed by this report appended to the assigned brief:

| File under `crates/rum-stdlib/src/tests/ownership_graph/inventory/` | Changed or added rows |
| --- | ---: |
| `host_payloads.tsv` | 6 |
| `host_payload_ledger.tsv` | 6 |
| `unknown_attributes.tsv` | 5 |
| `unknown_captures.tsv` | 18 |
| `unknown_macros.tsv` | 13 |
| `unknown_storage.tsv` | 1 |

**Every one of the 49 old → new row disclosures** is retained in
`.temp/brief-613/redispatch-ownership-disclosures.md` and its machine-readable
`redispatch-ownership-row-changes.json`; exact file hunks are in
`redispatch-ownership-replay.patch`. Of these, 34 pair existing rows and 15
add newly observed unresolved constructs. Host rows remain 79; constructs
change 7,694 → 7,709; self-reaching rows remain 59. Status, disposition,
witness, coverage and exclusion cells were not rewritten.

The director's tuple ruling is recorded explicitly: the
`(usize, usize, Arc<…>)` layouts in `session_memory_structures.rs` and typed
accounting tests are **stand-ins for the private `Queued<…>`**. The tests
check their size through exact independent memory equations. The saved-root
count 1 → 3 follows the full snapshot. No further pin edit was made this run.

### 3. Exact verification output and the remaining census failure

All gates in this redispatch ran in `/home/remi/dev/rum`, serially, offline,
with one build job and test thread, through a 900-second bounded command.
The target and TMPDIR were under brief-613; canonical census output used its
explicitly authorized path. Raw output was saved before parsing it.

The two strengthened runtime assertions are now tested. The corrected
macro adapter is watched green in both feature builds after its recorded red.
The following lines are copied from the named `redispatch-*.txt` files:

```text
context:
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1176 filtered out; finished in 0.15s
macro-default:
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 102.44s
macro-no-default:
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1149 filtered out; finished in 102.88s
memory (the brief's literal filter session_memory_maintained):
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1187 filtered out; finished in 0.00s
memory-module (the actual session_memory::maintained module):
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.23s
typed:
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 0.47s
root-trace:
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 1.28s
runtime:
test result: ok. 1184 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 29.68s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.41s
native48:
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.01s
ownership:
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 69.34s
compile-default:
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 55.46s
compile-no-default:
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.33s
census-default:
test result: FAILED. 2 passed; 6 failed; 1 ignored; 0 measured; 0 filtered out; finished in 170.35s
```

`cargo fmt --check` also passes (`redispatch-fmt.txt`, empty successful
output). Each compile-control build reports **34 positive controls / 34
intended rejections**, including E0271 at the helper's input and output
constraint sites. The full runtime's three ignores remain its release-only
cache/profile and two left-chain deployment/drop probes. The three existing
dead-code warnings in read-only runtime test builtins remain; they are not
new oracle warnings. The census's ignored `forced_red_probe` is invoked
explicitly by its bounded falsifier harness. The workspace did not run, so
there are no new native ZeroMQ results or skips to report.

The full ownership output confirms:

```text
HOST PAYLOAD LEDGER: 79 rows; 11 storage edges (3 Declared, 8 Unbounded); 68 PublicInput
EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 79 rows
EXACT INVENTORY UnknownConstruct: 7709 rows
```

The census failure is `X receipt mismatch: expected 40, discovered 48`, with
no missing existing receipt. All eight unrecorded sites are direct
`insert_suspension_frame` calls in the new G14 test controls. Their exact
emitted identities and coordinates are in
`redispatch-census-new-test-sites.json`, and the complete fresh 48-row source
list is copied to `redispatch-census-legacy-test-users.tsv`. The eight sites
are the two oracle witness handlers, three typed route-control insertions,
and three runtime context-control insertions. The production census remains
607 implementations (37 typed) and 577 macros; the test-only partition
remains 16 implementations and 11 macros. No G13 production receipt changed.

I had begun preparing the test-support count replay, but **no census receipt
was edited** before identifying the fixed-output conflict. Consequently the
six baseline-dependent census tests remain red, the no-default census was
not run, and the receipt replay is still pending. This is not a green census
or a reason to weaken its exact equality check.

### 4. Accepted semantics, outstanding work and stop-loss

The accepted prior red ledger, 71 strict passing descriptors, seven mutation
falsifiers, 2,000/default and 100/limit-32 runs are unchanged; this redispatch
does not claim it reran those full oracle populations. The 11-test context
rerun verifies the previously strengthened exact return-level and closed
source-boundary assertions, and again records the three contexts: full
four-entry false-cycle snapshots, three-entry depth-leak snapshots at fixed
depth 5 over 1,100 pauses, and two-entry passing-shape snapshots. All initial
entry epochs are retained, while intervening global epoch changes survive.

The required full suspension-oracle script has not yet run. Its default and
no-default fault-occurrence deltas are therefore **both unmeasured**. Local
production acceptance, contract fingerprints, certificates, production
rejection census and the full workspace also remain outstanding. The prior
report's semantic/accounting evidence is not substituted for those gates.

No performance measurement or candidate performance build has started.
All 107 allocation/byte rows, instruction slopes and address attribution,
memcpy/leaves, native stack tables/prologues and final monotonic timing A/B
remain unmeasured. Amendment D's per-workload widening counts, moved-cell
bytes and `Queued<Entry>` versus `Queued<Arc<…>>` sizes are likewise pending.
No redesign or performance acceptance is claimed. Supplied base binaries
were not changed or rebuilt.

There was no second semantic repair attempt and no performance repair
attempt. No stop-loss threshold was reached. This STOP is solely the
remaining scratch-boundary conflict, with the separately recorded census
test-support replay still unfinished. All pending work and exact evidence
paths are checkpointed in `.temp/brief-613/PROGRESS.md`.

## Director amendment — redispatch 2: canonical tools write where they write

The STOP was correct, and it is my omission. Amendment C named two output
paths where it should have stated a class. It is a class now.

### A. Canonical tool outputs

Any **unchanged** canonical gate tool, meaning a repository test or script
that this brief requires you to run, may write to its own fixed output
paths under `target/` or `.temp/`. This includes the four you found:
- `target/brief-535/`;
- `target/brief-540/oracle/`;
- `.temp/brief-542/falsifiers/`;
- `.temp/brief-543/oracle-reads/`;

and any others the workspace or the performance tools reach, such as
`scripts/core-language-microbench.sh`.

You may not modify, redirect or wrap those tools to move their output.
Disclose each such path in the report. Your own scratch stays in
`.temp/brief-613/`. Do not STOP for another path of this class.

### B. The census's test-support receipts

There are eight new direct `insert_suspension_frame` sites, all in G14's own
test controls (two oracle witness handlers, three typed route controls and
three runtime context controls). They join the census's test-support
partition by replay under the pins-that-follow-code class, each disclosed.
Test-only sites need the legacy route, because the route controls exist to
cover it, and they leave with `Legacy` at R.

The production receipts must not change. Any production receipt change, or
any new site outside those eight, is a STOP.

### C. Noted

The brief's `session_memory_maintained` filter name was mine and wrong. The
module is `session_memory::maintained`, as you ran it.

### D. Then continue Amendment E from where you stopped

- the no-default census;
- local production, fingerprints and certificates;
- the production rejection census;
- the oracle script in both builds;
- the workspace;
- then the whole performance order, including Amendment D's widening
  attribution.

The process is as before. Append `## Execution report — redispatch 2`
after this section.
