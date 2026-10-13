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
- `crates/rum-runtime/src/runtime/suspension/typed/tests.rs` and any new
  file under `crates/rum-runtime/src/runtime/suspension/typed/tests/`
  (redispatch 5: the typed unit tests live here; the earlier
  `runtime/tests/typed_slots*` paths must not remain)
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
- `crates/rum-stdlib/tests/typed_slot_census/scan.rs` (new), and any new file
  under `crates/rum-stdlib/tests/typed_slot_census/` (added at redispatch 5)
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
   a spelling count. It must report the baseline, all untyped, as two
   cohorts asserted separately (corrected at redispatch 4): production, 553
   frames + 54 finalizers = 607 implementations and 345 + 244 = 589 macro
   sites; and the test-only interpreted matcher behind
   `runtime/mod.rs:48-49`, 11 + 5 = 16 implementations and 11 macro sites
   (3 + 8). Their sum, 623/600, is the partition's population. It also
   reports the 23 legacy types with their live decoder uses. Show it can
   fail: run it against a scratch copy with one production frame
   implementation removed, and against one with the `#[cfg(test)]` on
   `mod.rs:48` removed, and paste both red outputs.
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



## Execution report — redispatch 3: production/test baseline conflict

S1 is **incomplete**. The amended ownership verifier is implemented and its
falsifiers passed. The dormant runtime tests and all 22 compiler pairs passed
in both requested compiler-control builds. Source discovery then exposed a
baseline population conflict. No row was reclassified to force acceptance.

The brief requires 623 implementations and **600 production macro sites**.
The actual module tree has **607 production implementations and 589
production macro sites**. The other 16 implementations and 11 macro sites
are precisely S33's interpreted-matcher family, guarded by `#[cfg(test)]`
at `crates/rum-runtime/src/runtime/mod.rs:48-49`. That owner file is
byte-identical to the pre-edit redispatch-1 baseline. The previous design's
623/600 totals combine the production and test-only cohorts.

The new census retains its required-count assertion and fails. Calling the
S33 cohort production, dropping it from the planned partition, or changing
its population/receipt status requires a director decision under Step 7's
rule against unapproved identity/status/disposition changes. No cfg was
removed, no implementation was converted, and no test was weakened.

### 1. Files changed and dormant audit

All paths below are relative to the repository root. This includes inherited
S1 scaffold edits and this redispatch's work. The scope/hash audit records
**47 changed or new files under crates, 0 removals, 0 out-of-scope paths**.
This report and scratch evidence are the other writes. The seven Rust files
under ownership_graph are listed explicitly; each change serves the amended
adapter rule or its falsifiers.

| File | Semantic change |
| --- | --- |
| `crates/rum-runtime/src/runtime/debug_tools/mod.rs` | Runtime-visible test_report module path only (line 14). |
| `crates/rum-runtime/src/runtime/debug_tools/test_report.rs` | Runtime-visible EvalCapturedOutcome re-export only (line 10). |
| `crates/rum-runtime/src/runtime/debug_tools/test_report/model.rs` | Runtime-visible EvalCapturedOutcome declaration only (line 29); fields unchanged. |
| `crates/rum-runtime/src/runtime/definitions/application/mod.rs` | Runtime-visible ReplaceAllTraversalOutcome re-export (line 8); formatter reflow. |
| `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs` | Runtime-visible ReplaceAllTraversalOutcome declaration only (line 13). |
| `crates/rum-runtime/src/runtime/eval_call_prepare.rs` | Dormant PreparedCallArguments wire delegation; cfg(test) old-finalizer fixture helper. |
| `crates/rum-runtime/src/runtime/kernel_api.rs` | Re-export the nine S1 public items. |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data.rs` | Runtime-visible DeleteCasesOutcome declaration only (line 23). |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/mod.rs` | Runtime-visible replace module only (line 5). |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace.rs` | Runtime-visible ReplacePartTarget declaration only (line 51); fields unchanged. |
| `crates/rum-runtime/src/runtime/suspension.rs` | Private typed module and nine-item public re-export. |
| `crates/rum-runtime/src/runtime/suspension/typed.rs` | Public sealed slot/frame/finalizer traits and cfg(test) unit-module wiring. |
| `crates/rum-runtime/src/runtime/suspension/typed/action.rs` | Opaque actions, checked constructors, finish_work and ResumeStop. |
| `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs` | Private generic erasure; decode rejection before resume; trace/abandon/boundary delegation; token mint. |
| `crates/rum-runtime/src/runtime/suspension/typed/chain.rs` | Private nonempty chain start and typed adjacency. |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | Private key/channel and Value/Legacy/State strategies; associated mode only. |
| `crates/rum-runtime/src/runtime/suspension/typed/legacy.rs` | 21 pure delegations to existing wire codecs, including approved fallible prepared decoder. |
| `crates/rum-runtime/src/runtime/suspension/typed/registry.rs` | Expr=Value; 21 existing types=Legacy; dormant exact state variants/providers. |
| `crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs` | Invariant lifetime/type token with no Arc, constructor, Clone or Copy. |
| `crates/rum-runtime/src/runtime/suspension/typed/witness.rs` | Invariant ZST witness from actual Result and checked chain start. |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | Add only suspension/typed/adapter.rs:TypedAdapter to the closed derive array. |
| `crates/rum-runtime/src/runtime/tests/typed_slots.rs` | Test-only frame/finalizer support and unit modules. |
| `crates/rum-runtime/src/runtime/tests/typed_slots/accounting.rs` | Root/capture-boundary delegation without executing a frame. |
| `crates/rum-runtime/src/runtime/tests/typed_slots/composition.rs` | Dormant composition, typed finalization and stopped deferred work. |
| `crates/rum-runtime/src/runtime/tests/typed_slots/faults.rs` | Wrong channel/variant rejection and error forwarding before user resume. |
| `crates/rum-runtime/src/runtime/tests/typed_slots/registry.rs` | Mode equality for all 22 registrations; actual prepared-state roundtrips/finalizer equality; leniency. |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger.rs` | Wire the new adapter witness module. |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/adapters.rs` | Computed private-trait/single-bound/owned-parameter rule; concrete paths supersede interim witnesses. |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/ownership.rs` | Try the new rule after existing concrete attribution; share the existing expression-path proof. |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs` | Three required falsifiers, additional-impl-bound rejection, and computed production witnesses. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv` | Append seven approved PublicInput and three computed Unbounded entries; preserve all 65 old rows byte-for-byte. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv` | Replay discovered positions and new scaffold constructs; exact changes in ownership-discovery-diff.tsv. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv` | Replay discovered positions and new scaffold constructs; exact changes in ownership-discovery-diff.tsv. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | Replay discovered positions and new scaffold constructs; exact changes in ownership-discovery-diff.tsv. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | Replay discovered positions and new scaffold constructs; exact changes in ownership-discovery-diff.tsv. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | Replay discovered positions and new scaffold constructs; exact changes in ownership-discovery-diff.tsv. |
| `crates/rum-stdlib/src/tests/ownership_graph/model.rs` | Retain impl generics, polarity and conditionality for the narrow adapter proof. |
| `crates/rum-stdlib/src/tests/ownership_graph/source.rs` | Fixture loader accepting explicit crate roots, used by the concrete-Expr falsifier. |
| `crates/rum-stdlib/src/tests/ownership_graph/source_items.rs` | Collect those implementation facts from the actual impl AST. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs` | Declare the static typed_slots stage-selection module. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs` | S1/no-flip selection in source; no environment-selected gate. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-shapes.tsv` | 67 planned slot rows copied from the immutable mode plan; no new carrier registered. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` | Unfinished S1 stage header only; no F/M/D transition receipt earned or invented. |
| `crates/rum-stdlib/tests/typed_slot_census.rs` | Unfinished source gate, exact-receipt checks, output artifacts and unexecuted scratch-red driver. |
| `crates/rum-stdlib/tests/typed_slot_census/classify.rs` | Canonical trait/type resolution and separate definitions/calls/provider-closure records; incomplete gate integration. |
| `crates/rum-stdlib/tests/typed_slot_census/scan.rs` | AST module/function/site census, macro-token parsing, separate test population, explicit unresolved sites. |
| `crates/rum-stdlib/tests/typed_slot_compile_controls.rs` | All 22 paired controls; scratch-only runtime mutations; raw Cargo JSON/code/primary-span checks. |

The eight registered-type visibility changes are the exact lines recorded
above. No field visibility was widened. The other production changes are
imports/exports, new dormant API bodies, and test-module wiring. The new
prepared finalizer comparison helper is `#[cfg(test)]`.

A fresh read-only syn audit compared every old function body in every changed
production file against the original snapshot. Actual output:

```text
DORMANT AUDIT: 58 old production function bodies byte-identical; 109 new dormant bodies. cfg(test) bodies excluded. TypedAdapter's generated trace implementation is additionally dormant.
```

The full list, with current source positions, is
[function-body-audit.tsv](../../../.temp/brief-607/redispatch-3/function-body-audit.tsv).
Every new body is off the live evaluation path: no queue installs a typed
adapter, no production TypedFrame/SlotFinalizer implementation was added,
and no existing evaluator calls the new prepared encoder. The adapter's
new derived trace implementation is likewise dormant. No old evaluator,
matcher, traversal, block or selector function body changed, so no performance
measurement was run under S1's stated exemption.

For completeness, these are the 25 bodies outside the repeated providers:

| File | Body line | Function |
| --- | --- | --- |
| `crates/rum-runtime/src/runtime/eval_call_prepare.rs` | 17 | `::<PreparedCallArguments as >::into_continuation_expr` |
| `crates/rum-runtime/src/runtime/suspension/typed.rs` | 45 | `::TypedFrame::output_boundary` |
| `crates/rum-runtime/src/runtime/suspension/typed/action.rs` | 70 | `::<ResumeStop < O > as >::failure` |
| `crates/rum-runtime/src/runtime/suspension/typed/action.rs` | 29 | `::<TypedAction < O > as >::complete` |
| `crates/rum-runtime/src/runtime/suspension/typed/action.rs` | 36 | `::<TypedAction < O > as >::evaluate` |
| `crates/rum-runtime/src/runtime/suspension/typed/action.rs` | 46 | `::<TypedAction < O > as >::finalize` |
| `crates/rum-runtime/src/runtime/suspension/typed/action.rs` | 56 | `::<TypedAction < O > as >::finish_work` |
| `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs` | 70 | `::<FinalizerAdapter < F > as ErasedFinalizer>::finish` |
| `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs` | 56 | `::<TypedAdapter < F > as ErasedFrame>::output_boundary` |
| `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs` | 38 | `::<TypedAdapter < F > as ErasedFrame>::resume` |
| `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs` | 28 | `::<TypedAdapter < F > as FrameAbandon>::abandon` |
| `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs` | 61 | `::erase` |
| `crates/rum-runtime/src/runtime/suspension/typed/chain.rs` | 20 | `::<FrameChain < I , O > as >::then` |
| `crates/rum-runtime/src/runtime/suspension/typed/chain.rs` | 12 | `::start` |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | 9 | `::<CodecKey as >::new` |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | 91 | `::<LegacyMode as Transport < T >>::decode` |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | 88 | `::<LegacyMode as Transport < T >>::encode` |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | 103 | `::<StateMode as Transport < T >>::decode` |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | 100 | `::<StateMode as Transport < T >>::encode` |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | 79 | `::<ValueMode as Transport < Expr >>::decode` |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | 76 | `::<ValueMode as Transport < Expr >>::encode` |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | 52 | `::decode` |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | 48 | `::encode` |
| `crates/rum-runtime/src/runtime/suspension/typed/witness.rs` | 15 | `::<SlotWitness < I > as >::chain` |
| `crates/rum-runtime/src/runtime/suspension/typed/witness.rs` | 10 | `::slot_witness` |

The other 84 bodies are exactly `LegacyCodec::{old_encode,old_decode}` in
`typed/legacy.rs` and `StateCodec::{pack,unpack}` in `typed/registry.rs` for
**each of the 21 types listed in part 2**. They only delegate old wire codecs
or construct/match the corresponding dormant state variant. This enumerates
all 109 new handwritten production bodies without omitting trait methods.

Size notes: registry.rs is 11,307 characters (one closed registration table);
the existing eval_call_prepare.rs is 22,759, extended by the dormant owner
helper and test helper only. The new census files are unfinished at 14,442,
10,839 and 12,911 characters; compile controls are 15,613. No unrelated split
was made. Formatting was applied to the first 27 changed Rust files before
the ownership replay; later runtime tests/census/control additions still need
the final formatting pass and mandatory `cargo fmt --check`.

### 2. Public API, transport and registrations

The public S1 list is exactly `SlotValue`, `TypedFrame`, `SlotFinalizer`,
`TypedAction`, `SlotWitness`, `slot_witness`, `FrameChain`, `ContinueToken`
and `ResumeStop`. Constructors enforce input/output equalities; witness and
chain endpoints are invariant. The token is invariant, non-Clone/non-Copy,
contains no Arc, and is minted only by the private adapter. S1 exposes no
suspend method or typed suspension macro and installs nothing in the driver.

`Codec` declares only associated `Mode`; keyed free channel functions select
Value, Legacy or State. Expr is Value only. Legacy receives Value/Expr only;
State receives State and the exact variant only. Providers do not receive
Slot. The approved prepared decoder rejection retains `Err(Slot::Value)`.

All 21 S1 registrations are Legacy:

