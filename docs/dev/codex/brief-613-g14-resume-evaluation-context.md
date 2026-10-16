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

## Execution report — redispatch 2

**STOP: the retention frontier contains an unpaired removal. This is still an
incomplete execution, not a green gate.** The canonical-output authorization
was used, the eight approved test-support receipts were replayed, and both
census builds and both local-production builds are green. The production
rejection census is red. No census pin was changed after discovering the
unpaired removal. The source remains uncommitted; no git, network, sub-agent,
new unsafe code or aborting probe was used.

### 1. STOP evidence and the precise remaining boundary

The canonical repository command
`cargo test -p rum-stdlib --lib production_rejection_census_is_complete -- --nocapture`
first completed its independent all-definition audit, then failed at
`crates/rum-stdlib/src/tests/retention_closure/census/tests.rs:191`. Its
abandonment anchors `[42, 43, 59]` now lie at `[41, 42, 60]` in the same file.
Those three moves are ordinary source pins. Before editing them or the
aggregate count, I compared the old and current option frontiers.

The existing S5 frontier is
`.temp/brief-611/r3/frontier-candidate.tsv` (937 rows). Auxiliary discovery
using the current analyzer and reading the **real repository** emitted
`.temp/brief-613/frontier-candidate.tsv` (950 rows). This auxiliary discovery
is diagnostic evidence, not a replacement run or a claimed pass of the
canonical gate. Its source and raw outputs are in
`.temp/brief-613/frontier-discovery/` and `r2-frontier-discovery-2.txt`.

The comparison finds a removed, rather than relocated, diagnostic:

```text
kind: RET002
file: crates/rum-runtime/src/runtime/suspension/core/abandon.rs
old line: 41
old operation: self.evaluation_stack.len()
old statement: self.evaluation_depth = self.evaluation_stack.len();
reason: method syntax: method receiver lacks checked nominal/place provenance
new counterpart: absent
```

Its source deletion is the approved removal of depth-from-length restoration.
Its existing frontier rejection nevertheless disappears; it cannot be paired
as the same operation moved to a new line. The brief separately says:
“A new `UnknownHostPayload`, an unpaired removal, or a changed identity,
status, disposition or exclusion is a STOP.” I have not treated approval of
the implementation model as an exception to that replay rule, or replaced
this row with the different `len()` operation in accounted truncation.

The complete classification is:

| Frontier fact | Old → current |
| --- | --- |
| Total | 937 → 950 |
| RET001 | 10 → 10 |
| RET002 | 901 → 913 |
| RET003 | 26 → 27 |
| Same diagnostic operation, allowing coordinate moves | 930 pairs |
| Same macro operations after changing only the selected arm | 5 further pairs |
| Checked macro-return rejection | `witness.suspend` → private `Expr` `witness.suspend_evaluation_return` |
| New cold-operation rejections | 14: twelve in maintained stack handling, one return-event operation, one context assertion |
| Unpaired removal | The old abandonment `evaluation_stack.len()` rejection above |

The checked macro return remains rejected; the new event and newly reached
cold operations also remain rejected. No retention coverage or local mark is
claimed for them. Full old/new rows, complete witness paths and source
snippets are preserved in `r2-frontier-comparison.json`,
`r2-frontier-stop.json` and `r2-frontier-disclosures.md` under brief-613.
The latter records the return-operation transition and every new diagnostic.

No edit was made to `retention_closure/census/tests.rs`. Besides the three
line pins, its assertions at lines 223 and 228 still require 937 and
`(10, 901, 26)`. The stale printed baseline at line 246 also remains as it
was. A director ruling on the disappearing frontier diagnostic and the
checked return-operation transition is needed before replaying these facts.
There was no further correctness or performance execution after identifying
this STOP.

### 2. Files changed, replay constraints and all row disclosures

Only the following three repository files changed in this redispatch,
followed by this report appended to the assigned brief:

| File | Lines / cells changed | Evidence and preserved contract |
| --- | --- | --- |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` | Eight appended rows, lines 2341–2348 | Exactly the eight approved discovered test-support insertions; every previous row is byte-identical, including every production receipt. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | Fingerprint cell in each of 68 rows, lines 2–69 | Recomputed with the canonical length-prefixed FNV algorithm; the other eighteen cells in each row are byte-identical. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | Four certificate cells across C01/C02, lines 2–3 | Copied from both canonical emissions after all seven production/probe rows earned zero rejections; the other seven columns are unchanged. |

The census replay uses the canonical `stable` identity algorithm for the new
IDs and copies site identity/file/name from discovery. All eight remain at
stage R, status `legacy-test`, population `test-support`. They are the two
oracle witness handlers, three typed route insertions and three runtime
context insertions authorized by Amendment B. No G13 production receipt or
stage assignment changed.

**Every added or changed row is disclosed old → new** in these brief-613
artifacts:

- Census: `r2-census-disclosures.md`, `r2-census-row-changes.json` and
  `r2-census-replay.patch`. The disclosure also records the unchanged
  production-receipt digest.
- Contracts: `r2-contract-disclosures.md`, `contract-fingerprint-diff.tsv`,
  `r2-contract-row-changes.json` (complete old/new rows), and
  `r2-contract-replay.patch`. The replayed context is
  `fnv1a64:b37a54abc5819a88`.
- Certificates: `r2-certificate-disclosures.md`, `certificate-diff.json`,
  `certificate-replay.patch`, and copies of both emitted
  `promoted-hash-{true,false}.tsv` files.

The earlier six-file ownership replay is unchanged and remains disclosed in
the preceding report. The exact queue-envelope transition, all four ledger
cells and the 79 host/59 self-reaching/7,709 unknown-construct populations
have not been edited again. No obligations row or other retention inventory
was changed this redispatch.

`r2-final-file-audit.json` confirms 31 source/inventory files differ from the
initial snapshot: the prior 22 Rust files, six ownership inventories, and the
three files above. All are disclosed. `r2-source-check.json` confirms that
**all 22 implementation/test Rust hashes are unchanged from the first
execution**. In particular the rejected census test is unchanged. Initial
source hunks and the accepted tuple-layout stand-ins remain documented in the
first two reports.

### 3. Exact verification output and canonical outputs

Commands ran serially in `/home/remi/dev/rum`, offline, with one build job and
test thread. Each gate used `scripts/run-bounded.py --timeout 900`, with raw
output redirected under brief-613 before parsing. The following lines are
copied from their `r2-*.txt` logs:

```text
census-default:
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 232.40s
census-no-default:
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 231.79s
local-default:
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.37s
local-no-default:
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.21s
production-census:
EDGE_ALL roots=7355 definitions=7355 raw=48522 deduplicated=48522 resolved_sites=16230
FIRST_FAILURE EDGE_ALL rejected=7352 returned_ok=3 (effects::check per root; not edge frequency or production coverage)
INDEPENDENT_AUDIT EDGE_ALL roots=7355 definitions=7355 raw=48522 deduplicated=48522 reason_classes=62 all_spans_read=true
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 17.31s
```

Both census logs include intentionally failing child probes, and each ends
with the green eight-test parent result above. The one ignored test is the
forced-red harness entry invoked explicitly by those controls. Production
counts remain 607 implementations (37 typed), 577 macros; the test-only
partition remains 16 implementations and 11 macros.

Before replay, each local-production emission earned all seven zero-rejection
rows and failed only on `stale or omitted instance certificate C01 hash=true`
or `hash=false`. Those two expected failures are retained in
`r2-local-emission-{default,no-default}.txt`; neither is counted as a passing
gate. Their emitted cells were replayed, then both final tests passed as
shown above. The auxiliary frontier probe initially had missing imports from
unneeded compiler-oracle test modules. Removing that unused module from the
scratch probe resolved its compile error; no repository source changed.

Unchanged canonical tools wrote to these fixed output roots this redispatch:

| Output root | Writer |
| --- | --- |
| `.temp/brief-611/r3/census/` | Both typed-slot census builds and their falsifiers |
| `.temp/brief-546/certificates/` | Local-production emissions and final acceptance runs |
| `target/brief-535/EDGE_ALL/` | Production rejection census and independent audit |

These are covered by Amendment A. No canonical tool was modified or
redirected to move its fixed outputs. The `EDGE_ALL` artifacts were copied
after the run into `.temp/brief-613/r2-canonical-EDGE_ALL/` for this report.
Own scripts, copied harness/programs, auxiliary discovery, targets, TMPDIR and
raw logs all stayed under brief-613. The other previously disclosed canonical
paths were not newly exercised because the remaining gates did not run.

### 4. Accepted semantics, unmeasured work and stop-loss

The accepted prior semantic evidence is unchanged: S5 rejects 25/34 seeds and
27/37 dimensions; the candidate passes all 71 strict descriptors; six initial
invariants turn green; all seven class mutants are rejected with their
input-free control green; 2,000/default and 100/limit-32 runs complete; both
resource witnesses reach 99. The three full-context traces and the eleven
strengthened runtime controls are recorded in the prior reports and logs.
They were not rerun in this redispatch and are not a substitute for final
correctness closure.

The full oracle script and workspace **have not run**. Default and no-default
full-oracle fault-occurrence deltas remain **unmeasured**; neither oracle
build's warning-free gate is claimed. There are no new native ZeroMQ results
or skips. Earlier green runtime, maintained-memory, typed, root-trace,
macro-adapter, compile-control and ownership counts remain in the redispatch-1
report, with no affected Rust source change in this redispatch.

No performance build or measurement has run. The exact 139-label panel and
resume harness were copied under brief-613, with source-copy hashes in
`performance-input-copy.json`. Scratch runners were prepared for complete
N/2N result/resume-count preflight and explicit ineligible-base reporting,
but were not executed. The supplied S5 binaries were neither modified nor
rebuilt; their previously verified hashes were not redundantly rechecked.

All 107 ordinary allocation/byte comparisons, instruction slopes and address
attribution, resume allocation/cost attribution, per-workload widening owners
and moved-cell bytes, memcpy/leaves, native-stack samples/prologues/slot
attribution, and final monotonic timing remain outstanding. There is no
default-path performance claim and no proposed acceptance of an unmeasured
resume cost. Physical-safety equivalence remains the approved native policy;
identical addresses or polling cadence are not claimed.

The semantic attempt count remains **1 / 6**, with no new semantic or
performance repair attempt and no stop-loss threshold reached. G13's
remaining 73 steps and every production receipt are unchanged. The next
action requires a director ruling on the retention frontier transition above;
after that, replay the permitted pins and any resulting fingerprints and
certificates, finish the rejection census/oracle/workspace gates, then execute
the full performance order. State and evidence are checkpointed in
`.temp/brief-613/PROGRESS.md`.

## Director amendment — redispatch 3: derived facts follow the diff

The STOP was correct under the rule's wording, and stopping was the safe
reading. The rule was written for the ownership inventories, and its words
reached further than its purpose. Here is the purpose, stated as a class,
for every derived inventory and frontier this step touches:

### A. The class rule

Each row or diagnostic falls into exactly one case:

| Its operation in this brief's diff | Ruling |
| --- | --- |
| **Deleted** by the ruled design | Retired with its source. Disclose it. |
| **Added** by this diff | A new row. It stays rejected or unknown; no mark or coverage is claimed. Disclose it. |
| **Moved**, or its text follows a ruled rename (such as the selected macro arm) | Paired. Disclose it old → new. |

A STOP is a change of *meaning* where the source did not change:
- a row whose operation's source text is unchanged disappears, or changes
  kind, reason, status, disposition or exclusion;
- a retention mark, coverage or promotion is gained or lost;
- a new `UnknownHostPayload`;
- any change in a production census receipt.

### B. This case

- The `RET002` at old `abandon.rs:41` (`self.evaluation_stack.len()` in
  `self.evaluation_depth = self.evaluation_stack.len();`) is **retired with
  its statement**. Deleting it is the ruled removal of depth-from-length.
  brief-609 S3 retired a Try boundary the same way.
- `witness.suspend` → `witness.suspend_evaluation_return` at the two
  fixed-point sites is **paired**, because it follows the ruled arm.
- The fourteen new cold-operation rejections are **new rows**: twelve in
  the maintained stack handling, the return event and the context
  assertion.
- In `retention_closure/census/tests.rs`, replay as follows:
  - the abandonment anchors `[42, 43, 59]` → `[41, 42, 60]`;
  - the total 937 → 950;
  - the kinds `(10, 901, 26)` → `(10, 913, 27)`;
  - a comment in the file's existing per-brief style that names the
    retired row, the paired transition and the fourteen additions.
- Leave the printed `ACCEPTANCE_BASELINE` line as it is. It is a historical
  constant, not the current frontier.

Then replay whatever fingerprints and certificates follow from it.

### C. Then continue

Continue with:
- the production rejection census;
- the oracle script in both builds;
- the workspace;
- the whole performance order, including the widening attribution.

The process is as before. Append `## Execution report — redispatch 3`
after this section.

## Execution report — redispatch 3

**Execution is complete; the director's cost ruling remains open.** All required
correctness commands and performance sections ran on the final implementation.
The 107-label ordinary allocation panel is exactly equal, no hardware instruction
flag persists after the prescribed repeats, both copy counts are at or below
base, the three leaves have no deterministic instruction growth, and all 120
native-stack pairs are byte-identical to S5. Resume costs and cold prologue growth
are measured below. The final canonical timing indication retains a material
association-read increase: **20.8206% at width 32,000** across two A/B pairs. Its
cause is unresolved; this report does not certify an overall timing no-regression
result or pre-approve any resume cost.

No git command, network operation, sub-agent, new unsafe code, or aborting probe
was used. Source was left for the director; no commit was made. Own scratch is
under `.temp/brief-613/`; canonical fixed-output exceptions are disclosed below.

