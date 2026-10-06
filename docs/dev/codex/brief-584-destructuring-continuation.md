# brief-584 — G9: list-destructuring assignment resumes into its remaining stores

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

brief-583's re-census found group G9: a list-destructuring `Set` walks its
left-hand side recursively, storing leaf by leaf, and a leaf whose target asks
for input resumes **past** the rest — the remaining sibling and nested stores
are skipped and the result is the leaf's value instead of the complete
right-hand side:

```
log = {}; a = {0}; b = 0;
{a[[(AppendTo[log, 1]; InputString["p: "]; 1)]], b} = {7, 9}
```

resumes to `7` where the queued path returns `List[7, 9]` (and `b` stays `0`).
With two suspending targets
(`r = ({a[[(…"p: "…; 1)]], a[[(…"q: "…; 2)]], b} = {7, 8, 9}); {r, a, b}`),
the resumed path reaches only the first prompt and returns
`List[7, List[7, 0], 0]`, log `{1}`; the queued path returns
`List[List[7, 8, 9], List[7, 8], 9]`, log `{1, 2}`. Three obligations in one
mechanism (`crates/rum-runtime/src/runtime/definitions/definitions.rs:467`,
`:480`, `:485` — the full-RHS return, the sibling cursor, the leaf
completion): lost continuations **4 -> 1** by the re-census.

Replace the recursive store continuation with an owned cursor and parent
stack, keeping the original right-hand side as the final result: resume the
pending leaf once, perform the remaining stores in order, return the complete
evaluated right-hand side. Shape validation still runs before any store;
completed stores stay committed; ordinary errors propagate unchanged.

brief-583's report and `.temp/brief-583/draft-G9.md` are the raw material;
**this brief governs** where they differ.

## Context pointers

- brief-583 and its report; `.temp/brief-583/{draft-G9.md,new-lost.tsv,new-probes.json,new-probes.tsv,reviewed-census.tsv}`.
- `definitions/definitions.rs` (the destructuring entry and the lvalue
  dispatcher at `:312`), `finish_definition_assignment`, the Part assignment
  continuations; `docs/guide/evaluation.md:116-176`, `:286-295`; the generated
  `Set` reference.
- The accepted continuation and flat-walk patterns of briefs 563, 568, 572-582;
  their structural depth contracts
  (`tests/association_depth_contract/*_contract.rs`).

## Scope

**Objective:** the resumable destructuring driver, its frame, its tests, and
the standing and performance gates.

**Edit surface:** `definitions/definitions.rs` (only the destructuring entry,
the driver and module wiring); new `definitions/definitions/destructuring.rs`
(and `destructuring/resume_frames.rs` if size requires); new
`crates/rum-stdlib/tests/destructuring_continuations.rs` and its support
children (reuse the existing oracle helpers without weakening them); a
structural depth contract for the new module beside the existing ones under
`tests/association_depth_contract/` with its `mod` line;
`docs/guide/evaluation.md`, only to state the existing store order, the
full-RHS result and the suspension behaviour; rum-runtime's frame lists
(exactly the new types).

**Standing duty — always in scope** (`TEMPLATE.md`): fingerprints, both
certificates, ownership reconciliation, every row disclosed. **STOP** on any
existing row change or disappearance, any new `UnknownHostPayload` or
`self_reaching` row, any new row in untouched code.

**Fence:** no change to `Expr`, definition precedence, Part selector
semantics, other assignment paths, public host APIs, manifests or the
lockfile; no leases; **`scripts/core-language-microbench.sh` stays
unchanged** (labels are scratch workloads). No edits to `AGENTS.md`, `TODO.md`,
root `PROGRESS.md`, earlier briefs or other docs.

**Disclosure duty:** every changed function, state field and root, inventory
row, measured label, adjacent defect.

## Deliverable

An owned traversal — cursor plus parent stack, no recursion — with the
original right-hand side retained separately. Once a leaf is selected, its
pending completion is recorded before `finish_definition_assignment` runs; on
`InputRequired` (on the error edge only) the remaining cursor and the original
right-hand side freeze in a rooted frame; on resume the child's outcome is
consumed once and the same next-leaf helper continues. A second suspension
freezes the new state once; pure abandonment drops pending work and leaves
committed stores intact (no new rollback semantics). No `Session` borrow or
borrowed iterator crosses a suspension; no deep owned-tree clone; no frame
allocation on an ordinary symbol `Set`.

## Tests

Red first, recorded before any production edit: the two witnesses above, the
nested witness in `new-probes.json`, a completed store before the first pause,
a pause in the middle and in the last leaf, an ordinary error in a later
target after a resume, a shape mismatch (validated before any store),
cancellation at a suspended target, answer-free abandonment, and a fresh
`1 + 1` after — through the rum-vs-rum oracle (value, error, termination,
ordered messages and prints, `log`, the stored variables, actual suspension,
exact prompts). The structural contract for the new module: the recursion
scanner per file and combined, no self-reaching state, root completeness, each
with a falsifier. Every earlier group's suite stays green.

## rum / Rust pitfalls (read twice)