```text
Option<Expr>
Vec<Expr>
Vec<SymbolExprMap>
Option<Vec<Expr>>
Option<Vec<RewriteRuleInfo>>
Option<CasesLevelSpec>
bool
String
CompiledMatchOutcome
CasesTraversalResolution
TraversalWithoutLimitResolution
CasesCollectionOutcome
PositionCollectionOutcome
DeleteCasesOutcome
TraversalCountOutcome
ReplaceOutcome
ReplaceAllTraversalOutcome
Vec<ReplacePartTarget>
EvalCapturedOutcome
PreparedCallArguments
EvaluatedPartSelectors
```

No PreparedInterpreted type or new carrier was registered. The runtime mode
unit compiles all 22 equality witnesses (Expr plus 21 Legacy types). Its
prepared-state test obtains states through actual argument preparation:
empty call, ordinary arguments, Unevaluated restore, spliced Sequence, and
empty/spliced Sequences combined with a restore. It compares all decoded
fields to the original state and exact old_encode Expr equality with the
existing suspended-finalizer encoder. Wrong prepared tags remain rejected.
All eight dormant unit tests passed; the workspace gate has not run.

### 3. Compiler controls

Every S1 ID from the immutable table is present exactly once: **22/22**.
The positive fixture for every pair compiled before the negative mutation.
The external positives share a compiled crate; private positives share a
scratch runtime test crate. Each negative changes one intended property,
checks the primary code/category and exact file/line/column, and preserves raw
Cargo JSON. No failed missing-support-item fixture is counted as a control.

Both feature runs produced the same checked codes/spans:

| Pair | Positive default / no-default | Rejection | Primary span |
| --- | --- | --- | --- |
| first-chain-input | PASS / PASS | E0271 | `src/lib.rs:26:35` |
| chain-adjacency | PASS / PASS | E0271 | `src/lib.rs:33:70` |
| unbound-chain-constructor | PASS / PASS | E0599 | `src/lib.rs:39:41` |
| evaluate-input | PASS / PASS | E0271 | `src/lib.rs:45:66` |
| evaluate-output | PASS / PASS | E0271 | `src/lib.rs:51:66` |
| finalizer-output | PASS / PASS | E0271 | `src/lib.rs:57:48` |
| opaque-action | PASS / PASS | E0599 | `src/lib.rs:62:39` |
| token-no-clone | PASS / PASS | E0599 | `src/lib.rs:68:18` |
| token-no-mint | PASS / PASS | E0599 | `src/lib.rs:74:27` |
| token-no-escape | PASS / PASS | lifetime category | `src/lib.rs:80:1` |
| sealed-extension | PASS / PASS | E0277 | `src/lib.rs:86:20` |
| codec-key-private | PASS / PASS | E0603 | `src/lib.rs:92:18` |
| opaque-selector-pass | PASS / PASS | E0271 | `src/lib.rs:99:35` |
| mode-not-frame-chosen | PASS / PASS | E0599 | `src/lib.rs:105:65` |
| mode-private-selection | PASS / PASS | E0603 | `src/lib.rs:111:18` |
| sibling-erased-private | PASS / PASS | E0603 | `crates/rum-runtime/src/runtime/mod.rs:131:47` |
| mode-expr-fixed | PASS / PASS | E0277 | `crates/rum-runtime/src/runtime/suspension/typed/registry.rs:21:17` |
| mode-new-state-only | PASS / PASS | E0277 | `crates/rum-runtime/src/runtime/suspension/typed.rs:61:40` |
| mode-one-associated-type | PASS / PASS | E0201 | `crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5` |
| mode-no-codec-override | PASS / PASS | E0407 | `crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5` |
| mode-legacy-provider-narrow | PASS / PASS | E0053 | `crates/rum-runtime/src/runtime/suspension/typed.rs:69:19` |
| mode-state-provider-narrow | PASS / PASS | E0053 | `crates/rum-runtime/src/runtime/suspension/typed.rs:74:15` |

Token escape pins the `lifetime may not live long enough` category (no error
code). The Expr-mode mutation also trips the actual runtime unit's ValueMode
equality witness (E0271); that related diagnostic is retained and checked at
its real test source. The required primary remains E0277 at the mutated mode.

First default attempt caught an unsupported fixture import (E0412 for
StateMode), not an API failure. The import was supplied in both positive and
negative scratch sources. The next attempt found the related E0271 from the
real unit witness; it was explicitly classified rather than removing that
witness. The third default run and first no-default run passed all pairs.

Sources, command arrays, raw diagnostics and full primary-span records:
[default results](../../../.temp/brief-607/compile-controls/default/results.json),
[no-default results](../../../.temp/brief-607/compile-controls/no-default/results.json).
The records refer to each group's successful positive Cargo JSON and every
individual negative Cargo JSON. Runtime-private mutations stayed in scratch.

### 4. Census output, conflict and forced red

The census now walks AST module declarations, records qualified identities,
associated types, macro tokens, definitions separately from calls, raw insert
sites, action constructions, legacy heads and function references. It uses
the existing lexical resolver for nominal/trait identity and keeps unknown
receiver/alias/pointer targets explicit. Current output is under
[the default census directory](../../../.temp/brief-607/census/default/):
frames, definitions, macro-sites, live-calls, unresolved, wire-provider-closure,
typed-registry, heads, legacy-test-users and transition-receipt TSVs. Source
rows include SHA256 and spans. This is unfinished source-gate work, **not a
completed live-use proof or preflip acceptance**.

The actual failed source-baseline gate says:

```text
test source_baseline_matches_earned_receipt ... S1 CENSUS frames=553 finalizers=54 implementations=607 typed=0 macros=589 result=345 insert=244
S1 baseline requires 564 frames + 59 finalizers = 623 untyped implementations and 600 macros
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 12.61s
```

The read-only reconciliation against the immutable baseline tables found:

```text
PRODUCTION: 553 SuspensionFrame + 54 PureFrameFinalizer = 607; typed=0.
PRODUCTION MACROS: 345 result + 244 insert = 589.
S33 CFG(TEST) BASELINE: 11 SuspensionFrame + 5 PureFrameFinalizer = 16; macros=11 (3 result + 8 insert).
TOTAL PLANNED BASELINE: 607 + 16 = 623; macros 589 + 11 = 600.
SOURCE GUARD: crates/rum-runtime/src/runtime/mod.rs:48 #[cfg(test)], :49 mod pattern_matching_interpreted;
Guard owner is byte-identical to the redispatch-1 baseline.
No unrelated baseline implementation is missing; every missing production row belongs to S33's cfg(test) module.
No frame or macro row was reclassified to make the gate pass.

```

The exact affected IDs/coordinates are
[census-population-conflict.tsv](../../../.temp/brief-607/redispatch-3/census-population-conflict.tsv):
F0515–F0525, F0614–F0618, and the 11 S33 macro rows. Normalizing generic
parameter spellings during this comparison accounts for the generic frame
names; there are no other missing planned frame implementations.

The 67-row shape plan has been copied, but the stage TSV intentionally has
only the unfinished S1 stage header: no F/M/D transition receipt has been
earned or invented. Thus the required 23-type live-decoder report and all
exact baseline receipts remain unfinished. The code includes a scratch-copy
forced-red driver and a deliberately ignored child probe that it would invoke
explicitly; **neither ran**, because the original baseline is already red.
No 622-implementation forced-red output is claimed. The static preflip
selection remains S1/no flip; S54's receipt is mandatory before a flip.

The director needs to decide whether to authorize **623/600 as a combined
baseline with an explicit 16/11 cfg(test) S33 cohort**, while recording
607/589 as production, or correct the required production counts and state
how the S33 cohort participates in receipts. Its rows must remain accounted
for. The current assertion and source guard were left intact.

### 5. Executed gates, inventory rows and certificates

All Cargo/compiler runs were serial through the required bounded wrapper:

```text
python3 scripts/run-bounded.py --timeout 900 -- env CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 CARGO_NET_OFFLINE=true TMPDIR=/home/remi/dev/rum/.temp/brief-607/tmp <command>
```

The compiler harness additionally bounds its complete child process group
and each nested Cargo command. Read-only Python audits used the same wrapper.
The body-audit Cargo package and its lockfile are scratch-only; repository
manifests, lockfile and toolchain were not edited.

Executed command families were the adapter falsifier filter, ownership_graph
(with --nocapture discovery followed by exact replay), rum-runtime's
`--lib typed_slots`, both mandatory compiler-control commands, and a focused
`typed_slot_census source_baseline_matches_earned_receipt -- --nocapture`.
Every complete log is in
[redispatch-3](../../../.temp/brief-607/redispatch-3/).
Verbatim result lines from every attempt:

```text
adapter-falsifiers.log
test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 1140 filtered out; finished in 5.80s

adapter-falsifiers-2.log
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1140 filtered out; finished in 5.82s

ownership-discovery.log
test result: FAILED. 22 passed; 3 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 65.65s

ownership-replayed.log
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 67.79s

typed-slots.log
error[E0583]: file not found for module `accounting`
error[E0583]: file not found for module `composition`
error[E0583]: file not found for module `faults`
error[E0583]: file not found for module `registry`

typed-slots-2.log
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1160 filtered out; finished in 0.00s

typed-slots-3.log
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.01s

compile-controls-default-1.log
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 68.71s

compile-controls-default-2.log
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.78s

compile-controls-default-3.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.32s

compile-controls-no-default.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.75s

census-initial.log
error[E0432]: unresolved imports `super::model`, `super::resolve`

census-discovery.log
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 11.36s

census-discovery-2.log
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 12.61s
```

The zero-test runtime run above was a filter mistake, not unit evidence; the
module name was corrected and the following run executed all eight tests.
The first census attempt had E0432 imports; the second reached a missing
lexical-scope lookup; the third collected the actual population and exposed
the contract conflict. None was reported as a pass.

The mandatory verification state at this stop is:

```text
cargo fmt --check — NOT RUN
cargo test -p rum-stdlib --test typed_slot_compile_controls — PASS (22 pairs)
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls — PASS (22 pairs)
cargo test -p rum-stdlib --test typed_slot_census — full target NOT RUN; focused baseline FAILED as above
cargo test -p rum-stdlib --no-default-features --test typed_slot_census — NOT RUN
scripts/check-suspension-oracle.sh 1500 — NOT RUN
cargo test -p rum-stdlib --lib ownership_graph — 25 passed at replay checkpoint; final rerun still owed after later test/census wiring
cargo test -p rum-stdlib --lib local_production_acceptance — NOT RUN
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance — NOT RUN
cargo test --workspace --no-fail-fast — NOT RUN
```

No native ZeroMQ test ran; no executed native skip is claimed. The whole
oracle, final derive/named-frame gate, both certificate gates and workspace
remain unverified. **No complete current-tree green claim is made.**

Ownership replay at its recorded checkpoint passed all 25 tests. The three
required falsifiers printed the actual rejected proof results: PhantomData
has no verified owned path; each direct/Box/Arc adapter with a private bound
has no interim witness; adding the concrete Expr owner invalidates the
recorded interim witness and computes the adapter/concrete/Expr path.
The extra `+ Clone` impl-bound falsifier also rejects. The verifier checks
all erased-trait implementations before selecting an adapter witness.

Every changed ownership discovery row is disclosed in
[ownership-discovery-diff.tsv](../../../.temp/brief-607/redispatch-3/ownership-discovery-diff.tsv).
The replay changed these exact populations:

| Partition | Before | Replayed | Removed rows | Added rows |
| --- | ---: | ---: | ---: | ---: |
| host_payloads | 65 | 75 | 0 | 10 |
| unknown_attributes | 1499 | 1500 | 4 | 5 |
| unknown_captures | 2946 | 2949 | 11 | 14 |
| unknown_macros | 3144 | 3145 | 14 | 15 |
| unknown_storage | 91 | 91 | 0 | 0 |
| self_reaching | 59 | 59 | 1 | 1 |

Removed/added pairs are source-position refreshes; new constructs belong to
the scaffold and its prepared encoder. No unrelated normalized identity
vanished. UnevaluatedRestoreNode moved 29→43. The host ledger appended seven
PublicInput and three Unbounded entries from the director's decisions; all
65 prior entries, including the old rows 56–58, remain byte-identical. The
three Unbounded witnesses were copied from actual verifier output, retain
TODO.md#standing-engineering-debt and unresolved external coverage, and do
not claim bounded destruction. Full new ledger rows:
[ledger-added.tsv](../../../.temp/brief-607/redispatch-3/ledger-added.tsv).

The derive inventory adds only TypedAdapter; old named-frame/finalizer counts
were not changed. Later test-module wiring still needs a fresh ownership
replay, so the table above is a checkpoint result, not a final assertion that
all current discovery rows are accounted for. Contract fingerprints and
promoted certificate cells were not refreshed: certificate transitions in
this redispatch are **none**, and both stale sets remain owed.

### 6. Stop-loss, deviations and exact next action

This is a population/status decision stop, not the two-attempt stop-loss.
Each implementation iteration reduced compile errors, corrected fixture
plumbing, reached new checked control pairs, or exposed the next real gate.
Consecutive no-progress attempts: **0**. After the population conflict was
identified, no implementation or receipt classification was changed; only
read-only evidence/audits, the checkpoint, and this report were added.

The sequencing brought ownership replay and dormant units ahead of the
compiler/census work so the amended proof rule could be exercised first.
No git command, network access, sub-agent, new unsafe, production frame
conversion, live-driver change, or out-of-scope source edit occurred. The
original baseline and all prior reports remain intact; the tree is uncommitted.

Next: the director resolves the production versus cfg(test) baseline receipt.
Then finish the exact F/M/D/I/C/T receipts and 23-type decoder-use census,
exercise the scratch forced red, review the census's unresolved/module cases,
format the unfinished new tests, replay current ownership/derive inventories
and both earned certificate sets, and run every mandatory gate to completion.
The correct checkpoint is `.temp/brief-607/PROGRESS.md`; the original baseline
remains `.temp/brief-607/redispatch-1/baseline/`.