### 1. Final source and replay disclosures

There are **32 changed code/inventory files**, plus this assigned brief. All 22
original implementation/test files retain the hashes of semantic attempt 1.
Redispatch 3 changes only the approved census pins/comment and the fingerprints
and emitted certificates that follow them. No implementation changed during
performance measurement. The initial snapshot and final hashes delimit this
agent's edits and preserve prior worktree changes.

| File | Final hunk lines | Reason |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/debug_tools/resources/maintained.rs` | 205–222 | Cold accounted truncation; preserve vector capacity and occurrence accounting. |
| `crates/rum-runtime/src/runtime/debug_tools/resources/occurrence/suspension.rs` | 11–19, 27–33 | Independent queue-size model for context cells and inline events. |
| `crates/rum-runtime/src/runtime/resolution.rs` | 376–382, 440–446 | Only the two InputRequired branches select evaluation_return. |
| `crates/rum-runtime/src/runtime/session/guest_loading/scope.rs` | 178–184 | Delete the obsolete top-two comment; behavior unchanged. |
| `crates/rum-runtime/src/runtime/suspension/core.rs` | 13–25, 169–190, 196–202, 492–500, 521–532, 556–567, 610–617, 636–643, 664–671, 701–745, 764–790, 811–829 | Full snapshot, capture/restore at queue/driver boundaries, native-state preservation. |
| `crates/rum-runtime/src/runtime/suspension/core/abandon.rs` | 38–50 | Restore each queued context before abandonment; remove depth-from-length. |
| `crates/rum-runtime/src/runtime/suspension/core/evaluation_context.rs` | 1–128 | Private context, envelope, return epilogue, native polling, test observations. |
| `crates/rum-runtime/src/runtime/suspension/core/queue.rs` | 1–10, 15–24, 28–99, 101–128, 146–153, 156–170, 179–190, 204–210 | Mandatory context on Legacy/Mixed cells; preserve widen/append; inline return event. |
| `crates/rum-runtime/src/runtime/suspension/typed/entry.rs` | 9–21, 23–29, 34–40, 43–49, 53–59, 61–67, 73–79 | Return-event traversal, no retained roots or per-event allocation. |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs` | 59–76, 91–99 | Exact size/count pins for context cells; adapter allocation unchanged. |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/evaluation_context.rs` | 1–256 | Route, widening, splice, Evaluate, Finalize, error and fault controls. |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/live.rs` | 383–388 | Register the new route controls. |
| `crates/rum-runtime/src/runtime/suspension/typed/witness.rs` | 28–71 | Private Expr-to-Expr helper and new macro arm; original arms unchanged. |
| `crates/rum-runtime/src/runtime/tests/mod.rs` | 192–196 | Register runtime context tests. |
| `crates/rum-runtime/src/runtime/tests/resume_evaluation_context.rs` | 1–420 | Context, epoch, accounting, native, lifecycle, trace and caller-boundary tests. |
| `crates/rum-runtime/src/runtime/tests/session_memory_structures.rs` | 249–258 | Full-snapshot and private-envelope size/count pins approved in amendment 1. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv` | 50–61 | Discovery-only replay; approved envelope hop or moved/new unknown constructs. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv` | 49–60 | Discovery-only replay; approved envelope hop or moved/new unknown constructs. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | 912–922 | Discovery-only replay; approved envelope hop or moved/new unknown constructs. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 185–191, 1753–1775 | Discovery-only replay; approved envelope hop or moved/new unknown constructs. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 1770–1789, 1868–1874 | Discovery-only replay; approved envelope hop or moved/new unknown constructs. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv` | 70–76 | Discovery-only replay; approved envelope hop or moved/new unknown constructs. |
| `crates/rum-stdlib/src/tests/retention_closure/census/tests.rs` | 187–193, 214–237 | Approved anchors, 950 frontier and per-brief comment; historical printed baseline retained. |
| `crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs` | 371–388, 513–552 | Four-view arm checks and drift mutations. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 1–69 | Replay derived fingerprints or emitter-earned certificate cells; other columns unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 1–3 | Replay derived fingerprints or emitter-earned certificate cells; other columns unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs` | 54–80, 134–145, 152–161, 178–195, 205–211 | Exact checked expansion for the third arm and drift checking. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_stack.rs` | 1–477 | 34 seeds, 37 dimensions, complete ledgers and mutation panel. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/evaluation_stack_support.rs` | 1–131 | Test-only resource and populated-widening witnesses. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs` | 16–22 | Register G14 fixtures. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` | 2338–2348 | Replay eight G14 test-support receipts; every production receipt unchanged. |
| `crates/rum-stdlib/tests/typed_slot_compile_controls.rs` | 230–261, 327–332 | Two input/output type-constraint pairs; paired count 32 to 34. |

Complete hashes and old/new hunks: [source audit](../../../.temp/brief-613/r3/final-file-audit.json)
and [complete source patch](../../../.temp/brief-613/r3/complete-source.patch).
Every inventory row is disclosed initial → final in
[all 127 TSV row pairs](../../../.temp/brief-613/r3/final-inventory-disclosures.md) and
[machine-readable row pairs](../../../.temp/brief-613/r3/final-inventory-row-changes.json).
Applying those evidence rows as a multiset reproduces all nine final TSVs
exactly; no inventory was hand-refreshed. The rows comprise 49 ownership
disclosures (34 paired, 15 new unknown constructs), eight new test-support
receipts, 68 contract fingerprints, and two promoted rows containing four emitted
certificate cells. Intermediate replay history remains in the earlier reports
and `r2-*`/`r3/` artifacts.

The only host identity transition is the approved
`FrameQueue.Legacy.0/VecDeque/arg0/Arc` →
`FrameQueue.Legacy.0/VecDeque/arg0/Queued<arg0> via payload/Arc` hop.
`Unbounded`, witness, `TODO.md#standing-engineering-debt`, and `unresolved`
coverage remain unchanged. Discovery is 79 hosts, 59 self-reaching identities,
and 7,709 unknown constructs; the final discovery replay changes zero additional
rows. There is no new UnknownHostPayload or production census receipt change.

The frontier follows amendment 3: old `abandon.rs:41` RET002
`self.evaluation_stack.len()` retires with the depth assignment; the two
`witness.suspend` selections pair with `suspend_evaluation_return`; fourteen
newly reached cold rejections remain rejected. All individual frontier records
are preserved in [frontier old/new evidence](../../../.temp/brief-613/r2-frontier-disclosures.md)
(its historical STOP is resolved by the amendment). The exact source pins are
`[42,43,59]` → `[41,42,60]`, total `937` → `950`, and kinds
`(10,901,26)` → `(10,913,27)`. `ACCEPTANCE_BASELINE` remains historical and
unchanged. The final dependency context is `fnv1a64:409e4bf550c83fe9`;
[redispatch-3 replay disclosure](../../../.temp/brief-613/r3/replay-disclosures.md) records all
68 recalculated fingerprints and four emitted certificate cells. Both emission
runs earned seven zero-rejection rows. No retention mark, coverage, promotion,
status, disposition, or exclusion was gained or lost.

Other count pins: saved roots 1 → 3; queue layout gains two usize words;
insertion charge 136 → 200 bytes for four cells; adapter allocation stays five
usize words. `(usize, usize, Arc<…>)` is the approved size-test stand-in for the
private envelope. Compile controls are 32 → 34 positive/negative pairs.
`core.rs` is 31,587 bytes in the split-plan range; the 4,647-byte context module
holds the new responsibility. No unrelated split was made.

### 2. Red ledger, attempts, and exact correctness results

Before production edits, all 34 seeds were attempted through Fixture and
`accept(Checks::ALL)`: 25 failed, nine passed, and 25 raw pairs were unequal.
The first 32 dimensions had 25 failures; all six initial runtime invariants
failed. The complete final S5 replay attempted all 71 descriptors: 25/34 seeds
and 27/37 dimensions failed, for 52 failures total. The originally passing
populated-widening and post-child epoch controls reject their targeted mutants.

One compiled semantic repair changed the original 50 unequal pairs to zero and
all six invariant failures to zero. A strict fixture typo (HoldForm instead of
the observed/documented queued Hold) and two exact layout pins were corrected;
the expanded final population is 71/71 green. The macro adapter was watched red
(1 pass/5 failures), then caught a checked-token brace mismatch (1 pass/6
failures), and is now seven tests green in each feature configuration. The two
pre-existing macro arms are byte-identical. No further semantic repair was
compiled. The red, mutation and restoration ledgers remain in `red-*.txt`,
`mutation-summary.json`, and `mutants/`.

All seven class-mutant executions rejected their assigned cases and kept the
separate input-free control green: drop-two; omitted restore; depth-from-length;
lost widen record; lost append record; epoch rewind; skipped success epilogue.
Initial mutation-harness misses were retained and the controls strengthened;
no failing descriptor was removed. Mutated files were restored and hashes checked.

| Required gate | Exact result line(s) |
| --- | --- |
| [Final fmt](../../../.temp/brief-613/r3-fmt.txt) | exit 0; formatter emits no test-result line |
| [Strengthened context assertions](../../../.temp/brief-613/redispatch-context.txt) | test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1176 filtered out; finished in 0.15s |
| [Maintained-memory actual module filter](../../../.temp/brief-613/redispatch-memory-module.txt) | test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1173 filtered out; finished in 0.23s |
| [Typed routes](../../../.temp/brief-613/redispatch-typed.txt) | test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1163 filtered out; finished in 0.47s |
| [Frame root tracing](../../../.temp/brief-613/redispatch-root-trace.txt) | test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 1.28s |
| [Runtime unit and rustdoc suite](../../../.temp/brief-613/redispatch-runtime.txt) | test result: ok. 1184 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 29.68s<br>test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.41s |
| [48-level native control](../../../.temp/brief-613/redispatch-native48.txt) | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.01s |
| [Macro adapter, default](../../../.temp/brief-613/redispatch-macro-default.txt) | test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 102.44s |
| [Macro adapter, no-default](../../../.temp/brief-613/redispatch-macro-no-default.txt) | test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1149 filtered out; finished in 102.88s |
| [Compile controls, default (34 positive/34 negative)](../../../.temp/brief-613/redispatch-compile-default.txt) | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 55.46s |
| [Compile controls, no-default (34 positive/34 negative)](../../../.temp/brief-613/redispatch-compile-no-default.txt) | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.33s |
| [Census, default](../../../.temp/brief-613/r2-census-default.txt) | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.52s<br>test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.66s<br>test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 232.40s |
| [Census, no-default](../../../.temp/brief-613/r2-census-no-default.txt) | test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.84s<br>test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.89s<br>test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 231.79s |
| [Ownership suite](../../../.temp/brief-613/redispatch-ownership.txt) | test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1131 filtered out; finished in 69.34s |
| [Local production, default](../../../.temp/brief-613/r3-local-default.txt) | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 18.31s |
| [Local production, no-default](../../../.temp/brief-613/r3-local-no-default.txt) | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1155 filtered out; finished in 18.16s |
| [Production rejection census](../../../.temp/brief-613/r3-production-census.txt) | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1161 filtered out; finished in 25.47s |

The independent production audit covers **7,355 definitions, 48,522 rejection
rows, and 62 reason classes**. The original literal memory filter matched zero;
the corrected `session_memory::maintained` filter ran 14 tests, and the complete
runtime/workspace suites ran it again. The historical census harness ignore is
intentional, not an omitted acceptance run.

| Canonical oracle build | Exact result | Fault guards | Maximum delta | Final observations | Warnings |
| --- | --- | --- | --- | --- | --- |
| default | test result: ok. 136 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.71s | 87 | 0 | 90318 | 0 |
| no-default | test result: ok. 136 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.57s | 87 | 0 | 90318 | 0 |

The unchanged `scripts/check-suspension-oracle.sh 1500` passed in the repository.
[Full oracle raw output](../../../.temp/brief-613/r3-oracle.txt); per-build logs and summary are
under `r3/`. Both builds have 136 passes, zero warnings, and all 87 fault guards
at delta zero.

Workspace `--no-fail-fast` exits 0: **53 top-level result groups, 3,568 libtest
passes, zero failures, 59 ignores**. Nine native ZeroMQ tests explicitly skipped
their bodies because loopback binding is unavailable with
`RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1`. They are included in libtest's pass count:
**3,559 executed passes plus nine explicit skips**. The two expected forced-red
child results are recorded separately and are not top-level failures.
[Complete raw workspace output](../../../.temp/brief-613/r3-workspace.txt) and
[suite/count/skip ledger](../../../.temp/brief-613/r3/workspace-summary.json) preserve the distinction.
Exact final top-level output follows.

