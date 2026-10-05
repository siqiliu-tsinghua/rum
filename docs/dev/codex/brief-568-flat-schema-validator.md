# brief-568 — G3: a flat, resumable schema validator

Implementation brief. **Do not run `git` in any form**, including before you
have read this brief. Leave every change uncommitted for the director's gate.

## Goal

`ConformSchema` and `SchemaErrors` (`crates/rum-stdlib/src/schema.rs`) resume
**past** their work when a predicate or a default asks for input, at seven
sites — P2056, P2061, P2065, P2066, P2067, P2068, P2069 — and their validator
walks the value and schema by native recursion, which aborts the process on
the ruled 2 MiB stack (release, one codegen unit: a literal list schema aborts
at 6 000 levels, an association at 5 000; `.temp/brief-567/depth-runs.jsonl`).
Replace the recursion with brief-567's design: one heap-held depth-first work
stack, one mutable path buffer, one suspension frame. Lost continuations
**66 -> 59**.

brief-567's report and `.temp/brief-567/draft-G3-implementation.md` are the
raw material; **this brief governs** where they differ (the depth boundary,
the depth tests, the performance gate).

## Director rulings on brief-567's questions

1. **The depth contract covers the walk the validator owns**: the validation
   loop, path building and rendering, and cloning, tracing, dropping and
   abandoning the validator's own state types. The `Expr`-depth helpers
   brief-567 found outside the module — `Expr`'s destructor, the memory and
   gensym root visitors' recursive `Expr` walks, association-data flattening,
   `Expr`'s `Display` — are existing recorded debt, not G3's; they are not
   touched here and are recorded for the hardening line. A deep test that
   aborts in one of them rather than in the validator is a **STOP with
   evidence**, never a pass, and never a reason to lower n silently.
2. **Performance**: the standard gate, one codegen unit on both arms; a
   residual is reported with its attribution for a director decision.
3. **Inventory**: brief-567's plan (`.temp/brief-567/inventory-plan.md`) is
   authorized exactly as written — listed under Scope.

## Context pointers

- brief-567 and its report; `.temp/brief-567/design.md` (types, loop,
  semantics to preserve), `inventory-plan.md`, `inventory-before.tsv`,
  `perf-plan.md`, `verification.md`, `probe.rs` (the red, control and deep
  tests), `post-fix-depth/` (release programs), `depth-runs.jsonl`.
- The accepted G2 pattern: `crates/rum-stdlib/src/lazy/continuation.rs` (cold
  capture on the error edge, immutable Session-rooted frame, cold working copy
  on an actual resume), `tests/association_depth_contract/lazy_depth_contract.rs`
  (the structural depth checks and falsifiers), `tests/lazy_pull_continuations.rs`.
- `docs/guide/structured-data.md:555-610`; `docs/dev/codex/TEMPLATE.md`
  (standing duty; pitfall 8).

## Scope

**Objective:** the flat validator, its suspension frame and finisher
completion, the structural and empirical depth checks, the owning tests, one
narrow guide paragraph, and the standing and performance gates.

**Edit surface:** `crates/rum-stdlib/src/schema.rs` and new children
`schema/{state,parse,format,association,machine,continuation}.rs` (design.md's
layout; split further only at a named responsibility boundary); new
`crates/rum-stdlib/tests/schema_continuations.rs` (and children by
responsibility, e.g. a depth child); new
`crates/rum-stdlib/src/tests/association_depth_contract/schema_depth_contract.rs`
and its `mod` line; `docs/guide/structured-data.md`, one paragraph saying a
schema callback or default that asks for input resumes into the validation —
no claim about depth beyond the validator's own walk, none about the matcher.
**`scripts/core-language-microbench.sh` stays unchanged**; the schema labels
live as scratch workloads.

**Inventory exceptions, authorized by name** (`inventory-plan.md`): the 41
schema rows mapped as written — the four old data types (`SchemaOptions`,
`SchemaError`, `SchemaEntry`, `SchemaPath`) gain `FrameRootTrace`, the other
data derives and the named captures and macros relocate, the two existing
input frames keep their identity, the key-path `format!` at `schema.rs:363:27`
is removed and the index-path `format!` at `:370:23` becomes a `write!` row in
`schema/format.rs`; the new `FrameRootTrace` types (`SchemaMode`,
`SchemaState`, `SchemaContainer`, `ListWork`, `AssociationWork`,
`SchemaDestination`, `SchemaAwait`, `SchemaValidationSuspensionFrame`) and the
frame's `PureFrameAbandon`; rum-runtime's lists get exactly the 12 root-trace
entries and one abandon entry, and the named-frame count goes **555 -> 556**.
`Unsplit:ConformSchema` and `Unsplit:SchemaErrors` keep class, status, owner
and exclusions; only their evidence sites relocate.

**Standing maintenance duty — always in scope** (`TEMPLATE.md`): replay the 68
fingerprints and both certificates, reconcile every ownership inventory from
discovery, disclose every changed row. **STOP** on any other existing row
change or disappearance, any new `UnknownHostPayload`, any self-reaching
schema type, any new row in untouched code, or any retention reclassification.

**Fence:** no rum-runtime or rum-core production change, no matcher change, no
`Expr`, serde or public representation change, no other suspension group, no
dependency, no lease. No edits to `AGENTS.md`, `TODO.md`, root `PROGRESS.md`,
earlier briefs or other docs.

**Disclosure duty:** every changed file and moved item, every frame and state
field with its roots, every changed inventory row, every measured label, and
any adjacent defect.

## Deliverable

As design.md specifies, with every observable semantic preserved — output
shape and order, error order, paths and messages, Optional laziness, defaults
evaluated only when the key is missing and before their validation, strict
extra keys after the specified ones, malformed-schema and `Eval` abort
precedence, full traversal under the 64-error presentation cap,
`MatchQ`'s True-only interpretation with the original leaf value kept. One
mutable path buffer with byte-length checkpoints replaces the per-level path
copies (an O(n^2) cost today). Capture only on `InputRequired`, on the error
edge; no continuation allocation or clone on a run that never suspends; a cold
working copy on an actual `Ok` resume; a second suspension freezes the new
state; abandonment runs no skipped validation, default or message. Both
finishers complete through one shared helper on the ordinary and resumed paths.

## Tests

Red first, recorded on the incoming tree before any production edit: brief-567's
scratch suite (`.temp/brief-567/probe.rs`) moved into
`schema_continuations.rs` — the nine unchanged brief-548 G3 witnesses and the
eleven further semantic cases through the rum-vs-rum oracle (value, `log`,
errors and termination, messages, prints, actual suspension, exact prompts,
pending request identity, no owner after, a working `1 + 1`), the
cancellation and control cases, and the structural checks. Pre-fix: 20
semantic reds, one structural red, the rest controls.

**Structural depth checks** in `schema_depth_contract.rs`, each with a
falsifier that is seen to fire: the recursion scanner over every schema source
file individually and as one combined unit reports no recursive edge (direct
and cross-file falsifiers); no `rum_stdlib::schema::` row in
`ownership_graph/inventory/self_reaching.tsv` (injected-row falsifier); a
root-completeness test visits every `Expr`-bearing field of the state and
frame.

**Depth tests — run controls first, never lower n silently:**

- in-process, on a `2 * 1024 * 1024` thread that also owns the Session's
  teardown: construction controls at **list 1 000 / association 500** (they
  pass today), then `ConformSchema` exact result, `SchemaErrors` empty result,
  and `SchemaErrors` on a terminal scalar mismatch with the exact full path,
  for both container kinds; plus a deep suspension, a second suspension and an
  abandonment at the same depths. Assert every level and the leaf; never
  format a deep `Expr` in an assertion message;
- release, through the candidate one-unit `rum-cli` under
  `bash -c 'ulimit -s 2048; ulimit -c 0; …'`, stdin `/dev/null`, each program
  run alone: construction controls, then the same three validations at **list
  6 000 / association 5 000** — the depths at which construction passes today
  and the validator aborts (brief-567's `post-fix-depth/` programs, with n set
  to these depths). Do not run any validation pre-fix; the pre-fix evidence is
  complete.

Every existing test stays as it is and green.

## rum / Rust pitfalls (read twice)

1. An ordinary error is not `InputRequired` and creates no frame.
2. Share `Expr`/`SharedArgs`; the state owns its buffers, not a borrowed view
   of itself; no stage or container reference survives an evaluation.
3. One change per measured candidate.
4. Stop-loss is keyed on progress: failing tests, remaining Lost sites,
   regressing labels, the worst regression. Stop after two consecutive
   compiling candidates that move none of them. Falling compile-error counts
   are progress before the tree compiles.
5. No recursion as a fallback, no depth cap, no fail-closed result counted as
   a depth pass.
6. **Both performance arms use one codegen unit** (`TEMPLATE.md` pitfall 8).

## Verification (MANDATORY — run to green or evidence)

Serial, through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-568/tmp …`. Bounds from measured serial history: a
focused test target 150 s; `ownership_graph`, `local_production_acceptance`
(either feature set) and `cargo test -p rum-runtime` 300 s; `cargo test
--workspace --no-fail-fast` 2 400 s (observed 651-1 012 s); a one-unit release
build 600 s (observed 98-105 s), checking the binary's hash changed; a release
depth program 60 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test schema_continuations
cargo test -p rum-stdlib --lib schema_depth_contract
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test -p rum-runtime
cargo test --workspace --no-fail-fast
```

plus the release depth programs, each alone. Report native ZeroMQ skips as
skips.

**Performance gate.** Base: `.temp/brief-568/base/rum-cli` and
`rum-cli-alloc` — the incoming tree built with one codegen unit by the
director (hashes in `.temp/brief-568/base/sha256.txt`); do not rebuild them.
Candidates: the working tree built with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`
into a fresh target directory each, default and `--features
rum-cli/alloc-count`. Panel: the 18 schema labels of
`.temp/brief-567/perf-plan.md` as scratch workloads (`Do[<exact statement>,
{i, 1, N}]` at N and 2N, setup outside the loop, each workload's result checked
first), plus the neighbours it names (`list-literal-small`,
`association-small-literal-repeat`, `matchq-integer-success`,
`matchq-integer-fail`, `table-small`, `table-iterator-null`) and the controls
`dot-three-no-input`, `differences-no-input`, `take-multispec-small` from
`.temp/brief-563/perf/`. Allocation counts first, then two paired samples of
`taskset -c 2 perf stat -x, -e cpu_core/instructions/u`, one label per
process, stdin `/dev/null`, `(count(2N) - count(N)) / N`, raw output saved
before parsing. A label regresses when **both** samples exceed +0.05%. Report
every row. No timing suite. At most six measured candidates.

## Checkpointing — protect the scene

Everything under `.temp/brief-568/`. After each milestone update
`.temp/brief-568/PROGRESS.md`: what is done and verified, the exact next
action, whether the tree compiles, half-edited files, the counters, and any
depth or dependency blocker.

## Hard rules

- **No `git`, in any form.**
- No network; nothing outside this repository; all scratch, `.rum` programs,
  `TMPDIR` and targets under `.temp/brief-568/`.
- Commands: scoped reads and edits, `rg`/`grep`, `python3`, bounded serial
  Cargo, the release `rum-cli`, `taskset`/`perf`, `valgrind --tool=callgrind`.
- Scope is the objective, the fence and the disclosure duty; STOP and report
  before editing past the fence.
- **Testing Policy is a hard line** (`AGENTS.md`).
- No new `unsafe`; never store a `Session` borrow, the cancellation observer
  or a `NativeStackDepth` in a frame.
- Paste real output; say plainly what did not run or did not pass.

## Report format (append to this brief)

1. Files, items and fields changed or moved, roots, module sizes.
2. Verbatim red and green output: the oracle suite, cancellation and controls,
   structural checks and falsifiers, in-process and release depth (exact
   results, exit statuses), the broad suites; every inventory row change.
3. Lost continuations before and after (66 -> 59 predicted).
4. Every performance row for base and every candidate, both samples.
5. Stop-loss counters, residuals with attribution, open questions.

---

## Execution report — brief 568 (2026-10-05; STOP, incomplete)

**Stopped on an unauthorized ownership row.** The designed owned association
index introduces an `UnknownConstruct` storage row for `HashMap`'s implicit
`RandomState` hasher. The authorized `.temp/brief-567/inventory-plan.md` says:

> No new UnknownHostPayload, unknown storage or self-reaching schema row is authorized.

No representation substitution or inventory waiver was made. The new storage
row is **not admitted**. Separately, the required request-level list
`ConformSchema` depth 1,000 fails closed with `StackLimit`; the direct validator
at the same depth/stack returns the exact result and tears down successfully.
Neither result is presented as a passing public depth gate.

No git, network, unsafe, dependency, matcher change, runtime/core production
change, lease, external-engine comparison, depth reduction or disabled guard
was used. Every change remains uncommitted. All scratch generated by this run
is under `.temp/brief-568/`.

### 1. Files, items, fields, roots and sizes

The snapshot/hash audit is at `.temp/brief-568/scope-audit.json`; eight existing
source/inventory/guide files changed and nine new source/test files were added.
This report is the additional authorized edit to this brief. No unrelated
source change was detected. The temporary unauthorized generated storage row
was restored to the incoming inventory bytes before this report.

| File | Characters | Change |
| --- | ---: | --- |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` | 30,342 | Only named frame count 555 -> 556. |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | 169,908 | Exactly 12 root-trace registrations and one PureFrameAbandon registration. |
| `crates/rum-stdlib/src/schema.rs` | 5,619 | Entry points and the two existing first-argument frames; shared driver/finisher entry. |
| `crates/rum-stdlib/src/schema/association.rs` | 4,701 | Association entry/index selection, completion slots, strict extras and ordered output. |
| `crates/rum-stdlib/src/schema/continuation.rs` | 2,623 | One immutable SchemaValidationSuspensionFrame, cold capture and cold working copy on Ok resume. |
| `crates/rum-stdlib/src/schema/format.rs` | 4,149 | One path buffer, errors, rendering, malformed/Eval completion and shared successful finisher. |
| `crates/rum-stdlib/src/schema/machine.rs` | 7,085 | Iterative Visit/Next/ChildDone descent/ascent and owned default/leaf evaluation. |
| `crates/rum-stdlib/src/schema/parse.rs` | 4,216 | Call/options, association/list and Optional parsing; EntrySpec validation_spec. |
| `crates/rum-stdlib/src/schema/state.rs` | 3,492 | Flat records, pending phase and live-only tasks/errors; crate visibility supports owning root tests. |
| `crates/rum-stdlib/src/tests/association_depth_contract/mod.rs` | 379 | Register the schema depth contract child. |
| `crates/rum-stdlib/src/tests/association_depth_contract/schema_depth_contract.rs` | 9,363 | Per-file/combined recursion checks, falsifiers, root completeness/flat state tests and direct-handler attribution. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | 246,920 | Discovery-derived authorized relocation/addition/removal rows; complete changes below. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 419,349 | Discovery-derived authorized relocation/addition/removal rows; complete changes below. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 395,802 | Discovery-derived authorized relocation/addition/removal rows; complete changes below. |
| `crates/rum-stdlib/tests/schema_continuations.rs` | 12,975 | Promoted unchanged 20 semantic witnesses, two no-input controls and two cancellation controls. |
| `crates/rum-stdlib/tests/schema_continuations/depth.rs` | 11,992 | Construction, exact conformance/errors/paths, shallow controls and deep lifecycle cases on 2 MiB threads. |
| `docs/guide/structured-data.md` | 24,143 | One paragraph specifying once-only continuation of predicates/defaults after input. |

All seven production schema modules are below 10k characters. The former
19,179-character implementation is split by the authorized responsibilities.
The integration test file keeps the cohesive oracle cases together; depth has
its own child. This adds the explicitly named test/module files without
regrouping unrelated tests in the existing large test directory.

Items moved/replaced:

- `SchemaOptions`, `SchemaError`, `SchemaEntry`, `EntrySpec`, `SchemaPath` and
  `SchemaAbort`, plus `From<EvalError>`, move to `state.rs`. The four authorized
  old data types gain root tracing; existing traits are preserved.
- `parse_schema_call`, `parse_options`, `option_rule`,
  `parse_association_schema`, `parse_list_schema`, `parse_entry_spec` and
  `EntrySpec::validation_spec` move to `parse.rs`.
- `SchemaValidator` and its borrowed Session/native-recursive conform methods
  are replaced by `schema_drive` and association transitions. The list and
  association shape predicates move to `machine.rs`. Input items use
  `SharedArgs`; schema/value Expr clones share call subtrees.
- `SchemaPath::key/index` become mutable append operations with byte-length
  checkpoints and truncate. The copied key `format!` is removed; the copied
  index `format!` becomes the authorized `write!`. Only errors copy a path.
- `malformed`, error rendering/capping/message helpers, key/value rendering,
  `fail_schema`, `finish_abort`, and error accumulation move to `format.rs`.
  `finish_leaf` preserves True-only interpretation and the original value;
  `finish_validation` is shared by normal and resumed completion.
- Association entry parsing, the >16 index threshold, the four named closures,
  matched/slot/default buffers, extra-key checks and output ordering move to
  `association.rs`. The stored index uses owned String keys, as designed.
