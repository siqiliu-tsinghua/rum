# brief-636 — One immutable parse per test process: retention source, interfaces, typed discovery

**Shape: implementation.** Worker: kimi. The third of brief-633's three
implementation steps. brief-634 (artifact constructors) and brief-635
(profiler lease) have landed.

## Goal

At HEAD, every retention test and every `typed_slot_census` test parses the
whole workspace with `syn` and holds its own trees. Many tests also run
`Interfaces::new`. That work grows linearly with the libtest thread count:
- `rum-stdlib --lib` at 32 threads reaches the 14G cap (the director's
  calibration is below);
- brief-633 measured per-worker discovery at 4.7 GB for 4 workers and
  9.4 GB for 8.

syn trees are `!Send + !Sync`, so a `static` cannot hold them.

After this brief, each process builds three values once, immutably, and
`Sync`:
- the retention `Source`;
- its production `Interfaces`;
- the test-population source with its index.

The typed census builds its `Discovery` once in the same way.

A mutant is an explicit owned copy. Copy-on-write clones only the map that
changes. Every verdict, receipt, source coordinate and forced-red result
stays the same.

## Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`, and brief-633's director gate
  (the end of `docs/dev/codex/brief-633-test-isolation-design.md`).
- Design evidence under `.temp/brief-633/`:
  - the draft `drafts/brief-shared-parse.md`;
  - the integrated tree `tree-final/` and its diff
    `logs/final/integrated.diff`;
  - the decomposition `inventory/e1-retention-decomp.tsv` and
    `inventory/e1-typed-decomp.tsv`;
  - the options `inventory/e2-options.tsv` and
    `inventory/e2-shared-option.tsv`;
  - the equivalence logs `logs/options/option-i-full-model-equivalence-r05.log`
    and `logs/typed-options/option-i-full-discovery-equivalence-r04.log`.
- Option (i), sharing the source alone, is **not** sufficient. The real
  eight-thread retention suite hit the 14G cap
  (`logs/final/retention-gate-r01.log`). This brief is option (iv).
- HEAD sites:
  - the retention `Source`: `crates/rum-stdlib/src/tests/retention_closure/model.rs:136-148`
    and `source.rs`;
  - `production()`: 27 call sites under `retention_closure/`;
  - `Interfaces::new`: `local/interface.rs`;
  - the typed `discover`: `crates/rum-stdlib/tests/typed_slot_census/discovery.rs`;
  - `Function.signature_ast`: `typed_slot_census/scan.rs:72`.

## Scope

- **MAY modify:** `crates/rum-stdlib/src/tests/retention_closure/**`,
  `crates/rum-stdlib/tests/typed_slot_census.rs` and
  `crates/rum-stdlib/tests/typed_slot_census/**`. Also the standing
  inventories, through their replays (`TEMPLATE.md` Scope).
- **May also update, under one rule:** `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv`.
  - A typed-slot X-row identity (`…#method9:…`, `…#reference57:…`) can move
    because this brief edits the function that contains it. Update that
    row's identity, keeping its X id, stage, population and disposition.
    List every before/after pair in `disclosures`.
  - A moved identity in a function this brief does not edit is a STOP.
- **MUST NOT:**
  - change production code;
  - change a retention census definition, reason or count
    (`census/tests.rs`), the pinned 1039-row frontier, `baseline`'s tuples,
    or the forced-red markers;
  - add a dependency or `unsafe` (that includes `unsafe impl Send/Sync`);
  - keep a `syn` or `proc_macro2` value in a static;
  - **shape code to keep an ordinal.** brief-633's tree-final added a
    behaviour-free type ascription to keep `#reference57`/`#method9`, and
    picked a macro destructuring to keep
    `legacy_macro_stays_opaque_in_all_four_views#reference7`. Write the code
    naturally, and move the identity in the inventory by the rule above.

## Target shape

The shape is fixed. A departure from it is a STOP.

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Shared<T>(Arc<T>);
impl<T> std::ops::Deref for Shared<T> { type Target = T; fn deref(&self) -> &T { &self.0 } }
impl<T: Clone> std::ops::DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut T { Arc::make_mut(&mut self.0) }
}