1. Never make a pre-fix value the expected answer.
2. The result is the complete right-hand side, not the last leaf.
3. One change per measured candidate; stop after two consecutive compiling
   candidates that move none of: repaired sites, passing fixtures, regressing
   labels, worst regression.

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-584/tmp …`. Bounds: focused 150 s; runtime,
ownership and retention 300 s; a one-unit release build 900 s; a metric
process 150 s; `cargo test --workspace --no-fail-fast` 2 400 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test destructuring_continuations
cargo test -p rum-stdlib --lib association_depth_contract
cargo test -p rum-runtime
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report native ZeroMQ skips as skips.

**Performance gate.** Base: `.temp/brief-584/base/rum-cli` and
`rum-cli-alloc` — brief-582's selected one-unit candidate, whose production
source is the incoming tree (hashes in `sha256.txt`); do not rebuild them.
Candidates with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`, default and
`--features rum-cli/alloc-count` (two arguments), fresh target directory
each, hash checked. Panel (scratch workloads, each result checked first):
destructuring of symbols (`{x, y} = {1, 2}`), of held parts
(`{a[[1]], b} = {7, 9}` with setup), nested destructuring, an ordinary
`x = 1`, a part assignment, and the controls `dot-three-no-input`,
`differences-no-input`, `take-multispec-small` (`.temp/brief-563/perf/`).
**Noise floor first** (base against itself, two pairs); a label regresses
when both candidate samples exceed `max(+0.05%, floor)`. Allocation counts,
then two paired samples of `taskset -c 2 perf stat -x, -e
cpu_core/instructions/u`, one label per process, stdin `/dev/null`,
`(count(2N) - count(N)) / N`, raw output first. No timing suite. At most eight
measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-584/`; `PROGRESS.md` after the red capture, each
compiling candidate and each gate.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  (except the legacy certificate output `.temp/brief-546/certificates/`).
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the release `rum-cli` on passing programs, `taskset`/`perf`,
  `valgrind --tool=callgrind`. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`). No new `unsafe`. Never store
  a `Session` borrow, the cancellation observer or a `NativeStackDepth` in a
  frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Changed files and functions, state fields and roots.
2. Verbatim red and green output, the structural contract and falsifiers,
   ownership and certificate results, the broad suites; every inventory row
   change.
3. Three obligations repaired (4 -> 1 by the re-census); the noise floors and
   every measured row.
4. Stop-loss counters, residuals, open questions.

---

## Execution report

### 1. Changed files, functions, fields and roots

Selected implementation: **candidate-03**. List-destructuring `Set` now retains
its complete evaluated RHS separately from an owned current cursor, an inline
nearest parent, and a spill stack of older parents. Leaf selection advances the
cursor and records pending completion before calling the existing lvalue
assignment dispatcher. A successful child consumes that completion once; an
ordinary error propagates unchanged. Only `InputRequired` creates the rooted
frame. Resume enters the same next-leaf helper and ultimately returns the full
RHS. Pure abandonment drops pending work and leaves completed stores committed.

Production changes:

- `definitions/definitions.rs`: adds module wiring and changes
  `try_set_list_destructuring_lvalue` to return the owned driver's result. Its
  recursive `store_list_destructuring_assignments` is replaced by the new
  module. The shape validator and other assignment dispatcher bodies are
  unchanged.
- `definitions/definitions/destructuring.rs`: new
  `DestructuringWork::{next_leaf, complete_leaf}`;
  `Session::{store_list_destructuring_assignments, drive_list_destructuring}`;
  `DestructuringSuspensionFrame::resume`; `suspend_destructuring`;
  `resume_destructuring`.
- `frame_root_trace/derived_inventory.rs`: exactly three new root derives
  (`DestructuringCursor`, `DestructuringWork`, `DestructuringSuspensionFrame`)
  and one pure-abandon entry (the frame). In `frame_root_trace.rs`,
  `suspension_protocol_repository_invariants_hold` changes only the
  named-frame count, 562 -> 563.
- `docs/guide/evaluation.md`: states existing shape-first validation, store
  order, full-RHS result, suspension and abandonment behavior.

Every new state field and root:

| Type | Fields | Roots |
| --- | --- | --- |
| DestructuringCursor | lhs: SharedArgs; rhs: SharedArgs; next: usize | Both argument arrays |
| DestructuringWork | current: DestructuringCursor; parent: Option<DestructuringCursor>; parents: Vec<DestructuringCursor>; pending_leaf: bool | Current cursor, inline parent and every spill parent |
| DestructuringSuspensionFrame | symbol: Symbol; original_rhs: Expr; work: DestructuringWork | Assignment label, full original RHS and all cursor roots |

All three types derive root tracing; the frame derives pure abandonment. Resume
clones shared handles and a flat parent stack, while the immutable frame remains
rooted. No Session borrow, borrowed iterator, cancellation observer,
NativeStackDepth, lease or deep owned-tree clone crosses suspension. An ordinary
symbol Set does not allocate a frame.

The new driver is under 10k characters. Its extraction begins the split of the
21.9k definitions file; other dispatcher responsibilities remain outside scope.
The 30.3k frame protocol file changes one count, and its large generated lists
receive only the new types. The new integration suite owns its support children;
no unrelated directory reorganization was made. `association_depth_contract`
gains one module and its wiring. All test/helper functions are disclosed below;
`rule_pattern_continuations/support.rs` is reused unchanged. The test-only
`Observation` holds `view: View`, `log: String`, `prompts: Vec<String>` and
`stored: Expr`; it has no suspension-frame role.

- `crates/rum-stdlib/tests/destructuring_continuations.rs`: module wiring.

- `crates/rum-stdlib/tests/destructuring_continuations/lifecycle.rs`: `cancel`, `cancel_at_first_target`, `cancel_at_second_target`, `abandon_first_target_without_answer`, `abandon_second_target_without_answer`.

- `crates/rum-stdlib/tests/destructuring_continuations/stores.rs`: `first_leaf_returns_complete_rhs`, `enclosing_assignment_keeps_complete_rhs`, `two_targets_suspend_in_store_order`, `nested_witness_finishes_parent_sibling`, `completed_prefix_and_middle_pause_keep_ordered_output`, `last_leaf_still_returns_whole_rhs`, `ordinary_later_error_after_resume_preserves_only_committed_prefix`, `nested_shape_mismatch_validates_before_any_store_or_prompt`, `nested_lists_keep_work_between_pauses_and_after_last_pause`, `empty_lists_and_list_valued_leaf_keep_original_rhs`, `association_and_definition_lvalues_continue_to_next_store`, `deeper_parent_stack_resumes_through_each_level`.

- `crates/rum-stdlib/tests/destructuring_continuations/support.rs`: `run`, `compare`.