## Director amendment — redispatch 4

The stop was correct and the error was the director's. "600 production macro
sites" came from a director scan that counted `pattern_matching_interpreted`
as production. It is not: the module has been `#[cfg(test)]` since
`9d1954ed` (2026-03-18). It is rum's own reference matcher for
`compiled_matcher_matches_interpreted_matcher_on_supported_subset` and the
interpreted resume test (`tests/definitions_and_rules_part2.rs:4,67`). Your
counts reproduce: 11 `SuspensionFrame` + 5 `PureFrameFinalizer` impls and
11 macro sites in that file. Step 6 is corrected above. The ruling:

1. **Two cohorts, one partition.** The census asserts production (607
   implementations, 589 macro sites) and the test-only S33 cohort (16, 11)
   separately, and their sum against the partition (623, 600). Every one of
   the 1 399 rows keeps its owner; F0515-F0525, F0614-F0618 and the 11 S33
   macro rows stay S33's and carry a `test-only` population mark in the
   receipts. They are still typed in S33: they compile in every test build,
   and R deletes the old traits.
2. **The cohort comes from the cfg, not from a list.** Classify a module as
   test-only by the `#[cfg(test)]` on its declaration while walking the
   module tree; do not name the file. If the guard changes, the cohort moves
   and the census turns red, which is the second forced red in Step 6.
3. **In-crate test support stays outside the 623.** Frames and macro sites
   in `#[cfg(test)] mod tests`, `tests.rs` and `tests/` directories form the
   separate test population your scanner already keeps. Report its counts;
   do not fold it into either cohort.
4. Nothing else changes. Finish the receipts and the 23-type decoder-use
   census, both forced reds, the ownership/derive replay on the final tree,
   both certificate sets, fingerprints, `cargo fmt`, and every gate in the
   Verification list. Append a new `## Execution report` after this section.


## Execution report — redispatch 4: census earned; ownership module-path scope stop

The amended census is green in both configurations, with both required forced
reds observed. **S1 remains incomplete:** the final ownership replay fails on
six source-loader rows, and the necessary general module-path fix is outside
the purpose restriction of the redispatch-3 scope amendment. No loader fix or
inventory admission was made. This is not an all-gates-green handoff.

The precise required edit is
`crates/rum-stdlib/src/tests/ownership_graph/source.rs:243`: normalize joined
module paths before comparing them with canonical file keys. Currently
`runtime/suspension/../tests/typed_slots.rs` is compared literally with
`runtime/tests/typed_slots.rs`. Rust resolves that path; the ownership loader
does not. A regression fixture belongs next to the existing path-resolution
fixture at `crates/rum-stdlib/src/tests/ownership_graph/fixtures.rs:299`.

The scope amendment authorizes ownership-module Rust edits **“for that rule
and its falsifiers only”**, meaning the computed generic adapter witness rule.
General module-path normalization and its discovery fixture need an explicit
scope extension. Six new unresolved-source rows would conceal a known loader
gap rather than repair it; they were not added to an inventory. The proposed
fix is concrete but **unapplied and untested**:
[proposed-module-normalization.patch](../../../.temp/brief-607/redispatch-4/proposed-module-normalization.patch).
The production loader and fixture source files have not changed in this
redispatch. Its existing adapter-rule changes from prior dispatches remain.

### 1. Files changed and dormant audit

The final hash/scope audit reports 47 changed/new crate files against the
original redispatch-1 baseline, no removals and no out-of-scope edits. Eleven
files changed during this redispatch: the three census files and its receipt
inventory; formatting in the five typed unit files, the compiler-control
harness and the cfg(test) prepared-state helper in eval_call_prepare.rs.
This brief and scratch evidence are the other writes. Prior baseline and
reports remain intact.

| File | Change |
| --- | --- |
| `crates/rum-runtime/src/runtime/debug_tools/mod.rs` | Runtime-visible test_report module path only (line 14). |
| `crates/rum-runtime/src/runtime/debug_tools/test_report.rs` | Runtime-visible EvalCapturedOutcome re-export only (line 10). |
| `crates/rum-runtime/src/runtime/debug_tools/test_report/model.rs` | Runtime-visible EvalCapturedOutcome declaration only (line 29); fields unchanged. |
| `crates/rum-runtime/src/runtime/definitions/application/mod.rs` | Runtime-visible ReplaceAllTraversalOutcome re-export (line 8); formatter reflow. |
| `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs` | Runtime-visible ReplaceAllTraversalOutcome declaration only (line 13). |
| `crates/rum-runtime/src/runtime/eval_call_prepare.rs` | 17 | `::<PreparedCallArguments as >::into_continuation_expr` |
| `crates/rum-runtime/src/runtime/kernel_api.rs` | Re-export the nine S1 public items. |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data.rs` | Runtime-visible DeleteCasesOutcome declaration only (line 23). |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/mod.rs` | Runtime-visible replace module only (line 5). |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace.rs` | Runtime-visible ReplacePartTarget declaration only (line 51); fields unchanged. |
| `crates/rum-runtime/src/runtime/suspension.rs` | Private typed module and nine-item public re-export. |
| `crates/rum-runtime/src/runtime/suspension/typed.rs` | 45 | `::TypedFrame::output_boundary` |
| `crates/rum-runtime/src/runtime/suspension/typed/action.rs` | 56 | `::<TypedAction < O > as >::finish_work` |
| `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs` | 61 | `::erase` |
| `crates/rum-runtime/src/runtime/suspension/typed/chain.rs` | 12 | `::start` |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | 48 | `::encode` |
| `crates/rum-runtime/src/runtime/suspension/typed/legacy.rs` | 21 pure delegations to existing wire codecs, including approved fallible prepared decoder. |
| `crates/rum-runtime/src/runtime/suspension/typed/registry.rs` | Expr=Value; 21 existing types=Legacy; dormant exact state variants/providers. |
| `crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs` | Invariant lifetime/type token with no Arc, constructor, Clone or Copy. |
| `crates/rum-runtime/src/runtime/suspension/typed/witness.rs` | 10 | `::slot_witness` |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | Add only suspension/typed/adapter.rs:TypedAdapter to the closed derive array. |
| `crates/rum-runtime/src/runtime/tests/typed_slots.rs` | Test-only frame/finalizer support and unit modules. |
| `crates/rum-runtime/src/runtime/tests/typed_slots/accounting.rs` | Root/capture-boundary delegation without executing a frame. |
| `crates/rum-runtime/src/runtime/tests/typed_slots/composition.rs` | Dormant composition, typed finalization and stopped deferred work. |
| `crates/rum-runtime/src/runtime/tests/typed_slots/faults.rs` | Wrong channel/variant rejection and error forwarding before user resume. |
| `crates/rum-runtime/src/runtime/tests/typed_slots/registry.rs` | Mode equality for all 22 registrations; actual prepared-state roundtrips/finalizer equality; leniency. |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger.rs` | Wire the new adapter witness module. |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/adapters.rs` | Computed private-trait/single-bound/owned-parameter rule; concrete paths supersede interim witnesses. |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/ownership.rs` | Try the new rule after existing concrete attribution; share the existing expression-path proof. |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs` | Three required falsifiers, additional-impl-bound rejection, and computed production witnesses. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv` | Append seven approved PublicInput and three computed Unbounded entries; preserve all 65 old rows byte-for-byte. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv` | Replay discovered positions and new scaffold constructs; exact changes in ownership-discovery-diff.tsv. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv` | Replay discovered positions and new scaffold constructs; exact changes in ownership-discovery-diff.tsv. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | Replay discovered positions and new scaffold constructs; exact changes in ownership-discovery-diff.tsv. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | Replay discovered positions and new scaffold constructs; exact changes in ownership-discovery-diff.tsv. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | Replay discovered positions and new scaffold constructs; exact changes in ownership-discovery-diff.tsv. |
| `crates/rum-stdlib/src/tests/ownership_graph/model.rs` | Retain impl generics, polarity and conditionality for the narrow adapter proof. |
| `crates/rum-stdlib/src/tests/ownership_graph/source.rs` | Fixture loader accepting explicit crate roots, used by the concrete-Expr falsifier. |
| `crates/rum-stdlib/src/tests/ownership_graph/source_items.rs` | Collect those implementation facts from the actual impl AST. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs` | Declare the static typed_slots stage-selection module. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs` | S1/no-flip selection in source; no environment-selected gate. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-shapes.tsv` | 67 planned slot rows copied from the immutable mode plan; no new carrier registered. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` | Earned S1 source receipts for all 1,399 partition rows with unchanged owners; cfg-derived populations; 134 decoders, explicit test users and whole/nested decoder-use anchors. |
| `crates/rum-stdlib/tests/typed_slot_census.rs` | Two-cohort baseline gate, exact source/owner/population reconciliation, indexed receipt lookup, source evidence outputs, S54 prerequisite, and both exercised scratch-copy falsifiers. |
| `crates/rum-stdlib/tests/typed_slot_census/classify.rs` | Canonical trait/type and free-function visibility resolution; associated inputs/outputs/modes; call edges and conservative wire-provider closure with unresolved calls retained. |
| `crates/rum-stdlib/tests/typed_slot_census/scan.rs` | Actual module/cfg walk with separate ordinary tests; AST definitions, default bodies, paths, calls, references, patterns, macro arguments and heads; explicit opaque/unreachable cases. Resolver projection is private census input, not an ownership collector change. |
| `crates/rum-stdlib/tests/typed_slot_compile_controls.rs` | All 22 paired controls; scratch-only runtime mutations; raw Cargo JSON/code/primary-span checks. |

Formatting passed. The new census test modules are 28,330 characters for the
gate/report driver, 29,083 for the module/AST walker, and 14,228 for resolution;
the compiler-control harness is 15,825. The two larger test modules need a
future split between report emission and receipt verification, and between
module discovery/projection and AST site visitation. They stay within the
three named census files for this scoped step; no additional module was
created outside Scope. The inherited 11,307-character production registry
keeps its explicit 21-type strategy/variant registrations together. Existing
large production owners have only visibility changes or the dormant prepared
helper; moving their evaluation bodies was not part of S1.

The bounded AST body audit succeeded:

```text
DORMANT AUDIT: 58 old production function bodies byte-identical; 109 new dormant bodies. cfg(test) bodies excluded. TypedAdapter's generated trace implementation is additionally dormant.
```

Every previously existing production function in the changed files retains
its exact body bytes; other production files retain their baseline hashes.
The only new production bodies are listed below. **Each is off the live
path:** no existing evaluator/matcher/traversal/block/selector caller invokes
the typed API or its providers, and no typed frame enters the queue in S1.
The prepared encoder is reached only by the dormant provider. Generated
`TypedAdapter::trace_roots` likewise has no production adapter instance.
There is no live driver change, frame conversion, typed insertion method,
exported typed suspension macro, or placeholder suspension stub. No
performance measurement ran under the brief's unchanged-body exemption.

The eight visibility-only sites are unchanged from the prior report:
`debug_tools/mod.rs:14`, `debug_tools/test_report.rs:10`,
`debug_tools/test_report/model.rs:29`, `definitions/application/mod.rs:8`,
`definitions/application/replace_all.rs:13`, `kernel_data/rewrite_data.rs:23`,
`kernel_data/structure_data/mod.rs:5`, and
`kernel_data/structure_data/replace.rs:51` (all under runtime/). These raise
only the registered type/re-export/module to pub(in crate::runtime); no field
visibility changed.

All 109 handwritten new production bodies, with exact current locations:

```text
crates/rum-runtime/src/runtime/eval_call_prepare.rs:17	::<PreparedCallArguments as >::into_continuation_expr
crates/rum-runtime/src/runtime/suspension/typed.rs:45	::TypedFrame::output_boundary
crates/rum-runtime/src/runtime/suspension/typed/action.rs:70	::<ResumeStop < O > as >::failure
crates/rum-runtime/src/runtime/suspension/typed/action.rs:29	::<TypedAction < O > as >::complete
crates/rum-runtime/src/runtime/suspension/typed/action.rs:36	::<TypedAction < O > as >::evaluate
crates/rum-runtime/src/runtime/suspension/typed/action.rs:46	::<TypedAction < O > as >::finalize
crates/rum-runtime/src/runtime/suspension/typed/action.rs:56	::<TypedAction < O > as >::finish_work
crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:70	::<FinalizerAdapter < F > as ErasedFinalizer>::finish
crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:56	::<TypedAdapter < F > as ErasedFrame>::output_boundary
crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:38	::<TypedAdapter < F > as ErasedFrame>::resume
crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:28	::<TypedAdapter < F > as FrameAbandon>::abandon
crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:61	::erase
crates/rum-runtime/src/runtime/suspension/typed/chain.rs:20	::<FrameChain < I , O > as >::then
crates/rum-runtime/src/runtime/suspension/typed/chain.rs:12	::start
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:9	::<CodecKey as >::new
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:91	::<LegacyMode as Transport < T >>::decode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:88	::<LegacyMode as Transport < T >>::encode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:103	::<StateMode as Transport < T >>::decode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:100	::<StateMode as Transport < T >>::encode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:79	::<ValueMode as Transport < Expr >>::decode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:76	::<ValueMode as Transport < Expr >>::encode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:52	::decode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:48	::encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:109	::<CasesCollectionOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:106	::<CasesCollectionOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:91	::<CasesTraversalResolution as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:88	::<CasesTraversalResolution as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:82	::<CompiledMatchOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:79	::<CompiledMatchOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:127	::<DeleteCasesOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:124	::<DeleteCasesOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:172	::<EvalCapturedOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:169	::<EvalCapturedOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:190	::<EvaluatedPartSelectors as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:187	::<EvaluatedPartSelectors as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:55	::<Option < CasesLevelSpec > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:52	::<Option < CasesLevelSpec > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:10	::<Option < Expr > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:7	::<Option < Expr > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:37	::<Option < Vec < Expr > > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:34	::<Option < Vec < Expr > > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:46	::<Option < Vec < RewriteRuleInfo > > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:43	::<Option < Vec < RewriteRuleInfo > > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:118	::<PositionCollectionOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:115	::<PositionCollectionOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:181	::<PreparedCallArguments as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:178	::<PreparedCallArguments as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:154	::<ReplaceAllTraversalOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:151	::<ReplaceAllTraversalOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:145	::<ReplaceOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:142	::<ReplaceOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:73	::<String as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:70	::<String as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:136	::<TraversalCountOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:133	::<TraversalCountOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:100	::<TraversalWithoutLimitResolution as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:97	::<TraversalWithoutLimitResolution as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:19	::<Vec < Expr > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:16	::<Vec < Expr > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:163	::<Vec < ReplacePartTarget > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:160	::<Vec < ReplacePartTarget > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:28	::<Vec < SymbolExprMap > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:25	::<Vec < SymbolExprMap > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:64	::<bool as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:61	::<bool as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:205	::<CasesCollectionOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:208	::<CasesCollectionOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:173	::<CasesTraversalResolution as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:176	::<CasesTraversalResolution as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:157	::<CompiledMatchOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:160	::<CompiledMatchOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:237	::<DeleteCasesOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:240	::<DeleteCasesOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:317	::<EvalCapturedOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:320	::<EvalCapturedOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:349	::<EvaluatedPartSelectors as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:352	::<EvaluatedPartSelectors as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:109	::<Option < CasesLevelSpec > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:112	::<Option < CasesLevelSpec > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:29	::<Option < Expr > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:32	::<Option < Expr > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:77	::<Option < Vec < Expr > > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:80	::<Option < Vec < Expr > > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:93	::<Option < Vec < RewriteRuleInfo > > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:96	::<Option < Vec < RewriteRuleInfo > > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:221	::<PositionCollectionOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:224	::<PositionCollectionOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:333	::<PreparedCallArguments as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:336	::<PreparedCallArguments as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:285	::<ReplaceAllTraversalOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:288	::<ReplaceAllTraversalOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:269	::<ReplaceOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:272	::<ReplaceOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:141	::<String as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:144	::<String as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:253	::<TraversalCountOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:256	::<TraversalCountOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:189	::<TraversalWithoutLimitResolution as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:192	::<TraversalWithoutLimitResolution as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:45	::<Vec < Expr > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:48	::<Vec < Expr > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:301	::<Vec < ReplacePartTarget > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:304	::<Vec < ReplacePartTarget > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:61	::<Vec < SymbolExprMap > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:64	::<Vec < SymbolExprMap > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:125	::<bool as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:128	::<bool as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/witness.rs:15	::<SlotWitness < I > as >::chain
crates/rum-runtime/src/runtime/suspension/typed/witness.rs:10	::slot_witness
```

Full audit artifacts: [scope](../../../.temp/brief-607/redispatch-4/scope-audit.txt),
[files](../../../.temp/brief-607/redispatch-4/files-changed.tsv),
[body audit](../../../.temp/brief-607/redispatch-4/function-body-audit.tsv).

### 2. Public API, modes and registrations

The public S1 list remains exactly `SlotValue`, `TypedFrame`, `SlotFinalizer`,
`TypedAction`, `SlotWitness`, `slot_witness`, `FrameChain`, `ContinueToken`, and
`ResumeStop`, re-exported through suspension and kernel_api. TypedAction has
complete/evaluate/finalize/finish_work; the witness comes from an actual
Result, chains require an initial witness and typed adjacency, and the
adapter-minted token is invariant, lifetime-scoped, non-Clone/non-Copy and
contains no Arc. The erased channel/actions/adapters/key stay private.

Expr alone uses ValueMode. Each sealed Codec declares one associated mode;
central keyed free functions choose the channel arm. LegacyMode accepts only
Value and gives the old provider an Expr. StateMode accepts only State and
the exact variant. State providers receive ContinuationState, never Slot.
The approved prepared decoder rejection remains Err(Slot::Value). No frame
can select a mode and no codec can override central encode/decode.

The 21 S1 Legacy registrations remain Option<Expr>, Vec<Expr>,
Vec<SymbolExprMap>, Option<Vec<Expr>>, Option<Vec<RewriteRuleInfo>>,
Option<CasesLevelSpec>, bool, String, CompiledMatchOutcome,
CasesTraversalResolution, TraversalWithoutLimitResolution,
CasesCollectionOutcome, PositionCollectionOutcome, DeleteCasesOutcome,
TraversalCountOutcome, ReplaceOutcome, ReplaceAllTraversalOutcome,
Vec<ReplacePartTarget>, EvalCapturedOutcome, PreparedCallArguments, and
EvaluatedPartSelectors. The two PreparedInterpreted enums remain unregistered
until S33; no new carrier was registered. Prior eight runtime units passed,
including actual prepared states and exact existing-finalizer wire equality;
no new unit rerun or workspace pass is claimed after this formatting pass.

### 3. Compiler-control evidence and its limits

The 22 pairs passed in both builds in redispatch 3. Those results and raw
Cargo JSON remain available. **They were not rerun in redispatch 4 before the
scope stop**, so the table is explicitly earlier checkpoint evidence, not
current-tree verification. Formatting affected unit/harness source; a fresh
mandatory run remains owed. Runtime-private negative edits stayed in scratch.

| Pair | Earlier default / no-default positive | Rejection | Primary span at that checkpoint |
| --- | --- | --- | --- |
| first-chain-input | PASS / PASS | E0271 | `src/lib.rs:26:35` |
| chain-adjacency | PASS / PASS | E0271 | `src/lib.rs:33:70` |
| unbound-chain-constructor | PASS / PASS | E0599 | `src/lib.rs:39:41` |
| evaluate-input | PASS / PASS | E0271 | `src/lib.rs:45:66` |
| evaluate-output | PASS / PASS | E0271 | `src/lib.rs:51:66` |
| finalizer-output | PASS / PASS | E0271 | `src/lib.rs:57:48` |
| opaque-action | PASS / PASS | E0599 | `src/lib.rs:62:39` |
| token-no-clone | PASS / PASS | E0599 | `src/lib.rs:68:18` |
| token-no-mint | PASS / PASS | E0599 | `src/lib.rs:74:27` |
| token-no-escape | PASS / PASS | lifetime category | `src/lib.rs:80:1` |
| sealed-extension | PASS / PASS | E0277 | `src/lib.rs:86:20` |
| codec-key-private | PASS / PASS | E0603 | `src/lib.rs:92:18` |
| opaque-selector-pass | PASS / PASS | E0271 | `src/lib.rs:99:35` |
| mode-not-frame-chosen | PASS / PASS | E0599 | `src/lib.rs:105:65` |
| mode-private-selection | PASS / PASS | E0603 | `src/lib.rs:111:18` |
| sibling-erased-private | PASS / PASS | E0603 | `crates/rum-runtime/src/runtime/mod.rs:131:47` |
| mode-expr-fixed | PASS / PASS | E0277 | `crates/rum-runtime/src/runtime/suspension/typed/registry.rs:21:17` |
| mode-new-state-only | PASS / PASS | E0277 | `crates/rum-runtime/src/runtime/suspension/typed.rs:61:40` |
| mode-one-associated-type | PASS / PASS | E0201 | `crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5` |
| mode-no-codec-override | PASS / PASS | E0407 | `crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5` |
| mode-legacy-provider-narrow | PASS / PASS | E0053 | `crates/rum-runtime/src/runtime/suspension/typed.rs:69:19` |
| mode-state-provider-narrow | PASS / PASS | E0053 | `crates/rum-runtime/src/runtime/suspension/typed.rs:74:15` |


Token escape uses the compiler's lifetime category rather than an error code.
The Expr-mode mutation also checks the related E0271 from the actual runtime
mode-equality unit; the primary required diagnostic is E0277 at the mutated
associated mode. All positive groups compiled before individual negatives.

### 4. Census, exact receipts and both forced reds

Both mandatory census configurations passed on the formatted tree. Their
baseline output is:

```text
S1 CENSUS production frames=553 finalizers=54 implementations=607 typed=0 macros=589 result=345 insert=244
S1 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S1 CENSUS test-support frames=4 finalizers=0 implementations=6 typed=2 macros=7 result=2 insert=5
S1 PARTITION implementations=623 macros=600
S1 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; production evaluation remains untyped
```

Every original partition owner was independently compared with all six
immutable input tables: 623 F, 600 M, 62 C chains, 14 D deferred obligations,
10 I routes, and 90 T transformations, totalling 1,399. The 16 F and 11 M
rows in the test-only cohort all retain S33. Their population comes from the
actual cfg module walk, not a file allowlist. The checked receipt adds 134
separate decoder definition identities, 35 explicitly enumerated ordinary
legacy test sites, 209 whole-slot records expanded to 321 actual pattern/helper
sites, and all 434 nested decoder-use sites. No guessed Input/Output type was
written for an old frame. Source positions, SHA256 and associated types/modes
are emitted separately from the stable identities.

The shared ownership resolver is reused as a lexical index. The census opens
only frame-bearing test-module ancestry in a private in-memory projection;
populations and locations always come from the original AST walk. Its path
aliases do not edit or fix the ownership loader. The first broad projection
timed out at 900 seconds; the narrowed discovery takes roughly 15–19 seconds.
Decoder replay exposed and fixed hidden references inside immediate closures,
inline Expr destructuring/helper decoding, and calls inside expression-list
macros such as vec![decode(value)]. Opaque macro expansion, unknown receivers,
aliases and function-pointer candidates remain explicit. S1 refuses preflip
at the S54 prerequisite; **this is no zero-legacy or completed preflip proof**.
The closure report retains unresolved calls and forbids evaluator/channel
operations rather than declaring them pure from their directory.

The final 23-type decoder-use report (same-name unresolved candidates are
explicit candidates, not additional proved calls):

```text
slot	decoder_id	definition	population	registered_mode	resolved_uses	unresolved_same_name_candidates
CasesCollectionOutcome	D110	rum_runtime::runtime::suspension::collections::<CasesCollectionOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	1	31
CasesTraversalResolution	D134	rum_runtime::runtime::suspension::traversal::<CasesTraversalResolution as ContinuationValue>::from_continuation_expr	production	LegacyMode	3	31
CompiledMatchOutcome	D049	rum_runtime::runtime::pattern_matching::outcome::<CompiledMatchOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	90	31
DeleteCasesOutcome	D012	rum_runtime::runtime::kernel_data::rewrite_data::<DeleteCasesOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	10	31
EvalCapturedOutcome	D001	rum_runtime::runtime::debug_tools::test_report::evaluation::<EvalCapturedOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	3	31
EvaluatedPartSelectors	D011	rum_runtime::runtime::kernel_data::evaluated_part_selectors::<EvaluatedPartSelectors as inherent>::from_continuation_expr	production	LegacyMode	4	31
Option<CasesLevelSpec>	D132	rum_runtime::runtime::suspension::traversal::<Option < CasesLevelSpec > as ContinuationValue>::from_continuation_expr	production	LegacyMode	1	31
Option<Expr>	D125	rum_runtime::runtime::suspension::primitive::<Option < Expr > as ContinuationValue>::from_continuation_expr	production	LegacyMode	57	31
Option<Vec<Expr>>	D128	rum_runtime::runtime::suspension::primitive::<Option < Vec < Expr > > as ContinuationValue>::from_continuation_expr	production	LegacyMode	13	31
Option<Vec<RewriteRuleInfo>>	D130	rum_runtime::runtime::suspension::rewrite_rules::<Option < Vec < RewriteRuleInfo > > as ContinuationValue>::from_continuation_expr	production	LegacyMode	1	31
PositionCollectionOutcome	D111	rum_runtime::runtime::suspension::collections::<PositionCollectionOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	1	31
PreparedCallArguments	D009	rum_runtime::runtime::eval_call_prepare::prepared_call_arguments_from_continuation_expr	production	LegacyMode	2	0
PreparedInterpretedAlternative	D086	rum_runtime::runtime::pattern_matching_interpreted::decode_prepared_interpreted_alternative	test-only	unregistered until S33	1	0
PreparedInterpretedCall	D085	rum_runtime::runtime::pattern_matching_interpreted::decode_prepared_interpreted_call	test-only	unregistered until S33	1	0
ReplaceAllTraversalOutcome	D005	rum_runtime::runtime::definitions::application::replace_all::<ReplaceAllTraversalOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	2	31
ReplaceOutcome	D013	rum_runtime::runtime::kernel_data::rewrite_data::<ReplaceOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	7	31
String	D124	rum_runtime::runtime::suspension::primitive::<String as ContinuationValue>::from_continuation_expr	production	LegacyMode	7	31
TraversalCountOutcome	D112	rum_runtime::runtime::suspension::collections::<TraversalCountOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	1	31
TraversalWithoutLimitResolution	D006	rum_runtime::runtime::definitions::definition_catalog::traversal_continuation::<TraversalWithoutLimitResolution as ContinuationValue>::from_continuation_expr	production	LegacyMode	1	31
Vec<Expr>	D126	rum_runtime::runtime::suspension::primitive::<Vec < Expr > as ContinuationValue>::from_continuation_expr	production	LegacyMode	47	31
Vec<ReplacePartTarget>	D026	rum_runtime::runtime::kernel_data::structure_data::replace::plan::<Vec < ReplacePartTarget > as ContinuationValue>::from_continuation_expr	production	LegacyMode	4	31
Vec<SymbolExprMap>	D127	rum_runtime::runtime::suspension::primitive::<Vec < SymbolExprMap > as ContinuationValue>::from_continuation_expr	production	LegacyMode	17	31
bool	D123	rum_runtime::runtime::suspension::primitive::<bool as ContinuationValue>::from_continuation_expr	production	LegacyMode	35	31
```

