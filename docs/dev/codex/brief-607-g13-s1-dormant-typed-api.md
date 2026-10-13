# brief-607 — G13 step S1: the dormant typed API and per-type transport mode

## Goal

Add G13's typed continuation API to `rum-runtime`, **dormant**: nothing in the
live driver, no production frame converted, evaluation unchanged. It contains
the opaque typed frame/action/finalizer API, the input witness, `FrameChain`,
`ContinueToken`, `ResumeStop`/`finish_work`, the sealed keyed codec with **one
associated transport mode per slot type**, the 21 existing codec types
registered as `Legacy`, a source census of the frames still to type, and the
22 compile-control pairs that first apply at this step. When you are done the
API compiles, its controls reject exactly what they should, the census reports
the baseline, and every existing gate is unchanged.

This is step S1 of G13's 78-step implementation (design round 5, brief-606,
ratified by the director). S2 activates the driver; S3-S54 type the frames;
F1-F23 flip the transport per type; R removes the legacy machinery.

## Context pointers

- `AGENTS.md`, `docs/testing-strategy.md`, `docs/code-organization-guidelines.md`.
- The design, in order of authority: this brief; `docs/dev/codex/brief-606-g13-design-round-5-type-then-flip.md`
  (its D1/D2 and the appended report) with `.temp/brief-606/design.md`;
  `.temp/brief-605/design.md` for everything round 5 did not change (API,
  token, deferred families, routes, fatal unwind, accounting).