- `crates/rum-stdlib/src/tests/association_depth_contract/destructuring_contract.rs`: `sources`, `assert_acyclic`, `recursion_scanner_per_file_and_combined`, `direct_recursion_falsifier`, `cross_file_recursion_falsifier`, `compact`, `assert_roots`, `assert_no_self_reaching`, `complete_roots_flat_fields_and_no_self_reaching_state`, `missing_root_falsifier`, `skipped_root_falsifier`, `recursive_state_falsifier`, `self_reaching_inventory_falsifier`.

### 2. Verification and every inventory row change

Red was captured before any production edit. `red-01.log` records test-code
quoting/dereference compilation errors, corrected before the red capture below.
`green-01.log` records missing derive-trait imports, corrected before a compiling
candidate or measurement. Neither compile failure is a measured candidate.

`python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-stdlib --test destructuring_continuations -- --nocapture` — exit 101. [red-01.log](../../../.temp/brief-584/red-01.log)

~~~text

Some errors have detailed explanations: E0061, E0277.
For more information about an error, try `rustc --explain E0061`.
error: could not compile `rum-stdlib` (test "destructuring_continuations") due to 4 previous errors
~~~

`python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-stdlib --test destructuring_continuations -- --nocapture` — exit 101. [red-02.log](../../../.temp/brief-584/red-02.log)

~~~text
test result: FAILED. 3 passed; 12 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
~~~

Verbatim representative pre-fix differences (the link contains the full capture):

~~~text
  left: View { status: "OK", value: Some("8"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[7, 8, 9]"), termination: "None", messages: [], prints: [] }
  left: View { status: "OK", value: Some("7"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }], prints: ["before", "after"] }
 right: View { status: "OK", value: Some("List[5, 7, 9]"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }], prints: ["before", "after"] }
  left: View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[List[], 7, List[8, 9]]"), termination: "None", messages: [], prints: [] }
  left: View { status: "OK", value: Some("List[7, List[7], 0]"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[List[7, 9], List[7], 9]"), termination: "None", messages: [], prints: [] }
  left: View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[7, 9]"), termination: "None", messages: [], prints: [] }
  left: View { status: "OK", value: Some("7"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[5, 7]"), termination: "None", messages: [], prints: [] }
  left: ["p: "]
 right: ["p: ", "q: "]
  left: View { status: "OK", value: Some("List[7, List[7], 0]"), termination: "None", messages: [], prints: [] }
 right: View { status: "OK", value: Some("List[List[List[7], 9], List[7], 9]"), termination: "None", messages: [], prints: [] }
  left: View { status: "OK", value: Some("7"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }], prints: ["before"] }
 right: View { status: "Other { message: \"uncaught throw: \\\"leaf boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "probe", tag: "tag", text: "G4 message" }, RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"leaf boom\"] returned to top level." }], prints: ["before", "error"] }
  left: ["p: "]
 right: ["p: ", "q: "]
~~~

`python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-stdlib --test destructuring_continuations -- --nocapture` — exit 101. [green-01.log](../../../.temp/brief-584/green-01.log)

~~~text

Some errors have detailed explanations: E0277, E0405, E0433.
For more information about an error, try `rustc --explain E0277`.
error: could not compile `rum-runtime` (lib) due to 10 previous errors
~~~

The initial formatting check also failed on new-file formatting only: [fmt-01-check.log](../../../.temp/brief-584/fmt-01-check.log). Formatting was applied and the final check below is green.

`python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-stdlib --test destructuring_continuations -- --nocapture` — exit 0. [green-02.log](../../../.temp/brief-584/green-02.log)

~~~text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
~~~

`python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-stdlib --test destructuring_continuations -- --nocapture` — exit 0. [green-candidate-02.log](../../../.temp/brief-584/green-candidate-02.log)

~~~text
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
~~~

`python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-stdlib --test destructuring_continuations -- --nocapture` — exit 0. [green-candidate-03.log](../../../.temp/brief-584/green-candidate-03.log)

~~~text
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
~~~

`python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-stdlib --lib association_depth_contract -- --nocapture` — exit 0. [structural-03.log](../../../.temp/brief-584/structural-03.log)

~~~text
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 1082 filtered out; finished in 1.58s
~~~

`python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-runtime frame_root_trace -- --nocapture` — exit 0. [frame-protocol-03.log](../../../.temp/brief-584/frame-protocol-03.log)

~~~text
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1127 filtered out; finished in 1.19s
~~~

The structural contract checks per-file and combined recursion, exact fields
and complete derived roots, pure abandonment, and no self-reaching state.
Six falsifiers challenge direct recursion, a cross-file cycle, a missing root,
a skipped root, recursive state and a self-reaching inventory entry. All detect
the introduced violation. Existing pre-store shape validation, leaf evaluation,
and Expr internals are explicit boundaries, not claimed recursion-free here.

The first runtime gate found a frame directly owning String. It was fixed using
a rooted system Symbol, without weakening the protocol:

`python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-runtime` — exit 101. [runtime-final.log](../../../.temp/brief-584/runtime-final.log)

~~~text
test result: FAILED. 1137 passed; 1 failed; 3 ignored; 0 measured; 0 filtered out; finished in 30.69s
~~~

~~~text
/home/remi/dev/rum/crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs: SuspensionFrame DestructuringSuspensionFrame directly owns String
~~~

`python3 scripts/run-bounded.py --timeout 150 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo fmt --check` — exit 0. [fmt-final-03.log](../../../.temp/brief-584/fmt-final-03.log)

~~~text
(no output)
~~~

`python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-runtime` — exit 0. [runtime-final-03.log](../../../.temp/brief-584/runtime-final-03.log)

~~~text
test result: ok. 1138 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.82s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.55s
~~~

`python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-stdlib --lib ownership_graph -- --nocapture` — exit 0. [ownership-final-03.log](../../../.temp/brief-584/ownership-final-03.log)

~~~text
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... HOST PAYLOAD LEDGER: 65 rows; 6 storage edges (3 Declared, 3 Unbounded); 59 PublicInput
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7680 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 60.76s
~~~

`python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-stdlib --lib regenerate_local_contract_fingerprints -- --ignored --nocapture` — exit 0. [fingerprints-replay.log](../../../.temp/brief-584/fingerprints-replay.log)

~~~text
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 0.22s
~~~

`python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture` — exit 101. [certificates-true-replay.log](../../../.temp/brief-584/certificates-true-replay.log)

~~~text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.75s
~~~

`python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture` — exit 101. [certificates-false-replay.log](../../../.temp/brief-584/certificates-false-replay.log)

~~~text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 18.92s
~~~

`python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-stdlib --lib local_production_acceptance -- --nocapture` — exit 0. [certificates-true-final.log](../../../.temp/brief-584/certificates-true-final.log)

~~~text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1139 filtered out; finished in 18.45s
~~~

`python3 scripts/run-bounded.py --timeout 300 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance -- --nocapture` — exit 0. [certificates-false-final.log](../../../.temp/brief-584/certificates-false-final.log)

~~~text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1133 filtered out; finished in 18.61s
~~~

`python3 scripts/run-bounded.py --timeout 2400 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-584/tmp env RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 cargo test --workspace --no-fail-fast -- --nocapture` — exit 0. [workspace-final.log](../../../.temp/brief-584/workspace-final.log)

~~~text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.92s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 48.63s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
test result: ok. 1138 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 28.25s
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... HOST PAYLOAD LEDGER: 65 rows; 6 storage edges (3 Declared, 3 Unbounded); 59 PublicInput
test tests::ownership_graph::production_inventory_is_exact ... EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 65 rows
EXACT INVENTORY UnknownConstruct: 7680 rows
ACCEPTANCE_BASELINE OPTION_FRONTIER=927 RET001=10 RET002=890 RET003=27 PUBLIC_WRAPPER=16 LOCAL_PRODUCTION_BODY_ROWS=2
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
PRODUCTION_BODY_ROWS=2 N=78 UNSPLIT=147 UNSETTLED=90 LOCAL_OPTION_BODIES=2 PENDING=315
test result: ok. 1087 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1200.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.07s
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.30s
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.56s
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.84s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.68s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
~~~

The complete per-group workspace table is [workspace-summary.tsv](../../../.temp/brief-584/workspace-summary.tsv); the ignored Rust test names are in [ignored-tests.txt](../../../.temp/brief-584/ignored-tests.txt).

Certificate replays earned current marks and rejected stale incoming cells.
Their exact generated cells were installed, then both feature modes replayed
green. No identity, status, disposition, strength or exclusion changed.

Native ZeroMQ capability skips: 9. These are skips, not exercised native transport passes. The existing `RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1` flag permits the loopback-PermissionDenied skip.

~~~text
test zeromq::tests::native::basic::native_zeromq_backend_echoes_heartbeat_over_loopback_req_socket ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::basic::native_zeromq_backend_round_trips_execute_over_loopback_sockets ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_frontend_style_control_interrupt_round_trip ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::control::native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::execute::native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::session_flow::native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::frontend::stdin::native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_rejects_tampered_signed_requests_end_to_end ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
test zeromq::tests::native::signed::native_zeromq_backend_round_trips_signed_execute_and_signed_iopub_messages ... skipping native ZeroMQ test: loopback socket binding is not permitted; RUM_ALLOW_NATIVE_ZEROMQ_SKIP explicitly allows this skip
~~~

[ownership-row-changes.json](../../../.temp/brief-584/ownership-row-changes.json)

`host_payloads.tsv`: 0 position changes, 0 additions, 0 removals.

`unknown_attributes.tsv`: 0 position changes, 3 additions, 0 removals.

~~~text
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs	8	1	unresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace)
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs	15	1	unresolved attribute/expansion derive (Clone , rum_derive :: FrameRootTrace)
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs	114	1	unresolved attribute/expansion derive (rum_derive :: FrameRootTrace , rum_derive :: PureFrameAbandon)
~~~