Both forced-red children ran in each configuration. The production deletion
reduces the combined population to 622. Removing the actual cfg(test) guard
moves S33 into production (623/600) and leaves the test-only cohort at 0/0;
its combined population remains 623/600, so a sum-only gate would miss it.
The parent requires each child's nonzero exit and the specific population
failure. The deliberately ignored child test is explicitly invoked by these
parents, never counted as an unexecuted falsifier.

### Removed production implementation (default)

```text

running 1 test
test forced_red_probe ... S1 CENSUS production frames=552 finalizers=54 implementations=606 typed=0 macros=589 result=345 insert=244
S1 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S1 CENSUS test-support frames=4 finalizers=0 implementations=6 typed=2 macros=7 result=2 insert=5
S1 PARTITION implementations=622 macros=600
FAILED

failures:

failures:
    forced_red_probe

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.82s

census walk: 14783 functions, 297609 uses, 1472 module visits
census resolver: 34615 scopes

thread 'forced_red_probe' panicked at crates/rum-stdlib/tests/typed_slot_census.rs:864:31:
called `Result::unwrap()` on an `Err` value: "S1 baseline requires production 607/589 and test-only 16/11; combined 623/600"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### Removed cfg(test) guard (default)

```text

running 1 test
test forced_red_probe ... S1 CENSUS production frames=564 finalizers=59 implementations=623 typed=0 macros=600 result=348 insert=252
S1 CENSUS test-only frames=0 finalizers=0 implementations=0 typed=0 macros=0 result=0 insert=0
S1 CENSUS test-support frames=4 finalizers=0 implementations=6 typed=2 macros=7 result=2 insert=5
S1 PARTITION implementations=623 macros=600
FAILED

failures:

failures:
    forced_red_probe

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.86s

census walk: 14784 functions, 297628 uses, 1472 module visits
census resolver: 34618 scopes

thread 'forced_red_probe' panicked at crates/rum-stdlib/tests/typed_slot_census.rs:864:31:
called `Result::unwrap()` on an `Err` value: "S1 baseline requires production 607/589 and test-only 16/11; combined 623/600"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```


No-default counterparts are in
[no-default frame red](../../../.temp/brief-607/census/forced-red/no-default-frame.log)
and [no-default guard red](../../../.temp/brief-607/census/forced-red/no-default-guard.log).
Full source outputs, including whole/nested tables and unresolved sites, are
under `.temp/brief-607/census/default/` and `no-default/`.

### 5. Gates, inventory rows and certificates

All Cargo/compiler commands ran serially with the mandatory bounded wrapper,
offline environment, one Cargo job, one test thread and brief-607 TMPDIR.
The read-only scratch AST audit used the same wrapper. No network, git,
sub-agent, new unsafe, or production manifest/lockfile/toolchain edit occurred.
The scratch audit has its own allowed manifest/lockfile.

Verbatim result lines for every census/ownership attempt in this redispatch
(the child failures inside passing full-target logs are the required reds):

```text
census-1.log


census-2.log
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 14.85s

census-3.log
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 14.83s

census-4.log
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 15.00s

census-5.log
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 15.75s

census-default-1.log
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 14.73s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 14.62s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 272.23s

census-default-2.log
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.82s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.86s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 133.51s

census-no-default.log
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.72s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.91s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 133.63s

ownership-discovery.log
test result: FAILED. 24 passed; 1 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 67.54s
```

`cargo fmt --check` exited 0 with empty output. The mandatory gate state is:

```text
cargo fmt --check — PASS
cargo test -p rum-stdlib --test typed_slot_compile_controls — NOT RERUN; prior default 22 pairs passed
cargo test -p rum-stdlib --no-default-features --test typed_slot_compile_controls — NOT RERUN; prior no-default 22 pairs passed
cargo test -p rum-stdlib --test typed_slot_census — PASS: 5 passed, 0 failed, 1 deliberately ignored child (explicitly executed by both falsifiers)
cargo test -p rum-stdlib --no-default-features --test typed_slot_census — PASS: 5 passed, 0 failed, 1 deliberately ignored child (explicitly executed by both falsifiers)
scripts/check-suspension-oracle.sh 1500 — NOT RUN
cargo test -p rum-stdlib --lib ownership_graph — FAILED: 24 passed, 1 failed
cargo test -p rum-stdlib --lib local_production_acceptance — NOT RUN
cargo test -p rum-stdlib --no-default-features --lib local_production_acceptance — NOT RUN
cargo test --workspace --no-fail-fast — NOT RUN
```

The final derive/named-frame gate was not run. No native ZeroMQ test ran;
there is no executed native skip or native pass to claim.

The ownership ledger, its computed witnesses and all required adapter
falsifiers passed. The exact self-reaching and host inventories passed:

```text
EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 75 rows
```

The only failing ownership check is the six unadmitted UnknownConstruct rows:

```text
UnknownConstruct	crates/rum-runtime/src/runtime/suspension/typed.rs	20	1	module typed_slots has 0 source candidates
UnknownConstruct	crates/rum-runtime/src/runtime/tests/typed_slots.rs	1	1	source file is not reachable from a production or test module declaration
UnknownConstruct	crates/rum-runtime/src/runtime/tests/typed_slots/accounting.rs	1	1	source file is not reachable from a production or test module declaration
UnknownConstruct	crates/rum-runtime/src/runtime/tests/typed_slots/composition.rs	1	1	source file is not reachable from a production or test module declaration
UnknownConstruct	crates/rum-runtime/src/runtime/tests/typed_slots/faults.rs	1	1	source file is not reachable from a production or test module declaration
UnknownConstruct	crates/rum-runtime/src/runtime/tests/typed_slots/registry.rs	1	1	source file is not reachable from a production or test module declaration
```

All ownership inventory bytes are unchanged from redispatch 3. The earlier
seven approved PublicInput and three computed Unbounded additions remain;
all 65 original ledger rows, including 56–58, remain byte-identical. No debt
was discharged or disposition strengthened. The six rows above were saved
for review only. No contract fingerprint or certificate cell changed in this
redispatch. Both retention TSVs are still byte-identical to the original
baseline and stale relative to the source changes; both earned replays remain
owed. The scratch fingerprint script was prepared but **not executed**.

The sole new inventory change here is the census receipt TSV. Exact owner and
population audit:
[receipt-and-inventory-audit.txt](../../../.temp/brief-607/redispatch-4/receipt-and-inventory-audit.txt).
Historical ownership row differences and added ledger text remain at the
redispatch-3 links in the preceding report.

### 6. Stop-loss, deviations and exact next action

This is the explicit scope-purpose stop, not the two-attempt stop-loss.
Consecutive no-progress attempts: 0. Discovery progressed from the initial
timeout to both correct cohorts; receipt replay then identified missing
closure references, inline uses and macro arguments (Whole4 → Whole52 →
Whole75 → Whole127 → Nested26 → Nested417 → complete). The final census
passes and both observed reds are concrete gate progress. The later ownership
failure is a newly exposed module-path defect; it was not retried unchanged.

After identifying this fence, no production/test source or inventory was
edited. Only read-only source/body/receipt audits, the unapplied scratch
proposal, checkpoint and this report were written. Formatting and the two
cohort gates completed before the stop. No API/driver work was added in this
redispatch. There was no rollback of the original baseline or unrelated user
edits; the task changes remain uncommitted.

The director's next decision is narrowly specified: authorize normalization
of parent components in ownership_graph/source.rs and the corresponding
module-discovery fixture in fixtures.rs, preserving cfg classification and
all unrelated discovery rows. Then apply/review that fix, rerun ownership and
derive checks, refresh all source fingerprints and both actually earned
certificate sets, rerun both compiler-control configurations, the whole oracle
and workspace, and append the final complete gate report. Current checkpoint:
`.temp/brief-607/PROGRESS.md`; original baseline:
`.temp/brief-607/redispatch-1/baseline/`.

## Director amendment — redispatch 5

The stop was correct, and the cause was the director's Scope: it placed the
typed unit tests under `runtime/tests/typed_slots*`, while they need
`suspension::typed`'s private items and so must be a child of that module.
`#[path = "../tests/typed_slots.rs"]` reconciled the two, and the ownership
loader does not follow `..`. The loader is not changed. It failed closed: it
reported the six files instead of skipping them. And the layout it rejected is
not one this project uses. `docs/code-organization-guidelines.md` puts a
module's tests in a sibling `tests.rs`, and no `src/` tree in the workspace
declares a module through a parent-directory path.

1. **Move the typed unit tests** to `suspension/typed/tests.rs` (declared
   `#[cfg(test)] mod tests;` in `suspension/typed.rs`, no `#[path]`) with its
   behaviour-family children under `suspension/typed/tests/`. Delete the
   `runtime/tests/typed_slots*` files. Prove identity: the test-name set
   before and after is equal modulo the module-path prefix (`cargo test -p
   rum-runtime --lib -- --list` both sides, diffed). The census must still
   class these files as test support.
2. **Split the census now, not later.** `typed_slot_census.rs` (28 k) and
   `typed_slot_census/scan.rs` (29 k) are past the test-file warning range,
   and the census grows at every later step. Split them along the lines your
   report names: report emission vs receipt verification, and module
   discovery/projection vs AST site visitation. New files under
   `typed_slot_census/` are in Scope. Same test-name identity proof.
3. Then finish: ownership and derive replay on the final tree (the six rows
   must vanish, not be admitted), fingerprints, both earned certificate
   sets, both compile-control configurations rerun, both census
   configurations with both forced reds, the oracle, and the workspace.
   Append a new `## Execution report` after this section.

## Execution report — S1 complete (redispatch 5)

All mandatory gates passed on the final tree. The API remains dormant: no production frame, macro or live driver was converted. No git command, network access, sub-agent, unsafe addition, dependency/manifest/lockfile/toolchain edit or unrelated rollback occurred. Work remains uncommitted for the director.

### 1. Files changed, layout identity and dormant audit

The final baseline audit has 56 changed/new crate files, plus this report. No original baseline file was deleted. The five interim runtime/tests/typed_slots* files were removed as directed; the sibling suspension/typed/tests tree now owns that code. The ownership loader has no change from this redispatch, and the old normalization proposal remains unapplied.