```text
Running unittests src/main.rs (.temp/brief-613/target/debug/deps/rum_cli-0aee412e0b99cbf1)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
Running tests/cli.rs (.temp/brief-613/target/debug/deps/cli-bc0b4c8761b62ae9)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
Running tests/input_requests.rs (.temp/brief-613/target/debug/deps/input_requests-e84d0b9e62f1dd0f)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
Running tests/terminal.rs (.temp/brief-613/target/debug/deps/terminal-70e027c8b6d403e3)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.94s
Running unittests src/lib.rs (.temp/brief-613/target/debug/deps/rum_core-05ee71324ea83d3f)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/lib.rs (.temp/brief-613/target/debug/deps/rum_derive-058a94fcf3064f03)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/accumulator_chain.rs (.temp/brief-613/target/debug/deps/accumulator_chain-c657e563db7a31de)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/derive.rs (.temp/brief-613/target/debug/deps/derive-cb1c0b1fa13c3a60)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/lib.rs (.temp/brief-613/target/debug/deps/rum_host-0c15fc615add95e2)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s
Running tests/adapter_contract.rs (.temp/brief-613/target/debug/deps/adapter_contract-75627f143cb45b08)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
Running tests/batch_boundary_ownership.rs (.temp/brief-613/target/debug/deps/batch_boundary_ownership-f1d7a1c822224b1c)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
Running tests/capture_output_lifecycle.rs (.temp/brief-613/target/debug/deps/capture_output_lifecycle-3426e74b0e43035a)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
Running tests/guest_loading_resume.rs (.temp/brief-613/target/debug/deps/guest_loading_resume-3f3ebe72da026eee)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
Running tests/host_payload_destruction_contract.rs (.temp/brief-613/target/debug/deps/host_payload_destruction_contract-52956b33f57a0130)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 70.15s
Running tests/installer.rs (.temp/brief-613/target/debug/deps/installer-245b9a9d0dd76683)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
Running tests/lifecycle_buildout.rs (.temp/brief-613/target/debug/deps/lifecycle_buildout-cf4abded19978b7b)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
Running tests/lifecycle_contract.rs (.temp/brief-613/target/debug/deps/lifecycle_contract-7db6f3bc8fd1b373)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
Running tests/random_password_policy.rs (.temp/brief-613/target/debug/deps/random_password_policy-bcf408336a660b97)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
Running tests/shared_accumulator_completion.rs (.temp/brief-613/target/debug/deps/shared_accumulator_completion-4fc6d427214e6aa2)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
Running tests/stress.rs (.temp/brief-613/target/debug/deps/stress-29e64a7d12eeb866)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.30s
Running tests/suspended_timeout_contract.rs (.temp/brief-613/target/debug/deps/suspended_timeout_contract-0e6a452c6cdaf889)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.03s
Running tests/suspension_abandonment_contract.rs (.temp/brief-613/target/debug/deps/suspension_abandonment_contract-7953000542543e9e)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
Running unittests src/lib.rs (.temp/brief-613/target/debug/deps/rum_jupyter-598afbd7e7b200bf)
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
Running unittests src/bin/rum-jupyter-kernel.rs (.temp/brief-613/target/debug/deps/rum_jupyter_kernel-0f1e351c2b3d67dc)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running unittests src/lib.rs (.temp/brief-613/target/debug/deps/rum_repl-4387fdc47fc58f50)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
Running tests/input_abandonment.rs (.temp/brief-613/target/debug/deps/input_abandonment-11b7b14c69d5d259)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
Running tests/input_continuation.rs (.temp/brief-613/target/debug/deps/input_continuation-02bf146f80080963)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
Running unittests src/lib.rs (.temp/brief-613/target/debug/deps/rum_runtime-f6425a9dcb35714f)
test result: ok. 1184 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.06s
Running unittests src/lib.rs (.temp/brief-613/target/debug/deps/rum_stdlib-8951251f7d2e445e)
test result: ok. 1109 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1354.98s
Running unittests src/bin/generate-builtin-reference.rs (.temp/brief-613/target/debug/deps/generate_builtin_reference-d2ad2c98e57a1cba)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Running tests/default_fallback_continuations.rs (.temp/brief-613/target/debug/deps/default_fallback_continuations-f117f222c1cc964c)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
Running tests/destructuring_continuations.rs (.temp/brief-613/target/debug/deps/destructuring_continuations-368dcefe778b9dd9)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
Running tests/guest_loading_resume.rs (.temp/brief-613/target/debug/deps/guest_loading_resume-2c9bd859cff3337d)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
Running tests/lazy_pull_continuations.rs (.temp/brief-613/target/debug/deps/lazy_pull_continuations-ef76e1f37b5e402d)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.26s
Running tests/loop_continuations.rs (.temp/brief-613/target/debug/deps/loop_continuations-dbe263d2d0c33a6d)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
Running tests/option_reader_continuations.rs (.temp/brief-613/target/debug/deps/option_reader_continuations-13289bfd2ecc5d28)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.96s
Running tests/rule_pattern_continuations.rs (.temp/brief-613/target/debug/deps/rule_pattern_continuations-487a4fb3b8b94ac7)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s
Running tests/schema_continuations.rs (.temp/brief-613/target/debug/deps/schema_continuations-4182a401e6279d72)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.98s
Running tests/sow_trace_continuations.rs (.temp/brief-613/target/debug/deps/sow_trace_continuations-757402035d2c0050)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
Running tests/suspension_oracle.rs (.temp/brief-613/target/debug/deps/suspension_oracle-58e5a1c5f9b651ff)
test result: ok. 136 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.02s
Running tests/typed_slot_census.rs (.temp/brief-613/target/debug/deps/typed_slot_census-3f7c0defd057db37)
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 233.61s
Running tests/typed_slot_compile_controls.rs (.temp/brief-613/target/debug/deps/typed_slot_compile_controls-d72233bfaf7c8103)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.81s
Running unittests src/lib.rs (.temp/brief-613/target/debug/deps/rum_syntax-08cb56e2fd17a1b7)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.65s
Running unittests src/lib.rs (.temp/brief-613/target/debug/deps/rum_terminal-f8efeb939a755a12)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s
Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.53s
Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.86s
Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Native ZeroMQ skip lines:

```text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
```

Expected forced-red child output:

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.62s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 19.65s
```

The unchanged canonical gates used their authorized fixed outputs under
`.temp/suspension-oracle/`, `.temp/brief-611/r3/census/`,
`.temp/typed-slot-compile-controls/`, `.temp/brief-546/certificates/`,
`target/brief-535/`, `target/brief-540/oracle/` (including falsifiers),
`target/brief-539/falsifiers/`, `target/brief-544/`,
`.temp/brief-542/falsifiers/`, `.temp/brief-543/oracle-reads/`,
`.temp/brief-543/falsifiers/`, and `.temp/brief-546/falsifiers/`.
No canonical writer was edited or redirected to bypass its path. Virtual FileIo
test paths are not represented as actual filesystem writes. Own builds, probes,
reports and measurements remain under brief-613. Commands were serial, offline,
one build job/test thread, bounded, with raw logs preserved before analysis.

### 3. Context, routes, resources, and lifecycle

T = Table, F = definition call, C = input compound, I = InputString. The
snapshot/return traces preserve entry epochs; the global epoch is never rewound.

| Witness | S5 before | Candidate after |
| --- | --- | --- |
| False cycle, Table[f[1],{3}] | [T,F]/depth 2; F survives handoff and the second call is rejected. | [T,F,C,I]/4 at epoch 255; returns [T,F,C]/3, [T,F]/2, [T]/1. Each next input restores the full four-entry context. 3 pauses, 50 events, List[7,7,7]. |
| Changing-expression depth leak | [F]/1 initially; retained F prefixes grow. S5 stops at 1,021 default-limit pauses. | [F,C,I]/5 at every input; returns [F,C]/4, [F]/3, []/2, []/1. Epochs 256…1100 advance with assignments. 1,100 pauses, 23,105 events; depth stays5. |
| Passing direct Table | []/0 loses T although the value happens to pass. | [T,I]/2 at input, [T]/1 at Table handoff, []/0 at completion. 3 pauses, 23 events; 1,100-pause direct control also passes. |

The strict final ledgers include **1,100 and 2,000 actual resumes at the unchanged
default recursion limit, and 100 at limit 32**, with equal queued results. S5's
long changing-expression run terminates at 1,021 (and at 29 with limit 32); those
terminated runs are not speed results. All N/2N runs in the separate supplied
performance panel do complete in both arms.

Routes cover Legacy and Mixed, populated widening, witnessed insertion, fresh
tokens, FrameChain, both deferred Evaluate arms, nested append, Finalize, normal
errors, wrong-slot structural faults, abandonment and cancellation. Existing
frame implementations, chain/token contracts, Session fields, Expr, public
carriers, manifests and eval_core are unchanged. Common insertion endpoints
capture context, so chain/token source needs no edit. Each frame/event restores
its own captured prefix/depth before dispatch; abandonment does the same.

The inline return event restores the parent context and runs the resource
epilogue only for a successful Expr. Errors propagate without a success check;
a wrong successful slot becomes a DriverFault. The test-only memory witness
exceeds its5,000,000-byte limit (about8 MB retained), and the time witness sleeps
150 ms against100 ms, satisfying the1.5× margin. Queued and resumed outcomes are
99, with exactly one actual pause; S5 resumed produced77. No production builtin
was added. Independent occurrence accounting covers shared roots, truncation,
retained capacity, arming/rearming, a second pause, completion, error/fault
cleanup and cancellation. Guest prefix-restore behavior remains covered.

### 4. Allocation, instruction, copy, stack, and timing evidence

The six supplied S5 binaries matched `base-hashes.txt` before work; they were
not rebuilt. Four candidates use the supplied commands/cwd and separate targets,
with one release codegen unit. Build commands, source/binary hashes and raw logs
are in `cand/` and `build-candidate-*.txt`. The copied harness/panel has an input
hash manifest. Both arms pass all **385 preflight observations**: 107 ordinary
check/N/2N triples and 32 resume N/2N pairs with exact actual resume counts.
There are no ineligible performance pairs.

Hardware instructions use CPU2 and exactly
`perf stat -x, -e cpu_core/instructions/u`, stdin `/dev/null`, with the original
N/2N workloads. Slope is `(I(2N)-I(N))/N`. Two interleaved pairs were followed by
five additional samples per arm for nine conservatively flagged ordinary rows.
The threshold is `max(0.05% of base mean, base range, candidate range)`;
the initial screen used maximum paired increase, stricter than a mean screen.
No flag persists, so the persistent-flag six-process escalation is not triggered.

For ordinary Callgrind address/copy attribution, the supplied S5 profiling budget
is retained: `max(1,min(original_N,2000,floor(20M/base_instruction_slope)))`.
The largest original hardware observation exceeds 1.048 trillion instructions;
the bounded profiling sizes are disclosed, separately preflighted and retain
every label. No failed workload was shrunk or removed. Resume Callgrind uses
the original N/2N. Each leaf has three N/2N pairs per arm (six processes).
In total,1,292 hardware-counter processes and580 Callgrind processes completed;
each Callgrind process also has its own successful native-size preflight.

**Ordinary panel, all 107 labels.** `Alloc=B` means both raw allocation counts
and bytes are exactly equal at original N and2N. Instruction columns use the
five-sample mean/range for initially flagged rows and the two-sample results
otherwise. Copy N is the separately disclosed profiling size; `memcpy B=C`
shows both raw N/2N counts (all equal here). Full raw allocation and instruction
tables: [allocation](../../../.temp/brief-613/perf/allocation-comparison.tsv),
[instructions](../../../.temp/brief-613/perf/instruction-comparison.tsv),
[five-sample follow-up](../../../.temp/brief-613/perf/ordinary-five-run-comparison.tsv).