- The compiled model `.temp/brief-606/api_prototype.rs` (four tests passed in
  round 5 and at the director's gate) is your starting point for the mode
  strategies and visibility; it is a model, not production code.
- `.temp/brief-606/compile-controls.tsv`: the 22 rows with `first_gate = S1`
  are this step's controls. `.temp/brief-606/slot-modes.tsv`: the 21 types
  with `introducing_step = S1` and `initial_mode = Legacy`.
  `.temp/brief-606/census-spec.md`: the census you build.
- These tables are inputs: read them, never edit them.

## Scope

Only these files may change, and only for this step's work. A listed file is
not permission to convert a frame, change evaluation or tidy. New files are
marked.

- `crates/rum-runtime/src/runtime/debug_tools/mod.rs` (added at redispatch 1)
- `crates/rum-runtime/src/runtime/debug_tools/test_report.rs`
- `crates/rum-runtime/src/runtime/debug_tools/test_report/model.rs` (added at redispatch 1)
- `crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs`
- `crates/rum-runtime/src/runtime/definitions/application/mod.rs`
- `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_catalog.rs`
- `crates/rum-runtime/src/runtime/definitions/definition_catalog/traversal_continuation.rs`
- `crates/rum-runtime/src/runtime/eval_call_prepare.rs`
- `crates/rum-runtime/src/runtime/kernel_api.rs`
- `crates/rum-runtime/src/runtime/kernel_data/evaluated_part_selectors.rs`
- `crates/rum-runtime/src/runtime/kernel_data/mod.rs`
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data.rs`
- `crates/rum-runtime/src/runtime/kernel_data/structure_data/mod.rs` (added at redispatch 1)
- `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace.rs`
- `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/plan.rs`
- `crates/rum-runtime/src/runtime/mod.rs`
- `crates/rum-runtime/src/runtime/pattern_matching/mod.rs`
- `crates/rum-runtime/src/runtime/pattern_matching/outcome.rs`
- `crates/rum-runtime/src/runtime/suspension.rs`
- `crates/rum-runtime/src/runtime/suspension/collections.rs`
- `crates/rum-runtime/src/runtime/suspension/primitive.rs`
- `crates/rum-runtime/src/runtime/suspension/rewrite_rules.rs`
- `crates/rum-runtime/src/runtime/suspension/traversal.rs`
- `crates/rum-runtime/src/runtime/suspension/typed.rs` (new)
- `crates/rum-runtime/src/runtime/suspension/typed/action.rs` (new)
- `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs` (new)
- `crates/rum-runtime/src/runtime/suspension/typed/chain.rs` (new)
- `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` (new)
- `crates/rum-runtime/src/runtime/suspension/typed/legacy.rs` (new)
- `crates/rum-runtime/src/runtime/suspension/typed/registry.rs` (new)
- `crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs` (new)
- `crates/rum-runtime/src/runtime/suspension/typed/witness.rs` (new)
- `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs`
- `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs`
- `crates/rum-runtime/src/runtime/tests/mod.rs`
- `crates/rum-runtime/src/runtime/tests/typed_slots.rs` (new)
- `crates/rum-runtime/src/runtime/tests/typed_slots/accounting.rs` (new)
- `crates/rum-runtime/src/runtime/tests/typed_slots/composition.rs` (new)
- `crates/rum-runtime/src/runtime/tests/typed_slots/faults.rs` (new)
- `crates/rum-runtime/src/runtime/tests/typed_slots/registry.rs` (new)
- `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger.rs` (added at redispatch 2)
- `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/ownership.rs` (added at redispatch 2)
- `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs` (added at redispatch 2)
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv`
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv`
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv`
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv`
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv`
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv`
- `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_storage.tsv`
- `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv`
- `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv`
- `crates/rum-stdlib/src/tests/retention_closure/local/contracts.rs`
- `crates/rum-stdlib/src/tests/retention_closure/local/mod.rs`
- `crates/rum-stdlib/src/tests/retention_closure/local/resume_macro.rs`
- `crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs`
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs`
- `crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs` (new)
- `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-shapes.tsv` (new)
- `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` (new)
- `crates/rum-stdlib/tests/typed_slot_census.rs` (new)
- `crates/rum-stdlib/tests/typed_slot_census/classify.rs` (new)
- `crates/rum-stdlib/tests/typed_slot_census/scan.rs` (new)
- `crates/rum-stdlib/tests/typed_slot_compile_controls.rs` (new)

Manifests, `Cargo.lock`, the toolchain, docs, `AGENTS.md`, `TODO.md` and the
core `Expr`/`Symbol` representation stay unchanged; S2 owns the two feature
edits. Scratch, compiler fixtures and logs go under `.temp/brief-607/`, and the
certificate replay may write its usual output under
`.temp/brief-546/certificates/`. Append your report to this file.

**Visibility on a registered type's path (added at redispatch 1).** For the
21 types registered here and the codec functions they delegate to, you may
make *visibility-only* edits in any module file on the path from
`crate::runtime` to the declaration: raise the item or a `mod` declaration to
`pub(in crate::runtime)`, or add a `pub(in crate::runtime) use` re-export.
Nothing wider than `pub(in crate::runtime)`, no field visibility, no other
change in such a file. List every one in the report with file and line.

**The ownership-graph test module (added at redispatch 3).** To implement
the verifier rule of redispatch 2 you may edit any Rust file under
`crates/rum-stdlib/src/tests/ownership_graph/` (test code: discovery model,
collectors, resolution, the ledger verifier, fixtures), for that rule and its
falsifiers only. Discovery must keep producing the same rows for everything
else: a changed or vanished unrelated row is still a STOP. List every edited
file in the report.

**Disclosure duty instead of a longer list.** If a necessary change falls
outside the list (a module root that must re-export, a visibility change in an
owner file not listed), make no edit there: STOP and report the file, the line
and why. Do not route around the fence.

## Steps

1. **The API.** Implement the accepted opaque `TypedFrame`, `TypedAction`
   (`complete`, `evaluate`, `finalize`, `finish_work`), `SlotFinalizer`, the
   invariant zero-sized `SlotWitness` taken from an actual `&Result`,
   `FrameChain<I, O>` with no unbound constructor, the non-`Clone`,
   adapter-minted, lifetime-scoped `ContinueToken` holding no `Arc`, and
   `ResumeStop<O>`. `TypedFrame` does not inherit the old `SuspensionFrame`.
   Public items are exactly the list in `.temp/brief-605/design.md` §1, minus
   what S2 adds.
2. **The mode.** The hidden keyed `Codec` declares one associated `Mode`;
   central free functions do the channel encode/decode, so no slot
   implementation can override them. `ValueMode` for `Expr` only;
   `LegacyMode` matches only `Slot::Value` and hands the `Expr` to the
   existing codec; `StateMode` matches only `Slot::State` and then the exact
   variant. A legacy provider receives `Expr`, a state provider receives
   `ContinuationState`; neither ever sees `Slot`. No runtime switch, no
   per-frame choice, nothing public that names a mode.
3. **Registration.** Register the 21 `S1` types as `Legacy`, delegating to
   their existing codecs with wire layout and decoder behaviour unchanged,
   including today's leniency: that is deliberate until each type flips. Do
   not register the two `PreparedInterpreted*` enums (S33) or any new carrier.
4. **Dormant means dormant.** No adapter in the queue, no typed insertion, no
   `resume_typed_or_suspend!` export: those arrive with the driver in S2. No
   stub that panics, pretends to suspend, or inserts nothing. Existing frames,
   macros and public behaviour are unchanged.
5. **Compile controls.** Implement the 22 `S1` pairs in
   `crates/rum-stdlib/tests/typed_slot_compile_controls.rs`. For each: the
   positive fixture compiles first; exactly one intended invalidity is
   introduced; the test checks the primary diagnostic's code/category and
   span, not merely a failed compile. Fixtures that need runtime-private
   access compile a scratch copy under `.temp/brief-607/`; never edit the
   working production crate to make a fixture fail. A control that fails only
   because an API is missing is not a valid control.
6. **The census.** Build `typed_slot_census` per `census-spec.md`: a source
   gate with module-qualified identities and an explicit unresolved list, not
   a spelling count. It must report the baseline: 623 frame/finalizer
   implementations and 600 production macro sites, all untyped, and the 23
   legacy types with their live decoder uses. Show it can fail: run it once
   against a scratch copy with one frame implementation removed and paste the
   red output.
7. **Inventories.** Replay source fingerprints, both builds' earned
   certificates and ownership discovery; update the frame derive/named-frame
   inventories for the new types. Disclose every changed row. Planned
   transitions in `.temp/brief-606/inventory-transitions.tsv` are authorized;
   any other identity, status or disposition change, a vanished unrelated
   row, or a new `UnknownHostPayload` without an approved disposition is a
   STOP. Keep recorded destruction debt as it is.

## Verification (MANDATORY — run to green or evidence)

Serial, each through `python3 scripts/run-bounded.py --timeout <s> -- env
CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true
TMPDIR=<repo>/.temp/brief-607/tmp …`. Bounds from the last gates (the
workspace took ~10 minutes warm): workspace 3 600 s; oracle 1 500 s per build;
the others 900 s.

```
cargo fmt --check
cargo test -p rum-stdlib --test typed_slot_compile_controls
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls
cargo test -p rum-stdlib --test typed_slot_census
cargo test -p rum-stdlib --no-default-features --test typed_slot_census
scripts/check-suspension-oracle.sh 1500
cargo test -p rum-stdlib --lib ownership_graph
cargo test -p rum-stdlib --lib local_production_acceptance
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance
cargo test --workspace --no-fail-fast
```

Report the `test result:` totals and native ZeroMQ skips as skips.

**Performance.** This step is exempt from measurement only if no function
body on an evaluator, matcher, traversal, block or selector path changes:
list every production function whose body you changed (trait impls and
visibility edits included) and say why each is not on such a path. If any is,
measure per `TEMPLATE.md` pitfall 8: allocation counts first; then
per-iteration instruction slopes (`taskset -c 2 perf stat -x, -e
cpu_core/instructions/u`, `(count(2N) - count(N)) / N`, stdin `/dev/null`) on
the affected labels and four untouched controls, base and candidate both
built with `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` into separate target
directories under `.temp/brief-607/` (base built before your first edit);
then the full `scripts/core-language-microbench.sh` panel on both binaries as
an indication, same-binary spread first. Profiling env off.

## rum / Rust pitfalls (read twice)

The `TEMPLATE.md` list applies. In particular: rum against rum only; no
default-path cost (no counter, `Mutex` or allocation on any evaluation path);
no owned-tree clones; the witness comes from the actual `Result`; the token
carries no `Arc`; the codec, mode and actions stay sealed and opaque. New
modules prefer under 10 k characters; check
`.temp/brief-606/file-size-assessments.tsv` before adding to a large file.

## Checkpointing — protect the scene

Everything under `.temp/brief-607/`; `PROGRESS.md` after each step with the
verified state, compile/test counters and the exact next action.

## Hard rules

- **No `git`, in any form.** No network; nothing outside this repository
  except the certificate replay output named above.
- Modify only the Scope list. No new `unsafe`; no dependency, lockfile or
  toolchain change.
- Commands: scoped reads and edits, `rg`/`grep`, bounded `python3`, the bounded
  Cargo commands above and bounded compiler fixtures,
  `scripts/check-suspension-oracle.sh`, and the measurement commands named
  above. No probe that aborts a process.
- **Testing Policy is a hard line** (`AGENTS.md`): rum against rum only.
- No dual-arm decoder, per-frame mode override, transit record or public raw
  adapter. Do not weaken an oracle `Checks` flag, a retention reject or a
  documented test to pass.
- Stop-loss keyed on progress: compile errors you are reducing are progress;
  stop after two attempts that move none of the named failing gates.
- Paste real output; say plainly what did not run or did not pass. Leave the
  tree uncommitted.

## Report format (append to this brief)

1. Files changed, each with the semantic change; the dormant audit (every
   production function body touched and why it is off the hot path).
2. The public item list as built; the mode as built; the 21 registrations.
3. Compile controls: each pair's positive result and rejected diagnostic.
4. The census output and its forced red.
5. All gates verbatim; inventory and certificate transitions.
6. Stop-loss counters, deviations and open questions.

---

## Execution report — stopped at the scope fence

### 1. Files changed and dormant audit

No production or test source was changed. This brief received this appended
report. Scratch files are `.temp/brief-607/PROGRESS.md`,
`scope-stop-evidence.txt`, and `source-snapshot.json`; the requested scratch
`tmp/` directory was created. The snapshot hashes the `.rs`, `.py`, and `.toml`
files under `crates/` for the unchanged-source check.

Production function bodies touched: **none**. Trait implementations and
production visibility declarations changed: **none**. Evaluation remains
unchanged by this dispatch. No performance measurement ran.

The required `EvalCapturedOutcome` registration encounters two out-of-scope
visibility sites:

- `crates/rum-runtime/src/runtime/debug_tools/test_report/model.rs:29`: the
  existing type is visible only inside `test_report`. The accepted central
  registration needs the existing type to be `pub(in crate::runtime)`; its
  fields can retain their current visibility. A re-export in the permitted
  `test_report.rs` cannot widen this declaration's visibility.
- `crates/rum-runtime/src/runtime/debug_tools/mod.rs:14`: `mod test_report;`
  leaves the module path private to `debug_tools`. The central registry also
  needs runtime-visible access through this module declaration or a re-export
  in this same out-of-scope owner file.

The accepted visibility design is `.temp/brief-605/design.md` section 1,
lines 27–29; round 5 retains that boundary. The slot is explicitly assigned
`Legacy` at `S1` in `.temp/brief-606/slot-modes.tsv:21`. Moving its declaration,
relocating registration to evade the central private boundary, or omitting
the required slot would route around the fence. None was done.

### 2. Public items, mode and registrations

Not implemented. No new public items, modes, adapters, suspension methods,
macros, or registrations were added. Required S1 registrations completed: 0/21.

### 3. Compile controls

Not implemented or run: 0/22 positive/negative pairs. No compiler diagnostic
is claimed, and no missing-API failure is counted as a control.

### 4. Census and forced red

Not implemented or run. The planned 623-implementation / 600-macro baseline
was not remeasured, and the required scratch-copy forced red was not run.

### 5. Verification, evidence and inventories

The bounded source-evidence check succeeded with exit code 0. Its actual
output is in `.temp/brief-607/scope-stop-evidence.txt`:

```text
crates/rum-runtime/src/runtime/debug_tools/test_report/model.rs:29:pub(in crate::runtime::debug_tools::test_report) struct EvalCapturedOutcome {
crates/rum-runtime/src/runtime/debug_tools/mod.rs:14:mod test_report;
crates/rum-runtime/src/runtime/debug_tools/test_report/evaluation.rs:5:impl ContinuationValue for EvalCapturedOutcome {
.temp/brief-605/design.md:28:become `pub(in crate::runtime)`; their fields need not become public. Their
docs/dev/codex/brief-607-g13-s1-dormant-typed-api.md:109:owner file not listed), make no edit there: STOP and report the file, the line
OUTSIDE_SCOPE crates/rum-runtime/src/runtime/debug_tools/test_report/model.rs
OUTSIDE_SCOPE crates/rum-runtime/src/runtime/debug_tools/mod.rs
REQUIRED_REGISTRATION slot=EvalCapturedOutcome initial_mode=Legacy introducing_step=S1
STOP: required owner-type and module visibility changes are outside brief-607 Scope.
```

The bounded check used the required serial environment:

```text
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-607/tmp python3 -
```

The mandatory implementation gates did **not run**, because the explicit
scope STOP was reached before implementation:

```text
cargo fmt --check — NOT RUN
cargo test -p rum-stdlib --test typed_slot_compile_controls — NOT RUN
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls — NOT RUN
cargo test -p rum-stdlib --test typed_slot_census — NOT RUN
cargo test -p rum-stdlib --no-default-features --test typed_slot_census — NOT RUN
scripts/check-suspension-oracle.sh 1500 — NOT RUN
cargo test -p rum-stdlib --lib ownership_graph — NOT RUN
cargo test -p rum-stdlib --lib local_production_acceptance — NOT RUN
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance — NOT RUN
cargo test --workspace --no-fail-fast — NOT RUN
```

There are no `test result:` totals from this dispatch. Native ZeroMQ tests
were not run; no executed skip is claimed. Source fingerprints, ownership
inventories, derive/named-frame lists and certificate cells were not edited
or replayed. Inventory row transitions: none. Certificate transitions: none.

### 6. Stop-loss, deviations and next action

Implementation attempts: 0. Consecutive no-progress gate attempts: 0.
Implementation steps completed: 0/7. Compile-control pairs executed: 0/22.
Mandatory verification commands executed: 0/10. The explicit Scope rule
triggered this STOP; the two-attempt stop-loss did not trigger.

No git command, network operation, out-of-scope edit, or production workaround
was performed. No new unsafe code or dependency change was introduced.

Next action belongs to the director: authorize the two exact owner paths above
in a revised scope, or revise the accepted registration/visibility design.
This dispatch does not claim S1 is complete.

Final bounded source-snapshot verification (exit code 0):

```text
SOURCE_UNCHANGED files=1408 changed=0 added=0 removed=0
```

## Director amendment — redispatch 1

The stop above was correct: round 5's scope omitted the owner files of
`EvalCapturedOutcome`. The director checked all 21 registrations' paths and
added the three files that block them (`debug_tools/mod.rs`,
`debug_tools/test_report/model.rs`, and `kernel_data/structure_data/mod.rs`,
which keeps `ReplacePartTarget`'s module private), plus the visibility rule in
Scope for anything else on those paths. Proceed with Steps 1-7; append a new
`## Execution report` after this section.


## Execution report — redispatch 1: stopped on unapproved host-payload rows

### 1. Files changed and dormant audit

The amended visibility scope unblocked implementation. The dormant API now
compiles, but **S1 is incomplete**: the early ownership replay found new
`UnknownHostPayload` rows without approved dispositions. Per Step 7, work
stopped at that evidence. No production edits followed that STOP.

Exactly 20 production files changed (11 existing, 9 new). No test source,
manifest, lockfile, core representation, or inventory was changed. The
bounded baseline comparison found no out-of-scope source changes or removed
files. All paths below are under `crates/rum-runtime/src/runtime/`.

| File | Change |
| --- | --- |
| `debug_tools/mod.rs` | Make the test_report module runtime-visible (line 14). |
| `debug_tools/test_report.rs` | Re-export EvalCapturedOutcome within runtime (line 10). |
| `debug_tools/test_report/model.rs` | Widen only the EvalCapturedOutcome type item to runtime visibility (line 29). |
| `definitions/application/mod.rs` | Re-export ReplaceAllTraversalOutcome within runtime (line 8). |
| `definitions/application/replace_all.rs` | Widen only the ReplaceAllTraversalOutcome type item (line 13). |
| `eval_call_prepare.rs` | Add a dormant PreparedCallArguments::into_continuation_expr method reusing the existing suspended wire encoder (line 18); existing bodies unchanged. |
| `kernel_api.rs` | Export the requested S1 public typed API (line 9). |
| `kernel_data/rewrite_data.rs` | Widen only DeleteCasesOutcome to runtime visibility (line 23). |
| `kernel_data/structure_data/mod.rs` | Make the replace module runtime-visible (line 5). |
| `kernel_data/structure_data/replace.rs` | Widen only ReplacePartTarget to runtime visibility (line 51). |
| `suspension.rs` | Declare the private typed module and re-export only its public API. |
| `suspension/typed.rs` | Declare the sealed SlotValue trait, TypedFrame and SlotFinalizer; no old SuspensionFrame supertrait. |
| `suspension/typed/action.rs` | Implement opaque actions and ResumeStop with typed completion/evaluation/finalizer bounds. |
| `suspension/typed/adapter.rs` | Implement private, dormant frame/finalizer erasure, decode rejection, trace/abandon/boundary delegation and the sole token mint; no queue installation. |
| `suspension/typed/chain.rs` | Implement witnessed, nonempty invariant chains and checked adjacency; no public unbound constructor or suspension method. |
| `suspension/typed/channel.rs` | Implement keyed central transport with one associated mode and one accepted channel arm per mode. |
| `suspension/typed/legacy.rs` | Delegate the 21 registered payloads to their existing wire codecs. |
| `suspension/typed/registry.rs` | Select Expr=Value and 21 existing payloads=Legacy; provide their closed state variants for later flips. |
| `suspension/typed/resume_token.rs` | Define a non-Clone, invariant lifetime-scoped ZST token without Arc storage or a public constructor. |
| `suspension/typed/witness.rs` | Bind a ZST invariant witness to an actual borrowed Result and build the first checked chain link. |

The eight item/module visibility edits are identified above with exact current
lines. They stop at `pub(in crate::runtime)`; no field visibility changed.

All **pre-existing function bodies remain byte-identical** to the redispatch
baseline. The audit replayed the exact permitted item/import/export changes
and the one added PreparedCallArguments method against the saved originals.
There are **109 new production function bodies**, plus the generated
`TypedAdapter<F>::trace_roots` derive. The complete file/line/function inventory
is `.temp/brief-607/redispatch-1/function-body-audit.tsv`; the exact old-body
check is `existing-body-audit.txt`.

For each type in the 21-registration table below, the four new bodies are
`LegacyCodec::old_encode`, `LegacyCodec::old_decode`, `StateCodec::pack`, and
`StateCodec::unpack` (84 bodies). Every one is used only by the dormant typed
transport; the live driver still calls the old codecs directly. The other
25 new bodies are listed here, with the same dormant-only disposition:

| File | Line | Owner | Function |
| --- | ---: | --- | --- |
| `suspension/typed.rs` | 41 | `pub trait TypedFrame: FrameRootTrace + FrameAbandon + Send + Sync + Sized + 'static` | `output_boundary` |
| `suspension/typed/action.rs` | 29 | `impl<O: SlotValue> TypedAction<O>` | `complete` |
| `suspension/typed/action.rs` | 36 | `impl<O: SlotValue> TypedAction<O>` | `evaluate` |
| `suspension/typed/action.rs` | 46 | `impl<O: SlotValue> TypedAction<O>` | `finalize` |
| `suspension/typed/action.rs` | 53 | `impl<O: SlotValue> TypedAction<O>` | `finish_work` |
| `suspension/typed/action.rs` | 70 | `impl<O: SlotValue> ResumeStop<O>` | `failure` |
| `suspension/typed/adapter.rs` | 28 | `impl<F: TypedFrame> FrameAbandon for TypedAdapter<F>` | `abandon` |
| `suspension/typed/adapter.rs` | 34 | `impl<F: TypedFrame> ErasedFrame for TypedAdapter<F>` | `resume` |
| `suspension/typed/adapter.rs` | 56 | `impl<F: TypedFrame> ErasedFrame for TypedAdapter<F>` | `output_boundary` |
| `suspension/typed/adapter.rs` | 61 | `<module>` | `erase` |
| `suspension/typed/adapter.rs` | 70 | `impl<F: SlotFinalizer> ErasedFinalizer for FinalizerAdapter<F>` | `finish` |
| `suspension/typed/chain.rs` | 12 | `<module>` | `start` |
| `suspension/typed/chain.rs` | 22 | `impl<I: SlotValue, O: SlotValue> FrameChain<I, O>` | `then` |
| `suspension/typed/channel.rs` | 9 | `impl CodecKey` | `new` |
| `suspension/typed/channel.rs` | 48 | `<module>` | `encode` |
| `suspension/typed/channel.rs` | 52 | `<module>` | `decode` |
| `suspension/typed/channel.rs` | 76 | `impl Transport<Expr> for ValueMode` | `encode` |
| `suspension/typed/channel.rs` | 79 | `impl Transport<Expr> for ValueMode` | `decode` |
| `suspension/typed/channel.rs` | 88 | `impl<T: LegacyCodec> Transport<T> for LegacyMode` | `encode` |
| `suspension/typed/channel.rs` | 91 | `impl<T: LegacyCodec> Transport<T> for LegacyMode` | `decode` |
| `suspension/typed/channel.rs` | 100 | `impl<T: StateCodec> Transport<T> for StateMode` | `encode` |
| `suspension/typed/channel.rs` | 103 | `impl<T: StateCodec> Transport<T> for StateMode` | `decode` |
| `suspension/typed/witness.rs` | 10 | `<module>` | `slot_witness` |
| `suspension/typed/witness.rs` | 15 | `impl<I: SlotValue> SlotWitness<I>` | `chain` |
| `eval_call_prepare.rs` | 17 | `impl PreparedCallArguments` | `into_continuation_expr` |

The derive delegates borrowed tracing to the inner Arc<F>. It cannot reach
a live queue in S1. The PreparedCallArguments adapter method calls the old
encoder from the dormant provider only; no old caller was redirected. No
evaluator, matcher, traversal, block or selector function body changed.
Performance measurement therefore did not run under the S1 exemption.

Size assessment: `eval_call_prepare.rs` is 22,259 characters; the only added
body is its 14-line owner-local wire adapter, which needs private accumulator
types. A later cohesive split can separate wire encoding from the preparation
driver; this dispatch did not rewrite live code to split it.
`typed/registry.rs` is 11,289 characters (warning range): it keeps the explicit
closed registration/state-provider table together. Split registration from
state providers when that table grows. Every other new module is below 10k.
`files-changed.tsv` records all measured sizes.

### 2. Public items, transport and registrations

Built and exported through `kernel_api`: `SlotValue`, `TypedFrame`,
`SlotFinalizer`, `TypedAction::{complete,evaluate,finalize,finish_work}`,
`SlotWitness`, `slot_witness`, `SlotWitness::chain`, `FrameChain`,
`FrameChain::then`, `ContinueToken`, `ResumeStop`, and `ResumeStop::failure`.
The two traits retain their required associated types and methods;
TypedFrame retains `output_boundary`. No suspension methods, macro export,
queue adapter, raw constructor, public mode, key or codec was added.

The hidden Codec contains only its associated Mode. Central free encode/decode
functions dispatch to ValueMode, LegacyMode or StateMode; legacy providers
receive Expr and state providers receive ContinuationState. Expr has ValueMode
and no StateCodec. Every existing registered non-Expr payload remains Legacy;
the closed state providers match their exact variant. There is no runtime
switch, frame override, dual-arm fallback or transit record.

The production providers exposed one difference from the compiled model:
`PreparedCallArguments` has a fallible free-function decoder and an encoder
accepting `SuspendedPreparedCallArguments`, rather than a ContinuationValue
implementation. The first compile rejected the assumed trait implementation
with two E0277 errors. The fix preserves the old decoder unchanged:
LegacyCodec::old_decode returns `Result<Self, Expr>`; the ordinary codecs
wrap their old result in Ok, while PreparedCallArguments returns the existing
decoder's Result directly. LegacyMode preserves rejection as Err(Slot::Value)
without trying State. The added owner-local encode method consumes the
prepared fields into the existing suspended accumulator representation and
calls `finish_suspended_prepared_call`. This has compiled, but its layout and
error behavior still need the planned controls; no runtime equivalence claim
is made from compilation alone. The director should review this necessary
fallibility adaptation before S2 connects the driver.

| Registered slot | Associated mode |
| --- | --- |
| `Expr` | `ValueMode` |
| `Option<Expr>` | `LegacyMode` |
| `Vec<Expr>` | `LegacyMode` |
| `Vec<SymbolExprMap>` | `LegacyMode` |
| `Option<Vec<Expr>>` | `LegacyMode` |
| `Option<Vec<RewriteRuleInfo>>` | `LegacyMode` |
| `Option<CasesLevelSpec>` | `LegacyMode` |
| `bool` | `LegacyMode` |
| `String` | `LegacyMode` |
| `CompiledMatchOutcome` | `LegacyMode` |
| `CasesTraversalResolution` | `LegacyMode` |
| `TraversalWithoutLimitResolution` | `LegacyMode` |
| `CasesCollectionOutcome` | `LegacyMode` |
| `PositionCollectionOutcome` | `LegacyMode` |
| `DeleteCasesOutcome` | `LegacyMode` |
| `TraversalCountOutcome` | `LegacyMode` |
| `ReplaceOutcome` | `LegacyMode` |
| `ReplaceAllTraversalOutcome` | `LegacyMode` |
| `Vec<ReplacePartTarget>` | `LegacyMode` |
| `EvalCapturedOutcome` | `LegacyMode` |
| `PreparedCallArguments` | `LegacyMode` |
| `EvaluatedPartSelectors` | `LegacyMode` |

The two PreparedInterpreted enums and all new nominal carriers remain
unregistered. No production frame or finalizer was converted.

### 3. Compile controls

The 22 required S1 pairs were read and selected, but **0/22 were implemented
or executed** before the ownership STOP. There is no paired-control evidence
and no negative diagnostic is claimed as a successful control. Runtime unit
tests for the new API are also still outstanding.

### 4. Census and forced red

Not implemented or run. The 623-implementation, 600-macro and 23-legacy-type
baseline has not been freshly measured by the required source gate. The
scratch-copy forced red remains outstanding.

### 5. Verification and inventory transitions

Two serial invocations ran, with exit code 101 each:

```text
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-607/tmp cargo test -p rum-stdlib --lib ownership_graph
```

First invocation, verbatim (`ownership-initial.log`):

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
error[E0277]: the trait bound `eval_call_prepare::PreparedCallArguments: suspension::core::ContinuationValue` is not satisfied
   --> crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:179:10
    |
179 |         <Self as ContinuationValue>::into_continuation_expr(self)
    |          ^^^^ the trait `suspension::core::ContinuationValue` is not implemented for `eval_call_prepare::PreparedCallArguments`
    |
    = help: the following other types implement trait `suspension::core::ContinuationValue`:
              Option<Vec<rum_core::Expr>>
              Option<Vec<runtime::model::definitions::RewriteRuleInfo>>
              Option<rum_core::Expr>
              Option<runtime::model::traversal::CasesLevelSpec>
              TraversalWithoutLimitNormalizationError
              Vec<rum_core::Expr>
              Vec<std::collections::HashMap<rum_core::Symbol, rum_core::Expr, BuildHasherDefault<cache::RuntimeCacheHasher>>>
              Vec<structure_data::replace::ReplacePartTarget>
            and 17 others

error[E0277]: the trait bound `eval_call_prepare::PreparedCallArguments: suspension::core::ContinuationValue` is not satisfied
   --> crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:182:10
    |
182 |         <Self as ContinuationValue>::from_continuation_expr(expr)
    |          ^^^^ the trait `suspension::core::ContinuationValue` is not implemented for `eval_call_prepare::PreparedCallArguments`
    |
    = help: the following other types implement trait `suspension::core::ContinuationValue`:
              Option<Vec<rum_core::Expr>>
              Option<Vec<runtime::model::definitions::RewriteRuleInfo>>
              Option<rum_core::Expr>
              Option<runtime::model::traversal::CasesLevelSpec>
              TraversalWithoutLimitNormalizationError
              Vec<rum_core::Expr>
              Vec<std::collections::HashMap<rum_core::Symbol, rum_core::Expr, BuildHasherDefault<cache::RuntimeCacheHasher>>>
              Vec<structure_data::replace::ReplacePartTarget>
            and 17 others

For more information about this error, try `rustc --explain E0277`.
error: could not compile `rum-runtime` (lib) due to 2 previous errors
```

Second invocation, verbatim (`ownership-compiled.log`):

```text
   Compiling rum-runtime v0.2.0 (/home/remi/dev/rum/crates/rum-runtime)
   Compiling rum-stdlib v0.2.0 (/home/remi/dev/rum/crates/rum-stdlib)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 45.22s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-3a556b9db7fd59ac)

running 20 tests
test tests::ownership_graph::accumulator_coverage::every_self_reaching_arc_chain_participates ... ok
test tests::ownership_graph::accumulator_coverage::fresh_names_and_additional_self_edges_are_checked ... ok
test tests::ownership_graph::discovery_report_without_inventory_input ... ok
test tests::ownership_graph::fixtures::coverage_is_not_inferred_from_test_like_filenames ... ok
test tests::ownership_graph::fixtures::fresh_names_cross_module_pair_and_recursion_after_comma ... ok
test tests::ownership_graph::fixtures::inline_module_path_attribute_and_super_imports_resolve ... ok
test tests::ownership_graph::fixtures::manually_checked_definition_counts ... ok
test tests::ownership_graph::fixtures::named_generic_public_inputs_are_opaque_but_borrowed_inputs_are_not ... ok
test tests::ownership_graph::fixtures::opaque_constructs_are_located_and_inventory_removal_fails ... ok
test tests::ownership_graph::fixtures::owning_ports_and_nonowning_storage_do_not_mix_instantiations ... ok
test tests::ownership_graph::fixtures::rediscovery_and_each_missing_mutual_member ... ok
test tests::ownership_graph::fixtures::resolution_uses_declarations_not_container_spellings ... ok
test tests::ownership_graph::fixtures::unknown_configuration_cannot_invent_an_unconditional_cycle ... ok
test tests::ownership_graph::fixtures::unmodeled_lexical_modules_and_async_captures_are_explicit_unknowns ... ok
test tests::ownership_graph::host_payload_ledger::ownership::attribution_resolves_alias_scopes_and_excludes_nonowning_paths ... ok
test tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed ... FAILED
test tests::ownership_graph::host_payload_ledger::tests::metadata_duplicates_and_witnesses_are_enforced ... ok
test tests::ownership_graph::host_payload_ledger::tests::missing_stale_and_unrelated_expression_debt_are_rejected ... FAILED
test tests::ownership_graph::production_inventory_is_exact ... FAILED
test tests::ownership_graph::source::root_set::syntax_root_discovers_added_recursive_type_and_restores_source ... ok

failures:

---- tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed stdout ----

thread 'tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed' panicked at crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger.rs:145:57:
check 1: missing ledger entry: UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	17	15	rum_runtime::runtime::suspension::typed::action::Action.Evaluate.next/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame
check 1: missing ledger entry: UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	19	14	rum_runtime::runtime::suspension::typed::action::Action.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn ErasedFinalizer
check 1: missing ledger entry: UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	29	29	public input rum_runtime::runtime::suspension::typed::action::impl(TypedAction < O >)@28::complete(result)/std::result::Result[Own]/arg0: opaque owned generic parameter O : SlotValue
check 1: missing ledger entry: UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	36	80	public input rum_runtime::runtime::suspension::typed::action::impl(TypedAction < O >)@28::evaluate(next): opaque owned generic parameter F : TypedFrame < Input = Expr , Output = O >
check 1: missing ledger entry: UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	46	62	public input rum_runtime::runtime::suspension::typed::action::impl(TypedAction < O >)@28::finalize(finalizer): opaque owned generic parameter F : SlotFinalizer < Output = O >
check 1: missing ledger entry: UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	54	15	public input rum_runtime::runtime::suspension::typed::action::impl(TypedAction < O >)@28::finish_work(work)/std::result::Result[Own]/arg0: opaque owned generic parameter T
check 1: missing ledger entry: UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/action.rs	55	17	public input rum_runtime::runtime::suspension::typed::action::impl(TypedAction < O >)@28::finish_work(finish): opaque owned payload impl FnOnce (T) -> F
check 1: missing ledger entry: UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/chain.rs	22	59	public input rum_runtime::runtime::suspension::typed::chain::impl(FrameChain < I , O >)@21::then(next): opaque owned generic parameter F : TypedFrame < Input = O >
check 1: missing ledger entry: UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/chain.rs	8	13	rum_runtime::runtime::suspension::typed::chain::FrameChain.frames/std::vec::Vec[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame
check 1: missing ledger entry: UnknownHostPayload	crates/rum-runtime/src/runtime/suspension/typed/witness.rs	15	57	public input rum_runtime::runtime::suspension::typed::witness::impl(SlotWitness < I >)@14::chain(first): opaque owned generic parameter F : TypedFrame < Input = I >
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- tests::ownership_graph::host_payload_ledger::tests::missing_stale_and_unrelated_expression_debt_are_rejected stdout ----

thread 'tests::ownership_graph::host_payload_ledger::tests::missing_stale_and_unrelated_expression_debt_are_rejected' panicked at crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs:70:5:
assertion `left == right` failed
  left: "check 1: missing ledger entry: UnknownHostPayload\tcrates/rum-runtime/src/runtime/suspension/typed/action.rs\t17\t15\trum_runtime::runtime::suspension::typed::action::Action.Evaluate.next/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame\ncheck 1: missing ledger entry: UnknownHostPayload\tcrates/rum-runtime/src/runtime/suspension/typed/action.rs\t19\t14\trum_runtime::runtime::suspension::typed::action::Action.Finalize.0/std::boxed::Box[Own]/arg0: opaque owned payload dyn ErasedFinalizer\ncheck 1: missing ledger entry: UnknownHostPayload\tcrates/rum-runtime/src/runtime/suspension/typed/action.rs\t29\t29\tpublic input rum_runtime::runtime::suspension::typed::action::impl(TypedAction < O >)@28::complete(result)/std::result::Result[Own]/arg0: opaque owned generic parameter O : SlotValue\ncheck 1: missing ledger entry: UnknownHostPayload\tcrates/rum-runtime/src/runtime/suspension/typed/action.rs\t36\t80\tpublic input rum_runtime::runtime::suspension::typed::action::impl(TypedAction < O >)@28::evaluate(next): opaque owned generic parameter F : TypedFrame < Input = Expr , Output = O >\ncheck 1: missing ledger entry: UnknownHostPayload\tcrates/rum-runtime/src/runtime/suspension/typed/action.rs\t46\t62\tpublic input rum_runtime::runtime::suspension::typed::action::impl(TypedAction < O >)@28::finalize(finalizer): opaque owned generic parameter F : SlotFinalizer < Output = O >\ncheck 1: missing ledger entry: UnknownHostPayload\tcrates/rum-runtime/src/runtime/suspension/typed/action.rs\t54\t15\tpublic input rum_runtime::runtime::suspension::typed::action::impl(TypedAction < O >)@28::finish_work(work)/std::result::Result[Own]/arg0: opaque owned generic parameter T\ncheck 1: missing ledger entry: UnknownHostPayload\tcrates/rum-runtime/src/runtime/suspension/typed/action.rs\t55\t17\tpublic input rum_runtime::runtime::suspension::typed::action::impl(TypedAction < O >)@28::finish_work(finish): opaque owned payload impl FnOnce (T) -> F\ncheck 1: missing ledger entry: UnknownHostPayload\tcrates/rum-runtime/src/runtime/suspension/typed/chain.rs\t22\t59\tpublic input rum_runtime::runtime::suspension::typed::chain::impl(FrameChain < I , O >)@21::then(next): opaque owned generic parameter F : TypedFrame < Input = O >\ncheck 1: missing ledger entry: UnknownHostPayload\tcrates/rum-runtime/src/runtime/suspension/typed/chain.rs\t8\t13\trum_runtime::runtime::suspension::typed::chain::FrameChain.frames/std::vec::Vec[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0: opaque owned payload dyn ErasedFrame\ncheck 1: missing ledger entry: UnknownHostPayload\tcrates/rum-runtime/src/runtime/suspension/typed/witness.rs\t15\t57\tpublic input rum_runtime::runtime::suspension::typed::witness::impl(SlotWitness < I >)@14::chain(first): opaque owned generic parameter F : TypedFrame < Input = I >\ncheck 1: missing ledger entry: UnknownHostPayload\ttest-side-suspension-copy.rs\t1\t1\trum_runtime::runtime::suspension::core::SuspendedEvaluation.unrelated_payload/fixture::OtherChain<arg0> via payload/std::boxed::Box[Own]/arg0: opaque owned payload dyn fixture :: OtherPayload"
 right: "check 1: missing ledger entry: UnknownHostPayload\ttest-side-suspension-copy.rs\t1\t1\trum_runtime::runtime::suspension::core::SuspendedEvaluation.unrelated_payload/fixture::OtherChain<arg0> via payload/std::boxed::Box[Own]/arg0: opaque owned payload dyn fixture :: OtherPayload"

---- tests::ownership_graph::production_inventory_is_exact stdout ----

thread 'tests::ownership_graph::production_inventory_is_exact' panicked at crates/rum-stdlib/src/tests/ownership_graph/mod.rs:73:37:
self-reaching types inventory mismatch:
missing inventory row: rum_runtime::runtime::eval_call_prepare::UnevaluatedRestoreNode	crates/rum-runtime/src/runtime/eval_call_prepare.rs	43	rum_runtime::runtime::eval_call_prepare::UnevaluatedRestoreNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::eval_call_prepare::UnevaluatedRestoreNode
stale inventory row: rum_runtime::runtime::eval_call_prepare::UnevaluatedRestoreNode	crates/rum-runtime/src/runtime/eval_call_prepare.rs	29	rum_runtime::runtime::eval_call_prepare::UnevaluatedRestoreNode.previous/rum_runtime::runtime::accumulator_chain::AccumulatorChain<arg0> via 0/std::option::Option[Own]/arg0/std::sync::Arc[LastStrongHolder]/arg0 -> rum_runtime::runtime::eval_call_prepare::UnevaluatedRestoreNode


failures:
    tests::ownership_graph::host_payload_ledger::production_host_payload_ledger_is_complete_and_attributed
    tests::ownership_graph::host_payload_ledger::tests::missing_stale_and_unrelated_expression_debt_are_rejected
    tests::ownership_graph::production_inventory_is_exact

test result: FAILED. 17 passed; 3 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 59.40s

error: test failed, to rerun pass `-p rum-stdlib --lib`
```

The second run compiled the runtime and stdlib test binary and executed the
ownership gate: **17 passed, 3 failed**. The failures are the missing host
ledger entries, a ledger falsifier affected by those missing entries, and
the expected source-position drift of UnevaluatedRestoreNode from line 29
to 43. No documented contract or rejection was weakened.

There are ten newly discovered host-payload rows: three storage sites and
seven public inputs. Full rows are saved in `new-host-payloads.tsv`; the
comparison with the approved transitions is `host-payload-review.tsv`.

- `action.rs:36:80` evaluate(next) and `action.rs:46:62` finalize(finalizer)
  correspond to the planned S1/R public-input transitions, rows 56 and 57.
  The old API remains present, so its old ledger rows must also remain.
- Five public inputs have no explicit approved disposition in that table:
  `action.rs:29:29` complete(result), `action.rs:54:15` finish_work(work),
  `action.rs:55:17` finish_work(finish), `chain.rs:22:59` then(next), and
  `witness.rs:15:57` chain(first). These are the concrete STOP trigger.
- The storage rows are `action.rs:17:15` Action.Evaluate.next,
  `action.rs:19:14` Action.Finalize.0, and `chain.rs:8:13` FrameChain.frames.
  The table discusses S2/R erased storage migration, but no S1 disposition
  receipt was assigned here. The director should specify their treatment
  while the old storage remains and no production TypedFrame exists.

Inventory rows edited: **0**. No identity, disposition, exclusion or debt
entry was removed or reclassified. Source-position and new UnknownConstruct
rows have not yet been replayed. Contract fingerprints and both certificate
sets remain stale and were not edited or replayed. The new TypedAdapter trace
derive has **not yet been added** to the closed derive inventory; no old
named-frame count was changed. These are unfinished maintenance obligations.

The remaining mandatory gates did not run after the explicit STOP:

```text
cargo fmt --check — NOT RUN
cargo test -p rum-stdlib --test typed_slot_compile_controls — NOT RUN
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls — NOT RUN
cargo test -p rum-stdlib --test typed_slot_census — NOT RUN
cargo test -p rum-stdlib --no-default-features --test typed_slot_census — NOT RUN
scripts/check-suspension-oracle.sh 1500 — NOT RUN
cargo test -p rum-stdlib --lib local_production_acceptance — NOT RUN
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance — NOT RUN
cargo test --workspace --no-fail-fast — NOT RUN
```

No native ZeroMQ test ran; no executed skip is claimed. Formatting has not
been checked or applied. No complete workspace or no-default-features result
is claimed. The source-scope and unchanged-old-body audits ran as bounded
Python checks and passed.

### 6. Stop-loss, deviations and exact next action

Compiler errors moved from **2 to 0** across the two attempts, followed by
17 passing ownership tests and the new inventory evidence. Consecutive
no-progress attempts: **0**. The stop-loss did not trigger; the explicit
new-UnknownHostPayload rule did. Mandatory commands attempted: **1/10**,
executed twice; mandatory commands green: **0/10**. Controls: **0/22**.
Census and forced red: **0**. Steps 1–4 have a compiling scaffold; none is
claimed fully verified, and Steps 5–7 are unfinished.

The ownership replay was deliberately brought forward after compilation so
that an explicit inventory STOP could be surfaced before more implementation.
That is the only sequencing change. The fallible legacy-provider adaptation
is described above. No git, network, unsafe, dependency or toolchain change
was used, and all production edits are on the exact Scope list.

The director's next action is to decide the dispositions for the five
unlisted public-input rows and the three dormant erased-storage rows,
preserving recorded destruction debt. No such decision was invented here.
After authorization, review the PreparedCallArguments provider adaptation;
finish the paired controls, unit evidence and census; add the exact trace
inventory entry; replay all maintenance inventories and both earned
certificates; format and run every required gate. The unchanged-source
baseline, both real logs and exact row/function audits are under
`.temp/brief-607/redispatch-1/`.

## Director amendment — redispatch 2

The stop above was correct. Decisions, in order:

1. **The seven new public inputs are `PublicInput`**, with the standard
   witness `public by-value input; owning storage sites are separate rows`:
   `TypedAction::complete(result)`, `evaluate(next)`, `finalize(finalizer)`,
   `finish_work(work)`, `finish_work(finish)`, `FrameChain::then(next)` and
   `SlotWitness::chain(first)`. Ledger rows 56-58 (the old
   `FrameAction::evaluate`/`finalize` and `insert_suspension_frame`) stay
   exactly as they are: the old API still exists, and those transitions
   happen in step R, not here.
2. **The three erased-storage rows are `Unbounded`** (`Action.Evaluate.next`,
   `Action.Finalize.0`, `FrameChain.frames`), owed to the expression debt
   (`TODO.md#standing-engineering-debt`), external implementation coverage
   `unresolved` — the same truth as rows 53-55: destroying a stored frame can
   destroy an `Expr`. Their witness cannot be written by hand, and today's
   verifier (`host_payload_ledger/ownership.rs:121`) skips generic
   implementations, while the only implementors of `ErasedFrame` and
   `ErasedFinalizer` are the generic `TypedAdapter<F>` and
   `FinalizerAdapter<F>`. **Extend the verifier** with one rule, computed, not
   written:
   - It applies only when the payload trait is crate-private and every
     implementor is a generic adapter that owns its parameter by value
     (`Arc<F>`, `Box<F>`, or `F` directly) under a single trait bound.
   - Witness = an existing-style existential path from a **concrete, non-
     generic in-tree implementor of the bound** to `Expr`/`SharedExpr`/
     `SharedArgs`, with the adapter hop shown in the text, whenever one
     exists.
   - Only while no such implementor exists (true in S1 and S2; from S3 on the
     first typed frame supplies one, and the replayed witness must change to
     it), and only when the bound trait is **public**, the witness names the
     adapter, its owning field and the public bound, and says that no in-tree
     instantiation exists yet. A crate-private bound with no implementor
     yields no witness.
   - Three falsifiers in `host_payload_ledger/tests.rs`, each watched failing:
     an adapter holding `PhantomData<F>` (not owned) yields no witness; a
     private bound with no implementor yields no witness; a fixture that adds
     a concrete `Expr`-owning implementor of the bound makes the interim text
     a witness mismatch.
3. **The fallible `PreparedCallArguments` provider is approved**
   (`old_decode` returning `Result<Self, Expr>`; rejection stays
   `Err(Slot::Value)`, which S2 turns into a fault). Add a runtime unit test
   for it now: for representative prepared states (with and without
   `Unevaluated` restore, an empty prefix, a spliced `Sequence`),
   `old_decode(old_encode(x)) == x`, and `old_encode` produces exactly the
   `Expr` the existing finalizer path produces for the same state.
4. The `UnevaluatedRestoreNode` line drift is a position refresh: replay it.

Then finish Steps 5-7 and every gate. Append a new `## Execution report`
after this section.


## Execution report — redispatch 2: implementation bounds are missing from discovery

### 1. Files changed and dormant audit

**No production, test source or inventory file changed in this redispatch.**
The previous compiling scaffold and its outstanding maintenance work remain
as recorded in redispatch 1. Only scratch evidence under
`.temp/brief-607/redispatch-2/`, the checkpoint, and this appended report
changed. No production function body or visibility declaration was touched;
no performance measurement was applicable.

The amendment's seven PublicInput and three Unbounded decisions, and its
fallible PreparedCallArguments approval, were read and accepted. The new
verifier rule was checked against its actual available source facts before
implementation. This exposed a necessary change outside the amended scope:

- `crates/rum-stdlib/src/tests/ownership_graph/model.rs:99`: `DropImpl`
  retains scope, self type, trait path and location, but **no implementation
  generics, bounds or where-clause**. It must retain `syn::Generics` so the
  verifier can enforce the new rule's single-trait-bound requirement and
  determine whether a concrete bound implementor satisfies the adapter impl.
- `crates/rum-stdlib/src/tests/ownership_graph/source_items.rs:409`: the
  Collector constructs that record without `item.generics`. It must populate
  the new field from the actual impl AST, including its where-clause.

Neither file is listed in Scope; the registered-runtime-type visibility
exception does not apply to ownership-test metadata. No edit was made there.
The explicit disclosure STOP was followed; no secondary source scanner or
other workaround was introduced to bypass these owner files.

`Definition::generics` is insufficient: it describes the adapter **type
declaration**, whose bound can be weaker than the bound on a particular
trait implementation. In the concrete probe, the declaration is unchanged:

```rust
struct Adapter<F: Bound> { inner: F }
```

The first implementation is `impl<F: Bound> Erased for Adapter<F> {}`; the
second adds only `+ Clone`. `Concrete` implements Bound and owns an Expr,
but does not implement Clone. These are different instantiation obligations.
The current collector discards that difference.

### 2. Public items, mode and registrations

Unchanged from redispatch 1: dormant API, Expr=Value, 21 existing Legacy
registrations, no live-driver integration or production frame conversion.
The director-approved fallible provider remains intact. Its newly required
prepared-state roundtrip/finalizer-equivalence unit test has not yet been
implemented because this scope preflight stopped first.

### 3. Compile controls and the scope evidence

The required 22 S1 control pairs remain **0/22**. The additional scope probe
below is not counted as one of those pairs or as one of the three requested
ledger falsifiers.

The scratch probe imports the real ownership `model`, `source`, `resolve`,
`effects`, `graph` and `evidence` modules read-only. It runs both fixture
versions through `source::fixture` and `graph::discover`, and compares **all
Source and Discovery fields** (including declarations, generics, bindings,
locations, implementations, boundaries, unknowns and ownership edges).
Padding keeps unchanged source coordinates identical. The test confirms that
the two snapshots are identical despite the extra implementation bound.

Separate bounded rustc fixtures compile the positive first, then introduce
only that extra `Clone` constraint. The negative checks one primary error,
E0277, its `Concrete: Clone` message, and its exact source line; the emitted
primary span starts at `negative.rs:11:37`. No production crate was mutated.

Actual results:

```text
     Locking 4 packages to latest Rust 1.85.0 compatible versions
      Adding syn v2.0.119 (available: v3.0.4)
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   Compiling syn v2.0.119
   Compiling brief-607-bounds-probe v0.0.0 (/home/remi/dev/rum/.temp/brief-607/redispatch-2/bounds-probe)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.01s
     Running unittests src/lib.rs (.temp/brief-607/redispatch-2/bounds-probe/target/debug/deps/brief_607_bounds_probe-3544d042759d699e)

running 1 test
test tests::ownership_graph::implementation_bound_metadata_is_lost ... IDENTICAL_DISCOVERY facts=18 despite additional impl Clone bound
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

```

```text
RUST_POSITIVE exit=0: Adapter<Concrete> implements Erased under F: Bound
RUST_NEGATIVE exit=1: E0277 at negative.rs:11:37: the trait bound `Concrete: Clone` is not satisfied
```

Raw rustc JSON diagnostics, exact command arrays and both fixture sources are
saved under `.temp/brief-607/redispatch-2/`. The fixture cargo manifest and
lockfile are scratch-only; no repository manifest or lockfile changed.

### 4. Census and forced red

Still not implemented or run. No new census, 623/600 baseline verification,
23-type live-decoder report or forced-red result is claimed.

### 5. Gates, inventories and certificate transitions

The bounded scratch discovery probe ran with exit code 0:

```text
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-607/tmp CARGO_TARGET_DIR=/home/remi/dev/rum/.temp/brief-607/redispatch-2/bounds-probe/target cargo test --manifest-path .temp/brief-607/redispatch-2/bounds-probe/Cargo.toml implementation_bound_metadata_is_lost -- --nocapture
```

The positive and negative rustc fixtures were serial, each inside the same
required environment and `python3 scripts/run-bounded.py --timeout 900 --`,
using `rustc --edition=2024 --crate-name bounds_control --emit=metadata
--error-format=json --out-dir <scratch> <positive.rs|negative.rs>`. Their exact
expanded arrays are `bounds-positive.command.json` and
`bounds-negative.command.json`. Positive exit: 0; expected negative exit: 1.

No mandatory implementation gate was rerun in this redispatch:

```text
cargo fmt --check — NOT RUN
cargo test -p rum-stdlib --test typed_slot_compile_controls — NOT RUN
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls — NOT RUN
cargo test -p rum-stdlib --test typed_slot_census — NOT RUN
cargo test -p rum-stdlib --no-default-features --test typed_slot_census — NOT RUN
scripts/check-suspension-oracle.sh 1500 — NOT RUN
cargo test -p rum-stdlib --lib ownership_graph — NOT RUN
cargo test -p rum-stdlib --lib local_production_acceptance — NOT RUN
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance — NOT RUN
cargo test --workspace --no-fail-fast — NOT RUN
```

The 17-pass/3-fail ownership result belongs to redispatch 1, not a new run.
The only executed test total this redispatch is the scratch probe's
`1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out`. No native ZeroMQ
test ran, so no executed skip is claimed. Inventory/certificate transitions
this redispatch: **none**. No discovery rejection, oracle Checks flag or debt
entry was weakened.

Bounded source verification:

```text
SOURCE_UNCHANGED files=1436 changed=0 added=0 removed=0
```

### 6. Stop-loss, deviations and next action

Implementation attempts this redispatch: 0. One scratch characterization
test passed; one additional positive/negative compiler pair produced the
expected result. Neither is counted toward the required S1 controls.
Consecutive no-progress implementation-gate attempts: 0. The explicit Scope
STOP triggered; the two-attempt stop-loss did not.

The necessary next action is to add the two exact discovery-owner files above
to Scope so implementation generics can be retained. Then enforce the single
bound using the actual impl constraints, implement and watch all three ledger
falsifiers reject, replay the computed witnesses and approved rows, add the
PreparedCallArguments unit cases, and finish the outstanding S1 controls,
census, inventories, certificates and mandatory gates.

No git command, network access, unsafe code, production rollback, manifest
change or out-of-scope source edit was performed. S1 remains incomplete.

## Director amendment — redispatch 3

The stop above was correct, and the gap was the director's: the verifier rule
needs the implementation's own generics and where-clause, which discovery
discards (`ownership_graph/model.rs:99`, `source_items.rs:409`), and your
fixture shows why the type declaration's generics are not a substitute. Scope
now covers the ownership-graph test module as a class (see Scope). Implement
the rule on the implementation's bounds, add its three falsifiers, then finish
Steps 5-7 and every gate. Append a new `## Execution report` after this
section.