| File | Semantic or organizational change |
| --- | --- |
| `crates/rum-runtime/src/runtime/debug_tools/mod.rs` | Runtime-visible test_report module path only (line 14). |
| `crates/rum-runtime/src/runtime/debug_tools/test_report/model.rs` | Runtime-visible EvalCapturedOutcome declaration only (line 29); fields unchanged. |
| `crates/rum-runtime/src/runtime/debug_tools/test_report.rs` | Runtime-visible EvalCapturedOutcome re-export only (line 10). |
| `crates/rum-runtime/src/runtime/definitions/application/mod.rs` | Runtime-visible ReplaceAllTraversalOutcome re-export (line 8); formatter reflow. |
| `crates/rum-runtime/src/runtime/definitions/application/replace_all.rs` | Runtime-visible ReplaceAllTraversalOutcome declaration only (line 13). |
| `crates/rum-runtime/src/runtime/eval_call_prepare.rs` | Dormant move-based prepared encoder using the existing wire finalizer; cfg(test) wire/state comparison helper. Existing evaluator bodies unchanged. |
| `crates/rum-runtime/src/runtime/kernel_api.rs` | Re-export the nine S1 public items. |
| `crates/rum-runtime/src/runtime/kernel_data/rewrite_data.rs` | Runtime-visible DeleteCasesOutcome declaration only (line 23). |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/mod.rs` | Runtime-visible replace module only (line 5). |
| `crates/rum-runtime/src/runtime/kernel_data/structure_data/replace.rs` | Runtime-visible ReplacePartTarget declaration only (line 51); fields unchanged. |
| `crates/rum-runtime/src/runtime/suspension/typed/action.rs` | Opaque output-typed complete/evaluate/finalize/finish_work actions and ResumeStop::failure. |
| `crates/rum-runtime/src/runtime/suspension/typed/adapter.rs` | Private erasure, root/abandon/boundary delegation, decode-before-resume and sole token mint. |
| `crates/rum-runtime/src/runtime/suspension/typed/chain.rs` | Witnessed nonempty chain start, invariant typed adjacency and private erased storage. |
| `crates/rum-runtime/src/runtime/suspension/typed/channel.rs` | Private keyed associated-mode strategies and central encode/decode; wrong-arm and exact-variant checks. |
| `crates/rum-runtime/src/runtime/suspension/typed/legacy.rs` | 21 pure delegations to existing wire codecs, including approved fallible prepared decoder. |
| `crates/rum-runtime/src/runtime/suspension/typed/registry.rs` | Expr=Value; 21 existing types=Legacy; dormant exact state variants/providers. |
| `crates/rum-runtime/src/runtime/suspension/typed/resume_token.rs` | Invariant lifetime/type token with no Arc, constructor, Clone or Copy. |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/accounting.rs` | Root/capture-boundary delegation without executing a frame. Moved byte-for-byte. |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/composition.rs` | Dormant composition, typed finalization and stopped deferred work. Moved byte-for-byte. |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/faults.rs` | Wrong channel/variant rejection and error forwarding before user resume. Moved byte-for-byte. |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/registry.rs` | Mode equality for all 22 registrations; actual prepared-state roundtrips/finalizer equality; leniency. Moved byte-for-byte. |
| `crates/rum-runtime/src/runtime/suspension/typed/tests/support.rs` | Shared Echo/Finish/echo fixtures moved into support; bodies unchanged, minimal test-only visibility. |
| `crates/rum-runtime/src/runtime/suspension/typed/tests.rs` | Sibling test facade, behavior-family declarations and shared support imports. |
| `crates/rum-runtime/src/runtime/suspension/typed/witness.rs` | Invariant zero-sized witness from an actual Result and bound chain start. |
| `crates/rum-runtime/src/runtime/suspension/typed.rs` | SlotValue, TypedFrame and SlotFinalizer contracts; private modules and conventional cfg(test) mod tests. No driver integration. |
| `crates/rum-runtime/src/runtime/suspension.rs` | Private typed module and nine-item public re-export. |
| `crates/rum-runtime/src/runtime/tests/frame_root_trace/derived_inventory.rs` | Add only suspension/typed/adapter.rs:TypedAdapter to the closed derive array. |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/adapters.rs` | Computed private-trait/single-bound/owned-parameter rule; concrete paths supersede interim witnesses. |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/ownership.rs` | Try the new rule after existing concrete attribution; share the existing expression-path proof. |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger/tests.rs` | Three required falsifiers, additional-impl-bound rejection, and computed production witnesses. |
| `crates/rum-stdlib/src/tests/ownership_graph/host_payload_ledger.rs` | Wire the new adapter witness module. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payload_ledger.tsv` | Append seven approved PublicInput and three computed Unbounded entries; preserve all 65 old rows byte-for-byte. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/host_payloads.tsv` | Exact discovered rows/positions replayed; all row differences are in inventory-row-diff.patch. Unchanged during redispatch 5. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/self_reaching.tsv` | Exact discovered rows/positions replayed; all row differences are in inventory-row-diff.patch. Unchanged during redispatch 5. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_attributes.tsv` | Exact discovered rows/positions replayed; all row differences are in inventory-row-diff.patch. Unchanged during redispatch 5. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_captures.tsv` | Exact discovered rows/positions replayed; all row differences are in inventory-row-diff.patch. Unchanged during redispatch 5. |
| `crates/rum-stdlib/src/tests/ownership_graph/inventory/unknown_macros.tsv` | Exact discovered rows/positions replayed; all row differences are in inventory-row-diff.patch. Unchanged during redispatch 5. |
| `crates/rum-stdlib/src/tests/ownership_graph/model.rs` | Retain impl generics, polarity and conditionality for the narrow adapter proof. |
| `crates/rum-stdlib/src/tests/ownership_graph/source.rs` | Fixture loader accepting explicit crate roots, used by the concrete-Expr falsifier. |
| `crates/rum-stdlib/src/tests/ownership_graph/source_items.rs` | Collect those implementation facts from the actual impl AST. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | Refresh only 68 fingerprint cells; all first 18 fields unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | Copy only four actually earned cells: C01/C02 in default/no-default builds. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/mod.rs` | Declare the static typed_slots stage-selection module. |
| `crates/rum-stdlib/tests/suspension_oracle/fixtures/typed_slots.rs` | S1/no-flip selection in source; no environment-selected gate. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-shapes.tsv` | 67 planned slot rows copied from the immutable mode plan; no new carrier registered. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` | S1 receipts retain all 1,399 partition owners; redispatch 5 only relocates X0023/X0024 test prefixes/paths. |
| `crates/rum-stdlib/tests/typed_slot_census/classify.rs` | Canonical trait/type and free-function visibility resolution; associated inputs/outputs/modes; call edges and conservative wire-provider closure with unresolved calls retained. |
| `crates/rum-stdlib/tests/typed_slot_census/discovery.rs` | Workspace/build selection and source resolution orchestration. |
| `crates/rum-stdlib/tests/typed_slot_census/receipts.rs` | Receipt indexing, exact cohorts/owners/anchors and preflip acceptance/refusal. |
| `crates/rum-stdlib/tests/typed_slot_census/reporting.rs` | Source hashes and deterministic census/decoder-use report emission. |
| `crates/rum-stdlib/tests/typed_slot_census/scan/discovery.rs` | File/module discovery, cfg/test-support classification and Walker state. |
| `crates/rum-stdlib/tests/typed_slot_census/scan/projection.rs` | Private resolver projection; original populations and source spans stay authoritative. |
| `crates/rum-stdlib/tests/typed_slot_census/scan/sites.rs` | AST site visitation; one forwarding method calls the moved module-discovery body. |
| `crates/rum-stdlib/tests/typed_slot_census/scan.rs` | Shared AST facts and scanner facade. |
| `crates/rum-stdlib/tests/typed_slot_census.rs` | Test facade and bounded forced-red orchestration. |
| `crates/rum-stdlib/tests/typed_slot_compile_controls.rs` | 22 compiler-control pairs with primary diagnostic code/category/span checks; companion diagnostic path follows the moved unit tests. |


The eight visibility-only sites below runtime/ are debug_tools/mod.rs:14, debug_tools/test_report.rs:10, debug_tools/test_report/model.rs:29, definitions/application/mod.rs:8, definitions/application/replace_all.rs:13, kernel_data/rewrite_data.rs:23, kernel_data/structure_data/mod.rs:5 and kernel_data/structure_data/replace.rs:51. Visibility goes only to crate::runtime; no production field visibility changed. The seven ownership Rust files above implement the previously authorized adapter proof and falsifiers; all are byte-identical to this redispatch's entry.


```text
runtime: 1160 tests before = 1160 after; normalized diff empty
typed cohort: 8 tests
census: 6 tests before = 6 after; normalized diff empty
All four moved behavior-family files byte-identical; old paths absent.
Final runtime list after support extraction: 1160 tests; normalized diff empty.
```


```text
SPLIT AUDIT: 77 pre-split function bodies byte-identical; one new Visit::visit_item_mod forwarding body. Runtime family files also byte-identical.
```

The identity evidence uses actual bounded cargo test -- --list runs on both sides. Only the typed::typed_slots to typed::tests prefix is normalized. The four behavior-family files are byte-identical. Shared fixtures moved to tests/support.rs to follow the documented support boundary and keep test tracing outside the unchanged production inventory scanner.

The split map preceded edits. The census facade is 5,326 characters; scanner facade 2,713; module discovery 9,139; projection 5,284; AST visitation 13,023; report emission 10,195; receipt checks 12,205; classification 14,228; orchestration 1,656. All census modules are below the 15k preferred test threshold. The 15,830-character compiler harness retains its cohesive paired-fixture/diagnostic driver in the warning range; its next growth boundary is fixture construction versus runner/diagnostics. The 11,307-character registry retains the explicit 21-type mode/variant mappings together. Existing large production owners have visibility-only edits or the dormant prepared helper; their evaluation bodies were not split in S1.


```text
DORMANT AUDIT: 58 old production function bodies byte-identical; 109 new dormant bodies. cfg(test) bodies excluded. TypedAdapter's generated trace implementation is additionally dormant.
```

Every existing production body in touched files is byte-identical; other production files retain baseline hashes. All 109 new production bodies are listed below. Each belongs only to the dormant typed API, keyed strategies, legacy/state providers, or the new prepared wire encoder reached through its dormant provider. None is on a live evaluator/matcher/traversal/block/selector/queue path. Empty marker/Codec impls select the fixed mode; TypedAdapter's generated root-trace impl is also dormant. The brief's performance exemption applies: no allocation, instruction-slope or microbench measurement ran.


```text
crates/rum-runtime/src/runtime/eval_call_prepare.rs:17	::<PreparedCallArguments as >::into_continuation_expr
crates/rum-runtime/src/runtime/suspension/typed/action.rs:70	::<ResumeStop < O > as >::failure
crates/rum-runtime/src/runtime/suspension/typed/action.rs:29	::<TypedAction < O > as >::complete
crates/rum-runtime/src/runtime/suspension/typed/action.rs:36	::<TypedAction < O > as >::evaluate
crates/rum-runtime/src/runtime/suspension/typed/action.rs:46	::<TypedAction < O > as >::finalize
crates/rum-runtime/src/runtime/suspension/typed/action.rs:56	::<TypedAction < O > as >::finish_work
crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:70	::<FinalizerAdapter < F > as ErasedFinalizer>::finish
crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:56	::<TypedAdapter < F > as ErasedFrame>::output_boundary
crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:38	::<TypedAdapter < F > as ErasedFrame>::resume
crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:28	::<TypedAdapter < F > as FrameAbandon>::abandon
crates/rum-runtime/src/runtime/suspension/typed/adapter.rs:61	::erase
crates/rum-runtime/src/runtime/suspension/typed/chain.rs:20	::<FrameChain < I , O > as >::then
crates/rum-runtime/src/runtime/suspension/typed/chain.rs:12	::start
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:9	::<CodecKey as >::new
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:91	::<LegacyMode as Transport < T >>::decode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:88	::<LegacyMode as Transport < T >>::encode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:103	::<StateMode as Transport < T >>::decode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:100	::<StateMode as Transport < T >>::encode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:79	::<ValueMode as Transport < Expr >>::decode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:76	::<ValueMode as Transport < Expr >>::encode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:52	::decode
crates/rum-runtime/src/runtime/suspension/typed/channel.rs:48	::encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:109	::<CasesCollectionOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:106	::<CasesCollectionOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:91	::<CasesTraversalResolution as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:88	::<CasesTraversalResolution as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:82	::<CompiledMatchOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:79	::<CompiledMatchOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:127	::<DeleteCasesOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:124	::<DeleteCasesOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:172	::<EvalCapturedOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:169	::<EvalCapturedOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:190	::<EvaluatedPartSelectors as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:187	::<EvaluatedPartSelectors as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:55	::<Option < CasesLevelSpec > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:52	::<Option < CasesLevelSpec > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:10	::<Option < Expr > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:7	::<Option < Expr > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:37	::<Option < Vec < Expr > > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:34	::<Option < Vec < Expr > > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:46	::<Option < Vec < RewriteRuleInfo > > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:43	::<Option < Vec < RewriteRuleInfo > > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:118	::<PositionCollectionOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:115	::<PositionCollectionOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:181	::<PreparedCallArguments as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:178	::<PreparedCallArguments as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:154	::<ReplaceAllTraversalOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:151	::<ReplaceAllTraversalOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:145	::<ReplaceOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:142	::<ReplaceOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:73	::<String as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:70	::<String as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:136	::<TraversalCountOutcome as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:133	::<TraversalCountOutcome as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:100	::<TraversalWithoutLimitResolution as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:97	::<TraversalWithoutLimitResolution as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:19	::<Vec < Expr > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:16	::<Vec < Expr > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:163	::<Vec < ReplacePartTarget > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:160	::<Vec < ReplacePartTarget > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:28	::<Vec < SymbolExprMap > as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:25	::<Vec < SymbolExprMap > as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:64	::<bool as LegacyCodec>::old_decode
crates/rum-runtime/src/runtime/suspension/typed/legacy.rs:61	::<bool as LegacyCodec>::old_encode
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:205	::<CasesCollectionOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:208	::<CasesCollectionOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:173	::<CasesTraversalResolution as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:176	::<CasesTraversalResolution as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:157	::<CompiledMatchOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:160	::<CompiledMatchOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:237	::<DeleteCasesOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:240	::<DeleteCasesOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:317	::<EvalCapturedOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:320	::<EvalCapturedOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:349	::<EvaluatedPartSelectors as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:352	::<EvaluatedPartSelectors as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:109	::<Option < CasesLevelSpec > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:112	::<Option < CasesLevelSpec > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:29	::<Option < Expr > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:32	::<Option < Expr > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:77	::<Option < Vec < Expr > > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:80	::<Option < Vec < Expr > > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:93	::<Option < Vec < RewriteRuleInfo > > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:96	::<Option < Vec < RewriteRuleInfo > > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:221	::<PositionCollectionOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:224	::<PositionCollectionOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:333	::<PreparedCallArguments as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:336	::<PreparedCallArguments as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:285	::<ReplaceAllTraversalOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:288	::<ReplaceAllTraversalOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:269	::<ReplaceOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:272	::<ReplaceOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:141	::<String as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:144	::<String as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:253	::<TraversalCountOutcome as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:256	::<TraversalCountOutcome as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:189	::<TraversalWithoutLimitResolution as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:192	::<TraversalWithoutLimitResolution as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:45	::<Vec < Expr > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:48	::<Vec < Expr > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:301	::<Vec < ReplacePartTarget > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:304	::<Vec < ReplacePartTarget > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:61	::<Vec < SymbolExprMap > as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:64	::<Vec < SymbolExprMap > as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:125	::<bool as StateCodec>::pack
crates/rum-runtime/src/runtime/suspension/typed/registry.rs:128	::<bool as StateCodec>::unpack
crates/rum-runtime/src/runtime/suspension/typed/witness.rs:15	::<SlotWitness < I > as >::chain
crates/rum-runtime/src/runtime/suspension/typed/witness.rs:10	::slot_witness
crates/rum-runtime/src/runtime/suspension/typed.rs:44	::TypedFrame::output_boundary
```

Final verification left all 1,456 source/inventory files byte-identical to the pre-workspace checkpoint: [final-source-audit.txt](../../../.temp/brief-607/redispatch-5/final-source-audit.txt).