| Label | Original N | Alloc=B | Samples/arm | Base I/iter | Candidate I/iter | ΔI | Noise bar | Copy N | memcpy B=C N/2N |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| lazy-range-take-small | 20000 | B | 2 | 41858.991 | 41857.229 | -1.762 | 21.779 | 477 | 62674/124207 |
| lazy-range-normal-small | 20000 | B | 5 | 53559.3 | 53599.427 | 40.127 | 166.002 | 373 | 54498/107837 |
| lazy-map-realize-small | 20000 | B | 2 | 176366.671 | 176304.048 | -62.624 | 88.183 | 113 | 34685/68020 |
| lazy-select-realize-small | 20000 | B | 2 | 112488.338 | 112431.523 | -56.815 | 205.758 | 177 | 40345/79462 |
| lazy-select-scan-small | 20000 | B | 2 | 793191.833 | 793549.609 | 357.776 | 396.596 | 25 | 36174/69949 |
| lazy-nestlist-realize-small | 20000 | B | 2 | 142953.93 | 142952.685 | -1.245 | 71.477 | 139 | 25825/50428 |
| lazy-foldlist-realize-small | 20000 | B | 2 | 110964.617 | 110797.247 | -167.37 | 360.177 | 180 | 37775/74315 |
| lazy-drop-realize-small | 20000 | B | 2 | 188567.074 | 188635.245 | 68.171 | 187.399 | 106 | 34323/67289 |
| lazy-first-small | 20000 | B | 2 | 40641.023 | 40594.217 | -46.807 | 70.738 | 492 | 30094/59122 |
| lazy-part-small | 20000 | B | 5 | 87099.085 | 87097.851 | -1.234 | 299.512 | 229 | 32523/63896 |
| lazy-transparent-adapters-small | 20000 | B | 2 | 178871.918 | 178592.629 | -279.29 | 664.374 | 111 | 32970/64605 |
| lazy-table-construct-short | 20000 | B | 2 | 23366.187 | 23346.98 | -19.207 | 46.467 | 855 | 43826/86576 |
| lazy-table-construct-full | 20000 | B | 5 | 23851.258 | 23863.121 | 11.863 | 74.166 | 839 | 43878/86667 |
| lazy-table-realize-small | 20000 | B | 2 | 115745.244 | 115735.161 | -10.083 | 57.873 | 172 | 37527/73819 |
| lazy-permutations-take-small | 20000 | B | 2 | 27856.712 | 27858.311 | 1.599 | 13.928 | 717 | 18276/35484 |
| lazy-subsets-take-small | 20000 | B | 2 | 25027.489 | 25010.471 | -17.018 | 33.273 | 799 | 20204/39380 |
| lazy-tuples-take-small | 20000 | B | 2 | 26246.842 | 26270.915 | 24.073 | 30.384 | 761 | 19307/37571 |
| scan-no-input | 20000 | B | 2 | 28966.459 | 28967.828 | 1.369 | 14.483 | 690 | 36235/71425 |
| optionvalue-three-no-input | 20000 | B | 2 | 9491.201 | 9492.086 | 0.885 | 4.746 | 2000 | 15162/29162 |
| replacerepeated-no-input | 20000 | B | 5 | 38997.912 | 38990.346 | -7.566 | 26.303 | 513 | 13894/26719 |
| dot-three-no-input | 20000 | B | 2 | 249975.165 | 249967.957 | -7.209 | 124.988 | 80 | 38718/75918 |
| differences-no-input | 20000 | B | 5 | 9915.842 | 9914.479 | -1.364 | 21.487 | 2000 | 19017/37017 |
| do-count-null | 20000 | B | 2 | 534.96 | 535.054 | 0.094 | 0.267 | 2000 | 2961/4961 |
| do-iterator-null | 20000 | B | 2 | 1152.062 | 1152.008 | -0.053 | 0.576 | 2000 | 4984/8984 |
| table-iterator-null | 20000 | B | 2 | 387.026 | 386.948 | -0.077 | 0.194 | 2000 | 989/989 |
| replaceall-simple | 20000 | B | 2 | 11361.239 | 11335.143 | -26.095 | 52.16 | 1760 | 38092/75052 |
| table-small | 20000 | B | 2 | 14509.497 | 14508.841 | -0.656 | 7.255 | 1378 | 18939/36853 |
| table-iterator-plus | 20000 | B | 2 | 563.501 | 563.464 | -0.037 | 0.286 | 2000 | 3006/5006 |
| table-iterator-pair-list | 20000 | B | 2 | 1640.486 | 1640.54 | 0.054 | 0.82 | 2000 | 1034/1034 |
| take-multispec-small | 20000 | B | 2 | 49714.132 | 49714.296 | 0.164 | 24.857 | 402 | 20091/38985 |
| lazy-depth-map-build-8 | 1000 | B | 2 | 17802.917 | 17806.897 | 3.98 | 21.011 | 1000 | 26827/51827 |
| lazy-depth-map-first-8 | 1000 | B | 2 | 128203.101 | 128099.567 | -103.534 | 198.11 | 156 | 28800/55632 |
| lazy-depth-map-eight-8 | 1000 | B | 2 | 945275.923 | 945293.133 | 17.211 | 472.638 | 21 | 28655/54254 |
| lazy-depth-select-restart-8 | 1000 | B | 2 | 177887.098 | 177777.087 | -110.011 | 278.016 | 112 | 33598/65070 |
| lazy-depth-drop-all-one-8 | 1000 | B | 2 | 46373.533 | 46373.019 | -0.513 | 23.187 | 431 | 68727/135532 |
| lazy-depth-take-map-8 | 1000 | B | 2 | 136825.715 | 136845.295 | 19.579 | 68.413 | 146 | 29682/57130 |
| lazy-depth-fold-through-source-8 | 1000 | B | 2 | 581117.612 | 581159.14 | 41.528 | 290.559 | 34 | 26241/49905 |
| lazy-depth-nest-map-8 | 1000 | B | 2 | 138763.646 | 138674.499 | -89.147 | 206.926 | 144 | 27644/53276 |
| lazy-depth-map-build-64 | 1000 | B | 2 | 29816.067 | 29819.467 | 3.401 | 15.955 | 670 | 23057/39807 |
| lazy-depth-map-first-64 | 1000 | B | 2 | 933200.71 | 934034.927 | 834.218 | 4991.413 | 21 | 31004/54608 |
| lazy-depth-map-eight-64 | 1000 | B | 2 | 7298588.677 | 7298851.001 | 262.325 | 3649.294 | 2 | 32822/50492 |
| lazy-depth-select-restart-64 | 1000 | B | 2 | 971892.205 | 970712.939 | -1179.266 | 8857.999 | 20 | 35746/63766 |
| lazy-depth-drop-all-one-64 | 1000 | B | 2 | 327354.286 | 327357.307 | 3.022 | 163.677 | 61 | 67937/128632 |
| lazy-depth-take-map-64 | 1000 | B | 2 | 993043.809 | 993035.163 | -8.646 | 496.522 | 20 | 34498/59538 |
| lazy-depth-fold-through-source-64 | 1000 | B | 2 | 33527833.483 | 33527354.341 | -479.142 | 16763.917 | 1 | 81798/119347 |
| lazy-depth-nest-map-64 | 1000 | B | 2 | 943116.338 | 943174.334 | 57.996 | 1739.347 | 21 | 31174/54904 |
| lazy-depth-map-build-256 | 1000 | B | 2 | 70439.651 | 70443.394 | 3.743 | 35.22 | 283 | 28742/35817 |
| lazy-depth-map-first-256 | 1000 | B | 2 | 3659415.114 | 3659463.021 | 47.907 | 6844.489 | 5 | 47964/69904 |
| lazy-depth-map-eight-256 | 1000 | B | 2 | 29095122.455 | 29097039.829 | 1917.375 | 14547.561 | 1 | 91569/126518 |
| lazy-depth-select-restart-256 | 1000 | B | 2 | 3652199.318 | 3652731.461 | 532.143 | 38985.065 | 5 | 53131/79336 |
| lazy-depth-drop-all-one-256 | 1000 | B | 2 | 2218613.615 | 2218608.022 | -5.592 | 1109.307 | 9 | 60357/95232 |
| lazy-depth-take-map-256 | 1000 | B | 2 | 3925976.072 | 3925891.163 | -84.909 | 1962.988 | 5 | 58726/83226 |
| lazy-depth-fold-through-source-256 | 1000 | B | 2 | 523339679.646 | 523295052.034 | -44627.612 | 261669.84 | 1 | 1208082/1800513 |
| lazy-depth-nest-map-256 | 1000 | B | 2 | 3674640.437 | 3671302.183 | -3338.254 | 6651.282 | 5 | 48038/70008 |
| literal-integer | 20000 | B | 2 | 292.073 | 292.025 | -0.047 | 0.22 | 2000 | 2942/4942 |
| downvalue-pattern-hit | 20000 | B | 2 | 13647.302 | 13647.033 | -0.269 | 6.824 | 1465 | 26031/50936 |
| module-local | 20000 | B | 2 | 22433.773 | 22433.139 | -0.634 | 11.217 | 891 | 29534/58046 |
| block-local | 20000 | B | 2 | 14462.965 | 14462.911 | -0.053 | 7.231 | 1382 | 39765/78461 |
| matcher-single-wide-small | 1000 | B | 2 | 760080.94 | 760061.016 | -19.924 | 380.04 | 26 | 14257/14647 |
| cases-wide-no-match | 1600 | B | 2 | 2400978.124 | 2401015.603 | 37.479 | 1200.489 | 8 | 1402/1698 |
| count-wide-no-match | 1600 | B | 2 | 2179368.009 | 2179414.401 | 46.392 | 1089.684 | 9 | 1389/1677 |
| position-wide-no-match | 1600 | B | 2 | 2573899.725 | 2573900.144 | 0.418 | 1286.95 | 7 | 1397/1684 |
| deletecases-wide-no-match | 1600 | B | 2 | 2670930.611 | 2670970.816 | 40.205 | 1335.465 | 7 | 1267/1421 |
| deletecases-wide-association-no-match | 1600 | B | 2 | 7501957.183 | 7501653.694 | -303.489 | 3750.979 | 2 | 1272/1334 |
| deletecases-leaf-delete-small | 500 | B | 2 | 63553.291 | 63149.829 | -403.462 | 545.614 | 314 | 4922/8690 |
| replace-wide-no-match | 1600 | B | 2 | 1164274.951 | 1164296.254 | 21.302 | 582.137 | 17 | 1687/2231 |
| replace-leaf-rebuild-small | 500 | B | 2 | 74050.795 | 73524.169 | -526.626 | 1117.886 | 270 | 7124/13064 |
| rewrite-guarded-hit | 500 | B | 2 | 52794.278 | 52768.453 | -25.825 | 117.058 | 378 | 20455/39733 |
| rewrite-guarded-decline-fallback | 500 | B | 2 | 76950.442 | 76861.554 | -88.888 | 278.188 | 259 | 20718/40143 |
| rule-rhs-deep-small | 500 | B | 2 | 82485.893 | 82437.98 | -47.913 | 114.978 | 242 | 19898/38532 |
| part-list | 2000 | B | 2 | 8303.786 | 8302.827 | -0.959 | 4.152 | 2000 | 31126/61126 |
| appendto-part-outer1000-small | 100 | B | 2 | 2217811.64 | 2216567.54 | -1244.1 | 17751.78 | 9 | 29607/55185 |
| appendto-part-outer1000-large | 200 | B | 2 | 3604501.463 | 3610605.21 | 6103.747 | 35200.15 | 5 | 29045/52260 |
| set-part-fixed100-len100 | 100 | B | 5 | 1267229.408 | 1267794.306 | 564.898 | 1157.38 | 15 | 28624/54319 |
| set-part-fixed100-len800 | 100 | B | 2 | 1268058.635 | 1266971.755 | -1086.88 | 634.029 | 15 | 28624/54319 |
| set-part-fill-small | 200 | B | 2 | 2781854.588 | 2783321.43 | 1466.842 | 13560.47 | 7 | 31704/58395 |
| set-part-fill-large | 100 | B | 5 | 5592111.816 | 5589467.58 | -2644.236 | 10144.08 | 3 | 31652/54491 |
| table-iterator-symbol | 2000 | B | 2 | 333.601 | 332.838 | -0.763 | 4.242 | 2000 | 1005/1005 |
| symbol-ownvalue-read | 2000 | B | 2 | 2206.795 | 2207.033 | 0.237 | 2.38 | 2000 | 8992/16992 |
| symbol-ownvalue-guarded-read | 2000 | B | 2 | 3565.349 | 3565.226 | -0.123 | 2.306 | 2000 | 13006/25006 |
| block-symbol-only-repeat | 2000 | B | 2 | 10687.346 | 10685.616 | -1.73 | 5.344 | 1871 | 45875/90779 |
| downvalue-exact-hit | 2000 | B | 2 | 5751.603 | 5751.116 | -0.487 | 2.876 | 2000 | 17023/33023 |
| downvalue-exact-miss | 2000 | B | 2 | 997.754 | 997.939 | 0.185 | 1.707 | 2000 | 5021/9021 |
| downvalue-pattern-miss | 2000 | B | 5 | 997.484 | 998.106 | 0.622 | 2.653 | 2000 | 5044/9044 |
| downvalue-guarded-hit | 2000 | B | 2 | 27569.164 | 27570.306 | 1.142 | 13.785 | 725 | 22830/44580 |
| downvalue-guarded-decline-fallback | 2000 | B | 2 | 20863.846 | 20871.412 | 7.566 | 10.432 | 958 | 19325/37527 |
| upvalue-guarded-hit | 2000 | B | 2 | 33851.32 | 33852.757 | 1.437 | 16.926 | 590 | 21783/42433 |
| subvalue-unguarded-hit | 2000 | B | 2 | 20066.456 | 20067.366 | 0.91 | 10.033 | 996 | 25977/50877 |
| subvalue-unguarded-miss | 2000 | B | 2 | 11281.451 | 11282.394 | 0.943 | 5.641 | 1772 | 31216/61340 |
| subvalue-guarded-hit | 2000 | B | 2 | 35217.114 | 35216.051 | -1.063 | 17.609 | 567 | 22083/43062 |
| default-unguarded-hit | 2000 | B | 2 | 17192.557 | 17192.206 | -0.351 | 8.596 | 1163 | 15021/28977 |
| default-unguarded-miss | 2000 | B | 2 | 12783.072 | 12784.093 | 1.021 | 6.392 | 1564 | 18259/35463 |
| default-guarded-hit | 2000 | B | 2 | 22361.688 | 22361.797 | 0.109 | 11.181 | 894 | 12695/24317 |
| downvalue-exact-miss-repeat | 2000 | B | 5 | 775.493 | 776.544 | 1.051 | 2.607 | 2000 | 5016/9016 |
| downvalue-pattern-miss-repeat | 2000 | B | 2 | 775.784 | 775.146 | -0.638 | 2.862 | 2000 | 5039/9039 |
| upvalue-candidates-tiny | 2000 | B | 2 | 31374.564 | 31372.018 | -2.546 | 28.541 | 637 | 27334/53451 |
| upvalue-candidates-wide-small | 20 | B | 2 | 3063960.2 | 3062261.375 | -1698.825 | 4094.65 | 6 | 39884/49760 |
| upvalue-candidates-wide-large | 20 | B | 2 | 6120979.2 | 6121054.85 | 75.65 | 3060.49 | 3 | 68555/78296 |
| sort-default-wide | 20 | B | 2 | 342457.7 | 342406.675 | -51.025 | 171.229 | 20 | 1297/1537 |
| sortby-default-wide | 20 | B | 2 | 7426115.2 | 7425441.275 | -673.925 | 3713.058 | 2 | 34715/57159 |
| reversesort-default-wide | 20 | B | 2 | 340900.4 | 340919.95 | 19.55 | 170.7 | 20 | 1299/1539 |
| keysort-default-wide | 20 | B | 2 | 8258908.4 | 8260436.575 | 1528.175 | 4129.454 | 2 | 6055/9333 |
| keysortby-default-wide | 20 | B | 2 | 15448981.55 | 15448402.65 | -578.9 | 7724.491 | 1 | 26822/39664 |
| sort-symbol-default-width4000 | 10 | B | 2 | 5885090.95 | 5885043 | -47.95 | 3333.2 | 3 | 193186/193240 |
| sort-symbol-default-width16000 | 10 | B | 2 | 23369660 | 23376477.75 | 6817.75 | 28853.7 | 1 | 769160/769183 |
| valueq-undefined-symbol-definitions-2000 | 20 | B | 2 | 5416.9 | 5451.125 | 34.225 | 1065.35 | 20 | 74709/74849 |
| valueq-undefined-symbol-definitions-8000 | 20 | B | 2 | 10957.825 | 4982.95 | -5974.875 | 10696.75 | 20 | 295467/295607 |