#[derive(Clone, Default)]
pub(super) struct Source {
    pub functions: Shared<BTreeMap<Id, Vec<Function>>>,
    pub types: Shared<BTreeMap<Id, Vec<Nominal>>>,
    pub impls: Shared<Vec<(Id, String, Site, bool)>>,
    pub traits: Shared<BTreeMap<Id, Vec<Trait>>>,
    pub trait_impls: Shared<Vec<(Id, String, Site, bool)>>,
    pub constants: Shared<Vec<(Id, String, String)>>,
    pub files: Shared<BTreeMap<String, String>>,
    // every other heavy map or vector of Source uses Shared<_> as well
}

#[derive(Clone)]
pub(super) struct Function {
    pub sig: String,   // exact source slice, leading newlines/indentation kept
    pub body: String,  // so a local re-parse reproduces original line:column
    pub attrs: Vec<String>,
    pub impl_self: Option<String>,
    pub impl_generics: String,
    // identity, coordinates, substitutions and conditions: owned Sync data
}
impl Function {
    pub fn sig(&self) -> syn::Signature { /* parse self.sig locally */ }
    pub fn body(&self) -> syn::Block { /* parse self.body locally */ }
}

pub(super) fn production() -> &'static Source {
    static PRODUCTION: OnceLock<Source> = OnceLock::new();
    PRODUCTION.get_or_init(fresh_production)
}

impl Interfaces {
    pub fn shared_production() -> &'static Self {
        static PRODUCTION: OnceLock<Interfaces> = OnceLock::new();
        PRODUCTION.get_or_init(|| Self::new(source::production()))
    }
}

pub(super) struct SharedPopulation { pub source: Source, pub index: Interfaces }
pub(super) fn shared_population() -> &'static SharedPopulation { /* OnceLock, built once */ }

// typed_slot_census
type Discovery = (BTreeMap<String, String>, Facts, Report);
pub(super) fn production() -> &'static Discovery {
    static PRODUCTION: OnceLock<Discovery> = OnceLock::new();
    PRODUCTION.get_or_init(|| discover(&root()))
}
```

### Exact text and coordinates

These are calibrated in tree-final; keep them.

- **Slices.** Project each item from its file using span line and column.
  Do not use `byte_range()`: a nested group's range can be relative to its
  enclosing stream (tree-final saw `fn choose` at line 133 with range
  `160..184`).
- **Re-parsing.** Re-parse every stored string locally before any
  `tokens(...)` comparison. Never call `ToTokens` on a `String`: that makes
  a string-literal token.
- **Generics.** Clear generics by blanking their span in place. Add generics
  by inserting text right after the identifier.

### Sharing

- `Interfaces::new` projects its signature-only `Source` directly:
  - bodies are `{ }`;
  - `files` and `constants` start empty;
  - each function's `Self::syntax(f)` is computed once and reused for its
    fingerprint.
- Every fixture, every mutant and every changed interface still calls
  `Interfaces::new` on its own owned input.
- The typed census's `forced_red` reads `production()` and writes its scratch
  copy from that value's files. brief-634's `PrivateScratch` stays as it is.

## Must-not-change beside the cost bars

- **Equivalence, proved at stage 2 on complete dumps, not counts.**
  - Write two probes in `.temp/brief-636/probe/`, not landed. They build the
    HEAD-shaped fresh syn model and the shared model of the same tree.
  - Each probe dumps every function identity, census key and verdict, one
    per line, sorted, and the two dumps must be byte-identical (`cmp`).
  - Retention must give 7,660 functions and 50,584 keys, `Ok`. Typed must
    give 16,129 functions, 321,701 uses and 647 frames, `Ok`.
  - Save both dumps.
- **Receipts.** Every retention and typed-slot receipt is unchanged, except
  X-row identities moved under the Scope rule.
  - `source_baseline_matches_earned_receipt` prints the same baseline and
    `G15-C1` lines.
  - The legacy macro stays opaque in all four views.
- **Copy-on-write is a landed contract test.**
  - Clone `production()`, mutate one function's body in the clone, and
    assert three things:
    - the shared value's `functions` still points at the same `Arc` as
      before;
    - its text is unchanged;
    - the clone differs.
  - `&'static Source` cannot be mutated: the compiler enforces that.