`unknown_captures.tsv`: 3 position changes, 0 additions, 0 removals.

~~~text
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	439	78	closure captures: || (capture types/modes unresolved)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	440	78	closure captures: || (capture types/modes unresolved)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	506	35	closure captures: |existing| (capture types/modes unresolved)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	489	35	closure captures: |existing| (capture types/modes unresolved)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	535	18	closure captures: |(lhs_item , rhs_item)| (capture types/modes unresolved)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	518	18	closure captures: |(lhs_item , rhs_item)| (capture types/modes unresolved)
~~~

`unknown_macros.tsv`: 15 position changes, 6 additions, 0 removals.

~~~text
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	41	30	unexpanded macro crate :: resume_or_suspend! (ends at 49:14)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	42	30	unexpanded macro crate :: resume_or_suspend! (ends at 50:14)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	85	9	unexpanded macro crate :: resume_or_suspend! (ends at 90:10)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	86	9	unexpanded macro crate :: resume_or_suspend! (ends at 91:10)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	102	9	unexpanded macro crate :: resume_or_suspend! (ends at 107:10)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	103	9	unexpanded macro crate :: resume_or_suspend! (ends at 108:10)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	118	62	unexpanded macro vec! (ends at 118:76)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	119	62	unexpanded macro vec! (ends at 119:76)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	121	19	unexpanded macro crate :: resume_or_suspend! (ends at 129:10)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	122	19	unexpanded macro crate :: resume_or_suspend! (ends at 130:10)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	161	12	unexpanded macro matches! (ends at 161:166)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	162	12	unexpanded macro matches! (ends at 162:166)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	169	13	unexpanded macro debug_assert! (ends at 169:62)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	170	13	unexpanded macro debug_assert! (ends at 170:62)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	178	44	unexpanded macro vec! (ends at 178:58)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	179	44	unexpanded macro vec! (ends at 179:58)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	359	16	unexpanded macro matches! (ends at 364:14)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	360	16	unexpanded macro matches! (ends at 365:14)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	386	52	unexpanded macro vec! (ends at 386:66)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	387	52	unexpanded macro vec! (ends at 387:66)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	401	52	unexpanded macro vec! (ends at 401:66)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	402	52	unexpanded macro vec! (ends at 402:66)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	462	17	unexpanded macro format! (ends at 462:73)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	463	17	unexpanded macro format! (ends at 463:73)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	491	5	unexpanded macro matches! (ends at 491:126)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	474	5	unexpanded macro matches! (ends at 474:126)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	495	5	unexpanded macro matches! (ends at 495:123)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	478	5	unexpanded macro matches! (ends at 478:123)
BEFORE UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	499	5	unexpanded macro matches! (ends at 499:119)
AFTER  UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions.rs	482	5	unexpanded macro matches! (ends at 482:119)
~~~

