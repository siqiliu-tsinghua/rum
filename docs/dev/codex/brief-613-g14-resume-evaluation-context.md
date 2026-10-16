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