**Resume allocation and instruction slopes.** All rows use the original N/2N.
Rates divide by actual resume counts, including two-prompt iterations. These
are finite N→2N slopes, not isolated-resume constants or an extrapolation beyond
base completion. Allocation reconciliation accounts for every raw count/byte
delta at both sizes; untracked allocations are identical in every pair.

| Label | N | Resumes/iter | Δalloc/resume | Δbytes/resume | Hardware ΔI/resume | Callgrind ΔI/resume | Return events N/2N | Event slope/resume |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| resume-table | 1000 | 1 | 1 | 752 | 1531.126 | 1488.933 | 1001/2001 | 1 |
| resume-do-chain | 1000 | 1 | 0 | 328 | 1449.163 | 1393.61 | 1001/2001 | 1 |
| resume-call-repeat | 1000 | 1 | 0 | 72 | 1105.95 | 1236.368 | 1001/2001 | 1 |
| resume-cleanup | 1000 | 1 | 1 | 720 | 1839.225 | 1743.283 | 1003/2003 | 1 |
| resume-fold | 1000 | 1 | 0 | 328 | 1998.273 | 1964.413 | 2001/4001 | 2 |
| resume-argument-twice | 1000 | 2 | 0.5 | 1192 | 3619.306 | 3408.297 | 5002/10002 | 2.5 |
| resume-rewrite-exprs | 1000 | 1 | 0 | 840 | 3540.031 | 3445.088 | 3001/6001 | 3 |
| resume-rewrite-compiled | 1000 | 1 | 1 | 1608 | 3746.524 | 3789.697 | 3001/6001 | 3 |
| resume-cases-spec | 1000 | 1 | 0 | 880 | 5042.938 | 4981.171 | 4001/8001 | 4 |
| resume-cases-traversal | 1000 | 1 | 0 | 880 | 3713.39 | 4063.362 | 3001/6001 | 3 |
| resume-position-traversal | 1000 | 1 | 0 | 880 | 3863.076 | 3893.945 | 3001/6001 | 3 |
| resume-count-traversal | 1000 | 1 | 0 | 880 | 3757.88 | 3664.972 | 3001/6001 | 3 |
| resume-cases-shared | 1000 | 2 | 0 | 880 | 4000.605 | 4004.42 | 6001/12001 | 3 |
| resume-s5-down-rhs | 100 | 1 | -1 | -16900 | -5161.395 | -5952.09 | 402/802 | 4 |
| resume-s5-down-guard | 100 | 2 | 0 | -15372 | -4381.018 | -4981.14 | 602/1202 | 3 |
| resume-s5-sub-rhs | 100 | 1 | -1 | -16900 | -6125.125 | -6666.11 | 402/802 | 4 |
| resume-s5-sub-guard | 100 | 2 | 0 | -15372 | -4763.22 | -5270.84 | 602/1202 | 3 |
| resume-s5-up-rhs | 100 | 1 | -1 | -16900 | -6187.115 | -6800.75 | 402/802 | 4 |
| resume-s5-up-guard | 100 | 2 | 0 | -15372 | -4705.083 | -5369.785 | 602/1202 | 3 |
| resume-s5-default-rhs | 100 | 1 | 0 | -32280 | -15305.39 | -27785.29 | 402/802 | 4 |
| resume-s5-default-guard | 100 | 2 | -0.5 | -16664 | -4784.592 | -5360.65 | 602/1202 | 3 |
| resume-s5-default-fallback-rhs | 100 | 1 | -1 | -16900 | -5242.08 | -5480.38 | 402/802 | 4 |
| resume-s5-set | 100 | 1 | 0 | 880 | 4740.52 | 4972.87 | 401/801 | 4 |
| resume-s5-upset | 100 | 1 | 0 | 880 | 4626.59 | 4861.16 | 401/801 | 4 |
| resume-s5-tagset | 100 | 1 | 0 | 880 | 4626.505 | 4689.47 | 401/801 | 4 |
| resume-s5-rule-rhs | 100 | 1 | 0 | 880 | 5617.39 | 5643.37 | 501/1001 | 5 |
| resume-s5-rule-lhs | 100 | 1 | 0 | 880 | 6118.13 | 5848.92 | 501/1001 | 5 |
| resume-s5-destructure | 100 | 1 | 1 | 3440 | 5673.465 | 5942.3 | 402/802 | 4 |
| resume-s5-lhs-twice | 100 | 2 | 0.5 | 1204 | 3768.865 | 3652.855 | 601/1201 | 3 |
| resume-s5-own-rhs | 100 | 1 | -1 | -34840 | -19953.53 | -32611.31 | 402/802 | 4 |
| resume-s5-destructure-twice | 100 | 2 | 0.5 | 2200 | 4410.622 | 4569.35 | 602/1202 | 3 |
| resume-s5-rule-lhs-twice | 100 | 2 | 0.5 | 1156 | 4475.218 | 4642.135 | 701/1401 | 3.5 |

Queue cells are base Legacy16/Mixed24 bytes, candidate Queued<Arc>32 and
Queued<Entry>40 bytes. The context component is exactly 16 times the observed
candidate allocated capacity units. The remaining queue delta is the changed
event/widen capacity at old cell sizes:16×ΔLegacy capacity+24×ΔMixed capacity.
The additional full-snapshot component is 40×min(2,observed tracked length) per
pause; corrected-prefix changes and avoided active-stack growth are separate.
An event is an inline enum payload, with no Arc or independent allocation.
The source, independent model, scoped-observer controls and full delta
reconciliation support that statement; the direct observer does not dynamically
watch every allocation within every event.

Allocation byte components below are per iteration (divide by Resumes/iter
above to obtain the per-actual-resume rate). They sum to each measured delta.

| Label | Context cells | Event/widen capacity | Full snapshot extra | Snapshot prefix change | Active-stack growth Δ | Total bytes Δ |
| --- | --- | --- | --- | --- | --- | --- |
| resume-table | 448 | 384 | 80 | 0 | -160 | 752 |
| resume-do-chain | 448 | 0 | 40 | 0 | -160 | 328 |
| resume-call-repeat | 192 | 0 | 40 | 0 | -160 | 72 |
| resume-cleanup | 512 | 288 | 80 | 0 | -160 | 720 |
| resume-fold | 448 | 0 | 40 | 0 | -160 | 328 |
| resume-argument-twice | 1920 | 624 | 160 | 0 | -320 | 2384 |
| resume-rewrite-exprs | 960 | 0 | 40 | 0 | -160 | 840 |
| resume-rewrite-compiled | 960 | 768 | 40 | 0 | -160 | 1608 |
| resume-cases-spec | 960 | 0 | 80 | 0 | -160 | 880 |
| resume-cases-traversal | 960 | 0 | 80 | 0 | -160 | 880 |
| resume-position-traversal | 960 | 0 | 80 | 0 | -160 | 880 |
| resume-count-traversal | 960 | 0 | 80 | 0 | -160 | 880 |
| resume-cases-shared | 1920 | 0 | 160 | 0 | -320 | 1760 |
| resume-s5-down-rhs | 960 | 0 | 80 | -5980 | -11960 | -16900 |
| resume-s5-down-guard | 2944 | 2112 | 160 | -11960 | -24000 | -30744 |
| resume-s5-sub-rhs | 960 | 0 | 80 | -5980 | -11960 | -16900 |
| resume-s5-sub-guard | 2944 | 2112 | 160 | -11960 | -24000 | -30744 |
| resume-s5-up-rhs | 960 | 0 | 80 | -5980 | -11960 | -16900 |
| resume-s5-up-guard | 2944 | 2112 | 160 | -11960 | -24000 | -30744 |
| resume-s5-default-rhs | 1984 | 1536 | 80 | -11960 | -23920 | -32280 |
| resume-s5-default-guard | 1920 | 552 | 160 | -11960 | -24000 | -33328 |
| resume-s5-default-fallback-rhs | 960 | 0 | 80 | -5980 | -11960 | -16900 |
| resume-s5-set | 960 | 0 | 80 | 0 | -160 | 880 |
| resume-s5-upset | 960 | 0 | 80 | 0 | -160 | 880 |
| resume-s5-tagset | 960 | 0 | 80 | 0 | -160 | 880 |
| resume-s5-rule-rhs | 960 | 0 | 80 | 0 | -160 | 880 |
| resume-s5-rule-lhs | 960 | 0 | 80 | 0 | -160 | 880 |
| resume-s5-destructure | 1984 | 1536 | 80 | 0 | -160 | 3440 |
| resume-s5-lhs-twice | 1920 | 648 | 160 | 0 | -320 | 2408 |
| resume-s5-own-rhs | 960 | 0 | 80 | -11960 | -23920 | -34840 |
| resume-s5-destructure-twice | 2976 | 1584 | 160 | 0 | -320 | 4400 |
| resume-s5-rule-lhs-twice | 1920 | 552 | 160 | 0 | -320 | 2312 |

For example, Table's +752 bytes/resume is 448 context+384 event/capacity+80
full snapshot−160 avoided active growth. The full count/byte intercepts and raw
N/2N values are in [allocation reconciliation](../../../.temp/brief-613/allocation-attribution/allocation-reconciliation.tsv)
and [raw measured allocations](../../../.temp/brief-613/perf/allocation-comparison.tsv).

Native GDB startup was denied by sandbox ptrace restrictions; no escalation was
attempted. The fallback was Valgrind tool=none with its local-pipe debugger relay
and explicit `--max-invoke-ms=0` (ptrace disabled), with FIFOs/caches confined to
brief-613 and network/debuginfod disabled. One full scoped base-Table2N observer
exceeded its800s debugger bound; the incomplete raw log is retained. A direct
observer then completed all 128 runs and matched unobserved native counters and
actual resume counts exactly. It reads stack lengths, actual requested allocation
byte arguments and the widening branch; snapshot bytes are derived from observed
lengths and the verified40-byte layout. Three direct/scoped controls agree.
No production instrumentation or performance binary was modified.
[observer controls](../../../.temp/brief-613/allocation-attribution/direct-observer-controls.json);
[per-run observations](../../../.temp/brief-613/allocation-attribution/per-run-summary.json).

Ten S5 rows contain a quadratic old root-buffer term in the completed workloads.
The interior allocation sequences, with at most nine endpoint iterations, reproduce
both measured N/2N deltas exactly; these are structural allocation models, not
elapsed-time fits or unrestricted asymptotic speed claims. All 32 models are in
[root cost proof](../../../.temp/brief-613/allocation-attribution/root-cost-proof.tsv).

| Label | Old root bytes N² coefficient | Candidate coefficient | Total allocation Δ bytes over measured range |
| --- | --- | --- | --- |
| resume-s5-down-rhs | 60 | 0 | -60N² + 1100N + -400 |
| resume-s5-down-guard | 120 | 0 | -120N² + 5256N + -480 |
| resume-s5-sub-rhs | 60 | 0 | -60N² + 1100N + -400 |
| resume-s5-sub-guard | 120 | 0 | -120N² + 5256N + -480 |
| resume-s5-up-rhs | 60 | 0 | -60N² + 1100N + -400 |
| resume-s5-up-guard | 120 | 0 | -120N² + 5256N + -480 |
| resume-s5-default-rhs | 120 | 0 | -120N² + 3720N + -320 |
| resume-s5-default-guard | 120 | 0 | -120N² + 2672N + -480 |
| resume-s5-default-fallback-rhs | 60 | 0 | -60N² + 1100N + -400 |
| resume-s5-own-rhs | 120 | 0 | -120N² + 1160N + -320 |

**Widening, every workload.** Both arms widen exactly one empty owner per actual
resume. All populated-owner counts, moved cells, read bytes, write bytes and
widen destination allocation bytes are zero in this panel. This does not assert
that populated widening is free; populated correctness/mutation controls are
separate. The table retains each workload's actual N/2N owner counts, identical
in both arms; the move column uses the actual candidate32→40-byte cell sizes.