~~~text
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs	25	9	unexpanded macro debug_assert! (ends at 25:42)
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs	37	25	unexpanded macro unreachable! (ends at 37:81)
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs	61	9	unexpanded macro debug_assert! (ends at 61:41)
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs	74	13	unexpanded macro unreachable! (ends at 74:64)
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs	80	13	unexpanded macro unreachable! (ends at 80:69)
UnknownConstruct	crates/rum-runtime/src/runtime/definitions/definitions/destructuring.rs	143	5	unexpanded macro crate :: resume_or_suspend! (ends at 147:7)
~~~

`unknown_storage.tsv`: 0 position changes, 0 additions, 0 removals.

`self_reaching.tsv`: 0 position changes, 0 additions, 0 removals.

`host_payload_ledger.tsv`: 0 position changes, 0 additions, 0 removals.

[obligation-row-changes.json](../../../.temp/brief-584/obligation-row-changes.json)

~~~text
ROW 177
BEFORE N:NormalizeRules	NormalizeRules	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:385	rule_exprs/rules rebuilt from evaluated rewrite-rule expression	crates/rum-runtime/src/runtime/definitions/definitions.rs:147; crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:200; crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:248	rum_runtime::runtime::builtin_adapters::builtin_normalize_rules
AFTER  N:NormalizeRules	NormalizeRules	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:385	rule_exprs/rules rebuilt from evaluated rewrite-rule expression	crates/rum-runtime/src/runtime/definitions/definitions.rs:148; crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:200; crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:248	rum_runtime::runtime::builtin_adapters::builtin_normalize_rules
ROW 227
BEFORE N:Rule	Rule	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:490	lhs/resolved_args retaining evaluated LHS subexpressions	crates/rum-runtime/src/runtime/definitions/definitions.rs:84; crates/rum-runtime/src/runtime/definitions/definitions.rs:101; crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:128; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:11; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:13	rum_runtime::runtime::builtin_adapters::builtin_rule
AFTER  N:Rule	Rule	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:490	lhs/resolved_args retaining evaluated LHS subexpressions	crates/rum-runtime/src/runtime/definitions/definitions.rs:85; crates/rum-runtime/src/runtime/definitions/definitions.rs:102; crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:128; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:11; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:13	rum_runtime::runtime::builtin_adapters::builtin_rule
ROW 228
BEFORE N:RuleDelayed	RuleDelayed	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:491	lhs/resolved_args retaining evaluated LHS subexpressions	crates/rum-runtime/src/runtime/definitions/definitions.rs:84; crates/rum-runtime/src/runtime/definitions/definitions.rs:101; crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:128; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:11; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:13	rum_runtime::runtime::builtin_adapters::builtin_rule_delayed
AFTER  N:RuleDelayed	RuleDelayed	N	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs:491	lhs/resolved_args retaining evaluated LHS subexpressions	crates/rum-runtime/src/runtime/definitions/definitions.rs:85; crates/rum-runtime/src/runtime/definitions/definitions.rs:102; crates/rum-runtime/src/runtime/definitions/rule_lhs.rs:128; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:11; crates/rum-runtime/src/runtime/definitions/rule_lhs/state.rs:13	rum_runtime::runtime::builtin_adapters::builtin_rule_delayed
~~~

All 68 contract rows changed only the replayed fingerprint cell: [fingerprint-row-changes.json](../../../.temp/brief-584/fingerprint-row-changes.json)