- **Memory**, in each run's own 14G scope. This is a bound, never permission
  to change a receipt.
  - `cargo test -p rum-stdlib --lib --offline -- --test-threads=32` passes
    with `oom_kill=0`. Record its peak.
  - The typed target at 32 threads passes. Record its peak.
- **No measurement module** (`decomp`, `options_probe`) lands.

## Falsifiers

Each runs in its own `scripts/memory-scope.sh b636-<label>-<n> 14G` scope,
with `--offline` and `CARGO_BUILD_JOBS=8`. Copy HEAD to `.temp/brief-636/head/`
before your first edit, as brief-634 did.

1. **rum-stdlib lib at 32 threads.**
   - HEAD is killed at the 14G cap (the director's log is recorded below).
   - The change passes, with no cap kill.
   - Run it three times on the change. Bound: the director's HEAD
     measurement, recorded below.
2. **The retention suite at 8 threads and the typed target at 32 threads**:
   green on the change.
3. **Equivalence dumps**: byte-identical, as above.
4. **The copy-on-write contract test**: green on the change. It is red with
   `Shared::deref_mut` replaced by an in-place `Arc::get_mut().unwrap()` on
   a clone that is still shared: that panics. Save both logs.

## Stages and stops

1. **Shape.**
   - Build the shared `Source`, `Interfaces` and population, and the typed
     `Discovery`.
   - Then run `cargo fmt --check` and `cargo build --tests -p rum-stdlib
     --offline`, with no new warning.
2. **Equivalence.** Make the dumps and `cmp` them. STOP on any difference.
3. **Suites and memory.** Run falsifiers 1, 2 and 4.
4. **Replay and workspace.**
   - Replay the standing inventories, X-row moves included.
   - Then run `cargo test --workspace --no-fail-fast --offline --
     --test-threads=1`, bound 5400 s, in one 14G scope. The nine native
     ZeroMQ EPERM tests are skipped by name, as brief-634 lists them.

## Forbidden

- syn values in statics.
- Locks around discovery.
- Shared mutable overlays.
- Interior mutability in `Source`.
- Normalized-token substitutions that move spans.
- Ordinal-preserving code.
- Editing a receipt other than a moved X-row identity in an edited function.
- `#[ignore]` as a fix, retries, sleeps, raised or guessed bounds,
  `--test-threads=1` as a fix, merged tests, `serial_test` or any dependency.

## Report

- Write `.temp/brief-636/report.json`. Quote every `test result:` line, every
  verdict line, and both `cmp` results.
- Your final message is `python3 scripts/report.py summary 636`.

## Director calibration before dispatch (2026-10-31, HEAD `f38657d0`)

One run, serial, in its own 14G scope; the log is
`.temp/brief-636/calib-head.log`.

- `cargo test -p rum-stdlib --lib --offline -- --test-threads=32`, bounded
  at 3600 s: the test binary was killed at the cap.
  - `process didn't exit successfully: .../rum_stdlib-... --test-threads=32
    (signal: 9, SIGKILL: kill)`.
  - `# memory-scope: unit=b636-calib-head max=15032385536
    peak=15032385536 oom=13 oom_kill=1 rc=101`.
  - So HEAD reaches the 14G cap at 32 threads; falsifier 1's premise holds,
    and the worker's runs use this same scope, cap and bound.
## report (kimi, DONE)

Implemented one immutable Sync retention Source, shared production Interfaces/test population, and shared typed Discovery per process. Exact source slices are reparsed locally; mutants use Arc::make_mut. Complete fresh/shared dumps match byte-for-byte (7,660/50,584 and 16,129/321,701/647, all Ok). Three 32-thread lib runs passed with oom_kill=0 (peaks 15.03, 8.43, 9.46 GB); retention 8-thread and typed 32-thread passed. CoW green/red/restored verified. Replayed 68 fingerprints, four certificate cells and two permitted X identities. Workspace: 59 binaries, 4015 passed, 0 failed, 61 ignored, nine standing skips, peak 9.11 GB.

| File | Kind | Lines | Why |
| --- | --- | --- | --- |
| `crates/rum-stdlib/src/tests/retention_closure/catalog.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/corpus.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/exact.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/methods.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/compiler_oracle/sample.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/consumers.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/effects.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/effects/call.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/local-contracts.tsv` | modified | - | Replay 68 generated contract fingerprint cells; every other column unchanged. |
| `crates/rum-stdlib/src/tests/retention_closure/inventory/promoted.tsv` | modified | - | Merge four generated C01/C02 certificate cells from both feature builds. |
| `crates/rum-stdlib/src/tests/retention_closure/leaves.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/admission_tests.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/calls.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/certificates.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/contract_tests.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/flow/mod.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/interface.rs` | modified | - | Share production Interfaces and the test-population source/index once per process with selective signature projection. |
| `crates/rum-stdlib/src/tests/retention_closure/local/mod.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/option_contract_tests.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/option_domain.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/oracle/mod.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/production.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/promotion.rs` | modified | - | Share production Interfaces and the test-population source/index once per process with selective signature projection. |
| `crates/rum-stdlib/src/tests/retention_closure/local/promotion_tests.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/protocol.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/reads/contracts.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/reads/oracle.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/reads/tests.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/resume_macro.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/rule_tests.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/rules.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/tests.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/local/type_tests.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/method_tests.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/methods/context.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/methods/mod.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/methods/traits.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/mod.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/model.rs` | modified | - | Store exact source slices in Sync owned data, reparse locally, and wrap heavy Source collections in Shared<T>. |
| `crates/rum-stdlib/src/tests/retention_closure/mutations.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/regions.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/resolve.rs` | modified | - | Store exact source slices in Sync owned data, reparse locally, and wrap heavy Source collections in Shared<T>. |
| `crates/rum-stdlib/src/tests/retention_closure/source.rs` | modified | - | Store exact source slices in Sync owned data, reparse locally, and wrap heavy Source collections in Shared<T>. |
| `crates/rum-stdlib/src/tests/retention_closure/typed_resume_macro.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/types.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/types/context.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/types/exact.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/types/modes.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/src/tests/retention_closure/types/modes/projections.rs` | modified | - | Adapt retention analysis and mutants to the string-backed shared Source with explicit owned copies. |
| `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv` | modified | - | Move only the two permitted X-row identities in edited functions. |
| `crates/rum-stdlib/tests/typed_slot_census.rs` | modified | - | Build typed discovery once per process and store function signatures/bodies as exact strings for local reparsing. |
| `crates/rum-stdlib/tests/typed_slot_census/discovery.rs` | modified | - | Build typed discovery once per process and store function signatures/bodies as exact strings for local reparsing. |
| `crates/rum-stdlib/tests/typed_slot_census/scan.rs` | modified | - | Build typed discovery once per process and store function signatures/bodies as exact strings for local reparsing. |
| `crates/rum-stdlib/tests/typed_slot_census/scan/sites.rs` | modified | - | Build typed discovery once per process and store function signatures/bodies as exact strings for local reparsing. |
| `crates/rum-stdlib/tests/typed_slot_census/witnessed.rs` | modified | - | Build typed discovery once per process and store function signatures/bodies as exact strings for local reparsing. |

- Disclosed: `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv`: X0035 permitted identity move in an edited function: BEFORE rum_stdlib::tests::retention_closure::local::rules::tests::local_rule_macro_definition_arms_hygiene_and_raw_frame_controls#reference57:resume_or_suspend: AFTER rum_stdlib::tests::retention_closure::local::rules::tests::local_rule_macro_definition_arms_hygiene_and_raw_frame_controls#reference56:resume_or_suspend:; stage=R; population=test-support; disposition=-
- Disclosed: `crates/rum-stdlib/tests/suspension_oracle/inventory/typed-slot-stages.tsv`: X0041 permitted identity move in an edited function: BEFORE rum_stdlib::tests::retention_closure::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views#reference7:resume_or_suspend: AFTER rum_stdlib::tests::retention_closure::compiler_oracle::corpus::legacy_macro_stays_opaque_in_all_four_views#reference8:resume_or_suspend:; stage=R; population=test-support; disposition=S3 A3 AST-only legacy-opacity control; macro argument spans preserved

| Run | Try | Exit | Tests | Log |
| --- | ---: | ---: | --- | --- |
| fmt-check | 1 | 0 | - | `.temp/brief-636/logs/fmt-check.log` |
| build-tests-1 | 1 | 0 | - | `.temp/brief-636/logs/build-tests.log` |
| build-tests-2 | 2 | 101 | - | `.temp/brief-636/logs/build-tests-r02.log` |
| build-tests-3 | 3 | 0 | - | `.temp/brief-636/logs/build-tests-r03.log` |
| equivalence-retention-1 | 1 | 101 | 0/1/0 | `.temp/brief-636/logs/equivalence-retention.log` |
| equivalence-retention-2 | 2 | 0 | 1/0/0 | `.temp/brief-636/logs/equivalence-retention-r02.log` |
| cmp-retention | 1 | 0 | - | `.temp/brief-636/logs/cmp-retention.log` |
| equivalence-typed | 1 | 0 | 1/0/0 | `.temp/brief-636/logs/equivalence-typed.log` |
| cmp-typed | 1 | 0 | - | `.temp/brief-636/logs/cmp-typed.log` |
| lib32-pre-replay | 1 | 101 | 1094/17/53 | `.temp/brief-636/logs/lib32-r01.log` |
| fingerprints-generate | 1 | 0 | 1/0/0 | `.temp/brief-636/logs/fingerprints-generate.log` |
| fingerprints-replay | 1 | 0 | - | `.temp/brief-636/logs/fingerprints-replay.log` |
| certificate-default-earn | 1 | 101 | 0/1/0 | `.temp/brief-636/logs/certificate-default-earn.log` |
| certificate-nodefault-earn | 1 | 101 | 0/1/0 | `.temp/brief-636/logs/certificate-nodefault-earn.log` |
| certificate-replay | 1 | 0 | - | `.temp/brief-636/logs/certificate-replay.log` |
| certificate-default-replay | 2 | 0 | 1/0/0 | `.temp/brief-636/logs/certificate-default-replay.log` |
| certificate-nodefault-replay | 2 | 0 | 1/0/0 | `.temp/brief-636/logs/certificate-nodefault-replay.log` |
| lib32-green-1 | 2 | 0 | 1111/0/53 | `.temp/brief-636/logs/lib32-r02.log` |
| lib32-green-2 | 3 | 0 | 1111/0/53 | `.temp/brief-636/logs/lib32-r03.log` |
| lib32-green-3 | 4 | 0 | 1111/0/53 | `.temp/brief-636/logs/lib32-r04.log` |
| retention8 | 1 | 0 | 59/0/1 | `.temp/brief-636/logs/retention8.log` |
| typed32-pre-replay | 1 | 101 | 2/6/1 | `.temp/brief-636/logs/typed32.log` |
| typed32-green | 2 | 0 | 8/0/1 | `.temp/brief-636/logs/typed32-r02.log` |
| cow-green | 1 | 0 | 1/0/0 | `.temp/brief-636/logs/cow-green.log` |
| cow-red-get-mut | 1 | 101 | 0/1/0 | `.temp/brief-636/logs/cow-red.log` |
| cow-restored | 2 | 0 | 1/0/0 | `.temp/brief-636/logs/cow-restored.log` |
| ownership | 1 | 0 | 3/0/0 | `.temp/brief-636/logs/ownership.log` |
| lp01 | 1 | 0 | 6/0/0 | `.temp/brief-636/logs/lp01.log` |
| frame-root | 1 | 0 | 15/0/0 | `.temp/brief-636/logs/frame-root.log` |
| workspace | 1 | 0 | 4015/0/61 | `.temp/brief-636/logs/workspace.log` |
| typed-receipt | 1 | 0 | 1/0/0 | `.temp/brief-636/logs/typed-receipt.log` |
| legacy-macro | 1 | 0 | 1/0/0 | `.temp/brief-636/logs/legacy-macro.log` |
| shape-check | 2 | 0 | - | `.temp/brief-636/logs/shape-check.log` |

| Check | Bar | Result | Verdict | Note |
| --- | --- | --- | --- | --- |
| format-build | cargo fmt --check and cargo build --tests -p rum-stdlib --offline pass with no new warning | both green; final build has zero warnings | PASS | The first draft build had two dead-code warnings and the second had a pointer assertion type error; both were corrected before the final build. |
| equivalence-retention | complete fresh/shared function identities, census keys, and verdict are byte-identical | 7,660 functions; 50,584 keys; Ok; cmp=0; 58,245 lines in each dump | PASS | The first attempt used the newer production input and failed only its calibrated count assertion; the required calibrated HEAD-shaped input produced matching complete dumps. |
| equivalence-typed | complete fresh/shared typed function identities, uses, frames, and verdict are byte-identical | 16,129 functions; 321,701 uses; 647 frames; Ok; cmp=0; 339,865 lines in each dump | PASS |  |
| lib32-three | three rum-stdlib lib runs at 32 threads pass in separate 14G scopes with oom_kill=0 | 3/3 green, 1111 passed each; peaks 15032385536, 8425742336, 9461608448; all oom_kill=0 | PASS | The pre-replay attempt failed only stale generated fingerprints/certificates; it was not counted among the three green runs. |
| retention8 | full retention suite at eight threads is green with no cap kill | 59 passed, 1 ignored; peak 6020395008; oom_kill=0 | PASS |  |
| typed32 | typed target at 32 threads is green after only permitted X-row identity replay | 8 passed, 1 ignored; peak 3560525824; oom_kill=0 | PASS | The initial run exposed exactly reference7→reference8 and reference57→reference56 in the two edited functions; no other identity moved. |
| cow-contract | Arc::make_mut clone is green and in-place Arc::get_mut unwrap on a shared clone is red | green; expected None unwrap panic; restored green | PASS |  |
| receipts | typed baseline, G15-C1 lines, and legacy macro four-view opacity remain intact | all receipt tests green and markers present | PASS |  |
| inventory-replay | standing inventories replay only generated cells and permitted X identities | 68 fingerprint cells; four certificate cells; two X identities; ownership/LP01/frame-root green | PASS |  |
| scope | only allowed files change; pinned census definitions/baseline and forced-red behavior stay intact; no measurement module or unsafe lands | 55 allowed files; two X moves; zero unexpected; pinned census files byte-identical | PASS |  |
| workspace | single serial workspace in one 14G scope with exactly nine standing native ZeroMQ skips | 59 binaries; 4015 passed; 0 failed; 61 ignored; peak 9111609344; oom_kill=0 | PASS |  |

| Measurement | Bar | Result | Verdict | Note |
| --- | --- | --- | --- | --- |
| lib32-memory | three changed runs stay within the director's 14G/15032385536-byte HEAD bound with no OOM kill | peaks 15032385536, 8425742336, 9461608448 bytes; oom=0 and oom_kill=0 in all three | PASS |  |
| retention8-memory | retention suite at eight threads stays in its 14G scope | peak 6020395008 bytes; oom=0; oom_kill=0 | PASS |  |
| typed32-memory | typed target at 32 threads stays in its 14G scope | peak 3560525824 bytes; oom=0; oom_kill=0 | PASS |  |
| workspace-memory | serial workspace stays in one 14G scope | peak 9111609344 bytes; oom=0; oom_kill=0 | PASS |  |

### Shared immutable discovery

`Source` stores exact line/column slices as owned strings and every heavy map/vector in `Shared<T>`. `Function`, `Trait`, `Nominal`, and `Field` reparse only locally. `production()`, `Interfaces::shared_production()`, `shared_population()`, and typed `production()` use `OnceLock`; no AST node is static and no discovery lock or mutable overlay exists. Signature-only interface projection starts with empty files/constants and `{ }` bodies, and reuses each function syntax fingerprint. Mutants clone the handle and clone only the changed map through `Arc::make_mut`.

### Complete dump comparison

The two probes in `.temp/brief-636/probe/` build the HEAD-shaped fresh AST model and string-backed model of the same calibrated input tree. Retention dumps each contain 58,245 sorted lines: 7,660 function identities, 50,584 keys, and `Ok(())`. Typed dumps each contain 339,865 sorted lines covering 16,129 functions, 321,701 uses, 647 frames, and `Ok(())`. `cmp` returned 0 for both pairs. The first retention attempt used the newer production tree and failed only its expected-count assertion (7,670/50,625); it was retained in the run log and the calibrated input was used for the required comparison.

### Generated replay and X-row moves

Only the final fingerprint cell changed in each of 68 contract rows; only the hash=true/hash=false certificate cells changed for C01/C02. Ownership, LP01, and frame-root inventories required no row edits. The two typed X moves were exactly `legacy_macro_stays_opaque_in_all_four_views#reference7` → `#reference8` (X0041) and `local_rule_macro_definition_arms_hygiene_and_raw_frame_controls#reference57` → `#reference56` (X0035). X ids, stages, populations, and dispositions are unchanged. No ordinal-preserving type ascription or macro destructuring remains.

### Memory and copy-on-write

Three post-replay lib runs at 32 threads each passed 1111 tests with `oom_kill=0`; their peaks were 15032385536, 8425742336, and 9461608448 bytes. Retention at eight threads passed 59 tests with one ignored; typed at 32 passed eight with one ignored. The copy-on-write contract passed, then the temporary `Arc::get_mut().unwrap()` mutation panicked on `None` as required, and the restored `Arc::make_mut` implementation passed again. The pre-replay lib failure was solely stale generated fingerprints/certificates; the first typed failure was solely the two permitted X moves.

## Director gate (2026-10-31): brief-636 committed; the shared parse is done

One immutable parse per test process: the retention `Source`, its
production `Interfaces`, the test population and the typed `Discovery` are
`OnceLock` values; every stored AST is an exact source slice re-parsed
locally. The 32-thread `rum-stdlib` lib, SIGKILLed at the 14G cap at HEAD,
passes three times; retention at eight threads and the typed target at 32
pass; the complete dumps are byte-identical; the copy-on-write contract is
green and red under its mutant.

- **Code.** I read the whole diff (55 files, 831 insertions, 451
  deletions). It is the brief's shape:
  - `Shared<T>(Arc<T>)` with `Deref`/`DerefMut` through `Arc::make_mut`,
    `FromIterator` and the three `IntoIterator` impls; every heavy
    `Source` field is `Shared<_>`; `Function`, `Trait`, `Nominal` and
    `Field` store `String` slices with local `sig()`/`body()`/`attrs()`
    re-parses.
  - `source::production()` and `Interfaces::shared_production()` are
    `OnceLock`s; `promotion::shared_population()` builds the population
    once; the typed `production()` wraps `discover(&root())`.
  - `located()` projects each item from its span start and end (never
    `byte_range()`), prepending newlines and the start column so a local
    re-parse reproduces line:column; `rewrite_generics` blanks the span in
    place and inserts additions after the identifier.
  - `Interfaces::new` starts from an empty `Source` (empty files and
    constants), gives every function a `{ }` body, and reuses one syntax
    fingerprint per function.
  - No production code, no dependency, no `unsafe`, no static `syn`, no
    lock or mutable overlay. `census/tests.rs`, the 1039-row frontier,
    `receipts::baseline`'s tuples and the forced-red markers are untouched.
  - The two X-row moves are exactly the disclosures: X0035
    `#reference57`→`#reference56` and X0041 `#reference7`→`#reference8`,
    both in functions this brief edits, with id, stage, population and
    disposition unchanged. No ordinal-preserving ascription or macro
    destructuring remains.
  - Shape greps: no new `unreachable!`, `expect`, `unsafe`, `Mutex`,
    `RwLock`, `RefCell`, `OnceCell` or static `syn`; no touched file over
    20k lines. The only duplicated body is `located()` (and its offset
    helper), which the lib unit-test target and the typed integration-test
    target cannot share.
- **kimi's report** (`.temp/brief-636/report.json`, DONE, 0 deviations, 2
  disclosures): `report.py check --against-git` — 4,367 citations
  verified; 55 changes, all in scope.