| Label | Actual resumes N/2N | Owners widened base N/2N | Owners widened candidate N/2N | Populated / moved cells | Move read/write bytes |
| --- | --- | --- | --- | --- | --- |
| resume-table | 1000/2000 | 1000/2000 | 1000/2000 | 0 / 0 | 0 / 0 |
| resume-do-chain | 1000/2000 | 1000/2000 | 1000/2000 | 0 / 0 | 0 / 0 |
| resume-call-repeat | 1000/2000 | 1000/2000 | 1000/2000 | 0 / 0 | 0 / 0 |
| resume-cleanup | 1000/2000 | 1000/2000 | 1000/2000 | 0 / 0 | 0 / 0 |
| resume-fold | 1000/2000 | 1000/2000 | 1000/2000 | 0 / 0 | 0 / 0 |
| resume-argument-twice | 2000/4000 | 2000/4000 | 2000/4000 | 0 / 0 | 0 / 0 |
| resume-rewrite-exprs | 1000/2000 | 1000/2000 | 1000/2000 | 0 / 0 | 0 / 0 |
| resume-rewrite-compiled | 1000/2000 | 1000/2000 | 1000/2000 | 0 / 0 | 0 / 0 |
| resume-cases-spec | 1000/2000 | 1000/2000 | 1000/2000 | 0 / 0 | 0 / 0 |
| resume-cases-traversal | 1000/2000 | 1000/2000 | 1000/2000 | 0 / 0 | 0 / 0 |
| resume-position-traversal | 1000/2000 | 1000/2000 | 1000/2000 | 0 / 0 | 0 / 0 |
| resume-count-traversal | 1000/2000 | 1000/2000 | 1000/2000 | 0 / 0 | 0 / 0 |
| resume-cases-shared | 2000/4000 | 2000/4000 | 2000/4000 | 0 / 0 | 0 / 0 |
| resume-s5-down-rhs | 100/200 | 100/200 | 100/200 | 0 / 0 | 0 / 0 |
| resume-s5-down-guard | 200/400 | 200/400 | 200/400 | 0 / 0 | 0 / 0 |
| resume-s5-sub-rhs | 100/200 | 100/200 | 100/200 | 0 / 0 | 0 / 0 |
| resume-s5-sub-guard | 200/400 | 200/400 | 200/400 | 0 / 0 | 0 / 0 |
| resume-s5-up-rhs | 100/200 | 100/200 | 100/200 | 0 / 0 | 0 / 0 |
| resume-s5-up-guard | 200/400 | 200/400 | 200/400 | 0 / 0 | 0 / 0 |
| resume-s5-default-rhs | 100/200 | 100/200 | 100/200 | 0 / 0 | 0 / 0 |
| resume-s5-default-guard | 200/400 | 200/400 | 200/400 | 0 / 0 | 0 / 0 |
| resume-s5-default-fallback-rhs | 100/200 | 100/200 | 100/200 | 0 / 0 | 0 / 0 |
| resume-s5-set | 100/200 | 100/200 | 100/200 | 0 / 0 | 0 / 0 |
| resume-s5-upset | 100/200 | 100/200 | 100/200 | 0 / 0 | 0 / 0 |
| resume-s5-tagset | 100/200 | 100/200 | 100/200 | 0 / 0 | 0 / 0 |
| resume-s5-rule-rhs | 100/200 | 100/200 | 100/200 | 0 / 0 | 0 / 0 |
| resume-s5-rule-lhs | 100/200 | 100/200 | 100/200 | 0 / 0 | 0 / 0 |
| resume-s5-destructure | 100/200 | 100/200 | 100/200 | 0 / 0 | 0 / 0 |
| resume-s5-lhs-twice | 200/400 | 200/400 | 200/400 | 0 / 0 | 0 / 0 |
| resume-s5-own-rhs | 100/200 | 100/200 | 100/200 | 0 / 0 | 0 / 0 |
| resume-s5-destructure-twice | 200/400 | 200/400 | 200/400 | 0 / 0 | 0 / 0 |
| resume-s5-rule-lhs-twice | 200/400 | 200/400 | 200/400 | 0 / 0 | 0 / 0 |

**Resume instruction attribution.** All nonzero addresses resolve to disassembly.
[Per-function self instructions and call rates](../../../.temp/brief-613/attribution/resume-functions.tsv)
records base/candidate rates and instructions per call; each workload also has
complete address/call-site TSVs under `attribution/resume/`. The grouped table
below sums every Callgrind function delta, with zero arithmetic residual. The
last column is a ledger of named functions, not omitted work. Hardware and
Callgrind totals are different measurements; no one-for-one reconciliation of
their totals is claimed. One profiling pair does not establish repeatability of
allocator, symbol-interning, or hash-seed effects. Those named effects are retained.

| Label | Resume driver ΔI/resume | Context/return/native poll | Queue/transport | Allocator/system library | Other named functions | Sum residual |
| --- | --- | --- | --- | --- | --- | --- |
| resume-table | 551 | 312 | 188 | 160.807 | 277.126 | 0 |
| resume-do-chain | 626 | 359 | 187 | 21.324 | 200.286 | 0 |
| resume-call-repeat | 519 | 312 | 202 | -122.692 | 326.06 | 0 |
| resume-cleanup | 733 | 312 | 188 | 267.964 | 242.319 | 0 |
| resume-fold | 929 | 577 | 307 | -66.368 | 217.781 | 0 |
| resume-argument-twice | 1486 | 827 | 504 | 217.007 | 374.29 | 0 |
| resume-rewrite-exprs | 1574 | 983 | 612 | 8.11 | 267.978 | 0 |
| resume-rewrite-compiled | 1496 | 936 | 580 | 449.892 | 327.805 | 0 |
| resume-cases-spec | 2124 | 1342 | 814 | 163.312 | 537.859 | 0 |
| resume-cases-traversal | 1653 | 1030 | 635 | 46.057 | 699.305 | 0 |
| resume-position-traversal | 1653 | 1030 | 635 | 138.079 | 437.866 | 0 |
| resume-count-traversal | 1653 | 1030 | 635 | 9.268 | 337.704 | 0 |
| resume-cases-shared | 1698.5 | 1030 | 630.5 | 164.038 | 481.383 | 0 |
| resume-s5-down-rhs | -2028.5 | 1253 | 719 | -921.96 | -4973.63 | 0 |
| resume-s5-down-guard | -2216 | 1056 | 613.5 | -559.41 | -3875.23 | 0 |
| resume-s5-sub-rhs | -2028.5 | 1253 | 719 | -924.75 | -5684.86 | 0 |
| resume-s5-sub-guard | -2216 | 1056 | 613.5 | -542.82 | -4181.52 | 0 |
| resume-s5-up-rhs | -2028.5 | 1253 | 719 | -1003.27 | -5740.98 | 0 |
| resume-s5-up-guard | -2214 | 1056 | 613.5 | -546 | -4279.285 | 0 |
| resume-s5-default-rhs | -5781 | 1436 | 841 | -12696.33 | -11584.96 | 0 |
| resume-s5-default-guard | -2324 | 983 | 594 | -747.075 | -3866.575 | 0 |
| resume-s5-default-fallback-rhs | -1950.5 | 1295 | 768 | -898.99 | -4693.89 | 0 |
| resume-s5-set | 2042 | 1295 | 768 | 109.19 | 758.68 | 0 |
| resume-s5-upset | 2042 | 1295 | 768 | 43.43 | 712.73 | 0 |
| resume-s5-tagset | 2042 | 1295 | 768 | 43.96 | 540.51 | 0 |
| resume-s5-rule-rhs | 2442 | 1565 | 927 | 87.23 | 622.14 | 0 |
| resume-s5-rule-lhs | 2520 | 1607 | 980 | 226.15 | 515.77 | 0 |
| resume-s5-destructure | 2365 | 1436 | 870 | 581.36 | 689.94 | 0 |
| resume-s5-lhs-twice | 1629 | 959.5 | 584.5 | 214.49 | 265.365 | 0 |
| resume-s5-own-rhs | -5116.5 | 1201 | 676 | -12924.54 | -16447.27 | 0 |
| resume-s5-destructure-twice | 1952 | 1100.5 | 668.5 | 405.055 | 443.295 | 0 |
| resume-s5-rule-lhs-twice | 1913 | 1115.5 | 690.5 | 359.485 | 563.65 | 0 |

Event insertion self cost ranges from 44 to 45.25 instructions per event in
these profiles; completion self cost is 62. The full function ledger separates those
from native polling, accounted truncation, queue insertion/widening/append,
snapshot clone/drop, the resume driver and allocator/copy work. PLT entries
initially reported as unknown addresses are bound by direct call targets and
ELF-relative offsets in `attribution/plt-bindings.json`.

**Copies and leaves.** The resolved `memcpy` and separate `memmove` checks are
107/107 at N,2N and slope. The shared libc IFUNC target alone cannot distinguish
the two names. Final attribution uses GOT/PLT calls, 66 cached-register call sites
(including the explicit register transfer), and libc IRELATIVE
`0x24ff00`→resolver`0xc2ce0`→`memcpy`. Initial incomplete analysis logs are
retained. [Resolved copy table](../../../.temp/brief-613/ordinary-callgrind/copy-gate-resolved.tsv),
[every copy caller and proof](../../../.temp/brief-613/ordinary-callgrind/copy-callers-resolved.tsv),
[cached-register bindings](../../../.temp/brief-613/attribution/copy-register-bindings.json).

Each leaf has six Callgrind processes per arm. Every aligned evaluator address
has identical counts: 110 instructions/iteration in all samples. There are zero
positive deterministic function deltas. Whole-process equality is **not** claimed;
startup/interner/hashbrown/libc-comparison variation remains, including some
function means outside one standard error. The complete changed-address ledger
and those residuals are in [leaf proof](../../../.temp/brief-613/ordinary-callgrind/leaf-proof.json).

| Leaf | Profiling N | Base whole-process slopes | Candidate whole-process slopes | Evaluator both arms | Positive deterministic function growth |
| --- | --- | --- | --- | --- | --- |
| literal-integer | 2000 | 292.1475, 291.3725, 290.988 | 292.1355, 293.5135, 292.6155 | 110, 110, 110 | 0 |
| do-count-null | 2000 | 535.577, 535.3485, 536.4735 | 534.563, 534.6235, 534.0285 | 110, 110, 110 | 0 |
| do-iterator-null | 2000 | 1153.52, 1151.6245, 1153.8255 | 1152.5845, 1153.5995, 1151.4685 | 110, 110, 110 | 0 |

**Native stack, full table.** The supplied S5 probe is appended byte-for-byte
only to the scratch runtime test file. Frozen base binaries were copied, never
rebuilt. Candidate builds use the matching debug/release settings (release cgu1).
All 240 runs pass:five shapes×four heights×three samples×two profiles×two arms.
The1 MiB native budget and the probe's unchanged high logical limit isolate
native usage; the semantic long-run tests retain their original/default limits.
Each cell below lists all three measured bytes. All 120 paired deltas are exactly
zero, including constant offsets, not just per-level slopes. No StackLimit or
aborting overflow probe occurred.

| Profile | Shape | Height | S5 bytes, samples1/2/3 | Candidate bytes, samples1/2/3 | Paired Δ |
| --- | --- | --- | --- | --- | --- |
| debug | peel | 2 | 75704, 75704, 75704 | 75704, 75704, 75704 | 0, 0, 0 |
| debug | peel | 4 | 113752, 113752, 113752 | 113752, 113752, 113752 | 0, 0, 0 |
| debug | peel | 6 | 151800, 151800, 151800 | 151800, 151800, 151800 | 0, 0, 0 |
| debug | peel | 8 | 189848, 189848, 189848 | 189848, 189848, 189848 | 0, 0, 0 |
| debug | plus | 2 | 103528, 103528, 103528 | 103528, 103528, 103528 | 0, 0, 0 |
| debug | plus | 4 | 173352, 173352, 173352 | 173352, 173352, 173352 | 0, 0, 0 |
| debug | plus | 6 | 243176, 243176, 243176 | 243176, 243176, 243176 | 0, 0, 0 |
| debug | plus | 8 | 313000, 313000, 313000 | 313000, 313000, 313000 | 0, 0, 0 |
| debug | sub | 2 | 90872, 90872, 90872 | 90872, 90872, 90872 | 0, 0, 0 |
| debug | sub | 4 | 149624, 149624, 149624 | 149624, 149624, 149624 | 0, 0, 0 |
| debug | sub | 6 | 208376, 208376, 208376 | 208376, 208376, 208376 | 0, 0, 0 |
| debug | sub | 8 | 267128, 267128, 267128 | 267128, 267128, 267128 | 0, 0, 0 |
| debug | up | 2 | 86936, 86936, 86936 | 86936, 86936, 86936 | 0, 0, 0 |
| debug | up | 4 | 143064, 143064, 143064 | 143064, 143064, 143064 | 0, 0, 0 |
| debug | up | 6 | 199192, 199192, 199192 | 199192, 199192, 199192 | 0, 0, 0 |
| debug | up | 8 | 255320, 255320, 255320 | 255320, 255320, 255320 | 0, 0, 0 |
| debug | guarded | 2 | 105384, 105384, 105384 | 105384, 105384, 105384 | 0, 0, 0 |
| debug | guarded | 4 | 177064, 177064, 177064 | 177064, 177064, 177064 | 0, 0, 0 |
| debug | guarded | 6 | 248744, 248744, 248744 | 248744, 248744, 248744 | 0, 0, 0 |
| debug | guarded | 8 | 320424, 320424, 320424 | 320424, 320424, 320424 | 0, 0, 0 |
| release | peel | 5 | 49465, 49465, 49465 | 49465, 49465, 49465 | 0, 0, 0 |
| release | peel | 10 | 86185, 86185, 86185 | 86185, 86185, 86185 | 0, 0, 0 |
| release | peel | 15 | 122905, 122905, 122905 | 122905, 122905, 122905 | 0, 0, 0 |
| release | peel | 20 | 159625, 159625, 159625 | 159625, 159625, 159625 | 0, 0, 0 |
| release | plus | 5 | 80409, 80409, 80409 | 80409, 80409, 80409 | 0, 0, 0 |
| release | plus | 10 | 149369, 149369, 149369 | 149369, 149369, 149369 | 0, 0, 0 |
| release | plus | 15 | 218329, 218329, 218329 | 218329, 218329, 218329 | 0, 0, 0 |
| release | plus | 20 | 287289, 287289, 287289 | 287289, 287289, 287289 | 0, 0, 0 |
| release | sub | 5 | 77961, 77961, 77961 | 77961, 77961, 77961 | 0, 0, 0 |
| release | sub | 10 | 143801, 143801, 143801 | 143801, 143801, 143801 | 0, 0, 0 |
| release | sub | 15 | 209641, 209641, 209641 | 209641, 209641, 209641 | 0, 0, 0 |
| release | sub | 20 | 275481, 275481, 275481 | 275481, 275481, 275481 | 0, 0, 0 |
| release | up | 5 | 67689, 67689, 67689 | 67689, 67689, 67689 | 0, 0, 0 |
| release | up | 10 | 124969, 124969, 124969 | 124969, 124969, 124969 | 0, 0, 0 |
| release | up | 15 | 182249, 182249, 182249 | 182249, 182249, 182249 | 0, 0, 0 |
| release | up | 20 | 239529, 239529, 239529 | 239529, 239529, 239529 | 0, 0, 0 |
| release | guarded | 5 | 82649, 82649, 82649 | 82649, 82649, 82649 | 0, 0, 0 |
| release | guarded | 10 | 153849, 153849, 153849 | 153849, 153849, 153849 | 0, 0, 0 |
| release | guarded | 15 | 225049, 225049, 225049 | 225049, 225049, 225049 | 0, 0, 0 |
| release | guarded | 20 | 296249, 296249, 296249 | 296249, 296249, 296249 | 0, 0, 0 |