~~~text
row	id	before	after
2	B01	fnv1a64:dd1ab3ba48892605	fnv1a64:3f0979518b0821b3
3	B02	fnv1a64:7480471aa298c050	fnv1a64:7e0ff69775ffe85a
4	B03	fnv1a64:0977ccb66394cadb	fnv1a64:b2a5f700a154ef79
5	B04	fnv1a64:550ad67307ba59ad	fnv1a64:a3f7db810a2d93ab
6	B05	fnv1a64:bd94e844a3bca044	fnv1a64:944ffd1213ede956
7	B06	fnv1a64:427160595d077c31	fnv1a64:29eaf326ca8093d7
8	B07	fnv1a64:7d94979446aaa714	fnv1a64:e01c42fefbfa8766
9	B08	fnv1a64:832d8589c38181bf	fnv1a64:2666d0698a6dac45
10	B09	fnv1a64:6970137158bf9cd9	fnv1a64:4d41e98df81c484f
11	B10	fnv1a64:6c59f4063fe314b6	fnv1a64:df8feb4d5efd32b0
12	B11	fnv1a64:166287d021ae7fad	fnv1a64:654f8cde2421b9ab
13	B12	fnv1a64:287c936d34c831d9	fnv1a64:0c4e6989d424dd4f
14	I01	fnv1a64:bafc9eb7b5411c85	fnv1a64:1ceb644ef7c01833
15	I02	fnv1a64:14d665d89bf32844	fnv1a64:eb917aa60c247156
16	I03	fnv1a64:8193d31309477699	fnv1a64:3da12d052f716e0f
17	I04	fnv1a64:ce8728e44345b4f0	fnv1a64:8ff8da88b4dd8cfa
18	P01	fnv1a64:43c5e12ed1ef5251	fnv1a64:251cb8a2732de577
19	P02	fnv1a64:d6c1771ca9287b26	fnv1a64:6569d20b7bea10a0
20	P03	fnv1a64:18f207d36ab28973	fnv1a64:db84ff7a45f156a1
21	P04	fnv1a64:695b65302741dc50	fnv1a64:72eb14acfaa9045a
22	P05	fnv1a64:3bd45f6a1726aa35	fnv1a64:c75400b350f7daa3
23	P06	fnv1a64:78fbb83dc0f7cb59	fnv1a64:5ccd8e5a605476cf
24	P070	fnv1a64:b349547f9f6d4559	fnv1a64:971b2a9c3ec9f0cf
25	P080	fnv1a64:2143e68b1c3d5276	fnv1a64:6cb561a7c224bc70
26	P090	fnv1a64:084a666c51183b10	fnv1a64:5dc864a654b4ab1a
27	P100	fnv1a64:3baee80a61fe7e7d	fnv1a64:b3cdff23ddca09fb
28	P110	fnv1a64:5a89f727ce2a610e	fnv1a64:6cf699d893b118f8
29	P120	fnv1a64:bff5f31523e45cdf	fnv1a64:40283cf3bde57f65
30	P130	fnv1a64:2161f0f1755e8114	fnv1a64:83e99c5c2aae6166
31	P140	fnv1a64:642ee63d1b7d6188	fnv1a64:202cfb210b205482
32	P150	fnv1a64:61317430935327b0	fnv1a64:ee8eaa5a9bd8efba
33	P071	fnv1a64:c6be0cda740d96b4	fnv1a64:0d0e3da33bd63106
34	P081	fnv1a64:d2bea43c999cfdf9	fnv1a64:7f3504739ce7b66f
35	P091	fnv1a64:bd07c37a271e6911	fnv1a64:189ea16a15628b37
36	P101	fnv1a64:528ce7f5038dfe0e	fnv1a64:64f98aa5c914b5f8
37	P111	fnv1a64:674f013cccdc3869	fnv1a64:e0eb9b90a0ae905f
38	P121	fnv1a64:50cbaa9c687761d2	fnv1a64:2a789ec6267f34d4
39	P131	fnv1a64:f2b4f8d93e0e7b2d	fnv1a64:41a1fde74081b52b
40	P141	fnv1a64:b7d1f23f0ad432f1	fnv1a64:5cbaa3a25689eb97
41	P151	fnv1a64:6b80406265f9ac1f	fnv1a64:c7f783bfcaf694a5
42	B13	fnv1a64:d081f3e28da43831	fnv1a64:b7fb86affb1d4fd7
43	R01	fnv1a64:920938e1b91a3d20	fnv1a64:eb0407c19b96402a
44	R02	fnv1a64:ebf1771746ea2733	fnv1a64:5b6aadbcdee26661
45	R03	fnv1a64:845bd136f9cd5ddc	fnv1a64:a006ec98da3c071e
46	R040	fnv1a64:daa0a8440b6706c0	fnv1a64:807aaf60ec2ac8ca
47	R050	fnv1a64:16d70e24e25d2795	fnv1a64:9d43edf79fcec783
48	R060	fnv1a64:a97fb10338312698	fnv1a64:be6971a2ade04a52
49	R070	fnv1a64:3b2c6c567b3ebcf9	fnv1a64:e7a2cc8d7e89756f
50	R080	fnv1a64:958b91d712ca7df0	fnv1a64:56fd437b846255fa
51	R090	fnv1a64:690a4ef5a6f6223d	fnv1a64:976d1260eded08bb
52	R100	fnv1a64:ce9d71090d6a755b	fnv1a64:77cb9b534b2a99f9
53	R110	fnv1a64:78b604a5002daa5e	fnv1a64:13117faf43a5f788
54	R041	fnv1a64:b6bb570ca4cbf731	fnv1a64:9e34e9da12450ed7
55	R051	fnv1a64:966c9481de464790	fnv1a64:ebea92bbe1e2b79a
56	R061	fnv1a64:7e863ee20445712f	fnv1a64:70513d8aa2c3e835
57	R071	fnv1a64:275e3dd25b74877a	fnv1a64:7178b0f5144c978c
58	R081	fnv1a64:f799515efdaa537d	fnv1a64:6fb868787975defb
59	R091	fnv1a64:9dda44f9757fc7c8	fnv1a64:51192339c62484c2
60	R101	fnv1a64:0b2a5581e52b4658	fnv1a64:d103cac95bc25612
61	R111	fnv1a64:f59ccec5dcf7bf87	fnv1a64:2b08e46fcee587dd
62	R12	fnv1a64:99a34d0f078ef0c7	fnv1a64:8c7e814ed7b95a1d
63	R13	fnv1a64:bfce75b544491f8b	fnv1a64:d233e665deb76ba9
64	R14	fnv1a64:5606f6803eaf3e7d	fnv1a64:ce260d99ba7ac9fb
65	R15	fnv1a64:8229575fa1a4a7a1	fnv1a64:4020736ecc0b2107
66	O010	fnv1a64:6802bedd40d9930f	fnv1a64:25fbb039427d2495
67	O0200	fnv1a64:5ef688116a348788	fnv1a64:1af49cf559d77a82
68	O011	fnv1a64:ec086d8b45c2eb18	fnv1a64:00f22e2abb720ed2
69	O0210	fnv1a64:a4f75775f419f2a7	fnv1a64:85deb18a149adb7d
~~~

[certificate-cell-changes-true.json](../../../.temp/brief-584/certificate-cell-changes-true.json) — every changed certificate cell (complete before/after cells in JSON; changes are fingerprint-only):

~~~text
promoted.tsv:2 C01 column=5 hash=true source=crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6 -> crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6
promoted.tsv:3 C02 column=5 hash=true source=crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6 -> crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6
~~~

[certificate-cell-changes-false.json](../../../.temp/brief-584/certificate-cell-changes-false.json) — every changed certificate cell (complete before/after cells in JSON; changes are fingerprint-only):

~~~text
promoted.tsv:2 C01 column=6 hash=false source=crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6 -> crates/rum-runtime/src/runtime/options/core.rs:106:5-122:6
promoted.tsv:3 C02 column=6 hash=false source=crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6 -> crates/rum-runtime/src/runtime/options/core.rs:224:5-238:6
~~~

