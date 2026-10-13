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

- `crates/rum-runtime/src/runtime/debug_tools/test_report.rs`
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