- Both original input-value frames remain in the facade with unchanged
  fields/derives/resume closures. New capture is only on InputRequired;
  ordinary errors create no frame. A cold Ok resume clones the flat state;
  Err forwards without a copy. The driver imports no continuation module.
- The live-only `SchemaTask::Default` phase separates association child
  selection from evaluation so no container reference crosses a callback.
  It adds no frozen type/derive/frame. The other live phases are Visit, Next
  and ChildDone. `SchemaDriveError` carries an abort and optional pending phase.

Field/root disclosure (no Session borrow, observer, NativeStackDepth, closure,
parent/child state link or interior mutability is stored):

| Type | Fields | Expr roots |
| --- | --- | --- |
| `SchemaOptions` | allow_extra_keys | none |
| `SchemaError` | path, message | none |
| `SchemaEntry` | key, spec | spec |
| `EntrySpec` (live only) | Required(spec); Optional(spec, default) | spec and optional default; never directly frozen |
| `SchemaPath` | rendered | none |
| `SchemaAbort` (live only) | Malformed(path, message); Eval(error) | EvalError is propagated; never stored in the frame |
| `SchemaMode` | Errors / Conform | none |
| `SchemaState` | mode, options, stack, path, errors | all container roots in stack |
| `SchemaContainer` | List(work) / Association(work) | variant's roots |
| `ListWork` | items, item_schema, next, output, path_len, waiting | every shared input item, item_schema, every completed output |
| `AssociationWork` | schema_entries, value_entries, next_schema, matched, conformed_by_index, default_entries, positions, path_len, waiting | every schema spec; every input key/value; every completed slot; every default key/value |
| `SchemaDestination` | ListItem; ExistingEntry(index); DefaultEntry(key) | none; key is a String until committed |
| `SchemaAwait` | Leaf(value, schema); Default(spec) | original leaf value/schema, or pending default validation spec |
| `SchemaValidationSuspensionFrame` | state, pending | both transitive root sets |
| `EvalSchemaErrorsSuspensionFrame`, `EvalConformSchemaSuspensionFrame` | args | all held arguments; unchanged |
| `SchemaTask` (live only) | Visit(value, schema); Default(value, schema); Next; ChildDone(value) | owned values/specs while running; never frozen |
| `SchemaDriveError` (live only) | abort, pending | optional pending phase only transferred on InputRequired |

The HashMap index has no Expr roots, but discovery cannot establish its
implicit hasher representation. That is the blocking inventory obligation,
not an omitted Expr root or a newly admitted host payload.

Runtime closed-list additions are exactly:

```text
FrameRootTrace:
crates/rum-stdlib/src/schema/state.rs:AssociationWork
crates/rum-stdlib/src/schema/state.rs:ListWork
crates/rum-stdlib/src/schema/state.rs:SchemaAwait
crates/rum-stdlib/src/schema/state.rs:SchemaContainer
crates/rum-stdlib/src/schema/state.rs:SchemaDestination
crates/rum-stdlib/src/schema/state.rs:SchemaEntry
crates/rum-stdlib/src/schema/state.rs:SchemaError
crates/rum-stdlib/src/schema/state.rs:SchemaMode
crates/rum-stdlib/src/schema/state.rs:SchemaOptions
crates/rum-stdlib/src/schema/state.rs:SchemaPath
crates/rum-stdlib/src/schema/state.rs:SchemaState
crates/rum-stdlib/src/schema/continuation.rs:SchemaValidationSuspensionFrame
PureFrameAbandon:
crates/rum-stdlib/src/schema/continuation.rs:SchemaValidationSuspensionFrame
named-frame assertion: 555 -> 556
```

The closed-list tests themselves have not run after the stop.

### 2. Verification and inventory evidence

All subprocesses ran serially through `scripts/run-bounded.py`, with
`CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true`, TMPDIR and
CARGO_TARGET_DIR under `.temp/brief-568/`, and stdin `/dev/null`. Exact commands,
exit codes and bookkeeping durations are in each `logs/<label>.json`; complete
verbatim output is in the adjacent `.txt`. Durations were not used as performance
evidence. Core dumps were disabled before candidate depth validations.

The semantic strings were promoted unchanged from brief-567. The complete
rum-vs-rum records include values, logs, errors/termination, messages, prints,
actual suspension, exact prompts and pending identity, no owner afterward and
working `1+1`. Full raw oracle output is preserved in
`.temp/brief-568/logs/red-semantics.txt` and
`.temp/brief-568/logs/candidate-1-semantics.txt`; excerpts below are verbatim.

**red-semantics** — exit 101. Full log: `.temp/brief-568/logs/red-semantics.txt`.
```text
test cancel_default ... CANCEL default=true (View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }, "List[1]", ["default: "])
test cancel_predicate ... CANCEL default=false (View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }, "List[1]", ["predicate: "])
    oracle_allow_extra
    oracle_capped_errors_continue
    oracle_conformschema_association
    oracle_conformschema_default
    oracle_conformschema_default_predicate
    oracle_conformschema_leaf
    oracle_default_then_predicate
    oracle_error_after_resume
    oracle_error_prefix
    oracle_eval_error_after_resume
    oracle_malformed_after_resume
    oracle_nested_assoc_list_assoc
    oracle_positions_distinct
    oracle_predicate_twice
    oracle_schema_defaults_twice
    oracle_schema_twice
    oracle_schemaerrors_association
    oracle_schemaerrors_default
    oracle_schemaerrors_leaf
    oracle_strict_extra
test result: FAILED. 8 passed; 20 failed; 6 ignored; 0 measured; 0 filtered out; finished in 0.27s
```

**red-structural** — exit 101. Full log: `.temp/brief-568/logs/red-structural.txt`.
```text
test tests::association_depth_contract::schema_depth_contract::structural_no_schema_recursion ... SCHEMA_RECURSION [RecursiveEdge { path: "/home/remi/dev/rum/crates/rum-stdlib/src/schema.rs", caller: "SchemaValidator::conform_association", callee: "SchemaValidator::conform_value", count: 2 }, RecursiveEdge { path: "/home/remi/dev/rum/crates/rum-stdlib/src/schema.rs", caller: "SchemaValidator::conform_list", callee: "SchemaValidator::conform_value", count: 1 }, RecursiveEdge { path: "/home/remi/dev/rum/crates/rum-stdlib/src/schema.rs", caller: "SchemaValidator::conform_value", callee: "SchemaValidator::conform_association", count: 1 }, RecursiveEdge { path: "/home/remi/dev/rum/crates/rum-stdlib/src/schema.rs", caller: "SchemaValidator::conform_value", callee: "SchemaValidator::conform_list", count: 1 }, RecursiveEdge { path: "schema (all modules)", caller: "SchemaValidator::conform_association", callee: "SchemaValidator::conform_value", count: 2 }, RecursiveEdge { path: "schema (all modules)", caller: "SchemaValidator::conform_list", callee: "SchemaValidator::conform_value", count: 1 }, RecursiveEdge { path: "schema (all modules)", caller: "SchemaValidator::conform_value", callee: "SchemaValidator::conform_association", count: 1 }, RecursiveEdge { path: "schema (all modules)", caller: "SchemaValidator::conform_value", callee: "SchemaValidator::conform_list", count: 1 }]
test tests::association_depth_contract::schema_depth_contract::structural_recursion_falsifiers ... CROSS_MODULE_FALSIFIER [RecursiveEdge { path: "all", caller: "fold_child", callee: "schema_drive", count: 1 }, RecursiveEdge { path: "all", caller: "schema_drive", callee: "fold_child", count: 1 }]
self-reaching schema ownership
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 1087 filtered out; finished in 0.05s
```

**candidate-1-semantics** — exit 0. Full log: `.temp/brief-568/logs/candidate-1-semantics.txt`.
```text
test cancel_default ... CANCEL default=true (View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }, "List[1]", ["default: "])
test cancel_predicate ... CANCEL default=false (View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }, "List[1]", ["predicate: "])
test result: ok. 28 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 0.30s
```

**format-apply** — exit 0. Full log: `.temp/brief-568/logs/format-apply.txt`.
```text
(no stdout/stderr; exit 0)
```

**format-attribution** — exit 0. Full log: `.temp/brief-568/logs/format-attribution.txt`.
```text
(no stdout/stderr; exit 0)
```

**depth-controls** — exit 0. Full log: `.temp/brief-568/logs/depth-controls.txt`.
```text

running 2 tests
test depth::depth_control_association ... BUILT kind=association mode=control n=500
PASSED kind=association mode=control n=500 including teardown
ok
test depth::depth_control_list ... BUILT kind=list mode=control n=1000
PASSED kind=list mode=control n=1000 including teardown
ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out; finished in 0.05s

```

**depth-validation** — exit 101. Full log: `.temp/brief-568/logs/depth-validation.txt`.
```text

running 8 tests
test depth::depth_association_literal ... BUILT kind=association mode=literal n=500
PASSED kind=association mode=literal n=500 including teardown
ok
test depth::depth_control_association ... BUILT kind=association mode=control n=500
PASSED kind=association mode=control n=500 including teardown
ok
test depth::depth_control_list ... BUILT kind=list mode=control n=1000
PASSED kind=list mode=control n=1000 including teardown
ok
test depth::depth_list_literal ... BUILT kind=list mode=literal n=1000

thread 'schema-2mib' panicked at crates/rum-stdlib/tests/schema_continuations/depth.rs:93:13:
assertion `left == right` failed: must complete, not fail closed
  left: Some(StackLimit)
 right: None
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'depth::depth_list_literal' panicked at crates/rum-stdlib/tests/schema_continuations/depth.rs:11:10:
called `Result::unwrap()` on an `Err` value: Any { .. }
FAILED
test depth::depth_mismatch_association ... BUILT kind=association mode=mismatch n=500
PASSED kind=association mode=mismatch n=500 including teardown
ok
test depth::depth_mismatch_list ... BUILT kind=list mode=mismatch n=1000
PASSED kind=list mode=mismatch n=1000 including teardown
ok
test depth::depth_schemaerrors_association ... BUILT kind=association mode=errors n=500
PASSED kind=association mode=errors n=500 including teardown
ok
test depth::depth_schemaerrors_list ... BUILT kind=list mode=errors n=1000
PASSED kind=list mode=errors n=1000 including teardown
ok

failures:

failures:
    depth::depth_list_literal

test result: FAILED. 7 passed; 1 failed; 0 ignored; 0 measured; 32 filtered out; finished in 0.32s

error: test failed, to rerun pass `-p rum-stdlib --test schema_continuations`
```

**direct-depth-attribution** — exit 0. Full log: `.temp/brief-568/logs/direct-depth-attribution.txt`.
```text

running 1 test
test tests::association_depth_contract::schema_depth_contract::schema_direct_list_depth_isolates_the_validator_from_result_evaluation ... DIRECT CONFORM list n=1000 exact; 2 MiB including Session teardown
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1093 filtered out; finished in 0.04s

```

**depth-lifecycle** — exit 101. Full log: `.temp/brief-568/logs/depth-lifecycle.txt`.
```text

running 6 tests
test depth::lifecycle_association_abandonment ... LIFECYCLE BUILT kind=association mode=abandon n=500
LIFECYCLE SUSPEND kind=association mode=abandon n=500 ordinal=1
LIFECYCLE SUSPEND kind=association mode=abandon n=500 ordinal=2

thread 'schema-2mib' panicked at crates/rum-stdlib/tests/schema_continuations/depth.rs:270:22:
pending owner abandoned
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'depth::lifecycle_association_abandonment' panicked at crates/rum-stdlib/tests/schema_continuations/depth.rs:11:10:
called `Result::unwrap()` on an `Err` value: Any { .. }
FAILED
test depth::lifecycle_association_suspension ... LIFECYCLE BUILT kind=association mode=single n=500
LIFECYCLE SUSPEND kind=association mode=single n=500 ordinal=1
LIFECYCLE PASSED kind=association mode=single n=500 including Session teardown
ok
test depth::lifecycle_association_twice ... LIFECYCLE BUILT kind=association mode=twice n=500
LIFECYCLE SUSPEND kind=association mode=twice n=500 ordinal=1
LIFECYCLE SUSPEND kind=association mode=twice n=500 ordinal=2
LIFECYCLE PASSED kind=association mode=twice n=500 including Session teardown
ok
test depth::lifecycle_list_abandonment ... LIFECYCLE BUILT kind=list mode=abandon n=1000
LIFECYCLE SUSPEND kind=list mode=abandon n=1000 ordinal=1
LIFECYCLE SUSPEND kind=list mode=abandon n=1000 ordinal=2

thread 'schema-2mib' panicked at crates/rum-stdlib/tests/schema_continuations/depth.rs:270:22:
pending owner abandoned

thread 'depth::lifecycle_list_abandonment' panicked at crates/rum-stdlib/tests/schema_continuations/depth.rs:11:10:
called `Result::unwrap()` on an `Err` value: Any { .. }
FAILED
test depth::lifecycle_list_suspension ... LIFECYCLE BUILT kind=list mode=single n=1000
LIFECYCLE SUSPEND kind=list mode=single n=1000 ordinal=1

thread 'schema-2mib' panicked at crates/rum-stdlib/tests/schema_continuations/depth.rs:283:13:
assertion failed: result.termination.is_none()

thread 'depth::lifecycle_list_suspension' panicked at crates/rum-stdlib/tests/schema_continuations/depth.rs:11:10:
called `Result::unwrap()` on an `Err` value: Any { .. }
FAILED
test depth::lifecycle_list_twice ... LIFECYCLE BUILT kind=list mode=twice n=1000
LIFECYCLE SUSPEND kind=list mode=twice n=1000 ordinal=1
LIFECYCLE SUSPEND kind=list mode=twice n=1000 ordinal=2

thread 'schema-2mib' panicked at crates/rum-stdlib/tests/schema_continuations/depth.rs:283:13:
assertion failed: result.termination.is_none()

thread 'depth::lifecycle_list_twice' panicked at crates/rum-stdlib/tests/schema_continuations/depth.rs:11:10:
called `Result::unwrap()` on an `Err` value: Any { .. }
FAILED

failures:

failures:
    depth::lifecycle_association_abandonment
    depth::lifecycle_list_abandonment
    depth::lifecycle_list_suspension
    depth::lifecycle_list_twice

test result: FAILED. 2 passed; 4 failed; 0 ignored; 0 measured; 34 filtered out; finished in 0.45s

error: test failed, to rerun pass `-p rum-stdlib --test schema_continuations`
```

**ownership-discovery** — exit 0. Full log: `.temp/brief-568/logs/ownership-discovery.txt`.
```text
test tests::ownership_graph::discovery_report_without_inventory_input ... UnknownHostPayload count: 63
UnknownConstruct count: 7597
UnknownHostPayload breakdown: 6 storage edges; 57 public input sites
Self-reaching type count: 59
Production files: 646; test-only files: 409; nominal definitions: 1621; aliases: 40; owning edges: 3688; resolved Drop owners: 18
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1093 filtered out; finished in 5.89s
```


Interpretation and limits:

- Red-first totals match the brief: 20 semantic reds plus one structural red,
  with 11 passing controls across the two suites; six deep validation tests
  were intentionally ignored before the fix.
- The first candidate compiled and all 20 semantic reds turned green. Its
  28/28 run still ignored the six post-fix depth tests; it is not a final
  all-depth green run. Cancellation and shallow/construction controls passed.
- The later depth run enabled all six validations without lowering depths.
  Five validations passed; list ConformSchema was the one failure. Two repeated
  controls account for the seven passes in that run. Exact error paths were
  `[1]` repeated 1,000 times and 500 `k` segments joined by dots.
- The direct-handler attribution test passes exact list conformance at 1,000
  and includes teardown on the 2 MiB thread. The request-level failure occurs
  outside that successful handler. Source suggests the fixed-point result
  stability check: `resolution.rs:349/401`, `eval_core/stability.rs:30`,
  `evaluation_cache/stability.rs:323`, then `eval_core/mod.rs:107`. This is
  source-based attribution, not an instrumented stack trace. No runtime fix or
  depth-contract weakening was attempted.
- Deep association single/second suspension passed, including exact result
  traversal and teardown. List single/second reached their prompts but failed
  the completion assertion. Those lifecycle failures were not printed with
  their termination variant in the first run; the assertion now reports it.
- Both new abandonment tests initially called a ticket-specific cancellation
  finisher that returned None for this request setup. They were corrected to
  the existing controls' `cancel(); resume_eval_request_with_input(...)`
  abandonment path. That final test-only correction is **unverified** because
  the subsequent inventory stop prevents further gate work.
- The new root-completeness/falsifier and flat clone/trace/drop tests compiled
  in the successful unit build but were **not run**. The candidate recursion
  checks/falsifiers have not been rerun. No self-reaching result is accepted
  on the stale inventory: fresh discovery found the blocker first.

The following mandatory gates remain **unrun**, not green: final
`cargo fmt --check`; complete enabled `schema_continuations`; complete
`schema_depth_contract`; full `ownership_graph` and exactness in both feature
sets; 68 fingerprint and C01/C02 replay; both `local_production_acceptance`
modes; `cargo test -p rum-runtime`; `cargo test --workspace --no-fail-fast`;
both one-unit candidate release builds; all release construction/validation
depth programs; the allocation/instruction panel. No native ZeroMQ test ran,
so there are no observed ZeroMQ skips to report.