No new UnknownHostPayload or self-reaching rows, no existing identity changes or
disappearances, and no additions in untouched code were found. Unknown
constructs remain unknown; no new body or lease is promoted.

### 3. Three repaired obligations and every measured label

The old P0354-P0356 sites (definitions.rs:467,480,485) now belong to the owned
store continuation: full-RHS return, remaining sibling cursor, pending-leaf
completion. Three obligations in one mechanism are repaired: residual lost
continuations 4 -> 1. G10 remains outside this change.

Provided base binaries were hash-checked and never rebuilt. Every candidate
uses fresh default and alloc-count target directories and one release codegen
unit (`--features rum-cli/alloc-count` as two arguments). All label results were
checked first, raw output saved before parsing, allocation counts run before
two paired instruction samples. One label per process, stdin `/dev/null`,
`taskset -c 2 perf stat -x, -e cpu_core/instructions/u`; slopes are
(count(2N)-count(N))/N at N=20,000. The last three source workloads are exact
brief-563 control copies. No metric ran concurrently with Cargo or another
agent workload. No timing suite ran; core-language-microbench.sh is unchanged.

The workload/result manifest is [perf/labels.json](../../../.temp/brief-584/perf/labels.json); exact `.rum` source hashes are in [perf/sources-sha256.json](../../../.temp/brief-584/perf/sources-sha256.json).

The larger absolute delta from two base/base pairs is each noise floor.
Regression requires both candidate instruction samples to exceed
max(+0.05%, floor). All measured rows, including rejected candidates, follow.

noise-floor: [noise-floor/perf/paired-results.json](../../../.temp/brief-584/noise-floor/perf/paired-results.json) (raw process ledger), [noise-floor/perf/slopes.tsv](../../../.temp/brief-584/noise-floor/perf/slopes.tsv)

~~~text
label	N	floor	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
destructuring-symbols	20000	0.0029262873206037376	7225.87965	7225.6682	-0.0029262873206037376	7225.88545	7225.8684	-0.00023595724174541033	False
destructuring-held-parts	20000	0.0008277920706234454	13433.3251	13433.2139	-0.0008277920706234454	13433.0715	13433.15795	0.0006435609309463075	False
destructuring-nested	20000	0.0016147536009647467	16302.8176	16302.8764	0.00036067385063009993	16302.7969	16303.06015	0.0016147536009647467	False
ordinary-symbol-set	20000	0.003935365981011518	4630.834	4630.83995	0.00012848657497954719	4631.08135	4630.8991	-0.003935365981011518	False
ordinary-part-set	20000	0.0027311404278140827	10894.71625	10894.4187	-0.0027311404278140827	10894.56795	10894.5293	-0.00035476395372491254	False
dot-three-no-input	20000	0.00024600729822132905	254199.78695	254199.1913	-0.00023432356381913877	254199.7756	254200.40095	0.00024600729822132905	False
differences-no-input	20000	0.14238502548915566	10037.71285	10052.00505	0.14238502548915566	10044.887	10052.0016	0.07082807402412072	False
take-multispec-small	20000	0.013839066272813838	49923.02455	49923.24285	0.00043727318601582965	49922.8045	49915.89565	-0.013839066272813838	False
~~~

candidate-01: [candidate-01/perf/paired-results.json](../../../.temp/brief-584/candidate-01/perf/paired-results.json) (raw process ledger), [candidate-01/perf/slopes.tsv](../../../.temp/brief-584/candidate-01/perf/slopes.tsv)

~~~text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
destructuring-symbols	20000	0.0029262873206037376	0.0	0.0	32.0	32.0	7225.78255	7200.79725	-0.3457798491320552	7225.9072	7200.63935	-0.3496841199399747	False
destructuring-held-parts	20000	0.0008277920706234454	20.0	20.0	738.0	738.0	13433.00635	13377.93715	-0.40995439565172376	13433.17975	13378.1833	-0.40940753435536603	False
destructuring-nested	20000	0.0016147536009647467	20.0	21.0	738.0	834.0	16303.0891	16514.7926	1.2985483836925216	16303.21685	16514.6811	1.2970707066317422	True
ordinary-symbol-set	20000	0.003935365981011518	1.0	1.0	160.0	160.0	4630.88325	4630.8671	-0.00034874556597008777	4631.0179	4631.05435	0.0007870839799695162	False
ordinary-part-set	20000	0.0027311404278140827	21.0	21.0	866.0	866.0	10894.4343	10852.49385	-0.38497134266072264	10894.47085	10852.44695	-0.38573603600031703	False
dot-three-no-input	20000	0.00024600729822132905	596.0	596.0	23504.0	23504.0	254198.99965	254198.2666	-0.0002883764298911018	254198.6979	254198.2656	-0.00017006381368700474	False
differences-no-input	20000	0.14238502548915566	13.0	13.0	704.0	704.0	10030.6792	10035.6073	0.04913027225512678	10030.6459	10033.9038	0.03247946376017108	False
take-multispec-small	20000	0.013839066272813838	109.0	109.0	5856.0	5856.0	49923.42395	49922.0814	-0.0026892185947424707	49915.29545	49922.5399	0.014513487168010819	False
~~~

candidate-02: [candidate-02/perf/paired-results.json](../../../.temp/brief-584/candidate-02/perf/paired-results.json) (raw process ledger), [candidate-02/perf/slopes.tsv](../../../.temp/brief-584/candidate-02/perf/slopes.tsv)