Full audits: [scope-audit.txt](../../../.temp/brief-607/redispatch-5/scope-audit.txt), [split-map.md](../../../.temp/brief-607/redispatch-5/split-map.md), [function-body-audit.tsv](../../../.temp/brief-607/redispatch-5/function-body-audit.tsv), [file-sizes.tsv](../../../.temp/brief-607/redispatch-5/file-sizes.tsv).

### 2. Public API, modes and registrations

The public S1 items are exactly SlotValue, TypedFrame, SlotFinalizer, TypedAction, SlotWitness, slot_witness, FrameChain, ContinueToken and ResumeStop, re-exported through suspension and kernel_api. TypedFrame does not inherit SuspensionFrame. TypedAction exposes complete/evaluate/finalize/finish_work; SlotWitness::chain requires a witness from an actual &Result; FrameChain::then checks adjacency. ResumeStop::failure preserves stopped work. ContinueToken is invariant, lifetime-scoped, non-Clone/non-Copy, holds no Arc and is minted only by the adapter. There is no insertion method, suspension stub, new macro or driver activation.

Hidden Codec has one associated Mode; central keyed free encode/decode functions cannot be overridden. Expr alone uses ValueMode. LegacyMode accepts only Value and gives the old codec an Expr, retaining wire shape and leniency; the approved fallible prepared decoder preserves rejection as the original Value slot. StateMode accepts only State and the exact variant; providers receive ContinuationState, never Slot. Neither frames nor public callers choose modes.

The 21 S1 Legacy registrations are:


```text
CasesCollectionOutcome
CasesTraversalResolution
CompiledMatchOutcome
DeleteCasesOutcome
EvalCapturedOutcome
EvaluatedPartSelectors
Option<CasesLevelSpec>
Option<Expr>
Option<Vec<Expr>>
Option<Vec<RewriteRuleInfo>>
PositionCollectionOutcome
PreparedCallArguments
ReplaceAllTraversalOutcome
ReplaceOutcome
String
TraversalCountOutcome
TraversalWithoutLimitResolution
Vec<Expr>
Vec<ReplacePartTarget>
Vec<SymbolExprMap>
bool
```

The two PreparedInterpreted enums remain unregistered until S33. No new carrier is registered. All eight typed units passed in the final workspace run, covering rejection before resume, channel/variant faults, root/output-boundary delegation, witnessed composition, deferred stopping, modes, prepared-state wire equality and leniency.

### 3. Compiler controls

All 22 pairs ran again on the final source in both configurations. Positive fixtures compile first; each negative introduces one intended invalidity. The harness requires the primary diagnostic code/category and exact file/line/column and rejects unrelated errors. Runtime-private mutations are scratch-only. The Expr-mode control also explicitly classifies the intended companion mode-equality diagnostic in the moved unit fixture.

| Pair | Positive default / no-default | Rejection in both | Primary span in both |
| --- | --- | --- | --- |
| first-chain-input | PASS / PASS | E0271 | `src/lib.rs:26:35` |
| chain-adjacency | PASS / PASS | E0271 | `src/lib.rs:33:70` |
| unbound-chain-constructor | PASS / PASS | E0599 | `src/lib.rs:39:41` |
| evaluate-input | PASS / PASS | E0271 | `src/lib.rs:45:66` |
| evaluate-output | PASS / PASS | E0271 | `src/lib.rs:51:66` |
| finalizer-output | PASS / PASS | E0271 | `src/lib.rs:57:48` |
| opaque-action | PASS / PASS | E0599 | `src/lib.rs:62:39` |
| token-no-clone | PASS / PASS | E0599 | `src/lib.rs:68:18` |
| token-no-mint | PASS / PASS | E0599 | `src/lib.rs:74:27` |
| token-no-escape | PASS / PASS | lifetime may not live long enough | `src/lib.rs:80:1` |
| sealed-extension | PASS / PASS | E0277 | `src/lib.rs:86:20` |
| codec-key-private | PASS / PASS | E0603 | `src/lib.rs:92:18` |
| opaque-selector-pass | PASS / PASS | E0271 | `src/lib.rs:99:35` |
| mode-not-frame-chosen | PASS / PASS | E0599 | `src/lib.rs:105:65` |
| mode-private-selection | PASS / PASS | E0603 | `src/lib.rs:111:18` |
| sibling-erased-private | PASS / PASS | E0603 | `crates/rum-runtime/src/runtime/mod.rs:131:47` |
| mode-expr-fixed | PASS / PASS | E0277 | `crates/rum-runtime/src/runtime/suspension/typed/registry.rs:21:17` |
| mode-new-state-only | PASS / PASS | E0277 | `crates/rum-runtime/src/runtime/suspension/typed.rs:60:40` |
| mode-one-associated-type | PASS / PASS | E0201 | `crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5` |
| mode-no-codec-override | PASS / PASS | E0407 | `crates/rum-runtime/src/runtime/suspension/typed/registry.rs:27:5` |
| mode-legacy-provider-narrow | PASS / PASS | E0053 | `crates/rum-runtime/src/runtime/suspension/typed.rs:68:19` |
| mode-state-provider-narrow | PASS / PASS | E0053 | `crates/rum-runtime/src/runtime/suspension/typed.rs:73:15` |


Full positive/negative sources and Cargo JSON remain under .temp/brief-607/compile-controls/{default,no-default}/. Snapshots: [controls-default-results.json](../../../.temp/brief-607/redispatch-5/controls-default-results.json) and [controls-no-default-results.json](../../../.temp/brief-607/redispatch-5/controls-no-default-results.json).

### 4. Census and all four forced failures

The separate production/test-only cohorts are asserted independently:


```text
S1 CENSUS production frames=553 finalizers=54 implementations=607 typed=0 macros=589 result=345 insert=244
S1 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S1 CENSUS test-support frames=4 finalizers=0 implementations=6 typed=2 macros=7 result=2 insert=5
S1 PARTITION implementations=623 macros=600
S1 REGISTRY Value=1 Legacy=21; legacy whole-slot types=23; production evaluation remains untyped
```


```text
Moved test cohort: 18 function definitions across all five family/support files, all test-support; 2 typed implementations, both test-support. tests.rs is the cfg(test) mod tests facade. Stage receipt bytes differ from entry only by the two authorized X0023/X0024 path/prefix relocations; all 1399 partition owners unchanged.
```

All 623 F, 600 M, 62 chain, 14 deferred-obligation, 10 route and 90 transformation owners remain from the immutable partition (1,399 total). All 16 test-only implementations and 11 macro sites remain S33, discovered through the actual cfg(test) declaration. Other receipts cover 134 decoder definitions, 35 ordinary legacy test sites, 209 whole-slot records expanded to 321 pattern/helper sites and all 434 nested decoder uses. Only X0023/X0024 moved with their unit module. No old Input/Output type was guessed.

The census preserves module-qualified identities, source spans/hashes and explicit unresolved calls, macros and references. Its resolver projection is private census input; original AST populations stay authoritative. S1 refuses preflip at the required S54 receipt: this is no zero-legacy or completed preflip proof. The provider closure retains forbidden/unresolved calls. The 23-type decoder-use report is:


```text
slot	decoder_id	definition	population	registered_mode	resolved_uses	unresolved_same_name_candidates
CasesCollectionOutcome	D110	rum_runtime::runtime::suspension::collections::<CasesCollectionOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	1	31
CasesTraversalResolution	D134	rum_runtime::runtime::suspension::traversal::<CasesTraversalResolution as ContinuationValue>::from_continuation_expr	production	LegacyMode	3	31
CompiledMatchOutcome	D049	rum_runtime::runtime::pattern_matching::outcome::<CompiledMatchOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	90	31
DeleteCasesOutcome	D012	rum_runtime::runtime::kernel_data::rewrite_data::<DeleteCasesOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	10	31
EvalCapturedOutcome	D001	rum_runtime::runtime::debug_tools::test_report::evaluation::<EvalCapturedOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	3	31
EvaluatedPartSelectors	D011	rum_runtime::runtime::kernel_data::evaluated_part_selectors::<EvaluatedPartSelectors as inherent>::from_continuation_expr	production	LegacyMode	4	31
Option<CasesLevelSpec>	D132	rum_runtime::runtime::suspension::traversal::<Option < CasesLevelSpec > as ContinuationValue>::from_continuation_expr	production	LegacyMode	1	31
Option<Expr>	D125	rum_runtime::runtime::suspension::primitive::<Option < Expr > as ContinuationValue>::from_continuation_expr	production	LegacyMode	57	31
Option<Vec<Expr>>	D128	rum_runtime::runtime::suspension::primitive::<Option < Vec < Expr > > as ContinuationValue>::from_continuation_expr	production	LegacyMode	13	31
Option<Vec<RewriteRuleInfo>>	D130	rum_runtime::runtime::suspension::rewrite_rules::<Option < Vec < RewriteRuleInfo > > as ContinuationValue>::from_continuation_expr	production	LegacyMode	1	31
PositionCollectionOutcome	D111	rum_runtime::runtime::suspension::collections::<PositionCollectionOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	1	31
PreparedCallArguments	D009	rum_runtime::runtime::eval_call_prepare::prepared_call_arguments_from_continuation_expr	production	LegacyMode	2	0
PreparedInterpretedAlternative	D086	rum_runtime::runtime::pattern_matching_interpreted::decode_prepared_interpreted_alternative	test-only	unregistered until S33	1	0
PreparedInterpretedCall	D085	rum_runtime::runtime::pattern_matching_interpreted::decode_prepared_interpreted_call	test-only	unregistered until S33	1	0
ReplaceAllTraversalOutcome	D005	rum_runtime::runtime::definitions::application::replace_all::<ReplaceAllTraversalOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	2	31
ReplaceOutcome	D013	rum_runtime::runtime::kernel_data::rewrite_data::<ReplaceOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	7	31
String	D124	rum_runtime::runtime::suspension::primitive::<String as ContinuationValue>::from_continuation_expr	production	LegacyMode	7	31
TraversalCountOutcome	D112	rum_runtime::runtime::suspension::collections::<TraversalCountOutcome as ContinuationValue>::from_continuation_expr	production	LegacyMode	1	31
TraversalWithoutLimitResolution	D006	rum_runtime::runtime::definitions::definition_catalog::traversal_continuation::<TraversalWithoutLimitResolution as ContinuationValue>::from_continuation_expr	production	LegacyMode	1	31
Vec<Expr>	D126	rum_runtime::runtime::suspension::primitive::<Vec < Expr > as ContinuationValue>::from_continuation_expr	production	LegacyMode	47	31
Vec<ReplacePartTarget>	D026	rum_runtime::runtime::kernel_data::structure_data::replace::plan::<Vec < ReplacePartTarget > as ContinuationValue>::from_continuation_expr	production	LegacyMode	4	31
Vec<SymbolExprMap>	D127	rum_runtime::runtime::suspension::primitive::<Vec < SymbolExprMap > as ContinuationValue>::from_continuation_expr	production	LegacyMode	17	31
bool	D123	rum_runtime::runtime::suspension::primitive::<bool as ContinuationValue>::from_continuation_expr	production	LegacyMode	35	31
```

Each forced-red parent explicitly invokes the ignored child with --exact and --ignored, requiring a nonzero exit plus the specific population error. The single ignored entry in the parent totals is an exercised child, not an unrun falsifier. Real red output in both builds:


default: removed frame


```text

running 1 test
test forced_red_probe ... S1 CENSUS production frames=552 finalizers=54 implementations=606 typed=0 macros=589 result=345 insert=244
S1 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S1 CENSUS test-support frames=4 finalizers=0 implementations=6 typed=2 macros=7 result=2 insert=5
S1 PARTITION implementations=622 macros=600
FAILED

failures:

failures:
    forced_red_probe

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.65s

census walk: 14784 functions, 297612 uses, 1479 module visits
census resolver: 34616 scopes

thread 'forced_red_probe' panicked at crates/rum-stdlib/tests/typed_slot_census.rs:171:31:
called `Result::unwrap()` on an `Err` value: "S1 baseline requires production 607/589 and test-only 16/11; combined 623/600"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```


default: removed guard


```text

running 1 test
test forced_red_probe ... S1 CENSUS production frames=564 finalizers=59 implementations=623 typed=0 macros=600 result=348 insert=252
S1 CENSUS test-only frames=0 finalizers=0 implementations=0 typed=0 macros=0 result=0 insert=0
S1 CENSUS test-support frames=4 finalizers=0 implementations=6 typed=2 macros=7 result=2 insert=5
S1 PARTITION implementations=623 macros=600
FAILED

failures:

failures:
    forced_red_probe

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.70s

census walk: 14785 functions, 297631 uses, 1479 module visits
census resolver: 34619 scopes

thread 'forced_red_probe' panicked at crates/rum-stdlib/tests/typed_slot_census.rs:171:31:
called `Result::unwrap()` on an `Err` value: "S1 baseline requires production 607/589 and test-only 16/11; combined 623/600"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```


no-default: removed frame


```text

running 1 test
test forced_red_probe ... S1 CENSUS production frames=552 finalizers=54 implementations=606 typed=0 macros=589 result=345 insert=244
S1 CENSUS test-only frames=11 finalizers=5 implementations=16 typed=0 macros=11 result=3 insert=8
S1 CENSUS test-support frames=4 finalizers=0 implementations=6 typed=2 macros=7 result=2 insert=5
S1 PARTITION implementations=622 macros=600
FAILED

failures:

failures:
    forced_red_probe

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.88s

census walk: 14784 functions, 297612 uses, 1479 module visits
census resolver: 34616 scopes

thread 'forced_red_probe' panicked at crates/rum-stdlib/tests/typed_slot_census.rs:171:31:
called `Result::unwrap()` on an `Err` value: "S1 baseline requires production 607/589 and test-only 16/11; combined 623/600"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```