**Ownership reconciliation.** Discovery ran without inventory input. The full
row review is `.temp/brief-568/ownership-reconciliation.json`. The initial
scratch reconciliation script had a comma-spacing normalization error; this
was fixed without a source or classification change. The next reconciliation
matched every authorized old schema row, but erroneously permitted the new
storage UnknownConstruct in edited code. Inspection immediately found the
more specific prohibition; that one generated inventory row was restored to
the incoming bytes, the script now rejects it before writes, and work stopped.
No unapproved storage classification remains admitted.

| Inventory | Incoming | Discovered | Maintained disposition |
| --- | ---: | ---: | --- |
| host_payloads.tsv | 63 | 63 | unchanged |
| unknown_attributes.tsv | 1,445 | 1,453 | 8 relocated/authorized re-identifications; 8 authorized new derives |
| unknown_captures.tsv | 2,938 | 2,938 | 11 authorized relocations |
| unknown_macros.tsv | 3,114 | 3,117 | 21 authorized relocations/re-identification; 1 authorized removal; 4 new macros |
| unknown_storage.tsv | 88 | 89 | **STOP**; kept at incoming 88 rows |
| self_reaching.tsv | 59 | 59 | byte-identical; no schema row |

Every changed ownership row follows. BEFORE/AFTER are copied from actual
incoming/discovery cells; nothing is reclassified. The sole NOT ADMITTED row
is disclosed separately.

**unknown_attributes.tsv**

```text
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	15	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default , PartialEq , Eq)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	6	1	unresolved attribute/expansion derive (Clone , Copy , Debug , Default , PartialEq , Eq , FrameRootTrace)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	20	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	11	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , FrameRootTrace)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	26	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	17	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , FrameRootTrace)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	32	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	23	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	38	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	29	1	unresolved attribute/expansion derive (Clone , Debug , PartialEq , Eq , FrameRootTrace)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	43	1	unresolved attribute/expansion derive (Debug)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	34	1	unresolved attribute/expansion derive (Debug)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	585	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema.rs	134	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	606	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema.rs	155	1	unresolved attribute/expansion derive (rum_runtime :: kernel_api :: FrameRootTrace , rum_runtime :: kernel_api :: PureFrameAbandon)
ADDED	UnknownConstruct	crates/rum-stdlib/src/schema/continuation.rs	13	1	unresolved attribute/expansion derive (FrameRootTrace , PureFrameAbandon)
ADDED	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	40	1	unresolved attribute/expansion derive (Clone , Copy , FrameRootTrace)
ADDED	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	55	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
ADDED	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	64	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
ADDED	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	70	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
ADDED	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	80	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
ADDED	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	93	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
ADDED	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	100	1	unresolved attribute/expansion derive (Clone , FrameRootTrace)
```

**unknown_captures.tsv**

```text
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	211	19	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/association.rs	33	15	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	215	33	closure captures: |(index , (key , _))| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/association.rs	37	29	closure captures: |(index , (key , _))| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	215	77	closure captures: |key| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/association.rs	38	45	closure captures: |key| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	227	31	closure captures: |(key , _)| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/association.rs	69	27	closure captures: |(key , _)| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	427	18	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/parse.rs	64	18	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	427	50	closure captures: |symbol| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/parse.rs	64	50	closure captures: |symbol| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	436	21	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/parse.rs	73	21	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	440	14	closure captures: |(key , spec)| (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/parse.rs	77	14	closure captures: |(key , spec)| (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	554	25	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/format.rs	123	25	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	599	27	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema.rs	148	27	closure captures: || (capture types/modes unresolved)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	620	27	closure captures: || (capture types/modes unresolved)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema.rs	169	27	closure captures: || (capture types/modes unresolved)
```

**unknown_macros.tsv**

```text
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	66	21	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 74:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema.rs	29	21	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 37:10)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	117	21	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 125:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema.rs	75	21	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 83:10)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	202	35	unexpanded macro format! (ends at 202:89)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/association.rs	24	13	unexpanded macro format! (ends at 24:67)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	206	27	unexpanded macro vec! (ends at 206:59)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/association.rs	28	19	unexpanded macro vec! (ends at 28:51)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	207	38	unexpanded macro vec! (ends at 207:69)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/association.rs	29	30	unexpanded macro vec! (ends at 29:61)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	291	35	unexpanded macro format! (ends at 291:82)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/machine.rs	33	29	unexpanded macro format! (ends at 33:77)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	312	17	unexpanded macro format! (ends at 316:18)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/format.rs	51	13	unexpanded macro format! (ends at 55:14)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	327	17	unexpanded macro vec! (ends at 327:52)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/machine.rs	55	25	unexpanded macro vec! (ends at 55:60)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	370	23	unexpanded macro format! (ends at 370:60)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/format.rs	27	9	unexpanded macro write! (ends at 27:48)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	386	20	unexpanded macro format! (ends at 388:10)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/parse.rs	23	20	unexpanded macro format! (ends at 25:10)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	399	24	unexpanded macro format! (ends at 399:76)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/parse.rs	36	24	unexpanded macro format! (ends at 36:76)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	402	24	unexpanded macro format! (ends at 402:64)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/parse.rs	39	24	unexpanded macro format! (ends at 39:64)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	444	21	unexpanded macro format! (ends at 447:22)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/parse.rs	81	21	unexpanded macro format! (ends at 84:22)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	466	13	unexpanded macro format! (ends at 469:14)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/parse.rs	106	13	unexpanded macro format! (ends at 109:14)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	493	13	unexpanded macro format! (ends at 496:14)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/parse.rs	133	13	unexpanded macro format! (ends at 136:14)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	502	5	unexpanded macro matches! (ends at 502:90)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/machine.rs	165	5	unexpanded macro matches! (ends at 165:90)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	506	5	unexpanded macro matches! (ends at 506:83)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/machine.rs	169	5	unexpanded macro matches! (ends at 169:83)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	524	24	unexpanded macro vec! (ends at 527:6)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/format.rs	93	24	unexpanded macro vec! (ends at 96:6)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	538	18	unexpanded macro format! (ends at 538:77)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/format.rs	107	18	unexpanded macro format! (ends at 107:77)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	547	9	unexpanded macro format! (ends at 547:53)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/format.rs	116	9	unexpanded macro format! (ends at 116:53)
BEFORE	UnknownConstruct	crates/rum-stdlib/src/schema.rs	575	13	unexpanded macro format! (ends at 575:76)
AFTER	UnknownConstruct	crates/rum-stdlib/src/schema/format.rs	144	13	unexpanded macro format! (ends at 144:76)
REMOVED (authorized)	UnknownConstruct	crates/rum-stdlib/src/schema.rs	363	27	unexpanded macro format! (ends at 363:63)
ADDED	UnknownConstruct	crates/rum-stdlib/src/schema/continuation.rs	30	9	unexpanded macro rum_runtime :: resume_or_suspend! (ends at 30:108)
ADDED	UnknownConstruct	crates/rum-stdlib/src/schema/machine.rs	38	29	unexpanded macro unreachable! (ends at 38:43)
ADDED	UnknownConstruct	crates/rum-stdlib/src/schema/machine.rs	111	29	unexpanded macro unreachable! (ends at 111:43)
ADDED	UnknownConstruct	crates/rum-stdlib/src/schema/machine.rs	124	60	unexpanded macro unreachable! (ends at 124:74)
```

**unknown_storage.tsv**

```text
NOT ADMITTED (STOP)	UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	88	27	rum_stdlib::schema::state::AssociationWork.positions/std::option::Option[Own]/arg0/std::collections::HashMap[Own]/arg2: foreign default hasher representation std::collections::hash_map::RandomState
```


All other ownership inventories are unchanged. The four old data derives are
mapped by their authorized type identities; all other moved derives and named
captures preserve their reasons. All existing non-schema production rows were
required to be literally unchanged by the reconciliation.

**Retention maintenance is incomplete.** The 68 local-contract fingerprints
and C01/C02 cells were not replayed and are stale. Both schema obligations
remain Unsplit/Pending with their existing declaration owners and exclusions;
their evidence cells still point to the old source positions and have not been
relocated. No retention reclassification or certificate claim was made. These
are explicit handoff blockers, not a maintained green tree.

### 3. Lost continuations

Incoming count: **66**, from the governing brief. Named-site repair accounting
removes the seven specified losses: P2056/P2061 now complete through the shared
finisher; P2065/P2067/P2068 have explicit parent destinations and work cursors;
P2066 freezes the pending default spec; P2069 freezes the leaf value/schema and
applies the completed MatchQ result once. All 20 prescribed semantic reds passed
on the first candidate. This gives **59 by named-site accounting**. A global or
authoritative inventory rescan was not run; the director's count remains to be
confirmed.

### 4. Performance

The director's preserved base binaries were not rebuilt. Actual SHA-256:

```text
74db20192d34707bd71a37486140004edb5ee032ea4d291cb88cdca2f4a32009  .temp/brief-568/base/rum-cli
572222df9775a3237f4e6c5754cd793ac8ebb684e243326bd0bf73542f98a124  .temp/brief-568/base/rum-cli-alloc
```

**No performance label was measured and no candidate release binary was built.**
There are no allocation slopes, hardware instruction samples, candidate hashes
or performance acceptance claims. The script `core-language-microbench.sh` is
byte-identical to the incoming snapshot. No timing suite ran. Both required
samples for every planned label remain unrun:

| Label | Allocation base/candidate | Instructions sample 1 base/candidate | Instructions sample 2 base/candidate |
| --- | --- | --- | --- |
| `schema-conform-list-small` | not run | not run | not run |
| `schema-errors-list-small` | not run | not run | not run |
| `schema-conform-association-defaults-small` | not run | not run | not run |
| `schema-errors-association-defaults-small` | not run | not run | not run |
| `schema-conform-list-depth-8` | not run | not run | not run |
| `schema-errors-list-depth-8` | not run | not run | not run |
| `schema-conform-list-depth-32` | not run | not run | not run |
| `schema-errors-list-depth-32` | not run | not run | not run |
| `schema-conform-list-depth-128` | not run | not run | not run |
| `schema-errors-list-depth-128` | not run | not run | not run |
| `schema-conform-association-depth-8` | not run | not run | not run |
| `schema-errors-association-depth-8` | not run | not run | not run |
| `schema-conform-association-depth-32` | not run | not run | not run |
| `schema-errors-association-depth-32` | not run | not run | not run |
| `schema-conform-association-depth-128` | not run | not run | not run |
| `schema-errors-association-depth-128` | not run | not run | not run |
| `schema-conform-association-index-33` | not run | not run | not run |
| `schema-errors-association-index-33` | not run | not run | not run |
| `list-literal-small` | not run | not run | not run |
| `association-small-literal-repeat` | not run | not run | not run |
| `matchq-integer-success` | not run | not run | not run |
| `matchq-integer-fail` | not run | not run | not run |
| `table-small` | not run | not run | not run |
| `table-iterator-null` | not run | not run | not run |
| `dot-three-no-input` | not run | not run | not run |
| `differences-no-input` | not run | not run | not run |
| `take-multispec-small` | not run | not run | not run |

### 5. Stop-loss, residuals and director handoff

- One production candidate compiled on its first attempt. Semantic reds moved
  20 -> 0. No-progress compiling candidate counter: **0**. Measured candidates:
  **0/6**. Stop-loss did not trigger.
- Required fixed-depth validations: **5 pass / 1 failure**; the failing public
  list result is not replaced by the passing direct-handler attribution test.
- Deep lifecycle first run: **2 pass / 4 failures**, including two now-corrected
  test harness errors awaiting verification. Root/structural candidate checks
  remain unrun. No process-aborting depth failure occurred in this run.
- Hard stop: the designed owned HashMap introduces the unapproved hasher
  storage row. Director authorization or an explicitly approved representation
  alternative is required before resuming.
- Adjacent defect: public result evaluation fails closed on the rebuilt
  1,000-level list even though the schema-owned walk succeeds. The fixed-depth
  requirement remains red; runtime production is fenced out.
- No measured performance residual can be attributed because the performance
  gate did not run. Source-level absence of eager continuation capture is not
  presented as cost evidence.

Resume from `.temp/brief-568/PROGRESS.md`. First settle the storage-row and
external result-depth decisions. Then rerun the corrected abandonment tests,
finish discovery/exactness and all standing maintenance, run the structural
root/falsifier gates, and complete the remaining mandatory/release/performance
panel. Production is not half-edited and the last production/unit builds passed;
the final integration test-only correction is unverified. This is an incomplete,
uncommitted implementation for director review, **not a ready-to-merge result**.


## Director resume note (2026-10-05) — read this first, then resume from `.temp/brief-568/PROGRESS.md`

Your stop was correct on both counts; here are the rulings.