~~~text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
destructuring-symbols	20000	0.0029262873206037376	0.0	0.0	32.0	32.0	7225.7101	7215.99035	-0.1345161910107695	7225.72065	7215.5932	-0.14015833839355896	False
destructuring-held-parts	20000	0.0008277920706234454	20.0	20.0	738.0	738.0	13432.99565	13394.02965	-0.29007677077599325	13433.01455	13394.047	-0.29008790137876606	False
destructuring-nested	20000	0.0016147536009647467	20.0	20.0	738.0	738.0	16303.0568	16296.053	-0.04296004170212253	16302.7266	16296.13895	-0.04040827133787728	False
ordinary-symbol-set	20000	0.003935365981011518	1.0	1.0	160.0	160.0	4630.97	4630.8404	-0.002798549763871616	4630.79335	4630.9056	0.0024239906969647507	False
ordinary-part-set	20000	0.0027311404278140827	21.0	21.0	866.0	866.0	10894.4809	10852.5448	-0.3849297675119212	10894.58435	10852.3695	-0.38748472308627324	False
dot-three-no-input	20000	0.00024600729822132905	596.0	596.0	23504.0	23504.0	254199.4092	254198.94425	-0.00018290758482253366	254198.70575	254198.31315	-0.000154446105005146	False
differences-no-input	20000	0.14238502548915566	13.0	13.0	704.0	704.0	10038.39375	10033.7411	-0.04634855053379239	10031.2228	10035.75595	0.04519040290882348	False
take-multispec-small	20000	0.013839066272813838	109.0	109.0	5856.0	5856.0	49939.56615	49922.4603	-0.03425310093527756	49923.1698	49922.0405	-0.002262075914905015	False
~~~

candidate-03: [candidate-03/perf/paired-results.json](../../../.temp/brief-584/candidate-03/perf/paired-results.json) (raw process ledger), [candidate-03/perf/slopes.tsv](../../../.temp/brief-584/candidate-03/perf/slopes.tsv)

~~~text
label	N	floor	alloc1_base	alloc1_new	bytes1_base	bytes1_new	instructions1_base	instructions1_new	instructions1_pct	instructions2_base	instructions2_new	instructions2_pct	regression
destructuring-symbols	20000	0.0029262873206037376	0.0	0.0	32.0	32.0	7225.71695	7215.8205	-0.13696149556481352	7226.02465	7215.79195	-0.1416089827482203	False
destructuring-held-parts	20000	0.0008277920706234454	20.0	20.0	738.0	738.0	13433.12975	13394.18955	-0.28988181253889467	13433.1853	13394.1104	-0.290883354374627	False
destructuring-nested	20000	0.0016147536009647467	20.0	20.0	738.0	738.0	16302.99475	16296.04095	-0.04265351309150622	16303.0934	16296.0613	-0.04313353194677072	False
ordinary-symbol-set	20000	0.003935365981011518	1.0	1.0	160.0	160.0	4630.99935	4630.6726	-0.007055712499726141	4631.12845	4631.0512	-0.0016680599735918733	False
ordinary-part-set	20000	0.0027311404278140827	21.0	21.0	866.0	866.0	10894.4355	10852.42615	-0.38560373320857266	10894.5507	10852.509	-0.38589659323904346	False
dot-three-no-input	20000	0.00024600729822132905	596.0	596.0	23504.0	23504.0	254199.8854	254198.2598	-0.0006394967477785229	254199.43675	254198.47225	-0.0003794264898160371	False
differences-no-input	20000	0.14238502548915566	13.0	13.0	704.0	704.0	10030.7787	10033.12725	0.023413436486219297	10052.06905	10035.3341	-0.16648264070570074	False
take-multispec-small	20000	0.013839066272813838	109.0	109.0	5856.0	5856.0	49923.2083	49921.9934	-0.002433537509649586	49915.2486	49922.14685	0.01381992516009678	False
~~~

candidate-01 binary hashes:

~~~text
rum-cli 2abe4754b5dabb2cf012d825a3f5ada2a32460d9f7bbb823edb453e0af66f26c
rum-cli-alloc ba6bb9e15ca839e056bf22ec9ab6605d953d807bc3cd18ab28b015e37a1e25be
~~~

candidate-02 binary hashes:

~~~text
rum-cli 744a26bbfc4a6c2de0bea4b46438467ba951a914a84f1a354d18a34d7c7b65d6
rum-cli-alloc 74fce36084c43440bae109bae378264c4c9fce21e5694ce7d42eb882ed6d7912
~~~

candidate-03 binary hashes:

~~~text
rum-cli 75566f4023256c11610f42abaa43f9b7829278f8d8ca09021275dff1b702fbf4
rum-cli-alloc 3bc29c1ecad109c5c8b8ce74221979d38ede2436171d5d1ff2849bbe7ec5c5d5
~~~

### 4. Stop-loss, residuals and handoff

Three measured candidates out of eight allowed. C1 repaired all three G9 sites
and passed 15 fixtures, but nested destructuring regressed 1.298548% and added an
allocation. C2's inline parent restored allocation/byte counts and reduced
regressing labels 1 -> 0 and worst regression to 0; it passed 16 fixtures. C3
fixed the mandatory frame protocol failure and retains all semantic,
structural and performance passes. No two consecutive compiling candidates
stalled. Conservatively counting only the four focal metrics, C3 is one
unchanged candidate (3 repaired sites, 16 fixtures, 0 regressing labels, worst
regression 0), below the two-candidate stop-loss. It also repairs the mandatory
frame fixture. The selected candidate has zero regressing labels and zero
allocation or byte-count increases in the measured panel.

Known adjacent residual: G10 direct Default fallback from brief-583. It was not
edited or claimed repaired. The existing recursive pre-store shape validator
and Expr internals remain outside this flat store-walk contract. The parent
stack spills beyond one inline parent; performance claims are limited to the
specified panel, while the deeper spill path has a functional fixture.
No other adjacent defect was observed. No scope or authorization question remains.

All required gates ran to green; ignored tests and native capability skips are
reported as such above. No git command or network access was used. Changes are
left uncommitted for the director's authoritative gate.

Final scope/hash evidence: [final-audit.json](../../../.temp/brief-584/final-audit.json), [changed-files.json](../../../.temp/brief-584/changed-files.json), [final.patch](../../../.temp/brief-584/final.patch). All task artifacts are under `.temp/brief-584/`; prescribed fixed-path retention generator review/certificate output was copied here.