- **The gate** (`python3 -I scripts/director-gate.py run 636`,
  `.temp/director/gate-636/`, 2026-10-31 22:27–22:50): GREEN, every step
  PASS.
  - scope: 4,367 citations; fmt formatted; check 3 warnings, 0 new.
  - ownership: 61/79/7974, 0 replay rounds. oracle: 1 passed.
  - census: 1039 {RET001 10, RET002 996, RET003 33}; paired 1039, retired
    0, new 0 — identical to base.
  - standing: lp01 1/0; typed-slot 8/0. inventories: 19 passed, 0 failed;
    frontier 1039.
  - fingerprints: 68 contracts, **0 changed, copied in** — the worker's
    replayed cells are exactly the gate's earned cells.
  - certificates: both builds pass; 0 promoted rows merged, 1 round.
  - workspace: 59 binaries, 4024 passed, 0 failed, 61 ignored; peak
    4.88 GB of 14, oom_kill 0. (The worker's 4015 is this minus the nine
    native ZeroMQ tests it skips by name.)
- **My reproductions** (`.temp/director/gate-636-repro/`, each in its own
  14G scope, serial):
  - `rum-stdlib --lib` at 32 threads: `test result: ok. 1111 passed; 0
    failed; 53 ignored` in 131.29 s; peak 8,989,696,000 B, oom_kill 0.
  - retention at 8 threads: `ok. 59 passed; 0 failed; 1 ignored`; peak
    6,027,255,808 B.
  - typed at 32 threads: `ok. 8 passed; 0 failed; 1 ignored`; peak
    3,288,215,552 B.
  - both probes reran (`ok. 1 passed` each) and the dumps stayed
    byte-identical: retention 58,245 lines, sha256 `17fbe39f…1db`; typed
    339,865 lines, sha256 `2c3c255d…b79`. Both pairs also match the hashes
    of the worker's dumps, so the probe is deterministic.
