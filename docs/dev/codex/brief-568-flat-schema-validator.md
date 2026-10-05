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