| Profile | Shape | Base bytes/level | Candidate bytes/level | Δ | Maximum fit residual |
| --- | --- | --- | --- | --- | --- |
| debug | peel | 19024 | 19024 | 0 | 0 |
| debug | plus | 34912 | 34912 | 0 | 0 |
| debug | sub | 29376 | 29376 | 0 | 0 |
| debug | up | 28064 | 28064 | 0 | 0 |
| debug | guarded | 35840 | 35840 | 0 | 0 |
| release | peel | 7344 | 7344 | 0 | 0 |
| release | plus | 13792 | 13792 | 0 | 0 |
| release | sub | 13168 | 13168 | 0 | 0 |
| release | up | 11456 | 11456 | 0 | 0 |
| release | guarded | 14240 | 14240 | 0 | 0 |

**Changed resident prologues: none.** The recursive segment audit and the full
runtime backtrace audit both find identical prologue sizes/counts by function.
There is no positive resident closure-slot delta to attribute. Raw marker logs,
backtraces and disassemblies are under `stack/{base,candidate}/native/{debug,release}/`;
[per-function resident delta table](../../../.temp/brief-613/stack/frame-deltas.tsv) and
[full-backtrace audit](../../../.temp/brief-613/stack/full-resident-changes.json) record the result.

The broader touched-family prologue inventory has 2,411 rows,29 increases and61
unmatched/new helper or generic rows. These are distinct from the measured
resident recursion stack. All 29 increased cold prologues are disclosed below,
including the release resume-driver 1160→1480 and outer resume 1384→1400 bytes.
They remain cold stack costs for the director's ruling. This table measures
function prologues; it does not claim a new per-slot proof of the cold frames.
The complete matched/unmatched inventory is in
[all touched-family prologues](../../../.temp/brief-613/stack/native-changed-prologues.tsv).

| Profile | Cold function | Base prologue bytes | Candidate prologue bytes | Δ |
| --- | --- | --- | --- | --- |
| debug | rum_runtime::runtime::suspension::core::&lt;impl rum_runtime::runtime::model::session::Session&gt;::drive_resuming_frames | 4488 | 6872 | 2384 |
| debug | rum_runtime::runtime::suspension::core::&lt;impl rum_runtime::runtime::model::session::Session&gt;::drive_resuming_frames::{{closure}} | 24 | 296 | 272 |
| debug | rum_runtime::runtime::suspension::core::&lt;impl rum_runtime::runtime::model::session::Session&gt;::insert_suspension_frame | 808 | 856 | 48 |
| debug | rum_runtime::runtime::suspension::core::&lt;impl rum_runtime::runtime::model::session::Session&gt;::insert_typed_erased_frame | 88 | 136 | 48 |
| debug | rum_runtime::runtime::suspension::core::SuspendedEvaluation::front | 8 | 24 | 16 |
| debug | rum_runtime::runtime::suspension::core::SuspendedEvaluation::pop_front | 56 | 72 | 16 |
| debug | rum_runtime::runtime::suspension::core::SuspendedEvaluation::push_back | 24 | 40 | 16 |
| debug | rum_runtime::runtime::suspension::core::SuspendedEvaluation::push_front | 24 | 40 | 16 |
| debug | rum_runtime::runtime::suspension::core::abandon::&lt;impl rum_runtime::runtime::model::session::Session&gt;::drain_abandonment_worklist | 1384 | 1432 | 48 |
| debug | rum_runtime::runtime::suspension::core::occurrence_memory::&lt;impl rum_runtime::runtime::suspension::core::SuspendedEvaluation&gt;::occurrence_bytes::{{closure}} | 280 | 296 | 16 |
| debug | rum_runtime::runtime::suspension::core::queue::FrameQueue::pop_front | 56 | 88 | 32 |
| debug | rum_runtime::runtime::suspension::core::queue::FrameQueue::pop_front_legacy | 56 | 120 | 64 |
| debug | rum_runtime::runtime::suspension::core::queue::FrameQueue::pop_front_mixed_legacy | 168 | 200 | 32 |
| debug | rum_runtime::runtime::suspension::core::queue::FrameQueue::push_back | 88 | 200 | 112 |
| debug | rum_runtime::runtime::suspension::core::queue::FrameQueue::push_back_typed | 88 | 168 | 80 |
| debug | rum_runtime::runtime::suspension::core::queue::FrameQueue::push_front | 88 | 200 | 112 |
| debug | rum_runtime::runtime::suspension::core::queue::FrameQueue::push_front_typed | 88 | 168 | 80 |
| debug | rum_runtime::runtime::suspension::typed::action::TypedAction&lt;O&gt;::finish_work | 376 | 408 | 32 |
| debug | rum_runtime::runtime::suspension::typed::chain::start | 168 | 200 | 32 |
| debug | rum_runtime::runtime::suspension::typed::entry::Entry::abandon | 72 | 88 | 16 |
| release | rum_runtime::runtime::session::session_requests::&lt;impl rum_runtime::runtime::model::session::Session&gt;::resume_eval_request_from_checkpoint | 1384 | 1400 | 16 |
| release | rum_runtime::runtime::suspension::core::&lt;impl rum_runtime::runtime::model::session::Session&gt;::drive_resuming_frames | 1160 | 1480 | 320 |
| release | rum_runtime::runtime::suspension::core::&lt;impl rum_runtime::runtime::model::session::Session&gt;::insert_suspension_frame | 760 | 776 | 16 |
| release | rum_runtime::runtime::suspension::core::&lt;impl rum_runtime::runtime::model::session::Session&gt;::insert_typed_erased_frame | 88 | 120 | 32 |
| release | rum_runtime::runtime::suspension::core::abandon::&lt;impl rum_runtime::runtime::model::session::Session&gt;::drain_abandonment_worklist | 760 | 808 | 48 |
| release | rum_runtime::runtime::suspension::core::queue::FrameQueue::append_mixed | 136 | 184 | 48 |
| release | rum_runtime::runtime::suspension::core::queue::FrameQueue::pop_front_mixed_legacy | 104 | 120 | 16 |
| release | rum_runtime::runtime::suspension::core::queue::FrameQueue::push_front_typed | 72 | 104 | 32 |
| release | rum_runtime::runtime::suspension::core::queue::FrameQueue::widen | 104 | 200 | 96 |

**Timing, last, indication only.** The unchanged canonical
`scripts/core-language-microbench.sh` ran four times in B,C,B,C order on CPU2,
profiling off, scale1, reps2, retained-module GC batches3. It emits a different
120-label panel: 61 labels overlap the fixed 107-label allocation/instruction panel,
59 occur only in timing and 46 only in that counter panel. Durations are
in-kernel AbsoluteTiming from monotonic Instant; no wall-clock differences are
used. No other agent workload ran concurrently. Each arm had five quiet
preflight samples and one measurement sample; no competing visible process or
CPU2 activity above the 0.15-core monitor threshold was observed. The maximum
unattributed CPU2 rate was below 0.01 core. GPU telemetry was unavailable; the
process namespace and sampling limit visibility, so this is not a claim of
complete host isolation.

The second A/B pair was run because the first had a material association-read
increase. The first/second pairs have 47/52 higher candidate rows;30 labels are
higher in both, 18 have disjoint observed higher candidate ranges. There are no
further timing retries. Width 32,000 association reads are 8.02826/7.93227 ms on
base versus 10.0302/9.25341 ms on candidate:mean+20.8206%,+1.66154 ms. Width 4,000
is+11.8322%; large incremental AppendTo is+4.8429%. These residuals, including
same-family neighbors, are reported rather than declared harmless. Their
physical cause is unresolved; counter-panel and stack results do not establish
timing equality, and the association-key labels are outside that fixed 107 panel.
Full 120-label table follows (milliseconds; positive changes are retained).