- **The first 32-thread run touched the cap.** The worker's first green
  run peaked at exactly 15,032,385,536 B (the cgroup `memory.max`) with
  oom=0/oom_kill=0; the other two peaked at 8,425,742,336 and
  9,461,608,448 B, and my rerun at 8,989,696,000 B. The cap touch is page
  cache charged to the cold scope, not anonymous memory: no OOM event, the
  same binary, and the warm scopes sit at 8.4–9.0 GB. The gate's serial
  workspace peaked at 4.88 GB. The falsifier's bar is the HEAD SIGKILL at
  the cap; the change passes it three times plus my rerun.
- **The typed pre-replay failure was the X moves.** Its first run was
  `FAILED. 2 passed; 6 failed`, with `X receipt mismatch: expected 48,
  discovered 48`, listing `…legacy_macro…#reference7` and
  `…local_rule_macro…#reference57` as missing and `#reference8` /
  `#reference56` as unrecorded. The worker replayed the two rows and the
  same suite is green. That is the second instance of "suite before
  replay" in this brief (the first is the retention
  fingerprint/certificate run); the ordering fix queued for the next
  commit covers the class.
- **Equivalence is on the brief's calibrated tree.** The probes compare
  the HEAD-shaped model against the string round-trip on the calibrated
  roots, so the counts are the brief's 7,660/50,584 and
  16,129/321,701/647. The worker's first retention attempt on the newer
  tree recorded 7,670/50,625 and aborted at the count assertion after
  writing the dumps; the second run overwrote them at 21:16, so the newer
  tree's dump comparison is not on record. The report discloses this. The
  current tree's extra functions are covered by the 1,111-test lib run,
  not by the dump comparison.
- **Not rerun:** the CoW red mutant. Its panic is `model.rs:285:46`,
  `Option::unwrap()` on `None` — the mutant's own `Arc::get_mut(...)` line
  in `deref_mut` (the restored line 285 is `Arc::make_mut`); the
  green/red/restored logs are on disk, and I read the mutation and its
  trace directly.
- **Disposition.** Brief-636 is complete; test-isolation item 1 is done
  (briefs 634, 635, 636). Next: the workspace thread-count measurement
  (4/8/16/32) for the user's default-thread-count ruling, then LP03.