no-default: removed guard


```text

running 1 test
test forced_red_probe ... S1 CENSUS production frames=564 finalizers=59 implementations=623 typed=0 macros=600 result=348 insert=252
S1 CENSUS test-only frames=0 finalizers=0 implementations=0 typed=0 macros=0 result=0 insert=0
S1 CENSUS test-support frames=4 finalizers=0 implementations=6 typed=2 macros=7 result=2 insert=5
S1 PARTITION implementations=623 macros=600
FAILED

failures:

failures:
    forced_red_probe

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.97s

census walk: 14785 functions, 297631 uses, 1479 module visits
census resolver: 34619 scopes

thread 'forced_red_probe' panicked at crates/rum-stdlib/tests/typed_slot_census.rs:171:31:
called `Result::unwrap()` on an `Err` value: "S1 baseline requires production 607/589 and test-only 16/11; combined 623/600"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```


Full source tables are under .temp/brief-607/census/{default,no-default}/.

### 5. Gates, inventories and earned certificates

Gates ran serially through bounded process-group cleanup with one Cargo job, one test thread, offline mode and brief-607 TMPDIR. Focused bounds were 900 seconds; workspace 3,600. The required unmodified oracle script used 1,500 seconds per build (3,100 outer bound), retaining its own usual .temp/suspension-oracle logs/tmp; both logs were copied into brief-607. Expected failed compiler fixtures and census children are acceptance evidence, not failed final gates.

Final gate result lines, verbatim:


```text
cargo fmt --check: exit 0; empty output
controls-default.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.50s
controls-no-default.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.12s
census-default.log
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.65s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.70s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 133.96s
census-no-default.log
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.88s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.97s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 134.83s
oracle-default.log
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.49s
oracle-no-default.log
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.78s
ownership-final.log
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 68.39s
derive-final.log
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1145 filtered out; finished in 1.26s
cert-default-final.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.19s
cert-no-default-final.log
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 18.06s
```


Workspace: cargo test --workspace --no-fail-fast -- --nocapture, exit 0. The 53 passing target summaries total 3,434 harness passes and 59 ignored entries; nine of those counted-as-ok entries are the native skips disclosed below. The only two failed summaries are the required census-child reds. Verbatim target/result sequence (including intentional forced-red child summaries):


```text
     Running unittests src/main.rs (target/debug/deps/rum_cli-675a9b7fb4c09027)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running tests/cli.rs (target/debug/deps/cli-417da8e232f3f31f)
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running tests/input_requests.rs (target/debug/deps/input_requests-79d242612dc5614e)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
     Running tests/terminal.rs (target/debug/deps/terminal-cd41974a9dfe37a5)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.90s
     Running unittests src/lib.rs (target/debug/deps/rum_core-49fc2a810a19f202)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_derive-b50628eb478e3209)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/accumulator_chain.rs (target/debug/deps/accumulator_chain-f4b1762065518378)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/derive.rs (target/debug/deps/derive-b38e62f915ec5c7d)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_host-a119f062096badf7)
test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s
     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-36e2ac93dcf516de)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/batch_boundary_ownership.rs (target/debug/deps/batch_boundary_ownership-62cec20e1429084a)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
     Running tests/capture_output_lifecycle.rs (target/debug/deps/capture_output_lifecycle-c541c492bd454062)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-4d8893fc9e0cdd96)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/host_payload_destruction_contract.rs (target/debug/deps/host_payload_destruction_contract-1be17d3af592a343)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 50.13s
     Running tests/installer.rs (target/debug/deps/installer-8ebb99c00d41e78e)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-5e85c48ede3e9593)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-a2c563982853655a)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s
     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-ddd2fec076417963)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/shared_accumulator_completion.rs (target/debug/deps/shared_accumulator_completion-067001aa2ecf9629)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/stress.rs (target/debug/deps/stress-5ec2a75152f1305e)
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.28s
     Running tests/suspended_timeout_contract.rs (target/debug/deps/suspended_timeout_contract-258c488f1b37cde7)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.07s
     Running tests/suspension_abandonment_contract.rs (target/debug/deps/suspension_abandonment_contract-ac93b53b9ae0d1d1)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running unittests src/lib.rs (target/debug/deps/rum_jupyter-edb9ee3aea58135e)
test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s
     Running unittests src/bin/rum-jupyter-kernel.rs (target/debug/deps/rum_jupyter_kernel-965588c9ac9ffa23)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/lib.rs (target/debug/deps/rum_repl-edcfd3382d4bfc76)
test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s
     Running tests/input_abandonment.rs (target/debug/deps/input_abandonment-144216c549618286)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/input_continuation.rs (target/debug/deps/input_continuation-ebdbc4f7476ef252)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
     Running unittests src/lib.rs (target/debug/deps/rum_runtime-562b258c6eef9b02)
test result: ok. 1157 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 43.67s
     Running unittests src/lib.rs (target/debug/deps/rum_stdlib-c8c5a89643371dc7)
test result: ok. 1092 passed; 0 failed; 53 ignored; 0 measured; 0 filtered out; finished in 1142.90s
     Running unittests src/bin/generate-builtin-reference.rs (target/debug/deps/generate_builtin_reference-7e0be655aec7b381)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/default_fallback_continuations.rs (target/debug/deps/default_fallback_continuations-c846b88aba46e50a)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running tests/destructuring_continuations.rs (target/debug/deps/destructuring_continuations-dfe1523f84cb69ee)
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
     Running tests/guest_loading_resume.rs (target/debug/deps/guest_loading_resume-0992a185a5c665db)
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
     Running tests/lazy_pull_continuations.rs (target/debug/deps/lazy_pull_continuations-4b49946100b6004b)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.87s
     Running tests/loop_continuations.rs (target/debug/deps/loop_continuations-d0d938f953da2f0b)
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
     Running tests/option_reader_continuations.rs (target/debug/deps/option_reader_continuations-5a0cec5f31c0a5b0)
test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s
     Running tests/rule_pattern_continuations.rs (target/debug/deps/rule_pattern_continuations-c61abf30d2dd2405)
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
     Running tests/schema_continuations.rs (target/debug/deps/schema_continuations-65e2f27929ed24fb)
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s
     Running tests/sow_trace_continuations.rs (target/debug/deps/sow_trace_continuations-93367493ef68b082)
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
     Running tests/suspension_oracle.rs (target/debug/deps/suspension_oracle-1756073e73a214f6)
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 27.68s
     Running tests/typed_slot_census.rs (target/debug/deps/typed_slot_census-acdff08e82b67d6e)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.73s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 18.88s
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 132.87s
     Running tests/typed_slot_compile_controls.rs (target/debug/deps/typed_slot_compile_controls-a27c323074551308)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.72s
     Running unittests src/lib.rs (target/debug/deps/rum_syntax-e24f301251f42f42)
test result: ok. 193 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s
     Running unittests src/lib.rs (target/debug/deps/rum_terminal-bae91ae0a1529dd6)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_core
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_derive
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
   Doc-tests rum_host
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.74s
   Doc-tests rum_jupyter
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_repl
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_runtime
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.45s
   Doc-tests rum_stdlib
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_syntax
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   Doc-tests rum_terminal
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```


Native ZeroMQ: **9 executed skips**, explicitly allowed by RUM_ALLOW_NATIVE_ZEROMQ_SKIP=1 when loopback binding was denied. Rust counts these early returns as ok; they are reported here as skips, not native transport passes.


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


The unchanged compiler-oracle gates also reported:

```text
ORACLE_TOTAL agreements=8 disagreements=0 techniques=3/3
A1_CORPUS_TOTAL agreements=7308 disagreements=0 claims=7308
METHOD_TOTAL agreements=3500 disagreements=0 coverage=3500/3500
METHOD_TOTAL agreements=3500 disagreements=0 coverage=3500/3500
```

Final exact ownership inventory:


```text
EXACT INVENTORY self-reaching types: 59 rows
EXACT INVENTORY UnknownHostPayload: 75 rows
EXACT INVENTORY UnknownConstruct: 7685 rows
```

The six prior module-discovery rows vanished through the layout correction; none was admitted. The derive inventory adds only TypedAdapter. Named frames/finalizers in the combined population, including S33, remain 564/59, with 531 pure abandonment derives and 33 manual stateful implementations.


```text
host_payload_ledger.tsv: 65 -> 75 data rows (header excluded); 0 removed or replaced, 10 added or replacement rows
host_payloads.tsv: 65 -> 75 rows; 0 removed or replaced, 10 added or replacement rows
self_reaching.tsv: 59 -> 59 rows; 1 removed or replaced, 1 added or replacement rows
unknown_attributes.tsv: 1499 -> 1500 rows; 4 removed or replaced, 5 added or replacement rows
unknown_captures.tsv: 2946 -> 2949 rows; 11 removed or replaced, 14 added or replacement rows
unknown_macros.tsv: 3144 -> 3145 rows; 14 removed or replaced, 15 added or replacement rows
unknown_storage.tsv: 91 -> 91 rows; 0 removed or replaced, 0 added or replacement rows
Exact changed rows (including source-coordinate replacements): inventory-row-diff.patch.
```

Every changed baseline-to-final ownership/contract/certificate row, including source-coordinate replacements, is disclosed in [inventory-row-diff.patch](../../../.temp/brief-607/redispatch-5/inventory-row-diff.patch). The host ledger adds only the seven approved PublicInput and three approved Unbounded rows. Actual impl bounds/owned F storage supply computed witnesses, and the phantom-only, private-bound, concrete-Expr and extra-bound falsifiers all pass. All 65 old ledger rows, including 56–58, and unrelated identities/statuses/dispositions remain unchanged. Destruction debt is not discharged or strengthened.

All 68 fingerprint cells were recomputed, preserving the first 18 fields: [contract-fingerprint-diff.tsv](../../../.temp/brief-607/redispatch-5/contract-fingerprint-diff.tsv). The 62 Discharged/six Pending statuses and strengths remain. Both initial earning runs produced zero rejections and failed only final reconciliation against stale C01 cells. Four actual new cells were copied from promoted-hash-true.tsv/promoted-hash-false.tsv, then both final replays passed.


```text
body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=true	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=true	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
```


```text
body	instance	rejections
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::Unarmed;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C01	rum_runtime::runtime::model::session::Session::merged_option_exprs_for_symbol::<rum_runtime::runtime::accounting_mode::counter::AccountingShadow,rum_stdlib::option_consumers::OptionConsumer>;M=rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=rum_stdlib::option_consumers::OptionConsumer;hash=false	0
C02	rum_runtime::runtime::model::session::Session::evaluate_runtime_option_expr;M=runtime dispatch via rule contracts: rum_runtime::runtime::accounting_mode::counter::Unarmed,rum_runtime::runtime::accounting_mode::counter::AccountingShadow;R=none (nongeneric C02);hash=false	0
LeaseProbe538::probe	sealed modes	0
LeaseProbe538::start	sealed modes	0
LeaseProbe538::drive	sealed modes	0
LeaseProbe538::resume	sealed modes	0
PRODUCTION_BODY_ROWS=2 PRODUCTION_INSTANCE_MARKS=3 FIXTURE_MARKS=0 PROBE_BODY_MARKS=4 HELPER_BODY_MARKS=0
```

Old/new earned cells: [certificate-cell-diff.tsv](../../../.temp/brief-607/redispatch-5/certificate-cell-diff.tsv). Two LocalCovered body rows remain: two C01 instances and one nongeneric C02 instance per build, plus four separate probe marks. No helper/caller/consumer was promoted; all four consumer obligations remain unresolved.

The initial post-move ownership replay also passed before shared support extraction:

```text
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1120 filtered out; finished in 67.70s
```

Observed intermediate failures, all resolved without weakening an assertion:


```text
census-after.log
error[E0583]: file not found for module `projection`
error[E0583]: file not found for module `sites`
error[E0432]: unresolved imports `discovery::files`, `discovery::normalize`, `discovery::scan`
error[E0432]: unresolved imports `super::classify`, `super::classify`, `super::receipts`, `super::source`
derive.log
test result: FAILED. 14 passed; 1 failed; 0 ignored; 0 measured; 1145 filtered out; finished in 1.31s
cert-default-earn.log
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=true"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1144 filtered out; finished in 18.04s
cert-no-default-earn.log
called `Result::unwrap()` on an `Err` value: "stale or omitted instance certificate C01 hash=false"
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1138 filtered out; finished in 18.23s
```

### 6. Stop-loss, deviations and remaining scope

Final consecutive no-progress count: 0; no stop-loss triggered. Four split compile errors were resolved by restoring the existing classify/scan namespace. The first derive replay found the shared Echo fixture in tests.rs; moving it to the documented support module made the next replay pass. Each initial certificate failure earned new build-specific marks; copying those actual cells made both replays pass. No unchanged failing gate was retried twice without progress. A scratch report-generation quoting error made no edits and was corrected.

The layout and census split follow amendment 5. Two X receipt prefixes/paths and one diagnostic companion path follow the moved tests. All old split bodies remain identical; only one module-visitor forwarding body was added. No ownership path normalization was added. The required oracle script's usual scratch location and the native loopback skip setting are disclosed above. No S1 gate or decision remains open.

Original baseline: .temp/brief-607/redispatch-1/baseline/. Final checkpoint: .temp/brief-607/PROGRESS.md. All work remains uncommitted. S2 driver activation, later typing/transport flips, S54 preflip proof and recorded destruction debt retain their planned scope.