1. **The `RandomState` storage row is authorized.** `AssociationWork.positions`
   (`HashMap<String, usize>`, built only above the existing 16-key threshold, as
   the old local `HashMap` was) adds one `UnknownConstruct` row to
   `ownership_graph/inventory/unknown_storage.tsv` ("foreign default hasher
   representation `RandomState`"). It is an `UnknownConstruct` in code this
   brief added — the kind the standing duty expects — with precedent
   (`RewriteRuleExactIndex.first_rule_by_pattern`). inventory-plan.md's
   blanket "no unknown storage row" was over-strict; admit exactly this row,
   from discovery, and disclose it. Any other new storage row still STOPs.
2. **Depth: the result height is the evaluator's contract, not the
   validator's.** A freshly built deep result is walked by the evaluator, whose
   guard fails closed at a height that differs by profile (the director saw
   `ConstantArray[1, Table[1, {1000}]]` fail closed in the one-unit release
   build while `Do[v = {v}, …]` builds 6 000). So:
   - **`SchemaErrors`** (its result is not deep) must return the exact result
     at every required depth — in-process list 1 000 / association 500,
     release list 6 000 / association 5 000 — on the public path. Together with
     your direct-validator tests this proves the walk is depth-safe;
   - **`ConformSchema`** must never abort. At each depth, in the same program
     and profile, first run the control `ConstantArray[1, Table[1, {n}]]` (a
     fresh deep result of the same shape; for associations, the equivalent
     fresh nested association built by a builtin that does not recurse in
     Rust, or `ConstantArray` if none — say which): if the control succeeds,
     `ConformSchema` must return the exact result; if the control fails closed,
     `ConformSchema` must fail closed with the same error. Record which.
   - The direct-validator tests stay, in both modes, exact.
3. Then continue as the brief says: rerun the corrected abandonment tests, the
   structural and root checks with falsifiers, discovery and the standing
   duty, the broad suites, the release depth programs (each alone; no
   validation is run pre-fix), and the full one-unit performance panel.

## Resume execution report — brief 568 (2026-10-05; functional gates verified, performance residuals)

The director resume rulings were applied. The flat validator and its continuation are verified, and the previously blocked RandomState row is admitted exactly as authorized. **The performance gate is not green: 10 of 27 labels exceed +0.05% in both hardware samples.** Every residual was attributed and is submitted for the director decision explicitly allowed by this brief. No further implementation/representation candidate was attempted. All edits remain uncommitted; no git command was run.

### 1. Files, items, fields and roots

| File | Characters | Change |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` | 30342 | Exactly the named-frame count 555 -> 556. |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | 169908 | Exactly 12 FrameRootTrace and one PureFrameAbandon additions. |
| `crates/rum-stdlib/src/schema.rs` | 5619 | Held entry points, original two input frames, shared driver/finisher entry. |
| `crates/rum-stdlib/src/schema/association.rs` | 4701 | Selection, owned >16-key index, defaults and ordered completion. |
| `crates/rum-stdlib/src/schema/continuation.rs` | 2623 | One cold immutable suspension frame and cold working copy. |
| `crates/rum-stdlib/src/schema/format.rs` | 4149 | Mutable path buffer, exact errors and shared completion. |
| `crates/rum-stdlib/src/schema/machine.rs` | 7085 | Iterative depth-first Visit/Next/ChildDone/default driver. |
| `crates/rum-stdlib/src/schema/parse.rs` | 4216 | Call/options/schema and Optional parsing. |
| `crates/rum-stdlib/src/schema/state.rs` | 3492 | Flat typed state, work cursors, destinations and pending phases. |
| `crates/rum-stdlib/src/tests/association_depth_contract/mod.rs` | 379 | Register schema contract tests. |
| `crates/rum-stdlib/src/tests/association_depth_contract/schema_depth_contract.rs` | 11558 | Recursion/root/ownership falsifiers and exact direct tests for both containers and both modes. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | 246920 | Discovery/replay maintenance; every changed row/cell disclosed below. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | 419349 | Discovery/replay maintenance; every changed row/cell disclosed below. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | 395802 | Discovery/replay maintenance; every changed row/cell disclosed below. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv` | 20858 | Discovery/replay maintenance; every changed row/cell disclosed below. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | 91646 | Discovery/replay maintenance; every changed row/cell disclosed below. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/obligations.tsv` | 111755 | Discovery/replay maintenance; every changed row/cell disclosed below. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | 18161 | Discovery/replay maintenance; every changed row/cell disclosed below. |
| `crates/rum-stdlib/tests/schema_continuations.rs` | 12975 | Unchanged promoted semantic witnesses, cancellation and controls. |
| `crates/rum-stdlib/tests/schema_continuations/depth.rs` | 14535 | Fixed-depth public tests, fresh-result controls and deep lifecycle; cancel handle acquired for validation after setup. |
| `docs/guide/structured-data.md` | 24143 | One paragraph specifying continuation of predicates/defaults after input. |

The own-brief append is the additional authorized documentation edit. The snapshot audit found 12 changed existing source/inventory/guide files and nine new source/test files, with no unrelated change. All production schema modules remain under 10k characters. The test children retain their responsibility boundaries; the existing large runtime closed-list files receive only the named registrations/count, because their broader reorganization is outside this brief. `scripts/core-language-microbench.sh` is byte-identical to the original snapshot.

The resumed phase changed no production schema code. It corrected the new depth tests to the director ruling, expanded direct validation to both finishers/container kinds, fixed the cancellation test handle generation, admitted the one storage row and completed the standing inventories. The earlier report's moved-item/field disclosure remains current; it is repeated here for review:

Items moved/replaced:

- `SchemaOptions`, `SchemaError`, `SchemaEntry`, `EntrySpec`, `SchemaPath` and
  `SchemaAbort`, plus `From<EvalError>`, move to `state.rs`. The four authorized
  old data types gain root tracing; existing traits are preserved.
- `parse_schema_call`, `parse_options`, `option_rule`,
  `parse_association_schema`, `parse_list_schema`, `parse_entry_spec` and
  `EntrySpec::validation_spec` move to `parse.rs`.
- `SchemaValidator` and its borrowed Session/native-recursive conform methods
  are replaced by `schema_drive` and association transitions. The list and
  association shape predicates move to `machine.rs`. Input items use
  `SharedArgs`; schema/value Expr clones share call subtrees.
- `SchemaPath::key/index` become mutable append operations with byte-length
  checkpoints and truncate. The copied key `format!` is removed; the copied
  index `format!` becomes the authorized `write!`. Only errors copy a path.
- `malformed`, error rendering/capping/message helpers, key/value rendering,
  `fail_schema`, `finish_abort`, and error accumulation move to `format.rs`.
  `finish_leaf` preserves True-only interpretation and the original value;
  `finish_validation` is shared by normal and resumed completion.
- Association entry parsing, the >16 index threshold, the four named closures,
  matched/slot/default buffers, extra-key checks and output ordering move to
  `association.rs`. The stored index uses owned String keys, as designed.
- Both original input-value frames remain in the facade with unchanged
  fields/derives/resume closures. New capture is only on InputRequired;
  ordinary errors create no frame. A cold Ok resume clones the flat state;
  Err forwards without a copy. The driver imports no continuation module.
- The live-only `SchemaTask::Default` phase separates association child
  selection from evaluation so no container reference crosses a callback.
  It adds no frozen type/derive/frame. The other live phases are Visit, Next
  and ChildDone. `SchemaDriveError` carries an abort and optional pending phase.

Field/root disclosure (no Session borrow, observer, NativeStackDepth, closure,
parent/child state link or interior mutability is stored):

| Type | Fields | Expr roots |
| --- | --- | --- |
| `SchemaOptions` | allow_extra_keys | none |
| `SchemaError` | path, message | none |
| `SchemaEntry` | key, spec | spec |
| `EntrySpec` (live only) | Required(spec); Optional(spec, default) | spec and optional default; never directly frozen |
| `SchemaPath` | rendered | none |
| `SchemaAbort` (live only) | Malformed(path, message); Eval(error) | EvalError is propagated; never stored in the frame |
| `SchemaMode` | Errors / Conform | none |
| `SchemaState` | mode, options, stack, path, errors | all container roots in stack |
| `SchemaContainer` | List(work) / Association(work) | variant's roots |
| `ListWork` | items, item_schema, next, output, path_len, waiting | every shared input item, item_schema, every completed output |
| `AssociationWork` | schema_entries, value_entries, next_schema, matched, conformed_by_index, default_entries, positions, path_len, waiting | every schema spec; every input key/value; every completed slot; every default key/value |
| `SchemaDestination` | ListItem; ExistingEntry(index); DefaultEntry(key) | none; key is a String until committed |
| `SchemaAwait` | Leaf(value, schema); Default(spec) | original leaf value/schema, or pending default validation spec |
| `SchemaValidationSuspensionFrame` | state, pending | both transitive root sets |
| `EvalSchemaErrorsSuspensionFrame`, `EvalConformSchemaSuspensionFrame` | args | all held arguments; unchanged |
| `SchemaTask` (live only) | Visit(value, schema); Default(value, schema); Next; ChildDone(value) | owned values/specs while running; never frozen |
| `SchemaDriveError` (live only) | abort, pending | optional pending phase only transferred on InputRequired |

No Session borrow, cancellation observer, NativeStackDepth, new unsafe, dependency, lease, matcher change, public representation change or runtime/core production change was introduced. The RandomState index adds no Expr roots; its unknown storage identity remains UnknownConstruct.

### 2. Verification and standing maintenance

All Cargo/CLI commands were serial, offline and bounded with CARGO_BUILD_JOBS=1, RUST_TEST_THREADS=1 and TMPDIR under `.temp/brief-568/`. The original-path focused integration tests passed; tests with hard-coded legacy output directories ran from the byte-identical source export `.temp/brief-568/verification-source/`, so their target/brief-544 and .temp/brief-546 artifacts also stayed under this brief. The final audit verifies crate sources and inventories match the working tree. Both release binaries were built at the original source paths into fresh targets. The cold exported workspace run took 2119.81 seconds, inside its required 2400-second bound; these durations are command bookkeeping, never performance evidence.

Original red evidence remains in the earlier report and its complete saved logs; no validation at the release failure depths was run pre-fix in this resumption.

**red-semantics — exit 101.** Full raw output: `.temp/brief-568/logs/red-semantics.txt`.

```text
test result: FAILED. 8 passed; 20 failed; 6 ignored; 0 measured; 0 filtered out; finished in 0.27s
```

**red-structural — exit 101.** Full raw output: `.temp/brief-568/logs/red-structural.txt`.

```text
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 1087 filtered out; finished in 0.05s
```

The first resumed integration run exposed two remaining test-harness failures: the cancel handle reserved the setup request instead of the validation request. It was moved immediately before validation; no runtime cancellation behavior was changed.

**resume-schema-continuations — exit 101.** Full raw output: `.temp/brief-568/logs/resume-schema-continuations.txt`.

```text
    depth::lifecycle_association_abandonment
    depth::lifecycle_list_abandonment
test result: FAILED. 38 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s
```

**resume-depth-controls — exit 0.** Full raw output: `.temp/brief-568/logs/resume-depth-controls.txt`.

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.62s
     Running tests/schema_continuations.rs (.temp/brief-568/target-tests/debug/deps/schema_continuations-de422bcc93a4672c)

running 2 tests
test depth::depth_control_association ... BUILT kind=association mode=control n=500
PASSED kind=association mode=control n=500 including teardown
ok
test depth::depth_control_list ... BUILT kind=list mode=control n=1000
PASSED kind=list mode=control n=1000 including teardown
ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out; finished in 0.06s
```

**resume-schema-continuations-fixed — exit 0.** Full raw output: `.temp/brief-568/logs/resume-schema-continuations-fixed.txt`.

```text
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.10s
     Running tests/schema_continuations.rs (.temp/brief-568/target-tests/debug/deps/schema_continuations-de422bcc93a4672c)

running 40 tests
test cancel_default ... CANCEL default=true (View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }, "List[1]", ["default: "])
ok
test cancel_predicate ... CANCEL default=false (View { status: "OK", value: Some("TerminatedEvaluation[\"Cancelled\"]"), termination: "Some(Cancelled)", messages: [RuntimeMessage { symbol: "General", tag: "cancel", text: "evaluation cancelled by host" }], prints: ["before"] }, "List[1]", ["predicate: "])
ok
test depth::depth_association_literal ... BUILT kind=association mode=literal n=500
FRESH CONTROL kind=association n=500 twice=false constructor=ConstantArray termination=None
VALIDATED kind=association mode=literal n=500 termination=None
PASSED kind=association mode=literal n=500 including teardown
ok
test depth::depth_control_association ... BUILT kind=association mode=control n=500
PASSED kind=association mode=control n=500 including teardown
ok
test depth::depth_control_list ... BUILT kind=list mode=control n=1000
PASSED kind=list mode=control n=1000 including teardown
ok
test depth::depth_list_literal ... BUILT kind=list mode=literal n=1000
FRESH CONTROL kind=list n=1000 twice=false constructor=ConstantArray termination=Some(StackLimit)
VALIDATED kind=list mode=literal n=1000 termination=Some(StackLimit)
PASSED kind=list mode=literal n=1000 including teardown
ok
test depth::depth_mismatch_association ... BUILT kind=association mode=mismatch n=500
VALIDATED kind=association mode=mismatch n=500 termination=None
PASSED kind=association mode=mismatch n=500 including teardown
ok
test depth::depth_mismatch_list ... BUILT kind=list mode=mismatch n=1000
VALIDATED kind=list mode=mismatch n=1000 termination=None
PASSED kind=list mode=mismatch n=1000 including teardown
ok
test depth::depth_schemaerrors_association ... BUILT kind=association mode=errors n=500
VALIDATED kind=association mode=errors n=500 termination=None
PASSED kind=association mode=errors n=500 including teardown
ok
test depth::depth_schemaerrors_list ... BUILT kind=list mode=errors n=1000
VALIDATED kind=list mode=errors n=1000 termination=None
PASSED kind=list mode=errors n=1000 including teardown
ok
test depth::lifecycle_association_abandonment ... FRESH CONTROL kind=association n=500 twice=true constructor=ConstantArray termination=None
LIFECYCLE BUILT kind=association mode=abandon n=500
LIFECYCLE SUSPEND kind=association mode=abandon n=500 ordinal=1
LIFECYCLE SUSPEND kind=association mode=abandon n=500 ordinal=2
LIFECYCLE PASSED kind=association mode=abandon n=500 including Session teardown
ok
test depth::lifecycle_association_suspension ... FRESH CONTROL kind=association n=500 twice=false constructor=ConstantArray termination=None
LIFECYCLE BUILT kind=association mode=single n=500
LIFECYCLE SUSPEND kind=association mode=single n=500 ordinal=1
LIFECYCLE RESULT kind=association mode=single n=500 termination=None
LIFECYCLE PASSED kind=association mode=single n=500 including Session teardown
ok
test depth::lifecycle_association_twice ... FRESH CONTROL kind=association n=500 twice=true constructor=ConstantArray termination=None
LIFECYCLE BUILT kind=association mode=twice n=500
LIFECYCLE SUSPEND kind=association mode=twice n=500 ordinal=1
LIFECYCLE SUSPEND kind=association mode=twice n=500 ordinal=2
LIFECYCLE RESULT kind=association mode=twice n=500 termination=None
LIFECYCLE PASSED kind=association mode=twice n=500 including Session teardown
ok
test depth::lifecycle_list_abandonment ... FRESH CONTROL kind=list n=1000 twice=true constructor=ConstantArray termination=Some(StackLimit)
LIFECYCLE BUILT kind=list mode=abandon n=1000
LIFECYCLE SUSPEND kind=list mode=abandon n=1000 ordinal=1
LIFECYCLE SUSPEND kind=list mode=abandon n=1000 ordinal=2
LIFECYCLE PASSED kind=list mode=abandon n=1000 including Session teardown
ok
test depth::lifecycle_list_suspension ... FRESH CONTROL kind=list n=1000 twice=false constructor=ConstantArray termination=Some(StackLimit)
LIFECYCLE BUILT kind=list mode=single n=1000
LIFECYCLE SUSPEND kind=list mode=single n=1000 ordinal=1
LIFECYCLE RESULT kind=list mode=single n=1000 termination=Some(StackLimit)
LIFECYCLE PASSED kind=list mode=single n=1000 including Session teardown
ok
test depth::lifecycle_list_twice ... FRESH CONTROL kind=list n=1000 twice=true constructor=ConstantArray termination=Some(StackLimit)
LIFECYCLE BUILT kind=list mode=twice n=1000
LIFECYCLE SUSPEND kind=list mode=twice n=1000 ordinal=1
LIFECYCLE SUSPEND kind=list mode=twice n=1000 ordinal=2
LIFECYCLE RESULT kind=list mode=twice n=1000 termination=Some(StackLimit)
LIFECYCLE PASSED kind=list mode=twice n=1000 including Session teardown
ok
test depth::shallow_association_literal ... BUILT kind=association mode=literal n=16
FRESH CONTROL kind=association n=16 twice=false constructor=ConstantArray termination=None
VALIDATED kind=association mode=literal n=16 termination=None
PASSED kind=association mode=literal n=16 including teardown
ok
test depth::shallow_list_literal ... BUILT kind=list mode=literal n=16
FRESH CONTROL kind=list n=16 twice=false constructor=ConstantArray termination=None
VALIDATED kind=list mode=literal n=16 termination=None
PASSED kind=list mode=literal n=16 including teardown
ok
test oracle_absent_optional_control ... CASE	absent_optional_control	resumed=(View { status: "OK", value: Some("Association[Rule[\"a\", 1]]"), termination: "None", messages: [], prints: [] }, "List[]", [])	queued=(View { status: "OK", value: Some("Association[Rule[\"a\", 1]]"), termination: "None", messages: [], prints: [] }, "List[]", [])
ROW	absent_optional_control	"Association[Rule[\"a\", 1]]"	"List[]"	"Association[Rule[\"a\", 1]]"	"List[]"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_allow_extra ... CASE	allow_extra	resumed=(View { status: "OK", value: Some("Association[Rule[\"extra\", 9], Rule[\"a\", 1], Rule[\"default\", 2]]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "OK", value: Some("Association[Rule[\"extra\", 9], Rule[\"a\", 1], Rule[\"default\", 2]]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])
ROW	allow_extra	"Association[Rule[\"extra\", 9], Rule[\"a\", 1], Rule[\"default\", 2]]"	"List[1, 2]"	"Association[Rule[\"extra\", 9], Rule[\"a\", 1], Rule[\"default\", 2]]"	"List[1, 2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_capped_errors_continue ... CASE	capped_errors_continue	resumed=(View { status: "OK", value: Some("List[Association[Rule[\"Path\", \"[1]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 1\"]], Association[Rule[\"Path\", \"[2]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 2\"]], Association[Rule[\"Path\", \"[3]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 3\"]], Association[Rule[\"Path\", \"[4]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 4\"]], Association[Rule[\"Path\", \"[5]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 5\"]], Association[Rule[\"Path\", \"[6]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 6\"]], Association[Rule[\"Path\", \"[7]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 7\"]], Association[Rule[\"Path\", \"[8]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 8\"]], Association[Rule[\"Path\", \"[9]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 9\"]], Association[Rule[\"Path\", \"[10]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 10\"]], Association[Rule[\"Path\", \"[11]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 11\"]], Association[Rule[\"Path\", \"[12]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 12\"]], Association[Rule[\"Path\", \"[13]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 13\"]], Association[Rule[\"Path\", \"[14]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 14\"]], Association[Rule[\"Path\", \"[15]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 15\"]], Association[Rule[\"Path\", \"[16]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 16\"]], Association[Rule[\"Path\", \"[17]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 17\"]], Association[Rule[\"Path\", \"[18]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 18\"]], Association[Rule[\"Path\", \"[19]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 19\"]], Association[Rule[\"Path\", \"[20]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 20\"]], Association[Rule[\"Path\", \"[21]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 21\"]], Association[Rule[\"Path\", \"[22]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 22\"]], Association[Rule[\"Path\", \"[23]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 23\"]], Association[Rule[\"Path\", \"[24]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 24\"]], Association[Rule[\"Path\", \"[25]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 25\"]], Association[Rule[\"Path\", \"[26]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 26\"]], Association[Rule[\"Path\", \"[27]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 27\"]], Association[Rule[\"Path\", \"[28]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 28\"]], Association[Rule[\"Path\", \"[29]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 29\"]], Association[Rule[\"Path\", \"[30]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 30\"]], Association[Rule[\"Path\", \"[31]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 31\"]], Association[Rule[\"Path\", \"[32]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 32\"]], Association[Rule[\"Path\", \"[33]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 33\"]], Association[Rule[\"Path\", \"[34]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 34\"]], Association[Rule[\"Path\", \"[35]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 35\"]], Association[Rule[\"Path\", \"[36]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 36\"]], Association[Rule[\"Path\", \"[37]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 37\"]], Association[Rule[\"Path\", \"[38]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 38\"]], Association[Rule[\"Path\", \"[39]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 39\"]], Association[Rule[\"Path\", \"[40]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 40\"]], Association[Rule[\"Path\", \"[41]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 41\"]], Association[Rule[\"Path\", \"[42]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 42\"]], Association[Rule[\"Path\", \"[43]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 43\"]], Association[Rule[\"Path\", \"[44]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 44\"]], Association[Rule[\"Path\", \"[45]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 45\"]], Association[Rule[\"Path\", \"[46]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 46\"]], Association[Rule[\"Path\", \"[47]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 47\"]], Association[Rule[\"Path\", \"[48]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 48\"]], Association[Rule[\"Path\", \"[49]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 49\"]], Association[Rule[\"Path\", \"[50]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 50\"]], Association[Rule[\"Path\", \"[51]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 51\"]], Association[Rule[\"Path\", \"[52]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 52\"]], Association[Rule[\"Path\", \"[53]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 53\"]], Association[Rule[\"Path\", \"[54]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 54\"]], Association[Rule[\"Path\", \"[55]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 55\"]], Association[Rule[\"Path\", \"[56]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 56\"]], Association[Rule[\"Path\", \"[57]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 57\"]], Association[Rule[\"Path\", \"[58]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 58\"]], Association[Rule[\"Path\", \"[59]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 59\"]], Association[Rule[\"Path\", \"[60]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 60\"]], Association[Rule[\"Path\", \"[61]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 61\"]], Association[Rule[\"Path\", \"[62]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 62\"]], Association[Rule[\"Path\", \"[63]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 63\"]], Association[Rule[\"Path\", \"[64]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 64\"]], Association[Rule[\"Path\", \"\"], Rule[\"Message\", \"… and 3 more errors\"]]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67]", ["cap: "])	queued=(View { status: "OK", value: Some("List[Association[Rule[\"Path\", \"[1]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 1\"]], Association[Rule[\"Path\", \"[2]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 2\"]], Association[Rule[\"Path\", \"[3]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 3\"]], Association[Rule[\"Path\", \"[4]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 4\"]], Association[Rule[\"Path\", \"[5]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 5\"]], Association[Rule[\"Path\", \"[6]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 6\"]], Association[Rule[\"Path\", \"[7]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 7\"]], Association[Rule[\"Path\", \"[8]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 8\"]], Association[Rule[\"Path\", \"[9]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 9\"]], Association[Rule[\"Path\", \"[10]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 10\"]], Association[Rule[\"Path\", \"[11]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 11\"]], Association[Rule[\"Path\", \"[12]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 12\"]], Association[Rule[\"Path\", \"[13]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 13\"]], Association[Rule[\"Path\", \"[14]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 14\"]], Association[Rule[\"Path\", \"[15]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 15\"]], Association[Rule[\"Path\", \"[16]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 16\"]], Association[Rule[\"Path\", \"[17]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 17\"]], Association[Rule[\"Path\", \"[18]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 18\"]], Association[Rule[\"Path\", \"[19]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 19\"]], Association[Rule[\"Path\", \"[20]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 20\"]], Association[Rule[\"Path\", \"[21]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 21\"]], Association[Rule[\"Path\", \"[22]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 22\"]], Association[Rule[\"Path\", \"[23]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 23\"]], Association[Rule[\"Path\", \"[24]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 24\"]], Association[Rule[\"Path\", \"[25]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 25\"]], Association[Rule[\"Path\", \"[26]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 26\"]], Association[Rule[\"Path\", \"[27]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 27\"]], Association[Rule[\"Path\", \"[28]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 28\"]], Association[Rule[\"Path\", \"[29]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 29\"]], Association[Rule[\"Path\", \"[30]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 30\"]], Association[Rule[\"Path\", \"[31]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 31\"]], Association[Rule[\"Path\", \"[32]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 32\"]], Association[Rule[\"Path\", \"[33]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 33\"]], Association[Rule[\"Path\", \"[34]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 34\"]], Association[Rule[\"Path\", \"[35]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 35\"]], Association[Rule[\"Path\", \"[36]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 36\"]], Association[Rule[\"Path\", \"[37]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 37\"]], Association[Rule[\"Path\", \"[38]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 38\"]], Association[Rule[\"Path\", \"[39]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 39\"]], Association[Rule[\"Path\", \"[40]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 40\"]], Association[Rule[\"Path\", \"[41]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 41\"]], Association[Rule[\"Path\", \"[42]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 42\"]], Association[Rule[\"Path\", \"[43]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 43\"]], Association[Rule[\"Path\", \"[44]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 44\"]], Association[Rule[\"Path\", \"[45]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 45\"]], Association[Rule[\"Path\", \"[46]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 46\"]], Association[Rule[\"Path\", \"[47]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 47\"]], Association[Rule[\"Path\", \"[48]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 48\"]], Association[Rule[\"Path\", \"[49]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 49\"]], Association[Rule[\"Path\", \"[50]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 50\"]], Association[Rule[\"Path\", \"[51]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 51\"]], Association[Rule[\"Path\", \"[52]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 52\"]], Association[Rule[\"Path\", \"[53]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 53\"]], Association[Rule[\"Path\", \"[54]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 54\"]], Association[Rule[\"Path\", \"[55]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 55\"]], Association[Rule[\"Path\", \"[56]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 56\"]], Association[Rule[\"Path\", \"[57]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 57\"]], Association[Rule[\"Path\", \"[58]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 58\"]], Association[Rule[\"Path\", \"[59]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 59\"]], Association[Rule[\"Path\", \"[60]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 60\"]], Association[Rule[\"Path\", \"[61]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 61\"]], Association[Rule[\"Path\", \"[62]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 62\"]], Association[Rule[\"Path\", \"[63]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 63\"]], Association[Rule[\"Path\", \"[64]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 64\"]], Association[Rule[\"Path\", \"\"], Rule[\"Message\", \"… and 3 more errors\"]]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67]", [])
ROW	capped_errors_continue	"List[Association[Rule[\"Path\", \"[1]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 1\"]], Association[Rule[\"Path\", \"[2]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 2\"]], Association[Rule[\"Path\", \"[3]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 3\"]], Association[Rule[\"Path\", \"[4]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 4\"]], Association[Rule[\"Path\", \"[5]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 5\"]], Association[Rule[\"Path\", \"[6]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 6\"]], Association[Rule[\"Path\", \"[7]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 7\"]], Association[Rule[\"Path\", \"[8]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 8\"]], Association[Rule[\"Path\", \"[9]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 9\"]], Association[Rule[\"Path\", \"[10]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 10\"]], Association[Rule[\"Path\", \"[11]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 11\"]], Association[Rule[\"Path\", \"[12]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 12\"]], Association[Rule[\"Path\", \"[13]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 13\"]], Association[Rule[\"Path\", \"[14]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 14\"]], Association[Rule[\"Path\", \"[15]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 15\"]], Association[Rule[\"Path\", \"[16]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 16\"]], Association[Rule[\"Path\", \"[17]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 17\"]], Association[Rule[\"Path\", \"[18]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 18\"]], Association[Rule[\"Path\", \"[19]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 19\"]], Association[Rule[\"Path\", \"[20]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 20\"]], Association[Rule[\"Path\", \"[21]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 21\"]], Association[Rule[\"Path\", \"[22]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 22\"]], Association[Rule[\"Path\", \"[23]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 23\"]], Association[Rule[\"Path\", \"[24]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 24\"]], Association[Rule[\"Path\", \"[25]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 25\"]], Association[Rule[\"Path\", \"[26]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 26\"]], Association[Rule[\"Path\", \"[27]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 27\"]], Association[Rule[\"Path\", \"[28]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 28\"]], Association[Rule[\"Path\", \"[29]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 29\"]], Association[Rule[\"Path\", \"[30]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 30\"]], Association[Rule[\"Path\", \"[31]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 31\"]], Association[Rule[\"Path\", \"[32]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 32\"]], Association[Rule[\"Path\", \"[33]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 33\"]], Association[Rule[\"Path\", \"[34]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 34\"]], Association[Rule[\"Path\", \"[35]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 35\"]], Association[Rule[\"Path\", \"[36]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 36\"]], Association[Rule[\"Path\", \"[37]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 37\"]], Association[Rule[\"Path\", \"[38]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 38\"]], Association[Rule[\"Path\", \"[39]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 39\"]], Association[Rule[\"Path\", \"[40]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 40\"]], Association[Rule[\"Path\", \"[41]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 41\"]], Association[Rule[\"Path\", \"[42]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 42\"]], Association[Rule[\"Path\", \"[43]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 43\"]], Association[Rule[\"Path\", \"[44]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 44\"]], Association[Rule[\"Path\", \"[45]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 45\"]], Association[Rule[\"Path\", \"[46]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 46\"]], Association[Rule[\"Path\", \"[47]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 47\"]], Association[Rule[\"Path\", \"[48]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 48\"]], Association[Rule[\"Path\", \"[49]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 49\"]], Association[Rule[\"Path\", \"[50]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 50\"]], Association[Rule[\"Path\", \"[51]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 51\"]], Association[Rule[\"Path\", \"[52]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 52\"]], Association[Rule[\"Path\", \"[53]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 53\"]], Association[Rule[\"Path\", \"[54]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 54\"]], Association[Rule[\"Path\", \"[55]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 55\"]], Association[Rule[\"Path\", \"[56]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 56\"]], Association[Rule[\"Path\", \"[57]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 57\"]], Association[Rule[\"Path\", \"[58]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 58\"]], Association[Rule[\"Path\", \"[59]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 59\"]], Association[Rule[\"Path\", \"[60]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 60\"]], Association[Rule[\"Path\", \"[61]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 61\"]], Association[Rule[\"Path\", \"[62]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 62\"]], Association[Rule[\"Path\", \"[63]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 63\"]], Association[Rule[\"Path\", \"[64]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 64\"]], Association[Rule[\"Path\", \"\"], Rule[\"Message\", \"… and 3 more errors\"]]]"	"List[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67]"	"List[Association[Rule[\"Path\", \"[1]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 1\"]], Association[Rule[\"Path\", \"[2]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 2\"]], Association[Rule[\"Path\", \"[3]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 3\"]], Association[Rule[\"Path\", \"[4]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 4\"]], Association[Rule[\"Path\", \"[5]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 5\"]], Association[Rule[\"Path\", \"[6]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 6\"]], Association[Rule[\"Path\", \"[7]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 7\"]], Association[Rule[\"Path\", \"[8]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 8\"]], Association[Rule[\"Path\", \"[9]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 9\"]], Association[Rule[\"Path\", \"[10]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 10\"]], Association[Rule[\"Path\", \"[11]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 11\"]], Association[Rule[\"Path\", \"[12]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 12\"]], Association[Rule[\"Path\", \"[13]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 13\"]], Association[Rule[\"Path\", \"[14]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 14\"]], Association[Rule[\"Path\", \"[15]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 15\"]], Association[Rule[\"Path\", \"[16]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 16\"]], Association[Rule[\"Path\", \"[17]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 17\"]], Association[Rule[\"Path\", \"[18]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 18\"]], Association[Rule[\"Path\", \"[19]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 19\"]], Association[Rule[\"Path\", \"[20]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 20\"]], Association[Rule[\"Path\", \"[21]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 21\"]], Association[Rule[\"Path\", \"[22]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 22\"]], Association[Rule[\"Path\", \"[23]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 23\"]], Association[Rule[\"Path\", \"[24]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 24\"]], Association[Rule[\"Path\", \"[25]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 25\"]], Association[Rule[\"Path\", \"[26]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 26\"]], Association[Rule[\"Path\", \"[27]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 27\"]], Association[Rule[\"Path\", \"[28]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 28\"]], Association[Rule[\"Path\", \"[29]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 29\"]], Association[Rule[\"Path\", \"[30]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 30\"]], Association[Rule[\"Path\", \"[31]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 31\"]], Association[Rule[\"Path\", \"[32]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 32\"]], Association[Rule[\"Path\", \"[33]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 33\"]], Association[Rule[\"Path\", \"[34]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 34\"]], Association[Rule[\"Path\", \"[35]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 35\"]], Association[Rule[\"Path\", \"[36]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 36\"]], Association[Rule[\"Path\", \"[37]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 37\"]], Association[Rule[\"Path\", \"[38]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 38\"]], Association[Rule[\"Path\", \"[39]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 39\"]], Association[Rule[\"Path\", \"[40]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 40\"]], Association[Rule[\"Path\", \"[41]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 41\"]], Association[Rule[\"Path\", \"[42]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 42\"]], Association[Rule[\"Path\", \"[43]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 43\"]], Association[Rule[\"Path\", \"[44]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 44\"]], Association[Rule[\"Path\", \"[45]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 45\"]], Association[Rule[\"Path\", \"[46]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 46\"]], Association[Rule[\"Path\", \"[47]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 47\"]], Association[Rule[\"Path\", \"[48]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 48\"]], Association[Rule[\"Path\", \"[49]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 49\"]], Association[Rule[\"Path\", \"[50]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 50\"]], Association[Rule[\"Path\", \"[51]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 51\"]], Association[Rule[\"Path\", \"[52]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 52\"]], Association[Rule[\"Path\", \"[53]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 53\"]], Association[Rule[\"Path\", \"[54]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 54\"]], Association[Rule[\"Path\", \"[55]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 55\"]], Association[Rule[\"Path\", \"[56]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 56\"]], Association[Rule[\"Path\", \"[57]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 57\"]], Association[Rule[\"Path\", \"[58]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 58\"]], Association[Rule[\"Path\", \"[59]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 59\"]], Association[Rule[\"Path\", \"[60]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 60\"]], Association[Rule[\"Path\", \"[61]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 61\"]], Association[Rule[\"Path\", \"[62]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 62\"]], Association[Rule[\"Path\", \"[63]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 63\"]], Association[Rule[\"Path\", \"[64]\"], Rule[\"Message\", \"expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 2], InputString[\\\"cap: \\\"]]; False], got 64\"]], Association[Rule[\"Path\", \"\"], Rule[\"Message\", \"… and 3 more errors\"]]]"	"List[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_conformschema_association ... CASE	ConformSchema-association	resumed=(View { status: "OK", value: Some("Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	ConformSchema-association	"Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3]]"	"List[1, 2, 3]"	"Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_conformschema_default ... CASE	ConformSchema-default	resumed=(View { status: "OK", value: Some("Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	ConformSchema-default	"Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3]]"	"List[1, 2, 3]"	"Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3]]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_conformschema_default_predicate ... CASE	ConformSchema-default-predicate	resumed=(View { status: "OK", value: Some("Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3]]"), termination: "None", messages: [], prints: [] }, "List[2]", ["p: "])	queued=(View { status: "OK", value: Some("Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3]]"), termination: "None", messages: [], prints: [] }, "List[2]", [])
ROW	ConformSchema-default-predicate	"Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3]]"	"List[2]"	"Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3]]"	"List[2]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_conformschema_leaf ... CASE	ConformSchema-leaf	resumed=(View { status: "OK", value: Some("List[1, 2, 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	ConformSchema-leaf	"List[1, 2, 3]"	"List[1, 2, 3]"	"List[1, 2, 3]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_default_then_predicate ... CASE	default_then_predicate	resumed=(View { status: "OK", value: Some("Association[Rule[\"a\", 7], Rule[\"b\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["default: ", "predicate: "])	queued=(View { status: "OK", value: Some("Association[Rule[\"a\", 7], Rule[\"b\", 3]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	default_then_predicate	"Association[Rule[\"a\", 7], Rule[\"b\", 3]]"	"List[1, 2, 3]"	"Association[Rule[\"a\", 7], Rule[\"b\", 3]]"	"List[1, 2, 3]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_error_after_resume ... CASE	error_after_resume	resumed=(View { status: "OK", value: Some("$Failed"), termination: "None", messages: [RuntimeMessage { symbol: "ConformSchema", tag: "schema", text: "[3]: expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 1], InputString[\"p: \"]]; Less[x, 3]], got 3" }], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("$Failed"), termination: "None", messages: [RuntimeMessage { symbol: "ConformSchema", tag: "schema", text: "[3]: expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 1], InputString[\"p: \"]]; Less[x, 3]], got 3" }], prints: [] }, "List[1, 2, 3]", [])
ROW	error_after_resume	"$Failed"	"List[1, 2, 3]"	"$Failed"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "ConformSchema", tag: "schema", text: "[3]: expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 1], InputString[\"p: \"]]; Less[x, 3]], got 3" }]	[RuntimeMessage { symbol: "ConformSchema", tag: "schema", text: "[3]: expected to match _?Function[x, AppendTo[log, x]; If[Equal[x, 1], InputString[\"p: \"]]; Less[x, 3]], got 3" }]	[]	[]
ok
test oracle_error_prefix ... CASE	error_prefix	resumed=(View { status: "OK", value: Some("List[Association[Rule[\"Path\", \"bad\"], Rule[\"Message\", \"expected to match _Integer, got \\\"x\\\"\"]], Association[Rule[\"Path\", \"late\"], Rule[\"Message\", \"expected to match _Integer, got \\\"y\\\"\"]]]"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "schema message" }], prints: ["predicate"] }, "List[2]", ["p: "])	queued=(View { status: "OK", value: Some("List[Association[Rule[\"Path\", \"bad\"], Rule[\"Message\", \"expected to match _Integer, got \\\"x\\\"\"]], Association[Rule[\"Path\", \"late\"], Rule[\"Message\", \"expected to match _Integer, got \\\"y\\\"\"]]]"), termination: "None", messages: [RuntimeMessage { symbol: "probe", tag: "tag", text: "schema message" }], prints: ["predicate"] }, "List[2]", [])
ROW	error_prefix	"List[Association[Rule[\"Path\", \"bad\"], Rule[\"Message\", \"expected to match _Integer, got \\\"x\\\"\"]], Association[Rule[\"Path\", \"late\"], Rule[\"Message\", \"expected to match _Integer, got \\\"y\\\"\"]]]"	"List[2]"	"List[Association[Rule[\"Path\", \"bad\"], Rule[\"Message\", \"expected to match _Integer, got \\\"x\\\"\"]], Association[Rule[\"Path\", \"late\"], Rule[\"Message\", \"expected to match _Integer, got \\\"y\\\"\"]]]"	"List[2]"	1	1	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "probe", tag: "tag", text: "schema message" }]	[RuntimeMessage { symbol: "probe", tag: "tag", text: "schema message" }]	["predicate"]	["predicate"]
ok
test oracle_eval_error_after_resume ... CASE	eval_error_after_resume	resumed=(View { status: "Other { message: \"uncaught throw: \\\"schema boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"schema boom\"] returned to top level." }], prints: [] }, "List[1, 2]", ["p: "])	queued=(View { status: "Other { message: \"uncaught throw: \\\"schema boom\\\" with tag None\" }", value: None, termination: "None", messages: [RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"schema boom\"] returned to top level." }], prints: [] }, "List[1, 2]", [])
ROW	eval_error_after_resume	"<error>"	"List[1, 2]"	"<error>"	"List[1, 2]"	1	1	true	"Other { message: \"uncaught throw: \\\"schema boom\\\" with tag None\" }"	"Other { message: \"uncaught throw: \\\"schema boom\\\" with tag None\" }"	"None"	"None"	[RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"schema boom\"] returned to top level." }]	[RuntimeMessage { symbol: "Throw", tag: "nocatch", text: "Uncaught Throw[\"schema boom\"] returned to top level." }]	[]	[]
ok
test oracle_malformed_after_resume ... CASE	malformed_after_resume	resumed=(View { status: "OK", value: Some("$Failed"), termination: "None", messages: [RuntimeMessage { symbol: "SchemaErrors", tag: "schema", text: "Schema path b: list schema must contain exactly one element; got 2" }], prints: [] }, "List[1]", ["p: "])	queued=(View { status: "OK", value: Some("$Failed"), termination: "None", messages: [RuntimeMessage { symbol: "SchemaErrors", tag: "schema", text: "Schema path b: list schema must contain exactly one element; got 2" }], prints: [] }, "List[1]", [])
ROW	malformed_after_resume	"$Failed"	"List[1]"	"$Failed"	"List[1]"	1	1	true	"OK"	"OK"	"None"	"None"	[RuntimeMessage { symbol: "SchemaErrors", tag: "schema", text: "Schema path b: list schema must contain exactly one element; got 2" }]	[RuntimeMessage { symbol: "SchemaErrors", tag: "schema", text: "Schema path b: list schema must contain exactly one element; got 2" }]	[]	[]
ok
test oracle_nested_assoc_list_assoc ... CASE	nested_assoc_list_assoc	resumed=(View { status: "OK", value: Some("Association[Rule[\"rows\", List[Association[Rule[\"a\", 1], Rule[\"d\", 9]], Association[Rule[\"a\", 2], Rule[\"d\", 9]]]]]"), termination: "None", messages: [], prints: [] }, "List[1, 9, 2, 9]", ["p: "])	queued=(View { status: "OK", value: Some("Association[Rule[\"rows\", List[Association[Rule[\"a\", 1], Rule[\"d\", 9]], Association[Rule[\"a\", 2], Rule[\"d\", 9]]]]]"), termination: "None", messages: [], prints: [] }, "List[1, 9, 2, 9]", [])
ROW	nested_assoc_list_assoc	"Association[Rule[\"rows\", List[Association[Rule[\"a\", 1], Rule[\"d\", 9]], Association[Rule[\"a\", 2], Rule[\"d\", 9]]]]]"	"List[1, 9, 2, 9]"	"Association[Rule[\"rows\", List[Association[Rule[\"a\", 1], Rule[\"d\", 9]], Association[Rule[\"a\", 2], Rule[\"d\", 9]]]]]"	"List[1, 9, 2, 9]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_positions_distinct ... CASE	positions_distinct	resumed=(View { status: "OK", value: Some("List[1, 2, 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["first: ", "third: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	positions_distinct	"List[1, 2, 3]"	"List[1, 2, 3]"	"List[1, 2, 3]"	"List[1, 2, 3]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_predicate_twice ... CASE	predicate_twice	resumed=(View { status: "OK", value: Some("List[1, 2]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", ["one: ", "two: "])	queued=(View { status: "OK", value: Some("List[1, 2]"), termination: "None", messages: [], prints: [] }, "List[1, 2]", [])
ROW	predicate_twice	"List[1, 2]"	"List[1, 2]"	"List[1, 2]"	"List[1, 2]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_present_default_control ... CASE	present_default_control	resumed=(View { status: "OK", value: Some("Association[Rule[\"a\", 1]]"), termination: "None", messages: [], prints: [] }, "List[]", [])	queued=(View { status: "OK", value: Some("Association[Rule[\"a\", 1]]"), termination: "None", messages: [], prints: [] }, "List[]", [])
ROW	present_default_control	"Association[Rule[\"a\", 1]]"	"List[]"	"Association[Rule[\"a\", 1]]"	"List[]"	0	0	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_schema_defaults_twice ... CASE	Schema-defaults-twice	resumed=(View { status: "OK", value: Some("Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3], Rule[\"d\", 4], Rule[\"e\", 5]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3], Rule[\"d\", 4], Rule[\"e\", 5]]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	Schema-defaults-twice	"Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3], Rule[\"d\", 4], Rule[\"e\", 5]]"	"List[1, 2, 3, 4, 5]"	"Association[Rule[\"a\", 1], Rule[\"b\", 2], Rule[\"c\", 3], Rule[\"d\", 4], Rule[\"e\", 5]]"	"List[1, 2, 3, 4, 5]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_schema_twice ... CASE	Schema-twice	resumed=(View { status: "OK", value: Some("List[1, 2, 3, 4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", ["p: ", "p: "])	queued=(View { status: "OK", value: Some("List[1, 2, 3, 4, 5]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3, 4, 5]", [])
ROW	Schema-twice	"List[1, 2, 3, 4, 5]"	"List[1, 2, 3, 4, 5]"	"List[1, 2, 3, 4, 5]"	"List[1, 2, 3, 4, 5]"	2	2	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_schemaerrors_association ... CASE	SchemaErrors-association	resumed=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	SchemaErrors-association	"List[]"	"List[1, 2, 3]"	"List[]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_schemaerrors_default ... CASE	SchemaErrors-default	resumed=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	SchemaErrors-default	"List[]"	"List[1, 2, 3]"	"List[]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_schemaerrors_leaf ... CASE	SchemaErrors-leaf	resumed=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", ["p: "])	queued=(View { status: "OK", value: Some("List[]"), termination: "None", messages: [], prints: [] }, "List[1, 2, 3]", [])
ROW	SchemaErrors-leaf	"List[]"	"List[1, 2, 3]"	"List[]"	"List[1, 2, 3]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok
test oracle_strict_extra ... CASE	strict_extra	resumed=(View { status: "OK", value: Some("List[Association[Rule[\"Path\", \"extra\"], Rule[\"Message\", \"extra key is not allowed\"]]]"), termination: "None", messages: [], prints: [] }, "List[1]", ["p: "])	queued=(View { status: "OK", value: Some("List[Association[Rule[\"Path\", \"extra\"], Rule[\"Message\", \"extra key is not allowed\"]]]"), termination: "None", messages: [], prints: [] }, "List[1]", [])
ROW	strict_extra	"List[Association[Rule[\"Path\", \"extra\"], Rule[\"Message\", \"extra key is not allowed\"]]]"	"List[1]"	"List[Association[Rule[\"Path\", \"extra\"], Rule[\"Message\", \"extra key is not allowed\"]]]"	"List[1]"	1	1	true	"OK"	"OK"	"None"	"None"	[]	[]	[]	[]
ok

test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
```

**resume-schema-depth-contract — exit 0.** Full raw output: `.temp/brief-568/logs/resume-schema-depth-contract.txt`.

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 10 tests
test tests::association_depth_contract::schema_depth_contract::schema_direct_association_depth_isolates_the_validator_from_result_evaluation ... DIRECT conform association n=500 exact; 2 MiB including Session teardown
ok
test tests::association_depth_contract::schema_depth_contract::schema_direct_errors_association_depth ... DIRECT errors association n=500 exact; 2 MiB including Session teardown
ok
test tests::association_depth_contract::schema_depth_contract::schema_direct_errors_list_depth ... DIRECT errors list n=1000 exact; 2 MiB including Session teardown
ok
test tests::association_depth_contract::schema_depth_contract::schema_direct_list_depth_isolates_the_validator_from_result_evaluation ... DIRECT conform list n=1000 exact; 2 MiB including Session teardown
ok
test tests::association_depth_contract::schema_depth_contract::schema_frame_roots_are_complete_and_flat_clone_drop_is_bounded ... SCHEMA_ROOTS every field and both pending phases; clone/trace/drop list=1000 association=500 on 2 MiB
ok
test tests::association_depth_contract::schema_depth_contract::schema_root_completeness_falsifier - should panic ... 
thread 'tests::association_depth_contract::schema_depth_contract::schema_root_completeness_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/schema_depth_contract.rs:149:5:
assertion `left == right` failed: every schema Expr-bearing field must be traced
  left: [Atom(Integer(Small(1))), Atom(Integer(Small(2))), Atom(Integer(Small(3))), Atom(Integer(Small(4))), Atom(Integer(Small(5))), Atom(Integer(Small(6))), Atom(Integer(Small(7))), Atom(Integer(Small(8))), Atom(Integer(Small(9))), Atom(Integer(Small(10))), Atom(Integer(Small(11))), Atom(Integer(Small(12)))]
 right: [Atom(Integer(Small(1))), Atom(Integer(Small(2))), Atom(Integer(Small(3))), Atom(Integer(Small(4))), Atom(Integer(Small(5))), Atom(Integer(Small(6))), Atom(Integer(Small(7))), Atom(Integer(Small(8))), Atom(Integer(Small(9))), Atom(Integer(Small(10))), Atom(Integer(Small(11))), Atom(Integer(Small(12))), Atom(Integer(Small(13))), Atom(Integer(Small(14))), Atom(Integer(Small(15))), Atom(Integer(Small(16)))]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
ok
test tests::association_depth_contract::schema_depth_contract::structural_no_schema_recursion ... SCHEMA_RECURSION []
ok
test tests::association_depth_contract::schema_depth_contract::structural_no_schema_self_reaching ... ok
test tests::association_depth_contract::schema_depth_contract::structural_recursion_falsifiers ... CROSS_MODULE_FALSIFIER [RecursiveEdge { path: "all", caller: "fold_child", callee: "schema_drive", count: 1 }, RecursiveEdge { path: "all", caller: "schema_drive", callee: "fold_child", count: 1 }]
ok
test tests::association_depth_contract::schema_depth_contract::structural_self_reaching_falsifier - should panic ... 
thread 'tests::association_depth_contract::schema_depth_contract::structural_self_reaching_falsifier' panicked at crates/rum-stdlib/src/tests/association_depth_contract/schema_depth_contract.rs:55:5:
self-reaching schema ownership
ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1087 filtered out; finished in 0.17s
```

The public in-process list control and ConformSchema both return Some(StackLimit) at 1000; association 500 returns exact results. SchemaErrors always returns exact empty/error results at those depths. Direct-validator tests return exact values in both modes for list 1000 and association 500. Session teardown stays on the 2 MiB thread. Each falsifier fired: direct/cross-file recursion, injected schema self-reachability and missing Expr roots. No deep Expr is formatted to establish a success result.

**resume-ownership-default — exit 0.** Full raw output: `.temp/brief-568/logs/resume-ownership-default.txt`.

```text
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1077 filtered out; finished in 57.38s
```

**resume-ownership-no-default — exit 0.** Full raw output: `.temp/brief-568/logs/resume-ownership-no-default.txt`.

```text
EXACT INVENTORY UnknownHostPayload: 63 rows
EXACT INVENTORY UnknownConstruct: 7597 rows
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1071 filtered out; finished in 59.29s
```

**resume-fingerprints — exit 0.** Full raw output: `.temp/brief-568/logs/resume-fingerprints.txt`.

```text
test tests::retention_closure::local::contract_tests::regenerate_local_contract_fingerprints ... Regenerated 68 fingerprints at /home/remi/dev/rum/.temp/brief-568/verification-source/target/brief-544/local-contracts.tsv
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 0.18s
```

**resume-certificates-default-replay — exit 101.** Full raw output: `.temp/brief-568/logs/resume-certificates-default-replay.txt`.

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 17.15s
```

**resume-certificates-no-default-replay — exit 101.** Full raw output: `.temp/brief-568/logs/resume-certificates-no-default-replay.txt`.

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1090 filtered out; finished in 17.34s
```

**resume-certificates-default-green — exit 0.** Full raw output: `.temp/brief-568/logs/resume-certificates-default-green.txt`.

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out; finished in 16.99s
```

**resume-certificates-no-default-green — exit 0.** Full raw output: `.temp/brief-568/logs/resume-certificates-no-default-green.txt`.

```text
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1090 filtered out; finished in 17.32s
```

**resume-fmt-check — exit 0.** Full raw output: `.temp/brief-568/logs/resume-fmt-check.txt`.

```text
(no stdout/stderr)
```

**resume-runtime — exit 0.** Full raw output: `.temp/brief-568/logs/resume-runtime.txt`.

```text
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 23.05s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.55s
```

The certificate replay failures above are the expected rejection of the old installed cells after new cells were emitted. Only the actual emitted cells were copied; both acceptance modes subsequently passed.

**Full workspace:** 43 target summaries; 2996 passed, nine native ZeroMQ failures, 58 ignored. The sole failed target was rum-jupyter --lib. These were sandbox EPERM failures when binding loopback. No passing target was repeated; the failed target was retried with the supported RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 flag permitted by the brief. Cargo then reported 151 passed, which includes **nine native skips and 142 executed checks**. Native networking is not claimed tested. The original workspace command really exited 101; the targeted platform retry exited 0.

**resume-workspace — exit 101.** Full raw output: `.temp/brief-568/logs/resume-workspace.txt`.

```text
     Running unittests src/main.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running tests/cli.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running tests/input_requests.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
     Running tests/terminal.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s
     Running tests/adapter_contract.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/batch_boundary_ownership.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/guest_loading_resume.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/host_payload_destruction_contract.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 303.86s
     Running tests/installer.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/lifecycle_buildout.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_contract.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/random_password_policy.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/stress.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.34s
     Running tests/suspended_timeout_contract.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.02s
     Running tests/suspension_abandonment_contract.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: FAILED. 142 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s
     Running unittests src/bin/rum-jupyter-kernel.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
     Running tests/input_abandonment.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/input_continuation.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1094 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 24.63s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/rum_stdlib-c8c5a89643371dc7)
A1_CORPUS_TOTAL agreements=7337 disagreements=0 claims=7337
test result: ok. 1044 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1433.26s
     Running unittests src/bin/generate-builtin-reference.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/guest_loading_resume.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
     Running tests/lazy_pull_continuations.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.72s
     Running tests/loop_continuations.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s
     Running tests/schema_continuations.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.58s
     Running unittests src/lib.rs (/home/remi/dev/rum/.temp/brief-568/target-verification/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.44s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.64s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: 1 target failed:
    `-p rum-jupyter --lib`
```

**resume-native-zeromq-skips — exit 0.** Full raw output: `.temp/brief-568/logs/resume-native-zeromq-skips.txt`.

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
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
```

**Release depth programs:** the candidate one-unit CLI, each program alone, under ulimit -s 2048 / ulimit -c 0, stdin /dev/null, 60-second bound. Both construction controls ran before any validation. Conform programs first construct/inspect the fresh ConstantArray result in the same program/profile; association uses the explicitly allowed ConstantArray fallback. Both calls are wrapped in Check restricted to General::reslim so matching fail-closed outcomes can be compared while retaining the actual error messages. Every successful deep result is checked level-by-level and at its scalar leaf. No depth was lowered.

**release-candidate-1-association-control-5000 — exit 0.** Full raw output: `.temp/brief-568/logs/release-candidate-1-association-control-5000.txt`.

```text
CONTROL association n=5000 exact
2
Null
```

**release-candidate-1-list-control-6000 — exit 0.** Full raw output: `.temp/brief-568/logs/release-candidate-1-list-control-6000.txt`.

```text
CONTROL list n=6000 exact
2
Null
```

**release-candidate-1-association-conform-5000 — exit 0.** Full raw output: `.temp/brief-568/logs/release-candidate-1-association-conform-5000.txt`.

```text
CONTROL association n=5000 exact
FRESH association n=5000 ConstantArray exact
CONFORM association n=5000 exact
2
Null
```

**release-candidate-1-association-errors-5000 — exit 0.** Full raw output: `.temp/brief-568/logs/release-candidate-1-association-errors-5000.txt`.

```text
CONTROL association n=5000 exact
ERRORS association n=5000 exact
2
Null
```

**release-candidate-1-association-mismatch-5000 — exit 0.** Full raw output: `.temp/brief-568/logs/release-candidate-1-association-mismatch-5000.txt`.

```text
CONTROL association n=5000 exact
MISMATCH association n=5000 exact
2
Null
```

The omitted display line is the full raw path in the linked log, not an omitted assertion: the program checks the exact complete error object and the runner independently matched the entire path (9999 bytes). `exact_full_path=true`, process exit 0. No deep Expr display is used.

**release-candidate-1-list-conform-6000 — exit 0.** Full raw output: `.temp/brief-568/logs/release-candidate-1-list-conform-6000.txt`.

```text
CONTROL list n=6000 exact
FRESH list n=6000 ConstantArray reslim
CONFORM list n=6000 matching reslim
2
! General::reslim: evaluation exceeded native stack limit (1048576 bytes)
! General::reslim: evaluation exceeded native stack limit (1048576 bytes)
Null
```

**release-candidate-1-list-errors-6000 — exit 0.** Full raw output: `.temp/brief-568/logs/release-candidate-1-list-errors-6000.txt`.

```text
CONTROL list n=6000 exact
ERRORS list n=6000 exact
2
Null
```

**release-candidate-1-list-mismatch-6000 — exit 0.** Full raw output: `.temp/brief-568/logs/release-candidate-1-list-mismatch-6000.txt`.

```text
CONTROL list n=6000 exact
MISMATCH list n=6000 exact
2
Null
```

The omitted display line is the full raw path in the linked log, not an omitted assertion: the program checks the exact complete error object and the runner independently matched the entire path (18000 bytes). `exact_full_path=true`, process exit 0. No deep Expr display is used.

**Ownership reconciliation:** both feature-mode discovery/exactness tests pass. No existing non-schema row changed, no host payload was added, no retention disposition/exclusion changed, and all 59 self-reaching rows remain byte-identical with no schema row. The full BEFORE/AFTER records are `.temp/brief-568/ownership-reconciliation.json`; the earlier report lists the unchanged 41 old schema-row mappings, eight new derives and four new macros verbatim. The previously rejected row is now admitted by the director resume note:

| Inventory | Before | After | Changed old rows | Added | Removed |
| --- | --- | --- | --- | --- | --- |
| host_payloads.tsv | 63 | 63 | 0 | 0 | 0 |
| unknown_attributes.tsv | 1445 | 1453 | 8 | 8 | 0 |
| unknown_captures.tsv | 2938 | 2938 | 11 | 0 | 0 |
| unknown_macros.tsv | 3114 | 3117 | 21 | 4 | 1 |
| unknown_storage.tsv | 88 | 89 | 0 | 1 | 0 |
| self_reaching.tsv | 59 | 59 | 0 | 0 | 0 |

```text
UnknownConstruct	crates/rum-stdlib/src/schema/state.rs	88	27	rum_stdlib::schema::state::AssociationWork.positions/std::option::Option[Own]/arg0/std::collections::HashMap[Own]/arg2: foreign default hasher representation std::collections::hash_map::RandomState
```

The only retention obligation changes are these two evidence relocations; class/status, declaration owner, disposition and exclusions are unchanged:

```text
BEFORE	Unsplit:ConformSchema	ConformSchema	Unsplit	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs	Inherited membership; subclass not re-audited	crates/rum-stdlib/src/schema.rs:207; crates/rum-stdlib/src/schema.rs:208; crates/rum-stdlib/src/schema.rs:334	rum_stdlib::schema::eval_conform_schema
AFTER	Unsplit:ConformSchema	ConformSchema	Unsplit	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs	Inherited membership; subclass not re-audited	crates/rum-stdlib/src/schema/association.rs:29; crates/rum-stdlib/src/schema/association.rs:30; crates/rum-stdlib/src/schema/format.rs:40	rum_stdlib::schema::eval_conform_schema
```

```text
BEFORE	Unsplit:SchemaErrors	SchemaErrors	Unsplit	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs	Inherited membership; subclass not re-audited	crates/rum-stdlib/src/schema.rs:207; crates/rum-stdlib/src/schema.rs:208; crates/rum-stdlib/src/schema.rs:334	rum_stdlib::schema::eval_schema_errors
AFTER	Unsplit:SchemaErrors	SchemaErrors	Unsplit	Pending	crates/rum-stdlib/src/catalog/memory_growth_contract.rs	Inherited membership; subclass not re-audited	crates/rum-stdlib/src/schema/association.rs:29; crates/rum-stdlib/src/schema/association.rs:30; crates/rum-stdlib/src/schema/format.rs:40	rum_stdlib::schema::eval_schema_errors
```

**All 68 replayed fingerprint cells:** only column 19 changed. Every preceding field remains byte-identical, including identity, status and exclusions. Actual generator output is in `verification-source/target/brief-544/local-contracts.tsv`; complete records are `fingerprint-reconciliation.json`.

| ID | Before | After |
| --- | --- | --- |
| B01 | fnv1a64:e9fa6f18b50b8566 | fnv1a64:7c00f621b606499b |
| B02 | fnv1a64:08ea10fd5b2bc8df | fnv1a64:26d528f1ff8c8216 |
| B03 | fnv1a64:a3a6341a1fa04d54 | fnv1a64:05a7c127d7cd7b91 |
| B04 | fnv1a64:fb22d495d4bf858e | fnv1a64:fa51195ba1a372d3 |
| B05 | fnv1a64:3912d23cc3404fbb | fnv1a64:15230586e494d51a |
| B06 | fnv1a64:e1923469a3bc125a | fnv1a64:28e2034d43fac337 |
| B07 | fnv1a64:25ca2d347d7ea4ab | fnv1a64:bf39f6434d8939ca |
| B08 | fnv1a64:be0f3ff851165e30 | fnv1a64:d3bd84f43e3d5805 |
| B09 | fnv1a64:d4ab03a29da66342 | fnv1a64:e9d2deb61487ec2f |
| B10 | fnv1a64:c215ddb20a95c425 | fnv1a64:534207fd38c2f704 |
| B11 | fnv1a64:bc7a85f2eeb3ab8e | fnv1a64:bba8cab8bb9798d3 |
| B12 | fnv1a64:93b7839e79aef842 | fnv1a64:a8df5eb1f090812f |
| I01 | fnv1a64:c7dc5a1621c37be6 | fnv1a64:59e2e11f22be401b |
| I02 | fnv1a64:90544fd0bb76d7bb | fnv1a64:6c64831adccb5d1a |
| I03 | fnv1a64:35190841e354e682 | fnv1a64:cbe27412f0ab616f |
| I04 | fnv1a64:c842732cf1f848ff | fnv1a64:efdd312ba32f13b6 |
| P01 | fnv1a64:a9de46132044a2fa | fnv1a64:302d2f176bdf11d7 |
| P02 | fnv1a64:dfb6787ec3c003b5 | fnv1a64:b5d884775f610f14 |
| P03 | fnv1a64:2e6faac91544fb6c | fnv1a64:149dfd962bb2d019 |
| P04 | fnv1a64:fdc52f12dfd4e4df | fnv1a64:1bb0470784359e16 |
| P05 | fnv1a64:5be438d2783f7ff6 | fnv1a64:7bb4d7ad752d952b |
| P06 | fnv1a64:e436a86f05de91c2 | fnv1a64:f95e83827cc01aaf |
| P070 | fnv1a64:1e8444b0e4540bc2 | fnv1a64:33ac1fc45b3594af |
| P080 | fnv1a64:4dc8455bce0d9f65 | fnv1a64:d3eac7324e8b8644 |
| P090 | fnv1a64:f1410000eca1801f | fnv1a64:89c414f3ea5d7756 |
| P100 | fnv1a64:a27742229d6dd2fe | fnv1a64:bdeaa37c2e689f03 |
| P110 | fnv1a64:2ce5fd8ef351c8fd | fnv1a64:66cb596fffa740ec |
| P120 | fnv1a64:2f094bde7fe421d0 | fnv1a64:ec4784f2befabfa5 |
| P130 | fnv1a64:c9978691ac327eab | fnv1a64:63074fa07c3d13ca |
| P140 | fnv1a64:9871b27cb12dfc37 | fnv1a64:c4c486468088bcfe |
| P150 | fnv1a64:19ceebbfbff20b3f | fnv1a64:bc01a6bcca8413f6 |
| P071 | fnv1a64:3bbd73c95e92ff4b | fnv1a64:d4d03ac3d610246a |
| P081 | fnv1a64:b9e296465adeb2e2 | fnv1a64:49f15a67cc91b94f |
| P091 | fnv1a64:b71f8dab1b16a53a | fnv1a64:ddb044b2879fd717 |
| P101 | fnv1a64:24e8ee5c28b565fd | fnv1a64:5ece4a3d350addec |
| P111 | fnv1a64:79d03fa5164c3932 | fnv1a64:e05d0283ed0a8c9f |
| P121 | fnv1a64:78660d6f80c328b9 | fnv1a64:0187d3b84a485d10 |
| P131 | fnv1a64:98ccf6fc0b13a70e | fnv1a64:97fb3bc1d7f79453 |
| P141 | fnv1a64:c60ec886b36cd49a | fnv1a64:d456bf77c62bde77 |
| P151 | fnv1a64:df593dce88e5b690 | fnv1a64:66f69ef03d618965 |
| B13 | fnv1a64:6fa2c7f2d458ce5a | fnv1a64:b6f296d674977f37 |
| R01 | fnv1a64:04101fc5d3e2678f | fnv1a64:99734c93db5147c6 |
| R02 | fnv1a64:fca8756a2a8ea0ac | fnv1a64:1db1971edc4ed259 |
| R03 | fnv1a64:64331647b3d8f653 | fnv1a64:bd548a42fd2ee0c2 |
| R040 | fnv1a64:01505dc33876d0af | fnv1a64:d6d20d6507301066 |
| R050 | fnv1a64:7c0a1e99082250d6 | fnv1a64:5a64c5244b88ec8b |
| R060 | fnv1a64:72adf3d4d053f4a7 | fnv1a64:955f25afafdf446e |
| R070 | fnv1a64:22505e603c8071e2 | fnv1a64:b25f2281ae33784f |
| R080 | fnv1a64:8f46dc1fc17d11ff | fnv1a64:b6e19a1e72b3dcb6 |
| R090 | fnv1a64:cb0d046b1b79313e | fnv1a64:24c034ac4aa7d043 |
| R100 | fnv1a64:68cbd86cc975f7d4 | fnv1a64:cacd657a81a32611 |
| R110 | fnv1a64:ba273dc48c74716d | fnv1a64:ea8dd94ca60ecfdc |
| R041 | fnv1a64:55dc2b1ceb808d5a | fnv1a64:9d2bfa008bbf3e37 |
| R051 | fnv1a64:7f632e1679cf8c9f | fnv1a64:17e64309778b83d6 |
| R061 | fnv1a64:6c7cf7e6bcceaa80 | fnv1a64:279557924d0efa55 |
| R071 | fnv1a64:8cf07a6e72d729c1 | fnv1a64:6b1706840c869988 |
| R081 | fnv1a64:5e61ab773919a7fe | fnv1a64:79d50cd0ca147403 |
| R091 | fnv1a64:6561e60324154ef7 | fnv1a64:cd94b1b316dc9dbe |
| R101 | fnv1a64:6857fda022f0ffe7 | fnv1a64:3083f4733420f1ae |
| R111 | fnv1a64:74580d8ced382a18 | fnv1a64:ef826609395a3a3d |
| R12 | fnv1a64:1ac4597763f45bd8 | fnv1a64:62adb102a042e5fd |
| R13 | fnv1a64:848d00cc79ce8864 | fnv1a64:83e7e936df406b61 |
| R14 | fnv1a64:bccf50987a1e92fe | fnv1a64:d842b1f20b195f03 |
| R15 | fnv1a64:30b729ea1c7c56aa | fnv1a64:fff775059a0ea847 |
| O010 | fnv1a64:d3a290ad74d3a760 | fnv1a64:2e49f705c7e97f35 |
| O0200 | fnv1a64:93395450ffe52237 | fnv1a64:bf8c281acf3fe2fe |
| O011 | fnv1a64:b536b05cdde5b927 | fnv1a64:d7e7e237bd7108ee |
| O0210 | fnv1a64:34f8a6407df52f38 | fnv1a64:92f3165d4b69d15d |

**C01/C02 default/no-default cells:** actual emitted files are in `verification-source/.temp/brief-546/certificates/`. The complete before/after cells are `certificate-reconciliation.json`. Every non-fingerprint byte is unchanged. The following lists every distinct changed hash pair within each of the four changed cells; repeated appearances use the same mapping.

| Cell | Feature hash | Before hash | After hash |
| --- | --- | --- | --- |
| C01 | true | fnv1a64:38253ab66367efa5 | fnv1a64:f97ba6746479148d |
| C01 | true | fnv1a64:f359b9f478e116d7 | fnv1a64:31be954bd4123f0f |
| C01 | true | fnv1a64:34f8a6407df52f38 | fnv1a64:92f3165d4b69d15d |
| C01 | true | fnv1a64:b536b05cdde5b927 | fnv1a64:d7e7e237bd7108ee |
| C01 | true | fnv1a64:e109fc5a9003218d | fnv1a64:61c7930d91db32b9 |
| C01 | true | fnv1a64:93395450ffe52237 | fnv1a64:bf8c281acf3fe2fe |
| C01 | true | fnv1a64:d3a290ad74d3a760 | fnv1a64:2e49f705c7e97f35 |
| C01 | false | fnv1a64:76f9953bbd805a89 | fnv1a64:4a25f2c9e3cf5a9e |
| C01 | false | fnv1a64:8acb9d1ecbb26414 | fnv1a64:df59416d98f87cd5 |
| C01 | false | fnv1a64:34f8a6407df52f38 | fnv1a64:92f3165d4b69d15d |
| C01 | false | fnv1a64:b536b05cdde5b927 | fnv1a64:d7e7e237bd7108ee |
| C01 | false | fnv1a64:7aa37f877a395c22 | fnv1a64:88da74988ccb7e7f |
| C01 | false | fnv1a64:93395450ffe52237 | fnv1a64:bf8c281acf3fe2fe |
| C01 | false | fnv1a64:d3a290ad74d3a760 | fnv1a64:2e49f705c7e97f35 |
| C02 | true | fnv1a64:a190d45191f19f6b | fnv1a64:a7e3a12bf5d22803 |
| C02 | true | fnv1a64:44920b49eb9fd146 | fnv1a64:3ec0afe0fca7239b |
| C02 | true | fnv1a64:c7dc5a1621c37be6 | fnv1a64:59e2e11f22be401b |
| C02 | true | fnv1a64:90544fd0bb76d7bb | fnv1a64:6c64831adccb5d1a |
| C02 | true | fnv1a64:c842732cf1f848ff | fnv1a64:efdd312ba32f13b6 |
| C02 | true | fnv1a64:04101fc5d3e2678f | fnv1a64:99734c93db5147c6 |
| C02 | true | fnv1a64:fca8756a2a8ea0ac | fnv1a64:1db1971edc4ed259 |
| C02 | true | fnv1a64:6fa2c7f2d458ce5a | fnv1a64:b6f296d674977f37 |
| C02 | true | fnv1a64:3912d23cc3404fbb | fnv1a64:15230586e494d51a |
| C02 | false | fnv1a64:163fdcff0fceca07 | fnv1a64:6aa547fd9d30d4e0 |
| C02 | false | fnv1a64:fdf9db253499cc59 | fnv1a64:6ae3dd4f5ea5be24 |
| C02 | false | fnv1a64:c7dc5a1621c37be6 | fnv1a64:59e2e11f22be401b |
| C02 | false | fnv1a64:90544fd0bb76d7bb | fnv1a64:6c64831adccb5d1a |
| C02 | false | fnv1a64:c842732cf1f848ff | fnv1a64:efdd312ba32f13b6 |
| C02 | false | fnv1a64:04101fc5d3e2678f | fnv1a64:99734c93db5147c6 |
| C02 | false | fnv1a64:fca8756a2a8ea0ac | fnv1a64:1db1971edc4ed259 |
| C02 | false | fnv1a64:6fa2c7f2d458ce5a | fnv1a64:b6f296d674977f37 |
| C02 | false | fnv1a64:3912d23cc3404fbb | fnv1a64:15230586e494d51a |

The runtime closed lists retain exactly the previously disclosed 12 root-trace entries and one abandon entry; the named frame count is 556. Runtime unit and workspace tests verified those lists.

### 3. Lost continuations

**66 -> 59 by the seven named-site repair accounting.** P2056/P2061 share normal/resumed completion; P2065/P2067/P2068 retain cursors and destinations; P2066 retains the default validation spec; P2069 retains the original leaf value/schema and consumes the MatchQ answer once. All 20 original semantic reds are green. No new authoritative global Lost-site rescan was performed; the director owns that global inventory decision. No other suspension group was changed.

### 4. Performance: base and measured candidate 1

One codegen unit in both default and alloc-count arms; preserved director binaries were not rebuilt. Candidate targets were fresh. Hashes:

| Binary | Base SHA-256 | Candidate SHA-256 |
| --- | --- | --- |
| rum-cli | 74db20192d34707bd71a37486140004edb5ee032ea4d291cb88cdca2f4a32009 | 6ef9cbea47b3c9cc8808c74857057c5f04b8bd7be7f1bac7e52191e54b2fb816 |
| rum-cli-alloc | 572222df9775a3237f4e6c5754cd793ac8ebb684e243326bd0bf73542f98a124 | fb5d8dbd7ae19380b0dba73ae02a5fda3bc0418539592d5cc9776c853383eac6 |

All 27 workload results were checked on both binaries before measurement, plus 16/17/33 index-boundary controls on both. The exact statements, setups, N values and expected results are `perf/labels.json`; `.rum` hashes are `perf/sources-sha256.json`. The 18 schema labels use Do[exact statement, {i,1,N}] and 2N with setup outside. Neighbors retain their standard repeat/iterator shapes, and the three controls are copied from brief-563 definitions. No microbench script edit and no timing suite.

108 allocation processes ran first, then 216 hardware instruction processes: two paired base/candidate samples, one label per process, taskset -c 2 perf stat -x, -e cpu_core/instructions/u, stdin /dev/null. All 324 exited 0 and returned Null. Each raw stdout/stderr/perf CSV was closed before parsing. Every count is in `perf/candidate-1-paired-results.json`; every command/raw file is under `perf/{alloc,instructions}/`. Slopes are (count(2N)-count(N))/N. Only both instruction deltas >+0.05% make a regression. A single high sample is not silently discarded or remeasured.

| Label | N | Alloc base / new | Bytes base / new | Ir sample 1 base / new | Delta 1 % | Ir sample 2 base / new | Delta 2 % | Gate |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| schema-conform-list-small | 20000 | 19.0000 / 18.0000 | 1144.0000 / 1960.0000 | 27248.33100 / 27456.43925 | +0.763747 | 27178.11410 / 27455.48740 | +1.020576 | REGRESSION |
| schema-conform-association-defaults-small | 20000 | 40.0004 / 40.0004 | 3139.0066 / 3977.0066 | 29200.28945 / 30119.97580 | +3.149580 | 29200.23250 / 30149.44210 | +3.250692 | REGRESSION |
| schema-errors-list-small | 20000 | 21.0000 / 20.0000 | 1232.0000 / 2048.0000 | 27802.20315 / 27955.84350 | +0.552619 | 27708.57115 / 27837.46655 | +0.465182 | REGRESSION |
| schema-errors-association-defaults-small | 20000 | 39.0001 / 39.0001 | 3161.0017 / 3999.0017 | 26802.05535 / 27721.54485 | +3.430668 | 26958.07135 / 27720.03365 | +2.826472 | REGRESSION |
| schema-conform-list-depth-8 | 1000 | 124.0000 / 115.0000 | 5297.0000 / 7584.0000 | 73396.60500 / 74020.84200 | +0.850498 | 73414.03800 / 74017.87600 | +0.822510 | REGRESSION |
| schema-errors-list-depth-8 | 1000 | 126.0010 / 117.0010 | 9087.8000 / 11374.8000 | 67728.37400 / 68305.92900 | +0.852752 | 67706.88100 / 68312.32100 | +0.894207 | REGRESSION |
| schema-conform-list-depth-32 | 1000 | 330.0000 / 277.0000 | 25821.0000 / 34072.0000 | 182706.59000 / 180862.17000 | -1.009498 | 182670.66400 / 180872.06700 | -0.984612 | pass |
| schema-errors-list-depth-32 | 1000 | 332.0010 / 279.0010 | 26834.7120 / 35085.7120 | 150390.81000 / 147630.95900 | -1.835119 | 150387.94900 / 147613.23100 | -1.845040 | pass |
| schema-conform-list-depth-128 | 1000 | 1196.0000 / 955.0000 | 152109.0000 / 132376.0000 | 649554.75800 / 624340.27800 | -3.881810 | 649544.71700 / 624339.76900 | -3.880402 | pass |
| schema-errors-list-depth-128 | 1000 | 1198.0010 / 957.0010 | 153122.7120 / 133389.7120 | 508318.29800 / 482088.34500 | -5.160143 | 508335.04700 / 482115.09300 | -5.158006 | pass |
| schema-conform-association-depth-8 | 1000 | 386.0320 / 378.0320 | 36028.5280 / 38400.5280 | 181736.25800 / 182207.38100 | +0.259234 | 181722.11200 / 182205.95500 | +0.266254 | REGRESSION |
| schema-errors-association-depth-8 | 1000 | 372.0160 / 364.0160 | 35852.2640 / 38224.2640 | 162786.86100 / 163362.21600 | +0.353441 | 162788.55000 / 163360.93700 | +0.351614 | REGRESSION |
| schema-conform-association-depth-32 | 1000 | 1347.3800 / 1295.3800 | 170289.0480 / 180005.0480 | 499547.93100 / 495203.49700 | -0.869673 | 499540.30200 / 497336.41400 | -0.441183 | pass |
| schema-errors-association-depth-32 | 1000 | 1254.0010 / 1202.0010 | 133640.7120 / 143356.7120 | 379805.41700 / 374962.25900 | -1.275168 | 379739.30100 / 374994.32200 | -1.249536 | pass |
| schema-conform-association-depth-128 | 1000 | 5101.1600 / 4861.1600 | 687312.8880 / 691844.8880 | 1904156.98200 / 1872883.69000 | -1.642369 | 1904166.57400 / 1879039.35600 | -1.319591 | pass |
| schema-errors-association-depth-128 | 1000 | 4712.0010 / 4472.0010 | 534536.7120 / 539068.7120 | 1374948.23800 / 1352018.60900 | -1.667672 | 1374955.55700 / 1345871.72800 | -2.115256 | pass |
| schema-conform-association-index-33 | 2000 | 2843.1400 / 2845.1400 | 327044.0480 / 328876.0480 | 2074526.51150 / 2085416.77950 | +0.524952 | 2075269.21150 / 2085490.59550 | +0.492533 | REGRESSION |
| schema-errors-association-index-33 | 2000 | 2810.0700 / 2812.0700 | 324364.5240 / 326196.5240 | 2005738.21300 / 2016681.67800 | +0.545608 | 2005693.68150 / 2017151.75200 | +0.571277 | REGRESSION |
| list-literal-small | 20000 | 0.0000 / 0.0000 | 32.0000 / 32.0000 | 999.02760 / 998.97740 | -0.005025 | 999.00775 / 999.15855 | +0.015095 | pass |
| association-small-literal-repeat | 20000 | 23.0000 / 23.0000 | 2176.0000 / 2176.0000 | 24645.73180 / 24660.69525 | +0.060714 | 24661.37200 / 24660.89060 | -0.001952 | pass |
| matchq-integer-success | 20000 | 1.0000 / 1.0000 | 160.0000 / 160.0000 | 5144.11025 / 5144.31070 | +0.003897 | 5144.47710 / 5144.24575 | -0.004497 | pass |
| matchq-integer-fail | 20000 | 2.0000 / 2.0000 | 161.0000 / 161.0000 | 5266.30115 / 5266.40405 | +0.001954 | 5266.21770 / 5266.17550 | -0.000801 | pass |
| table-small | 20000 | 22.0000 / 22.0000 | 977.0000 / 977.0000 | 14635.37475 / 14634.47260 | -0.006164 | 14635.71740 / 14635.66405 | -0.000365 | pass |
| table-iterator-null | 20000 | 0.0000 / 0.0000 | 32.0000 / 32.0000 | 386.98990 / 387.27290 | +0.073129 | 387.03125 / 386.90980 | -0.031380 | pass |
| dot-three-no-input | 20000 | 596.0000 / 596.0000 | 23504.0000 / 23504.0000 | 254196.25900 / 254204.01625 | +0.003052 | 254200.17150 / 254204.05285 | +0.001527 | pass |
| differences-no-input | 20000 | 13.0000 / 13.0000 | 704.0000 / 704.0000 | 10037.59395 / 10033.45465 | -0.041238 | 10030.76215 / 10033.74040 | +0.029691 | pass |
| take-multispec-small | 20000 | 109.0000 / 109.0000 | 5856.0000 / 5856.0000 | 49950.43580 / 49948.94025 | -0.002994 | 49950.31455 / 49949.42340 | -0.001784 | pass |

The exact unrounded table is also retained verbatim:

```text
label	N	base_count	candidate-1_count	base_bytes_total	candidate-1_bytes_total	base_s1	candidate_s1	delta_s1	base_s2	candidate_s2	delta_s2	regression
schema-conform-list-small	20000	19.0	18.0	1144.0	1960.0	27248.331	27456.43925	0.7637467777384277	27178.1141	27455.4874	1.020575964099013	True
schema-conform-association-defaults-small	20000	40.0004	40.0004	3139.0066	3977.0066	29200.28945	30119.9758	3.149579566924454	29200.2325	30149.4421	3.250691925141358	True
schema-errors-list-small	20000	21.0	20.0	1232.0	2048.0	27802.20315	27955.8435	0.5526193344141506	27708.57115	27837.46655	0.4651824134208393	True
schema-errors-association-defaults-small	20000	39.0001	39.0001	3161.00165	3999.00165	26802.05535	27721.54485	3.430667864806125	26958.07135	27720.03365	2.8264718573793868	True
schema-conform-list-depth-8	1000	124.0	115.0	5297.0	7584.0	73396.605	74020.842	0.8504984665162807	73414.038	74017.876	0.8225102670418538	True
schema-errors-list-depth-8	1000	126.001	117.001	9087.8	11374.8	67728.374	68305.929	0.8527519057227151	67706.881	68312.321	0.8942074883053541	True
schema-conform-list-depth-32	1000	330.0	277.0	25821.0	34072.0	182706.59	180862.17	-1.0094983437652583	182670.664	180872.067	-0.9846118476910859	False
schema-errors-list-depth-32	1000	332.001	279.001	26834.712	35085.712	150390.81	147630.959	-1.8351194464608533	150387.949	147613.231	-1.8450401235274416	False
schema-conform-list-depth-128	1000	1196.0	955.0	152109.0	132376.0	649554.758	624340.278	-3.8818097611410307	649544.717	624339.769	-3.880402278754891	False
schema-errors-list-depth-128	1000	1198.001	957.001	153122.712	133389.712	508318.298	482088.345	-5.160143379296578	508335.047	482115.093	-5.1580063493045	False
schema-conform-association-depth-8	1000	386.032	378.032	36028.528	38400.528	181736.258	182207.381	0.2592344561204607	181722.112	182205.955	0.26625433453029146	True
schema-errors-association-depth-8	1000	372.016	364.016	35852.264	38224.264	162786.861	163362.216	0.35344068708345056	162788.55	163360.937	0.35161379593344755	True
schema-conform-association-depth-32	1000	1347.38	1295.38	170289.048	180005.048	499547.931	495203.497	-0.8696731045013562	499540.302	497336.414	-0.4411832220896583	False
schema-errors-association-depth-32	1000	1254.001	1202.001	133640.712	143356.712	379805.417	374962.259	-1.2751682264710795	379739.301	374994.322	-1.2495359283341556	False
schema-conform-association-depth-128	1000	5101.16	4861.16	687312.888	691844.888	1904156.982	1872883.69	-1.6423694209892647	1904166.574	1879039.356	-1.3195913815048477	False
schema-errors-association-depth-128	1000	4712.001	4472.001	534536.712	539068.712	1374948.238	1352018.609	-1.6676721614883094	1374955.557	1345871.728	-2.1152559333232457	False
schema-conform-association-index-33	2000	2843.14	2845.14	327044.048	328876.048	2074526.5115	2085416.7795	0.5249519801087343	2075269.2115	2085490.5955	0.49253291781898856	True
schema-errors-association-index-33	2000	2810.07	2812.07	324364.524	326196.524	2005738.213	2016681.678	0.5456078429912248	2005693.6815	2017151.752	0.5712771898164881	True
list-literal-small	20000	0.0	0.0	32.0	32.0	999.0276	998.9774	-0.005024886199345158	999.00775	999.15855	0.015094977991902958	False
association-small-literal-repeat	20000	23.0	23.0	2176.0	2176.0	24645.7318	24660.69525	0.06071416390240891	24661.372	24660.8906	-0.0019520406245088218	False
matchq-integer-success	20000	1.0	1.0	160.0	160.0	5144.11025	5144.3107	0.0038966894226222237	5144.4771	5144.24575	-0.004497055687158191	False
matchq-integer-fail	20000	2.0	2.0	161.0	161.0	5266.30115	5266.40405	0.0019539330750273365	5266.2177	5266.1755	-0.000801334133981868	False
table-small	20000	22.0	22.0	977.0	977.0	14635.37475	14634.4726	-0.006164174238187936	14635.7174	14635.66405	-0.00036451920013558237	False
table-iterator-null	20000	0.0	0.0	32.0	32.0	386.9899	387.2729	0.07312852350926935	387.03125	386.9098	-0.03137989503431138	False
dot-three-no-input	20000	596.0	596.0	23504.0	23504.0	254196.259	254204.01625	0.0030516774835787075	254200.1715	254204.05285	0.0015268872468165995	False
differences-no-input	20000	13.0	13.0	704.0	704.0	10037.59395	10033.45465	-0.041237970181096095	10030.76215	10033.7404	0.029691163597167503	False
take-multispec-small	20000	109.0	109.0	5856.0	5856.0	49950.4358	49948.94025	-0.002994067971673342	49950.31455	49949.4234	-0.0017840728492535263	False
```

**Attribution, not a substituted acceptance gate:** all ten residual labels were run through Callgrind at 10 and 20 iterations on both binaries (40 bounded processes, all exit 0). The exact statement/setup is unchanged. The schema_self column includes every function whose demangled name contains rum_stdlib::schema::, including drops of those types. Self-only sums exactly equal each profile's instruction total; the reducer excludes the cost row following calls= and does not add recursive inclusive costs. Raw profiles/commands and per-function deltas are under `attribution/`, especially `details.json`.

```text
label	base_total	candidate-1_total	base_schema_self	candidate-1_schema_self	candidate-1_continuation_self
schema-conform-list-small	27240.8	27928.7	867.0	1534.0	0
schema-conform-association-defaults-small	30165.0	30602.3	1421.0	2095.0	0
schema-errors-list-small	28236.6	28308.5	936.0	1596.0	0
schema-errors-association-defaults-small	26811.3	27573.7	1490.0	2157.0	0
schema-conform-list-depth-8	75199.0	75264.8	1603.0	3825.0	0
schema-errors-list-depth-8	67173.0	67360.0	1672.0	3887.0	0
schema-conform-association-depth-8	185973.7	184888.5	5448.0	8461.0	0
schema-errors-association-depth-8	164644.6	164893.9	5517.0	8523.0	0
schema-conform-association-index-33	2080149.8	2089329.2	18675.8	23188.2	0
schema-errors-association-index-33	2011316.5	2019761.0	18723.5	23257.8	0
```

The flat driver owns its work records and ordinary Expr handles. For the short list, schema self-cost rises 867 -> 1534 Ir/validation, with Expr::clone self-cost +168; allocator/path work offsets part of it (format_inner -219, String::write_str -101). For small association defaults, schema self-cost rises 1421 -> 2095, Expr::clone +46 and Vec growth is visible. The allocation-count slope is unchanged there, but bytes rise by 838; short list bytes rise by 816 despite one fewer allocation. These measured costs are consistent with the new work Vec, cursor transitions and owned handles, partly offset by removal of copied paths. They are not continuation capture costs: no continuation-module self-cost appears in any of the 40 no-input profiles, and capture/working-copy construction remains on the cold input/resume edges.

At index 33, the designed owned String-key HashMap replaces the old borrowed-key local index. Allocation slope rises by two and bytes by 1832 after path-copy savings; conform schema self-cost rises 18675.8 -> 23188.2, Expr::clone self-cost +980 and aggregate HashMap::insert self-cost about +2793.7 in this attribution sample. The latter also contains runtime caches and is not claimed to isolate only the schema index. The 8-level residuals show the same flat-driver cost before the longer-path savings dominate. At depths 32 and 128 all eight rows improve in both hardware samples, up to about 5.16%. All nine neighbor/control labels pass the prescribed two-sample rule. Callgrind totals have their own startup/allocator variability and do not replace or waive the hardware failures.

### 5. Counters, residuals and handoff

- Production implementations: one; resumed phase changed only the new tests and authorized maintenance. Measured candidates: **1/6**. No-progress compiling-candidate counter: **0**; stop-loss did not trigger. No optimization candidate was tried after the first panel.
- Semantic reds **20 -> 0**; current focused suite **40/40** and structural/direct suite **10/10**. Release depth **8/8** under the clarified contract. No native process abort and no lowered n.
- Performance: **10 regressions**, largest recurring increase min(sample1,sample2) **+3.149580%**, small association defaults conform. Largest single increase is +3.430668% on small association defaults errors. All rows and attribution are above.
- Per the explicit performance ruling, these residuals need the director's decision; this is **not a performance-approved merge result**. No guard, retention class, inventory exclusion, recursion fallback, depth cap, diagnostic counter, timing metric or alternate representation was introduced to make a gate appear green.
- Adjacent boundaries: public result-height failure remains outside the validator and is matched by the fresh control; native loopback tests are nine skips. Existing Expr destruction/visitation/display and matcher depth debt remain untouched.
- Full workspace command exit 101 is preserved honestly; its only failing target passed the supported retry with nine skips. Every other target passed. No other unresolved functional failure was found.

The final scope audit, command ledger, exact workloads, raw output and `.temp/brief-568/PROGRESS.md` provide the handoff. The source compiles, no source file is half-edited, standing inventories are current, and all changes remain uncommitted.