| Label | Base ms pair1/2 | Candidate ms pair1/2 | Mean Δms | Mean % change | Higher both |
| --- | --- | --- | --- | --- | --- |
| association-part-key-read-width4000 | 1.30647 / 1.03781 | 1.41796 / 1.2037 | 0.13869 | 11.8322 | yes |
| association-part-key-read-width32000 | 8.02826 / 7.93227 | 10.0302 / 9.25341 | 1.66154 | 20.8206 | yes |
| association-part-positional-read-width4000 | 0.030545 / 0.030595 | 0.031088 / 0.030833 | 0.00039 | 1.2774 | yes |
| association-part-positional-read-width32000 | 0.030983 / 0.031147 | 0.031806 / 0.030659 | 0.000167 | 0.5392 | no |
| loop-empty | 0.265656 / 0.271224 | 0.26167 / 0.267658 | -0.003776 | -1.4066 | no |
| do-count-null | 0.245961 / 0.246315 | 0.254892 / 0.24618 | 0.004398 | 1.7868 | no |
| do-iterator-null | 0.060764 / 0.054874 | 0.05747 / 0.057999 | -0.000085 | -0.1461 | no |
| table-count-null | 0.070856 / 0.067768 | 0.07877 / 0.071442 | 0.005794 | 8.3593 | yes |
| table-iterator-null | 0.017291 / 0.0174 | 0.019619 / 0.018202 | 0.001565 | 9.0225 | yes |
| table-iterator-symbol | 0.007018 / 0.006425 | 0.008367 / 0.006482 | 0.000703 | 10.459 | yes |
| literal-integer | 0.0932 / 0.090161 | 0.094456 / 0.094477 | 0.002786 | 3.0388 | yes |
| literal-string | 0.097052 / 0.094765 | 0.091857 / 0.091798 | -0.004081 | -4.2551 | no |
| symbol-ownvalue-read | 0.222745 / 0.21823 | 0.221595 / 0.220368 | 0.000494 | 0.224 | no |
| symbol-ownvalue-guarded-read | 0.188211 / 0.191168 | 0.188082 / 0.185991 | -0.002653 | -1.3986 | no |
| set-ownvalue | 0.269064 / 0.267294 | 0.263541 / 0.263165 | -0.004826 | -1.7995 | no |
| list-literal-small | 0.057185 / 0.054225 | 0.056373 / 0.05445 | -0.000293 | -0.5269 | no |
| head-list | 0.951954 / 0.951772 | 0.943718 / 0.950405 | -0.004802 | -0.5044 | no |
| length-list | 0.932196 / 0.937737 | 0.921619 / 0.923261 | -0.012526 | -1.3398 | no |
| part-list | 1.10416 / 1.11781 | 1.09354 / 1.09435 | -0.01704 | -1.5338 | no |
| sameq-integer | 0.625716 / 0.645465 | 0.612939 / 0.61134 | -0.023451 | -3.6896 | no |
| sameq-list | 0.628213 / 0.626416 | 0.621826 / 0.621383 | -0.00571 | -0.9102 | no |
| if-true | 0.711308 / 0.712627 | 0.69677 / 0.688038 | -0.019563 | -2.7478 | no |
| module-local | 0.744198 / 0.740484 | 0.743583 / 0.750977 | 0.004939 | 0.6653 | no |
| module-chain-escape-small | 0.052363 / 0.051107 | 0.055194 / 0.051456 | 0.00159 | 3.0734 | yes |
| module-chain-escape-large | 0.094903 / 0.094411 | 0.100228 / 0.096464 | 0.003689 | 3.8972 | yes |
| block-local | 0.465461 / 0.465767 | 0.468186 / 0.467391 | 0.002174 | 0.467 | yes |
| block-symbol-only-repeat | 0.313116 / 0.306651 | 0.306714 / 0.315069 | 0.001008 | 0.3253 | no |
| block-init-null-repeat | 0.349519 / 0.349706 | 0.356441 / 0.370013 | 0.013615 | 3.8942 | yes |
| block-init-read-repeat | 0.402782 / 0.404314 | 0.404467 / 0.41563 | 0.006501 | 1.6108 | yes |
| with-local | 0.47706 / 0.47862 | 0.475333 / 0.472797 | -0.003775 | -0.79 | no |
| with-deep-body-small | 2.03899 / 2.02008 | 2.09917 / 2.04016 | 0.04013 | 1.9773 | yes |
| table-small | 0.189259 / 0.18758 | 0.18841 / 0.186 | -0.001215 | -0.6446 | no |
| table-iterator-plus | 0.036111 / 0.036548 | 0.037422 / 0.038792 | 0.001777 | 4.8927 | yes |
| table-iterator-pair-list | 0.065016 / 0.062485 | 0.070311 / 0.061491 | 0.00215 | 3.3733 | no |
| do-small | 0.91804 / 0.908692 | 0.925057 / 0.914482 | 0.006403 | 0.7011 | yes |
| d-nested-sin-small | 5.25213 / 5.22335 | 5.15016 / 5.1549 | -0.08521 | -1.6268 | no |
| downvalue-exact-hit | 0.765485 / 0.775958 | 0.774968 / 0.765526 | -0.000475 | -0.0616 | no |
| downvalue-exact-miss | 0.544456 / 0.558479 | 0.539377 / 0.532209 | -0.015675 | -2.8423 | no |
| downvalue-pattern-hit | 0.982872 / 0.966199 | 0.973861 / 0.956754 | -0.009228 | -0.9469 | no |
| downvalue-pattern-miss | 0.282373 / 0.28799 | 0.276122 / 0.274163 | -0.010039 | -3.5202 | no |
| downvalue-exact-miss-repeat | 1.2031 / 1.20695 | 1.18868 / 1.23438 | 0.006505 | 0.5398 | no |
| downvalue-pattern-miss-repeat | 1.25371 / 1.2515 | 1.23474 / 1.2787 | 0.004115 | 0.3285 | no |
| downvalue-guarded-hit | 1.89721 / 1.90357 | 1.87274 / 1.8959 | -0.01607 | -0.8456 | no |
| downvalue-guarded-decline-fallback | 1.42337 / 1.42019 | 1.41617 / 1.42021 | -0.00359 | -0.2525 | no |
| upvalue-guarded-hit | 2.337 / 2.36017 | 2.32653 / 2.33369 | -0.018475 | -0.7866 | no |
| subvalue-unguarded-hit | 1.39607 / 1.37285 | 1.39691 / 1.3874 | 0.007695 | 0.5558 | yes |
| subvalue-unguarded-miss | 0.792575 / 0.761587 | 0.769625 / 0.770774 | -0.006881 | -0.8856 | no |
| subvalue-guarded-hit | 2.48048 / 2.50985 | 2.47863 / 2.4707 | -0.0205 | -0.8216 | no |
| default-unguarded-hit | 1.50235 / 1.58927 | 1.50633 / 1.35742 | -0.113935 | -7.3706 | no |
| default-unguarded-miss | 1.07698 / 1.09143 | 1.05896 / 1.05743 | -0.02601 | -2.399 | no |
| default-guarded-hit | 1.64698 / 1.65932 | 1.6176 / 1.62024 | -0.03423 | -2.0706 | no |
| matchq-integer-success | 0.288121 / 0.28995 | 0.289238 / 0.289803 | 0.000485 | 0.1678 | no |
| matchq-integer-fail | 0.291414 / 0.294882 | 0.291892 / 0.294204 | -0.0001 | -0.0341 | no |
| matcher-single-tiny | 0.690445 / 0.684449 | 0.67839 / 0.685418 | -0.005543 | -0.8063 | no |
| matcher-single-wide-small | 0.048758 / 0.042823 | 0.04642 / 0.044693 | -0.000234 | -0.511 | no |
| matcher-single-wide-large | 0.088843 / 0.081161 | 0.085273 / 0.079774 | -0.002479 | -2.9158 | no |
| replaceall-simple | 0.389646 / 0.393963 | 0.357183 / 0.359625 | -0.033401 | -8.5248 | no |
| rule-rhs-deep-small | 3.08587 / 3.01435 | 3.07952 / 3.01105 | -0.004825 | -0.1582 | no |
| rewrite-guarded-hit | 1.77591 / 1.77019 | 1.78638 / 1.78591 | 0.013095 | 0.7386 | yes |
| rewrite-guarded-decline-fallback | 2.51005 / 2.52581 | 2.55605 / 2.54521 | 0.0327 | 1.2987 | yes |
| cases-wide-no-match | 0.1488 / 0.145255 | 0.152317 / 0.144975 | 0.001619 | 1.1008 | no |
| count-wide-no-match | 0.140892 / 0.138113 | 0.140152 / 0.137118 | -0.000867 | -0.6219 | no |
| position-wide-no-match | 0.176972 / 0.166774 | 0.174854 / 0.173129 | 0.002118 | 1.2326 | no |
| deletecases-wide-no-match | 0.171604 / 0.170502 | 0.173556 / 0.166559 | -0.000996 | -0.582 | no |
| deletecases-wide-association-no-match | 0.440879 / 0.428181 | 0.424444 / 0.440683 | -0.001967 | -0.4526 | no |
| deletecases-leaf-delete-small | 2.23329 / 2.2236 | 2.2526 / 2.21517 | 0.00544 | 0.2441 | no |
| replace-wide-no-match | 0.08402 / 0.07762 | 0.082593 / 0.079974 | 0.000464 | 0.5735 | no |
| replace-leaf-rebuild-small | 2.71038 / 2.67878 | 2.71284 / 2.65111 | -0.012605 | -0.4678 | no |
| take-multispec-small | 1.8804 / 1.80007 | 1.90171 / 1.82702 | 0.02413 | 1.3112 | yes |
| replacepart-deep-path-small | 1.3241 / 1.29278 | 1.33 / 1.28504 | -0.00092 | -0.0703 | no |
| replacepart-wide-rule-list | 0.299024 / 0.289642 | 0.290867 / 0.294008 | -0.001896 | -0.644 | no |
| replacepart-guarded-hit | 0.533486 / 0.540028 | 0.537319 / 0.536927 | 0.000366 | 0.0682 | no |
| replacepart-guarded-decline-fallback | 0.591193 / 0.597125 | 0.593948 / 0.593709 | -0.00033 | -0.0556 | no |
| string-repeat-small | 0.187513 / 0.189758 | 0.184285 / 0.182937 | -0.005025 | -2.6636 | no |
| character-range-small | 0.146022 / 0.134036 | 0.136901 / 0.130644 | -0.006257 | -4.468 | no |
| stringtrim-literal-list-4000 | 0.685992 / 0.65712 | 0.689275 / 0.656681 | 0.001422 | 0.2117 | no |
| stringtrim-literal-list-32000 | 5.52256 / 5.05183 | 5.19904 / 5.09172 | -0.141815 | -2.6822 | no |
| stringtrim-single-literal-4000 | 0.666153 / 0.688326 | 0.633869 / 0.635692 | -0.042459 | -6.2694 | no |
| stringtrim-single-literal-32000 | 5.45299 / 5.02175 | 5.02158 / 5.01977 | -0.216695 | -4.1375 | no |
| stringtrim-whitespace-4000 | 0.655399 / 0.630868 | 0.630128 / 0.629589 | -0.013275 | -2.0641 | no |
| stringtrim-whitespace-32000 | 5.43544 / 5.04559 | 5.00914 / 5.03113 | -0.22038 | -4.2053 | no |
| stringreplace-single-literal-unguarded | 0.396904 / 0.38872 | 0.388064 / 0.38576 | -0.0059 | -1.502 | no |
| stringreplace-pattern-unguarded | 1.04536 / 1.05452 | 1.05059 / 1.06143 | 0.00607 | 0.5781 | yes |
| stringreplace-guarded-hit | 0.70344 / 0.707982 | 0.694639 / 0.707583 | -0.0046 | -0.6518 | no |
| stringreplace-guarded-decline-fallback | 0.821408 / 0.831158 | 0.818949 / 0.822574 | -0.005522 | -0.6682 | no |
| appendto-list-incremental-small | 0.636477 / 0.62207 | 0.6495 / 0.67059 | 0.030771 | 4.89 | yes |
| appendto-list-incremental-large | 2.04838 / 2.00561 | 2.14743 / 2.10289 | 0.098165 | 4.8429 | yes |
| appendto-part-outer1000-small | 2.37525 / 2.18741 | 2.32545 / 2.22033 | -0.00844 | -0.37 | no |
| appendto-part-outer1000-large | 5.08004 / 4.4041 | 4.71208 / 4.47719 | -0.147435 | -3.1091 | no |
| association-set-key-incremental-small | 1.48034 / 1.45235 | 1.46789 / 1.44726 | -0.00877 | -0.5981 | no |
| association-set-key-incremental-large | 5.66814 / 5.55812 | 5.55168 / 5.44308 | -0.11575 | -2.0621 | no |
| association-small-literal-repeat | 0.740134 / 0.737079 | 0.743454 / 0.739625 | 0.002933 | 0.3971 | yes |
| merge-nested-small | 1.1367 / 1.11254 | 1.1526 / 1.11522 | 0.00929 | 0.8261 | yes |
| set-part-fixed100-len100 | 0.153442 / 0.145814 | 0.147181 / 0.140238 | -0.005919 | -3.9555 | no |
| set-part-fixed100-len800 | 0.707547 / 0.598459 | 0.657057 / 0.54464 | -0.052155 | -7.9869 | no |
| set-part-fill-small | 0.434845 / 0.424442 | 0.40746 / 0.402421 | -0.024703 | -5.7497 | no |
| set-part-fill-large | 1.44084 / 1.38988 | 1.31402 / 1.28939 | -0.113655 | -8.0301 | no |
| upvalue-candidates-tiny | 1.96433 / 1.97067 | 1.94728 / 1.96057 | -0.013575 | -0.69 | no |
| upvalue-candidates-wide-small | 0.153926 / 0.153518 | 0.157527 / 0.155795 | 0.002939 | 1.9119 | yes |
| upvalue-candidates-wide-large | 0.303365 / 0.301647 | 0.31418 / 0.305183 | 0.007175 | 2.372 | yes |
| sort-callback-small | 0.169199 / 0.163921 | 0.165561 / 0.164418 | -0.001571 | -0.9429 | no |
| sort-callback-large | 0.381541 / 0.380456 | 0.377096 / 0.371706 | -0.006597 | -1.7316 | no |
| sortby-callback-small | 0.198675 / 0.195015 | 0.193307 / 0.194364 | -0.003009 | -1.5289 | no |
| sortby-callback-large | 0.433699 / 0.436605 | 0.433566 / 0.430252 | -0.003243 | -0.7453 | no |
| reversesort-callback-small | 0.156593 / 0.158222 | 0.156669 / 0.158107 | -0.000019 | -0.0124 | no |
| reversesort-callback-large | 0.357174 / 0.352869 | 0.348716 / 0.353889 | -0.003719 | -1.0475 | no |
| keysort-callback-small | 0.096928 / 0.090303 | 0.093611 / 0.089517 | -0.002051 | -2.1914 | no |
| keysort-callback-large | 0.186213 / 0.181007 | 0.181072 / 0.178613 | -0.003768 | -2.0519 | no |
| keysortby-callback-small | 0.045238 / 0.044693 | 0.045788 / 0.045373 | 0.000615 | 1.3677 | yes |
| keysortby-callback-large | 0.097871 / 0.095942 | 0.099121 / 0.09707 | 0.001189 | 1.227 | yes |
| sort-default-wide | 0.019573 / 0.017002 | 0.019697 / 0.017195 | 0.000158 | 0.8667 | yes |
| sort-symbol-default-width4000 | 0.306113 / 0.294757 | 0.303807 / 0.291306 | -0.002879 | -0.9581 | no |
| sort-symbol-default-width16000 | 1.37401 / 1.27065 | 1.36807 / 1.28911 | 0.00626 | 0.4734 | no |
| sortby-default-wide | 0.499337 / 0.494772 | 0.497522 / 0.4938 | -0.001393 | -0.2804 | no |
| reversesort-default-wide | 0.019347 / 0.016668 | 0.019906 / 0.016856 | 0.000373 | 2.0741 | yes |
| keysort-default-wide | 0.382207 / 0.377813 | 0.383442 / 0.364265 | -0.006157 | -1.6201 | no |
| keysortby-default-wide | 0.902109 / 0.884026 | 0.881886 / 0.880985 | -0.011632 | -1.3025 | no |
| valueq-undefined-symbol-definitions-2000 | 0.009314 / 0.00918 | 0.00886 / 0.008862 | -0.000386 | -4.1743 | no |
| valueq-undefined-symbol-definitions-8000 | 0.008766 / 0.00887 | 0.008418 / 0.008475 | -0.000371 | -4.213 | no |
| module-retained-gensym-gc-batch | 0.067448 / 0.063823 | 0.066632 / 0.064322 | -0.000159 | -0.2415 | no |

Raw canonical `timings.tsv`, command manifests and outputs are under
`perf/timing-pinned/`; host observations are `timing-pinned-*-host.json`.
[Two-pair timing comparison](../../../.temp/brief-613/perf/timing-two-pair-comparison.json).

### 5. Remaining costs, G13, native policy, and stop-loss

G13 remains at 73 steps. No retained-payload obligation, production receipt,
coverage mark or promotion is discharged by G14. The eight test-only Legacy
users and the existing unresolved host edge remain visible. The new operations
remain rejected/unknown under the approved derived-fact rule.

Native polling saves/restores the displaced base and threshold, rebases to the
live resumed segment, polls actual headroom at each resumed frame and deferred
Evaluate entry, then sets the threshold to restored depth+4. This is physical
safety equivalence, not identical native addresses or polling cadence. It is
not a universal stack/RSS bound or a complete native-stack redesign.

Open costs are the measured resume allocation/instruction rates, the broader
cold prologue increases, variable startup components in the leaf profiles, and
the canonical timing increases above. No percentage is pre-approved. The
hardware/Callgrind numerical gap is not silently assigned a causal explanation;
the complete function/address ledger supports the structural attribution.
No speed result is claimed for a terminated base long-run witness.

Stop-loss ledger: one of six compiled semantic attempts used. Baseline final
population 52/71 failures plus six invariants → attempt 1 final 0/71 and 0/6.
Existing suite conflicts were the disclosed fixture/layout/macro pins, all now
resolved under the rulings. Subsequent redispatches replayed approved facts and
completed gates; no further semantic or performance repair attempt was made.
There are no two consecutive non-improving compiled attempts, and no stop-loss
threshold is reached. No mutant remains. Checkpoints and raw analysis failures
are preserved in [PROGRESS](../../../.temp/brief-613/PROGRESS.md).

The director has a complete, reviewable implementation and evidence set; the
timing residual and resume/cold-stack costs still require the director's ruling.
